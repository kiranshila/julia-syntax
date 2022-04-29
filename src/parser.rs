use crate::extensions::{is_char_id, is_char_id_start};
use crate::syntax::{Span, Token};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{anychar, char, digit1, hex_digit1, oct_digit1, one_of, satisfy},
    combinator::{all_consuming, complete, cut, map_res, opt, recognize, verify},
    multi::{many0, many1, separated_list1},
    sequence::{pair, preceded, tuple},
    IResult,
};

// Numbers

fn hex_body(input: Span) -> IResult<Span, Span> {
    recognize(separated_list1(char('_'), hex_digit1))(input)
}

fn hexadecimal(input: Span) -> IResult<Span, Token> {
    map_res(
        recognize(preceded(tag("0x"), hex_body)),
        |s| -> Result<Token, ()> { Ok(Token::Hexadecimal(s)) },
    )(input)
}

fn octal_body(input: Span) -> IResult<Span, Span> {
    recognize(separated_list1(char('_'), oct_digit1))(input)
}

fn octal(input: Span) -> IResult<Span, Token> {
    map_res(
        recognize(preceded(tag("0o"), octal_body)),
        |s| -> Result<Token, ()> { Ok(Token::Octal(s)) },
    )(input)
}

fn binary_body(input: Span) -> IResult<Span, Span> {
    recognize(separated_list1(char('_'), many1(one_of("01"))))(input)
}

fn binary(input: Span) -> IResult<Span, Token> {
    map_res(
        recognize(preceded(tag("0b"), binary_body)),
        |s| -> Result<Token, ()> { Ok(Token::Binary(s)) },
    )(input)
}

fn integer(input: Span) -> IResult<Span, Token> {
    map_res(
        recognize(separated_list1(tag("_"), digit1)),
        |s| -> Result<Token, ()> { Ok(Token::Integer(s)) },
    )(input)
}

fn float(input: Span) -> IResult<Span, Token> {
    map_res(
        alt((
            // Case one: .42
            recognize(tuple((
                char('.'),
                integer,
                opt(tuple((
                    one_of("eEfF"),
                    opt(one_of("+-")),
                    many1(one_of("0123456789")),
                ))),
            ))),
            // Case two: 42e42 and 42.42e42
            recognize(tuple((
                integer,
                opt(preceded(char('.'), integer)),
                one_of("eEfF"),
                opt(one_of("+-")),
                many1(one_of("0123456789")), // No underscores in exponent
            ))),
            // Case three: 42. and 42.42
            recognize(tuple((integer, char('.'), opt(integer)))),
            // Special case literals
            recognize(alt((
                tag("NaN32"),
                tag("NaN64"),
                tag("NaN"),
                tag("-Inf32"),
                tag("-Inf64"),
                tag("-Inf"),
                tag("Inf32"),
                tag("Inf64"),
                tag("Inf"),
            ))),
            // Cursed hex float literals
            recognize(preceded(
                tag("0x"),
                tuple((
                    alt((
                        recognize(tuple((hex_body, opt(tuple((char('.'), hex_body)))))),
                        recognize(tuple((char('.'), hex_body))),
                    )),
                    char('p'),
                    opt(one_of("+-")),
                    many1(one_of("0123456789")),
                )),
            )),
        )),
        |s| -> Result<Token, ()> { Ok(Token::Float(s)) },
    )(input)
}

/// Internal testing fn
fn test_spanned(parser: fn(Span) -> IResult<Span, Token>, input: &str, remaining: &str) {
    let in_span = Span::new(input);
    let (remaining_span, _) = parser(in_span).unwrap();
    assert_eq!(remaining_span.fragment(), &remaining);
}

#[cfg(test)]
mod num_tests {
    use super::*;

    #[test]
    fn hex() {
        test_spanned(hexadecimal, "0xDEAD__BEEF", "__BEEF");
        test_spanned(hexadecimal, "0xDEAD_BEEF", "");
        test_spanned(hexadecimal, "0xCAFE_BEED_BAD_F00D", "");
        test_spanned(hexadecimal, "0xcAfE_b0BA", "");
        test_spanned(hexadecimal, "0xFF*0xFF", "*0xFF");
    }

    #[test]
    fn oct() {
        test_spanned(octal, "0o420", "");
        test_spanned(octal, "0o420_0123", "");
        test_spanned(octal, "0o420_0123_", "_");
    }

    #[test]
    fn bin() {
        test_spanned(binary, "0b10010", "");
        test_spanned(binary, "0b10_10_10", "");
        test_spanned(binary, "0b10_10_", "_");
        test_spanned(binary, "0b10__10", "__10");
    }

    #[test]
    fn int() {
        test_spanned(integer, "0123456789", "");
        test_spanned(integer, "420_69", "");
        test_spanned(integer, "420__69", "__69");
    }

    #[test]
    fn floats() {
        test_spanned(float, "1.0", "");
        test_spanned(float, "1.", "");
        test_spanned(float, "0.5", "");
        test_spanned(float, ".5", "");
        test_spanned(float, "1e10", "");
        test_spanned(float, "2.5e-4", "");
        test_spanned(float, "2.5f+4", "");
        test_spanned(float, "1_00_00f-4", "");
        test_spanned(float, "-Inf32", "");
        test_spanned(float, "0xDEAD.BEEF_420p69", "");
        test_spanned(float, "0xDEAD_BEEFp-420", "");
        test_spanned(float, "0xD_E_A_Dp-69", "");
        test_spanned(float, "0x.4p-1", "");
    }
}

// Identifiers (non operator)

fn identifier(input: Span) -> IResult<Span, Token> {
    map_res(
        recognize(pair(satisfy(is_char_id_start), many0(satisfy(is_char_id)))),
        |s| -> Result<Token, ()> { Ok(Token::Identifier(s)) },
    )(input)
}

#[cfg(test)]
mod ident_tests {
    use super::*;

    #[test]
    fn ident() {
        test_spanned(identifier, "hello_world", "");
        test_spanned(identifier, "a_valid_ident not", " not");
        test_spanned(identifier, "🐈", "");
        test_spanned(identifier, "𒀃", "");
        test_spanned(identifier, "my_β̂₂", "");
        test_spanned(identifier, "ĉ̄", "");
    }
}
