//! Follows the unicode logic from `julia_extensions.c`
use unicode_general_category::{get_general_category, GeneralCategory};

fn is_char_cat_id_start(c: char, cat: GeneralCategory) -> bool {
    let wc = c as u32;

    cat == GeneralCategory::UppercaseLetter     ||
        cat == GeneralCategory::LowercaseLetter ||
        cat == GeneralCategory::TitlecaseLetter ||
        cat == GeneralCategory::ModifierLetter  ||
        cat == GeneralCategory::OtherLetter     ||
        cat == GeneralCategory::LetterNumber    ||
        cat == GeneralCategory::CurrencySymbol  ||  // allow currency symbols

    // other symbols, but not arrows or replacement characters
        (cat == GeneralCategory::OtherSymbol && !(0x2190..=0x21FF).contains(&wc) &&
         wc != 0xfffc && wc != 0xfffd &&
         wc != 0x233f &&  // notslash
         wc != 0x00a6) || // broken bar

    // math symbol (category Sm) whitelist
        ((0x2140..=0x2a1c).contains(&wc) &&
        ((0x2140..=0x2144).contains(&wc) || // ⅀, ⅁, ⅂, ⅃, ⅄
        wc == 0x223f || wc == 0x22be || wc == 0x22bf || // ∿, ⊾, ⊿
        wc == 0x22a4 || wc == 0x22a5 ||   // ⊤ ⊥

        ((0x2200..=0x2233).contains(&wc) &&
         (wc == 0x2202 || wc == 0x2205 || wc == 0x2206 || // ∂, ∅, ∆
          wc == 0x2207 || wc == 0x220e || wc == 0x220f || // ∇, ∎, ∏
          wc == 0x2200 || wc == 0x2203 || wc == 0x2204 || // ∀, ∃, ∄
          wc == 0x2210 || wc == 0x2211 || // ∐, ∑
          wc == 0x221e || wc == 0x221f || // ∞, ∟
          wc >= 0x222b)) || // ∫, ∬, ∭, ∮, ∯, ∰, ∱, ∲, ∳

        (0x22c0..=0x22c3).contains(&wc) ||  // N-ary big ops: ⋀, ⋁, ⋂, ⋃
        (0x25F8..=0x25ff).contains(&wc) ||  // ◸, ◹, ◺, ◻, ◼, ◽, ◾, ◿

        (wc >= 0x266f &&
         (wc == 0x266f || wc == 0x27d8 || wc == 0x27d9 || // ♯, ⟘, ⟙
          (0x27c0..=0x27c1).contains(&wc) ||  // ⟀, ⟁
          (0x29b0..=0x29b4).contains(&wc) ||  // ⦰, ⦱, ⦲, ⦳, ⦴
          (0x2a00..=0x2a06).contains(&wc) ||  // ⨀, ⨁, ⨂, ⨃, ⨄, ⨅, ⨆
        (0x2a09..=0x2a16).contains(&wc) ||  // ⨉, ⨊, ⨋, ⨌, ⨍, ⨎, ⨏, ⨐, ⨑, ⨒, ⨓, ⨔, ⨕, ⨖
          wc == 0x2a1b || wc == 0x2a1c)))) || // ⨛, ⨜
        (wc >= 0x1d6c1 && // variants of \nabla and \partial
         (wc == 0x1d6c1 || wc == 0x1d6db ||
          wc == 0x1d6fb || wc == 0x1d715 ||
          wc == 0x1d735 || wc == 0x1d74f ||
          wc == 0x1d76f || wc == 0x1d789 ||
          wc == 0x1d7a9 || wc == 0x1d7c3)) ||

    // super- and subscript +-=()
        (0x207a..=0x207e).contains(&wc) ||
        (0x208a..=0x208e).contains(&wc) ||

    // angle symbols
        (0x2220..=0x2222).contains(&wc) || // ∠, ∡, ∢
        (0x299b..=0x29af).contains(&wc) || // ⦛, ⦜, ⦝, ⦞, ⦟, ⦠, ⦡, ⦢, ⦣, ⦤, ⦥, ⦦, ⦧, ⦨, ⦩, ⦪, ⦫, ⦬, ⦭, ⦮, ⦯

    // Other_ID_Start
        wc == 0x2118 || wc == 0x212E || // ℘, ℮
        (0x309B..=0x309C).contains(&wc) || // katakana-hiragana sound marks

    // bold-digits and double-struck digits
        (0x1D7CE..=0x1D7E1).contains(&wc) // 𝟎 through 𝟗 (inclusive), 𝟘 through 𝟡 (inclusive)
}

/// Tests whether a character `c` starts a valid identifier
pub fn is_char_id_start(c: char) -> bool {
    match c as u32 {
        0x41..=0x5A | 0x61..=0x7A | 0x5F => true, // [a-zA-Z_]
        0x00..=0xA0 | 0x110000..=0xFFFFFFFF => false,
        _ => is_char_cat_id_start(c, get_general_category(c)),
    }
}

/// Tests whether a character `c` is valid in an identifier
/// This is more lax than the starting character
pub fn is_char_id(c: char) -> bool {
    let wc = c as u32;
    match wc {
        0x41..=0x5A | 0x61..=0x7A | 0x5F | 0x30..=0x39 | 0x21 => true, // [a-zA-Z_0-9!]
        0x00..=0xA0 | 0x110000..=0xFFFFFFFF => false,
        _ => {
            let cat = get_general_category(c);
            is_char_cat_id_start(c,cat) ||
                cat == GeneralCategory::NonspacingMark
                    || cat == GeneralCategory::SpacingMark
                    || cat == GeneralCategory::DecimalNumber
                    || cat == GeneralCategory::ConnectorPunctuation
                    || cat == GeneralCategory::ModifierSymbol
                    || cat == GeneralCategory::EnclosingMark
                    || cat == GeneralCategory::OtherNumber ||
            // primes (single, double, triple, their reverses, and quadruple)
                (0x2032..=0x2037).contains(&wc) ||
                wc == 0x2057
        }
    }
}

