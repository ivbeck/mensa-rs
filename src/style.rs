pub fn style_danger(text: &str) -> String {
    format!("\x1b[31m\x1b[1m{}\x1b[0m", text)
}

pub fn style_category(text: &str) -> String {
    format!("\x1b[36m{}\x1b[0m", text)
}

pub fn style_dim(text: &str) -> String {
    format!("\x1b[2m{}\x1b[0m", text)
}

pub fn style_header(text: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", text)
}
