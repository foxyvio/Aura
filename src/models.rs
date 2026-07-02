use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub capabilities: Option<String>,
    pub reputation_score: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub price: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub buyer_agent_id: String,
    pub seller_agent_id: String,
    pub skill_id: String,
    pub status: String,
    pub amount: f64,
    pub created_at: String,
    pub updated_at: String,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub capabilities: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSkillRequest {
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub price: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub buyer_agent_id: String,
    pub seller_agent_id: String,
    pub skill_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}
