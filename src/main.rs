use chrono::Datelike;
use std::process;

mod api;
mod display;
mod ingredients;
mod meal;
mod style;

use api::cached_fetch;
use display::{terminal_width, wrap_line};
use meal::{parse_menu, Meal};
use style::{style_category, style_dim, style_header};

const COLUMN_WIDTH: usize = 16;

struct Args {
    lang: String,
    no_cache: bool,
}

fn parse_args() -> Args {
    let mut lang = "de".to_owned();
    let mut no_cache = false;
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--lang" => {
                if let Some(l) = iter.next() {
                    match l.as_str() {
                        "de" | "en" => lang = l,
                        other => eprintln!("Unknown lang '{other}', using 'de'"),
                    }
                }
            }
            "--no-cache" => no_cache = true,
            other => eprintln!("Unknown argument '{other}'"),
        }
    }
    Args { lang, no_cache }
}

fn render_output(
    meals: &[Meal],
    today: chrono::NaiveDate,
    lang: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let weekday_idx = today.weekday().num_days_from_monday() as usize;
    let weekday = if lang == "en" {
        let days_en = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        format!("{} {}", days_en[weekday_idx], today.format("%m/%d"))
    } else {
        let days_de = [
            "Montag",
            "Dienstag",
            "Mittwoch",
            "Donnerstag",
            "Freitag",
            "Samstag",
            "Sonntag",
        ];
        format!("{} {}", days_de[weekday_idx], today.format("%d.%m."))
    };

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
        lines.push(wrap_line(&prefix, &items, width)?);

        let price = style_dim(&meal.price_info());
        lines.push(format!("  {} {}", " ".repeat(COLUMN_WIDTH), price));
    }

    Ok(lines.join("\n"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    let today = chrono::Local::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();

    let html = match cached_fetch(&date_str, &args.lang, args.no_cache) {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "{}",
                style_dim(&format!("Mensa: could not fetch menu ({e})"))
            );
            process::exit(1);
        }
    };

    let meals = parse_menu(&html)?;
    if meals.is_empty() {
        println!("{}", style_dim("Mensa am Schloss: no meals today"));
        return Ok(());
    }

    println!("{}", render_output(&meals, today, &args.lang)?);

    Ok(())
}
