//! Every day of the supported range agrees with the `calendrical_calculations`
//! oracle (the Reingold and Dershowitz algorithms as ICU4X carries them).

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

use calendrical_calculations::rata_die::RataDie;
use calendrical_calculations::{gregorian, julian};
use teistro_calendar::fixed::FixedDay;
use teistro_calendar::gregorian::{YEAR_RANGE, fixed_from_gregorian, gregorian_from_fixed};
use teistro_calendar::julian::{fixed_from_julian, julian_from_fixed};

#[test]
fn gregorian_agrees_with_the_oracle_on_every_day() {
    let first = fixed_from_gregorian(YEAR_RANGE.0, 1, 1).get();
    let last = fixed_from_gregorian(YEAR_RANGE.1, 12, 31).get();
    for fixed in first..=last {
        let ours = gregorian_from_fixed(FixedDay::new(fixed));
        let theirs = gregorian::gregorian_from_fixed(RataDie::new(fixed)).unwrap();
        assert_eq!(ours, theirs, "fixed {fixed}");
        assert_eq!(
            gregorian::fixed_from_gregorian(ours.0, ours.1, ours.2).to_i64_date(),
            fixed
        );
    }
}

#[test]
fn julian_agrees_with_the_oracle_on_every_day() {
    let first = fixed_from_julian(YEAR_RANGE.0, 1, 1).get();
    let last = fixed_from_julian(YEAR_RANGE.1, 12, 31).get();
    for fixed in first..=last {
        let ours = julian_from_fixed(FixedDay::new(fixed));
        let theirs = julian::julian_from_fixed(RataDie::new(fixed)).unwrap();
        assert_eq!(ours, theirs, "fixed {fixed}");
        assert_eq!(
            julian::fixed_from_julian(ours.0, ours.1, ours.2).to_i64_date(),
            fixed
        );
    }
}
