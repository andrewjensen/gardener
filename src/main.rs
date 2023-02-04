use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
use std::collections::HashMap;
use std::sync::Mutex;

mod api;
mod boards;
mod health_checks;
mod upload;
mod views;

use crate::api::{get_upload_by_id_route, list_uploads_route};
use crate::boards::Board;
use crate::health_checks::{liveness_probe_route, readiness_probe_route};
use crate::upload::{upload_route, UploadMeta};
use crate::views::get_view_path;

pub struct AppState {
    uploads: Mutex<HashMap<String, UploadMeta>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let mut mock_uploads = HashMap::new();
    mock_uploads.insert(
        "fc2af63b-2e9d-467b-a598-4c603ab46dda".to_string(),
        UploadMeta {
            id: "fc2af63b-2e9d-467b-a598-4c603ab46dda".to_string(),
            board: Board::Seed,
            filename: "some_file.pd".to_string(),
            file_contents: "contents...".to_string(),
        },
    );
    mock_uploads.insert(
        "45b40974-a68e-4ea9-ae90-556b28f77aa2".to_string(),
        UploadMeta {
            id: "45b40974-a68e-4ea9-ae90-556b28f77aa2".to_string(),
            board: Board::Seed,
            filename: "some_other_file.pd".to_string(),
            file_contents: "other contents...".to_string(),
        },
    );

    let app_state = web::Data::new(AppState {
        uploads: Mutex::new(mock_uploads.clone()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .service(Files::new("/static", "./public/static"))
            .service(index_route)
            .service(upload_route)
            .service(list_uploads_route)
            .service(get_upload_by_id_route)
            .service(echo_route)
            .service(liveness_probe_route)
            .service(readiness_probe_route)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index_route() -> Result<NamedFile> {
    let view_path = get_view_path("home");

    Ok(NamedFile::open(view_path)?)
}

#[post("/echo")]
async fn echo_route(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
