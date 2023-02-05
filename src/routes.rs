use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use log::{error, info, warn};
use serde::Serialize;
use std::collections::HashMap;

use crate::patches::{PatchMeta, PatchesStore};
use crate::upload::{process_patch_upload, write_patch_to_disk};
use crate::views::get_view_path;

#[derive(Serialize, Debug)]
struct PatchListResponse {
    patches: HashMap<String, PatchMeta>,
}

impl Responder for PatchListResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[get("/")]
pub async fn index_route() -> Result<NamedFile> {
    let view_path = get_view_path("home");

    Ok(NamedFile::open(view_path)?)
}

#[post("/upload")]
pub async fn upload_route(
    payload: Multipart,
    patches_store: web::Data<PatchesStore>,
) -> Result<NamedFile> {
    info!("Starting the upload endpoint...");

    match process_patch_upload(payload).await {
        Some(patch_meta) => {
            write_patch_to_disk(&patch_meta.id, &patch_meta.file_contents).await;

            let patch_id = patch_meta.id.clone();

            let mut patches = patches_store.patches.lock().unwrap();
            patches.insert(patch_id.clone(), patch_meta);

            let mut queue = patches_store.compilation_queue.lock().unwrap();
            queue.push_back(patch_id);

            let view_path = get_view_path("upload_success");

            Ok(NamedFile::open(view_path).unwrap())
        }
        None => {
            error!("TODO: Something went wrong during the upload, handle gracefully");

            panic!();
        }
    }
}

#[get("/api/patches")]
async fn list_patches_route(patches_store: web::Data<PatchesStore>) -> impl Responder {
    warn!("TODO: put this endpoint behind authentication");

    let patches = patches_store.patches.lock().unwrap().clone();

    PatchListResponse { patches }
}

#[get("/api/patches/{patch_id}")]
async fn get_patch_by_id_route(
    path: web::Path<String>,
    patches_store: web::Data<PatchesStore>,
) -> impl Responder {
    let patch_id = path.into_inner();

    let patches = patches_store.patches.lock().unwrap();

    match patches.get(&patch_id) {
        Some(patch_meta) => patch_meta.clone(),
        None => {
            warn!("TODO: figure out how to handle the not-found case properly");

            panic!()
        }
    }
}

#[get("/health/live")]
pub async fn liveness_probe_route() -> impl Responder {
    HttpResponse::Ok().body("App is live")
}

#[get("/health/ready")]
pub async fn readiness_probe_route() -> impl Responder {
    HttpResponse::Ok().body("App is ready to receive traffic")
}
