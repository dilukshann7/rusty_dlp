#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use rusty_dlp::{app::App, storage, ui};

fn main() -> iced::Result {
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
        .theme(App::theme)
        .subscription(App::subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1180.0, 780.0),
            min_size: Some(iced::Size::new(900.0, 600.0)),
            exit_on_close_request: false,
            ..Default::default()
        })
        .centered()
        .run()
}

fn startup_error(message: &str) {
    rfd::MessageDialog::new()
        .set_title("rusty_dlp")
        .set_description(message)
        .set_level(rfd::MessageLevel::Error)
        .show();
}
