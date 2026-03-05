//! RE# regex engine with boolean operations.
//!
//! This crate provides a regex engine that supports standard regex syntax
//! plus RE# extensions:
//!
//! - **Conjunction** (`&`): Match both patterns, e.g., `.*foo.*&.*bar.*`
//! - **Negation** (`~`): Match anything except pattern, e.g., `~(.*error.*)`
//! - **Universal wildcard** (`_`): Match any character including newlines
//!
//! # Example
//!
//! ```
//! use resharp::Regex;
//!
//! // Compile a pattern
//! let re = Regex::new(r"\d+").unwrap();
//! println!("Pattern: {}", re.pattern());
//! println!("IR: {}", re.pretty_print());
//!
//! // RE# conjunction: patterns requiring both conditions
//! let re = Regex::new(r".*foo.*&.*bar.*").unwrap();
//! assert!(!re.pattern().is_empty());
//!
//! // RE# negation: patterns excluding certain content
//! let re = Regex::new(r"~(.*error.*)").unwrap();
//! assert!(!re.pattern().is_empty());
//! ```
//!
//! # Status
//!
//! This crate currently supports:
//! - Pattern compilation and IR generation
//! - Pretty printing for debugging
//!
//! Full matching support is in development.

use cstree::syntax::SyntaxNode;
use resharp_ir::{
    derivative, is_nullable, BitSetSolver, LocationKind, NodeId, PrettyPrinter, RegexBuilder,
    RegexNodeArena,
};
use resharp_parser::{cst_to_ir, parse, ParseError, RegexOptions};
use resharp_syntax::SyntaxKind;

/// A compiled RE# regex pattern.
#[derive(Debug)]
pub struct Regex {
    /// The original pattern string.
    pattern: String,
    /// The IR arena containing all nodes.
    arena: RegexNodeArena<u64>,
    /// The root node ID of the pattern.
    root: NodeId,
}

/// A match found in the input string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match {
    /// Start index of the match (byte offset).
    pub start: usize,
    /// End index of the match (byte offset, exclusive).
    pub end: usize,
}

impl Match {
    /// Returns the length of the match in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if this is an empty match.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Extract the matched text from the input.
    pub fn as_str<'a>(&self, input: &'a str) -> &'a str {
        &input[self.start..self.end]
    }
}

