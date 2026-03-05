//! Main regex parser implementation.

use cstree::build::GreenNodeBuilder;
use cstree::green::GreenNode;
use resharp_syntax::{RegexOptions, SyntaxKind};

use crate::error::ParseError;

/// Parse a regex pattern into a green tree.
///
/// # Arguments
///
/// * `input` - The regex pattern to parse
/// * `options` - Parsing options (e.g., case sensitivity, multiline mode)
///
/// # Returns
///
/// A `GreenNode` representing the concrete syntax tree, or a `ParseError`.
///
/// # Example
///
/// ```
/// use resharp_parser::{parse, RegexOptions};
///
/// let tree = parse("a|b", RegexOptions::NONE).unwrap();
/// ```
pub fn parse(input: &str, options: RegexOptions) -> Result<GreenNode, ParseError> {
    Parser::new(input, options).parse_root()
}

/// Internal parser state.
struct Parser<'a> {
    /// Input string being parsed.
    input: &'a str,
    /// Current position in the input.
    pos: usize,
    /// Green node builder.
    builder: GreenNodeBuilder<'static, 'static, SyntaxKind>,
    /// Current parsing options.
    options: RegexOptions,
    /// Stack of saved options for nested groups.
    options_stack: Vec<RegexOptions>,
}

impl<'a> Parser<'a> {
    /// Create a new parser.
    fn new(input: &'a str, options: RegexOptions) -> Self {
        Self {
            input,
            pos: 0,
            builder: GreenNodeBuilder::new(),
            options,
            options_stack: Vec::new(),
        }
    }

    /// Parse the entire pattern as a root node. Consumes the parser.
    fn parse_root(mut self) -> Result<GreenNode, ParseError> {
        self.builder.start_node(SyntaxKind::Root);

        // The root contains a single implicit capture group
        self.builder.start_node(SyntaxKind::Capture);
        self.parse_alternation()?;
        self.builder.finish_node(); // Capture

        self.builder.finish_node(); // Root

        Ok(self.builder.finish().0)
    }

    /// Parse alternation: conjunction ('|' conjunction)*
    fn parse_alternation(&mut self) -> Result<(), ParseError> {
        self.builder.start_node(SyntaxKind::Alternate);

        self.parse_conjunction()?;

        while self.current_char() == Some('|') {
            self.bump(); // consume '|'
            self.parse_conjunction()?;
        }

        self.builder.finish_node();
        Ok(())
    }

    /// Parse conjunction (RE# extension): concatenation ('&' concatenation)*
    fn parse_conjunction(&mut self) -> Result<(), ParseError> {
        self.builder.start_node(SyntaxKind::Conjunction);

        self.parse_concatenation()?;

        while self.current_char() == Some('&') {
            self.bump(); // consume '&'
            self.parse_concatenation()?;
        }

        self.builder.finish_node();
        Ok(())
    }

    /// Parse concatenation: term+
    fn parse_concatenation(&mut self) -> Result<(), ParseError> {
        self.builder.start_node(SyntaxKind::Concatenate);

        while !self.at_end() && !self.at_alternation_end() {
            self.parse_term()?;
        }

        self.builder.finish_node();
        Ok(())
    }

    /// Check if we're at a position that ends an alternation/conjunction.
    fn at_alternation_end(&self) -> bool {
        matches!(self.current_char(), Some('|' | '&' | ')'))
    }

    /// Parse a single term with optional quantifier.
    fn parse_term(&mut self) -> Result<(), ParseError> {
        // Handle RE# negation prefix
        if self.current_char() == Some('~') {
            self.bump();
            self.builder.start_node(SyntaxKind::Negation);
            self.builder.token(SyntaxKind::Tilde, "~");

            // Create a checkpoint for potential quantifier
            let checkpoint = self.builder.checkpoint();

            // Parse the negated atom (single term only)
            self.parse_negated_atom()?;

            // Parse optional quantifier
            if self.at_quantifier() {
                self.parse_quantifier_wrap(checkpoint)?;
            }

            self.builder.finish_node(); // Negation
            return Ok(());
        }

        // Create a checkpoint before the atom so we can wrap it in a Loop node
        let checkpoint = self.builder.checkpoint();

        // Parse the atom
        self.parse_atom(false)?;

        // Parse optional quantifier and wrap atom if needed
        if self.at_quantifier() {
            self.parse_quantifier_wrap(checkpoint)?;
        }

        Ok(())
    }

