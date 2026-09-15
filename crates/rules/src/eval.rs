//! Evaluating rules over a chart.

use serde::Serialize;
use teistro_aspect::{conjunction, drishti};
use teistro_core::catalogue::{Dignity, Graha, Rashi};
use teistro_core::settings::NodeAspects;

use teistro_points::arudha;

use crate::chart::{
    Benefics, Conjunction, DignityMatch, Gathering, Houses, NATURAL_BENEFICS, NATURAL_MALEFICS,
    NodeMotion, NodeSides, Placement, Readings, RuleChart, Upapada,
};
use crate::language::{Body, Condition, House, KarakaScheme, Rule};
use crate::reference::{BodyRef, SignRef, Subject};
use crate::table::{Table, Tables};
use crate::trace::{Explanation, NoTrace, Recorder, Resolved, Tracer};

/// The tables an evaluator looks in until it is given some.
static NO_TABLES: Tables = Tables::EMPTY;

/// The bodies a rule consulted, in the order they were first consulted, each
/// once. A fixed array: evaluating a rule allocates nothing for it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(into = "Vec<Body>")]
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

    /// Adds the body a spot was reached through, if any.
    fn push_through(&mut self, spot: Spot) {
        if let Some(body) = spot.through {
            self.push(body);
        }
    }

    /// The body at a place in the list.
    fn nth(&self, index: usize) -> Option<Body> {
        self.bodies.get(index).copied().flatten()
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

/// Where a reference resolved: a sign, the body it was reached through, and
/// whether it is that body's own place, whose house the readings decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spot {
    sign: Rashi,
    through: Option<Body>,
    standing: bool,
}

impl Spot {
    const fn sign(sign: Rashi, through: Option<Body>) -> Spot {
        Spot {
            sign,
            through,
            standing: false,
        }
    }
}

impl From<Participants> for Vec<Body> {
    fn from(participants: Participants) -> Vec<Body> {
        participants.iter().collect()
    }
}

