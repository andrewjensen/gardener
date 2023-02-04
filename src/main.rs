use actix_files::{Files, NamedFile};
use actix_multipart::{Field, Multipart};
use actix_web::middleware::Logger;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
use futures_util::StreamExt as _;
use lazy_static::lazy_static;
use log::{debug, info};
use regex::Regex;
use std::env;
use std::str::FromStr;
use uuid::Uuid;

mod boards;

use boards::Board;

lazy_static! {
    static ref REGEX_FILENAME: Regex = Regex::new(r#"filename="(.*?)""#).unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(Files::new("/static", "./public/static"))
            .service(index)
            .service(build)
            .service(echo)
            .service(liveness_probe)
            .service(readiness_probe)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index() -> Result<NamedFile> {
    let mut index_path = env::current_dir().unwrap();
    index_path.push("public");
    index_path.push("index.html");
    debug!("Path to index file: {:?}", index_path);

    Ok(NamedFile::open(index_path)?)
}

#[post("/build")]
async fn build(mut payload: Multipart) -> Result<HttpResponse> {
    let upload_id = Uuid::new_v4();
    info!("Starting the build endpoint... upload_id = {}", upload_id);

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

    Ok(HttpResponse::Ok().into())
}

enum UploadFormItem {
    BoardOption(Board),
    FileUpload {
        filename: String,
        file_contents: String,
    },
    Unrecognized,
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

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/health/live")]
async fn liveness_probe() -> impl Responder {
    HttpResponse::Ok().body("App is live")
}

#[get("/health/ready")]
async fn readiness_probe() -> impl Responder {
    HttpResponse::Ok().body("App is ready to receive traffic")
}
