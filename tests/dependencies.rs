use rusty_dlp::{
    engine::dependencies::{catalog, extract_asset, verify_sha256},
    model::ToolKind,
};

#[test]
fn relative_managed_tools_survive_portable_folder_relocation() {
    use rusty_dlp::engine::dependencies::resolve_managed_path;
    let root = tempfile::tempdir().unwrap();
    let old = root.path().join("old/tools/yt-dlp");
    std::fs::create_dir_all(old.join("version")).unwrap();
    std::fs::write(old.join("version/tool.exe"), b"fixture").unwrap();
    let moved = root.path().join("moved");
    std::fs::rename(root.path().join("old"), &moved).unwrap();
    let base = moved.join("tools/yt-dlp");
    let relative = std::path::Path::new("version/tool.exe");
    assert_eq!(
        resolve_managed_path(&base, relative),
        Some(base.join(relative).canonicalize().unwrap())
    );
    assert!(resolve_managed_path(&base, &old.join(relative)).is_none());
    assert!(
        resolve_managed_path(&base, std::path::Path::new("../yt-dlp/version/tool.exe")).is_none()
    );
    let outside = root.path().join("outside.exe");
    std::fs::write(&outside, b"fixture").unwrap();
    assert!(resolve_managed_path(&base, &outside).is_none());
}

#[test]
fn catalog_covers_every_tool_on_every_release_target() {
    let entries = catalog().unwrap();
    for target in [
        "windows-x86_64",
        "linux-x86_64",
        "macos-x86_64",
        "macos-aarch64",
    ] {
        for kind in [ToolKind::YtDlp, ToolKind::Deno, ToolKind::Ffmpeg] {
            let entry = entries
                .entries
                .iter()
                .find(|e| e.target == target && e.kind == kind)
                .unwrap();
            assert!(!entry.assets.is_empty());
            assert!(
                entry
                    .assets
                    .iter()
                    .all(|a| a.sha256.len() == 64 && a.url.starts_with("https://"))
            );
        }
    }
}

#[test]
fn checksum_mismatch_is_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("download");
    std::fs::write(&path, b"abc").unwrap();
    assert!(
        verify_sha256(
            &path,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        )
        .is_ok()
    );
    assert!(verify_sha256(&path, &"0".repeat(64)).is_err());
}

#[test]
fn archive_traversal_cannot_write_outside_staging() {
    use std::io::Write;
    let directory = tempfile::tempdir().unwrap();
    let archive_path = directory.path().join("unsafe.zip");
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
    archive
        .start_file("../escaped.exe", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(b"bad").unwrap();
    archive.finish().unwrap();
    let staging = directory.path().join("staging");
    std::fs::create_dir(&staging).unwrap();
    assert!(extract_asset(&archive_path, "zip", &["ffmpeg.exe".into()], &staging).is_err());
    assert!(!directory.path().join("escaped.exe").exists());
}

#[test]
fn missing_executable_cannot_be_activated() {
    let directory = tempfile::tempdir().unwrap();
    let archive_path = directory.path().join("empty.zip");
    zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap())
        .finish()
        .unwrap();
    let staging = directory.path().join("staging");
    std::fs::create_dir(&staging).unwrap();
    assert!(
        extract_asset(
            &archive_path,
            "zip",
            &["ffmpeg".into(), "ffprobe".into()],
            &staging
        )
        .is_err()
    );
}

#[tokio::test]
async fn failed_version_probe_preserves_active_installation() {
    use rusty_dlp::engine::dependencies::{CatalogEntry, activate_staged};
    let directory = tempfile::tempdir().unwrap();
    let base = directory.path();
    std::fs::write(base.join("active.json"), b"previous working pointer").unwrap();
    let staging = tempfile::tempdir_in(base).unwrap();
    std::fs::write(
        staging
            .path()
            .join(format!("yt-dlp{}", std::env::consts::EXE_SUFFIX)),
        b"not an executable",
    )
    .unwrap();
    let entry = CatalogEntry {
        kind: ToolKind::YtDlp,
        target: "fixture".into(),
        version: "broken".into(),
        source: "fixture".into(),
        assets: vec![],
    };
    assert!(
        activate_staged(
            staging,
            &entry,
            base,
            tokio_util::sync::CancellationToken::new()
        )
        .await
        .is_err()
    );
    assert_eq!(
        std::fs::read(base.join("active.json")).unwrap(),
        b"previous working pointer"
    );
}

#[tokio::test]
async fn canceled_installation_preserves_active_installation() {
    use rusty_dlp::engine::dependencies::{CatalogEntry, activate_staged};
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("active.json"), b"old").unwrap();
    let staging = tempfile::tempdir_in(directory.path()).unwrap();
    let cancel = tokio_util::sync::CancellationToken::new();
    cancel.cancel();
    let entry = CatalogEntry {
        kind: ToolKind::YtDlp,
        target: "fixture".into(),
        version: "broken".into(),
        source: "fixture".into(),
        assets: vec![],
    };
    assert!(
        activate_staged(staging, &entry, directory.path(), cancel)
            .await
            .is_err()
    );
    assert_eq!(
        std::fs::read(directory.path().join("active.json")).unwrap(),
        b"old"
    );
}
