//! Intersection tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_08_IntersectionTests.fs`
//!
//! These tests verify that intersection (&) patterns are correctly parsed
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
// Basic Intersection Parsing Tests
// =============================================================================

/// Port of: `conjunction match tests 1` - pattern structure only
///
/// Tests that `c...&...s` parses to an And node.
#[test]
fn intersection_parse_dots_pattern() {
    let (arena, root) = parse_to_ir("c...&...s");

    // Should be an And node with two children
    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!(
            "Expected And node for c...&...s, got {:?}",
            arena.node(root)
        );
    }
}

/// Port of: `conjunction match tests 1` - pattern structure only
///
/// Tests that `c.*&.*s` parses to an And node.
#[test]
fn intersection_parse_stars_pattern() {
    let (arena, root) = parse_to_ir("c.*&.*s");

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node for c.*&.*s");
    }
}

/// Port of: `.*rain.*&.*dogs.*` - pattern structure only
#[test]
fn intersection_parse_two_patterns() {
    let (arena, root) = parse_to_ir(".*rain.*&.*dogs.*");

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }
}

// =============================================================================
// Multi-way Intersection Tests
// =============================================================================

/// Port of: `more than 3 cases short`
///
/// Tests that four-way intersection parses correctly.
#[test]
fn intersection_four_way() {
    let pattern = "_*Thursday_*&_*April_*&_*Went_*&_*ashore_*";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 4, "Should have 4 conjunction branches");
    } else {
        panic!("Expected And node with 4 branches");
    }
}

/// Port of: `more than 3 cases short 2` with [\s\S]
#[test]
fn intersection_four_way_with_charclass() {
    let pattern =
        r"[\s\S]*French[\s\S]*&[\s\S]*English[\s\S]*&[\s\S]*Chinese[\s\S]*&[\s\S]*Arabs[\s\S]*";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 4, "Should have 4 conjunction branches");
    } else {
        panic!("Expected And node with 4 branches");
    }
}

// =============================================================================
// Intersection with Lookaround Tests
// =============================================================================

/// Port of: `twain paragraph test 5` - pattern structure only
///
/// Tests intersection with negation and lookaround.
#[test]
fn intersection_with_negation() {
    let pattern = r"\n\n~(_*\n\n_*)\n\n&_*(Arkansaw)_*";
    let (arena, root) = parse_to_ir(pattern);

    // Should parse without error and be an And node
    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }
}

/// Port of: `implication 1` - pattern structure only
#[test]
fn intersection_with_lookbehind() {
    let pattern = r"(?<=\n\n|\A)(~(_*\n\n_*)&(~(_*mistake_*)|(_*strawberries_*)))(?=\n\n)";
    let (arena, root) = parse_to_ir(pattern);

    // Should parse without error
    assert_ne!(root, NodeId::NOTHING, "Pattern should parse successfully");
    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
    assert!(
        flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD),
        "Should have suffix lookahead"
    );
}

/// Port of: `implication 2` - pattern structure only
#[test]
fn intersection_with_negation_in_both_branches() {
    let pattern = r"\n~(_*\n\n_*)\n&~(_*honor_*)";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }
}

// =============================================================================
// Intersection with Anchors Tests
// =============================================================================

/// Tests intersection with anchors
#[test]
fn intersection_with_anchors() {
    let pattern = r"[0-9]{2}[/.-][0-9]{2}[/.-]([0-9]{4}|[0-9]{2})&^.*$";
    let (arena, root) = parse_to_ir(pattern);

    // Should have anchor dependency
    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::DEPENDS_ON_ANCHOR),
        "Pattern with ^ and $ should depend on anchor"
    );
}

/// Tests intersection with end anchor only
#[test]
fn intersection_with_end_anchor() {
    let pattern = r"[0-9]{2}[/.-][0-9]{2}[/.-]([0-9]{4}|[0-9]{2})&.*$";
    let (arena, root) = parse_to_ir(pattern);

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::DEPENDS_ON_ANCHOR),
        "Pattern with $ should depend on anchor"
    );
}

// =============================================================================
// Complex Intersection Tests
// =============================================================================

/// Tests intersection with complement prefix
#[test]
fn intersection_complement_prefix() {
    let pattern = r"~(.*\d\d.*)&^.*$";
    let (arena, root) = parse_to_ir(pattern);

    // Should parse without error
    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }
}

/// Tests intersection with lookbehind prefix
#[test]
fn intersection_lookbehind_prefix() {
    let pattern = r"(?<=author).*&.*";
    let (arena, root) = parse_to_ir(pattern);

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
}

/// Tests intersection with lookbehind and complement
#[test]
fn intersection_lookbehind_complement() {
    let pattern = r"(?<=__).*&~(.*and.*)";
    let (arena, root) = parse_to_ir(pattern);

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
}

/// Tests intersection with lookahead
#[test]
fn intersection_with_lookahead() {
    let pattern = r"(?<=__).*(?=.*def)&.*and.*";
    let (arena, root) = parse_to_ir(pattern);

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
    assert!(
        flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD),
        "Should have suffix lookahead"
    );
}

/// Port of: `script test 1` - pattern structure only
#[test]
fn intersection_two_way_literal() {
    let pattern = r"THE.*LIFE&.*FIVE.*";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }
}

// =============================================================================
// Intersection Nullability Tests
// =============================================================================

/// Tests nullability of intersection patterns
#[test]
fn intersection_nullable() {
    // .*&.* is nullable (both branches can match empty)
    let pattern = r".*&.*";
    let flags = get_flags(pattern);
    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        ".*&.* should be nullable"
    );
}

/// Tests non-nullability of intersection patterns
#[test]
fn intersection_not_nullable() {
    // a&b requires at least one character from each
    let pattern = r"a&b";
    let flags = get_flags(pattern);
    assert!(
        !flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "a&b should not be nullable"
    );
}

// =============================================================================
// Subsumption in Intersection Tests
// =============================================================================

/// Tests intersection with RE# wildcard
#[test]
fn intersection_with_resharp_wildcard() {
    let pattern = r"(a_*&(~(a.*)|.*b>))";
    let (_arena, root) = parse_to_ir(pattern);

    // Should parse without error
    assert_ne!(root, NodeId::NOTHING, "Pattern should parse successfully");
}

/// Tests intersection with password-like patterns
#[test]
fn intersection_password_pattern() {
    let pattern = r"~(.*\d\d.*)&[a-zA-Z\d]{8,}";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }
}

/// Tests three-way intersection
#[test]
fn intersection_three_way() {
    let pattern = r".*a.*&.*b.*&.*c.*";
    let (arena, root) = parse_to_ir(pattern);

    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 3, "Should have 3 conjunction branches");
    } else {
        panic!("Expected And node with 3 branches");
    }
}
