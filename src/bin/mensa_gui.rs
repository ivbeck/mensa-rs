#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::mpsc;
use std::thread;

use chrono::{Datelike as _, Duration};
use eframe::egui;
use eframe::egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Pos2, Rect, Rounding, Stroke, Vec2};

use mensa::api::cached_fetch;
use mensa::meal::{parse_menu, Meal};
use mensa::preferences::{load_preferences, Preferences};

const MLG_PHRASES: &[&str] = &[
    "420", "MLG", "YOLO", "360 NOSCOPE", "DANK", "REKT", "SWAG", "DORITOS",
    "MTN DEW", "PRO GAMER", "GGWP", "EZ", "CLUTCH", "PWNED", "HEADSHOT", "ACE",
    "HACKS", "MONTAGE", "SICK", "BEAST MODE", "NO SCOPE", "TRIGGERED", "OMEGALUL",
    "KEKW", "POG",
];

// Editorial dark palette
const BG_DEEP: Color32 = Color32::from_rgb(0x10, 0x0E, 0x0B);
const BG_SURFACE: Color32 = Color32::from_rgb(0x1B, 0x17, 0x12);
const BG_ELEVATED: Color32 = Color32::from_rgb(0x24, 0x1F, 0x18);
const INK: Color32 = Color32::from_rgb(0xF1, 0xEA, 0xDD);
const INK_MUTED: Color32 = Color32::from_rgb(0x95, 0x8B, 0x7C);
const INK_DIM: Color32 = Color32::from_rgb(0x5C, 0x53, 0x47);
const ACCENT: Color32 = Color32::from_rgb(0xE4, 0xA3, 0x3D);
const ACCENT_HOT: Color32 = Color32::from_rgb(0xF4, 0xBA, 0x55);
const SAGE: Color32 = Color32::from_rgb(0x9C, 0xB5, 0x82);
const OXBLOOD_BG: Color32 = Color32::from_rgb(0x4A, 0x1C, 0x18);
const OXBLOOD_INK: Color32 = Color32::from_rgb(0xFF, 0xC9, 0xC0);
const OXBLOOD_STROKE: Color32 = Color32::from_rgb(0x8C, 0x33, 0x2C);
const RULE: Color32 = Color32::from_rgb(0x2D, 0x26, 0x1E);

// Font family identifiers
const FAM_DISPLAY: &str = "display";
const FAM_BODY: &str = "body";
const FAM_BODY_MEDIUM: &str = "body_medium";

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
    preferences: Preferences,
    today: chrono::NaiveDate,
    rx: Option<mpsc::Receiver<Result<Vec<Meal>, String>>>,
    mlg_mode: bool,
    /// Time (`ctx.input.time`) when current `Ready` state began — used for stagger reveal.
    ready_started_at: f64,
}

impl MensaApp {
    fn start_fetch(&mut self) {
        let date_str = self.today.format("%Y-%m-%d").to_string();
        let lang = self.preferences.language.clone();
        let no_cache = self.preferences.no_cache;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.state = FetchState::Loading;
        drop(thread::spawn(move || {
            drop(tx.send(do_fetch(&date_str, &lang, no_cache)));
        }));
    }

    #[must_use]
    fn weekday_label(&self) -> (String, String) {
        const DAYS_DE: [&str; 7] = [
            "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag", "Sonntag",
        ];
        const DAYS_EN: [&str; 7] = [
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];
        let idx = self.today.weekday().num_days_from_monday() as usize;
        if self.preferences.language == "en" {
            (
                DAYS_EN[idx].to_owned(),
                self.today.format("%m / %d / %Y").to_string(),
            )
        } else {
            (
                DAYS_DE[idx].to_owned(),
                self.today.format("%d.%m.%Y").to_string(),
            )
        }
    }
}

impl Default for MensaApp {
    fn default() -> Self {
        let mut app = Self {
            state: FetchState::Loading,
            preferences: load_preferences().unwrap_or_else(|_| Preferences::default()),
            today: chrono::Local::now().date_naive(),
            rx: None,
            mlg_mode: false,
            ready_started_at: 0.0,
        };
        app.start_fetch();
        app
    }
}

