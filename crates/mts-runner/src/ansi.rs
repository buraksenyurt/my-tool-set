//! Removes ANSI escape sequences (colors, cursor moves) from a line, so build
//! output reads cleanly in the log panel.

pub fn strip(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            // CSI: ESC [ params... final byte in @..~
            Some('[') => {
                chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            // OSC: ESC ] ... (BEL | ESC \)
            Some(']') => {
                chars.next();
                while let Some(c) = chars.next() {
                    if c == '\u{7}' {
                        break;
                    }
                    if c == '\u{1b}' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            }
            // Two-character sequences such as ESC ( B.
            Some(_) => {
                chars.next();
            }
            None => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::strip;

    #[test]
    fn removes_colors_cursor_moves_and_links() {
        assert_eq!(
            strip("\u{1b}[1;32m   Compiling\u{1b}[0m mts"),
            "   Compiling mts"
        );
        assert_eq!(strip("a\u{1b}[2Kb\u{1b}[1Gc"), "abc");
        assert_eq!(
            strip("\u{1b}]8;;https://x\u{1b}\\link\u{1b}]8;;\u{7}"),
            "link"
        );
        assert_eq!(strip("düz metin"), "düz metin");
        assert_eq!(strip("sonda \u{1b}"), "sonda ");
    }
}
