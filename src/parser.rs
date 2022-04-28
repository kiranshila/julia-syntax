use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{opt, recognize},
    multi::{many1, separated_list1},
    sequence::{preceded, tuple},
    IResult,
};

// Numbers

fn hex_body(input: &str) -> IResult<&str, &str> {
    recognize(separated_list1(
        char('_'),
        many1(one_of("0123456789abcdefABCDEF")),
    ))(input)
}

fn octal_body(input: &str) -> IResult<&str, &str> {
    recognize(separated_list1(char('_'), many1(one_of("01234567"))))(input)
}

fn binary_body(input: &str) -> IResult<&str, &str> {
    recognize(separated_list1(char('_'), many1(one_of("01"))))(input)
}

fn hexadecimal(input: &str) -> IResult<&str, &str> {
    recognize(preceded(tag("0x"), hex_body))(input)
}

fn octal(input: &str) -> IResult<&str, &str> {
    recognize(preceded(tag("0o"), octal_body))(input)
}

fn binary(input: &str) -> IResult<&str, &str> {
    recognize(preceded(tag("0b"), binary_body))(input)
}

fn integer(input: &str) -> IResult<&str, &str> {
    recognize(separated_list1(tag("_"), many1(one_of("0123456789"))))(input)
}

fn float(input: &str) -> IResult<&str, &str> {
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
    ))(input)
}

#[cfg(test)]
mod num_tests {
    use super::*;

    #[test]
    fn hex() {
        assert_eq!(
            hexadecimal("0x1234567890ABCDEF"),
            Ok(("", "0x1234567890ABCDEF"))
        );
        assert_eq!(
            hexadecimal("0xCAFE_BEEF_BAD_F00D"),
            Ok(("", "0xCAFE_BEEF_BAD_F00D"))
        );
        assert_eq!(hexadecimal("0xDEAD_BEEF"), Ok(("", "0xDEAD_BEEF")));
        assert_eq!(hexadecimal("0xCaFe_B0bA_"), Ok(("_", "0xCaFe_B0bA")));
        assert_eq!(hexadecimal("0xDEAD__BEEF"), Ok(("__BEEF", "0xDEAD")));
    }

    #[test]
    fn oct() {
        assert_eq!(octal("0o420"), Ok(("", "0o420")));
        assert_eq!(octal("0o420_0123"), Ok(("", "0o420_0123")));
        assert_eq!(octal("0o420_0123_"), Ok(("_", "0o420_0123")));
    }

    #[test]
    fn bin() {
        assert_eq!(binary("0b10010"), Ok(("", "0b10010")));
        assert_eq!(binary("0b10_10_10"), Ok(("", "0b10_10_10")));
        assert_eq!(binary("0b10_10_10_"), Ok(("_", "0b10_10_10")));
        assert_eq!(binary("0b10__10"), Ok(("__10", "0b10")));
    }

    #[test]
    fn int() {
        assert_eq!(integer("0123456789"), Ok(("", "0123456789")));
        assert_eq!(integer("420_69"), Ok(("", "420_69")));
        assert_eq!(integer("420_69_"), Ok(("_", "420_69")));
        assert_eq!(integer("420__69"), Ok(("__69", "420")));
    }

    #[test]
    fn floats() {
        assert_eq!(float("1.0"), Ok(("", "1.0")));
        assert_eq!(float("1."), Ok(("", "1.")));
        assert_eq!(float("0.5"), Ok(("", "0.5")));
        assert_eq!(float(".5"), Ok(("", ".5")));
        assert_eq!(float("1e10"), Ok(("", "1e10")));
        assert_eq!(float("2.5e-4"), Ok(("", "2.5e-4")));
        assert_eq!(float("2.5f-4"), Ok(("", "2.5f-4")));
        assert_eq!(float("1_000_000f-4"), Ok(("", "1_000_000f-4")));
        assert_eq!(float("-Inf32"), Ok(("", "-Inf32")));
        assert_eq!(float("0xDEAD.BEEF_420p69"), Ok(("", "0xDEAD.BEEF_420p69")));
        assert_eq!(
            float("0xDEAD.BEEF_420p+69"),
            Ok(("", "0xDEAD.BEEF_420p+69"))
        );
        assert_eq!(
            float("0xDEAD.BEEF_420p-69"),
            Ok(("", "0xDEAD.BEEF_420p-69"))
        );
        assert_eq!(float("0x.BEEF_420p69"), Ok(("", "0x.BEEF_420p69")));
        assert_eq!(float("0x.BEEF_420p-69"), Ok(("", "0x.BEEF_420p-69")));
        assert_eq!(float("0x.D_E_A_Dp-69"), Ok(("", "0x.D_E_A_Dp-69")));
        assert_eq!(float("0x.4p-1"), Ok(("", "0x.4p-1")));
    }
}
