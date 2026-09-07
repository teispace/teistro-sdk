//! The lunar month, under both conventions at once.
//!
//! An amanta month runs new moon to new moon and a purnimanta month full
//! moon to full moon, so through the dark fortnight the purnimanta month
//! is already the next amanta one. Measured on all 55 recorded days
//! (`03-design/panchanga-day-conventions.md` §6), and it is the whole of
//! the relation between them.
//!
//! `calendars.lunar_month` chooses which one a value leads with; the
//! other is carried beside it, because an application that shows one
//! usually has a reader who wants the other.
//!
//! What is **not** here: the intercalary month. The corpus marks two
//! adhika months and no kshaya month, which shows the field is computed
//! and does not test the rule that computes it. Adhika and kshaya belong
//! to the Indian lunisolar calendar; this module reads the month and does
//! not decide it.

use serde::Serialize;
use teistro_core::catalogue::{Masa, Paksha, Tithi};
use teistro_core::settings::LunarMonth as Convention;

/// The lunar month a day falls in, under both conventions.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct LunarMonth {
    /// The month under the profile's own convention.
    pub month: Masa,
    /// Which convention that is.
    pub convention: Convention,
    /// The amanta month: new moon to new moon.
    pub amanta: Masa,
    /// The purnimanta month: full moon to full moon.
    pub purnimanta: Masa,
    /// Which fortnight the day opens in.
    pub paksha: Paksha,
}

impl LunarMonth {
    /// The month under a convention, whichever the value leads with.
    #[must_use]
    pub const fn under(&self, convention: Convention) -> Masa {
        match convention {
            Convention::Purnimanta => self.purnimanta,
            _ => self.amanta,
        }
    }
}

/// The month of a day, from its amanta month and the tithi it opens in.
///
/// The amanta month is the solar month the new moon that began it fell
/// in, which the lunisolar calendar decides; this turns it into both
/// readings and the paksha.
#[must_use]
pub fn of(amanta: Masa, at_sunrise: Tithi, convention: Convention) -> LunarMonth {
    let paksha = at_sunrise.attributes().paksha;
    let purnimanta = if matches!(paksha, Paksha::Krishna) {
        next(amanta)
    } else {
        amanta
    };
    LunarMonth {
        month: match convention {
            Convention::Purnimanta => purnimanta,
            _ => amanta,
        },
        convention,
        amanta,
        purnimanta,
        paksha,
    }
}

/// The month after another.
#[must_use]
fn next(month: Masa) -> Masa {
    let count = u16::try_from(Masa::ALL.len()).unwrap_or(12);
    Masa::from_id((month.id() + 1) % count).unwrap_or(month)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::{next, of};
    use teistro_core::catalogue::{Masa, Paksha, Tithi};
    use teistro_core::settings::LunarMonth as Convention;

    #[test]
    fn the_purnimanta_month_runs_ahead_through_the_dark_fortnight() {
        // The bright fortnight: both conventions name the same month.
        let bright = of(Masa::Chaitra, Tithi::ShuklaPratipada, Convention::Amanta);
        assert_eq!(bright.amanta, Masa::Chaitra);
        assert_eq!(bright.purnimanta, Masa::Chaitra);
        assert_eq!(bright.paksha, Paksha::Shukla);

        // The dark one: the purnimanta month is already the next.
        let dark = of(Masa::Chaitra, Tithi::KrishnaPratipada, Convention::Amanta);
        assert_eq!(dark.amanta, Masa::Chaitra);
        assert_eq!(dark.purnimanta, Masa::Vaishakha);
        assert_eq!(dark.paksha, Paksha::Krishna);
        assert_eq!(dark.month, Masa::Chaitra, "the profile asked for amanta");
        assert_eq!(dark.under(Convention::Purnimanta), Masa::Vaishakha);
    }

    #[test]
    fn the_convention_decides_which_month_the_value_leads_with() {
        let dark = of(Masa::Chaitra, Tithi::KrishnaAshtami, Convention::Purnimanta);
        assert_eq!(dark.month, Masa::Vaishakha);
        assert_eq!(dark.convention, Convention::Purnimanta);
        assert_eq!(dark.under(Convention::Amanta), Masa::Chaitra);
    }

    #[test]
    fn the_year_turns() {
        assert_eq!(next(Masa::Phalguna), Masa::Chaitra);
        assert_eq!(next(Masa::Chaitra), Masa::Vaishakha);
        let turn = of(
            Masa::Phalguna,
            Tithi::KrishnaChaturdashi,
            Convention::Amanta,
        );
        assert_eq!(turn.purnimanta, Masa::Chaitra);
    }

    #[test]
    fn every_month_has_a_successor() {
        for month in Masa::ALL {
            assert_ne!(next(month), month);
        }
    }
}
