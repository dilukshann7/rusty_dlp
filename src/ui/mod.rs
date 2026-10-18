pub mod material;

use crate::{
    app::{App, Message, Screen},
    model::*,
};
use iced::{
    Alignment, Element, Fill, Length, Theme,
    widget::{self, column, container, image, row, scrollable, space, stack, text, text_editor},
};
use std::borrow::Borrow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quality(pub Option<u32>);
impl std::fmt::Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(height) => write!(f, "Up to {height}p"),
            None => f.write_str("Best available"),
        }
    }
}
const QUALITIES: &[Quality] = &[
    Quality(None),
    Quality(Some(144)),
    Quality(Some(240)),
    Quality(Some(360)),
    Quality(Some(480)),
    Quality(Some(720)),
    Quality(Some(1080)),
    Quality(Some(1440)),
    Quality(Some(2160)),
];

fn button<'a>(content: impl Into<Element<'a, Message>>) -> widget::Button<'a, Message> {
    widget::button(container(content).center_y(Fill))
        .padding([0, 12])
        .height(material::CONTROL_HEIGHT)
        .style(material::primary_button)
}
fn text_input<'a>(placeholder: &str, value: &str) -> widget::TextInput<'a, Message> {
    widget::text_input(placeholder, value)
        .padding([8, 10])
        .size(14)
        .line_height(material::CONTROL_LINE_HEIGHT)
        .style(material::input)
}
fn pick_list<'a, T, L, V>(
    options: L,
    selected: Option<V>,
    on_select: impl Fn(T) -> Message + 'a,
) -> widget::PickList<'a, T, L, V, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    widget::pick_list(options, selected, on_select)
        .padding([8, 10])
        .text_size(14)
        .text_line_height(material::CONTROL_LINE_HEIGHT)
        .style(material::select)
        .menu_style(material::select_menu)
}
fn checkbox<'a>(checked: bool) -> widget::Checkbox<'a, Message> {
    widget::checkbox(checked)
        .size(16)
        .text_size(14)
        .spacing(8)
        .style(material::check)
}
fn progress_bar<'a>(range: std::ops::RangeInclusive<f32>, value: f32) -> widget::ProgressBar<'a> {
    widget::progress_bar(range, value)
        .girth(5)
        .style(material::progress)
}

