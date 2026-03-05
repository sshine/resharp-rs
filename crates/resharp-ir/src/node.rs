//! Regex node types for the intermediate representation.

use crate::flags::NodeFlags;

/// A unique identifier for a regex node in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u32);

impl NodeId {
    /// The empty pattern (matches empty string only).
    pub const EPSILON: NodeId = NodeId(0);
    /// The pattern that never matches.
    pub const NOTHING: NodeId = NodeId(1);
    /// The pattern that matches any single character (`.`).
    pub const ANY: NodeId = NodeId(2);
    /// The universal pattern `_*` (matches anything including newlines).
    pub const TOP_STAR: NodeId = NodeId(3);
}

/// A regex node in the intermediate representation.
///
/// This is a normalized, simplified representation that's easier to work with
/// than the raw CST. Nodes are stored in an arena for efficient memory management.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexNode<T> {
    /// Concatenation of two patterns: `head` followed by `tail`.
    Concat { head: NodeId, tail: NodeId },

    /// Alternation (union) of patterns: matches if any child matches.
    Or { nodes: Vec<NodeId> },

    /// Conjunction (intersection) of patterns: matches if all children match.
    And { nodes: Vec<NodeId> },

    /// A single character or character class.
    Singleton(T),

    /// Repetition of a pattern.
    Loop {
        node: NodeId,
        low: u32,
        high: u32,
        lazy: bool,
    },

    /// Negation (complement) of a pattern.
    Not { inner: NodeId },

    /// Lookaround assertion.
    LookAround {
        inner: NodeId,
        look_back: bool,
        negative: bool,
    },

    /// Beginning of input/line anchor.
    Begin,

    /// End of input/line anchor.
    End,
}

/// Information about a node including computed flags and properties.
#[derive(Debug, Clone)]
pub struct NodeInfo<T> {
    /// The regex node itself.
    pub node: RegexNode<T>,
    /// Computed flags for this node.
    pub flags: NodeFlags,
    /// Minimum length this pattern can match.
    pub min_length: Option<u32>,
    /// Maximum length this pattern can match (None = unbounded).
    pub max_length: Option<u32>,
}

/// Arena for storing regex nodes with efficient allocation and lookup.
#[derive(Debug)]
pub struct RegexNodeArena<T> {
    nodes: Vec<NodeInfo<T>>,
}

impl<T: Clone + Default> Default for RegexNodeArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Default> RegexNodeArena<T> {
    /// Create a new arena with pre-allocated special nodes.
    pub fn new() -> Self {
        let mut arena = Self { nodes: Vec::new() };

        // Pre-allocate special nodes at fixed positions
        // EPSILON (id=0): matches empty string
        arena.nodes.push(NodeInfo {
            node: RegexNode::Loop {
                node: NodeId::ANY,
                low: 0,
                high: 0,
                lazy: false,
            },
            flags: NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE,
            min_length: Some(0),
            max_length: Some(0),
        });

        // NOTHING (id=1): never matches
        arena.nodes.push(NodeInfo {
            node: RegexNode::Not {
                inner: NodeId::TOP_STAR,
            },
            flags: NodeFlags::empty(),
            min_length: None,
            max_length: None,
        });

        // ANY (id=2): matches any single character
        arena.nodes.push(NodeInfo {
            node: RegexNode::Singleton(T::default()),
            flags: NodeFlags::empty(),
            min_length: Some(1),
            max_length: Some(1),
        });

        // TOP_STAR (id=3): matches anything (universal pattern)
        arena.nodes.push(NodeInfo {
            node: RegexNode::Loop {
                node: NodeId::ANY,
                low: 0,
                high: u32::MAX,
                lazy: false,
            },
            flags: NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE,
            min_length: Some(0),
            max_length: None,
        });

        arena
    }

    /// Get the node info for a given node ID.
    pub fn get(&self, id: NodeId) -> Option<&NodeInfo<T>> {
        self.nodes.get(id.0 as usize)
    }

    /// Get the regex node for a given node ID.
    pub fn node(&self, id: NodeId) -> Option<&RegexNode<T>> {
        self.get(id).map(|info| &info.node)
    }

    /// Get the flags for a given node ID.
    pub fn flags(&self, id: NodeId) -> NodeFlags {
        self.get(id)
            .map(|info| info.flags)
            .unwrap_or(NodeFlags::empty())
    }

