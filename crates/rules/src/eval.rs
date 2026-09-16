//! Evaluating rules over a chart.

use serde::Serialize;
use teistro_aspect::{conjunction, drishti};
use teistro_core::catalogue::{Dignity, Graha, Modality, Nakshatra, Rashi};
use teistro_core::settings::NodeAspects;

use teistro_points::arudha;

use crate::chart::{
    AspectGathering, Benefics, Conjunction, DignityMatch, Eclipse, Gathering, Houses,
    NATURAL_BENEFICS, NATURAL_MALEFICS, NodeMotion, NodeSides, Panchanga, Placement, PointAt,
    Readings, RuleChart, Strengths, Upapada, VargaSigns,
};
use crate::language::{Body, Condition, EclipseKind, Edge, House, KarakaScheme, NodeSide};
use crate::reference::{BodyRef, BodySubject, Class, SignRef, Subject};
use crate::rule::{NetStatus, Outcome, Rule, Severity};
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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RuleResult {
    /// Whether it is present.
    pub present: bool,
    /// The bodies its conditions consulted, when present.
    pub participants: Participants,
    /// The houses its participant grahas stand in, distinct and ascending.
    pub houses: Vec<House>,
    /// Where it was found from, in order: its conditions, then each group that
    /// held.
    pub found_from: Vec<Found>,
    /// Which of its cancellations held, by their place in the rule, when
    /// present.
    pub cancellations: Vec<usize>,
    /// How grave it is, when present and the rule says.
    pub severity: Option<u16>,
    /// What the rule says happens, when present and its verse says: as many
    /// statements as the verse makes.
    pub outcomes: Vec<Outcome>,
    /// Whether it stands after its cancellations, when present.
    pub status: Option<NetStatus>,
}

/// A place a rule was found from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "kind", content = "group", rename_all = "kebab-case")]
pub enum Found {
    /// Its conditions all held.
    Conditions,
    /// The group at this place in the rule held.
    Group(usize),
}

impl RuleResult {
    /// Whether a cancellation held.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        !self.cancellations.is_empty()
    }

    /// The span of life the rule's verse gives, in days, when it gives one.
    #[must_use]
    pub fn life_span(&self) -> Option<f64> {
        self.outcomes.iter().find_map(Outcome::days)
    }

    /// What the rule's verse says follows, in words, when it says it in words.
    #[must_use]
    pub fn effect(&self) -> Option<&str> {
        self.outcomes.iter().find_map(Outcome::text)
    }

    /// The class of life the verse gives, when present and it gives one.
    #[must_use]
    pub fn life_class(&self) -> Option<crate::rule::LifeClass> {
        self.outcomes.iter().find_map(Outcome::class)
    }
}