// --- Type setup --------------------------------------------------------------

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FAM_DISPLAY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Fraunces-SemiBold.ttf")),
    );
    fonts.font_data.insert(
        FAM_BODY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/IBMPlexSans-Regular.ttf")),
    );
    fonts.font_data.insert(
        FAM_BODY_MEDIUM.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/IBMPlexSans-Medium.ttf")),
    );

    fonts.families.insert(
        FontFamily::Name(FAM_DISPLAY.into()),
        vec![FAM_DISPLAY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(FAM_BODY_MEDIUM.into()),
        vec![FAM_BODY_MEDIUM.to_owned(), FAM_BODY.to_owned()],
    );

    // Make Plex the default proportional font so every untouched widget benefits.
    if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
        prop.insert(0, FAM_BODY.to_owned());
    }

    ctx.set_fonts(fonts);
}

fn display_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FAM_DISPLAY.into()))
}

const fn body_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

fn body_medium_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FAM_BODY_MEDIUM.into()))
}

fn apply_editorial_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(INK);
    visuals.panel_fill = BG_DEEP;
    visuals.window_fill = BG_DEEP;
    visuals.extreme_bg_color = BG_DEEP;
    visuals.faint_bg_color = BG_SURFACE;
    visuals.code_bg_color = BG_SURFACE;
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = Color32::from_rgb(0x4A, 0x36, 0x16);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, RULE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, INK_MUTED);
    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, RULE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, INK_MUTED);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);
    visuals.widgets.hovered.bg_fill = BG_ELEVATED;
    visuals.widgets.hovered.weak_bg_fill = BG_ELEVATED;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, INK);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);
    visuals.widgets.active.bg_fill = BG_ELEVATED;
    visuals.widgets.active.weak_bg_fill = BG_ELEVATED;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_HOT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, ACCENT_HOT);
    visuals.widgets.active.rounding = Rounding::same(4.0);
    visuals.widgets.open.bg_fill = BG_ELEVATED;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.window_stroke = Stroke::new(1.0, RULE);
    visuals.window_rounding = Rounding::same(2.0);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 6.0);
    style.spacing.window_margin = egui::Margin::symmetric(0.0, 0.0);
    style.spacing.scroll.bar_width = 6.0;
    ctx.set_style(style);
}

// --- MLG mode helpers --------------------------------------------------------

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

// --- Editorial primitives ----------------------------------------------------

fn hairline(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, Rounding::ZERO, color);
}

fn allergen_pill(ui: &mut egui::Ui, text: &str) {
    let label = text.to_uppercase();
    let font = body_medium_font(11.0);
    let galley = ui.painter().layout_no_wrap(label, font, OXBLOOD_INK);
    let pad_x = 7.0_f32;
    let pad_y = 3.0_f32;
    let size = Vec2::new(
        pad_x.mul_add(2.0, galley.size().x),
        pad_y.mul_add(2.0, galley.size().y),
    );
    let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect(
        rect,
        Rounding::same(3.0),
        OXBLOOD_BG,
        Stroke::new(1.0, OXBLOOD_STROKE),
    );
    let text_pos = Pos2::new(rect.min.x + pad_x, rect.min.y + pad_y);
    painter.galley(text_pos, galley, OXBLOOD_INK);
}

