use regex::Regex;

#[derive(Debug, thiserror::Error)]
pub enum DisplayError {
    #[error("Failed to parse visible length: {0}")]
    VisibleLength(String),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

/// # Errors
/// Returns an error if the ANSI-stripping regex cannot be compiled.
pub fn visible_len(s: &str) -> Result<usize, DisplayError> {
    let re = Regex::new(r"\x1b\[[0-9;]*m")
        .map_err(|e| DisplayError::VisibleLength(e.to_string()))?;
    Ok(re.replace_all(s, "").chars().count())
}

#[must_use]
pub fn terminal_width() -> usize {
    #[cfg(unix)]
    {
        use std::mem::zeroed;
        unsafe {
            let mut ws: libc::winsize = zeroed();
            if libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
                return ws.ws_col as usize;
            }
        }
    }
    80
}

/// # Errors
/// Returns an error if visible-length calculation fails.
pub fn wrap_line(prefix: &str, text: &str, width: usize) -> Result<String, DisplayError> {
    let indent = " ".repeat(visible_len(prefix)?);
    let avail = width.saturating_sub(visible_len(prefix)?);
    if avail < 20 {
        return Ok(format!("{prefix}{text}"));
    }

    let words: Vec<&str> = text.split(' ').collect();
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_vis = 0usize;

    for word in &words {
        let wlen = visible_len(word)?;
        if current.is_empty() {
            current.push_str(word);
            current_vis = wlen;
        } else if current_vis + 1 + wlen > avail {
            lines.push(current);
            current = (*word).to_string();
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
    Ok(result)
}

/// # Errors
/// Returns an error if the tag-stripping regex cannot be compiled.
pub fn strip_tags(html: &str) -> Result<String, DisplayError> {
    let re = Regex::new(r"<[^>]+>")?;
    let text = re.replace_all(html, "");
    Ok(html_escape::decode_html_entities(&text).trim().to_string())
}
