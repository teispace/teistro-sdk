//! The **sixteen Tajika yogas** of the annual chart
//! (`03-design/tajika-yogas.md`).
//!
//! # They answer a question, they are not facts about a chart
//!
//! This is the thing that decides the module's shape, and it is not how
//! a Parashari yoga works. Gaja Kesari either holds in a chart or does
//! not. **Fourteen of these sixteen are judgements about a pair**:
//!
//! - the **lagnesha**, the lord of the annual lagna — fixed by the chart;
//! - the **karyesha**, the lord of the house *the matter asked about*
//!   belongs to, which the chart cannot know.
//!
//! So "what yogas does this year have?" is not a well-formed question.
//! "Is the marriage promised this year?" is: it fixes the karyesha as the
//! seventh lord, and the sixteen say whether the promise is fulfilled,
//! delayed, carried by someone else, or negated. [`year_yogas`]
//! therefore takes a [`House`] and answers for it.
//!
//! # What is built
//!
//! All but the few [`YearYoga::awaiting`] names, and those say so rather
//! than being silently absent: [`YearYogas::unanswered`] lists them at
//! every call, because "Kuttha did not hold" and "this build cannot tell
//! you about Kuttha" are different statements and a consumer that cannot
//! tell them apart has been misled. [`YearYogas::why`] carries the reason
//! for each, and `check-muntha` counts both sets from the type — which is
//! why no count is written here, where it would go stale.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::bala::{
    AnnualSky, Bala, SEVEN, drekkana_lord, hudda_lord, navamsha_lord, panchavargiya,
    sign_of_longitude,
};
use crate::drishti::{
    Between, Drishti, DrishtiRules, between_with_rules, between_within, deeptamsha, speed_rank,
};
use crate::states::AnnualStates;

/// One of the sixteen yogas of Tajika's own reckoning (K.S. Charak, *A
/// Textbook of Varshaphala*, Table X-3).
///
/// Named in the table's order. Only [`YearYoga::Ikabala`] and
/// [`YearYoga::Induvara`] are facts about a chart alone; the other
/// fourteen are judgements about the lagnesha and the karyesha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum YearYoga {
    /// Every planet in a kendra or a panaphara.
    Ikabala,
    /// Every planet in an apoklima.
    Induvara,
    /// The pair are coming together, in one of the three kinds
    /// [`crate::Yoga`] enumerates.
    Ithasala,
    /// The pair are drawing apart.
    Ishrafa,
    /// The two do not aspect, and a planet **faster than both** carries
    /// the light between them: past one, coming to the other.
    Nakta,
    /// The two do not aspect, and a planet **slower than both** gathers
    /// their light: both are coming to it.
    Yamaya,
    /// An Ithasala a malefic destroys.
    Manau,
    /// An Ithasala the Moon joins.
    Kamboola,
    /// An Ithasala an unqualified Moon completes on entering the next
    /// sign.
    GairiKamboola,
    /// An Ithasala an unqualified Moon negates by standing apart from it.
    Khallasara,
    /// An Ithasala where either of the pair is afflicted.
    Rudda,
    /// An Ithasala where the slower is strong and the faster weak.
    DuhphaliKuttha,
    /// Both weak, and one in Ithasala with a third, strong planet.
    DutthotthaDavira,
    /// No aspect and no Ithasala, the karyesha completing one from the
    /// next sign.
    Tambira,
    /// Both powerful, well placed and under benefic influence.
    Kuttha,
    /// Both weak, in the trika houses, combust or retrograde.
    Durapha,
}

impl YearYoga {
    /// All sixteen, in the source's own table order.
    pub const ALL: [YearYoga; 16] = [
        YearYoga::Ikabala,
        YearYoga::Induvara,
        YearYoga::Ithasala,
        YearYoga::Ishrafa,
        YearYoga::Nakta,
        YearYoga::Yamaya,
        YearYoga::Manau,
        YearYoga::Kamboola,
        YearYoga::GairiKamboola,
        YearYoga::Khallasara,
        YearYoga::Rudda,
        YearYoga::DuhphaliKuttha,
        YearYoga::DutthotthaDavira,
        YearYoga::Tambira,
        YearYoga::Kuttha,
        YearYoga::Durapha,
    ];

    /// What this build still needs before it can answer for this yoga;
    /// nothing when it answers today.
    ///
    /// Matched exhaustively on purpose: a yoga added to [`YearYoga::ALL`]
    /// cannot compile until somebody has said which side of this it
    /// falls on, and `check-muntha` prints both sets with these reasons.
    #[must_use]
    pub const fn awaiting(self) -> Option<&'static str> {
        match self {
            YearYoga::Ithasala
            | YearYoga::Ishrafa
            | YearYoga::Nakta
            | YearYoga::Yamaya
            | YearYoga::Manau
            | YearYoga::Kamboola
            | YearYoga::Khallasara
            | YearYoga::DutthotthaDavira
            | YearYoga::Rudda
            | YearYoga::DuhphaliKuttha
            | YearYoga::Durapha
            | YearYoga::Ikabala
            | YearYoga::Induvara => None,
            YearYoga::GairiKamboola => Some(
                "an unqualified Moon, and where it will stand in the next sign: the only one of the sixteen that asks what happens next",
            ),
            YearYoga::Tambira => {
                Some("the karyesha at a sign's end, completing an Ithasala from the next")
            }
            YearYoga::Kuttha => Some(
                "Tajika's own benefics, which Table X-3 does not enumerate as it enumerates the malefics (crux C117)",
            ),
        }
    }

    /// Whether answering needs the chart's [`AnnualStates`] — which
    /// planets are retrograde and which combust.
    ///
    /// A call that does not supply them lists these under
    /// [`YearYogas::unanswered`], and [`YearYogas::why`] says so: the
    /// build can answer, and this call gave it nothing to answer from.
    #[must_use]
    pub const fn needs_states(self) -> bool {
        matches!(
            self,
            YearYoga::Rudda | YearYoga::DuhphaliKuttha | YearYoga::Durapha
        )
    }

    /// Whether it is a judgement **about** an Ithasala the pair already
    /// make — destroying, joining, negating or spoiling it — and so can
    /// hold only in a matter where one stands.
    ///
    /// A fact about the definition and not the build, so it answers for
    /// the unbuilt Gairi-Kamboola too; the measured page holds every
    /// count of these under the Ithasala's own.
    #[must_use]
    pub const fn judges_an_ithasala(self) -> bool {
        matches!(
            self,
            YearYoga::Manau
                | YearYoga::Kamboola
                | YearYoga::GairiKamboola
                | YearYoga::Khallasara
                | YearYoga::Rudda
                | YearYoga::DuhphaliKuttha
        )
    }

    /// Whether it is a fact about the **chart** rather than a judgement
    /// about a pair, and so holds in every matter of a chart or in none.
    #[must_use]
    pub const fn is_chart_fact(self) -> bool {
        matches!(self, YearYoga::Ikabala | YearYoga::Induvara)
    }

    /// Whether it needs **both** lords weak before asking anything else,
    /// and so can hold only where they are.
    #[must_use]
    pub const fn needs_a_weak_pair(self) -> bool {
        matches!(self, YearYoga::DutthotthaDavira | YearYoga::Durapha)
    }

    /// Whether this build answers for it.
    #[must_use]
    pub const fn is_built(self) -> bool {
        self.awaiting().is_none()
    }
}

/// Tajika's own malefics, as Manau names them: **Mars and Saturn**.
///
/// Deliberately *not* the catalogue's `Nature::Malefic`, which follows
/// the Parashari reckoning and also carries the Sun, Rahu and Ketu. The
/// source's Table X-3 names two planets for this yoga and naming them
/// here is the only way that stays true when the catalogue's own list is
/// read for some other purpose (crux C114).
pub const MALEFICS: [Graha; 2] = [Graha::Mars, Graha::Saturn];

/// Why a planet is not **unqualified**, clause by clause.
///
/// The source defines the word outright: "a planet is unqualified when
/// it is neither exalted nor debilitated, nor aspected/associated, nor
/// in its own Hudda, Drekkana or Navamsha". Every clause is carried
/// rather than collapsed into the bool, because a reader asking why a
/// Khallasara did not hold wants the clause and not the verdict — and
/// because a pass can then count which clause does the disqualifying.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[expect(
    clippy::struct_excessive_bools,
    reason = "six independent clauses of one printed definition, any of which may hold at once; \
              collapsing them into a state would lose the clause a reader asks for. \
              `Qualification::clauses` is how a consumer walks them without naming fields."
)]
pub struct Qualification {
    /// Whose.
    pub graha: Graha,
    /// In its sign of exaltation.
    pub exalted: bool,
    /// In its sign of debilitation.
    pub debilitated: bool,
    /// Aspected by, or sharing a sign with, another of the seven.
    ///
    /// One clause and not two: in Tajika a planet in the **same sign**
    /// stands at house 1, which is `Drishti::Inimical` and so an aspect
    /// already. "Aspected or associated" is one question here because
    /// the tradition's own geometry makes it one.
    pub aspected: bool,
    /// In a Hudda it rules.
    ///
    /// **Never true of the Sun or the Moon.** The Hudda is the Egyptian
    /// terms, which divide every sign among Mars, Mercury, Jupiter,
    /// Venus and Saturn and give the luminaries nothing — so for the one
    /// planet the source applies `unqualified` to, this clause is
    /// vacuous. Kept rather than special-cased, because the definition
    /// states it and a reader comparing code to text should find all six
    /// clauses; the measured zero is explained in
    /// `03-design/muntha-measured.md` §10 rather than left to look like
    /// an absence of data.
    pub own_hudda: bool,
    /// In a Drekkana it rules.
    pub own_drekkana: bool,
    /// In a Navamsha it rules.
    pub own_navamsha: bool,
}

