use nom_locate::{position, LocatedSpan};

// Type Alias for the span fragment
pub type Span<'a> = LocatedSpan<&'a str>;

/// Terminal Tokens
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NumericLiteral<'a> {
    Hexadecimal(Span<'a>),
    Octal(Span<'a>),
    Binary(Span<'a>),
    Integer(Span<'a>),
    Float(Span<'a>),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct IdentifierLiteral<'a>(pub Span<'a>);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OperatorLiteral<'a> {
    Assignment(Span<'a>),
}

/// Toy example
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AssignmentExpr<'a> {
    pub lvalue: IdentifierLiteral<'a>,
    pub operator: OperatorLiteral<'a>,
    pub rvalue: NumericLiteral<'a>,
}
