use log::{debug, error, info};
use std::collections::{HashMap, VecDeque};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::env_config::get_env_config;
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
                debug!("Found {} items in the queue", queue.len());

                if let Some(patch_id) = queue.pop_front() {
                    if let Some(patch_meta) = patches.get(&patch_id) {
                        break 'queue_result Some(patch_meta.clone());
                    }
                }
            }

            None
        };

        if let Some(patch) = patch_to_compile {
            info!("Compiling patch {}...", patch.id);

            // TODO: set the patch's status as "compiling"

            compile_patch(&patch.id).await;

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

async fn compile_patch(patch_id: &str) {
    let env_config = get_env_config();

    let mut filename_pd2dsy_script = env_config.dir_pd2dsy.clone();
    filename_pd2dsy_script.push("pd2dsy.py");

    let mut filename_patch = env_config.dir_workspace.clone();
    filename_patch.push(format!("{patch_id}.pd"));

    let mut dir_patch_build = env_config.dir_pd2dsy.clone();
    dir_patch_build.push("builds");
    dir_patch_build.push(patch_id);

    // Step 1: generate C++ code from the patch
    let mut child = Command::new("python3")
        .arg(filename_pd2dsy_script.as_path())
        .arg("--board")
        .arg("pod")
        .arg("--directory")
        .arg("builds")
        .arg("--libdaisy-depth")
        .arg("2")
        .arg("--no-build")
        .arg(filename_patch.as_path())
        .current_dir(env_config.dir_pd2dsy.as_path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn");
    let status_code = child.wait().await.unwrap();

    if !status_code.success() {
        panic!("TODO: handle case when pd2dsy fails");
    }

    // Step 2: compile binary
    let mut child_2 = Command::new("make")
        .current_dir(dir_patch_build.as_path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn");
    let status_code_2 = child_2.wait().await.unwrap();

    if !status_code_2.success() {
        panic!("TODO: handle case when make fails");
    }
}
