//! The arudha padas: where each house's matter is seen to stand
//! (`03-design/arudhas-measured.md`).
//!
//! A house's pada is the count from its sign to that sign's lord, counted
//! again from the lord. A pada that falls in the house itself or in the
//! seventh from it moves to the tenth from where it fell, so one in the
//! seventh lands in the fourth house. The lords are the catalogue's, Mars
//! for Scorpio and Saturn for Aquarius, which is what the corpus's
//! recording engine takes for every one of 852 recorded padas; the `_by`
//! forms take the lord a house counts to from the caller, for a school
//! that makes the nodes co-lords and counts a two-lorded sign to the
//! stronger (BPHS ch. 29 v. 7; crux C135).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};

/// One house's arudha pada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Arudha {
    /// The house, 1 to 12.
    pub house: u8,
    /// The sign the pada stands in.
    pub sign: Rashi,
    /// The sign the count first reached, before any exception moved it.
    pub counted: Rashi,
    /// Whether the count reached the house or the seventh from it and the
    /// pada moved to the tenth from there.
    pub moved: bool,
}

/// The sign `steps` signs on from `sign`.
fn on(sign: Rashi, steps: usize) -> Rashi {
    u16::try_from((sign as usize + steps) % 12)
        .ok()
        .and_then(Rashi::from_id)
        .unwrap_or(sign)
}

/// The steps forward from `from` to `to`, 0 to 11.
fn steps(from: Rashi, to: Rashi) -> usize {
    (to as usize + 12 - from as usize) % 12
}

/// The pada of `house` (1 to 12) for a chart whose lagna is in `lagna` and
/// whose grahas stand where `sign_of` says.
#[must_use]
pub fn arudha(lagna: Rashi, house: u8, sign_of: impl Fn(Graha) -> Rashi) -> Arudha {
    arudha_by(lagna, house, catalogue_lord, sign_of)
}

/// [`arudha`] with the lord each sign counts to named by `lord_of`.
#[must_use]
pub fn arudha_by(
    lagna: Rashi,
    house: u8,
    lord_of: impl Fn(Rashi) -> Graha,
    sign_of: impl Fn(Graha) -> Rashi,
) -> Arudha {
    let house = house.clamp(1, 12);
    let (counted, moved, sign) = count(on(lagna, usize::from(house - 1)), lord_of, sign_of);
    Arudha {
        house,
        sign,
        counted,
        moved,
    }
}

/// The pada of whichever house stands in `sign`, counted from that sign
/// whatever the lagna: a pada counted from another pada, as the upapada's
/// seventh is, as well as a house's.
#[must_use]
pub fn pada(sign: Rashi, sign_of: impl Fn(Graha) -> Rashi) -> Rashi {
    pada_by(sign, catalogue_lord, sign_of)
}

/// [`pada`] with the lord each sign counts to named by `lord_of`.
#[must_use]
pub fn pada_by(
    sign: Rashi,
    lord_of: impl Fn(Rashi) -> Graha,
    sign_of: impl Fn(Graha) -> Rashi,
) -> Rashi {
    count(sign, lord_of, sign_of).2
}

/// A sign's lord in the catalogue: Mars for Scorpio, Saturn for Aquarius.
fn catalogue_lord(sign: Rashi) -> Graha {
    sign.attributes().lord
}

/// The count from `sign` to its lord and as far again, whether it moved, and
/// where the pada stands.
fn count(
    sign: Rashi,
    lord_of: impl Fn(Rashi) -> Graha,
    sign_of: impl Fn(Graha) -> Rashi,
) -> (Rashi, bool, Rashi) {
    let at = sign_of(lord_of(sign));
    let counted = on(at, steps(sign, at));
    let moved = matches!(steps(sign, counted), 0 | 6);
    (counted, moved, if moved { on(counted, 9) } else { counted })
}

/// All twelve padas, the first house's first: the arudha lagna.
#[must_use]
pub fn arudha_padas(lagna: Rashi, sign_of: impl Fn(Graha) -> Rashi) -> [Arudha; 12] {
    arudha_padas_by(lagna, catalogue_lord, sign_of)
}

/// [`arudha_padas`] with the lord each sign counts to named by `lord_of`.
#[must_use]
pub fn arudha_padas_by(
    lagna: Rashi,
    lord_of: impl Fn(Rashi) -> Graha,
    sign_of: impl Fn(Graha) -> Rashi,
) -> [Arudha; 12] {
    let mut house = 0_u8;
    [(); 12].map(|()| {
        house += 1;
        arudha_by(lagna, house, &lord_of, &sign_of)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pada_counts_to_the_lord_and_as_far_again() {
        // Aries lagna, Mars in Cancer: three signs on, three more is Libra,
        // the seventh, so the pada moves to the tenth from Libra.
        let sign_of = |graha: Graha| match graha {
            Graha::Mars => Rashi::Cancer,
            Graha::Venus => Rashi::Taurus,
            _ => Rashi::Leo,
        };
        let first = arudha(Rashi::Aries, 1, sign_of);
        assert_eq!(
            (first.counted, first.moved, first.sign),
            (Rashi::Libra, true, Rashi::Cancer)
        );
        // The second house, Taurus, with Venus in it: the count is none and
        // the pada is the house itself, moved to the tenth from it.
        let second = arudha(Rashi::Aries, 2, sign_of);
        assert_eq!(
            (second.counted, second.moved, second.sign),
            (Rashi::Taurus, true, Rashi::Aquarius)
        );
        // A pada counted from its sign is the house's, whatever the lagna.
        assert_eq!(pada(Rashi::Taurus, sign_of), second.sign);
        assert_eq!(pada(Rashi::Aries, sign_of), first.sign);
        let all = arudha_padas(Rashi::Aries, sign_of);
        assert_eq!(all[0], first);
        assert_eq!(
            all.map(|pada| pada.house),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
        );
    }
}
