<div align="center">
  <img src="assets/branding/logo.svg" width="64" height="64" alt="rusty_dlp logo">
  <h1>rusty_dlp</h1>
  <p>A compact desktop app for downloading videos, playlists, and audio.</p>
  <p>
    Windows: <a href="https://github.com/dilukshann7/rusty_dlp/releases/download/v1.0.0/rusty_dlp-1.0.0-windows-x64-setup.exe"><strong>installer</strong></a>
    · <a href="https://github.com/dilukshann7/rusty_dlp/releases/download/v1.0.0/rusty-dlp-1.0.0-windows-x86_64-portable.zip"><strong>portable ZIP</strong></a><br>
    Linux x64: <a href="https://github.com/dilukshann7/rusty_dlp/releases/download/v1.0.0/rusty-dlp-1.0.0-linux-x86_64.tar.gz"><strong>tar.gz</strong></a>
    · macOS Intel: <a href="https://github.com/dilukshann7/rusty_dlp/releases/download/v1.0.0/rusty-dlp-1.0.0-macos-x86_64.zip"><strong>ZIP</strong></a>
    · Apple silicon: <a href="https://github.com/dilukshann7/rusty_dlp/releases/download/v1.0.0/rusty-dlp-1.0.0-macos-aarch64.zip"><strong>ZIP</strong></a><br>
    <a href="docs/RELEASE_NOTES_1.0.0.md">Release notes</a> · <a href="https://github.com/dilukshann7/rusty_dlp/releases">All releases</a>
  </p>
</div>

v1.0.0 stable · Unsigned packages with SHA-256 checksums. Windows end-to-end testing is verified; Linux and macOS are CI-built and tested, with runtime testing not verified.

## Features

- Download single videos, multiple links, or selected playlist entries.
- Choose quality up to 4K, exact streams, and MP4, MKV, or WebM output.
- Keep original audio or convert to MP3 and M4A.
- Manage concurrent downloads, cancellation, retry/resume, and history.
- Save presets and switch between dark, light, and system themes.

Powered by **yt-dlp**, with **FFmpeg** for merging and audio conversion.

## Get started

1. Open **Settings** and install yt-dlp, FFmpeg/ffprobe, and Deno—or select tools already installed.
2. Paste your links in **New download** and choose **Inspect links**.
3. Select the videos, quality, format, and destination.
4. Choose **Add to queue**, then **Start / resume** in Queue.

The installer requires no administrator access. The portable edition stores settings and managed tools in its own `data/` folder; keep `portable.flag` beside the executable.

See the [user guide](docs/USER_GUIDE.md) for playlist controls, tool updates, and recovery.

## Development

Built with Rust and iced. Requires **Rust 1.88+** and native build tools.

```sh
cargo run --locked --bin rusty-dlp
```

[Build and test instructions](docs/DEVELOPMENT.md) · [Report an issue](https://github.com/dilukshann7/rusty_dlp/issues)

## License

[MIT](LICENSE). External tools retain their own licenses; see [attributions](THIRD_PARTY.md).
