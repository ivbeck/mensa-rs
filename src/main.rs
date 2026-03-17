use chrono::Datelike;
use std::process;

mod api;
mod display;
mod ingredients;
mod meal;
mod style;

use api::cached_fetch;
use display::{terminal_width, wrap_line};
use style::{style_category, style_dim, style_header};
use meal::{parse_menu, Meal};

const COLUMN_WIDTH: usize = 16;

fn render_output(meals: &[Meal], today: &chrono::NaiveDate) -> String {
    let days_de = [
        "Montag",
        "Dienstag",
        "Mittwoch",
        "Donnerstag",
        "Freitag",
        "Samstag",
        "Sonntag",
    ];
    let weekday_idx = today.weekday().num_days_from_monday() as usize;
    let weekday = format!("{} {}", days_de[weekday_idx], today.format("%d.%m."));

    let mut lines = Vec::new();
    lines.push(style_header(&format!(
        "\u{1F37D}  Mensa am Schloss \u{2014} {weekday}"
    )));

    let width = terminal_width();

    for meal in meals {
        let name_chars = meal.name.chars().count();
        let dots = if name_chars < COLUMN_WIDTH {
            ".".repeat(COLUMN_WIDTH - name_chars)
        } else {
            String::new()
        };
        let name_col = format!("{}{}", meal.name, dots);
        let items = meal.render_items();
        let prefix = format!("  {} ", style_category(&name_col));
        lines.push(wrap_line(&prefix, &items, width));

        let price = style_dim(&meal.price_info());
        lines.push(format!("  {} {}", " ".repeat(COLUMN_WIDTH), price));
    }

    lines.join("\n")
}

fn main() {
    let today = chrono::Local::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();

    let html = match cached_fetch(&date_str) {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "{}",
                style_dim(&format!("Mensa: could not fetch menu ({e})"))
            );
            process::exit(1);
        }
    };

    let meals = parse_menu(&html);
    if meals.is_empty() {
        println!("{}", style_dim("Mensa am Schloss: no meals today"));
        return;
    }

    println!("{}", render_output(&meals, &today));
}
