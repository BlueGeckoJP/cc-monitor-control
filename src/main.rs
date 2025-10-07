use actix_web::{App, HttpResponse, HttpServer, Responder, get};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

#[get("/healthcheck")]
async fn healthcheck() -> impl Responder {
    let response = HealthResponse {
        status: "ok".to_string(),
        message: "Service is healthy".to_string(),
    };
    HttpResponse::Ok().json(response)
}

#[get("/download-client")]
async fn download_client() -> impl Responder {
    use actix_files::NamedFile;
    use std::path::PathBuf;

    let path: PathBuf = "./client.lua".parse().unwrap();
    NamedFile::open(path)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://0.0.0.0:8080");

    HttpServer::new(|| App::new().service(healthcheck).service(download_client))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
