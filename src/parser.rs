use crate::extensions::{is_char_id, is_char_id_start};
use crate::syntax::{AssignmentExpr, IdentifierLiteral, NumericLiteral, OperatorLiteral, Span};
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

fn hexadecimal(input: Span) -> IResult<Span, NumericLiteral> {
    map_res(
        recognize(preceded(tag("0x"), hex_body)),
        |s| -> Result<NumericLiteral, ()> { Ok(NumericLiteral::Hexadecimal(s)) },
    )(input)
}

fn octal_body(input: Span) -> IResult<Span, Span> {
    recognize(separated_list1(char('_'), oct_digit1))(input)
}

fn octal(input: Span) -> IResult<Span, NumericLiteral> {
    map_res(
        recognize(preceded(tag("0o"), octal_body)),
        |s| -> Result<NumericLiteral, ()> { Ok(NumericLiteral::Octal(s)) },
    )(input)
}

fn binary_body(input: Span) -> IResult<Span, Span> {
    recognize(separated_list1(char('_'), many1(one_of("01"))))(input)
}

fn binary(input: Span) -> IResult<Span, NumericLiteral> {
    map_res(
        recognize(preceded(tag("0b"), binary_body)),
        |s| -> Result<NumericLiteral, ()> { Ok(NumericLiteral::Binary(s)) },
    )(input)
}

fn integer(input: Span) -> IResult<Span, NumericLiteral> {
    map_res(
        recognize(separated_list1(tag("_"), digit1)),
        |s| -> Result<NumericLiteral, ()> { Ok(NumericLiteral::Integer(s)) },
    )(input)
}

fn float(input: Span) -> IResult<Span, NumericLiteral> {
    map_res(
        alt((
            // Case one: .42
            recognize(tuple((
                char('.'),
                integer,
                opt(tuple((one_of("eEfF"), opt(one_of("+-")), digit1))),
            ))),
            // Case two: 42e42 and 42.42e42
            recognize(tuple((
                integer,
                opt(preceded(char('.'), integer)),
                one_of("eEfF"),
                opt(one_of("+-")),
                digit1, // No underscores in exponent
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
                    digit1,
                )),
            )),
        )),
        |s| -> Result<NumericLiteral, ()> { Ok(NumericLiteral::Float(s)) },
    )(input)
}

/// Internal testing fn

#[cfg(test)]
mod num_tests {
    use super::*;

    fn test_spanned(
        parser: fn(Span) -> IResult<Span, NumericLiteral>,
        input: &str,
        remaining: &str,
    ) {
        let in_span = Span::new(input);
        let (remaining_span, _) = parser(in_span).unwrap();
        assert_eq!(remaining_span.fragment(), &remaining);
    }

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

fn identifier(input: Span) -> IResult<Span, IdentifierLiteral> {
    map_res(
        recognize(pair(satisfy(is_char_id_start), many0(satisfy(is_char_id)))),
        |s| -> Result<IdentifierLiteral, ()> { Ok(IdentifierLiteral(s)) },
    )(input)
}

#[cfg(test)]
mod ident_tests {
    use super::*;

    fn test_spanned(
        parser: fn(Span) -> IResult<Span, IdentifierLiteral>,
        input: &str,
        remaining: &str,
    ) {
        let in_span = Span::new(input);
        let (remaining_span, _) = parser(in_span).unwrap();
        assert_eq!(remaining_span.fragment(), &remaining);
    }

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

// Operators (Buckle up)

fn assignment(input: Span) -> IResult<Span, OperatorLiteral> {
    map_res(
        alt((
            recognize(pair(
                opt(tag(".")),
                alt((
                    tag("="),
                    tag("+="),
                    tag("-="),
                    tag("−="),
                    tag("*="),
                    tag("\\="),
                    tag("//="),
                    tag("\\\\="),
                    tag("^="),
                    tag("÷="),
                    tag("<<="),
                    tag(">>="),
                    tag(">>>="),
                    tag("|="),
                    tag("&="),
                    tag("⊻="),
                    tag("≔"),
                    tag("⩴"),
                    tag("≕"),
                )),
            )),
            tag("~"),
            tag(":="),
            tag("$="),
        )),
        |s| -> Result<OperatorLiteral, ()> { Ok(OperatorLiteral::Assignment(s)) },
    )(input)
}

#[cfg(test)]
mod operator_tests {
    use super::*;

    fn test_spanned(
        parser: fn(Span) -> IResult<Span, OperatorLiteral>,
        input: &str,
        remaining: &str,
    ) {
        let in_span = Span::new(input);
        let (remaining_span, _) = parser(in_span).unwrap();
        assert_eq!(remaining_span.fragment(), &remaining);
    }

    #[test]
    fn assign() {
        test_spanned(assignment, "=", "");
        test_spanned(assignment, "+=", "");
        test_spanned(assignment, ".+=", "");
    }
}

fn numeric(input: Span) -> IResult<Span, NumericLiteral> {
    alt((float, hexadecimal, octal, binary, integer))(input)
}

// Toy example, not real
fn assignment_expr(input: Span) -> IResult<Span, AssignmentExpr> {
    let (input, lvalue) = identifier(input)?;
    let (input, operator) = assignment(input)?;
    let (input, rvalue) = numeric(input)?;
    Ok((
        input,
        AssignmentExpr {
            lvalue,
            operator,
            rvalue,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        println!("{:#?}", assignment_expr(Span::new("foo_bar+=120e23")));
        assert!(false);
    }
}
