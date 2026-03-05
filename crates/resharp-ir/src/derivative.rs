//! Brzozowski derivatives for regex patterns.
//!
//! This module implements the derivative operation for regex patterns,
//! which is the foundation of derivative-based matching algorithms.
//!
//! The derivative of a pattern R with respect to a character c is a new
//! pattern R' such that: L(R') = { w | cw ∈ L(R) }
//!
//! In other words, R' matches the "remainders" of strings in L(R) that
//! start with c.

use crate::builder::RegexBuilder;
use crate::node::{NodeId, RegexNode};
use crate::solver::CharSetSolver;

/// Location within the input string, used for anchor handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationKind {
    /// At the beginning of the input.
    Begin,
    /// In the middle of the input (not at either end).
    Center,
    /// At the end of the input.
    End,
}

/// Check if a regex pattern is nullable (can match the empty string)
/// at the given location.
///
/// This takes into account anchors (^ and $) which are only nullable
/// at specific locations.
pub fn is_nullable<S: CharSetSolver>(
    builder: &RegexBuilder<S>,
    loc: LocationKind,
    id: NodeId,
) -> bool {
    let flags = builder.flags(id);

    // Fast path: use precomputed flags
    if !flags.can_be_nullable() {
        return false;
    }
    if flags.is_always_nullable() {
        return true;
    }

    // Need to check recursively
    let Some(info) = builder.get_info(id) else {
        return false;
    };

    match &info.node {
        RegexNode::Singleton(_) => false,

        RegexNode::Or { nodes } => nodes.iter().any(|&n| is_nullable(builder, loc, n)),

        RegexNode::And { nodes } => nodes.iter().all(|&n| is_nullable(builder, loc, n)),

        RegexNode::Loop { node, low, .. } => *low == 0 || is_nullable(builder, loc, *node),

        RegexNode::Not { inner } => !is_nullable(builder, loc, *inner),

        RegexNode::Concat { head, tail } => {
            is_nullable(builder, loc, *head) && is_nullable(builder, loc, *tail)
        }

        RegexNode::LookAround { inner, .. } => is_nullable(builder, loc, *inner),

        RegexNode::Begin => loc == LocationKind::Begin,

        RegexNode::End => loc == LocationKind::End,
    }
}

