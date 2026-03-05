//! Syntax definitions for the RE# extended regex parser.
//!
//! This crate provides the [`SyntaxKind`] enum for concrete syntax tree nodes
//! and [`RegexOptions`] bitflags for parsing options.

mod kind;

pub use kind::SyntaxKind;

use bitflags::bitflags;

bitflags! {
    /// Options that modify regex parsing and matching behavior.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RegexOptions: u32 {
        /// No options set.
        const NONE = 0x0000;
        /// Case-insensitive matching (i flag).
        const IGNORE_CASE = 0x0001;
        /// Multi-line mode: ^ and $ match line boundaries (m flag).
        const MULTILINE = 0x0002;
        /// Only named captures are valid (n flag).
        const EXPLICIT_CAPTURE = 0x0004;
        /// Single-line mode: . matches newlines (s flag).
        const SINGLELINE = 0x0010;
        /// Ignore whitespace and allow comments (x flag).
        const IGNORE_WHITESPACE = 0x0020;
        /// Right-to-left matching mode.
        const RIGHT_TO_LEFT = 0x0040;
        /// ECMAScript-compatible behavior.
        const ECMA_SCRIPT = 0x0100;
        /// Culture-invariant matching.
        const CULTURE_INVARIANT = 0x0200;
        /// Non-backtracking mode.
        const NON_BACKTRACKING = 0x0400;
        /// RE# extension: node is negated.
        const NEGATED = 0x0800;
    }
}

/// Type alias for cstree syntax nodes using our SyntaxKind.
pub type SyntaxNode = cstree::syntax::SyntaxNode<SyntaxKind>;

/// Type alias for cstree syntax tokens using our SyntaxKind.
pub type SyntaxToken = cstree::syntax::SyntaxToken<SyntaxKind>;

/// Type alias for cstree green nodes.
pub type GreenNode = cstree::green::GreenNode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_kind_roundtrip() {
        use cstree::Syntax;

        for kind in [
            SyntaxKind::Root,
            SyntaxKind::One,
            SyntaxKind::Conjunction,
            SyntaxKind::Alternate,
        ] {
            let raw = kind.into_raw();
            let back = SyntaxKind::from_raw(raw);
            assert_eq!(kind, back);
        }
    }

    #[test]
    fn regex_options_flags() {
        let opts = RegexOptions::IGNORE_CASE | RegexOptions::MULTILINE;
        assert!(opts.contains(RegexOptions::IGNORE_CASE));
        assert!(opts.contains(RegexOptions::MULTILINE));
        assert!(!opts.contains(RegexOptions::SINGLELINE));
    }
}
