//! Strips a trailing comment (`# ...`) outside any quoted text.

/// Strips a trailing comment (`# ...`) outside any quoted text.
/// Returns `None` if a text literal is unterminated, otherwise a slice of
/// `text` up to (but not including) the comment start.
pub fn strip_trailing_comment(text: &str) -> Option<&str> {
    let mut in_text = false;
    for (i, ch) in text.char_indices() {
        if in_text {
            if ch == '"' {
                in_text = false;
            }
        } else if ch == '"' {
            in_text = true;
        } else if ch == '#' {
            return Some(&text[..i]);
        }
    }
    if in_text {
        None
    } else {
        Some(text)
    }
}
