use chrono::{Datelike as _, Duration, NaiveDate};
use std::process;

use mensa::api::cached_fetch;
use mensa::display::{terminal_width, wrap_line};
use mensa::meal::{parse_menu, Meal};
use mensa::preferences::{config_path, load_preferences, Preferences};
use mensa::style::{style_category, style_dim, style_header};

const COLUMN_WIDTH: usize = 16;

#[derive(Clone, Copy)]
enum View {
    Day,
    Week,
}

struct Args {
    preferences: Preferences,
    date: NaiveDate,
    view: View,
}

fn parse_args() -> Args {
    let mut preferences = load_preferences().unwrap_or_else(|e| {
        eprintln!("Mensa: could not load config ({e}), using defaults");
        Preferences::default()
    });
    let mut date = chrono::Local::now().date_naive();
    let mut view = View::Day;
    let mut iter = std::env::args().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--lang" => {
                if let Some(l) = iter.next() {
                    if !preferences.set_language(&l) {
                        eprintln!("Unknown lang '{l}', keeping {}", preferences.language);
                    }
                }
            }
            "--date" => {
                if let Some(raw) = iter.next() {
                    match NaiveDate::parse_from_str(&raw, "%Y-%m-%d") {
                        Ok(parsed) => date = parsed,
                        Err(e) => eprintln!("Invalid date '{raw}' ({e}), keeping {date}"),
                    }
                }
            }
            "--tomorrow" => date += Duration::days(1),
            "--week" => view = View::Week,
            "--no-cache" => preferences.no_cache = true,
            "--hide-allergens" => preferences.hide_allergens = true,
            "--show-allergens" => preferences.hide_allergens = false,
            "--allergen" => {
                if let Some(code) = iter.next() {
                    preferences.add_allergen(&code);
                }
            }
            "--allergens" => {
                if let Some(codes) = iter.next() {
                    preferences.allergens.clear();
                    add_csv(&mut preferences.allergens, &codes);
                }
            }
            "--favorite" => {
                if let Some(favorite) = iter.next() {
                    preferences.add_favorite(&favorite);
                }
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            other => eprintln!("Unknown argument '{other}'"),
        }
    }

    Args {
        preferences,
        date,
        view,
    }
}

fn add_csv(values: &mut Vec<String>, raw: &str) {
    for value in raw.split(',') {
        let clean = value.trim();
        if clean.is_empty() {
            continue;
        }
        if !values
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(clean))
        {
            values.push(clean.to_owned());
        }
    }
}

fn print_help() {
    let help = [
        "Usage: mensa [OPTIONS]",
        "",
        "Options:",
        "  --lang de|en             Set menu language",
        "  --date YYYY-MM-DD        Show a specific date",
        "  --tomorrow               Show tomorrow",
        "  --week                   Show Monday-Friday for the selected week",
        "  --no-cache               Fetch fresh menu data",
        "  --allergen CODE          Highlight another allergen code",
        "  --allergens A,B          Replace highlighted allergen codes",
        "  --hide-allergens         Hide meals with configured allergens",
        "  --show-allergens         Show allergen matches again",
        "  --favorite WORD          Mark meals matching a keyword",
        "  --help                   Show this help",
    ]
    .join("\n");

    println!(
        "{help}\n\nConfig: {}",
        config_path().display()
    );
}

fn render_output(
    meals: &[Meal],
    date: NaiveDate,
    preferences: &Preferences,
) -> Result<String, Box<dyn std::error::Error>> {
    let weekday = weekday_label(date, &preferences.language);
    let mut lines = vec![style_header(&format!(
        "\u{1F37D}  Mensa am Schloss \u{2014} {weekday}"
    ))];
    let width = terminal_width();
    let mut hidden_count = 0usize;
    let mut visible_count = 0usize;

    for meal in meals {
        if should_hide(meal, preferences) {
            hidden_count += 1;
            continue;
        }
        visible_count += 1;
        push_meal_lines(&mut lines, meal, preferences, width)?;
    }

    if visible_count == 0 {
        let msg = if preferences.language == "en" {
            "Mensa am Schloss: no meals for this date"
        } else {
            "Mensa am Schloss: keine Gerichte fuer dieses Datum"
        };
        lines.push(style_dim(msg));
    }

    if hidden_count > 0 {
        lines.push(style_dim(&format!(
            "{hidden_count} hidden by allergen filter"
        )));
    }

    Ok(lines.join("\n"))
}

fn weekday_label(date: NaiveDate, lang: &str) -> String {
    let weekday_idx = date.weekday().num_days_from_monday() as usize;
    if lang == "en" {
        let days_en = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        format!("{} {}", days_en[weekday_idx], date.format("%m/%d"))
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
        format!("{} {}", days_de[weekday_idx], date.format("%d.%m."))
    }
}

fn should_hide(meal: &Meal, preferences: &Preferences) -> bool {
    preferences.hide_allergens && meal.has_any_allergen(&preferences.allergens)
}

fn push_meal_lines(
    lines: &mut Vec<String>,
    meal: &Meal,
    preferences: &Preferences,
    width: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let favorite = meal.matches_favorites(&preferences.favorites);
    let name = if favorite {
        format!("* {}", meal.name)
    } else {
        meal.name.clone()
    };
    let name_chars = name.chars().count();
    let dots = if name_chars < COLUMN_WIDTH {
        ".".repeat(COLUMN_WIDTH - name_chars)
    } else {
        String::new()
    };
    let name_col = format!("{name}{dots}");
    let items = meal.render_items_with_allergens(&preferences.allergens);
    let prefix = format!("  {} ", style_category(&name_col));
    lines.push(wrap_line(&prefix, &items, width)?);

    let price = style_dim(&meal.price_info());
    let label = if favorite { " FAVORITE" } else { "" };
    lines.push(format!("  {} {}{label}", " ".repeat(COLUMN_WIDTH), price));
    Ok(())
}

fn fetch_meals(date: NaiveDate, preferences: &Preferences) -> Result<Vec<Meal>, Box<dyn std::error::Error>> {
    let date_str = date.format("%Y-%m-%d").to_string();
    let html = cached_fetch(&date_str, &preferences.language, preferences.no_cache)?;
    parse_menu(&html).map_err(Into::into)
}

fn week_dates(date: NaiveDate) -> Vec<NaiveDate> {
    let monday = date - Duration::days(i64::from(date.weekday().num_days_from_monday()));
    (0_i64..5).map(|offset| monday + Duration::days(offset)).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    let dates = match args.view {
        View::Day => vec![args.date],
        View::Week => week_dates(args.date),
    };
    let mut sections = Vec::new();

    for date in dates {
        let meals = match fetch_meals(date, &args.preferences) {
            Ok(meals) => meals,
            Err(e) => {
                eprintln!("{}", style_dim(&format!("Mensa: could not fetch menu ({e})")));
                process::exit(1);
            }
        };
        sections.push(render_output(&meals, date, &args.preferences)?);
    }

    println!("{}", sections.join("\n\n"));

    Ok(())
}
