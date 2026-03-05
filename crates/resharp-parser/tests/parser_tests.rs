//! Parser tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_01_ParserTests.fs`

use cstree::syntax::SyntaxNode;
use resharp_parser::{parse, RegexOptions};
use resharp_syntax::SyntaxKind;

/// Parse a pattern with extended RE# options.
fn parse_extended(pattern: &str) -> SyntaxNode<SyntaxKind> {
    let options = RegexOptions::EXPLICIT_CAPTURE | RegexOptions::NON_BACKTRACKING;
    let green = parse(pattern, options).expect("parse should succeed");
    SyntaxNode::new_root(green)
}

/// Get the first meaningful child of the root (skips Root -> Capture wrapper).
fn get_root_child(tree: &SyntaxNode<SyntaxKind>) -> SyntaxNode<SyntaxKind> {
    // Root -> Capture -> Alternate -> actual content
    tree.children()
        .next()
        .expect("should have Capture")
        .children()
        .next()
        .expect("should have Alternate")
        .children()
        .next()
        .expect("should have child")
        .clone()
}

// =============================================================================
// Conjunction Tests
// =============================================================================

/// Port of: `conjunction parse test 1: c..&..t`
#[test]
fn conjunction_parse_test_1_c_dot_dot_and_dot_dot_t() {
    let tree = parse_extended("c..&..t");
    let node = get_root_child(&tree);

    assert_eq!(
        node.kind(),
        SyntaxKind::Conjunction,
        "should be Conjunction"
    );
    assert_eq!(
        node.children().count(),
        2,
        "Conjunction should have 2 children"
    );

    let children: Vec<_> = node.children().collect();
    assert_eq!(children[0].kind(), SyntaxKind::Concatenate);
    assert_eq!(children[1].kind(), SyntaxKind::Concatenate);
}

/// Port of: `conjunction parse test 2: aa&bb`
#[test]
fn conjunction_parse_test_2_aa_and_bb() {
    let tree = parse_extended("aa&bb");
    let node = get_root_child(&tree);

    assert_eq!(
        node.kind(),
        SyntaxKind::Conjunction,
        "should be Conjunction"
    );
    assert_eq!(
        node.children().count(),
        2,
        "Conjunction should have 2 children"
    );

    let children: Vec<_> = node.children().collect();
    assert_eq!(children[0].kind(), SyntaxKind::Concatenate);
    assert_eq!(children[1].kind(), SyntaxKind::Concatenate);

    // Each Concatenate should contain a Multi
    let first_concat_child = children[0].children().next().expect("should have child");
    let second_concat_child = children[1].children().next().expect("should have child");
    assert_eq!(first_concat_child.kind(), SyntaxKind::Multi);
    assert_eq!(second_concat_child.kind(), SyntaxKind::Multi);
}

/// Port of: `conjunction parse test 3: (ca..)&(...s)`
#[test]
fn conjunction_parse_test_3_groups() {
    let tree = parse_extended("(ca..)&(...s)");
    let node = get_root_child(&tree);

    assert_eq!(
        node.kind(),
        SyntaxKind::Conjunction,
        "should be Conjunction"
    );
    assert_eq!(
        node.children().count(),
        2,
        "Conjunction should have 2 children"
    );

    let children: Vec<_> = node.children().collect();
    assert_eq!(children[0].kind(), SyntaxKind::Concatenate);
    assert_eq!(children[1].kind(), SyntaxKind::Concatenate);
}

/// Port of: `conjunction parse test 4: (?<=\()c` (lookbehind test)
#[test]
fn conjunction_parse_test_4_lookbehind() {
    let tree = parse_extended(r"(?<=\()c");
    let node = get_root_child(&tree);

    // This is actually Conjunction -> Concatenate, and within Concatenate we have the content
    assert_eq!(node.kind(), SyntaxKind::Conjunction);

    let concat = node.children().next().expect("should have Concatenate");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(concat.children().count(), 2, "should have 2 children");

    let children: Vec<_> = concat.children().collect();
    assert_eq!(children[0].kind(), SyntaxKind::PositiveLookbehind);
    assert_eq!(children[1].kind(), SyntaxKind::One);
}

/// Port of: `conjunction parse test 5: aa&bb&cc`
#[test]
fn conjunction_parse_test_5_three_way() {
    let tree = parse_extended("aa&bb&cc");
    let node = get_root_child(&tree);

    assert_eq!(
        node.kind(),
        SyntaxKind::Conjunction,
        "should be Conjunction"
    );
    assert_eq!(
        node.children().count(),
        3,
        "Conjunction should have 3 children"
    );
}

/// Port of: `conjunction parse test 6: (?<=a)b&c`
#[test]
fn conjunction_parse_test_6_lookbehind_conjunction() {
    let tree = parse_extended("(?<=a)b&c");
    let node = get_root_child(&tree);

    assert_eq!(
        node.kind(),
        SyntaxKind::Conjunction,
        "should be Conjunction"
    );
    assert_eq!(
        node.children().count(),
        2,
        "Conjunction should have 2 children"
    );
}

// =============================================================================
// Negation Tests
// =============================================================================

// Note: Negation is tracked via RegexOptions::Negated flag in the original F# implementation.
// The structural tests below verify node kinds and child counts. Full negation flag testing
// requires implementing a negation tracking mechanism.

