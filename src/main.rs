use std::sync::Mutex;

use actix_files::{self, Files};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct FrameData {
    frame: String,
}

#[derive(Deserialize)]
struct ClientQuery {
    // Example: http://localhost:8080/get-frame
    backend_url: String,
    // https://tweaked.cc/module/peripheral.html
    // Example: "bottom", "top", "left", "right", "front", "back"
    monitor_side: String,
}

impl ClientQuery {
    /// Validate monitor_side is one of the allowed values
    fn validate_monitor_side(&self) -> Result<(), String> {
        const VALID_SIDES: &[&str] = &["top", "bottom", "left", "right", "front", "back"];
        if VALID_SIDES.contains(&self.monitor_side.as_str()) {
            Ok(())
        } else {
            Err(format!(
                "Invalid monitor_side '{}'. Must be one of: {}",
                self.monitor_side,
                VALID_SIDES.join(", ")
            ))
        }
    }

    /// Validate backend_url is a valid HTTP/HTTPS URL
    fn validate_backend_url(&self) -> Result<(), String> {
        // Basic URL validation
        if !self.backend_url.starts_with("http://") && !self.backend_url.starts_with("https://") {
            return Err("backend_url must start with http:// or https://".to_string());
        }

        // Check for dangerous characters that could break Lua string
        if self.backend_url.contains('"')
            || self.backend_url.contains('\n')
            || self.backend_url.contains('\r')
        {
            return Err("backend_url contains invalid characters".to_string());
        }

        // Length check
        if self.backend_url.len() > 100 {
            return Err("backend_url is too long (max 100 characters)".to_string());
        }

        Ok(())
    }

    /// Validate all fields
    fn validate(&self) -> Result<(), String> {
        self.validate_monitor_side()?;
        self.validate_backend_url()?;
        Ok(())
    }
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

#[get("/client")]
async fn download_client(query: web::Query<ClientQuery>) -> impl Responder {
    // Validate input parameters
    if let Err(e) = query.validate() {
        log::warn!("Invalid client query parameters: {}", e);
        return HttpResponse::BadRequest().body(format!("Validation error: {}", e));
    }

    // Load the client.lua file
    let lua_content = match std::fs::read_to_string("./client.lua") {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read client.lua: {}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to read client.lua: {}", e));
        }
    };

    // Escape strings for Lua (already validated, but extra safety)
    let safe_monitor_side = query.monitor_side.replace('"', r#"\""#);
    let safe_backend_url = query.backend_url.replace('"', r#"\""#);

    // Replace placeholders with actual values
    let modified_content = lua_content
        .replace(
            r#"local monitor = peripheral.wrap("") -- This value is set before the download"#,
            &format!(
                r#"local monitor = peripheral.wrap("{}")"#,
                safe_monitor_side
            ),
        )
        .replace(
            r#"local frame_endpoint = "" -- This value is set before the download"#,
            &format!(r#"local frame_endpoint = "{}""#, safe_backend_url),
        );

    // Return the modified Lua file
    HttpResponse::Ok()
        .content_type("text/x-lua")
        .insert_header(("Content-Disposition", "attachment; filename=\"client.lua\""))
        .body(modified_content)
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

    // Validate frame data size (e.g., max 1MB)
    const MAX_FRAME_SIZE: usize = 1_000_000;
    if frame_data.len() > MAX_FRAME_SIZE {
        log::warn!("Frame data too large: {} bytes", frame_data.len());
        return HttpResponse::PayloadTooLarge().body(format!(
            "Frame data exceeds maximum size of {} bytes",
            MAX_FRAME_SIZE
        ));
    }

    // Validate frame data contains only valid characters (hex digits)
    if !frame_data.chars().all(|c| c.is_ascii_hexdigit()) {
        log::warn!("Frame data contains invalid characters");
        return HttpResponse::BadRequest()
            .body("Frame data must contain only hexadecimal characters (0-9, a-f)");
    }

    data.current_frame
        .lock()
        .unwrap()
        .replace_range(.., &frame_data);

    HttpResponse::Ok().body("Frame saved successfully")
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
