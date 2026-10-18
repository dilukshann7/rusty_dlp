use crate::{
    engine::{
        self,
        dependencies::{self, DependencyReport, InstallProgress, InstalledDependency},
        metadata::InspectionRequest,
    },
    model::*,
    storage::{self, StorageCommand, StorageWriter},
};
use iced::{
    Subscription, Task, Theme,
    widget::{image, text_editor},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    NewDownload,
    Queue,
    History,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Screen),
    Tick,
    CloseRequested,
    ShutdownFinished(Result<(), String>),
    DismissNotice,
    RetrySaving,
    SystemTheme(iced::theme::Mode),
    EditUrls(text_editor::Action),
    ImportUrls,
    Imported(Result<Option<String>, String>),
    Inspect,
    Scope(InspectScope),
    Inspected(Uuid, InspectionRequest, Result<MediaPreview, String>),
    LoadMore(usize),
    SelectPreview(String, bool),
    SelectEntry(usize, usize, bool),
    SelectAll(usize, bool),
    RangeInput(String),
    SelectRange(usize),
    InspectEntry(String),
    Mode(MediaMode),
    Quality(Option<u32>),
    Container(Container),
    AudioFormat(AudioFormat),
    Bitrate(u32),
    ExactVideo(String),
    ExactAudio(String),
    Destination(String),
    BrowseDestination(bool),
    FolderChosen(bool, Option<PathBuf>),
    Template(String),
    PlaylistFolder(bool),
    PresetName(String),
    SavePreset,
    ApplyPreset(String),
    DeletePreset(String),
    SaveDefaults,
    Enqueue,
    StartQueue,
    HoldQueue,
    CancelJob(Uuid),
    RetryJob(Uuid),
    RemoveJob(Uuid),
    ClearFinished,
    JobFinished(Uuid, Uuid, Result<CompletedDownload, String>),
    ToggleLog(Uuid),
    OpenFile(PathBuf),
    OpenFolder(PathBuf),
    Opened(Result<(), String>),
    Theme(ThemeChoice),
    ReduceTransparency(bool),
    Concurrency(usize),
    BrowseTool(ToolKind),
    ToolChosen(ToolKind, Option<PathBuf>),
    ToolPath(ToolKind, String),
    UseAutomaticTool(ToolKind),
    Discover,
    DependenciesFound(DependencyReport),
    Install(ToolKind, bool),
    CancelInstall,
    Installed(Result<InstalledDependency, String>),
    Thumbnail(String, Result<Vec<u8>, String>),
}

pub struct App {
    pub state: PersistedState,
    pub screen: Screen,
    pub urls: text_editor::Content,
    pub scope: InspectScope,
    pub previews: Vec<MediaPreview>,
    pub selected_previews: HashSet<String>,
    pub options: DownloadOptions,
    pub exact_video: String,
    pub exact_audio: String,
    pub range_input: String,
    pub preset_name: String,
    pub selected_preset: Option<String>,
    pub dependencies: DependencyReport,
    pub discovering: bool,
    pub installing: Option<ToolKind>,
    pub installation: BTreeMap<ToolKind, InstallProgress>,
    pub notice: Option<String>,
    pub thumbnails: HashMap<String, image::Handle>,
    pub expanded_logs: HashSet<Uuid>,
    pub closing: bool,
    pub storage_blocked: bool,
    pub storage_failed: bool,
    pub inspecting: Option<CancellationToken>,
    pub system_light: bool,
    root: PathBuf,
    writer: StorageWriter,
    dirty_since: Option<Instant>,
    engine_tx: EventSender,
    engine_rx: mpsc::Receiver<(Uuid, Uuid, EngineEvent)>,
    install_tx: mpsc::Sender<InstallProgress>,
    install_rx: mpsc::Receiver<InstallProgress>,
    active: HashMap<Uuid, (Uuid, CancellationToken)>,
    install_cancel: Option<CancellationToken>,
    inspection_generation: Uuid,
    pending_inspections: VecDeque<InspectionRequest>,
    retired_inspections: HashSet<Uuid>,
    shutdown_started: bool,
}

