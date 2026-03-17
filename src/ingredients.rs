use regex::Regex;

pub struct IngredientToken {
    pub text: String,
    pub has_milk: bool,
}

pub fn contains_milk(codes: &[&str]) -> bool {
    codes.contains(&"Mi")
}

pub fn normalize_whitespace(s: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(s, " ")
        .trim()
        .trim_matches(',')
        .trim()
        .to_string()
}

pub fn tokenize_ingredients(raw: &str) -> Vec<IngredientToken> {
    let re = Regex::new(r"(.*?)\s*\(([^)]*)\)|(.+?)$").unwrap();
    let mut tokens = Vec::new();

    for cap in re.captures_iter(raw) {
        if let Some(text_match) = cap.get(1) {
            let text = text_match.as_str().trim();
            let codes_str = cap.get(2).map_or("", |m| m.as_str());
            let codes: Vec<&str> = codes_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !text.is_empty() {
                tokens.push(IngredientToken {
                    text: normalize_whitespace(text),
                    has_milk: contains_milk(&codes),
                });
            }
        } else if let Some(tail) = cap.get(3) {
            let text = tail.as_str().trim();
            if !text.is_empty() {
                tokens.push(IngredientToken {
                    text: normalize_whitespace(text),
                    has_milk: false,
                });
            }
        }
    }

    // Fallback: if tokenization produced nothing, return the whole string
    if tokens.is_empty() && !raw.trim().is_empty() {
        let strip_parens = Regex::new(r"\s*\([^)]*\)").unwrap();
        let cleaned = strip_parens.replace_all(raw, "");
        let cleaned = normalize_whitespace(cleaned.trim());
        tokens.push(IngredientToken {
            text: cleaned,
            has_milk: false,
        });
    }

    tokens
}