impl Qualification {
    /// The six clauses, each with the source's own words for it, in the
    /// order the definition states them.
    ///
    /// A consumer counting or displaying them walks this rather than
    /// naming the fields, so a clause cannot be relabelled by being
    /// moved — which is what a pass indexing a fixed-length array of
    /// counts would otherwise risk every time the struct is edited.
    #[must_use]
    pub const fn clauses(self) -> [(&'static str, bool); 6] {
        [
            ("exalted", self.exalted),
            ("debilitated", self.debilitated),
            (
                "aspected or associated by another of the seven",
                self.aspected,
            ),
            ("in a Hudda it rules", self.own_hudda),
            ("in a Drekkana it rules", self.own_drekkana),
            ("in a Navamsha it rules", self.own_navamsha),
        ]
    }

    /// Whether every clause is false, which is what the source calls
    /// **unqualified**.
    #[must_use]
    pub fn is_unqualified(self) -> bool {
        self.clauses().iter().all(|(_, holds)| !*holds)
    }
}

/// How a planet of an annual chart stands to the source's definition of
/// **unqualified**.
///
/// # Errors
///
/// A body outside the seven, named `graha`.
pub fn qualification(graha: Graha, sky: &AnnualSky) -> Result<Qualification, Error> {
    if speed_rank(graha).is_none() {
        return Err(
            Error::invalid_arg(format!("{graha:?} is not one of the seven"))
                .with_field(String::from("graha")),
        );
    }
    let longitude = sky.longitude_of(graha);
    if !longitude.is_finite() {
        return Err(
            Error::invalid_arg(format!("the annual chart does not place {graha:?}"))
                .with_field(String::from("graha")),
        );
    }
    let sign = sign_of_longitude(longitude);
    let attributes = graha.attributes();
    let aspected = SEVEN.into_iter().any(|other| {
        other != graha
            && Drishti::between_signs(sign_of_longitude(sky.longitude_of(other)), sign).is_aspect()
    });
    Ok(Qualification {
        graha,
        exalted: attributes.exaltation.is_some_and(|at| at.sign == sign),
        debilitated: attributes.debilitation.is_some_and(|at| at.sign == sign),
        aspected,
        own_hudda: hudda_lord(longitude) == graha,
        own_drekkana: drekkana_lord(longitude) == graha,
        own_navamsha: navamsha_lord(longitude) == graha,
    })
}

/// The Vishwa bala below which a planet is **weak** for the yogas.
///
/// Two sources meet on five. K.S. Charak gives it for the
/// *office-bearers* — below five units the Muntha lord takes the year —
/// and the graded reading of the Vishwa scale calls a planet under five
/// *Nirbali*, strengthless. Neither says it for the yogas, which name
/// *weak* and stop, so it is a [`YogaRules`] field and not a constant
/// in the judgement (crux C116, measured in
/// `03-design/muntha-measured.md` §11).
///
/// Stated here in full rather than aliased to [`crate::WEAK_BELOW`],
/// though the two carry the same number today. They answer **different
/// questions** — which office-bearer may hold the year, and which
/// planet is weak for a yoga — and an alias would let a later reading
/// of one silently move the other. A test asserts the coincidence, so
/// parting them is a decision somebody has to make on purpose.
pub const YOGA_WEAK_BELOW: Bala = Bala::new(5, 0, 0);

/// The Vishwa bala from which a planet is **strong** on its bala alone,
/// without a dignity.
///
/// Charak never says where strength begins. The graded reading of the
/// scale does: under five *Nirbali*, five to ten *Madhya* (middling),
/// ten to fifteen *Poorna* (fully strong), above fifteen *Parakrami* —
/// so a planet the yogas call *strong* or *powerful* is taken to be one
/// that is fully strong, and the five-to-ten band is **neither**. That
/// grading is a second book's and not Charak's, which is why this too
/// is a [`YogaRules`] field (crux C116); setting it equal to
/// [`YOGA_WEAK_BELOW`] recovers the reading with no middle at all.
pub const YOGA_STRONG_FROM: Bala = Bala::new(10, 0, 0);

/// How the sixteen are read where the source leaves a choice.
///
/// One value for the whole judgement rather than one argument per
/// question: the readings the yogas need are not all about aspects, and
/// a caller that wants to move a strength floor should not have to
/// learn which of them travels in `DrishtiRules`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
pub struct YogaRules {
    /// How the aspects and the band between Poorna and Ishrafa are read.
    pub drishti: DrishtiRules,
    /// The Vishwa bala below which a planet with no dignity is weak.
    /// Defaults to [`YOGA_WEAK_BELOW`].
    pub weak_below: Bala,
    /// The Vishwa bala from which a planet is strong without a dignity.
    /// Defaults to [`YOGA_STRONG_FROM`]; never below `weak_below`, since
    /// no planet can be both.
    pub strong_from: Bala,
}

impl Default for YogaRules {
    fn default() -> YogaRules {
        YogaRules {
            drishti: DrishtiRules::default(),
            weak_below: YOGA_WEAK_BELOW,
            strong_from: YOGA_STRONG_FROM,
        }
    }
}

impl From<DrishtiRules> for YogaRules {
    fn from(drishti: DrishtiRules) -> YogaRules {
        YogaRules {
            drishti,
            ..YogaRules::default()
        }
    }
}

impl YogaRules {
    /// Refuses floors that would let a planet be strong and weak at
    /// once, naming the field to move.
    ///
    /// # Errors
    ///
    /// `strong_from` below `weak_below`.
    pub fn check(self) -> Result<YogaRules, Error> {
        if self.strong_from < self.weak_below {
            return Err(Error::invalid_arg(format!(
                "strength begins at {} but weakness runs up to {}: a planet between would be both",
                self.strong_from, self.weak_below
            ))
            .with_field("strong_from")
            .with_hint(format!(
                "raise strong_from to at least {}, or lower weak_below",
                self.weak_below
            )));
        }
        Ok(self)
    }
}

/// How a planet of an annual chart stands to **strong** and **weak**,
/// clause by clause.
///
/// Strong is a disjunction of the source's own three — "exalted, in its
/// own house or otherwise strong". Weak is the absence of all three
/// **and** a bala under the lower floor. Between the floors lies the
/// middling planet, which is neither: the yogas that want a weak pair
/// and the one that wants a strong pair both pass it by. Carrying the
/// clauses and both floors rather than a verdict is what lets a reader
/// asking *why* be answered, and lets the floors move (crux C116)
/// without the yogas that read them being rewritten.
///
/// Not to be confused with the rule kernel's `Strengths`, the Parashari
/// figures of BPHS ch. 27 a rule compares against its requirement: this
/// is Tajika's own verdict, on the Panchavargiya scale of twenty. The two
/// traditions share a word and nothing else, and the SDK root exports
/// both under their own names, because a type a signature names must be
/// found by the name the signature gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Strength {
    /// Whose.
    pub graha: Graha,
    /// Its Panchavargiya strength, on the scale of twenty the source
    /// prints it on.
    pub vishwa: Bala,
    /// In its sign of exaltation.
    pub exalted: bool,
    /// In a sign it rules.
    pub own_sign: bool,
    /// The floors this reading was taken against, carried so that a
    /// verdict can be read back without its rules.
    pub weak_below: Bala,
    /// See [`Strength::weak_below`].
    pub strong_from: Bala,
}

impl Strength {
    /// The three ways of being strong, each with the source's own words
    /// for it, in the order it states them.
    ///
    /// Walked rather than named, for the same reason
    /// [`Qualification::clauses`] is: a clause cannot then be
    /// relabelled by being moved.
    #[must_use]
    pub fn clauses(self) -> [(&'static str, bool); 3] {
        [
            ("exalted", self.exalted),
            ("in a sign it rules", self.own_sign),
            (
                "fully strong in Vishwa bala",
                self.vishwa >= self.strong_from,
            ),
        ]
    }

    /// Whether any clause holds, which is what the source calls
    /// **strong**.
    #[must_use]
    pub fn is_strong(self) -> bool {
        self.clauses().iter().any(|(_, holds)| *holds)
    }

    /// Whether no clause holds **and** the bala is under the lower
    /// floor, which is what the source calls **weak**.
    #[must_use]
    pub fn is_weak(self) -> bool {
        !self.is_strong() && self.vishwa < self.weak_below
    }

    /// Neither: no dignity, and a bala between the floors.
    #[must_use]
    pub fn is_middling(self) -> bool {
        !self.is_strong() && !self.is_weak()
    }
}

/// How every one of the seven stands to strong and weak, under `rules`.
///
/// The Panchavargiya bala is computed **once** for the chart and shared,
/// because every planet's strength is read from the same five divisions
/// and a per-planet call would recompute all seven to answer for one.
fn strengths(sky: &AnnualSky, rules: YogaRules) -> Result<[Strength; 7], Error> {
    let rules = rules.check()?;
    // `map` over the array keeps the length a type fact, and
    // `Panchavargiya` already carries the sign it was read in, so the
    // dignities cost no second lookup.
    Ok(panchavargiya(sky)?.map(|bala| Strength {
        graha: bala.graha,
        vishwa: bala.vishwa,
        exalted: bala
            .graha
            .attributes()
            .exaltation
            .is_some_and(|at| at.sign == bala.sign),
        own_sign: bala.sign.attributes().lord == bala.graha,
        weak_below: rules.weak_below,
        strong_from: rules.strong_from,
    }))
}

/// How a planet of an annual chart stands to **strong** and **weak**,
/// under the default floors.
///
/// # Errors
///
/// A body outside the seven, named `graha`; an annual chart that does
/// not place it.
pub fn strength(graha: Graha, sky: &AnnualSky) -> Result<Strength, Error> {
    strength_with_rules(graha, sky, YogaRules::default())
}

/// How a planet stands to strong and weak, with the floors chosen.
///
/// # Errors
///
/// A body outside the seven, named `graha`; an annual chart that does
/// not place it; floors that would let a planet be both, named
/// `strong_from`.
pub fn strength_with_rules(
    graha: Graha,
    sky: &AnnualSky,
    rules: YogaRules,
) -> Result<Strength, Error> {
    if speed_rank(graha).is_none() {
        return Err(
            Error::invalid_arg(format!("{graha:?} is not one of the seven"))
                .with_field(String::from("graha")),
        );
    }
    strengths(sky, rules)?
        .into_iter()
        .find(|one| one.graha == graha)
        .ok_or_else(|| {
            Error::invalid_arg(format!("the annual chart does not place {graha:?}"))
                .with_field(String::from("graha"))
        })
}

/// The house a planet stands in, counted by whole signs from the annual
/// lagna — which Ikabala, Induvara and the *trika* clause of an
/// affliction all read, so it is reckoned once.
fn house_of(graha: Graha, lagna: Rashi, sky: &AnnualSky) -> House {
    House::between(lagna, sign_of_longitude(sky.longitude_of(graha)))
}

/// Whether a malefic standing in `from` reaches a planet in `to`: joined
/// with it, or aspecting it inimically.
///
/// One condition and not two, since in Tajika a shared sign is house 1
/// and house 1 is inimical. This is Table X-3's own wording for Manau,
/// the one place it says how a malefic reaches; Rudda's "under malefic
/// influence" is read the same way rather than given a second
/// definition (crux C118).
fn malefic_reaches(from: Rashi, to: Rashi) -> bool {
    matches!(
        Drishti::between_signs(from, to),
        Drishti::Inimical | Drishti::SecretlyInimical
    )
}

/// How a planet of an annual chart stands to the source's afflictions,
/// clause by clause.
///
/// Rudda's list — "retrograde, combust, debilitated, in the 6th, 8th or
/// 12th, or under malefic influence" — of which Durapha reads three.
/// Carried as clauses, as [`Qualification`] and [`Strength`] are, so a
/// reader asking *why* a Rudda held gets the clause that made it.
#[expect(
    clippy::struct_excessive_bools,
    reason = "five named clauses of one definition, each read on its own"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Affliction {
    /// Whose.
    pub graha: Graha,
    /// Going backwards through the zodiac.
    pub retrograde: bool,
    /// Burnt by the Sun.
    pub combust: bool,
    /// In its sign of debilitation.
    pub debilitated: bool,
    /// In the 6th, 8th or 12th house from the annual lagna.
    pub trika: bool,
    /// Joined, or aspected inimically, by Mars or Saturn other than
    /// itself — the partner in the pair included, since the clause is a
    /// condition of the planet and not a third party's act (crux C118).
    pub under_malefic: bool,
}

impl Affliction {
    /// The five clauses, each with the source's own words for it, in the
    /// order it states them.
    #[must_use]
    pub const fn clauses(self) -> [(&'static str, bool); 5] {
        [
            ("retrograde", self.retrograde),
            ("combust", self.combust),
            ("debilitated", self.debilitated),
            ("in the 6th, 8th or 12th", self.trika),
            ("under malefic influence", self.under_malefic),
        ]
    }

    /// Whether any clause holds.
    #[must_use]
    pub fn is_afflicted(self) -> bool {
        self.clauses().iter().any(|(_, holds)| *holds)
    }
}

/// How a planet of an annual chart stands to the source's afflictions.
///
/// # Errors
///
/// A body outside the seven, named `graha`; an annual lagna that is not a
/// number, named `annual_lagna_deg`; states that no chart can hold,
/// named by their field.
pub fn affliction(
    graha: Graha,
    annual_lagna_deg: f64,
    sky: &AnnualSky,
    states: &AnnualStates,
) -> Result<Affliction, Error> {
    if speed_rank(graha).is_none() {
        return Err(
            Error::invalid_arg(format!("{graha:?} is not one of the seven"))
                .with_field(String::from("graha")),
        );
    }
    states.validate()?;
    Ok(afflicted(graha, lagna_of(annual_lagna_deg)?, sky, states))
}

/// The annual lagna's sign, refusing a lagna that is not a number.
fn lagna_of(annual_lagna_deg: f64) -> Result<Rashi, Error> {
    if annual_lagna_deg.is_finite() {
        Ok(sign_of_longitude(annual_lagna_deg))
    } else {
        Err(
            Error::invalid_arg("the annual lagna must be a number of degrees")
                .with_field(String::from("annual_lagna_deg")),
        )
    }
}

/// [`affliction`] once its inputs are known good.
fn afflicted(graha: Graha, lagna: Rashi, sky: &AnnualSky, states: &AnnualStates) -> Affliction {
    let sign = sign_of_longitude(sky.longitude_of(graha));
    Affliction {
        graha,
        retrograde: states.is_retrograde(graha),
        combust: states.is_combust(graha),
        debilitated: graha
            .attributes()
            .debilitation
            .is_some_and(|at| at.sign == sign),
        trika: house_of(graha, lagna, sky).is_trika(),
        under_malefic: MALEFICS.into_iter().any(|malefic| {
            malefic != graha && malefic_reaches(sign_of_longitude(sky.longitude_of(malefic)), sign)
        }),
    }
}

/// One of the sixteen, found holding, with what made it hold.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Held {
    /// Which of the sixteen.
    pub yoga: YearYoga,
    /// The pair's own relation, where that is what made it: an Ithasala
    /// or an Ishrafa between the lagnesha and the karyesha.
    pub between: Option<Between>,
    /// The third planet that carried or gathered the light, where one
    /// did: Nakta's faster intermediary or Yamaya's slower one.
    pub through: Option<Graha>,
    /// How the intermediary stands to the lagnesha, and to the karyesha.
    pub legs: Option<[Between; 2]>,
    /// How the lagnesha and the karyesha stand to the afflictions, where
    /// those are what made it: Rudda and Durapha.
    pub afflictions: Option<[Affliction; 2]>,
}

/// Every one of the sixteen this build can answer for, for one matter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YearYogas {
    /// The house the question was asked about, counted from the annual
    /// lagna by whole signs.
    pub house: House,
    /// The sign that house falls in.
    pub sign: Rashi,
    /// The **lagnesha**: the lord of the annual lagna.
    pub lagnesha: Graha,
    /// The **karyesha**: the lord of the house asked about.
    pub karyesha: Graha,
    /// Whether one planet is both.
    ///
    /// Always true of the **first** house, whose lord is the lagna's lord
    /// by definition — so a question about the native's own self can
    /// never be a judgement about a pair. True of one further house
    /// under a lagna ruled by one of the five that rule two signs, and of
    /// no other under Cancer or Leo.
    ///
    /// A planet makes no yoga with itself, so the fourteen pair yogas
    /// have nothing to judge when this is true. Whether the tradition
    /// reads that as the matter being promised outright is a question no
    /// text in reach answers, so it is reported and not decided.
    pub same_lord: bool,
    /// How the two stand to each other, where they are two.
    pub between: Option<Between>,
    /// Every one of the sixteen that holds.
    pub held: Vec<Held>,
    /// The ones this call cannot answer for: those the build does not
    /// compute, and — where no [`AnnualStates`] were supplied — the three
    /// that need them.
    ///
    /// Never empty today, and that is the point: an absent yoga in
    /// [`YearYogas::held`] means it did not hold **only** for the ones
    /// not listed here. [`YearYogas::why`] says which reason applies.
    pub unanswered: Vec<YearYoga>,
    /// The retrograde and combust planets the judgement read, where it
    /// was given any — reported, so an answer can be read back without
    /// the call that produced it.
    pub states: Option<AnnualStates>,
}

