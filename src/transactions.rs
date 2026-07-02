use actix_web::{web, HttpResponse, HttpRequest as ActixHttpRequest};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Transaction, CreateTransactionRequest, ApiResponse};
use crate::auth::verify_token;

pub async fn create_transaction(
    pool: web::Data<SqlitePool>,
    req: ActixHttpRequest,
    body: web::Json<CreateTransactionRequest>,
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

    let _claims = match verify_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Invalid token".to_string()));
        }
    };

    let transaction_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    match sqlx::query(
        "INSERT INTO transactions (id, buyer_agent_id, seller_agent_id, skill_id, status, amount, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&transaction_id)
    .bind(&body.buyer_agent_id)
    .bind(&body.seller_agent_id)
    .bind(&body.skill_id)
    .bind("pending")
    .bind(body.amount)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            let transaction = Transaction {
                id: transaction_id,
                buyer_agent_id: body.buyer_agent_id.clone(),
                seller_agent_id: body.seller_agent_id.clone(),
                skill_id: body.skill_id.clone(),
                status: "pending".to_string(),
                amount: body.amount,
                created_at: now.clone(),
                updated_at: now,
            };
            HttpResponse::Ok().json(ApiResponse::success(transaction))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create transaction".to_string()))
        }
    }
}

pub async fn get_transaction(
    pool: web::Data<SqlitePool>,
    transaction_id: web::Path<String>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, String, String, f64, String, String)>(
        "SELECT id, buyer_agent_id, seller_agent_id, skill_id, status, amount, created_at, updated_at FROM transactions WHERE id = ?"
    )
    .bind(transaction_id.as_str())
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some((id, buyer_agent_id, seller_agent_id, skill_id, status, amount, created_at, updated_at))) => {
            let transaction = Transaction {
                id,
                buyer_agent_id,
                seller_agent_id,
                skill_id,
                status,
                amount,
                created_at,
                updated_at,
            };
            HttpResponse::Ok().json(ApiResponse::success(transaction))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Transaction not found".to_string()))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()))
        }
    }
}

pub async fn list_transactions(
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, String, String, f64, String, String)>(
        "SELECT id, buyer_agent_id, seller_agent_id, skill_id, status, amount, created_at, updated_at FROM transactions LIMIT 100"
    )
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

pub async fn update_transaction_status(
    pool: web::Data<SqlitePool>,
    req: ActixHttpRequest,
    transaction_id: web::Path<String>,
    status: web::Json<serde_json::Value>,
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

    let _claims = match verify_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Invalid token".to_string()));
        }
    };

    let new_status = status.get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("completed");

    let now = Utc::now().to_rfc3339();

    match sqlx::query(
        "UPDATE transactions SET status = ?, updated_at = ? WHERE id = ?"
    )
    .bind(new_status)
    .bind(&now)
    .bind(transaction_id.as_str())
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({"status": "updated"})))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update transaction".to_string()))
        }
    }
}