/// Compute the Brzozowski derivative of a pattern with respect to a character.
///
/// # Arguments
///
/// * `builder` - The regex builder (for creating new nodes)
/// * `loc` - The current location in the input
/// * `char_class` - The character class (minterm) being consumed
/// * `id` - The node ID to derive
///
/// # Returns
///
/// The node ID of the derivative pattern.
pub fn derivative<S: CharSetSolver>(
    builder: &mut RegexBuilder<S>,
    loc: LocationKind,
    char_class: S::CharSet,
    id: NodeId,
) -> NodeId {
    // Handle special nodes
    match id {
        NodeId::EPSILON => return NodeId::NOTHING,
        NodeId::NOTHING => return NodeId::NOTHING,
        // D_c(_*) = _* (TOP_STAR matches any string including empty)
        NodeId::TOP_STAR => return NodeId::TOP_STAR,
        // D_c(.) = ε (ANY matches any single character)
        NodeId::ANY => return NodeId::EPSILON,
        _ => {}
    }

    let Some(info) = builder.get_info(id).cloned() else {
        return NodeId::NOTHING;
    };

    match &info.node {
        RegexNode::Singleton(pred) => {
            // D_c([pred]) = ε if c ∈ pred, else ⊥
            if builder.solver().contains(pred, &char_class) {
                NodeId::EPSILON
            } else {
                NodeId::NOTHING
            }
        }

        RegexNode::Loop {
            node: r,
            low,
            high,
            lazy,
        } => {
            // D_c(R{m,n}) = D_c(R) · R{m-1, n-1}
            let r = *r;
            let low = *low;
            let high = *high;
            let lazy = *lazy;

            let decr = |x: u32| {
                if x == u32::MAX || x == 0 {
                    x
                } else {
                    x - 1
                }
            };

            let r_decr = builder.loop_(r, decr(low), decr(high), lazy);
            let dr = derivative(builder, loc, char_class, r);
            builder.concat(dr, r_decr)
        }

        RegexNode::Or { nodes } => {
            // D_c(R|S) = D_c(R) | D_c(S)
            let nodes = nodes.clone();
            let mut result = NodeId::NOTHING;

            for node in nodes {
                let d = derivative(builder, loc, char_class.clone(), node);
                if d != NodeId::NOTHING {
                    if result == NodeId::NOTHING {
                        result = d;
                    } else {
                        result = builder.or(result, d);
                    }
                }
            }

            result
        }

        RegexNode::And { nodes } => {
            // D_c(R&S) = D_c(R) & D_c(S)
            let nodes = nodes.clone();
            let mut result = NodeId::TOP_STAR;

            for node in nodes {
                let d = derivative(builder, loc, char_class.clone(), node);
                if d != NodeId::TOP_STAR {
                    if result == NodeId::TOP_STAR {
                        result = d;
                    } else {
                        result = builder.and(result, d);
                    }
                }
            }

            result
        }

        RegexNode::Not { inner } => {
            // D_c(~R) = ~D_c(R)
            let inner = *inner;
            let d = derivative(builder, loc, char_class, inner);
            builder.not(d)
        }

        RegexNode::Concat { head, tail } => {
            // D_c(RS) = D_c(R)S | nullable(R)?D_c(S)
            let head = *head;
            let tail = *tail;

            let dr = derivative(builder, loc, char_class.clone(), head);
            let dr_s = builder.concat(dr, tail);

            if is_nullable(builder, loc, head) {
                let ds = derivative(builder, loc, char_class, tail);
                if ds == NodeId::NOTHING {
                    dr_s
                } else if dr_s == NodeId::NOTHING {
                    ds
                } else {
                    builder.or(dr_s, ds)
                }
            } else {
                dr_s
            }
        }

        RegexNode::LookAround {
            inner,
            look_back,
            negative,
        } => {
            let inner = *inner;
            let look_back = *look_back;
            let negative = *negative;

            if look_back {
                // Lookbehind: D_c((?<=R)) = (?<=D_c(R))
                // We need to handle the case where the inner pattern starts with _*
                if let Some(RegexNode::Concat { head, tail }) =
                    builder.get_info(inner).map(|i| &i.node).cloned()
                {
                    if head == NodeId::TOP_STAR {
                        // (?<=_*R) -> (?<=D_c(R))
                        let d = derivative(builder, loc, char_class, tail);
                        return builder.lookaround(d, true, negative);
                    }
                }
                let d = derivative(builder, loc, char_class, inner);
                builder.lookaround(d, true, negative)
            } else {
                // Lookahead: D_c((?=R)) = (?=D_c(R))
                let d = derivative(builder, loc, char_class, inner);
                builder.lookaround(d, false, negative)
            }
        }

        RegexNode::Begin | RegexNode::End => {
            // Anchors don't consume characters
            NodeId::NOTHING
        }
    }
}