/// Translucency ramp for staggered reveal. Returns 0..1.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn reveal_alpha(now: f64, start: f64, idx: usize) -> f32 {
    let stagger = 0.06_f64 * idx as f64;
    let dur = 0.35_f64;
    let elapsed = (now - start - stagger).max(0.0);
    let progress = (elapsed / dur).min(1.0) as f32;
    // ease-out cubic
    1.0 - (1.0 - progress).powi(3)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ui_with_alpha(alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        INK.r(),
        INK.g(),
        INK.b(),
        (alpha * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn color_with_alpha(c: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        c.r(),
        c.g(),
        c.b(),
        (f32::from(c.a()) * alpha).round().clamp(0.0, 255.0) as u8,
    )
}

// --- App ---------------------------------------------------------------------

impl eframe::App for MensaApp {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::too_many_lines
    )]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = ctx.input(|i| i.time);
        let t32 = t as f32;

        // Drain worker.
        if let Some(recv) = self.rx.as_ref().map(mpsc::Receiver::try_recv) {
            match recv {
                Ok(fetch_result) => {
                    self.rx = None;
                    self.state = match fetch_result {
                        Ok(meals) if meals.is_empty() => FetchState::Empty,
                        Ok(meals) => {
                            self.ready_started_at = t;
                            FetchState::Ready(meals)
                        }
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

        // Mode-specific visuals.
        if self.mlg_mode {
            ctx.request_repaint();
            paint_mlg_overlay(ctx, t);
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = Color32::from_rgba_premultiplied(8, 0, 20, 245);
            visuals.window_fill = Color32::from_rgba_premultiplied(8, 0, 20, 245);
            visuals.widgets.noninteractive.bg_stroke =
                Stroke::new(1.0, rainbow((t32 * 0.3) % 1.0).gamma_multiply(0.5));
            ctx.set_visuals(visuals);
        } else {
            apply_editorial_visuals(ctx);
        }

        // Stagger animation needs continuous repaint until done.
        if matches!(self.state, FetchState::Ready(_)) && t - self.ready_started_at < 2.5 {
            ctx.request_repaint();
        }

        let central_frame = egui::Frame::none()
            .fill(if self.mlg_mode { Color32::TRANSPARENT } else { BG_DEEP })
            .inner_margin(egui::Margin::symmetric(36.0, 28.0));

        egui::CentralPanel::default().frame(central_frame).show(ctx, |ui| {
            if self.mlg_mode {
                paint_mlg_background(ui.painter(), ui.max_rect(), t);
                render_mlg_header(ui, self, t32);
                ui.add_space(8.0);
                render_mlg_toolbar(ui, self, ctx, t32);
                ui.add_space(12.0);
                render_mlg_body(ui, self, t);
            } else {
                render_editorial_header(ui, self);
                ui.add_space(18.0);
                hairline(ui, RULE);
                ui.add_space(10.0);
                render_editorial_toolbar(ui, self);
                ui.add_space(6.0);
                hairline(ui, RULE);
                ui.add_space(16.0);
                render_editorial_body(ui, self, t);
            }
        });
    }
}

// --- Editorial rendering -----------------------------------------------------

fn render_editorial_header(ui: &mut egui::Ui, app: &MensaApp) {
    let (weekday, date) = app.weekday_label();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            // Eyebrow
            ui.label(
                egui::RichText::new(if app.preferences.language == "en" { "TODAY · LUNCH" } else { "HEUTE · MITTAG" })
                    .font(body_medium_font(10.5))
                    .color(ACCENT)
                    .extra_letter_spacing(2.4),
            );
            ui.add_space(2.0);
            // Title
            ui.label(
                egui::RichText::new("Mensa am Schloss")
                    .font(display_font(36.0))
                    .color(INK),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.vertical(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(&date)
                            .font(body_font(12.5))
                            .color(INK_MUTED),
                    );
                });
                ui.add_space(2.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(&weekday)
                            .font(display_font(20.0))
                            .color(ACCENT),
                    );
                });
            });
        });
    });
}

fn render_editorial_toolbar(ui: &mut egui::Ui, app: &mut MensaApp) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(14.0, 6.0);

        if toggle_chip(ui, "DE", app.preferences.language == "de") && app.preferences.language != "de" {
            app.preferences.set_language("de");
            app.start_fetch();
        }
        if toggle_chip(ui, "EN", app.preferences.language == "en") && app.preferences.language != "en" {
            app.preferences.set_language("en");
            app.start_fetch();
        }

        ui.add_space(8.0);
        tiny_divider(ui);
        ui.add_space(8.0);

        if ghost_button(ui, "PREV").clicked() {
            app.today -= Duration::days(1);
            app.start_fetch();
        }
        if ghost_button(ui, "TODAY").clicked() {
            app.today = chrono::Local::now().date_naive();
            app.start_fetch();
        }
        if ghost_button(ui, "NEXT").clicked() {
            app.today += Duration::days(1);
            app.start_fetch();
        }

        ui.add_space(8.0);
        tiny_divider(ui);
        ui.add_space(8.0);

        if ghost_button(ui, "REFRESH").clicked() {
            app.start_fetch();
        }

        ui.add_space(4.0);
        let cache_label = if app.preferences.no_cache { "BYPASS CACHE · ON" } else { "BYPASS CACHE" };
        if ghost_button(ui, cache_label).clicked() {
            app.preferences.no_cache = !app.preferences.no_cache;
        }

        let allergen_label = if app.preferences.hide_allergens { "HIDE ALLERGENS · ON" } else { "HIDE ALLERGENS" };
        if ghost_button(ui, allergen_label).clicked() {
            app.preferences.hide_allergens = !app.preferences.hide_allergens;
        }

        // MLG toggle pinned right.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if mlg_pill_button(ui).clicked() {
                app.mlg_mode = true;
            }
        });
    });
}

