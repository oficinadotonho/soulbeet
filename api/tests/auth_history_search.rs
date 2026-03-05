#![cfg(feature = "server")]

mod common;

use api::auth;
use api::models::download_history::{DownloadHistoryRow, NewDownloadHistory};
use api::models::search_attempt_history::{NewSearchAttempt, SearchAttemptRow};
use api::models::user::User;
use shared::history::{
    DownloadHistoryStatus, HistoryKind, HistoryQuery, ImportHistoryStatus, SearchAttemptStatus,
    SearchAttemptType,
};

fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[tokio::test]
async fn auth_history_search_flow() {
    let _db_path = common::setup_test_env("auth_history_search");
    common::init_db().await;

    let user = User::create("integration_user", "password123")
        .await
        .expect("user should be created");
    let verified = User::verify("integration_user", "password123")
        .await
        .expect("password verification should pass");
    assert_eq!(user.id, verified.id);

    let token = auth::create_token(user.id.clone(), user.username.clone())
        .expect("token should be created");
    let claims = auth::verify_token(&token).expect("token should be verifiable");
    assert_eq!(claims.sub, user.id);
    assert_eq!(claims.username, user.username);

    let ts = now_utc();
    DownloadHistoryRow::insert(&NewDownloadHistory {
        id: "dh-1".to_string(),
        user_id: user.id.clone(),
        action_id: "action-1".to_string(),
        backend_id: "slskd".to_string(),
        queue_item_id: "q-1".to_string(),
        source: "slskd".to_string(),
        title: "Song A".to_string(),
        artist: "Artist A".to_string(),
        release_name: Some("Album A".to_string()),
        item_label: "Artist A - Song A".to_string(),
        size_bytes: 42,
        target_folder: "/downloads".to_string(),
        download_status: DownloadHistoryStatus::Completed,
        import_status: ImportHistoryStatus::Completed,
        error_message: None,
        needs_manual_action: false,
        downloadable_item_json: "{}".to_string(),
        tracks_json: Some("[]".to_string()),
        started_at: ts.clone(),
        downloaded_at: Some(ts.clone()),
        imported_at: Some(ts.clone()),
        ended_at: Some(ts.clone()),
        created_at: ts.clone(),
        updated_at: ts.clone(),
    })
    .await
    .expect("download history insert should succeed");

    let download_page = DownloadHistoryRow::query_for_user(
        &user.id,
        &HistoryQuery {
            kind: HistoryKind::DownloadAttempt,
            per_page: Some(20),
            page: Some(1),
            ..HistoryQuery::default()
        },
    )
    .await
    .expect("download history query should succeed");
    assert_eq!(download_page.total, 1);
    assert_eq!(download_page.entries.len(), 1);
    assert_eq!(
        download_page.entries[0].download_status,
        Some(DownloadHistoryStatus::Completed)
    );

    SearchAttemptRow::insert(&NewSearchAttempt {
        id: "sa-1".to_string(),
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
    .expect("search attempt insert should succeed");

    let search_page = SearchAttemptRow::query_for_user(
        &user.id,
        &HistoryQuery {
            kind: HistoryKind::SearchAttempt,
            per_page: Some(20),
            page: Some(1),
            ..HistoryQuery::default()
        },
    )
    .await
    .expect("search history query should succeed");
    assert_eq!(search_page.total, 1);
    assert_eq!(search_page.entries.len(), 1);
    assert_eq!(
        search_page.entries[0].search_attempt_status,
        Some(SearchAttemptStatus::Completed)
    );

    let deleted_downloads = DownloadHistoryRow::clear_for_user(&user.id)
        .await
        .expect("download clear should succeed");
    let deleted_searches = SearchAttemptRow::clear_for_user(&user.id)
        .await
        .expect("search clear should succeed");

    assert_eq!(deleted_downloads, 1);
    assert_eq!(deleted_searches, 1);
}
