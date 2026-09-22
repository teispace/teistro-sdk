//! What a placement says of that graha's dasha: when in the dasha its
//! effects come, whether its place is auspicious, the points its dignity
//! earns and whether the placement makes the dasha favourable — with the
//! reading a loaded corpus carries of that graha as a dasha lord
//! (`03-design/interpret-composers.md` §4).
//!
//! The eleventh composer, and the second over a **section**: it reads
//! `Document.dasha_phala`, which a request asks for by name, and needs no
//! rules beside it.
//!
//! **It was the cheapest thing left in the queue, and it was still words.**
//! The measured page's table of sections said so: of the six a composer
//! could not say, three were short a *name* rather than a sentence, and
//! this one is short only one — `nature`, which is a catalogue member that
//! no strict locale names. So its place is said the way
//! `sdk.reading.lifeClass` says a class of life: **matched on its key**,
//! with the words written in each locale, rather than through an entity
//! slot that would print nothing. Being a catalogue member is not being
//! named, and this composer is where that distinction was first spent.
//!
//! **The corpus's own two are asked for separately.** `dasha-lord-effect`
//! and `dasha-lord-activation` key onto a graha under the forms
//! `dashaPhala` and `dashaActivation`, and they were 18 of the state
//! corpus's readings with no composer to attach to. They are asked of the
//! **base** locale, as every corpus reading is, so a plan is the same
//! whoever reads it.

use teistro_intl::messages::sdk::phala as phala_messages;
use teistro_intl::messages::sdk::reason::dasha;
use teistro_strength::dasha_phala::DashaPhalaReading;

use crate::{Plan, Vocabulary};

/// The form a corpus carries a dasha lord's own reading under.
const LORD_FORM: &str = "dashaPhala";

/// The form it carries what that lord's dasha sets in motion under.
///
/// Its own and not `dashaPhala`, because `graha.SUN` carries both and a
/// corpus that put them under one name would have to choose — the same
/// reason `rashi.ARIES` keeps `lagnaPhala` apart from `phala`.
const ACTIVATION_FORM: &str = "dashaActivation";

/// The key a message selects on where a placement is both favourable and
/// unfavourable, which BPHS ch. 47 vv. 5 and 6 allow together.
const BOTH: &str = "both";

/// Which of the two flags stands, where either does.
///
/// `None` when neither does: a placement that makes the dasha neither one
/// nor the other says so by saying nothing, as the prose of a rule does.
const fn favour(favourable: bool, unfavourable: bool) -> Option<&'static str> {
    match (favourable, unfavourable) {
        (true, true) => Some(BOTH),
        (true, false) => Some("favourable"),
        (false, true) => Some("unfavourable"),
        (false, false) => None,
    }
}

