use super::process;
use crate::{
    model::{EngineError, Settings, ToolKind},
    storage,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    pub schema_version: u32,
    pub generated_at: String,
    pub entries: Vec<CatalogEntry>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogEntry {
    pub kind: ToolKind,
    pub target: String,
    pub version: String,
    pub source: String,
    pub assets: Vec<Asset>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub url: String,
    pub sha256: String,
    pub archive: String,
    pub outputs: Vec<String>,
}

pub fn catalog() -> Result<Catalog, EngineError> {
    Ok(serde_json::from_str(include_str!(
        "../../assets/dependencies.json"
    ))?)
}
pub fn target() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}
fn executable_name(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledDependency {
    pub kind: ToolKind,
    pub path: PathBuf,
    pub version: String,
    pub source: String,
}
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub source: String,
    pub problem: Option<String>,
    pub auxiliary: Option<PathBuf>,
}
#[derive(Debug, Clone, Default)]
pub struct DependencyReport {
    pub tools: BTreeMap<ToolKind, ToolStatus>,
}
#[derive(Debug, Clone)]
pub struct ResolvedDependencies {
    pub yt_dlp: PathBuf,
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
    pub deno: PathBuf,
}
#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub kind: ToolKind,
    pub fraction: Option<f32>,
    pub message: String,
}

impl DependencyReport {
    pub fn resolve(&self) -> Result<ResolvedDependencies, EngineError> {
        let required = |kind| {
            self.tools
                .get(&kind)
                .and_then(|s| s.path.clone())
                .ok_or_else(|| EngineError::Message(format!("Set up {kind} in Settings first.")))
        };
        Ok(ResolvedDependencies {
            yt_dlp: required(ToolKind::YtDlp)?,
            ffmpeg: required(ToolKind::Ffmpeg)?,
            deno: required(ToolKind::Deno)?,
            ffprobe: self
                .tools
                .get(&ToolKind::Ffmpeg)
                .and_then(|s| s.auxiliary.clone())
                .ok_or_else(|| {
                    EngineError::Message("ffprobe must be available beside FFmpeg.".into())
                })?,
        })
    }
    pub fn ready(&self) -> bool {
        self.resolve().is_ok()
    }
}

fn path_candidates(name: &str) -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths)
                .map(|p| p.join(executable_name(name)))
                .filter(|p| p.is_file())
                .collect()
        })
        .unwrap_or_default()
}

pub async fn probe(path: &Path, kind: ToolKind) -> Result<String, EngineError> {
    probe_with_cancel(path, kind, CancellationToken::new()).await
}

async fn probe_with_cancel(
    path: &Path,
    kind: ToolKind,
    cancellation: CancellationToken,
) -> Result<String, EngineError> {
    let flag = if kind == ToolKind::Ffmpeg {
        "-version"
    } else {
        "--version"
    };
    let output =
        process::capture(path, &[flag.into()], cancellation, Duration::from_secs(12)).await?;
    let line = String::from_utf8_lossy(&output)
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(200)
        .collect::<String>();
    let recognized = match kind {
        ToolKind::YtDlp => {
            line.chars().next().is_some_and(|c| c.is_ascii_digit()) || line.starts_with("yt-dlp")
        }
        ToolKind::Ffmpeg => {
            line.starts_with("ffmpeg version") || line.starts_with("ffprobe version")
        }
        ToolKind::Deno => line.starts_with("deno "),
    };
    if !recognized {
        return Err(EngineError::Message(format!(
            "{} did not identify itself as {kind}.",
            path.display()
        )));
    }
    Ok(line)
}

