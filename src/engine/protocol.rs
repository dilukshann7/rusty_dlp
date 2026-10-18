use crate::model::{EngineEvent, JobState, Progress};
use serde_json::Value;
use std::path::PathBuf;

pub fn parse_record(line: &str) -> Option<EngineEvent> {
    if let Some(payload) = line.strip_prefix("RDL_PROGRESS:") {
        let value: Value = serde_json::from_str(payload).ok()?;
        if !value.is_object() {
            return None;
        }
        Some(EngineEvent::Progress(Progress {
            downloaded: value["downloaded_bytes"].as_u64().unwrap_or(0),
            total: value["total_bytes"]
                .as_u64()
                .or_else(|| value["total_bytes_estimate"].as_u64()),
            speed: value["speed"].as_f64(),
            eta: value["eta"].as_f64(),
        }))
    } else if line.starts_with("RDL_POST:") {
        Some(EngineEvent::Stage(JobState::PostProcessing))
    } else if line == "RDL_DOWNLOAD" {
        Some(EngineEvent::Stage(JobState::Downloading))
    } else if let Some(payload) = line.strip_prefix("RDL_OUTPUT:") {
        serde_json::from_str::<String>(payload)
            .ok()
            .map(|s| EngineEvent::Output(PathBuf::from(s)))
    } else {
        None
    }
}
