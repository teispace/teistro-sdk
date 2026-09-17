//! The marakas and their periods (BPHS ch. 44 vv. 2 to 24).
//!
//! The second and seventh are the houses of death, the twelfth from the two
//! houses of longevity, and the second the stronger (vv. 2 to 5). A graha is a
//! maraka for a reason the verses give — it lords one of them, a malefic
//! stands in one or joins its lord, it is the eighth lord or the sixth, it
//! lords the second or twelfth from the Moon, it is a node placed where the
//! verses place one — and each reason brings death, or only illness and
//! difficulty. The periods of a maraka are when the texts say death may come;
//! v. 8 adds that a malefic's major period kills in a malefic's sub-period and
//! not a benefic's, and vv. 15 and 16 name the periods of the third, fifth and
//! seventh stars from the birth star for short, medium and long lives.
//!
//! The texts say what may come, and the translator's own notes warn that a
//! maraka period read alone is "a misnomer". So a result here is a
//! [`Vulnerability`], a period to be read with the span of life and never a
//! date, and says so in its type.
//!
//! What the translation leaves loose is read as crux C105 records.

use serde::Serialize;
use teistro_core::catalogue::{Dignity, Graha, Rashi};

use crate::eval::{Evaluator, Participants, star};
use crate::language::{Body, Condition, House};
use crate::longevity::ayurdaya::drekkana_of;
use crate::reference::{BodyRef, BodySubject, Class, SignRef};
use crate::rule::LifeClass;
use crate::timing::{Levels, Running};

/// The nine grahas, in the catalogue's order.
const NINE: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// Why a graha is a maraka.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum Reason {
    /// It lords the second, the stronger house of death (vv. 3 to 5).
    LordOfSecond,
    /// It lords the seventh (vv. 3 to 5).
    LordOfSeventh,
    /// A malefic in the second (vv. 3 to 5).
    MaleficInSecond,
    /// A malefic in the seventh (vv. 3 to 5).
    MaleficInSeventh,
    /// A malefic joining the second lord (vv. 3 to 5).
    MaleficWithSecondLord,
    /// A malefic joining the seventh lord (vv. 3 to 5).
    MaleficWithSeventhLord,
    /// A benefic related to the twelfth lord (v. 6).
    BeneficRelatedToTwelfthLord,
    /// The eighth lord (v. 7).
    LordOfEighth,
    /// Saturn ill-disposed and related to a maraka, the first to kill (v. 9).
    SaturnIllDisposed,
    /// A malefic lording the second or the twelfth from the Moon (v. 18).
    MaleficLordFromMoon,
    /// A benefic lording the second or the twelfth from the Moon, which brings
    /// illness and not death (v. 18).
    BeneficLordFromMoon,
    /// The sixth lord (v. 19).
    LordOfSixth,
    /// A node in the lagna, the seventh, the eighth or the twelfth, in the
    /// seventh from a maraka lord, or joining one (v. 22).
    NodePlaced,
    /// Rahu, for Capricorn and Scorpio rising (v. 23).
    RahuForItsLagnas,
    /// Rahu in the sixth, eighth or twelfth with no benefic joining or
    /// aspecting him, who gives difficulty and not death (v. 24).
    RahuInDusthana,
    /// The lord of the third star from the birth star, Vipat, for a short life
    /// (v. 15).
    VipatStarLord,
    /// The lord of the fifth star, Pratyak, for a medium life (v. 15).
    PratyakStarLord,
    /// The lord of the seventh star, Vadha, for a long life (v. 15).
    VadhaStarLord,
    /// The lord of the twenty-third star (v. 16).
    TwentyThirdStarLord,
    /// The lord of the twenty-second decanate (v. 16).
    TwentySecondDecanateLord,
}

impl Reason {
    /// Every reason, in the verses' order.
    pub const ALL: [Reason; 20] = [
        Reason::LordOfSecond,
        Reason::LordOfSeventh,
        Reason::MaleficInSecond,
        Reason::MaleficInSeventh,
        Reason::MaleficWithSecondLord,
        Reason::MaleficWithSeventhLord,
        Reason::BeneficRelatedToTwelfthLord,
        Reason::LordOfEighth,
        Reason::SaturnIllDisposed,
        Reason::MaleficLordFromMoon,
        Reason::BeneficLordFromMoon,
        Reason::LordOfSixth,
        Reason::NodePlaced,
        Reason::RahuForItsLagnas,
        Reason::RahuInDusthana,
        Reason::VipatStarLord,
        Reason::PratyakStarLord,
        Reason::VadhaStarLord,
        Reason::TwentyThirdStarLord,
        Reason::TwentySecondDecanateLord,
    ];

