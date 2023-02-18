use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use serde::{Serialize, Serializer};
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
    pub time_upload: DateTime,
    pub time_compile_start: Option<DateTime>,
    pub time_compile_end: Option<DateTime>,
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
    Failed {
        summary: String,
        details: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct DateTime {
    inner: chrono::DateTime<chrono::offset::Utc>,
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self.inner))
    }
}

impl DateTime {
    pub fn now() -> Self {
        DateTime {
            inner: chrono::offset::Utc::now(),
        }
    }
}

pub fn validate_patch_file_contents(file_contents: &str) -> Result<()> {
    if file_contents.contains("#N canvas") {
        Ok(())
    } else {
        Err(anyhow!("File does not appear to be a Pd patch"))
    }
}
