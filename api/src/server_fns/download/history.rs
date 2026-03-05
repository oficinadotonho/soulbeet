#[cfg(feature = "server")]
use dioxus::logger::tracing::{info, warn};
#[cfg(feature = "server")]
use shared::download::{DownloadProgress, DownloadState, DownloadableItem, QueuedDownload};
#[cfg(feature = "server")]
use shared::history::{DownloadHistoryStatus, ImportHistoryStatus};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use crate::models::download_history::{DownloadHistoryRow, DownloadHistoryUpdate, NewDownloadHistory};

#[cfg(feature = "server")]
fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(feature = "server")]
fn item_matches(queue: &QueuedDownload, item: &DownloadableItem) -> bool {
    queue.id == item.id || queue.item == item.id || queue.item == item.title
}

#[cfg(feature = "server")]
fn fallback_item(queue: &QueuedDownload) -> DownloadableItem {
    DownloadableItem {
        id: queue.item.clone(),
        source: queue.source.clone(),
        title: queue.item.clone(),
        artist: "Unknown Artist".to_string(),
        album: String::new(),
        size: Some(queue.size),
        duration: None,
        quality: "Unknown".to_string(),
        quality_score: 0.0,
        backend_data: None,
    }
}

#[cfg(feature = "server")]
pub async fn persist_queued_attempts(
    user_id: &str,
    action_id: &str,
    backend_id: &str,
    target_folder: &str,
    request_items: &[DownloadableItem],
    queued: &[QueuedDownload],
) {
    for queue in queued {
        let now = now_utc();
        let item = request_items
            .iter()
            .find(|candidate| item_matches(queue, candidate))
            .cloned()
            .unwrap_or_else(|| fallback_item(queue));
        let status = if queue.error.is_some() {
            DownloadHistoryStatus::Failed
        } else {
            DownloadHistoryStatus::Queued
        };
        let release_name = if item.album.trim().is_empty() {
            None
        } else {
            Some(item.album.clone())
        };
        let downloadable_item_json = serde_json::to_string(&item).unwrap_or_else(|_| "{}".into());

        let row = NewDownloadHistory {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            action_id: action_id.to_string(),
            backend_id: backend_id.to_string(),
            queue_item_id: queue.item.clone(),
            source: item.source,
            title: item.title,
            artist: item.artist,
            release_name,
            item_label: queue.item.clone(),
            size_bytes: queue.size as i64,
            target_folder: target_folder.to_string(),
            download_status: status,
            import_status: ImportHistoryStatus::NotStarted,
            error_message: queue.error.clone(),
            needs_manual_action: queue.error.is_some(),
            downloadable_item_json,
            tracks_json: None,
            started_at: now.clone(),
            downloaded_at: None,
            imported_at: None,
            ended_at: queue.error.as_ref().map(|_| now.clone()),
            created_at: now.clone(),
            updated_at: now,
        };

        if let Err(error) = DownloadHistoryRow::insert(&row).await {
            warn!(
                "Failed to persist queued history entry for user {} queue item {}: {}",
                user_id, queue.item, error
            );
        }
    }
}

#[cfg(feature = "server")]
async fn persist_update(user_id: &str, queue_item_id: &str, update: DownloadHistoryUpdate) {
    match DownloadHistoryRow::update_by_queue_item(&update).await {
        Ok(true) => {}
        Ok(false) => {
            info!(
                "No download history row found for user {} queue item {}",
                user_id, queue_item_id
            );
        }
        Err(error) => {
            warn!(
                "Failed updating download history for user {} queue item {}: {}",
                user_id, queue_item_id, error
            );
        }
    }
}

#[cfg(feature = "server")]
pub async fn persist_download_state(user_id: &str, progress: &DownloadProgress) {
    let now = now_utc();
    let queue_item_id = progress.item.clone();

    let update = match &progress.state {
        DownloadState::Queued => DownloadHistoryUpdate {
            queue_item_id: queue_item_id.clone(),
            user_id: user_id.to_string(),
            download_status: Some(DownloadHistoryStatus::Queued),
            import_status: None,
            error_message: None,
            needs_manual_action: None,
            downloaded_at: None,
            imported_at: None,
            ended_at: None,
            updated_at: now,
        },
        DownloadState::InProgress => DownloadHistoryUpdate {
            queue_item_id: queue_item_id.clone(),
            user_id: user_id.to_string(),
            download_status: Some(DownloadHistoryStatus::InProgress),
            import_status: None,
            error_message: None,
            needs_manual_action: None,
            downloaded_at: None,
            imported_at: None,
            ended_at: None,
            updated_at: now,
        },
        DownloadState::Completed => DownloadHistoryUpdate {
            queue_item_id: queue_item_id.clone(),
            user_id: user_id.to_string(),
            download_status: Some(DownloadHistoryStatus::Completed),
            import_status: None,
            error_message: Some(None),
            needs_manual_action: Some(false),
            downloaded_at: Some(Some(now.clone())),
            imported_at: None,
            ended_at: None,
            updated_at: now,
        },
        DownloadState::Failed(error) => DownloadHistoryUpdate {
            queue_item_id: queue_item_id.clone(),
            user_id: user_id.to_string(),
            download_status: Some(DownloadHistoryStatus::Failed),
            import_status: None,
            error_message: Some(Some(error.clone())),
            needs_manual_action: Some(true),
            downloaded_at: None,
            imported_at: None,
            ended_at: Some(Some(now.clone())),
            updated_at: now,
        },
        DownloadState::Cancelled => DownloadHistoryUpdate {
            queue_item_id: queue_item_id.clone(),
            user_id: user_id.to_string(),
            download_status: Some(DownloadHistoryStatus::Cancelled),
            import_status: None,
            error_message: None,
            needs_manual_action: Some(false),
            downloaded_at: None,
            imported_at: None,
            ended_at: Some(Some(now.clone())),
            updated_at: now,
        },
        DownloadState::Importing | DownloadState::Imported | DownloadState::ImportSkipped => {
            return;
        }
    };

    persist_update(user_id, &queue_item_id, update).await;
}

#[cfg(feature = "server")]
pub async fn persist_download_timeout(user_id: &str, queue_item_id: &str, error_message: &str) {
    let now = now_utc();
    let update = DownloadHistoryUpdate {
        queue_item_id: queue_item_id.to_string(),
        user_id: user_id.to_string(),
        download_status: Some(DownloadHistoryStatus::Timeout),
        import_status: None,
        error_message: Some(Some(error_message.to_string())),
        needs_manual_action: Some(true),
        downloaded_at: None,
        imported_at: None,
        ended_at: Some(Some(now.clone())),
        updated_at: now,
    };

    persist_update(user_id, queue_item_id, update).await;
}

#[cfg(feature = "server")]
pub async fn persist_import_state(
    user_id: &str,
    queue_item_id: &str,
    import_status: ImportHistoryStatus,
    error_message: Option<String>,
    needs_manual_action: bool,
    imported_at: Option<String>,
    ended_at: Option<String>,
) {
    let now = now_utc();
    let update = DownloadHistoryUpdate {
        queue_item_id: queue_item_id.to_string(),
        user_id: user_id.to_string(),
        download_status: None,
        import_status: Some(import_status),
        error_message: Some(error_message),
        needs_manual_action: Some(needs_manual_action),
        downloaded_at: None,
        imported_at: Some(imported_at),
        ended_at: Some(ended_at),
        updated_at: now,
    };

    persist_update(user_id, queue_item_id, update).await;
}
