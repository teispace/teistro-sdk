//! What the chart's subjects mean: the reading a loaded pack carries for
//! a graha in a bhava, for the lagna's sign, and for each limb of the
//! panchanga (`03-design/state-readings.md` §5).
//!
//! Every other composer says what the SDK **computed**. This one says what
//! a corpus **carries**, so it is silent unless a pack of state readings
//! has been loaded — a chart composes to the same plan as before until a
//! consumer asks for the words.
//!
//! It asks the base locale rather than the reader's, as `readings` does
//! and for the same reason: a plan whose shape changed with the reader
//! would not be language-neutral, and the same chart would compose
//! differently for two people.

use teistro_core::catalogue::Graha;
use teistro_intl::messages::sdk::phala;
use teistro_intl::source::graha_bhava_key;
use teistro_rules::{Body, RuleChart};

use crate::{NAME_FORM, Plan, Vocabulary};

/// The form a subject's own reading is carried under, where the record is
/// one the SDK already names and the reading is a form on it.
const PHALA_FORM: &str = "phala";

/// The form the lagna's sign carries its reading under.
///
/// Its own and not `phala`, because a sign is read one way as a lagna and
/// another as a subject: `rashi.ARIES` carries both, and a corpus that put
/// them under one name would have to choose.
const LAGNA_FORM: &str = "lagnaPhala";

/// What a loaded corpus says of this chart's subjects.
///
/// The order is the chart's: the nine grahas in their houses, the lagna's
/// sign, then the panchanga's tithi, vara, nakshatra and yoga. Nothing is
/// said for a subject the base locale carries no reading of, so the plan
/// grows with the packs a consumer loads and with nothing else.
#[must_use]
pub fn phala(chart: &RuleChart, vocabulary: &dyn Vocabulary) -> Plan {
    let mut plan = Plan::default();
    for graha in Graha::ALL.into_iter().take(9) {
        let Some(at) = chart.placements.get(Body::Graha(graha).index()) else {
            continue;
        };
        let key = graha_bhava_key(graha.key(), at.house.get());
        if vocabulary.has_form(&key, NAME_FORM) {
            plan.say(&phala::GrahaInBhava {
                graha,
                bhava: i64::from(at.house.get()),
                phala: key,
            });
        }
    }
    if let Some(at) = chart.placements.get(Body::Lagna.index())
        && vocabulary.has_form(at.sign.full_key(), LAGNA_FORM)
    {
        plan.say(&phala::LagnaRashi { rashi: at.sign });
    }
    let Some(panchanga) = chart.panchanga.as_ref() else {
        return plan;
    };
    if vocabulary.has_form(panchanga.tithi.full_key(), PHALA_FORM) {
        plan.say(&phala::Tithi {
            tithi: panchanga.tithi,
        });
    }
    if vocabulary.has_form(panchanga.vara.full_key(), PHALA_FORM) {
        plan.say(&phala::Vara {
            vara: panchanga.vara,
        });
    }
    if vocabulary.has_form(panchanga.nakshatra.full_key(), PHALA_FORM) {
        plan.say(&phala::Nakshatra {
            nakshatra: panchanga.nakshatra,
        });
    }
    if vocabulary.has_form(panchanga.yoga.full_key(), PHALA_FORM) {
        plan.say(&phala::Yoga {
            yoga: panchanga.yoga,
        });
    }
    plan
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_core::catalogue::{Dignity, Rashi};
    use teistro_rules::{House, Placement, RuleChart};

    use super::phala;
    use crate::{NoReadings, Vocabulary};

    /// A vocabulary carrying exactly the keys and forms it was given.
    struct Some(&'static [(&'static str, &'static str)]);

    impl Vocabulary for Some {
        fn has_form(&self, key: &str, form: &str) -> bool {
            self.0.iter().any(|(k, f)| *k == key && *f == form)
        }
    }

    fn chart() -> RuleChart {
        let placement = Placement {
            longitude: 15.0,
            sign: Rashi::Aries,
            house: House::try_new(1).unwrap(),
            dignity: Dignity::Neutral,
            retrograde: false,
            combust: false,
            karaka7: None,
            karaka8: None,
            navamsha: Rashi::Aries,
        };
        RuleChart {
            placements: [placement; 10],
            panchanga: None,
            strengths: None,
        }
    }

    /// A consumer that has loaded no corpus gets the plan it had before.
    #[test]
    fn nothing_is_said_without_a_corpus() {
        assert!(phala(&chart(), &NoReadings).is_empty());
    }

    /// And one that has loaded part of a corpus gets that part, keyed the
    /// way the migration writes it.
    #[test]
    fn what_the_corpus_carries_is_what_is_said() {
        let vocabulary = Some(&[
            ("graha_bhava.SUN_IN_1", "name"),
            ("graha_bhava.MOON_IN_1", "name"),
            ("rashi.ARIES", "lagnaPhala"),
        ]);
        let plan = phala(&chart(), &vocabulary);
        assert_eq!(plan.len(), 3);
        assert_eq!(plan.items[0].key, "sdk.phala.grahaInBhava");
        assert_eq!(plan.items[2].key, "sdk.phala.lagnaRashi");
        assert!(
            plan.items
                .iter()
                .all(|item| crate::KEYS.contains(&item.key.as_str()))
        );
    }
}