/// What each graha's placement says of its dasha, the reading's own order.
///
/// One graha's items are in this order: when its effects come, whether its
/// place is auspicious, the points its dignity earns, and whether the
/// placement makes the dasha favourable — then what a loaded corpus says
/// of that graha as a dasha lord, and what its dasha sets in motion.
///
/// The order is the reading's, which is the catalogue's, so the same chart
/// always gives the same plan.
#[must_use]
pub fn dasha_phala(reading: &DashaPhalaReading, vocabulary: &dyn Vocabulary) -> Plan {
    let mut plan = Plan::default();
    for graha in &reading.grahas {
        plan.say(&dasha::Phase {
            graha: graha.graha,
            phase: String::from(graha.phase.key()),
        });
        plan.say(&dasha::Place {
            graha: graha.graha,
            // The key and not the entity: `nature` is catalogued and
            // unnamed, so a locale writes the words itself.
            nature: String::from(graha.nature.key()),
        });
        plan.say(&dasha::Points {
            graha: graha.graha,
            subhanka: graha.subhanka,
            asubhanka: graha.asubhanka,
        });
        if let Some(favour) = favour(graha.favourable, graha.unfavourable) {
            plan.say(&dasha::Favour {
                graha: graha.graha,
                favour: String::from(favour),
            });
        }
        let key = graha.graha.full_key();
        if vocabulary.has_form(key, LORD_FORM) {
            plan.say(&phala_messages::DashaLord { graha: graha.graha });
        }
        if vocabulary.has_form(key, ACTIVATION_FORM) {
            plan.say(&phala_messages::DashaActivation { graha: graha.graha });
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_core::catalogue::{Graha, Nature};
    use teistro_strength::dasha_phala::{DashaPhalaReading, DashaPhase, GrahaDashaPhala};

    use super::{dasha_phala, favour};
    use crate::{KEYS, NoReadings, Vocabulary};

    /// A vocabulary carrying exactly the keys and forms it was given.
    struct Carries(&'static [(&'static str, &'static str)]);

    impl Vocabulary for Carries {
        fn has_form(&self, key: &str, form: &str) -> bool {
            self.0.iter().any(|(k, f)| *k == key && *f == form)
        }
    }

    fn one(graha: Graha) -> GrahaDashaPhala {
        GrahaDashaPhala {
            graha,
            subhankas: [30.0; 7],
            subhanka: 210.0,
            asubhanka: 30.0,
            nature: Nature::Benefic,
            phase: DashaPhase::Middle,
            favourable: true,
            unfavourable: false,
        }
    }

    fn reading(grahas: Vec<GrahaDashaPhala>) -> DashaPhalaReading {
        DashaPhalaReading { grahas }
    }

    /// Every item a graha contributes without a corpus, in the order the
    /// composer states.
    #[test]
    fn a_placement_says_four_things_of_its_dasha() {
        let plan = dasha_phala(&reading(vec![one(Graha::Sun)]), &NoReadings);
        let said: Vec<&str> = plan.items.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(
            said,
            [
                "sdk.reason.dasha.phase",
                "sdk.reason.dasha.place",
                "sdk.reason.dasha.points",
                "sdk.reason.dasha.favour",
            ]
        );
        for key in plan.keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    /// A placement that makes the dasha neither favourable nor
    /// unfavourable says nothing of it, rather than saying "neither".
    #[test]
    fn a_placement_that_tilts_neither_way_says_nothing_of_it() {
        assert_eq!(favour(false, false), None);
        let mut graha = one(Graha::Sun);
        graha.favourable = false;
        let plan = dasha_phala(&reading(vec![graha]), &NoReadings);
        assert!(
            plan.items
                .iter()
                .all(|item| item.key != "sdk.reason.dasha.favour"),
            "{plan:?}"
        );
    }

    /// And both flags together are one item and not two, because the
    /// verses allow a placement to be both and a plan that said so twice
    /// would be saying it twice.
    #[test]
    fn both_at_once_is_one_item() {
        assert_eq!(favour(true, true), Some("both"));
        let mut graha = one(Graha::Sun);
        graha.unfavourable = true;
        let plan = dasha_phala(&reading(vec![graha]), &NoReadings);
        assert_eq!(
            plan.items
                .iter()
                .filter(|item| item.key == "sdk.reason.dasha.favour")
                .count(),
            1
        );
    }

    /// The corpus's two readings are asked for independently, and neither
    /// is said without a pack.
    #[test]
    fn the_corpus_readings_are_two_claims() {
        let both = Carries(&[
            ("graha.SUN", "dashaPhala"),
            ("graha.SUN", "dashaActivation"),
        ]);
        let said = |vocabulary: &dyn Vocabulary| {
            dasha_phala(&reading(vec![one(Graha::Sun)]), vocabulary)
                .items
                .iter()
                .map(|item| item.key.clone())
                .collect::<Vec<_>>()
        };
        let all = said(&both);
        assert!(all.contains(&String::from("sdk.phala.dashaLord")));
        assert!(all.contains(&String::from("sdk.phala.dashaActivation")));

        let one_only = Carries(&[("graha.SUN", "dashaActivation")]);
        let some = said(&one_only);
        assert!(!some.contains(&String::from("sdk.phala.dashaLord")));
        assert!(some.contains(&String::from("sdk.phala.dashaActivation")));

        let none = said(&NoReadings);
        assert!(
            none.iter().all(|key| !key.starts_with("sdk.phala.")),
            "{none:?}"
        );
    }

    /// A corpus carrying another graha's reading says nothing of this one.
    #[test]
    fn another_grahas_reading_is_not_this_ones() {
        let vocabulary = Carries(&[("graha.MOON", "dashaPhala")]);
        let plan = dasha_phala(&reading(vec![one(Graha::Sun)]), &vocabulary);
        assert!(
            plan.items
                .iter()
                .all(|item| !item.key.starts_with("sdk.phala.")),
            "{plan:?}"
        );
    }
}
