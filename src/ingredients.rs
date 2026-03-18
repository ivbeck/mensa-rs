use regex::Regex;

#[derive(Debug, thiserror::Error)]
pub enum IngredientError {
    #[error("Regex error: {0}")]
    ParseError(#[from] regex::Error),
}

pub struct IngredientToken {
    pub text: String,
    pub has_milk: bool,
}

#[must_use]
pub fn contains_milk(codes: &[&str]) -> bool {
    codes.contains(&"Mi")
}

/// # Errors
/// Returns an error if the whitespace-normalizing regex cannot be compiled.
pub fn normalize_whitespace(s: &str) -> Result<String, IngredientError> {
    let re = Regex::new(r"\s+")?;
    Ok(re
        .replace_all(s, " ")
        .trim()
        .trim_matches(',')
        .trim()
        .to_string())
}

/// # Errors
/// Returns an error if any ingredient-parsing regex cannot be compiled.
pub fn tokenize_ingredients(raw: &str) -> Result<Vec<IngredientToken>, IngredientError> {
    let re = Regex::new(r"(.*?)\s*\(([^)]*)\)|(.+?)$")?;
    let mut tokens = Vec::new();

    for cap in re.captures_iter(raw) {
        if let Some(text_match) = cap.get(1) {
            let text = text_match.as_str().trim();
            let codes_str = cap.get(2).map_or("", |m| m.as_str());
            let codes: Vec<&str> = codes_str
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if !text.is_empty() {
                tokens.push(IngredientToken {
                    text: normalize_whitespace(text)?,
                    has_milk: contains_milk(&codes),
                });
            }
        } else if let Some(tail) = cap.get(3) {
            let text = tail.as_str().trim();
            if !text.is_empty() {
                tokens.push(IngredientToken {
                    text: normalize_whitespace(text)?,
                    has_milk: false,
                });
            }
        }
    }

    // Fallback: if tokenization produced nothing, return the whole string
    if tokens.is_empty() && !raw.trim().is_empty() {
        let strip_parens = Regex::new(r"\s*\([^)]*\)")?;
        let cleaned = strip_parens.replace_all(raw, "");
        let cleaned = normalize_whitespace(cleaned.trim());
        tokens.push(IngredientToken {
            text: cleaned?,
            has_milk: false,
        });
    }

    Ok(tokens)
}
