pub fn style_danger(text: &str) -> String {
    format!("\x1b[31m\x1b[1m{text}\x1b[0m")
}

pub fn style_category(text: &str) -> String {
    format!("\x1b[36m{text}\x1b[0m")
}

pub fn style_dim(text: &str) -> String {
    format!("\x1b[2m{text}\x1b[0m")
}

pub fn style_header(text: &str) -> String {
    format!("\x1b[1m{text}\x1b[0m")
}
