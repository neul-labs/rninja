/// Simple line-based lexer for ninja files
pub struct Lexer<'a> {
    content: &'a str,
    lines: std::iter::Peekable<std::str::Lines<'a>>,
    line_number: usize,
    /// Buffer for line continuation
    continued_line: Option<String>,
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            lines: content.lines().peekable(),
            line_number: 0,
            continued_line: None,
        }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// Peek at the next line without consuming it
    pub fn peek_line(&mut self) -> Option<&str> {
        self.lines.peek().copied()
    }

    /// Get the next logical line (handling $ line continuations)
    pub fn next_line(&mut self) -> Option<String> {
        let mut result = String::new();
        let mut first = true;

        loop {
            let line = self.lines.next()?;
            self.line_number += 1;

            if first {
                first = false;
            } else {
                // For continuation lines, add a space
                result.push(' ');
            }

            // Check for line continuation (ends with $)
            if line.ends_with('$') && !line.ends_with("$$") {
                result.push_str(&line[..line.len() - 1]);
                continue;
            }

            result.push_str(line);
            break;
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_lines() {
        let content = "line1\nline2\nline3";
        let mut lexer = Lexer::new(content);
        assert_eq!(lexer.next_line(), Some("line1".to_string()));
        assert_eq!(lexer.next_line(), Some("line2".to_string()));
        assert_eq!(lexer.next_line(), Some("line3".to_string()));
        assert_eq!(lexer.next_line(), None);
    }

    #[test]
    fn test_line_continuation() {
        let content = "line1 $\ncontinued";
        let mut lexer = Lexer::new(content);
        assert_eq!(lexer.next_line(), Some("line1  continued".to_string()));
    }

    #[test]
    fn test_escaped_dollar() {
        let content = "echo $$PATH";
        let mut lexer = Lexer::new(content);
        assert_eq!(lexer.next_line(), Some("echo $$PATH".to_string()));
    }

    #[test]
    fn test_peek() {
        let content = "line1\nline2";
        let mut lexer = Lexer::new(content);
        assert_eq!(lexer.peek_line(), Some("line1"));
        assert_eq!(lexer.peek_line(), Some("line1"));
        assert_eq!(lexer.next_line(), Some("line1".to_string()));
        assert_eq!(lexer.peek_line(), Some("line2"));
    }
}