/// A chart read under a choice at every open place, ready to evaluate rules.
#[derive(Clone, Copy, Debug)]
pub struct Evaluator<'a> {
    chart: &'a RuleChart,
    readings: Readings,
    tables: &'a Tables,
    /// The divisional charts an `in-varga` condition can step into.
    vargas: &'a [VargaSigns],
    /// The points a `{"point": …}` reference can name.
    points: &'a [PointAt],
    /// The rules a `{"type": "rule"}` condition can name.
    rules: &'a [Rule],
    /// The body a `for-any` bound, which `SELF` names.
    bound: Option<Body>,
    /// Each body's benefic nature under the readings, by index.
    benefic: [bool; 10],
    /// Each body's malefic nature.
    malefic: [bool; 10],
    /// Whether each body is a maraka (BPHS ch. 44 vv. 3 to 5).
    maraka: [bool; 10],
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
        let mut evaluator = Evaluator {
            chart,
            readings,
            tables: &NO_TABLES,
            vargas: &[],
            points: &[],
            rules: &[],
            bound: None,
            benefic,
            malefic,
            maraka: [false; 10],
        };
        evaluator.maraka = evaluator.maraka_class();
        evaluator
    }

    /// The marakas, BPHS ch. 44 vv. 3 to 5: the lords of the second and the
    /// seventh, the malefics standing in either, and the malefics joining
    /// either lord. Malefic means malefic under the readings, and a house is
    /// counted as the readings count houses.
    fn maraka_class(&self) -> [bool; 10] {
        let mut maraka = [false; 10];
        let houses = House::MARAKAS;
        let lords = houses.map(|house| graha_body(self.house_sign(house).attributes().lord));
        for lord in lords {
            set(&mut maraka, lord.index(), true);
        }
        for body in Body::ALL {
            let malefic = self.malefic.get(body.index()).copied().unwrap_or(false);
            let placed = houses.contains(&self.house_of(body));
            let joining = lords
                .iter()
                .any(|lord| *lord != body && self.at(*lord).sign == self.at(body).sign);
            if malefic && (placed || joining) {
                set(&mut maraka, body.index(), true);
            }
        }
        maraka
    }

    /// The same evaluator, looking tables up in `tables`; without, a table
    /// predicate never holds. Check rules against the set first with
    /// [`Tables::check`].
    #[must_use]
    pub const fn with_tables(self, tables: &'a Tables) -> Evaluator<'a> {
        Evaluator { tables, ..self }
    }

    /// The same evaluator, able to read `in-varga` conditions in these
    /// divisional charts; without the one a condition names, it never holds.
    #[must_use]
    pub const fn with_vargas(self, vargas: &'a [VargaSigns]) -> Evaluator<'a> {
        Evaluator { vargas, ..self }
    }

    /// The same evaluator, able to resolve these points; a point it was not
    /// given resolves to nothing, and the condition reading it never holds.
    #[must_use]
    pub const fn with_points(self, points: &'a [PointAt]) -> Evaluator<'a> {
        Evaluator { points, ..self }
    }

    /// The same evaluator, able to read a rule that names another by key. A
    /// key the set does not hold never holds; a rule that reaches itself is
    /// refused when the set is checked
    /// ([`Rule::references`](crate::Rule::references)).
    #[must_use]
    pub const fn with_rules(self, rules: &'a [Rule]) -> Evaluator<'a> {
        Evaluator { rules, ..self }
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
            BodyRef::Bound => self.bound,
            BodyRef::ExaltedIn(sign) => {
                let sign = self.spot(sign, rec)?.sign;
                Body::ALL
                    .into_iter()
                    .find(|body| exaltation(*body) == Some(sign))
            }
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
            SignRef::Point(point) => Spot::sign(
                self.points
                    .iter()
                    .find(|at| at.point == *point)
                    .map(|at| at.sign)?,
                None,
            ),
            SignRef::Exaltation(body) => Spot::sign(exaltation(self.body(body, rec)?)?, None),
            SignRef::Debilitation(body) => Spot::sign(debilitation(self.body(body, rec)?)?, None),
            SignRef::Badhaka(sign) => {
                let sign = self.spot(sign, rec)?.sign;
                let house = match sign.attributes().modality {
                    Modality::Chara => 10,
                    Modality::Sthira => 8,
                    // Dual, and any modality a later catalogue adds, the
                    // seventh: the dual reading.
                    _ => 6,
                };
                Spot::sign(step(sign, house), None)
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
        let Some(class) = subject.class() else {
            let Subject::Ref(reference) = subject else {
                return false;
            };
            return self
                .spot(reference, rec)
                .is_some_and(|spot| self.reached(spot, into, meets(spot)));
        };
        let nature = self.members(class);
        let found = Body::ALL.into_iter().find(|body| {
            nature.get(body.index()).copied().unwrap_or(false) && meets(self.standing(*body))
        });
        if let Some(body) = found {
            into.push(body);
        }
        found.is_some()
    }

    /// The bodies a subject stands for, in [`Body::ALL`]'s order: the one it
    /// names, or every benefic or every malefic.
    fn subject_bodies<'c, R: Recorder<'c>>(
        &self,
        subject: &BodySubject,
        rec: &mut R,
    ) -> impl Iterator<Item = Body> + use<'_, 'c, R> {
        let nature = match (subject, subject.class()) {
            (BodySubject::Ref(reference), _) => {
                return Either::One(self.body(reference, rec).into_iter());
            }
            (_, Some(class)) => self.members(class),
            (_, None) => [false; 10],
        };
        Either::Many(
            Body::ALL
                .into_iter()
                .filter(move |body| nature.get(body.index()).copied().unwrap_or(false)),
        )
    }

    /// The bodies that meet a condition naming each of them `SELF` in turn.
    /// Every body is tried; what the conditions inside consulted is the
    /// rule's business, not its participants'.
    fn meeting<'c>(
        &self,
        planets: &[Body],
        then: &'c Condition,
        rec: &mut impl Recorder<'c>,
    ) -> Participants {
        let mut met = Participants::default();
        for body in planets {
            let bound = Evaluator {
                bound: Some(*body),
                ..*self
            };
            if bound.check(then, &mut Participants::default(), rec) {
                met.push(*body);
            }
        }
        met
    }

    /// Which bodies belong to a class of grahas on this chart.
    const fn members(&self, class: Class) -> [bool; 10] {
        match class {
            Class::Benefic => self.benefic,
            Class::Malefic => self.malefic,
            Class::Maraka => self.maraka,
        }
    }

    /// The bodies of a subject's class. A subject naming a sign rather than a
    /// class has none, and every caller has already handled that case.
    fn of_that_nature(&self, subject: &Subject) -> impl Iterator<Item = Body> + use<'_> {
        let nature = subject
            .class()
            .map_or([false; 10], |class| self.members(class));
        Body::ALL
            .into_iter()
            .filter(move |body| nature.get(body.index()).copied().unwrap_or(false))
    }

    /// The nine grahas standing in a house counted from a spot.
    fn grahas_in(&self, on: Spot, house: u8) -> Participants {
        let house = counting(on, house);
        let mut counted = Participants::default();
        for body in Body::ALL.into_iter().take(9) {
            if House::between(on.sign, self.at(body).sign).get() == house {
                counted.push(body);
            }
        }
        counted
    }

    /// The chart it reads.
    #[must_use]
    pub const fn chart(&self) -> &RuleChart {
        self.chart
    }

    /// The points it was given.
    #[must_use]
    pub const fn points(&self) -> &[PointAt] {
        self.points
    }

    /// The sign a house falls in, whole signs from the lagna.
    #[must_use]
    pub fn house_sign(&self, house: House) -> Rashi {
        step(self.chart.lagna(), house.get() - 1)
    }

    /// What the chart says of strength, or nothing at all: a chart that
    /// carries none answers false to every question of strength.
    #[must_use]
    pub fn strengths(&self) -> Strengths {
        self.chart.strengths.unwrap_or(Strengths::NONE)
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
            Condition::AllPlanetsBetweenNodes { side } => {
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
                // The rule's own side, or the side the reading takes when it
                // names none.
                let held = match (side, self.readings.node_sides) {
                    (Some(NodeSide::Rahu), _) | (None, NodeSides::RahuToKetu) => ketu_side == 0,
                    (Some(NodeSide::Ketu), _) => rahu_side == 0,
                    (None, NodeSides::Either) => rahu_side == 0 || ketu_side == 0,
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
                let Some(target) = self.spot(target, rec) else {
                    return false;
                };
                let found = self.subject_bodies(from, rec).find(|body| {
                    aspects(
                        *body,
                        self.at(*body).sign,
                        target.sign,
                        self.readings.node_aspects,
                    )
                });
                if let Some(body) = found {
                    if self.readings.aspect_gathering == AspectGathering::Both {
                        into.push(body);
                        into.push_through(target);
                    }
                }
                found.is_some()
            }
            Condition::PlanetInNakshatra {
                planet,
                nakshatras,
                padas,
            } => self.body_meets(planet, into, rec, |body| {
                let (nakshatra, pada) = star(self.at(body).longitude);
                nakshatras.contains(&nakshatra)
                    && (padas.is_empty() || padas.iter().any(|named| named.get() == pada))
            }),
            Condition::SameNakshatra { of, as_body } => {
                let (Some(one), Some(other)) = (self.body(of, rec), self.body(as_body, rec)) else {
                    return false;
                };
                let held = star(self.at(one).longitude).0 == star(self.at(other).longitude).0;
                if held {
                    into.push(one);
                    into.push(other);
                }
                held
            }
            Condition::PlanetStrong { planet } => {
                self.body_meets(planet, into, rec, |body| self.strengths().is_strong(body))
            }
            Condition::PlanetWeak { planet } => {
                self.body_meets(planet, into, rec, |body| self.strengths().is_weak(body))
            }
            Condition::PlanetStrongerThan { planet, than } => {
                let (Some(one), Some(other)) = (self.body(planet, rec), self.body(than, rec))
                else {
                    return false;
                };
                let held = self.strengths().exceeds(one, other);
                if held {
                    into.push(one);
                }
                held
            }
            Condition::RashiAspects { from, target } => {
                let Some(target) = self.spot(target, rec) else {
                    return false;
                };
                let found = match from {
                    Subject::Ref(reference) => self.spot(reference, rec),
                    _ => self
                        .of_that_nature(from)
                        .map(|body| self.standing(body))
                        .find(|spot| rashi_aspects(spot.sign, target.sign)),
                };
                let Some(spot) = found.filter(|spot| rashi_aspects(spot.sign, target.sign)) else {
                    return false;
                };
                into.push_through(spot);
                into.push_through(target);
                true
            }
            Condition::Argala { on, place } => {
                let Some(on) = self.spot(on, rec) else {
                    return false;
                };
                let (from, obstructed) = place.houses();
                let (intervening, obstructing) =
                    (self.grahas_in(on, from), self.grahas_in(on, obstructed));
                // Verse 4 gives two tests and either serves: the intervening
                // grahas outnumber the obstructing ones, or one of them is
                // stronger than every one of them.
                let outnumber = intervening.len() > obstructing.len();
                let strengths = self.strengths();
                let stronger = intervening.iter().any(|one| {
                    obstructing
                        .iter()
                        .all(|other| strengths.exceeds(one, other))
                });
                let held = !intervening.is_empty() && (outnumber || stronger);
                if held {
                    into.extend(intervening);
                }
                held
            }
            Condition::VipareetaArgala { on } => {
                let Some(on) = self.spot(on, rec) else {
                    return false;
                };
                let mut counted = Participants::default();
                for body in Body::ALL.into_iter().take(9) {
                    if self.malefic.get(body.index()).copied().unwrap_or(false)
                        && House::between(on.sign, self.at(body).sign).get() == counting(on, 3)
                    {
                        counted.push(body);
                    }
                }
                let held = counted.len() >= 3;
                if held {
                    into.extend(counted);
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
                let tithi = self.panchanga().map(|p| p.tithi);
                let signs = match (self.tables.get(table.as_str()), tithi) {
                    (Some(found @ Table::SignsByTithi { .. }), Some(tithi)) => found.signs(tithi),
                    _ => &[],
                };
                rec.resolved(|| Resolved::Burnt {
                    table: table.clone(),
                    tithi,
                    signs: signs.to_vec(),
                });
                self.sign_meets(planet, into, rec, |spot| signs.contains(&spot.sign))
            }
            Condition::PlanetAspectsHouse { from, house_ruled } => {
                let target = step(self.chart.lagna(), house_ruled.get() - 1);
                let found = self.subject_bodies(from, rec).find(|body| {
                    aspects(
                        *body,
                        self.at(*body).sign,
                        target,
                        self.readings.node_aspects,
                    )
                });
                match found {
                    Some(body) if self.readings.aspect_gathering == AspectGathering::Both => {
                        self.single(body, into, true)
                    }
                    found => found.is_some(),
                }
            }
            Condition::LordOfHouseDebilitated { house_ruled } => {
                let lord = self.lord_of(*house_ruled, rec);
                self.dignity_meets(self.at(lord).dignity, &[Dignity::Debilitated])
            }
            Condition::LordOfHouseCombust { house_ruled } => {
                let lord = self.lord_of(*house_ruled, rec);
                lord != graha_body(Graha::Sun) && self.at(lord).combust
            }
            Condition::LordOfHouseStrong { house_ruled } => {
                let lord = self.lord_of(*house_ruled, rec);
                self.dignity_meets(
                    self.at(lord).dignity,
                    &[Dignity::OwnSign, Dignity::Exalted, Dignity::Mooltrikona],
                )
            }
            Condition::LordOfHouseIs {
                house_ruled,
                planets,
            } => planets.contains(&self.lord_of(*house_ruled, rec)),
            Condition::LordOfHouseConjunctPlanet {
                house_ruled,
                with_planet,
            } => {
                let lord = self.lord_of(*house_ruled, rec);
                self.at(lord).sign == self.at(*with_planet).sign
            }
            Condition::LagnaInSign { signs } => signs.contains(&self.chart.lagna()),
            Condition::PlanetInHouseAndSign {
                planet,
                houses,
                signs,
            } => houses.contains(&self.house_of(*planet)) && signs.contains(&self.at(*planet).sign),
            Condition::PlanetAtGandanta {
                planet,
                orb_degrees,
            } => {
                let placement = self.at(*planet);
                let orb = orb_degrees.unwrap_or(GANDANTA_ORB);
                let in_sign = placement.longitude.rem_euclid(30.0);
                match placement.sign {
                    Rashi::Cancer | Rashi::Scorpio | Rashi::Pisces => 30.0 - in_sign <= orb,
                    Rashi::Leo | Rashi::Sagittarius | Rashi::Aries => in_sign <= orb,
                    _ => false,
                }
            }
            Condition::PlanetInDegrees { planet, from, to } => {
                self.body_meets(planet, into, rec, |body| {
                    let in_sign = self.at(body).longitude.rem_euclid(30.0);
                    in_sign >= *from && in_sign < *to
                })
            }
            Condition::PanchangaTithi { tithis } => {
                self.panchanga().is_some_and(|p| tithis.contains(&p.tithi))
            }
            Condition::PanchangaPaksha { paksha } => self
                .panchanga()
                .is_some_and(|p| p.tithi.attributes().paksha == *paksha),
            Condition::PanchangaVara { varas } => {
                self.panchanga().is_some_and(|p| varas.contains(&p.vara))
            }
            Condition::PanchangaNakshatra { nakshatras, padas } => {
                self.panchanga().is_some_and(|p| {
                    nakshatras.contains(&p.nakshatra)
                        && (padas.is_empty() || padas.contains(&p.pada))
                })
            }
            Condition::PanchangaYoga { yogas } => {
                self.panchanga().is_some_and(|p| yogas.contains(&p.yoga))
            }
            Condition::PanchangaKarana { karanas } => self
                .panchanga()
                .is_some_and(|p| karanas.contains(&p.karana)),
            Condition::BirthDuringEclipse { kind } => self
                .panchanga()
                .and_then(|p| p.eclipse)
                .is_some_and(|eclipse| match (kind, eclipse) {
                    (None | Some(EclipseKind::Any), _)
                    | (Some(EclipseKind::Solar), Eclipse::Solar)
                    | (Some(EclipseKind::Lunar), Eclipse::Lunar) => true,
                    (Some(EclipseKind::Solar), Eclipse::Lunar)
                    | (Some(EclipseKind::Lunar), Eclipse::Solar) => false,
                }),
            Condition::BirthByDay => self.panchanga().is_some_and(|p| p.by_day == Some(true)),
            Condition::BirthOnSankranti { .. } => self.panchanga().is_some_and(|p| p.on_sankranti),
            Condition::AtLimbEdge {
                limb,
                edge,
                ghatikas,
            } => self
                .panchanga()
                .and_then(|p| p.spans.of(*limb))
                .is_some_and(|span| {
                    let stood = match edge {
                        Edge::First => span.elapsed,
                        Edge::Last => span.remaining,
                    };
                    stood >= 0.0 && stood <= *ghatikas
                }),
            Condition::ForAny { planets, then } => {
                let met = self.meeting(planets, then, rec);
                into.extend(met);
                !met.is_empty()
            }
            Condition::CountOf {
                planets,
                then,
                at_least,
                at_most,
            } => {
                let met = self.meeting(planets, then, rec);
                let held = met.len() >= usize::from(*at_least)
                    && at_most.is_none_or(|most| met.len() <= usize::from(most));
                if held {
                    into.extend(met);
                }
                held
            }
            Condition::InVarga { varga, condition } => {
                let Some(signs) = self.vargas.iter().find(|signs| signs.varga == *varga) else {
                    return false;
                };
                let chart = self.chart.in_varga(signs);
                let within = Evaluator::new(&chart, self.readings)
                    .with_tables(self.tables)
                    .with_vargas(self.vargas);
                let within = Evaluator {
                    bound: self.bound,
                    ..within
                };
                within.check(condition, into, rec)
            }
            Condition::CountInHouses {
                planets,
                houses,
                from,
                at_least,
                except,
            } => {
                let Some(from) = self.spot(from, rec) else {
                    return false;
                };
                let mut excepted = Participants::default();
                for reference in except {
                    if let Some(body) = self.body(reference, rec) {
                        excepted.push(body);
                    }
                }
                let mut counted = Participants::default();
                for body in self.subject_bodies(planets, rec) {
                    if !excepted.iter().any(|other| other == body)
                        && houses.contains(&House::between(from.sign, self.at(body).sign))
                    {
                        counted.push(body);
                    }
                }
                let held = counted.len() >= usize::from(*at_least);
                if held {
                    into.extend(counted);
                }
                held
            }
            Condition::CountAspecting {
                planets,
                target,
                at_least,
            } => {
                let Some(target) = self.spot(target, rec) else {
                    return false;
                };
                let mut counted = Participants::default();
                for body in self.subject_bodies(planets, rec) {
                    if aspects(
                        body,
                        self.at(body).sign,
                        target.sign,
                        self.readings.node_aspects,
                    ) {
                        counted.push(body);
                    }
                }
                let held = counted.len() >= usize::from(*at_least);
                if held && self.readings.aspect_gathering == AspectGathering::Both {
                    into.extend(counted);
                    into.push_through(target);
                }
                held
            }
            Condition::RuleHolds { key } => {
                let Some(rule) = self.rules.iter().find(|rule| rule.key == *key) else {
                    return false;
                };
                // A referenced rule answers as it would on its own, and what it
                // consulted is its own business.
                self.evaluate(rule).present
            }
            Condition::SameSign { of, as_sign } => {
                let (Some(one), Some(other)) = (self.spot(of, rec), self.spot(as_sign, rec)) else {
                    return false;
                };
                one.sign == other.sign
            }
            Condition::PlanetIs { planet, class } => self.body_meets(planet, into, rec, |body| {
                self.members(*class)
                    .get(body.index())
                    .copied()
                    .unwrap_or(false)
            }),
            Condition::SameBody { of, as_body } => {
                let (Some(one), Some(other)) = (self.body(of, rec), self.body(as_body, rec)) else {
                    return false;
                };
                one == other
            }
        }
    }

    fn panchanga(&self) -> Option<&Panchanga> {
        self.chart.panchanga.as_ref()
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
    /// A rule the language cannot evaluate answers not present; ask
    /// [`Rule::is_evaluable`] to tell the two apart.
    #[must_use]
    pub fn evaluate(&self, rule: &Rule) -> RuleResult {
        self.run(rule, &mut NoTrace, &mut Vec::new(), &mut NoTrace)
    }

    /// A rule's answer with how it was reached: each condition checked, in
    /// order, whether it held, the bodies it added and every reference it
    /// resolved on the way; then each group's and each cancellation's the same
    /// way.
    ///
    /// It allocates the trace; [`Evaluator::evaluate`] answers the same without
    /// one.
    #[must_use]
    pub fn explain<'c>(&self, rule: &'c Rule) -> Explanation<'c> {
        let (mut conditions, mut groups, mut cancellations) =
            (Tracer::default(), Vec::new(), Tracer::default());
        let result = self.run(rule, &mut conditions, &mut groups, &mut cancellations);
        Explanation {
            rule,
            result,
            conditions: conditions.finish(),
            groups: groups.into_iter().map(Tracer::finish).collect(),
            cancellations: cancellations.finish(),
        }
    }

    fn run<'c, R: Recorder<'c> + Default>(
        &self,
        rule: &'c Rule,
        conditions: &mut R,
        groups: &mut Vec<R>,
        cancelling: &mut R,
    ) -> RuleResult {
        let absent = RuleResult {
            present: false,
            participants: Participants::default(),
            houses: Vec::new(),
            found_from: Vec::new(),
            cancellations: Vec::new(),
            severity: None,
            outcomes: Vec::new(),
            status: None,
        };
        if !rule.is_evaluable() {
            return absent;
        }
        let mut participants = Participants::default();
        let mut found_from = Vec::new();
        if !rule.conditions.is_empty() {
            if !rule
                .conditions
                .iter()
                .all(|c| self.check(c, &mut participants, conditions))
            {
                return absent;
            }
            found_from.push(Found::Conditions);
        }
        // Every group is tried, and each that holds adds its bodies.
        for (at, group) in rule.groups.iter().enumerate() {
            let mut recorder = R::default();
            let mut own = Participants::default();
            if group
                .conditions
                .iter()
                .all(|c| self.check(c, &mut own, &mut recorder))
            {
                participants.extend(own);
                found_from.push(Found::Group(at));
            }
            groups.push(recorder);
        }
        if !rule.groups.is_empty() && !found_from.iter().any(|f| matches!(f, Found::Group(_))) {
            return absent;
        }
        let mut houses: Vec<House> = participants
            .iter()
            .filter(|b| *b != Body::Lagna)
            .map(|b| self.house_of(b))
            .collect();
        houses.sort_unstable();
        houses.dedup();
        let cancellations: Vec<usize> = rule
            .cancellations
            .iter()
            .enumerate()
            .filter(|(_, c)| self.check(&c.condition, &mut Participants::default(), cancelling))
            .map(|(i, _)| i)
            .collect();
        let severity = rule
            .severity
            .as_ref()
            .map(|severity| self.severity(severity, rule, &found_from));
        RuleResult {
            present: true,
            participants,
            houses,
            status: Some(rule.net_status(cancellations.len(), found_from.len())),
            found_from,
            cancellations,
            severity,
            outcomes: rule.outcomes.clone(),
        }
    }

    /// A present rule's severity, from where it was found.
    fn severity(&self, severity: &Severity, rule: &Rule, found_from: &[Found]) -> u16 {
        match severity {
            Severity::Fixed { value } => *value,
            Severity::CountBased {
                per_occurrence,
                cap,
            } => found_from
                .iter()
                .map(|found| match found {
                    Found::Conditions => 1,
                    Found::Group(at) => rule.groups.get(*at).map_or(1, |group| group.weight),
                })
                .fold(0_u16, u16::saturating_add)
                .saturating_mul(*per_occurrence)
                .min(*cap),
            Severity::HouseWeighted {
                planet,
                weights,
                default,
            } => {
                let sign = self.at(*planet).sign;
                found_from
                    .iter()
                    .map(|found| match found {
                        Found::Conditions => self.chart.lagna(),
                        Found::Group(at) => rule
                            .groups
                            .get(*at)
                            .map_or(self.chart.lagna(), |g| self.at(g.reference).sign),
                    })
                    .filter_map(|from| weights.get(&House::between(from, sign)).copied())
                    .max()
                    .unwrap_or(*default)
            }
            Severity::PlanetStrengthInverse {
                base_planet,
                max,
                min,
            } => {
                let dignity = self.at(*base_planet).dignity;
                if self.dignity_meets(dignity, &[Dignity::Exalted, Dignity::OwnSign]) {
                    *min
                } else if self.dignity_meets(dignity, &[Dignity::Debilitated]) {
                    *max
                } else {
                    // Half way, a half rounded up.
                    u16::try_from((u32::from(*min) + u32::from(*max)).div_ceil(2)).unwrap_or(*max)
                }
            }
            Severity::KootShortfall { full, .. } => *full,
        }
    }
}

/// One iterator or another, so a subject's bodies need no allocation.
enum Either<A, B> {
    /// The body a reference names, if it resolves.
    One(A),
    /// Every body of a nature.
    Many(B),
}

impl<A: Iterator<Item = Body>, B: Iterator<Item = Body>> Iterator for Either<A, B> {
    type Item = Body;

    fn next(&mut self) -> Option<Body> {
        match self {
            Either::One(one) => one.next(),
            Either::Many(many) => many.next(),
        }
    }
}

/// The sign a body is exalted in, when the catalogue gives it one.
fn exaltation(body: Body) -> Option<Rashi> {
    match body {
        Body::Lagna => None,
        Body::Graha(graha) => graha.attributes().exaltation.map(|at| at.sign),
    }
}

/// The sign a body is debilitated in.
fn debilitation(body: Body) -> Option<Rashi> {
    match body {
        Body::Lagna => None,
        Body::Graha(graha) => graha.attributes().debilitation.map(|at| at.sign),
    }
}

/// The 3°20′ either side of a gandanta junction, a navamsha.
const GANDANTA_ORB: f64 = 10.0 / 3.0;

fn set(nature: &mut [bool; 10], index: usize, value: bool) {
    if let Some(slot) = nature.get_mut(index) {
        *slot = value;
    }
}

/// The sign `count` signs on from `sign`.
/// The nakshatra a sidereal longitude falls in, and its pada counted 1 to 4:
/// 13°20′ to a nakshatra, a quarter of that to a pada.
pub(crate) fn star(longitude: f64) -> (Nakshatra, u8) {
    const WIDTH: f64 = 360.0 / 27.0;
    let into = longitude.rem_euclid(360.0);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude brought into 0..360 gives 0..26 and 0..3"
    )]
    let at = (into / WIDTH) as u16;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude brought into 0..360 gives 0..26 and 0..3"
    )]
    let quarter = ((into % WIDTH) / (WIDTH / 4.0)) as u8;
    (
        Nakshatra::from_id(at.min(26)).unwrap_or(Nakshatra::Ashwini),
        quarter.min(3) + 1,
    )
}