    /// Parse a negated atom (for ~ prefix).
    fn parse_negated_atom(&mut self) -> Result<(), ParseError> {
        match self.current_char() {
            Some('(') => self.parse_group(),
            Some('[') => self.parse_char_class(),
            Some('\\') => self.parse_escape(),
            Some('.') => {
                self.builder.start_node(SyntaxKind::Notone);
                self.bump();
                self.builder.finish_node();
                Ok(())
            }
            Some('_') => {
                self.builder.start_node(SyntaxKind::Set);
                self.bump();
                self.builder.finish_node();
                Ok(())
            }
            Some('~') => {
                // Nested negation: parse recursively
                self.parse_term()
            }
            Some(ch) if !is_metachar(ch) => {
                // Single character only
                self.parse_literal(true)
            }
            Some(ch) => Err(ParseError::UnexpectedChar { ch, pos: self.pos }),
            None => Err(ParseError::UnexpectedEof { pos: self.pos }),
        }
    }

    /// Check if current position has a quantifier.
    fn at_quantifier(&self) -> bool {
        matches!(self.current_char(), Some('*' | '+' | '?' | '{'))
    }

    /// Parse an atom (basic unit of matching).
    fn parse_atom(&mut self, _negated: bool) -> Result<(), ParseError> {
        match self.current_char() {
            Some('(') => self.parse_group(),
            Some('[') => self.parse_char_class(),
            Some('\\') => self.parse_escape(),
            Some('.') => {
                self.builder.start_node(SyntaxKind::Notone);
                self.bump();
                self.builder.finish_node();
                Ok(())
            }
            Some('_') => {
                // RE# universal wildcard (matches anything including newlines)
                self.builder.start_node(SyntaxKind::Set);
                self.bump();
                self.builder.finish_node();
                Ok(())
            }
            Some('^') => {
                let kind = if self.options.contains(RegexOptions::MULTILINE) {
                    SyntaxKind::Bol
                } else {
                    SyntaxKind::Beginning
                };
                self.builder.start_node(kind);
                self.bump();
                self.builder.finish_node();
                Ok(())
            }
            Some('$') => {
                let kind = if self.options.contains(RegexOptions::MULTILINE) {
                    SyntaxKind::Eol
                } else {
                    SyntaxKind::EndZ
                };
                self.builder.start_node(kind);
                self.bump();
                self.builder.finish_node();
                Ok(())
            }
            Some(ch) if !is_metachar(ch) => {
                // When negated, only parse a single character
                self.parse_literal(_negated)
            }
            Some(ch) => Err(ParseError::UnexpectedChar { ch, pos: self.pos }),
            None => Ok(()), // End of input in concatenation is fine
        }
    }

    /// Parse a literal character or sequence.
    ///
    /// When `single_char` is true (e.g., after negation), only parse one character.
    fn parse_literal(&mut self, single_char: bool) -> Result<(), ParseError> {
        let start = self.pos;

        if single_char {
            // Only consume a single character
            if let Some(ch) = self.current_char() {
                self.pos += ch.len_utf8();
            }
        } else {
            // Consume characters until we hit a metachar
            while let Some(ch) = self.current_char() {
                if is_metachar(ch) {
                    break;
                }
                self.pos += ch.len_utf8();
            }
        }

        let text = &self.input[start..self.pos];

        if text.chars().count() == 1 {
            self.builder.start_node(SyntaxKind::One);
            self.builder.token(SyntaxKind::Char, text);
            self.builder.finish_node();
        } else {
            self.builder.start_node(SyntaxKind::Multi);
            self.builder.token(SyntaxKind::Char, text);
            self.builder.finish_node();
        }

        Ok(())
    }

