mod headlines;

use eframe::{
    egui::{
        style::Visuals,
        widgets::{Label, Separator},
        CentralPanel, CtxRef, Hyperlink, ScrollArea, TextStyle, TopBottomPanel, Ui, Vec2,
    },
    epi::{App, Frame, IconData, Storage},
    run_native, NativeOptions,
};
use headlines::{fetch_news, Headlines, Msg, SCALEFACTOR, PADDING10, PADDING5};
use std::{
    sync::mpsc::{channel, sync_channel},
    thread,
};

impl App for Headlines {
    fn setup(&mut self, ctx: &CtxRef, _frame: &mut Frame<'_>, _storage: Option<&dyn Storage>) {
        let api_key = self.config.api_key.to_string();
        let (mut news_tx, news_rx) = channel();
        let (app_tx, app_rx) = sync_channel(1);
        self.app_tx = Some(app_tx);
        self.news_rx = Some(news_rx);
        thread::spawn(move || {
            if !api_key.is_empty() {
                fetch_news(&api_key, &mut news_tx);
            } else {
                loop {
                    match app_rx.recv() {
                        Ok(Msg::ApiKeySet(api_key)) => {
                            fetch_news(&api_key, &mut news_tx);
                        }
                        Err(e) => {
                            tracing::warn!("Failed receiving msg: {}", e);
                        }
                    }
                }
            }
        });
        self.configure_fonts(ctx);
    }
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        ctx.request_repaint();
        if self.config.dark_mode {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }
        if !self.api_key_initialized {
            self.render_config(ctx);
        } else {
            self.preload_articles();
            self.render_top_panel(ctx, frame);
            CentralPanel::default().show(ctx, |ui| {
                render_header(ui);
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.render_news_cards(ui);
                    });
                render_footer(ctx);
            });
        }
    }

    fn name(&self) -> &str {
        "Headlines"
    }
}

fn render_footer(ctx: &CtxRef) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(PADDING10);
            ui.add(Label::new("API Source: newsapi.org").monospace());
            ui.add(
                Hyperlink::new("https://github.com/emilk/egui")
                    .text("Made with egui")
                    .text_style(TextStyle::Monospace),
            );
            ui.add(
                Hyperlink::new("https://github.com/arnabanimesh/headlines")
                    .text("arnabanimesh/headlines")
                    .text_style(TextStyle::Monospace),
            );
            ui.add_space(PADDING10);
        });
    });
}

fn render_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("headlines");
    });
    ui.add_space(PADDING5);
    let sep = Separator::default().spacing(20. / SCALEFACTOR);
    ui.add(sep);
}

fn icon_create() -> Option<IconData> {
    let bytes = include_bytes!("../../newspaper.png");
    let image_buffer = image::load_from_memory(bytes).ok().unwrap();
    let img = image_buffer.to_rgba8();
    let size = (img.width() as u32, img.height() as u32);
    let pixels = img.into_vec();
    let icon_data = IconData {
        rgba: pixels,
        width: size.0,
        height: size.1,
    };
    Some(icon_data)
}

fn main() {
    tracing_subscriber::fmt::init();
    let app = Headlines::new();
    let mut win_option = NativeOptions::default();
    win_option.initial_window_size = Some(Vec2::new(540. / SCALEFACTOR, 720. / SCALEFACTOR));
    win_option.icon_data = icon_create();
    run_native(Box::new(app), win_option)
}
