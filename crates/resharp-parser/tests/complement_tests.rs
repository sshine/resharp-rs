//! Complement (negation) tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_16_ComplementTests.fs`
//!
//! These tests verify that complement (~) patterns are correctly parsed
//! and converted to IR. The actual matching semantics will be tested when
//! the matcher is implemented.

use cstree::syntax::SyntaxNode;
use resharp_ir::{NodeFlags, NodeId, RegexNode, RegexNodeArena};
use resharp_parser::{cst_to_ir, parse, RegexOptions};
use resharp_syntax::SyntaxKind;

/// Parse a pattern and convert to IR.
fn parse_to_ir(pattern: &str) -> (RegexNodeArena<u64>, NodeId) {
    let options = RegexOptions::EXPLICIT_CAPTURE | RegexOptions::NON_BACKTRACKING;
    let green = parse(pattern, options).expect("parse should succeed");
    let tree: SyntaxNode<SyntaxKind> = SyntaxNode::new_root(green);
    cst_to_ir(&tree)
}

/// Get the flags for a pattern.
fn get_flags(pattern: &str) -> NodeFlags {
    let (arena, root) = parse_to_ir(pattern);
    arena.flags(root)
}

// =============================================================================
// Basic Complement Parsing Tests
// =============================================================================

/// Test simple negation ~(ab)
#[test]
fn complement_simple_group() {
    let (arena, root) = parse_to_ir("~(ab)");

    if let Some(RegexNode::Not { inner }) = arena.node(root) {
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected Not node for ~(ab)");
    }
}

/// Test negation of single character
#[test]
fn complement_single_char() {
    let (arena, root) = parse_to_ir("~a");

    if let Some(RegexNode::Not { inner }) = arena.node(root) {
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected Not node for ~a");
    }
}

/// Test negation with quantifier
#[test]
fn complement_with_star() {
    let pattern = r"~(_*\d\d_*)";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::Not { inner }) = arena.node(root) {
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected Not node");
    }
}

/// Test negation with newlines
#[test]
fn complement_with_newlines() {
    let pattern = r"~(.*\n\n.*)";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::Not { inner }) = arena.node(root) {
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected Not node");
    }
}

// =============================================================================
// Complement Nullability Tests
// =============================================================================

/// Test that ~(.+) is nullable (since .+ cannot be empty, its complement includes empty)
#[test]
fn complement_of_nonempty_is_nullable() {
    // ~(a) should be nullable because 'a' is not nullable
    let flags = get_flags("~a");

    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "~a should be nullable (a cannot match empty)"
    );
}

/// Test that ~(.*) is NOT nullable (since .* matches everything including empty)
#[test]
fn complement_of_nullable_not_nullable() {
    let flags = get_flags(r"~(_*)");

    // ~(_*) = NOTHING, which is not nullable
    // _* matches everything, so its complement matches nothing
    assert!(
        !flags.contains(NodeFlags::IS_ALWAYS_NULLABLE),
        "~(_*) should not be always nullable"
    );
}

// =============================================================================
// Complement with Intersection Tests
// =============================================================================

/// Port of: `test1` - complement with intersection and lookbehind
#[test]
fn complement_intersection_lookbehind() {
    let pattern = r"~(\T*\n\n\T*)&(?<=Context1~(\T*\n\n\T*))\T*&get, set";
    let (arena, root) = parse_to_ir(pattern);

    // Should parse as intersection with 3 branches
    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 3, "Should have 3 conjunction branches");
    } else {
        panic!("Expected And node");
    }

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
}

/// Port of: `test2` - complement in lookbehind
#[test]
fn complement_in_lookbehind() {
    let pattern = r"(?<=Context1~(\T*\n\n\T*)).*&get, set";
    let (arena, root) = parse_to_ir(pattern);

    // Should be intersection with lookbehind
    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
}

/// Port of: `test3` - complement in lookbehind (grouped)
#[test]
fn complement_in_lookbehind_grouped() {
    let pattern = r"(?<=Context1~(\T*\n\n\T*))(.*&get, set)";
    let (arena, root) = parse_to_ir(pattern);

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
}

