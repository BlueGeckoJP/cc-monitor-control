use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct FrameData {
    frame: String,
}

#[derive(Debug, strum_macros::Display)]
enum CCColors {
    #[strum(serialize = "0")]
    White,
    #[strum(serialize = "1")]
    Orange,
    #[strum(serialize = "2")]
    Magenta,
    #[strum(serialize = "3")]
    LightBlue,
    #[strum(serialize = "4")]
    Yellow,
    #[strum(serialize = "5")]
    Lime,
    #[strum(serialize = "6")]
    Pink,
    #[strum(serialize = "7")]
    Gray,
    #[strum(serialize = "8")]
    LightGray,
    #[strum(serialize = "9")]
    Cyan,
    #[strum(serialize = "a")]
    Purple,
    #[strum(serialize = "b")]
    Blue,
    #[strum(serialize = "c")]
    Brown,
    #[strum(serialize = "d")]
    Green,
    #[strum(serialize = "e")]
    Red,
    #[strum(serialize = "f")]
    Black,
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

#[get("/test-frame")]
async fn test_frame() -> impl Responder {
    let frame_json = FrameData {
        frame: generate_test_frame(82, 40, "rainbow"),
    };

    let json_string = serde_json::to_string(&frame_json).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_string)
}

fn generate_test_frame(width: usize, height: usize, pattern: &str) -> String {
    let mut frame = String::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            // Create different patterns based on the pattern parameter
            let color = match pattern {
                "gradient" => {
                    // Horizontal gradient
                    let ratio = (x as f32 / width as f32 * 15.0) as u8;
                    format!("{:x}", ratio)
                }
                "vertical" => {
                    // Vertical gradient
                    let ratio = (y as f32 / height as f32 * 15.0) as u8;
                    format!("{:x}", ratio)
                }
                "checkerboard" => {
                    // Checkerboard pattern
                    if (x + y) % 2 == 0 {
                        "f".to_string()
                    } else {
                        "0".to_string()
                    }
                }
                "rainbow" => {
                    // Rainbow pattern
                    let color_idx = ((x + y) % 16) as u8;
                    format!("{:x}", color_idx)
                }
                _ => {
                    // Default: random-like pattern based on position
                    format!("{:x}", ((x * 7 + y * 13) % 16) as u8)
                }
            };
            frame.push_str(&color);
        }
    }

    frame
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting server at http://0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(healthcheck)
            .service(download_client)
            .service(test_frame)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
