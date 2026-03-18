#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::mpsc;
use std::thread;

use chrono::Datelike as _;
use eframe::egui;

use mensa::api::cached_fetch;
use mensa::meal::{parse_menu, Meal};

// ---------------------------------------------------------------------------
// Background fetch
// ---------------------------------------------------------------------------

fn do_fetch(date_str: &str, lang: &str, no_cache: bool) -> Result<Vec<Meal>, String> {
    let html = cached_fetch(date_str, lang, no_cache).map_err(|e| e.to_string())?;
    parse_menu(&html).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

enum FetchState {
    Loading,
    Ready(Vec<Meal>),
    Empty,
    Failed(String),
}

struct MensaApp {
    state: FetchState,
    lang: &'static str,
    no_cache: bool,
    today: chrono::NaiveDate,
    rx: Option<mpsc::Receiver<Result<Vec<Meal>, String>>>,
}

impl MensaApp {
    fn start_fetch(&mut self) {
        let date_str = self.today.format("%Y-%m-%d").to_string();
        let lang = self.lang.to_owned();
        let no_cache = self.no_cache;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.state = FetchState::Loading;
        drop(thread::spawn(move || {
            drop(tx.send(do_fetch(&date_str, &lang, no_cache)));
        }));
    }

    #[must_use]
    fn weekday_label(&self) -> String {
        const DAYS_DE: [&str; 7] = [
            "Montag",
            "Dienstag",
            "Mittwoch",
            "Donnerstag",
            "Freitag",
            "Samstag",
            "Sonntag",
        ];
        const DAYS_EN: [&str; 7] = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        let idx = self.today.weekday().num_days_from_monday() as usize;
        if self.lang == "en" {
            format!("{} {}", DAYS_EN[idx], self.today.format("%m/%d"))
        } else {
            format!("{} {}", DAYS_DE[idx], self.today.format("%d.%m."))
        }
    }
}

impl Default for MensaApp {
    fn default() -> Self {
        let mut app = Self {
            state: FetchState::Loading,
            lang: "de",
            no_cache: false,
            today: chrono::Local::now().date_naive(),
            rx: None,
        };
        app.start_fetch();
        app
    }
}

// ---------------------------------------------------------------------------
// egui rendering
// ---------------------------------------------------------------------------

impl eframe::App for MensaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll background thread
        let poll = self.rx.as_ref().map(mpsc::Receiver::try_recv);
        if let Some(recv) = poll {
            match recv {
                Ok(fetch_result) => {
                    self.rx = None;
                    self.state = match fetch_result {
                        Ok(meals) if meals.is_empty() => FetchState::Empty,
                        Ok(meals) => FetchState::Ready(meals),
                        Err(e) => FetchState::Failed(e),
                    };
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.rx = None;
                    self.state = FetchState::Failed("Worker thread disconnected".to_owned());
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let weekday = self.weekday_label();
            ui.heading(format!("\u{1F37D}  Mensa am Schloss \u{2014} {weekday}"));
            ui.separator();

            // Controls
            ui.horizontal(|ui| {
                if ui.selectable_label(self.lang == "de", "DE").clicked() {
                    self.lang = "de";
                    self.start_fetch();
                }
                if ui.selectable_label(self.lang == "en", "EN").clicked() {
                    self.lang = "en";
                    self.start_fetch();
                }
                ui.separator();
                if ui.button("\u{27F3}  Refresh").clicked() {
                    self.start_fetch();
                }
                ui.checkbox(&mut self.no_cache, "No cache");
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| match &self.state {
                FetchState::Loading => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading menu\u{2026}");
                    });
                }
                FetchState::Empty => {
                    ui.label(
                        egui::RichText::new("No meals available today.")
                            .color(egui::Color32::GRAY),
                    );
                }
                FetchState::Failed(msg) => {
                    ui.label(
                        egui::RichText::new(format!("Error: {msg}"))
                            .color(egui::Color32::RED),
                    );
                }
                FetchState::Ready(meals) => {
                    for meal in meals {
                        render_meal(ui, meal);
                        ui.add_space(6.0);
                    }
                }
            });
        });
    }
}

fn render_meal(ui: &mut egui::Ui, meal: &Meal) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(egui::RichText::new(&meal.name).strong());

        ui.horizontal_wrapped(|ui| {
            let tokens = &meal.ingredients;
            let last = tokens.len().saturating_sub(1);
            for (i, token) in tokens.iter().enumerate() {
                let sep = if i < last { ", " } else { "" };
                let display = format!("{}{sep}", token.text);
                if token.has_milk {
                    ui.label(
                        egui::RichText::new(display.to_uppercase())
                            .color(egui::Color32::from_rgb(220, 50, 50))
                            .strong(),
                    );
                } else {
                    ui.label(display);
                }
            }
        });

        ui.label(
            egui::RichText::new(meal.price_info())
                .color(egui::Color32::GRAY)
                .small(),
        );
    });
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Mensa am Schloss",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MensaApp::default()))),
    )
}