impl App {
    pub fn new(root: PathBuf) -> (Self, Task<Message>) {
        let (mut state, notice, storage_blocked) = match storage::load_from(&root) {
            Ok((state, warning)) => (state, warning, false),
            Err(error) => (
                PersistedState::default(),
                Some(format!(
                    "{error} Saving is disabled to preserve existing files."
                )),
                true,
            ),
        };
        state.queue.recover();
        let settings = state.settings.clone();
        let options = state.settings.defaults.clone();
        let (engine_tx, engine_rx) = mpsc::sync_channel(512);
        let (install_tx, install_rx) = mpsc::channel();
        let app = Self {
            state,
            screen: Screen::NewDownload,
            urls: text_editor::Content::new(),
            scope: InspectScope::Video,
            previews: Vec::new(),
            selected_previews: HashSet::new(),
            options,
            exact_video: "Automatic".into(),
            exact_audio: "Automatic".into(),
            range_input: String::new(),
            preset_name: String::new(),
            selected_preset: None,
            dependencies: DependencyReport::default(),
            discovering: true,
            installing: None,
            installation: BTreeMap::new(),
            notice,
            thumbnails: HashMap::new(),
            expanded_logs: HashSet::new(),
            closing: false,
            storage_blocked,
            storage_failed: false,
            inspecting: None,
            system_light: matches!(dark_light::detect(), Ok(dark_light::Mode::Light)),
            writer: StorageWriter::start(root.clone()),
            root,
            dirty_since: None,
            engine_tx,
            engine_rx,
            install_tx,
            install_rx,
            active: HashMap::new(),
            install_cancel: None,
            inspection_generation: Uuid::new_v4(),
            pending_inspections: VecDeque::new(),
            retired_inspections: HashSet::new(),
            shutdown_started: false,
        };
        (
            app,
            Task::perform(
                dependencies::discover_dependencies(settings),
                Message::DependenciesFound,
            ),
        )
    }

