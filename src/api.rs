use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use log::warn;
use serde::Serialize;
use std::collections::HashMap;

use crate::upload::UploadMeta;
use crate::AppState;

#[derive(Serialize, Debug)]
struct UploadListResponse {
    uploads: HashMap<String, UploadMeta>,
}

impl Responder for UploadListResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[get("/api/uploads")]
async fn list_uploads_route(data: web::Data<AppState>) -> impl Responder {
    warn!("TODO: put this endpoint behind authentication");

    let uploads = data.uploads.clone();

    UploadListResponse { uploads }
}

#[get("/api/uploads/{upload_id}")]
async fn get_upload_by_id_route(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let upload_id = path.into_inner();

    let uploads = &data.uploads;

    match uploads.get(&upload_id) {
        Some(upload_meta) => upload_meta.clone(),
        None => {
            warn!("TODO: figure out how to handle the not-found case properly");

            panic!()
        }
    }
}
