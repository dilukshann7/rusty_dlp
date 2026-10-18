# Verification status — 2026-10-18

The core application, compact interface, branding, and Windows installer/portable packaging are implemented. The source supports Windows x64, Linux x64, Intel macOS, and Apple Silicon macOS; this release preparation targets Windows x64.

## Automated checks

- `cargo fmt --all -- --check` and `git diff --check` pass.
- `cargo test --locked --features test-support`: 42 tests pass on Windows, including format policies, queue transitions, persistence/recovery, checksums, unsafe archives, metadata, stderr flooding, process failures, and descendant cancellation.
- `cargo clippy --locked --all-targets --features test-support -- -D warnings`: passes.
- `python -m unittest discover -s tools -p test_tools.py -v`: five tests pass. Fixtures cover all four archive layouts, icons, portable markers, permissions, checksums, and preservation of existing outputs. Fixture bytes are not platform executables.
- Independent source review identified and verified fixes for relocatable managed-tool pointers and output locking shared between installed and portable instances. Regression checks cover both behaviors.
- The optimized Windows x64 build passes with the static C runtime. `--version` reports `rusty_dlp 0.1.0`, Windows finds the embedded executable icon/version metadata, and import inspection finds no external MSVC runtime or liblzma DLL dependency.
- The actual Windows installer passed a silent installation/version-probe/uninstallation test in an isolated folder. It did not enable portable mode, removed its binary on uninstall, and preserved a user-owned data file. The GUI was not launched.
- Installer and portable artifacts were created with individual SHA-256 files. Release publishing is separate from creating these local artifacts.

## Distribution

- The original app mark is exported to SVG, PNG, multi-size ICO, and ICNS. Windows builds embed icon/version resources and set the window icon; macOS packaging includes its bundle icon.
- The Windows installer uses Inno Setup, installs for the current user without elevation, offers shortcuts and an uninstaller, and preserves application data and downloaded media.
- The Windows portable ZIP opts into adjacent `data/` storage using `portable.flag`. Managed tool pointers are relative; explicitly configured tool paths, output destinations, history, and queued jobs may still contain absolute paths.
- A destination's `.rusty-dlp-output-locks` directory coordinates final output ownership across copies of the application. Lock files are harmless after the owning process exits.
- Release binaries use the static Windows C runtime. Media tools download explicitly through Settings and are not bundled.
- The four-target GitHub verification/packaging workflow remains manually triggered. No cross-platform workflow run is implied by this Windows release.

## Verification limits

This is an unsigned preview. GUI interaction, actual network media downloads, real dependency installation, conversion, and cancel/resume/restart smoke checks are still separate from the offline fixtures. Linux and macOS binaries and runtime behavior have not been verified on this Windows host. Signing, notarization, and application self-updates remain outside this release.