    /// Allocate a new node in the arena.
    pub fn alloc(&mut self, node: RegexNode<T>) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        let (flags, min_length, max_length) = self.compute_info(&node);
        self.nodes.push(NodeInfo {
            node,
            flags,
            min_length,
            max_length,
        });
        id
    }

    /// Compute flags and length info for a node.
    fn compute_info(&self, node: &RegexNode<T>) -> (NodeFlags, Option<u32>, Option<u32>) {
        match node {
            RegexNode::Concat { head, tail } => {
                let h_flags = self.flags(*head);
                let t_flags = self.flags(*tail);
                let h_info = self.get(*head);
                let t_info = self.get(*tail);

                let flags = self.infer_concat_flags(h_flags, t_flags, *head, *tail);

                let min_length = match (
                    h_info.and_then(|i| i.min_length),
                    t_info.and_then(|i| i.min_length),
                ) {
                    (Some(h), Some(t)) => Some(h + t),
                    _ => None,
                };

                let max_length = match (
                    h_info.and_then(|i| i.max_length),
                    t_info.and_then(|i| i.max_length),
                ) {
                    (Some(h), Some(t)) => h.checked_add(t),
                    _ => None,
                };

                (flags, min_length, max_length)
            }

            RegexNode::Or { nodes } => {
                let flags = self.infer_or_flags(nodes);

                let min_length = nodes
                    .iter()
                    .filter_map(|id| self.get(*id).and_then(|i| i.min_length))
                    .min();

                let max_length = if nodes
                    .iter()
                    .all(|id| self.get(*id).and_then(|i| i.max_length).is_some())
                {
                    nodes
                        .iter()
                        .filter_map(|id| self.get(*id).and_then(|i| i.max_length))
                        .max()
                } else {
                    None
                };

                (flags, min_length, max_length)
            }

            RegexNode::And { nodes } => {
                let flags = self.infer_and_flags(nodes);

                let min_length = nodes
                    .iter()
                    .filter_map(|id| self.get(*id).and_then(|i| i.min_length))
                    .max();

                let max_length = if nodes
                    .iter()
                    .all(|id| self.get(*id).and_then(|i| i.max_length).is_some())
                {
                    nodes
                        .iter()
                        .filter_map(|id| self.get(*id).and_then(|i| i.max_length))
                        .min()
                } else {
                    None
                };

                (flags, min_length, max_length)
            }

            RegexNode::Singleton(_) => (NodeFlags::empty(), Some(1), Some(1)),

            RegexNode::Loop {
                node, low, high, ..
            } => {
                let inner_flags = self.flags(*node);

                let nullable_flags = if *low == 0 {
                    NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE
                } else {
                    inner_flags & (NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE)
                };

                // Propagate other flags from inner
                let other_flags =
                    inner_flags & (NodeFlags::CONTAINS_LOOKAROUND | NodeFlags::DEPENDS_ON_ANCHOR);

                let max_length = if *high == u32::MAX { None } else { Some(*high) };

                (nullable_flags | other_flags, Some(*low), max_length)
            }

            RegexNode::Not { inner } => {
                let inner_flags = self.flags(*inner);
                let flags = self.infer_not_flags(inner_flags);
                (flags, None, None)
            }

            RegexNode::LookAround {
                inner,
                look_back,
                negative,
            } => {
                let inner_flags = self.flags(*inner);
                let flags = self.infer_lookaround_flags(inner_flags, *look_back, *negative);
                (flags, Some(0), Some(0))
            }

            RegexNode::Begin | RegexNode::End => (
                // Anchors CAN be nullable (at the right position) but are NOT always nullable
                NodeFlags::CAN_BE_NULLABLE | NodeFlags::DEPENDS_ON_ANCHOR,
                Some(0),
                Some(0),
            ),
        }
    }

    fn infer_concat_flags(
        &self,
        h_flags: NodeFlags,
        t_flags: NodeFlags,
        _head: NodeId,
        _tail: NodeId,
    ) -> NodeFlags {
        let contains_lookaround = (h_flags | t_flags) & NodeFlags::CONTAINS_LOOKAROUND;

        let nullable =
            (h_flags & t_flags) & (NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE);

        let depends_on_anchor = if h_flags.contains(NodeFlags::DEPENDS_ON_ANCHOR)
            || (h_flags.contains(NodeFlags::CAN_BE_NULLABLE)
                && t_flags.contains(NodeFlags::DEPENDS_ON_ANCHOR))
        {
            NodeFlags::DEPENDS_ON_ANCHOR
        } else {
            NodeFlags::empty()
        };

        let suffix_lookahead = if t_flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD) {
            NodeFlags::HAS_SUFFIX_LOOKAHEAD
        } else {
            NodeFlags::empty()
        };

        let prefix_lookbehind = if h_flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND) {
            NodeFlags::HAS_PREFIX_LOOKBEHIND
        } else {
            NodeFlags::empty()
        };

        contains_lookaround | nullable | depends_on_anchor | suffix_lookahead | prefix_lookbehind
    }

    fn infer_or_flags(&self, nodes: &[NodeId]) -> NodeFlags {
        let all_flags = NodeFlags::CAN_BE_NULLABLE
            | NodeFlags::IS_ALWAYS_NULLABLE
            | NodeFlags::CONTAINS_LOOKAROUND
            | NodeFlags::DEPENDS_ON_ANCHOR
            | NodeFlags::HAS_SUFFIX_LOOKAHEAD
            | NodeFlags::HAS_PREFIX_LOOKBEHIND;

        nodes.iter().fold(NodeFlags::empty(), |acc, id| {
            (self.flags(*id) & all_flags) | acc
        })
    }

    fn infer_and_flags(&self, nodes: &[NodeId]) -> NodeFlags {
        let init = NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE;

        nodes.iter().fold(init, |acc, id| {
            let flags = self.flags(*id);

            // Flags that propagate with OR (present if any child has them)
            let or_flags = (acc | flags)
                & (NodeFlags::CONTAINS_LOOKAROUND
                    | NodeFlags::DEPENDS_ON_ANCHOR
                    | NodeFlags::HAS_SUFFIX_LOOKAHEAD
                    | NodeFlags::HAS_PREFIX_LOOKBEHIND);

            // Flags that propagate with AND (present only if all children have them)
            let and_flags =
                (acc & flags) & (NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE);

            or_flags | and_flags
        })
    }

    fn infer_not_flags(&self, inner_flags: NodeFlags) -> NodeFlags {
        let nullable = match (
            inner_flags.contains(NodeFlags::CAN_BE_NULLABLE),
            inner_flags.contains(NodeFlags::IS_ALWAYS_NULLABLE),
        ) {
            (false, _) => NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE,
            (true, true) => NodeFlags::empty(),
            (true, false) => NodeFlags::CAN_BE_NULLABLE,
        };

        let other = inner_flags & (NodeFlags::CONTAINS_LOOKAROUND | NodeFlags::DEPENDS_ON_ANCHOR);

        nullable | other
    }

    fn infer_lookaround_flags(
        &self,
        inner_flags: NodeFlags,
        look_back: bool,
        _negative: bool,
    ) -> NodeFlags {
        // Lookarounds are zero-width assertions, so they're always nullable
        let nullable = NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE;
        let anchor = inner_flags & NodeFlags::DEPENDS_ON_ANCHOR;

        let lookaround = if look_back {
            NodeFlags::CONTAINS_LOOKAROUND | NodeFlags::HAS_PREFIX_LOOKBEHIND
        } else {
            NodeFlags::CONTAINS_LOOKAROUND | NodeFlags::HAS_SUFFIX_LOOKAHEAD
        };

        nullable | anchor | lookaround
    }

    /// Get the fixed length of a pattern, if it has one.
    pub fn get_fixed_length(&self, id: NodeId) -> Option<u32> {
        let info = self.get(id)?;
        match (info.min_length, info.max_length) {
            (Some(min), Some(max)) if min == max => Some(min),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_nodes() {
        let arena: RegexNodeArena<u64> = RegexNodeArena::new();

        // EPSILON should be nullable
        let eps_flags = arena.flags(NodeId::EPSILON);
        assert!(eps_flags.contains(NodeFlags::CAN_BE_NULLABLE));
        assert!(eps_flags.contains(NodeFlags::IS_ALWAYS_NULLABLE));

        // TOP_STAR should be nullable
        let top_flags = arena.flags(NodeId::TOP_STAR);
        assert!(top_flags.contains(NodeFlags::CAN_BE_NULLABLE));
    }

    #[test]
    fn test_fixed_length() {
        let arena: RegexNodeArena<u64> = RegexNodeArena::new();

        // ANY has fixed length 1
        assert_eq!(arena.get_fixed_length(NodeId::ANY), Some(1));

        // EPSILON has fixed length 0
        assert_eq!(arena.get_fixed_length(NodeId::EPSILON), Some(0));

        // TOP_STAR has no fixed length
        assert_eq!(arena.get_fixed_length(NodeId::TOP_STAR), None);
    }
}
