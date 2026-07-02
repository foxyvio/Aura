use actix_web::{web, HttpResponse, HttpRequest as ActixHttpRequest};
use sqlx::SqlitePool;

use crate::models::{Skill, Transaction, ApiResponse};
use crate::auth::verify_token;

pub async fn get_user_dashboard(
    pool: web::Data<SqlitePool>,
    req: ActixHttpRequest,
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

    // Get agent for this user
    let agent_id = match sqlx::query_scalar::<_, String>(
        "SELECT id FROM agents WHERE user_id = ? LIMIT 1"
    )
    .bind(&claims.sub)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Agent not found for user".to_string()));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()));
        }
    };

    // Get owned skills
    let skills = match sqlx::query_as::<_, (String, String, String, Option<String>, String, f64, String, String)>(
        "SELECT id, agent_id, title, description, category, price, created_at, updated_at FROM skills WHERE agent_id = ?"
    )
    .bind(&agent_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => rows.into_iter().map(|(id, agent_id, title, description, category, price, created_at, updated_at)| {
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
        }).collect::<Vec<_>>(),
        Err(_) => vec![],
    };

    // Get active transactions
    let transactions = match sqlx::query_as::<_, (String, String, String, String, String, f64, String, String)>(
        "SELECT id, buyer_agent_id, seller_agent_id, skill_id, status, amount, created_at, updated_at FROM transactions WHERE seller_agent_id = ? OR buyer_agent_id = ? LIMIT 20"
    )
    .bind(&agent_id)
    .bind(&agent_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => rows.into_iter().map(|(id, buyer_agent_id, seller_agent_id, skill_id, status, amount, created_at, updated_at)| {
            Transaction {
                id,
                buyer_agent_id,
                seller_agent_id,
                skill_id,
                status,
                amount,
                created_at,
                updated_at,
            }
        }).collect::<Vec<_>>(),
        Err(_) => vec![],
    };

    // Get total earnings
    let total_earnings: (Option<f64>,) = match sqlx::query_as(
        "SELECT SUM(amount) FROM transactions WHERE seller_agent_id = ? AND status = 'completed'"
    )
    .bind(&agent_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok((Some(earnings),)) => (Some(earnings),),
        _ => (Some(0.0),),
    };

    // Get agent reputation
    let reputation: (f64,) = match sqlx::query_as(
        "SELECT reputation_score FROM agents WHERE id = ?"
    )
    .bind(&agent_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(rep) => rep,
        Err(_) => (0.0,),
    };

    let dashboard = serde_json::json!({
        "agent_id": agent_id,
        "owned_skills": skills,
        "active_transactions": transactions,
        "total_earnings": total_earnings.0.unwrap_or(0.0),
        "reputation_score": reputation.0,
        "skills_count": skills.len(),
        "transactions_count": transactions.len(),
    });

    HttpResponse::Ok().json(ApiResponse::success(dashboard))
}

pub async fn get_user_transactions(
    pool: web::Data<SqlitePool>,
    req: ActixHttpRequest,
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

    // Get agent for this user
    let agent_id = match sqlx::query_scalar::<_, String>(
        "SELECT id FROM agents WHERE user_id = ? LIMIT 1"
    )
    .bind(&claims.sub)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Agent not found for user".to_string()));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()));
        }
    };

    // Get all transactions for this agent
    match sqlx::query_as::<_, (String, String, String, String, String, f64, String, String)>(
        "SELECT id, buyer_agent_id, seller_agent_id, skill_id, status, amount, created_at, updated_at FROM transactions WHERE seller_agent_id = ? OR buyer_agent_id = ? ORDER BY created_at DESC LIMIT 100"
    )
    .bind(&agent_id)
    .bind(&agent_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let transactions: Vec<Transaction> = rows.into_iter().map(|(id, buyer_agent_id, seller_agent_id, skill_id, status, amount, created_at, updated_at)| {
                Transaction {
                    id,
                    buyer_agent_id,
                    seller_agent_id,
                    skill_id,
                    status,
                    amount,
                    created_at,
                    updated_at,
                }
            }).collect();
            HttpResponse::Ok().json(ApiResponse::success(transactions))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()))
        }
    }
}

pub async fn get_user_skills(
    pool: web::Data<SqlitePool>,
    req: ActixHttpRequest,
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

    // Get agent for this user
    let agent_id = match sqlx::query_scalar::<_, String>(
        "SELECT id FROM agents WHERE user_id = ? LIMIT 1"
    )
    .bind(&claims.sub)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Agent not found for user".to_string()));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()));
        }
    };

    // Get all skills for this agent
    match sqlx::query_as::<_, (String, String, String, Option<String>, String, f64, String, String)>(
        "SELECT id, agent_id, title, description, category, price, created_at, updated_at FROM skills WHERE agent_id = ? ORDER BY created_at DESC"
    )
    .bind(&agent_id)
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
