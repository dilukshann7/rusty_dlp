# Rusty DLP — Project Brief

## Goal
Build a Rust desktop GUI using **iced** that makes yt-dlp easy to use, including playlist downloads and quality selection. Use yt-dlp as the download engine rather than reimplementing its extractors.

## Proposed approach
- **Recommended:** run the yt-dlp executable from Rust. This keeps the GUI separate from the download engine and allows independent engine updates.
- Alternatives: embedding Python adds runtime complexity; rebuilding download logic in Rust would make site compatibility difficult to maintain.
- Confirmed initial targets: Windows x64, Linux x64, Intel macOS and Apple Silicon macOS; portable archives and macOS application bundles.

## Features
- **Add downloads:** paste one or multiple URLs, import a URL list, and preview titles, thumbnails, durations, and available formats.
- **Playlists:** preview entries, select all or individual items, choose index ranges, preserve ordering, and optionally create a playlist folder. Skip unavailable entries with a visible reason.
- **Quality selector:** best available or a resolution limit, plus exact format selection showing codec, frame rate, audio language, and size when known. Offer video with audio, video only, and audio only. Resolve formats per playlist entry; show when a requested quality is unavailable and apply a visible fallback policy.
- **Output:** choose destination, filename template, container, audio conversion format and bitrate, and reusable download presets. Explain when conversion is required.
- **Queue:** adjustable concurrency, item and overall progress, speed, ETA, cancel, retry, and history with open-file/open-folder actions. Save pending jobs across restarts; offer stop-and-resume using partial files where supported, without promising instant pause.
- **Later releases:** subtitles, metadata/chapter embedding, SponsorBlock, time ranges, download archives, browser cookies, proxy/rate-limit controls and custom arguments.
- **Settings:** theme, default presets, dependency paths and versions, dependency diagnostics, and an explicit action to update an app-managed yt-dlp executable.

“All features” means dedicated controls for common workflows plus advanced access to other yt-dlp options; a separate widget for every CLI flag is outside the initial scope. Availability depends on the site, media, and installed dependencies.

## Interface
Use four main views: **New Download**, **Queue**, **History**, and **Settings**. The normal flow is paste URL → inspect media or playlist → select quality and destination → add to queue. Keep detailed logs and advanced options expandable.

## Technical outline
- Use iced's message/update model; run metadata requests and child processes asynchronously so the UI stays responsive.
- Suggested supporting libraries: Tokio for process orchestration and Serde for structured data. Exact versions and persistence format are left to implementation.
- Read metadata as JSON and progress through yt-dlp's structured output options; track download, post-processing, and final completion separately. Success requires a successful process exit and confirmed output paths.
- Pass subprocess arguments directly without a shell. Define precedence between presets and advanced arguments, and reserve machine-output arguments needed by the GUI. Advanced execution hooks require explicit opt-in.
- Detect configured tools, then app-managed tools, then PATH. Offer explicit in-app downloads of yt-dlp, Deno and FFmpeg/ffprobe on all four targets, verifying published SHA-256 hashes before installation. Never silently install dependencies.
- Persist preferences, queue state, and history; redact sensitive values from logs and avoid storing passwords or cookie contents in history.

## Delivery order
1. Core release: shell and dependency manager; URL/playlist inspection; quality/output controls; process engine; persistent queue, presets and history; manual release workflows.
2. Later releases: authentication, subtitles, SponsorBlock, time ranges and advanced CLI access.

Confirmed defaults: clean dark utility UI with light/system themes; two concurrent jobs (adjustable 1–4); resolution selections cap quality rather than requiring an exact height; original/MP3/M4A audio with 192 kbps default conversion; public media only. See `docs/superpowers/plans/2026-10-18-rusty-dlp-core.md` for the approved implementation plan.

Acceptance: a single video and selected playlist entries download with the chosen policy; unavailable formats, missing dependencies, interruptions, and failed jobs produce clear recoverable states. Verify these workflows before release. Builds and a dev server are only run when explicitly requested.

## References
- [iced — official repository](https://github.com/iced-rs/iced)
- [yt-dlp — options, dependencies, and embedding guidance](https://github.com/yt-dlp/yt-dlp#readme)
