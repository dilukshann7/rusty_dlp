# Dependency sources and attribution

Rusty DLP uses separately maintained tools and Rust libraries. This application's license does not replace their licenses.

| Component | Source | Purpose |
|---|---|---|
| iced | https://github.com/iced-rs/iced | Native GUI (MIT) |
| yt-dlp | https://github.com/yt-dlp/yt-dlp | Extraction and downloading; official executable releases |
| Deno | https://github.com/denoland/deno | JavaScript runtime for extraction (MIT) |
| FFmpeg/ffprobe | https://ffmpeg.org | Media merging, probing, extraction and conversion |
| FFmpeg Windows/Linux builds | https://github.com/yt-dlp/FFmpeg-Builds | Managed platform-specific GPL builds |
| FFmpeg macOS builds | https://ffmpeg.martin-riedl.de | Managed Intel and Apple Silicon builds |
| winresource | https://github.com/BenjaminRi/winresource | Windows icon/version resource generation (MIT) |
| Inno Setup | https://jrsoftware.org | Windows installer compiler and runtime; see https://jrsoftware.org/files/is/license.txt |

Exact tool versions, URLs and published SHA-256 hashes are embedded from `assets/dependencies.json`. FFmpeg build configurations have their own licensing requirements, including GPL components. yt-dlp executable distributions can include dependencies under additional licenses. Consult each provider's release notices and source references when redistributing tools.

App packages do not bundle media tools. The user invokes their downloads through Settings. Relevant upstream notices remain available at the linked sources. The Cargo lockfile records Rust package versions; use Cargo license-report tooling before a public distribution if a complete transitive license inventory is needed.

The original `rusty_dlp` logo and its PNG/ICO/ICNS exports are covered by this repository's MIT license. They are not YouTube or Vercel brand assets. Pillow is used only to regenerate icons and is not required to run the app.
