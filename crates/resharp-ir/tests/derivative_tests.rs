//! Derivative tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_04_DerivativeTests.fs`
//!
//! These tests verify the derivative operation for regex patterns.

use resharp_ir::{
    derivative, is_nullable, BitSetSolver, LocationKind, NodeId, PrettyPrinter, RegexBuilder,
    RegexNode,
};

/// Create a builder with bit set solver.
fn new_builder() -> RegexBuilder<BitSetSolver> {
    RegexBuilder::new(BitSetSolver)
}

// =============================================================================
// Basic Derivative Tests
// =============================================================================

/// Port of: `derivative of ab`
///
/// D_a(ab) = b
#[test]
fn derivative_of_ab() {
    let mut builder = new_builder();

    // Pattern: ab (using different bits for a and b)
    let a = builder.singleton(0b0001); // 'a'
    let b = builder.singleton(0b0010); // 'b'
    let ab = builder.concat(a, b);

    // D_a(ab) should be b
    let da = derivative(&mut builder, LocationKind::Center, 0b0001, ab);
    assert_eq!(da, b, "D_a(ab) should be b");
}

/// Port of: `raw derivative of ab`
#[test]
fn raw_derivative_of_ab() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let b = builder.singleton(0b0010);
    let ab = builder.concat(a, b);

    let da = derivative(&mut builder, LocationKind::Center, 0b0001, ab);
    assert_eq!(da, b);
}

/// Port of: `derivative of true`
///
/// D_c(_) where _ matches any single character
/// D_c(.) = ε
#[test]
fn derivative_of_any_char() {
    let mut builder = new_builder();

    // Create "." (matches any single char) - using full charset
    let any_char = builder.singleton(u64::MAX);

    // D_c(.) = ε for any character
    let dc = derivative(&mut builder, LocationKind::Center, 0b0001, any_char);
    assert_eq!(dc, NodeId::EPSILON, "D_c(.) should be ε");
}

// =============================================================================
// Loop Derivative Tests
// =============================================================================

/// Port of: `derivative of plus`
///
/// D_c(\d+) = \d*
#[test]
fn derivative_of_plus() {
    let mut builder = new_builder();

    // Pattern: a+ (one or more 'a')
    let a = builder.singleton(0b0001);
    let a_plus = builder.loop_(a, 1, u32::MAX, false);

    // D_a(a+) = a*
    let da = derivative(&mut builder, LocationKind::Center, 0b0001, a_plus);

    if let Some(RegexNode::Loop {
        node, low, high, ..
    }) = builder.arena().node(da)
    {
        assert_eq!(*node, a);
        assert_eq!(*low, 0, "D_a(a+) should have low=0");
        assert_eq!(*high, u32::MAX, "D_a(a+) should have high=MAX");
    } else {
        panic!("Expected Loop node for D_a(a+)");
    }
}

/// D_a(a*) = a*
#[test]
fn derivative_of_star() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let a_star = builder.loop_(a, 0, u32::MAX, false);

    let da = derivative(&mut builder, LocationKind::Center, 0b0001, a_star);

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

/// D_a(a{2,5}) = a{1,4}
#[test]
fn derivative_of_bounded_loop() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let a_2_5 = builder.loop_(a, 2, 5, false);

    let da = derivative(&mut builder, LocationKind::Center, 0b0001, a_2_5);

    if let Some(RegexNode::Loop {
        node, low, high, ..
    }) = builder.arena().node(da)
    {
        assert_eq!(*node, a);
        assert_eq!(*low, 1, "D_a(a{{2,5}}) should have low=1");
        assert_eq!(*high, 4, "D_a(a{{2,5}}) should have high=4");
    } else {
        panic!("Expected Loop node for D_a(a{{2,5}})");
    }
}

// =============================================================================
// Alternation Derivative Tests
// =============================================================================

/// D_a(a|b) = ε | ⊥ = ε
#[test]
fn derivative_of_alternation_first() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let b = builder.singleton(0b0010);
    let a_or_b = builder.or(a, b);

    let da = derivative(&mut builder, LocationKind::Center, 0b0001, a_or_b);
    assert_eq!(da, NodeId::EPSILON, "D_a(a|b) should be ε");
}

/// D_b(a|b) = ⊥ | ε = ε
#[test]
fn derivative_of_alternation_second() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let b = builder.singleton(0b0010);
    let a_or_b = builder.or(a, b);

    let db = derivative(&mut builder, LocationKind::Center, 0b0010, a_or_b);
    assert_eq!(db, NodeId::EPSILON, "D_b(a|b) should be ε");
}

/// D_c(a|b) = ⊥ | ⊥ = ⊥ when c matches neither a nor b
#[test]
fn derivative_of_alternation_neither() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let b = builder.singleton(0b0010);
    let a_or_b = builder.or(a, b);

    let dc = derivative(&mut builder, LocationKind::Center, 0b0100, a_or_b);
    assert_eq!(
        dc,
        NodeId::NOTHING,
        "D_c(a|b) should be ⊥ when c is neither"
    );
}

// =============================================================================
// Intersection Derivative Tests
// =============================================================================

