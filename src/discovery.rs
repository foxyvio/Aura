use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;

use crate::models::{Agent, Skill, ApiResponse};

pub async fn get_trending_skills(
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, Option<String>, String, f64, String, String)>(
        "SELECT id, agent_id, title, description, category, price, created_at, updated_at FROM skills ORDER BY created_at DESC LIMIT 10"
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

pub async fn get_recent_agents(
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, f64, String, String)>(
        "SELECT id, user_id, name, description, capabilities, reputation_score, created_at, updated_at FROM agents ORDER BY created_at DESC LIMIT 10"
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

pub async fn search_skills(
    pool: web::Data<SqlitePool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let search_term = query.get("q").cloned().unwrap_or_default();
    let category = query.get("category").cloned();
    let min_price = query.get("min_price").and_then(|p| p.parse::<f64>().ok());
    let max_price = query.get("max_price").and_then(|p| p.parse::<f64>().ok());

    let mut sql = "SELECT id, agent_id, title, description, category, price, created_at, updated_at FROM skills WHERE 1=1".to_string();
    
    if !search_term.is_empty() {
        sql.push_str(&format!(" AND (title LIKE '%{}%' OR description LIKE '%{}%')", search_term, search_term));
    }
    
    if let Some(cat) = category {
        sql.push_str(&format!(" AND category = '{}'", cat));
    }
    
    if let Some(min) = min_price {
        sql.push_str(&format!(" AND price >= {}", min));
    }
    
    if let Some(max) = max_price {
        sql.push_str(&format!(" AND price <= {}", max));
    }

    sql.push_str(" LIMIT 50");

    match sqlx::query_as::<_, (String, String, String, Option<String>, String, f64, String, String)>(&sql)
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

pub async fn get_agent_stats(
    pool: web::Data<SqlitePool>,
    agent_id: web::Path<String>,
) -> HttpResponse {
    let agent_id_str = agent_id.as_str();

    // Get agent info
    let agent = match sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, f64, String, String)>(
        "SELECT id, user_id, name, description, capabilities, reputation_score, created_at, updated_at FROM agents WHERE id = ?"
    )
    .bind(agent_id_str)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some((id, user_id, name, description, capabilities, reputation_score, created_at, updated_at))) => {
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
        }
        _ => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Agent not found".to_string()));
        }
    };

    // Get skills count
    let skills_count: (i64,) = match sqlx::query_as(
        "SELECT COUNT(*) FROM skills WHERE agent_id = ?"
    )
    .bind(agent_id_str)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(count) => count,
        Err(_) => (0,),
    };

    // Get transactions count
    let transactions_count: (i64,) = match sqlx::query_as(
        "SELECT COUNT(*) FROM transactions WHERE seller_agent_id = ?"
    )
    .bind(agent_id_str)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(count) => count,
        Err(_) => (0,),
    };

    // Get total earnings
    let total_earnings: (Option<f64>,) = match sqlx::query_as(
        "SELECT SUM(amount) FROM transactions WHERE seller_agent_id = ? AND status = 'completed'"
    )
    .bind(agent_id_str)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok((Some(earnings),)) => (Some(earnings),),
        _ => (Some(0.0),),
    };

    let stats = serde_json::json!({
        "agent": agent,
        "skills_count": skills_count.0,
        "transactions_count": transactions_count.0,
        "total_earnings": total_earnings.0.unwrap_or(0.0),
        "reputation_score": agent.reputation_score,
    });

    HttpResponse::Ok().json(ApiResponse::success(stats))
}
