mod headlines;

use eframe::{
    egui::{
        style::Visuals,
        widgets::{Label, Separator},
        CentralPanel, CtxRef, Hyperlink, ScrollArea, TextStyle, TopBottomPanel, Ui,
    },
    epi::{App, Frame, IconData, Storage},
};
pub use headlines::{Headlines, Msg, PADDING10, PADDING5, SCALEFACTOR};
use std::sync::mpsc::{channel, sync_channel};

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

#[cfg(target_arch = "wasm32")]
use headlines::fetch_web;

#[cfg(not(target_arch = "wasm32"))]
use headlines::fetch_news;

impl App for Headlines {
    fn setup(&mut self, ctx: &CtxRef, _frame: &mut Frame<'_>, storage: Option<&dyn Storage>) {
        if let Some(storage) = storage {
            self.config = eframe::epi::get_value(storage, "headlines").unwrap_or_default();
            self.api_key_initialized = !self.config.api_key.is_empty();
        }
        let api_key = self.config.api_key.to_string();
        let (news_tx, news_rx) = channel();
        #[cfg(not(target_arch = "wasm32"))]
        let mut news_tx = news_tx;
        let (app_tx, app_rx) = sync_channel(1);
        self.app_tx = Some(app_tx);
        self.news_rx = Some(news_rx);
        #[cfg(target_arch = "wasm32")]
        let api_key_web = api_key.clone();
        #[cfg(target_arch = "wasm32")]
        let news_tx_web = news_tx.clone();
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            if !api_key.is_empty() {
                fetch_news(&api_key, &mut news_tx);
            }
            loop {
                match app_rx.recv() {
                    Ok(Msg::ApiKeySet(api_key)) => {
                        fetch_news(&api_key, &mut news_tx);
                    }
                    Ok(Msg::Refresh(api_key)) => {
                        fetch_news(&api_key, &mut news_tx);
                    }
                    Ok(Msg::Theme) => {}
                    Err(e) => {
                        tracing::warn!("Failed receiving msg: {}", e);
                    }
                }
            }
        });
        #[cfg(target_arch = "wasm32")]
        gloo_timers::callback::Timeout::new(10, move || {
            wasm_bindgen_futures::spawn_local(async {
                fetch_web(api_key_web, news_tx_web).await;
            })
        })
        .forget();
        #[cfg(target_arch = "wasm32")]
        gloo_timers::callback::Interval::new(500, move || match app_rx.try_recv() {
            Ok(Msg::ApiKeySet(api_key)) => {
                wasm_bindgen_futures::spawn_local(fetch_web(api_key.clone(), news_tx.clone()));
            }
            Ok(Msg::Refresh(api_key)) => {
                wasm_bindgen_futures::spawn_local(fetch_web(api_key.clone(), news_tx.clone()));
            }
            Ok(Msg::Theme) => {}
            Err(e) => {
                tracing::error!("Failed receiving msg: {}", e);
            }
        })
        .forget();
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
                if self.articles.is_empty() {
                    ui.vertical_centered_justified(|ui| {
                        ui.heading("Loading ⌛");
                    });
                } else {
                    render_header(ui);
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.render_news_cards(ui);
                        });
                    render_footer(ctx);
                }
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::epi::Storage) {
        eframe::epi::set_value(storage, "headlines", &self.config);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(3)
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

pub fn icon_create() -> Option<IconData> {
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

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_web(canvas_id: &str) {
    let headlines = Headlines::new();
    // Uncomment this line while debugging
    // tracing_wasm::set_as_global_default();
    eframe::start_web(canvas_id, Box::new(headlines)).expect("Could not start web app");
}
