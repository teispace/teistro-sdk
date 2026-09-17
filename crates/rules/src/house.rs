//! What a house says when everything bearing on it is gathered
//! (`03-design/rules-engine.md`, "What a house says when several grahas share
//! it").
//!
//! A rule answers one figure. A consumer asking "what does the fourth house
//! say?" wants every figure that touches it at once — the grahas standing
//! there, the reading of each, the reading of each pair they make, the reading
//! of the pair in that angle where a text gives one — and then wants to know
//! how to take them together. The texts answer that last question themselves,
//! in four ways and a refusal, so the SDK gathers rather than decides: a
//! [`HouseReading`] carries what held and the [`Composition`]s that bear on
//! it, each with its verse, and invents no summary of its own.

use serde::Serialize;
use teistro_core::catalogue::Rashi;

use crate::eval::{Evaluator, RuleResult};
use crate::language::{Body, EvidenceRank, House, Source};
use crate::rule::Rule;

/// Everything of a chart that bears on one house.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HouseReading<'r> {
    /// Which house.
    pub house: House,
    /// The sign it falls in.
    pub sign: Rashi,
    /// The grahas standing in it, in [`Body::ALL`]'s order.
    pub occupants: Vec<Body>,
    /// Every rule of the set that held and whose participants stand here, in
    /// the set's own order.
    pub held: Vec<Held<'r>>,
    /// What the texts say about reading those together.
    pub composition: Vec<&'static Composition>,
}

/// A rule that held, and what it answered.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Held<'r> {
    /// Which rule; written as its key, the rule being the set's.
    #[serde(serialize_with = "crate::rule::key_of")]
    pub rule: &'r Rule,
    /// What it answered.
    pub result: RuleResult,
}

/// What a text says about reading several of a house's readings together.
///
/// A composition is not a reading: it says nothing of the native, only how the
/// readings that held are to be taken — or, where a text declines to give a
/// reading at all, that it declines. Each carries its verse, so a consumer can
/// show the authority for combining as readily as for the readings combined.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Composition {
    /// Its key.
    pub key: &'static str,
    /// Which text says it, and where.
    pub text: &'static str,
    /// Its chapter.
    pub chapter: &'static str,
    /// Its verse.
    pub verse: &'static str,
    /// What kind of instruction it is.
    pub kind: Kind,
    /// The houses it bears on; empty means any.
    pub houses: &'static [u8],
    /// How many grahas must share the house before it bears.
    pub at_least: u8,
    /// Whether those grahas must be strong for it to bear.
    pub strong: bool,
    /// What it says, in the SDK's own short statement of the verse.
    pub says: &'static str,
}

impl Composition {
    /// Its citation, as a rule's would be: rank 1, every one of these being
    /// read from a verse rather than from a translator's note.
    #[must_use]
    pub fn source(&self) -> Source {
        Source {
            text: self.text.to_owned(),
            chapter: Some(self.chapter.to_owned()),
            verse: Some(self.verse.to_owned()),
            note: Some(self.says.to_owned()),
            rank: EvidenceRank::try_new(1).ok(),
        }
    }

    /// Whether it bears on a house that so many grahas share, so many of them
    /// strong.
    #[must_use]
    pub fn bears_on(&self, house: House, occupants: usize, strong: usize) -> bool {
        let enough = if self.strong { strong } else { occupants };
        (self.houses.is_empty() || self.houses.contains(&house.get()))
            && enough >= usize::from(self.at_least)
    }
}

/// What kind of instruction a composition is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Read the several as the pairs they contain.
    Compose,
    /// One of them decides, and the text says which.
    Arbitrate,
    /// The readings stand increased, decreased or mixed by something else.
    Modulate,
    /// The text declines to give a reading for this at all.
    Refuse,
}

