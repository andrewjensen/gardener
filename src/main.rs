use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;

mod boards;
mod health_checks;
mod upload;
mod views;

use crate::health_checks::{liveness_probe_route, readiness_probe_route};
use crate::upload::upload_route;
use crate::views::get_view_path;

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
    let view_path = get_view_path("home");

    Ok(NamedFile::open(view_path)?)
}

#[post("/echo")]
async fn echo_route(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
