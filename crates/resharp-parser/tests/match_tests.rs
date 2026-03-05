//! Match tests ported from F# test suite.
//!
//! Original: `resharp-dotnet/src/Resharp.Test/_06_MatchTests.fs`
//!
//! These tests verify that patterns from the TOML test files can be parsed
//! and converted to IR. Full matching functionality will be tested when
//! the matcher is implemented.

use cstree::syntax::SyntaxNode;
use resharp_ir::PrettyPrinter;
use resharp_parser::{cst_to_ir, parse, RegexOptions};
use resharp_syntax::SyntaxKind;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// A single test case from the TOML file.
#[derive(Deserialize)]
struct TestCase {
    pattern: String,
    input: String,
    #[serde(default)]
    matches: Vec<[i64; 2]>,
    #[serde(default, rename = "end_positions")]
    _end_positions: Vec<i64>,
    #[serde(default, rename = "nullable_positions")]
    _nullable_positions: Vec<i64>,
}

/// The root structure of a test TOML file.
#[derive(Deserialize)]
struct TestFile {
    #[serde(default, rename = "description")]
    _description: String,
    #[serde(default)]
    test: Vec<TestCase>,
}

/// Get the path to a test data file.
fn test_data_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data");
    path.push(name);
    path
}

/// Parse a TOML test file.
fn parse_test_file(name: &str) -> TestFile {
    let path = test_data_path(name);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    toml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

/// Parse a pattern to IR.
fn parse_to_ir(pattern: &str) -> Result<(), String> {
    let options = RegexOptions::EXPLICIT_CAPTURE | RegexOptions::NON_BACKTRACKING;
    let green = parse(pattern, options).map_err(|e| format!("Parse error: {:?}", e))?;
    let tree: SyntaxNode<SyntaxKind> = SyntaxNode::new_root(green);
    let (arena, node_id) = cst_to_ir::<u64>(&tree);

    // Try to print it (exercises more code paths)
    let mut printer = PrettyPrinter::new(&arena);
    let _printed = printer.print(node_id);

    Ok(())
}

/// Test that all patterns in a TOML file can be parsed and converted to IR.
fn test_patterns_compile(name: &str) {
    let test_file = parse_test_file(name);
    let mut failed = Vec::new();

    for (i, test) in test_file.test.iter().enumerate() {
        if let Err(e) = parse_to_ir(&test.pattern) {
            failed.push((i, test.pattern.clone(), e));
        }
    }

    if !failed.is_empty() {
        let failures: Vec<String> = failed
            .iter()
            .map(|(i, p, e)| format!("  [{}] {} - {}", i, p, e))
            .collect();
        panic!(
            "Failed to compile {} pattern(s) from {}:\n{}",
            failed.len(),
            name,
            failures.join("\n")
        );
    }
}

// =============================================================================
// Pattern Compilation Tests
// =============================================================================
// These tests verify that all patterns from the test files can be parsed
// and converted to IR without errors.

/// Port of: tests01 - various llmatch tests
#[test]
fn tests01_patterns_compile() {
    test_patterns_compile("tests01.toml");
}

/// Port of: tests02_lookaround - lookaround pattern tests
#[test]
fn tests02_lookaround_patterns_compile() {
    test_patterns_compile("tests02_lookaround.toml");
}

/// Port of: tests03_boolean - complement (~) and intersection (&) tests
#[test]
fn tests03_boolean_patterns_compile() {
    test_patterns_compile("tests03_boolean.toml");
}

/// Port of: tests04_anchors - anchor pattern tests
#[test]
fn tests04_anchors_patterns_compile() {
    test_patterns_compile("tests04_anchors.toml");
}

/// Port of: tests05_match_end - match end position tests
#[test]
fn tests05_match_end_patterns_compile() {
    test_patterns_compile("tests05_match_end.toml");
}

/// Port of: tests06_nullable_positions - nullable position tests
#[test]
fn tests06_nullable_positions_patterns_compile() {
    test_patterns_compile("tests06_nullable_positions.toml");
}

/// Port of: tests07_unsupported - unsupported pattern tests
/// These patterns are expected to fail during matching, but should still parse.
#[test]
fn tests07_unsupported_patterns_compile() {
    test_patterns_compile("tests07_unsupported.toml");
}

/// Port of: tests08_semantics - semantic tests
#[test]
fn tests08_semantics_patterns_compile() {
    test_patterns_compile("tests08_semantics.toml");
}

// =============================================================================
// Match Result Tests (Pending Matcher Implementation)
// =============================================================================
// The following tests will verify actual match results once the matcher
// is implemented. For now, they are ignored.

/// Placeholder for full matching tests.
/// Will be enabled when the matcher is implemented.
#[test]
#[ignore = "Matcher not yet implemented"]
fn tests01_matches() {
    let test_file = parse_test_file("tests01.toml");

    for test in test_file.test.iter() {
        // TODO: When matcher is implemented:
        // let regex = Regex::new(&test.pattern).unwrap();
        // let matches = regex.find_all(&test.input);
        // assert_eq!(matches, test.matches);
        let _ = (&test.input, &test.matches);
    }
}

#[test]
#[ignore = "Matcher not yet implemented"]
fn tests02_lookaround_matches() {
    let _test_file = parse_test_file("tests02_lookaround.toml");
    // TODO: Implement when matcher is ready
}

#[test]
#[ignore = "Matcher not yet implemented"]
fn tests03_boolean_matches() {
    let _test_file = parse_test_file("tests03_boolean.toml");
    // TODO: Implement when matcher is ready
}

#[test]
#[ignore = "Matcher not yet implemented"]
fn tests04_anchors_matches() {
    let _test_file = parse_test_file("tests04_anchors.toml");
    // TODO: Implement when matcher is ready
}

#[test]
#[ignore = "Matcher not yet implemented"]
fn tests05_match_end_positions() {
    let _test_file = parse_test_file("tests05_match_end.toml");
    // TODO: Implement when matcher is ready
}

#[test]
#[ignore = "Matcher not yet implemented"]
fn tests06_nullable_positions() {
    let _test_file = parse_test_file("tests06_nullable_positions.toml");
    // TODO: Implement when matcher is ready
}

#[test]
#[ignore = "Matcher not yet implemented"]
fn tests07_unsupported_detection() {
    let _test_file = parse_test_file("tests07_unsupported.toml");
    // TODO: Implement when matcher is ready
}

#[test]
#[ignore = "Matcher not yet implemented"]
fn tests08_semantics_matches() {
    let _test_file = parse_test_file("tests08_semantics.toml");
    // TODO: Implement when matcher is ready
}

// =============================================================================
// Individual Pattern Tests
// =============================================================================
// These tests verify specific patterns from the test suite.

/// Test basic anchored pattern parsing
#[test]
fn test_anchored_digit() {
    let pattern = r"^\d$";
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Test complement pattern parsing
#[test]
fn test_complement_pattern() {
    let pattern = r"~(_*\d\d_*)";
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Test intersection pattern parsing
#[test]
fn test_intersection_pattern() {
    let pattern = r".*a.*&.*b.*&.*c.*";
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Test lookahead pattern parsing
#[test]
fn test_lookahead_pattern() {
    let pattern = r".*(?=aaa)";
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Test lookbehind pattern parsing
#[test]
fn test_lookbehind_pattern() {
    let pattern = r"(?<=author).*";
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Test word boundary pattern parsing
#[test]
fn test_word_boundary_pattern() {
    let pattern = r"\b1\b";
    parse_to_ir(pattern).expect("Pattern should compile");
}

/// Test complex date pattern parsing
#[test]
fn test_complex_date_pattern() {
    let pattern = r"((\d{2})|(\d))\/((\d{2})|(\d))\/((\d{4})|(\d{2}))";
    parse_to_ir(pattern).expect("Pattern should compile");
}