pub async fn discover_dependencies(settings: Settings) -> DependencyReport {
    let root = storage::data_dir().ok();
    let mut report = DependencyReport::default();
    for &kind in ToolKind::ALL {
        let mut candidates = Vec::new();
        if let Some(path) = settings.tool_paths.get(&kind) {
            candidates.push((path.clone(), "Configured".to_string()));
        }
        if let Some(root) = &root
            && let Ok(bytes) = fs::read(root.join("tools").join(kind.key()).join("active.json"))
            && let Ok(installed) = serde_json::from_slice::<InstalledDependency>(&bytes)
        {
            let base = root.join("tools").join(kind.key());
            if installed.kind == kind
                && let Some(path) = resolve_managed_path(&base, &installed.path)
            {
                candidates.push((path, format!("Managed ({})", installed.version)));
            }
        }
        candidates.extend(
            path_candidates(kind.executable())
                .into_iter()
                .map(|p| (p, "PATH".into())),
        );
        let mut status = ToolStatus {
            path: None,
            version: None,
            source: "Not installed".into(),
            problem: None,
            auxiliary: None,
        };
        for (path, source) in candidates {
            match probe(&path, kind).await {
                Ok(version) => {
                    if kind == ToolKind::Ffmpeg {
                        let auxiliary = path
                            .parent()
                            .unwrap_or(Path::new("."))
                            .join(executable_name("ffprobe"));
                        match probe(&auxiliary, ToolKind::Ffmpeg).await {
                            Ok(_) => status.auxiliary = Some(auxiliary),
                            Err(_) => {
                                status.problem = Some(format!(
                                    "ffprobe was not found beside {}.",
                                    path.display()
                                ));
                                continue;
                            }
                        }
                    }
                    status.path = Some(path);
                    status.version = Some(version);
                    status.source = source;
                    status.problem = None;
                    break;
                }
                Err(error) => status.problem = Some(super::input::redact(&error.to_string())),
            }
        }
        report.tools.insert(kind, status);
    }
    report
}

/// Resolve relative portable pointers and legacy contained absolute pointers.
/// Canonicalization also rejects symlinks escaping this managed tool folder.
pub fn resolve_managed_path(base: &Path, path: &Path) -> Option<PathBuf> {
    let base = base.canonicalize().ok()?;
    if !path.is_absolute()
        && path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    let resolved = base.join(path).canonicalize().ok()?;
    (resolved.starts_with(&base) && resolved.is_file()).then_some(resolved)
}

pub fn verify_sha256(path: &Path, expected: &str) -> Result<(), EngineError> {
    let mut source = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let size = source.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hash.update(&buffer[..size]);
    }
    if hex::encode(hash.finalize()) != expected.to_ascii_lowercase() {
        return Err(EngineError::Message(
            "Dependency checksum mismatch. The download was not installed.".into(),
        ));
    }
    Ok(())
}

fn safe_archive_path(path: &Path) -> bool {
    !path.is_absolute()
        && path
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
        && !path.to_string_lossy().contains(['\\', ':'])
}

