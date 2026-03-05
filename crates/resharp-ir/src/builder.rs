//! Regex pattern builder with subsumption-based normalization.
//!
//! This module provides a builder that constructs normalized regex patterns
//! by applying algebraic simplifications and subsumption rules.

use crate::flags::NodeFlags;
use crate::node::{NodeId, NodeInfo, RegexNode, RegexNodeArena};
use crate::solver::CharSetSolver;

/// Builder for constructing normalized regex patterns.
///
/// The builder applies simplification rules during pattern construction,
/// including:
/// - Identity rules (A | ⊥ = A, A & ⊤ = A)
/// - Idempotency (A | A = A, A & A = A)
/// - Absorption (A | ⊤ = ⊤, A & ⊥ = ⊥)
/// - Subsumption (a* | .* = .*, .* & .*s = .*s)
pub struct RegexBuilder<S: CharSetSolver> {
    pub(crate) arena: RegexNodeArena<S::CharSet>,
    pub(crate) solver: S,
}

impl<S: CharSetSolver> RegexBuilder<S> {
    /// Create a new builder with the given solver.
    pub fn new(solver: S) -> Self {
        Self {
            arena: RegexNodeArena::new(),
            solver,
        }
    }

    /// Create a builder from an existing arena.
    pub fn with_arena(arena: RegexNodeArena<S::CharSet>, solver: S) -> Self {
        Self { arena, solver }
    }

    /// Get a reference to the arena.
    pub fn arena(&self) -> &RegexNodeArena<S::CharSet> {
        &self.arena
    }

    /// Consume the builder and return the arena.
    pub fn into_arena(self) -> RegexNodeArena<S::CharSet> {
        self.arena
    }

    /// Create a singleton node matching the given character set.
    pub fn singleton(&mut self, charset: S::CharSet) -> NodeId {
        if self.solver.is_empty(&charset) {
            return NodeId::NOTHING;
        }
        // Note: We don't return NodeId::ANY for full charsets because
        // the pre-allocated ANY node doesn't have the correct charset value.
        // This could be optimized later with proper arena initialization.
        self.arena.alloc(RegexNode::Singleton(charset))
    }

    /// Create a concatenation of two patterns.
    pub fn concat(&mut self, head: NodeId, tail: NodeId) -> NodeId {
        self.mk_concat(head, tail)
    }

    /// Create a normalized concatenation.
    fn mk_concat(&mut self, head: NodeId, tail: NodeId) -> NodeId {
        // Identity rules
        if head == NodeId::EPSILON {
            return tail;
        }
        if tail == NodeId::EPSILON {
            return head;
        }

        // Absorption
        if head == NodeId::NOTHING || tail == NodeId::NOTHING {
            return NodeId::NOTHING;
        }

        // TODO: More simplification rules

        self.arena.alloc(RegexNode::Concat { head, tail })
    }

    /// Create an alternation (union) of two patterns.
    pub fn or(&mut self, node1: NodeId, node2: NodeId) -> NodeId {
        self.mk_or2(node1, node2)
    }

