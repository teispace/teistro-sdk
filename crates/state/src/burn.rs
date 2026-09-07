//! Combustion: how near the Sun a body is, and what that makes it.
//!
//! An orb per body, and in one of the two shipped tables a deeper orb
//! inside it; both differ by whether the body is retrograde. The corpus
//! does not state the orb — it **brackets** it, between the widest
//! reading that is combust and the nearest that is not — and every
//! bracket contains the classical value
//! (`03-design/state-tables-measured.md` §3).
//!
//! The table is a **setting** and not a constant, because the traditions
//! divide over how deep the burning goes and an implementation that
//! hard-codes one cannot say which it used. Two ship:
//!
//! | key | orb | deeper orb |
//! |---|---|---|
//! | [`SURYA_SIDDHANTA`] | the degrees of time (IX.6 to 8; the Moon's, X.1) | none: the text gives none |
//! | [`BPHS`] | the same six numbers | an inner orb the corpus brackets |
//!
//! The two agree on the outer orb — the classical degrees of time are the
//! same numbers in both — and part on what lies inside it. So under
//! `SURYA_SIDDHANTA`, which the default profile names (ADR-0024, "the
//! texts as read"), a body is combust or it is not: [`Burning::Deep`]
//! never arises, because no text the profile cites says where deep
//! begins. A caller who wants that reading asks for [`BPHS`], and
//! [`Combustion::orbs`] reports which orbs the answer was judged
//! against, so a reading always carries its own provenance.

use serde::Serialize;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

/// The Surya Siddhanta's degrees of time, by the key
/// `state.combustion_orbs` names: an orb and nothing inside it.
pub const SURYA_SIDDHANTA: &str = "SURYA_SIDDHANTA";

/// The same orbs with a deeper one inside each, by the key
/// `state.combustion_orbs` names.
pub const BPHS: &str = "BPHS";

/// Every table the SDK ships, in the order [`table`] names them when it
/// refuses one it does not have.
pub const SHIPPED: [&str; 2] = [SURYA_SIDDHANTA, BPHS];

/// The orbs one reading was judged against, degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Applied {
    /// Combust inside this.
    pub orb_deg: f64,
    /// Deeply combust inside this, where the table gives a deeper orb.
    pub deep_deg: Option<f64>,
}

/// One body's orbs, degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Orbs {
    /// Which body burns at these distances.
    pub graha: Graha,
    /// Combust inside this, moving forward.
    pub direct: f64,
    /// Combust inside this, moving backward.
    pub retrograde: f64,
    /// Deeply combust inside this, moving forward.
    pub deep_direct: Option<f64>,
    /// Deeply combust inside this, moving backward.
    pub deep_retrograde: Option<f64>,
}

impl Orbs {
    /// A row of a table that gives one orb per motion and nothing
    /// inside it.
    #[must_use]
    pub const fn plain(graha: Graha, direct: f64, retrograde: f64) -> Orbs {
        Orbs {
            graha,
            direct,
            retrograde,
            deep_direct: None,
            deep_retrograde: None,
        }
    }

    /// A row that gives a deeper orb inside each.
    #[must_use]
    pub const fn deep(graha: Graha, direct: (f64, f64), retrograde: (f64, f64)) -> Orbs {
        Orbs {
            graha,
            direct: direct.0,
            retrograde: retrograde.0,
            deep_direct: Some(direct.1),
            deep_retrograde: Some(retrograde.1),
        }
    }

    /// The orbs that apply to a body going one way or the other.
    #[must_use]
    pub const fn for_motion(&self, retrograde: bool) -> Applied {
        if retrograde {
            Applied {
                orb_deg: self.retrograde,
                deep_deg: self.deep_retrograde,
            }
        } else {
            Applied {
                orb_deg: self.direct,
                deep_deg: self.deep_direct,
            }
        }
    }
}

/// The Surya Siddhanta's degrees of time (IX.6 to 8; the Moon's, X.1):
/// Jupiter 11, Saturn 15, Mars 17; Venus 10 direct and 8 retrograde;
/// Mercury 14 direct and 12 retrograde; the Moon 12. The text gives no
/// orb inside those, so neither does this table.
///
/// They are the same numbers `teistro_astro::visibility::Table`
/// carries for heliacal visibility, and a test holds the two together.
///
/// The Sun burns nothing and the shadow grahas do not burn, so neither
/// appears.
pub const SURYA_SIDDHANTA_ORBS: [Orbs; 6] = [
    Orbs::plain(Graha::Moon, 12.0, 12.0),
    Orbs::plain(Graha::Mars, 17.0, 17.0),
    Orbs::plain(Graha::Mercury, 14.0, 12.0),
    Orbs::plain(Graha::Jupiter, 11.0, 11.0),
    Orbs::plain(Graha::Venus, 10.0, 8.0),
    Orbs::plain(Graha::Saturn, 15.0, 15.0),
];

