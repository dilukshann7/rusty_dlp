use rusty_dlp::{
    engine::{dependencies::ResolvedDependencies, download::arguments, formats::validate_exact},
    model::*,
};
use std::path::PathBuf;

fn tools() -> ResolvedDependencies {
    ResolvedDependencies {
        yt_dlp: "tools/yt-dlp".into(),
        ffmpeg: "tools/ffmpeg".into(),
        ffprobe: "tools/ffprobe".into(),
        deno: "tools/deno".into(),
    }
}

#[test]
fn conversion_is_explicit_and_paths_stay_separate_arguments() {
    let options = DownloadOptions {
        mode: MediaMode::AudioOnly,
        audio_format: AudioFormat::Mp3,
        destination: PathBuf::from("a folder with spaces"),
        ..Default::default()
    };
    let job = DownloadJob::new(
        "https://example.com/media?a=1&b=2".into(),
        "Title".into(),
        options,
    );
    let args = arguments(&job, &tools()).unwrap();
    assert!(args.windows(2).any(|a| a == ["--audio-format", "mp3"]));
    assert!(args.windows(2).any(|a| a == ["--audio-quality", "192K"]));
    assert!(
        args.windows(2)
            .any(|a| a == ["--paths", "a folder with spaces"])
    );
    assert_eq!(args.last().unwrap(), "https://example.com/media?a=1&b=2");
    assert!(args.contains(&"--ignore-config".into()));
    assert!(args.contains(&"--no-plugin-dirs".into()));
    assert!(args.contains(&"--no-overwrites".into()));
    assert!(args.contains(&"--no-post-overwrites".into()));
}

#[test]
fn original_audio_is_extracted_from_combined_only_sources_too() {
    let options = DownloadOptions {
        mode: MediaMode::AudioOnly,
        audio_format: AudioFormat::Original,
        ..Default::default()
    };
    let job = DownloadJob::new("https://example.com/a".into(), "a".into(), options);
    let args = arguments(&job, &tools()).unwrap();
    assert!(args.contains(&"--extract-audio".into()));
    assert!(args.windows(2).any(|a| a == ["--audio-format", "best"]));
    assert!(!args.contains(&"--audio-quality".into()));
}

#[test]
fn jobs_have_separate_stable_partial_directories() {
    use rusty_dlp::engine::download::partial_directory;
    let first = DownloadJob::new(
        "https://example.com/a".into(),
        "a".into(),
        DownloadOptions::default(),
    );
    let second = DownloadJob::new(
        "https://example.com/a".into(),
        "a".into(),
        DownloadOptions::default(),
    );
    assert_ne!(partial_directory(&first), partial_directory(&second));
    let mut retry = first.clone();
    retry.attempt = uuid::Uuid::new_v4();
    assert_eq!(partial_directory(&first), partial_directory(&retry));
}

#[test]
fn predicted_conversion_path_uses_final_extension() {
    use rusty_dlp::engine::download::predicted_output;
    let options = DownloadOptions {
        mode: MediaMode::AudioOnly,
        audio_format: AudioFormat::Mp3,
        ..Default::default()
    };
    let job = DownloadJob::new("https://example.com/a".into(), "a".into(), options);
    let path = predicted_output(
        &job,
        &serde_json::json!({"_filename":"downloads/audio.webm"}),
    )
    .unwrap();
    assert_eq!(path, PathBuf::from("downloads/audio.mp3"));
}

#[test]
fn unavailable_exact_format_does_not_fall_back() {
    let options = DownloadOptions {
        exact_format: Some("137+140".into()),
        ..Default::default()
    };
    assert!(validate_exact(&options, &[]).is_err());
}

#[test]
fn playlist_jobs_use_original_indexes_in_output_names() {
    let mut job = DownloadJob::new(
        "https://example.com/a".into(),
        "a".into(),
        DownloadOptions::default(),
    );
    job.playlist_index = Some(7);
    let args = arguments(&job, &tools()).unwrap();
    let template = &args[args.iter().position(|a| a == "--output").unwrap() + 1];
    assert!(template.starts_with("0007 - "));
    assert!(template.contains("%(id)s"));
}