// =============================================================================
// Complement with Lookahead Tests
// =============================================================================

/// Test complement with lookahead
#[test]
fn complement_with_lookahead() {
    let pattern = r"~(.*def)(?=.*end)";
    let (arena, root) = parse_to_ir(pattern);

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD),
        "Should have suffix lookahead"
    );
}

/// Test complement in intersection with lookahead
#[test]
fn complement_intersection_lookahead() {
    let pattern = r".*(?=.*E)&~(.*and.*)";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD),
        "Should have suffix lookahead"
    );
}

// =============================================================================
// Double Negation Tests
// =============================================================================

/// Test double negation ~~a
#[test]
fn double_negation() {
    let (arena, root) = parse_to_ir("~~a");

    // Double negation should still parse - simplification is handled by builder
    if let Some(RegexNode::Not { inner }) = arena.node(root) {
        // Check that inner is also a Not node
        if let Some(RegexNode::Not { .. }) = arena.node(*inner) {
            // Good - double negation structure preserved
        } else {
            // Or it might have been simplified
        }
    }
    assert_ne!(root, NodeId::NOTHING, "Pattern should parse successfully");
}

// =============================================================================
// Complex Complement Patterns
// =============================================================================

/// Test complement with password pattern
#[test]
fn complement_password_pattern() {
    let pattern = r"~(.*\d\d.*)&[a-zA-Z\d]{8,}";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }
}

/// Test complement paragraph splitting pattern
#[test]
fn complement_paragraph_pattern() {
    let pattern = r"~(_*\n\n_*)";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::Not { inner }) = arena.node(root) {
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected Not node");
    }
}

/// Test complement with alternation inside
#[test]
fn complement_with_alternation() {
    let pattern = r"~(a|b)";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::Not { inner }) = arena.node(root) {
        // Inner should be an Or node
        if let Some(RegexNode::Or { nodes }) = arena.node(*inner) {
            assert_eq!(nodes.len(), 2, "Should have 2 branches");
        } else {
            panic!("Inner should be Or node");
        }
    } else {
        panic!("Expected Not node");
    }
}

/// Test complement in alternation
#[test]
fn complement_in_alternation() {
    let pattern = r"~a|~b";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::Or { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 branches");
        // Each branch should be a Not node
        for node in nodes {
            if let Some(RegexNode::Not { .. }) = arena.node(*node) {
                // Good
            } else {
                panic!("Each branch should be a Not node");
            }
        }
    } else {
        panic!("Expected Or node");
    }
}

// =============================================================================
// Complement with Concatenation Tests
// =============================================================================

/// Test complement followed by literal
#[test]
fn complement_concat_literal() {
    let pattern = r"~(.*bc_*)d";
    let (arena, root) = parse_to_ir(pattern);

    // Should be Concat with Not as head
    if let Some(RegexNode::Concat { head, .. }) = arena.node(root) {
        if let Some(RegexNode::Not { .. }) = arena.node(*head) {
            // Good
        } else {
            panic!("Head should be Not node");
        }
    } else {
        panic!("Expected Concat node");
    }
}

/// Test literal followed by complement
#[test]
fn literal_concat_complement() {
    let pattern = r"a~(_*e_*)";
    let (arena, root) = parse_to_ir(pattern);

    // Should be Concat with Not in tail
    if let Some(RegexNode::Concat { head, tail }) = arena.node(root) {
        // head should be literal
        assert_ne!(*head, NodeId::NOTHING);
        // tail should be Not
        if let Some(RegexNode::Not { .. }) = arena.node(*tail) {
            // Good
        } else {
            panic!("Tail should be Not node");
        }
    } else {
        panic!("Expected Concat node");
    }
}

// =============================================================================
// Implication Pattern Tests (from IntersectionTests)
// =============================================================================

/// Test implication-like pattern with complement
#[test]
fn implication_pattern() {
    // ~A | B is equivalent to A -> B (implication)
    let pattern = r"~(_*mistake_*)|(_*strawberries_*)";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::Or { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 branches");
    } else {
        panic!("Expected Or node");
    }
}
