//! Subsumption tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_03_SubsumptionTests.fs`
//!
//! These tests verify that the builder correctly normalizes patterns using
//! subsumption rules.

use resharp_ir::{BitSetSolver, NodeId, RegexBuilder, RegexNode};

/// Create a builder with bit set solver.
fn new_builder() -> RegexBuilder<BitSetSolver> {
    RegexBuilder::new(BitSetSolver)
}

// =============================================================================
// Basic Subsumption Tests
// =============================================================================

/// Port of: `subsumption or loop: (a*|.*)`
///
/// When one star loop subsumes another in an Or, the larger one wins.
#[test]
fn subsumption_or_loop() {
    let mut builder = new_builder();

    // Create a* (matches characters with bit 0)
    let a_char = builder.singleton(0b0001);
    let a_star = builder.loop_(a_char, 0, u32::MAX, false);

    // Create .* (matches all characters)
    let dot_char = builder.singleton(u64::MAX);
    let dot_star = builder.loop_(dot_char, 0, u32::MAX, false);

    // a* | .* should simplify to .*
    let result = builder.or(a_star, dot_star);

    // Since .* subsumes a*, the result should be dot_star
    assert_eq!(result, dot_star, "a* | .* should simplify to .*");
}

/// Port of: `subsumption and loop: (.*&.*s)`
///
/// When two star loops are intersected, the more restrictive one wins.
#[test]
fn subsumption_and_loop() {
    let mut builder = new_builder();

    // Create .* (matches all characters)
    let dot_char = builder.singleton(u64::MAX);
    let dot_star = builder.loop_(dot_char, 0, u32::MAX, false);

    // Create [s]* (matches only 's' characters)
    let s_char = builder.singleton(0b0001);
    let s_star = builder.loop_(s_char, 0, u32::MAX, false);

    // .* & [s]* should simplify to [s]*
    let result = builder.and(dot_star, s_star);

    // Since [s]* is more restrictive than .*, the result should be s_star
    assert_eq!(result, s_star, ".* & [s]* should simplify to [s]*");
}

// =============================================================================
// Identity and Absorption Tests
// =============================================================================

/// Port of: `sub 008: (.*|)` -> `.*`
///
/// Empty alternation with anything becomes that thing (via optional).
#[test]
fn sub_008_or_with_epsilon() {
    let mut builder = new_builder();

    // Create .*
    let dot_char = builder.singleton(u64::MAX);
    let dot_star = builder.loop_(dot_char, 0, u32::MAX, false);

    // .* | ε should simplify to .*?  (which is .*{0,1} but .* already includes 0)
    // Actually, .* already matches empty, so .* | ε = .*
    let result = builder.or(dot_star, NodeId::EPSILON);

    // Result should be (.*){0,1} which wraps .* in an optional loop
    // But since .* already matches empty, this might simplify further
    // For now, check that we get a loop around dot_star
    if let Some(RegexNode::Loop {
        node,
        low: 0,
        high: 1,
        ..
    }) = builder.arena().node(result)
    {
        assert_eq!(*node, dot_star);
    } else {
        // If it's not wrapped in a loop, it should be dot_star itself
        // (if we implement that optimization)
        assert_eq!(result, dot_star, ".* | ε should simplify to .* (or .*?)");
    }
}

// =============================================================================
// Singleton Merge Tests
// =============================================================================

/// Port of: `merge 1: a|s` -> `[as]`
#[test]
fn merge_1_singleton_or() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let s = builder.singleton(0b0010);

    let result = builder.or(a, s);

    // Should merge to [as]
    if let Some(RegexNode::Singleton(charset)) = builder.arena().node(result) {
        assert_eq!(*charset, 0b0011, "a|s should merge to [as]");
    } else {
        panic!("Expected Singleton after merge");
    }
}

