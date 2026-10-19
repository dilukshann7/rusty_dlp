# Development

Use Rust 1.88 or newer and the native build tools for your platform. The Cargo lockfile is checked in.

```sh
cargo run --locked --bin rusty-dlp
cargo fmt --all -- --check
cargo test --locked --features test-support
cargo clippy --locked --all-targets --features test-support -- -D warnings
python -m unittest discover -s tools -p test_tools.py -v
```

`test-support` provides an offline fake downloader for deterministic integration tests. It is excluded from normal release packages. Packaging tests use inert fixture bytes.

## Windows release

Install Inno Setup 6, then run in PowerShell:

```powershell
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
cargo build --locked --release --bin rusty-dlp --target x86_64-pc-windows-msvc
python tools/package.py --target x86_64-pc-windows-msvc --platform windows-x86_64
python tools/build_installer.py
```

The installer compiler can also be supplied using `--compiler path/to/ISCC.exe`. Packaging scripts use an existing executable, emit SHA-256 files, and refuse to replace previous outputs. Preserve or move old output files and folders before creating a replacement.

Brand assets are checked in. Regenerate them with Pillow using `python tools/generate_icons.py`.

## Other platforms

The [release workflow](../.github/workflows/verify-and-package.yml) verifies and packages Windows x64, Linux x64, Intel macOS, and Apple silicon macOS. Pushing the `v1.0.0` tag runs the four-target matrix and publishes the stable GitHub release only when every target passes. The workflow validates artifact names and SHA-256 sidecars before publication. A manual `workflow_dispatch` run can also create unsigned packages without publishing a release.

Linux desktop use requires a display environment and native file-dialog portal. CI builds and tests Linux and macOS packages, but runtime behavior on those platforms is not verified by the Windows end-to-end test.

See [verification status](IMPLEMENTATION_STATUS.md) for checks completed and remaining platform smoke tests. Real dependency-install, download, conversion, cancellation/resume, and restart checks must run on each platform before that platform is called verified.
