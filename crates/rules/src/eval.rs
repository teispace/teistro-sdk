//! Evaluating rules over a chart.

use teistro_aspect::{conjunction, drishti};
use teistro_core::catalogue::{Dignity, Graha, Rashi};
use teistro_core::settings::NodeAspects;

use crate::chart::{
    Benefics, Conjunction, DignityMatch, Gathering, Houses, NATURAL_BENEFICS, NATURAL_MALEFICS,
    NodeMotion, NodeSides, Placement, Readings, RuleChart,
};
use crate::language::{Body, Condition, House, KarakaScheme, Rule, Subject};

/// The bodies a rule consulted, in the order they were first consulted, each
/// once. A fixed array: evaluating a rule allocates nothing for it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Participants {
    bodies: [Option<Body>; 10],
    len: usize,
}

impl Participants {
    fn push(&mut self, body: Body) {
        if self.bodies.iter().flatten().any(|b| *b == body) {
            return;
        }
        if let Some(slot) = self.bodies.get_mut(self.len) {
            *slot = Some(body);
            self.len += 1;
        }
    }

    fn extend(&mut self, other: Participants) {
        for body in other.iter() {
            self.push(body);
        }
    }

    /// The bodies, in the order they were first consulted.
    pub fn iter(&self) -> impl Iterator<Item = Body> + '_ {
        self.bodies.iter().take(self.len).flatten().copied()
    }

    /// How many.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether there are none.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// What a rule answers for a chart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleResult {
    /// Whether it is present.
    pub present: bool,
    /// The bodies its conditions consulted, when present.
    pub participants: Participants,
    /// The houses its participant grahas stand in, distinct and ascending.
    pub houses: Vec<House>,
    /// Which of its cancellations held, by their place in the rule, when
    /// present.
    pub cancellations: Vec<usize>,
}

impl RuleResult {
    /// Whether a cancellation held.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        !self.cancellations.is_empty()
    }
}

/// A chart read under a choice at every open place, ready to evaluate rules.
#[derive(Clone, Copy, Debug)]
pub struct Evaluator<'a> {
    chart: &'a RuleChart,
    readings: Readings,
    /// Each body's benefic nature under the readings, by index.
    benefic: [bool; 10],
    /// Each body's malefic nature.
    malefic: [bool; 10],
}

const fn graha_body(graha: Graha) -> Body {
    Body::Graha(graha)
}

