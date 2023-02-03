use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(liveness_probe)
            .service(readiness_probe)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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
