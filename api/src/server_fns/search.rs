use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use shared::{
    download::{DownloadQuery, SearchResult as DownloadSearchResult},
    metadata::{AlbumWithTracks, Provider, SearchResults},
};
#[cfg(feature = "server")]
use shared::download::SearchState as DownloadSearchState;
#[cfg(feature = "server")]
use shared::history::{SearchAttemptStatus, SearchAttemptType};

#[cfg(feature = "server")]
use dioxus::logger::tracing::warn;
#[cfg(feature = "server")]
use uuid::Uuid;
#[cfg(feature = "server")]
use crate::{server_fns::server_error, AuthSession};
#[cfg(feature = "server")]
use crate::models::search_attempt_history::{NewSearchAttempt, SearchAttemptRow, SearchAttemptUpdate};
#[cfg(feature = "server")]
use crate::services::{download_backend, downloaders, metadata_provider};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub artist: Option<String>,
    pub query: String,
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumQuery {
    pub id: String,
    #[serde(default)]
    pub provider: Option<Provider>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PollQuery {
    pub search_id: String,
    #[serde(default)]
    pub backend: Option<String>,
}

#[cfg(feature = "server")]
fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(feature = "server")]
async fn persist_search_attempt(new_attempt: NewSearchAttempt) {
    if let Err(error) = SearchAttemptRow::insert(&new_attempt).await {
        warn!("Failed to persist search attempt {}: {}", new_attempt.id, error);
    }
}

#[cfg(feature = "server")]
fn source_query_fields(data: &DownloadQuery) -> (Option<String>, Option<String>) {
    if let Some(album) = &data.album {
        return (Some(album.title.clone()), Some(album.artist.clone()));
    }

    data.tracks.first().map_or((None, None), |track| {
        (Some(track.title.clone()), Some(track.artist.clone()))
    })
}

#[post("/api/metadata/search/album", auth: AuthSession)]
pub async fn search_album(input: SearchQuery) -> Result<SearchResults, ServerFnError> {
    let user_id = auth.0.sub;
    let started_at = now_utc();
    let requested_provider = input.provider.clone();

    let provider = match metadata_provider(input.provider.as_deref()).await {
        Ok(provider) => provider,
        Err(error) => {
            let error_message = error.to_string();
            let now = now_utc();
            persist_search_attempt(NewSearchAttempt {
                id: Uuid::new_v4().to_string(),
                user_id,
                attempt_type: SearchAttemptType::MetadataAlbum,
                provider_id: requested_provider,
                backend_id: None,
                search_id: None,
                query_text: Some(input.query),
                artist_text: input.artist,
                status: SearchAttemptStatus::Failed,
                result_count: 0,
                error_message: Some(error_message.clone()),
                started_at,
                ended_at: Some(now.clone()),
                created_at: now.clone(),
                updated_at: now,
            })
            .await;
            return Err(server_error(error_message));
        }
    };

    let provider_id = Some(provider.id().to_string());
    let provider_enum: Provider = provider.id().parse().unwrap_or_default();
    let query_text = input.query.clone();
    let artist_text = input.artist.clone();

    let results = match provider
        .search_albums(input.artist.as_deref(), &input.query, 25)
        .await
    {
        Ok(results) => results,
        Err(error) => {
            let error_message = error.to_string();
            let now = now_utc();
            persist_search_attempt(NewSearchAttempt {
                id: Uuid::new_v4().to_string(),
                user_id,
                attempt_type: SearchAttemptType::MetadataAlbum,
                provider_id,
                backend_id: None,
                search_id: None,
                query_text: Some(query_text),
                artist_text,
                status: SearchAttemptStatus::Failed,
                result_count: 0,
                error_message: Some(error_message.clone()),
                started_at,
                ended_at: Some(now.clone()),
                created_at: now.clone(),
                updated_at: now,
            })
            .await;
            return Err(server_error(error_message));
        }
    };

    let now = now_utc();
    persist_search_attempt(NewSearchAttempt {
        id: Uuid::new_v4().to_string(),
        user_id,
        attempt_type: SearchAttemptType::MetadataAlbum,
        provider_id,
        backend_id: None,
        search_id: None,
        query_text: Some(query_text),
        artist_text,
        status: if results.is_empty() {
            SearchAttemptStatus::NoResults
        } else {
            SearchAttemptStatus::Completed
        },
        result_count: results.len() as i64,
        error_message: None,
        started_at,
        ended_at: Some(now.clone()),
        created_at: now.clone(),
        updated_at: now,
    })
    .await;

    Ok(SearchResults {
        provider: provider_enum,
        results,
    })
}

