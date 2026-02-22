use chrono::Local;
use serde_json::json;
use sqlx::{Row, SqlitePool};

/// Initialize toggle states from persistent storage
pub async fn load_toggle_states(db: &SqlitePool) -> Result<(bool, bool), String> {
    // Load internet_enabled state
    let internet_enabled = load_internet_enabled_state(db).await.unwrap_or(false);

    // Load training_enabled state
    let training_enabled = load_training_enabled_state(db).await.unwrap_or(false);

    Ok((internet_enabled, training_enabled))
}

/// Load internet toggle state from database
async fn load_internet_enabled_state(db: &SqlitePool) -> Result<bool, String> {
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind("toggle:internet_enabled")
        .fetch_optional(db)
        .await
    {
        let raw: Vec<u8> = row.get(0);
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&raw) {
            if let Some(enabled) = val.get("enabled").and_then(|v| v.as_bool()) {
                return Ok(enabled);
            }
        }
    }
    Ok(false) // Default to disabled
}

/// Load training toggle state from database
async fn load_training_enabled_state(db: &SqlitePool) -> Result<bool, String> {
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind("toggle:training_enabled")
        .fetch_optional(db)
        .await
    {
        let raw: Vec<u8> = row.get(0);
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&raw) {
            if let Some(enabled) = val.get("enabled").and_then(|v| v.as_bool()) {
                return Ok(enabled);
            }
        }
    }
    Ok(false) // Default to disabled
}

/// Save internet toggle state
pub async fn save_internet_toggle_state(db: &SqlitePool, enabled: bool) -> Result<(), String> {
    let payload = serde_json::to_vec(&json!({
        "enabled": enabled,
        "updated_at": Local::now().to_rfc3339(),
    }))
    .map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind("toggle:internet_enabled")
        .bind(payload)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Save training toggle state
pub async fn save_training_toggle_state(db: &SqlitePool, enabled: bool) -> Result<(), String> {
    let payload = serde_json::to_vec(&json!({
        "enabled": enabled,
        "updated_at": Local::now().to_rfc3339(),
    }))
    .map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind("toggle:training_enabled")
        .bind(payload)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get current toggle states
pub async fn get_toggle_states(db: &SqlitePool) -> Result<(bool, bool), String> {
    let internet = load_internet_enabled_state(db).await?;
    let training = load_training_enabled_state(db).await?;
    Ok((internet, training))
}
