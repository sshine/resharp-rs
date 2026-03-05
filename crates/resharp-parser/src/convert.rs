//! Convert CST to IR.

use cstree::syntax::SyntaxNode;
use resharp_ir::{NodeId, RegexNode, RegexNodeArena};
use resharp_syntax::SyntaxKind;

/// Converter from CST to IR.
pub struct CstToIr<T> {
    arena: RegexNodeArena<T>,
}

impl<T: Clone + Default> Default for CstToIr<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Default> CstToIr<T> {
    /// Create a new converter.
    pub fn new() -> Self {
        Self {
            arena: RegexNodeArena::new(),
        }
    }

    /// Convert a CST root node to IR, returning the arena and root node ID.
    pub fn convert(mut self, root: &SyntaxNode<SyntaxKind>) -> (RegexNodeArena<T>, NodeId) {
        let root_id = self.convert_node(root);
        (self.arena, root_id)
    }

    /// Get a reference to the arena.
    pub fn arena(&self) -> &RegexNodeArena<T> {
        &self.arena
    }

    fn convert_node(&mut self, node: &SyntaxNode<SyntaxKind>) -> NodeId {
        match node.kind() {
            SyntaxKind::Root => {
                // Root contains a single Capture
                if let Some(child) = node.children().next() {
                    self.convert_node(child)
                } else {
                    NodeId::EPSILON
                }
            }

            SyntaxKind::Capture | SyntaxKind::Group => {
                // Groups contain an Alternate
                if let Some(child) = node.children().next() {
                    self.convert_node(child)
                } else {
                    NodeId::EPSILON
                }
            }

            SyntaxKind::Alternate => {
                let children: Vec<_> = node.children().collect();
                if children.len() == 1 {
                    self.convert_node(children[0])
                } else {
                    let nodes: Vec<NodeId> = children
                        .iter()
                        .map(|child| self.convert_node(child))
                        .collect();
                    self.arena.alloc(RegexNode::Or { nodes })
                }
            }

            SyntaxKind::Conjunction => {
                let children: Vec<_> = node.children().collect();
                if children.len() == 1 {
                    self.convert_node(children[0])
                } else {
                    let nodes: Vec<NodeId> = children
                        .iter()
                        .map(|child| self.convert_node(child))
                        .collect();
                    self.arena.alloc(RegexNode::And { nodes })
                }
            }

            SyntaxKind::Concatenate => {
                let children: Vec<_> = node.children().collect();
                if children.is_empty() {
                    NodeId::EPSILON
                } else if children.len() == 1 {
                    self.convert_node(children[0])
                } else {
                    // Build a right-associative concatenation
                    let mut result = self.convert_node(children[children.len() - 1]);
                    for i in (0..children.len() - 1).rev() {
                        let head = self.convert_node(children[i]);
                        result = self.arena.alloc(RegexNode::Concat { head, tail: result });
                    }
                    result
                }
            }

            SyntaxKind::One | SyntaxKind::Multi => {
                // Single character or multi-character literal
                // For now, just create a placeholder singleton
                self.arena.alloc(RegexNode::Singleton(T::default()))
            }

            SyntaxKind::Notone => {
                // Negated single character (like .)
                self.arena.alloc(RegexNode::Singleton(T::default()))
            }

            SyntaxKind::Set | SyntaxKind::CharClass | SyntaxKind::CharClassNegated => {
                // Character class
                self.arena.alloc(RegexNode::Singleton(T::default()))
            }

            SyntaxKind::Oneloop | SyntaxKind::Notoneloop | SyntaxKind::Setloop => {
                // Greedy loop (e.g., a*, [^a]*, [a-z]*)
                let inner = if let Some(child) = node.children().next() {
                    self.convert_node(child)
                } else {
                    self.arena.alloc(RegexNode::Singleton(T::default()))
                };
                self.arena.alloc(RegexNode::Loop {
                    node: inner,
                    low: 0,
                    high: u32::MAX,
                    lazy: false,
                })
            }

            SyntaxKind::Onelazy | SyntaxKind::Notonelazy | SyntaxKind::Setlazy => {
                // Lazy loop
                let inner = if let Some(child) = node.children().next() {
                    self.convert_node(child)
                } else {
                    self.arena.alloc(RegexNode::Singleton(T::default()))
                };
                self.arena.alloc(RegexNode::Loop {
                    node: inner,
                    low: 0,
                    high: u32::MAX,
                    lazy: true,
                })
            }

            SyntaxKind::Loop => {
                // General loop with bounds
                let inner = if let Some(child) = node.children().next() {
                    self.convert_node(child)
                } else {
                    NodeId::EPSILON
                };
                // TODO: extract actual bounds from the CST
                self.arena.alloc(RegexNode::Loop {
                    node: inner,
                    low: 0,
                    high: u32::MAX,
                    lazy: false,
                })
            }

            SyntaxKind::PositiveLookahead => {
                let inner = node
                    .children()
                    .next()
                    .map(|child| self.convert_node(child))
                    .unwrap_or(NodeId::EPSILON);

                self.arena.alloc(RegexNode::LookAround {
                    inner,
                    look_back: false,
                    negative: false,
                })
            }

            SyntaxKind::PositiveLookbehind => {
                let inner = node
                    .children()
                    .next()
                    .map(|child| self.convert_node(child))
                    .unwrap_or(NodeId::EPSILON);

                self.arena.alloc(RegexNode::LookAround {
                    inner,
                    look_back: true,
                    negative: false,
                })
            }

            SyntaxKind::NegativeLookahead => {
                let inner = node
                    .children()
                    .next()
                    .map(|child| self.convert_node(child))
                    .unwrap_or(NodeId::EPSILON);

                self.arena.alloc(RegexNode::LookAround {
                    inner,
                    look_back: false,
                    negative: true,
                })
            }

            SyntaxKind::NegativeLookbehind => {
                let inner = node
                    .children()
                    .next()
                    .map(|child| self.convert_node(child))
                    .unwrap_or(NodeId::EPSILON);

                self.arena.alloc(RegexNode::LookAround {
                    inner,
                    look_back: true,
                    negative: true,
                })
            }

            SyntaxKind::Atomic => {
                // Atomic group - treat as regular group for now
                if let Some(child) = node.children().next() {
                    self.convert_node(child)
                } else {
                    NodeId::EPSILON
                }
            }

            SyntaxKind::Beginning | SyntaxKind::Bol => self.arena.alloc(RegexNode::Begin),

            SyntaxKind::End | SyntaxKind::EndZ | SyntaxKind::Eol => {
                self.arena.alloc(RegexNode::End)
            }

            SyntaxKind::Empty => NodeId::EPSILON,

            SyntaxKind::Nothing => NodeId::NOTHING,

            SyntaxKind::Negation => {
                // RE# negation: ~expr
                let inner = node
                    .children()
                    .next()
                    .map(|child| self.convert_node(child))
                    .unwrap_or(NodeId::EPSILON);
                self.arena.alloc(RegexNode::Not { inner })
            }

            // Default: recurse into children
            _ => {
                let children: Vec<_> = node.children().collect();
                if children.is_empty() {
                    NodeId::EPSILON
                } else if children.len() == 1 {
                    self.convert_node(children[0])
                } else {
                    // Treat as concatenation
                    let mut result = self.convert_node(children[children.len() - 1]);
                    for i in (0..children.len() - 1).rev() {
                        let head = self.convert_node(children[i]);
                        result = self.arena.alloc(RegexNode::Concat { head, tail: result });
                    }
                    result
                }
            }
        }
    }
}

