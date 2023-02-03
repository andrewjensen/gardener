use actix_files::NamedFile;
use actix_web::middleware::Logger;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
use log::debug;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(index)
            .service(echo)
            .service(liveness_probe)
            .service(readiness_probe)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index() -> Result<NamedFile> {
    let root_path = env::current_dir().unwrap();
    let mut index_path = root_path.clone();
    index_path.push("public");
    index_path.push("index.html");
    debug!("Path to index file: {:?}", index_path);

    Ok(NamedFile::open(index_path)?)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/health/live")]
async fn liveness_probe() -> impl Responder {
    HttpResponse::Ok().body("App is live")
}

#[get("/health/ready")]
async fn readiness_probe() -> impl Responder {
    HttpResponse::Ok().body("App is ready to receive traffic")
}
