use nom_locate::{position, LocatedSpan};

// Type Alias for the span fragment
pub type Span<'a> = LocatedSpan<&'a str>;

/// Terminal Tokens
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Token<'a> {
    Hexadecimal(Span<'a>),
    Octal(Span<'a>),
    Binary(Span<'a>),
    Integer(Span<'a>),
    Float(Span<'a>),
    Identifier(Span<'a>),
}
