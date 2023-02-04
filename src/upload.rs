use actix_files::NamedFile;
use actix_multipart::{Field, Multipart};
use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{post, HttpRequest, HttpResponse, Responder, Result};
use futures_util::StreamExt as _;
use lazy_static::lazy_static;
use log::{debug, info};
use regex::Regex;
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::boards::Board;
use crate::views::get_view_path;

lazy_static! {
    static ref REGEX_FILENAME: Regex = Regex::new(r#"filename="(.*?)""#).unwrap();
}

#[derive(Serialize, Debug, Clone)]
pub struct UploadMeta {
    pub id: String,
    pub board: Board,
    pub filename: String,
    pub file_contents: String,
}

impl Responder for UploadMeta {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

enum UploadFormItem {
    BoardOption(Board),
    FileUpload {
        filename: String,
        file_contents: String,
    },
    Unrecognized,
}

#[post("/upload")]
pub async fn upload_route(mut payload: Multipart) -> Result<NamedFile> {
    let upload_id = Uuid::new_v4();
    info!("Starting the upload endpoint... upload_id = {}", upload_id);

    let mut board: Option<Board> = None;
    let mut filename: Option<String> = None;
    let mut file_contents: Option<String> = None;

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
            UploadFormItem::BoardOption(board_value) => board = Some(board_value),
            UploadFormItem::FileUpload {
                filename: found_filename,
                file_contents: found_contents,
            } => {
                filename = Some(found_filename);
                file_contents = Some(found_contents);
            }
            UploadFormItem::Unrecognized => {
                unreachable!();
            }
        }
    }

    info!("Board result: {:?}", board);
    info!("Filename: {:?}", filename);
    info!("File contents: {:?}", file_contents);

    let view_path = get_view_path("upload_success");

    Ok(NamedFile::open(view_path)?)
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