impl YearYogas {
    /// Whether a named yoga holds.
    ///
    /// `None` where this build cannot say, which is not the same answer
    /// as `Some(false)` and is why this returns an option rather than a
    /// bool.
    #[must_use]
    pub fn holds(&self, yoga: YearYoga) -> Option<bool> {
        // Asked of this call's own list and not of the build: a yoga the
        // build answers can still be one this call could not.
        if self.unanswered.contains(&yoga) {
            None
        } else {
            Some(self.held.iter().any(|one| one.yoga == yoga))
        }
    }

    /// Why this call could not answer for `yoga`, or `None` where it did.
    ///
    /// Either what the build still needs ([`YearYoga::awaiting`]) or, for
    /// the three that read retrograde and combustion, that this call was
    /// not given them — two different remedies, so two different answers.
    #[must_use]
    pub fn why(&self, yoga: YearYoga) -> Option<&'static str> {
        if !self.unanswered.contains(&yoga) {
            return None;
        }
        yoga.awaiting().or(Some(
            "retrograde and combustion, which this call was not given: pass `AnnualStates` to \
             `year_yogas_with_states`, or ask the façade, which reads them from the founded chart",
        ))
    }
}

/// The sixteen yogas of an annual chart, for one matter, under the
/// readings this module defaults to.
///
/// # Errors
///
/// An annual lagna that is not a number, named `annual_lagna_deg`; a sky
/// that does not place one of the seven.
pub fn year_yogas(
    annual_lagna_deg: f64,
    house: House,
    sky: &AnnualSky,
) -> Result<YearYogas, Error> {
    year_yogas_with_rules(annual_lagna_deg, house, sky, YogaRules::default())
}

/// The sixteen yogas for one matter, under stated readings.
///
/// Without [`AnnualStates`], so Rudda, Duhphali-kuttha and Durapha are
/// listed under [`YearYogas::unanswered`]; [`year_yogas_with_states`]
/// answers them too.
///
/// # Errors
///
/// As [`year_yogas`]; and floors that would let a planet be strong and
/// weak at once, named `strong_from` — refused on every call, and not
/// only on the matters that happen to ask about strength.
pub fn year_yogas_with_rules(
    annual_lagna_deg: f64,
    house: House,
    sky: &AnnualSky,
    rules: YogaRules,
) -> Result<YearYogas, Error> {
    judge(annual_lagna_deg, house, sky, None, rules)
}

/// The sixteen yogas for one matter, with the chart's retrograde and
/// combust planets, so every yoga the build computes is answered.
///
/// # Errors
///
/// As [`year_yogas_with_rules`]; and states that no chart can hold, named
/// `retrograde` or `combust`.
pub fn year_yogas_with_states(
    annual_lagna_deg: f64,
    house: House,
    sky: &AnnualSky,
    states: &AnnualStates,
    rules: YogaRules,
) -> Result<YearYogas, Error> {
    judge(annual_lagna_deg, house, sky, Some(states), rules)
}

/// The one judgement every entry point makes, `states` or not.
fn judge(
    annual_lagna_deg: f64,
    house: House,
    sky: &AnnualSky,
    states: Option<&AnnualStates>,
    rules: YogaRules,
) -> Result<YearYogas, Error> {
    let rules = rules.check()?;
    if let Some(states) = states {
        states.validate()?;
    }
    let lagna = lagna_of(annual_lagna_deg)?;
    let sign = house.sign_from(lagna);
    let lagnesha = lagna.attributes().lord;
    let karyesha = sign.attributes().lord;
    let same_lord = lagnesha == karyesha;
    let between = if same_lord {
        None
    } else {
        Some(between_with_rules(lagnesha, karyesha, sky, rules.drishti)?)
    };
    // Ikabala and Induvara are facts about the chart, so they answer
    // every matter alike -- the first house, which has no pair, included.
    let mut held = chart_facts(lagna, sky);
    if let Some(pair) = between {
        // Every pair judgement below that reads strength reads the same
        // seven, so they are computed once for the matter.
        let all = strengths(sky, rules)?;
        if let Some(yoga) = pair.yoga {
            held.push(Held {
                afflictions: None,
                yoga: if yoga.is_ithasala() {
                    YearYoga::Ithasala
                } else {
                    YearYoga::Ishrafa
                },
                between: Some(pair),
                through: None,
                legs: None,
            });
            if yoga.is_ithasala() {
                // The judgements **about** an Ithasala rather than
                // alternatives to it, asked only where one stands -- and
                // holding beside it rather than instead of it. Manau,
                // Khallasara and Rudda say the Ithasala is destroyed or
                // spoilt; the Ithasala is still the configuration that
                // was, and a consumer that saw only the verdict could not
                // say what happened.
                held.extend(upon_the_ithasala(&pair, sky, rules.drishti)?);
                if let Some(states) = states {
                    held.extend(spoilt(
                        &pair,
                        [lagnesha, karyesha],
                        lagna,
                        &all,
                        sky,
                        states,
                    )?);
                }
            }
        } else if !pair.drishti.is_aspect() {
            // Only a pair that does not aspect at all can be reached by a
            // third planet: the source asks for the light to be carried
            // where there is no aspect to carry it.
            held.extend(carried(lagnesha, karyesha, sky, rules.drishti)?);
        }
        // Asked whatever the pair do between themselves: the source
        // conditions these on the pair's weakness, and on nothing they
        // make together.
        held.extend(dutthottha_davira(lagnesha, karyesha, &all, sky, rules)?);
        if let Some(states) = states {
            held.extend(durapha([lagnesha, karyesha], lagna, &all, sky, states)?);
        }
    }
    Ok(YearYogas {
        house,
        sign,
        lagnesha,
        karyesha,
        same_lord,
        between,
        held,
        unanswered: YearYoga::ALL
            .into_iter()
            .filter(|yoga| !yoga.is_built() || (states.is_none() && yoga.needs_states()))
            .collect(),
        states: states.cloned(),
    })
}

/// One of the seven's strength from a set already computed.
fn strength_in(all: &[Strength; 7], graha: Graha) -> Result<Strength, Error> {
    all.iter()
        .copied()
        .find(|one| one.graha == graha)
        .ok_or_else(|| {
            Error::invalid_arg(format!("the annual chart does not place {graha:?}"))
                .with_field(String::from("graha"))
        })
}

