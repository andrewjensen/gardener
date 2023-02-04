use log::{debug, error, info};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::patches::{PatchMeta, PatchStatus, PatchesStore};

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
        let patch_to_compile: Option<PatchMeta> = 'queue_result: {
            // only _try_ to lock so reads and writes from route handlers do not get blocked
            let queue_lock = patches_store.compilation_queue.try_lock();
            let patches_lock = patches_store.patches.try_lock();

            if let (Ok(mut queue), Ok(patches)) = (queue_lock, patches_lock) {
                debug!(
                    "Found {} patches, {} items in the queue",
                    patches.len(),
                    queue.len()
                );

                if let Some(patch_id) = queue.pop_front() {
                    if let Some(patch_meta) = patches.get(&patch_id) {
                        break 'queue_result Some(patch_meta.clone());
                    }
                }
            }

            None
        };

        if let Some(patch) = patch_to_compile {
            info!("Time to compile patch {}", patch.id);

            // TODO: set the patch's status as "compiling"

            // TODO: actually compile here
            sleep(Duration::from_secs(20)).await;

            info!("Finished compiling patch {}", patch.id);

            if let Ok(mut patches) = patches_store.patches.try_lock() {
                patches.insert(
                    patch.id.clone(),
                    PatchMeta {
                        status: PatchStatus::Compiled,
                        ..patch
                    },
                );
            } else {
                error!("TODO: could not update patches after compilation, handle gracefully");

                panic!();
            }
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
