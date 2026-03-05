#![cfg(feature = "server")]

mod common;

use api::models::download_history::{DownloadHistoryRow, NewDownloadHistory};
use api::models::search_attempt_history::{NewSearchAttempt, SearchAttemptRow};
use api::models::user::User;
use shared::history::{DownloadHistoryStatus, ImportHistoryStatus, SearchAttemptStatus, SearchAttemptType};

fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[tokio::test]
async fn fk_and_cascade_regressions() {
    let _db_path = common::setup_test_env("history_fk_regressions");
    common::init_db().await;

    let missing_user_insert = SearchAttemptRow::insert(&NewSearchAttempt {
        id: "sa-missing-user".to_string(),
        user_id: "user-does-not-exist".to_string(),
        attempt_type: SearchAttemptType::MetadataAlbum,
        provider_id: Some("musicbrainz".to_string()),
        backend_id: None,
        search_id: None,
        query_text: Some("test query".to_string()),
        artist_text: None,
        status: SearchAttemptStatus::Failed,
        result_count: 0,
        error_message: Some("expected failure".to_string()),
        started_at: now_utc(),
        ended_at: Some(now_utc()),
        created_at: now_utc(),
        updated_at: now_utc(),
    })
    .await;

    assert!(
        missing_user_insert
            .expect_err("missing-user insert should fail")
            .to_lowercase()
            .contains("foreign key"),
        "error should mention foreign key constraint"
    );

    let user = User::create("cascade_user", "pw123456")
        .await
        .expect("user should be created");
    let ts = now_utc();

    DownloadHistoryRow::insert(&NewDownloadHistory {
        id: "dh-cascade".to_string(),
        user_id: user.id.clone(),
        action_id: "a-cascade".to_string(),
        backend_id: "slskd".to_string(),
        queue_item_id: "q-cascade".to_string(),
        source: "slskd".to_string(),
        title: "Song".to_string(),
        artist: "Artist".to_string(),
        release_name: Some("Release".to_string()),
        item_label: "Artist - Song".to_string(),
        size_bytes: 99,
        target_folder: "/downloads".to_string(),
        download_status: DownloadHistoryStatus::Failed,
        import_status: ImportHistoryStatus::NotStarted,
        error_message: Some("boom".to_string()),
        needs_manual_action: true,
        downloadable_item_json: "{}".to_string(),
        tracks_json: Some("[]".to_string()),
        started_at: ts.clone(),
        downloaded_at: None,
        imported_at: None,
        ended_at: Some(ts.clone()),
        created_at: ts.clone(),
        updated_at: ts.clone(),
    })
    .await
    .expect("download history insert should succeed");

    SearchAttemptRow::insert(&NewSearchAttempt {
        id: "sa-cascade".to_string(),
        user_id: user.id.clone(),
        attempt_type: SearchAttemptType::MetadataTrack,
        provider_id: Some("musicbrainz".to_string()),
        backend_id: None,
        search_id: None,
        query_text: Some("everlong".to_string()),
        artist_text: Some("foo fighters".to_string()),
        status: SearchAttemptStatus::Completed,
        result_count: 1,
        error_message: None,
        started_at: ts.clone(),
        ended_at: Some(ts.clone()),
        created_at: ts.clone(),
        updated_at: ts.clone(),
    })
    .await
    .expect("search history insert should succeed");

    assert_eq!(common::count_rows("download_history").await, 1);
    assert_eq!(common::count_rows("search_attempt_history").await, 1);

    User::delete(&user.id).await.expect("user delete should succeed");

    assert_eq!(common::count_rows("download_history").await, 0);
    assert_eq!(common::count_rows("search_attempt_history").await, 0);
}
