//! Parser for RE# extended regex patterns.
//!
//! This crate provides a parser that produces a concrete syntax tree (CST)
//! using the `cstree` library. The parser uses `nom` combinators internally.
//!
//! # RE# Extensions
//!
//! In addition to standard regex syntax, this parser supports:
//!
//! - **Conjunction** (`&`): Intersection of patterns, e.g., `a.*&.*b`
//! - **Negation** (`~`): Pattern negation, e.g., `~(abc)`
//! - **Universal wildcard** (`_`): Matches any character including newlines
//!
//! # Example
//!
//! ```
//! use resharp_parser::{parse, RegexOptions};
//! use resharp_syntax::SyntaxKind;
//! use cstree::syntax::SyntaxNode;
//!
//! let green = parse("a|b", RegexOptions::NONE).unwrap();
//! let tree: SyntaxNode<SyntaxKind> = SyntaxNode::new_root(green);
//! assert_eq!(tree.kind(), SyntaxKind::Root);
//! ```

mod convert;
mod error;
mod parser;

pub use convert::{cst_to_ir, CstToIr};
pub use error::ParseError;
pub use parser::parse;

// Re-export commonly used types from resharp_syntax for convenience
pub use resharp_syntax::RegexOptions;

// Re-export IR types for convenience
pub use resharp_ir::{NodeFlags, NodeId, RegexNode, RegexNodeArena};
