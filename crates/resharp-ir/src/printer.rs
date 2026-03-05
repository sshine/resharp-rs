//! Pretty printer for regex nodes.

use crate::node::{NodeId, RegexNode, RegexNodeArena};
use std::fmt::Write;

/// Pretty printer for regex nodes.
pub struct PrettyPrinter<'a, T> {
    arena: &'a RegexNodeArena<T>,
    buffer: String,
}

impl<'a, T: CharSetDisplay + Clone + Default> PrettyPrinter<'a, T> {
    /// Create a new pretty printer for the given arena.
    pub fn new(arena: &'a RegexNodeArena<T>) -> Self {
        Self {
            arena,
            buffer: String::new(),
        }
    }

    /// Print a node to a string.
    pub fn print(&mut self, id: NodeId) -> String {
        self.buffer.clear();
        self.print_node(id, false);
        std::mem::take(&mut self.buffer)
    }

    fn print_node(&mut self, id: NodeId, in_concat: bool) {
        // Handle special nodes
        match id {
            NodeId::EPSILON => {
                // Empty pattern - print nothing or ()
                return;
            }
            NodeId::NOTHING => {
                let _ = write!(self.buffer, "(?!)");
                return;
            }
            NodeId::TOP_STAR => {
                let _ = write!(self.buffer, "_*");
                return;
            }
            _ => {}
        }

        let Some(info) = self.arena.get(id) else {
            return;
        };

        match &info.node {
            RegexNode::Concat { head, tail } => {
                self.print_node(*head, true);
                self.print_node(*tail, true);
            }

            RegexNode::Or { nodes } => {
                if nodes.is_empty() {
                    let _ = write!(self.buffer, "(?!)");
                    return;
                }

                let needs_parens = in_concat;
                if needs_parens {
                    self.buffer.push('(');
                }

                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        self.buffer.push('|');
                    }
                    self.print_node(*node, false);
                }

                if needs_parens {
                    self.buffer.push(')');
                }
            }

            RegexNode::And { nodes } => {
                if nodes.is_empty() {
                    let _ = write!(self.buffer, "_*");
                    return;
                }

                let needs_parens = in_concat || nodes.len() > 1;
                if needs_parens {
                    self.buffer.push('(');
                }

                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        self.buffer.push('&');
                    }
                    self.print_node(*node, false);
                }

                if needs_parens {
                    self.buffer.push(')');
                }
            }

            RegexNode::Singleton(charset) => {
                charset.fmt_charset(&mut self.buffer);
            }

            RegexNode::Loop {
                node,
                low,
                high,
                lazy,
            } => {
                let needs_parens = matches!(
                    self.arena.node(*node),
                    Some(RegexNode::Concat { .. } | RegexNode::Or { .. } | RegexNode::And { .. })
                );

                if needs_parens {
                    self.buffer.push('(');
                }
                self.print_node(*node, false);
                if needs_parens {
                    self.buffer.push(')');
                }

                match (*low, *high) {
                    (0, u32::MAX) => self.buffer.push('*'),
                    (1, u32::MAX) => self.buffer.push('+'),
                    (0, 1) => self.buffer.push('?'),
                    (n, m) if n == m => {
                        let _ = write!(self.buffer, "{{{n}}}");
                    }
                    (n, u32::MAX) => {
                        let _ = write!(self.buffer, "{{{n},}}");
                    }
                    (n, m) => {
                        let _ = write!(self.buffer, "{{{n},{m}}}");
                    }
                }

                if *lazy {
                    self.buffer.push('?');
                }
            }

            RegexNode::Not { inner } => {
                self.buffer.push('~');
                let needs_parens =
                    !matches!(self.arena.node(*inner), Some(RegexNode::Singleton(_)));
                if needs_parens {
                    self.buffer.push('(');
                }
                self.print_node(*inner, false);
                if needs_parens {
                    self.buffer.push(')');
                }
            }

            RegexNode::LookAround {
                inner,
                look_back,
                negative,
            } => {
                match (*look_back, *negative) {
                    (false, false) => self.buffer.push_str("(?="),
                    (false, true) => self.buffer.push_str("(?!"),
                    (true, false) => self.buffer.push_str("(?<="),
                    (true, true) => self.buffer.push_str("(?<!"),
                }
                self.print_node(*inner, false);
                self.buffer.push(')');
            }

            RegexNode::Begin => {
                self.buffer.push('^');
            }

            RegexNode::End => {
                self.buffer.push('$');
            }
        }
    }
}

/// Trait for displaying character sets in regex syntax.
pub trait CharSetDisplay {
    /// Format the character set to the buffer.
    fn fmt_charset(&self, buffer: &mut String);
}

// Default implementation for u64 (placeholder)
impl CharSetDisplay for u64 {
    fn fmt_charset(&self, buffer: &mut String) {
        // For now, just print a placeholder
        let _ = write!(buffer, "φ");
    }
}

// Default implementation for char
impl CharSetDisplay for char {
    fn fmt_charset(&self, buffer: &mut String) {
        match self {
            '\\' | '|' | '&' | '~' | '(' | ')' | '[' | ']' | '{' | '}' | '*' | '+' | '?' | '.'
            | '^' | '$' => {
                buffer.push('\\');
                buffer.push(*self);
            }
            '\n' => buffer.push_str("\\n"),
            '\r' => buffer.push_str("\\r"),
            '\t' => buffer.push_str("\\t"),
            c => buffer.push(*c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_special_nodes() {
        let arena: RegexNodeArena<u64> = RegexNodeArena::new();
        let mut printer = PrettyPrinter::new(&arena);

        assert_eq!(printer.print(NodeId::TOP_STAR), "_*");
        assert_eq!(printer.print(NodeId::NOTHING), "(?!)");
    }
}
