use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

pub async fn initialize_db(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Configura la connessione: crea il file .db se non esiste
    let connection_options = SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(connection_options).await?;

    // Eseguiamo la creazione delle tabelle all'avvio
    setup_tables(&pool).await?;

    Ok(pool)
}

async fn setup_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Tabella per i Ticket
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tickets (
            channel_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            status TEXT DEFAULT 'OPEN',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).execute(pool).await?;

    // Tabella per le Candidature (Applications)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS applications (
            user_id TEXT PRIMARY KEY,
            staff_msg_id TEXT NOT NULL,
            status TEXT DEFAULT 'PENDING',
            submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).execute(pool).await?;

    // --- TABELLA PER I DOSSIER ---
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dossiers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            reason TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).execute(pool).await?;

    // --- NUOVA TABELLA PER LA DASHBOARD (VERCEL) ---
    // Memorizza i titoli e le descrizioni dei pannelli modificati via Web
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS web_configs (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );"
    ).execute(pool).await?;

    Ok(())
}
