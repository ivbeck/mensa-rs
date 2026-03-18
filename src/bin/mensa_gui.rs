#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::mpsc;
use std::thread;

use chrono::Datelike as _;
use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Rounding, Vec2};

use mensa::api::cached_fetch;
use mensa::meal::{parse_menu, Meal};

const MLG_PHRASES: &[&str] = &[
    "420",
    "MLG",
    "YOLO",
    "360 NOSCOPE",
    "DANK",
    "REKT",
    "SWAG",
    "DORITOS",
    "MTN DEW",
    "PRO GAMER",
    "GGWP",
    "EZ",
    "CLUTCH",
    "PWNED",
    "HEADSHOT",
    "ACE",
    "HACKS",
    "MONTAGE",
    "SICK",
    "BEAST MODE",
    "NO SCOPE",
    "TRIGGERED",
    "OMEGALUL",
    "KEKW",
    "POG",
];

fn do_fetch(date_str: &str, lang: &str, no_cache: bool) -> Result<Vec<Meal>, String> {
    let html = cached_fetch(date_str, lang, no_cache).map_err(|e| e.to_string())?;
    parse_menu(&html).map_err(|e| e.to_string())
}




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
    mlg_mode: bool,
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
            mlg_mode: false,
        };
        app.start_fetch();
        app
    }
}





/
/
/
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn rainbow(hue: f32) -> Color32 {
    let segment = hue.rem_euclid(1.0) * 6.0;
    let chroma = 1.0_f32 - (segment % 2.0 - 1.0_f32).abs();
    let (red, green, blue) = if segment < 1.0 {
        (1.0_f32, chroma, 0.0_f32)
    } else if segment < 2.0 {
        (chroma, 1.0_f32, 0.0_f32)
    } else if segment < 3.0 {
        (0.0_f32, 1.0_f32, chroma)
    } else if segment < 4.0 {
        (0.0_f32, chroma, 1.0_f32)
    } else if segment < 5.0 {
        (chroma, 0.0_f32, 1.0_f32)
    } else {
        (1.0_f32, 0.0_f32, chroma)
    };
    Color32::from_rgb(
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
    )
}

/
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn paint_mlg_background(painter: &egui::Painter, screen: Rect, t: f64) {
    let n = 18_i32;
    let band_h = screen.height() / n as f32;
    for i in 0..n {
        let hue = (t as f32).mul_add(0.22, i as f32 / n as f32) % 1.0;
        let y = (i as f32).mul_add(band_h, screen.min.y);
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(screen.min.x, y),
                Vec2::new(screen.width(), band_h),
            ),
            Rounding::ZERO,
            rainbow(hue).gamma_multiply(0.08),
        );
    }
}

/
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn paint_mlg_overlay(ctx: &egui::Context, t: f64) {
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("mlg_phrases"),
    ));
    let screen = ctx.screen_rect();
    let n = MLG_PHRASES.len();
    for (i, phrase) in MLG_PHRASES.iter().enumerate() {
        
        let seed = i as f64 * 2.399_963_229_728_65;
        let sx = (seed * 0.31).fract().mul_add(0.28, 0.38);
        let sy = (seed * 0.73).fract().mul_add(0.38, 0.27);
        let px = screen.width().mul_add(
            0.44f64.mul_add(t.mul_add(sx, seed * std::f64::consts::E).sin(), 0.5) as f32,
            screen.min.x,
        );
        let py = screen.height().mul_add(
            0.38f64.mul_add(t.mul_add(sy, seed * std::f64::consts::SQRT_2).cos(), 0.5) as f32,
            screen.min.y,
        );
        let hue = (t as f32).mul_add(0.33, i as f32 / n as f32) % 1.0;
        let size = 5.0f64.mul_add(t.mul_add(1.15, seed).sin().abs(), 11.0) as f32;
        painter.text(
            Pos2::new(px, py),
            egui::Align2::CENTER_CENTER,
            phrase,
            egui::FontId::proportional(size),
            rainbow(hue).gamma_multiply(0.20),
        );
    }
}