impl<'a> Evaluator<'a> {
    /// A chart under readings.
    #[must_use]
    pub fn new(chart: &'a RuleChart, readings: Readings) -> Evaluator<'a> {
        let mut benefic = [false; 10];
        let mut malefic = [false; 10];
        for graha in NATURAL_BENEFICS {
            if let Some(slot) = benefic.get_mut(graha_body(graha).index()) {
                *slot = true;
            }
        }
        for graha in NATURAL_MALEFICS {
            if let Some(slot) = malefic.get_mut(graha_body(graha).index()) {
                *slot = true;
            }
        }
        if readings.benefics == Benefics::ByCompany {
            let at = |graha| chart.placement(graha_body(graha));
            let moon = graha_body(Graha::Moon).index();
            if (at(Graha::Moon).longitude - at(Graha::Sun).longitude).rem_euclid(360.0) > 180.0 {
                set(&mut benefic, moon, false);
                set(&mut malefic, moon, true);
            }
            let mercury = graha_body(Graha::Mercury).index();
            let mercury_sign = at(Graha::Mercury).sign;
            let company = |nature: &[bool; 10]| {
                Body::ALL.iter().any(|body| {
                    body.index() != mercury
                        && nature.get(body.index()).copied().unwrap_or(false)
                        && chart.placement(*body).sign == mercury_sign
                })
            };
            if company(&malefic) && !company(&benefic) {
                set(&mut benefic, mercury, false);
                set(&mut malefic, mercury, true);
            }
        }
        Evaluator {
            chart,
            readings,
            benefic,
            malefic,
        }
    }

    fn at(&self, body: Body) -> &Placement {
        self.chart.placement(body)
    }

    fn house_of(&self, body: Body) -> House {
        match self.readings.houses {
            Houses::Recorded => self.at(body).house,
            Houses::WholeSign => House::between(self.chart.lagna(), self.at(body).sign),
        }
    }

    fn lord_of(&self, house: House) -> Body {
        let sign = step(self.chart.lagna(), house.get() - 1);
        graha_body(sign.attributes().lord)
    }

    fn dignity_meets(&self, dignity: Dignity, wanted: &[Dignity]) -> bool {
        wanted.contains(&dignity)
            || (self.readings.dignity == DignityMatch::DeepMeetsPlain
                && match dignity {
                    Dignity::DeepExalted => wanted.contains(&Dignity::Exalted),
                    Dignity::DeepDebilitated => wanted.contains(&Dignity::Debilitated),
                    _ => false,
                })
    }

    fn nature(&self, subject: Subject) -> Option<&[bool; 10]> {
        match subject {
            Subject::AnyBenefic => Some(&self.benefic),
            Subject::AnyMalefic => Some(&self.malefic),
            Subject::Body(_) => None,
        }
    }

    /// Whether a subject meets a test of its placement, adding the body that
    /// met it.
    fn subject_meets(
        &self,
        subject: Subject,
        into: &mut Participants,
        meets: impl Fn(Body) -> bool,
    ) -> bool {
        let found = match (subject, self.nature(subject)) {
            (Subject::Body(body), _) => meets(body).then_some(body),
            (_, Some(nature)) => Body::ALL
                .into_iter()
                .find(|body| nature.get(body.index()).copied().unwrap_or(false) && meets(*body)),
            _ => None,
        };
        if let Some(body) = found {
            into.push(body);
        }
        found.is_some()
    }

    /// A sub-condition of `and`, `or` or `not`: under the engine's gathering
    /// its bodies are added whatever it answers; otherwise only when `keep`.
    fn branch(&self, condition: &Condition, into: &mut Participants, keep: bool) -> bool {
        let mut own = Participants::default();
        let held = self.holds(condition, &mut own);
        if self.readings.gathering == Gathering::EveryHeld || held == keep {
            into.extend(own);
        }
        held
    }

    /// Whether a condition holds, adding the bodies it consulted.
    #[allow(clippy::too_many_lines, reason = "one arm a predicate of the language")]
    pub fn holds(&self, condition: &Condition, into: &mut Participants) -> bool {
        match condition {
            Condition::And { conditions } => conditions.iter().all(|c| self.branch(c, into, true)),
            Condition::Or { conditions } => conditions.iter().any(|c| self.branch(c, into, true)),
            Condition::Not { condition } => {
                let mut own = Participants::default();
                let held = self.holds(condition, &mut own);
                if self.readings.gathering == Gathering::EveryHeld {
                    into.extend(own);
                }
                !held
            }
            Condition::PlanetInHouse { planet, houses } => {
                self.subject_meets(*planet, into, |body| houses.contains(&self.house_of(body)))
            }
            Condition::PlanetInHouseFrom {
                planet,
                reference,
                houses,
            } => {
                let from = self.at(*reference).sign;
                self.subject_meets(*planet, into, |body| {
                    houses.contains(&House::between(from, self.at(body).sign))
                })
            }
            Condition::PlanetInSign { planet, signs } => {
                self.single(*planet, into, signs.contains(&self.at(*planet).sign))
            }
            Condition::PlanetDignity { planet, dignities } => {
                let held = self.dignity_meets(self.at(*planet).dignity, dignities);
                self.single(*planet, into, held)
            }
            Condition::PlanetInKendra { planet } => self.single(
                *planet,
                into,
                House::KENDRAS.contains(&self.house_of(*planet)),
            ),
            Condition::PlanetInTrikona { planet } => self.single(
                *planet,
                into,
                House::TRIKONAS.contains(&self.house_of(*planet)),
            ),
            Condition::PlanetInKendraFrom { planet, reference } => {
                let house = House::between(self.at(*reference).sign, self.at(*planet).sign);
                let held = House::KENDRAS.contains(&house);
                if held {
                    into.push(*planet);
                    into.push(*reference);
                }
                held
            }
            Condition::LordOfHouseInKendra { house_ruled } => {
                let lord = self.lord_of(*house_ruled);
                self.single(lord, into, House::KENDRAS.contains(&self.house_of(lord)))
            }
            Condition::LordOfHouseInHouse {
                house_ruled,
                house_occupied,
            } => {
                let lord = self.lord_of(*house_ruled);
                self.single(lord, into, self.house_of(lord) == *house_occupied)
            }
            Condition::PlanetConjunct { planets, max_orb } => {
                let orb = max_orb
                    .filter(|orb| *orb > 0.0)
                    .or(match self.readings.conjunction {
                        Conjunction::Orb(degrees) => Some(degrees),
                        Conjunction::SameSign => None,
                    });
                let held = match orb {
                    Some(orb) => planets.iter().enumerate().all(|(i, a)| {
                        planets.iter().skip(i + 1).all(|b| {
                            conjunction::within(self.at(*a).longitude, self.at(*b).longitude, orb)
                        })
                    }),
                    None => planets.windows(2).all(|pair| {
                        matches!(pair, [a, b] if conjunction::together(self.at(*a).sign, self.at(*b).sign))
                    }),
                };
                if held {
                    for body in planets {
                        into.push(*body);
                    }
                }
                held
            }
            Condition::NoPlanetInHousesFrom {
                reference,
                houses,
                except,
            } => {
                let from = self.at(*reference).sign;
                !Body::ALL.iter().any(|body| {
                    *body != *reference
                        && *body != Body::Lagna
                        && !except.contains(body)
                        && houses.contains(&House::between(from, self.at(*body).sign))
                })
            }
            Condition::MutualExchange { house1, house2 } => {
                let (one, other) = (self.lord_of(*house1), self.lord_of(*house2));
                let held = self.house_of(one) == *house2 && self.house_of(other) == *house1;
                if held {
                    into.push(one);
                    into.push(other);
                }
                held
            }
            Condition::LordConjunctLord { house1, house2 } => {
                let (one, other) = (self.lord_of(*house1), self.lord_of(*house2));
                let held = one == other || self.at(one).sign == self.at(other).sign;
                if held {
                    into.push(one);
                    into.push(other);
                }
                held
            }
            Condition::AllPlanetsBetweenNodes => {
                let rahu = self.at(graha_body(Graha::Rahu)).sign as u8;
                let ketu = self.at(graha_body(Graha::Ketu)).sign as u8;
                let arc = (ketu + 12 - rahu) % 12;
                let mut sides = [0_u8; 2];
                for body in Body::SEVEN {
                    let from = (self.at(body).sign as u8 + 12 - rahu) % 12;
                    if from == 0 || from == arc {
                        return false;
                    }
                    if let Some(side) = sides.get_mut(usize::from(from > arc)) {
                        *side += 1;
                    }
                }
                let [rahu_side, ketu_side] = sides;
                let held = match self.readings.node_sides {
                    NodeSides::Either => rahu_side == 0 || ketu_side == 0,
                    NodeSides::RahuToKetu => ketu_side == 0,
                };
                if held {
                    for body in Body::SEVEN {
                        into.push(body);
                    }
                }
                held
            }
            Condition::OccupiedSignCount { planets, count } => {
                let mut signs = [false; 12];
                for body in planets {
                    if let Some(slot) = signs.get_mut(self.at(*body).sign as usize) {
                        *slot = true;
                    }
                }
                let held = signs.iter().filter(|s| **s).count() == usize::from(*count);
                if held {
                    for body in planets {
                        into.push(*body);
                    }
                }
                held
            }
            Condition::AllClassicalGrahasInHouses {
                houses,
                require_all_houses_filled,
            } => {
                let mut filled = [false; 13];
                for body in Body::SEVEN {
                    let house = self.house_of(body);
                    if !houses.contains(&house) {
                        return false;
                    }
                    if let Some(slot) = filled.get_mut(usize::from(house.get())) {
                        *slot = true;
                    }
                }
                let held = !require_all_houses_filled
                    || houses
                        .iter()
                        .all(|h| filled.get(usize::from(h.get())).copied().unwrap_or(false));
                if held {
                    for body in Body::SEVEN {
                        into.push(body);
                    }
                }
                held
            }
            Condition::NGrahasConjunctWith { anchor, min_count } => {
                let sign = self.at(*anchor).sign;
                let cluster = || {
                    Body::SEVEN
                        .into_iter()
                        .filter(move |b| self.at(*b).sign == sign)
                };
                let held = cluster().count() >= usize::from(*min_count);
                if held {
                    for body in cluster() {
                        into.push(body);
                    }
                }
                held
            }
            Condition::CharaKarakaInHouse {
                karaka,
                houses,
                karaka_scheme,
            } => {
                let holder = Body::ALL.into_iter().find(|body| {
                    let placement = self.at(*body);
                    let held = match karaka_scheme {
                        KarakaScheme::Seven => placement.karaka7,
                        KarakaScheme::Eight => placement.karaka8,
                    };
                    held == Some(karaka.0)
                });
                match holder {
                    Some(body) => self.single(body, into, houses.contains(&self.house_of(body))),
                    None => false,
                }
            }
            Condition::PlanetCombust { planet } => {
                self.single(*planet, into, self.at(*planet).combust)
            }
            Condition::PlanetRetrograde { planet } => {
                let held = if planet.is_node() {
                    self.readings.node_motion == NodeMotion::AlwaysRetrograde
                } else {
                    self.at(*planet).retrograde
                };
                self.single(*planet, into, held)
            }
            Condition::PlanetAspectsPlanet { from, target } => {
                let held = aspects(
                    *from,
                    self.at(*from).sign,
                    self.at(*target).sign,
                    self.readings.node_aspects,
                );
                if held {
                    into.push(*from);
                    into.push(*target);
                }
                held
            }
            Condition::PlanetAspectsHouse { from, house_ruled } => {
                let target = step(self.chart.lagna(), house_ruled.get() - 1);
                let held = aspects(
                    *from,
                    self.at(*from).sign,
                    target,
                    self.readings.node_aspects,
                );
                self.single(*from, into, held)
            }
        }
    }

    /// A predicate about one body: adds it when the predicate held.
    fn single(&self, body: Body, into: &mut Participants, held: bool) -> bool {
        let _ = self;
        if held {
            into.push(body);
        }
        held
    }

    /// A rule's answer for the chart.
    ///
    /// A rule the language cannot evaluate, with no conditions, answers not
    /// present; ask [`Rule::is_evaluable`] to tell the two apart.
    #[must_use]
    pub fn evaluate(&self, rule: &Rule) -> RuleResult {
        let mut participants = Participants::default();
        let present = rule.is_evaluable()
            && rule
                .conditions
                .iter()
                .all(|c| self.holds(c, &mut participants));
        if !present {
            return RuleResult {
                present: false,
                participants: Participants::default(),
                houses: Vec::new(),
                cancellations: Vec::new(),
            };
        }
        let mut houses: Vec<House> = participants
            .iter()
            .filter(|b| *b != Body::Lagna)
            .map(|b| self.house_of(b))
            .collect();
        houses.sort_unstable();
        houses.dedup();
        let cancellations = rule
            .cancellations
            .iter()
            .enumerate()
            .filter(|(_, c)| self.holds(c, &mut Participants::default()))
            .map(|(i, _)| i)
            .collect();
        RuleResult {
            present,
            participants,
            houses,
            cancellations,
        }
    }
}

fn set(nature: &mut [bool; 10], index: usize, value: bool) {
    if let Some(slot) = nature.get_mut(index) {
        *slot = value;
    }
}

/// The sign `count` signs on from `sign`.
fn step(sign: Rashi, count: u8) -> Rashi {
    Rashi::from_id(u16::from((sign as u8 + count) % 12)).unwrap_or(Rashi::Aries)
}

/// Whether a body in `from` fully aspects the sign `target` by graha drishti,
/// as `teistro-aspect` reads it, the nodes under `nodes`; the lagna, a point,
/// aspects nothing.
fn aspects(body: Body, from: Rashi, target: Rashi, nodes: NodeAspects) -> bool {
    match body {
        Body::Lagna => false,
        Body::Graha(graha) => drishti::between_under(graha, from, target, nodes).is_full(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests unwrap what they built and index their own charts"
    )]

    use teistro_core::catalogue::{CharaKaraka, Dignity, Graha, Rashi};

    use super::*;
    use crate::chart::{
        Benefics, Conjunction, DignityMatch, Gathering, Houses, NodeMotion, NodeSides,
    };
    use crate::language::Karaka;

    const SUN: Body = Body::Graha(Graha::Sun);
    const MOON: Body = Body::Graha(Graha::Moon);
    const MARS: Body = Body::Graha(Graha::Mars);
    const MERCURY: Body = Body::Graha(Graha::Mercury);
    const JUPITER: Body = Body::Graha(Graha::Jupiter);
    const SATURN: Body = Body::Graha(Graha::Saturn);
    const RAHU: Body = Body::Graha(Graha::Rahu);

    fn house(n: u8) -> House {
        House::try_new(n).unwrap()
    }

    /// Every body at 15° Aries in the first house, neutral, direct, clear.
    fn chart() -> RuleChart {
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
            }; 10],
        }
    }

    fn place(chart: &mut RuleChart, body: Body, sign: Rashi) {
        let p = &mut chart.placements[body.index()];
        p.sign = sign;
        p.longitude = f64::from(sign as u8) * 30.0 + 15.0;
        p.house = House::between(Rashi::Aries, sign);
    }

    fn holds(chart: &RuleChart, readings: Readings, condition: &Condition) -> (bool, Vec<Body>) {
        let mut into = Participants::default();
        let held = Evaluator::new(chart, readings).holds(condition, &mut into);
        (held, into.iter().collect())
    }

    const ENGINE: Readings = Readings::RECORDING_ENGINE;

    #[test]
    fn a_waning_moon_and_a_mercury_among_malefics_are_malefics_by_company_only() {
        let mut c = chart();
        place(&mut c, SUN, Rashi::Aries);
        place(&mut c, MOON, Rashi::Scorpio); // 210° past the Sun: waning.
        place(&mut c, MERCURY, Rashi::Leo);
        place(&mut c, SATURN, Rashi::Leo); // Mercury with a malefic alone.
        for body in [
            MARS,
            JUPITER,
            Body::Graha(Graha::Venus),
            RAHU,
            Body::Graha(Graha::Ketu),
        ] {
            place(&mut c, body, Rashi::Taurus);
        }
        let benefic_in = |sign_house: u8| Condition::PlanetInHouse {
            planet: Subject::AnyBenefic,
            houses: vec![house(sign_house)],
        };
        // Scorpio is the eighth house and Leo the fifth.
        assert!(!holds(&c, ENGINE, &benefic_in(8)).0);
        assert!(!holds(&c, ENGINE, &benefic_in(5)).0);
        let natural = Readings {
            benefics: Benefics::Natural,
            ..ENGINE
        };
        assert_eq!(holds(&c, natural, &benefic_in(8)), (true, vec![MOON]));
        assert_eq!(holds(&c, natural, &benefic_in(5)), (true, vec![MERCURY]));
    }

    #[test]
    fn each_reading_changes_exactly_what_it_names() {
        let mut c = chart();
        c.placements[JUPITER.index()].dignity = Dignity::DeepExalted;
        let exalted = Condition::PlanetDignity {
            planet: JUPITER,
            dignities: vec![Dignity::Exalted],
        };
        assert!(holds(&c, ENGINE, &exalted).0);
        assert!(
            !holds(
                &c,
                Readings {
                    dignity: DignityMatch::Exact,
                    ..ENGINE
                },
                &exalted
            )
            .0
        );

        // Mars recorded in the fourth house while it stands in the lagna's sign.
        c.placements[MARS.index()].house = house(4);
        let kendra = Condition::PlanetInHouse {
            planet: Subject::Body(MARS),
            houses: vec![house(4)],
        };
        assert!(holds(&c, ENGINE, &kendra).0);
        assert!(
            !holds(
                &c,
                Readings {
                    houses: Houses::WholeSign,
                    ..ENGINE
                },
                &kendra
            )
            .0
        );

        let retro = Condition::PlanetRetrograde { planet: RAHU };
        c.placements[RAHU.index()].retrograde = true;
        assert!(!holds(&c, ENGINE, &retro).0);
        assert!(
            holds(
                &c,
                Readings {
                    node_motion: NodeMotion::AlwaysRetrograde,
                    ..ENGINE
                },
                &retro
            )
            .0
        );

        // The Sun at 29° Aries and the Moon at 1° Taurus: two degrees apart,
        // two signs.
        c.placements[SUN.index()].longitude = 29.0;
        place(&mut c, MOON, Rashi::Taurus);
        c.placements[MOON.index()].longitude = 31.0;
        let together = Condition::PlanetConjunct {
            planets: vec![SUN, MOON],
            max_orb: None,
        };
        assert!(!holds(&c, ENGINE, &together).0);
        assert!(
            holds(
                &c,
                Readings {
                    conjunction: Conjunction::Orb(10.0),
                    ..ENGINE
                },
                &together
            )
            .0
        );
    }

    #[test]
    fn the_seven_between_the_nodes_on_either_side_or_from_rahu_only() {
        let mut c = chart();
        place(&mut c, RAHU, Rashi::Aries);
        place(&mut c, Body::Graha(Graha::Ketu), Rashi::Libra);
        for body in Body::SEVEN {
            place(&mut c, body, Rashi::Sagittarius); // Ketu's side.
        }
        let kala = Condition::AllPlanetsBetweenNodes;
        assert!(holds(&c, ENGINE, &kala).0);
        assert!(
            !holds(
                &c,
                Readings {
                    node_sides: NodeSides::RahuToKetu,
                    ..ENGINE
                },
                &kala
            )
            .0
        );
        place(&mut c, SUN, Rashi::Libra); // On a node: broken.
        assert!(!holds(&c, ENGINE, &kala).0);
    }

    #[test]
    fn a_failed_branch_adds_its_bodies_only_under_the_engine_s_gathering() {
        let c = chart();
        // Mars is in the first house; the `and` fails on Jupiter's dignity after
        // Mars has held, then the second branch holds on the Sun.
        let rule = Condition::Or {
            conditions: vec![
                Condition::And {
                    conditions: vec![
                        Condition::PlanetInKendra { planet: MARS },
                        Condition::PlanetDignity {
                            planet: JUPITER,
                            dignities: vec![Dignity::Exalted],
                        },
                    ],
                },
                Condition::PlanetInKendra { planet: SUN },
            ],
        };
        assert_eq!(holds(&c, ENGINE, &rule), (true, vec![MARS, SUN]));
        let deciding = Readings {
            gathering: Gathering::DecidingBranch,
            ..ENGINE
        };
        assert_eq!(holds(&c, deciding, &rule), (true, vec![SUN]));
    }

    #[test]
    fn lords_karakas_and_aspects_read_the_chart_by_whole_signs() {
        let mut c = chart();
        // Lagna Aries: the tenth's lord is Saturn. Saturn in Capricorn, its
        // own tenth house.
        place(&mut c, SATURN, Rashi::Capricorn);
        let lord = Condition::LordOfHouseInHouse {
            house_ruled: house(10),
            house_occupied: house(10),
        };
        assert_eq!(holds(&c, ENGINE, &lord), (true, vec![SATURN]));
        c.placements[SATURN.index()].karaka7 = Some(CharaKaraka::Atmakaraka);
        let ak = Condition::CharaKarakaInHouse {
            karaka: Karaka(CharaKaraka::Atmakaraka),
            houses: vec![house(10)],
            karaka_scheme: crate::language::KarakaScheme::Seven,
        };
        assert_eq!(holds(&c, ENGINE, &ak), (true, vec![SATURN]));
        // Saturn in Capricorn aspects Pisces, its third, and not Taurus, its fifth.
        let aspect = |target| Condition::PlanetAspectsPlanet {
            from: SATURN,
            target,
        };
        place(&mut c, SUN, Rashi::Pisces);
        assert!(holds(&c, ENGINE, &aspect(SUN)).0);
        place(&mut c, SUN, Rashi::Taurus);
        assert!(!holds(&c, ENGINE, &aspect(SUN)).0);
        // The lagna aspects nothing.
        let lagna = Condition::PlanetAspectsHouse {
            from: Body::Lagna,
            house_ruled: house(7),
        };
        assert!(!holds(&c, ENGINE, &lagna).0);
    }

    #[test]
    fn a_rule_without_conditions_is_not_evaluable_and_not_present() {
        let rule = Rule {
            key: String::from("OUTSIDE"),
            category: String::from("example"),
            source: crate::language::Source {
                text: String::from("an example"),
                chapter: None,
                verse: None,
                note: None,
            },
            conditions: Vec::new(),
            cancellations: Vec::new(),
        };
        assert!(!rule.is_evaluable());
        assert!(!Evaluator::new(&chart(), ENGINE).evaluate(&rule).present);
    }
}