/// What a rule answers for a chart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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
    tables: &'a Tables,
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
            tables: &NO_TABLES,
            benefic,
            malefic,
        }
    }

    /// The same evaluator, looking tables up in `tables`; without, a table
    /// predicate never holds. Check rules against the set first with
    /// [`Tables::check`].
    #[must_use]
    pub const fn with_tables(self, tables: &'a Tables) -> Evaluator<'a> {
        Evaluator { tables, ..self }
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

    fn lord_of<'c>(&self, house: House, rec: &mut impl Recorder<'c>) -> Body {
        let lord = graha_body(step(self.chart.lagna(), house.get() - 1).attributes().lord);
        rec.resolved(|| Resolved::Body {
            reference: BodyRef::lord_of(house),
            body: Some(lord),
        });
        lord
    }

    /// The body a reference resolves to, if the chart holds one: a karaka no
    /// graha holds resolves to none.
    fn body<'c>(&self, reference: &BodyRef, rec: &mut impl Recorder<'c>) -> Option<Body> {
        let body = match reference {
            BodyRef::Body(body) => Some(*body),
            BodyRef::LordOf(sign) => self
                .spot(sign, rec)
                .map(|spot| graha_body(spot.sign.attributes().lord)),
            BodyRef::Karaka { karaka, scheme } => Body::ALL.into_iter().find(|body| {
                let placement = self.at(*body);
                let held = match scheme {
                    KarakaScheme::Seven => placement.karaka7,
                    KarakaScheme::Eight => placement.karaka8,
                };
                held == Some(karaka.0)
            }),
        };
        rec.resolved(|| Resolved::Body {
            reference: reference.clone(),
            body,
        });
        body
    }

    /// Where a sign reference resolves, if it does.
    fn spot<'c>(&self, reference: &SignRef, rec: &mut impl Recorder<'c>) -> Option<Spot> {
        let lagna = self.chart.lagna();
        let spot = self.resolve(reference, lagna, rec);
        rec.resolved(|| Resolved::Sign {
            reference: reference.clone(),
            place: spot.map(|spot| (spot.sign, self.house_at(spot))),
        });
        spot
    }

    fn resolve<'c>(
        &self,
        reference: &SignRef,
        lagna: Rashi,
        rec: &mut impl Recorder<'c>,
    ) -> Option<Spot> {
        Some(match reference {
            SignRef::Of(body) => {
                let body = self.body(body, rec)?;
                Spot {
                    sign: self.at(body).sign,
                    through: Some(body),
                    standing: true,
                }
            }
            SignRef::House(house) => Spot::sign(step(lagna, house.get() - 1), None),
            SignRef::Arudha(sign) => Spot::sign(self.pada(self.spot(sign, rec)?.sign), None),
            SignRef::Upapada => {
                let odd = (lagna as u8) % 2 == 0;
                let house = match self.readings.upapada {
                    Upapada::ByLagnaParity if !odd => 1,
                    Upapada::Twelfth | Upapada::ByLagnaParity => 11,
                };
                Spot::sign(self.pada(step(lagna, house)), None)
            }
            SignRef::Navamsha(body) => {
                let body = self.body(body, rec)?;
                Spot::sign(self.at(body).navamsha, Some(body))
            }
            SignRef::Counted { from, house } => {
                let from = self.spot(from, rec)?;
                Spot::sign(step(from.sign, house.get() - 1), from.through)
            }
        })
    }

    /// The pada of the house standing in `sign`.
    fn pada(&self, sign: Rashi) -> Rashi {
        arudha::pada(sign, |graha| self.at(graha_body(graha)).sign)
    }

    /// The house a spot is in: a body's own under the readings, a sign's by
    /// whole signs.
    fn house_at(&self, spot: Spot) -> House {
        match (spot.standing, spot.through) {
            (true, Some(body)) => self.house_of(body),
            _ => House::between(self.chart.lagna(), spot.sign),
        }
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

    /// Whether a subject meets a test of where it is, adding the body it
    /// reached.
    fn subject_meets<'c>(
        &self,
        subject: &Subject,
        into: &mut Participants,
        rec: &mut impl Recorder<'c>,
        meets: impl Fn(Spot) -> bool,
    ) -> bool {
        let nature = match subject {
            Subject::Ref(reference) => {
                return self
                    .spot(reference, rec)
                    .is_some_and(|spot| self.reached(spot, into, meets(spot)));
            }
            Subject::AnyBenefic => &self.benefic,
            Subject::AnyMalefic => &self.malefic,
        };
        let found = Body::ALL.into_iter().find(|body| {
            nature.get(body.index()).copied().unwrap_or(false) && meets(self.standing(*body))
        });
        if let Some(body) = found {
            into.push(body);
        }
        found.is_some()
    }

    /// A body where it stands.
    fn standing(&self, body: Body) -> Spot {
        Spot {
            sign: self.at(body).sign,
            through: Some(body),
            standing: true,
        }
    }

    /// Adds the body a spot was reached through when `held`.
    fn reached(&self, spot: Spot, into: &mut Participants, held: bool) -> bool {
        match spot.through {
            Some(body) => self.single(body, into, held),
            None => held,
        }
    }

    /// A predicate about one body reference: false when it resolves to none,
    /// and the body added when the predicate held.
    fn body_meets<'c>(
        &self,
        reference: &BodyRef,
        into: &mut Participants,
        rec: &mut impl Recorder<'c>,
        meets: impl Fn(Body) -> bool,
    ) -> bool {
        self.body(reference, rec)
            .is_some_and(|body| self.single(body, into, meets(body)))
    }

    /// A predicate about one sign reference.
    fn sign_meets<'c>(
        &self,
        reference: &SignRef,
        into: &mut Participants,
        rec: &mut impl Recorder<'c>,
        meets: impl Fn(Spot) -> bool,
    ) -> bool {
        self.spot(reference, rec)
            .is_some_and(|spot| self.reached(spot, into, meets(spot)))
    }

    /// A sub-condition of `and`, `or` or `not`: under the engine's gathering
    /// its bodies are added whatever it answers; otherwise only when `keep`.
    fn branch<'c>(
        &self,
        condition: &'c Condition,
        into: &mut Participants,
        keep: bool,
        rec: &mut impl Recorder<'c>,
    ) -> bool {
        let mut own = Participants::default();
        let held = self.check(condition, &mut own, rec);
        if self.readings.gathering == Gathering::EveryHeld || held == keep {
            into.extend(own);
        }
        held
    }

    /// Whether a condition holds, adding the bodies it consulted.
    pub fn holds(&self, condition: &Condition, into: &mut Participants) -> bool {
        self.check(condition, into, &mut NoTrace)
    }

    /// [`Evaluator::holds`], recording each step for `rec`.
    fn check<'c>(
        &self,
        condition: &'c Condition,
        into: &mut Participants,
        rec: &mut impl Recorder<'c>,
    ) -> bool {
        rec.enter();
        let before = into.len();
        let held = self.predicate(condition, into, rec);
        rec.leave(condition, held, || into.iter().skip(before).collect());
        held
    }

    #[allow(clippy::too_many_lines, reason = "one arm a predicate of the language")]
    fn predicate<'c>(
        &self,
        condition: &'c Condition,
        into: &mut Participants,
        rec: &mut impl Recorder<'c>,
    ) -> bool {
        match condition {
            Condition::And { conditions } => {
                conditions.iter().all(|c| self.branch(c, into, true, rec))
            }
            Condition::Or { conditions } => {
                conditions.iter().any(|c| self.branch(c, into, true, rec))
            }
            Condition::Not { condition } => {
                let mut own = Participants::default();
                let held = self.check(condition, &mut own, rec);
                if self.readings.gathering == Gathering::EveryHeld {
                    into.extend(own);
                }
                !held
            }
            Condition::PlanetInHouse { planet, houses } => {
                self.subject_meets(planet, into, rec, |spot| {
                    houses.contains(&self.house_at(spot))
                })
            }
            Condition::PlanetInHouseFrom {
                planet,
                reference,
                houses,
            } => self.spot(reference, rec).is_some_and(|from| {
                self.subject_meets(planet, into, rec, |spot| {
                    houses.contains(&House::between(from.sign, spot.sign))
                })
            }),
            Condition::PlanetInSign { planet, signs } => {
                self.sign_meets(planet, into, rec, |spot| signs.contains(&spot.sign))
            }
            Condition::PlanetDignity { planet, dignities } => {
                self.body_meets(planet, into, rec, |body| {
                    self.dignity_meets(self.at(body).dignity, dignities)
                })
            }
            Condition::PlanetInKendra { planet } => self.sign_meets(planet, into, rec, |spot| {
                House::KENDRAS.contains(&self.house_at(spot))
            }),
            Condition::PlanetInTrikona { planet } => self.sign_meets(planet, into, rec, |spot| {
                House::TRIKONAS.contains(&self.house_at(spot))
            }),
            Condition::PlanetInKendraFrom { planet, reference } => {
                let (Some(spot), Some(from)) = (self.spot(planet, rec), self.spot(reference, rec))
                else {
                    return false;
                };
                let held = House::KENDRAS.contains(&House::between(from.sign, spot.sign));
                if held {
                    into.push_through(spot);
                    into.push_through(from);
                }
                held
            }
            Condition::LordOfHouseInKendra { house_ruled } => {
                let lord = self.lord_of(*house_ruled, rec);
                self.single(lord, into, House::KENDRAS.contains(&self.house_of(lord)))
            }
            Condition::LordOfHouseInHouse {
                house_ruled,
                house_occupied,
            } => {
                let lord = self.lord_of(*house_ruled, rec);
                self.single(lord, into, self.house_of(lord) == *house_occupied)
            }
            Condition::PlanetConjunct { planets, max_orb } => {
                let mut bodies = Participants::default();
                for reference in planets {
                    match self.body(reference, rec) {
                        Some(body) => bodies.push(body),
                        None => return false,
                    }
                }
                let orb = max_orb
                    .filter(|orb| *orb > 0.0)
                    .or(match self.readings.conjunction {
                        Conjunction::Orb(degrees) => Some(degrees),
                        Conjunction::SameSign => None,
                    });
                let at = |i: usize| bodies.nth(i).map(|body| self.at(body));
                let held = match orb {
                    Some(orb) => (0..bodies.len()).all(|i| {
                        (i + 1..bodies.len()).all(|j| {
                            matches!((at(i), at(j)), (Some(a), Some(b)) if conjunction::within(a.longitude, b.longitude, orb))
                        })
                    }),
                    None => (1..bodies.len()).all(|i| {
                        matches!((at(i - 1), at(i)), (Some(a), Some(b)) if conjunction::together(a.sign, b.sign))
                    }),
                };
                if held {
                    into.extend(bodies);
                }
                held
            }
            Condition::NoPlanetInHousesFrom {
                reference,
                houses,
                except,
            } => self.spot(reference, rec).is_some_and(|from| {
                let own = if from.standing { from.through } else { None };
                !Body::ALL.iter().any(|body| {
                    Some(*body) != own
                        && *body != Body::Lagna
                        && !except.contains(body)
                        && houses.contains(&House::between(from.sign, self.at(*body).sign))
                })
            }),
            Condition::MutualExchange { house1, house2 } => {
                let (one, other) = (self.lord_of(*house1, rec), self.lord_of(*house2, rec));
                let held = self.house_of(one) == *house2 && self.house_of(other) == *house1;
                if held {
                    into.push(one);
                    into.push(other);
                }
                held
            }
            Condition::LordConjunctLord { house1, house2 } => {
                let (one, other) = (self.lord_of(*house1, rec), self.lord_of(*house2, rec));
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
                let mut reached = Participants::default();
                for reference in planets {
                    let Some(spot) = self.spot(reference, rec) else {
                        return false;
                    };
                    if let Some(slot) = signs.get_mut(spot.sign as usize) {
                        *slot = true;
                    }
                    reached.push_through(spot);
                }
                let held = signs.iter().filter(|s| **s).count() == usize::from(*count);
                if held {
                    into.extend(reached);
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
                let Some(anchor) = self.spot(anchor, rec) else {
                    return false;
                };
                let cluster = || {
                    Body::SEVEN
                        .into_iter()
                        .filter(move |b| self.at(*b).sign == anchor.sign)
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
                let holder = BodyRef::Karaka {
                    karaka: *karaka,
                    scheme: *karaka_scheme,
                };
                self.body_meets(&holder, into, rec, |body| {
                    houses.contains(&self.house_of(body))
                })
            }
            Condition::PlanetCombust { planet } => {
                self.body_meets(planet, into, rec, |body| self.at(body).combust)
            }
            Condition::PlanetRetrograde { planet } => self.body_meets(planet, into, rec, |body| {
                if body.is_node() {
                    self.readings.node_motion == NodeMotion::AlwaysRetrograde
                } else {
                    self.at(body).retrograde
                }
            }),
            Condition::PlanetAspectsPlanet { from, target } => {
                let (Some(body), Some(target)) = (self.body(from, rec), self.spot(target, rec))
                else {
                    return false;
                };
                let held = aspects(
                    body,
                    self.at(body).sign,
                    target.sign,
                    self.readings.node_aspects,
                );
                if held {
                    into.push(body);
                    into.push_through(target);
                }
                held
            }
            Condition::PlanetAtTableDegree { planet, table } => {
                let Some(body) = self.body(planet, rec) else {
                    return false;
                };
                let placement = self.at(body);
                let in_sign = placement.longitude.rem_euclid(30.0);
                let degree = self
                    .tables
                    .get(table.as_str())
                    .and_then(|found| found.degree(body, placement.sign));
                rec.resolved(|| Resolved::Degree {
                    table: table.clone(),
                    body,
                    sign: placement.sign,
                    degree,
                    in_sign,
                });
                let held =
                    degree.is_some_and(|degree| self.readings.bhaga.contains(degree, in_sign));
                self.single(body, into, held)
            }
            Condition::PlanetInTableSign { planet, table } => {
                let signs = match (self.tables.get(table.as_str()), self.chart.tithi) {
                    (Some(found @ Table::SignsByTithi { .. }), Some(tithi)) => found.signs(tithi),
                    _ => &[],
                };
                rec.resolved(|| Resolved::Burnt {
                    table: table.clone(),
                    tithi: self.chart.tithi,
                    signs: signs.to_vec(),
                });
                self.sign_meets(planet, into, rec, |spot| signs.contains(&spot.sign))
            }
            Condition::PlanetAspectsHouse { from, house_ruled } => {
                let target = step(self.chart.lagna(), house_ruled.get() - 1);
                self.body_meets(from, into, rec, |body| {
                    aspects(body, self.at(body).sign, target, self.readings.node_aspects)
                })
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
        self.run(rule, &mut NoTrace, &mut NoTrace)
    }

    /// A rule's answer with how it was reached: each condition checked, in
    /// order, whether it held, the bodies it added and every reference it
    /// resolved on the way, then each cancellation the same way.
    ///
    /// It allocates the trace; [`Evaluator::evaluate`] answers the same without
    /// one.
    #[must_use]
    pub fn explain<'c>(&self, rule: &'c Rule) -> Explanation<'c> {
        let (mut conditions, mut cancellations) = (Tracer::default(), Tracer::default());
        let result = self.run(rule, &mut conditions, &mut cancellations);
        Explanation {
            rule,
            result,
            conditions: conditions.finish(),
            cancellations: cancellations.finish(),
        }
    }

    fn run<'c>(
        &self,
        rule: &'c Rule,
        conditions: &mut impl Recorder<'c>,
        cancelling: &mut impl Recorder<'c>,
    ) -> RuleResult {
        let mut participants = Participants::default();
        let present = rule.is_evaluable()
            && rule
                .conditions
                .iter()
                .all(|c| self.check(c, &mut participants, conditions));
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
            .filter(|(_, c)| self.check(c, &mut Participants::default(), cancelling))
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
        clippy::panic,
        reason = "tests unwrap what they built, index their own charts and fail by panicking"
    )]

    use teistro_core::catalogue::{CharaKaraka, Dignity, Graha, Rashi};

    use super::*;
    use crate::chart::Upapada;
    use crate::chart::{
        Benefics, Conjunction, DignityMatch, Gathering, Houses, NodeMotion, NodeSides,
    };
    use crate::language::Karaka;
    use crate::reference::SignRef;
    use crate::trace::Step;

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
                navamsha: Rashi::Aries,
            }; 10],
            tithi: None,
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
            planet: JUPITER.into(),
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
            planet: MARS.into(),
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

        let retro = Condition::PlanetRetrograde {
            planet: RAHU.into(),
        };
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
            planets: vec![SUN.into(), MOON.into()],
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
                        Condition::PlanetInKendra {
                            planet: MARS.into(),
                        },
                        Condition::PlanetDignity {
                            planet: JUPITER.into(),
                            dignities: vec![Dignity::Exalted],
                        },
                    ],
                },
                Condition::PlanetInKendra { planet: SUN.into() },
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
            from: SATURN.into(),
            target: SignRef::from(target),
        };
        place(&mut c, SUN, Rashi::Pisces);
        assert!(holds(&c, ENGINE, &aspect(SUN)).0);
        place(&mut c, SUN, Rashi::Taurus);
        assert!(!holds(&c, ENGINE, &aspect(SUN)).0);
        // The lagna aspects nothing.
        let lagna = Condition::PlanetAspectsHouse {
            from: Body::Lagna.into(),
            house_ruled: house(7),
        };
        assert!(!holds(&c, ENGINE, &lagna).0);
    }

    /// A condition as a rule writes it.
    fn written(json: &str) -> Condition {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn a_house_from_the_arudha_lagna_bphs_29_30() {
        // Aries lagna, Mars in Capricorn: nine signs on and nine again is
        // Libra, the seventh, so the pada moves to Cancer.
        let mut c = chart();
        place(&mut c, MARS, Rashi::Capricorn);
        place(&mut c, MERCURY, Rashi::Leo);
        let budh = written(
            r#"{"type": "planet-in-house-from", "planet": "MERCURY", "reference": {"arudha": 1}, "houses": [2]}"#,
        );
        assert_eq!(holds(&c, ENGINE, &budh), (true, vec![MERCURY]));
        place(&mut c, MERCURY, Rashi::Virgo);
        assert!(!holds(&c, ENGINE, &budh).0);
        // The pada itself is in the fourth house, a kendra, and no body stands
        // there to take part.
        let pada = written(r#"{"type": "planet-in-kendra", "planet": {"arudha": 1}}"#);
        assert_eq!(holds(&c, ENGINE, &pada), (true, vec![]));
        let pada =
            written(r#"{"type": "planet-in-sign", "planet": {"arudha": 1}, "signs": ["CANCER"]}"#);
        assert!(holds(&c, ENGINE, &pada).0);
    }

    #[test]
    fn the_amatyakaraka_with_the_atmakaraka_s_dispositor_bphs_40_3() {
        let mut c = chart();
        place(&mut c, SATURN, Rashi::Aquarius);
        place(&mut c, Body::Graha(Graha::Venus), Rashi::Aquarius);
        c.placements[Body::Graha(Graha::Venus).index()].karaka7 = Some(CharaKaraka::Atmakaraka);
        c.placements[MARS.index()].karaka7 = Some(CharaKaraka::Amatyakaraka);
        place(&mut c, MARS, Rashi::Capricorn);
        let minister = written(
            r#"{"type": "planet-conjunct", "planets": [{"karaka": "AmK"}, {"lordOf": {"karaka": "AK"}}]}"#,
        );
        assert!(!holds(&c, ENGINE, &minister).0);
        place(&mut c, MARS, Rashi::Aquarius);
        assert_eq!(holds(&c, ENGINE, &minister), (true, vec![MARS, SATURN]));
        // No graha holds the karaka: the reference resolves to nothing.
        c.placements[MARS.index()].karaka7 = None;
        assert!(!holds(&c, ENGINE, &minister).0);
    }

    #[test]
    fn the_second_from_the_lord_of_the_seventh_from_the_upapada_bphs_30_42() {
        // Aries lagna: the twelfth, Pisces, has Jupiter in Sagittarius; nine on
        // and nine again is Virgo, the seventh from Pisces, so the upapada
        // moves to Gemini. Its seventh is Sagittarius, whose lord Jupiter
        // stands there; the second from him is Capricorn.
        let mut c = chart();
        place(&mut c, JUPITER, Rashi::Sagittarius);
        place(&mut c, RAHU, Rashi::Capricorn);
        let teeth = written(
            r#"{"type": "planet-in-house-from", "planet": "RAHU",
                "reference": {"lordOf": {"from": "UPAPADA", "house": 7}}, "houses": [2]}"#,
        );
        assert_eq!(holds(&c, ENGINE, &teeth), (true, vec![RAHU]));
        place(&mut c, RAHU, Rashi::Aquarius);
        assert!(!holds(&c, ENGINE, &teeth).0);
    }

    #[test]
    fn the_fourth_from_the_karakamsha_bphs_40_14() {
        let mut c = chart();
        let venus = Body::Graha(Graha::Venus);
        c.placements[SATURN.index()].karaka7 = Some(CharaKaraka::Atmakaraka);
        c.placements[SATURN.index()].navamsha = Rashi::Pisces;
        place(&mut c, venus, Rashi::Gemini);
        place(&mut c, MOON, Rashi::Gemini);
        let insignia = written(
            r#"{"type": "and", "conditions": [
                {"type": "planet-in-house-from", "planet": "VENUS", "reference": {"navamsha": {"karaka": "AK"}}, "houses": [4]},
                {"type": "planet-in-house-from", "planet": "MOON", "reference": {"navamsha": {"karaka": "AK"}}, "houses": [4]}
            ]}"#,
        );
        assert_eq!(holds(&c, ENGINE, &insignia), (true, vec![venus, MOON]));
        assert_eq!(
            written(
                r#"{"type": "planet-in-sign", "planet": {"navamsha": {"karaka": "AK"}}, "signs": ["PISCES"]}"#
            ),
            Condition::PlanetInSign {
                planet: SignRef::karakamsha(),
                signs: vec![Rashi::Pisces],
            }
        );
        c.placements[SATURN.index()].navamsha = Rashi::Aries;
        assert!(!holds(&c, ENGINE, &insignia).0);
    }

    #[test]
    fn the_upapada_is_the_twelfth_s_pada_or_by_the_lagna_s_parity() {
        // A Taurus lagna, every graha in Aries. The twelfth, Aries, holds its
        // own lord, so its pada moves to Capricorn; the second, Gemini, has
        // Mercury ten signs on, and ten again is Aquarius, which stays.
        let mut c = chart();
        place(&mut c, Body::Lagna, Rashi::Taurus);
        let upapada = |sign: &str| {
            written(&format!(
                r#"{{"type": "planet-in-sign", "planet": "UPAPADA", "signs": ["{sign}"]}}"#
            ))
        };
        let parity = Readings {
            upapada: Upapada::ByLagnaParity,
            ..ENGINE
        };
        assert!(holds(&c, ENGINE, &upapada("CAPRICORN")).0);
        assert!(!holds(&c, parity, &upapada("CAPRICORN")).0);
        assert!(holds(&c, parity, &upapada("AQUARIUS")).0);
        // An odd lagna reads the twelfth either way.
        place(&mut c, Body::Lagna, Rashi::Gemini);
        assert_eq!(
            holds(&c, ENGINE, &upapada("TAURUS")),
            holds(&c, parity, &upapada("TAURUS"))
        );
    }

    fn rule(conditions: Vec<Condition>, cancellations: Vec<Condition>) -> Rule {
        Rule {
            key: String::from("EXAMPLE"),
            category: String::from("example"),
            source: crate::language::Source {
                text: String::from("an example"),
                chapter: None,
                verse: None,
                note: None,
            },
            conditions,
            cancellations,
        }
    }

    #[test]
    fn an_explanation_holds_the_checks_made_and_only_those() {
        let c = chart();
        let example = rule(
            vec![written(
                r#"{"type": "or", "conditions": [
                    {"type": "and", "conditions": [
                        {"type": "planet-in-kendra", "planet": "MARS"},
                        {"type": "planet-dignity", "planet": "JUPITER", "dignities": ["EXALTED"]},
                        {"type": "planet-combust", "planet": "SUN"}
                    ]},
                    {"type": "planet-in-kendra", "planet": {"lordOf": 4}},
                    {"type": "planet-in-kendra", "planet": "VENUS"}
                ]}"#,
            )],
            vec![written(
                r#"{"type": "planet-dignity", "planet": {"karaka": "AmK"}, "dignities": ["EXALTED"]}"#,
            )],
        );
        let evaluator = Evaluator::new(&c, ENGINE);
        let explanation = evaluator.explain(&example);
        assert_eq!(explanation.result, evaluator.evaluate(&example));
        let [or] = explanation.conditions.as_slice() else {
            panic!("one condition")
        };
        // The `and` stopped at Jupiter's dignity, so the Sun was never asked;
        // the `or` stopped at the Moon, the fourth's lord, so Venus was not.
        let kinds = |steps: &[Step<'_>]| {
            steps
                .iter()
                .map(|s| (s.condition.kind(), s.held))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            kinds(&or.steps),
            [("and", false), ("planet-in-kendra", true)]
        );
        assert_eq!(
            kinds(&or.steps[0].steps),
            [("planet-in-kendra", true), ("planet-dignity", false)]
        );
        assert_eq!(
            or.added,
            [MARS, MOON],
            "the engine gathers the failed branch's Mars"
        );
        assert_eq!(
            or.steps[1]
                .resolved
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            [
                "house 4 is CANCER",
                "the lord of house 4 is MOON",
                "the lord of house 4 stands in ARIES, house 1"
            ]
        );
        // Present, so its cancellation was checked: no graha holds the karaka.
        let [cancellation] = explanation.cancellations.as_slice() else {
            panic!("one cancellation")
        };
        assert!(!cancellation.held);
        assert_eq!(cancellation.resolved[0].to_string(), "the AmK is no body");
        assert_eq!(
            serde_json::to_value(cancellation).unwrap(),
            serde_json::json!({
                "type": "planet-dignity",
                "held": false,
                "added": [],
                "resolved": [{"kind": "body", "reference": {"karaka": "AmK"}, "body": null}],
                "steps": []
            })
        );
        assert!(explanation.to_string().starts_with(
            "EXAMPLE: present, MARS, MOON in house 1\n  holds or, adding MARS, MOON\n"
        ));

        // Not present: no cancellation is checked.
        let absent = rule(
            vec![written(
                r#"{"type": "planet-in-house", "planet": "SUN", "houses": [7]}"#,
            )],
            example.cancellations.clone(),
        );
        let explanation = evaluator.explain(&absent);
        assert!(!explanation.result.present && explanation.cancellations.is_empty());
        assert_eq!(
            explanation.to_string(),
            "EXAMPLE: not present\n  fails planet-in-house\n    SUN stands in ARIES, house 1\n"
        );
    }

    #[test]
    fn a_table_degree_is_the_stretch_the_reading_names() {
        use crate::chart::Bhaga;
        use crate::table::Tables;

        // The Moon's Mrityu Bhaga in Mesha is her eighth degree (Jataka
        // Parijata ch. 1 v. 57); in Brihat Prajapatya's row, her 26th.
        let mut c = chart();
        let mrityu = written(
            r#"{"type": "planet-at-table-degree", "planet": "MOON", "table": "MRITYU_BHAGA"}"#,
        );
        let at = |c: &RuleChart, bhaga, condition: &Condition| {
            let readings = Readings { bhaga, ..ENGINE };
            let mut into = Participants::default();
            Evaluator::new(c, readings)
                .with_tables(Tables::classical())
                .holds(condition, &mut into)
        };
        let spans = [
            Bhaga::Running,
            Bhaga::Centred,
            Bhaga::Completed,
            Bhaga::WithinOne,
        ];
        for (degrees, held) in [
            (7.2, [true, false, false, true]),
            (7.5, [true, true, false, true]),
            (8.0, [false, true, true, true]),
            (8.5, [false, false, true, true]),
            (9.0, [false, false, false, true]),
            (6.9, [false, false, false, false]),
        ] {
            c.placements[MOON.index()].longitude = degrees;
            let answers = spans.map(|bhaga| at(&c, bhaga, &mrityu));
            assert_eq!(answers, held, "the Moon at {degrees}° of Mesha");
        }
        assert_eq!(Readings::default().bhaga, Bhaga::Running);

        // Another table, another degree; a table with no row for the body, or
        // an evaluator given no tables, never holds.
        c.placements[MOON.index()].longitude = 25.5;
        let prajapatya = written(
            r#"{"type": "planet-at-table-degree", "planet": "MOON", "table": "MRITYU_BHAGA_MOON_BRIHAT_PRAJAPATYA"}"#,
        );
        assert!(at(&c, Bhaga::Running, &prajapatya));
        let sun = written(
            r#"{"type": "planet-at-table-degree", "planet": "SUN", "table": "MRITYU_BHAGA_MOON_BRIHAT_PRAJAPATYA"}"#,
        );
        c.placements[SUN.index()].longitude = 19.5;
        assert!(!at(&c, Bhaga::WithinOne, &sun));
        assert!(!holds(&c, ENGINE, &prajapatya).0);

        let rule = rule(vec![prajapatya], Vec::new());
        let explanation = Evaluator::new(&c, Readings::TEXTS)
            .with_tables(Tables::classical())
            .explain(&rule);
        assert_eq!(
            explanation.conditions[0].resolved[0].to_string(),
            "MRITYU_BHAGA_MOON_BRIHAT_PRAJAPATYA gives MOON in ARIES its degree 26, and it stands 25.50° in"
        );
    }

    #[test]
    fn the_signs_a_tithi_burns_hold_only_with_a_tithi_and_the_table() {
        use teistro_core::catalogue::Tithi;

        use crate::table::Tables;

        fn evaluator(c: &RuleChart) -> Evaluator<'_> {
            Evaluator::new(c, Readings::TEXTS).with_tables(Tables::classical())
        }

        // Shashthi burns Mesha and Simha in either paksha.
        let mut c = chart();
        place(&mut c, MARS, Rashi::Leo);
        let burnt = |planet: &str| {
            written(&format!(
                r#"{{"type": "planet-in-table-sign", "planet": {planet}, "table": "DAGDHA_RASHI"}}"#
            ))
        };
        let held = |c: &RuleChart, condition: &Condition| {
            let mut into = Participants::default();
            (
                evaluator(c).holds(condition, &mut into),
                into.iter().collect::<Vec<_>>(),
            )
        };
        assert!(!held(&c, &burnt(r#""MARS""#)).0, "no tithi, no burnt sign");
        for tithi in [Tithi::ShuklaShashthi, Tithi::KrishnaShashthi] {
            c.tithi = Some(tithi);
            assert_eq!(held(&c, &burnt(r#""MARS""#)), (true, vec![MARS]));
            // The lagna's lord, Mars, stands in a burnt sign too; the sign of the
            // twelfth, Pisces, is not burnt.
            assert!(held(&c, &burnt(r#"{"lordOf": 1}"#)).0);
            assert!(!held(&c, &burnt("12")).0);
        }
        c.tithi = Some(Tithi::Purnima);
        assert!(!held(&c, &burnt(r#""MARS""#)).0);

        let rule = rule(vec![burnt(r#""MARS""#)], Vec::new());
        assert!(rule.reads_tithi());
        c.tithi = None;
        assert_eq!(
            evaluator(&c).explain(&rule).conditions[0].resolved[0].to_string(),
            "DAGDHA_RASHI needs a tithi, and the chart has none"
        );
        c.tithi = Some(Tithi::KrishnaShashthi);
        assert_eq!(
            evaluator(&c).explain(&rule).conditions[0].resolved[0].to_string(),
            "DAGDHA_RASHI gives KRISHNA_SHASHTHI ARIES, LEO"
        );
        assert!(
            !self::rule(
                vec![written(r#"{"type": "planet-in-kendra", "planet": "SUN"}"#)],
                Vec::new()
            )
            .reads_tithi()
        );
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
