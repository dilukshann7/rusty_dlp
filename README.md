<div align="center">
  <img src="assets/branding/logo.svg" width="64" height="64" alt="rusty_dlp logo">
  <h1>rusty_dlp</h1>
  <p>A compact desktop app for downloading videos, playlists, and audio.</p>
  <p>
    <a href="https://github.com/dilukshann7/rusty_dlp/releases/download/v0.1.0/rusty_dlp-0.1.0-windows-x64-setup.exe"><strong>Windows installer</strong></a>
    &nbsp;·&nbsp;
    <a href="https://github.com/dilukshann7/rusty_dlp/releases/download/v0.1.0/rusty-dlp-0.1.0-windows-x86_64-portable.zip"><strong>Portable ZIP</strong></a>
    &nbsp;·&nbsp;
    <a href="https://github.com/dilukshann7/rusty_dlp/releases">Releases</a>
  </p>
</div>

Windows x64 · v0.1.0 preview · Unsigned packages. Checksums are included with each release.

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
