use scraper::{ElementRef, Html, Selector};

use crate::display::{strip_tags, DisplayError};
use crate::ingredients::{normalize_whitespace, tokenize_ingredients, IngredientToken};
use crate::style::style_danger;

#[derive(Debug, thiserror::Error)]
pub enum MealError {
    #[error("Failed to build HTML selector: {0}")]
    Selector(String),
    #[error("Failed to parse ingredients: {0}")]
    Ingredient(#[from] crate::ingredients::IngredientError),
    #[error("Failed to display meal: {0}")]
    Display(#[from] DisplayError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meal {
    pub name: String,
    pub ingredients: Vec<IngredientToken>,
    pub price: String,
    pub unit: String,
}

impl Meal {
    #[must_use]
    pub fn render_items(&self) -> String {
        self.render_items_with_allergens(&["Mi".to_owned()])
    }

    #[must_use]
    pub fn render_items_with_allergens(&self, allergens: &[String]) -> String {
        self.ingredients
            .iter()
            .map(|t| {
                if t.has_any_code(allergens) {
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

    #[must_use]
    pub fn has_any_allergen(&self, allergens: &[String]) -> bool {
        self.ingredients
            .iter()
            .any(|token| token.has_any_code(allergens))
    }

    #[must_use]
    pub fn matches_favorites(&self, favorites: &[String]) -> bool {
        if favorites.is_empty() {
            return false;
        }

        let haystack = self.search_text();
        favorites.iter().any(|favorite| {
            let needle = favorite.trim().to_ascii_lowercase();
            !needle.is_empty() && haystack.contains(&needle)
        })
    }

    fn search_text(&self) -> String {
        let mut text = self.name.to_ascii_lowercase();
        for token in &self.ingredients {
            text.push(' ');
            text.push_str(&token.text.to_ascii_lowercase());
        }
        text
    }
}

/// # Errors
/// Returns an error if the HTML selectors cannot be compiled or ingredient
/// tokenization fails.
pub fn parse_menu(html: &str) -> Result<Vec<Meal>, MealError> {
    let document = Html::parse_fragment(html);
    let row = selector("tr")?;
    let headline = selector("td.speiseplan-table-menu-headline strong")?;
    let content = selector("td.speiseplan-table-menu-content")?;
    let price = selector("i.price")?;
    let unit = selector("i.customSelection")?;

    document
        .select(&row)
        .filter_map(|row| {
            let name = row.select(&headline).next()?;
            let content = row.select(&content).next()?;
            let price = row.select(&price).next()?;
            let unit = row.select(&unit).next()?;
            Some(parse_row(name, content, price, unit))
        })
        .collect()
}

fn parse_row(
    name: ElementRef<'_>,
    content: ElementRef<'_>,
    price: ElementRef<'_>,
    unit: ElementRef<'_>,
) -> Result<Meal, MealError> {
    let name = element_text(name)?;
    let raw_items = element_text(content)?;
    let price = element_text(price)?;
    let unit = element_text(unit)?;
    let ingredients = tokenize_ingredients(&raw_items)?;

    Ok(Meal {
        name,
        ingredients,
        price,
        unit,
    })
}

fn selector(selector: &str) -> Result<Selector, MealError> {
    Selector::parse(selector).map_err(|e| MealError::Selector(e.to_string()))
}

fn element_text(element: ElementRef<'_>) -> Result<String, MealError> {
    let text = element.text().collect::<Vec<_>>().join(" ");
    if text.trim().is_empty() {
        return strip_tags(&element.html()).map_err(Into::into);
    }
    normalize_whitespace(&text).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::parse_menu;

    #[test]
    fn parses_menu_rows_with_allergen_codes() {
        let html = r#"
        <table>
            <tr>
                <td class="speiseplan-table-menu-headline"><strong>Pasta</strong></td>
                <td class="speiseplan-table-menu-content">
                    Spaghetti Bolognese (Rind, Ei, Weizen), Pudding (Mi)
                </td>
                <td><i class="price">2,40 €</i></td>
                <td><i class="customSelection">Portion</i></td>
            </tr>
        </table>
        "#;

        let meals = parse_menu(html).expect("fixture should parse");

        assert_eq!(meals.len(), 1);
        assert_eq!(meals[0].name, "Pasta");
        assert_eq!(meals[0].price_info(), "2,40 €");
        assert!(meals[0].has_any_allergen(&["Mi".to_owned()]));
        assert!(meals[0].matches_favorites(&["pudding".to_owned()]));
    }
}
