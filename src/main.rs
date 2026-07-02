// सारे mod हटा दे अभी
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    reply: String,
    model: String,
    latency_ms: u128,
    private: bool,
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "live",
        "service": "Aura Rust Core",
        "version": "0.1.0-yc",
        "rust": "1.88",
        "modules": ["chat", "health"]
    }))
}

async fn chat(req: web::Json<ChatRequest>) -> Result<HttpResponse> {
    let start = std::time::Instant::now();

    let reply = format!(
        "[DEMO MODE] Aura On-Device Engine: '{}'. Privacy-first AI. Auth, Agents, Skills coming post-YC.",
        req.message
    );

    Ok(HttpResponse::Ok().json(ChatResponse {
        reply,
        model: "Aura-Core-Rust-1.88".to_string(),
        latency_ms: start.elapsed().as_millis(),
        private: true,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Aura YC Demo starting on {}", addr);

    HttpServer::new(|| {
        let cors = Cors::default()
         .allow_any_origin()
         .allow_any_method()
         .allow_any_header();

        App::new()
         .wrap(Logger::default())
         .wrap(cors)
         .route("/", web::get().to(health))
         .route("/health", web::get().to(health))
         .route("/api/chat", web::post().to(chat))
    })
  .bind(&addr)?
  .run()
  .await
}
