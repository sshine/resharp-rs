//! Character set solver for subsumption checking.
//!
//! The solver provides operations on character sets (minterms) that enable
//! subsumption detection during pattern normalization.

/// Trait for character set algebra.
///
/// This trait defines the operations needed for character set manipulation
/// and subsumption checking. Character sets are typically represented as
/// bitmasks or more sophisticated structures.
pub trait CharSetSolver: Clone {
    /// The character set type.
    type CharSet: Clone + PartialEq + Default;

    /// The empty character set (matches nothing).
    fn empty(&self) -> Self::CharSet;

    /// The full character set (matches everything).
    fn full(&self) -> Self::CharSet;

    /// Check if a character set is empty.
    fn is_empty(&self, set: &Self::CharSet) -> bool;

    /// Check if a character set is full (matches all characters).
    fn is_full(&self, set: &Self::CharSet) -> bool;

    /// Compute the intersection of two character sets.
    fn and(&self, a: &Self::CharSet, b: &Self::CharSet) -> Self::CharSet;

    /// Compute the union of two character sets.
    fn or(&self, a: &Self::CharSet, b: &Self::CharSet) -> Self::CharSet;

    /// Compute the complement of a character set.
    fn not(&self, set: &Self::CharSet) -> Self::CharSet;

    /// Check if `larger` contains (subsumes) `smaller`.
    ///
    /// This is true if `smaller ∩ larger == smaller`, meaning all characters
    /// matched by `smaller` are also matched by `larger`.
    fn contains(&self, larger: &Self::CharSet, smaller: &Self::CharSet) -> bool {
        let intersection = self.and(smaller, larger);
        &intersection == smaller
    }
}

/// A simple 64-bit character set solver.
///
/// This uses a 64-bit bitmask to represent character sets, which is efficient
/// but limited to 64 distinct character classes (minterms).
#[derive(Debug, Clone, Default)]
pub struct BitSetSolver;

impl CharSetSolver for BitSetSolver {
    type CharSet = u64;

    fn empty(&self) -> u64 {
        0
    }

    fn full(&self) -> u64 {
        u64::MAX
    }

    fn is_empty(&self, set: &u64) -> bool {
        *set == 0
    }

    fn is_full(&self, set: &u64) -> bool {
        *set == u64::MAX
    }

    fn and(&self, a: &u64, b: &u64) -> u64 {
        a & b
    }

    fn or(&self, a: &u64, b: &u64) -> u64 {
        a | b
    }

    fn not(&self, set: &u64) -> u64 {
        !set
    }
}

/// Compute the ceiling of log2(n), i.e., how many bits are needed to represent n distinct values.
///
/// This is used to determine the bit width needed for minterm representation.
///
/// # Examples
/// - `minterms_log(7)` = 3 (need 3 bits for 7 values: 0-6)
/// - `minterms_log(8)` = 3 (need 3 bits for 8 values: 0-7)
/// - `minterms_log(9)` = 4 (need 4 bits for 9 values: 0-8)
#[inline]
pub fn minterms_log(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    if n.is_power_of_two() {
        n.trailing_zeros()
    } else {
        u32::BITS - n.leading_zeros()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minterms_log() {
        // Powers of 2: log2(n) bits needed
        assert_eq!(minterms_log(1), 0); // 1 value needs 0 bits (just one state)
        assert_eq!(minterms_log(2), 1); // 2 values need 1 bit
        assert_eq!(minterms_log(4), 2); // 4 values need 2 bits
        assert_eq!(minterms_log(8), 3); // 8 values need 3 bits
        assert_eq!(minterms_log(16), 4); // 16 values need 4 bits
        assert_eq!(minterms_log(32), 5); // 32 values need 5 bits

        // Non-powers of 2: ceil(log2(n)) bits needed
        assert_eq!(minterms_log(7), 3); // 7 values need 3 bits
        assert_eq!(minterms_log(15), 4); // 15 values need 4 bits
        assert_eq!(minterms_log(31), 5); // 31 values need 5 bits
        assert_eq!(minterms_log(33), 6); // 33 values need 6 bits
    }

    #[test]
    fn test_bitset_solver() {
        let solver = BitSetSolver;

        assert!(solver.is_empty(&solver.empty()));
        assert!(solver.is_full(&solver.full()));

        let a = 0b1100u64;
        let b = 0b1010u64;

        assert_eq!(solver.and(&a, &b), 0b1000);
        assert_eq!(solver.or(&a, &b), 0b1110);

        // b is not contained in a (b has bit 1 which a doesn't)
        assert!(!solver.contains(&a, &b));

        // 0b1000 is contained in a
        assert!(solver.contains(&a, &0b1000));

        // 0b0100 is contained in a
        assert!(solver.contains(&a, &0b0100));
    }

    #[test]
    fn test_containment() {
        let solver = BitSetSolver;

        // Full contains everything
        assert!(solver.contains(&solver.full(), &0b1111));

        // Everything contains empty
        assert!(solver.contains(&0b1111, &solver.empty()));

        // Self-containment
        let set = 0b1010u64;
        assert!(solver.contains(&set, &set));
    }
}
