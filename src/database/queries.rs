use sqlx::{SqlitePool, Error};

// --- LOGICA TICKETS ---

pub async fn create_ticket(pool: &SqlitePool, channel_id: &str, user_id: &str) -> Result<(), Error> {
    sqlx::query("INSERT OR REPLACE INTO tickets (channel_id, user_id) VALUES (?, ?)")
        .bind(channel_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_ticket_user(pool: &SqlitePool, channel_id: &str) -> Result<Option<String>, Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT user_id FROM tickets WHERE channel_id = ?")
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn delete_ticket(pool: &SqlitePool, channel_id: &str) -> Result<(), Error> {
    sqlx::query("DELETE FROM tickets WHERE channel_id = ?")
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- LOGICA CANDIDATURE ---

pub async fn has_pending_application(pool: &SqlitePool, user_id: &str) -> Result<bool, Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT status FROM applications WHERE user_id = ? AND status = 'PENDING'")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

pub async fn save_application(pool: &SqlitePool, user_id: &str, msg_id: &str) -> Result<(), Error> {
    sqlx::query("INSERT OR REPLACE INTO applications (user_id, staff_msg_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(msg_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_app_status(pool: &SqlitePool, user_id: &str, status: &str) -> Result<(), Error> {
    sqlx::query("UPDATE applications SET status = ? WHERE user_id = ?")
        .bind(status)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- LOGICA DOSSIER ---

pub async fn save_dossier(pool: &SqlitePool, user_id: &str, reason: &str) -> Result<(), Error> {
    sqlx::query("INSERT OR REPLACE INTO dossiers (user_id, reason) VALUES (?, ?)")
        .bind(user_id)
        .bind(reason)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_all_dossiers(pool: &SqlitePool) -> Result<Vec<(String, String, String)>, Error> {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT user_id, reason, created_at FROM dossiers ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// --- LOGICA DASHBOARD WEB (Nuova Sezione) ---

/// Salva le configurazioni inviate dalla dashboard Vercel
pub async fn save_web_config(pool: &SqlitePool, key: &str, value: &str) -> Result<(), Error> {
    sqlx::query("INSERT OR REPLACE INTO web_configs (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}

/// Recupera una configurazione specifica (es. titolo ticket o descrizione)
pub async fn get_web_config(pool: &SqlitePool, key: &str) -> Result<Option<String>, Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM web_configs WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}
