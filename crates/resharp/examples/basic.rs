//! Basic example showing how to compile and inspect RE# patterns.
//!
//! Run with: `cargo run --example basic`

use resharp::Regex;

fn main() {
    println!("=== RE# Pattern Examples ===\n");

    // Example 1: Simple literal pattern
    println!("1. Simple literal pattern:");
    let re = Regex::new("hello").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    // Example 2: Character class pattern
    println!("2. Character class pattern:");
    let re = Regex::new(r"\d+").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    // Example 3: RE# conjunction (AND)
    println!("3. RE# conjunction - match both 'foo' AND 'bar':");
    let re = Regex::new(r".*foo.*&.*bar.*").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    // Example 4: RE# negation (NOT)
    println!("4. RE# negation - match anything NOT containing 'error':");
    let re = Regex::new(r"~(.*error.*)").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    // Example 5: Combined conjunction and negation
    println!("5. Combined - match 'success' but NOT 'warning':");
    let re = Regex::new(r".*success.*&~(.*warning.*)").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    // Example 6: Lookaround assertions
    println!("6. Lookahead assertion:");
    let re = Regex::new(r"foo(?=bar)").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    // Example 7: Lookbehind assertion
    println!("7. Lookbehind assertion:");
    let re = Regex::new(r"(?<=foo)bar").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    // Example 8: Universal wildcard
    println!("8. RE# universal wildcard (matches newlines too):");
    let re = Regex::new(r"_*foo_*").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    // Example 9: Complex BibTeX extraction pattern
    println!("9. Complex pattern - BibTeX author extraction:");
    let re =
        Regex::new(r"(?<=author=\{)(~(.*and.*)&[A-Z][\w ,]+)(?=\})").expect("failed to compile");
    println!("   Pattern: {}", re.pattern());
    println!("   IR:      {}", re.pretty_print());
    println!();

    println!("=== End of Examples ===");
}
