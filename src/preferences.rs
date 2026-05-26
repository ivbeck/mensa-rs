use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::Deserialize;

const DEFAULT_LANG: &str = "de";
const DEFAULT_ALLERGENS: &[&str] = &["Mi"];

#[derive(Debug, Deserialize)]
struct ConfigFile {
    language: Option<String>,
    no_cache: Option<bool>,
    allergens: Option<Vec<String>>,
    hide_allergens: Option<bool>,
    favorites: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Preferences {
    pub language: String,
    pub no_cache: bool,
    pub allergens: Vec<String>,
    pub hide_allergens: bool,
    pub favorites: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PreferencesError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: DEFAULT_LANG.to_owned(),
            no_cache: false,
            allergens: DEFAULT_ALLERGENS.iter().map(|code| (*code).to_owned()).collect(),
            hide_allergens: false,
            favorites: Vec::new(),
        }
    }
}

impl Preferences {
    #[must_use]
    fn with_config(config: ConfigFile) -> Self {
        let mut prefs = Self::default();

        if let Some(language) = clean_language(config.language.as_deref()) {
            language.clone_into(&mut prefs.language);
        }
        if let Some(no_cache) = config.no_cache {
            prefs.no_cache = no_cache;
        }
        if let Some(allergens) = config.allergens {
            prefs.allergens = clean_list(allergens);
        }
        if let Some(hide_allergens) = config.hide_allergens {
            prefs.hide_allergens = hide_allergens;
        }
        if let Some(favorites) = config.favorites {
            prefs.favorites = clean_list(favorites);
        }

        prefs
    }

    pub fn set_language(&mut self, language: &str) -> bool {
        if let Some(clean) = clean_language(Some(language)) {
            clean.clone_into(&mut self.language);
            true
        } else {
            false
        }
    }

    pub fn add_allergen(&mut self, code: &str) {
        push_unique(&mut self.allergens, code);
    }

    pub fn add_favorite(&mut self, favorite: &str) {
        push_unique(&mut self.favorites, favorite);
    }
}

#[must_use]
pub fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").map_or_else(
        |_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".config")
        },
        PathBuf::from,
    );

    base.join("mensa").join("config.toml")
}

/// # Errors
/// Returns an error if the config file exists but cannot be read or parsed.
pub fn load_preferences() -> Result<Preferences, PreferencesError> {
    let path = config_path();
    match std::fs::read_to_string(path) {
        Ok(raw) => {
            let config = toml::from_str::<ConfigFile>(&raw)?;
            Ok(Preferences::with_config(config))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Preferences::default()),
        Err(e) => Err(e.into()),
    }
}

fn clean_language(language: Option<&str>) -> Option<&str> {
    language.and_then(|lang| match lang.trim() {
        "de" => Some("de"),
        "en" => Some("en"),
        _ => None,
    })
}

fn clean_list(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter_map(|value| {
            let clean = value.trim();
            if clean.is_empty() || !seen.insert(clean.to_ascii_lowercase()) {
                None
            } else {
                Some(clean.to_owned())
            }
        })
        .collect()
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    let clean = value.trim();
    if clean.is_empty() {
        return;
    }

    if !values
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(clean))
    {
        values.push(clean.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigFile, Preferences};

    #[test]
    fn config_overrides_defaults_and_deduplicates_lists() {
        let prefs = Preferences::with_config(ConfigFile {
            language: Some("en".to_owned()),
            no_cache: Some(true),
            allergens: Some(vec!["Mi".to_owned(), "mi".to_owned(), "Ei".to_owned()]),
            hide_allergens: Some(true),
            favorites: Some(vec!["Curry".to_owned(), " curry ".to_owned()]),
        });

        assert_eq!(prefs.language, "en");
        assert!(prefs.no_cache);
        assert_eq!(prefs.allergens, ["Mi", "Ei"]);
        assert!(prefs.hide_allergens);
        assert_eq!(prefs.favorites, ["Curry"]);
    }
}
