//! Native iced materials with Vercel-inspired neutral surfaces and media layouts.
//!
//! Glass uses alpha-composited gradients, specular borders and static, diffuse
//! lighting. It does not sample/blur the live framebuffer or require OS vibrancy.
use iced::{
    Background, Border, Color, Font, Radians, Shadow, Theme, Vector, gradient,
    widget::{
        button, checkbox, container, image, overlay::menu, pick_list, progress_bar, scrollable,
        svg, text_editor, text_input,
    },
};
use std::sync::OnceLock;

pub const STRONG: Font = Font {
    weight: iced::font::Weight::Semibold,
    ..Font::DEFAULT
};
pub const MEDIUM: Font = Font {
    weight: iced::font::Weight::Medium,
    ..Font::DEFAULT
};
pub const BRAND: Color = Color::from_rgb8(255, 0, 0);
pub const FOCUS: Color = Color::from_rgb8(62, 166, 255);
pub const CONTROL_HEIGHT: f32 = 44.0;
pub const CONTROL_LINE_HEIGHT: iced::widget::text::LineHeight =
    iced::widget::text::LineHeight::Absolute(iced::Pixels(20.0));

pub fn dark(theme: &Theme) -> bool {
    theme.palette().background.r < 0.5
}
fn alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}
fn gray(value: u8) -> Color {
    Color::from_rgb8(value, value, value)
}
pub fn secondary(theme: &Theme) -> Color {
    if dark(theme) { gray(161) } else { gray(86) }
}
fn line(theme: &Theme) -> Color {
    if dark(theme) { gray(51) } else { gray(224) }
}
fn control(theme: &Theme) -> Color {
    if dark(theme) { gray(26) } else { gray(250) }
}
fn linear(top: Color, bottom: Color) -> Background {
    gradient::Linear::new(Radians(1.15))
        .add_stop(0.0, top)
        .add_stop(1.0, bottom)
        .into()
}

pub fn theme(light: bool) -> Theme {
    Theme::custom(
        "rusty_dlp",
        iced::theme::Palette {
            background: if light {
                Color::from_rgb8(247, 248, 250)
            } else {
                gray(10)
            },
            text: if light { gray(15) } else { gray(237) },
            primary: if light { gray(15) } else { gray(237) },
            success: if light {
                Color::from_rgb8(32, 118, 80)
            } else {
                Color::from_rgb8(87, 204, 151)
            },
            warning: if light {
                Color::from_rgb8(156, 101, 0)
            } else {
                Color::from_rgb8(255, 196, 0)
            },
            danger: if light {
                Color::from_rgb8(194, 35, 43)
            } else {
                Color::from_rgb8(255, 127, 134)
            },
        },
    )
}

pub fn glass(theme: &Theme, solid: bool) -> container::Style {
    let is_dark = dark(theme);
    let background = if solid {
        (if is_dark { gray(17) } else { Color::WHITE }).into()
    } else if is_dark {
        linear(alpha(gray(23), 0.96), alpha(gray(17), 0.98))
    } else {
        linear(alpha(Color::WHITE, 0.88), alpha(Color::WHITE, 0.65))
    };
    container::Style {
        text_color: Some(theme.palette().text),
        background: Some(background),
        border: Border {
            color: if solid || is_dark {
                line(theme)
            } else {
                alpha(Color::WHITE, 0.96)
            },
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: if solid {
            Shadow::default()
        } else {
            Shadow {
                color: alpha(Color::BLACK, if is_dark { 0.20 } else { 0.06 }),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            }
        },
        ..Default::default()
    }
}

pub fn sidebar(theme: &Theme, solid: bool) -> container::Style {
    let mut style = glass(theme, solid);
    if !solid {
        style.background = Some(if dark(theme) {
            linear(alpha(gray(17), 0.98), alpha(gray(17), 0.96))
        } else {
            linear(alpha(Color::WHITE, 0.88), alpha(gray(246), 0.87))
        });
    }
    style.border.radius = 20.0.into();
    style
}

pub fn inset(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(control(theme).into()),
        border: Border {
            radius: 10.0.into(),
            color: line(theme),
            width: 1.0,
        },
        ..Default::default()
    }
}

