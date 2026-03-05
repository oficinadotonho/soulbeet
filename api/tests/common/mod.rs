#[cfg(feature = "server")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "server")]
pub fn setup_test_env(test_name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid monotonic clock")
        .as_nanos();
    let db_path = format!("/tmp/soulbeet_{test_name}_{nanos}.db");
    let db_url = format!("sqlite:{db_path}");

    std::env::set_var("DATABASE_URL", &db_url);
    std::env::set_var("SECRET_KEY", "test-secret-key");
    std::env::set_var("DOWNLOAD_PATH", "/tmp");
    std::env::set_var("BEETS_CONFIG", "beets_config.yaml");
    std::env::set_var("BEETS_ALBUM_MODE", "true");

    db_path
}

#[cfg(feature = "server")]
pub async fn init_db() {
    let _ = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&*api::db::DB)
        .await
        .expect("db should initialize");
}

#[cfg(feature = "server")]
pub async fn count_rows(table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    sqlx::query_scalar::<_, i64>(&sql)
        .fetch_one(&*api::db::DB)
        .await
        .expect("count query should succeed")
}
