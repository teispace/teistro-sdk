//! Vishu's day, held to the Kerala Gazette (`docs/calendars/regional-solar.md`,
//! crux C1301): the first civil day whose sunrise at Thiruvananthapuram
//! follows the Mesha sankranti, which is the day after the sankranti's
//! sunrise-to-sunrise day. It is not Medam 1: the Malabar rule begins
//! Medam on the sankranti's own day when it falls in the first three
//! fifths of the daylight, and Vishu waits for the next sunrise.
//!
//! The festival rule language keeps a solar day by a calendar's date or by
//! the civil day, midnight to midnight, of the Sun's entry into a
//! nakshatra; neither is the sunrise day, so Vishu is measured here, over
//! the regional calendars' own sky, rather than shipped as a rule.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_calendar::bikram_sambat::engine::Engine;
use teistro_calendar::gregorian::gregorian_from_fixed;
use teistro_calendar::regional_solar::{IST, RegionalSolar, THIRUVANANTHAPURAM};
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarSystem, MonthStartRule};
use teistro_core::catalogue::{Ayanamsha, Calendar};
use teistro_core::settings::{AyanamshaBasis, OverridePolicy, Sunrise};
use teistro_ephemeris_builtin::provider::Builtin;

/// A Gregorian date: year, month, day.
type Ymd = (i32, u8, u8);

/// The gazette's Vishu, from the Kerala Gazette's public holiday
/// notifications for 2021 to 2026 (G.O.(P) No. 16/2020 to 14/2025/GAD),
/// with its Medam 1 the same notification prints.
const PRINTED: [(i32, Ymd, Ymd); 6] = [
    (2021, (2021, 4, 14), (2021, 4, 14)),
    (2022, (2022, 4, 15), (2022, 4, 14)),
    (2023, (2023, 4, 15), (2023, 4, 15)),
    (2024, (2024, 4, 14), (2024, 4, 14)),
    (2025, (2025, 4, 14), (2025, 4, 14)),
    (2026, (2026, 4, 15), (2026, 4, 14)),
];

#[test]
fn vishu_is_the_first_sunrise_after_the_mesha_sankranti() {
    let provider = Builtin::new();
    // The regional tables' sky: Lahiri on the true basis, the upper
    // limb with standard refraction, on IST.
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::UpperLimbRefraction.into(),
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    )
    .with_ayanamsha_basis(AyanamshaBasis::True);
    let clock = IST;
    // A year opening at Mesha, so month 1's start is the sankranti's
    // sunrise-to-sunrise day at Thiruvananthapuram.
    let engine = Engine::new(
        &model,
        &clock,
        THIRUVANANTHAPURAM,
        MonthStartRule::SunriseToSunrise,
    );
    let malayalam = RegionalSolar::shipped(Calendar::MalayalamSolar).unwrap();
    let mut parted = 0;
    for (year, vishu, medam) in PRINTED {
        // The engine's year opening at Mesha in `year`, counted as the
        // Bikram Sambat engine counts it (CE + 57).
        let row = engine.year(year + 57).unwrap();
        let day = row.start.plus_days(1);
        assert_eq!(gregorian_from_fixed(day), vishu, "Vishu {year}");
        let first = malayalam.to_fixed_ymd(year - 825, 9, 1).unwrap();
        assert_eq!(gregorian_from_fixed(first), medam, "Medam {year}");
        parted += usize::from(first != day);
    }
    // Vishu is Medam 2 in 2022 and 2026, when the sankranti fell in the
    // morning (08:41 and 09:33 IST), before the three-fifths mark.
    assert_eq!(parted, 2);
}
