//! Syntax kinds for RE# regex patterns.

use cstree::{RawSyntaxKind, Syntax};

/// All syntax kinds for RE# regex patterns.
///
/// Tokens are leaf elements with source text.
/// Nodes are composite elements containing other nodes/tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // ===========================
    // TOKENS (Terminals)
    // ===========================

    // Literal characters
    /// Single character literal: a, b, 1
    Char = 0,
    /// Escape sequence: \n, \t, \x41, \u0041
    EscapeSeq,

    // Operators
    /// `|` alternation
    Pipe,
    /// `&` RE# conjunction
    Ampersand,
    /// `~` RE# negation
    Tilde,
    /// `^` anchor or char class negation
    Caret,
    /// `$` end anchor
    Dollar,
    /// `.` any char (except newline)
    Dot,
    /// `_` RE# universal wildcard
    Underscore,
    /// `*` zero or more
    Star,
    /// `+` one or more
    Plus,
    /// `?` zero or one / lazy modifier
    Question,

    // Delimiters
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `{`
    LBrace,
    /// `}`
    RBrace,

    // Special tokens
    /// `\`
    Backslash,
    /// `-` in character classes
    Hyphen,
    /// `:` in (?:...)
    Colon,
    /// `=` in (?=...)
    Equals,
    /// `!` in (?!...)
    Exclaim,
    /// `<`
    LAngle,
    /// `>`
    RAngle,

    // Quantifier parts
    /// Numeric literal in {n,m}
    Number,
    /// `,` in {n,m}
    Comma,

    // Whitespace/comments
    /// Whitespace (in IgnorePatternWhitespace mode)
    Whitespace,
    /// Comment: (?#...) or # in x-mode
    Comment,

    // Error recovery
    /// Error token
    Error,

    // ===========================
    // NODES (Non-terminals)
    // ===========================
    /// Root node containing the entire pattern
    Root,

    // Leaf node kinds (from C# RegexNodeKind)
    /// Single character match: `a`
    One,
    /// Negated single character: `[^a]` or `.`
    Notone,
    /// Character class: `[a-z]`
    Set,
    /// Multi-character literal sequence: `abc`
    Multi,

    // Loop variants (greedy)
    /// `a*`
    Oneloop,
    /// `[^a]*`
    Notoneloop,
    /// `[a-z]*`
    Setloop,

    // Loop variants (lazy)
    /// `a*?`
    Onelazy,
    /// `[^a]*?`
    Notonelazy,
    /// `[a-z]*?`
    Setlazy,

    // Loop variants (atomic)
    /// `(?>a*)`
    Oneloopatomic,
    /// `(?>[^a]*)`
    Notoneloopatomic,
    /// `(?>[a-z]*)`
    Setloopatomic,

    /// Backreference: `\1`, `\k<name>`
    Backreference,

    // Anchors
    /// `^` in multiline mode
    Bol,
    /// `$` in multiline mode
    Eol,
    /// `\A` or `^` in singleline mode
    Beginning,
    /// `\z`
    End,
    /// `\Z` or `$` in singleline mode
    EndZ,
    /// `\G`
    Start,
    /// `\b`
    Boundary,
    /// `\B`
    NonBoundary,
    /// `\b` in ECMA mode
    ECMABoundary,
    /// `\B` in ECMA mode
    NonECMABoundary,

    // Special
    /// Zero-width match
    Empty,
    /// Never matches: `(?!)`
    Nothing,
    /// Internal optimization marker
    UpdateBumpalong,

    // Composite nodes
    /// Alternation: `a|b|c`
    Alternate,
    /// Concatenation: `abc`
    Concatenate,
    /// RE# conjunction: `a&b`
    Conjunction,
    /// RE# negation: `~a`
    Negation,

    // Quantified
    /// General loop: `(expr){n,m}`
    Loop,
    /// General lazy loop: `(expr){n,m}?`
    Lazyloop,

    // Groups
    /// Capturing group: `(expr)` or `(?<name>expr)`
    Capture,
    /// Non-capturing group: `(?:expr)`
    Group,
    /// Atomic group: `(?>expr)`
    Atomic,

    // Lookaround
    /// `(?=...)`
    PositiveLookahead,
    /// `(?<=...)`
    PositiveLookbehind,
    /// `(?!...)`
    NegativeLookahead,
    /// `(?<!...)`
    NegativeLookbehind,

    // Conditionals
    /// `(?(1)yes|no)`
    BackreferenceConditional,
    /// `(?(expr)yes|no)`
    ExpressionConditional,

    // Character class internals
    /// `[...]`
    CharClass,
    /// `[^...]`
    CharClassNegated,
    /// `a-z` inside class
    CharRange,

    /// Quantifier: `{n,m}` or `{n,}` or `{n}`
    Quantifier,

    /// Group modifiers: `(?imsxn-imsxn:...)`
    GroupModifiers,

    /// Named group: `<name>` or `'name'` part
    GroupName,
}

impl SyntaxKind {
    /// Returns true if this kind represents a leaf token.
    pub fn is_token(self) -> bool {
        (self as u16) < (Self::Root as u16)
    }

    /// Returns true if this kind represents a composite node.
    pub fn is_node(self) -> bool {
        !self.is_token()
    }

    /// Returns true if this is an anchor kind.
    pub fn is_anchor(self) -> bool {
        matches!(
            self,
            Self::Bol
                | Self::Eol
                | Self::Beginning
                | Self::End
                | Self::EndZ
                | Self::Start
                | Self::Boundary
                | Self::NonBoundary
                | Self::ECMABoundary
                | Self::NonECMABoundary
        )
    }

    /// Returns true if this is a loop kind.
    pub fn is_loop(self) -> bool {
        matches!(
            self,
            Self::Oneloop
                | Self::Notoneloop
                | Self::Setloop
                | Self::Onelazy
                | Self::Notonelazy
                | Self::Setlazy
                | Self::Oneloopatomic
                | Self::Notoneloopatomic
                | Self::Setloopatomic
                | Self::Loop
                | Self::Lazyloop
        )
    }
}

impl Syntax for SyntaxKind {
    fn from_raw(raw: RawSyntaxKind) -> Self {
        // Safety: we ensure all values in the enum are valid
        assert!(raw.0 <= Self::GroupName as u32, "Invalid SyntaxKind value");
        // SAFETY: raw.0 is within valid enum range
        unsafe { std::mem::transmute(raw.0 as u16) }
    }

    fn into_raw(self) -> RawSyntaxKind {
        RawSyntaxKind(self as u32)
    }

    fn static_text(self) -> Option<&'static str> {
        match self {
            Self::Pipe => Some("|"),
            Self::Ampersand => Some("&"),
            Self::Tilde => Some("~"),
            Self::Caret => Some("^"),
            Self::Dollar => Some("$"),
            Self::Dot => Some("."),
            Self::Underscore => Some("_"),
            Self::Star => Some("*"),
            Self::Plus => Some("+"),
            Self::Question => Some("?"),
            Self::LParen => Some("("),
            Self::RParen => Some(")"),
            Self::LBracket => Some("["),
            Self::RBracket => Some("]"),
            Self::LBrace => Some("{"),
            Self::RBrace => Some("}"),
            Self::Backslash => Some("\\"),
            Self::Hyphen => Some("-"),
            Self::Colon => Some(":"),
            Self::Equals => Some("="),
            Self::Exclaim => Some("!"),
            Self::LAngle => Some("<"),
            Self::RAngle => Some(">"),
            Self::Comma => Some(","),
            _ => None,
        }
    }
}