    /// The six reasons of vv. 3 to 5, which make a graha a maraka in the class
    /// a rule names `any-maraka`.
    pub const PRIME: [Reason; 6] = [
        Reason::LordOfSecond,
        Reason::LordOfSeventh,
        Reason::MaleficInSecond,
        Reason::MaleficInSeventh,
        Reason::MaleficWithSecondLord,
        Reason::MaleficWithSeventhLord,
    ];

    /// What the verses say its periods bring.
    #[must_use]
    pub const fn brings(self) -> Brings {
        match self {
            Reason::BeneficLordFromMoon | Reason::RahuInDusthana => Brings::Difficulty,
            _ => Brings::Death,
        }
    }

    /// The class of life whose span this reason's periods belong to, where the
    /// verse ties it to one (v. 15).
    #[must_use]
    pub const fn class(self) -> Option<LifeClass> {
        match self {
            Reason::VipatStarLord => Some(LifeClass::Short),
            Reason::PratyakStarLord => Some(LifeClass::Medium),
            Reason::VadhaStarLord => Some(LifeClass::Long),
            _ => None,
        }
    }

    const fn bit(self) -> u32 {
        1 << self as u8
    }
}

/// What a maraka's periods bring, as the verses grade it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Brings {
    /// Illness, misery and difficulty equal to death, not death.
    Difficulty,
    /// Death, when the span of life agrees.
    Death,
}

/// A set of reasons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Reasons(u32);

impl Reasons {
    /// No reason.
    pub const NONE: Reasons = Reasons(0);

    /// The same set and `reason`.
    #[must_use]
    pub const fn with(self, reason: Reason) -> Reasons {
        Reasons(self.0 | reason.bit())
    }

    /// Whether it holds `reason`.
    #[must_use]
    pub const fn contains(self, reason: Reason) -> bool {
        self.0 & reason.bit() != 0
    }

    /// Whether it holds none.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The reasons it holds, in the verses' order.
    pub fn iter(self) -> impl Iterator<Item = Reason> {
        Reason::ALL
            .into_iter()
            .filter(move |reason| self.contains(reason.to_owned()))
    }

    /// The gravest thing its reasons bring, when it holds any.
    #[must_use]
    pub fn brings(self) -> Option<Brings> {
        self.iter().map(Reason::brings).max()
    }

    /// Whether it holds one of the six reasons of vv. 3 to 5.
    #[must_use]
    pub fn is_prime(self) -> bool {
        Reason::PRIME.iter().any(|reason| self.contains(*reason))
    }
}

impl Serialize for Reasons {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}

/// Every graha's reasons for being a maraka on a chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Marakas {
    /// Each graha's reasons, in [`Body::ALL`]'s order of the nine.
    by_graha: [Reasons; 9],
}

impl Marakas {
    /// A graha's reasons.
    #[must_use]
    pub fn of(&self, graha: Graha) -> Reasons {
        NINE.iter()
            .position(|nine| *nine == graha)
            .and_then(|at| self.by_graha.get(at))
            .copied()
            .unwrap_or(Reasons::NONE)
    }

    /// The grahas with any reason, and their reasons.
    pub fn iter(&self) -> impl Iterator<Item = (Graha, Reasons)> + '_ {
        NINE.into_iter()
            .zip(self.by_graha)
            .filter(|(_, reasons)| !reasons.is_empty())
    }
}

/// How a period is to be presented: always as a vulnerability to be read with
/// the span of life, never as a date. A consumer rendering one honours it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Presentation {
    /// A period of vulnerability, to be read with the span of life.
    Vulnerability,
}

/// What the running periods of a dasha are, as marakas.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Vulnerability {
    /// The reasons of each running level's lord, from the mahadasha down.
    pub levels: [Reasons; Levels::MAX],
    /// How many levels were read.
    pub depth: usize,
    /// Whether the mahadasha's lord is a malefic maraka running in a malefic's
    /// sub-period, which v. 8 makes the fatal one; a benefic's sub-period
    /// spares it.
    pub malefic_in_malefic: bool,
    /// Whether a sub-period of the sixth, eighth or twelfth lord runs (v. 19).
    pub dusthana_sub_period: bool,
    /// The gravest thing the running periods bring, if any.
    pub brings: Option<Brings>,
    /// How it is to be presented.
    pub presentation: Presentation,
}

