use actix_multipart::Multipart;
use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use askama::Template;
use log::{info, warn};
use serde::Serialize;
use std::collections::HashMap;

use crate::env_config::get_env_config;
use crate::patches::{PatchMeta, PatchesStore};
use crate::upload::process_patch_upload;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate;

#[derive(Template)]
#[template(path = "upload_success.html")]
struct UploadSuccessTemplate<'a> {
    patch_id: &'a str,
}

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
pub async fn index_route() -> Result<HttpResponse> {
    let res_body = HomeTemplate.render().unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(res_body))
}

#[get("/about")]
pub async fn about_route() -> Result<HttpResponse> {
    let res_body = AboutTemplate.render().unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(res_body))
}

#[post("/upload")]
pub async fn upload_route(
    payload: Multipart,
    patches_store: web::Data<PatchesStore>,
) -> Result<HttpResponse> {
    info!("Starting the upload endpoint...");

    match process_patch_upload(payload).await {
        Ok(patch_meta) => {
            let patch_id = patch_meta.id.clone();

            let mut patches = patches_store.patches.lock().unwrap();
            patches.insert(patch_id.clone(), patch_meta);

            let mut queue = patches_store.compilation_queue.lock().unwrap();
            queue.push_back(patch_id.clone());

            let res_body = UploadSuccessTemplate {
                patch_id: &patch_id,
            }
            .render()
            .unwrap();

            Ok(HttpResponse::Ok().content_type("text/html").body(res_body))
        }
        Err(reason) => {
            warn!("Error uploading patch: {reason}");

            let res_body = format!("Error uploading your patch: {reason}");

            // TODO: return an actual HTML page
            Ok(HttpResponse::BadRequest()
                .content_type("text/html")
                .body(res_body))
        }
    }
}

#[get("/api/patches")]
async fn list_patches_route(
    req: HttpRequest,
    patches_store: web::Data<PatchesStore>,
) -> impl Responder {
    // TODO: maybe replace with something in here:
    // https://github.com/actix/actix-extras
    if is_authenticated(&req) {
        let patches = patches_store.patches.lock().unwrap().clone();
        let response_body = PatchListResponse { patches };

        HttpResponse::Ok().body(serde_json::to_string(&response_body).unwrap())
    } else {
        HttpResponse::Unauthorized().body("You are not authenticated!")
    }
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

fn is_authenticated(req: &HttpRequest) -> bool {
    match req.headers().get("Authentication") {
        Some(auth_header) => {
            let header_value = auth_header.to_str().unwrap();

            header_value == get_env_config().admin_token
        }
        None => false,
    }
}