/// Which house an intervention counts to: forward from a sign, backwards from
/// a node, whose motion runs the other way (BPHS ch. 31 v. 6).
fn counting(on: Spot, house: u8) -> u8 {
    if on.through.is_some_and(Body::is_node) {
        14 - house
    } else {
        house
    }
}

/// Whether `from` aspects `target` by rashi drishti (BPHS ch. 26 vv. 1 to 3):
/// a movable sign aspects the three fixed signs but the next, a fixed sign the
/// three movable but the last, a dual sign the other three dual signs. The
/// relation is mutual, and no sign aspects itself.
fn rashi_aspects(from: Rashi, target: Rashi) -> bool {
    use teistro_core::catalogue::Modality;

    let (one, other) = (from.attributes().modality, target.attributes().modality);
    match (one, other) {
        (Modality::Dwiswabhava, Modality::Dwiswabhava) => from != target,
        (Modality::Chara, Modality::Sthira) => target != step(from, 1),
        (Modality::Sthira, Modality::Chara) => from != step(target, 1),
        _ => false,
    }
}

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
    use crate::language::{ArgalaPlace, Karaka};
    use crate::reference::{SignRef, Subject};
    use crate::trace::Step;

    const SUN: Body = Body::Graha(Graha::Sun);
    const MOON: Body = Body::Graha(Graha::Moon);
    const MARS: Body = Body::Graha(Graha::Mars);
    const MERCURY: Body = Body::Graha(Graha::Mercury);
    const JUPITER: Body = Body::Graha(Graha::Jupiter);
    const SATURN: Body = Body::Graha(Graha::Saturn);
    const RAHU: Body = Body::Graha(Graha::Rahu);

    use crate::test_chart::{chart, house, place};

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
    fn the_marakas_are_the_second_and_seventh_lords_and_the_malefics_there_or_with_them_bphs_44_3_5()
     {
        // Aries rising: Venus lords both the second (Taurus) and the seventh (Libra).
        let mut c = chart();
        let venus = Body::Graha(Graha::Venus);
        let ketu = Body::Graha(Graha::Ketu);
        place(&mut c, venus, Rashi::Cancer);
        place(&mut c, SATURN, Rashi::Cancer); // a malefic joining the lord
        place(&mut c, MARS, Rashi::Taurus); // a malefic in the second
        place(&mut c, JUPITER, Rashi::Taurus); // a benefic in the second: no maraka
        place(&mut c, RAHU, Rashi::Libra); // a malefic in the seventh
        place(&mut c, ketu, Rashi::Aries);
        place(&mut c, SUN, Rashi::Leo);
        place(&mut c, MOON, Rashi::Sagittarius); // waxing, so a benefic
        place(&mut c, MERCURY, Rashi::Gemini);
        let marakas: Vec<Body> = {
            let evaluator = Evaluator::new(&c, ENGINE);
            Body::ALL
                .into_iter()
                .filter(|body| evaluator.maraka[body.index()])
                .collect()
        };
        assert_eq!(marakas, vec![MARS, venus, SATURN, RAHU]);

        let maraka_in = |n: u8| Condition::PlanetInHouse {
            planet: Subject::AnyMaraka,
            houses: vec![house(n)],
        };
        assert_eq!(holds(&c, ENGINE, &maraka_in(4)), (true, vec![venus]));
        assert!(!holds(&c, ENGINE, &maraka_in(5)).0);
        assert_eq!(
            written(r#"{"type": "planet-in-house", "planet": "any-maraka", "houses": [4]}"#),
            maraka_in(4)
        );
        // Venus is a maraka itself, joined in Cancer by Saturn; Jupiter in
        // Taurus is joined by Mars.
        let joined = |of: &str, at_least: u8| {
            written(&format!(
                r#"{{"type": "count-in-houses", "planets": "any-maraka", "houses": [1],
                    "from": "{of}", "except": ["{of}"], "atLeast": {at_least}}}"#
            ))
        };
        assert_eq!(holds(&c, ENGINE, &joined("VENUS", 1)), (true, vec![SATURN]));
        assert!(!holds(&c, ENGINE, &joined("VENUS", 2)).0);
        assert_eq!(holds(&c, ENGINE, &joined("JUPITER", 1)), (true, vec![MARS]));
        assert!(!holds(&c, ENGINE, &joined("SUN", 1)).0);

        let is = |planet: &str, class: &str| {
            written(&format!(
                r#"{{"type": "planet-is", "planet": "{planet}", "class": "{class}"}}"#
            ))
        };
        assert_eq!(
            holds(&c, ENGINE, &is("SATURN", "maraka")),
            (true, vec![SATURN])
        );
        assert!(!holds(&c, ENGINE, &is("JUPITER", "maraka")).0);
        assert!(holds(&c, ENGINE, &is("JUPITER", "benefic")).0);
        assert!(!holds(&c, ENGINE, &is("MOON", "malefic")).0);
    }

    #[test]
    fn a_count_of_bodies_meeting_a_condition_holds_between_its_bounds_bphs_39_44() {
        let mut c = chart();
        place(&mut c, JUPITER, Rashi::Cancer); // exalted
        place(&mut c, SATURN, Rashi::Libra); // exalted
        place(&mut c, MARS, Rashi::Capricorn); // exalted
        place(&mut c, MOON, Rashi::Leo);
        let exalted = |at_least: u8, at_most: &str| {
            written(&format!(
                r#"{{"type": "count-of", "atLeast": {at_least}{at_most},
                    "then": {{"type": "same-sign", "of": "SELF", "as": {{"exaltationOf": "SELF"}}}}}}"#
            ))
        };
        // The Sun stays in Aries, where he is exalted too: four in all.
        assert_eq!(
            holds(&c, ENGINE, &exalted(1, r#", "atMost": 4"#)),
            (true, vec![SUN, MARS, JUPITER, SATURN])
        );
        assert!(!holds(&c, ENGINE, &exalted(1, r#", "atMost": 3"#)).0);
        assert!(holds(&c, ENGINE, &exalted(4, "")).0);
        let five = exalted(5, "");
        assert_eq!(holds(&c, ENGINE, &five), (false, vec![]));
        // It binds SELF as for-any does, so a rule naming SELF inside is read.
        assert_eq!(five.unbound_self(), None);
        assert_eq!(
            serde_json::from_value::<Condition>(serde_json::to_value(&five).unwrap()).unwrap(),
            five
        );
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
        let kala = Condition::AllPlanetsBetweenNodes { side: None };
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
            conditions,
            cancellations: cancellations.into_iter().map(Into::into).collect(),
            ..Rule::new(
                "EXAMPLE",
                "example",
                crate::language::Source::text("an example"),
            )
        }
    }

    /// A panchanga on a tithi, every other limb the first.
    fn panchanga(tithi: teistro_core::catalogue::Tithi) -> crate::chart::Panchanga {
        use teistro_core::catalogue::{Karana, Nakshatra, Vara, Yoga};
        crate::chart::Panchanga {
            tithi,
            vara: Vara::Ravivara,
            nakshatra: Nakshatra::Ashwini,
            pada: crate::language::Pada::try_new(1).unwrap(),
            yoga: Yoga::Vishkambha,
            karana: Karana::Bava,
            spans: crate::chart::Spans::default(),
            by_day: None,
            on_sankranti: false,
            eclipse: None,
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
            example
                .cancellations
                .iter()
                .map(|c| c.condition.clone())
                .collect(),
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
            c.panchanga = Some(panchanga(tithi));
            assert_eq!(held(&c, &burnt(r#""MARS""#)), (true, vec![MARS]));
            // The lagna's lord, Mars, stands in a burnt sign too; the sign of the
            // twelfth, Pisces, is not burnt.
            assert!(held(&c, &burnt(r#"{"lordOf": 1}"#)).0);
            assert!(!held(&c, &burnt("12")).0);
        }
        c.panchanga = Some(panchanga(Tithi::Purnima));
        assert!(!held(&c, &burnt(r#""MARS""#)).0);

        let rule = rule(vec![burnt(r#""MARS""#)], Vec::new());
        assert!(rule.reads_panchanga());
        c.panchanga = None;
        assert_eq!(
            evaluator(&c).explain(&rule).conditions[0].resolved[0].to_string(),
            "DAGDHA_RASHI needs a tithi, and the chart has none"
        );
        c.panchanga = Some(panchanga(Tithi::KrishnaShashthi));
        assert_eq!(
            evaluator(&c).explain(&rule).conditions[0].resolved[0].to_string(),
            "DAGDHA_RASHI gives KRISHNA_SHASHTHI ARIES, LEO"
        );
        assert!(
            !self::rule(
                vec![written(r#"{"type": "planet-in-kendra", "planet": "SUN"}"#)],
                Vec::new()
            )
            .reads_panchanga()
        );
    }

    #[test]
    fn the_dosha_predicates_read_lords_the_lagna_and_gandanta_and_add_no_participant() {
        let mut c = chart();
        // Aries lagna: the fourth's lord is the Moon; the seventh's Venus.
        c.placements[MOON.index()].dignity = Dignity::DeepDebilitated;
        c.placements[Body::Graha(Graha::Venus).index()].combust = true;
        let checks = [
            (
                r#"{"type": "lord-of-house-debilitated", "houseRuled": 4}"#,
                true,
            ),
            (
                r#"{"type": "lord-of-house-strong", "houseRuled": 4}"#,
                false,
            ),
            (
                r#"{"type": "lord-of-house-combust", "houseRuled": 7}"#,
                true,
            ),
            (
                r#"{"type": "lord-of-house-is", "houseRuled": 4, "planets": ["MOON", "MARS"]}"#,
                true,
            ),
            (
                r#"{"type": "lord-of-house-conjunct-planet", "houseRuled": 7, "withPlanet": "SATURN"}"#,
                true,
            ),
            (
                r#"{"type": "lagna-in-sign", "signs": ["ARIES", "SCORPIO"]}"#,
                true,
            ),
            (
                r#"{"type": "planet-in-house-and-sign", "planet": "SUN", "houses": [1], "signs": ["TAURUS"]}"#,
                false,
            ),
        ];
        for (json, expected) in checks {
            // None of them adds a participant.
            assert_eq!(
                holds(&c, ENGINE, &written(json)),
                (expected, vec![]),
                "{json}"
            );
        }
        // The Sun is never combust by himself: Leo lagna, the first's lord.
        place(&mut c, Body::Lagna, Rashi::Leo);
        c.placements[SUN.index()].combust = true;
        assert!(
            !holds(
                &c,
                ENGINE,
                &written(r#"{"type": "lord-of-house-combust", "houseRuled": 1}"#)
            )
            .0
        );

        // Gandanta: 3°20′ into Aries or before the end of Pisces, and not Taurus.
        let gandanta = written(r#"{"type": "planet-at-gandanta", "planet": "MARS"}"#);
        for (sign, degrees, held) in [
            (Rashi::Aries, 3.3, true),
            (Rashi::Aries, 3.4, false),
            (Rashi::Pisces, 26.7, true),
            (Rashi::Pisces, 26.6, false),
            (Rashi::Taurus, 1.0, false),
            (Rashi::Scorpio, 29.9, true),
        ] {
            place(&mut c, MARS, sign);
            c.placements[MARS.index()].longitude = f64::from(sign as u8) * 30.0 + degrees;
            assert_eq!(
                holds(&c, ENGINE, &gandanta).0,
                held,
                "Mars {degrees}° into {sign:?}"
            );
        }
        let wide = written(r#"{"type": "planet-at-gandanta", "planet": "MARS", "orbDegrees": 1}"#);
        assert!(holds(&c, ENGINE, &wide).0);
    }

    #[test]
    fn the_panchanga_predicates_read_the_birth_s_limbs_and_refuse_a_wrong_one() {
        use teistro_core::catalogue::{Nakshatra, Tithi, Vara};

        let mut c = chart();
        // No panchanga, no panchanga predicate holds.
        let tithi = written(r#"{"type": "panchanga-tithi", "tithis": ["AMAVASYA"]}"#);
        assert!(!holds(&c, ENGINE, &tithi).0);
        c.panchanga = Some(panchanga(Tithi::Amavasya));
        let p = c.panchanga.as_mut().unwrap();
        p.vara = Vara::Shanivara;
        p.nakshatra = Nakshatra::Mula;
        p.pada = crate::language::Pada::try_new(4).unwrap();
        p.eclipse = Some(crate::chart::Eclipse::Lunar);
        for (json, expected) in [
            (
                r#"{"type": "panchanga-tithi", "tithis": ["AMAVASYA"]}"#,
                true,
            ),
            (r#"{"type": "panchanga-paksha", "paksha": "krishna"}"#, true),
            (r#"{"type": "panchanga-paksha", "paksha": "shukla"}"#, false),
            (
                r#"{"type": "panchanga-vara", "varas": ["SHANIVARA"]}"#,
                true,
            ),
            (
                r#"{"type": "panchanga-nakshatra", "nakshatras": ["MULA"]}"#,
                true,
            ),
            (
                r#"{"type": "panchanga-nakshatra", "nakshatras": ["MULA"], "padas": [1]}"#,
                false,
            ),
            (
                r#"{"type": "panchanga-nakshatra", "nakshatras": ["MULA"], "padas": [1, 4]}"#,
                true,
            ),
            (
                r#"{"type": "panchanga-yoga", "yogas": ["VAIDHRITI"]}"#,
                false,
            ),
            (r#"{"type": "panchanga-karana", "karanas": ["BAVA"]}"#, true),
            (r#"{"type": "birth-during-eclipse"}"#, true),
            (
                r#"{"type": "birth-during-eclipse", "kind": "solar"}"#,
                false,
            ),
            (r#"{"type": "birth-during-eclipse", "kind": "lunar"}"#, true),
            (r#"{"type": "birth-on-sankranti", "windowHours": 6}"#, false),
        ] {
            assert_eq!(holds(&c, ENGINE, &written(json)).0, expected, "{json}");
        }
        for (json, reason) in [
            (
                r#"{"type": "panchanga-paksha", "paksha": "waxing"}"#,
                "`waxing` is not a paksha",
            ),
            (
                r#"{"type": "panchanga-nakshatra", "nakshatras": ["MULA"], "padas": [5]}"#,
                "pada 5",
            ),
            (
                r#"{"type": "birth-during-eclipse", "kind": "partial"}"#,
                "partial",
            ),
        ] {
            let error = serde_json::from_str::<Condition>(json)
                .unwrap_err()
                .to_string();
            assert!(error.contains(reason), "{json}: {error}");
        }
    }

    /// Mars in the lagna's seventh, the Moon's fourth, and with Venus; a\n    /// Mangal-like rule found from all three references.
    fn mangal() -> (RuleChart, Rule) {
        use crate::rule::{Group, Severity};

        let mut c = chart();
        place(&mut c, MARS, Rashi::Libra); // The seventh from the Aries lagna.
        place(&mut c, MOON, Rashi::Cancer); // Mars the fourth from the Moon.
        place(&mut c, Body::Graha(Graha::Venus), Rashi::Libra); // With Venus.
        place(&mut c, JUPITER, Rashi::Taurus); // Aspecting neither.
        let group = |reference: Body, label: &str, house: u8| Group {
            reference,
            label: String::from(label),
            weight: 1,
            conditions: vec![written(&format!(
                r#"{{"type": "planet-in-house-from", "planet": "MARS", "reference": "{}", "houses": [{house}]}}"#,
                reference.key()
            ))],
        };
        let mangal = Rule {
            groups: vec![
                group(Body::Lagna, "Lagna", 7),
                group(MOON, "Moon", 4),
                group(Body::Graha(Graha::Venus), "Venus", 8),
            ],
            cancellations: vec![
                written(r#"{"type": "planet-conjunct", "planets": ["VENUS", "MARS"]}"#).into(),
                written(
                    r#"{"type": "planet-aspects-planet", "from": "JUPITER", "target": "MARS"}"#,
                )
                .into(),
            ],
            severity: Some(Severity::HouseWeighted {
                planet: MARS,
                weights: [(house(7), 100), (house(4), 50)].into_iter().collect(),
                default: 40,
            }),
            ..rule(Vec::new(), Vec::new())
        };
        (c, mangal)
    }

    #[test]
    fn a_rule_is_found_from_its_conditions_and_every_group_that_holds() {
        use crate::rule::{Group, NetStatus};

        let (c, mangal) = mangal();
        let evaluator = Evaluator::new(&c, Readings::RECORDING_ENGINE_DOSHAS);
        let result = evaluator.evaluate(&mangal);
        assert!(result.present);
        assert_eq!(result.found_from, [Found::Group(0), Found::Group(1)]);
        assert_eq!(result.participants.iter().collect::<Vec<_>>(), [MARS]);
        assert_eq!(
            result.severity,
            Some(100),
            "the gravest house of those found from"
        );
        // Venus with Mars cancels once; the threshold, unset, is the two places
        // found from.
        assert_eq!(result.cancellations, [0]);
        assert_eq!(result.status, Some(NetStatus::PartiallyCancelled));
        let explanation = evaluator.explain(&mangal);
        assert_eq!(explanation.result, result);
        assert_eq!(
            explanation.to_string(),
            "EXAMPLE: present, MARS in house 7, severity 100, partly cancelled\n\
             \u{20} from Lagna (LAGNA):\n\
             \u{20}   holds planet-in-house-from, adding MARS\n\
             \u{20}     LAGNA stands in ARIES, house 1\n\
             \u{20}     MARS stands in LIBRA, house 7\n\
             \u{20} from Moon (MOON):\n\
             \u{20}   holds planet-in-house-from, adding MARS\n\
             \u{20}     MOON stands in CANCER, house 4\n\
             \u{20}     MARS stands in LIBRA, house 7\n\
             \u{20} from Venus (VENUS):\n\
             \u{20}   fails planet-in-house-from\n\
             \u{20}     VENUS stands in LIBRA, house 7\n\
             \u{20}     MARS stands in LIBRA, house 7\n\
             \u{20} cancellations:\n\
             \u{20}   holds planet-conjunct, adding VENUS, MARS\n\
             \u{20}   fails planet-aspects-planet\n\
             \u{20}     MARS stands in LIBRA, house 7\n"
        );
        assert_eq!(
            explanation
                .groups
                .iter()
                .map(|g| g[0].held)
                .collect::<Vec<_>>(),
            [true, true, false]
        );

        // With conditions too, both must hold: the conditions, and a group.
        let both = Rule {
            conditions: vec![written(r#"{"type": "planet-in-kendra", "planet": "MARS"}"#)],
            groups: vec![Group {
                label: String::from("Venus"),
                ..mangal.groups[2].clone()
            }],
            ..rule(Vec::new(), Vec::new())
        };
        assert!(!evaluator.evaluate(&both).present);
    }

    #[test]
    fn each_severity_rule_and_the_dosha_evaluator_s_aspects() {
        use crate::rule::Severity;

        let (mut c, mangal) = mangal();
        let evaluator = Evaluator::new(&c, Readings::RECORDING_ENGINE_DOSHAS);
        let severity = |severity: Severity| {
            evaluator
                .evaluate(&Rule {
                    severity: Some(severity),
                    ..mangal.clone()
                })
                .severity
        };
        assert_eq!(
            severity(Severity::CountBased {
                per_occurrence: 30,
                cap: 50
            }),
            Some(50)
        );
        assert_eq!(
            severity(Severity::CountBased {
                per_occurrence: 20,
                cap: 50
            }),
            Some(40)
        );
        assert_eq!(
            severity(Severity::KootShortfall {
                full: 80,
                per_point_missing: 5
            }),
            Some(80)
        );
        let inverse = |c: &RuleChart| {
            Evaluator::new(c, Readings::RECORDING_ENGINE_DOSHAS)
                .evaluate(&Rule {
                    severity: Some(Severity::PlanetStrengthInverse {
                        base_planet: MOON,
                        max: 90,
                        min: 21,
                    }),
                    ..mangal.clone()
                })
                .severity
        };
        assert_eq!(inverse(&c), Some(56), "half way, a half rounded up");
        c.placements[MOON.index()].dignity = Dignity::OwnSign;
        assert_eq!(inverse(&c), Some(21));
        c.placements[MOON.index()].dignity = Dignity::DeepDebilitated;
        assert_eq!(inverse(&c), Some(90));

        // The dosha evaluator's aspects add no participant; the yoga evaluator's
        // add both.
        place(&mut c, JUPITER, Rashi::Aries); // Aspects Libra, its seventh.
        let aspect =
            written(r#"{"type": "planet-aspects-planet", "from": "JUPITER", "target": "MARS"}"#);
        assert_eq!(holds(&c, ENGINE, &aspect), (true, vec![JUPITER, MARS]));
        assert_eq!(
            holds(&c, Readings::RECORDING_ENGINE_DOSHAS, &aspect),
            (true, vec![])
        );
    }

    #[test]
    fn a_condition_read_in_a_division_moves_the_signs_houses_and_dignities() {
        use teistro_core::catalogue::Varga;

        use crate::chart::VargaSigns;

        // Every body in Aries in the rasi; in the navamsha the lagna rises in
        // Cancer, Mars stands in Capricorn, its exaltation, and the Sun in Leo,
        // its own sign and the second from that lagna.
        let c = chart();
        let mut signs = [Rashi::Aries; 10];
        signs[Body::Lagna.index()] = Rashi::Cancer;
        signs[MARS.index()] = Rashi::Capricorn;
        signs[SUN.index()] = Rashi::Leo;
        let navamsha = [VargaSigns {
            varga: Varga::D9,
            signs,
        }];
        let held = |condition: &Condition| {
            let mut into = Participants::default();
            let held = Evaluator::new(&c, ENGINE)
                .with_vargas(&navamsha)
                .holds(condition, &mut into);
            (held, into.iter().collect::<Vec<_>>())
        };
        let inside = |json: &str| {
            written(&format!(
                r#"{{"type": "in-varga", "varga": "D9", "condition": {json}}}"#
            ))
        };
        assert_eq!(
            held(&inside(
                r#"{"type": "planet-dignity", "planet": "MARS", "dignities": ["EXALTED"]}"#
            )),
            (true, vec![MARS])
        );
        assert!(
            held(&inside(
                r#"{"type": "planet-in-sign", "planet": "SUN", "signs": ["LEO"]}"#
            ))
            .0
        );
        // The second house of the navamsha, counted from its own lagna.
        assert!(
            held(&inside(
                r#"{"type": "planet-in-house", "planet": "SUN", "houses": [2]}"#
            ))
            .0
        );
        // The rasi chart still reads as it did.
        assert!(
            holds(
                &c,
                ENGINE,
                &written(r#"{"type": "planet-in-sign", "planet": "SUN", "signs": ["ARIES"]}"#)
            )
            .0
        );
        // A division the evaluator was not given never holds.
        let without = {
            let mut into = Participants::default();
            Evaluator::new(&c, ENGINE).holds(
                &inside(r#"{"type": "planet-in-sign", "planet": "SUN", "signs": ["LEO"]}"#),
                &mut into,
            )
        };
        assert!(!without, "no division, no answer");
        let other = inside(r#"{"type": "planet-in-sign", "planet": "SUN", "signs": ["LEO"]}"#);
        let Condition::InVarga { varga, .. } = &other else {
            panic!("in-varga")
        };
        assert_eq!(*varga, Varga::D9);

        // A rule that reads a longitude inside a division is refused.
        let refused = serde_json::from_str::<Rule>(
            r#"{"key": "R", "category": "c", "source": {"text": "t"}, "conditions": [
                {"type": "in-varga", "varga": "D9", "condition":
                    {"type": "planet-at-table-degree", "planet": "MOON", "table": "MRITYU_BHAGA"}}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(
            refused.contains("`planet-at-table-degree` reads a longitude"),
            "{refused}"
        );
    }

    #[test]
    fn a_class_can_aspect_and_be_counted_in_houses_from_a_reference() {
        // Aries lagna. Saturn in Capricorn aspects Pisces (its third) and Cancer
        // (its tenth); Jupiter in Taurus aspects Scorpio, Virgo and Capricorn.
        let mut c = chart();
        place(&mut c, SATURN, Rashi::Capricorn);
        place(&mut c, JUPITER, Rashi::Taurus);
        place(&mut c, MOON, Rashi::Pisces);
        for body in [SUN, MARS, MERCURY, RAHU, Body::Graha(Graha::Ketu)] {
            place(&mut c, body, Rashi::Leo);
        }
        let aspected = |by: &str| {
            written(&format!(
                r#"{{"type": "planet-aspects-planet", "from": "{by}", "target": "MOON"}}"#
            ))
        };
        // A malefic aspects the Moon: Mars in Leo by its eighth, the first
        // malefic in the chart's order that does.
        assert_eq!(
            holds(&c, ENGINE, &aspected("any-malefic")),
            (true, vec![MARS, MOON])
        );
        // No benefic does: Jupiter aspects Scorpio, Virgo and Capricorn.
        assert!(!holds(&c, ENGINE, &aspected("any-benefic")).0);
        let house =
            written(r#"{"type": "planet-aspects-house", "from": "any-benefic", "houseRuled": 8}"#);
        assert_eq!(holds(&c, ENGINE, &house), (true, vec![JUPITER]));

        // Five bodies stand in Leo, the fifth house: four malefics and Mercury,
        // which is malefic here by its company.
        let counted = |json: &str| written(json);
        let five = counted(
            r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [5], "atLeast": 5}"#,
        );
        assert!(holds(&c, ENGINE, &five).0);
        let six = counted(
            r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [5], "atLeast": 6}"#,
        );
        assert!(!holds(&c, ENGINE, &six).0);
        // Counted from the Moon in Pisces, Leo is the sixth.
        let from_moon = counted(
            r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [6], "from": "MOON", "atLeast": 5}"#,
        );
        assert!(holds(&c, ENGINE, &from_moon).0);
        assert!(!holds(&c, ENGINE, &counted(r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [6], "atLeast": 1}"#)).0);
        // A named body counts as itself.
        let one = counted(
            r#"{"type": "count-in-houses", "planets": "SATURN", "houses": [10], "atLeast": 1}"#,
        );
        assert_eq!(holds(&c, ENGINE, &one), (true, vec![SATURN]));
        // The lagna is what a rule counts from unless it says, and writing it
        // back leaves that out.
        let back: Condition = serde_json::from_value(serde_json::to_value(&five).unwrap()).unwrap();
        assert_eq!(back, five);
        assert_eq!(
            serde_json::to_value(&five).unwrap()["from"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn papa_kartari_is_the_twelfth_and_second_from_the_lagna_phaladeepika_6_8() {
        // "When the 12th and the 2nd Bhavas from the Lagna are occupied by
        // benefics, the Yoga is Subhakartari. It is called Papakartari, when
        // the above two houses are occupied by malefics" (Phaladeepika ch. 6
        // sl. 8, V. Subrahmanya Sastri's translation).
        let kartari = |by: &str| {
            written(&format!(
                r#"{{"type": "and", "conditions": [
                    {{"type": "planet-in-house-from", "planet": "{by}", "reference": "LAGNA", "houses": [12]}},
                    {{"type": "planet-in-house-from", "planet": "{by}", "reference": "LAGNA", "houses": [2]}}
                ]}}"#
            ))
        };
        let mut c = chart();
        // Aries lagna, malefics in Pisces and Taurus: the twelfth and the
        // second.
        place(&mut c, SATURN, Rashi::Pisces);
        place(&mut c, MARS, Rashi::Taurus);
        for body in [SUN, MOON, MERCURY, JUPITER, Body::Graha(Graha::Venus), RAHU] {
            place(&mut c, body, Rashi::Leo);
        }
        assert_eq!(
            holds(&c, ENGINE, &kartari("any-malefic")),
            (true, vec![SATURN, MARS])
        );
        assert!(!holds(&c, ENGINE, &kartari("any-benefic")).0);
        // Benefics there instead: Jupiter and Venus hem the lagna.
        place(&mut c, SATURN, Rashi::Leo);
        place(&mut c, MARS, Rashi::Leo);
        place(&mut c, JUPITER, Rashi::Pisces);
        place(&mut c, Body::Graha(Graha::Venus), Rashi::Taurus);
        assert_eq!(
            holds(&c, ENGINE, &kartari("any-benefic")),
            (true, vec![JUPITER, Body::Graha(Graha::Venus)])
        );
        assert!(!holds(&c, ENGINE, &kartari("any-malefic")).0);
    }

    #[test]
    fn a_point_the_chart_carries_is_a_place_a_rule_can_name() {
        use teistro_core::catalogue::Point;

        use crate::chart::PointAt;

        // BPHS ch. 83 reads the curses with Gulika standing somewhere: here it
        // rises with Rahu in the lagna's own sign.
        let mut c = chart();
        place(&mut c, RAHU, Rashi::Aries);
        let points = [
            PointAt {
                point: Point::Gulika,
                longitude: 12.0,
                sign: Rashi::Aries,
            },
            PointAt {
                point: Point::HoraLagna,
                longitude: 100.0,
                sign: Rashi::Cancer,
            },
        ];
        let held = |condition: &Condition, points: &[PointAt]| {
            let mut into = Participants::default();
            let held = Evaluator::new(&c, ENGINE)
                .with_points(points)
                .holds(condition, &mut into);
            (held, into.iter().collect::<Vec<_>>())
        };
        let with_rahu = written(
            r#"{"type": "and", "conditions": [
                {"type": "planet-in-house", "planet": "RAHU", "houses": [1]},
                {"type": "same-sign", "of": {"point": "GULIKA"}, "as": 1}
            ]}"#,
        );
        assert_eq!(held(&with_rahu, &points), (true, vec![RAHU]));
        // The hora lagna stands in the fourth, and a point the chart does not
        // carry resolves to nothing.
        assert!(held(&written(r#"{"type": "planet-in-house", "planet": {"point": "HORA_LAGNA"}, "houses": [4]}"#), &points).0);
        assert!(!held(&written(r#"{"type": "planet-in-house", "planet": {"point": "SREE_LAGNA"}, "houses": [4]}"#), &points).0);
        assert!(!held(&with_rahu, &[]).0, "no points, no answer");

        // It reads and writes back as the catalogue spells it, and a point the
        // catalogue does not name is refused.
        let reference: SignRef = serde_json::from_str(r#"{"point": "MANDI"}"#).unwrap();
        assert_eq!(reference, SignRef::Point(Point::Mandi));
        assert_eq!(
            serde_json::to_string(&reference).unwrap(),
            r#"{"point":"MANDI"}"#
        );
        assert_eq!(reference.to_string(), "MANDI");
        let refused = serde_json::from_str::<SignRef>(r#"{"point": "NOT_A_POINT"}"#)
            .unwrap_err()
            .to_string();
        assert!(
            refused.contains("`NOT_A_POINT` is not a point"),
            "{refused}"
        );
    }

    #[test]
    fn the_gandantas_of_bphs_92_hold_within_their_ghatikas_and_not_outside() {
        use teistro_core::catalogue::{Nakshatra, Tithi};

        use crate::chart::{Span, Spans};
        use crate::shipped;

        let at = |c: &RuleChart, key: &str| {
            shipped::gandantas()
                .iter()
                .find(|rule| rule.key == key)
                .is_some_and(|rule| Evaluator::new(c, ENGINE).evaluate(rule).present)
        };
        let mut c = chart();
        let set = |c: &mut RuleChart, tithi, nakshatra, spans| {
            let mut p = panchanga(tithi);
            p.nakshatra = nakshatra;
            p.spans = spans;
            c.panchanga = Some(p);
        };

        // The last two ghatikas of a Purna tithi (BPHS ch. 92 v. 2).
        let ending = |remaining| Spans {
            tithi: Some(Span {
                elapsed: 58.0,
                remaining,
            }),
            ..Spans::default()
        };
        set(&mut c, Tithi::Purnima, Nakshatra::Ashwini, ending(1.5));
        assert!(at(&c, "TITHI_GANDANTA"));
        set(&mut c, Tithi::Purnima, Nakshatra::Ashwini, ending(2.5));
        assert!(!at(&c, "TITHI_GANDANTA"), "past the last two ghatikas");
        // A Nanda tithi counts from its first two instead.
        let starting = |elapsed| Spans {
            tithi: Some(Span {
                elapsed,
                remaining: 40.0,
            }),
            ..Spans::default()
        };
        set(
            &mut c,
            Tithi::ShuklaShashthi,
            Nakshatra::Ashwini,
            starting(1.0),
        );
        assert!(at(&c, "TITHI_GANDANTA"));
        set(
            &mut c,
            Tithi::ShuklaDwadashi,
            Nakshatra::Ashwini,
            starting(1.0),
        );
        assert!(!at(&c, "TITHI_GANDANTA"), "neither Purna nor Nanda");

        // The nakshatra junctions, and Abhukta Moola's wider window (vv. 3, 5).
        let nakshatra = |elapsed, remaining| Spans {
            nakshatra: Some(Span { elapsed, remaining }),
            ..Spans::default()
        };
        set(
            &mut c,
            Tithi::ShuklaDwadashi,
            Nakshatra::Jyeshtha,
            nakshatra(50.0, 1.0),
        );
        assert!(at(&c, "NAKSHATRA_GANDANTA") && at(&c, "ABHUKTA_MOOLA"));
        set(
            &mut c,
            Tithi::ShuklaDwadashi,
            Nakshatra::Jyeshtha,
            nakshatra(50.0, 5.0),
        );
        assert!(
            !at(&c, "NAKSHATRA_GANDANTA") && at(&c, "ABHUKTA_MOOLA"),
            "the last six ghatikas of Jyeshtha are Abhukta Moola, the last two gandanta"
        );
        set(
            &mut c,
            Tithi::ShuklaDwadashi,
            Nakshatra::Mula,
            nakshatra(7.0, 40.0),
        );
        assert!(at(&c, "ABHUKTA_MOOLA") && !at(&c, "NAKSHATRA_GANDANTA"));
        set(
            &mut c,
            Tithi::ShuklaDwadashi,
            Nakshatra::Mula,
            nakshatra(1.0, 50.0),
        );
        assert!(at(&c, "NAKSHATRA_GANDANTA"), "Moola's first two ghatikas");
        set(
            &mut c,
            Tithi::ShuklaDwadashi,
            Nakshatra::Rohini,
            nakshatra(1.0, 1.0),
        );
        assert!(!at(&c, "NAKSHATRA_GANDANTA"), "not a junction nakshatra");
    }

    #[test]
    fn the_lagna_gandanta_is_half_a_ghatika_either_side_of_the_junction() {
        use teistro_core::catalogue::{Rashi as R, Tithi};

        use crate::chart::{Span, Spans};
        use crate::shipped;

        fn with_lagna(c: &mut RuleChart, elapsed: f64, remaining: f64) {
            if let Some(p) = c.panchanga.as_mut() {
                p.spans = Spans {
                    lagna: Some(Span { elapsed, remaining }),
                    ..Spans::default()
                };
            }
        }

        let at = |c: &RuleChart, key: &str| {
            shipped::gandantas()
                .iter()
                .find(|rule| rule.key == key)
                .is_some_and(|rule| Evaluator::new(c, ENGINE).evaluate(rule).present)
        };
        let mut c = chart();
        c.panchanga = Some(panchanga(Tithi::ShuklaDwadashi));
        // Half a ghatika either side of the rising sign's junction (v. 4).
        place(&mut c, Body::Lagna, R::Pisces);
        with_lagna(&mut c, 4.0, 0.25);
        assert!(at(&c, "LAGNA_GANDANTA"));
        with_lagna(&mut c, 4.0, 0.75);
        assert!(!at(&c, "LAGNA_GANDANTA"));
        place(&mut c, Body::Lagna, R::Aries);
        with_lagna(&mut c, 0.25, 4.0);
        assert!(at(&c, "LAGNA_GANDANTA"));
        place(&mut c, Body::Lagna, R::Taurus);
        assert!(!at(&c, "LAGNA_GANDANTA"), "not a junction sign");

        // A chart that measures no ghatikas answers no gandanta.
        c.panchanga = Some(panchanga(Tithi::Purnima));
        assert!(!at(&c, "TITHI_GANDANTA") && !at(&c, "NAKSHATRA_GANDANTA"));
        c.panchanga = None;
        assert!(!at(&c, "LAGNA_GANDANTA"));
    }

    #[test]
    fn a_rule_can_name_another_as_its_cancellation() {
        use crate::rule::NetStatus;
        use crate::shipped;

        // BPHS ch. 9's evils name ch. 10's antidotes by key, so an evil that
        // holds on a chart where an antidote holds is cancelled.
        let pack: Vec<Rule> = shipped::arishtas().to_vec();
        let Some(evil) = pack
            .iter()
            .find(|rule| rule.key == "ARISHTA_MALEFICS_IN_SIXTH_AND_TWELFTH")
        else {
            panic!("shipped")
        };
        assert_eq!(evil.references().count(), 4);

        let mut c = chart();
        // Malefics in the sixth and the twelfth: Saturn in Virgo, Mars in
        // Pisces, from an Aries lagna.
        place(&mut c, SATURN, Rashi::Virgo);
        place(&mut c, MARS, Rashi::Pisces);
        // No benefic in a kendra or trikona yet: Jupiter, Venus and Mercury in
        // the eleventh, with the Moon there too.
        for body in [JUPITER, Body::Graha(Graha::Venus), MERCURY, MOON] {
            place(&mut c, body, Rashi::Aquarius);
        }
        let answer = |c: &RuleChart| Evaluator::new(c, ENGINE).with_rules(&pack).evaluate(evil);
        let result = answer(&c);
        assert!(result.present);
        assert_eq!(result.status, Some(NetStatus::Active), "no antidote holds");

        // Jupiter into the fourth: ch. 10 v. 2's antidote holds and cancels it.
        place(&mut c, JUPITER, Rashi::Cancer);
        let result = answer(&c);
        assert!(result.present);
        // Two antidotes hold at once: the benefic in a kendra (v. 2), and
        // Jupiter in Cancer aspecting Mars in Pisces by his ninth (v. 7).
        assert_eq!(result.cancellations, [0, 3]);
        assert_eq!(result.status, Some(NetStatus::FullyCancelled));

        // An evaluator given no rules cannot read the reference, so nothing
        // cancels and the evil stands.
        let alone = Evaluator::new(&c, ENGINE).evaluate(evil);
        assert!(alone.present && alone.cancellations.is_empty());
        assert_eq!(alone.status, Some(NetStatus::Active));
    }

    #[test]
    fn a_degree_band_reads_a_body_s_place_within_its_sign() {
        // Brihat Jataka ch. 6 v. 8 reads the Moon in the last navamsa of a
        // sign, her last 3°20′.
        let mut c = chart();
        let last_navamsha = written(
            r#"{"type": "planet-in-degrees", "planet": "MOON", "from": 26.666666666666668, "to": 30.0}"#,
        );
        for (degrees, held) in [(26.0, false), (26.7, true), (29.99, true), (0.5, false)] {
            c.placements[MOON.index()].longitude = degrees;
            assert_eq!(holds(&c, ENGINE, &last_navamsha).0, held, "{degrees}°");
        }
        c.placements[MOON.index()].longitude = 27.0;
        assert_eq!(holds(&c, ENGINE, &last_navamsha), (true, vec![MOON]));
        // It reads the degrees of whichever sign the body stands in.
        place(&mut c, MOON, Rashi::Leo);
        c.placements[MOON.index()].longitude = 120.0 + 27.0;
        assert!(holds(&c, ENGINE, &last_navamsha).0);
        // A division moves signs and not degrees, so a rule may not read one
        // inside an in-varga.
        let refused = serde_json::from_str::<Rule>(
            r#"{"key": "R", "category": "c", "source": {"text": "t"}, "conditions": [
                {"type": "in-varga", "varga": "D9", "condition":
                    {"type": "planet-in-degrees", "planet": "MOON", "from": 0.0, "to": 1.0}}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(
            refused.contains("`planet-in-degrees` reads a longitude"),
            "{refused}"
        );
    }

    #[test]
    fn a_rule_without_conditions_is_not_evaluable_and_not_present() {
        let rule = Rule::new(
            "OUTSIDE",
            "example",
            crate::language::Source::text("an example"),
        );
        assert!(!rule.is_evaluable());
        assert!(!Evaluator::new(&chart(), ENGINE).evaluate(&rule).present);
    }

    /// The table BPHS ch. 26 prints under vv. 1 to 3, sign by sign.
    const PRINTED: [(Rashi, [Rashi; 3]); 12] = [
        (Rashi::Aries, [Rashi::Leo, Rashi::Scorpio, Rashi::Aquarius]),
        (
            Rashi::Taurus,
            [Rashi::Cancer, Rashi::Libra, Rashi::Capricorn],
        ),
        (
            Rashi::Gemini,
            [Rashi::Virgo, Rashi::Sagittarius, Rashi::Pisces],
        ),
        (
            Rashi::Cancer,
            [Rashi::Scorpio, Rashi::Aquarius, Rashi::Taurus],
        ),
        (Rashi::Leo, [Rashi::Libra, Rashi::Capricorn, Rashi::Aries]),
        (
            Rashi::Virgo,
            [Rashi::Gemini, Rashi::Sagittarius, Rashi::Pisces],
        ),
        (Rashi::Libra, [Rashi::Aquarius, Rashi::Taurus, Rashi::Leo]),
        (
            Rashi::Scorpio,
            [Rashi::Capricorn, Rashi::Aries, Rashi::Cancer],
        ),
        (
            Rashi::Sagittarius,
            [Rashi::Gemini, Rashi::Virgo, Rashi::Pisces],
        ),
        (
            Rashi::Capricorn,
            [Rashi::Taurus, Rashi::Leo, Rashi::Scorpio],
        ),
        (Rashi::Aquarius, [Rashi::Aries, Rashi::Cancer, Rashi::Libra]),
        (
            Rashi::Pisces,
            [Rashi::Gemini, Rashi::Virgo, Rashi::Sagittarius],
        ),
    ];

    #[test]
    fn rashi_drishti_is_the_table_the_chapter_prints() {
        for (from, aspected) in PRINTED {
            for target in Rashi::ALL {
                assert_eq!(
                    rashi_aspects(from, target),
                    aspected.contains(&target),
                    "{from:?} to {target:?}"
                );
            }
            // Three signs each, itself never among them, and the relation runs
            // both ways.
            assert!(!rashi_aspects(from, from));
            for target in aspected {
                assert!(rashi_aspects(target, from), "{target:?} back to {from:?}");
            }
        }
    }

    #[test]
    fn a_body_lends_the_aspect_of_the_sign_it_stands_in() {
        let mut c = chart();
        place(&mut c, Body::Graha(Graha::Mars), Rashi::Aries);
        place(&mut c, Body::Graha(Graha::Venus), Rashi::Leo);
        place(&mut c, Body::Graha(Graha::Saturn), Rashi::Taurus);
        let aspects = |from: Subject, target: SignRef| Condition::RashiAspects { from, target };
        let mars = SignRef::from(Body::Graha(Graha::Mars));
        let venus = SignRef::from(Body::Graha(Graha::Venus));
        let saturn = SignRef::from(Body::Graha(Graha::Saturn));
        // Aries aspects Leo, and Leo Aries; neither aspects Taurus.
        assert!(
            holds(
                &c,
                ENGINE,
                &aspects(Subject::Ref(mars.clone()), venus.clone())
            )
            .0
        );
        assert!(
            holds(
                &c,
                ENGINE,
                &aspects(Subject::Ref(venus.clone()), mars.clone())
            )
            .0
        );
        assert!(
            !holds(
                &c,
                ENGINE,
                &aspects(Subject::Ref(mars.clone()), saturn.clone())
            )
            .0
        );
        // The participants are the two bodies the aspect ran through.
        let (held, bodies) = holds(&c, ENGINE, &aspects(Subject::Ref(mars), venus));
        assert!(held);
        assert_eq!(
            bodies,
            [Body::Graha(Graha::Mars), Body::Graha(Graha::Venus)]
        );
    }

    #[test]
    fn an_intervention_stands_when_it_outnumbers_what_obstructs_it() {
        // An Aries lagna: the second is Taurus, the twelfth Pisces.
        let mut c = chart();
        for body in Body::ALL {
            place(&mut c, body, Rashi::Aries);
        }
        place(&mut c, Body::Graha(Graha::Venus), Rashi::Taurus);
        let lagna = SignRef::House(house(1));
        let argala = |place| Condition::Argala {
            on: lagna.clone(),
            place,
        };
        // One graha in the second and none in the twelfth: it stands.
        let (held, bodies) = holds(&c, ENGINE, &argala(ArgalaPlace::Second));
        assert!(held);
        assert_eq!(bodies, [Body::Graha(Graha::Venus)]);
        // Two in the twelfth against one in the second: it does not.
        place(&mut c, Body::Graha(Graha::Saturn), Rashi::Pisces);
        place(&mut c, Body::Graha(Graha::Mars), Rashi::Pisces);
        assert!(!holds(&c, ENGINE, &argala(ArgalaPlace::Second)).0);
        // Nothing in the fourth is no intervention, however empty the tenth.
        assert!(!holds(&c, ENGINE, &argala(ArgalaPlace::Fourth)).0);
    }

    #[test]
    fn a_node_counts_its_intervention_backwards() {
        // Rahu in Aries: the second from him is Pisces, his motion running the
        // other way, and the twelfth that obstructs it Taurus.
        let mut c = chart();
        for body in Body::ALL {
            place(&mut c, body, Rashi::Leo);
        }
        place(&mut c, Body::Graha(Graha::Rahu), Rashi::Aries);
        place(&mut c, Body::Graha(Graha::Venus), Rashi::Pisces);
        let rahu = SignRef::from(Body::Graha(Graha::Rahu));
        let on_rahu = Condition::Argala {
            on: rahu,
            place: ArgalaPlace::Second,
        };
        assert!(holds(&c, ENGINE, &on_rahu).0);
        // The same graha in the sign that is second going forward does not.
        place(&mut c, Body::Graha(Graha::Venus), Rashi::Taurus);
        assert!(!holds(&c, ENGINE, &on_rahu).0);
    }

    #[test]
    fn three_malefics_in_the_third_are_a_contrary_intervention() {
        let mut c = chart();
        for body in Body::ALL {
            place(&mut c, body, Rashi::Aries);
        }
        let lagna = SignRef::House(house(1));
        let contrary = Condition::VipareetaArgala { on: lagna };
        assert!(!holds(&c, ENGINE, &contrary).0);
        for body in [Graha::Sun, Graha::Mars, Graha::Saturn] {
            place(&mut c, Body::Graha(body), Rashi::Gemini);
        }
        let (held, bodies) = holds(&c, ENGINE, &contrary);
        assert!(held);
        assert_eq!(bodies.len(), 3);
    }

    /// A chart that carries a strength for the Sun and the Moon and nothing
    /// for the rest.
    fn measured() -> RuleChart {
        let mut of = [None; 10];
        let mut required = [None; 10];
        of[SUN.index()] = Some(8.8);
        required[SUN.index()] = Some(5.0);
        of[MOON.index()] = Some(4.0);
        required[MOON.index()] = Some(6.0);
        RuleChart {
            strengths: Some(Strengths {
                measure: crate::chart::StrengthMeasure::Shadbala,
                of,
                required,
            }),
            ..chart()
        }
    }

    #[test]
    fn strong_and_weak_are_two_questions_and_a_silent_chart_answers_neither() {
        let strong = |body| Condition::PlanetStrong {
            planet: BodyRef::Body(body),
        };
        let weak = |body| Condition::PlanetWeak {
            planet: BodyRef::Body(body),
        };
        let c = measured();
        assert!(holds(&c, ENGINE, &strong(SUN)).0);
        assert!(!holds(&c, ENGINE, &weak(SUN)).0);
        assert!(holds(&c, ENGINE, &weak(MOON)).0);
        assert!(!holds(&c, ENGINE, &strong(MOON)).0);
        // Mars has no number, so neither question is answered of him — and
        // that is not the same as answering "weak".
        assert!(!holds(&c, ENGINE, &strong(MARS)).0);
        assert!(!holds(&c, ENGINE, &weak(MARS)).0);
        // Nor on a chart that says nothing at all.
        let silent = chart();
        for body in [SUN, MOON, MARS] {
            assert!(!holds(&silent, ENGINE, &strong(body)).0);
            assert!(!holds(&silent, ENGINE, &weak(body)).0);
        }
        // The body a comparison found is its participant.
        let (held, bodies) = holds(
            &c,
            ENGINE,
            &Condition::PlanetStrongerThan {
                planet: BodyRef::Body(SUN),
                than: BodyRef::Body(MOON),
            },
        );
        assert!(held);
        assert_eq!(bodies, [SUN]);
        assert!(
            !holds(
                &c,
                ENGINE,
                &Condition::PlanetStrongerThan {
                    planet: BodyRef::Body(MOON),
                    than: BodyRef::Body(SUN),
                },
            )
            .0
        );
        // A comparison against a body with no number is no answer either.
        assert!(
            !holds(
                &c,
                ENGINE,
                &Condition::PlanetStrongerThan {
                    planet: BodyRef::Body(SUN),
                    than: BodyRef::Body(MARS),
                },
            )
            .0
        );
    }
}