// From `julia_opsuffs.h`
const OPSUFFS: [u32; 117] = [
    0x00b2, // ²
    0x00b3, // ³
    0x00b9, // ¹
    0x02b0, // ʰ
    0x02b2, // ʲ
    0x02b3, // ʳ
    0x02b7, // ʷ
    0x02b8, // ʸ
    0x02e1, // ˡ
    0x02e2, // ˢ
    0x02e3, // ˣ
    0x1d2c, // ᴬ
    0x1d2e, // ᴮ
    0x1d30, // ᴰ
    0x1d31, // ᴱ
    0x1d33, // ᴳ
    0x1d34, // ᴴ
    0x1d35, // ᴵ
    0x1d36, // ᴶ
    0x1d37, // ᴷ
    0x1d38, // ᴸ
    0x1d39, // ᴹ
    0x1d3a, // ᴺ
    0x1d3c, // ᴼ
    0x1d3e, // ᴾ
    0x1d3f, // ᴿ
    0x1d40, // ᵀ
    0x1d41, // ᵁ
    0x1d42, // ᵂ
    0x1d43, // ᵃ
    0x1d47, // ᵇ
    0x1d48, // ᵈ
    0x1d49, // ᵉ
    0x1d4d, // ᵍ
    0x1d4f, // ᵏ
    0x1d50, // ᵐ
    0x1d52, // ᵒ
    0x1d56, // ᵖ
    0x1d57, // ᵗ
    0x1d58, // ᵘ
    0x1d5b, // ᵛ
    0x1d5d, // ᵝ
    0x1d5e, // ᵞ
    0x1d5f, // ᵟ
    0x1d60, // ᵠ
    0x1d61, // ᵡ
    0x1d62, // ᵢ
    0x1d63, // ᵣ
    0x1d64, // ᵤ
    0x1d65, // ᵥ
    0x1d66, // ᵦ
    0x1d67, // ᵧ
    0x1d68, // ᵨ
    0x1d69, // ᵩ
    0x1d6a, // ᵪ
    0x1d9c, // ᶜ
    0x1da0, // ᶠ
    0x1da5, // ᶥ
    0x1da6, // ᶦ
    0x1dab, // ᶫ
    0x1db0, // ᶰ
    0x1db8, // ᶸ
    0x1dbb, // ᶻ
    0x1dbf, // ᶿ
    0x2032, // ′
    0x2033, // ″
    0x2034, // ‴
    0x2035, // ‵
    0x2036, // ‶
    0x2037, // ‷
    0x2057, // ⁗
    0x2070, // ⁰
    0x2071, // ⁱ
    0x2074, // ⁴
    0x2075, // ⁵
    0x2076, // ⁶
    0x2077, // ⁷
    0x2078, // ⁸
    0x2079, // ⁹
    0x207a, // ⁺
    0x207b, // ⁻
    0x207c, // ⁼
    0x207d, // ⁽
    0x207e, // ⁾
    0x207f, // ⁿ
    0x2080, // ₀
    0x2081, // ₁
    0x2082, // ₂
    0x2083, // ₃
    0x2084, // ₄
    0x2085, // ₅
    0x2086, // ₆
    0x2087, // ₇
    0x2088, // ₈
    0x2089, // ₉
    0x208a, // ₊
    0x208b, // ₋
    0x208c, // ₌
    0x208d, // ₍
    0x208e, // ₎
    0x2090, // ₐ
    0x2091, // ₑ
    0x2092, // ₒ
    0x2093, // ₓ
    0x2095, // ₕ
    0x2096, // ₖ
    0x2097, // ₗ
    0x2098, // ₘ
    0x2099, // ₙ
    0x209a, // ₚ
    0x209b, // ₛ
    0x209c, // ₜ
    0x2c7c, // ⱼ
    0x2c7d, // ⱽ
    0xa71b, // ꜛ
    0xa71c, // ꜜ
    0xa71d, // ꜝ
];

/// Returns true if character `c` can follow an operator (e.g. +) and be parsed as part of the operator
pub fn is_op_suffix_char(c: char) -> bool {
    let wc = c as u32;
    match wc {
        0x00..=0xA0 | 0x110000..=0xFFFFFFFF => false,
        _ => {
            let cat = get_general_category(c);
            cat == GeneralCategory::NonspacingMark
                || cat == GeneralCategory::SpacingMark
                || cat == GeneralCategory::EnclosingMark
                || OPSUFFS.contains(&wc) // Do we really need a hash table? There aren't that many
        }
    }
}

#[cfg(test)]
mod ident_tests {
    use super::*;

    #[test]
    fn identifier_start_characters() {
        assert!(is_char_id_start('a'));
        assert!(is_char_id_start('_'));
        assert!(is_char_id_start('λ'));
        assert!(is_char_id_start('⨊'));
        assert!(!is_char_id_start('′'));
    }

    #[test]
    fn identifier_characters() {
        assert!(is_char_id('a'));
        assert!(is_char_id('_'));
        assert!(is_char_id('λ'));
        assert!(is_char_id('⨊'));
        assert!(is_char_id('′'));
    }

    #[test]
    fn op_suffix() {
        assert!(is_op_suffix_char('ᵡ'));
        assert!(!is_op_suffix_char('1'));
    }
}
