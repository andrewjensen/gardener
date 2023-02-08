use anyhow::{anyhow, Result};
use log::{debug, error, info};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::boards::Board;
use crate::env_config::{get_env_config, EnvConfig};
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

            // TODO: this loop is too big and unruly, clean it up

            match compile_patch(&patch.id, &patch.board).await {
                Ok(()) => {},
                Err(_) => {
                    panic!("TODO: handle cases where compilation fails");
                }
            };

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

async fn compile_patch(patch_id: &str, board: &Board) -> Result<()> {
    let env_config = get_env_config();

    generate_cpp_code(patch_id, board, &env_config).await?;

    compile_binary(patch_id, &env_config).await?;

    move_binary_into_workspace(patch_id, &env_config).await?;

    remove_build_dir(patch_id, &env_config).await?;

    Ok(())
}

async fn generate_cpp_code(patch_id: &str, board: &Board, env_config: &EnvConfig) -> Result<()> {
    debug!("Generating C++ code...");

    let mut filename_pd2dsy_script = env_config.dir_pd2dsy.clone();
    filename_pd2dsy_script.push("pd2dsy.py");

    let mut filename_patch = env_config.dir_workspace.clone();
    filename_patch.push("uploads");
    filename_patch.push(format!("{patch_id}.pd"));

    let mut child = Command::new("python3")
        .arg(filename_pd2dsy_script.as_path())
        .arg("--board")
        .arg(board.to_str())
        .arg("--directory")
        .arg("builds")
        .arg("--libdaisy-depth")
        .arg("2")
        .arg("--no-build")
        .arg(filename_patch.as_path())
        .current_dir(env_config.dir_pd2dsy.as_path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // if !env_config.display_compilation_output {
    //     child
    //     .stdout(Stdio::null())
    //     .stderr(Stdio::null());
    // }

    // child.spawn()?;

    let status_code = child.wait().await?;

    if !status_code.success() {
        return Err(anyhow!(
            "pd2dsy script exited with non-zero exit status: {}",
            status_code
        ));
    }

    Ok(())
}

async fn compile_binary(patch_id: &str, env_config: &EnvConfig) -> Result<()> {
    debug!("Compiling binary...");

    let dir_patch_build = get_dir_patch_build(patch_id, env_config);

    let mut child = Command::new("make")
        .current_dir(dir_patch_build)
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
        .spawn()?;

    let status_code = child.wait().await?;

    if !status_code.success() {
        return Err(anyhow!(
            "make exited with non-zero exit status: {}",
            status_code
        ));
    }

    Ok(())
}

async fn move_binary_into_workspace(patch_id: &str, env_config: &EnvConfig) -> Result<()> {
    debug!("Moving binary into workspace...");

    let dir_patch_build = get_dir_patch_build(patch_id, env_config);

    let mut filename_compiled_binary = dir_patch_build.to_path_buf();
    filename_compiled_binary.push("build");
    filename_compiled_binary.push(format!("HeavyDaisy_{}.bin", patch_id.replace('-', "_")));

    let mut filename_in_downloads = env_config.dir_workspace.clone();
    filename_in_downloads.push("downloads");
    filename_in_downloads.push(format!("daisy-{patch_id}.bin"));

    let mut child = Command::new("mv")
        .arg(filename_compiled_binary.as_path())
        .arg(filename_in_downloads.as_path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let status_code = child.wait().await?;

    if !status_code.success() {
        return Err(anyhow!(
            "mv exited with non-zero exit status: {}",
            status_code
        ));
    }

    Ok(())
}

async fn remove_build_dir(patch_id: &str, env_config: &EnvConfig) -> Result<()> {
    debug!("Cleaning up...");

    let dir_patch_build = get_dir_patch_build(patch_id, env_config);

    let mut child = Command::new("rm")
        .arg("-rf")
        .arg(dir_patch_build.as_path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let status_code = child.wait().await?;

    if !status_code.success() {
        return Err(anyhow!(
            "rm exited with non-zero exit status: {}",
            status_code
        ));
    }

    Ok(())
}

fn get_dir_patch_build(patch_id: &str, env_config: &EnvConfig) -> PathBuf {
    let mut dir_patch_build = env_config.dir_pd2dsy.clone();
    dir_patch_build.push("builds");
    dir_patch_build.push(patch_id);

    dir_patch_build
}
