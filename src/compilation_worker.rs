use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Stdio;
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::boards::Board;
use crate::env_config::{get_env_config, EnvConfig};
use crate::patches::{DateTime, PatchMeta, PatchStatus, PatchesStore};

lazy_static! {
    static ref REGEX_ESCAPE_SEQUENCE: Regex = Regex::new(r#"\x1b\[([0-9]+;)?[0-9]+m"#).unwrap();
}

#[derive(Error, Debug)]
pub enum CompilationError {
    #[error("pd2dsy failed")]
    Pd2dsyFailed { stdout: String },

    #[error("make command failed")]
    MakeFailed,

    #[error("move command failed")]
    MoveFailed,

    #[error("rm command failed")]
    RemoveFailed,

    #[error("I/O error occurred")]
    UnknownIOError(#[from] std::io::Error),
}

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
                trace!("Found {} items in the queue", queue.len());

                if let Some(patch_id) = queue.pop_front() {
                    if let Some(patch_meta) = patches.get(&patch_id) {
                        break 'queue_result Some(patch_meta.clone());
                    }
                }
            }

            None
        };

        if let Some(patch) = patch_to_compile {
            process_patch(patch, Arc::clone(&patches_store)).await;
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

async fn process_patch(patch: PatchMeta, patches_store: Arc<PatchesStore>) {
    let patch_id = patch.id.clone();

    info!("Compiling patch {}...", patch_id);

    let compiling_patch = PatchMeta {
        status: PatchStatus::Compiling,
        time_compile_start: Some(DateTime::now()),
        ..patch.clone()
    };
    update_patches_store_item(&patch_id, &compiling_patch, Arc::clone(&patches_store));

    let compilation_result = compile_patch(&patch_id, &patch.board).await;

    match compilation_result {
        Ok(()) => {
            info!("Finished compiling patch {}", patch_id);

            let compiled_patch = PatchMeta {
                status: PatchStatus::Compiled,
                time_compile_end: Some(DateTime::now()),
                ..compiling_patch
            };
            update_patches_store_item(&patch_id, &compiled_patch, Arc::clone(&patches_store));
        }
        Err(err) => {
            warn!("Failed to compile patch {}", patch_id);

            let failed_status = match &err {
                CompilationError::Pd2dsyFailed { stdout } => PatchStatus::Failed {
                    summary: err.to_string(),
                    details: Some(remove_escape_sequences(stdout)),
                },
                _ => PatchStatus::Failed {
                    summary: err.to_string(),
                    details: None,
                },
            };

            let failed_patch = PatchMeta {
                status: failed_status,
                ..compiling_patch
            };
            update_patches_store_item(&patch_id, &failed_patch, Arc::clone(&patches_store));
        }
    };
}

fn update_patches_store_item(patch_id: &str, patch: &PatchMeta, patches_store: Arc<PatchesStore>) {
    if let Ok(mut patches) = patches_store.patches.try_lock() {
        patches.insert(patch_id.to_string(), patch.clone());
    } else {
        error!("TODO: could not update PatchesStore, handle gracefully");

        panic!();
    }
}

async fn compile_patch(patch_id: &str, board: &Board) -> Result<(), CompilationError> {
    let env_config = get_env_config();

    generate_cpp_code(patch_id, board, &env_config).await?;

    compile_binary(patch_id, &env_config).await?;

    move_binary_into_workspace(patch_id, &env_config).await?;

    remove_build_dir(patch_id, &env_config).await?;

    Ok(())
}

async fn generate_cpp_code(
    patch_id: &str,
    board: &Board,
    env_config: &EnvConfig,
) -> Result<(), CompilationError> {
    debug!("Generating C++ code...");

    let mut filename_pd2dsy_script = env_config.dir_pd2dsy.clone();
    filename_pd2dsy_script.push("pd2dsy.py");

    let mut filename_patch = env_config.dir_workspace.clone();
    filename_patch.push("uploads");
    filename_patch.push(format!("{patch_id}.pd"));

    let mut command = Command::new("python3");
    command
        .arg(filename_pd2dsy_script.as_path())
        .arg("--board")
        .arg(board.to_str())
        .arg("--directory")
        .arg("builds")
        .arg("--libdaisy-depth")
        .arg("2")
        .arg("--no-build")
        .arg(filename_patch.as_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(env_config.dir_pd2dsy.as_path());

    let child = command.spawn()?;

    let output = child.wait_with_output().await?;

    let status_code = output.status;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if env_config.display_compilation_output {
        debug!("Command output:\n{}", stdout);
    }

    if !status_code.success() {
        return Err(CompilationError::Pd2dsyFailed {
            stdout: stdout.to_string(),
        });
    }

    Ok(())
}

async fn compile_binary(patch_id: &str, env_config: &EnvConfig) -> Result<(), CompilationError> {
    debug!("Compiling binary...");

    let dir_patch_build = get_dir_patch_build(patch_id, env_config);

    let mut command = Command::new("make");
    command.current_dir(dir_patch_build);

    if !env_config.display_compilation_output {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let mut child = command.spawn()?;

    let status_code = child.wait().await?;

    if !status_code.success() {
        return Err(CompilationError::MakeFailed);
    }

    Ok(())
}

async fn move_binary_into_workspace(
    patch_id: &str,
    env_config: &EnvConfig,
) -> Result<(), CompilationError> {
    debug!("Moving binary into workspace...");

    let dir_patch_build = get_dir_patch_build(patch_id, env_config);

    let mut filename_compiled_binary = dir_patch_build.to_path_buf();
    filename_compiled_binary.push("build");
    filename_compiled_binary.push(format!("HeavyDaisy_{}.bin", patch_id.replace('-', "_")));

    let mut filename_in_downloads = env_config.dir_workspace.clone();
    filename_in_downloads.push("downloads");
    filename_in_downloads.push(format!("daisy-{patch_id}.bin"));

    let mut command = Command::new("mv");
    command
        .arg(filename_compiled_binary.as_path())
        .arg(filename_in_downloads.as_path());

    if !env_config.display_compilation_output {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let mut child = command.spawn()?;

    let status_code = child.wait().await?;

    if !status_code.success() {
        return Err(CompilationError::MoveFailed);
    }

    Ok(())
}

async fn remove_build_dir(patch_id: &str, env_config: &EnvConfig) -> Result<(), CompilationError> {
    debug!("Cleaning up...");

    let dir_patch_build = get_dir_patch_build(patch_id, env_config);

    let mut command = Command::new("rm");
    command.arg("-rf").arg(dir_patch_build.as_path());

    if !env_config.display_compilation_output {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let mut child = command.spawn()?;

    let status_code = child.wait().await?;

    if !status_code.success() {
        return Err(CompilationError::RemoveFailed);
    }

    Ok(())
}

fn get_dir_patch_build(patch_id: &str, env_config: &EnvConfig) -> PathBuf {
    let mut dir_patch_build = env_config.dir_pd2dsy.clone();
    dir_patch_build.push("builds");
    dir_patch_build.push(patch_id);

    dir_patch_build
}

fn remove_escape_sequences(terminal_output: &str) -> String {
    REGEX_ESCAPE_SEQUENCE
        .replace_all(terminal_output, "")
        .to_string()
}