/// Convert a parsed regex pattern to IR.
pub fn cst_to_ir<T: Clone + Default>(cst: &SyntaxNode<SyntaxKind>) -> (RegexNodeArena<T>, NodeId) {
    CstToIr::new().convert(cst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, NodeFlags, RegexOptions};

    fn parse_to_ir(pattern: &str) -> (RegexNodeArena<u64>, NodeId) {
        let options = RegexOptions::EXPLICIT_CAPTURE | RegexOptions::NON_BACKTRACKING;
        let green = parse(pattern, options).expect("parse should succeed");
        let tree: SyntaxNode<SyntaxKind> = SyntaxNode::new_root(green);
        cst_to_ir(&tree)
    }

    #[test]
    fn test_simple_literal() {
        let (arena, root) = parse_to_ir("abc");
        let flags = arena.flags(root);
        assert!(!flags.contains(NodeFlags::CAN_BE_NULLABLE));
    }

    #[test]
    fn test_alternation() {
        let (arena, root) = parse_to_ir("a|b");
        // Should be an Or node
        assert!(matches!(arena.node(root), Some(RegexNode::Or { .. })));
    }

    #[test]
    fn test_conjunction() {
        let (arena, root) = parse_to_ir("a&b");
        // Should be an And node
        assert!(matches!(arena.node(root), Some(RegexNode::And { .. })));
    }

    #[test]
    fn test_lookahead() {
        let (arena, root) = parse_to_ir("a(?=b)");
        let flags = arena.flags(root);
        assert!(flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD));
    }

    #[test]
    fn test_anchor() {
        let (arena, root) = parse_to_ir("^a$");
        let flags = arena.flags(root);
        assert!(flags.contains(NodeFlags::DEPENDS_ON_ANCHOR));
    }
}
