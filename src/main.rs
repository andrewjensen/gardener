use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
use log::info;
use std::sync::Arc;

mod api;
mod boards;
mod compilation_worker;
mod health_checks;
mod patches;
mod upload;
mod views;

use crate::api::{get_patch_by_id_route, list_patches_route};
use crate::compilation_worker::init_compilation_worker;
use crate::health_checks::{liveness_probe_route, readiness_probe_route};
use crate::upload::upload_route;
use crate::views::get_view_path;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let (patches_store, worker_join_handle, worker_cancel) = init_compilation_worker();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(Arc::clone(&patches_store)))
            .wrap(Logger::default())
            .service(Files::new("/static", "./public/static"))
            .service(index_route)
            .service(upload_route)
            .service(list_patches_route)
            .service(get_patch_by_id_route)
            .service(echo_route)
            .service(liveness_probe_route)
            .service(readiness_probe_route)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    worker_cancel.cancel();

    worker_join_handle.await.unwrap();

    info!("All processes shut down gracefully.");

    Ok(())
}

#[get("/")]
async fn index_route() -> Result<NamedFile> {
    let view_path = get_view_path("home");

    Ok(NamedFile::open(view_path)?)
}

#[post("/echo")]
async fn echo_route(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
