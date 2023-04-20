use actix_multipart::{Field, Multipart};
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::web;
use anyhow::{anyhow, Result};
use futures_util::StreamExt as _;
use lazy_static::lazy_static;
use log::{debug, trace};
use regex::Regex;
use std::fs;
use std::io::prelude::*;
use std::str::FromStr;
use uuid::Uuid;

use crate::boards::Board;
use crate::patches::{validate_patch_file_contents, DateTime, PatchMeta, PatchStatus};

lazy_static! {
    static ref REGEX_FILENAME: Regex = Regex::new(r#"filename="(.*?)""#).unwrap();
}

enum UploadFormItem {
    BoardOption(Board),
    BoardDefinitionUpload {
        filename: String,
        file_contents: String,
    },
    PatchFileUpload {
        filename: String,
        file_contents: String,
    },
    Unrecognized,
}

pub async fn process_patch_upload(mut payload: Multipart) -> Result<PatchMeta> {
    let mut board_in: Option<Board> = None;

    let mut board_def_filename_in: Option<String> = None;
    let mut board_def_contents_in: Option<String> = None;

    let mut patch_filename_in: Option<String> = None;
    let mut patch_contents_in: Option<String> = None;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        debug!("Item is a field: {:?}", field);

        let mut field_contents: String = "".to_string();

        while let Some(chunk) = field.next().await {
            let chunk_bytes = chunk?;
            let chunk_contents = std::str::from_utf8(&chunk_bytes)?;
            debug!("chunk contents:\n{}", chunk_contents);

            field_contents.push_str(chunk_contents);
        }

        match parse_upload_form_item(&field, &field_contents) {
            UploadFormItem::BoardOption(board_value) => board_in = Some(board_value),
            UploadFormItem::BoardDefinitionUpload {
                filename: found_filename,
                file_contents: found_contents,
            } => {
                board_def_filename_in = Some(found_filename);
                board_def_contents_in = Some(found_contents);
            }
            UploadFormItem::PatchFileUpload {
                filename: found_filename,
                file_contents: found_contents,
            } => {
                patch_filename_in = Some(found_filename);
                patch_contents_in = Some(found_contents);
            }
            UploadFormItem::Unrecognized => {
                // Ignore missing files or unexpected items
            }
        }
    }

    if board_in.is_none() {
        return Err(anyhow!("Missing board option"));
    }

    if patch_contents_in.is_none() {
        return Err(anyhow!("Missing patch file"));
    }

    if patch_filename_in.is_none() {
        return Err(anyhow!("Missing filename"));
    }

    if let Some(Board::SeedCustomJson) = board_in {
        if board_def_filename_in.is_none() {
            return Err(anyhow!("Missing custom board definition filename"));
        }

        if board_def_contents_in.is_none() {
            return Err(anyhow!("Missing custom board definition file"));
        }
    }

    let board = board_in.unwrap();
    let filename = patch_filename_in.unwrap();
    let patch_contents = patch_contents_in.unwrap();

    trace!("Board result: {:?}", board);
    trace!("Filename: {:?}", filename);
    trace!("File contents: {:?}", patch_contents);
    trace!("Board definition: {:?}", board_def_contents_in);

    if !filename.ends_with(".pd") {
        return Err(anyhow!("File does not appear to be a Pd patch"));
    }

    validate_patch_file_contents(&patch_contents)?;

    let patch_id = Uuid::new_v4();

    let patch_meta = PatchMeta {
        id: patch_id.to_string(),
        status: PatchStatus::Uploaded,
        board,
        filename,
        time_upload: DateTime::now(),
        time_compile_start: None,
        time_compile_end: None,
    };
    debug!("Created patch meta: {:?}", &patch_meta);

    write_patch_to_disk(&patch_meta.id, &patch_contents).await?;

    if let Some(board_def_contents) = board_def_contents_in {
        write_board_def_to_disk(&patch_meta.id, &board_def_contents).await?;
    }

    Ok(patch_meta)
}

async fn write_patch_to_disk(patch_id: &str, file_contents: &str) -> Result<()> {
    let patch_id = patch_id.to_string();
    let file_contents = file_contents.to_string();

    let result = web::block(move || {
        let mut file = fs::File::create(format!("workspace/uploads/{patch_id}.pd")).unwrap();

        file.write_all(file_contents.as_bytes())
    })
    .await?;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!("Failed to save patch to disk")),
    }
}

async fn write_board_def_to_disk(patch_id: &str, file_contents: &str) -> Result<()> {
    let patch_id = patch_id.to_string();
    let file_contents = file_contents.to_string();

    let result = web::block(move || {
        let mut file =
            fs::File::create(format!("workspace/uploads/{patch_id}_board_def.json")).unwrap();

        file.write_all(file_contents.as_bytes())
    })
    .await?;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!("Failed to save board definition to disk")),
    }
}

fn parse_upload_form_item(multipart_field: &Field, chunk_contents: &str) -> UploadFormItem {
    let content_disposition = multipart_field.content_disposition();

    match (&content_disposition.disposition, multipart_field.name()) {
        (&DispositionType::FormData, "board") => {
            let board_option = Board::from_str(chunk_contents).unwrap();
            debug!("Parsed a board option: {:?}", board_option);
            return UploadFormItem::BoardOption(board_option);
        }
        (&DispositionType::FormData, "pd_patch") => {
            let filename = get_filename(&content_disposition);
            if filename.is_some() && !chunk_contents.is_empty() {
                let filename = filename.unwrap();
                debug!("Parsed a file upload: {}", filename);
                return UploadFormItem::PatchFileUpload {
                    filename: filename.to_string(),
                    file_contents: chunk_contents.to_string(),
                };
            } else {
                return UploadFormItem::Unrecognized;
            }
        }
        (&DispositionType::FormData, "board_def") => {
            let filename = get_filename(&content_disposition);
            if filename.is_some() && !chunk_contents.is_empty() {
                let filename = filename.unwrap();
                debug!("Parsed a board definition: {}", filename);
                return UploadFormItem::BoardDefinitionUpload {
                    filename: filename.to_string(),
                    file_contents: chunk_contents.to_string(),
                };
            } else {
                return UploadFormItem::Unrecognized;
            }
        }
        _ => {
            return UploadFormItem::Unrecognized;
        }
    }
}

fn get_filename(content_disposition: &ContentDisposition) -> Option<String> {
    for param in content_disposition.parameters.iter() {
        if let DispositionParam::Filename(filename) = param {
            return Some(filename.to_string());
        }
    }

    None
}