/// Port of: `negation parse test 01: ~(ab)`
#[test]
fn negation_parse_test_01_group() {
    let tree = parse_extended("~(ab)");
    let node = get_root_child(&tree);

    // Should have a Negation containing a Capture
    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    let negation = concat.children().next().expect("should have negation");
    assert_eq!(negation.kind(), SyntaxKind::Negation);
    // The Negation node contains the Capture
    let capture = negation
        .children()
        .find(|c| c.kind() == SyntaxKind::Capture);
    assert!(capture.is_some(), "Negation should contain Capture");
}

/// Port of: `negation parse test 02: ~ab`
#[test]
fn negation_parse_test_02_single_char() {
    let tree = parse_extended("~ab");
    let node = get_root_child(&tree);

    // Structure: Conjunction -> Concatenate with 2 children
    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(concat.children().count(), 2, "should have 2 children");
}

/// Port of: `negation parse test 03: ~(ab|cd)`
#[test]
fn negation_parse_test_03_alternation_group() {
    let tree = parse_extended("~(ab|cd)");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    let negation = concat.children().next().expect("should have negation");
    assert_eq!(negation.kind(), SyntaxKind::Negation);
    // The Negation node contains the Capture
    let capture = negation
        .children()
        .find(|c| c.kind() == SyntaxKind::Capture);
    assert!(capture.is_some(), "Negation should contain Capture");
}

/// Port of: `negation parse test 04: (ab|~cd)`
#[test]
fn negation_parse_test_04_negated_branch() {
    let tree = parse_extended("(ab|~cd)");
    let node = get_root_child(&tree);

    // Inside the Capture there should be an Alternate with 2 children
    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    let capture = concat.children().next().expect("should have capture");
    assert_eq!(capture.kind(), SyntaxKind::Capture);
}

/// Port of: `negation parse test 05: (ab|~(cd))`
#[test]
fn negation_parse_test_05_negated_group_branch() {
    let tree = parse_extended("(ab|~(cd))");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    let capture = concat.children().next().expect("should have capture");
    assert_eq!(capture.kind(), SyntaxKind::Capture);
}

/// Port of: `negation parse test 06: ab~c`
#[test]
fn negation_parse_test_06_mid_negation() {
    let tree = parse_extended("ab~c");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(
        concat.children().count(),
        2,
        "should have 2 children: Multi + negated One"
    );
}

/// Port of: `negation parse test 07: ab~cd`
#[test]
fn negation_parse_test_07_mid_negation_multi() {
    let tree = parse_extended("ab~cd");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(
        concat.children().count(),
        3,
        "should have 3 children: Multi + negated One + One"
    );
}

/// Port of: `negation parse test 08: ab~c.`
#[test]
fn negation_parse_test_08_mid_negation_dot() {
    let tree = parse_extended("ab~c.");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(concat.children().count(), 3, "should have 3 children");
}

/// Port of: `negation parse test 09: b~(c.)`
#[test]
fn negation_parse_test_09_negated_group() {
    let tree = parse_extended("b~(c.)");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(
        concat.children().count(),
        2,
        "should have 2 children: One + negated Capture"
    );
}

/// Port of: `negation parse test 10: \(~\)`
#[test]
fn negation_parse_test_10_escaped_parens() {
    let tree = parse_extended(r"\(~\)");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(concat.children().count(), 2, "should have 2 children");
}

/// Port of: `negation parse test 11: ~(d)f`
#[test]
fn negation_parse_test_11_negated_then_char() {
    let tree = parse_extended("~(d)f");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(concat.children().count(), 2, "should have 2 children");
}

/// Port of: `negation parse test 12: b(~d|~e|~f)`
#[test]
fn negation_parse_test_12_alternation_of_negated() {
    let tree = parse_extended("b(~d|~e|~f)");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(concat.children().count(), 2, "should have 2 children");

    // Second child should be a Capture containing an Alternate with 3 branches
    let children: Vec<_> = concat.children().collect();
    assert_eq!(children[1].kind(), SyntaxKind::Capture);
}

/// Port of: `negation parse test 13: ~(abc)`
#[test]
fn negation_parse_test_13_negated_multi() {
    let tree = parse_extended("~(abc)");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    let negation = concat.children().next().expect("should have negation");
    assert_eq!(negation.kind(), SyntaxKind::Negation);
    // The Negation node contains the Capture
    let capture = negation
        .children()
        .find(|c| c.kind() == SyntaxKind::Capture);
    assert!(capture.is_some(), "Negation should contain Capture");
}

/// Port of: `negation parse test 14: .*\d.*&~(.*01.*)`
#[test]
fn negation_parse_test_14_conjunction_with_negation() {
    let tree = parse_extended(r".*\d.*&~(.*01.*)");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    assert_eq!(
        node.children().count(),
        2,
        "should have 2 conjunction branches"
    );
}

/// Port of: `negation parse test 15: French~(\n\n)`
#[test]
fn negation_parse_test_15_multi_negation() {
    let tree = parse_extended(r"French~(\n\n)");
    let node = get_root_child(&tree);

    assert_eq!(node.kind(), SyntaxKind::Conjunction);
    let concat = node.children().next().expect("should have concat");
    assert_eq!(concat.kind(), SyntaxKind::Concatenate);
    assert_eq!(
        concat.children().count(),
        2,
        "should have 2 children: Multi + negated Capture"
    );
}
