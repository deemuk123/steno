/// Layer 1: Remove structural noise from text before substitution.
/// Targets: redundant blank lines, trailing whitespace, markdown decoration noise.
/// Does NOT remove meaningful markdown (headers, bullets stay — they carry structure).
pub fn strip(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut prev_blank = false;

    for line in &lines {
        let trimmed = line.trim_end();

        // Collapse multiple blank lines into one
        if trimmed.is_empty() {
            if !prev_blank {
                out.push("");
            }
            prev_blank = true;
        } else {
            out.push(trimmed);
            prev_blank = false;
        }
    }

    // Remove leading/trailing blank lines
    while out.first() == Some(&"") {
        out.remove(0);
    }
    while out.last() == Some(&"") {
        out.pop();
    }

    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strips_trailing_whitespace() {
        assert_eq!(strip("hello   \nworld  "), "hello\nworld");
    }

    #[test]
    fn test_collapses_blank_lines() {
        assert_eq!(strip("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn test_removes_leading_trailing_blanks() {
        assert_eq!(strip("\n\nhello\n\n"), "hello");
    }

    #[test]
    fn test_preserves_single_blank_line() {
        assert_eq!(strip("a\n\nb"), "a\n\nb");
    }

    #[test]
    fn test_preserves_markdown_headers() {
        let input = "# Title\n\n## Section\n\nText";
        assert_eq!(strip(input), "# Title\n\n## Section\n\nText");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(strip(""), "");
    }
}
