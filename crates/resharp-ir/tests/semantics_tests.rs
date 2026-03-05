//! Semantics tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_05_SemanticsTests.fs`
//!
//! These tests verify semantic utilities used in the regex engine.

use resharp_ir::minterms_log;

/// Port of: `mintermsLog`
///
/// Tests that the ceiling log2 computation works correctly for determining
/// how many bits are needed to represent n distinct minterm values.
#[test]
fn minterms_log_test() {
    // Helper to verify both implementations give the same result
    fn logceil(n: u32) -> u32 {
        // Alternative implementation using floating point
        if n == 0 {
            return 0;
        }
        let f = n as f64;
        let bits = f.to_bits();
        let exp = ((bits >> 52) & 0x7ff) as i32 - 1022;
        if n.is_power_of_two() {
            (exp - 1) as u32
        } else {
            exp as u32
        }
    }

    // Test that both implementations agree
    let test_eq = |n: u32| {
        assert_eq!(
            minterms_log(n),
            logceil(n),
            "minterms_log({}) should equal logceil({})",
            n,
            n
        );
    };

    // Test cases from the original F# test
    test_eq(7);
    test_eq(8);
    test_eq(15);
    test_eq(16);
    test_eq(31);
    test_eq(32);
    test_eq(33);
}

/// Additional tests for edge cases
#[test]
fn minterms_log_edge_cases() {
    // Zero - special case
    assert_eq!(minterms_log(0), 0);

    // One - needs 0 bits (single value)
    assert_eq!(minterms_log(1), 0);

    // Two - needs 1 bit
    assert_eq!(minterms_log(2), 1);

    // Three - needs 2 bits
    assert_eq!(minterms_log(3), 2);

    // Large values
    assert_eq!(minterms_log(64), 6);
    assert_eq!(minterms_log(65), 7);
    assert_eq!(minterms_log(128), 7);
    assert_eq!(minterms_log(256), 8);
}

/// Verify the function gives correct bit widths for minterm counts
#[test]
fn minterms_log_bit_widths() {
    // For n minterms, we need ceil(log2(n)) bits to represent indices 0..n-1

    // 1 minterm: needs 0 bits (trivial)
    assert_eq!(minterms_log(1), 0);

    // 2 minterms: needs 1 bit (values 0, 1)
    assert_eq!(minterms_log(2), 1);

    // 3-4 minterms: needs 2 bits (values 0-3)
    assert_eq!(minterms_log(3), 2);
    assert_eq!(minterms_log(4), 2);

    // 5-8 minterms: needs 3 bits (values 0-7)
    assert_eq!(minterms_log(5), 3);
    assert_eq!(minterms_log(8), 3);

    // 9-16 minterms: needs 4 bits (values 0-15)
    assert_eq!(minterms_log(9), 4);
    assert_eq!(minterms_log(16), 4);
}
