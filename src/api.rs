const API_URL: &str = "https://api.stw-ma.de/tl1/menuplan";
const MENU_ID: &str = "bc003e93a1e942f45a99dcf8082da289";
const LOCATION: &str = "610";

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    #[error("Failed to parse API response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("API response missing expected 'content' field")]
    MissingContent,
}

/// # Errors
/// Returns an error if the HTTP request fails or the API response is malformed.
pub fn fetch_menu(date_str: &str, lang: &str) -> Result<String, ApiError> {
    let resp = ureq::post(API_URL)
        .send_form(&[
            ("id", MENU_ID),
            ("location", LOCATION),
            ("lang", lang),
            ("date", date_str),
            ("mode", "day"),
        ])
        .map_err(|e| ApiError::HttpError(e.to_string()))?;

    let body: serde_json::Value = resp.into_json()?;
    body["content"]
        .as_str()
        .map(std::string::ToString::to_string)
        .ok_or(ApiError::MissingContent)
}

fn cache_dir() -> std::path::PathBuf {
    let base = std::env::var("XDG_CACHE_HOME").map_or_else(
        |_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            std::path::PathBuf::from(home).join(".cache")
        },
        std::path::PathBuf::from,
    );

    base.join("mensa")
}

/// # Errors
/// Returns an error if the HTTP request fails or the response cannot be parsed.
pub fn cached_fetch(
    date_str: &str,
    lang: &str,
    no_cache: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let dir = cache_dir();
    let path = dir.join(format!("{date_str}_{lang}.html"));

    if !no_cache {
        if let Ok(html) = std::fs::read_to_string(&path) {
            return Ok(html);
        }
    }

    let html = fetch_menu(date_str, lang)?;
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(&path, &html);
    Ok(html)
}
