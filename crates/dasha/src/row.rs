//! A nakshatra-seeded dasha system as a row over the K-udu kernel
//! (`03-design/dasha-kernels.md`): its lords and their years, and the map
//! from the Moon's nakshatra to the lord it starts with.
//!
//! A system is data. Vimshottari, Ashtottari and Dwadashottari differ in
//! the table and in four fields of the map, so each is a constant and not
//! a module; a row the checks refuse is refused by the field it gets wrong.

use teistro_core::catalogue::{DashaSystem, Graha};
use teistro_core::error::Error;

/// How many nakshatras a seed is counted over.
pub const NAKSHATRAS: u8 = 27;

/// One lord of a system and its years.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lord {
    /// The graha.
    pub graha: Graha,
    /// Its whole years in the cycle.
    pub years: u8,
}

/// Which way a system counts from its reference nakshatra to the seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Count {
    /// Forwards from the reference: Vimshottari from Ashwini.
    FromReference,
    /// Backwards to the reference: Dwadashottari to Revati.
    ToReference,
}

/// A nakshatra-seeded system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UduRow {
    /// Which system the row is.
    pub system: DashaSystem,
    /// The lords, in the order they run.
    pub lords: &'static [Lord],
    /// The nakshatra, 0 for Ashwini, that maps to the first lord.
    pub reference: u8,
    /// Which way the seed is counted.
    pub count: Count,
    /// How many nakshatras each lord covers: 1 for most systems, 3 for
    /// Ashtottari. The balance is the elapsed part of this window.
    pub span: u8,
    /// What is added after the division, before the modulo: 3 for Yogini.
    pub offset: u8,
    /// Whether the lords run round the nakshatras again once they are all
    /// used — Vimshottari's nine three times — or cover them once, leaving
    /// a seed past them outside the cycle, as Ashtottari's eight windows of
    /// three leave three. Stated rather than inferred: Yogini's eight lords
    /// do not divide 27 either, and repeat.
    pub repeats: bool,
}

/// Where a seed nakshatra falls in a row's cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seat {
    /// The first lord, as an index into [`UduRow::lords`].
    pub lord: usize,
    /// How many whole nakshatras of the lord's window are already behind
    /// the seed: 0 when each lord covers one.
    pub within: u8,
    /// Whether the seed lies past the nakshatras the lords cover, which a
    /// conditional system such as Ashtottari has, and which the settings'
    /// `seed_overflow` decides.
    pub overflow: bool,
}

impl UduRow {
    /// The cycle's years, the sum of the lords'.
    #[must_use]
    pub fn total_years(&self) -> u32 {
        self.lords.iter().map(|lord| u32::from(lord.years)).sum()
    }

    /// The lord a seed nakshatra starts with, and where it sits in that
    /// lord's window.
    #[must_use]
    pub fn seat(&self, nakshatra: u8) -> Seat {
        let cycle = i16::from(NAKSHATRAS);
        let (seed, reference) = (i16::from(nakshatra), i16::from(self.reference));
        let counted = match self.count {
            Count::FromReference => (seed - reference).rem_euclid(cycle),
            Count::ToReference => (reference - seed).rem_euclid(cycle),
        };
        let counted = u16::try_from(counted).unwrap_or_default();
        let span = u16::from(self.span.max(1));
        let lords = u16::try_from(self.lords.len().max(1)).unwrap_or(u16::MAX);
        let group = counted / span;
        Seat {
            lord: usize::from((group + u16::from(self.offset)) % lords),
            within: u8::try_from(counted % span).unwrap_or_default(),
            overflow: !self.repeats && group >= lords,
        }
    }

    /// The checks every row passes before it is used, naming the field it
    /// fails.
    ///
    /// # Errors
    ///
    /// No lords, a lord of no years, a reference past the last nakshatra, a
    /// span of nothing, or windows that cover more nakshatras than there
    /// are.
    pub fn validate(&self) -> Result<(), Error> {
        let refuse = |field: &str, message: String| {
            Err(Error::invalid_arg(message).with_field(field.to_owned()))
        };
        if self.lords.is_empty() {
            return refuse("lords", String::from("a dasha system needs a lord"));
        }
        if let Some(at) = self.lords.iter().position(|lord| lord.years == 0) {
            return refuse(
                &format!("lords[{at}].years"),
                String::from("a lord of no years"),
            );
        }
        if self.reference >= NAKSHATRAS {
            return refuse(
                "reference",
                format!("nakshatra {} is past Revati", self.reference),
            );
        }
        if self.span == 0 {
            return refuse("span", String::from("a lord covers at least one nakshatra"));
        }
        let covered = usize::from(self.span) * self.lords.len();
        if covered > usize::from(NAKSHATRAS) {
            return refuse(
                "span",
                format!(
                    "{} lords of {} nakshatras cover {covered}, past 27",
                    self.lords.len(),
                    self.span
                ),
            );
        }
        Ok(())
    }
}

