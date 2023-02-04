use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
use log::debug;
use std::env;

mod boards;
mod health_checks;
mod upload;

use crate::health_checks::{liveness_probe_route, readiness_probe_route};
use crate::upload::upload_route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(Files::new("/static", "./public/static"))
            .service(index_route)
            .service(upload_route)
            .service(echo_route)
            .service(liveness_probe_route)
            .service(readiness_probe_route)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index_route() -> Result<NamedFile> {
    let mut index_path = env::current_dir().unwrap();
    index_path.push("public");
    index_path.push("index.html");
    debug!("Path to index file: {:?}", index_path);

    Ok(NamedFile::open(index_path)?)
}

#[post("/echo")]
async fn echo_route(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
