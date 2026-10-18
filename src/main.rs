#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use rusty_dlp::{app::App, storage, ui};

fn main() -> iced::Result {
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--version")
    {
        println!("rusty_dlp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let root = match storage::data_dir() {
        Ok(root) => root,
        Err(error) => {
            startup_error(&error.to_string());
            return Ok(());
        }
    };
    let _lock = match storage::InstanceLock::acquire(&root) {
        Ok(lock) => lock,
        Err(error) => {
            startup_error(&error.to_string());
            return Ok(());
        }
    };
    iced::application(move || App::new(root.clone()), App::update, ui::view)
        .title("rusty_dlp")
        .settings(iced::Settings {
            default_text_size: iced::Pixels(14.0),
            ..Default::default()
        })
        .theme(App::theme)
        .subscription(App::subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1180.0, 780.0),
            min_size: Some(iced::Size::new(900.0, 600.0)),
            exit_on_close_request: false,
            icon: Some(application_icon()),
            ..Default::default()
        })
        .centered()
        .run()
}

fn application_icon() -> iced::window::Icon {
    let image = image::load_from_memory(include_bytes!("../assets/branding/icon.png"))
        .expect("bundled application icon must decode")
        .into_rgba8();
    let (width, height) = image.dimensions();
    iced::window::icon::from_rgba(image.into_raw(), width, height)
        .expect("bundled application icon must have valid RGBA dimensions")
}

fn startup_error(message: &str) {
    rfd::MessageDialog::new()
        .set_title("rusty_dlp")
        .set_description(message)
        .set_level(rfd::MessageLevel::Error)
        .show();
}
