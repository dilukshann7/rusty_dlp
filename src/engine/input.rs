#[derive(Debug)]
pub struct ParsedUrls {
    pub urls: Vec<String>,
    pub errors: Vec<(usize, String)>,
}

pub fn parse_urls(input: &str) -> ParsedUrls {
    let mut parsed = ParsedUrls {
        urls: Vec::new(),
        errors: Vec::new(),
    };
    for (index, line) in input.trim_start_matches('\u{feff}').lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match url::Url::parse(line) {
            Ok(url)
                if matches!(url.scheme(), "http" | "https")
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none() =>
            {
                if !parsed.urls.contains(&url.to_string()) {
                    parsed.urls.push(url.to_string());
                }
            }
            _ => parsed.errors.push((
                index + 1,
                "Expected an HTTP(S) URL without embedded credentials.".into(),
            )),
        }
    }
    parsed
}

pub fn parse_range(input: &str) -> Result<(usize, usize), String> {
    let normalized = input.trim().replace(':', "-");
    let mut pieces = normalized.split('-');
    let start = pieces
        .next()
        .unwrap_or_default()
        .trim()
        .parse::<usize>()
        .map_err(|_| "Use a range such as 2-10.")?;
    let end = pieces
        .next()
        .unwrap_or_default()
        .trim()
        .parse::<usize>()
        .map_err(|_| "Use a range such as 2-10.")?;
    if start == 0 || end < start || pieces.next().is_some() {
        return Err("Ranges start at 1 and must be in ascending order.".into());
    }
    Ok((start, end))
}

pub fn collection_url(input: &str) -> bool {
    url::Url::parse(input).ok().is_some_and(|u| {
        u.path().contains("playlist")
            || u.query_pairs().any(|(key, _)| key == "list")
            || u.path().starts_with("/@")
            || u.path().starts_with("/channel/")
    })
}

pub fn video_context(input: &str) -> bool {
    url::Url::parse(input).ok().is_some_and(|u| {
        u.query_pairs().any(|(key, _)| key == "v")
            || (u.host_str() == Some("youtu.be") && u.path().len() > 1)
            || ["/watch", "/shorts/", "/live/", "/embed/"]
                .iter()
                .any(|prefix| u.path().starts_with(prefix))
    })
}

pub fn safe_component(value: &str) -> String {
    let value: String = value
        .chars()
        .map(|c| {
            if c.is_control() || "<>:\"/\\|?*%".contains(c) {
                '_'
            } else {
                c
            }
        })
        .take(100)
        .collect();
    let value = value.trim_matches([' ', '.']);
    let stem = value
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if value.is_empty()
        || matches!(
            stem.as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        )
    {
        format!("_{value}")
    } else {
        value.into()
    }
}

/// Avoid retaining signed URLs and query tokens in diagnostics.
pub fn redact(line: &str) -> String {
    line.split_whitespace()
        .map(|word| {
            if (word.starts_with("http://") || word.starts_with("https://"))
                && let Ok(mut url) = url::Url::parse(word)
            {
                url.set_query(None);
                url.set_fragment(None);
                let _ = url.set_username("");
                let _ = url.set_password(None);
                return url.to_string();
            }
            word.chars().take(300).collect()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
