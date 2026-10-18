use std::{collections::BTreeMap, fmt, path::PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! choices {
    ($name:ident { $($variant:ident => $label:literal),+ $(,)? }) => {
        impl $name { pub const ALL: &'static [Self] = &[$(Self::$variant),+]; }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(match self { $(Self::$variant => $label),+ })
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MediaMode {
    #[default]
    VideoAudio,
    VideoOnly,
    AudioOnly,
}
choices!(MediaMode { VideoAudio => "Video + audio", VideoOnly => "Video only", AudioOnly => "Audio only" });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Container {
    #[default]
    Auto,
    Mp4,
    Mkv,
    Webm,
}
choices!(Container { Auto => "Automatic", Mp4 => "MP4", Mkv => "MKV", Webm => "WebM" });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AudioFormat {
    #[default]
    Original,
    Mp3,
    M4a,
}
choices!(AudioFormat { Original => "Original stream", Mp3 => "MP3", M4a => "M4A" });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeChoice {
    #[default]
    Dark,
    Light,
    System,
}
choices!(ThemeChoice { Dark => "Dark", Light => "Light", System => "System" });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectScope {
    Video,
    Playlist,
}
choices!(InspectScope { Video => "This video", Playlist => "Playlist / collection" });

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ToolKind {
    YtDlp,
    Ffmpeg,
    Deno,
}
choices!(ToolKind { YtDlp => "yt-dlp", Ffmpeg => "FFmpeg + ffprobe", Deno => "Deno" });

impl ToolKind {
    pub fn executable(self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp",
            Self::Ffmpeg => "ffmpeg",
            Self::Deno => "deno",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp",
            Self::Ffmpeg => "ffmpeg",
            Self::Deno => "deno",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DownloadOptions {
    pub mode: MediaMode,
    pub max_height: Option<u32>,
    pub exact_format: Option<String>,
    pub container: Container,
    pub audio_format: AudioFormat,
    pub audio_bitrate: u32,
    pub destination: PathBuf,
    pub filename_template: String,
    pub playlist_folder: bool,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            mode: MediaMode::VideoAudio,
            max_height: None,
            exact_format: None,
            container: Container::Auto,
            audio_format: AudioFormat::Original,
            audio_bitrate: 192,
            destination: directories::UserDirs::new()
                .and_then(|d| d.download_dir().map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from("downloads")),
            filename_template: "%(title).160B [%(id)s].%(ext)s".into(),
            playlist_folder: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub options: DownloadOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: ThemeChoice,
    pub concurrency: usize,
    pub defaults: DownloadOptions,
    pub tool_paths: BTreeMap<ToolKind, PathBuf>,
    pub presets: Vec<Preset>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Dark,
            concurrency: 2,
            defaults: DownloadOptions::default(),
            tool_paths: BTreeMap::new(),
            presets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Preparing,
    Downloading,
    PostProcessing,
    Completed,
    Failed,
    Canceled,
    Skipped,
    Interrupted,
}
impl JobState {
    pub fn active(self) -> bool {
        matches!(
            self,
            Self::Preparing | Self::Downloading | Self::PostProcessing
        )
    }
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Canceled | Self::Skipped
        )
    }
    pub fn retryable(self) -> bool {
        matches!(self, Self::Failed | Self::Canceled | Self::Interrupted)
    }
}
choices!(JobState {
    Queued => "Queued", Preparing => "Preparing", Downloading => "Downloading", PostProcessing => "Post-processing",
    Completed => "Completed", Failed => "Failed", Canceled => "Canceled", Skipped => "Skipped", Interrupted => "Interrupted"
});

#[derive(Debug, Clone, Default)]
pub struct Progress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed: Option<f64>,
    pub eta: Option<f64>,
}
impl Progress {
    pub fn percent(&self) -> Option<f32> {
        self.total
            .filter(|v| *v > 0)
            .map(|v| ((self.downloaded as f64 / v as f64) * 100.0).clamp(0.0, 100.0) as f32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadJob {
    pub id: Uuid,
    pub attempt: Uuid,
    pub url: String,
    pub title: String,
    pub media_id: Option<String>,
    pub playlist_title: Option<String>,
    pub playlist_index: Option<usize>,
    pub options: DownloadOptions,
    pub state: JobState,
    pub outputs: Vec<PathBuf>,
    pub error: Option<String>,
    pub created_at: u64,
    #[serde(skip)]
    pub progress: Progress,
    #[serde(skip)]
    pub logs: Vec<String>,
}
impl DownloadJob {
    pub fn new(url: String, title: String, options: DownloadOptions) -> Self {
        Self {
            id: Uuid::new_v4(),
            attempt: Uuid::new_v4(),
            url,
            title,
            media_id: None,
            playlist_title: None,
            playlist_index: None,
            options,
            state: JobState::Queued,
            outputs: Vec::new(),
            error: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            progress: Progress::default(),
            logs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub id: String,
    pub extension: String,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub language: Option<String>,
    pub size: Option<u64>,
}
impl FormatInfo {
    pub fn has_video(&self) -> bool {
        self.video_codec.as_deref().is_some_and(|s| s != "none")
    }
    pub fn has_audio(&self) -> bool {
        self.audio_codec.as_deref().is_some_and(|s| s != "none")
    }
}

#[derive(Debug, Clone)]
pub struct PlaylistEntry {
    pub id: Option<String>,
    pub url: Option<String>,
    pub title: String,
    pub index: usize,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub unavailable: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct MediaPreview {
    pub url: String,
    pub id: Option<String>,
    pub title: String,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub formats: Vec<FormatInfo>,
    pub entries: Vec<PlaylistEntry>,
    pub is_playlist: bool,
    pub next_page: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    Stage(JobState),
    Progress(Progress),
    Output(PathBuf),
    Log(String),
}

#[derive(Debug, Clone)]
pub struct CompletedDownload {
    pub paths: Vec<PathBuf>,
    pub skipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    pub schema_version: u32,
    pub settings: Settings,
    pub queue: crate::queue::Queue,
}
impl Default for PersistedState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            settings: Settings::default(),
            queue: crate::queue::Queue::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Canceled")]
    Canceled,
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid metadata: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
}

pub type EventSender = std::sync::mpsc::SyncSender<(Uuid, Uuid, EngineEvent)>;