/// Error type for regex compilation.
#[derive(Debug)]
pub enum Error {
    /// Parse error.
    Parse(ParseError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "parse error: {:?}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}

impl Regex {
    /// Compile a regex pattern.
    ///
    /// # Example
    ///
    /// ```
    /// use resharp::Regex;
    ///
    /// let re = Regex::new(r"\d+").unwrap();
    /// assert!(re.is_match("42"));
    /// ```
    pub fn new(pattern: &str) -> Result<Self, Error> {
        Self::with_options(pattern, RegexOptions::NONE)
    }

    /// Compile a regex pattern with options.
    pub fn with_options(pattern: &str, options: RegexOptions) -> Result<Self, Error> {
        let green = parse(pattern, options)?;
        let tree: SyntaxNode<SyntaxKind> = SyntaxNode::new_root(green);
        let (arena, root) = cst_to_ir(&tree);

        Ok(Self {
            pattern: pattern.to_string(),
            arena,
            root,
        })
    }

    /// Returns the original pattern string.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Returns the root node ID.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Returns a reference to the IR arena.
    pub fn arena(&self) -> &RegexNodeArena<u64> {
        &self.arena
    }

    /// Pretty print the compiled pattern (for debugging).
    pub fn pretty_print(&self) -> String {
        let mut printer = PrettyPrinter::new(&self.arena);
        printer.print(self.root)
    }

    /// Check if the pattern matches anywhere in the input.
    ///
    /// Note: Full matching support is in development.
    pub fn is_match(&self, input: &str) -> bool {
        self.find(input).is_some()
    }

    /// Find the first match in the input.
    ///
    /// Note: Full matching support is in development.
    pub fn find(&self, input: &str) -> Option<Match> {
        self.find_iter(input).next()
    }

    /// Find all non-overlapping matches in the input.
    ///
    /// Note: Full matching support is in development.
    pub fn find_iter<'r, 't>(&'r self, input: &'t str) -> Matches<'r, 't> {
        Matches {
            regex: self,
            input,
            pos: 0,
        }
    }

    /// Internal: Run the derivative-based matching from a given position.
    ///
    /// Returns the end position of the longest match starting at `start`, or None.
    fn match_at(&self, input: &str, start: usize) -> Option<usize> {
        // We compute derivatives character by character and track nullable positions
        let bytes = input.as_bytes();
        let mut last_match: Option<usize> = None;

        // Check if pattern is nullable at start position
        let location = if start == 0 {
            LocationKind::Begin
        } else {
            LocationKind::Center
        };

        // We need a mutable builder for derivatives
        let green = parse(&self.pattern, RegexOptions::NONE).ok()?;
        let tree: SyntaxNode<SyntaxKind> = SyntaxNode::new_root(green);
        let (arena, root) = cst_to_ir_with_builder(&tree);
        let mut builder = RegexBuilder::with_arena(arena, BitSetSolver);
        let mut current = root;

        if is_nullable(&builder, location, current) {
            last_match = Some(start);
        }

        let mut pos = start;
        while pos < bytes.len() {
            let ch = bytes[pos];
            let minterm = char_to_minterm(ch);

            let location = if pos == 0 {
                LocationKind::Begin
            } else if pos == bytes.len() - 1 {
                LocationKind::End
            } else {
                LocationKind::Center
            };

            current = derivative(&mut builder, location, minterm, current);

            if current == NodeId::NOTHING {
                break;
            }

            pos += 1;

            // Check if nullable at current position
            let end_location = if pos == bytes.len() {
                LocationKind::End
            } else {
                LocationKind::Center
            };

            if is_nullable(&builder, end_location, current) {
                last_match = Some(pos);
            }
        }

        last_match
    }
}

/// Iterator over matches in an input string.
pub struct Matches<'r, 't> {
    regex: &'r Regex,
    input: &'t str,
    pos: usize,
}

impl<'r, 't> Iterator for Matches<'r, 't> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos <= self.input.len() {
            if let Some(end) = self.regex.match_at(self.input, self.pos) {
                let m = Match {
                    start: self.pos,
                    end,
                };

                // Advance past this match (at least one char to avoid infinite loop)
                self.pos = if end > self.pos { end } else { self.pos + 1 };

                return Some(m);
            }

            self.pos += 1;
        }

        None
    }
}

/// Convert CST to IR, returning a mutable arena for derivative computation.
fn cst_to_ir_with_builder(tree: &SyntaxNode<SyntaxKind>) -> (RegexNodeArena<u64>, NodeId) {
    cst_to_ir(tree)
}

/// Map a character to a minterm (simplified for ASCII).
///
/// This is a basic implementation that maps common character classes.
fn char_to_minterm(ch: u8) -> u64 {
    let mut bits = 0u64;

    // Bit 0: is digit
    if ch.is_ascii_digit() {
        bits |= 1 << 0;
    }

    // Bit 1: is word char (alphanumeric + underscore)
    if ch.is_ascii_alphanumeric() || ch == b'_' {
        bits |= 1 << 1;
    }

    // Bit 2: is whitespace
    if ch.is_ascii_whitespace() {
        bits |= 1 << 2;
    }

    // Bit 3: is lowercase
    if ch.is_ascii_lowercase() {
        bits |= 1 << 3;
    }

    // Bit 4: is uppercase
    if ch.is_ascii_uppercase() {
        bits |= 1 << 4;
    }

    // Bit 5: is alphabetic
    if ch.is_ascii_alphabetic() {
        bits |= 1 << 5;
    }

    // Bits 8-15: actual character value for exact matching
    bits |= (ch as u64) << 8;

    bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        let re = Regex::new(r"\d+").expect("valid pattern");
        assert!(!re.pattern().is_empty());
    }

    #[test]
    fn test_compile_conjunction() {
        let re = Regex::new(r".*foo.*&.*bar.*").expect("valid pattern");
        assert!(!re.pattern().is_empty());
    }

    #[test]
    fn test_compile_negation() {
        let re = Regex::new(r"~(.*error.*)").expect("valid pattern");
        assert!(!re.pattern().is_empty());
    }

    #[test]
    fn test_pretty_print() {
        let re = Regex::new("a|b").expect("valid pattern");
        let pp = re.pretty_print();
        // The pretty print should contain something
        assert!(!pp.is_empty());
    }
}
