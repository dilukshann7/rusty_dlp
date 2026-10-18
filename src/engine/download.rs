use super::{
    dependencies::ResolvedDependencies,
    formats,
    input::{redact, safe_component},
    metadata,
    process::{ProcessTree, drain_records},
    protocol,
};
use crate::model::*;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

pub fn destination(job: &DownloadJob) -> PathBuf {
    if job.options.playlist_folder {
        if let Some(title) = &job.playlist_title {
            return job.options.destination.join(safe_component(title));
        }
    }
    job.options.destination.clone()
}

pub fn arguments(
    job: &DownloadJob,
    dependencies: &ResolvedDependencies,
) -> Result<Vec<String>, EngineError> {
    formats::validate_options(&job.options)?;
    let mut args = metadata::base_args(dependencies);
    args.extend([
        "--no-playlist".into(),
        "--newline".into(),
        "--progress".into(),
        "--progress-delta".into(),
        "0.2".into(),
        "--no-overwrites".into(),
        "--no-post-overwrites".into(),
        "--continue".into(),
        "--part".into(),
        "--windows-filenames".into(),
        "--retries".into(),
        "3".into(),
        "--fragment-retries".into(),
        "3".into(),
        "--ffmpeg-location".into(),
        dependencies
            .ffmpeg
            .parent()
            .unwrap_or(Path::new("."))
            .to_string_lossy()
            .into_owned(),
        "--format".into(),
        formats::format_selector(&job.options)?,
        "--paths".into(),
        destination(job).to_string_lossy().into_owned(),
    ]);
    args.extend([
        "--paths".into(),
        format!("temp:{}", partial_directory(job).display()),
    ]);
    let prefix = job
        .playlist_index
        .map(|i| format!("{i:04} - "))
        .unwrap_or_default();
    args.extend([
        "--output".into(),
        format!("{prefix}{}", job.options.filename_template),
        "--progress-template".into(),
        "download:RDL_PROGRESS:%(progress)j".into(),
        "--progress-template".into(),
        "postprocess:RDL_POST:%(progress)j".into(),
        "--print".into(),
        "before_dl:RDL_DOWNLOAD".into(),
        "--print".into(),
        "after_move:RDL_OUTPUT:%(filepath)j".into(),
        "--no-simulate".into(),
    ]);
    if job.options.mode == MediaMode::AudioOnly {
        args.extend([
            "--extract-audio".into(),
            "--audio-format".into(),
            match job.options.audio_format {
                AudioFormat::Mp3 => "mp3",
                AudioFormat::M4a => "m4a",
                AudioFormat::Original => "best",
            }
            .into(),
        ]);
        if job.options.audio_format != AudioFormat::Original {
            args.extend([
                "--audio-quality".into(),
                format!("{}K", job.options.audio_bitrate),
            ]);
        }
    } else if job.options.container != Container::Auto {
        let container = match job.options.container {
            Container::Mp4 => "mp4",
            Container::Mkv => "mkv",
            Container::Webm => "webm",
            _ => unreachable!(),
        };
        args.extend([
            "--merge-output-format".into(),
            container.into(),
            "--remux-video".into(),
            container.into(),
        ]);
    }
    args.extend(["--".into(), job.url.clone()]);
    Ok(args)
}