#[post("/api/metadata/search/track", auth: AuthSession)]
pub async fn search_track(input: SearchQuery) -> Result<SearchResults, ServerFnError> {
    let user_id = auth.0.sub;
    let started_at = now_utc();
    let requested_provider = input.provider.clone();

    let provider = match metadata_provider(input.provider.as_deref()).await {
        Ok(provider) => provider,
        Err(error) => {
            let error_message = error.to_string();
            let now = now_utc();
            persist_search_attempt(NewSearchAttempt {
                id: Uuid::new_v4().to_string(),
                user_id,
                attempt_type: SearchAttemptType::MetadataTrack,
                provider_id: requested_provider,
                backend_id: None,
                search_id: None,
                query_text: Some(input.query),
                artist_text: input.artist,
                status: SearchAttemptStatus::Failed,
                result_count: 0,
                error_message: Some(error_message.clone()),
                started_at,
                ended_at: Some(now.clone()),
                created_at: now.clone(),
                updated_at: now,
            })
            .await;
            return Err(server_error(error_message));
        }
    };

    let provider_id = Some(provider.id().to_string());
    let provider_enum: Provider = provider.id().parse().unwrap_or_default();
    let query_text = input.query.clone();
    let artist_text = input.artist.clone();

    let results = match provider
        .search_tracks(input.artist.as_deref(), &input.query, 25)
        .await
    {
        Ok(results) => results,
        Err(error) => {
            let error_message = error.to_string();
            let now = now_utc();
            persist_search_attempt(NewSearchAttempt {
                id: Uuid::new_v4().to_string(),
                user_id,
                attempt_type: SearchAttemptType::MetadataTrack,
                provider_id,
                backend_id: None,
                search_id: None,
                query_text: Some(query_text),
                artist_text,
                status: SearchAttemptStatus::Failed,
                result_count: 0,
                error_message: Some(error_message.clone()),
                started_at,
                ended_at: Some(now.clone()),
                created_at: now.clone(),
                updated_at: now,
            })
            .await;
            return Err(server_error(error_message));
        }
    };

    let now = now_utc();
    persist_search_attempt(NewSearchAttempt {
        id: Uuid::new_v4().to_string(),
        user_id,
        attempt_type: SearchAttemptType::MetadataTrack,
        provider_id,
        backend_id: None,
        search_id: None,
        query_text: Some(query_text),
        artist_text,
        status: if results.is_empty() {
            SearchAttemptStatus::NoResults
        } else {
            SearchAttemptStatus::Completed
        },
        result_count: results.len() as i64,
        error_message: None,
        started_at,
        ended_at: Some(now.clone()),
        created_at: now.clone(),
        updated_at: now,
    })
    .await;

    Ok(SearchResults {
        provider: provider_enum,
        results,
    })
}

#[post("/api/metadata/album", _: AuthSession)]
pub async fn find_album(input: AlbumQuery) -> Result<AlbumWithTracks, ServerFnError> {
    let provider_str = input.provider.map(|p| p.to_string());
    let provider = metadata_provider(provider_str.as_deref())
        .await
        .map_err(server_error)?;

    provider.get_album(&input.id).await.map_err(server_error)
}

