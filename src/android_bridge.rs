use serde::Serialize;

use crate::api::fetch_menu;
use crate::meal::{parse_menu, Meal, MealError};

#[derive(Debug, thiserror::Error)]
pub enum AndroidBridgeError {
    #[error("Failed to fetch menu: {0}")]
    Fetch(#[from] crate::api::ApiError),
    #[error("Failed to parse menu: {0}")]
    Parse(#[from] MealError),
    #[error("Failed to encode menu: {0}")]
    Encode(#[from] serde_json::Error),
}

#[derive(Serialize)]
struct MenuResponse<'a> {
    ok: bool,
    date: &'a str,
    lang: &'a str,
    meals: Vec<AndroidMeal<'a>>,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    ok: bool,
    date: &'a str,
    lang: &'a str,
    error: String,
    meals: Vec<AndroidMeal<'a>>,
}

#[derive(Serialize)]
struct AndroidMeal<'a> {
    name: &'a str,
    price: String,
    favorite: bool,
    has_allergen: bool,
    items: Vec<AndroidItem<'a>>,
}

#[derive(Serialize)]
struct AndroidItem<'a> {
    text: &'a str,
    has_allergen: bool,
}

/// # Errors
/// Returns an error if the HTML cannot be parsed or the response cannot be encoded.
pub fn menu_json_from_html(
    html: &str,
    date_str: &str,
    lang: &str,
    allergens: &[String],
    hide_allergens: bool,
    favorites: &[String],
) -> Result<String, AndroidBridgeError> {
    let meals = parse_menu(html)?;
    encode_meals(&meals, date_str, lang, allergens, hide_allergens, favorites)
}

/// Fetches and encodes the menu for Android. This always returns JSON so the
/// Java UI can render either data or an error without JNI exceptions.
#[must_use]
pub fn fetch_menu_json(
    date_str: &str,
    lang: &str,
    allergens_csv: &str,
    hide_allergens: bool,
    favorites_csv: &str,
) -> String {
    let allergens = split_csv(allergens_csv);
    let favorites = split_csv(favorites_csv);
    match fetch_menu(date_str, lang) {
        Ok(html) => {
            match menu_json_from_html(
                &html,
                date_str,
                lang,
                &allergens,
                hide_allergens,
                &favorites,
            ) {
                Ok(json) => json,
                Err(error) => error_json(date_str, lang, &error.to_string()),
            }
        }
        Err(error) => error_json(date_str, lang, &error.to_string()),
    }
}

fn encode_meals(
    meals: &[Meal],
    date_str: &str,
    lang: &str,
    allergens: &[String],
    hide_allergens: bool,
    favorites: &[String],
) -> Result<String, AndroidBridgeError> {
    let meals = meals
        .iter()
        .filter(|meal| !hide_allergens || !meal.has_any_allergen(allergens))
        .map(|meal| AndroidMeal {
            name: &meal.name,
            price: meal.price_info(),
            favorite: meal.matches_favorites(favorites),
            has_allergen: meal.has_any_allergen(allergens),
            items: meal
                .ingredients
                .iter()
                .map(|item| AndroidItem {
                    text: &item.text,
                    has_allergen: item.has_any_code(allergens),
                })
                .collect(),
        })
        .collect();

    serde_json::to_string(&MenuResponse {
        ok: true,
        date: date_str,
        lang,
        meals,
    })
    .map_err(Into::into)
}

fn error_json(date_str: &str, lang: &str, error: &str) -> String {
    let response = ErrorResponse {
        ok: false,
        date: date_str,
        lang,
        error: error.to_owned(),
        meals: Vec::new(),
    };
    serde_json::to_string(&response).unwrap_or_else(|_| {
        format!(
            r#"{{"ok":false,"date":"{date_str}","lang":"{lang}","error":"{error}","meals":[]}}"#
        )
    })
}

fn split_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(target_os = "android")]
mod jni_bridge {
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jstring};
    use jni::JNIEnv;

    use super::fetch_menu_json;

    #[no_mangle]
    pub extern "system" fn Java_de_ivbeck_mensa_MenuBridge_fetchMenuJson(
        mut env: JNIEnv<'_>,
        _class: JClass<'_>,
        date: JString<'_>,
        lang: JString<'_>,
        allergens: JString<'_>,
        hide_allergens: jboolean,
        favorites: JString<'_>,
    ) -> jstring {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let date = java_string(&mut env, &date);
            let lang = java_string(&mut env, &lang);
            let allergens = java_string(&mut env, &allergens);
            let favorites = java_string(&mut env, &favorites);
            fetch_menu_json(&date, &lang, &allergens, hide_allergens != 0, &favorites)
        }))
        .unwrap_or_else(|_| {
            r#"{"ok":false,"date":"","lang":"","error":"Rust panic","meals":[]}"#.to_owned()
        });

        env.new_string(result)
            .map_or(std::ptr::null_mut(), JString::into_raw)
    }

    fn java_string(env: &mut JNIEnv<'_>, value: &JString<'_>) -> String {
        env.get_string(value)
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::menu_json_from_html;

    #[test]
    fn android_json_filters_allergens_and_marks_favorites() {
        let html = r#"
        <table>
            <tr>
                <td class="speiseplan-table-menu-headline"><strong>Menü 1</strong></td>
                <td class="speiseplan-table-menu-content">Pasta (Weizen), Pudding (Mi)</td>
                <td><i class="price">4,00 €</i></td>
                <td><i class="customSelection">Portion</i></td>
            </tr>
            <tr>
                <td class="speiseplan-table-menu-headline"><strong>Vegan</strong></td>
                <td class="speiseplan-table-menu-content">Curry, Reis</td>
                <td><i class="price">3,50 €</i></td>
                <td><i class="customSelection">Portion</i></td>
            </tr>
        </table>
        "#;

        let json = menu_json_from_html(
            html,
            "2026-05-26",
            "de",
            &["Mi".to_owned()],
            true,
            &["curry".to_owned()],
        )
        .expect("fixture should encode");
        let value: Value = serde_json::from_str(&json).expect("json should parse");
        let meals = value["meals"].as_array().expect("meals should be an array");

        assert_eq!(meals.len(), 1);
        assert_eq!(meals[0]["name"], "Vegan");
        assert!(meals[0]["favorite"]
            .as_bool()
            .expect("favorite should be bool"));
    }
}