fn toggle_chip(ui: &mut egui::Ui, label: &str, active: bool) -> bool {
    let font = body_medium_font(11.5);
    let color = if active { INK } else { INK_MUTED };
    let text = egui::RichText::new(label)
        .font(font)
        .color(color)
        .extra_letter_spacing(1.4);
    let resp = ui.add(egui::Button::new(text).frame(false));
    if active {
        let r = resp.rect;
        let y = r.max.y + 1.0;
        ui.painter().line_segment(
            [Pos2::new(r.min.x, y), Pos2::new(r.max.x, y)],
            Stroke::new(1.5, ACCENT),
        );
    }
    resp.clicked()
}

fn tiny_divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 14.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, Rounding::ZERO, RULE);
}

fn ghost_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let text = egui::RichText::new(label)
        .font(body_medium_font(11.0))
        .color(INK_MUTED)
        .extra_letter_spacing(1.8);
    ui.add(egui::Button::new(text).frame(false))
}

fn mlg_pill_button(ui: &mut egui::Ui) -> egui::Response {
    let text = egui::RichText::new("MLG MODE ▸")
        .font(body_medium_font(11.0))
        .color(ACCENT)
        .extra_letter_spacing(1.8);
    let btn = egui::Button::new(text)
        .frame(true)
        .stroke(Stroke::new(1.0, ACCENT))
        .fill(Color32::TRANSPARENT)
        .rounding(Rounding::same(2.0));
    ui.add(btn)
}

fn render_editorial_body(ui: &mut egui::Ui, app: &MensaApp, t: f64) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| match &app.state {
            FetchState::Loading => render_loading_editorial(ui, app, t),
            FetchState::Empty => render_empty_editorial(ui, app),
            FetchState::Failed(msg) => render_error_editorial(ui, msg),
            FetchState::Ready(meals) => render_editorial_meals(ui, app, meals, t),
        });
}

fn render_editorial_meals(ui: &mut egui::Ui, app: &MensaApp, meals: &[Meal], t: f64) {
    let visible = visible_meals(meals, app);
    if visible.is_empty() {
        render_empty_editorial(ui, app);
        if app.preferences.hide_allergens && !meals.is_empty() {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Filtered by allergen preferences.")
                        .font(body_font(12.0))
                        .color(INK_MUTED),
                );
            });
        }
        return;
    }

    for (idx, meal) in visible.iter().enumerate() {
        let alpha = reveal_alpha(t, app.ready_started_at, idx);
        render_editorial_meal(ui, app, meal, idx, alpha);
        if idx + 1 < visible.len() {
            ui.add_space(14.0);
            hairline(ui, RULE);
            ui.add_space(14.0);
        }
    }
    ui.add_space(24.0);
}

fn visible_meals<'a>(meals: &'a [Meal], app: &MensaApp) -> Vec<&'a Meal> {
    meals
        .iter()
        .filter(|meal| {
            !app.preferences.hide_allergens
                || !meal.has_any_allergen(&app.preferences.allergens)
        })
        .collect()
}

#[allow(clippy::cast_possible_truncation)]
fn render_loading_editorial(ui: &mut egui::Ui, app: &MensaApp, t: f64) {
    ui.add_space(60.0);
    ui.vertical_centered(|ui| {
        let osc = (t * 1.6).sin() as f32;
        let pulse = osc.mul_add(0.25, 0.75);
        ui.label(
            egui::RichText::new("•")
                .font(display_font(28.0))
                .color(color_with_alpha(ACCENT, pulse)),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(if app.preferences.language == "en" { "Setting the table…" } else { "Wird angerichtet…" })
                .font(body_font(13.0))
                .color(INK_MUTED),
        );
    });
}

fn render_empty_editorial(ui: &mut egui::Ui, app: &MensaApp) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(if app.preferences.language == "en" { "Kitchen is dark today." } else { "Heute bleibt die Küche kalt." })
                .font(display_font(22.0))
                .color(INK),
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(if app.preferences.language == "en" { "No meals listed for this date." } else { "Keine Gerichte für heute eingetragen." })
                .font(body_font(12.5))
                .color(INK_MUTED),
        );
    });
}