fn copy_executable(
    mut reader: impl Read,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), EngineError> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let mut total = 0u64;
    let mut buffer = [0; 65536];
    loop {
        if cancellation.is_cancelled() {
            return Err(EngineError::Canceled);
        }
        let size = reader.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        total += size as u64;
        if total > 700 * 1024 * 1024 {
            return Err(EngineError::Message(
                "Dependency executable exceeds the size limit.".into(),
            ));
        }
        file.write_all(&buffer[..size])?;
    }
    if total == 0 {
        return Err(EngineError::Message(
            "Dependency executable is empty.".into(),
        ));
    }
    file.sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(destination, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

pub fn extract_asset(
    path: &Path,
    archive: &str,
    outputs: &[String],
    staging: &Path,
) -> Result<(), EngineError> {
    extract_with_cancel(path, archive, outputs, staging, &CancellationToken::new())
}

fn extract_with_cancel(
    path: &Path,
    archive: &str,
    outputs: &[String],
    staging: &Path,
    cancellation: &CancellationToken,
) -> Result<(), EngineError> {
    let error = |e: String| EngineError::Message(format!("Invalid dependency archive: {e}"));
    match archive {
        "raw" if outputs.len() == 1 => copy_executable(
            fs::File::open(path)?,
            &staging.join(&outputs[0]),
            cancellation,
        )?,
        "zip" => {
            let mut zip =
                zip::ZipArchive::new(fs::File::open(path)?).map_err(|e| error(e.to_string()))?;
            if zip.len() > 20000 {
                return Err(error("too many entries".into()));
            }
            for index in 0..zip.len() {
                if cancellation.is_cancelled() {
                    return Err(EngineError::Canceled);
                }
                let mut entry = zip.by_index(index).map_err(|e| error(e.to_string()))?;
                let name = PathBuf::from(entry.name());
                if !safe_archive_path(&name)
                    || entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
                {
                    return Err(error("unsafe path or symbolic link".into()));
                }
                if let Some(name) = name
                    .file_name()
                    .and_then(|n| n.to_str())
                    .filter(|n| outputs.iter().any(|o| o == n))
                    && !entry.is_dir()
                {
                    copy_executable(&mut entry, &staging.join(name), cancellation)?;
                }
            }
        }
        "tar.xz" => {
            let mut tar = tar::Archive::new(xz2::read::XzDecoder::new(fs::File::open(path)?));
            for (index, item) in tar.entries()?.enumerate() {
                if index > 20000 {
                    return Err(error("too many entries".into()));
                }
                if cancellation.is_cancelled() {
                    return Err(EngineError::Canceled);
                }
                let mut entry = item?;
                let path = entry.path()?.into_owned();
                if !safe_archive_path(&path)
                    || entry.header().entry_type().is_symlink()
                    || entry.header().entry_type().is_hard_link()
                {
                    return Err(error("unsafe path or link".into()));
                }
                if let Some(name) = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .filter(|n| outputs.iter().any(|o| o == n))
                    && entry.header().entry_type().is_file()
                {
                    copy_executable(&mut entry, &staging.join(name), cancellation)?;
                }
            }
        }
        _ => return Err(error("unsupported archive type".into())),
    }
    for output in outputs {
        if !staging.join(output).is_file() {
            return Err(error(format!("missing {output}")));
        }
    }
    Ok(())
}

pub async fn install_dependency(
    kind: ToolKind,
    target: String,
    events: std::sync::mpsc::Sender<InstallProgress>,
    cancellation: CancellationToken,
) -> Result<InstalledDependency, EngineError> {
    let entry = catalog()?
        .entries
        .into_iter()
        .find(|e| e.kind == kind && e.target == target)
        .ok_or_else(|| {
            EngineError::Message(format!(
                "No approved {kind} download for {target}. Choose an existing executable instead."
            ))
        })?;
    install_entry(entry, events, cancellation).await
}

pub async fn install_entry(
    entry: CatalogEntry,
    events: std::sync::mpsc::Sender<InstallProgress>,
    cancellation: CancellationToken,
) -> Result<InstalledDependency, EngineError> {
    let root = storage::data_dir().map_err(|e| EngineError::Message(e.to_string()))?;
    let base = root.join("tools").join(entry.kind.key());
    fs::create_dir_all(&base)?;
    let staging = tempfile::Builder::new()
        .prefix("version-")
        .tempdir_in(&base)?;
    let client = reqwest::Client::builder()
        .user_agent("RustyDLP/0.1")
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(900))
        .build()?;
    let emit = |fraction, message: String| {
        let _ = events.send(InstallProgress {
            kind: entry.kind,
            fraction,
            message,
        });
    };
    for (index, asset) in entry.assets.iter().enumerate() {
        if cancellation.is_cancelled() {
            return Err(EngineError::Canceled);
        }
        let url = url::Url::parse(&asset.url).map_err(|e| EngineError::Message(e.to_string()))?;
        if url.scheme() != "https" {
            return Err(EngineError::Message(
                "Dependency downloads require HTTPS.".into(),
            ));
        }
        let download = staging.path().join(format!("asset-{index}.download"));
        let mut response = tokio::select! { _ = cancellation.cancelled() => return Err(EngineError::Canceled), r = client.get(url).send() => r?.error_for_status()? };
        let total = response.content_length();
        let mut downloaded = 0u64;
        let mut file = tokio::fs::File::create(&download).await?;
        let mut last_update = std::time::Instant::now() - Duration::from_secs(1);
        loop {
            let chunk = tokio::select! { _ = cancellation.cancelled() => return Err(EngineError::Canceled), chunk = response.chunk() => chunk? };
            let Some(chunk) = chunk else {
                break;
            };
            downloaded += chunk.len() as u64;
            if downloaded > 800 * 1024 * 1024 {
                return Err(EngineError::Message(
                    "Dependency download exceeds the size limit.".into(),
                ));
            }
            file.write_all(&chunk).await?;
            if last_update.elapsed() >= Duration::from_millis(150) {
                emit(
                    total.filter(|t| *t > 0).map(|t| {
                        (index as f32 + downloaded as f32 / t as f32) / entry.assets.len() as f32
                    }),
                    format!(
                        "Downloading {}… {} MB",
                        entry.kind,
                        downloaded / 1024 / 1024
                    ),
                );
                last_update = std::time::Instant::now();
            }
        }
        file.sync_all().await?;
        drop(file);
        emit(None, "Verifying checksum and extracting…".into());
        let asset = asset.clone();
        let staging_path = staging.path().to_owned();
        let cancel = cancellation.clone();
        tokio::task::spawn_blocking(move || {
            verify_sha256(&download, &asset.sha256)?;
            extract_with_cancel(
                &download,
                &asset.archive,
                &asset.outputs,
                &staging_path,
                &cancel,
            )?;
            fs::remove_file(download)?;
            Ok::<_, EngineError>(())
        })
        .await
        .map_err(|e| EngineError::Message(e.to_string()))??;
    }
    if cancellation.is_cancelled() {
        return Err(EngineError::Canceled);
    }
    emit(None, "Checking executable…".into());
    let installed = activate_staged(staging, &entry, &base, cancellation).await?;
    emit(Some(1.0), "Installed and verified.".into());
    Ok(installed)
}

