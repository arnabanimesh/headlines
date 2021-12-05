use std::{
    borrow::Cow,
    sync::mpsc::{channel, Receiver, SyncSender, TryRecvError},
    thread,
};

use eframe::egui::{
    menu,
    widgets::{Button, Label, Separator},
    Align, Color32, CtxRef, FontDefinitions, FontFamily, Hyperlink, Layout, TextStyle,
    TopBottomPanel, Window,
};
use newsapi::NewsAPI;
use serde::{Deserialize, Serialize};

pub(crate) const DPI: f32 = 1.25;
pub(crate) const PADDING5: f32 = 5.0 / DPI;
pub(crate) const PADDING10: f32 = 10.0 / DPI;
const HEADINGFONTSIZE: f32 = 35.0 / DPI;
const BODYFONTSIZE: f32 = 20.0 / DPI;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const RED: Color32 = Color32::from_rgb(255, 0, 0);

pub(crate) enum Msg {
    ApiKeySet(String),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HeadlinesConfig {
    pub(crate) dark_mode: bool,
    pub(crate) api_key: String,
}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self {
            dark_mode: Default::default(),
            api_key: String::new(),
        }
    }
}
pub(crate) struct Headlines {
    pub(crate) articles: Vec<NewsCardData>,
    pub(crate) config: HeadlinesConfig,
    pub(crate) api_key_initialized: bool,
    pub(crate) news_rx: Option<Receiver<NewsCardData>>,
    pub(crate) app_tx: Option<SyncSender<Msg>>,
}
pub(crate) struct NewsCardData {
    pub(crate) title: String,
    pub(crate) desc: String,
    pub(crate) url: String,
}

impl Headlines {
    pub(crate) fn new() -> Headlines {
        let config: HeadlinesConfig = confy::load("headlines").unwrap_or_default();
        Headlines {
            api_key_initialized: !config.api_key.is_empty(),
            articles: vec![],
            config,
            news_rx: None,
            app_tx: None,
        }
    }
    pub(crate) fn configure_fonts(&self, ctx: &CtxRef) {
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
    pub(crate) fn render_news_cards(&self, ui: &mut eframe::egui::Ui) {
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
    pub(crate) fn render_top_panel(&mut self, ctx: &CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.add_space(PADDING10);
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📚").text_style(TextStyle::Heading));
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    let close_btn = ui.add(Button::new("❌").text_style(TextStyle::Body));
                    if close_btn.clicked() {
                        frame.quit();
                    }
                    let refresh_btn = ui.add(Button::new("⟳").text_style(TextStyle::Body));
                    if refresh_btn.clicked() {
                        let api_key = self.config.api_key.to_string();
                        let (mut news_tx, news_rx) = channel();
                        self.news_rx = Some(news_rx);
                        self.articles = vec![];
                        thread::spawn(move || {
                            fetch_news(&api_key, &mut news_tx);
                        });
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

    pub(crate) fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_data) => {
                    self.articles.push(news_data);
                }
                Err(e) => {
                    if e == TryRecvError::Empty {
                        tracing::warn!("Error receiving message: {}", e);
                    };
                }
            }
        }
    }

    pub(crate) fn render_config(&mut self, ctx: &CtxRef) {
        Window::new("Configuration").show(ctx, |ui| {
            ui.label("Enter your API_KEY for newsapi.org");
            let text_input = ui.text_edit_singleline(&mut self.config.api_key);
            if text_input.lost_focus() && ui.input().key_pressed(eframe::egui::Key::Enter) {
                if let Err(e) = confy::store(
                    "headlines",
                    HeadlinesConfig {
                        dark_mode: self.config.dark_mode,
                        api_key: self.config.api_key.to_string(),
                    },
                ) {
                    tracing::error!("Failed saving app state: {}", e);
                }
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

pub(crate) fn fetch_news(api_key: &str, news_tx: &mut std::sync::mpsc::Sender<NewsCardData>) {
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