#[post("/api/download/search/start", auth: AuthSession)]
pub async fn start_download_search(data: DownloadQuery) -> Result<String, ServerFnError> {
    let user_id = auth.0.sub;
    let backend_id = data
        .backend
        .clone()
        .unwrap_or_else(|| downloaders::SLSKD.to_string());
    let (query_text, artist_text) = source_query_fields(&data);
    let started_at = now_utc();

    let backend = download_backend(data.backend.as_deref())
        .await
        .map_err(|e| server_error(format!("download backend not available: {}", e)))?;

    let search_id = match backend
        .start_search(data.album.as_ref(), &data.tracks)
        .await
    {
        Ok(search_id) => search_id,
        Err(error) => {
            let error_message = error.to_string();
            let now = now_utc();
            persist_search_attempt(NewSearchAttempt {
                id: Uuid::new_v4().to_string(),
                user_id,
                attempt_type: SearchAttemptType::SourceDownload,
                provider_id: None,
                backend_id: Some(backend_id),
                search_id: None,
                query_text,
                artist_text,
                status: SearchAttemptStatus::Failed,
                result_count: 0,
                error_message: Some(error_message.clone()),
                started_at,
                ended_at: Some(now.clone()),
                created_at: now.clone(),
                updated_at: now,
            })
            .await;
            return Err(server_error(error_message));
        }
    };

    let now = now_utc();
    persist_search_attempt(NewSearchAttempt {
        id: Uuid::new_v4().to_string(),
        user_id,
        attempt_type: SearchAttemptType::SourceDownload,
        provider_id: None,
        backend_id: Some(backend_id),
        search_id: Some(search_id.clone()),
        query_text,
        artist_text,
        status: SearchAttemptStatus::InProgress,
        result_count: 0,
        error_message: None,
        started_at,
        ended_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
    .await;

    Ok(search_id)
}

#[post("/api/download/search/poll", auth: AuthSession)]
pub async fn poll_download_search(input: PollQuery) -> Result<DownloadSearchResult, ServerFnError> {
    let user_id = auth.0.sub;
    let backend = download_backend(input.backend.as_deref())
        .await
        .map_err(|e| server_error(format!("download backend not available: {}", e)))?;

    let response = match backend.poll_search(&input.search_id).await {
        Ok(response) => response,
        Err(error) => {
            let error_message = error.to_string();
            let now = now_utc();
            let _ = SearchAttemptRow::update_by_search_id(&SearchAttemptUpdate {
                user_id,
                search_id: input.search_id,
                status: SearchAttemptStatus::Failed,
                result_count: None,
                error_message: Some(Some(error_message.clone())),
                ended_at: Some(Some(now.clone())),
                updated_at: now,
            })
            .await;
            return Err(server_error(error_message));
        }
    };

    let status = match &response.state {
        DownloadSearchState::InProgress => SearchAttemptStatus::InProgress,
        DownloadSearchState::Completed => {
            if response.groups.is_empty() {
                SearchAttemptStatus::NoResults
            } else {
                SearchAttemptStatus::Completed
            }
        }
        DownloadSearchState::TimedOut => SearchAttemptStatus::TimedOut,
        DownloadSearchState::NotFound => SearchAttemptStatus::Failed,
    };

    let is_terminal = !matches!(status, SearchAttemptStatus::InProgress);
    let error_message = if matches!(&response.state, DownloadSearchState::NotFound) {
        Some(Some("search not found".to_string()))
    } else {
        None
    };
    let now = now_utc();
    let _ = SearchAttemptRow::update_by_search_id(&SearchAttemptUpdate {
        user_id,
        search_id: input.search_id,
        status,
        result_count: Some(response.groups.len() as i64),
        error_message,
        ended_at: if is_terminal {
            Some(Some(now.clone()))
        } else {
            None
        },
        updated_at: now,
    })
    .await;

    Ok(response)
}