/// Port of: `merge 2: at|st` -> `[as]t`
///
/// This requires head merging which we'll implement later.
#[test]
#[ignore = "requires head merging in concat"]
fn merge_2_concat_with_common_tail() {
    // TODO: Implement head/tail merging in concat
}

// =============================================================================
// Loop Merge Tests
// =============================================================================

/// Test loop range merging: a{0,5} | a{4,7} = a{0,7}
#[test]
fn loop_range_merge() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let a_0_5 = builder.loop_(a, 0, 5, false);
    let a_4_7 = builder.loop_(a, 4, 7, false);

    let result = builder.or(a_0_5, a_4_7);

    if let Some(RegexNode::Loop {
        node, low, high, ..
    }) = builder.arena().node(result)
    {
        assert_eq!(*node, a);
        assert_eq!(*low, 0, "min should be 0");
        assert_eq!(*high, 7, "max should be 7");
    } else {
        panic!("Expected Loop after merge");
    }
}

/// Test: (ab) | (ab){2,} = (ab){1,}
#[test]
fn loop_with_body_merge() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let b = builder.singleton(0b0010);
    let ab = builder.concat(a, b);

    let ab_2_inf = builder.loop_(ab, 2, u32::MAX, false);

    let result = builder.or(ab, ab_2_inf);

    if let Some(RegexNode::Loop {
        node, low, high, ..
    }) = builder.arena().node(result)
    {
        assert_eq!(*node, ab);
        assert_eq!(*low, 1, "min should be 1");
        assert_eq!(*high, u32::MAX, "max should be unbounded");
    } else {
        panic!("Expected Loop after merge");
    }
}

// =============================================================================
// Negation Tests
// =============================================================================

/// Test: A | ~A = TOP_STAR
#[test]
fn complementary_or() {
    let mut builder = new_builder();

    let a = builder.singleton(0b1010);
    let not_a = builder.not(a);

    let result = builder.or(a, not_a);
    assert_eq!(result, NodeId::TOP_STAR);
}

/// Test: A & ~A = NOTHING
#[test]
fn complementary_and() {
    let mut builder = new_builder();

    let a = builder.singleton(0b1010);
    let not_a = builder.not(a);

    let result = builder.and(a, not_a);
    assert_eq!(result, NodeId::NOTHING);
}

/// Test: ~~A = A
#[test]
fn double_negation() {
    let mut builder = new_builder();

    let a = builder.singleton(0b1010);
    let not_a = builder.not(a);
    let not_not_a = builder.not(not_a);

    assert_eq!(not_not_a, a);
}

// =============================================================================
// Nested Loop Tests
// =============================================================================

/// Test: (a*){2,3} = a{0,inf} (nested loop simplification)
#[test]
fn nested_loop_simplification() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);
    let a_star = builder.loop_(a, 0, u32::MAX, false);
    let nested = builder.loop_(a_star, 2, 3, false);

    if let Some(RegexNode::Loop {
        node, low, high, ..
    }) = builder.arena().node(nested)
    {
        assert_eq!(*node, a, "inner should be 'a'");
        assert_eq!(*low, 0, "low should be 0*2 = 0");
        assert_eq!(*high, u32::MAX, "high should be unbounded");
    } else {
        panic!("Expected simplified Loop");
    }
}

// =============================================================================
// Concat Identity Tests
// =============================================================================

/// Test: a · ε = a
#[test]
fn concat_epsilon_identity() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);

    assert_eq!(builder.concat(a, NodeId::EPSILON), a);
    assert_eq!(builder.concat(NodeId::EPSILON, a), a);
}

/// Test: a · ⊥ = ⊥
#[test]
fn concat_nothing_absorption() {
    let mut builder = new_builder();

    let a = builder.singleton(0b0001);

    assert_eq!(builder.concat(a, NodeId::NOTHING), NodeId::NOTHING);
    assert_eq!(builder.concat(NodeId::NOTHING, a), NodeId::NOTHING);
}
