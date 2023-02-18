use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::info;
use std::sync::Arc;

mod boards;
mod compilation_worker;
mod env_config;
mod patches;
mod routes;
mod upload;

use crate::compilation_worker::init_compilation_worker;
use crate::env_config::get_env_config;
use crate::routes::{
    about_route, get_patch_by_id_route, index_route, list_patches_route, liveness_probe_route,
    patch_page_route, readiness_probe_route, upload_route,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Make sure we have configured our env correctly
    let _ = get_env_config();

    let (patches_store, worker_join_handle, worker_cancel) = init_compilation_worker();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(Arc::clone(&patches_store)))
            .wrap(Logger::default())
            .service(Files::new("/static", "./public/static").use_etag(true))
            .service(Files::new("/downloads", "./workspace/downloads"))
            .service(index_route)
            .service(about_route)
            .service(patch_page_route)
            .service(upload_route)
            .service(list_patches_route)
            .service(get_patch_by_id_route)
            .service(liveness_probe_route)
            .service(readiness_probe_route)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    worker_cancel.cancel();

    worker_join_handle.await.unwrap();

    info!("All processes shut down gracefully.");

    Ok(())
}
