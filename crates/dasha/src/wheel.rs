//! The circle a nakshatra-seeded system counts its seed round: the
//! twenty-seven nakshatras, or the twenty-eight with Abhijit that BPHS
//! ch. 46 counts Ashtottari and Shashtihayani over
//! (`03-design/dasha-kernels.md`, "The 28-nakshatra wheel").
//!
//! Abhijit is not a catalogue member, since a chart has twenty-seven
//! nakshatras. It is a segment of this wheel, cut from its neighbours as the
//! translation of v. 22 cuts it: Uttarashadha's fourth pada and the first
//! fifteenth of Shravana. Uttarashadha keeps its first three padas and
//! Shravana the rest. Every bound is a whole number of nanoarcseconds, so a
//! seed is placed exactly.
//!
//! ```
//! use teistro_core::angle::Nas;
//! use teistro_dasha::Wheel;
//!
//! // 278°, inside Abhijit: the twenty-second segment of the twenty-eight.
//! let segment = Wheel::WithAbhijit.segment(Nas::new(278 * Nas::PER_DEGREE));
//! assert_eq!(segment.place, 21);
//! // On the twenty-seven it is Uttarashadha, the twenty-first.
//! assert_eq!(Wheel::Nakshatras.segment(Nas::new(278 * Nas::PER_DEGREE)).place, 20);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::angle::Nas;

/// Uttarashadha's place among the twenty-seven, the nakshatra Abhijit is cut
/// from.
const UTTARASHADHA: i64 = 20;

/// A pada, a quarter of a nakshatra.
const PADA: i64 = Nas::PER_NAKSHATRA / 4;

/// Where Abhijit begins: Uttarashadha's fourth pada, 276°40′.
const ABHIJIT_FROM: i64 = UTTARASHADHA * Nas::PER_NAKSHATRA + 3 * PADA;

/// Where it ends: a fifteenth of the way into Shravana, 280°53′20″.
const ABHIJIT_TO: i64 = (UTTARASHADHA + 1) * Nas::PER_NAKSHATRA + Nas::PER_NAKSHATRA / 15;

/// Which circle a system counts over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Wheel {
    /// The twenty-seven nakshatras, each 13°20′.
    #[default]
    Nakshatras,
    /// The twenty-eight with Abhijit, cut from Uttarashadha's last pada and
    /// Shravana's first fifteenth (BPHS ch. 46, the note to v. 22).
    WithAbhijit,
}

/// One segment of a wheel the Moon stands in: its place round the wheel,
/// its bounds, and how far into it the Moon is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    /// Its place, 0 for Ashwini.
    pub place: u8,
    /// Where it begins.
    pub from: Nas,
    /// Where it ends, exclusive; the circle's end for the last.
    pub to: i64,
    /// The Moon's distance past its beginning.
    pub past: i64,
}

impl Segment {
    /// How far through the segment the Moon is, 0 to 1.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "both are below a nakshatra's nanoarcseconds, 4.8e13, far inside the 2^53 a double holds exactly"
    )]
    pub fn elapsed(self) -> f64 {
        let width = self.to - self.from.get();
        if width <= 0 {
            return 0.0;
        }
        self.past as f64 / width as f64
    }

    /// Its bounds in degrees, for a search of when the Moon crossed them.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "a circle's nanoarcseconds, 1.3e18, round to the nearest double: a nanoarcsecond's error on a bound"
    )]
    pub fn degrees(self) -> (f64, f64) {
        let per_degree = Nas::PER_DEGREE as f64;
        (
            self.from.get() as f64 / per_degree,
            self.to as f64 / per_degree,
        )
    }
}

impl Wheel {
    /// How many segments it has.
    #[must_use]
    pub const fn size(self) -> u8 {
        match self {
            Wheel::Nakshatras => 27,
            Wheel::WithAbhijit => 28,
        }
    }

    /// A catalogue nakshatra's place on this wheel, 0 for Ashwini: past
    /// Abhijit every nakshatra is one further round.
    #[must_use]
    pub const fn place_of(self, nakshatra: u8) -> u8 {
        match self {
            Wheel::WithAbhijit if nakshatra as i64 > UTTARASHADHA => nakshatra + 1,
            _ => nakshatra,
        }
    }

