use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;

pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            capabilities TEXT,
            reputation_score REAL NOT NULL DEFAULT 0.0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            category TEXT NOT NULL,
            price REAL NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY,
            buyer_agent_id TEXT NOT NULL,
            seller_agent_id TEXT NOT NULL,
            skill_id TEXT NOT NULL,
            status TEXT NOT NULL,
            amount REAL NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (buyer_agent_id) REFERENCES agents(id),
            FOREIGN KEY (seller_agent_id) REFERENCES agents(id),
            FOREIGN KEY (skill_id) REFERENCES skills(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
