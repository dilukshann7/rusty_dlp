use crate::model::*;

pub fn validate_options(options: &DownloadOptions) -> Result<(), EngineError> {
    if options.destination.as_os_str().is_empty() {
        return Err(EngineError::Message("Choose a destination folder.".into()));
    }
    let template = options.filename_template.trim();
    if template.is_empty()
        || template.contains(['/', '\\', '\n', '\r'])
        || template == ".."
        || template.contains("%(filepath)")
    {
        return Err(EngineError::Message(
            "The filename template must be a filename, without folders or line breaks.".into(),
        ));
    }
    if !template.contains("%(ext)s") {
        return Err(EngineError::Message(
            "Keep %(ext)s in the filename template so converted files have the correct extension."
                .into(),
        ));
    }
    if ![64, 96, 128, 160, 192, 256, 320].contains(&options.audio_bitrate) {
        return Err(EngineError::Message(
            "Choose a supported audio bitrate.".into(),
        ));
    }
    if options.max_height.is_some_and(|h| h == 0) {
        return Err(EngineError::Message(
            "Resolution must be greater than zero.".into(),
        ));
    }
    Ok(())
}

pub fn format_selector(options: &DownloadOptions) -> Result<String, EngineError> {
    if let Some(exact) = &options.exact_format {
        if exact.is_empty()
            || !exact
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+'))
        {
            return Err(EngineError::Message(
                "Select valid format IDs from this video's format list.".into(),
            ));
        }
        return Ok(exact.clone());
    }
    if options.mode == MediaMode::AudioOnly {
        return Ok("ba/b".into());
    }
    let cap = options
        .max_height
        .map(|h| format!("[height<={h}]"))
        .unwrap_or_default();
    let (video, audio, combined) = match options.container {
        Container::Mp4 => (
            "[ext=mp4][vcodec~='^(avc1|h264|hev1|hvc1|hevc|av01|av1)']",
            "[ext=m4a]",
            "[ext=mp4]",
        ),
        Container::Webm => ("[ext=webm]", "[ext=webm]", "[ext=webm]"),
        _ => ("", "", ""),
    };
    match options.mode {
        MediaMode::VideoAudio => Ok(format!("bv{cap}{video}+ba{audio}/b{cap}{combined}")),
        MediaMode::VideoOnly => Ok(format!("bv{cap}{video}")),
        MediaMode::AudioOnly => unreachable!(),
    }
}

pub fn validate_exact(
    options: &DownloadOptions,
    formats: &[FormatInfo],
) -> Result<(), EngineError> {
    let Some(exact) = &options.exact_format else {
        return Ok(());
    };
    let selected: Vec<&FormatInfo> = exact
        .split('+')
        .map(|id| {
            formats.iter().find(|f| f.id == id).ok_or_else(|| {
                EngineError::Message(format!(
                    "Format {id} is no longer available. Inspect the media again."
                ))
            })
        })
        .collect::<Result<_, _>>()?;
    let has_video = selected.iter().any(|f| f.has_video());
    let has_audio = selected.iter().any(|f| f.has_audio());
    if (options.mode == MediaMode::VideoAudio && (!has_video || !has_audio))
        || (options.mode == MediaMode::VideoOnly && (!has_video || has_audio))
        || (options.mode == MediaMode::AudioOnly && (!has_audio || has_video))
    {
        return Err(EngineError::Message("The selected exact streams do not match the download mode. Select video and audio separately where needed.".into()));
    }
    if selected.iter().any(|f| {
        options
            .max_height
            .zip(f.height)
            .is_some_and(|(cap, height)| height > cap)
    }) {
        return Err(EngineError::Message(
            "The exact video format exceeds the selected resolution limit.".into(),
        ));
    }
    for format in selected {
        let compatible = match options.container {
            Container::Mp4 if options.mode != MediaMode::AudioOnly => {
                if format.has_video() {
                    format.extension == "mp4"
                } else {
                    format.extension == "m4a"
                }
            }
            Container::Webm if options.mode != MediaMode::AudioOnly => format.extension == "webm",
            _ => true,
        };
        if !compatible {
            return Err(EngineError::Message("The exact streams are incompatible with the selected container. Use Automatic or choose compatible streams.".into()));
        }
    }
    Ok(())
}
