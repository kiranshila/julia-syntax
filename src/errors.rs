use std::fmt;

use crate::parser::Span;
use miette::{Diagnostic, SourceCode, SourceSpan};
use nom::error::{ErrorKind, FromExternalError, ParseError};
use thiserror::Error;

// We need a custom error type for a few reasons, to hook into miette, which is a nice
// error handling crate as well as provide some glue to whatever context we're carrying with us

#[derive(Diagnostic, Error, Debug, PartialEq)]
#[diagnostic(help("Buh?"))]
pub enum JuliaParseError {
    #[error("invalid lvalue")]
    LValue,
    #[error("unknown parser error")]
    Unparseable,
}

impl<I> ParseError<I> for JuliaParseError {
    fn from_error_kind(_input: I, _kind: ErrorKind) -> Self {
        JuliaParseError::Unparseable
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for JuliaParseError {
    fn from_external_error(input: I, kind: ErrorKind, e: E) -> Self {
        JuliaParseError::Unparseable
    }
}