    pub fn theme(&self) -> Theme {
        let light = self.state.settings.theme == ThemeChoice::Light
            || (self.state.settings.theme == ThemeChoice::System && self.system_light);
        crate::ui::material::theme(light)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick),
            iced::event::listen_with(|event, _, _| {
                if matches!(
                    event,
                    iced::Event::Window(iced::window::Event::CloseRequested)
                ) {
                    Some(Message::CloseRequested)
                } else {
                    None
                }
            }),
            iced::system::theme_changes().map(Message::SystemTheme),
        ])
    }

    pub fn tools_busy(&self) -> bool {
        !self.active.is_empty()
            || self.inspecting.is_some()
            || !self.retired_inspections.is_empty()
            || self.installing.is_some()
            || self.discovering
            || self.closing
    }
    pub fn data_directory(&self) -> &std::path::Path {
        &self.root
    }
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    fn dirty(&mut self) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(Instant::now());
        }
    }
    fn persist(&mut self) {
        self.dirty_since = None;
        if !self.storage_blocked {
            if self
                .writer
                .sender
                .send(StorageCommand::Save(self.state.clone()))
                .is_err()
            {
                self.notice = Some(
                    "The storage writer stopped. Restart the application before adding more work."
                        .into(),
                );
                self.storage_blocked = true;
            }
        }
    }
    fn invalidate_inspection(&mut self) {
        if let Some(token) = self.inspecting.take() {
            token.cancel();
            self.retired_inspections.insert(self.inspection_generation);
        }
        self.inspection_generation = Uuid::new_v4();
        self.pending_inspections.clear();
        self.previews.clear();
        self.selected_previews.clear();
        self.clear_exact();
    }
    fn clear_exact(&mut self) {
        self.exact_video = "Automatic".into();
        self.exact_audio = "Automatic".into();
        self.options.exact_format = None;
    }

    fn next_inspection(&mut self) -> Task<Message> {
        if self.closing {
            return Task::none();
        }
        let Some(request) = self.pending_inspections.pop_front() else {
            return Task::none();
        };
        let dependencies = match self.dependencies.resolve() {
            Ok(d) => d,
            Err(error) => {
                self.notice = Some(error.to_string());
                return Task::none();
            }
        };
        let generation = self.inspection_generation;
        let cancel = CancellationToken::new();
        self.inspecting = Some(cancel.clone());
        let returned_request = request.clone();
        Task::perform(
            async move {
                engine::metadata::inspect(request, dependencies, cancel)
                    .await
                    .map_err(|e| e.to_string())
            },
            move |result| Message::Inspected(generation, returned_request.clone(), result),
        )
    }

    fn enqueue(&mut self) {
        if self.storage_blocked || self.storage_failed {
            self.notice =
                Some("Saving is disabled. Restore or reset local state before adding jobs.".into());
            return;
        }
        if let Err(error) = engine::formats::validate_options(&self.options) {
            self.notice = Some(error.to_string());
            return;
        }
        let exact_id = |choice: &str| {
            choice
                .split_whitespace()
                .next()
                .filter(|s| *s != "Automatic")
                .map(str::to_string)
        };
        let mut options = self.options.clone();
        options.exact_format = match options.mode {
            MediaMode::AudioOnly => exact_id(&self.exact_audio),
            MediaMode::VideoOnly => exact_id(&self.exact_video),
            MediaMode::VideoAudio => {
                match (exact_id(&self.exact_video), exact_id(&self.exact_audio)) {
                    (Some(video), Some(audio)) => Some(format!("{video}+{audio}")),
                    (Some(video), None) => Some(video),
                    (None, Some(_)) => {
                        self.notice = Some("Choose an exact video stream too, or leave both selectors on Automatic.".into());
                        return;
                    }
                    _ => None,
                }
            }
        };
        if options.exact_format.is_some()
            && (self.previews.len() != 1 || self.previews[0].is_playlist)
        {
            self.notice = Some("Exact format IDs apply to one inspected video. Use a quality policy for multiple videos or playlists.".into());
            return;
        }
        if options.exact_format.is_some() {
            if let Err(error) = engine::formats::validate_exact(&options, &self.previews[0].formats)
            {
                self.notice = Some(error.to_string());
                return;
            }
        }
        let mut added = 0;
        let mut errors = Vec::new();
        for preview in &self.previews {
            if !self.selected_previews.contains(&preview.url) {
                continue;
            }
            if preview.is_playlist {
                for entry in preview
                    .entries
                    .iter()
                    .filter(|e| e.selected && e.unavailable.is_none())
                {
                    let Some(url) = &entry.url else {
                        continue;
                    };
                    let mut job =
                        DownloadJob::new(url.clone(), entry.title.clone(), options.clone());
                    job.options.exact_format = None;
                    job.media_id = entry.id.clone();
                    job.playlist_title = Some(preview.title.clone());
                    job.playlist_index = Some(entry.index);
                    match self.state.queue.enqueue(job) {
                        Ok(()) => added += 1,
                        Err(e) => errors.push(e),
                    }
                }
            } else {
                let mut job =
                    DownloadJob::new(preview.url.clone(), preview.title.clone(), options.clone());
                job.media_id = preview.id.clone();
                match self.state.queue.enqueue(job) {
                    Ok(()) => added += 1,
                    Err(e) => errors.push(e),
                }
            }
        }
        if added > 0 {
            self.screen = Screen::Queue;
            self.persist();
        }
        self.notice = Some(if errors.is_empty() {
            if added == 0 {
                "Select at least one inspected item.".into()
            } else {
                format!("Added {added} item(s). Use Start / resume to run the queue.")
            }
        } else {
            format!(
                "Added {added} item(s). {}",
                errors.into_iter().take(3).collect::<Vec<_>>().join(" · ")
            )
        });
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        // After close, only completion and tick events may advance cleanup.
        if self.closing
            && !matches!(
                &message,
                Message::Tick
                    | Message::JobFinished(..)
                    | Message::Installed(..)
                    | Message::Inspected(..)
                    | Message::DependenciesFound(..)
                    | Message::ShutdownFinished(..)
                    | Message::CloseRequested
            )
        {
            return Task::none();
        }
        match message {
            Message::Navigate(screen) => self.screen = screen,
            Message::DismissNotice => self.notice = None,
            Message::RetrySaving => self.persist(),
            Message::SystemTheme(mode) => self.system_light = mode == iced::theme::Mode::Light,
            Message::Tick => return self.tick(),
            Message::CloseRequested => {
                if !self.closing {
                    self.closing = true;
                    self.state.queue.running = false;
                    for (_, token) in self.active.values() {
                        token.cancel();
                    }
                    if let Some(token) = &self.install_cancel {
                        token.cancel();
                    }
                    if let Some(token) = &self.inspecting {
                        token.cancel();
                    }
                    self.pending_inspections.clear();
                }
                return self.tick();
            }
            Message::ShutdownFinished(result) => match result {
                Ok(()) => return iced::exit(),
                Err(error) => {
                    self.closing = false;
                    self.shutdown_started = false;
                    self.notice = Some(format!(
                        "Could not save before exit: {error}. State files were preserved. Fix the folder permissions and close again."
                    ));
                }
            },
            Message::EditUrls(action) => {
                let edit = action.is_edit();
                self.urls.perform(action);
                if edit {
                    self.invalidate_inspection();
                }
            }
            Message::ImportUrls => {
                return Task::perform(
                    async {
                        let Some(file) = rfd::AsyncFileDialog::new()
                            .add_filter("Text files", &["txt", "list"])
                            .pick_file()
                            .await
                        else {
                            return Ok(None);
                        };
                        let metadata = tokio::fs::metadata(file.path())
                            .await
                            .map_err(|e| e.to_string())?;
                        if metadata.len() > 5 * 1024 * 1024 {
                            return Err("URL lists must be smaller than 5 MB.".into());
                        }
                        tokio::fs::read_to_string(file.path())
                            .await
                            .map(Some)
                            .map_err(|e| e.to_string())
                    },
                    Message::Imported,
                );
            }
            Message::Imported(result) => match result {
                Ok(Some(text)) => {
                    self.invalidate_inspection();
                    self.urls = text_editor::Content::with_text(&text);
                }
                Err(error) => self.notice = Some(error),
                _ => {}
            },
            Message::Scope(scope) => {
                self.scope = scope;
                self.invalidate_inspection();
            }
            Message::Inspect => {
                if self.installing.is_some() || self.discovering {
                    self.notice = Some("Wait for dependency setup to finish.".into());
                    return Task::none();
                }
                let parsed = engine::input::parse_urls(&self.urls.text());
                self.invalidate_inspection();
                if !parsed.errors.is_empty() {
                    self.notice = Some(
                        parsed
                            .errors
                            .iter()
                            .take(5)
                            .map(|(line, why)| format!("Line {line}: {why}"))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                }
                self.pending_inspections = parsed
                    .urls
                    .into_iter()
                    .map(|url| {
                        let scope = if engine::input::collection_url(&url) {
                            let pure_collection = !engine::input::video_context(&url);
                            if pure_collection {
                                InspectScope::Playlist
                            } else {
                                self.scope
                            }
                        } else {
                            self.scope
                        };
                        InspectionRequest {
                            url,
                            scope,
                            page_start: 1,
                        }
                    })
                    .collect();
                if self.pending_inspections.is_empty() {
                    self.notice = Some("Paste at least one valid HTTP(S) URL.".into());
                }
                return self.next_inspection();
            }
            Message::Inspected(generation, request, result) => {
                if generation != self.inspection_generation {
                    self.retired_inspections.remove(&generation);
                    return Task::none();
                }
                self.inspecting = None;
                let mut tasks = Vec::new();
                match result {
                    Ok(preview) if !self.closing => {
                        let thumbnails = std::iter::once(preview.thumbnail.clone()).chain(
                            preview
                                .entries
                                .iter()
                                .filter_map(|e| e.thumbnail.clone())
                                .map(Some),
                        );
                        for thumbnail in thumbnails.flatten().take(51) {
                            if !self.thumbnails.contains_key(&thumbnail)
                                && self.thumbnails.len() < 150
                            {
                                let url = thumbnail.clone();
                                tasks.push(Task::perform(fetch_thumbnail(thumbnail), move |r| {
                                    Message::Thumbnail(url.clone(), r)
                                }));
                            }
                        }
                        self.selected_previews.insert(preview.url.clone());
                        if request.page_start > 1 {
                            if let Some(existing) =
                                self.previews.iter_mut().find(|p| p.url == preview.url)
                            {
                                for entry in preview.entries {
                                    if !existing.entries.iter().any(|e| e.index == entry.index) {
                                        existing.entries.push(entry);
                                    }
                                }
                                existing.next_page = preview.next_page;
                            }
                        } else {
                            self.previews.push(preview);
                        }
                    }
                    Err(error) if !self.closing => {
                        self.notice = Some(format!(
                            "Could not inspect {}: {}",
                            engine::input::redact(&request.url),
                            engine::input::redact(&error)
                        ))
                    }
                    _ => {}
                }
                tasks.push(self.next_inspection());
                return Task::batch(tasks);
            }
            Message::LoadMore(index) => {
                if self.inspecting.is_none() {
                    if let Some(preview) = self.previews.get(index) {
                        if let Some(page_start) = preview.next_page {
                            self.pending_inspections.push_back(InspectionRequest {
                                url: preview.url.clone(),
                                scope: InspectScope::Playlist,
                                page_start,
                            });
                            return self.next_inspection();
                        }
                    }
                }
            }
            Message::InspectEntry(url) => {
                self.invalidate_inspection();
                self.urls = text_editor::Content::with_text(&url);
                self.scope = InspectScope::Video;
                self.pending_inspections.push_back(InspectionRequest {
                    url,
                    scope: InspectScope::Video,
                    page_start: 1,
                });
                return self.next_inspection();
            }
            Message::SelectPreview(url, selected) => {
                if selected {
                    self.selected_previews.insert(url);
                } else {
                    self.selected_previews.remove(&url);
                }
            }
            Message::SelectEntry(p, e, selected) => {
                if let Some(entry) = self.previews.get_mut(p).and_then(|p| p.entries.get_mut(e)) {
                    if entry.unavailable.is_none() {
                        entry.selected = selected;
                    }
                }
            }
            Message::SelectAll(index, selected) => {
                if let Some(preview) = self.previews.get_mut(index) {
                    for entry in &mut preview.entries {
                        entry.selected = selected && entry.unavailable.is_none();
                    }
                }
            }
            Message::RangeInput(text) => self.range_input = text,
            Message::SelectRange(index) => match engine::input::parse_range(&self.range_input) {
                Ok((start, end)) => {
                    if let Some(preview) = self.previews.get_mut(index) {
                        for entry in &mut preview.entries {
                            entry.selected =
                                entry.unavailable.is_none() && (start..=end).contains(&entry.index);
                        }
                    }
                }
                Err(error) => self.notice = Some(error),
            },
            Message::Mode(mode) => {
                self.options.mode = mode;
                self.clear_exact();
            }
            Message::Quality(height) => self.options.max_height = height,
            Message::Container(container) => {
                self.options.container = container;
                self.clear_exact();
            }
            Message::AudioFormat(format) => self.options.audio_format = format,
            Message::Bitrate(bitrate) => self.options.audio_bitrate = bitrate,
            Message::ExactVideo(value) => self.exact_video = value,
            Message::ExactAudio(value) => self.exact_audio = value,
            Message::Destination(path) => self.options.destination = PathBuf::from(path),
            Message::BrowseDestination(defaults) => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    move |p| Message::FolderChosen(defaults, p),
                );
            }
            Message::FolderChosen(defaults, path) => {
                if let Some(path) = path {
                    if defaults {
                        self.state.settings.defaults.destination = path;
                        self.dirty();
                    } else {
                        self.options.destination = path;
                    }
                }
            }
            Message::Template(template) => self.options.filename_template = template,
            Message::PlaylistFolder(value) => self.options.playlist_folder = value,
            Message::PresetName(name) => self.preset_name = name,
            Message::SavePreset => {
                let name = self.preset_name.trim().chars().take(60).collect::<String>();
                if name.is_empty() {
                    self.notice = Some("Enter a name for the preset.".into());
                } else {
                    let mut options = self.options.clone();
                    options.exact_format = None;
                    self.state.settings.presets.retain(|p| p.name != name);
                    self.state.settings.presets.push(Preset {
                        name: name.clone(),
                        options,
                    });
                    self.selected_preset = Some(name);
                    self.dirty();
                }
            }
            Message::ApplyPreset(name) => {
                if let Some(preset) = self.state.settings.presets.iter().find(|p| p.name == name) {
                    self.options = preset.options.clone();
                    self.selected_preset = Some(name);
                    self.clear_exact();
                }
            }
            Message::DeletePreset(name) => {
                self.state.settings.presets.retain(|p| p.name != name);
                self.selected_preset = None;
                self.dirty();
            }
            Message::SaveDefaults => {
                if let Err(error) = engine::formats::validate_options(&self.options) {
                    self.notice = Some(error.to_string());
                } else {
                    self.state.settings.defaults = self.options.clone();
                    self.state.settings.defaults.exact_format = None;
                    self.dirty();
                    self.notice = Some("Current download options saved as defaults.".into());
                }
            }
            Message::Enqueue => self.enqueue(),
            Message::StartQueue => {
                if !self.dependencies.ready() {
                    self.notice = Some(
                        "Complete dependency setup in Settings before starting the queue.".into(),
                    );
                } else if self.storage_blocked || self.storage_failed {
                    self.notice = Some("Restore local storage before starting downloads.".into());
                } else {
                    self.state.queue.resume_all();
                    self.dirty();
                }
            }
            Message::HoldQueue => {
                self.state.queue.running = false;
                self.dirty();
            }
            Message::CancelJob(id) => {
                if let Some((_, token)) = self.active.get(&id) {
                    token.cancel();
                } else if let Some(job) = self
                    .state
                    .queue
                    .jobs
                    .iter_mut()
                    .find(|j| j.id == id && j.state == JobState::Queued)
                {
                    job.state = JobState::Canceled;
                }
                self.dirty();
            }
            Message::RetryJob(id) => {
                if let Err(error) = self.state.queue.retry(id) {
                    self.notice = Some(error);
                }
                self.dirty();
            }
            Message::RemoveJob(id) => {
                self.state
                    .queue
                    .jobs
                    .retain(|j| j.id != id || j.state.active());
                self.dirty();
            }
            Message::ClearFinished => {
                self.state.queue.jobs.retain(|j| !j.state.terminal());
                self.dirty();
            }
            Message::JobFinished(id, attempt, result) => {
                if self
                    .active
                    .get(&id)
                    .is_some_and(|(current, _)| *current == attempt)
                {
                    self.active.remove(&id);
                }
                self.state.queue.finish(id, attempt, result);
                self.persist();
            }
            Message::ToggleLog(id) => {
                if !self.expanded_logs.remove(&id) {
                    self.expanded_logs.insert(id);
                }
            }
            Message::OpenFile(path) => {
                return Task::perform(open_path(path, false), Message::Opened);
            }
            Message::OpenFolder(path) => {
                return Task::perform(open_path(path, true), Message::Opened);
            }
            Message::Opened(Err(error)) => self.notice = Some(error),
            Message::Opened(Ok(())) => {}
            Message::Theme(theme) => {
                self.state.settings.theme = theme;
                self.system_light = matches!(dark_light::detect(), Ok(dark_light::Mode::Light));
                self.dirty();
            }
            Message::Concurrency(value) => {
                self.state.settings.concurrency = value.clamp(1, 4);
                self.dirty();
            }
            Message::ReduceTransparency(value) => {
                self.state.settings.reduce_transparency = value;
                self.dirty();
            }
            Message::BrowseTool(kind) => {
                if self.tools_busy() {
                    return Task::none();
                }
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    move |p| Message::ToolChosen(kind, p),
                );
            }
            Message::ToolChosen(kind, Some(path)) => {
                if !self.tools_busy() {
                    self.state.settings.tool_paths.insert(kind, path);
                    self.dirty();
                    return self.discover();
                }
            }
            Message::ToolChosen(_, None) => {}
            Message::ToolPath(kind, path) => {
                if !self.tools_busy() {
                    if path.trim().is_empty() {
                        self.state.settings.tool_paths.remove(&kind);
                    } else {
                        self.state
                            .settings
                            .tool_paths
                            .insert(kind, PathBuf::from(path));
                    }
                    self.dirty();
                }
            }
            Message::UseAutomaticTool(kind) => {
                if !self.tools_busy() {
                    self.state.settings.tool_paths.remove(&kind);
                    self.dirty();
                    return self.discover();
                }
            }
            Message::Discover => {
                if !self.tools_busy() {
                    return self.discover();
                }
            }
            Message::DependenciesFound(report) => {
                self.dependencies = report;
                self.discovering = false;
            }
            Message::Install(kind, update) => {
                if self.tools_busy() {
                    self.notice = Some(
                        "Finish active downloads and inspections before changing dependencies."
                            .into(),
                    );
                    return Task::none();
                }
                self.state.queue.running = false;
                self.installing = Some(kind);
                let cancel = CancellationToken::new();
                self.install_cancel = Some(cancel.clone());
                self.installation.insert(
                    kind,
                    InstallProgress {
                        kind,
                        fraction: None,
                        message: "Preparing managed installation…".into(),
                    },
                );
                let events = self.install_tx.clone();
                return Task::perform(
                    async move {
                        if update {
                            let entry = tokio::select! { _ = cancel.cancelled() => return Err("Canceled".into()), result = dependencies::latest_entry(kind) => result.map_err(|e| e.to_string())? };
                            dependencies::install_entry(entry, events, cancel)
                                .await
                                .map_err(|e| e.to_string())
                        } else {
                            dependencies::install_dependency(
                                kind,
                                dependencies::target(),
                                events,
                                cancel,
                            )
                            .await
                            .map_err(|e| e.to_string())
                        }
                    },
                    Message::Installed,
                );
            }
            Message::CancelInstall => {
                if let Some(token) = &self.install_cancel {
                    token.cancel();
                }
            }
            Message::Installed(result) => {
                let kind = self.installing.take();
                self.install_cancel = None;
                match result {
                    Ok(installed) => {
                        self.state.settings.tool_paths.remove(&installed.kind);
                        self.dirty();
                        self.notice = Some(format!(
                            "{} {} installed. Other managed versions were preserved.",
                            installed.kind, installed.version
                        ));
                    }
                    Err(error) => {
                        if let Some(kind) = kind {
                            self.installation.insert(
                                kind,
                                InstallProgress {
                                    kind,
                                    fraction: None,
                                    message: format!("Not installed: {error}"),
                                },
                            );
                        }
                        self.notice = Some(error);
                    }
                }
                if !self.closing {
                    return self.discover();
                }
            }
            Message::Thumbnail(url, Ok(bytes)) => {
                if self.thumbnails.len() < 150 {
                    self.thumbnails
                        .insert(url, image::Handle::from_bytes(bytes));
                }
            }
            Message::Thumbnail(_, Err(_)) => {}
        }
        Task::none()
    }

    fn discover(&mut self) -> Task<Message> {
        self.discovering = true;
        Task::perform(
            dependencies::discover_dependencies(self.state.settings.clone()),
            Message::DependenciesFound,
        )
    }

    fn tick(&mut self) -> Task<Message> {
        for _ in 0..512 {
            let Ok((id, attempt, event)) = self.engine_rx.try_recv() else {
                break;
            };
            let important = matches!(&event, EngineEvent::Stage(_) | EngineEvent::Output(_));
            if self.state.queue.apply(id, attempt, event) && important {
                self.dirty();
            }
        }
        while let Ok(progress) = self.install_rx.try_recv() {
            self.installation.insert(progress.kind, progress);
        }
        while let Ok(outcome) = self.writer.outcomes.try_recv() {
            self.storage_failed = outcome.is_err();
            if let Err(error) = outcome {
                self.state.queue.running = false;
                self.notice = Some(format!(
                    "{error}. Queue paused; fix storage and choose Retry saving in Settings."
                ));
            }
        }
        if self
            .dirty_since
            .is_some_and(|t| t.elapsed() >= Duration::from_millis(350))
        {
            self.persist();
        }
        if self.closing {
            if self.active.is_empty()
                && self.installing.is_none()
                && self.inspecting.is_none()
                && self.retired_inspections.is_empty()
                && !self.shutdown_started
            {
                self.shutdown_started = true;
                self.persist();
                let sender = self.writer.sender.clone();
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let (tx, rx) = mpsc::channel();
                            sender
                                .send(StorageCommand::Flush(tx))
                                .map_err(|e| e.to_string())?;
                            rx.recv_timeout(Duration::from_secs(15))
                                .map_err(|e| e.to_string())?
                        })
                        .await
                        .map_err(|e| e.to_string())?
                    },
                    Message::ShutdownFinished,
                );
            }
            return Task::none();
        }
        if self.installing.is_some()
            || self.discovering
            || self.storage_blocked
            || self.storage_failed
            || !self.dependencies.ready()
        {
            return Task::none();
        }
        let jobs = self.state.queue.schedule(self.state.settings.concurrency);
        if jobs.is_empty() {
            return Task::none();
        }
        self.persist();
        let dependencies = self.dependencies.resolve().unwrap();
        let mut tasks = Vec::new();
        for job in jobs {
            let (id, attempt) = (job.id, job.attempt);
            let cancel = CancellationToken::new();
            self.active.insert(id, (attempt, cancel.clone()));
            let events = self.engine_tx.clone();
            let dependencies = dependencies.clone();
            tasks.push(Task::perform(
                async move {
                    engine::download::start(job, dependencies, events, cancel)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |r| Message::JobFinished(id, attempt, r),
            ));
        }
        Task::batch(tasks)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        for (_, token) in self.active.values() {
            token.cancel();
        }
        if let Some(token) = &self.install_cancel {
            token.cancel();
        }
        if let Some(token) = &self.inspecting {
            token.cancel();
        }
    }
}

async fn fetch_thumbnail(url: String) -> Result<Vec<u8>, String> {
    let url = url::Url::parse(&url).map_err(|e| e.to_string())?;
    if !matches!(url.scheme(), "https" | "http") {
        return Err("Unsupported thumbnail URL".into());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
            return Err("Thumbnail exceeds 2 MB".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn open_path(path: PathBuf, folder: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let target = if folder && !path.is_dir() {
            path.parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| path.clone())
        } else {
            path
        };
        if !target.exists() {
            return Err(format!("This path no longer exists: {}", target.display()));
        }
        open::that_detached(target).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
