# Rusty DLP — Project Brief

## Goal
Build a Rust desktop GUI using **iced** that makes yt-dlp easy to use, including playlist downloads and quality selection. Use yt-dlp as the download engine rather than reimplementing its extractors.

## Proposed approach
- **Recommended:** run the yt-dlp executable from Rust. This keeps the GUI separate from the download engine and allows independent engine updates.
- Alternatives: embedding Python adds runtime complexity; rebuilding download logic in Rust would make site compatibility difficult to maintain.
- Proposed defaults: Windows first, a portable initial release, and an architecture that can later support Linux and macOS. These are design suggestions, not confirmed requirements.

## Features
- **Add downloads:** paste one or multiple URLs, import a URL list, and preview titles, thumbnails, durations, and available formats.
- **Playlists:** preview entries, select all or individual items, choose index ranges, preserve ordering, and optionally create a playlist folder. Skip unavailable entries with a visible reason.
- **Quality selector:** best available or a resolution limit, plus exact format selection showing codec, frame rate, audio language, and size when known. Offer video with audio, video only, and audio only. Resolve formats per playlist entry; show when a requested quality is unavailable and apply a visible fallback policy.
- **Output:** choose destination, filename template, container, audio conversion format and bitrate, and reusable download presets. Explain when conversion is required.
- **Queue:** adjustable concurrency, item and overall progress, speed, ETA, cancel, retry, and history with open-file/open-folder actions. Save pending jobs across restarts; offer stop-and-resume using partial files where supported, without promising instant pause.
- **Extras:** subtitle language selection and embedding, thumbnails, metadata, chapters, SponsorBlock, time-range downloads, and duplicate avoidance through a download archive.
- **Advanced:** browser cookies or a cookie file, proxy settings, rate limits, retries, and custom yt-dlp arguments/configuration for options without dedicated controls.
- **Settings:** theme, default presets, dependency paths and versions, dependency diagnostics, and an explicit action to update an app-managed yt-dlp executable.

“All features” means dedicated controls for common workflows plus advanced access to other yt-dlp options; a separate widget for every CLI flag is outside the initial scope. Availability depends on the site, media, and installed dependencies.

## Interface
Use four main views: **New Download**, **Queue**, **History**, and **Settings**. The normal flow is paste URL → inspect media or playlist → select quality and destination → add to queue. Keep detailed logs and advanced options expandable.

## Technical outline
- Use iced's message/update model; run metadata requests and child processes asynchronously so the UI stays responsive.
- Suggested supporting libraries: Tokio for process orchestration and Serde for structured data. Exact versions and persistence format are left to implementation.
- Read metadata as JSON and progress through yt-dlp's structured output options; track download, post-processing, and final completion separately. Success requires a successful process exit and confirmed output paths.
- Pass subprocess arguments directly without a shell. Define precedence between presets and advanced arguments, and reserve machine-output arguments needed by the GUI. Advanced execution hooks require explicit opt-in.
- Detect yt-dlp, FFmpeg/ffprobe, and required JavaScript runtime support; offer actionable setup guidance. Initially use configured or PATH executables rather than silently installing dependencies.
- Persist preferences, queue state, and history; redact sensitive values from logs and avoid storing passwords or cookie contents in history.

## Delivery order
1. Single URL, metadata preview, quality selection, destination, progress, and cancellation.
2. Playlist selection, persistent queue, retries/resume, presets, and history.
3. Subtitle/post-processing controls, advanced options, dependency updates, and packaging.

Acceptance: a single video and selected playlist entries download with the chosen policy; unavailable formats, missing dependencies, interruptions, and failed jobs produce clear recoverable states. Verify these workflows before release. Builds and a dev server are only run when explicitly requested.

## References
- [iced — official repository](https://github.com/iced-rs/iced)
- [yt-dlp — options, dependencies, and embedding guidance](https://github.com/yt-dlp/yt-dlp#readme)
