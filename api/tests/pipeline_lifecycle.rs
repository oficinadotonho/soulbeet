#![cfg(feature = "server")]

mod common;

use api::models::download_history::{DownloadHistoryRow, DownloadHistoryUpdate, NewDownloadHistory};
use api::models::search_attempt_history::{NewSearchAttempt, SearchAttemptRow, SearchAttemptUpdate};
use api::models::user::User;
use shared::history::{
    DownloadHistoryStatus, ImportHistoryStatus, SearchAttemptStatus, SearchAttemptType,
};

fn ts(offset_secs: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::seconds(offset_secs)).to_rfc3339()
}

#[tokio::test]
async fn download_attempt_lifecycle_transitions_are_persisted() {
    let _db_path = common::setup_test_env("pipeline_download_lifecycle");
    common::init_db().await;

    let user = User::create("pipeline_dl_user", "pw123456")
        .await
        .expect("user should be created");

    let t0 = ts(0);
    DownloadHistoryRow::insert(&NewDownloadHistory {
        id: "dh-pipeline-1".to_string(),
        user_id: user.id.clone(),
        action_id: "action-pipeline-1".to_string(),
        backend_id: "slskd".to_string(),
        queue_item_id: "queue-pipeline-1".to_string(),
        source: "slskd".to_string(),
        title: "Track One".to_string(),
        artist: "Artist One".to_string(),
        release_name: Some("Release One".to_string()),
        item_label: "Artist One - Track One".to_string(),
        size_bytes: 123,
        target_folder: "/downloads".to_string(),
        download_status: DownloadHistoryStatus::Queued,
        import_status: ImportHistoryStatus::NotStarted,
        error_message: None,
        needs_manual_action: false,
        downloadable_item_json: "{}".to_string(),
        tracks_json: Some("[]".to_string()),
        started_at: t0.clone(),
        downloaded_at: None,
        imported_at: None,
        ended_at: None,
        created_at: t0.clone(),
        updated_at: t0,
    })
    .await
    .expect("initial insert should succeed");

    let t1 = ts(10);
    let updated = DownloadHistoryRow::update_by_queue_item(&DownloadHistoryUpdate {
        queue_item_id: "queue-pipeline-1".to_string(),
        user_id: user.id.clone(),
        download_status: Some(DownloadHistoryStatus::InProgress),
        import_status: None,
        error_message: None,
        needs_manual_action: None,
        downloaded_at: None,
        imported_at: None,
        ended_at: None,
        updated_at: t1,
    })
    .await
    .expect("in-progress update should succeed");
    assert!(updated);

    let downloaded_at = ts(20);
    let t2 = ts(21);
    DownloadHistoryRow::update_by_queue_item(&DownloadHistoryUpdate {
        queue_item_id: "queue-pipeline-1".to_string(),
        user_id: user.id.clone(),
        download_status: Some(DownloadHistoryStatus::Completed),
        import_status: None,
        error_message: Some(None),
        needs_manual_action: Some(false),
        downloaded_at: Some(Some(downloaded_at.clone())),
        imported_at: None,
        ended_at: None,
        updated_at: t2,
    })
    .await
    .expect("completed update should succeed");

    let imported_at = ts(30);
    let ended_at = ts(31);
    let t3 = ts(32);
    DownloadHistoryRow::update_by_queue_item(&DownloadHistoryUpdate {
        queue_item_id: "queue-pipeline-1".to_string(),
        user_id: user.id.clone(),
        download_status: None,
        import_status: Some(ImportHistoryStatus::Completed),
        error_message: Some(None),
        needs_manual_action: Some(false),
        downloaded_at: None,
        imported_at: Some(Some(imported_at.clone())),
        ended_at: Some(Some(ended_at.clone())),
        updated_at: t3,
    })
    .await
    .expect("import completed update should succeed");

    let row = sqlx::query_as::<_, DownloadHistoryRow>(
        "SELECT * FROM download_history WHERE user_id = ? AND queue_item_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&user.id)
    .bind("queue-pipeline-1")
    .fetch_one(&*api::db::DB)
    .await
    .expect("row should exist");

    assert_eq!(row.download_status, DownloadHistoryStatus::Completed.as_str());
    assert_eq!(row.import_status, ImportHistoryStatus::Completed.as_str());
    assert_eq!(row.downloaded_at.as_deref(), Some(downloaded_at.as_str()));
    assert_eq!(row.imported_at.as_deref(), Some(imported_at.as_str()));
    assert_eq!(row.ended_at.as_deref(), Some(ended_at.as_str()));
    assert!(!row.needs_manual_action);
    assert!(row.error_message.is_none());
}

#[tokio::test]
async fn search_attempt_updates_latest_row_for_same_search_id() {
    let _db_path = common::setup_test_env("pipeline_search_lifecycle");
    common::init_db().await;

    let user = User::create("pipeline_search_user", "pw123456")
        .await
        .expect("user should be created");

    let search_id = "search-pipeline-1";

    SearchAttemptRow::insert(&NewSearchAttempt {
        id: "sa-old".to_string(),
        user_id: user.id.clone(),
        attempt_type: SearchAttemptType::SourceDownload,
        provider_id: None,
        backend_id: Some("slskd".to_string()),
        search_id: Some(search_id.to_string()),
        query_text: Some("old query".to_string()),
        artist_text: Some("old artist".to_string()),
        status: SearchAttemptStatus::InProgress,
        result_count: 0,
        error_message: None,
        started_at: ts(-60),
        ended_at: None,
        created_at: ts(-60),
        updated_at: ts(-60),
    })
    .await
    .expect("old attempt insert should succeed");

    SearchAttemptRow::insert(&NewSearchAttempt {
        id: "sa-new".to_string(),
        user_id: user.id.clone(),
        attempt_type: SearchAttemptType::SourceDownload,
        provider_id: None,
        backend_id: Some("slskd".to_string()),
        search_id: Some(search_id.to_string()),
        query_text: Some("new query".to_string()),
        artist_text: Some("new artist".to_string()),
        status: SearchAttemptStatus::InProgress,
        result_count: 0,
        error_message: None,
        started_at: ts(-30),
        ended_at: None,
        created_at: ts(-30),
        updated_at: ts(-30),
    })
    .await
    .expect("new attempt insert should succeed");

    let terminal_time = ts(1);
    let affected = SearchAttemptRow::update_by_search_id(&SearchAttemptUpdate {
        user_id: user.id.clone(),
        search_id: search_id.to_string(),
        status: SearchAttemptStatus::TimedOut,
        result_count: Some(0),
        error_message: Some(Some("search timeout".to_string())),
        ended_at: Some(Some(terminal_time.clone())),
        updated_at: terminal_time.clone(),
    })
    .await
    .expect("update by search id should succeed");
    assert!(affected);

    let old_row = sqlx::query_as::<_, SearchAttemptRow>(
        "SELECT * FROM search_attempt_history WHERE id = 'sa-old'",
    )
    .fetch_one(&*api::db::DB)
    .await
    .expect("old row should exist");

    let new_row = sqlx::query_as::<_, SearchAttemptRow>(
        "SELECT * FROM search_attempt_history WHERE id = 'sa-new'",
    )
    .fetch_one(&*api::db::DB)
    .await
    .expect("new row should exist");

    assert_eq!(old_row.status, SearchAttemptStatus::InProgress.as_str());
    assert!(old_row.ended_at.is_none());

    assert_eq!(new_row.status, SearchAttemptStatus::TimedOut.as_str());
    assert_eq!(new_row.result_count, 0);
    assert_eq!(new_row.error_message.as_deref(), Some("search timeout"));
    assert_eq!(new_row.ended_at.as_deref(), Some(terminal_time.as_str()));
}
