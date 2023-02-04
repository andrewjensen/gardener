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
    pub board: Board,
    pub filename: String,
    pub file_contents: String,
}