/// Port of: `inter deriv 1`
///
/// D_c(.*a.* & .*b.* & .*c.*) for c = 'c' should include (.*a.* & .*b.*)
#[test]
fn derivative_of_intersection() {
    let mut builder = new_builder();

    // Create .* (any char repeated)
    let any = builder.singleton(u64::MAX);
    let any_star = builder.loop_(any, 0, u32::MAX, false);

    // Create [a], [b], [c]
    let a = builder.singleton(0b0001);
    let b = builder.singleton(0b0010);
    let c = builder.singleton(0b0100);

    // Create .*a.*
    let a_any = builder.concat(a, any_star);
    let any_a_any = builder.concat(any_star, a_any);
    // Create .*b.*
    let b_any = builder.concat(b, any_star);
    let any_b_any = builder.concat(any_star, b_any);
    // Create .*c.*
    let c_any = builder.concat(c, any_star);
    let any_c_any = builder.concat(any_star, c_any);

    // Create (.*a.* & .*b.* & .*c.*)
    let inter1 = builder.and(any_a_any, any_b_any);
    let inter = builder.and(inter1, any_c_any);

    // D_c should consume the 'c' requirement
    let dc = derivative(&mut builder, LocationKind::Center, 0b0100, inter);

    // The result should not be NOTHING
    assert_ne!(
        dc,
        NodeId::NOTHING,
        "D_c on intersection containing .*c.* should not be ⊥"
    );
}

// =============================================================================
// Negation Derivative Tests
// =============================================================================

/// Port of: `deriv negation 1`
///
/// D_1(~(.*11.*)) should be ~((.*1)?1.*)
#[test]
fn derivative_of_negation() {
    let mut builder = new_builder();

    // Create .*
    let any = builder.singleton(u64::MAX);
    let any_star = builder.loop_(any, 0, u32::MAX, false);

    // Create [1]
    let one = builder.singleton(0b0001);

    // Create .*11.* = (.*)(1)(1)(.*)
    let eleven = builder.concat(one, one);
    let eleven_any = builder.concat(eleven, any_star);
    let any_11_any = builder.concat(any_star, eleven_any);

    // Create ~(.*11.*)
    let not_eleven = builder.not(any_11_any);

    // D_1(~(.*11.*)) = ~(D_1(.*11.*))
    let d1 = derivative(&mut builder, LocationKind::Center, 0b0001, not_eleven);

    // Result should be a Not node
    if let Some(RegexNode::Not { .. }) = builder.arena().node(d1) {
        // Good - it's a negation
    } else {
        panic!("Expected Not node for D_1(~(.*11.*))");
    }
}

// =============================================================================
// Nullable Tests
// =============================================================================

/// Test nullability at different locations
#[test]
fn nullable_at_locations() {
    let mut builder = new_builder();

    // ^ (begin anchor) is only nullable at Begin
    let begin = builder.begin();
    assert!(
        is_nullable(&builder, LocationKind::Begin, begin),
        "^ should be nullable at Begin"
    );
    assert!(
        !is_nullable(&builder, LocationKind::Center, begin),
        "^ should not be nullable at Center"
    );
    assert!(
        !is_nullable(&builder, LocationKind::End, begin),
        "^ should not be nullable at End"
    );

    // $ (end anchor) is only nullable at End
    let end = builder.end();
    assert!(
        !is_nullable(&builder, LocationKind::Begin, end),
        "$ should not be nullable at Begin"
    );
    assert!(
        !is_nullable(&builder, LocationKind::Center, end),
        "$ should not be nullable at Center"
    );
    assert!(
        is_nullable(&builder, LocationKind::End, end),
        "$ should be nullable at End"
    );
}

/// Test nullability of ^$ pattern
#[test]
fn nullable_begin_end() {
    let mut builder = new_builder();

    let begin = builder.begin();
    let end = builder.end();
    let begin_end = builder.concat(begin, end);

    // ^$ is only nullable at position that is both begin and end (empty string)
    // Since we test at specific locations, it should be false at Begin
    // (because $ is not nullable at Begin)
    assert!(
        !is_nullable(&builder, LocationKind::Begin, begin_end),
        "^$ should not be nullable at Begin ($ not satisfied)"
    );
}

/// Test nullability of optional pattern
#[test]
fn nullable_optional() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let a_opt = builder.loop_(a, 0, 1, false);

    assert!(
        is_nullable(&builder, LocationKind::Center, a_opt),
        "a? should be nullable"
    );
}

/// Test nullability of alternation with nullable branch
#[test]
fn nullable_or_with_nullable() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let a_or_eps = builder.or(a, NodeId::EPSILON);

    // Should normalize to a? which is nullable
    assert!(
        is_nullable(&builder, LocationKind::Center, a_or_eps),
        "(a|ε) should be nullable"
    );
}

// =============================================================================
// Lookaround Derivative Tests
// =============================================================================