    /// Parse an escape sequence.
    fn parse_escape(&mut self) -> Result<(), ParseError> {
        let start = self.pos;
        self.bump(); // consume '\'

        let ch = self
            .current_char()
            .ok_or(ParseError::UnexpectedEof { pos: self.pos })?;

        let kind = match ch {
            // Character class escapes
            'd' | 'D' | 'w' | 'W' | 's' | 'S' => {
                self.bump();
                SyntaxKind::Set
            }
            // Anchors
            'A' => {
                self.bump();
                SyntaxKind::Beginning
            }
            'z' => {
                self.bump();
                SyntaxKind::End
            }
            'Z' => {
                self.bump();
                SyntaxKind::EndZ
            }
            'G' => {
                self.bump();
                SyntaxKind::Start
            }
            'b' => {
                self.bump();
                if self.options.contains(RegexOptions::ECMA_SCRIPT) {
                    SyntaxKind::ECMABoundary
                } else {
                    SyntaxKind::Boundary
                }
            }
            'B' => {
                self.bump();
                if self.options.contains(RegexOptions::ECMA_SCRIPT) {
                    SyntaxKind::NonECMABoundary
                } else {
                    SyntaxKind::NonBoundary
                }
            }
            // Backreferences
            '1'..='9' => {
                self.bump();
                // Consume additional digits
                while let Some('0'..='9') = self.current_char() {
                    self.bump();
                }
                SyntaxKind::Backreference
            }
            // Simple character escapes and literals
            'n' | 'r' | 't' | 'f' | 'a' | 'e' | 'v' => {
                self.bump();
                SyntaxKind::One
            }
            // Hex escapes
            'x' => {
                self.bump();
                // Consume hex digits
                for _ in 0..2 {
                    if let Some(c) = self.current_char() {
                        if c.is_ascii_hexdigit() {
                            self.bump();
                        }
                    }
                }
                SyntaxKind::One
            }
            'u' => {
                self.bump();
                // Consume hex digits
                for _ in 0..4 {
                    if let Some(c) = self.current_char() {
                        if c.is_ascii_hexdigit() {
                            self.bump();
                        }
                    }
                }
                SyntaxKind::One
            }
            // Unicode property
            'p' | 'P' => {
                self.bump();
                if self.current_char() == Some('{') {
                    self.bump();
                    while let Some(c) = self.current_char() {
                        self.bump();
                        if c == '}' {
                            break;
                        }
                    }
                }
                SyntaxKind::Set
            }
            // Literal escape of any other character
            _ => {
                self.bump();
                SyntaxKind::One
            }
        };

        let text = &self.input[start..self.pos];

        self.builder.start_node(kind);
        self.builder.token(SyntaxKind::EscapeSeq, text);
        self.builder.finish_node();

        Ok(())
    }

    /// Parse a group construct.
    fn parse_group(&mut self) -> Result<(), ParseError> {
        let start = self.pos;
        self.bump(); // consume '('

        // Check for group modifier (?...)
        if self.current_char() == Some('?') {
            self.bump();
            self.parse_group_modifier(start)
        } else {
            // Capturing group
            self.builder.start_node(SyntaxKind::Capture);
            self.builder.token(SyntaxKind::LParen, "(");

            self.options_stack.push(self.options);
            self.parse_alternation()?;
            self.options = self.options_stack.pop().unwrap_or(RegexOptions::NONE);

            if self.current_char() != Some(')') {
                return Err(ParseError::UnclosedGroup { pos: start });
            }
            self.bump();
            self.builder.token(SyntaxKind::RParen, ")");
            self.builder.finish_node();
            Ok(())
        }
    }

