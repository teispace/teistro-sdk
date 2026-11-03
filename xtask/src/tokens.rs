//! A dependency-free token count (`03-design/dossier-tokens-measured.md`,
//! C1165): an approximation of what a byte-pair tokenizer spends on a
//! text, the same ruler for every mode and locale a dossier is written in.
//!
//! No model's tokenizer ships with the SDK, and every one is a vocabulary
//! of tens of thousands of pieces trained on its own corpus, so a count by
//! one is that model's alone. This one is the pieces such a tokenizer
//! cannot merge across, each priced as such vocabularies commonly price
//! it:
//!
//! - a run of ASCII letters: one token for each 4 letters or part of 4,
//!   so `Jupiter` is 2 and `PURVA` 2;
//! - a run of digits: one token for each 3 digits or part of 3;
//! - a newline: one token; other whitespace joins the piece after it;
//! - any other ASCII character (`|`, `.`, `_`, `-`): one token each;
//! - any other scalar value (Devanagari, IAST's marked letters): one
//!   token each, since such vocabularies hold few merges of them.
//!
//! It is a ruler, not a model: it says how the modes and locales compare,
//! and the page says so.

/// The approximate token count of `text`.
pub(crate) fn approximate(text: &str) -> usize {
    let mut tokens = 0;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_alphabetic() || c.is_ascii_digit() {
            let digits = c.is_ascii_digit();
            let mut run = 1_usize;
            while chars
                .next_if(|next| {
                    if digits {
                        next.is_ascii_digit()
                    } else {
                        next.is_ascii_alphabetic()
                    }
                })
                .is_some()
            {
                run += 1;
            }
            tokens += run.div_ceil(if digits { 3 } else { 4 });
        } else if c == '\n' {
            tokens += 1;
        } else if c.is_whitespace() {
            // Joins the piece after it.
        } else {
            tokens += 1;
        }
    }
    tokens
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::approximate;

    #[test]
    fn each_piece_is_priced_as_the_module_says() {
        assert_eq!(approximate(""), 0);
        // Letters by fours, digits by threes.
        assert_eq!(approximate("Jupiter"), 2);
        assert_eq!(approximate("PURVA_PHALGUNI"), 2 + 1 + 2);
        assert_eq!(approximate("2461055"), 3);
        // A row: the key, the bar, the digits and the point.
        assert_eq!(approximate("JUPITER|12.5\n"), 2 + 1 + 1 + 1 + 1 + 1);
        // Spaces join what follows; a newline is its own.
        assert_eq!(approximate("Purva Phalguni\n"), 2 + 2 + 1);
        // A Devanagari name is a token a scalar value.
        assert_eq!(approximate("गुरु"), 4);
        assert_eq!(approximate("Meṣa"), 1 + 1 + 1);
    }
}