    /// Create a normalized alternation of two patterns.
    fn mk_or2(&mut self, node1: NodeId, node2: NodeId) -> NodeId {
        // Idempotency
        if node1 == node2 {
            return node1;
        }

        // Identity rules
        if node1 == NodeId::NOTHING {
            return node2;
        }
        if node2 == NodeId::NOTHING {
            return node1;
        }

        // Absorption
        if node1 == NodeId::TOP_STAR || node2 == NodeId::TOP_STAR {
            return NodeId::TOP_STAR;
        }

        // Check for complementary patterns: A | ~A = ⊤
        if let Some(RegexNode::Not { inner }) = self.arena.node(node1) {
            if *inner == node2 {
                return NodeId::TOP_STAR;
            }
        }
        if let Some(RegexNode::Not { inner }) = self.arena.node(node2) {
            if *inner == node1 {
                return NodeId::TOP_STAR;
            }
        }

        // Merge singletons: [a] | [b] = [ab]
        if let (Some(RegexNode::Singleton(s1)), Some(RegexNode::Singleton(s2))) =
            (self.arena.node(node1), self.arena.node(node2))
        {
            let merged = self.solver.or(s1, s2);
            return self.singleton(merged);
        }

        // Epsilon creates optional: ε | A = A?
        // But if A is already nullable (can match empty), just return A
        if node1 == NodeId::EPSILON {
            let flags = self.flags(node2);
            if flags.contains(NodeFlags::CAN_BE_NULLABLE) {
                return node2;
            }
            return self.mk_loop(node2, 0, 1, false);
        }
        if node2 == NodeId::EPSILON {
            let flags = self.flags(node1);
            if flags.contains(NodeFlags::CAN_BE_NULLABLE) {
                return node1;
            }
            return self.mk_loop(node1, 0, 1, false);
        }

        // Subsumption for star loops
        if let (Some(pred1), Some(pred2)) = (self.get_star_pred(node1), self.get_star_pred(node2)) {
            if self.solver.contains(&pred1, &pred2) {
                return node1; // node1 subsumes node2
            }
            if self.solver.contains(&pred2, &pred1) {
                return node2; // node2 subsumes node1
            }
        }

        // Loop merging: a{m,n} | a{p,q} = a{min(m,p), max(n,q)}
        if let (
            Some(RegexNode::Loop {
                node: inner1,
                low: low1,
                high: high1,
                ..
            }),
            Some(RegexNode::Loop {
                node: inner2,
                low: low2,
                high: high2,
                ..
            }),
        ) = (self.arena.node(node1), self.arena.node(node2))
        {
            if inner1 == inner2 {
                let new_low = (*low1).min(*low2);
                let new_high = (*high1).max(*high2);
                return self.mk_loop(*inner1, new_low, new_high, false);
            }
        }

        // A | A{n,m} = A{1,max(1,m)} (when n >= 2)
        // A | A{n,m} = A{min(n,1),m} (general case)
        if let Some(RegexNode::Loop {
            node: inner,
            low,
            high,
            ..
        }) = self.arena.node(node2)
        {
            if *inner == node1 {
                let new_low = (*low).min(1);
                return self.mk_loop(node1, new_low, *high, false);
            }
        }
        if let Some(RegexNode::Loop {
            node: inner,
            low,
            high,
            ..
        }) = self.arena.node(node1)
        {
            if *inner == node2 {
                let new_low = (*low).min(1);
                return self.mk_loop(node2, new_low, *high, false);
            }
        }

        // Create Or node with sorted children
        let mut nodes = vec![node1, node2];
        nodes.sort();
        self.arena.alloc(RegexNode::Or { nodes })
    }

    /// Create a conjunction (intersection) of two patterns.
    pub fn and(&mut self, node1: NodeId, node2: NodeId) -> NodeId {
        self.mk_and2(node1, node2)
    }

    /// Create a normalized conjunction of two patterns.
    fn mk_and2(&mut self, node1: NodeId, node2: NodeId) -> NodeId {
        // Idempotency
        if node1 == node2 {
            return node1;
        }

        // Identity rules
        if node1 == NodeId::TOP_STAR {
            return node2;
        }
        if node2 == NodeId::TOP_STAR {
            return node1;
        }

        // Absorption
        if node1 == NodeId::NOTHING || node2 == NodeId::NOTHING {
            return NodeId::NOTHING;
        }

        // Check for complementary patterns: A & ~A = ⊥
        if let Some(RegexNode::Not { inner }) = self.arena.node(node1) {
            if *inner == node2 {
                return NodeId::NOTHING;
            }
        }
        if let Some(RegexNode::Not { inner }) = self.arena.node(node2) {
            if *inner == node1 {
                return NodeId::NOTHING;
            }
        }

        // Merge singletons: [a] & [b] = [a∩b]
        if let (Some(RegexNode::Singleton(s1)), Some(RegexNode::Singleton(s2))) =
            (self.arena.node(node1), self.arena.node(node2))
        {
            let merged = self.solver.and(s1, s2);
            return self.singleton(merged);
        }

        // Subsumption for star loops: .* & .*s = .*s
        if let (Some(pred1), Some(pred2)) = (self.get_star_pred(node1), self.get_star_pred(node2)) {
            if self.solver.contains(&pred1, &pred2) {
                return node2; // node2 is more restrictive
            }
            if self.solver.contains(&pred2, &pred1) {
                return node1; // node1 is more restrictive
            }
        }

        // Create And node with sorted children
        let mut nodes = vec![node1, node2];
        nodes.sort();
        self.arena.alloc(RegexNode::And { nodes })
    }