pub async fn start(
    job: DownloadJob,
    dependencies: ResolvedDependencies,
    events: EventSender,
    cancellation: CancellationToken,
) -> Result<CompletedDownload, EngineError> {
    if cancellation.is_cancelled() {
        return Err(EngineError::Canceled);
    }
    formats::validate_options(&job.options)?;
    let folder = destination(&job);
    tokio::fs::create_dir_all(&folder).await?;
    let args = arguments(&job, &dependencies)?;
    // Re-inspect with the job's own output/format options, without downloading media.
    let mut preflight = args.clone();
    preflight.splice(
        preflight.len() - 2..preflight.len() - 2,
        [
            "--simulate".into(),
            "--dump-single-json".into(),
            "--no-progress".into(),
        ],
    );
    let bytes = super::process::capture(
        &dependencies.yt_dlp,
        &preflight,
        cancellation.clone(),
        Duration::from_secs(150),
    )
    .await?;
    let info = String::from_utf8_lossy(&bytes)
        .lines()
        .rev()
        .find_map(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .filter(|v| v.is_object())
        })
        .ok_or_else(|| {
            EngineError::Message("The download preflight did not return media metadata.".into())
        })?;
    let preview = metadata::parse_preview(&job.url, &info, 1)?;
    formats::validate_exact(&job.options, &preview.formats)?;
    if let Some(expected) = &job.media_id {
        if preview.id.as_ref().is_some_and(|id| id != expected) {
            return Err(EngineError::Message(
                "This URL now points to a different media item. Inspect it again.".into(),
            ));
        }
    }
    let _write_probe = tempfile::NamedTempFile::new_in(&folder)
        .map_err(|e| EngineError::Message(format!("Destination is not writable: {e}")))?;
    let predicted = predicted_output(&job, &info);
    let _output_lock = if let Some(output) = &predicted {
        Some(
            crate::storage::lock_output(
                &crate::storage::data_dir().map_err(|e| EngineError::Message(e.to_string()))?,
                output,
            )
            .map_err(|e| EngineError::Message(e.to_string()))?,
        )
    } else {
        None
    };
    if let Some(output) = predicted.filter(|p| p.is_file()) {
        if tokio::fs::metadata(&output).await?.len() == 0 {
            return Err(EngineError::Message(
                "An empty output file already exists. Rename or remove it before retrying.".into(),
            ));
        }
        return Ok(CompletedDownload {
            paths: vec![std::path::absolute(output)?],
            skipped: true,
        });
    }
    tokio::fs::create_dir_all(partial_directory(&job)).await?;
    let mut process = ProcessTree::spawn(&dependencies.yt_dlp, &args)?;
    let stdout = process.child.stdout.take().unwrap();
    let stderr = process.child.stderr.take().unwrap();
    let outputs = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
    let logs = Arc::new(Mutex::new(Vec::<String>::new()));
    let send = |event: EngineEvent| {
        let _ = events.try_send((job.id, job.attempt, event));
    };
    let out_paths = outputs.clone();
    let stderr_paths = outputs.clone();
    let stdout_logs = logs.clone();
    let stderr_logs = logs.clone();
    let handle =
        |line: &str, paths: &Arc<Mutex<Vec<PathBuf>>>, retained: &Arc<Mutex<Vec<String>>>| {
            if let Some(event) = protocol::parse_record(line) {
                if let EngineEvent::Output(path) = &event {
                    let mut paths = paths.lock().unwrap();
                    if !paths.contains(path) {
                        paths.push(path.clone());
                    }
                }
                send(event);
            } else if !line.trim().is_empty() {
                let line = redact(line).chars().take(1000).collect::<String>();
                let mut logs = retained.lock().unwrap();
                if logs.len() >= 160 {
                    logs.remove(0);
                }
                logs.push(line.clone());
                send(EngineEvent::Log(line));
            }
        };
    let run = async {
        let (stdout, stderr, status) = tokio::join!(
            drain_records(stdout, |line| handle(line, &out_paths, &stdout_logs)),
            drain_records(stderr, |line| handle(line, &stderr_paths, &stderr_logs)),
            process.child.wait()
        );
        stdout?;
        stderr?;
        let status = status?;
        if !status.success() {
            let tail = logs
                .lock()
                .unwrap()
                .iter()
                .rev()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            return Err(EngineError::Message(format!(
                "Download failed ({status}). {tail}"
            )));
        }
        let paths = outputs.lock().unwrap().clone();
        if paths.is_empty() {
            return Err(EngineError::Message(
                "The downloader exited without confirming an output file. Review the log.".into(),
            ));
        }
        for path in &paths {
            let metadata = tokio::fs::metadata(path).await.map_err(|_| {
                EngineError::Message(format!("The final file is missing: {}", path.display()))
            })?;
            if !metadata.is_file() || metadata.len() == 0 {
                return Err(EngineError::Message(
                    "The output is missing or empty.".into(),
                ));
            }
        }
        let _ = tokio::fs::remove_dir(partial_directory(&job)).await;
        Ok(CompletedDownload {
            paths,
            skipped: false,
        })
    };
    tokio::select! {
        _ = cancellation.cancelled() => Err(EngineError::Canceled),
        result = run => result,
    }
}

pub fn partial_directory(job: &DownloadJob) -> PathBuf {
    destination(job)
        .join(".rusty-dlp-parts")
        .join(job.id.to_string())
}

pub fn predicted_output(job: &DownloadJob, info: &serde_json::Value) -> Option<PathBuf> {
    let mut path = PathBuf::from(
        info["_filename"]
            .as_str()
            .or_else(|| info["filename"].as_str())?,
    );
    if job.options.mode == MediaMode::AudioOnly {
        match job.options.audio_format {
            AudioFormat::Mp3 => {
                path.set_extension("mp3");
            }
            AudioFormat::M4a => {
                path.set_extension("m4a");
            }
            AudioFormat::Original => {
                // A combined source may be extracted to a different audio extension. Let yt-dlp decide it.
                return None;
            }
        }
    } else {
        match job.options.container {
            Container::Mp4 => {
                path.set_extension("mp4");
            }
            Container::Mkv => {
                path.set_extension("mkv");
            }
            Container::Webm => {
                path.set_extension("webm");
            }
            _ => {}
        }
    }
    Some(path)
}
