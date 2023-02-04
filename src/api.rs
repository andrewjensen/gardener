use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use log::warn;
use serde::Serialize;
use std::collections::HashMap;

use crate::patches::{PatchMeta, PatchesStore};

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
