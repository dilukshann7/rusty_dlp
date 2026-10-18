# rusty_dlp v0.1.0 — Windows preview

A compact Rust + iced desktop front end for yt-dlp.

## Downloads

- **Windows x64 installer:** per-user installation, Start menu shortcut, optional desktop shortcut, and uninstaller. No administrator access required.
- **Windows x64 portable ZIP:** extract into a writable folder and run `rusty-dlp.exe`. Keep `portable.flag`; local state and managed tools live in `data/` beside the app.
- Matching `.sha256` files are provided for both downloads. Compare with `Get-FileHash -Algorithm SHA256 <file>` in PowerShell.

## Included

- Single or multiple links, text-file import, incremental playlists, entry/range selection, and thumbnail previews.
- Resolution caps, exact formats for individual videos, compatible output containers, original audio, and MP3/M4A conversion.
- Persistent queue, retries/resume, cancellation, history, presets, and dark/light/system appearance.
- Compact neutral interface and a consistent original app, window, and installer icon.
- Verified managed-tool downloads plus configured/PATH tool discovery. Install yt-dlp, FFmpeg/ffprobe, and Deno through Settings before downloading.

## Preview limits

These Windows packages are unsigned. They use a static C runtime and do not bundle media tools. Windows may display an unsigned-app warning; no OS protections are bypassed.

Automated offline tests cover queue/storage, formats/arguments, dependency integrity, metadata, process failures, and cancellation. This is a preview: real-site downloads, GUI interactions, and Linux/macOS runtime behavior are not claimed verified. Cookies/login, subtitles, SponsorBlock, custom arguments, time ranges, code signing, and automatic app updates remain outside this release.

Uninstalling preserves installed-mode settings and downloaded media. Close the app before replacing binaries or moving a portable folder. Existing jobs and custom paths can contain absolute paths; review them after moving. See README for setup and recovery.
