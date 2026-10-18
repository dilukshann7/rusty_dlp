# Rusty DLP Core Release Implementation Plan

> Use superpowers:executing-plans sequentially. Implementation authorized 2026-10-18.

**Goal:** A cross-platform iced GUI for yt-dlp: videos, playlists, quality selection, audio, persistent queue, history, presets, and managed dependencies.

**Architecture:** One Rust 2024 Cargo package. iced 0.14 owns presentation; Tokio runs metadata and downloader processes; the queue owns job state; one storage owner writes versioned JSON. Stable job/attempt IDs reject stale events.

**Spec:** ../../../PROJECT_BRIEF.md, amended by the confirmed scope below.

## Confirmed scope and constraints
- Windows x64, Linux x64, Intel macOS, Apple Silicon macOS.
- Dark utility design, with light and system themes. Window 1180x780, minimum 900x600.
- In-app downloads of yt-dlp, Deno, FFmpeg/ffprobe on every target; detect existing and configured tools too.
- Best/resolution-cap selection, exact per-video formats, Auto/MP4/MKV/WebM, original audio/MP3/M4A and 192 kbps default.
- Resolution caps never fall back above the limit. Explicit containers do not silently transcode video.
- Public media only. Cookies/login, subtitles, SponsorBlock, time ranges, custom arguments and app self-update are deferred.
- No builds, app launches, dev servers or workflow dispatches unless explicitly requested. Prepare Rust tests without running compilation.

## Task 1: Foundation and shell
- [x] Cargo package, types/errors, app/ui/engine/queue/storage boundaries.
- [x] Four persistent views: New Download, Queue, History, Settings.
- [x] Theme, keyboard focus, scrollable lists, loading and error states.
- [x] Application-data paths and single-instance lock.

## Task 2: Dependencies
- [x] Configured > managed > PATH discovery with version-probe timeouts.
- [x] Checked-in catalog: exact versions, asset URLs, SHA256 and archive metadata for all four targets.
- [x] Official yt-dlp/Deno assets; yt-dlp FFmpeg builds for Windows/Linux; Martin Riedl ZIPs for macOS.
- [x] Download progress/cancel, hash validation, safe extraction, executable permissions, probe before atomic activation, preserve old version.
- [x] Explicit managed updates, manual browse, actionable failure feedback, no replacement during active work.

## Task 3: Inspection and playlists
- [x] Multiline HTTP(S) URLs and UTF-8 list import with per-line errors.
- [x] Flat paged playlist inspection; full metadata only on demand or execution.
- [x] Titles/thumbnails/durations/formats and unavailable reasons.
- [x] Select all/individual/range, original ordering, optional playlist folder.
- [x] This-video/playlist selector for video URLs with playlist context; default this video.
- [x] Cancel stale inspections and load long collections incrementally.

## Task 4: Quality/output/presets
- [x] Combined/video/audio modes, best and 144p-2160p caps.
- [x] Exact format IDs for individual media; never reuse across playlist entries.
- [x] Codec/FPS/language/size labels, explicit-container compatibility checks, original/MP3/M4A audio.
- [x] Destination, filenames with IDs and playlist indexes, named presets, immutable queued option snapshots.

## Task 5: Process engine
- [x] Direct argument vectors; ignore inherited configuration and plugins.
- [x] JSON metadata and structured prefixed records, concurrent stdout/stderr drains, bounded logs, throttled updates.
- [x] Distinct preparation/download/postprocess/completion; unknown totals indeterminate.
- [x] Success only after zero exit plus final-file confirmation.
- [x] Windows Job Objects/Unix process groups for cancellation and shutdown; preserve partial files; hidden Windows consoles.

## Task 6: Queue/persistence/history/settings
- [x] Full lifecycle including failed/canceled/skipped/interrupted; FIFO, default concurrency 2, range 1-4.
- [x] Cancel/retry/resume/remove-pending/clear-finished.
- [x] Atomic JSON writes plus backup, serialized writer, restore active jobs interrupted, explicit resume after restart.
- [x] Output actions/missing-file feedback, no overwrite default, active duplicate prevention.
- [x] Theme/default destination/presets/dependency settings and bounded diagnostics.

## Task 7: Release preparation
- [x] Portable Windows/Linux archives and macOS bundles on four targets.
- [x] Manual-only verification/packaging workflows, no dispatch.
- [x] Setup, dependency attribution, targets, recovery and unsigned-release documentation.
- [x] Save plan and update brief.

## Verification
Prepare meaningful tests for dependency targets, corrupt hashes/archives/installs; metadata omissions/unavailable entries/Unicode/cancellation; format caps/container compatibility/exact failures/audio arguments; fragmented/malformed progress, flooded stderr, unknown totals/nonzero exits; descendant cleanup; concurrency/stale events/duplicates/recovery/unwritable storage. Use a Rust fake downloader. Run tests and real smoke checks only when requested. Each platform must be verified separately before a release is called verified.

## Implementation record
Checkboxes indicate source implementation, not verified runtime behavior. See ../../IMPLEMENTATION_STATUS.md for performed checks and deferred validation.