    /// The segment a longitude falls in.
    #[must_use]
    pub fn segment(self, at: Nas) -> Segment {
        let nas = at.get();
        let nakshatra = nas / Nas::PER_NAKSHATRA;
        let whole = |index: i64| Segment {
            place: u8::try_from(index).unwrap_or_default(),
            from: Nas::new(index * Nas::PER_NAKSHATRA),
            to: (index + 1) * Nas::PER_NAKSHATRA,
            past: nas - index * Nas::PER_NAKSHATRA,
        };
        if self == Wheel::Nakshatras {
            return whole(nakshatra);
        }
        let place = |index: i64| u8::try_from(index).unwrap_or_default();
        match nas {
            _ if nas < UTTARASHADHA * Nas::PER_NAKSHATRA => whole(nakshatra),
            _ if nas < ABHIJIT_FROM => Segment {
                place: place(UTTARASHADHA),
                to: ABHIJIT_FROM,
                ..whole(UTTARASHADHA)
            },
            _ if nas < ABHIJIT_TO => Segment {
                place: place(UTTARASHADHA + 1),
                from: Nas::new(ABHIJIT_FROM),
                to: ABHIJIT_TO,
                past: nas - ABHIJIT_FROM,
            },
            _ if nas < (UTTARASHADHA + 2) * Nas::PER_NAKSHATRA => Segment {
                place: place(UTTARASHADHA + 2),
                from: Nas::new(ABHIJIT_TO),
                to: (UTTARASHADHA + 2) * Nas::PER_NAKSHATRA,
                past: nas - ABHIJIT_TO,
            },
            _ => Segment {
                place: place(nakshatra + 1),
                ..whole(nakshatra)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEGREE: i64 = Nas::PER_DEGREE;

    /// Every segment of each wheel follows the last without a gap, and the
    /// last ends at the circle.
    #[test]
    fn each_wheel_tiles_the_circle() {
        for wheel in [Wheel::Nakshatras, Wheel::WithAbhijit] {
            let mut at = 0;
            for place in 0..wheel.size() {
                let segment = wheel.segment(Nas::new(at));
                assert_eq!(segment.place, place, "{wheel:?} at {at}");
                assert_eq!(segment.from.get(), at);
                assert_eq!(segment.past, 0);
                at = segment.to;
            }
            assert_eq!(at, Nas::CIRCLE, "{wheel:?}");
        }
    }

    /// Abhijit's bounds are the ones the translation gives: Uttarashadha's
    /// three padas, 4°13′20″ of Abhijit, and Shravana's rest.
    #[test]
    fn abhijit_is_cut_where_the_translation_cuts_it() {
        let wheel = Wheel::WithAbhijit;
        let arcminutes = |nas: i64| nas / Nas::PER_ARCMINUTE;
        let uttarashadha = wheel.segment(Nas::new(270 * DEGREE));
        assert_eq!(uttarashadha.place, 20);
        assert_eq!(arcminutes(uttarashadha.to), 276 * 60 + 40);
        let abhijit = wheel.segment(Nas::new(278 * DEGREE));
        assert_eq!(abhijit.place, 21);
        assert_eq!(
            abhijit.to - abhijit.from.get(),
            4 * DEGREE + 13 * 60 * Nas::PER_ARCSECOND + 20 * Nas::PER_ARCSECOND
        );
        let shravana = wheel.segment(Nas::new(285 * DEGREE));
        assert_eq!(shravana.place, 22);
        assert_eq!(shravana.to, 22 * Nas::PER_NAKSHATRA);
        assert_eq!(wheel.segment(Nas::new(359 * DEGREE)).place, 27);
        assert_eq!(wheel.place_of(20), 20);
        assert_eq!(wheel.place_of(21), 22);
        assert_eq!(Wheel::Nakshatras.place_of(21), 21);
        let (from, to) = abhijit.degrees();
        assert!((from - (276.0 + 40.0 / 60.0)).abs() < 1e-12);
        assert!((to - (280.0 + 53.0 / 60.0 + 20.0 / 3600.0)).abs() < 1e-12);
    }
}