/// **Ikabala** and **Induvara**: every one of the seven in a kendra or a
/// panaphara, or every one in an apoklima, by whole signs from the
/// annual lagna.
///
/// The only two of the sixteen that are facts about a chart rather than
/// judgements about a pair, so they carry nothing but their name.
fn chart_facts(lagna: Rashi, sky: &AnnualSky) -> Vec<Held> {
    let houses = SEVEN.map(|graha| house_of(graha, lagna, sky));
    let fact = |yoga| Held {
        afflictions: None,
        yoga,
        between: None,
        through: None,
        legs: None,
    };
    let mut found = Vec::new();
    if houses
        .iter()
        .all(|house| house.is_kendra() || house.is_panaphara())
    {
        found.push(fact(YearYoga::Ikabala));
    }
    if houses.iter().all(|house| house.is_apoklima()) {
        found.push(fact(YearYoga::Induvara));
    }
    found
}

/// **Rudda** and **Duhphali-kuttha**: the two judgements upon an
/// Ithasala that read retrograde and combustion.
///
/// Rudda holds where either of the pair is afflicted at all, and carries
/// both afflictions so the clause that spoilt it can be read. Duhphali-
/// kuttha wants the slower strong and the faster weak "but neither
/// retrograde nor combust" — the one place the source asks for a planet
/// to be **free** of those two, which is why it is judged here beside
/// the yoga that asks for them.
fn spoilt(
    pair: &Between,
    lords: [Graha; 2],
    lagna: Rashi,
    all: &[Strength; 7],
    sky: &AnnualSky,
    states: &AnnualStates,
) -> Result<Vec<Held>, Error> {
    let upon = |yoga, afflictions| Held {
        afflictions,
        yoga,
        between: Some(*pair),
        through: None,
        legs: None,
    };
    let mut found = Vec::new();
    let afflictions = lords.map(|graha| afflicted(graha, lagna, sky, states));
    if afflictions.iter().any(|one| one.is_afflicted()) {
        found.push(upon(YearYoga::Rudda, Some(afflictions)));
    }
    let (slower, faster) = (
        strength_in(all, pair.slower)?,
        strength_in(all, pair.faster)?,
    );
    let free = !states.is_retrograde(pair.faster) && !states.is_combust(pair.faster);
    if slower.is_strong() && faster.is_weak() && free {
        found.push(upon(YearYoga::DuhphaliKuttha, None));
    }
    Ok(found)
}

/// **Durapha**: both lords weak, and each "in the trika houses, combust
/// or retrograde".
///
/// Read as a list of alternatives each planet must meet one of, as
/// Rudda's longer list plainly is (crux C119). Whatever the reading, it
/// can hold only where both lords are weak, which the corpus finds in a
/// handful of matters in twenty-two thousand
/// (`03-design/muntha-measured.md` §11) — so the crux is bounded by
/// that ceiling, and the pass checks it.
fn durapha(
    lords: [Graha; 2],
    lagna: Rashi,
    all: &[Strength; 7],
    sky: &AnnualSky,
    states: &AnnualStates,
) -> Result<Option<Held>, Error> {
    for graha in lords {
        if !strength_in(all, graha)?.is_weak() {
            return Ok(None);
        }
    }
    let afflictions = lords.map(|graha| afflicted(graha, lagna, sky, states));
    let marked = afflictions
        .iter()
        .all(|one| one.trika || one.combust || one.retrograde);
    Ok(marked.then_some(Held {
        afflictions: Some(afflictions),
        yoga: YearYoga::Durapha,
        between: None,
        through: None,
        legs: None,
    }))
}

/// The three yogas that judge an **Ithasala** the pair already makes:
/// Manau, which a malefic destroys; Kamboola, which the Moon joins; and
/// Khallasara, which an unqualified Moon negates by standing apart.
///
/// All three are asked of the same Ithasala and none excludes another,
/// so they are found in one pass over the third planets that matter.
fn upon_the_ithasala(
    pair: &Between,
    sky: &AnnualSky,
    rules: DrishtiRules,
) -> Result<Vec<Held>, Error> {
    let sign_of = |graha: Graha| sign_of_longitude(sky.longitude_of(graha));
    let (fast, slow) = (pair.faster, pair.slower);
    let mut found = Vec::new();

    // Manau. The source asks for a malefic "conjunct or inimically
    // aspecting the faster", which is one condition and not two: a planet
    // sharing a sign stands at house 1, and house 1 is inimical in
    // Tajika. A malefic that is itself one of the pair is not a third
    // planet and does not destroy its own Ithasala.
    for malefic in MALEFICS {
        if malefic == fast || malefic == slow {
            continue;
        }
        if !malefic_reaches(sign_of(malefic), sign_of(fast)) {
            continue;
        }
        found.push(Held {
            afflictions: None,
            yoga: YearYoga::Manau,
            between: Some(*pair),
            through: Some(malefic),
            legs: Some([
                between_with_rules(malefic, fast, sky, rules)?,
                between_with_rules(malefic, slow, sky, rules)?,
            ]),
        });
    }

    // The Moon's two: it joins the Ithasala, or it stands wholly apart
    // from it while unqualified. A Moon that is one of the pair is
    // neither -- it cannot join what it already is.
    if Graha::Moon != fast && Graha::Moon != slow {
        let legs = [
            between_with_rules(Graha::Moon, fast, sky, rules)?,
            between_with_rules(Graha::Moon, slow, sky, rules)?,
        ];
        let joins = legs
            .iter()
            .any(|leg| leg.yoga.is_some_and(crate::Yoga::is_ithasala));
        if joins {
            found.push(Held {
                afflictions: None,
                yoga: YearYoga::Kamboola,
                between: Some(*pair),
                through: Some(Graha::Moon),
                legs: Some(legs),
            });
        }
        // Khallasara wants the Moon neither aspecting nor sharing a sign
        // with either -- one question in Tajika, where a shared sign is
        // an aspect -- and unqualified besides.
        let apart = legs.iter().all(|leg| !leg.drishti.is_aspect());
        if apart && qualification(Graha::Moon, sky)?.is_unqualified() {
            found.push(Held {
                afflictions: None,
                yoga: YearYoga::Khallasara,
                between: Some(*pair),
                through: Some(Graha::Moon),
                legs: Some(legs),
            });
        }
    }
    Ok(found)
}

/// Nakta and Yamaya: the light carried or gathered by a third planet.
///
/// Both ask the same question of every other planet and differ only in
/// which way its speed must fall, so they are found in one pass rather
/// than two that would each walk the seven.
fn carried(
    lagnesha: Graha,
    karyesha: Graha,
    sky: &AnnualSky,
    rules: DrishtiRules,
) -> Result<Vec<Held>, Error> {
    let rank = |graha: Graha| speed_rank(graha).unwrap_or(usize::MAX);
    let (of_lagna, of_karya) = (rank(lagnesha), rank(karyesha));
    let mut found = Vec::new();
    for third in SEVEN {
        if third == lagnesha || third == karyesha {
            continue;
        }
        // The source measures the third planet's reach by **its own**
        // deeptamsha and not by the mean it would share with each.
        let Some(orb_deg) = deeptamsha(third) else {
            continue;
        };
        let here = rank(third);
        let (faster_than_both, slower_than_both) = (
            here < of_lagna && here < of_karya,
            here > of_lagna && here > of_karya,
        );
        if !faster_than_both && !slower_than_both {
            continue;
        }
        let to_lagna = between_within(third, lagnesha, sky, orb_deg, rules)?;
        let to_karya = between_within(third, karyesha, sky, orb_deg, rules)?;
        let (Some(first), Some(second)) = (to_lagna.yoga, to_karya.yoga) else {
            continue;
        };
        let yoga = if faster_than_both {
            // Translation: past one and coming to the other, so exactly
            // one of the two legs is an Ishrafa.
            if first.is_ithasala() == second.is_ithasala() {
                continue;
            }
            YearYoga::Nakta
        } else {
            // Collection: both of the pair are coming to it, so neither
            // leg may be an Ishrafa.
            if !first.is_ithasala() || !second.is_ithasala() {
                continue;
            }
            YearYoga::Yamaya
        };
        found.push(Held {
            afflictions: None,
            yoga,
            between: None,
            through: Some(third),
            legs: Some([to_lagna, to_karya]),
        });
    }
    Ok(found)
}

