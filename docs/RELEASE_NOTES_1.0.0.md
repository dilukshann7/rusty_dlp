# rusty_dlp v1.0.0

The first stable release of rusty_dlp is available for Windows x64, Linux x64, Intel macOS, and Apple silicon macOS.

## Downloads

- Windows x64 installer: `rusty_dlp-1.0.0-windows-x64-setup.exe`
- Windows x64 portable: `rusty-dlp-1.0.0-windows-x86_64-portable.zip`
- Linux x64: `rusty-dlp-1.0.0-linux-x86_64.tar.gz`
- Intel macOS: `rusty-dlp-1.0.0-macos-x86_64.zip`
- Apple silicon macOS: `rusty-dlp-1.0.0-macos-aarch64.zip`

Each package has a matching `.sha256` file. Portable archives and the Windows installer include the user and development guides. All v1.0.0 packages are unsigned; Windows may show an unsigned-app warning, and macOS packages are not notarized.

## Verification

- Windows end-to-end runtime testing was completed and passed, as confirmed by the maintainer.
- The Windows release executable reports `rusty_dlp 1.0.0`.
- CI passed formatting, packaging fixtures, Rust tests, strict Clippy, and release packaging on all four target platforms.
- Linux and macOS runtime behavior has not been verified; their status reflects CI checks only.

The app supports single and multiple links, playlist selection, quality and format choices, audio extraction or conversion, a persistent queue, retries, cancellation, history, presets, and managed media-tool setup. See the [user guide](USER_GUIDE.md) for operation and recovery details.

Cookies/login, subtitles, SponsorBlock, custom arguments, time ranges, signing, notarization, and automatic app updates are outside this release.
