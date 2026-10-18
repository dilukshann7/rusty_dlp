use rusty_dlp::{model::*, queue::Queue, storage::{load_from, save_to}};

#[test]
fn exact_formats_are_bound_to_one_media_item() {
    let options = DownloadOptions {
        exact_format: Some("137+140".into()),
        ..Default::default()
    };
    let mut queue = Queue::default();
    let mut first = DownloadJob::new("https://example.com/a".into(), "a".into(), options.clone());
    first.media_id = Some("a".into());
    assert!(queue.enqueue(first).is_ok());
    let second = DownloadJob::new("https://example.com/b".into(), "b".into(), options);
    assert!(queue.enqueue(second).is_err());
}

#[test]
fn queue_limits_concurrency_and_ignores_stale_attempts() {
    let mut queue = Queue::default();
    for i in 0..4 {
        queue
            .enqueue(DownloadJob::new(
                format!("https://example.com/{i}"),
                i.to_string(),
                DownloadOptions::default(),
            ))
            .unwrap();
    }
    let started = queue.schedule(2);
    assert_eq!(started.len(), 2);
    assert!(queue.schedule(2).is_empty());
    let first = &started[0];
    assert!(!queue.apply(
        first.id,
        uuid::Uuid::new_v4(),
        EngineEvent::Stage(JobState::Completed)
    ));
    assert!(queue.apply(
        first.id,
        first.attempt,
        EngineEvent::Stage(JobState::Downloading)
    ));
    assert_eq!(queue.jobs[0].state, JobState::Downloading);
}

#[test]
fn different_qualities_cannot_download_into_the_same_output_concurrently() {
    let mut queue = Queue::default();
    for cap in [720, 1080] {
        let options = DownloadOptions {
            max_height: Some(cap),
            ..Default::default()
        };
        queue
            .enqueue(DownloadJob::new(
                "https://example.com/a".into(),
                "a".into(),
                options,
            ))
            .unwrap();
    }
    let first = queue.schedule(2);
    assert_eq!(first.len(), 1);
    assert!(queue.schedule(2).is_empty());
    queue.finish(
        first[0].id,
        first[0].attempt,
        Ok(CompletedDownload {
            paths: vec![],
            skipped: false,
        }),
    );
    assert_eq!(queue.schedule(2).len(), 1);
}

#[test]
fn duplicate_detection_includes_options_and_destination() {
    let mut queue = Queue::default();
    let job = DownloadJob::new(
        "https://example.com/a".into(),
        "a".into(),
        DownloadOptions::default(),
    );
    queue.enqueue(job.clone()).unwrap();
    assert!(queue.enqueue(job.clone()).is_err());
    let mut alternate = job;
    alternate.id = uuid::Uuid::new_v4();
    alternate.options.max_height = Some(720);
    assert!(queue.enqueue(alternate).is_ok());
}

#[test]
fn retry_cannot_bypass_duplicate_prevention() {
    let mut queue = Queue::default();
    let mut canceled = DownloadJob::new(
        "https://example.com/a".into(),
        "a".into(),
        DownloadOptions::default(),
    );
    canceled.state = JobState::Canceled;
    let id = canceled.id;
    queue.jobs.push(canceled);
    queue
        .enqueue(DownloadJob::new(
            "https://example.com/a".into(),
            "a".into(),
            DownloadOptions::default(),
        ))
        .unwrap();
    assert!(queue.retry(id).is_err());
    assert_eq!(queue.jobs[0].state, JobState::Canceled);
}

#[test]
fn alternate_youtube_links_to_same_media_are_duplicates() {
    let mut queue = Queue::default();
    let mut first = DownloadJob::new(
        "https://www.youtube.com/watch?v=abc".into(),
        "a".into(),
        DownloadOptions::default(),
    );
    first.media_id = Some("abc".into());
    queue.enqueue(first).unwrap();
    let mut alternate = DownloadJob::new(
        "https://youtu.be/abc".into(),
        "a".into(),
        DownloadOptions::default(),
    );
    alternate.media_id = Some("abc".into());
    assert!(queue.enqueue(alternate).is_err());
}

#[test]
fn storage_recovers_backup_and_never_resumes_automatically() {
    let directory = tempfile::tempdir().unwrap();
    let mut state = PersistedState::default();
    let mut job = DownloadJob::new(
        "https://example.com/a".into(),
        "a".into(),
        DownloadOptions::default(),
    );
    job.state = JobState::Downloading;
    state.queue.jobs.push(job);
    save_to(directory.path(), &state).unwrap();
    save_to(directory.path(), &state).unwrap();
    std::fs::write(directory.path().join("state.json"), "broken").unwrap();
    let (restored, notice) = load_from(directory.path()).unwrap();
    assert!(notice.is_some());
    assert_eq!(restored.queue.jobs[0].state, JobState::Interrupted);
    assert!(!restored.queue.running);
}

#[test]
fn unwritable_state_path_reports_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("not-a-directory");
    std::fs::write(&file, "x").unwrap();
    assert!(save_to(&file, &PersistedState::default()).is_err());
}

#[test]
fn future_state_schema_cannot_be_replaced_with_an_old_backup() {
    let directory = tempfile::tempdir().unwrap();
    save_to(directory.path(), &PersistedState::default()).unwrap();
    save_to(directory.path(), &PersistedState::default()).unwrap();
    let future = b"{\"schema_version\":999}";
    std::fs::write(directory.path().join("state.json"), future).unwrap();
    assert!(load_from(directory.path()).is_err());
    assert_eq!(
        std::fs::read(directory.path().join("state.json")).unwrap(),
        future
    );
}

#[test]
fn output_lock_serializes_equivalent_final_paths() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("same.mp4");
    let first = rusty_dlp::storage::lock_output(directory.path(), &output).unwrap();
    assert!(rusty_dlp::storage::lock_output(directory.path(), &output).is_err());
    drop(first);
    assert!(rusty_dlp::storage::lock_output(directory.path(), &output).is_ok());
}

#[test]
fn single_instance_lock_is_released_when_owner_exits() {
    let directory = tempfile::tempdir().unwrap();
    let first = rusty_dlp::storage::InstanceLock::acquire(directory.path()).unwrap();
    assert!(rusty_dlp::storage::InstanceLock::acquire(directory.path()).is_err());
    drop(first);
    assert!(rusty_dlp::storage::InstanceLock::acquire(directory.path()).is_ok());
}