/// The ages a class of life's span runs to (vv. 10 to 11): short to 32,
/// medium to 64, long to 100, and past that the supreme; in years from birth.
#[must_use]
pub const fn age_span(class: LifeClass) -> (f64, Option<f64>) {
    match class {
        LifeClass::Balarishta | LifeClass::Yogarishta | LifeClass::Short => (0.0, Some(32.0)),
        LifeClass::Medium => (32.0, Some(64.0)),
        LifeClass::Long => (64.0, Some(100.0)),
        LifeClass::Divine | LifeClass::Unlimited => (100.0, None),
    }
}

impl Evaluator<'_> {
    /// Every graha's reasons for being a maraka on this chart. Houses are
    /// whole signs from the lagna, a malefic is one under the readings as
    /// every rule's is, and "related" a shared sign or a mutual aspect (crux
    /// C105).
    #[must_use]
    pub fn marakas(&self) -> Marakas {
        let chart = self.chart();
        let lagna = chart.lagna();
        let sign_of = |graha: Graha| chart.placement(Body::Graha(graha)).sign;
        let house_of = |graha: Graha| House::between(lagna, sign_of(graha)).get();
        let lord = |house: u8| step(lagna, house).attributes().lord;
        let moon = sign_of(Graha::Moon);
        let from_moon = |house: u8| step(moon, house).attributes().lord;
        let (second, seventh, sixth, eighth, twelfth) =
            (lord(2), lord(7), lord(6), lord(8), lord(12));
        let prime_lords = [second, seventh];
        let (birth_star, _) = star(chart.placement(Body::Graha(Graha::Moon)).longitude);
        let star_lord = |count: u16| {
            teistro_core::catalogue::Nakshatra::from_id((birth_star as u16 + count - 1) % 27)
                .map(|nakshatra| nakshatra.attributes().vimshottari_lord)
        };
        let decanate = drekkana_of(chart.placement(Body::Lagna).longitude + 210.0)
            .attributes()
            .lord;

        let mut by_graha = [Reasons::NONE; 9];
        for (slot, graha) in by_graha.iter_mut().zip(NINE) {
            let malefic = self.is_malefic(graha);
            let mut reasons = Reasons::NONE;
            let mut add = |holds: bool, reason: Reason| {
                if holds {
                    reasons = reasons.with(reason);
                }
            };
            add(graha == second, Reason::LordOfSecond);
            add(graha == seventh, Reason::LordOfSeventh);
            add(malefic && house_of(graha) == 2, Reason::MaleficInSecond);
            add(malefic && house_of(graha) == 7, Reason::MaleficInSeventh);
            add(
                malefic && graha != second && sign_of(graha) == sign_of(second),
                Reason::MaleficWithSecondLord,
            );
            add(
                malefic && graha != seventh && sign_of(graha) == sign_of(seventh),
                Reason::MaleficWithSeventhLord,
            );
            add(
                !malefic && graha != twelfth && self.related(graha, twelfth),
                Reason::BeneficRelatedToTwelfthLord,
            );
            add(graha == eighth, Reason::LordOfEighth);
            add(graha == sixth, Reason::LordOfSixth);
            let lords_from_moon = graha == from_moon(2) || graha == from_moon(12);
            add(lords_from_moon && malefic, Reason::MaleficLordFromMoon);
            add(lords_from_moon && !malefic, Reason::BeneficLordFromMoon);
            if matches!(graha, Graha::Rahu | Graha::Ketu) {
                let placed = matches!(house_of(graha), 1 | 7 | 8 | 12)
                    || prime_lords.iter().any(|maraka| {
                        House::between(sign_of(*maraka), sign_of(graha)).get() == 7
                            || sign_of(*maraka) == sign_of(graha)
                    });
                add(placed, Reason::NodePlaced);
            }
            if graha == Graha::Rahu {
                add(
                    matches!(lagna, Rashi::Capricorn | Rashi::Scorpio),
                    Reason::RahuForItsLagnas,
                );
                add(
                    matches!(house_of(graha), 6 | 8 | 12) && !self.touched_by_benefic(graha),
                    Reason::RahuInDusthana,
                );
            }
            add(star_lord(3) == Some(graha), Reason::VipatStarLord);
            add(star_lord(5) == Some(graha), Reason::PratyakStarLord);
            add(star_lord(7) == Some(graha), Reason::VadhaStarLord);
            add(star_lord(23) == Some(graha), Reason::TwentyThirdStarLord);
            add(decanate == graha, Reason::TwentySecondDecanateLord);
            *slot = reasons;
        }
        // Saturn ill-disposed and related to a maraka of vv. 3 to 5 (v. 9).
        let saturn = chart.placement(Body::Graha(Graha::Saturn));
        let ill = matches!(
            saturn.dignity,
            Dignity::Enemy | Dignity::GreatEnemy | Dignity::Debilitated | Dignity::DeepDebilitated
        );
        let related_to_prime = NINE.iter().zip(by_graha).any(|(graha, reasons)| {
            *graha != Graha::Saturn && reasons.is_prime() && self.related(Graha::Saturn, *graha)
        });
        if ill && related_to_prime {
            if let Some(slot) = NINE
                .iter()
                .position(|graha| *graha == Graha::Saturn)
                .and_then(|at| by_graha.get_mut(at))
            {
                *slot = slot.with(Reason::SaturnIllDisposed);
            }
        }
        Marakas { by_graha }
    }

    /// What the running periods are, as marakas: each level's lord's reasons,
    /// v. 8's malefic major period in a malefic sub-period, and v. 19's
    /// sub-period of a dusthana lord.
    #[must_use]
    pub fn vulnerability(
        &self,
        marakas: &Marakas,
        running: impl IntoIterator<Item = Running>,
    ) -> Vulnerability {
        let lagna = self.chart().lagna();
        let dusthana_lords = [6, 8, 12].map(|house| step(lagna, house).attributes().lord);
        let mut levels = [Reasons::NONE; Levels::MAX];
        let mut lords = [None; Levels::MAX];
        let mut depth = 0;
        for ((slot, lord), period) in levels.iter_mut().zip(lords.iter_mut()).zip(running) {
            *slot = marakas.of(period.lord);
            *lord = Some(period.lord);
            depth += 1;
        }
        let malefic = |graha: Option<Graha>| graha.is_some_and(|graha| self.is_malefic(graha));
        let [major, sub, ..] = lords;
        let [major_reasons, ..] = levels;
        let malefic_in_malefic = major_reasons
            .brings()
            .is_some_and(|brings| brings == Brings::Death)
            && malefic(major)
            && malefic(sub);
        let dusthana_sub_period = sub.is_some_and(|lord| dusthana_lords.contains(&lord));
        let brings = levels
            .iter()
            .take(depth)
            .filter_map(|reasons| reasons.brings())
            .max();
        Vulnerability {
            levels,
            depth,
            malefic_in_malefic,
            dusthana_sub_period,
            brings,
            presentation: Presentation::Vulnerability,
        }
    }

    /// Whether a graha is a malefic under the readings, as a rule's
    /// `planet-is` asks.
    fn is_malefic(&self, graha: Graha) -> bool {
        self.holds(
            &Condition::PlanetIs {
                planet: BodyRef::Body(Body::Graha(graha)),
                class: Class::Malefic,
            },
            &mut Participants::default(),
        )
    }

    /// A shared sign or a mutual aspect between two grahas.
    fn related(&self, one: Graha, other: Graha) -> bool {
        let body = |graha: Graha| BodyRef::Body(Body::Graha(graha));
        let chart = self.chart();
        let together =
            chart.placement(Body::Graha(one)).sign == chart.placement(Body::Graha(other)).sign;
        let aspects = |from: Graha, to: Graha| {
            self.holds(
                &Condition::PlanetAspectsPlanet {
                    from: BodySubject::Ref(body(from)),
                    target: SignRef::Of(body(to)),
                },
                &mut Participants::default(),
            )
        };
        together || (aspects(one, other) && aspects(other, one))
    }

    /// A natural benefic joining or aspecting a graha.
    fn touched_by_benefic(&self, graha: Graha) -> bool {
        let chart = self.chart();
        let sign = chart.placement(Body::Graha(graha)).sign;
        [Graha::Moon, Graha::Mercury, Graha::Jupiter, Graha::Venus]
            .iter()
            .any(|benefic| {
                chart.placement(Body::Graha(*benefic)).sign == sign
                    || self.holds(
                        &Condition::PlanetAspectsPlanet {
                            from: BodySubject::Ref(BodyRef::Body(Body::Graha(*benefic))),
                            target: SignRef::Of(BodyRef::Body(Body::Graha(graha))),
                        },
                        &mut Participants::default(),
                    )
            })
    }
}

