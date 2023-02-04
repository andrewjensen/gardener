use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use crate::boards::Board;

pub struct PatchesStore {
    pub patches: PatchesMap,
    pub compilation_queue: Mutex<VecDeque<String>>,
}

pub type PatchesMap = Mutex<HashMap<String, PatchMeta>>;

#[derive(Serialize, Debug, Clone)]
pub struct PatchMeta {
    pub id: String,
    pub status: PatchStatus,
    pub board: Board,
    pub filename: String,
    pub file_contents: String,
}

impl Responder for PatchMeta {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum PatchStatus {
    Uploaded,
    Compiling,
    Compiled,
}
