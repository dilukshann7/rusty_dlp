# User guide

## Downloading media

1. Open **Settings** and install the approved tools, or choose existing executables. FFmpeg and ffprobe must be in the same directory. Tool discovery prefers configured paths, then managed tools, then PATH.
2. Paste links in **New download** and select **Inspect links**. Pure collection links open as playlists; video links with playlist context default to **This video**.
3. Select entries and output options. Playlist ranges affect loaded entries only; **Load next 50 entries** extends the list.
4. Choose **Add to queue**, then choose **Start / resume**. Queued settings are snapshots: later edits do not alter jobs already added.

**More options** contains playlist folders, exact streams, filename templates, presets, and saved defaults.

A resolution cap never falls back above the selected height. Explicit containers filter compatible streams; they do not silently transcode video. Unavailable combinations fail with a log. Original audio preserves a supported source audio codec, extracting it from a combined source if necessary. Conversion bitrate does not increase source quality.

Partial downloads live under `.rusty-dlp-parts/<job-id>` in the selected destination, so different jobs do not share partial files. Quality variants that share output names are serialized; an already-existing final output is preserved and reported as skipped when its target can be predicted. Use a different filename template or folder to keep multiple variants of the same video.

Destination folders also contain `.rusty-dlp-output-locks` so installed and portable copies coordinate writes to the same filename. The OS releases ownership when a process exits; remaining lock files do not indicate an active download.

To use exact formats, inspect one individual video. Select compatible video and audio stream IDs as needed. For playlists, each item resolves its own formats from the quality policy. Video titles, durations, sizes, audio languages, and thumbnails depend on the source's metadata.

## Dependencies and updates

The app downloads tools into the data directory shown in Settings when you click Install. The portable edition uses its adjacent `data/` folder. Installation verifies the downloaded bytes against the approved catalog before extracting or executing them; version probes run before activation. A canceled or failed installation retains the previous active version. Downloads do not require administrator access or change global PATH.

**Update managed tool** explicitly queries published upstream release metadata for yt-dlp/Deno and Windows/Linux FFmpeg. Updates require a published SHA-256 digest. macOS FFmpeg updates use the approved catalog shipped with the app. User-installed tools are never updated by the app. Older managed version folders are retained for recovery.

Dependency changes are blocked while inspections, downloads, or other installation work use the tools. Hold the queue and finish or cancel active work first. Missing runtimes, OS execution restrictions, dependency errors and network failures appear in Settings. Manual executable selection remains available if managed installation is unavailable.

Maintainers can run `python tools/refresh_dependency_catalog.py` to refresh source URLs and published hashes without downloading or executing binaries. Review the resulting catalog before shipping it. SHA-256 checks establish integrity against those pinned values; they are not a claim of independent source-code auditing.

## Data and recovery

Settings shows the exact local data directory and an **Open data folder** action. It holds `state.json`, `state.backup.json`, `instance.lock`, and managed `tools/` versions. Only one app instance may own this data directory at a time.

The queue restores paused. Previously active jobs become Interrupted and require **Resume** or **Start / resume**. Holding the queue lets active jobs finish. Cancel terminates the job's process tree and retains partial files; resuming uses yt-dlp's continuation support where the source permits it. A graceful app close waits for cancellation cleanup and the serialized state writer.

If the main state is corrupt, the app attempts the backup. If both files are unreadable, or a newer unsupported schema is detected, files are preserved and saving/download scheduling is disabled. Close the app, copy both files somewhere safe, then rename them in the data folder to start with empty state. Never edit local state while the app is running. A stale `instance.lock` file is harmless; the OS lock releases when the owning process exits.

If a state write fails during use, the queue pauses pending jobs. Fix disk space or folder permissions and choose **Retry saving** in Settings before restarting queued work.

The app avoids overwriting existing outputs and ignores unrelated yt-dlp configurations/plugins. Logs are bounded and redact URL queries; logs and thumbnail caches are session-only. File URLs and URLs containing embedded credentials are rejected. Public media is the initial scope; login/cookies, subtitles, SponsorBlock, custom arguments, and time ranges are deferred.