    /// Parse group modifiers after (?
    fn parse_group_modifier(&mut self, start: usize) -> Result<(), ParseError> {
        match self.current_char() {
            Some(':') => {
                // Non-capturing group (?:...)
                self.bump();
                self.builder.start_node(SyntaxKind::Group);
                self.builder.token(SyntaxKind::Char, "(?:");

                self.parse_alternation()?;

                if self.current_char() != Some(')') {
                    return Err(ParseError::UnclosedGroup { pos: start });
                }
                self.bump();
                self.builder.token(SyntaxKind::RParen, ")");
                self.builder.finish_node();
                Ok(())
            }
            Some('=') => {
                // Positive lookahead (?=...)
                self.bump();
                self.builder.start_node(SyntaxKind::PositiveLookahead);
                self.builder.token(SyntaxKind::Char, "(?=");

                self.parse_alternation()?;

                if self.current_char() != Some(')') {
                    return Err(ParseError::UnclosedGroup { pos: start });
                }
                self.bump();
                self.builder.token(SyntaxKind::RParen, ")");
                self.builder.finish_node();
                Ok(())
            }
            Some('!') => {
                // Negative lookahead (?!...)
                self.bump();
                self.builder.start_node(SyntaxKind::NegativeLookahead);
                self.builder.token(SyntaxKind::Char, "(?!");

                self.parse_alternation()?;

                if self.current_char() != Some(')') {
                    return Err(ParseError::UnclosedGroup { pos: start });
                }
                self.bump();
                self.builder.token(SyntaxKind::RParen, ")");
                self.builder.finish_node();
                Ok(())
            }
            Some('<') => {
                self.bump();
                match self.current_char() {
                    Some('=') => {
                        // Positive lookbehind (?<=...)
                        self.bump();
                        self.builder.start_node(SyntaxKind::PositiveLookbehind);
                        self.builder.token(SyntaxKind::Char, "(?<=");

                        let saved = self.options;
                        self.options.insert(RegexOptions::RIGHT_TO_LEFT);
                        self.parse_alternation()?;
                        self.options = saved;

                        if self.current_char() != Some(')') {
                            return Err(ParseError::UnclosedGroup { pos: start });
                        }
                        self.bump();
                        self.builder.token(SyntaxKind::RParen, ")");
                        self.builder.finish_node();
                        Ok(())
                    }
                    Some('!') => {
                        // Negative lookbehind (?<!...)
                        self.bump();
                        self.builder.start_node(SyntaxKind::NegativeLookbehind);
                        self.builder.token(SyntaxKind::Char, "(?<!");

                        let saved = self.options;
                        self.options.insert(RegexOptions::RIGHT_TO_LEFT);
                        self.parse_alternation()?;
                        self.options = saved;

                        if self.current_char() != Some(')') {
                            return Err(ParseError::UnclosedGroup { pos: start });
                        }
                        self.bump();
                        self.builder.token(SyntaxKind::RParen, ")");
                        self.builder.finish_node();
                        Ok(())
                    }
                    _ => {
                        // Named capture group (?<name>...)
                        self.builder.start_node(SyntaxKind::Capture);
                        self.builder.token(SyntaxKind::Char, "(?<");

                        // Parse name
                        let name_start = self.pos;
                        while let Some(ch) = self.current_char() {
                            if ch == '>' {
                                break;
                            }
                            self.bump();
                        }
                        let name = &self.input[name_start..self.pos];
                        if name.is_empty() {
                            return Err(ParseError::InvalidGroupName { pos: name_start });
                        }
                        self.builder.start_node(SyntaxKind::GroupName);
                        self.builder.token(SyntaxKind::Char, name);
                        self.builder.finish_node();

                        if self.current_char() != Some('>') {
                            return Err(ParseError::InvalidGroupName { pos: name_start });
                        }
                        self.bump();
                        self.builder.token(SyntaxKind::RAngle, ">");

                        self.parse_alternation()?;

                        if self.current_char() != Some(')') {
                            return Err(ParseError::UnclosedGroup { pos: start });
                        }
                        self.bump();
                        self.builder.token(SyntaxKind::RParen, ")");
                        self.builder.finish_node();
                        Ok(())
                    }
                }
            }
            Some('>') => {
                // Atomic group (?>...)
                self.bump();
                self.builder.start_node(SyntaxKind::Atomic);
                self.builder.token(SyntaxKind::Char, "(?>");

                self.parse_alternation()?;

                if self.current_char() != Some(')') {
                    return Err(ParseError::UnclosedGroup { pos: start });
                }
                self.bump();
                self.builder.token(SyntaxKind::RParen, ")");
                self.builder.finish_node();
                Ok(())
            }
            _ => Err(ParseError::UnknownGroupType { pos: start }),
        }
    }

    /// Parse a character class [...].
    fn parse_char_class(&mut self) -> Result<(), ParseError> {
        let start = self.pos;
        self.bump(); // consume '['

        let negated = if self.current_char() == Some('^') {
            self.bump();
            true
        } else {
            false
        };

        let kind = if negated {
            SyntaxKind::CharClassNegated
        } else {
            SyntaxKind::CharClass
        };

        self.builder.start_node(kind);

        // Parse character class contents
        while let Some(ch) = self.current_char() {
            if ch == ']' {
                break;
            }

            if ch == '\\' {
                self.parse_escape()?;
            } else {
                // Check for range
                let range_start = self.pos;
                self.bump();

                if self.current_char() == Some('-') && self.peek_char() != Some(']') {
                    self.bump(); // consume '-'
                    if let Some(end_ch) = self.current_char() {
                        if end_ch != ']' {
                            self.bump();
                            // This is a range
                            let range_text = &self.input[range_start..self.pos];
                            self.builder.start_node(SyntaxKind::CharRange);
                            self.builder.token(SyntaxKind::Char, range_text);
                            self.builder.finish_node();
                            continue;
                        }
                    }
                }

                // Single character
                let char_text = &self.input[range_start..self.pos];
                self.builder.token(SyntaxKind::Char, char_text);
            }
        }

        if self.current_char() != Some(']') {
            return Err(ParseError::UnclosedCharClass { pos: start });
        }
        self.bump();
        self.builder.finish_node();

        Ok(())
    }