/// The same six orbs with a deeper one inside each, which is the reading
/// the corpus's brackets contain: the recording engine burns deeply
/// somewhere inside every one of them (§3 of the measured page).
pub const BPHS_ORBS: [Orbs; 6] = [
    Orbs::deep(Graha::Moon, (12.0, 6.0), (12.0, 6.0)),
    Orbs::deep(Graha::Mars, (17.0, 8.0), (17.0, 8.0)),
    Orbs::deep(Graha::Mercury, (14.0, 7.0), (12.0, 6.0)),
    Orbs::deep(Graha::Jupiter, (11.0, 5.0), (11.0, 5.0)),
    Orbs::deep(Graha::Venus, (10.0, 5.0), (8.0, 4.0)),
    Orbs::deep(Graha::Saturn, (15.0, 6.0), (15.0, 6.0)),
];

/// How badly the Sun burns a body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Burning {
    /// Far enough from the Sun to be itself.
    None,
    /// Combust.
    Combust,
    /// Deeply combust. Only a table that gives a deeper orb ever
    /// reaches it.
    Deep,
}

/// What the Sun does to one body.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Combustion {
    /// How badly it burns.
    pub burning: Burning,
    /// How far from the Sun it stands, degrees, or `None` when the chart
    /// carries no Sun — in which case nothing is burnt and the value
    /// says why rather than claiming the sky is clear.
    pub from_sun_deg: Option<f64>,
    /// The orbs it was judged against. `None` for a body that does not
    /// burn at all.
    pub orbs: Option<Applied>,
}

impl Combustion {
    /// Nothing burns, because nothing could.
    #[must_use]
    pub const fn clear(from_sun_deg: Option<f64>) -> Combustion {
        Combustion {
            burning: Burning::None,
            from_sun_deg,
            orbs: None,
        }
    }

    /// Whether the body is burnt at all.
    #[must_use]
    pub const fn is_combust(&self) -> bool {
        !matches!(self.burning, Burning::None)
    }
}

/// The table a settings key names, matched without regard to case.
///
/// # Errors
///
/// `UNSUPPORTED` for a table the SDK does not ship, naming the ones it
/// does.
pub fn table(key: &str) -> Result<&'static [Orbs], Error> {
    if key.eq_ignore_ascii_case(SURYA_SIDDHANTA) {
        Ok(&SURYA_SIDDHANTA_ORBS)
    } else if key.eq_ignore_ascii_case(BPHS) {
        Ok(&BPHS_ORBS)
    } else {
        Err(Error::unsupported(format!(
            "no combustion table `{key}`; the SDK ships {}",
            SHIPPED
                .iter()
                .map(|shipped| format!("`{shipped}`"))
                .collect::<Vec<_>>()
                .join(" and ")
        ))
        .with_field("state.combustion_orbs"))
    }
}

