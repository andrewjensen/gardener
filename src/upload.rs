use actix_multipart::{Field, Multipart};
use actix_web::web;
use futures_util::StreamExt as _;
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::fs;
use std::io::prelude::*;
use std::str::FromStr;
use uuid::Uuid;

use crate::boards::Board;
use crate::patches::PatchMeta;

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

pub async fn process_patch_upload(mut payload: Multipart) -> Option<PatchMeta> {
    // TODO: this should return a Result instead

    let mut board_in: Option<Board> = None;
    let mut filename_in: Option<String> = None;
    let mut file_contents_in: Option<String> = None;

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        debug!("Item is a field: {:?}", field);

        let mut field_contents: String = "".to_string();

        while let Some(chunk) = field.next().await {
            let chunk_bytes = chunk.unwrap();
            let chunk_contents = std::str::from_utf8(&chunk_bytes).unwrap();
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
                unreachable!();
            }
        }
    }

    // TODO: handle cases where parts are missing
    let patch_id = Uuid::new_v4();
    let board = board_in.unwrap();
    let filename = filename_in.unwrap();
    let file_contents = file_contents_in.unwrap();

    debug!("Board result: {:?}", board);
    debug!("Filename: {:?}", filename);
    debug!("File contents: {:?}", file_contents);

    let patch_meta = PatchMeta {
        id: patch_id.to_string(),
        board,
        filename,
        file_contents,
    };
    debug!("Created patch meta: {:?}", &patch_meta);

    Some(patch_meta)
}

pub async fn write_patch_to_disk(patch_id: &str, file_contents: &str) {
    // TODO: handle fs errors!

    let patch_id = patch_id.to_string();
    let file_contents = file_contents.to_string();

    web::block(move || {
        fs::create_dir(format!("workspace/{patch_id}")).unwrap();
        let mut file = fs::File::create(format!("workspace/{patch_id}/patch.pd")).unwrap();
        file.write_all(file_contents.as_bytes()).unwrap();
    })
    .await
    .unwrap();
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
            debug!("Parsed a file upload: {}", filename);
            return UploadFormItem::FileUpload {
                filename: filename.to_string(),
                file_contents: chunk_contents.to_string(),
            };
        }
    }

    UploadFormItem::Unrecognized
}