    /// Create a loop (repetition) pattern.
    pub fn loop_(&mut self, node: NodeId, low: u32, high: u32, lazy: bool) -> NodeId {
        self.mk_loop(node, low, high, lazy)
    }

    /// Create a normalized loop pattern.
    fn mk_loop(&mut self, node: NodeId, low: u32, high: u32, lazy: bool) -> NodeId {
        // Empty range
        if low > high {
            return NodeId::NOTHING;
        }

        // Exact match {1,1}
        if low == 1 && high == 1 {
            return node;
        }

        // Optional empty {0,0}
        if high == 0 {
            return NodeId::EPSILON;
        }

        // Handle special nodes
        if node == NodeId::EPSILON {
            return NodeId::EPSILON;
        }
        if node == NodeId::NOTHING {
            return if low == 0 {
                NodeId::EPSILON
            } else {
                NodeId::NOTHING
            };
        }

        // Nested loop simplification: (a{m,n}){p,q} = a{m*p, n*q}
        if let Some(RegexNode::Loop {
            node: inner,
            low: inner_low,
            high: inner_high,
            ..
        }) = self.arena.node(node)
        {
            let new_low = inner_low.saturating_mul(low);
            let new_high = if *inner_high == u32::MAX || high == u32::MAX {
                u32::MAX
            } else {
                inner_high.saturating_mul(high)
            };
            return self.mk_loop(*inner, new_low, new_high, lazy);
        }

        self.arena.alloc(RegexNode::Loop {
            node,
            low,
            high,
            lazy,
        })
    }

    /// Create a negation pattern.
    pub fn not(&mut self, inner: NodeId) -> NodeId {
        self.mk_not(inner)
    }

    /// Create a normalized negation pattern.
    fn mk_not(&mut self, inner: NodeId) -> NodeId {
        // Double negation: ~~A = A
        if let Some(RegexNode::Not { inner: nested }) = self.arena.node(inner) {
            return *nested;
        }

        // ~⊥ = ⊤*
        if inner == NodeId::NOTHING {
            return NodeId::TOP_STAR;
        }

        // ~⊤* = ⊥
        if inner == NodeId::TOP_STAR {
            return NodeId::NOTHING;
        }

        self.arena.alloc(RegexNode::Not { inner })
    }

    /// Create a lookaround assertion.
    pub fn lookaround(&mut self, inner: NodeId, look_back: bool, negative: bool) -> NodeId {
        // Empty lookaround
        if inner == NodeId::EPSILON {
            return if negative {
                NodeId::NOTHING
            } else {
                NodeId::EPSILON
            };
        }

        self.arena.alloc(RegexNode::LookAround {
            inner,
            look_back,
            negative,
        })
    }

    /// Create a begin anchor.
    pub fn begin(&mut self) -> NodeId {
        self.arena.alloc(RegexNode::Begin)
    }

    /// Create an end anchor.
    pub fn end(&mut self) -> NodeId {
        self.arena.alloc(RegexNode::End)
    }

    /// Get the predicate of a star loop (.*-like pattern), if this is one.
    fn get_star_pred(&self, id: NodeId) -> Option<S::CharSet> {
        // Handle TOP_STAR specially
        if id == NodeId::TOP_STAR {
            return Some(self.solver.full());
        }

        if let Some(RegexNode::Loop {
            node,
            low: 0,
            high: u32::MAX,
            ..
        }) = self.arena.node(id)
        {
            if let Some(RegexNode::Singleton(charset)) = self.arena.node(*node) {
                return Some(charset.clone());
            }
        }
        None
    }