/// The composition rules the texts state, each read from a verse.
///
/// They are few, and that is the finding: for three or more grahas in a
/// *named* house the corpus gives no reading at all, and says so
/// (`REFUSE_THREE_OR_MORE_IN_AN_ANGLE`).
pub const COMPOSITIONS: [Composition; 6] = [
    Composition {
        key: "COMPOSE_BY_PAIRS",
        text: "Phaladeepika",
        chapter: "18",
        verse: "5",
        kind: Kind::Compose,
        houses: &[],
        at_least: 3,
        strong: false,
        says: "More than two grahas in a house are read by combining the readings of \
               every pair they make, which is what one rule a pair does of itself: four \
               grahas in a house answer six of them.",
    },
    Composition {
        key: "ARBITRATE_THE_LIFE_SPAN_BY_THE_STRONGEST",
        text: "Phaladeepika",
        chapter: "22",
        verse: "19",
        kind: Kind::Arbitrate,
        houses: &[],
        at_least: 2,
        strong: false,
        says: "Where several grahas share a bhava, only the strongest of them shortens \
               the life span. The verse speaks of the Ayurdaya and of nothing else.",
    },
    Composition {
        key: "ARBITRATE_THE_ASCETIC_ORDER_BY_THE_STRONGEST",
        text: "BPHS",
        chapter: "79",
        verse: "2-3",
        kind: Kind::Arbitrate,
        houses: &[],
        at_least: 4,
        strong: true,
        says: "Four or more grahas possessed of strength in one house make an ascetic, \
               of the order the strongest of them signifies; where several are strong, \
               the order of the strongest is the one entered.",
    },
    Composition {
        key: "MODULATE_THE_ELEVENTH_BY_ASPECT",
        text: "Saravali",
        chapter: "34",
        verse: "65",
        kind: Kind::Modulate,
        houses: &[11],
        at_least: 0,
        strong: false,
        says: "The readings of the eleventh stand increased where benefics aspect it, \
               decreased where malefics do, and mixed where both do.",
    },
    Composition {
        key: "ARBITRATE_THE_ELEVENTH_BY_THE_STRONGEST",
        text: "Saravali",
        chapter: "34",
        verse: "66",
        kind: Kind::Arbitrate,
        houses: &[11],
        at_least: 0,
        strong: false,
        says: "Where the eleventh is aspected or occupied by many grahas, wealth comes \
               in as many ways, and the strongest of them has the highest influence.",
    },
    Composition {
        key: "REFUSE_THREE_OR_MORE_IN_AN_ANGLE",
        text: "Saravali",
        chapter: "31",
        verse: "87",
        kind: Kind::Refuse,
        houses: &[1, 4, 7, 10],
        at_least: 3,
        strong: false,
        says: "Having read every pair of the seven in each angle, the chapter says the \
               effects of three, four, five or six grahas in an angle should be \
               suitably understood — that is, it declines to write them, and no other \
               text read carries them either.",
    },
];

impl Evaluator<'_> {
    /// What a house says: the grahas standing in it, every rule of the set
    /// that held and whose participants stand there, and the compositions that
    /// bear on it.
    ///
    /// A rule bears on a house when the houses its participants stand in
    /// include it, which is what a result already reports; a rule with no
    /// participants — the lagna rising in a sign, a rule of the panchanga —
    /// bears on no house, and is left out rather than assigned to one.
    #[must_use]
    pub fn house_reading<'r>(&self, house: House, rules: &'r [Rule]) -> HouseReading<'r> {
        let held: Vec<Held<'r>> = rules
            .iter()
            .map(|rule| (rule, self.evaluate(rule)))
            .filter(|(_, result)| result.present && result.houses.contains(&house))
            .map(|(rule, result)| Held { rule, result })
            .collect();
        self.gathered(house, held)
    }

    /// Every house of the chart, the first to the twelfth.
    ///
    /// A rule is evaluated **once** here and its result handed to each house
    /// its participants stand in, which is what a caller wanting the whole
    /// chart should use: asking twelve houses one at a time would evaluate
    /// every rule twelve times.
    #[must_use]
    pub fn house_readings<'r>(&self, rules: &'r [Rule]) -> Vec<HouseReading<'r>> {
        let mut by_house: [Vec<Held<'r>>; 12] = Default::default();
        for rule in rules {
            let result = self.evaluate(rule);
            if !result.present {
                continue;
            }
            for house in &result.houses {
                if let Some(bucket) = by_house.get_mut(usize::from(house.get()) - 1) {
                    bucket.push(Held {
                        rule,
                        result: result.clone(),
                    });
                }
            }
        }
        by_house
            .into_iter()
            .enumerate()
            .filter_map(|(at, held)| {
                let house = House::try_new(u8::try_from(at).ok()? + 1).ok()?;
                Some(self.gathered(house, held))
            })
            .collect()
    }

    /// A house's reading, once what held there is known.
    fn gathered<'r>(&self, house: House, held: Vec<Held<'r>>) -> HouseReading<'r> {
        let sign = self.house_sign(house);
        let occupants: Vec<Body> = Body::nine()
            .into_iter()
            .filter(|body| self.chart().placement(*body).sign == sign)
            .collect();
        let strong = occupants
            .iter()
            .filter(|body| self.strengths().is_strong(**body))
            .count();
        let composition = COMPOSITIONS
            .iter()
            .filter(|composition| composition.bears_on(house, occupants.len(), strong))
            .collect();
        HouseReading {
            house,
            sign,
            occupants,
            held,
            composition,
        }
    }
}
