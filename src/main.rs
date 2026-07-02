mod db;
mod models;
mod auth;
mod agents;
mod skills;
mod transactions;
mod discovery;
mod dashboard;

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load environment variables
    dotenv::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:aura.db".to_string());
    
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    // Initialize database
    let pool = db::init_db(&database_url)
        .await
        .expect("Failed to initialize database");

    let pool_data = web::Data::new(pool);

    println!("Starting Aura backend server on {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .app_data(pool_data.clone())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            // Auth routes
            .route("/api/auth/register", web::post().to(auth::register))
            .route("/api/auth/login", web::post().to(auth::login))
            // Agent routes
            .route("/api/agents", web::post().to(agents::create_agent))
            .route("/api/agents", web::get().to(agents::list_agents))
            .route("/api/agents/{id}", web::get().to(agents::get_agent))
            // Skill routes
            .route("/api/skills", web::post().to(skills::create_skill))
            .route("/api/skills", web::get().to(skills::list_skills))
            .route("/api/skills/{id}", web::get().to(skills::get_skill))
            .route("/api/agents/{id}/skills", web::get().to(skills::list_skills_by_agent))
            // Transaction routes
            .route("/api/transactions", web::post().to(transactions::create_transaction))
            .route("/api/transactions", web::get().to(transactions::list_transactions))
            .route("/api/transactions/{id}", web::get().to(transactions::get_transaction))
            .route("/api/transactions/{id}/status", web::patch().to(transactions::update_transaction_status))
            // Discovery routes
            .route("/api/discovery/trending-skills", web::get().to(discovery::get_trending_skills))
            .route("/api/discovery/recent-agents", web::get().to(discovery::get_recent_agents))
            .route("/api/discovery/search-skills", web::get().to(discovery::search_skills))
            .route("/api/agents/{id}/stats", web::get().to(discovery::get_agent_stats))
            // Dashboard routes
            .route("/api/dashboard", web::get().to(dashboard::get_user_dashboard))
            .route("/api/dashboard/transactions", web::get().to(dashboard::get_user_transactions))
            .route("/api/dashboard/skills", web::get().to(dashboard::get_user_skills))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
