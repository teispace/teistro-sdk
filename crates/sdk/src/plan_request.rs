//! What a consumer asks a chart reading to say in words
//! (`03-design/plans-at-the-boundary.md`).
//!
//! A [`PlanRequest`] names the composers to run over every chart. It is the
//! record the C boundary's `interpret_json` carries, so Rust and every
//! binding read one type, and it is deliberately a set of named members
//! rather than a bit set: a composer will want options of its own, and a bit
//! set has nowhere to put them.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

/// Which narrative plans a chart reading is asked for.
///
/// ```
/// use teistro::PlanRequest;
///
/// let request = PlanRequest::from_json(r#"{"placements": true}"#)?;
/// assert_eq!(request, PlanRequest::default().with_placements());
/// assert!(request.asks_for_something());
///
/// // A composer that does not exist is refused rather than ignored.
/// let wrong = PlanRequest::from_json(r#"{"plcaements": true}"#).unwrap_err();
/// assert!(wrong.to_string().contains("placements"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
// **Why a bool a composer rather than a bit set.** `struct_excessive_bools` asks
// whether a bit set was meant; this design answered that before the fourth
// composer arrived, and the answer has not changed since. A composer will want options of its own — which rules to
// read, which house — and a bit set has nowhere to put them, while `sections`
// is a bit set because its members never will
// (`03-design/plans-at-the-boundary.md` §3). The members are also the JSON
// the boundary reads, one name a composer, so they are named in two senses.
#[expect(
    clippy::struct_excessive_bools,
    reason = "a named member per composer, decided in plans-at-the-boundary.md §3"
)]
pub struct PlanRequest {
    /// Where each of the nine grahas stands and who shares a sign.
    pub placements: bool,
    /// What each rule the chart held says — which needs rules to have been
    /// asked for, since a reading composes what rules answered.
    pub readings: bool,
    /// Each graha's Shadbala in rupas, the strongest first. It reads the
    /// Shadbala section, which the request computes for it.
    pub strength: bool,
    /// The lord of each of the twelve bhavas. It reads the houses section,
    /// which the request computes for it.
    pub houses: bool,
    /// Where each graha stands to the degree, which `placements` rounds
    /// away. It reads what `placements` reads, so it costs no section.
    pub positions: bool,
    /// Which graha looks at which, and how strongly. It reads the aspects
    /// section, which the request computes for it.
    pub aspects: bool,
    /// What each graha **is** where it stands: its dignity, its navamsha and
    /// whether that makes it vargottama, whether it is retrograde and
    /// whether the Sun burns it. It reads what `placements` reads, so it
    /// costs no section.
    pub conditions: bool,
    /// Which chara karaka each graha holds, under both schemes. It reads
    /// what `placements` reads, so it costs no section.
    pub karakas: bool,
    /// Where the two house readings put a graha in different bhavas: the
    /// placement system's and the chalit's. It reads the chart's own
    /// grahas, so it costs no section, and it says nothing of a chart
    /// whose readings agree.
    pub chalit: bool,
    /// What a loaded corpus of state readings says of this chart's
    /// subjects: a graha in a bhava, the lagna's sign, each limb of the
    /// panchanga, and what the birth nakshatra is. It costs no section,
    /// and it says **nothing** until a pack carrying those readings is
    /// loaded (`03-design/state-readings.md`).
    pub phala: bool,
    /// Each bhava's strength in virupas, the first house first. It reads
    /// `Document.bhava_bala`, so it costs that section, and it says the
    /// weight and never a verdict: a `BhavaStrength` carries no
    /// requirement, so there is nothing to be short of.
    pub bhava_bala: bool,
    /// Each graha's Vimshopaka under all four schemes, each item naming
    /// the scheme it belongs to. It reads `Document.vimshopaka`, so it
    /// costs that section.
    pub vimshopaka: bool,
    /// The almanac of the chart's day: the tithi with its paksha, the
    /// vara, the nakshatra and the Moon's pada in it, the yoga and the
    /// karana, and whether the birth fell by day where the chart says.
    /// It reads the panchanga the rules read, so it costs the same
    /// section they do.
    pub panchanga: bool,
    /// The other half of a graha's state: how it stands to its dispositor
    /// under all three friendships, and the four avasthas — the fifth of
    /// its sign, its wakefulness, its brightness where the chart decides
    /// one, and the lajjitadi that hold beside the ones nothing decides.
    /// It reads what `conditions` reads, so it costs no section beyond it.
    pub states: bool,
    /// What each graha's placement says of its dasha: when in the dasha
    /// its effects come, whether its place is auspicious, the points its
    /// dignity earns and whether the placement makes the dasha
    /// favourable — with the reading a loaded corpus carries of that
    /// graha as a dasha lord. It reads `Document.dasha_phala`, so it
    /// **costs that section**: ask for it with
    /// [`ChartRequest::with_dasha_phala`](crate::ChartRequest::with_dasha_phala).
    pub dasha_phala: bool,
}

impl PlanRequest {
    /// Every composer this request can name, which is every member it has.
    ///
    /// The boundary reads these names out of JSON, so they are API; a test
    /// holds the list against the record's own serialisation, both ways, so
    /// a composer added without a name here fails rather than going
    /// unmentioned in the refusal a typo earns.
    pub const MEMBERS: [&'static str; 15] = [
        "placements",
        "readings",
        "strength",
        "houses",
        "positions",
        "aspects",
        "conditions",
        "karakas",
        "chalit",
        "phala",
        "bhavaBala",
        "vimshopaka",
        "panchanga",
        "states",
        "dashaPhala",
    ];

    /// A request for each bhava's strength.
    #[must_use]
    pub const fn with_bhava_bala(mut self) -> PlanRequest {
        self.bhava_bala = true;
        self
    }