pub fn badge(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(theme.palette().text),
        background: Some(control(theme).into()),
        border: Border {
            radius: 99.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn duration_badge(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(Color::WHITE),
        background: Some(alpha(Color::BLACK, 0.80).into()),
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let is_dark = dark(theme);
    let fill = match status {
        button::Status::Pressed => {
            if is_dark {
                gray(198)
            } else {
                gray(60)
            }
        }
        button::Status::Hovered => {
            if is_dark {
                Color::WHITE
            } else {
                gray(42)
            }
        }
        _ => {
            if is_dark {
                gray(237)
            } else {
                gray(15)
            }
        }
    };
    let mut style = button::Style {
        background: Some(fill.into()),
        text_color: if is_dark { gray(15) } else { Color::WHITE },
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };
    if matches!(status, button::Status::Disabled) {
        style.background = Some(control(theme).into());
        style.text_color = secondary(theme);
        style.shadow = Shadow::default();
    }
    style
}

pub fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let is_dark = dark(theme);
    let fill = match status {
        button::Status::Pressed => {
            if is_dark {
                gray(46)
            } else {
                gray(218)
            }
        }
        button::Status::Hovered => {
            if is_dark {
                gray(36)
            } else {
                gray(230)
            }
        }
        _ => control(theme),
    };
    button::Style {
        background: Some(fill.into()),
        text_color: if matches!(status, button::Status::Disabled) {
            secondary(theme)
        } else {
            theme.palette().text
        },
        border: Border {
            radius: 10.0.into(),
            color: line(theme),
            width: 1.0,
        },
        ..Default::default()
    }
}

pub fn quiet_button(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: matches!(status, button::Status::Hovered | button::Status::Pressed)
            .then(|| control(theme).into()),
        text_color: if matches!(status, button::Status::Disabled) {
            secondary(theme)
        } else {
            theme.palette().text
        },
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn navigation(theme: &Theme, status: button::Status, selected: bool) -> button::Style {
    if selected {
        secondary_button(theme, status)
    } else {
        quiet_button(theme, status)
    }
}

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let focused = matches!(status, text_input::Status::Focused { .. });
    text_input::Style {
        background: control(theme).into(),
        border: Border {
            radius: 10.0.into(),
            width: if focused { 1.5 } else { 1.0 },
            color: if focused { FOCUS } else { line(theme) },
        },
        icon: secondary(theme),
        placeholder: secondary(theme),
        value: theme.palette().text,
        selection: alpha(FOCUS, 0.30),
    }
}

pub fn editor(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let focused = matches!(status, text_editor::Status::Focused { .. });
    text_editor::Style {
        background: control(theme).into(),
        border: Border {
            radius: 10.0.into(),
            width: if focused { 1.5 } else { 1.0 },
            color: if focused { FOCUS } else { line(theme) },
        },
        placeholder: secondary(theme),
        value: theme.palette().text,
        selection: alpha(FOCUS, 0.30),
    }
}

pub fn select(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: theme.palette().text,
        placeholder_color: secondary(theme),
        handle_color: secondary(theme),
        background: control(theme).into(),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: if matches!(status, pick_list::Status::Opened { .. }) {
                FOCUS
            } else {
                line(theme)
            },
        },
    }
}

/// Pick-list overlays are a separate widget; style them explicitly as part of
/// the same surface system instead of inheriting iced's blue selection menu.
pub fn select_menu(theme: &Theme) -> menu::Style {
    menu::Style {
        background: (if dark(theme) { gray(17) } else { Color::WHITE }).into(),
        border: Border {
            color: line(theme),
            width: 1.0,
            radius: 10.0.into(),
        },
        text_color: theme.palette().text,
        selected_text_color: theme.palette().text,
        selected_background: (if dark(theme) { gray(38) } else { gray(240) }).into(),
        shadow: Shadow {
            color: alpha(Color::BLACK, if dark(theme) { 0.40 } else { 0.12 }),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 20.0,
        },
    }
}

