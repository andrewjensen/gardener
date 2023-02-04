use log::{debug, info};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::patches::PatchesStore;

pub fn init_compilation_worker() -> (Arc<PatchesStore>, JoinHandle<()>, CancellationToken) {
    let patches_store = PatchesStore {
        patches: Mutex::new(HashMap::new()),
        compilation_queue: Mutex::new(VecDeque::new()),
    };

    let patches_store_container = Arc::new(patches_store);

    let worker_cancel = CancellationToken::new();

    (
        Arc::clone(&patches_store_container),
        tokio::spawn(spawn_worker(
            Arc::clone(&patches_store_container),
            worker_cancel.clone(),
        )),
        worker_cancel,
    )
}

async fn spawn_worker(patches_store: Arc<PatchesStore>, stop_signal: CancellationToken) {
    loop {
        // only _try_ to lock so reads and writes from route handlers do not get blocked
        if let Ok(/* mut */ patches) = patches_store.patches.try_lock() {
            debug!("Found {} patches", patches.len());
        }

        tokio::select! {
            _ = sleep(Duration::from_secs(5)) => {
                continue;
            }

            _ = stop_signal.cancelled() => {
                info!("gracefully shutting down compilation worker...");
                break;
            }
        };
    }
}