pub async fn activate_staged(
    staging: tempfile::TempDir,
    entry: &CatalogEntry,
    base: &Path,
    cancellation: CancellationToken,
) -> Result<InstalledDependency, EngineError> {
    if cancellation.is_cancelled() {
        return Err(EngineError::Canceled);
    }
    if staging.path().parent() != Some(base) {
        return Err(EngineError::Message(
            "Installation staging directory is outside the managed tool folder.".into(),
        ));
    }
    let path = staging
        .path()
        .join(executable_name(entry.kind.executable()));
    probe_with_cancel(&path, entry.kind, cancellation.clone()).await?;
    if entry.kind == ToolKind::Ffmpeg {
        probe_with_cancel(
            &staging.path().join(executable_name("ffprobe")),
            ToolKind::Ffmpeg,
            cancellation.clone(),
        )
        .await?;
    }
    if cancellation.is_cancelled() {
        return Err(EngineError::Canceled);
    }
    let installed = InstalledDependency {
        kind: entry.kind,
        path: path.clone(),
        version: entry.version.clone(),
        source: entry.source.clone(),
    };
    let mut portable_pointer = installed.clone();
    portable_pointer.path = installed
        .path
        .strip_prefix(base)
        .map_err(|e| EngineError::Message(e.to_string()))?
        .to_path_buf();
    let bytes = serde_json::to_vec_pretty(&portable_pointer)?;
    let mut pointer = tempfile::NamedTempFile::new_in(base)?;
    pointer.write_all(&bytes)?;
    pointer.as_file().sync_all()?;
    // Keep the version directory before activation; a failed pointer write preserves the previous version.
    let _kept_directory = staging.keep();
    pointer
        .persist(base.join("active.json"))
        .map_err(|e| EngineError::Message(e.to_string()))?;
    Ok(installed)
}

/// Explicit update action only. Freeze the selected release/hash before downloading it.
pub async fn latest_entry(kind: ToolKind) -> Result<CatalogEntry, EngineError> {
    let current_target = target();
    let mut entry = catalog()?
        .entries
        .into_iter()
        .find(|e| e.kind == kind && e.target == current_target)
        .ok_or_else(|| {
            EngineError::Message("This platform does not have a managed dependency catalog.".into())
        })?;
    if kind == ToolKind::Ffmpeg && std::env::consts::OS == "macos" {
        // The signed-off macOS catalog advances with the app; never scrape a new executable into trust.
        return Ok(entry);
    }
    let repo = match kind {
        ToolKind::YtDlp => "yt-dlp/yt-dlp",
        ToolKind::Deno => "denoland/deno",
        ToolKind::Ffmpeg => "yt-dlp/FFmpeg-Builds",
    };
    let client = reqwest::Client::builder()
        .user_agent("RustyDLP/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;
    let value: serde_json::Value = client
        .get(format!(
            "https://api.github.com/repos/{repo}/releases/latest"
        ))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let assets = value["assets"]
        .as_array()
        .ok_or_else(|| EngineError::Message("Release metadata did not include assets.".into()))?;
    let old_name = entry.assets[0].url.rsplit('/').next().unwrap_or_default();
    let suffix = if std::env::consts::OS == "windows" {
        "-win64-gpl.zip"
    } else {
        "-linux64-gpl.tar.xz"
    };
    let asset = assets
        .iter()
        .find(|a| {
            a["name"].as_str().is_some_and(|n| {
                if kind == ToolKind::Ffmpeg {
                    n.ends_with(suffix)
                } else {
                    n == old_name
                }
            })
        })
        .ok_or_else(|| {
            EngineError::Message(
                "The new release has no compatible asset. The current installation was preserved."
                    .into(),
            )
        })?;
    let digest = asset["digest"]
        .as_str()
        .and_then(|s| s.strip_prefix("sha256:"))
        .filter(|s| s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| {
            EngineError::Message(
                "The release has no published SHA-256 digest; update was not installed.".into(),
            )
        })?;
    entry.version = value["tag_name"].as_str().unwrap_or("unknown").into();
    entry.assets[0].url = asset["browser_download_url"]
        .as_str()
        .ok_or_else(|| EngineError::Message("Missing release URL.".into()))?
        .into();
    entry.assets[0].sha256 = digest.into();
    Ok(entry)
}
