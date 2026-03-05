use serde::{Deserialize, Serialize};
use shared::history::{HistoryEntry, HistoryKind, SearchAttemptStatus, SearchAttemptType};

#[cfg(feature = "server")]
use shared::history::{HistoryQuery, PaginatedHistory};

#[cfg(feature = "server")]
use crate::db::DB;
#[cfg(feature = "server")]
use sqlx::{QueryBuilder, Sqlite};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct SearchAttemptRow {
    pub id: String,
    pub user_id: String,
    pub attempt_type: String,
    pub provider_id: Option<String>,
    pub backend_id: Option<String>,
    pub search_id: Option<String>,
    pub query_text: Option<String>,
    pub artist_text: Option<String>,
    pub status: String,
    pub result_count: i64,
    pub error_message: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSearchAttempt {
    pub id: String,
    pub user_id: String,
    pub attempt_type: SearchAttemptType,
    pub provider_id: Option<String>,
    pub backend_id: Option<String>,
    pub search_id: Option<String>,
    pub query_text: Option<String>,
    pub artist_text: Option<String>,
    pub status: SearchAttemptStatus,
    pub result_count: i64,
    pub error_message: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchAttemptUpdate {
    pub user_id: String,
    pub search_id: String,
    pub status: SearchAttemptStatus,
    pub result_count: Option<i64>,
    pub error_message: Option<Option<String>>,
    pub ended_at: Option<Option<String>>,
    pub updated_at: String,
}

impl TryFrom<SearchAttemptRow> for HistoryEntry {
    type Error = String;

    fn try_from(row: SearchAttemptRow) -> Result<Self, Self::Error> {
        let title = row.query_text.clone().filter(|q| !q.is_empty());

        Ok(Self {
            id: row.id,
            kind: HistoryKind::SearchAttempt,
            action_id: None,
            title,
            artist: row.artist_text,
            release: None,
            item_label: None,
            download_status: None,
            import_status: None,
            search_attempt_status: Some(row.status.parse::<SearchAttemptStatus>()?),
            search_attempt_type: Some(row.attempt_type.parse::<SearchAttemptType>()?),
            provider_id: row.provider_id,
            backend_id: row.backend_id,
            search_id: row.search_id,
            result_count: Some(row.result_count),
            error_message: row.error_message,
            started_at: row.started_at,
            ended_at: row.ended_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[cfg(feature = "server")]
impl SearchAttemptRow {
    pub async fn insert(new: &NewSearchAttempt) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO search_attempt_history (
                id, user_id, attempt_type, provider_id, backend_id, search_id,
                query_text, artist_text,
                status, result_count, error_message,
                started_at, ended_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&new.id)
        .bind(&new.user_id)
        .bind(new.attempt_type.as_str())
        .bind(&new.provider_id)
        .bind(&new.backend_id)
        .bind(&new.search_id)
        .bind(&new.query_text)
        .bind(&new.artist_text)
        .bind(new.status.as_str())
        .bind(new.result_count)
        .bind(&new.error_message)
        .bind(&new.started_at)
        .bind(&new.ended_at)
        .bind(&new.created_at)
        .bind(&new.updated_at)
        .execute(&*DB)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn update_by_search_id(update: &SearchAttemptUpdate) -> Result<bool, String> {
        let id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT id
            FROM search_attempt_history
            WHERE user_id = ? AND search_id = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(&update.user_id)
        .bind(&update.search_id)
        .fetch_optional(&*DB)
        .await
        .map_err(|e| e.to_string())?;

        let Some(id) = id else {
            return Ok(false);
        };

        let mut qb = QueryBuilder::<Sqlite>::new("UPDATE search_attempt_history SET ");
        let mut wrote = false;

        let mut push_sep = |qb: &mut QueryBuilder<'_, Sqlite>| {
            if wrote {
                qb.push(", ");
            } else {
                wrote = true;
            }
        };

        push_sep(&mut qb);
        qb.push("status = ").push_bind(update.status.as_str());

        if let Some(result_count) = update.result_count {
            push_sep(&mut qb);
            qb.push("result_count = ").push_bind(result_count);
        }
        if let Some(error_message) = &update.error_message {
            push_sep(&mut qb);
            qb.push("error_message = ").push_bind(error_message);
        }
        if let Some(ended_at) = &update.ended_at {
            push_sep(&mut qb);
            qb.push("ended_at = ").push_bind(ended_at);
        }
        push_sep(&mut qb);
        qb.push("updated_at = ").push_bind(&update.updated_at);

        qb.push(" WHERE id = ")
            .push_bind(id)
            .push(" AND user_id = ")
            .push_bind(&update.user_id);

        let affected = qb
            .build()
            .execute(&*DB)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn get_by_id_for_user(id: &str, user_id: &str) -> Result<Option<HistoryEntry>, String> {
        let row = sqlx::query_as::<_, SearchAttemptRow>(
            "SELECT * FROM search_attempt_history WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&*DB)
        .await
        .map_err(|e| e.to_string())?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn delete_for_user(id: &str, user_id: &str) -> Result<bool, String> {
        let affected = sqlx::query("DELETE FROM search_attempt_history WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&*DB)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn clear_for_user(user_id: &str) -> Result<u64, String> {
        let affected = sqlx::query("DELETE FROM search_attempt_history WHERE user_id = ?")
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

        let mut count_qb = QueryBuilder::<Sqlite>::new(
            "SELECT COUNT(*) FROM search_attempt_history WHERE user_id = ",
        );
        count_qb.push_bind(user_id);
        apply_filters(&mut count_qb, query);

        let total: i64 = count_qb
            .build_query_scalar()
            .fetch_one(&*DB)
            .await
            .map_err(|e| e.to_string())?;

        let mut list_qb = QueryBuilder::<Sqlite>::new(
            "SELECT * FROM search_attempt_history WHERE user_id = ",
        );
        list_qb.push_bind(user_id);
        apply_filters(&mut list_qb, query);
        list_qb
            .push(" ORDER BY started_at DESC, id DESC LIMIT ")
            .push_bind(per_page as i64)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows = list_qb
            .build_query_as::<SearchAttemptRow>()
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
            .push("LOWER(COALESCE(query_text, '')) LIKE ")
            .push_bind(like.clone())
            .push(" ESCAPE '\\\\' OR LOWER(COALESCE(artist_text, '')) LIKE ")
            .push_bind(like)
            .push(" ESCAPE '\\\\')");
    }

    if let Some(status) = query.search_attempt_status {
        qb.push(" AND status = ").push_bind(status.as_str());
    }

    if let Some(attempt_type) = query.search_attempt_type {
        qb.push(" AND attempt_type = ").push_bind(attempt_type.as_str());
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