/// How the Sun treats one body.
#[must_use]
pub fn combustion(
    graha: Graha,
    from_sun_deg: Option<f64>,
    retrograde: bool,
    table: &[Orbs],
) -> Combustion {
    let Some(from_sun) = from_sun_deg else {
        return Combustion::clear(None);
    };
    let Some(orbs) = table.iter().find(|orbs| orbs.graha == graha) else {
        return Combustion::clear(Some(from_sun));
    };
    let applied = orbs.for_motion(retrograde);
    let burning = if applied.deep_deg.is_some_and(|deep| from_sun < deep) {
        Burning::Deep
    } else if from_sun < applied.orb_deg {
        Burning::Combust
    } else {
        Burning::None
    };
    Combustion {
        burning,
        from_sun_deg: Some(from_sun),
        orbs: Some(applied),
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]
    #![allow(
        clippy::float_cmp,
        reason = "the tables are exact constants, and drifting by a bit is the failure being tested for"
    )]

    use super::{
        Applied, BPHS, BPHS_ORBS, Burning, SHIPPED, SURYA_SIDDHANTA, SURYA_SIDDHANTA_ORBS,
        combustion, table,
    };
    use teistro_core::catalogue::Graha;

    #[test]
    fn a_deep_orb_is_inside_its_own_and_every_body_has_both() {
        for orbs in BPHS_ORBS {
            assert!(orbs.deep_direct.unwrap() < orbs.direct, "{:?}", orbs.graha);
            assert!(
                orbs.deep_retrograde.unwrap() <= orbs.retrograde,
                "{:?}",
                orbs.graha
            );
            assert!(orbs.retrograde <= orbs.direct, "{:?}", orbs.graha);
        }
        // Neither the Sun nor a shadow burns, under either table.
        for shipped in SHIPPED {
            let orbs = table(shipped).expect("a shipped table");
            for graha in [Graha::Sun, Graha::Rahu, Graha::Ketu] {
                assert!(!orbs.iter().any(|orbs| orbs.graha == graha), "{shipped}");
            }
        }
    }

    #[test]
    fn the_two_tables_share_an_orb_and_part_inside_it() {
        for (text, parashari) in SURYA_SIDDHANTA_ORBS.iter().zip(&BPHS_ORBS) {
            assert_eq!(text.graha, parashari.graha, "the same rows in order");
            assert_eq!(text.direct, parashari.direct, "{:?}", text.graha);
            assert_eq!(text.retrograde, parashari.retrograde, "{:?}", text.graha);
            assert_eq!(text.deep_direct, None, "the text gives no deeper orb");
            assert_eq!(text.deep_retrograde, None);
            assert!(parashari.deep_direct.is_some(), "{:?}", parashari.graha);
        }
    }

    /// One text, one set of numbers. The astro crate reads them as the
    /// arc that hides a body at the horizon and this one as the arc the
    /// Sun burns it in; they must not drift apart in the source.
    #[test]
    fn the_texts_orbs_are_the_degrees_of_time_the_astro_crate_reads() {
        use teistro_astro::visibility::{Motion, Table};
        use teistro_port_ephemeris::Body;

        for body in [
            Body::Moon,
            Body::Mars,
            Body::Mercury,
            Body::Jupiter,
            Body::Venus,
            Body::Saturn,
        ] {
            let graha = body.graha().expect("a catalogued graha");
            let row = SURYA_SIDDHANTA_ORBS
                .iter()
                .find(|orbs| orbs.graha == graha)
                .expect("a row per body of the table");
            assert_eq!(
                Table::SURYA_SIDDHANTA.of(body, Motion::Direct),
                Some(row.direct),
                "{graha:?} direct"
            );
            assert_eq!(
                Table::SURYA_SIDDHANTA.of(body, Motion::Retrograde),
                Some(row.retrograde),
                "{graha:?} retrograde"
            );
        }
    }

    #[test]
    fn the_orbs_decide_the_three_states() {
        let orbs = table(BPHS).expect("the shipped table");
        // Mercury direct: deep inside 7°, combust inside 14°.
        let deep = combustion(Graha::Mercury, Some(3.0), false, orbs);
        assert_eq!(deep.burning, Burning::Deep);
        assert!(deep.is_combust());
        assert_eq!(
            deep.orbs,
            Some(Applied {
                orb_deg: 14.0,
                deep_deg: Some(7.0)
            })
        );
        assert_eq!(
            combustion(Graha::Mercury, Some(10.0), false, orbs).burning,
            Burning::Combust
        );
        assert_eq!(
            combustion(Graha::Mercury, Some(14.0), false, orbs).burning,
            Burning::None,
            "the orb's own edge is clear"
        );
        // Retrograde it burns sooner.
        assert_eq!(
            combustion(Graha::Mercury, Some(13.0), true, orbs).burning,
            Burning::None
        );
        assert_eq!(
            combustion(Graha::Mercury, Some(6.5), true, orbs).burning,
            Burning::Combust,
            "and its deep orb shrinks to six"
        );
    }

    #[test]
    fn a_table_with_no_deeper_orb_never_burns_deeply() {
        let text = table(SURYA_SIDDHANTA).expect("the shipped table");
        for tenth in 0..200 {
            let from_sun = f64::from(tenth) / 10.0;
            let found = combustion(Graha::Mercury, Some(from_sun), false, text);
            assert_ne!(found.burning, Burning::Deep, "at {from_sun}°");
            assert_eq!(found.is_combust(), from_sun < 14.0, "at {from_sun}°");
            assert_eq!(
                found.orbs,
                Some(Applied {
                    orb_deg: 14.0,
                    deep_deg: None
                }),
                "and it says what it judged against"
            );
        }
    }

    #[test]
    fn a_body_that_cannot_burn_says_so_rather_than_claiming_it_is_clear() {
        let orbs = table(BPHS).expect("the shipped table");
        let sun = combustion(Graha::Sun, Some(0.0), false, orbs);
        assert_eq!(sun.burning, Burning::None);
        assert_eq!(sun.orbs, None, "no orb was applied");
        // And a chart with no Sun answers nothing rather than zero.
        let unknown = combustion(Graha::Mars, None, false, orbs);
        assert_eq!(unknown.from_sun_deg, None);
        assert_eq!(unknown.burning, Burning::None);
    }

    #[test]
    fn a_shipped_table_is_found_however_it_is_spelt() {
        for shipped in SHIPPED {
            assert!(table(shipped).is_ok(), "{shipped}");
            assert!(table(&shipped.to_lowercase()).is_ok(), "{shipped}");
        }
    }

    #[test]
    fn a_table_the_sdk_does_not_ship_is_refused_by_name() {
        let error = table("SOMEONE_ELSES").expect_err("no such table");
        assert!(error.message.contains("SOMEONE_ELSES"), "{error}");
        for shipped in SHIPPED {
            assert!(error.message.contains(shipped), "{error}");
        }
    }
}