/// Helper to get the solver from the builder.
impl<S: CharSetSolver> RegexBuilder<S> {
    /// Get a reference to the solver.
    pub fn solver(&self) -> &S {
        &self.solver
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BitSetSolver;

    fn new_builder() -> RegexBuilder<BitSetSolver> {
        RegexBuilder::new(BitSetSolver)
    }

    #[test]
    fn test_is_nullable_epsilon() {
        let builder = new_builder();
        assert!(is_nullable(&builder, LocationKind::Center, NodeId::EPSILON));
        assert!(is_nullable(&builder, LocationKind::Begin, NodeId::EPSILON));
        assert!(is_nullable(&builder, LocationKind::End, NodeId::EPSILON));
    }

    #[test]
    fn test_is_nullable_nothing() {
        let builder = new_builder();
        assert!(!is_nullable(
            &builder,
            LocationKind::Center,
            NodeId::NOTHING
        ));
    }

    #[test]
    fn test_is_nullable_top_star() {
        let builder = new_builder();
        assert!(is_nullable(
            &builder,
            LocationKind::Center,
            NodeId::TOP_STAR
        ));
    }

    #[test]
    fn test_derivative_singleton() {
        let mut builder = new_builder();
        let a = builder.singleton(0b0001);

        // D_a([a]) = ε
        let da = derivative(&mut builder, LocationKind::Center, 0b0001, a);
        assert_eq!(da, NodeId::EPSILON);

        // D_b([a]) = ⊥ (when b doesn't match)
        let db = derivative(&mut builder, LocationKind::Center, 0b0010, a);
        assert_eq!(db, NodeId::NOTHING);
    }

    #[test]
    fn test_derivative_concat() {
        let mut builder = new_builder();

        // Pattern: ab
        let a = builder.singleton(0b0001);
        let b = builder.singleton(0b0010);
        let ab = builder.concat(a, b);

        // D_a(ab) = b
        let da = derivative(&mut builder, LocationKind::Center, 0b0001, ab);
        assert_eq!(da, b);

        // D_b(ab) = ⊥ (a doesn't match b)
        let db = derivative(&mut builder, LocationKind::Center, 0b0010, ab);
        assert_eq!(db, NodeId::NOTHING);
    }

    #[test]
    fn test_derivative_or() {
        let mut builder = new_builder();

        // Pattern: a|b
        let a = builder.singleton(0b0001);
        let b = builder.singleton(0b0010);
        let a_or_b = builder.or(a, b);

        // D_a(a|b) = ε
        let da = derivative(&mut builder, LocationKind::Center, 0b0001, a_or_b);
        assert_eq!(da, NodeId::EPSILON);

        // D_b(a|b) = ε
        let db = derivative(&mut builder, LocationKind::Center, 0b0010, a_or_b);
        assert_eq!(db, NodeId::EPSILON);
    }

    #[test]
    fn test_derivative_loop() {
        let mut builder = new_builder();

        // Pattern: a*
        let a = builder.singleton(0b0001);
        let a_star = builder.loop_(a, 0, u32::MAX, false);

        // D_a(a*) = ε · a* = a* (structurally)
        // Since D_a([a]) = ε and ε · a* = a*
        let da = derivative(&mut builder, LocationKind::Center, 0b0001, a_star);

        // The result should be a loop with the same structure
        if let Some(RegexNode::Loop {
            node, low, high, ..
        }) = builder.arena().node(da)
        {
            assert_eq!(*node, a);
            assert_eq!(*low, 0);
            assert_eq!(*high, u32::MAX);
        } else {
            panic!("Expected Loop node for D_a(a*)");
        }
    }

    #[test]
    fn test_derivative_plus() {
        let mut builder = new_builder();

        // Pattern: a+
        let a = builder.singleton(0b0001);
        let a_plus = builder.loop_(a, 1, u32::MAX, false);

        // D_a(a+) = a*
        let da = derivative(&mut builder, LocationKind::Center, 0b0001, a_plus);
        // The result should be a loop with low=0 (a*)
        if let Some(RegexNode::Loop {
            node, low, high, ..
        }) = builder.arena().node(da)
        {
            assert_eq!(*node, a);
            assert_eq!(*low, 0);
            assert_eq!(*high, u32::MAX);
        } else {
            panic!("Expected Loop node");
        }
    }

    #[test]
    fn test_derivative_not() {
        let mut builder = new_builder();

        // Pattern: ~[a]
        let a = builder.singleton(0b0001);
        let not_a = builder.not(a);

        // D_a(~[a]) = ~D_a([a]) = ~ε
        // ~ε matches everything EXCEPT the empty string (i.e., all non-empty strings)
        // This is NOT the same as NOTHING (⊥)
        let da = derivative(&mut builder, LocationKind::Center, 0b0001, not_a);

        // The result should be ~ε (a Not node wrapping EPSILON)
        if let Some(RegexNode::Not { inner }) = builder.arena().node(da) {
            assert_eq!(*inner, NodeId::EPSILON);
        } else {
            panic!("Expected Not(EPSILON) for D_a(~[a])");
        }
    }
}
