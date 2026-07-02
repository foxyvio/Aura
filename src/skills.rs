use actix_web::{web, HttpResponse, HttpRequest as ActixHttpRequest};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Skill, CreateSkillRequest, ApiResponse};
use crate::auth::verify_token;

pub async fn create_skill(
    pool: web::Data<SqlitePool>,
    req: ActixHttpRequest,
    body: web::Json<CreateSkillRequest>,
) -> HttpResponse {
    // Extract and verify token
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

    let claims = match verify_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Invalid token".to_string()));
        }
    };

    // Get agent_id from user_id (for now, assume one agent per user)
    let agent_id = match sqlx::query_scalar::<_, String>(
        "SELECT id FROM agents WHERE user_id = ? LIMIT 1"
    )
    .bind(&claims.sub)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .json(ApiResponse::<()>::error("Agent not found for user".to_string()));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()));
        }
    };

    let skill_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    match sqlx::query(
        "INSERT INTO skills (id, agent_id, title, description, category, price, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&skill_id)
    .bind(&agent_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(&body.category)
    .bind(body.price)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            let skill = Skill {
                id: skill_id,
                agent_id,
                title: body.title.clone(),
                description: body.description.clone(),
                category: body.category.clone(),
                price: body.price,
                created_at: now.clone(),
                updated_at: now,
            };
            HttpResponse::Ok().json(ApiResponse::success(skill))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create skill".to_string()))
        }
    }
}

pub async fn get_skill(
    pool: web::Data<SqlitePool>,
    skill_id: web::Path<String>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, Option<String>, String, f64, String, String)>(
        "SELECT id, agent_id, title, description, category, price, created_at, updated_at FROM skills WHERE id = ?"
    )
    .bind(skill_id.as_str())
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some((id, agent_id, title, description, category, price, created_at, updated_at))) => {
            let skill = Skill {
                id,
                agent_id,
                title,
                description,
                category,
                price,
                created_at,
                updated_at,
            };
            HttpResponse::Ok().json(ApiResponse::success(skill))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Skill not found".to_string()))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()))
        }
    }
}

pub async fn list_skills(
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, Option<String>, String, f64, String, String)>(
        "SELECT id, agent_id, title, description, category, price, created_at, updated_at FROM skills LIMIT 100"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let skills: Vec<Skill> = rows.into_iter().map(|(id, agent_id, title, description, category, price, created_at, updated_at)| {
                Skill {
                    id,
                    agent_id,
                    title,
                    description,
                    category,
                    price,
                    created_at,
                    updated_at,
                }
            }).collect();
            HttpResponse::Ok().json(ApiResponse::success(skills))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()))
        }
    }
}

pub async fn list_skills_by_agent(
    pool: web::Data<SqlitePool>,
    agent_id: web::Path<String>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, Option<String>, String, f64, String, String)>(
        "SELECT id, agent_id, title, description, category, price, created_at, updated_at FROM skills WHERE agent_id = ?"
    )
    .bind(agent_id.as_str())
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let skills: Vec<Skill> = rows.into_iter().map(|(id, agent_id, title, description, category, price, created_at, updated_at)| {
                Skill {
                    id,
                    agent_id,
                    title,
                    description,
                    category,
                    price,
                    created_at,
                    updated_at,
                }
            }).collect();
            HttpResponse::Ok().json(ApiResponse::success(skills))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()))
        }
    }
}
