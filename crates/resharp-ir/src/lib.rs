//! Intermediate representation for RE# regex patterns.
//!
//! This crate provides a simplified, normalized representation of regex patterns
//! that is suitable for analysis, optimization, and matching.

mod builder;
mod derivative;
mod flags;
mod node;
mod printer;
mod solver;

pub use builder::RegexBuilder;
pub use derivative::{derivative, is_nullable, LocationKind};
pub use flags::NodeFlags;
pub use node::{NodeId, NodeInfo, RegexNode, RegexNodeArena};
pub use printer::{CharSetDisplay, PrettyPrinter};
pub use solver::{minterms_log, BitSetSolver, CharSetSolver};
