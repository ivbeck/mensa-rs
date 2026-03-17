use regex::Regex;

pub fn visible_len(s: &str) -> usize {
    let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").chars().count()
}

pub fn terminal_width() -> usize {
    use std::mem::zeroed;
    unsafe {
        let mut ws: libc::winsize = zeroed();
        if libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            ws.ws_col as usize
        } else {
            80
        }
    }
}

pub fn wrap_line(prefix: &str, text: &str, width: usize) -> String {
    let indent = " ".repeat(visible_len(prefix));
    let avail = width.saturating_sub(visible_len(prefix));
    if avail < 20 {
        return format!("{}{}", prefix, text);
    }

    let words: Vec<&str> = text.split(' ').collect();
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_vis = 0usize;

    for word in &words {
        let wlen = visible_len(word);
        if current.is_empty() {
            current.push_str(word);
            current_vis = wlen;
        } else if current_vis + 1 + wlen > avail {
            lines.push(current);
            current = word.to_string();
            current_vis = wlen;
        } else {
            current.push(' ');
            current.push_str(word);
            current_vis += 1 + wlen;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }

    let mut result = format!("{}{}", prefix, lines.first().unwrap_or(&String::new()));
    for line in lines.iter().skip(1) {
        result.push('\n');
        result.push_str(&indent);
        result.push_str(line);
    }
    result
}

pub fn strip_tags(html: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    let text = re.replace_all(html, "");
    html_escape::decode_html_entities(&text).trim().to_string()
}
