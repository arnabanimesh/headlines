use std::{
    borrow::Cow,
    sync::mpsc::{Receiver, SyncSender, TryRecvError},
};

use eframe::egui::{
    menu,
    widgets::{Button, Label, Separator},
    Align, Color32, CtxRef, FontDefinitions, FontFamily, Hyperlink, Layout, TextStyle,
    TopBottomPanel, Window,
};
use newsapi::NewsAPI;
use serde::{Deserialize, Serialize};

pub const SCALEFACTOR: f32 = 1.25;
pub const PADDING5: f32 = 5.0 / SCALEFACTOR;
pub const PADDING10: f32 = 10.0 / SCALEFACTOR;
const HEADINGFONTSIZE: f32 = 35.0 / SCALEFACTOR;
const BODYFONTSIZE: f32 = 20.0 / SCALEFACTOR;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const RED: Color32 = Color32::from_rgb(255, 0, 0);

pub enum Msg {
    ApiKeySet(String),
    Refresh(String),
    Theme,
}

#[derive(Serialize, Deserialize)]
pub struct HeadlinesConfig {
    pub dark_mode: bool,
    pub api_key: String,
}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self {
            dark_mode: Default::default(),
            api_key: String::new(),
        }
    }
}
pub struct Headlines {
    pub articles: Vec<NewsCardData>,
    pub config: HeadlinesConfig,
    pub api_key_initialized: bool,
    pub news_rx: Option<Receiver<NewsCardData>>,
    pub app_tx: Option<SyncSender<Msg>>,
    pub app_rx: Option<Receiver<Msg>>,
}
pub struct NewsCardData {
    pub title: String,
    pub desc: String,
    pub url: String,
}

impl Headlines {
    pub fn new() -> Headlines {
        Headlines {
            api_key_initialized: Default::default(),
            articles: vec![],
            config: Default::default(),
            news_rx: None,
            app_tx: None,
            app_rx: None,
        }
    }
    pub fn configure_fonts(&self, ctx: &CtxRef) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            Cow::Borrowed(include_bytes!("../../MesloLGS NF Regular.ttf")),
        );
        font_def.family_and_size.insert(
            TextStyle::Heading,
            (FontFamily::Proportional, HEADINGFONTSIZE),
        );
        font_def
            .family_and_size
            .insert(TextStyle::Body, (FontFamily::Proportional, BODYFONTSIZE));
        font_def
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());
        ctx.set_fonts(font_def);
    }
    pub fn render_news_cards(&self, ui: &mut eframe::egui::Ui) {
        for a in &self.articles {
            ui.add_space(PADDING5);
            let title = format!("▶ {}", a.title);
            if self.config.dark_mode {
                ui.colored_label(WHITE, title);
            } else {
                ui.colored_label(BLACK, title);
            }
            ui.add_space(PADDING5);
            let desc = Label::new(&a.desc).text_style(TextStyle::Button);
            ui.add(desc);
            if self.config.dark_mode {
                ui.style_mut().visuals.hyperlink_color = CYAN;
            } else {
                ui.style_mut().visuals.hyperlink_color = RED;
            }
            ui.add_space(PADDING5);
            ui.with_layout(Layout::right_to_left().with_cross_align(Align::Min), |ui| {
                ui.add(Hyperlink::new(&a.url).text("read more ⤴"));
            });
            ui.add_space(PADDING5);
            ui.add(Separator::default());
        }
    }
    pub fn render_top_panel(&mut self, ctx: &CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.add_space(PADDING10);
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📚").text_style(TextStyle::Heading));
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    if !cfg!(target_arch = "wasm32") {
                        let close_btn = ui.add(Button::new("❌").text_style(TextStyle::Body));
                        if close_btn.clicked() {
                            frame.quit();
                        }
                    }
                    let refresh_btn = ui.add(Button::new("⟳").text_style(TextStyle::Body));
                    if refresh_btn.clicked() {
                        self.articles.clear();
                        if let Some(tx) = &self.app_tx {
                            tx.send(Msg::Refresh(self.config.api_key.to_string())).ok();
                        }
                    }
                    let theme_btn = ui.add(
                        Button::new({
                            if self.config.dark_mode {
                                "🔆"
                            } else {
                                "🌙"
                            }
                        })
                        .text_style(TextStyle::Body),
                    );
                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    };
                });
            });
            ui.add_space(PADDING10);
        });
    }

    pub fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_data) => self.articles.push(news_data),
                Err(e) => {
                    if let TryRecvError::Disconnected = e {
                        self.app_rx = None;
                    }
                }
            }
        }
    }

    pub fn render_config(&mut self, ctx: &CtxRef) {
        Window::new("Configuration").show(ctx, |ui| {
            ui.label("Enter your API_KEY for newsapi.org");
            let text_input = ui.text_edit_singleline(&mut self.config.api_key);
            if text_input.lost_focus() && ui.input().key_pressed(eframe::egui::Key::Enter) {
                self.api_key_initialized = true;
                if let Some(tx) = &mut self.app_tx {
                    tx.send(Msg::ApiKeySet(self.config.api_key.to_string()))
                        .ok();
                };
                tracing::error!("api key set");
            }
            tracing::error!("{}", self.config.api_key);
            ui.label("If you haven't registered for the API_KEY, head over to");
            ui.hyperlink("https://newsapi.org");
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_news(api_key: &str, news_tx: &mut std::sync::mpsc::Sender<NewsCardData>) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch_blocking() {
        let resp_articles = response.articles();
        for a in resp_articles.iter() {
            let news = NewsCardData {
                title: a.title().to_string(),
                url: a.url().to_string(),
                desc: a.desc().map(|s| s.to_string()).unwrap_or("...".to_string()),
            };
            if let Err(e) = news_tx.send(news) {
                tracing::error!("Error sending news data: {}", e);
            };
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_web(api_key: String, news_tx: std::sync::mpsc::Sender<NewsCardData>) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch_web().await {
        let resp_articles = response.articles();
        for a in resp_articles.iter() {
            let news = NewsCardData {
                title: a.title().to_string(),
                url: a.url().to_string(),
                desc: a.desc().map(|s| s.to_string()).unwrap_or("...".to_string()),
            };
            if let Err(e) = news_tx.send(news) {
                tracing::error!("Error sending news data: {}", e);
            };
        }
    }
}
