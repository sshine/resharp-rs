//! Parse error types.

use thiserror::Error;

/// Errors that can occur during regex pattern parsing.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Unexpected end of input.
    #[error("unexpected end of input at position {pos}")]
    UnexpectedEof { pos: usize },

    /// Unexpected character encountered.
    #[error("unexpected character '{ch}' at position {pos}")]
    UnexpectedChar { ch: char, pos: usize },

    /// Unclosed group.
    #[error("unclosed group starting at position {pos}")]
    UnclosedGroup { pos: usize },

    /// Unclosed character class.
    #[error("unclosed character class starting at position {pos}")]
    UnclosedCharClass { pos: usize },

    /// Invalid escape sequence.
    #[error("invalid escape sequence at position {pos}")]
    InvalidEscape { pos: usize },

    /// Invalid quantifier.
    #[error("invalid quantifier at position {pos}")]
    InvalidQuantifier { pos: usize },

    /// Quantifier min greater than max.
    #[error("quantifier min ({min}) greater than max ({max}) at position {pos}")]
    QuantifierMinGreaterThanMax { min: u32, max: u32, pos: usize },

    /// Nothing to repeat.
    #[error("nothing to repeat at position {pos}")]
    NothingToRepeat { pos: usize },

    /// Invalid group name.
    #[error("invalid group name at position {pos}")]
    InvalidGroupName { pos: usize },

    /// Unknown group type.
    #[error("unknown group type at position {pos}")]
    UnknownGroupType { pos: usize },
}