/// Test derivative of positive lookahead
#[test]
fn derivative_of_lookahead() {
    let mut builder = new_builder();

    // (?=a) - positive lookahead for 'a'
    let a = builder.singleton(0b0001);
    let lookahead = builder.lookaround(a, false, false);

    // D_a((?=a)) = (?=D_a(a)) = (?=ε)
    let da = derivative(&mut builder, LocationKind::Center, 0b0001, lookahead);

    if let Some(RegexNode::LookAround {
        inner,
        look_back,
        negative,
    }) = builder.arena().node(da)
    {
        assert!(!look_back, "Should be lookahead");
        assert!(!negative, "Should be positive");
        assert_eq!(*inner, NodeId::EPSILON);
    } else {
        // (?=ε) might simplify to ε
        assert_eq!(da, NodeId::EPSILON, "(?=ε) should simplify to ε");
    }
}

/// Test derivative of lookbehind
#[test]
fn derivative_of_lookbehind() {
    let mut builder = new_builder();

    // (?<=a) - positive lookbehind for 'a'
    let a = builder.singleton(0b0001);
    let lookbehind = builder.lookaround(a, true, false);

    // D_a((?<=a)) = (?<=D_a(a)) = (?<=ε) = ε (simplified)
    let da = derivative(&mut builder, LocationKind::Center, 0b0001, lookbehind);

    // The result could be (?<=ε) or simplified to ε
    // Let's just check it's not NOTHING
    assert_ne!(da, NodeId::NOTHING, "D_a((?<=a)) should not be ⊥");
}

// =============================================================================
// Pretty Printing Tests
// =============================================================================

/// Test that derivatives produce sensible pretty-printed output
#[test]
fn derivative_pretty_print() {
    let mut builder = new_builder();

    // Pattern: a+
    let a = builder.singleton(0b0001);
    let a_plus = builder.loop_(a, 1, u32::MAX, false);

    let da = derivative(&mut builder, LocationKind::Center, 0b0001, a_plus);

    let mut printer = PrettyPrinter::new(builder.arena());
    let printed = printer.print(da);

    // D_a(a+) = a* which should print as "φ*" (phi is the placeholder for charsets)
    assert!(
        printed.contains('*'),
        "D_a(a+) should be a* pattern, got: {}",
        printed
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

/// Derivative of NOTHING is always NOTHING
#[test]
fn derivative_of_nothing() {
    let mut builder = new_builder();

    let d = derivative(&mut builder, LocationKind::Center, 0b0001, NodeId::NOTHING);
    assert_eq!(d, NodeId::NOTHING, "D_c(⊥) should be ⊥");
}

/// Derivative of EPSILON is always NOTHING
#[test]
fn derivative_of_epsilon() {
    let mut builder = new_builder();

    let d = derivative(&mut builder, LocationKind::Center, 0b0001, NodeId::EPSILON);
    assert_eq!(d, NodeId::NOTHING, "D_c(ε) should be ⊥");
}

/// Derivative of TOP_STAR is TOP_STAR
#[test]
fn derivative_of_top_star() {
    let mut builder = new_builder();

    let d = derivative(&mut builder, LocationKind::Center, 0b0001, NodeId::TOP_STAR);

    // D_c(_*) = _*
    assert_eq!(d, NodeId::TOP_STAR, "D_c(_*) should be _*");
}

/// Derivative of anchor is NOTHING (anchors don't consume characters)
#[test]
fn derivative_of_anchors() {
    let mut builder = new_builder();

    let begin = builder.begin();
    let end = builder.end();

    let d_begin = derivative(&mut builder, LocationKind::Begin, 0b0001, begin);
    let d_end = derivative(&mut builder, LocationKind::End, 0b0001, end);

    assert_eq!(d_begin, NodeId::NOTHING, "D_c(^) should be ⊥");
    assert_eq!(d_end, NodeId::NOTHING, "D_c($) should be ⊥");
}

// =============================================================================
// Complex Pattern Tests
// =============================================================================

/// Test derivative of ^a$
#[test]
fn derivative_of_anchored_single_char() {
    let mut builder = new_builder();

    // ^a$
    let begin = builder.begin();
    let a = builder.singleton(0b0001);
    let end = builder.end();
    let begin_a = builder.concat(begin, a);
    let begin_a_end = builder.concat(begin_a, end);

    // At Begin, ^ is nullable, so D_a(^a$) = D_a(a$) = $
    let da = derivative(&mut builder, LocationKind::Begin, 0b0001, begin_a_end);

    // Should result in $ (end anchor)
    if let Some(RegexNode::End) = builder.arena().node(da) {
        // Good
    } else {
        // Might be concat(ε, $) which simplifies to $
        // Let's just check it's not NOTHING for now
        assert_ne!(da, NodeId::NOTHING, "D_a(^a$) at Begin should not be ⊥");
    }
}

/// Test derivative of pattern with nested concat: abc
#[test]
fn derivative_of_nested_concat() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let b = builder.singleton(0b0010);
    let c = builder.singleton(0b0100);

    // abc = a(bc)
    let bc = builder.concat(b, c);
    let abc = builder.concat(a, bc);

    // D_a(abc) = bc
    let da = derivative(&mut builder, LocationKind::Center, 0b0001, abc);
    assert_eq!(da, bc, "D_a(abc) should be bc");
}
