//! The chart the kernel's unit tests build on, shared so each module's tests
//! place bodies the same way.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "test fixtures unwrap what they built and index their own chart"
)]

use teistro_core::catalogue::{Dignity, Rashi};

use crate::chart::{Placement, RuleChart};
use crate::language::{Body, House};

/// A house by number.
pub(crate) fn house(n: u8) -> House {
    House::try_new(n).unwrap()
}

/// Every body at 15° Aries in the first house, neutral, direct, clear.
pub(crate) fn chart() -> RuleChart {
    RuleChart {
        placements: [Placement {
            longitude: 15.0,
            sign: Rashi::Aries,
            house: house(1),
            dignity: Dignity::Neutral,
            retrograde: false,
            combust: false,
            karaka7: None,
            karaka8: None,
            navamsha: Rashi::Aries,
        }; 10],
        panchanga: None,
        strengths: None,
    }
}

/// A body moved to the middle of a sign, its house counted from Aries.
pub(crate) fn place(chart: &mut RuleChart, body: Body, sign: Rashi) {
    let p = &mut chart.placements[body.index()];
    p.sign = sign;
    p.longitude = f64::from(sign as u8) * 30.0 + 15.0;
    p.house = House::between(Rashi::Aries, sign);
}
