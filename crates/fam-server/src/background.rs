use sqlx::PgPool;
use std::time::Duration;

/// Spawn all background tasks.
pub fn spawn_background_tasks(db: PgPool) {
    // Session cleanup: every hour, delete expired sessions
    let db_clone = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_expired_sessions(&db_clone).await {
                tracing::warn!(error = %e, "session cleanup failed");
            }
        }
    });

    // Stale host cleanup: every 24 hours, remove hosts not seen in 90 days
    let db_clone = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(86400));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_stale_hosts(&db_clone).await {
                tracing::warn!(error = %e, "stale host cleanup failed");
            }
        }
    });
}

async fn cleanup_expired_sessions(db: &PgPool) -> Result<(), sqlx::Error> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
        .execute(db)
        .await?;
    let count = result.rows_affected();
    if count > 0 {
        tracing::info!(count, "cleaned up expired sessions");
    }
    Ok(())
}

async fn cleanup_stale_hosts(db: &PgPool) -> Result<(), sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM hosts WHERE last_rpc_at < NOW() - INTERVAL '90 days'")
            .execute(db)
            .await?;
    let count = result.rows_affected();
    if count > 0 {
        tracing::info!(count, "cleaned up stale hosts");
    }
    Ok(())
}
