mod app;
mod bookmarks;
mod config;
mod dialogs;
mod editor;
mod menu;
mod operations;
mod panel;
mod search;
mod util;
mod viewer;
mod vfs;

use app::App;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .default_font(iced::Font::MONOSPACE)
        .window_size(iced::Size::new(1200.0, 800.0))
        .run_with(App::new)
}
