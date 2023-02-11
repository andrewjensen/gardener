use actix_multipart::{Field, Multipart};
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
    FileUpload {
        filename: String,
        file_contents: String,
    },
    Unrecognized,
}

pub async fn process_patch_upload(mut payload: Multipart) -> Result<PatchMeta> {
    let mut board_in: Option<Board> = None;
    let mut filename_in: Option<String> = None;
    let mut file_contents_in: Option<String> = None;

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
            UploadFormItem::FileUpload {
                filename: found_filename,
                file_contents: found_contents,
            } => {
                filename_in = Some(found_filename);
                file_contents_in = Some(found_contents);
            }
            UploadFormItem::Unrecognized => {
                // Ignore missing files or unexpected items
            }
        }
    }

    if board_in.is_none() {
        return Err(anyhow!("Missing board option"));
    }

    if file_contents_in.is_none() {
        return Err(anyhow!("Missing patch file"));
    }

    if filename_in.is_none() {
        return Err(anyhow!("Missing filename"));
    }

    let board = board_in.unwrap();
    let filename = filename_in.unwrap();
    let file_contents = file_contents_in.unwrap();

    trace!("Board result: {:?}", board);
    trace!("Filename: {:?}", filename);
    trace!("File contents: {:?}", file_contents);

    if !filename.ends_with(".pd") {
        return Err(anyhow!("File does not appear to be a Pd patch"));
    }

    validate_patch_file_contents(&file_contents)?;

    let patch_id = Uuid::new_v4();

    let patch_meta = PatchMeta {
        id: patch_id.to_string(),
        status: PatchStatus::Uploaded,
        board,
        filename,
        file_contents,
        time_upload: DateTime::now(),
        time_compile_start: None,
        time_compile_end: None,
    };
    debug!("Created patch meta: {:?}", &patch_meta);

    write_patch_to_disk(&patch_meta.id, &patch_meta.file_contents).await?;

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

fn parse_upload_form_item(multipart_field: &Field, chunk_contents: &str) -> UploadFormItem {
    if let Some(content_disposition_header) = multipart_field.headers().get("content-disposition") {
        let content_disposition = content_disposition_header.to_str().unwrap();
        debug!("content-disposition: {}", content_disposition);

        if content_disposition.contains("name=\"board\"") {
            let board_option = Board::from_str(chunk_contents).unwrap();
            debug!("Parsed a board option: {:?}", board_option);
            return UploadFormItem::BoardOption(board_option);
        }

        if content_disposition.contains("name=\"pd_patch\"")
            && content_disposition.contains("filename=")
        {
            let filename_captures = REGEX_FILENAME.captures(content_disposition).unwrap();
            let filename = filename_captures.get(1).unwrap().as_str();

            if !filename.is_empty() && !chunk_contents.is_empty() {
                debug!("Parsed a file upload: {}", filename);
                return UploadFormItem::FileUpload {
                    filename: filename.to_string(),
                    file_contents: chunk_contents.to_string(),
                };
            }
        }
    }

    UploadFormItem::Unrecognized
}