fn muted<'a>(value: impl widget::text::IntoFragment<'a>) -> widget::Text<'a> {
    text(value)
        .size(13)
        .line_height(1.4)
        .style(|theme: &Theme| text::Style {
            color: Some(material::secondary(theme)),
        })
}
fn panel<'a>(app: &App, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    let solid = app.state.settings.reduce_transparency;
    container(content)
        .padding(12)
        .width(Fill)
        .style(move |theme| material::glass(theme, solid))
        .into()
}
fn field<'a>(
    label: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![muted(label), control.into()]
        .spacing(4)
        .width(Fill)
        .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let theme = app.theme();
    let solid = app.state.settings.reduce_transparency;
    if app.closing {
        return container(panel(
            app,
            column![
                text("Finishing safely…").size(24).font(material::STRONG),
                muted("Stopping downloads and saving your queue."),
            ]
            .spacing(12),
        ))
        .padding(32)
        .center_x(Fill)
        .center_y(Fill)
        .into();
    }
    let active = app.active_count();
    let queued = app
        .state
        .queue
        .jobs
        .iter()
        .filter(|j| j.state == JobState::Queued)
        .count();
    let mut sidebar = column![
        row![
            material::brand(),
            text("rusty_dlp").size(18).font(material::STRONG)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        space().height(4),
    ]
    .spacing(8);
    for (screen, label, icon) in [
        (
            Screen::NewDownload,
            "New download",
            material::Icon::Download,
        ),
        (Screen::Queue, "Queue", material::Icon::Queue),
        (Screen::History, "History", material::Icon::History),
        (Screen::Settings, "Settings", material::Icon::Settings),
    ] {
        let selected = screen == app.screen;
        let color = if selected {
            theme.palette().text
        } else {
            material::secondary(&theme)
        };
        let mut label_row = row![
            material::icon(icon, color),
            text(label).size(14).font(if selected {
                material::STRONG
            } else {
                iced::Font::DEFAULT
            })
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        if screen == Screen::Queue && queued > 0 {
            label_row = label_row
                .push(space().width(Fill))
                .push(text(queued.to_string()).size(12));
        }
        sidebar = sidebar.push(
            button(label_row)
                .width(Fill)
                .padding([0, 12])
                .style(move |theme, status| material::navigation(theme, status, selected))
                .on_press(Message::Navigate(screen)),
        );
    }
    sidebar = sidebar
        .push(space().height(Fill))
        .push(muted(format!("{active} active · {queued} queued")));
    let title = match app.screen {
        Screen::NewDownload => "New download",
        Screen::Queue => "Queue",
        Screen::History => "History",
        Screen::Settings => "Settings",
    };
    let page = match app.screen {
        Screen::NewDownload => new_download(app),
        Screen::Queue => queue(app, false),
        Screen::History => queue(app, true),
        Screen::Settings => settings(app),
    };
    let status = if app.installing.is_some() {
        "Setting up tools"
    } else if app.discovering {
        "Checking tools"
    } else if app.dependencies.ready() {
        "Tools ready"
    } else {
        "Setup needed"
    };
    let status_color = if app.dependencies.ready() {
        theme.palette().success
    } else {
        theme.palette().warning
    };
    let header = container(
        row![
            text(title).size(20).font(material::STRONG),
            space().width(Fill),
            button(
                row![text("●").size(9).color(status_color), text(status).size(12)]
                    .spacing(8)
                    .align_y(Alignment::Center)
            )
            .style(material::quiet_button)
            .on_press(Message::Navigate(Screen::Settings)),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .width(Fill);
    let mut content = column![
        header,
        scrollable(container(page).width(Fill))
            .style(material::scroll)
            .height(Fill),
    ]
    .spacing(12)
    .width(Fill)
    .height(Fill);
    if app.screen == Screen::NewDownload {
        content = content.push(download_action_bar(app));
    }
    if let Some(notice) = &app.notice {
        content = content.push(
            container(
                row![
                    scrollable(text(notice).size(13).line_height(1.4))
                        .style(material::scroll)
                        .height(64)
                        .width(Fill),
                    button("Dismiss")
                        .style(material::quiet_button)
                        .on_press(Message::DismissNotice),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding(12)
            .width(Fill)
            .style(move |theme| material::glass(theme, solid)),
        );
    }
    let chrome = row![
        container(sidebar)
            .padding(12)
            .width(184)
            .height(Fill)
            .style(move |theme| material::sidebar(theme, solid)),
        content,
    ]
    .spacing(16)
    .padding(16)
    .height(Fill)
    .width(Fill);
    if solid {
        container(chrome).into()
    } else {
        stack![
            image(material::backdrop(!material::dark(&theme)))
                .width(Fill)
                .height(Fill)
                .content_fit(iced::ContentFit::Fill),
            chrome
        ]
        .into()
    }
}

fn new_download(app: &App) -> Element<'_, Message> {
    let inspect_enabled = !app.discovering
        && app.installing.is_none()
        && app.inspecting.is_none()
        && app.dependencies.ready();
    let mut scope_controls = row![].spacing(4);
    for &scope in InspectScope::ALL {
        let selected = app.scope == scope;
        scope_controls = scope_controls.push(
            button(
                text(scope.to_string())
                    .size(13)
                    .font(material::MEDIUM)
                    .width(Fill)
                    .align_x(Alignment::Center),
            )
            .width(Fill)
            .height(28)
            .padding([0, 12])
            .style(move |theme, status| {
                if selected {
                    material::primary_button(theme, status)
                } else {
                    material::quiet_button(theme, status)
                }
            })
            .on_press(Message::Scope(scope)),
        );
    }
    let mut top = column![
        text_editor(&app.urls)
            .placeholder("Paste a video or playlist link…\nOne URL per line")
            .on_action(Message::EditUrls)
            .padding(10)
            .size(14)
            .height(72)
            .style(material::editor),
        row![
            container(scope_controls)
                .padding(4)
                .width(320)
                .style(material::segmented),
            space().width(Fill),
            button("Import list")
                .style(material::quiet_button)
                .on_press(Message::ImportUrls),
            button(if app.inspecting.is_some() {
                "Inspecting…"
            } else {
                "Inspect links"
            })
            .on_press_maybe(inspect_enabled.then_some(Message::Inspect)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(12);
    if !app.dependencies.ready() && !app.discovering {
        top = top.push(
            row![
                muted("Set up yt-dlp, FFmpeg and Deno to get started."),
                button("Open setup")
                    .style(material::quiet_button)
                    .on_press(Message::Navigate(Screen::Settings))
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        );
    }
    let mut page = column![top, options(app)].spacing(12);
    for (index, preview) in app.previews.iter().enumerate() {
        page = page.push(preview_card(app, index, preview));
    }
    page.into()
}

fn download_action_bar(app: &App) -> Element<'_, Message> {
    let selected = app
        .previews
        .iter()
        .filter(|p| app.selected_previews.contains(&p.url))
        .map(|p| {
            if p.is_playlist {
                p.entries.iter().filter(|e| e.selected).count()
            } else {
                1
            }
        })
        .sum::<usize>();
    container(
        row![
            muted(format!("{selected} selected")),
            space().width(Fill),
            button("Add to queue").on_press_maybe(
                (selected > 0 && !app.storage_blocked && !app.storage_failed)
                    .then_some(Message::Enqueue)
            )
        ]
        .align_y(Alignment::Center),
    )
    .padding([4, 0])
    .width(Fill)
    .into()
}

fn thumbnail<'a>(
    app: &'a App,
    url: Option<&String>,
    seconds: Option<f64>,
    width: f32,
    height: f32,
) -> Element<'a, Message> {
    let preview: Element<'a, Message> =
        if let Some(handle) = url.and_then(|url| app.thumbnails.get(url)) {
            image(handle.clone())
                .width(width)
                .height(height)
                .content_fit(iced::ContentFit::Cover)
                .border_radius(12.0)
                .into()
        } else {
            container(material::icon(
                material::Icon::Download,
                material::secondary(&app.theme()),
            ))
            .width(width)
            .height(height)
            .center_x(width)
            .center_y(height)
            .style(material::inset)
            .into()
        };
    let label = seconds
        .filter(|s| s.is_finite() && *s >= 0.0)
        .map(|s| duration(Some(s)))
        .unwrap_or_else(|| "—".into());
    stack![
        preview,
        container(
            container(text(label).size(11).font(material::MEDIUM))
                .padding([3, 6])
                .style(material::duration_badge)
        )
        .padding(6)
        .align_right(width)
        .align_bottom(height),
    ]
    .width(width)
    .height(height)
    .into()
}

fn preview_card<'a>(app: &'a App, index: usize, preview: &'a MediaPreview) -> Element<'a, Message> {
    let url = preview.url.clone();
    let details = if preview.is_playlist {
        format!("Playlist · {} loaded entries", preview.entries.len())
    } else {
        format!(
            "{} · {} available streams",
            duration(preview.duration),
            preview.formats.len()
        )
    };
    let mut body = column![
        row![
            thumbnail(
                app,
                preview.thumbnail.as_ref(),
                preview.duration,
                160.0,
                90.0
            ),
            column![
                checkbox(app.selected_previews.contains(&preview.url))
                    .label(preview.title.clone())
                    .text_size(16)
                    .on_toggle(move |v| Message::SelectPreview(url.clone(), v)),
                muted(details)
            ]
            .spacing(8)
            .width(Fill),
        ]
        .spacing(16)
        .align_y(Alignment::Center)
    ]
    .spacing(16);
    if preview.is_playlist {
        body = body.push(
            row![
                button("Select loaded")
                    .style(material::secondary_button)
                    .on_press(Message::SelectAll(index, true)),
                button("Clear")
                    .style(material::quiet_button)
                    .on_press(Message::SelectAll(index, false)),
                text_input("Range, e.g. 2-10", &app.range_input)
                    .on_input(Message::RangeInput)
                    .width(160),
                button("Select range")
                    .style(material::secondary_button)
                    .on_press(Message::SelectRange(index)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );
        let mut entries = column![].spacing(8);
        for (entry_index, entry) in preview.entries.iter().enumerate() {
            let entry_url = entry.url.clone();
            let label = format!("{:04}   {}", entry.index, entry.title);
            let check = checkbox(entry.selected).label(label).on_toggle_maybe(
                entry
                    .unavailable
                    .is_none()
                    .then_some(move |v| Message::SelectEntry(index, entry_index, v)),
            );
            let mut entry_row = row![
                thumbnail(app, entry.thumbnail.as_ref(), entry.duration, 80.0, 45.0),
                column![
                    check,
                    muted(
                        entry
                            .unavailable
                            .clone()
                            .unwrap_or_else(|| duration(entry.duration))
                    )
                ]
                .spacing(4)
                .width(Fill)
            ]
            .spacing(12)
            .align_y(Alignment::Center);
            if let Some(url) = entry_url {
                entry_row = entry_row.push(
                    button("Inspect video")
                        .style(material::quiet_button)
                        .on_press_maybe(
                            (entry.unavailable.is_none() && app.inspecting.is_none())
                                .then_some(Message::InspectEntry(url)),
                        ),
                );
            }
            entries = entries.push(container(entry_row).padding([4, 0]).width(Fill));
        }
        body = body.push(
            scrollable(entries)
                .style(material::scroll)
                .height(Length::Fixed(300.0)),
        );
        if preview.next_page.is_some() {
            body = body.push(
                button("Load next 50 entries")
                    .style(material::secondary_button)
                    .on_press_maybe(app.inspecting.is_none().then_some(Message::LoadMore(index))),
            );
        }
        body = body.push(muted("Selection applies to loaded entries. Inspect video opens that item individually for exact format selection."));
    }
    panel(app, body)
}

fn options(app: &App) -> Element<'_, Message> {
    let mut controls = row![field(
        "Media",
        pick_list(MediaMode::ALL, Some(app.options.mode), Message::Mode).width(Fill)
    )]
    .spacing(12);
    if app.options.mode == MediaMode::AudioOnly {
        controls = controls
            .push(field(
                "Audio format",
                pick_list(
                    AudioFormat::ALL,
                    Some(app.options.audio_format),
                    Message::AudioFormat,
                )
                .width(Fill),
            ))
            .push(field(
                "Bitrate (kbps)",
                pick_list(
                    [64, 96, 128, 160, 192, 256, 320],
                    Some(app.options.audio_bitrate),
                    Message::Bitrate,
                )
                .width(Fill),
            ));
    } else {
        controls = controls
            .push(field(
                "Quality limit",
                pick_list(QUALITIES, Some(Quality(app.options.max_height)), |q| {
                    Message::Quality(q.0)
                })
                .width(Fill),
            ))
            .push(field(
                "Container",
                pick_list(
                    Container::ALL,
                    Some(app.options.container),
                    Message::Container,
                )
                .width(Fill),
            ));
    }
    let mut body = column![controls].spacing(12);
    body = body.push(field(
        "Destination",
        row![
            text_input(
                "Choose a folder",
                &app.options.destination.to_string_lossy()
            )
            .on_input(Message::Destination)
            .width(Fill),
            button("Browse")
                .style(material::secondary_button)
                .on_press(Message::BrowseDestination(false))
        ]
        .spacing(8),
    ));
    body = body.push(
        button(if app.show_advanced {
            "Fewer options"
        } else {
            "More options"
        })
        .style(material::quiet_button)
        .on_press(Message::ToggleAdvanced),
    );
    if !app.show_advanced {
        let exact = app.exact_video != "Automatic" || app.exact_audio != "Automatic";
        if exact {
            body = body.push(muted(
                "Exact stream selection is active. Open More options to change it.",
            ));
        }
        return body.into();
    }
    body = body.push(
        checkbox(app.options.playlist_folder)
            .label("Create a folder for each playlist")
            .on_toggle(Message::PlaylistFolder),
    );
    body = body.push(muted(if app.options.mode == MediaMode::AudioOnly {
        "Original keeps the source audio; MP3/M4A use FFmpeg. Bitrate does not improve source quality."
    } else {
        "Quality is a maximum. Explicit containers select compatible streams without video transcoding."
    }));
    if app.previews.len() == 1 && !app.previews[0].is_playlist {
        let formats = &app.previews[0].formats;
        if app.options.mode != MediaMode::AudioOnly {
            let mut values = vec!["Automatic".to_string()];
            values.extend(
                formats
                    .iter()
                    .filter(|f| {
                        f.has_video()
                            && (app.options.mode != MediaMode::VideoOnly || !f.has_audio())
                    })
                    .map(format_label),
            );
            body = body.push(field(
                "Exact video stream (optional)",
                pick_list(values, Some(app.exact_video.clone()), Message::ExactVideo).width(Fill),
            ));
        }
        if app.options.mode != MediaMode::VideoOnly {
            let mut values = vec!["Automatic".to_string()];
            values.extend(
                formats
                    .iter()
                    .filter(|f| f.has_audio() && !f.has_video())
                    .map(format_label),
            );
            body = body.push(field(
                "Exact audio stream (optional)",
                pick_list(values, Some(app.exact_audio.clone()), Message::ExactAudio).width(Fill),
            ));
        }
        body = body.push(muted("For an exact silent video stream, choose an exact audio stream too. Exact IDs are checked again when the job starts."));
    }
    body = body.push(field(
        "Filename template",
        text_input("%(title)s [%(id)s].%(ext)s", &app.options.filename_template)
            .on_input(Message::Template),
    ));
    let presets = app
        .state
        .settings
        .presets
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<_>>();
    body = body.push(
        row![
            pick_list(presets, app.selected_preset.clone(), Message::ApplyPreset)
                .placeholder("Load preset")
                .width(Fill),
            text_input("New preset name", &app.preset_name)
                .on_input(Message::PresetName)
                .width(Fill),
            button("Save preset")
                .style(material::secondary_button)
                .on_press(Message::SavePreset),
        ]
        .spacing(8),
    );
    if let Some(name) = &app.selected_preset {
        body = body.push(
            button("Delete selected preset")
                .style(material::quiet_button)
                .on_press(Message::DeletePreset(name.clone())),
        );
    }
    body = body.push(
        button("Use these options as defaults")
            .style(material::quiet_button)
            .on_press(Message::SaveDefaults),
    );
    body.into()
}

fn queue(app: &App, history: bool) -> Element<'_, Message> {
    let mut jobs = app
        .state
        .queue
        .jobs
        .iter()
        .filter(|j| j.state.terminal() == history)
        .collect::<Vec<_>>();
    let mut page = column![].spacing(16);
    if history {
        page = page.push(
            row![
                muted(format!("{} finished item(s)", jobs.len())),
                space().width(Fill),
                button("Clear finished history")
                    .style(material::secondary_button)
                    .on_press(Message::ClearFinished)
            ]
            .align_y(Alignment::Center),
        );
    } else {
        let running = app.state.queue.running;
        let completed = app
            .state
            .queue
            .jobs
            .iter()
            .filter(|j| matches!(j.state, JobState::Completed | JobState::Skipped))
            .count();
        let total = app.state.queue.jobs.len();
        page = page.push(panel(app, column![
            row![text(if running { "Queue running" } else { "Queue paused" }).size(18), space().width(Fill),
                button(if running { "Hold queued jobs" } else { "Start / resume" }).on_press(if running { Message::HoldQueue } else { Message::StartQueue })].align_y(Alignment::Center),
            muted(format!("{completed} of {total} items completed · {} concurrent downloads", app.state.settings.concurrency)),
            progress_bar(0.0..=total.max(1) as f32, completed as f32),
            muted("Holding the queue lets active jobs finish. Cancel an item to stop its processes and retain resumable partial files."),
        ].spacing(12)));
    }
    if jobs.is_empty() {
        page = page.push(panel(
            app,
            column![
                text(if history {
                    "No finished downloads yet"
                } else {
                    "Your queue is clear"
                })
                .size(20),
                muted(if history {
                    "Completed, failed, canceled and skipped items will appear here."
                } else {
                    "Inspect a link and add selected items to start building your queue."
                }),
                button("Add a download")
                    .style(material::secondary_button)
                    .on_press(Message::Navigate(Screen::NewDownload)),
            ]
            .spacing(12),
        ));
    }
    if history {
        jobs.reverse();
    }
    for job in jobs {
        page = page.push(job_card(app, job));
    }
    page.into()
}

fn job_card<'a>(app: &'a App, job: &'a DownloadJob) -> Element<'a, Message> {
    let mut body = column![
        row![
            text(&job.title).size(16).font(material::MEDIUM).width(Fill),
            text(job.state.to_string()).size(13)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        muted(format!(
            "{} · {}",
            job.options.mode,
            job.options.destination.display()
        )),
    ]
    .spacing(12);
    if job.state.active() {
        if let Some(percent) = job.progress.percent() {
            body = body
                .push(progress_bar(0.0..=100.0, percent))
                .push(muted(format!(
                    "{percent:.1}% · {} · {}",
                    job.progress
                        .speed
                        .map(|s| format!("{}/s", bytes(s as u64)))
                        .unwrap_or_else(|| "Speed pending".into()),
                    job.progress
                        .eta
                        .map(|e| format!("ETA {}", duration(Some(e))))
                        .unwrap_or_else(|| "ETA unknown".into())
                )));
        } else {
            body = body.push(muted(if job.state == JobState::PostProcessing {
                "Merging or converting…"
            } else {
                "Working… total size is not available yet."
            }));
        }
    }
    if let Some(error) = &job.error {
        body = body.push(text(error).size(13).style(text::danger));
    }
    let mut actions = row![].spacing(8).align_y(Alignment::Center);
    if job.state.active() || job.state == JobState::Queued {
        actions = actions.push(
            button("Cancel")
                .style(material::secondary_button)
                .on_press(Message::CancelJob(job.id)),
        );
    }
    if job.state.retryable() {
        actions = actions.push(
            button(
                if job.state == JobState::Interrupted || job.state == JobState::Canceled {
                    "Resume"
                } else {
                    "Retry"
                },
            )
            .on_press(Message::RetryJob(job.id)),
        );
    }
    if !job.state.active() {
        actions = actions.push(
            button("Remove")
                .style(material::quiet_button)
                .on_press(Message::RemoveJob(job.id)),
        );
    }
    if let Some(path) = job
        .outputs
        .first()
        .filter(|_| matches!(job.state, JobState::Completed | JobState::Skipped))
    {
        actions = actions
            .push(
                button("Open file")
                    .style(material::secondary_button)
                    .on_press(Message::OpenFile(path.clone())),
            )
            .push(
                button("Open folder")
                    .style(material::secondary_button)
                    .on_press(Message::OpenFolder(path.clone())),
            );
    }
    actions = actions.push(space().width(Fill)).push(
        button(if app.expanded_logs.contains(&job.id) {
            "Hide log"
        } else {
            "Show log"
        })
        .style(material::quiet_button)
        .on_press(Message::ToggleLog(job.id)),
    );
    body = body.push(actions);
    if app.expanded_logs.contains(&job.id) {
        let log = if job.logs.is_empty() {
            "No diagnostic lines for this session.".into()
        } else {
            job.logs.join("\n")
        };
        body = body.push(
            scrollable(text(log).font(iced::Font::MONOSPACE).size(12))
                .style(material::scroll)
                .height(150),
        );
    }
    panel(app, body)
}

fn settings(app: &App) -> Element<'_, Message> {
    let mut page = column![panel(
        app,
        column![
            text("Preferences").size(18).font(material::STRONG),
            row![
                field(
                    "Appearance",
                    pick_list(
                        ThemeChoice::ALL,
                        Some(app.state.settings.theme),
                        Message::Theme
                    )
                    .width(Fill)
                ),
                field(
                    "Concurrent downloads",
                    pick_list(
                        [1usize, 2, 3, 4],
                        Some(app.state.settings.concurrency),
                        Message::Concurrency
                    )
                    .width(Fill)
                ),
            ]
            .spacing(16),
            checkbox(app.state.settings.reduce_transparency)
                .label("Reduce transparency")
                .on_toggle(Message::ReduceTransparency),
            muted("Use solid surfaces and remove ambient lighting for clearer separation."),
            row![
                column![
                    muted("Default destination"),
                    text(
                        app.state
                            .settings
                            .defaults
                            .destination
                            .to_string_lossy()
                            .into_owned()
                    )
                    .size(14)
                ]
                .spacing(5)
                .width(Fill),
                button("Change folder")
                    .style(material::secondary_button)
                    .on_press(Message::BrowseDestination(true))
            ]
            .align_y(Alignment::Center),
            muted("Save other default download options from the New download screen."),
        ]
        .spacing(16)
    )]
    .spacing(24);
    page = page.push(row![column![text("Download tools").size(20).font(material::STRONG), muted("Detected locally or installed into your user data folder. No administrator access needed.")].spacing(5).width(Fill),
        button(if app.discovering { "Checking…" } else { "Check tools" }).style(material::secondary_button).on_press_maybe((!app.tools_busy()).then_some(Message::Discover))].spacing(12).align_y(Alignment::Center));
    for &kind in ToolKind::ALL {
        let status = app.dependencies.tools.get(&kind);
        let idle = !app.tools_busy();
        let configured = app
            .state
            .settings
            .tool_paths
            .get(&kind)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut body = column![
            row![
                text(kind.to_string()).size(17).font(material::STRONG),
                space().width(Fill),
                muted(status.map(|s| s.source.as_str()).unwrap_or("Checking…"))
            ]
            .align_y(Alignment::Center)
        ]
        .spacing(12);
        if let Some(status) = status {
            if let Some(version) = &status.version {
                body = body.push(muted(version));
            }
            if let Some(path) = &status.path {
                body = body.push(muted(path.to_string_lossy().into_owned()));
            }
            if let Some(problem) = &status.problem {
                body = body.push(text(problem).size(13).style(text::danger));
            }
        }
        body = body.push(
            row![
                text_input("Optional custom executable path", &configured)
                    .on_input_maybe(idle.then_some(move |p| Message::ToolPath(kind, p)))
                    .width(Fill),
                button("Browse")
                    .style(material::secondary_button)
                    .on_press_maybe(idle.then_some(Message::BrowseTool(kind))),
                button("Auto")
                    .style(material::quiet_button)
                    .on_press_maybe(idle.then_some(Message::UseAutomaticTool(kind))),
            ]
            .spacing(8),
        );
        if let Some(progress) = app.installation.get(&kind) {
            body = body.push(muted(&progress.message));
            if let Some(fraction) = progress.fraction {
                body = body.push(progress_bar(0.0..=1.0, fraction));
            }
        }
        if app.installing == Some(kind) {
            body = body.push(
                button("Cancel setup")
                    .style(material::secondary_button)
                    .on_press(Message::CancelInstall),
            );
        } else {
            let mut actions = row![
                button("Install approved version")
                    .on_press_maybe(idle.then_some(Message::Install(kind, false)))
            ]
            .spacing(12);
            if status.is_some_and(|s| s.source.starts_with("Managed")) {
                actions = actions.push(
                    button(if kind == ToolKind::Ffmpeg && cfg!(target_os = "macos") {
                        "Reinstall approved update"
                    } else {
                        "Update managed tool"
                    })
                    .style(material::secondary_button)
                    .on_press_maybe(idle.then_some(Message::Install(kind, true))),
                );
            }
            body = body.push(actions);
        }
        if kind == ToolKind::Ffmpeg {
            body = body.push(muted("FFmpeg and ffprobe must be in the same folder. macOS approved versions advance with the app catalog."));
        }
        page = page.push(panel(app, body));
    }
    if app.tools_busy() {
        page = page.push(muted("Dependency changes are disabled while tools are in use. Hold the queue and finish or cancel active jobs first."));
    }
    if app.storage_failed {
        page = page.push(panel(app, column![
            text("Saving failed; queued downloads are paused").size(17),
            muted("Fix disk space or folder permissions, then retry saving. Existing files have been preserved."),
            button("Retry saving").on_press(Message::RetrySaving),
        ].spacing(12)));
    }
    if app.storage_blocked {
        page = page.push(text("Local state could not be restored. Existing files are preserved and saving is disabled. See README recovery instructions.").size(14).style(text::danger));
    }
    page.push(panel(app, column![text("Local data").size(18).font(material::STRONG), muted(app.data_directory().to_string_lossy().into_owned()),
        button("Open data folder").style(material::secondary_button).on_press(Message::OpenFolder(app.data_directory().to_path_buf())),
        muted("Preferences, presets, queue and history stay on this device. Cookie and account access are outside this release."),
    ].spacing(12))).into()
}

pub fn format_label(format: &FormatInfo) -> String {
    format!(
        "{} · {} · {} · {} · {} · {} · {}",
        format.id,
        format.extension,
        format
            .height
            .map(|h| format!("{h}p"))
            .unwrap_or_else(|| "Audio".into()),
        format
            .fps
            .map(|f| format!("{f:.0} fps"))
            .unwrap_or_else(|| "—".into()),
        format!(
            "{}/{}",
            format.video_codec.as_deref().unwrap_or("none"),
            format.audio_codec.as_deref().unwrap_or("none")
        ),
        format.language.as_deref().unwrap_or("und"),
        format
            .size
            .map(bytes)
            .unwrap_or_else(|| "Size unknown".into())
    )
}

fn duration(seconds: Option<f64>) -> String {
    match seconds.filter(|s| s.is_finite() && *s >= 0.0) {
        Some(seconds) => {
            let n = seconds as u64;
            if n >= 3600 {
                format!("{}:{:02}:{:02}", n / 3600, n / 60 % 60, n % 60)
            } else {
                format!("{}:{:02}", n / 60, n % 60)
            }
        }
        None => "Duration unknown".into(),
    }
}
fn bytes(value: u64) -> String {
    if value >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", value as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if value >= 1024 * 1024 {
        format!("{:.1} MB", value as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KB", value as f64 / 1024.0)
    }
}