pub fn segmented(theme: &Theme) -> container::Style {
    container::Style {
        background: Some((if dark(theme) { gray(10) } else { gray(242) }).into()),
        border: Border {
            color: line(theme),
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn check(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let checked = match status {
        checkbox::Status::Active { is_checked }
        | checkbox::Status::Hovered { is_checked }
        | checkbox::Status::Disabled { is_checked } => is_checked,
    };
    let disabled = matches!(status, checkbox::Status::Disabled { .. });
    checkbox::Style {
        background: if checked {
            theme.palette().text.into()
        } else {
            control(theme).into()
        },
        icon_color: theme.palette().background,
        border: Border {
            radius: 6.0.into(),
            color: if checked {
                theme.palette().text
            } else {
                secondary(theme)
            },
            width: 1.0,
        },
        text_color: Some(if disabled {
            secondary(theme)
        } else {
            theme.palette().text
        }),
    }
}

pub fn progress(theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: control(theme).into(),
        bar: FOCUS.into(),
        border: Border {
            radius: 99.0.into(),
            ..Default::default()
        },
    }
}

pub fn scroll(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    for rail in [&mut style.vertical_rail, &mut style.horizontal_rail] {
        rail.background = None;
        rail.scroller.background = alpha(secondary(theme), 0.45).into();
        rail.scroller.border.radius = 99.0.into();
    }
    style
}

/// A static diffuse environment behind the translucent controls. No animated
/// wallpaper, shader-only path, or per-frame image allocation is needed.
pub fn backdrop(light: bool) -> image::Handle {
    static LIGHT: OnceLock<image::Handle> = OnceLock::new();
    static DARK: OnceLock<image::Handle> = OnceLock::new();
    (if light { &LIGHT } else { &DARK })
        .get_or_init(|| {
            let (width, height) = (480u32, 320u32);
            let mut pixels = Vec::with_capacity((width * height * 4) as usize);
            for y in 0..height {
                for x in 0..width {
                    let px = x as f32 / width as f32;
                    let py = y as f32 / height as f32;
                    let glow = |cx: f32, cy: f32, spread: f32| {
                        (-((px - cx).powi(2) + (py - cy).powi(2)) / spread).exp()
                    };
                    let silver = glow(0.48, 0.85, 0.25);
                    let base = if light { 247.0 } else { 10.0 };
                    let colors = [base + silver * 3.0; 3];
                    pixels.extend(colors.map(|c| c.clamp(0.0, 255.0) as u8));
                    pixels.push(255);
                }
            }
            image::Handle::from_rgba(width, height, pixels)
        })
        .clone()
}

#[derive(Debug, Clone, Copy)]
pub enum Icon {
    Download,
    Queue,
    History,
    Settings,
    Link,
}

pub fn icon(kind: Icon, color: Color) -> svg::Svg<'static> {
    let path = match kind {
        Icon::Download => "<path d='M12 3v12m-5-5 5 5 5-5M4 15v5h16v-5'/>",
        Icon::Queue => {
            "<rect x='4' y='4' width='16' height='5' rx='2'/><path d='M6 13h12M6 18h12'/>"
        }
        Icon::History => "<circle cx='12' cy='12' r='8.5'/><path d='M12 7v5l3 2'/>",
        Icon::Settings => {
            "<path d='M4 7h5m5 0h6M4 17h10m5 0h1'/><circle cx='11.5' cy='7' r='2.5'/><circle cx='16.5' cy='17' r='2.5'/>"
        }
        Icon::Link => {
            "<path d='m9 15 6-6m-7 9-1 1a4.2 4.2 0 0 1-6-6l4-4a4.2 4.2 0 0 1 6 0m2-3 1-1a4.2 4.2 0 0 1 6 6l-4 4a4.2 4.2 0 0 1-6 0' transform='translate(1 0)'/>"
        }
    };
    let data = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='rgb({},{},{})' stroke-width='1.65' stroke-linecap='round' stroke-linejoin='round'>{path}</svg>",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    );
    svg::Svg::new(svg::Handle::from_memory(data.into_bytes()))
        .width(20)
        .height(20)
}

pub fn brand() -> svg::Svg<'static> {
    svg::Svg::new(svg::Handle::from_memory(b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 36 28'><rect x='1' y='2' width='34' height='24' rx='8' fill='#FF0000'/><path d='m15 8 10 6-10 6z' fill='white'/></svg>".as_slice())).width(32).height(26)
}
