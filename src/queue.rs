use crate::model::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub jobs: Vec<DownloadJob>,
    #[serde(skip)]
    pub running: bool,
}
impl Default for Queue {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            running: true,
        }
    }
}

impl Queue {
    fn conflicts(&self, candidate: &DownloadJob) -> bool {
        fn site(url: &str) -> String {
            let host = url::Url::parse(url)
                .ok()
                .and_then(|u| u.host_str().map(str::to_string))
                .unwrap_or_default();
            let host = host.strip_prefix("www.").unwrap_or(&host);
            if host == "youtu.be" {
                "youtube.com".into()
            } else {
                host.into()
            }
        }
        self.jobs.iter().any(|job| {
            let same_media = job.url == candidate.url
                || (job.media_id.is_some()
                    && job.media_id == candidate.media_id
                    && site(&job.url) == site(&candidate.url));
            job.id != candidate.id
                && !job.state.terminal()
                && same_media
                && job.options == candidate.options
                && job.playlist_index == candidate.playlist_index
                && (!job.options.playlist_folder || job.playlist_title == candidate.playlist_title)
        })
    }

    pub fn enqueue(&mut self, job: DownloadJob) -> Result<(), String> {
        if job.options.exact_format.is_some() && job.media_id.is_none() {
            return Err("Inspect an individual video before selecting exact formats.".into());
        }
        if self.conflicts(&job) || self.jobs.iter().any(|j| j.id == job.id) {
            return Err(format!("Already queued: {}", job.title));
        }
        self.jobs.push(job);
        Ok(())
    }

    pub fn schedule(&mut self, concurrency: usize) -> Vec<DownloadJob> {
        if !self.running {
            return Vec::new();
        }
        let active = self.jobs.iter().filter(|j| j.state.active()).count();
        let available = concurrency.clamp(1, 4).saturating_sub(active);
        let mut in_flight = self
            .jobs
            .iter()
            .filter(|j| j.state.active())
            .cloned()
            .collect::<Vec<_>>();
        let mut started = Vec::new();
        for j in &mut self.jobs {
            if started.len() >= available {
                break;
            }
            if j.state != JobState::Queued
                || in_flight
                    .iter()
                    .any(|active| same_output_namespace(active, j))
            {
                continue;
            }
            j.state = JobState::Preparing;
            j.attempt = Uuid::new_v4();
            j.progress = Progress::default();
            j.error = None;
            j.logs.clear();
            in_flight.push(j.clone());
            started.push(j.clone());
        }
        started
    }

    pub fn apply(&mut self, id: Uuid, attempt: Uuid, event: EngineEvent) -> bool {
        let Some(job) = self
            .jobs
            .iter_mut()
            .find(|j| j.id == id && j.attempt == attempt && j.state.active())
        else {
            return false;
        };
        match event {
            EngineEvent::Stage(state) if state.active() => job.state = state,
            EngineEvent::Stage(_) => return false,
            EngineEvent::Progress(progress) => {
                job.progress = progress;
            }
            EngineEvent::Output(path) => {
                if !job.outputs.contains(&path) {
                    job.outputs.push(path);
                }
            }
            EngineEvent::Log(line) => {
                if job.logs.len() >= 160 {
                    job.logs.remove(0);
                }
                job.logs.push(line.chars().take(1000).collect());
            }
        }
        true
    }

    pub fn finish(&mut self, id: Uuid, attempt: Uuid, result: Result<CompletedDownload, String>) {
        if let Some(job) = self
            .jobs
            .iter_mut()
            .find(|j| j.id == id && j.attempt == attempt && j.state.active())
        {
            match result {
                Ok(done) => {
                    job.state = if done.skipped {
                        JobState::Skipped
                    } else {
                        JobState::Completed
                    };
                    job.outputs = done.paths;
                }
                Err(error) => {
                    job.state = if error == "Canceled" {
                        JobState::Canceled
                    } else {
                        JobState::Failed
                    };
                    job.error = Some(error);
                }
            }
        }
    }

    pub fn retry(&mut self, id: Uuid) -> Result<(), String> {
        if let Some(job) = self.jobs.iter().find(|job| job.id == id) {
            if self.conflicts(job) {
                return Err("Another queued job already targets this media and output. Remove it before retrying.".into());
            }
        }
        if let Some(job) = self
            .jobs
            .iter_mut()
            .find(|j| j.id == id && j.state.retryable())
        {
            job.state = JobState::Queued;
            job.error = None;
            job.outputs.clear();
            job.attempt = Uuid::new_v4();
            self.running = true;
        }
        Ok(())
    }

    pub fn recover(&mut self) {
        self.running = false;
        for job in &mut self.jobs {
            if job.state.active() {
                job.state = JobState::Interrupted;
                job.error =
                    Some("Interrupted by the previous shutdown. Resume to continue.".into());
            }
        }
    }

    pub fn resume_all(&mut self) {
        for job in &mut self.jobs {
            if job.state == JobState::Interrupted {
                job.state = JobState::Queued;
            }
        }
        self.running = true;
    }
}

fn same_output_namespace(a: &DownloadJob, b: &DownloadJob) -> bool {
    if a.options.destination != b.options.destination
        || a.options.filename_template != b.options.filename_template
        || a.playlist_index != b.playlist_index
    {
        return false;
    }
    if a.options.playlist_folder
        && b.options.playlist_folder
        && a.playlist_title != b.playlist_title
    {
        return false;
    }
    // With an ID in the filename, independent IDs have independent output names.
    // Quality/container differences alone never isolate a shared media ID or its partial files.
    if a.options.filename_template.contains("%(id)") {
        if a.media_id.is_some() && b.media_id.is_some() {
            return a.media_id == b.media_id;
        }
        return a.url == b.url;
    }
    true
}