fn render_error_editorial(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(30.0);
    egui::Frame::none()
        .fill(BG_SURFACE)
        .stroke(Stroke::new(1.0, OXBLOOD_STROKE))
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(16.0, 12.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("UNAVAILABLE")
                    .font(body_medium_font(10.5))
                    .color(OXBLOOD_INK)
                    .extra_letter_spacing(2.0),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(msg)
                    .font(body_font(12.5))
                    .color(INK),
            );
        });
}

#[allow(clippy::cast_precision_loss)]
fn render_editorial_meal(ui: &mut egui::Ui, app: &MensaApp, meal: &Meal, idx: usize, alpha: f32) {
    let row_alpha = alpha;
    let y_offset = (1.0 - alpha) * 6.0;
    let favorite = meal.matches_favorites(&app.preferences.favorites);
    ui.add_space(y_offset);

    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(14.0, 6.0);

        // Number gutter (fixed width)
        let (gutter_rect, _) =
            ui.allocate_exact_size(Vec2::new(26.0, 22.0), egui::Sense::hover());
        let num = format!("{:02}", idx + 1);
        ui.painter().text(
            gutter_rect.left_top() + Vec2::new(0.0, 4.0),
            egui::Align2::LEFT_TOP,
            num,
            display_font(13.0),
            color_with_alpha(INK_DIM, row_alpha),
        );

        // Meal column takes rest of width
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&meal.name)
                        .font(display_font(20.0))
                        .color(ui_with_alpha(row_alpha)),
                );
                if favorite {
                    ui.label(
                        egui::RichText::new("FAVORITE")
                            .font(body_medium_font(10.5))
                            .color(color_with_alpha(ACCENT, row_alpha))
                            .extra_letter_spacing(1.4),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(
                        egui::RichText::new(meal.price_info())
                            .font(body_medium_font(13.5))
                            .color(color_with_alpha(SAGE, row_alpha)),
                    );
                });
            });
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
                let tokens = &meal.ingredients;
                let last = tokens.len().saturating_sub(1);
                for (i, token) in tokens.iter().enumerate() {
                    if token.has_any_code(&app.preferences.allergens) {
                        allergen_pill(ui, &token.text);
                    } else {
                        let txt = token.text.trim();
                        ui.label(
                            egui::RichText::new(txt)
                                .font(body_font(12.5))
                                .color(color_with_alpha(INK_MUTED, row_alpha)),
                        );
                    }
                    if i < last {
                        ui.label(
                            egui::RichText::new("·")
                                .font(body_font(12.5))
                                .color(color_with_alpha(INK_DIM, row_alpha)),
                        );
                    }
                }
            });
        });
    });
}

// --- Editorial toolbar (full impl, replacing the stub above) -----------------
// (We define this here because Rust allows multiple `fn` names only once; the
//  earlier `render_editorial_toolbar` was a stub placeholder. We swap it via
//  the binding below.)

// --- MLG rendering (compat shims so existing API stays) ----------------------

fn render_mlg_header(ui: &mut egui::Ui, app: &MensaApp, t32: f32) {
    let (weekday, date) = app.weekday_label();
    let hue = (t32 * 0.55) % 1.0;
    let pulse = 0.07_f32.mul_add((t32 * 5.5).sin(), 1.0);
    ui.label(
        egui::RichText::new(format!(
            "\u{1F3AE}\u{1F525} MENSA AM SCHLOSS \u{1F525}\u{1F3AE} \u{2014} {weekday} {date} \u{1F480}"
        ))
        .color(rainbow(hue))
        .strong()
        .size(22.0 * pulse),
    );
}

