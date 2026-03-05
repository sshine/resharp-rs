//! Comparison tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_07_ComparisonTests.fs`
//!
//! These tests compare RE# matching results with standard regex behavior.
//! Currently, they verify that patterns can be parsed and converted to IR.
//! Full comparison testing will be enabled when the matcher is implemented.

use cstree::syntax::SyntaxNode;
use resharp_parser::{cst_to_ir, parse, RegexOptions};
use resharp_syntax::SyntaxKind;

/// Parse a pattern to IR, returning an error message on failure.
fn parse_to_ir(pattern: &str) -> Result<(), String> {
    let options = RegexOptions::EXPLICIT_CAPTURE | RegexOptions::NON_BACKTRACKING;
    let green = parse(pattern, options).map_err(|e| format!("Parse error: {:?}", e))?;
    let tree: SyntaxNode<SyntaxKind> = SyntaxNode::new_root(green);
    let (_arena, _node_id) = cst_to_ir::<u64>(&tree);
    Ok(())
}

// =============================================================================
// Pattern Compilation Tests
// =============================================================================
// These tests verify that all comparison patterns can be parsed and converted
// to IR without errors.

/// Port of: same as runtime 1: short
#[test]
fn same_as_runtime_1_pattern_compiles() {
    let pattern = r#"^(31|(0[1-9]))$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: same as runtime 3 - complex date pattern
#[test]
fn same_as_runtime_3_pattern_compiles() {
    let pattern = r#"^(((0?[1-9]|[12]\d|3[01])[\.\-\/](0?[13578]|1[02])[\.\-\/]((1[6-9]|[2-9]\d)?\d{2}))|((0?[1-9]|[12]\d|30)[\.\-\/](0?[13456789]|1[012])[\.\-\/]((1[6-9]|[2-9]\d)?\d{2}))|((0?[1-9]|1\d|2[0-8])[\.\-\/]0?2[\.\-\/]((1[6-9]|[2-9]\d)?\d{2}))|(29[\.\-\/]0?2[\.\-\/]((1[6-9]|[2-9]\d)?(0[48]|[2468][048]|[13579][26])|((16|[2468][048]|[3579][26])00)|00)))$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: same as runtime 4
#[test]
fn same_as_runtime_4_pattern_compiles() {
    let pattern = r#"^((0?[13578]\.)|(0?[13456789]\.))$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: same as runtime 5 - postal code pattern
#[test]
fn same_as_runtime_5_pattern_compiles() {
    let pattern = r#"^((\d{5}-\d{4})|(\d{5})|([A-Z]\d[A-Z]\s\d[A-Z]\d))$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: same as runtime 7 - postal code without end anchor
#[test]
fn same_as_runtime_7_pattern_compiles() {
    let pattern = r#"^((\d{5}-\d{4})|(\d{5})|([A-Z]\d[A-Z]\s\d[A-Z]\d))"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: same as runtime 8 - postal code with end anchor
#[test]
fn same_as_runtime_8_pattern_compiles() {
    let pattern = r#"^((\d{5}-\d{4})|(\d{5})|([A-Z]\d[A-Z]\s\d[A-Z]\d))$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: same as runtime 9 - single digit
#[test]
fn same_as_runtime_9_pattern_compiles() {
    let pattern = r#"^\d$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: same as runtime 10 - or nullability test
#[test]
fn same_as_runtime_10_pattern_compiles() {
    let pattern = r#"(\s|\n|^)h:"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: regex with label 3 - named capture
#[test]
fn regex_with_label_3_pattern_compiles() {
    let pattern = r#"(?<Time>^\d)"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: deduplication test - multiline comments
#[test]
fn deduplication_test_pattern_compiles() {
    let pattern = r#"(\/\*(\s*|.*)*\*\/)|(\/\/.*)"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: deduplication test 2 - name pattern
#[test]
fn deduplication_test_2_pattern_compiles() {
    let pattern = r#"^[a-zA-Z]+(([\'\,\.\- ][a-zA-Z ])?[a-zA-Z]*)*$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: simple 1
#[test]
fn simple_1_pattern_compiles() {
    let pattern = "types";
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: multi-nodes ordering comparison
#[test]
fn multi_nodes_ordering_pattern_compiles() {
    let pattern = r#"((\s*|.*)*q/)"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: http address optional
#[test]
fn http_address_optional_pattern_compiles() {
    let pattern = r#"^(ht|f)tp(s?)\:\/\/[a-zA-Z0-9\-\._]+(\.[a-zA-Z0-9\-\._]+){2,}(\/?)([a-zA-Z0-9\-\.\?\,\'\/\\\+\&%\$#_]*)?$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: massive or pattern - date validation
#[test]
fn massive_or_pattern_compiles() {
    let pattern = r#"^((\d{2}((0[13578]|1[02])(0[1-9]|[12]\d|3[01])|(0[13456789]|1[012])(0[1-9]|[12]\d|30)|02(0[1-9]|1\d|2[0-8])))|([02468][048]|[13579][26])0229)$"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: semantics test 1
#[test]
fn semantics_test_1_pattern_compiles() {
    let pattern = r#"(a|ab)*"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Port of: top level duplicate test - phone number
#[test]
fn top_level_duplicate_test_pattern_compiles() {
    let pattern = r#"((\(\d{3}\)?)|(\d{3}))([\s-./]?)(\d{3})([\s-./]?)(\d{4})"#;
    parse_to_ir(pattern).expect("Pattern should compile");
}

// =============================================================================
// Runtime Comparison Tests (Pending Matcher Implementation)
// =============================================================================
// The following tests will compare RE# matching results with standard regex
// behavior once the matcher is implemented.

/// Test that RE# matches the same as .NET runtime for short pattern
#[test]
#[ignore = "Matcher not yet implemented"]
fn same_as_runtime_1_matches() {
    let _pattern = r#"^(31|(0[1-9]))$"#;
    let _input = "31 September";
    // TODO: Compare RE# match result with standard regex
}

/// Test that RE# matches the same as .NET runtime for complex date pattern
#[test]
#[ignore = "Matcher not yet implemented"]
fn same_as_runtime_3_matches() {
    let _pattern = r#"^(((0?[1-9]|[12]\d|3[01])[\.\-\/](0?[13578]|1[02])[\.\-\/]((1[6-9]|[2-9]\d)?\d{2}))|((0?[1-9]|[12]\d|30)[\.\-\/](0?[13456789]|1[012])[\.\-\/]((1[6-9]|[2-9]\d)?\d{2}))|((0?[1-9]|1\d|2[0-8])[\.\-\/]0?2[\.\-\/]((1[6-9]|[2-9]\d)?\d{2}))|(29[\.\-\/]0?2[\.\-\/]((1[6-9]|[2-9]\d)?(0[48]|[2468][048]|[13579][26])|((16|[2468][048]|[3579][26])00)|00)))$"#;
    let _input = "3.4.05";
    // TODO: Compare RE# match result with standard regex
}

/// Test that RE# matches the same as .NET runtime
#[test]
#[ignore = "Matcher not yet implemented"]
fn same_as_runtime_4_matches() {
    let _pattern = r#"^((0?[13578]\.)|(0?[13456789]\.))$"#;
    let _input = "4.";
    // TODO: Compare RE# match result with standard regex
}

/// Test that RE# matches the same as .NET runtime for postal code
#[test]
#[ignore = "Matcher not yet implemented"]
fn same_as_runtime_5_matches() {
    let _pattern = r#"^((\d{5}-\d{4})|(\d{5})|([A-Z]\d[A-Z]\s\d[A-Z]\d))$"#;
    let _input = "44240";
    // TODO: Compare RE# match result with standard regex
}

/// Test that RE# matches the same as .NET runtime for Canadian postal code
#[test]
#[ignore = "Matcher not yet implemented"]
fn same_as_runtime_7_matches() {
    let _pattern = r#"^((\d{5}-\d{4})|(\d{5})|([A-Z]\d[A-Z]\s\d[A-Z]\d))"#;
    let _input = "T2P 3C7";
    // TODO: Compare RE# match result with standard regex
}

/// Test that RE# matches the same as .NET runtime for Canadian postal code with anchor
#[test]
#[ignore = "Matcher not yet implemented"]
fn same_as_runtime_8_matches() {
    let _pattern = r#"^((\d{5}-\d{4})|(\d{5})|([A-Z]\d[A-Z]\s\d[A-Z]\d))$"#;
    let _input = "T2P 3C7";
    // TODO: Compare RE# match result with standard regex
}

/// Test that RE# matches the same as .NET runtime for single digit
#[test]
#[ignore = "Matcher not yet implemented"]
fn same_as_runtime_9_matches() {
    let _pattern = r#"^\d$"#;
    let _input = "24";
    // TODO: Compare RE# match result with standard regex (should NOT match)
}

/// Test that RE# handles or nullability correctly
#[test]
#[ignore = "Matcher not yet implemented"]
fn same_as_runtime_10_matches() {
    let _pattern = r#"(\s|\n|^)h:"#;
    let _input = r#"<a "h:"#;
    // TODO: Compare RE# match result with standard regex
}

/// Test multiline comment pattern matching
#[test]
#[ignore = "Matcher not yet implemented"]
fn deduplication_test_matches() {
    let _pattern = r#"(\/\*(\s*|.*)*\*\/)|(\/\/.*)"#;
    let _input = "/* This is a multi-line comment */";
    // TODO: Verify RE# correctly matches multiline comments
}

/// Test name pattern matching
#[test]
#[ignore = "Matcher not yet implemented"]
fn deduplication_test_2_matches() {
    let _pattern = r#"^[a-zA-Z]+(([\'\,\.\- ][a-zA-Z ])?[a-zA-Z]*)*$"#;
    let _input = "T.F. Johnson";
    // TODO: Verify first match is "T.F. Johnson"
}

/// Test simple literal matching
#[test]
#[ignore = "Matcher not yet implemented"]
fn simple_1_matches() {
    let _pattern = "types";
    let _input = "Lorem Ipsum is simply dummy text of the printing and typesetting industry.Lorem Ipsum has been the Aa11aBaAA standard";
    // TODO: Compare RE# match result with standard regex
}

/// Test http address pattern matching
#[test]
#[ignore = "Matcher not yet implemented"]
fn http_address_optional_matches() {
    let _pattern = r#"^(ht|f)tp(s?)\:\/\/[a-zA-Z0-9\-\._]+(\.[a-zA-Z0-9\-\._]+){2,}(\/?)([a-zA-Z0-9\-\.\?\,\'\/\\\+\&%\$#_]*)?$"#;
    let _input = "http://www.wikipedia.org";
    // TODO: Compare RE# match result with standard regex
}

/// Test massive alternation pattern matching
#[test]
#[ignore = "Matcher not yet implemented"]
fn massive_or_pattern_matches() {
    let _pattern = r#"^((\d{2}((0[13578]|1[02])(0[1-9]|[12]\d|3[01])|(0[13456789]|1[012])(0[1-9]|[12]\d|30)|02(0[1-9]|1\d|2[0-8])))|([02468][048]|[13579][26])0229)$"#;
    let _input = "751231";
    // TODO: Compare RE# match result with standard regex
}

/// Test semantics of alternation with overlap
#[test]
#[ignore = "Matcher not yet implemented"]
fn semantics_test_1_matches() {
    let _pattern = r#"(a|ab)*"#;
    let _input = "abab";
    // TODO: Verify first match is "abab"
}

/// Test phone number pattern matching
#[test]
#[ignore = "Matcher not yet implemented"]
fn top_level_duplicate_test_matches() {
    let _pattern = r#"((\(\d{3}\)?)|(\d{3}))([\s-./]?)(\d{3})([\s-./]?)(\d{4})"#;
    let _input = "1-(212)-123 4567";
    // TODO: Verify first match is "(212)-123 4567"
}