    /// Get the info for a node.
    pub fn get_info(&self, id: NodeId) -> Option<&NodeInfo<S::CharSet>> {
        self.arena.get(id)
    }

    /// Get the flags for a node.
    pub fn flags(&self, id: NodeId) -> NodeFlags {
        self.arena.flags(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::BitSetSolver;

    #[test]
    fn test_or_identity() {
        let solver = BitSetSolver;
        let mut builder = RegexBuilder::new(solver);

        // A | ⊥ = A
        let a = builder.singleton(0b1010);
        assert_eq!(builder.or(a, NodeId::NOTHING), a);
        assert_eq!(builder.or(NodeId::NOTHING, a), a);

        // A | ⊤* = ⊤*
        assert_eq!(builder.or(a, NodeId::TOP_STAR), NodeId::TOP_STAR);
    }

    #[test]
    fn test_and_identity() {
        let solver = BitSetSolver;
        let mut builder = RegexBuilder::new(solver);

        // A & ⊤* = A
        let a = builder.singleton(0b1010);
        assert_eq!(builder.and(a, NodeId::TOP_STAR), a);
        assert_eq!(builder.and(NodeId::TOP_STAR, a), a);

        // A & ⊥ = ⊥
        assert_eq!(builder.and(a, NodeId::NOTHING), NodeId::NOTHING);
    }

    #[test]
    fn test_singleton_merge() {
        let solver = BitSetSolver;
        let mut builder = RegexBuilder::new(solver);

        let a = builder.singleton(0b0011);
        let b = builder.singleton(0b1100);

        // [a] | [b] = [ab]
        let or_result = builder.or(a, b);
        if let Some(RegexNode::Singleton(s)) = builder.arena().node(or_result) {
            assert_eq!(*s, 0b1111);
        } else {
            panic!("Expected Singleton");
        }

        // [a] & [b] = [a∩b] = ∅ = ⊥
        let and_result = builder.and(a, b);
        assert_eq!(and_result, NodeId::NOTHING);
    }

    #[test]
    fn test_double_negation() {
        let solver = BitSetSolver;
        let mut builder = RegexBuilder::new(solver);

        let a = builder.singleton(0b1010);
        let not_a = builder.not(a);
        let not_not_a = builder.not(not_a);

        assert_eq!(not_not_a, a);
    }

    #[test]
    fn test_complementary_or() {
        let solver = BitSetSolver;
        let mut builder = RegexBuilder::new(solver);

        let a = builder.singleton(0b1010);
        let not_a = builder.not(a);

        // A | ~A = ⊤*
        assert_eq!(builder.or(a, not_a), NodeId::TOP_STAR);
    }

    #[test]
    fn test_complementary_and() {
        let solver = BitSetSolver;
        let mut builder = RegexBuilder::new(solver);

        let a = builder.singleton(0b1010);
        let not_a = builder.not(a);

        // A & ~A = ⊥
        assert_eq!(builder.and(a, not_a), NodeId::NOTHING);
    }

    #[test]
    fn test_loop_normalization() {
        let solver = BitSetSolver;
        let mut builder = RegexBuilder::new(solver);

        let a = builder.singleton(0b1010);

        // a{1,1} = a
        assert_eq!(builder.loop_(a, 1, 1, false), a);

        // a{0,0} = ε
        assert_eq!(builder.loop_(a, 0, 0, false), NodeId::EPSILON);

        // ε* = ε
        assert_eq!(
            builder.loop_(NodeId::EPSILON, 0, u32::MAX, false),
            NodeId::EPSILON
        );
    }

    #[test]
    fn test_concat_identity() {
        let solver = BitSetSolver;
        let mut builder = RegexBuilder::new(solver);

        let a = builder.singleton(0b1010);

        // a · ε = a
        assert_eq!(builder.concat(a, NodeId::EPSILON), a);
        assert_eq!(builder.concat(NodeId::EPSILON, a), a);

        // a · ⊥ = ⊥
        assert_eq!(builder.concat(a, NodeId::NOTHING), NodeId::NOTHING);
    }
}
