use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;
use uuid::Uuid;
use bcrypt::{hash, verify};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

use crate::models::{User, RegisterRequest, LoginRequest, AuthResponse, ApiResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

const JWT_SECRET: &[u8] = b"your-secret-key-change-this-in-production";

pub async fn register(
    pool: web::Data<SqlitePool>,
    req: web::Json<RegisterRequest>,
) -> HttpResponse {
    let user_id = Uuid::new_v4().to_string();
    
    match hash(&req.password, 4) {
        Ok(password_hash) => {
            match sqlx::query(
                "INSERT INTO users (id, username, email, password_hash) VALUES (?, ?, ?, ?)"
            )
            .bind(&user_id)
            .bind(&req.username)
            .bind(&req.email)
            .bind(&password_hash)
            .execute(pool.get_ref())
            .await
            {
                Ok(_) => {
                    let user = User {
                        id: user_id,
                        username: req.username.clone(),
                        email: req.email.clone(),
                        password_hash,
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Utc::now().to_rfc3339(),
                    };

                    let claims = Claims {
                        sub: user.id.clone(),
                        exp: (Utc::now() + Duration::days(7)).timestamp(),
                    };

                    match encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET)) {
                        Ok(token) => {
                            HttpResponse::Ok().json(ApiResponse::success(AuthResponse {
                                token,
                                user,
                            }))
                        }
                        Err(_) => {
                            HttpResponse::InternalServerError()
                                .json(ApiResponse::<()>::error("Failed to generate token".to_string()))
                        }
                    }
                }
                Err(_) => {
                    HttpResponse::BadRequest()
                        .json(ApiResponse::<()>::error("User already exists".to_string()))
                }
            }
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to hash password".to_string()))
        }
    }
}

pub async fn login(
    pool: web::Data<SqlitePool>,
    req: web::Json<LoginRequest>,
) -> HttpResponse {
    match sqlx::query_as::<_, (String, String, String, String, String, String)>(
        "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some((id, username, email, password_hash, created_at, updated_at))) => {
            if verify(&req.password, &password_hash).unwrap_or(false) {
                let user = User {
                    id: id.clone(),
                    username,
                    email,
                    password_hash,
                    created_at,
                    updated_at,
                };

                let claims = Claims {
                    sub: user.id.clone(),
                    exp: (Utc::now() + Duration::days(7)).timestamp(),
                };

                match encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET)) {
                    Ok(token) => {
                        HttpResponse::Ok().json(ApiResponse::success(AuthResponse {
                            token,
                            user,
                        }))
                    }
                    Err(_) => {
                        HttpResponse::InternalServerError()
                            .json(ApiResponse::<()>::error("Failed to generate token".to_string()))
                    }
                }
            } else {
                HttpResponse::Unauthorized()
                    .json(ApiResponse::<()>::error("Invalid credentials".to_string()))
            }
        }
        Ok(None) => {
            HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("User not found".to_string()))
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string()))
        }
    }
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
