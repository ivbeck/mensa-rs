use regex::Regex;

use crate::display::{strip_tags, DisplayError};
use crate::ingredients::{tokenize_ingredients, IngredientToken};
use crate::style::style_danger;

#[derive(Debug, thiserror::Error)]
pub enum MealError {
    #[error("Failed to parse meal data: {0}")]
    Parse(#[from] regex::Error),
    #[error("Failed to parse ingredients: {0}")]
    Ingredient(#[from] crate::ingredients::IngredientError),
    #[error("Failed to display meal: {0}")]
    Display(#[from] DisplayError),
}

pub struct Meal {
    pub name: String,
    pub ingredients: Vec<IngredientToken>,
    pub price: String,
    pub unit: String,
}

impl Meal {
    #[must_use]
    pub fn render_items(&self) -> String {
        self.ingredients
            .iter()
            .map(|t| {
                if t.has_milk {
                    style_danger(&t.text.to_uppercase())
                } else {
                    t.text.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[must_use]
    pub fn price_info(&self) -> String {
        if self.unit == "Portion" {
            self.price.clone()
        } else {
            let unit = self.unit.replace("pro", "");
            format!("{}/{}", self.price, unit.trim())
        }
    }
}

/// # Errors
/// Returns an error if the HTML parsing regex cannot be compiled or ingredient
/// tokenization fails.
pub fn parse_menu(html: &str) -> Result<Vec<Meal>, MealError> {
    let re = Regex::new(
        r#"(?s)<tr[^>]*>.*?<td class="speiseplan-table-menu-headline">\s*<strong>\s*(.*?)\s*</strong>.*?<td class="speiseplan-table-menu-content">\s*(.*?)\s*</td>.*?<i class="price">(.*?)</i>.*?<i class="customSelection">(.*?)</i>"#,
    )?;

    re.captures_iter(html)
        .map(|cap| {
            let name = strip_tags(&cap[1])?;
            let raw_items = strip_tags(&cap[2])?;
            let price = strip_tags(&cap[3])?;
            let unit = strip_tags(&cap[4])?;
            let ingredients = tokenize_ingredients(&raw_items)?;

            Ok(Meal {
                name,
                ingredients,
                price,
                unit,
            })
        })
        .collect()
}
