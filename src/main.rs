use std::{path::PathBuf, sync::Mutex};

use actix_files::{self, Files, NamedFile};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, web};
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

struct AppState {
    current_frame: Mutex<String>,
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

#[get("/save-frame/{frame}")]
async fn save_frame(data: web::Data<AppState>, frame: web::Path<String>) -> impl Responder {
    let frame_data = frame.into_inner();

    data.current_frame
        .lock()
        .unwrap()
        .replace_range(.., &frame_data);

    HttpResponse::Ok()
}

#[get("/get-frame")]
async fn get_frame(data: web::Data<AppState>) -> impl Responder {
    let frame_data = data.current_frame.lock().unwrap().clone();

    let frame_json = FrameData { frame: frame_data };

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

    let app_state = web::Data::new(AppState {
        current_frame: Mutex::new(String::new()),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .service(healthcheck)
            .service(download_client)
            .service(test_frame)
            .service(save_frame)
            .service(get_frame)
            .service(
                Files::new("/", "./web-ui/dist")
                    .index_file("index.html")
                    .use_last_modified(true),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
