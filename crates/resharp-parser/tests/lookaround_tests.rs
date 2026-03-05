//! Lookaround tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_11_LookaroundTests.fs`
//!
//! These tests verify that lookaround patterns are correctly parsed
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
// Positive Lookahead Tests
// =============================================================================

/// Test positive lookahead (?=...)
#[test]
fn positive_lookahead_structure() {
    let (arena, root) = parse_to_ir("(?=a)");

    if let Some(RegexNode::LookAround {
        inner,
        look_back,
        negative,
    }) = arena.node(root)
    {
        assert!(!look_back, "Should be lookahead (not lookbehind)");
        assert!(!negative, "Should be positive (not negative)");
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected LookAround node");
    }
}

/// Test lookahead flags
#[test]
fn positive_lookahead_flags() {
    let flags = get_flags("(?=a)");

    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "Lookahead should be nullable"
    );
    assert!(
        flags.contains(NodeFlags::IS_ALWAYS_NULLABLE),
        "Lookahead should be always nullable"
    );
    assert!(
        flags.contains(NodeFlags::CONTAINS_LOOKAROUND),
        "Should contain lookaround"
    );
    assert!(
        flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD),
        "Bare lookahead should have suffix lookahead flag"
    );
}

/// Test concatenation with lookahead suffix
#[test]
fn concat_with_lookahead_suffix() {
    let flags = get_flags("a(?=b)");

    assert!(
        flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD),
        "Pattern with lookahead suffix should have the flag"
    );
    assert!(
        !flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "a(?=b) should not be nullable"
    );
}

// =============================================================================
// Positive Lookbehind Tests
// =============================================================================

/// Test positive lookbehind (?<=...)
#[test]
fn positive_lookbehind_structure() {
    let (arena, root) = parse_to_ir("(?<=a)");

    if let Some(RegexNode::LookAround {
        inner,
        look_back,
        negative,
    }) = arena.node(root)
    {
        assert!(*look_back, "Should be lookbehind");
        assert!(!negative, "Should be positive (not negative)");
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected LookAround node");
    }
}

/// Test lookbehind flags
#[test]
fn positive_lookbehind_flags() {
    let flags = get_flags("(?<=a)");

    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "Lookbehind should be nullable"
    );
    assert!(
        flags.contains(NodeFlags::IS_ALWAYS_NULLABLE),
        "Lookbehind should be always nullable"
    );
    assert!(
        flags.contains(NodeFlags::CONTAINS_LOOKAROUND),
        "Should contain lookaround"
    );
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Bare lookbehind should have prefix lookbehind flag"
    );
}

/// Test concatenation with lookbehind prefix
#[test]
fn concat_with_lookbehind_prefix() {
    let flags = get_flags("(?<=a)b");

    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Pattern with lookbehind prefix should have the flag"
    );
    assert!(
        !flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "(?<=a)b should not be nullable"
    );
}

// =============================================================================
// Negative Lookahead Tests
// =============================================================================

/// Test negative lookahead (?!...)
#[test]
fn negative_lookahead_structure() {
    let (arena, root) = parse_to_ir("(?!a)");

    if let Some(RegexNode::LookAround {
        inner,
        look_back,
        negative,
    }) = arena.node(root)
    {
        assert!(!look_back, "Should be lookahead (not lookbehind)");
        assert!(*negative, "Should be negative");
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected LookAround node");
    }
}

/// Test negative lookahead flags
#[test]
fn negative_lookahead_flags() {
    let flags = get_flags("(?!a)");

    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "Negative lookahead should be nullable"
    );
    assert!(
        flags.contains(NodeFlags::CONTAINS_LOOKAROUND),
        "Should contain lookaround"
    );
    assert!(
        flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD),
        "Negative lookahead should have suffix lookahead flag"
    );
}

// =============================================================================
// Negative Lookbehind Tests
// =============================================================================

/// Test negative lookbehind (?<!...)
#[test]
fn negative_lookbehind_structure() {
    let (arena, root) = parse_to_ir("(?<!a)");

    if let Some(RegexNode::LookAround {
        inner,
        look_back,
        negative,
    }) = arena.node(root)
    {
        assert!(*look_back, "Should be lookbehind");
        assert!(*negative, "Should be negative");
        assert_ne!(*inner, NodeId::NOTHING, "Inner should not be NOTHING");
    } else {
        panic!("Expected LookAround node");
    }
}

/// Test negative lookbehind flags
#[test]
fn negative_lookbehind_flags() {
    let flags = get_flags("(?<!a)");

    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "Negative lookbehind should be nullable"
    );
    assert!(
        flags.contains(NodeFlags::CONTAINS_LOOKAROUND),
        "Should contain lookaround"
    );
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Negative lookbehind should have prefix lookbehind flag"
    );
}

// =============================================================================
// Complex Lookaround Patterns
// =============================================================================