fn render_mlg_toolbar(ui: &mut egui::Ui, app: &mut MensaApp, ctx: &egui::Context, t32: f32) {
    ui.horizontal(|ui| {
        if ui.selectable_label(app.preferences.language == "de", "DE").clicked() {
            app.preferences.set_language("de");
            app.start_fetch();
        }
        if ui.selectable_label(app.preferences.language == "en", "EN").clicked() {
            app.preferences.set_language("en");
            app.start_fetch();
        }
        ui.separator();
        if ui.button("\u{25C0}").clicked() {
            app.today -= Duration::days(1);
            app.start_fetch();
        }
        if ui.button("Today").clicked() {
            app.today = chrono::Local::now().date_naive();
            app.start_fetch();
        }
        if ui.button("\u{25B6}").clicked() {
            app.today += Duration::days(1);
            app.start_fetch();
        }
        ui.separator();
        if ui.button("\u{27F3}  Refresh").clicked() {
            app.start_fetch();
        }
        ui.checkbox(&mut app.preferences.no_cache, "No cache");
        ui.checkbox(&mut app.preferences.hide_allergens, "Hide allergens");
        ui.separator();
        let mlg_label = "\u{1F3AE} MLG MODE: ON \u{1F525}";
        let btn_color = rainbow((t32 * 2.8) % 1.0);
        if ui
            .add(egui::Button::new(
                egui::RichText::new(mlg_label).color(btn_color).strong(),
            ))
            .clicked()
        {
            app.mlg_mode = false;
            ctx.set_visuals(egui::Visuals::dark());
        }
    });
}

#[allow(clippy::cast_possible_truncation)]
fn render_mlg_body(ui: &mut egui::Ui, app: &MensaApp, t: f64) {
    let t32 = t as f32;
    egui::ScrollArea::vertical().show(ui, |ui| match &app.state {
        FetchState::Loading => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new("\u{1F525} FETCHING THE DANKEST MENU\u{2026} \u{1F525}")
                        .color(rainbow((t32 * 3.5) % 1.0))
                        .strong(),
                );
            });
        }
        FetchState::Empty => {
            ui.label(
                egui::RichText::new("\u{1F480} NO MEALS TODAY \u{2014} YOU GOT REKT. \u{1F480}")
                    .color(Color32::from_rgb(255, 60, 60))
                    .strong()
                    .size(18.0),
            );
        }
        FetchState::Failed(msg) => {
            ui.label(
                egui::RichText::new(format!("\u{1F480} EPIC FAIL: {msg} \u{1F480}"))
                    .color(rainbow((t32 * 5.5) % 1.0))
                    .strong(),
            );
        }
        FetchState::Ready(meals) => {
            let visible = visible_meals(meals, app);
            if visible.is_empty() {
                ui.label(
                    egui::RichText::new("\u{1F480} ALL MEALS FILTERED \u{1F480}")
                        .color(Color32::from_rgb(255, 60, 60))
                        .strong()
                        .size(18.0),
                );
                return;
            }
            for (idx, meal) in visible.iter().enumerate() {
                render_mlg_meal(ui, app, meal, t, idx);
                ui.add_space(6.0);
            }
        }
    });
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn render_mlg_meal(ui: &mut egui::Ui, app: &MensaApp, meal: &Meal, t: f64, idx: usize) {
    let t32 = t as f32;
    let fidx = idx as f32;
    let hue = fidx.mul_add(0.17, t32 * 0.38) % 1.0;
    let favorite = meal.matches_favorites(&app.preferences.favorites);
    egui::Frame::group(ui.style())
        .stroke(Stroke::new(2.5, rainbow(hue)))
        .show(ui, |ui| {
            let name_hue = fidx.mul_add(0.23, t32 * 0.65) % 1.0;
            let name_size = 1.5_f32.mul_add(t32.mul_add(3.2, fidx).sin(), 15.0);
            ui.label(
                egui::RichText::new(&meal.name)
                    .strong()
                    .color(rainbow(name_hue))
                    .size(name_size),
            );
            if favorite {
                ui.label(
                    egui::RichText::new("\u{2B50} FAVORITE MATCH \u{2B50}")
                        .color(Color32::from_rgb(255, 220, 80))
                        .strong(),
                );
            }
            ui.horizontal_wrapped(|ui| {
                let tokens = &meal.ingredients;
                let last = tokens.len().saturating_sub(1);
                for (i, token) in tokens.iter().enumerate() {
                    let sep = if i < last { ", " } else { "" };
                    if token.has_any_code(&app.preferences.allergens) {
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
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Mensa am Schloss",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([720.0, 720.0])
                .with_min_inner_size([520.0, 480.0])
                .with_title("Mensa am Schloss"),
            ..Default::default()
        },
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            apply_editorial_visuals(&cc.egui_ctx);
            Ok(Box::new(MensaApp::default()))
        }),
    )
}