/// **Dutthottha-Davira**: the pair both weak, and one of them drawn
/// into an Ithasala by a third planet that is strong.
///
/// Not a judgement *upon* the pair's own Ithasala the way Manau and
/// Khallasara are — the source conditions it on their weakness and on
/// the third planet, and on nothing the two make together — so it is
/// asked of every matter whose lords are distinct.
///
/// The source's "another strong planet, exalted or in its own house"
/// enumerates the same three alternatives [`Strength`] carries, so the
/// clause is `is_strong` and not a fourth reading of the words.
fn dutthottha_davira(
    lagnesha: Graha,
    karyesha: Graha,
    all: &[Strength; 7],
    sky: &AnnualSky,
    rules: YogaRules,
) -> Result<Vec<Held>, Error> {
    let of = |graha: Graha| strength_in(all, graha);
    // Both lords **weak** is the gate -- not merely short of strong: a
    // middling pair is not the case the source is describing, whatever
    // a third planet does about it. Asked positively, because with a
    // band between the floors `!is_strong` and `is_weak` differ.
    if !(of(lagnesha)?.is_weak() && of(karyesha)?.is_weak()) {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for third in all.iter().copied() {
        if third.graha == lagnesha || third.graha == karyesha || !third.is_strong() {
            continue;
        }
        let legs = [
            between_with_rules(third.graha, lagnesha, sky, rules.drishti)?,
            between_with_rules(third.graha, karyesha, sky, rules.drishti)?,
        ];
        if legs
            .iter()
            .any(|leg| leg.yoga.is_some_and(crate::Yoga::is_ithasala))
        {
            found.push(Held {
                afflictions: None,
                yoga: YearYoga::DutthotthaDavira,
                between: None,
                through: Some(third.graha),
                legs: Some(legs),
            });
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{
        Affliction, Held, MALEFICS, Strength, YOGA_STRONG_FROM, YOGA_WEAK_BELOW, YearYoga,
        YearYogas, YogaRules, affliction, qualification, strength, strength_with_rules, year_yogas,
        year_yogas_with_rules, year_yogas_with_states,
    };
    use crate::bala::{AnnualSky, Bala};
    use crate::drishti::{DrishtiRules, SubDegree, Yoga};
    use crate::states::AnnualStates;
    use teistro_core::catalogue::{Graha, Rashi};
    use teistro_core::house::House;

    /// The source's own worked annual chart (K.S. Charak, ch. III): Scorpio
    /// rising, the seven where its Table places them.
    fn worked() -> AnnualSky {
        let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
        AnnualSky {
            sun_deg: at(4.0, 3.0, 50.0),
            moon_deg: at(1.0, 9.0, 40.0),
            mars_deg: at(7.0, 7.0, 42.0),
            mercury_deg: at(4.0, 18.0, 20.0),
            jupiter_deg: at(8.0, 9.0, 38.0),
            venus_deg: at(4.0, 21.0, 45.0),
            saturn_deg: at(6.0, 17.0, 13.0),
        }
    }

    /// Scorpio 9°26′, the annual lagna of the source's worked year.
    const WORKED_LAGNA_DEG: f64 = 7.0 * 30.0 + 9.0 + 26.0 / 60.0;

    /// A sky with **two weak lords**, found by a deterministic sweep
    /// rather than composed: Mercury and Venus both fall below the
    /// floor, and the Sun and Jupiter are strong and reach them.
    ///
    /// Kept as literals so the test is a fixture and not a search.
    const WEAK_PAIR: AnnualSky = AnnualSky {
        sun_deg: 290.860_054,
        moon_deg: 143.097_181,
        mars_deg: 324.878_876,
        mercury_deg: 344.462_371,
        jupiter_deg: 265.131_919,
        venus_deg: 170.583_816,
        saturn_deg: 210.093_248,
    };

    /// The annual lagna that makes Mercury the lagnesha and, in the
    /// second house, Venus the karyesha.
    const WEAK_PAIR_LAGNA_DEG: f64 = 170.583_429;

    fn house(number: u8) -> House {
        House::try_new(number).unwrap()
    }

    fn asked(number: u8) -> YearYogas {
        year_yogas(WORKED_LAGNA_DEG, house(number), &worked()).unwrap()
    }

    /// The lagnesha is the lord of the annual lagna and nothing else; the
    /// karyesha is the lord of the house asked about, counted from it by
    /// whole signs.
    #[test]
    fn the_pair_is_the_lagna_lord_and_the_matter_lord() {
        // Scorpio rising: Mars is the lagnesha for every question.
        for number in 1..=12u8 {
            assert_eq!(asked(number).lagnesha, Graha::Mars);
        }
        // The seventh from Scorpio is Taurus, whose lord is Venus.
        let marriage = asked(7);
        assert_eq!(marriage.sign, Rashi::Taurus);
        assert_eq!(marriage.karyesha, Graha::Venus);
        assert_eq!(marriage.house.get(), 7);
        // The tenth from Scorpio is Leo, whose lord is the Sun.
        assert_eq!(asked(10).karyesha, Graha::Sun);
    }

    /// One planet is both lords for exactly one house of the twelve, and
    /// the pair yogas have nothing to judge there.
    #[test]
    fn one_planet_may_be_both_lords_and_that_is_reported() {
        // The first house is the lagna itself, so its lord is always the
        // lagnesha: a question about the self is never about a pair.
        let self_ = asked(1);
        assert_eq!(self_.karyesha, Graha::Mars);
        assert!(self_.same_lord);
        assert_eq!(self_.between, None, "a planet makes no yoga with itself");
        assert!(self_.held.is_empty());
        // Mars rules Scorpio and Aries too, and Aries is the sixth from
        // Scorpio -- so two of the twelve are like that under this lagna.
        let same = asked(6);
        assert_eq!(same.karyesha, Graha::Mars);
        assert!(same.same_lord);
        let both = (1..=12u8).filter(|number| asked(*number).same_lord).count();
        assert_eq!(both, 2, "the first, and the lagnesha's other sign");
        // Under a lagna one of the luminaries rules, only the first is:
        // the Sun rules Leo alone and the Moon Cancer alone.
        for lagna_deg in [4.0 * 30.0 + 10.0, 3.0 * 30.0 + 10.0] {
            let alone = (1..=12u8)
                .filter(|number| {
                    year_yogas(lagna_deg, house(*number), &worked())
                        .unwrap()
                        .same_lord
                })
                .count();
            assert_eq!(alone, 1, "only the first house");
        }
    }

    /// A pair that aspects and stands inside its orb makes the Ithasala or
    /// the Ishrafa, and the answer carries which kind.
    #[test]
    fn the_pair_makes_its_own_yoga_and_says_which_kind() {
        // The tenth: Mars and the Sun, the source's own worked pair. The
        // Sun is faster and behind, so they are coming together.
        let found = asked(10);
        assert_eq!(found.karyesha, Graha::Sun);
        assert_eq!(found.holds(YearYoga::Ithasala), Some(true));
        assert_eq!(found.holds(YearYoga::Ishrafa), Some(false));
        let held = &found.held[0];
        assert_eq!(held.yoga, YearYoga::Ithasala);
        assert_eq!(
            held.between.unwrap().yoga,
            Some(Yoga::IthasalaVartamana),
            "and by more than a degree"
        );
        assert_eq!(held.through, None, "no third planet carried it");
    }

    /// A yoga this call cannot answer for is **not** reported as absent:
    /// `holds` says nothing, and `why` says which of two reasons applies.
    #[test]
    fn an_unanswered_yoga_says_why_rather_than_reading_as_absent() {
        // Without states: the three the build does not compute, and the
        // three it could answer had it been given retrograde and combust.
        let found = asked(7);
        assert_eq!(found.states, None);
        assert_eq!(found.unanswered.len(), 6);
        for yoga in &found.unanswered {
            assert_eq!(found.holds(*yoga), None);
            let why = found.why(*yoga).unwrap();
            if yoga.needs_states() {
                assert!(yoga.is_built(), "{yoga:?}");
                assert_eq!(yoga.awaiting(), None);
                assert!(why.contains("not given"), "{yoga:?}: {why}");
            } else {
                assert_eq!(Some(why), yoga.awaiting(), "{yoga:?}");
            }
        }
        // An answered yoga has no reason to give.
        assert_eq!(found.why(YearYoga::Ithasala), None);

        // With states: only the build's own three remain, and the built
        // and the unanswered are the sixteen.
        let states = AnnualStates::default();
        let full = year_yogas_with_states(
            WORKED_LAGNA_DEG,
            house(7),
            &worked(),
            &states,
            YogaRules::default(),
        )
        .unwrap();
        assert_eq!(full.states.as_ref(), Some(&states));
        assert_eq!(full.unanswered.len(), 3);
        let built = YearYoga::ALL.into_iter().filter(|one| one.is_built());
        assert_eq!(built.count() + full.unanswered.len(), YearYoga::ALL.len());
        for yoga in YearYoga::ALL.into_iter().filter(|one| one.needs_states()) {
            assert!(
                full.holds(yoga).is_some(),
                "{yoga:?} is answered with states"
            );
        }
    }

    /// Nakta: the two do not aspect, and a planet faster than both is past
    /// one and coming to the other, within **its own** deeptamsha.
    #[test]
    fn nakta_carries_the_light_between_two_that_do_not_aspect() {
        // Mars at Scorpio 10° and Jupiter at Sagittarius 20°: the second
        // house from the first, which is no aspect at all. The Moon at
        // Leo 14° is faster than both, aspects Scorpio (from the tenth)
        // and Sagittarius (from the ninth), is 4° past Mars within its
        // sign and 6° behind Jupiter -- both inside its own 12°.
        let sky = AnnualSky {
            mars_deg: 7.0 * 30.0 + 10.0,
            jupiter_deg: 8.0 * 30.0 + 20.0,
            moon_deg: 4.0 * 30.0 + 14.0,
            ..worked()
        };
        // Scorpio rising, the second house: Sagittarius, ruled by Jupiter.
        let found = year_yogas(WORKED_LAGNA_DEG, house(2), &sky).unwrap();
        assert_eq!(
            (found.lagnesha, found.karyesha),
            (Graha::Mars, Graha::Jupiter)
        );
        assert!(!found.between.unwrap().drishti.is_aspect(), "no aspect");
        assert_eq!(found.holds(YearYoga::Nakta), Some(true));
        let nakta: &Held = found
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Nakta)
            .unwrap();
        assert_eq!(nakta.through, Some(Graha::Moon));
        let legs = nakta.legs.unwrap();
        // Exactly one leg is an Ishrafa: past one, coming to the other.
        let coming = legs.iter().filter(|leg| leg.yoga.unwrap().is_ithasala());
        assert_eq!(coming.count(), 1);
    }

    /// Yamaya: the intermediary is slower than both and both come to it.
    #[test]
    fn yamaya_gathers_the_light_of_two_that_do_not_aspect() {
        // The Moon at Aries 10° and Venus at Taurus 12° do not aspect --
        // the second house again. Saturn at Cancer 16° is slower than
        // both, aspects Aries (from the fourth) and Taurus (from the
        // third), and stands 6° ahead of the Moon and 4° ahead of Venus
        // within their signs, so both are coming to it inside Saturn's
        // own 9°.
        let sky = AnnualSky {
            moon_deg: 10.0,
            venus_deg: 30.0 + 12.0,
            saturn_deg: 3.0 * 30.0 + 16.0,
            ..worked()
        };
        // Aries rising makes Mars the lagnesha, so ask under Cancer
        // rising instead: the Moon rules the lagna and Venus the fifth
        // (Scorpio is the fifth... ) -- use Taurus rising, where Venus is
        // the lagnesha and the Moon rules the third, Cancer.
        let taurus = 30.0 + 5.0;
        let found = year_yogas(taurus, house(3), &sky).unwrap();
        assert_eq!(
            (found.lagnesha, found.karyesha),
            (Graha::Venus, Graha::Moon)
        );
        assert!(!found.between.unwrap().drishti.is_aspect(), "no aspect");
        assert_eq!(found.holds(YearYoga::Yamaya), Some(true));
        let yamaya = found
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Yamaya)
            .unwrap();
        assert_eq!(yamaya.through, Some(Graha::Saturn));
        // Neither leg is an Ishrafa: both of the pair are coming to it.
        assert!(
            yamaya
                .legs
                .unwrap()
                .iter()
                .all(|leg| leg.yoga.unwrap().is_ithasala())
        );
    }

    /// A pair that aspects is never reached by a third planet: the source
    /// asks for the light to be carried only where there is no aspect.
    #[test]
    fn a_pair_that_aspects_is_never_carried() {
        for number in 1..=12u8 {
            let found = asked(number);
            let Some(pair) = found.between else { continue };
            if pair.drishti.is_aspect() {
                assert!(
                    found.held.iter().all(|one| one.through.is_none()),
                    "house {number} aspects and was carried anyway"
                );
            }
        }
    }

    /// Manau: a malefic inimically aspecting the faster of a pair that is
    /// in Ithasala destroys it — and "conjunct or inimically aspecting"
    /// is one condition, because a shared sign is house 1 and house 1 is
    /// inimical in Tajika.
    #[test]
    fn manau_is_a_malefic_upon_the_faster_of_an_ithasala() {
        // Scorpio rising. The tenth is Leo, so the Sun is the karyesha
        // and Mars the lagnesha; the Sun is faster and behind, so they
        // are in Ithasala. Put Saturn in Leo with the Sun: a shared sign,
        // which is house 1 and inimical.
        let sky = AnnualSky {
            saturn_deg: 4.0 * 30.0 + 20.0,
            ..worked()
        };
        let found = year_yogas(WORKED_LAGNA_DEG, house(10), &sky).unwrap();
        assert_eq!(found.holds(YearYoga::Ithasala), Some(true));
        assert_eq!(found.holds(YearYoga::Manau), Some(true));
        let manau = found
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Manau)
            .unwrap();
        assert_eq!(manau.through, Some(Graha::Saturn));
        // The Ithasala is still reported beside it: Manau says what was
        // destroyed, and the thing destroyed is part of the answer.
        assert!(found.held.iter().any(|one| one.yoga == YearYoga::Ithasala));

        // Move Saturn to Virgo, the second from Leo, which aspects
        // nothing at all: no Manau, and the Ithasala stands.
        let clear = AnnualSky {
            saturn_deg: 5.0 * 30.0 + 20.0,
            ..worked()
        };
        let stands = year_yogas(WORKED_LAGNA_DEG, house(10), &clear).unwrap();
        assert_eq!(stands.holds(YearYoga::Ithasala), Some(true));
        assert_eq!(stands.holds(YearYoga::Manau), Some(false));
    }

    /// A malefic that is itself one of the pair does not destroy its own
    /// Ithasala: Manau asks for a **third** planet.
    #[test]
    fn a_malefic_of_the_pair_does_not_destroy_its_own_ithasala() {
        // Mars is the lagnesha under Scorpio and is one of the two
        // malefics, so every Ithasala it makes would be a Manau if the
        // pair counted as its own third planet.
        let found = asked(10);
        assert_eq!(found.lagnesha, Graha::Mars);
        assert_eq!(found.holds(YearYoga::Ithasala), Some(true));
        let by_pair = found
            .held
            .iter()
            .filter(|one| one.yoga == YearYoga::Manau)
            .any(|one| one.through == Some(Graha::Mars));
        assert!(!by_pair, "the lagnesha is not a third planet");
    }

    /// Kamboola: the Moon joins the pair's Ithasala by making one with
    /// either of them.
    #[test]
    fn kamboola_is_the_moon_joining_the_ithasala() {
        // Mars in Scorpio 7°42′ and the Sun in Leo 3°50′, as the source
        // has them. Put the Moon at Aquarius 1°: it aspects Leo from the
        // seventh and Scorpio from the tenth, and is behind both within
        // their signs, so it comes to each.
        let sky = AnnualSky {
            moon_deg: 10.0 * 30.0 + 1.0,
            ..worked()
        };
        let found = year_yogas(WORKED_LAGNA_DEG, house(10), &sky).unwrap();
        assert_eq!(found.holds(YearYoga::Ithasala), Some(true));
        assert_eq!(found.holds(YearYoga::Kamboola), Some(true));
        let joined = found
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Kamboola)
            .unwrap();
        assert_eq!(joined.through, Some(Graha::Moon));
        assert!(
            joined
                .legs
                .unwrap()
                .iter()
                .any(|leg| leg.yoga.is_some_and(crate::Yoga::is_ithasala)),
            "the Moon comes to at least one of them"
        );
    }

    /// The source's own definition of **unqualified**, clause by clause,
    /// and each clause shown to move the verdict on its own.
    #[test]
    fn unqualified_is_every_clause_false() {
        // The Moon in Taurus 3° is exalted, which disqualifies it
        // whatever else is true.
        let exalted = AnnualSky {
            moon_deg: 30.0 + 3.0,
            ..worked()
        };
        let found = qualification(Graha::Moon, &exalted).unwrap();
        assert!(found.exalted && !found.debilitated);
        assert!(!found.is_unqualified());
        // In Scorpio 3° it is debilitated instead.
        let fallen = AnnualSky {
            moon_deg: 7.0 * 30.0 + 3.0,
            ..worked()
        };
        assert!(qualification(Graha::Moon, &fallen).unwrap().debilitated);
        // A body outside the seven is refused by name.
        let refused = qualification(Graha::Rahu, &worked()).unwrap_err();
        assert_eq!(refused.field(), Some("graha"));
    }

    /// Khallasara: an unqualified Moon standing wholly apart from an
    /// Ithasala negates it.
    ///
    /// **The source's chart cannot make one, and no chart like it can.**
    /// Tajika counts eight of the twelve sign relations as an aspect, so
    /// a Moon "not aspected or associated" needs all six of the others
    /// inside the four neutral houses at once — and the worked chart has
    /// three planets in Leo with Mars and Jupiter in the signs either
    /// side, whose spacing no single sign can be neutral to. So the sky
    /// here is built to satisfy the definition rather than taken from
    /// the source, and `03-design/muntha-measured.md` §10 counts how
    /// often the corpus manages it.
    #[test]
    fn khallasara_needs_the_moon_unqualified_and_wholly_apart() {
        // Scorpio rising, the tenth: Mars the lagnesha in Scorpio and
        // the Sun the karyesha in Virgo, three signs apart and so
        // aspecting, the Sun faster and 5° behind within their signs.
        // Every other planet is put in one of the four houses neutral to
        // Aries — the 2nd, 6th, 8th and 12th — so a Moon in Aries is
        // aspected by nothing.
        let base = AnnualSky {
            sun_deg: 5.0 * 30.0 + 5.0,
            mars_deg: 7.0 * 30.0 + 10.0,
            mercury_deg: 30.0 + 12.0,
            jupiter_deg: 5.0 * 30.0 + 20.0,
            venus_deg: 11.0 * 30.0 + 8.0,
            saturn_deg: 7.0 * 30.0 + 25.0,
            moon_deg: 0.0,
        };
        // Within Aries the Moon must also fall in no division it rules,
        // so the degree is searched rather than guessed.
        let mut standing = None;
        for tenth in 0..300 {
            let moon_deg = f64::from(tenth) / 10.0;
            let sky = AnnualSky { moon_deg, ..base };
            let found = year_yogas(WORKED_LAGNA_DEG, house(10), &sky).unwrap();
            if found.holds(YearYoga::Khallasara) == Some(true) {
                standing = Some((moon_deg, found));
                break;
            }
        }
        let (moon_deg, found) =
            standing.expect("a degree of Aries at which the Moon rules nothing");
        let sky = AnnualSky { moon_deg, ..base };

        // The Ithasala it negates is there, and reported beside it.
        assert_eq!(found.holds(YearYoga::Ithasala), Some(true));
        // Every clause of the source's definition is false.
        let how = qualification(Graha::Moon, &sky).unwrap();
        assert!(how.is_unqualified());
        assert!(!how.exalted && !how.debilitated && !how.aspected);
        assert!(!how.own_hudda && !how.own_drekkana && !how.own_navamsha);
        // And it stands apart from both of the pair, which in Tajika is
        // one question and not two.
        let khalla = found
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Khallasara)
            .unwrap();
        assert!(
            khalla
                .legs
                .unwrap()
                .iter()
                .all(|leg| !leg.drishti.is_aspect())
        );
        // Khallasara and Kamboola are contraries: the Moon cannot both
        // join the Ithasala and stand wholly apart from it.
        assert_eq!(found.holds(YearYoga::Kamboola), Some(false));

        // Exalt the Moon where it stands and the yoga goes, though
        // nothing about its distance from the pair has changed: the
        // clause is doing the work and not the geometry.
        let exalted = AnnualSky {
            moon_deg: 30.0 + 3.0,
            ..base
        };
        let gone = year_yogas(WORKED_LAGNA_DEG, house(10), &exalted).unwrap();
        assert!(qualification(Graha::Moon, &exalted).unwrap().exalted);
        assert_eq!(gone.holds(YearYoga::Khallasara), Some(false));
    }

    /// None of the three is ever asked where the pair makes no Ithasala:
    /// all three are judgements **about** one.
    #[test]
    fn the_three_are_asked_only_of_an_ithasala() {
        for number in 1..=12u8 {
            let found = asked(number);
            let has = found.holds(YearYoga::Ithasala) == Some(true);
            for yoga in [YearYoga::Manau, YearYoga::Kamboola, YearYoga::Khallasara] {
                if !has {
                    assert_eq!(found.holds(yoga), Some(false), "house {number}: {yoga:?}");
                }
            }
        }
    }

    /// Tajika's malefics are two, and deliberately not the catalogue's,
    /// which carries the Sun as well (crux C114).
    #[test]
    fn tajikas_malefics_are_two_and_not_the_catalogues() {
        assert_eq!(MALEFICS, [Graha::Mars, Graha::Saturn]);
        assert!(
            !MALEFICS.contains(&Graha::Sun),
            "the Sun is a malefic in the catalogue and not in this yoga"
        );
    }

    /// A lagna that is not a number is refused by name, before anything is
    /// looked up.
    #[test]
    fn a_lagna_that_is_not_a_number_is_refused_by_name() {
        let refused = year_yogas(f64::NAN, house(1), &worked()).unwrap_err();
        assert_eq!(refused.field(), Some("annual_lagna_deg"));
    }

    /// The readings the source leaves open reach this module too: they
    /// decide the pair's own band exactly as they do a bare pair's.
    #[test]
    fn the_readings_reach_the_pair() {
        // Mars at Scorpio 10°30′ and the Sun at Leo 10°: the Sun is the
        // faster and half a degree past, which is the contested band.
        let sky = AnnualSky {
            mars_deg: 7.0 * 30.0 + 10.5,
            sun_deg: 4.0 * 30.0 + 11.0,
            ..worked()
        };
        let under = |sub_degree| {
            year_yogas_with_rules(
                WORKED_LAGNA_DEG,
                house(10),
                &sky,
                DrishtiRules { sub_degree }.into(),
            )
            .unwrap()
        };
        assert!(under(SubDegree::Poorna).between.unwrap().disputed());
        assert_eq!(
            under(SubDegree::Poorna).held[0].between.unwrap().yoga,
            Some(Yoga::IthasalaPoorna)
        );
        assert_eq!(
            under(SubDegree::Ishrafa).held[0].between.unwrap().yoga,
            Some(Yoga::Ishrafa)
        );
        assert!(under(SubDegree::None).held.is_empty());
    }

    /// Rules with both floors at `units`.
    fn floors(weak_below: i64, strong_from: i64) -> YogaRules {
        YogaRules {
            weak_below: Bala::new(weak_below, 0, 0),
            strong_from: Bala::new(strong_from, 0, 0),
            ..YogaRules::default()
        }
    }

    /// The weak floor carries the year lord's number and answers a
    /// different question; the strong floor is the graded scale's.
    ///
    /// The two weak constants are written out separately on purpose
    /// (crux C116): this asserts they still agree, so parting them has
    /// to be done deliberately rather than by editing one and moving
    /// both.
    #[test]
    fn the_default_floors_are_the_two_sources_numbers() {
        assert_eq!(YOGA_WEAK_BELOW, crate::WEAK_BELOW);
        assert_eq!(YOGA_WEAK_BELOW, Bala::new(5, 0, 0));
        assert_eq!(YOGA_STRONG_FROM, Bala::new(10, 0, 0));
        let rules = YogaRules::default();
        assert_eq!(
            (rules.weak_below, rules.strong_from),
            (YOGA_WEAK_BELOW, YOGA_STRONG_FROM)
        );
        assert_eq!(rules.check(), Ok(rules));
    }

    /// The worked chart has **no weak planet** and one middling one.
    ///
    /// Worth asserting rather than assuming: on a scale of twenty, a
    /// planet under five with no dignity is rare, which is the measured
    /// half of crux C116. Venus, with no dignity and five units and a
    /// little over, is the planet the middle band exists for.
    #[test]
    fn the_worked_chart_has_no_weak_planet_and_one_middling() {
        let sky = worked();
        for graha in crate::SEVEN {
            let how = strength(graha, &sky).unwrap();
            assert!(!how.is_weak(), "{graha:?} is weak in the worked chart");
            assert_eq!(how.is_middling(), graha == Graha::Venus, "{graha:?}");
        }
        // Each clause answers for somebody, so none of the three is dead.
        assert!(strength(Graha::Moon, &sky).unwrap().exalted);
        assert!(strength(Graha::Sun, &sky).unwrap().own_sign);
        let mercury = strength(Graha::Mercury, &sky).unwrap();
        assert!(!mercury.exalted && !mercury.own_sign && mercury.is_strong());
        let venus = strength(Graha::Venus, &sky).unwrap();
        assert!(venus.vishwa >= Bala::new(5, 0, 0) && venus.vishwa < Bala::new(6, 0, 0));
    }

    /// Each floor moves its own verdict and only that one.
    #[test]
    fn each_floor_is_a_knob() {
        let sky = worked();
        let venus = |rules| strength_with_rules(Graha::Venus, &sky, rules).unwrap();
        // Venus sits a little over five: middling by default, weak once
        // the lower floor passes it, strong once the upper one reaches
        // down to it.
        assert!(venus(floors(5, 10)).is_middling());
        assert!(venus(floors(6, 10)).is_weak());
        assert!(venus(floors(5, 5)).is_strong());
        // Raised to the top of the scale, the bala clause can never hold
        // and only the dignities keep anyone strong -- the clearest
        // demonstration that the three really are alternatives.
        for graha in [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Jupiter,
            Graha::Saturn,
        ] {
            assert!(
                strength_with_rules(graha, &sky, floors(5, 20))
                    .unwrap()
                    .is_strong()
            );
        }
        for graha in [Graha::Mercury, Graha::Venus] {
            assert!(
                !strength_with_rules(graha, &sky, floors(5, 20))
                    .unwrap()
                    .is_strong()
            );
        }
        // The floors a reading was taken under travel with it.
        let read = venus(floors(6, 12));
        assert_eq!(
            (read.weak_below, read.strong_from),
            (Bala::new(6, 0, 0), Bala::new(12, 0, 0))
        );
    }

    /// Equal floors are the reading with no middle: every planet is one
    /// or the other.
    #[test]
    fn equal_floors_leave_no_middle() {
        for units in [4, 5, 8, 10] {
            for graha in crate::SEVEN {
                let how = strength_with_rules(graha, &worked(), floors(units, units)).unwrap();
                assert!(!how.is_middling(), "{graha:?} at {units}");
                assert_ne!(how.is_strong(), how.is_weak(), "{graha:?} at {units}");
            }
        }
    }

    /// Floors that cross are refused, and refused on **every** call --
    /// including a first-house matter, whose one lord asks nothing about
    /// strength -- rather than only where a yoga happens to read them.
    #[test]
    fn floors_that_cross_are_refused() {
        let crossed = floors(10, 5);
        let refused = crossed.check().unwrap_err();
        assert_eq!(refused.field(), Some("strong_from"));
        let refused = strength_with_rules(Graha::Sun, &worked(), crossed).unwrap_err();
        assert_eq!(refused.field(), Some("strong_from"));
        let refused =
            year_yogas_with_rules(WORKED_LAGNA_DEG, house(1), &worked(), crossed).unwrap_err();
        assert_eq!(refused.field(), Some("strong_from"));
    }

    /// The clauses are named and ordered, as `Qualification`'s are, and
    /// the three verdicts partition every planet.
    #[test]
    fn strength_clauses_are_named_and_the_verdicts_partition() {
        let names: Vec<&str> = strength(Graha::Moon, &worked())
            .unwrap()
            .clauses()
            .iter()
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            names,
            [
                "exalted",
                "in a sign it rules",
                "fully strong in Vishwa bala"
            ]
        );
        for graha in crate::SEVEN {
            let how: Strength = strength(graha, &WEAK_PAIR).unwrap();
            let verdicts = [how.is_strong(), how.is_middling(), how.is_weak()];
            assert_eq!(verdicts.iter().filter(|is| **is).count(), 1, "{graha:?}");
        }
    }

    #[test]
    fn strength_refuses_a_body_outside_the_seven() {
        let refused = strength(Graha::Ketu, &worked()).unwrap_err();
        assert_eq!(refused.field(), Some("graha"));
    }

    /// Dutthottha-Davira: both lords weak, and a strong third planet
    /// drawing one of them into an Ithasala.
    ///
    /// **Found by search, like Khallasara's sky and for the same
    /// reason.** Weak means a Panchavargiya total under twenty of eighty,
    /// and four hostile divisional lords already cost fifteen of that: so
    /// a weak planet needs its sign's lord hostile, the others hostile or
    /// nearly, and a place close to its own debilitation, all at once.
    /// Two weak lords together is rare: a deterministic sweep of skies
    /// met it on the 1 122nd. The source's own worked chart cannot serve -- it
    /// has no weak planet at all.
    #[test]
    fn dutthottha_davira_wants_both_lords_weak_and_a_strong_third() {
        let sky = WEAK_PAIR;
        let found = year_yogas(WEAK_PAIR_LAGNA_DEG, house(2), &sky).unwrap();
        assert_eq!(found.lagnesha, Graha::Mercury);
        assert_eq!(found.karyesha, Graha::Venus);
        assert!(strength(found.lagnesha, &sky).unwrap().is_weak());
        assert!(strength(found.karyesha, &sky).unwrap().is_weak());
        assert_eq!(found.holds(YearYoga::DutthotthaDavira), Some(true));

        // Every one that held names a third planet that is strong, and
        // reaches one of the pair by Ithasala.
        let held: Vec<&Held> = found
            .held
            .iter()
            .filter(|one| one.yoga == YearYoga::DutthotthaDavira)
            .collect();
        assert!(!held.is_empty());
        for one in held {
            let third = one.through.unwrap();
            assert_ne!(third, found.lagnesha);
            assert_ne!(third, found.karyesha);
            assert!(strength(third, &sky).unwrap().is_strong());
            assert!(
                one.legs
                    .unwrap()
                    .iter()
                    .any(|leg| leg.yoga.is_some_and(crate::Yoga::is_ithasala))
            );
        }
    }

    /// The upper floor selects **which** third planet qualifies.
    ///
    /// Raised to the top of the scale, the Sun -- strong here only by
    /// its bala -- drops out, while Jupiter, standing in a sign it
    /// rules, is untouched by any floor at all. Nothing in the sky
    /// moved; the reading did.
    #[test]
    fn the_upper_floor_selects_which_third_planet_qualifies() {
        let through = |strong_from: i64| -> Vec<Graha> {
            year_yogas_with_rules(
                WEAK_PAIR_LAGNA_DEG,
                house(2),
                &WEAK_PAIR,
                floors(5, strong_from),
            )
            .unwrap()
            .held
            .iter()
            .filter(|one| one.yoga == YearYoga::DutthotthaDavira)
            .filter_map(|one| one.through)
            .collect()
        };
        assert_eq!(through(10), vec![Graha::Sun, Graha::Jupiter]);
        assert_eq!(through(20), vec![Graha::Jupiter]);
    }

    /// And it wants the pair **weak**, not merely short of strong: lower
    /// the weak floor under the two lords and they become middling, and
    /// the yoga goes though the third planets are untouched.
    #[test]
    fn dutthottha_davira_passes_a_middling_pair_by() {
        let rules = floors(1, 10);
        let found =
            year_yogas_with_rules(WEAK_PAIR_LAGNA_DEG, house(2), &WEAK_PAIR, rules).unwrap();
        for lord in [found.lagnesha, found.karyesha] {
            assert!(
                strength_with_rules(lord, &WEAK_PAIR, rules)
                    .unwrap()
                    .is_middling()
            );
        }
        assert_eq!(found.holds(YearYoga::DutthotthaDavira), Some(false));
    }

    /// A sky built so the Aries lagna's seventh-house pair -- Mars the
    /// lagnesha in Leo, Venus the karyesha in Sagittarius -- make a clean
    /// Ithasala: a friendly trine, Venus five degrees behind, neither
    /// debilitated, neither in a trika house, and Saturn in Libra, which
    /// reaches neither inimically. Only the states can afflict them.
    const CLEAN_ITHASALA: AnnualSky = AnnualSky {
        sun_deg: 10.0,
        moon_deg: 100.0,
        mars_deg: 130.0,
        mercury_deg: 20.0,
        jupiter_deg: 110.0,
        venus_deg: 245.0,
        saturn_deg: 190.0,
    };
    const CLEAN_LAGNA_DEG: f64 = 5.0;

    /// A sky with a Duhphali-kuttha in its third house, found by the same
    /// deterministic sweep as [`WEAK_PAIR`]: Venus, weak, coming to a
    /// strong Jupiter.
    ///
    /// The sweep's first find had the **Sun** as the faster, and the Sun
    /// can be neither retrograde nor combust, so the source's "but neither
    /// retrograde nor combust" could not be asked of it. This one was
    /// looked for with a faster that can be marked either way, on the
    /// 109th sky.
    const DUHPHALI: AnnualSky = AnnualSky {
        sun_deg: 136.346_319,
        moon_deg: 69.190_006,
        mars_deg: 204.854_078,
        mercury_deg: 117.352_653,
        jupiter_deg: 49.821_024,
        venus_deg: 135.096_967,
        saturn_deg: 232.528_038,
    };
    const DUHPHALI_LAGNA_DEG: f64 = 196.453_314;

    fn states(retrograde: &[Graha], combust: &[Graha]) -> AnnualStates {
        AnnualStates {
            retrograde: retrograde.to_vec(),
            combust: combust.to_vec(),
        }
    }

    fn with(sky: &AnnualSky, lagna: f64, number: u8, read: &AnnualStates) -> YearYogas {
        year_yogas_with_states(lagna, house(number), sky, read, YogaRules::default()).unwrap()
    }

    /// Every clause of an affliction answers for somebody in the worked
    /// chart or in the states given it, so none is dead.
    #[test]
    fn affliction_clauses_answer_for_the_worked_chart() {
        let read = states(&[Graha::Saturn], &[Graha::Mercury]);
        let of = |graha| affliction(graha, WORKED_LAGNA_DEG, &worked(), &read).unwrap();
        // Mars in Scorpio casts the tenth on Leo and the seventh on
        // Taurus, both inimical: the three in Leo and the Moon are under it.
        for graha in [Graha::Sun, Graha::Moon, Graha::Mercury, Graha::Venus] {
            assert!(of(graha).under_malefic, "{graha:?}");
        }
        // Saturn is in Libra, the twelfth from Scorpio, and turned back.
        let saturn = of(Graha::Saturn);
        assert!(saturn.trika && saturn.retrograde && !saturn.under_malefic);
        assert!(of(Graha::Mercury).combust);
        // Mars and Jupiter are clean, and nothing in this chart is
        // debilitated -- so that clause is asked of a constructed one.
        assert!(!of(Graha::Mars).is_afflicted());
        assert!(!of(Graha::Jupiter).is_afflicted());
        let debilitated = AnnualSky {
            moon_deg: 7.0 * 30.0 + 3.0,
            ..worked()
        };
        assert!(
            affliction(Graha::Moon, WORKED_LAGNA_DEG, &debilitated, &read)
                .unwrap()
                .debilitated
        );
        // The clauses are named in the source's order.
        let names: Vec<&str> = of(Graha::Mars)
            .clauses()
            .iter()
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            names,
            [
                "retrograde",
                "combust",
                "debilitated",
                "in the 6th, 8th or 12th",
                "under malefic influence"
            ]
        );
    }

    /// Rudda: an Ithasala where either of the pair is afflicted. A clean
    /// pair makes none, and the states alone can spoil it.
    #[test]
    fn rudda_needs_an_ithasala_and_an_affliction() {
        let clean = with(
            &CLEAN_ITHASALA,
            CLEAN_LAGNA_DEG,
            7,
            &AnnualStates::default(),
        );
        assert_eq!(
            (clean.lagnesha, clean.karyesha),
            (Graha::Mars, Graha::Venus)
        );
        assert_eq!(clean.holds(YearYoga::Ithasala), Some(true));
        assert_eq!(clean.holds(YearYoga::Rudda), Some(false));

        for read in [states(&[Graha::Venus], &[]), states(&[], &[Graha::Mars])] {
            let spoilt = with(&CLEAN_ITHASALA, CLEAN_LAGNA_DEG, 7, &read);
            assert_eq!(spoilt.holds(YearYoga::Rudda), Some(true), "{read:?}");
            // It stands beside the Ithasala it spoils, and says why.
            assert_eq!(spoilt.holds(YearYoga::Ithasala), Some(true));
            let rudda = spoilt
                .held
                .iter()
                .find(|one| one.yoga == YearYoga::Rudda)
                .unwrap();
            let [lagnesha, karyesha]: [Affliction; 2] = rudda.afflictions.unwrap();
            assert_eq!(
                (lagnesha.graha, karyesha.graha),
                (Graha::Mars, Graha::Venus)
            );
            assert_eq!(lagnesha.combust, read.is_combust(Graha::Mars));
            assert_eq!(karyesha.retrograde, read.is_retrograde(Graha::Venus));
        }
    }

    /// The worked chart's one Ithasala, in its tenth house, is a Rudda
    /// **only** because the Sun's own partner, Mars, reaches it from the
    /// tenth -- so whether a partner can afflict is load-bearing in the
    /// source's own chart (crux C118). Recorded here so a change of
    /// reading shows up as this test and not as a silent shift.
    #[test]
    fn the_worked_charts_rudda_turns_on_the_partner() {
        let found = with(&worked(), WORKED_LAGNA_DEG, 10, &AnnualStates::default());
        assert_eq!((found.lagnesha, found.karyesha), (Graha::Mars, Graha::Sun));
        assert_eq!(found.holds(YearYoga::Rudda), Some(true));
        let rudda = found
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Rudda)
            .unwrap();
        let [mars, sun] = rudda.afflictions.unwrap();
        assert!(!mars.is_afflicted());
        let clauses: Vec<&str> = sun
            .clauses()
            .iter()
            .filter(|(_, is)| *is)
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(clauses, ["under malefic influence"]);
    }

    /// Rudda judges an Ithasala and nothing else: the worked chart's
    /// ninth-house pair is an Ishrafa with an afflicted Moon, and is not
    /// a Rudda.
    #[test]
    fn rudda_is_not_asked_of_an_ishrafa() {
        let found = with(&worked(), WORKED_LAGNA_DEG, 9, &AnnualStates::default());
        assert_eq!(found.holds(YearYoga::Ishrafa), Some(true));
        assert!(
            affliction(
                Graha::Moon,
                WORKED_LAGNA_DEG,
                &worked(),
                &AnnualStates::default()
            )
            .unwrap()
            .is_afflicted()
        );
        assert_eq!(found.holds(YearYoga::Rudda), Some(false));
    }

    /// Duhphali-kuttha: the slower strong, the faster weak and **free**
    /// of retrograde and combustion -- the one place the source asks a
    /// planet to be clear of the two.
    #[test]
    fn duhphali_kuttha_wants_a_free_weak_faster_under_a_strong_slower() {
        let at = |read: &AnnualStates| with(&DUHPHALI, DUHPHALI_LAGNA_DEG, 3, read);
        let found = at(&AnnualStates::default());
        let pair = found.between.unwrap();
        assert_eq!((pair.faster, pair.slower), (Graha::Venus, Graha::Jupiter));
        assert!(strength(Graha::Venus, &DUHPHALI).unwrap().is_weak());
        assert!(strength(Graha::Jupiter, &DUHPHALI).unwrap().is_strong());
        assert_eq!(found.holds(YearYoga::DuhphaliKuttha), Some(true));
        // "But neither retrograde nor combust": either mark on the faster
        // spoils it, and the same mark on the slower does not.
        for read in [states(&[Graha::Venus], &[]), states(&[], &[Graha::Venus])] {
            assert_eq!(
                at(&read).holds(YearYoga::DuhphaliKuttha),
                Some(false),
                "{read:?}"
            );
        }
        assert_eq!(
            at(&states(&[Graha::Jupiter], &[Graha::Jupiter])).holds(YearYoga::DuhphaliKuttha),
            Some(true)
        );
        // Strengthen the faster past the floor and it goes.
        let rules = YogaRules {
            weak_below: Bala::new(1, 0, 0),
            ..YogaRules::default()
        };
        let lifted = year_yogas_with_states(
            DUHPHALI_LAGNA_DEG,
            house(3),
            &DUHPHALI,
            &AnnualStates::default(),
            rules,
        )
        .unwrap();
        assert_eq!(lifted.holds(YearYoga::DuhphaliKuttha), Some(false));
    }

    /// Durapha: both lords weak, and each in a trika house, combust or
    /// retrograde (crux C119). The weak pair from [`WEAK_PAIR`] stands in
    /// houses one and seven, so only the states can mark it -- and both
    /// must be marked.
    #[test]
    fn durapha_wants_both_weak_and_both_marked() {
        let at = |read: &AnnualStates| with(&WEAK_PAIR, WEAK_PAIR_LAGNA_DEG, 2, read);
        assert_eq!(
            at(&AnnualStates::default()).holds(YearYoga::Durapha),
            Some(false)
        );
        assert_eq!(
            at(&states(&[Graha::Mercury], &[])).holds(YearYoga::Durapha),
            Some(false)
        );
        let both = at(&states(&[Graha::Mercury], &[Graha::Venus]));
        assert_eq!(both.holds(YearYoga::Durapha), Some(true));
        let durapha = both
            .held
            .iter()
            .find(|one| one.yoga == YearYoga::Durapha)
            .unwrap();
        let [mercury, venus] = durapha.afflictions.unwrap();
        assert!(mercury.retrograde && venus.combust);
        // A middling pair, however marked, is not the case.
        let rules = YogaRules {
            weak_below: Bala::new(1, 0, 0),
            ..YogaRules::default()
        };
        let middling = year_yogas_with_states(
            WEAK_PAIR_LAGNA_DEG,
            house(2),
            &WEAK_PAIR,
            &states(&[Graha::Mercury], &[Graha::Venus]),
            rules,
        )
        .unwrap();
        assert_eq!(middling.holds(YearYoga::Durapha), Some(false));
    }

    /// Ikabala and Induvara are facts about the chart: they answer every
    /// matter alike, the first house -- which has no pair -- included.
    #[test]
    fn ikabala_and_induvara_are_facts_about_the_chart() {
        let all_in = |deg: f64| AnnualSky {
            sun_deg: deg,
            moon_deg: deg,
            mars_deg: deg,
            mercury_deg: deg,
            jupiter_deg: deg,
            venus_deg: deg,
            saturn_deg: deg,
        };
        // Everything in Aries under an Aries lagna: the first house, a
        // kendra.
        let first = all_in(10.0);
        for number in 1..=12u8 {
            let found = year_yogas(5.0, house(number), &first).unwrap();
            assert_eq!(found.holds(YearYoga::Ikabala), Some(true), "house {number}");
            assert_eq!(
                found.holds(YearYoga::Induvara),
                Some(false),
                "house {number}"
            );
        }
        assert!(year_yogas(5.0, house(1), &first).unwrap().same_lord);
        // Everything in Gemini: the third, an apoklima.
        let third = all_in(70.0);
        let found = year_yogas(5.0, house(1), &third).unwrap();
        assert_eq!(found.holds(YearYoga::Induvara), Some(true));
        assert_eq!(found.holds(YearYoga::Ikabala), Some(false));
        // The worked chart has Saturn in the twelfth and the Moon in the
        // seventh, so it is neither.
        assert_eq!(asked(1).holds(YearYoga::Ikabala), Some(false));
        assert_eq!(asked(1).holds(YearYoga::Induvara), Some(false));
    }

    /// States no chart can hold are refused by the judgement, on every
    /// call -- the first house, which reads no state, included.
    #[test]
    fn the_judgement_refuses_states_no_chart_can_hold() {
        let refused = year_yogas_with_states(
            WORKED_LAGNA_DEG,
            house(1),
            &worked(),
            &states(&[Graha::Moon], &[]),
            YogaRules::default(),
        )
        .unwrap_err();
        assert_eq!(refused.field(), Some("retrograde"));
        let refused = affliction(
            Graha::Mars,
            WORKED_LAGNA_DEG,
            &worked(),
            &states(&[], &[Graha::Sun]),
        )
        .unwrap_err();
        assert_eq!(refused.field(), Some("combust"));
    }

    /// The two groupings `YearYoga` describes itself by hold of every
    /// sky these tests use, under every set of states: no judgement upon
    /// an Ithasala without one, and no weak-pair yoga without a weak pair.
    /// The measured page holds the corpus to the same two ceilings.
    #[test]
    fn the_groupings_the_type_declares_are_what_the_judgement_does() {
        let skies = [
            (worked(), WORKED_LAGNA_DEG),
            (WEAK_PAIR, WEAK_PAIR_LAGNA_DEG),
            (CLEAN_ITHASALA, CLEAN_LAGNA_DEG),
            (DUHPHALI, DUHPHALI_LAGNA_DEG),
        ];
        let marked = states(
            &[Graha::Mars, Graha::Mercury, Graha::Venus],
            &[Graha::Mercury, Graha::Venus],
        );
        for (sky, lagna) in skies {
            for read in [AnnualStates::default(), marked.clone()] {
                for number in 1..=12u8 {
                    let found = with(&sky, lagna, number, &read);
                    let ithasala = found.holds(YearYoga::Ithasala) == Some(true);
                    let weak_pair = !found.same_lord
                        && strength(found.lagnesha, &sky).unwrap().is_weak()
                        && strength(found.karyesha, &sky).unwrap().is_weak();
                    for one in &found.held {
                        if one.yoga.judges_an_ithasala() {
                            assert!(ithasala, "{:?} without an Ithasala", one.yoga);
                        }
                        if one.yoga.needs_a_weak_pair() {
                            assert!(weak_pair, "{:?} without a weak pair", one.yoga);
                        }
                    }
                }
            }
        }
        // And the declarations are the source's: six judge an Ithasala,
        // two want a weak pair, none both.
        let count =
            |is: fn(YearYoga) -> bool| YearYoga::ALL.into_iter().filter(|one| is(*one)).count();
        assert_eq!(count(YearYoga::judges_an_ithasala), 6);
        assert_eq!(count(YearYoga::needs_a_weak_pair), 2);
        assert_eq!(count(YearYoga::is_chart_fact), 2);
        assert!(
            YearYoga::ALL
                .into_iter()
                .all(|one| !(one.judges_an_ithasala() && one.needs_a_weak_pair()))
        );
    }
}
