//! What the chart's subjects mean: the reading a loaded pack carries for
//! a graha in a bhava, for the lagna's sign, for each limb of the
//! panchanga, and for the six things the birth nakshatra *is*
//! (`03-design/state-readings.md` §5).
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

/// The form a nakshatra's naming syllables are carried under.
///
/// Its own for the same reason as the lagna's: `nakshatra.ASHWINI` is read
/// one way as the birth star and another as the source of a child's name,
/// and the corpus keys both onto the one record.
const NAMAKARANA_FORM: &str = "namakarana";

/// What a loaded corpus says of this chart's subjects.
///
/// The order is the chart's: the nine grahas in their houses, the lagna's
/// sign, the panchanga's tithi, vara, nakshatra and yoga, and then what
/// the birth nakshatra is — its naming syllables, its gana, nadi, yoni,
/// varna and element. Nothing is said for a subject the base locale
/// carries no reading of, so the plan grows with the packs a consumer
/// loads and with nothing else.
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
    // What the birth nakshatra *is*, which the catalogue already answers.
    //
    // These are not a sixth limb and not a rule: every nakshatra carries
    // its gana, nadi, yoni, varna and element as attributes, so the
    // subject of each reading is settled by the same nakshatra the limb
    // above names — the Moon's, which is the janma nakshatra every text
    // reads these from. Nothing is chosen here and nothing is computed;
    // the corpus keys its reading onto `gana.DEVA` and the catalogue says
    // which gana this nakshatra has.
    let star = panchanga.nakshatra;
    if vocabulary.has_form(star.full_key(), NAMAKARANA_FORM) {
        plan.say(&phala::Namakarana { nakshatra: star });
    }
    let attributes = star.attributes();
    if vocabulary.has_form(attributes.gana.full_key(), PHALA_FORM) {
        plan.say(&phala::Gana {
            gana: attributes.gana,
        });
    }
    if vocabulary.has_form(attributes.nadi.full_key(), PHALA_FORM) {
        plan.say(&phala::Nadi {
            nadi: attributes.nadi,
        });
    }
    if vocabulary.has_form(attributes.yoni.full_key(), PHALA_FORM) {
        plan.say(&phala::Yoni {
            yoni: attributes.yoni,
        });
    }
    if vocabulary.has_form(attributes.varna.full_key(), PHALA_FORM) {
        plan.say(&phala::Varna {
            varna: attributes.varna,
        });
    }
    if vocabulary.has_form(attributes.element.full_key(), PHALA_FORM) {
        plan.say(&phala::Tatwa {
            tatwa: attributes.element,
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

    use teistro_core::catalogue::{
        Dignity, Gana, Karana, Nadi, Nakshatra, Rashi, Tatwa, Tithi, Vara, Varna, Yoga, Yoni,
    };
    use teistro_rules::{House, Pada, Panchanga, Placement, RuleChart, Spans};

    use super::phala;
    use crate::{NoReadings, Vocabulary};

    /// A vocabulary carrying exactly the keys and forms it was given.
    ///
    /// Named for what it does rather than `Some`, which shadowed
    /// `Option::Some` inside this module and made a panchanga unbuildable.
    struct Carries(&'static [(&'static str, &'static str)]);

    impl Vocabulary for Carries {
        fn has_form(&self, key: &str, form: &str) -> bool {
            self.0.iter().any(|(k, f)| *k == key && *f == form)
        }
    }

    /// A panchanga carrying the birth nakshatra the attribute readings
    /// hang on. The other limbs are fixed: what is under test is that the
    /// nakshatra's own gana, nadi, yoni, varna and element are the
    /// subjects, and that is settled by this one field.
    fn day(nakshatra: Nakshatra) -> Panchanga {
        Panchanga {
            tithi: Tithi::ShuklaPratipada,
            vara: Vara::Ravivara,
            nakshatra,
            pada: Pada::try_new(1).unwrap(),
            yoga: Yoga::Vishkambha,
            karana: Karana::Bava,
            spans: Spans::default(),
            by_day: None,
            on_sankranti: false,
            eclipse: None,
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
        let vocabulary = Carries(&[
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

    /// The birth nakshatra settles six more subjects, and the catalogue
    /// already answers which: nothing here is computed or chosen, so the
    /// test states the catalogue's own answer for one star and asks the
    /// composer for exactly those records.
    #[test]
    fn what_the_birth_nakshatra_is_is_said_of_that_nakshatra() {
        let star = Nakshatra::Bharani;
        let attributes = star.attributes();
        // Bharani's, from the catalogue's generated table.
        assert_eq!(attributes.gana, Gana::Manushya);
        assert_eq!(attributes.nadi, Nadi::Madhya);
        assert_eq!(attributes.yoni, Yoni::Elephant);
        assert_eq!(attributes.varna, Varna::Mleccha);
        assert_eq!(attributes.element, Tatwa::Prithvi);

        let mut chart = chart();
        chart.panchanga = Some(day(star));
        let vocabulary = Carries(&[
            ("nakshatra.BHARANI", "namakarana"),
            ("gana.MANUSHYA", "phala"),
            ("nadi.MADHYA", "phala"),
            ("yoni.ELEPHANT", "phala"),
            ("varna.MLECCHA", "phala"),
            ("tatwa.PRITHVI", "phala"),
        ]);
        let plan = phala(&chart, &vocabulary);
        let said: Vec<&str> = plan.items.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(
            said,
            [
                "sdk.phala.namakarana",
                "sdk.phala.gana",
                "sdk.phala.nadi",
                "sdk.phala.yoni",
                "sdk.phala.varna",
                "sdk.phala.tatwa",
            ]
        );
    }

    /// And a corpus carrying another star's attributes says nothing,
    /// because the subject is this chart's nakshatra and not every one:
    /// Ashwini is Deva gana where Bharani is Manushya.
    #[test]
    fn another_stars_attributes_are_not_this_charts() {
        let mut chart = chart();
        chart.panchanga = Some(day(Nakshatra::Bharani));
        let vocabulary = Carries(&[("gana.DEVA", "phala"), ("nakshatra.ASHWINI", "namakarana")]);
        assert!(phala(&chart, &vocabulary).is_empty());
    }
}
