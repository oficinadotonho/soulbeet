use serde::{Deserialize, Serialize};
use shared::history::{DownloadHistoryStatus, HistoryEntry, HistoryKind, ImportHistoryStatus};

#[cfg(feature = "server")]
use shared::history::{HistoryQuery, PaginatedHistory};

#[cfg(feature = "server")]
use crate::db::DB;
#[cfg(feature = "server")]
use sqlx::{QueryBuilder, Sqlite};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct DownloadHistoryRow {
    pub id: String,
    pub user_id: String,
    pub action_id: String,
    pub backend_id: String,
    pub queue_item_id: String,
    pub source: String,
    pub title: String,
    pub artist: String,
    pub release_name: Option<String>,
    pub item_label: String,
    pub size_bytes: i64,
    pub target_folder: String,
    pub download_status: String,
    pub import_status: String,
    pub error_message: Option<String>,
    pub needs_manual_action: bool,
    pub downloadable_item_json: String,
    pub tracks_json: Option<String>,
    pub started_at: String,
    pub downloaded_at: Option<String>,
    pub imported_at: Option<String>,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDownloadHistory {
    pub id: String,
    pub user_id: String,
    pub action_id: String,
    pub backend_id: String,
    pub queue_item_id: String,
    pub source: String,
    pub title: String,
    pub artist: String,
    pub release_name: Option<String>,
    pub item_label: String,
    pub size_bytes: i64,
    pub target_folder: String,
    pub download_status: DownloadHistoryStatus,
    pub import_status: ImportHistoryStatus,
    pub error_message: Option<String>,
    pub needs_manual_action: bool,
    pub downloadable_item_json: String,
    pub tracks_json: Option<String>,
    pub started_at: String,
    pub downloaded_at: Option<String>,
    pub imported_at: Option<String>,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DownloadHistoryUpdate {
    pub queue_item_id: String,
    pub user_id: String,
    pub download_status: Option<DownloadHistoryStatus>,
    pub import_status: Option<ImportHistoryStatus>,
    pub error_message: Option<Option<String>>,
    pub needs_manual_action: Option<bool>,
    pub downloaded_at: Option<Option<String>>,
    pub imported_at: Option<Option<String>>,
    pub ended_at: Option<Option<String>>,
    pub updated_at: String,
}

impl TryFrom<DownloadHistoryRow> for HistoryEntry {
    type Error = String;

    fn try_from(row: DownloadHistoryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            kind: HistoryKind::DownloadAttempt,
            action_id: Some(row.action_id),
            title: Some(row.title),
            artist: Some(row.artist),
            release: row.release_name,
            item_label: Some(row.item_label),
            download_status: Some(row.download_status.parse::<DownloadHistoryStatus>()?),
            import_status: Some(row.import_status.parse::<ImportHistoryStatus>()?),
            search_attempt_status: None,
            search_attempt_type: None,
            provider_id: None,
            backend_id: Some(row.backend_id),
            search_id: None,
            result_count: None,
            error_message: row.error_message,
            started_at: row.started_at,
            ended_at: row.ended_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[cfg(feature = "server")]
impl DownloadHistoryRow {
    pub async fn insert(new: &NewDownloadHistory) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO download_history (
                id, user_id, action_id, backend_id, queue_item_id, source,
                title, artist, release_name, item_label, size_bytes, target_folder,
                download_status, import_status, error_message, needs_manual_action,
                downloadable_item_json, tracks_json,
                started_at, downloaded_at, imported_at, ended_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&new.id)
        .bind(&new.user_id)
        .bind(&new.action_id)
        .bind(&new.backend_id)
        .bind(&new.queue_item_id)
        .bind(&new.source)
        .bind(&new.title)
        .bind(&new.artist)
        .bind(&new.release_name)
        .bind(&new.item_label)
        .bind(new.size_bytes)
        .bind(&new.target_folder)
        .bind(new.download_status.as_str())
        .bind(new.import_status.as_str())
        .bind(&new.error_message)
        .bind(new.needs_manual_action)
        .bind(&new.downloadable_item_json)
        .bind(&new.tracks_json)
        .bind(&new.started_at)
        .bind(&new.downloaded_at)
        .bind(&new.imported_at)
        .bind(&new.ended_at)
        .bind(&new.created_at)
        .bind(&new.updated_at)
        .execute(&*DB)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn update_by_queue_item(update: &DownloadHistoryUpdate) -> Result<bool, String> {
        let id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT id
            FROM download_history
            WHERE user_id = ? AND queue_item_id = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(&update.user_id)
        .bind(&update.queue_item_id)
        .fetch_optional(&*DB)
        .await
        .map_err(|e| e.to_string())?;

        let Some(id) = id else {
            return Ok(false);
        };

        let mut qb = QueryBuilder::<Sqlite>::new("UPDATE download_history SET ");
        let mut wrote = false;

        let mut push_sep = |qb: &mut QueryBuilder<'_, Sqlite>| {
            if wrote {
                qb.push(", ");
            } else {
                wrote = true;
            }
        };

        if let Some(download_status) = update.download_status {
            push_sep(&mut qb);
            qb.push("download_status = ")
                .push_bind(download_status.as_str());
        }
        if let Some(import_status) = update.import_status {
            push_sep(&mut qb);
            qb.push("import_status = ").push_bind(import_status.as_str());
        }
        if let Some(error_message) = &update.error_message {
            push_sep(&mut qb);
            qb.push("error_message = ").push_bind(error_message);
        }
        if let Some(needs_manual_action) = update.needs_manual_action {
            push_sep(&mut qb);
            qb.push("needs_manual_action = ").push_bind(needs_manual_action);
        }
        if let Some(downloaded_at) = &update.downloaded_at {
            push_sep(&mut qb);
            qb.push("downloaded_at = ").push_bind(downloaded_at);
        }
        if let Some(imported_at) = &update.imported_at {
            push_sep(&mut qb);
            qb.push("imported_at = ").push_bind(imported_at);
        }
        if let Some(ended_at) = &update.ended_at {
            push_sep(&mut qb);
            qb.push("ended_at = ").push_bind(ended_at);
        }

        push_sep(&mut qb);
        qb.push("updated_at = ").push_bind(&update.updated_at);

        qb.push(" WHERE id = ").push_bind(id).push(" AND user_id = ").push_bind(&update.user_id);

        let affected = qb
            .build()
            .execute(&*DB)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn get_by_id_for_user(id: &str, user_id: &str) -> Result<Option<HistoryEntry>, String> {
        let row = sqlx::query_as::<_, DownloadHistoryRow>(
            "SELECT * FROM download_history WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&*DB)
        .await
        .map_err(|e| e.to_string())?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn delete_for_user(id: &str, user_id: &str) -> Result<bool, String> {
        let affected = sqlx::query("DELETE FROM download_history WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&*DB)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn clear_for_user(user_id: &str) -> Result<u64, String> {
        let affected = sqlx::query("DELETE FROM download_history WHERE user_id = ?")
            .bind(user_id)
            .execute(&*DB)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();

        Ok(affected)
    }

    pub async fn query_for_user(user_id: &str, query: &HistoryQuery) -> Result<PaginatedHistory, String> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
        let offset = ((page - 1) * per_page) as i64;

        let mut count_qb = QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM download_history WHERE user_id = ");
        count_qb.push_bind(user_id);
        apply_filters(&mut count_qb, query);

        let total: i64 = count_qb
            .build_query_scalar()
            .fetch_one(&*DB)
            .await
            .map_err(|e| e.to_string())?;

        let mut list_qb = QueryBuilder::<Sqlite>::new("SELECT * FROM download_history WHERE user_id = ");
        list_qb.push_bind(user_id);
        apply_filters(&mut list_qb, query);
        list_qb
            .push(" ORDER BY started_at DESC, id DESC LIMIT ")
            .push_bind(per_page as i64)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows = list_qb
            .build_query_as::<DownloadHistoryRow>()
            .fetch_all(&*DB)
            .await
            .map_err(|e| e.to_string())?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            entries.push(row.try_into()?);
        }

        let total_pages = if total == 0 {
            0
        } else {
            ((total as f64) / (per_page as f64)).ceil() as i32
        };

        Ok(PaginatedHistory {
            entries,
            total,
            page,
            per_page,
            total_pages,
        })
    }
}

#[cfg(feature = "server")]
fn apply_filters(qb: &mut QueryBuilder<'_, Sqlite>, query: &HistoryQuery) {
    if let Some(search) = query.search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let escaped = escape_like(search.to_lowercase().as_str());
        let like = format!("%{}%", escaped);
        qb.push(" AND (")
            .push("LOWER(title) LIKE ")
            .push_bind(like.clone())
            .push(" ESCAPE '\\\\' OR LOWER(artist) LIKE ")
            .push_bind(like.clone())
            .push(" ESCAPE '\\\\' OR LOWER(COALESCE(release_name, '')) LIKE ")
            .push_bind(like)
            .push(" ESCAPE '\\\\')");
    }

    if let Some(download_status) = query.download_status {
        qb.push(" AND download_status = ")
            .push_bind(download_status.as_str());
    }

    if let Some(import_status) = query.import_status {
        qb.push(" AND import_status = ").push_bind(import_status.as_str());
    }

    if let Some(from) = query
        .date_from_utc
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        qb.push(" AND started_at >= ").push_bind(from.to_string());
    }

    if let Some(to) = query
        .date_to_utc
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        qb.push(" AND started_at <= ").push_bind(to.to_string());
    }
}

#[cfg(feature = "server")]
fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
