pub mod queries;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::error::Error;

pub async fn initialize_db(db_url: &str) -> Result<SqlitePool, Box<dyn Error + Send + Sync>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    // --- TABELLA TICKETS ---
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS tickets (
            channel_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL
        )"
    ).execute(&pool).await?;

    // --- TABELLA CANDIDATURE ---
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS applications (
            user_id TEXT PRIMARY KEY,
            staff_msg_id TEXT NOT NULL,
            status TEXT DEFAULT 'PENDING'
        )"
    ).execute(&pool).await?;

    // --- TABELLA DOSSIER (Usa il plurale 'dossiers' come nelle tue queries) ---
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS dossiers (
            user_id TEXT PRIMARY KEY,
            reason TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&pool).await?;

    // --- TABELLA DASHBOARD WEB ---
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS web_configs (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )"
    ).execute(&pool).await?;

    println!("🗄️ Database RGA: Tutti i settori sono inizializzati e sincronizzati.");
    Ok(pool)
}
