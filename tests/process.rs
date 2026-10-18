#![cfg(feature = "test-support")]

use rusty_dlp::{
    engine::{
        dependencies::ResolvedDependencies,
        download,
        process::{capture, drain_records},
    },
    model::*,
};
use std::{path::PathBuf, sync::mpsc, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

fn dependencies() -> ResolvedDependencies {
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_fake-downloader"));
    ResolvedDependencies {
        yt_dlp: executable.clone(),
        ffmpeg: executable.clone(),
        ffprobe: executable.clone(),
        deno: executable,
    }
}
fn fixture_job(root: &std::path::Path, scenario: &str) -> DownloadJob {
    DownloadJob::new(
        format!("https://example.com/{scenario}"),
        "Fixture".into(),
        DownloadOptions {
            destination: root.to_path_buf(),
            ..Default::default()
        },
    )
}

#[tokio::test]
async fn fragmented_and_malformed_lines_do_not_corrupt_following_records() {
    let (mut sender, reader) = tokio::io::duplex(8);
    let writer = tokio::spawn(async move {
        for fragment in [
            "RDL_PRO",
            "GRESS:{broken}\nRDL_",
            "OUTPUT:\"C:/output.mp4\"\n",
        ] {
            sender.write_all(fragment.as_bytes()).await.unwrap();
        }
    });
    let mut lines = Vec::new();
    drain_records(reader, |line| lines.push(line.to_owned()))
        .await
        .unwrap();
    writer.await.unwrap();
    assert_eq!(
        lines,
        ["RDL_PROGRESS:{broken}", "RDL_OUTPUT:\"C:/output.mp4\""]
    );
}

#[tokio::test]
async fn postprocessing_failure_is_not_success_at_one_hundred_percent() {
    let directory = tempfile::tempdir().unwrap();
    let (sender, _receiver) = mpsc::sync_channel(512);
    let result = download::start(
        fixture_job(directory.path(), "fail"),
        dependencies(),
        sender,
        CancellationToken::new(),
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn zero_exit_without_final_output_is_not_success() {
    let directory = tempfile::tempdir().unwrap();
    let (sender, _receiver) = mpsc::sync_channel(512);
    let result = download::start(
        fixture_job(directory.path(), "missing"),
        dependencies(),
        sender,
        CancellationToken::new(),
    )
    .await;
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("without confirming")
    );
}

#[tokio::test]
async fn stderr_flood_does_not_deadlock_when_event_channel_is_full() {
    let directory = tempfile::tempdir().unwrap();
    let (sender, _receiver) = mpsc::sync_channel(2);
    let result = tokio::time::timeout(
        Duration::from_secs(20),
        download::start(
            fixture_job(directory.path(), "flood"),
            dependencies(),
            sender,
            CancellationToken::new(),
        ),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(result.paths[0].is_file());
}

#[tokio::test]
async fn cancellation_terminates_descendants_before_they_write() {
    let directory = tempfile::tempdir().unwrap();
    let (sender, _receiver) = mpsc::sync_channel(512);
    let cancellation = CancellationToken::new();
    let stop = cancellation.clone();
    let timer = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(350)).await;
        stop.cancel();
    });
    let result = download::start(
        fixture_job(directory.path(), "descendant"),
        dependencies(),
        sender,
        cancellation,
    )
    .await;
    timer.await.unwrap();
    assert!(matches!(result, Err(EngineError::Canceled)));
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert!(!directory.path().join("descendant-marker").exists());
}

#[tokio::test]
async fn canceled_inspection_does_not_launch_a_process() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let result = capture(
        &PathBuf::from("does-not-exist"),
        &[],
        cancellation,
        Duration::from_secs(1),
    )
    .await;
    assert!(matches!(result, Err(EngineError::Canceled)));
}
