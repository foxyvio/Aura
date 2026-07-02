use actix_web::{web, HttpResponse, HttpRequest as ActixHttpRequest};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Agent, CreateAgentRequest, ApiResponse};
use crate::auth::verify_token;

pub async fn create_agent(
    pool: web::Data<SqlitePool>,
    req: ActixHttpRequest,
    body: web::Json<CreateAgentRequest>,
) -> HttpResponse {
    // Extract token from header
    let token = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(auth_header) => {
                if auth_header.starts_with("Bearer ") {
                    &auth_header[7..]
                } else {
                    return HttpResponse::Unauthorized()
                        .json(ApiResponse::<()>::error("Invalid token format".to_string()));
                }
            }
            Err(_) => {
                return HttpResponse::Unauthorized()
                    .json(ApiResponse::<()>::error("Invalid token".to_string()));
            }
        },
        None => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Missing authorization header".to_string()));
        }
    };

    // Verify token
    let claims = match verify_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Invalid token".to_string()));
        }
    };

    let agent_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    match sqlx::query(
        "INSERT INTO agents (id, user_id, name, description, capabilities, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&agent_id)
    .bind(&claims.sub)
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.capabilities)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            let agent = Agent {
                id: agent_id,
                user_id: claims.sub,
                name: body.name.clone(),
                description: body.description.clone(),
                capabilities: body.capabilities.clone(),
                reputation_score: 0.0,
                created_at: now.clone(),
                updated_at: now,
            };
            HttpResponse::Ok().json(ApiResponse::success(agent))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create agent".to_string()))
        }
    }
}

pub async fn get_agent(
    pool: web::Data<SqlitePool>,
    agent_id: web::Path<String>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, f64, String, String)>(
        "SELECT id, user_id, name, description, capabilities, reputation_score, created_at, updated_at FROM agents WHERE id = ?"
    )
    .bind(agent_id.as_str())
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some((id, user_id, name, description, capabilities, reputation_score, created_at, updated_at))) => {
            let agent = Agent {
                id,
                user_id,
                name,
                description,
                capabilities,
                reputation_score,
                created_at,
                updated_at,
            };
            HttpResponse::Ok().json(ApiResponse::success(agent))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Agent not found".to_string()))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()))
        }
    }
}

pub async fn list_agents(
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, f64, String, String)>(
        "SELECT id, user_id, name, description, capabilities, reputation_score, created_at, updated_at FROM agents LIMIT 100"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let agents: Vec<Agent> = rows.into_iter().map(|(id, user_id, name, description, capabilities, reputation_score, created_at, updated_at)| {
                Agent {
                    id,
                    user_id,
                    name,
                    description,
                    capabilities,
                    reputation_score,
                    created_at,
                    updated_at,
                }
            }).collect();
            HttpResponse::Ok().json(ApiResponse::success(agents))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()))
        }
    }
}
