# Verification status — v1.0.0

The v1.0.0 release provides unsigned packages for Windows x64, Linux x64, Intel macOS, and Apple silicon macOS. The release workflow passed its automated checks on all four targets before publication.

## Automated checks

- `cargo fmt --all -- --check` passed.
- `python -m unittest discover -s tools -p test_tools.py -v` passed, covering all four archive layouts, included guides, icons, portable markers, executable permissions, checksums, and preservation of existing outputs.
- `cargo test --locked --features test-support --target <target>` and strict Clippy passed on each of the four release targets.
- Each target built and packaged its release binary. The Windows binary reported `rusty_dlp 1.0.0`, and the Windows job built the installer.
- The release job verified the five expected artifact filenames and every SHA-256 sidecar before publishing the stable release.

## Runtime verification

- Windows end-to-end runtime testing was completed and passed, as confirmed by the maintainer.
- Linux and macOS builds, tests, and packaging passed in CI. Runtime behavior on those platforms has not been verified.

## Distribution

- The Windows installer is a per-user install and preserves application data on uninstall. The portable ZIP uses adjacent `data/` storage through `portable.flag`.
- Linux is distributed as an x64 tar archive. Intel and Apple silicon macOS packages are app-bundle ZIPs.
- Portable archives and the Windows installer include `USER_GUIDE.md` and `DEVELOPMENT.md` under `docs/`.
- Every package has a SHA-256 sidecar. Release executables and archives are unsigned; macOS packages are not notarized. Media tools are downloaded through Settings and are not bundled.
- The Windows release uses a static C runtime. Signing, notarization, and application self-updates remain outside this release.
