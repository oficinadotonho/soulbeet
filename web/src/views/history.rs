use dioxus::prelude::*;
use shared::history::{
    DownloadHistoryStatus, HistoryEntry, HistoryIdRequest, HistoryKind, HistoryQuery,
    ImportHistoryStatus, SearchAttemptStatus, SearchAttemptType,
};
use ui::{Button, Modal};

use crate::auth::use_auth;

#[derive(Clone)]
struct DownloadActionGroup {
    action_id: String,
    entries: Vec<HistoryEntry>,
}

#[derive(Clone)]
struct DownloadReleaseGroup {
    release_name: String,
    actions: Vec<DownloadActionGroup>,
}

#[derive(Clone)]
struct DownloadDateGroup {
    date_key: String,
    releases: Vec<DownloadReleaseGroup>,
}

#[derive(Clone)]
struct SearchDateGroup {
    date_key: String,
    entries: Vec<HistoryEntry>,
}

#[component]
pub fn HistoryPage() -> Element {
    let auth = use_auth();

    let mut entries = use_signal(Vec::<HistoryEntry>::new);
    let mut page = use_signal(|| 1);
    let mut total_pages = use_signal(|| 0);
    let loading = use_signal(|| false);
    let action_busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut has_bootstrapped = use_signal(|| false);
    let mut show_clear_confirm = use_signal(|| false);

    let mut kind = use_signal(|| HistoryKind::DownloadAttempt);
    let mut search = use_signal(String::new);
    let mut download_status = use_signal(|| None::<DownloadHistoryStatus>);
    let mut import_status = use_signal(|| None::<ImportHistoryStatus>);
    let mut search_attempt_status = use_signal(|| None::<SearchAttemptStatus>);
    let mut search_attempt_type = use_signal(|| None::<SearchAttemptType>);
    let mut date_from = use_signal(|| None::<String>);
    let mut date_to = use_signal(|| None::<String>);

    let fetch_page = move |page_to_fetch: i32, replace: bool, suppress_error: bool| {
        let auth = auth;
        let current_kind = kind();
        let current_search = search();
        let current_download_status = download_status();
        let current_import_status = import_status();
        let current_search_status = search_attempt_status();
        let current_search_type = search_attempt_type();
        let current_date_from = date_from();
        let current_date_to = date_to();

        let mut entries = entries;
        let mut page = page;
        let mut total_pages = total_pages;
        let mut loading = loading;
        let mut error = error;

        spawn(async move {
            loading.set(true);
            error.set(None);

            let query = HistoryQuery {
                kind: current_kind,
                search: if current_search.trim().is_empty() {
                    None
                } else {
                    Some(current_search)
                },
                download_status: current_download_status,
                import_status: current_import_status,
                search_attempt_status: current_search_status,
                search_attempt_type: current_search_type,
                date_from_utc: current_date_from,
                date_to_utc: current_date_to,
                page: Some(page_to_fetch),
                per_page: Some(20),
            };

            match auth.call(api::query_history(query)).await {
                Ok(data) => {
                    page.set(data.page);
                    total_pages.set(data.total_pages);
                    if replace {
                        entries.set(data.entries);
                    } else {
                        let mut merged = entries();
                        merged.extend(data.entries);
                        entries.set(merged);
                    }
                }
                Err(fetch_error) => {
                    if !suppress_error {
                        error.set(Some(format!("Failed to load history: {fetch_error}")));
                    }
                }
            }

            loading.set(false);
        });
    };

    use_effect(move || {
        if !has_bootstrapped() {
            has_bootstrapped.set(true);
            fetch_page(1, true, true);
        }
    });

    let apply_filters = move |_| {
        fetch_page(1, true, false);
    };

    let load_more = move |_| {
        if loading() {
            return;
        }
        let next_page = page() + 1;
        if total_pages() == 0 || next_page > total_pages() {
            return;
        }
        fetch_page(next_page, false, false);
    };

    let mut switch_kind = move |next_kind: HistoryKind| {
        if kind() == next_kind {
            return;
        }
        kind.set(next_kind);
        error.set(None);
        page.set(1);
        total_pages.set(0);
        entries.set(vec![]);
        fetch_page(1, true, false);
    };

    let clear_all = move |_| {
        if action_busy() {
            return;
        }
        let auth = auth;
        let mut entries = entries;
        let mut page = page;
        let mut total_pages = total_pages;
        let mut action_busy = action_busy;
        let mut error = error;
        let mut show_clear_confirm = show_clear_confirm;

        spawn(async move {
            action_busy.set(true);
            show_clear_confirm.set(false);
            match auth.call(api::clear_history()).await {
                Ok(_) => {
                    entries.set(vec![]);
                    page.set(1);
                    total_pages.set(0);
                    error.set(None);
                }
                Err(clear_error) => {
                    error.set(Some(format!("Failed to clear history: {clear_error}")));
                }
            }
            action_busy.set(false);
        });
    };

    let delete_entry = move |entry: HistoryEntry| {
        if action_busy() {
            return;
        }
        let auth = auth;
        let mut action_busy = action_busy;
        let mut error = error;
        let id = entry.id.clone();
        let entry_kind = entry.kind;

        spawn(async move {
            action_busy.set(true);
            error.set(None);
            let req = HistoryIdRequest {
                id,
                kind: entry_kind,
            };
            match auth.call(api::delete_history(req)).await {
                Ok(_) => fetch_page(1, true, false),
                Err(delete_error) => {
                    error.set(Some(format!("Delete failed: {delete_error}")));
                }
            }
            action_busy.set(false);
        });
    };

    let download_groups = group_download_entries(&entries.read());
    let search_groups = group_search_entries(&entries.read());

    rsx! {
        if show_clear_confirm() {
            Modal {
                on_close: move |_| show_clear_confirm.set(false),
                header: rsx! {
                    h3 { class: "text-lg font-semibold text-white", "Clear History" }
                },
                div { class: "space-y-5",
                    p { class: "text-sm text-gray-200",
                        "Are you sure you want to delete ALL history? This action is irreversible."
                    }
                    div { class: "flex justify-end gap-3",
                        button {
                            class: "px-3 py-2 text-xs uppercase tracking-widest font-mono border border-white/20 rounded hover:bg-white/5 transition-colors cursor-pointer",
                            onclick: move |_| show_clear_confirm.set(false),
                            "Cancel"
                        }
                        button {
                            class: "px-3 py-2 text-xs uppercase tracking-widest font-mono border border-red-500/40 text-red-200 rounded hover:bg-red-500/10 transition-colors cursor-pointer",
                            onclick: clear_all,
                            "Delete All"
                        }
                    }
                }
            }
        }

        div { class: "fixed top-1/4 -left-10 w-64 h-64 bg-beet-accent/10 rounded-full blur-[100px] pointer-events-none" }
        div { class: "fixed bottom-1/4 -right-10 w-64 h-64 bg-beet-leaf/10 rounded-full blur-[100px] pointer-events-none" }

        div { class: "space-y-6 text-white w-full max-w-6xl z-10 mx-auto",
            div { class: "flex flex-wrap items-center justify-between gap-4",
                div { class: "space-y-1",
                    h1 { class: "text-4xl font-bold text-beet-accent font-display", "History" }
                    p { class: "text-gray-400 font-mono text-xs", "Download attempts and search attempts timeline" }
                }
                button {
                    class: "px-3 py-2 text-xs uppercase tracking-widest font-mono border border-white/20 rounded hover:bg-white/5 transition-colors cursor-pointer disabled:opacity-50",
                    disabled: action_busy(),
                    onclick: move |_| show_clear_confirm.set(true),
                    "Clear All"
                }
            }

            nav { class: "flex items-center gap-1 bg-beet-panel/50 p-1.5 rounded-full border border-white/5 backdrop-blur-sm w-fit",
                button {
                    class: if matches!(kind(), HistoryKind::DownloadAttempt) {
                        "flex items-center gap-2 px-4 py-2 rounded-full bg-white/10 text-white text-sm font-medium transition-all cursor-pointer"
                    } else {
                        "flex items-center gap-2 px-4 py-2 rounded-full text-gray-400 text-sm font-medium hover:text-white hover:bg-white/5 transition-all cursor-pointer"
                    },
                    onclick: move |_| switch_kind(HistoryKind::DownloadAttempt),
                    "Download Attempts"
                }
                button {
                    class: if matches!(kind(), HistoryKind::SearchAttempt) {
                        "flex items-center gap-2 px-4 py-2 rounded-full bg-white/10 text-white text-sm font-medium transition-all cursor-pointer"
                    } else {
                        "flex items-center gap-2 px-4 py-2 rounded-full text-gray-400 text-sm font-medium hover:text-white hover:bg-white/5 transition-all cursor-pointer"
                    },
                    onclick: move |_| switch_kind(HistoryKind::SearchAttempt),
                    "Search Attempts"
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-6 gap-3 bg-beet-panel/50 border border-white/5 p-4 rounded-lg",
                input {
                    class: "md:col-span-2 px-3 py-2 rounded bg-black/20 border border-white/10 text-white placeholder:text-gray-500",
                    value: "{search}",
                    placeholder: "Search title / artist / release",
                    oninput: move |ev| search.set(ev.value()),
                }
                input {
                    class: "px-3 py-2 rounded bg-black/20 border border-white/10 text-white",
                    r#type: "date",
                    value: "{date_from().as_deref().map(extract_date).unwrap_or_default()}",
                    oninput: move |ev| {
                        let value = ev.value();
                        if value.trim().is_empty() {
                            date_from.set(None);
                        } else {
                            date_from.set(Some(format!("{value}T00:00:00Z")));
                        }
                    },
                }
                input {
                    class: "px-3 py-2 rounded bg-black/20 border border-white/10 text-white",
                    r#type: "date",
                    value: "{date_to().as_deref().map(extract_date).unwrap_or_default()}",
                    oninput: move |ev| {
                        let value = ev.value();
                        if value.trim().is_empty() {
                            date_to.set(None);
                        } else {
                            date_to.set(Some(format!("{value}T23:59:59Z")));
                        }
                    },
                }

                if matches!(kind(), HistoryKind::DownloadAttempt) {
                    select {
                        class: "px-3 py-2 rounded bg-black/20 border border-white/10 text-white",
                        onchange: move |ev| {
                            let value = ev.value();
                            download_status.set(if value.is_empty() {
                                None
                            } else {
                                value.parse::<DownloadHistoryStatus>().ok()
                            });
                        },
                        option { value: "", selected: download_status().is_none(), "All Download States" }
                        option { value: "queued", selected: matches!(download_status(), Some(DownloadHistoryStatus::Queued)), "Queued" }
                        option { value: "in_progress", selected: matches!(download_status(), Some(DownloadHistoryStatus::InProgress)), "In Progress" }
                        option { value: "completed", selected: matches!(download_status(), Some(DownloadHistoryStatus::Completed)), "Completed" }
                        option { value: "failed", selected: matches!(download_status(), Some(DownloadHistoryStatus::Failed)), "Failed" }
                        option { value: "cancelled", selected: matches!(download_status(), Some(DownloadHistoryStatus::Cancelled)), "Cancelled" }
                        option { value: "timeout", selected: matches!(download_status(), Some(DownloadHistoryStatus::Timeout)), "Timeout" }
                    }

                    select {
                        class: "px-3 py-2 rounded bg-black/20 border border-white/10 text-white",
                        onchange: move |ev| {
                            let value = ev.value();
                            import_status.set(if value.is_empty() {
                                None
                            } else {
                                value.parse::<ImportHistoryStatus>().ok()
                            });
                        },
                        option { value: "", selected: import_status().is_none(), "All Import States" }
                        option { value: "not_started", selected: matches!(import_status(), Some(ImportHistoryStatus::NotStarted)), "Not Started" }
                        option { value: "in_progress", selected: matches!(import_status(), Some(ImportHistoryStatus::InProgress)), "In Progress" }
                        option { value: "completed", selected: matches!(import_status(), Some(ImportHistoryStatus::Completed)), "Completed" }
                        option { value: "skipped", selected: matches!(import_status(), Some(ImportHistoryStatus::Skipped)), "Skipped" }
                        option { value: "failed", selected: matches!(import_status(), Some(ImportHistoryStatus::Failed)), "Failed" }
                        option { value: "timeout", selected: matches!(import_status(), Some(ImportHistoryStatus::Timeout)), "Timeout" }
                    }
                } else {
                    select {
                        class: "px-3 py-2 rounded bg-black/20 border border-white/10 text-white",
                        onchange: move |ev| {
                            let value = ev.value();
                            search_attempt_type.set(if value.is_empty() {
                                None
                            } else {
                                value.parse::<SearchAttemptType>().ok()
                            });
                        },
                        option { value: "", selected: search_attempt_type().is_none(), "All Attempt Types" }
                        option { value: "metadata_album", selected: matches!(search_attempt_type(), Some(SearchAttemptType::MetadataAlbum)), "Metadata Album" }
                        option { value: "metadata_track", selected: matches!(search_attempt_type(), Some(SearchAttemptType::MetadataTrack)), "Metadata Track" }
                        option { value: "source_download", selected: matches!(search_attempt_type(), Some(SearchAttemptType::SourceDownload)), "Source Download" }
                    }

                    select {
                        class: "px-3 py-2 rounded bg-black/20 border border-white/10 text-white",
                        onchange: move |ev| {
                            let value = ev.value();
                            search_attempt_status.set(if value.is_empty() {
                                None
                            } else {
                                value.parse::<SearchAttemptStatus>().ok()
                            });
                        },
                        option { value: "", selected: search_attempt_status().is_none(), "All Attempt Statuses" }
                        option { value: "in_progress", selected: matches!(search_attempt_status(), Some(SearchAttemptStatus::InProgress)), "In Progress" }
                        option { value: "completed", selected: matches!(search_attempt_status(), Some(SearchAttemptStatus::Completed)), "Completed" }
                        option { value: "timed_out", selected: matches!(search_attempt_status(), Some(SearchAttemptStatus::TimedOut)), "Timed Out" }
                        option { value: "failed", selected: matches!(search_attempt_status(), Some(SearchAttemptStatus::Failed)), "Failed" }
                        option { value: "no_results", selected: matches!(search_attempt_status(), Some(SearchAttemptStatus::NoResults)), "No Results" }
                    }
                }
            }

            div { class: "flex items-center gap-3",
                Button {
                    class: "rounded whitespace-nowrap min-w-40 flex justify-center",
                    disabled: loading(),
                    onclick: apply_filters,
                    if loading() {
                        span { class: "inline-flex items-center gap-2",
                            span { class: "inline-block h-3 w-3 rounded-full border-2 border-beet-leaf border-t-transparent animate-spin" }
                            "Applying"
                        }
                    } else {
                        "Apply Filters"
                    }
                }
                if let Some(message) = error() {
                    p { class: "text-sm text-red-300", "{message}" }
                }
            }

            if entries.read().is_empty() && !loading() {
                div { class: "text-center py-16 text-gray-400 font-mono text-sm",
                    "No history entries yet."
                }
            }

            if matches!(kind(), HistoryKind::DownloadAttempt) {
                div { class: "space-y-6",
                    for day_group in download_groups {
                        section { key: "day-download-{day_group.date_key}", class: "space-y-4",
                            h2 { class: "text-lg font-semibold text-white border-b border-white/10 pb-2", "{day_group.date_key}" }
                            for release_group in day_group.releases {
                                article { key: "release-{day_group.date_key}-{release_group.release_name}", class: "rounded-lg border border-white/10 bg-black/20 p-4 space-y-4",
                                    h3 { class: "text-base font-semibold text-beet-leaf", "{release_group.release_name}" }
                                    for (idx , action_group) in release_group.actions.into_iter().enumerate() {
                                        div { key: "action-{action_group.action_id}", class: "space-y-2",
                                            p { class: "text-xs uppercase tracking-widest text-gray-400 font-mono",
                                                "Action {idx + 1} · {action_group.entries.len()} track(s)"
                                            }
                                            div { class: "space-y-2",
                                                for entry in action_group.entries {
                                                    DownloadEntryCard {
                                                        key: "{entry.id}",
                                                        entry: entry.clone(),
                                                        deleting: action_busy(),
                                                        on_delete: move |_| delete_entry(entry.clone()),
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "space-y-6",
                    for day_group in search_groups {
                        section { key: "day-search-{day_group.date_key}", class: "space-y-3",
                            h2 { class: "text-lg font-semibold text-white border-b border-white/10 pb-2", "{day_group.date_key}" }
                            div { class: "space-y-2",
                                for entry in day_group.entries {
                                    SearchEntryCard {
                                        key: "{entry.id}",
                                        entry: entry.clone(),
                                        deleting: action_busy(),
                                        on_delete: move |_| delete_entry(entry.clone()),
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if total_pages() > 0 && page() < total_pages() {
                div { class: "pt-2",
                    button {
                        class: "px-4 py-2 border border-white/20 rounded hover:bg-white/5 transition-colors cursor-pointer disabled:opacity-50",
                        disabled: loading(),
                        onclick: load_more,
                        if loading() { "Loading..." } else { "Load More" }
                    }
                }
            }
        }
    }
}

#[component]
fn DownloadEntryCard(entry: HistoryEntry, deleting: bool, on_delete: EventHandler<()>) -> Element {
    let download_status = entry
        .download_status
        .map(|status| pretty_status(status.as_str()))
        .unwrap_or_else(|| "unknown".to_string());
    let import_status = entry
        .import_status
        .map(|status| pretty_status(status.as_str()))
        .unwrap_or_else(|| "unknown".to_string());

    rsx! {
        div { class: "rounded border border-white/10 bg-black/30 p-3 space-y-2",
            div { class: "flex items-start justify-between gap-3",
                div {
                    p { class: "text-sm text-white font-medium", "{entry.title.clone().unwrap_or_else(|| \"Untitled\".to_string())}" }
                    p { class: "text-xs text-gray-400", "{entry.artist.clone().unwrap_or_else(|| \"Unknown Artist\".to_string())}" }
                }
                button {
                    class: "px-2 py-1 text-[11px] uppercase tracking-widest font-mono border border-red-500/40 text-red-200 rounded hover:bg-red-500/10 transition-colors cursor-pointer disabled:opacity-50",
                    disabled: deleting,
                    onclick: move |_| on_delete.call(()),
                    "Delete"
                }
            }
            div { class: "flex flex-wrap items-center gap-2 text-xs",
                span { class: "px-2 py-1 rounded bg-blue-500/15 text-blue-200 border border-blue-500/30", "Download: {download_status}" }
                span { class: "px-2 py-1 rounded bg-green-500/15 text-green-200 border border-green-500/30", "Import: {import_status}" }
            }
            if let Some(error) = entry.error_message.clone() {
                p { class: "text-xs text-red-300", "{error}" }
            }
        }
    }
}

#[component]
fn SearchEntryCard(entry: HistoryEntry, deleting: bool, on_delete: EventHandler<()>) -> Element {
    let status = entry
        .search_attempt_status
        .map(|value| pretty_status(value.as_str()))
        .unwrap_or_else(|| "unknown".to_string());
    let kind = entry
        .search_attempt_type
        .map(|value| pretty_status(value.as_str()))
        .unwrap_or_else(|| "unknown".to_string());

    rsx! {
        article { class: "rounded border border-white/10 bg-black/20 p-4 space-y-2",
            div { class: "flex justify-between items-start gap-3",
                div { class: "space-y-1",
                    p { class: "text-sm font-medium text-white",
                        "{entry.title.clone().unwrap_or_else(|| \"Search\".to_string())}"
                    }
                    if let Some(artist) = entry.artist.clone() {
                        p { class: "text-xs text-gray-400", "{artist}" }
                    }
                }
                button {
                    class: "px-2 py-1 text-[11px] uppercase tracking-widest font-mono border border-red-500/40 text-red-200 rounded hover:bg-red-500/10 transition-colors cursor-pointer disabled:opacity-50",
                    disabled: deleting,
                    onclick: move |_| on_delete.call(()),
                    "Delete"
                }
            }
            div { class: "flex flex-wrap items-center gap-2 text-xs",
                span { class: "px-2 py-1 rounded bg-purple-500/15 text-purple-200 border border-purple-500/30", "Type: {kind}" }
                span { class: "px-2 py-1 rounded bg-yellow-500/15 text-yellow-200 border border-yellow-500/30", "Status: {status}" }
                span { class: "px-2 py-1 rounded bg-white/10 text-gray-200 border border-white/20", "Results: {entry.result_count.unwrap_or(0)}" }
            }
            if let Some(error) = entry.error_message.clone() {
                p { class: "text-xs text-red-300", "{error}" }
            }
        }
    }
}

fn group_download_entries(entries: &[HistoryEntry]) -> Vec<DownloadDateGroup> {
    let mut groups = Vec::<DownloadDateGroup>::new();

    for entry in entries {
        let date_key = extract_date(&entry.started_at);
        let release = entry
            .release
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "Unknown Release".to_string());
        let action_id = entry
            .action_id
            .clone()
            .unwrap_or_else(|| format!("unknown-{}", entry.id));

        let day_index = groups
            .iter()
            .position(|group| group.date_key == date_key)
            .unwrap_or_else(|| {
                groups.push(DownloadDateGroup {
                    date_key: date_key.clone(),
                    releases: Vec::new(),
                });
                groups.len() - 1
            });

        let release_index = groups[day_index]
            .releases
            .iter()
            .position(|group| group.release_name == release)
            .unwrap_or_else(|| {
                groups[day_index].releases.push(DownloadReleaseGroup {
                    release_name: release.clone(),
                    actions: Vec::new(),
                });
                groups[day_index].releases.len() - 1
            });

        let action_index = groups[day_index].releases[release_index]
            .actions
            .iter()
            .position(|group| group.action_id == action_id)
            .unwrap_or_else(|| {
                groups[day_index].releases[release_index]
                    .actions
                    .push(DownloadActionGroup {
                        action_id: action_id.clone(),
                        entries: Vec::new(),
                    });
                groups[day_index].releases[release_index].actions.len() - 1
            });

        groups[day_index].releases[release_index].actions[action_index]
            .entries
            .push(entry.clone());
    }

    groups
}

fn group_search_entries(entries: &[HistoryEntry]) -> Vec<SearchDateGroup> {
    let mut groups = Vec::<SearchDateGroup>::new();

    for entry in entries {
        let date_key = extract_date(&entry.started_at);
        let index = groups
            .iter()
            .position(|group| group.date_key == date_key)
            .unwrap_or_else(|| {
                groups.push(SearchDateGroup {
                    date_key: date_key.clone(),
                    entries: Vec::new(),
                });
                groups.len() - 1
            });

        groups[index].entries.push(entry.clone());
    }

    groups
}

fn extract_date(timestamp: &str) -> String {
    timestamp
        .split('T')
        .next()
        .unwrap_or(timestamp)
        .to_string()
}

fn pretty_status(value: &str) -> String {
    value
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            first.to_uppercase().collect::<String>() + chars.as_str()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