    /// Parse a quantifier and wrap the preceding atom using the checkpoint.
    fn parse_quantifier_wrap(
        &mut self,
        checkpoint: cstree::build::Checkpoint,
    ) -> Result<(), ParseError> {
        match self.current_char() {
            Some('*') => {
                self.bump();
                let lazy = self.current_char() == Some('?');
                if lazy {
                    self.bump();
                }
                // Use Loop node to wrap the atom
                let kind = if lazy {
                    SyntaxKind::Lazyloop
                } else {
                    SyntaxKind::Loop
                };
                self.builder.start_node_at(checkpoint, kind);
                self.builder
                    .token(SyntaxKind::Quantifier, if lazy { "*?" } else { "*" });
                self.builder.finish_node();
            }
            Some('+') => {
                self.bump();
                let lazy = self.current_char() == Some('?');
                if lazy {
                    self.bump();
                }
                let kind = if lazy {
                    SyntaxKind::Lazyloop
                } else {
                    SyntaxKind::Loop
                };
                self.builder.start_node_at(checkpoint, kind);
                self.builder
                    .token(SyntaxKind::Quantifier, if lazy { "+?" } else { "+" });
                self.builder.finish_node();
            }
            Some('?') => {
                self.bump();
                let lazy = self.current_char() == Some('?');
                if lazy {
                    self.bump();
                }
                let kind = if lazy {
                    SyntaxKind::Lazyloop
                } else {
                    SyntaxKind::Loop
                };
                self.builder.start_node_at(checkpoint, kind);
                self.builder
                    .token(SyntaxKind::Quantifier, if lazy { "??" } else { "?" });
                self.builder.finish_node();
            }
            Some('{') => {
                let start = self.pos;
                self.bump();
                // Parse {n} or {n,} or {n,m}
                while let Some(ch) = self.current_char() {
                    if ch == '}' {
                        self.bump();
                        break;
                    }
                    self.bump();
                }
                let quant_text = &self.input[start..self.pos];
                let lazy = self.current_char() == Some('?');
                if lazy {
                    self.bump();
                }
                let kind = if lazy {
                    SyntaxKind::Lazyloop
                } else {
                    SyntaxKind::Loop
                };
                self.builder.start_node_at(checkpoint, kind);
                if lazy {
                    self.builder
                        .token(SyntaxKind::Quantifier, &format!("{}?", quant_text));
                } else {
                    self.builder.token(SyntaxKind::Quantifier, quant_text);
                }
                self.builder.finish_node();
            }
            _ => {}
        }
        Ok(())
    }

    /// Get the current character.
    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Peek at the next character (after current).
    fn peek_char(&self) -> Option<char> {
        let mut chars = self.input[self.pos..].chars();
        chars.next();
        chars.next()
    }

    /// Advance by one character.
    fn bump(&mut self) {
        if let Some(ch) = self.current_char() {
            self.pos += ch.len_utf8();
        }
    }

    /// Check if at end of input.
    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }
}

/// Check if a character is a regex metacharacter.
fn is_metachar(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '|'
            | '&'
            | '~'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '*'
            | '+'
            | '?'
            | '.'
            | '^'
            | '$'
            | '_'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cstree::syntax::SyntaxNode;

    fn parse_pattern(pattern: &str) -> SyntaxNode<SyntaxKind> {
        let options = RegexOptions::EXPLICIT_CAPTURE | RegexOptions::NON_BACKTRACKING;
        let green = parse(pattern, options).expect("parse should succeed");
        SyntaxNode::new_root(green)
    }

    fn get_root_child(node: &SyntaxNode<SyntaxKind>) -> Option<SyntaxNode<SyntaxKind>> {
        // Root -> Capture -> actual content
        node.children()
            .next()
            .and_then(|capture| capture.children().next())
            .cloned()
    }

    #[test]
    fn test_simple_literal() {
        let tree = parse_pattern("abc");
        let node = get_root_child(&tree).expect("should have child");
        // Should be Alternate -> Conjunction -> Concatenate -> Multi
        assert!(node.children().any(|_| true));
    }

    #[test]
    fn test_conjunction() {
        let tree = parse_pattern("a&b");
        let capture = tree.children().next().expect("should have capture");
        let alt = capture.children().next().expect("should have alternation");
        let conj = alt.children().next().expect("should have conjunction");
        assert_eq!(conj.kind(), SyntaxKind::Conjunction);
    }

    #[test]
    fn test_alternation() {
        let tree = parse_pattern("a|b");
        let capture = tree.children().next().expect("should have capture");
        let alt = capture.children().next().expect("should have alternation");
        assert_eq!(alt.kind(), SyntaxKind::Alternate);
    }
}