impl eframe::App for MensaApp {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::too_many_lines
    )]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = ctx.input(|i| i.time);
        let t32 = t as f32;

        
        if let Some(recv) = self.rx.as_ref().map(mpsc::Receiver::try_recv) {
            match recv {
                Ok(fetch_result) => {
                    self.rx = None;
                    self.state = match fetch_result {
                        Ok(meals) if meals.is_empty() => FetchState::Empty,
                        Ok(meals) => FetchState::Ready(meals),
                        Err(e) => FetchState::Failed(e),
                    };
                }
                Err(mpsc::TryRecvError::Empty) => ctx.request_repaint(),
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.rx = None;
                    self.state = FetchState::Failed("Worker thread disconnected".to_owned());
                }
            }
        }

        if self.mlg_mode {
            ctx.request_repaint();
            paint_mlg_overlay(ctx, t);
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = Color32::from_rgba_premultiplied(8, 0, 20, 245);
            visuals.window_fill = Color32::from_rgba_premultiplied(8, 0, 20, 245);
            visuals.widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, rainbow((t32 * 0.3) % 1.0).gamma_multiply(0.5));
            ctx.set_visuals(visuals);
        } else {
            ctx.set_visuals(egui::Visuals::dark());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.mlg_mode {
                paint_mlg_background(ui.painter(), ui.max_rect(), t);
            }

            let weekday = self.weekday_label();
            if self.mlg_mode {
                let hue = (t32 * 0.55) % 1.0;
                let pulse = 0.07f32.mul_add((t32 * 5.5).sin(), 1.0_f32);
                ui.label(
                    egui::RichText::new(format!(
                        "\u{1F3AE}\u{1F525} MENSA AM SCHLOSS \u{1F525}\u{1F3AE} \u{2014} {weekday} \u{1F480}"
                    ))
                    .color(rainbow(hue))
                    .strong()
                    .size(22.0 * pulse),
                );
            } else {
                ui.heading(format!("\u{1F37D}  Mensa am Schloss \u{2014} {weekday}"));
            }
            ui.separator();

            
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
                ui.separator();

                let mlg_label = if self.mlg_mode {
                    "\u{1F3AE} MLG MODE: ON \u{1F525}"
                } else {
                    "\u{1F3AE} MLG MODE"
                };
                let btn_color = if self.mlg_mode {
                    rainbow((t32 * 2.8) % 1.0)
                } else {
                    Color32::GRAY
                };
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new(mlg_label).color(btn_color).strong(),
                    ))
                    .clicked()
                {
                    self.mlg_mode = !self.mlg_mode;
                    if !self.mlg_mode {
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| match &self.state {
                FetchState::Loading => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        if self.mlg_mode {
                            ui.label(
                                egui::RichText::new(
                                    "\u{1F525} FETCHING THE DANKEST MENU\u{2026} \u{1F525}",
                                )
                                .color(rainbow((t32 * 3.5) % 1.0))
                                .strong(),
                            );
                        } else {
                            ui.label("Loading menu\u{2026}");
                        }
                    });
                }
                FetchState::Empty => {
                    if self.mlg_mode {
                        ui.label(
                            egui::RichText::new(
                                "\u{1F480} NO MEALS TODAY \u{2014} YOU GOT REKT. \u{1F480}",
                            )
                            .color(Color32::from_rgb(255, 60, 60))
                            .strong()
                            .size(18.0),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("No meals available today.")
                                .color(Color32::GRAY),
                        );
                    }
                }
                FetchState::Failed(msg) => {
                    if self.mlg_mode {
                        ui.label(
                            egui::RichText::new(format!(
                                "\u{1F480} EPIC FAIL: {msg} \u{1F480}"
                            ))
                            .color(rainbow((t32 * 5.5) % 1.0))
                            .strong(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(format!("Error: {msg}")).color(Color32::RED),
                        );
                    }
                }
                FetchState::Ready(meals) => {
                    for (idx, meal) in meals.iter().enumerate() {
                        render_meal(ui, meal, self.mlg_mode, t, idx);
                        ui.add_space(6.0);
                    }
                }
            });
        });
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn render_meal(ui: &mut egui::Ui, meal: &Meal, mlg: bool, t: f64, idx: usize) {
    if mlg {
        let t32 = t as f32;
        let fidx = idx as f32;
        let hue = fidx.mul_add(0.17, t32 * 0.38) % 1.0;
        egui::Frame::group(ui.style())
            .stroke(egui::Stroke::new(2.5, rainbow(hue)))
            .show(ui, |ui| {
                
                let name_hue = fidx.mul_add(0.23, t32 * 0.65) % 1.0;
                let name_size = 1.5f32.mul_add(t32.mul_add(3.2, fidx).sin(), 15.0_f32);
                ui.label(
                    egui::RichText::new(&meal.name)
                        .strong()
                        .color(rainbow(name_hue))
                        .size(name_size),
                );

                ui.horizontal_wrapped(|ui| {
                    let tokens = &meal.ingredients;
                    let last = tokens.len().saturating_sub(1);
                    for (i, token) in tokens.iter().enumerate() {
                        let sep = if i < last { ", " } else { "" };
                        if token.has_milk {
                            let milk_hue = t32.mul_add(9.0, i as f32 * 0.14) % 1.0;
                            ui.label(
                                egui::RichText::new(format!(
                                    "\u{1F95B} {} \u{1F95B}{sep}",
                                    token.text.to_uppercase()
                                ))
                                .color(rainbow(milk_hue))
                                .strong()
                                .size(13.5),
                            );
                        } else {
                            let tok_hue = (i as f32).mul_add(0.055, t32 * 0.14) % 1.0;
                            ui.label(
                                egui::RichText::new(format!("{}{sep}", token.text))
                                    .color(rainbow(tok_hue).gamma_multiply(0.8)),
                            );
                        }
                    }
                });

                ui.label(
                    egui::RichText::new(format!("\u{1F4B0} {}", meal.price_info()))
                        .color(Color32::from_rgb(55, 255, 110))
                        .strong()
                        .small(),
                );
            });
    } else {
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
                                .color(Color32::from_rgb(220, 50, 50))
                                .strong(),
                        );
                    } else {
                        ui.label(display);
                    }
                }
            });

            ui.label(
                egui::RichText::new(meal.price_info())
                    .color(Color32::GRAY)
                    .small(),
            );
        });
    }
}





fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Mensa am Schloss",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MensaApp::default()))),
    )
}
