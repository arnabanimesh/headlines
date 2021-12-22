#![windows_subsystem="windows"]

use eframe::{epi::egui::Vec2, run_native, NativeOptions};

use headlines::{icon_create, Headlines, SCALEFACTOR};

fn main() {
    tracing_subscriber::fmt::init();
    let app = Headlines::new();
    let mut win_option = NativeOptions::default();
    win_option.initial_window_size = Some(Vec2::new(540. / SCALEFACTOR, 720. / SCALEFACTOR));
    win_option.icon_data = icon_create();
    run_native(Box::new(app), win_option)
}