    /// A request for each graha's Vimshopaka.
    #[must_use]
    pub const fn with_vimshopaka(mut self) -> PlanRequest {
        self.vimshopaka = true;
        self
    }

    /// A request for the almanac of the chart's day.
    #[must_use]
    pub const fn with_panchanga(mut self) -> PlanRequest {
        self.panchanga = true;
        self
    }

    /// A request for the other half of each graha's state.
    #[must_use]
    pub const fn with_states(mut self) -> PlanRequest {
        self.states = true;
        self
    }

    /// A request for what a placement says of its dasha. It needs the
    /// dasha phala section beside it.
    #[must_use]
    pub const fn with_dasha_phala(mut self) -> PlanRequest {
        self.dasha_phala = true;
        self
    }

    /// A request for the placements.
    #[must_use]
    pub const fn with_placements(mut self) -> PlanRequest {
        self.placements = true;
        self
    }

    /// A request for the readings. It needs a rule request beside it.
    #[must_use]
    pub const fn with_readings(mut self) -> PlanRequest {
        self.readings = true;
        self
    }

    /// A request for the strengths.
    #[must_use]
    pub const fn with_strength(mut self) -> PlanRequest {
        self.strength = true;
        self
    }

    /// A request for the houses' lords.
    #[must_use]
    pub const fn with_houses(mut self) -> PlanRequest {
        self.houses = true;
        self
    }

    /// A request for the positions, to the degree.
    #[must_use]
    pub const fn with_positions(mut self) -> PlanRequest {
        self.positions = true;
        self
    }

    /// A request for the drishtis.
    #[must_use]
    pub const fn with_aspects(mut self) -> PlanRequest {
        self.aspects = true;
        self
    }

    /// A request for each graha's conditions.
    #[must_use]
    pub const fn with_conditions(mut self) -> PlanRequest {
        self.conditions = true;
        self
    }

    /// A request for the chara karakas.
    #[must_use]
    pub const fn with_karakas(mut self) -> PlanRequest {
        self.karakas = true;
        self
    }

    /// A request for where the two house readings disagree.
    #[must_use]
    pub const fn with_chalit(mut self) -> PlanRequest {
        self.chalit = true;
        self
    }

    /// A request for what a loaded corpus says of the chart's subjects.
    #[must_use]
    pub const fn with_phala(mut self) -> PlanRequest {
        self.phala = true;
        self
    }

    /// Whether any composer was asked for, so a caller can skip the work
    /// rather than compose an empty answer.
    #[must_use]
    pub const fn asks_for_something(self) -> bool {
        self.placements
            || self.readings
            || self.strength
            || self.houses
            || self.positions
            || self.aspects
            || self.conditions
            || self.karakas
            || self.chalit
            || self.phala
            || self.bhava_bala
            || self.vimshopaka
            || self.panchanga
            || self.states
            || self.dasha_phala
    }

    /// A request read from JSON.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for JSON that is not a request, and for a composer that
    /// does not exist — which serde names beside the ones that do, because a
    /// member silently composing nothing is the dead end a typo deserves to
    /// be caught by.
    pub fn from_json(text: &str) -> Result<PlanRequest, Error> {
        serde_json::from_str(text).map_err(|err| {
            Error::invalid_arg(format!("the plan request does not read: {err}")).with_hint(format!(
                "an object of `{}`",
                PlanRequest::MEMBERS.join("`, `")
            ))
        })
    }

    /// The request checked against what else was asked for.
    ///
    /// `readings` composes what rules answered, so without a rule request
    /// there is nothing for it to say — and an empty plan would tell the
    /// consumer nothing about why. A set of rules none of which hold is not
    /// this case: that composes to a plan with no items, which is an answer.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `readings` where it is asked for without rules.
    pub fn check(self, has_rules: bool) -> Result<(), Error> {
        if self.readings && !has_rules {
            return Err(Error::invalid_arg(
                "`readings` says what the rules a chart held answer, so it needs rules to \
                 answer",
            )
            .with_field("readings")
            .with_hint("name the rules in the rule request beside this one"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use super::PlanRequest;

    /// The names the boundary reads are the record's own members, held both
    /// ways: a composer added without a name here, or a name here that no
    /// member answers to, fails. It is the same shape as `KEYS` against the
    /// packs, for the same reason — the list is what a refusal quotes, and a
    /// list written by hand beside a struct is the claim that rots.
    #[test]
    fn every_member_is_named_and_every_name_is_a_member() {
        let serialised = serde_json::to_value(PlanRequest::default()).unwrap();
        let members: Vec<&str> = serialised
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        let mut named = PlanRequest::MEMBERS.to_vec();
        named.sort_unstable();
        let mut sorted = members.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, named, "MEMBERS and the record disagree");
    }

    /// Any one member is enough to ask for something, so a composer added
    /// without being counted in `asks_for_something` fails here rather than
    /// composing nothing at the boundary.
    #[test]
    fn any_member_asks_for_something() {
        assert!(!PlanRequest::default().asks_for_something());
        for member in PlanRequest::MEMBERS {
            let request = PlanRequest::from_json(&format!(r#"{{"{member}": true}}"#)).unwrap();
            assert!(request.asks_for_something(), "`{member}` asks for nothing");
        }
    }

    /// A typo is refused, and the refusal names what it could have meant.
    #[test]
    fn a_name_that_is_not_a_member_is_refused_with_the_ones_that_are() {
        let wrong = PlanRequest::from_json(r#"{"karakaz": true}"#).unwrap_err();
        let said = wrong.to_string();
        for member in PlanRequest::MEMBERS {
            assert!(said.contains(member), "`{member}` unmentioned in {said}");
        }
    }
}
