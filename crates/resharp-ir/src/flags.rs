//! Node flags for regex pattern analysis.

use bitflags::bitflags;

bitflags! {
    /// Flags describing properties of a regex node.
    ///
    /// These flags are computed during IR construction and used for
    /// optimization and analysis.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NodeFlags: u8 {
        /// The pattern can match the empty string (in some contexts).
        const CAN_BE_NULLABLE = 0x01;

        /// The pattern always matches the empty string (in all contexts).
        const IS_ALWAYS_NULLABLE = 0x02;

        /// The pattern contains a lookaround assertion.
        const CONTAINS_LOOKAROUND = 0x04;

        /// The pattern's nullability depends on anchor position.
        const DEPENDS_ON_ANCHOR = 0x08;

        /// The pattern has a lookahead at its suffix.
        const HAS_SUFFIX_LOOKAHEAD = 0x10;

        /// The pattern has a lookbehind at its prefix.
        const HAS_PREFIX_LOOKBEHIND = 0x20;
    }
}

impl NodeFlags {
    /// Check if the pattern is always nullable.
    pub fn is_always_nullable(self) -> bool {
        self.contains(Self::IS_ALWAYS_NULLABLE)
    }

    /// Check if the pattern can be nullable.
    pub fn can_be_nullable(self) -> bool {
        self.contains(Self::CAN_BE_NULLABLE)
    }

    /// Check if the pattern contains lookaround.
    pub fn contains_lookaround(self) -> bool {
        self.contains(Self::CONTAINS_LOOKAROUND)
    }

    /// Check if nullability depends on anchor position.
    pub fn depends_on_anchor(self) -> bool {
        self.contains(Self::DEPENDS_ON_ANCHOR)
    }

    /// Check if the pattern has a suffix lookahead.
    pub fn has_suffix_lookahead(self) -> bool {
        self.contains(Self::HAS_SUFFIX_LOOKAHEAD)
    }

    /// Check if the pattern has a prefix lookbehind.
    pub fn has_prefix_lookbehind(self) -> bool {
        self.contains(Self::HAS_PREFIX_LOOKBEHIND)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_combinations() {
        let nullable = NodeFlags::CAN_BE_NULLABLE | NodeFlags::IS_ALWAYS_NULLABLE;
        assert!(nullable.is_always_nullable());
        assert!(nullable.can_be_nullable());
        assert!(!nullable.depends_on_anchor());
    }

    #[test]
    fn test_flag_helpers() {
        let flags = NodeFlags::DEPENDS_ON_ANCHOR | NodeFlags::HAS_SUFFIX_LOOKAHEAD;
        assert!(flags.depends_on_anchor());
        assert!(flags.has_suffix_lookahead());
        assert!(!flags.has_prefix_lookbehind());
    }
}
