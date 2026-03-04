use dioxus::prelude::*;

pub use shared::history::{ClearHistoryResponse, HistoryEntry, HistoryIdRequest, HistoryQuery, PaginatedHistory};

#[cfg(feature = "server")]
use crate::{
    models::{download_history::DownloadHistoryRow, search_attempt_history::SearchAttemptRow},
    server_fns::server_error,
    AuthSession,
};
#[cfg(feature = "server")]
use shared::history::HistoryKind;

/// Query current user's history with filters + pagination.
#[post("/api/history/query", auth: AuthSession)]
pub async fn query_history(query: HistoryQuery) -> Result<PaginatedHistory, ServerFnError> {
    let user_id = auth.0.sub;
    let result = match query.kind {
        HistoryKind::DownloadAttempt => DownloadHistoryRow::query_for_user(&user_id, &query).await,
        HistoryKind::SearchAttempt => SearchAttemptRow::query_for_user(&user_id, &query).await,
    };

    result.map_err(server_error)
}

/// Get a single history item.
#[post("/api/history/get", auth: AuthSession)]
pub async fn get_history(input: HistoryIdRequest) -> Result<Option<HistoryEntry>, ServerFnError> {
    let user_id = auth.0.sub;
    let result = match input.kind {
        HistoryKind::DownloadAttempt => DownloadHistoryRow::get_by_id_for_user(&input.id, &user_id).await,
        HistoryKind::SearchAttempt => SearchAttemptRow::get_by_id_for_user(&input.id, &user_id).await,
    };

    result.map_err(server_error)
}

/// Hard-delete one history item belonging to the current user.
#[post("/api/history/delete", auth: AuthSession)]
pub async fn delete_history(input: HistoryIdRequest) -> Result<bool, ServerFnError> {
    let user_id = auth.0.sub;
    let result = match input.kind {
        HistoryKind::DownloadAttempt => DownloadHistoryRow::delete_for_user(&input.id, &user_id).await,
        HistoryKind::SearchAttempt => SearchAttemptRow::delete_for_user(&input.id, &user_id).await,
    };

    result.map_err(server_error)
}

/// Hard-delete all history rows for the current user.
#[post("/api/history/clear", auth: AuthSession)]
pub async fn clear_history() -> Result<ClearHistoryResponse, ServerFnError> {
    let user_id = auth.0.sub;

    let deleted_downloads = DownloadHistoryRow::clear_for_user(&user_id)
        .await
        .map_err(server_error)?;
    let deleted_searches = SearchAttemptRow::clear_for_user(&user_id)
        .await
        .map_err(server_error)?;

    Ok(ClearHistoryResponse {
        deleted: deleted_downloads + deleted_searches,
    })
}