/// Port of: `c intersect 1.2b` - pattern structure only
#[test]
fn lookaround_intersection_structure() {
    let pattern = r"(?<=author).*&.*and.*";
    let (arena, root) = parse_to_ir(pattern);

    // Should be an And node with lookbehind in first branch
    if let Some(RegexNode::And { nodes }) = arena.node(root) {
        assert_eq!(nodes.len(), 2, "Should have 2 conjunction branches");
    } else {
        panic!("Expected And node");
    }

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind from intersection"
    );
}

/// Port of: `g bibtex extraction 1.3` - pattern structure only
#[test]
fn complex_lookaround_extraction() {
    let pattern = r"(?<=or=(\{|.*\W))(~(.*and.*)&\S[\w-{}\\' ,]+\w)(?=(\W.*|)\},)";
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

/// Port of: BibTeX extraction pattern 1.4
#[test]
fn complex_lookaround_extraction_2() {
    let pattern = r"(?<=or=\{.*)(?<=\W)(~(.*and.*)&[A-Z][\w-{}\\' ,]+)(?=.*\},)(?=\W)";
    let (arena, root) = parse_to_ir(pattern);

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

/// Test word boundary pattern
#[test]
fn word_boundary_pattern() {
    let pattern = r"\ba";
    let (_arena, root) = parse_to_ir(pattern);

    assert_ne!(root, NodeId::NOTHING, "Pattern should parse successfully");
}

/// Port of: `testing anchors 1.1` - pattern structure
#[test]
fn word_boundary_with_literal() {
    let (_arena, root) = parse_to_ir(r"\b11");

    assert_ne!(root, NodeId::NOTHING, "Pattern should parse successfully");
}

// =============================================================================
// URL Pattern Tests
// =============================================================================

/// Port of: `lookback 01` - URL extraction pattern
#[test]
fn url_extraction_pattern() {
    let pattern = r"(?<=([a-zA-Z][a-zA-Z0-9]*)://([^ /]+)(/[^ ]*)?).*";
    let (arena, root) = parse_to_ir(pattern);

    assert_ne!(root, NodeId::NOTHING, "Pattern should parse successfully");

    let flags = arena.flags(root);
    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        ".* at end makes it nullable"
    );
}

// =============================================================================
// Nested Lookaround Tests
// =============================================================================

/// Test nested lookarounds
#[test]
fn nested_lookahead() {
    let pattern = r"(?=(?=a)b)c";
    let (_arena, root) = parse_to_ir(pattern);

    assert_ne!(root, NodeId::NOTHING, "Nested lookahead should parse");
}

/// Test nested lookbehind
#[test]
fn nested_lookbehind() {
    let pattern = r"(?<=(?<=a)b)c";
    let (_arena, root) = parse_to_ir(pattern);

    assert_ne!(root, NodeId::NOTHING, "Nested lookbehind should parse");
}

/// Test mixed nested lookarounds
#[test]
fn mixed_nested_lookarounds() {
    let pattern = r"(?=(?<=a)b)c";
    let (_arena, root) = parse_to_ir(pattern);

    assert_ne!(
        root,
        NodeId::NOTHING,
        "Mixed nested lookarounds should parse"
    );
}

// =============================================================================
// Lookaround with Quantifiers Tests
// =============================================================================

/// Test lookahead with star quantifier
#[test]
fn lookahead_with_star() {
    let pattern = r"(?=.*)a";
    let flags = get_flags(pattern);

    // (?=.*) is always true, so this is just 'a'
    assert!(
        !flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "(?=.*)a should not be nullable"
    );
}

/// Test lookbehind with star quantifier
#[test]
fn lookbehind_with_star() {
    let pattern = r"(?<=.*)a";
    let flags = get_flags(pattern);

    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
}

/// Test optional lookahead
#[test]
fn optional_lookahead() {
    let pattern = r"(?=a)?";
    let flags = get_flags(pattern);

    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "Optional lookahead should be nullable"
    );
}

// =============================================================================
// Lookaround with Alternation Tests
// =============================================================================

/// Test alternation with lookahead
#[test]
fn alternation_with_lookahead() {
    let pattern = r"(a|b)(?=c)";
    let flags = get_flags(pattern);

    assert!(
        flags.contains(NodeFlags::HAS_SUFFIX_LOOKAHEAD),
        "Should have suffix lookahead"
    );
}

/// Test alternation with lookbehind
#[test]
fn alternation_with_lookbehind() {
    let pattern = r"(?<=c)(a|b)";
    let flags = get_flags(pattern);

    assert!(
        flags.contains(NodeFlags::HAS_PREFIX_LOOKBEHIND),
        "Should have prefix lookbehind"
    );
}

/// Test lookahead in alternation
#[test]
fn lookahead_in_alternation() {
    let pattern = r"(?=a)|(?=b)";
    let flags = get_flags(pattern);

    assert!(
        flags.contains(NodeFlags::CONTAINS_LOOKAROUND),
        "Should contain lookaround"
    );
    assert!(
        flags.contains(NodeFlags::CAN_BE_NULLABLE),
        "Both lookaheads are nullable, so alternation is too"
    );
}
