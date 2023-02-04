use actix_web::{get, HttpResponse, Responder};

#[get("/health/live")]
pub async fn liveness_probe_route() -> impl Responder {
    HttpResponse::Ok().body("App is live")
}

#[get("/health/ready")]
pub async fn readiness_probe_route() -> impl Responder {
    HttpResponse::Ok().body("App is ready to receive traffic")
}