/// The sign `house` counts to from `from`, the first being itself.
fn step(from: Rashi, house: u8) -> Rashi {
    Rashi::from_id((u16::from(from as u8) + u16::from(house.saturating_sub(1))) % 12)
        .unwrap_or(from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::Readings;
    use crate::test_chart::{chart, place};

    #[test]
    fn a_graha_is_a_maraka_for_each_reason_the_verses_give_it() {
        // Aries rising: Venus lords the second and the seventh, Mars the eighth.
        let mut c = chart();
        let body = |graha| Body::Graha(graha);
        place(&mut c, body(Graha::Venus), Rashi::Cancer);
        place(&mut c, body(Graha::Saturn), Rashi::Cancer); // a malefic with both lords
        place(&mut c, body(Graha::Mars), Rashi::Taurus); // a malefic in the second
        place(&mut c, body(Graha::Jupiter), Rashi::Taurus);
        place(&mut c, body(Graha::Rahu), Rashi::Libra); // a malefic, and a node, in the seventh
        place(&mut c, body(Graha::Ketu), Rashi::Aries); // a node in the lagna
        place(&mut c, body(Graha::Sun), Rashi::Leo);
        place(&mut c, body(Graha::Moon), Rashi::Sagittarius);
        place(&mut c, body(Graha::Mercury), Rashi::Gemini);
        let evaluator = Evaluator::new(&c, Readings::TEXTS);
        let marakas = evaluator.marakas();
        let has = |graha, reason| marakas.of(graha).contains(reason);
        assert!(
            has(Graha::Venus, Reason::LordOfSecond) && has(Graha::Venus, Reason::LordOfSeventh)
        );
        assert!(has(Graha::Saturn, Reason::MaleficWithSecondLord));
        assert!(
            has(Graha::Mars, Reason::MaleficInSecond) && has(Graha::Mars, Reason::LordOfEighth)
        );
        assert!(has(Graha::Rahu, Reason::MaleficInSeventh) && has(Graha::Rahu, Reason::NodePlaced));
        assert!(has(Graha::Ketu, Reason::NodePlaced));
        // From the Moon in Sagittarius, Saturn lords the second and Mars the
        // twelfth, both malefics.
        assert!(has(Graha::Saturn, Reason::MaleficLordFromMoon));
        assert!(has(Graha::Mars, Reason::MaleficLordFromMoon));
        assert!(!marakas.of(Graha::Jupiter).is_prime());
        // The prime reasons are the kernel's maraka class, graha for graha.
        for (graha, reasons) in NINE.map(|graha| (graha, marakas.of(graha))) {
            let class = evaluator.holds(
                &Condition::PlanetIs {
                    planet: BodyRef::Body(Body::Graha(graha)),
                    class: Class::Maraka,
                },
                &mut Participants::default(),
            );
            assert_eq!(reasons.is_prime(), class, "{graha:?}");
        }

        // V. 8: Mars's major period kills in Saturn's sub-period, and is
        // spared in Jupiter's; Mars lords the eighth, a dusthana.
        let chain = |major, sub| [Running::graha(major), Running::graha(sub)];
        let fatal = evaluator.vulnerability(&marakas, chain(Graha::Mars, Graha::Saturn));
        assert!(fatal.malefic_in_malefic && !fatal.dusthana_sub_period);
        assert_eq!((fatal.depth, fatal.brings), (2, Some(Brings::Death)));
        assert_eq!(fatal.presentation, Presentation::Vulnerability);
        let spared = evaluator.vulnerability(&marakas, chain(Graha::Mars, Graha::Jupiter));
        assert!(!spared.malefic_in_malefic);
        let dusthana = evaluator.vulnerability(&marakas, chain(Graha::Jupiter, Graha::Mars));
        assert!(dusthana.dusthana_sub_period);
    }

    #[test]
    fn a_set_of_reasons_says_what_it_brings() {
        let difficulty = Reasons::NONE.with(Reason::BeneficLordFromMoon);
        assert_eq!(difficulty.brings(), Some(Brings::Difficulty));
        assert_eq!(
            difficulty.with(Reason::LordOfEighth).brings(),
            Some(Brings::Death)
        );
        assert_eq!(Reasons::NONE.brings(), None);
        assert_eq!(
            Reasons::NONE
                .with(Reason::VadhaStarLord)
                .iter()
                .collect::<Vec<_>>(),
            [Reason::VadhaStarLord]
        );
        assert_eq!(Reason::VipatStarLord.class(), Some(LifeClass::Short));
        assert_eq!(age_span(LifeClass::Medium), (32.0, Some(64.0)));
        assert_eq!(age_span(LifeClass::Unlimited), (100.0, None));
    }
}
