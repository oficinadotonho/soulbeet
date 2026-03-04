use core::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryKind {
    DownloadAttempt,
    SearchAttempt,
}

impl Default for HistoryKind {
    fn default() -> Self {
        Self::DownloadAttempt
    }
}

impl HistoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DownloadAttempt => "download_attempt",
            Self::SearchAttempt => "search_attempt",
        }
    }
}

impl std::fmt::Display for HistoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HistoryKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "download_attempt" => Ok(Self::DownloadAttempt),
            "search_attempt" => Ok(Self::SearchAttempt),
            _ => Err(format!("invalid history kind: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadHistoryStatus {
    Queued,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

impl DownloadHistoryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Timeout => "timeout",
        }
    }
}

impl std::fmt::Display for DownloadHistoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DownloadHistoryStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "timeout" => Ok(Self::Timeout),
            _ => Err(format!("invalid download status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportHistoryStatus {
    NotStarted,
    InProgress,
    Completed,
    Skipped,
    Failed,
    Timeout,
}

impl ImportHistoryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
        }
    }
}

impl std::fmt::Display for ImportHistoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ImportHistoryStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "not_started" => Ok(Self::NotStarted),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "skipped" => Ok(Self::Skipped),
            "failed" => Ok(Self::Failed),
            "timeout" => Ok(Self::Timeout),
            _ => Err(format!("invalid import status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchAttemptStatus {
    InProgress,
    Completed,
    TimedOut,
    Failed,
    NoResults,
}

impl SearchAttemptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::TimedOut => "timed_out",
            Self::Failed => "failed",
            Self::NoResults => "no_results",
        }
    }
}

impl std::fmt::Display for SearchAttemptStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SearchAttemptStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "timed_out" => Ok(Self::TimedOut),
            "failed" => Ok(Self::Failed),
            "no_results" => Ok(Self::NoResults),
            _ => Err(format!("invalid search attempt status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchAttemptType {
    MetadataAlbum,
    MetadataTrack,
    SourceDownload,
}

impl SearchAttemptType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MetadataAlbum => "metadata_album",
            Self::MetadataTrack => "metadata_track",
            Self::SourceDownload => "source_download",
        }
    }
}

impl std::fmt::Display for SearchAttemptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SearchAttemptType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "metadata_album" => Ok(Self::MetadataAlbum),
            "metadata_track" => Ok(Self::MetadataTrack),
            "source_download" => Ok(Self::SourceDownload),
            _ => Err(format!("invalid search attempt type: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub kind: HistoryKind,
    pub action_id: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub release: Option<String>,
    pub item_label: Option<String>,
    pub download_status: Option<DownloadHistoryStatus>,
    pub import_status: Option<ImportHistoryStatus>,
    pub search_attempt_status: Option<SearchAttemptStatus>,
    pub search_attempt_type: Option<SearchAttemptType>,
    pub provider_id: Option<String>,
    pub backend_id: Option<String>,
    pub search_id: Option<String>,
    pub result_count: Option<i64>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HistoryQuery {
    pub kind: HistoryKind,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub download_status: Option<DownloadHistoryStatus>,
    #[serde(default)]
    pub import_status: Option<ImportHistoryStatus>,
    #[serde(default)]
    pub search_attempt_status: Option<SearchAttemptStatus>,
    #[serde(default)]
    pub search_attempt_type: Option<SearchAttemptType>,
    #[serde(default)]
    pub date_from_utc: Option<String>,
    #[serde(default)]
    pub date_to_utc: Option<String>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginatedHistory {
    pub entries: Vec<HistoryEntry>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryIdRequest {
    pub id: String,
    pub kind: HistoryKind,
}
