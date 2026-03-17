const API_URL: &str = "https://api.stw-ma.de/tl1/menuplan";
const MENU_ID: &str = "bc003e93a1e942f45a99dcf8082da289";
const LOCATION: &str = "610";

pub fn fetch_menu(date_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = ureq::post(API_URL).send_form(&[
        ("id", MENU_ID),
        ("location", LOCATION),
        ("lang", "de"),
        ("date", date_str),
        ("mode", "day"),
    ])?;

    let body: serde_json::Value = resp.into_json()?;
    body["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "missing 'content' field in API response".into())
}

fn cache_dir() -> std::path::PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            std::path::PathBuf::from(home).join(".cache")
        });

    dbg!(base.join("mensa"));
    base.join("mensa")
}

pub fn cached_fetch(date_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let dir = cache_dir();
    let path = dir.join(format!("{}.html", date_str));

    if let Ok(html) = std::fs::read_to_string(&path) {
        return Ok(html);
    }

    let html = fetch_menu(date_str)?;
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(&path, &html);
    Ok(html)
}
