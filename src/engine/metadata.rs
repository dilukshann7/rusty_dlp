use super::{dependencies::ResolvedDependencies, process};
use crate::model::*;
use serde_json::Value;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub const PAGE_SIZE: usize = 50;

#[derive(Debug, Clone)]
pub struct InspectionRequest {
    pub url: String,
    pub scope: InspectScope,
    pub page_start: usize,
}

pub fn base_args(dependencies: &ResolvedDependencies) -> Vec<String> {
    vec![
        "--ignore-config".into(),
        "--no-plugin-dirs".into(),
        "--no-remote-components".into(),
        "--no-js-runtimes".into(),
        "--js-runtimes".into(),
        format!("deno:{}", dependencies.deno.display()),
        "--color".into(),
        "never".into(),
        "--encoding".into(),
        "utf-8".into(),
        "--socket-timeout".into(),
        "30".into(),
    ]
}

pub async fn inspect(
    request: InspectionRequest,
    dependencies: ResolvedDependencies,
    cancellation: CancellationToken,
) -> Result<MediaPreview, EngineError> {
    let mut args = base_args(&dependencies);
    args.extend([
        "--dump-single-json".into(),
        "--skip-download".into(),
        "--no-warnings".into(),
    ]);
    match request.scope {
        InspectScope::Video => args.push("--no-playlist".into()),
        InspectScope::Playlist => args.extend([
            "--yes-playlist".into(),
            "--flat-playlist".into(),
            "--ignore-errors".into(),
            "--playlist-items".into(),
            format!(
                "{}:{}",
                request.page_start,
                request.page_start.saturating_add(PAGE_SIZE - 1)
            ),
        ]),
    }
    args.extend(["--".into(), request.url.clone()]);
    let output = process::capture(
        &dependencies.yt_dlp,
        &args,
        cancellation,
        Duration::from_secs(150),
    )
    .await?;
    let value: Value = serde_json::from_slice(&output)?;
    parse_preview(&request.url, &value, request.page_start)
}

fn string(value: &Value, key: &str) -> Option<String> {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}
fn thumbnail(value: &Value) -> Option<String> {
    string(value, "thumbnail").or_else(|| {
        value["thumbnails"]
            .as_array()?
            .iter()
            .rev()
            .find_map(|v| string(v, "url"))
    })
}

pub fn parse_preview(
    url: &str,
    value: &Value,
    page_start: usize,
) -> Result<MediaPreview, EngineError> {
    if !value.is_object() {
        return Err(EngineError::Message(
            "No public media metadata was returned. Check the URL and whether it needs login."
                .into(),
        ));
    }
    let is_playlist = value["entries"].is_array()
        || matches!(value["_type"].as_str(), Some("playlist" | "multi_video"));
    let mut entries = Vec::new();
    if let Some(items) = value["entries"].as_array() {
        for (offset, item) in items.iter().enumerate() {
            let id = string(item, "id");
            let candidate = string(item, "webpage_url").or_else(|| string(item, "url"));
            let entry_url = candidate
                .filter(|s| {
                    url::Url::parse(s)
                        .ok()
                        .is_some_and(|u| matches!(u.scheme(), "http" | "https"))
                })
                .or_else(|| {
                    let youtube = value["extractor"]
                        .as_str()
                        .unwrap_or_default()
                        .starts_with("youtube")
                        || item["ie_key"].as_str() == Some("Youtube");
                    if youtube {
                        id.as_ref()
                            .filter(|s| {
                                s.chars()
                                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                            })
                            .map(|id| format!("https://www.youtube.com/watch?v={id}"))
                    } else {
                        None
                    }
                });
            let title = string(item, "title").unwrap_or_else(|| "Unavailable entry".into());
            let availability = item["availability"].as_str().unwrap_or_default();
            let unavailable = if item.is_null() || entry_url.is_none() {
                Some("This entry has no accessible public URL.".into())
            } else if matches!(
                availability,
                "private" | "premium_only" | "subscriber_only" | "needs_auth"
            ) {
                Some("This entry requires login or membership.".into())
            } else if matches!(title.as_str(), "[Private video]" | "[Deleted video]") {
                Some(title.clone())
            } else {
                None
            };
            entries.push(PlaylistEntry {
                id,
                url: entry_url,
                title,
                index: item["playlist_index"]
                    .as_u64()
                    .or_else(|| value["requested_entries"][offset].as_u64())
                    .map(|i| i as usize)
                    .unwrap_or(page_start + offset),
                duration: item["duration"].as_f64(),
                thumbnail: thumbnail(item),
                selected: unavailable.is_none(),
                unavailable,
            });
        }
    }
    let formats = value["formats"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|f| {
                    let id = string(f, "format_id")?;
                    let format = FormatInfo {
                        id,
                        extension: string(f, "ext").unwrap_or_default(),
                        height: f["height"].as_u64().map(|v| v as u32),
                        fps: f["fps"].as_f64(),
                        video_codec: string(f, "vcodec"),
                        audio_codec: string(f, "acodec"),
                        language: string(f, "language"),
                        size: f["filesize"]
                            .as_u64()
                            .or_else(|| f["filesize_approx"].as_u64()),
                    };
                    if format.has_video() || format.has_audio() {
                        Some(format)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let next = page_start + PAGE_SIZE;
    let total = value["playlist_count"].as_u64().map(|n| n as usize);
    let next_page =
        if total.is_some_and(|n| next <= n) || (entries.len() == PAGE_SIZE && total.is_none()) {
            Some(next)
        } else {
            None
        };
    Ok(MediaPreview {
        url: url.into(),
        id: string(value, "id"),
        title: string(value, "title").unwrap_or_else(|| "Untitled media".into()),
        duration: value["duration"].as_f64(),
        thumbnail: thumbnail(value),
        formats,
        entries,
        is_playlist,
        next_page,
    })
}