const fn lord(graha: Graha, years: u8) -> Lord {
    Lord { graha, years }
}

/// Vimshottari: nine lords over 120 years, counted from Ashwini, one
/// nakshatra each. Measured over the whole corpus
/// (`03-design/dasha-measured.md`).
pub const VIMSHOTTARI: UduRow = UduRow {
    system: DashaSystem::Vimshottari,
    lords: &[
        lord(Graha::Ketu, 7),
        lord(Graha::Venus, 20),
        lord(Graha::Sun, 6),
        lord(Graha::Moon, 10),
        lord(Graha::Mars, 7),
        lord(Graha::Rahu, 18),
        lord(Graha::Jupiter, 16),
        lord(Graha::Saturn, 19),
        lord(Graha::Mercury, 17),
    ],
    reference: 0,
    count: Count::FromReference,
    span: 1,
    offset: 0,
    repeats: true,
};

/// Every row this build implements, in the catalogue's order. A system
/// the catalogue names and no row implements is refused by name.
pub const ROWS: &[UduRow] = &[VIMSHOTTARI];

/// The row of a system, when this build implements one.
#[must_use]
pub fn row(system: DashaSystem) -> Option<&'static UduRow> {
    ROWS.iter().find(|row| row.system == system)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::*;

    #[test]
    fn every_shipped_row_passes_its_checks_and_sums_to_its_cycle() {
        for row in ROWS {
            row.validate().unwrap();
        }
        assert_eq!(VIMSHOTTARI.total_years(), 120);
    }

    #[test]
    fn vimshottari_seats_every_nakshatra_once_round_the_nine() {
        for nakshatra in 0..NAKSHATRAS {
            let seat = VIMSHOTTARI.seat(nakshatra);
            assert_eq!(seat.lord, usize::from(nakshatra) % 9);
            assert_eq!(seat.within, 0);
            assert!(!seat.overflow);
        }
        // Anuradha, the seventeenth, is Saturn's.
        assert_eq!(
            VIMSHOTTARI.lords[VIMSHOTTARI.seat(16).lord].graha,
            Graha::Saturn
        );
    }

    #[test]
    fn a_wide_window_a_backward_count_and_an_offset_each_move_the_seat() {
        let lords = VIMSHOTTARI.lords;
        let wide = UduRow {
            reference: 5,
            span: 3,
            lords: &lords[..8],
            repeats: false,
            ..VIMSHOTTARI
        };
        // Ardra itself starts the first lord's window; two past it is still
        // the first lord, two nakshatras into its window.
        assert_eq!(
            wide.seat(5),
            Seat {
                lord: 0,
                within: 0,
                overflow: false
            }
        );
        assert_eq!(
            wide.seat(7),
            Seat {
                lord: 0,
                within: 2,
                overflow: false
            }
        );
        assert_eq!(wide.seat(8).lord, 1);
        // Eight lords of three cover 24; the three before Ardra overflow.
        assert!(wide.seat(4).overflow);
        let backward = UduRow {
            reference: 26,
            count: Count::ToReference,
            ..VIMSHOTTARI
        };
        assert_eq!(backward.seat(26).lord, 0);
        assert_eq!(backward.seat(25).lord, 1);
        let offset = UduRow {
            offset: 3,
            ..VIMSHOTTARI
        };
        assert_eq!(offset.seat(0).lord, 3);
    }

    #[test]
    fn a_row_is_refused_by_the_field_it_gets_wrong() {
        const IDLE: &[Lord] = &[lord(Graha::Sun, 0)];
        let empty = UduRow {
            lords: &[],
            ..VIMSHOTTARI
        };
        assert_eq!(empty.validate().unwrap_err().field(), Some("lords"));
        let far = UduRow {
            reference: 27,
            ..VIMSHOTTARI
        };
        assert_eq!(far.validate().unwrap_err().field(), Some("reference"));
        let wide = UduRow {
            span: 4,
            ..VIMSHOTTARI
        };
        assert_eq!(wide.validate().unwrap_err().field(), Some("span"));
        let idle = UduRow {
            lords: IDLE,
            ..VIMSHOTTARI
        };
        assert_eq!(idle.validate().unwrap_err().field(), Some("lords[0].years"));
    }
}
