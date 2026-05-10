use std::collections::HashSet;
use std::fs;
use std::io;

/// Parse a Makefile-style depfile (gcc -MD output)
///
/// Format: target: dep1 dep2 dep3 \
///                 dep4 dep5
pub fn parse(path: &str) -> io::Result<DepfileResult> {
    let content = fs::read_to_string(path)?;
    parse_content(&content)
}

/// Parsed depfile result
#[derive(Debug, Default)]
pub struct DepfileResult {
    /// The target (output) file
    pub target: String,
    /// Dependencies discovered
    pub deps: HashSet<String>,
}

fn parse_content(content: &str) -> io::Result<DepfileResult> {
    let mut result = DepfileResult::default();

    // Join continuation lines
    let content = content.replace("\\\n", " ").replace("\\\r\n", " ");

    // Split on first colon
    let (target, deps) = content
        .split_once(':')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "depfile missing ':'"))?;

    result.target = target.trim().to_string();

    // Parse dependencies (space-separated, may have escaped spaces)
    let mut current = String::new();
    let mut chars = deps.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Escape sequence
                if let Some(&next) = chars.peek() {
                    if next == ' ' || next == '#' || next == '\\' {
                        // Safe to unwrap here since we just peeked
                        if let Some(escaped) = chars.next() {
                            current.push(escaped);
                        }
                    } else {
                        current.push(c);
                    }
                }
                // If chars.peek() returns None, the backslash is at end of input
                // We just skip it (treat as literal backslash)
            }
            ' ' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    result.deps.insert(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        result.deps.insert(current);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_depfile() {
        let content = "foo.o: foo.c foo.h bar.h";
        let result = parse_content(content).unwrap();
        assert_eq!(result.target, "foo.o");
        assert!(result.deps.contains("foo.c"));
        assert!(result.deps.contains("foo.h"));
        assert!(result.deps.contains("bar.h"));
    }

    #[test]
    fn test_multiline_depfile() {
        let content = "foo.o: foo.c \\\n  foo.h \\\n  bar.h";
        let result = parse_content(content).unwrap();
        assert_eq!(result.target, "foo.o");
        assert_eq!(result.deps.len(), 3);
    }

    #[test]
    fn test_escaped_spaces() {
        let content = r"foo.o: path\ with\ spaces.c";
        let result = parse_content(content).unwrap();
        assert!(result.deps.contains("path with spaces.c"));
    }
}
