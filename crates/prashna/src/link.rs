//! What the Tajika links mean in a query: the Prashna Tantra's final
//! section (Tajika Nilakanthi, 1893, leaves n203–n205) reads the yogas
//! between the lagnesha and the karyesha as a promise, a promise through
//! another, or a failure, and its general section (n178–n179) reads the
//! lagnesha's own yogas as the past, the present and the future.

use serde::{Deserialize, Serialize};
use teistro_core::Error;
use teistro_core::catalogue::Graha;
use teistro_core::catalogue::Rashi;
use teistro_tajika::{
    AnnualSky, Drishti, KamboolaGrade, OfficeRules, SEVEN, YearYoga, YearYogas, Yoga, between,
    kamboola_grade,
};

/// When the matter asked about falls (crux C370).
///
/// A separation reads two ways in the text: v. 16 calls it inauspicious,
/// which a matter still to come reads as failure, and the general
/// section reads the graha the lagnesha separates from as what is past.
/// The question decides which, so it says.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// // A question of the 7th, Scorpio: Venus the lagnesha, Mars the karyesha.
/// let answer = read(&sky, Question::about(7), PrashnaRules::default())?;
/// use teistro_prashna::Tense;
///
/// let past = Question { tense: Tense::Past, ..Question::about(7) };
/// assert!(read(&sky, past, PrashnaRules::default())?.link.is_some());
/// # let _ = answer;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaTense"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Tense {
    /// Not said: a separation is reported both ways.
    #[default]
    Unspecified,
    /// A matter still to come: will it happen?
    Future,
    /// A matter that may already have happened: did it?
    Past,
}

impl Tense {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `UNSPECIFIED`.
    ///
    /// ```
    /// use teistro_prashna::Tense;
    ///
    /// assert_eq!(Tense::Unspecified.key(), "UNSPECIFIED");
    /// assert_eq!(serde_json::to_string(&Tense::Unspecified)?, r#""UNSPECIFIED""#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Tense::Unspecified => "UNSPECIFIED",
            Tense::Future => "FUTURE",
            Tense::Past => "PAST",
        }
    }
}

/// How the lagnesha and the karyesha are linked, as the Prashna Tantra
/// reads it.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// // A question of the 7th, Scorpio: Venus the lagnesha, Mars the karyesha.
/// let answer = read(&sky, Question::about(7), PrashnaRules::default())?;
/// use teistro_prashna::{LinkKind, Meaning};
///
/// let link = answer.link.ok_or(teistro_core::Error::internal("a house was asked"))?;
/// // An unlinked pair, or one lord of both, is a link the text does not read.
/// if matches!(link.kind, LinkKind::Unlinked | LinkKind::SameLord) {
///     assert_eq!(link.meaning, Meaning::Unread);
/// }
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaLinkKind"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LinkKind {
    /// An Ithasala between the two (v. 14).
    Direct,
    /// A third graha carries the light (Nakta, v. 17) or gathers it
    /// (Yamaya, v. 20): "attainment through another's hand" (vv. 18–19).
    Through,
    /// A malefic destroys the Ithasala (Manau). The Samjna Tantra's
    /// worked example to v. 35 is a query, a kingdom "taken away", so
    /// Manau overrides an Ithasala in a query too.
    Destroyed,
    /// The faster has passed the slower (Ishrafa, v. 16).
    Separated,
    /// One graha rules both houses, which the text does not judge.
    SameLord,
    /// None of the links the section names.
    Unlinked,
}

impl LinkKind {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `DIRECT`.
    ///
    /// ```
    /// use teistro_prashna::LinkKind;
    ///
    /// assert_eq!(LinkKind::Destroyed.key(), "DESTROYED");
    /// assert_eq!(serde_json::to_string(&LinkKind::Destroyed)?, r#""DESTROYED""#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            LinkKind::Direct => "DIRECT",
            LinkKind::Through => "THROUGH",
            LinkKind::Destroyed => "DESTROYED",
            LinkKind::Separated => "SEPARATED",
            LinkKind::SameLord => "SAME_LORD",
            LinkKind::Unlinked => "UNLINKED",
        }
    }
}

/// What a link says of the matter.
///
/// ```
/// use teistro_prashna::Meaning;
///
/// assert_eq!(Meaning::FailsOrPast.key(), "FAILS_OR_PAST");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaMeaning"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Meaning {
    /// It is promised.
    Promised,
    /// It fails.
    Fails,
    /// It is past.
    Past,
    /// A separation under [`Tense::Unspecified`]: it fails if it is to
    /// come, and is past if it may have happened.
    FailsOrPast,
    /// The text says nothing: no link of its kinds, or one lord of both.
    Unread,
}

impl Meaning {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `PROMISED`.
    ///
    /// ```
    /// use teistro_prashna::Meaning;
    ///
    /// assert_eq!(Meaning::Promised.key(), "PROMISED");
    /// assert_eq!(serde_json::to_string(&Meaning::Promised)?, r#""PROMISED""#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Meaning::Promised => "PROMISED",
            Meaning::Fails => "FAILS",
            Meaning::Past => "PAST",
            Meaning::FailsOrPast => "FAILS_OR_PAST",
            Meaning::Unread => "UNREAD",
        }
    }
}

/// Which questions v. 96's "all yogas are fruitless without the Moon"
/// governs (crux C376).
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// // A question of the 7th, Scorpio: Venus the lagnesha, Mars the karyesha.
/// let answer = read(&sky, Question::about(7), PrashnaRules::default())?;
/// use teistro_prashna::MoonScope;
///
/// // v. 96 reaches every question by default, so the Moon is reported.
/// assert_eq!(PrashnaRules::default().moon_scope, MoonScope::Every);
/// assert!(answer.link.is_some_and(|link| link.moonless.is_some()));
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MoonScope {
    /// Every question, as the design page and the maxim's wording read it.
    #[default]
    Every,
    /// Questions of gain, the 11th house, where the verse stands (vv.
    /// 94–96, the Bhuvanadīpaka's gain section).
    Gain,
}

impl MoonScope {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `EVERY`.
    ///
    /// ```
    /// use teistro_prashna::MoonScope;
    ///
    /// assert_eq!(MoonScope::Gain.key(), "GAIN");
    /// assert_eq!(serde_json::to_string(&MoonScope::Gain)?, r#""GAIN""#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            MoonScope::Every => "EVERY",
            MoonScope::Gain => "GAIN",
        }
    }
}

/// A condition Samarasiṁha (a Tajika author the Prashna Tantra's general
/// section quotes, n178) sets for a question, reported and never weighed.
/// His clause on "loss of the matter" when the karyesha looks at the
/// lagnesha and the lagna is not here: it contradicts his clauses of
/// success and one print cannot say which is the slip (crux C374).
///
/// ```
/// use teistro_prashna::SamarasimhaClause;
///
/// assert_eq!(SamarasimhaClause::ALL[3], SamarasimhaClause::MoonSeesLagnesha);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SamarasimhaClause {
    /// The lagnesha looks at the lagna and the karyesha at the house
    /// asked about (*lagne lagnādhipatiḥ kāryaṁ kāryādhipaḥ paśyet*).
    LordsSeeTheirHouses,
    /// The karyesha stands in the lagna and looks at the lagnesha: the
    /// matter comes then (*lagnasthaḥ kāryeśaḥ paśyati cel lagnapaṁ
    /// tadaiva bhavet*).
    KaryeshaInLagnaSeesLagnesha,
    /// The karyesha looks at the lagnesha from elsewhere: not quickly
    /// (*yady anyasthitaṁ tadā sa tvaraṁ na syāt*).
    KaryeshaElsewhereSeesLagnesha,
    /// The Moon looks at the lagnesha (*drakṣyati candraṁ vilagnapaṁ*),
    /// one of the conditions of success.
    MoonSeesLagnesha,
}

impl SamarasimhaClause {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `LORDS_SEE_THEIR_HOUSES`.
    ///
    /// ```
    /// use teistro_prashna::SamarasimhaClause;
    ///
    /// assert_eq!(SamarasimhaClause::LordsSeeTheirHouses.key(), "LORDS_SEE_THEIR_HOUSES");
    /// assert_eq!(serde_json::to_string(&SamarasimhaClause::LordsSeeTheirHouses)?, r#""LORDS_SEE_THEIR_HOUSES""#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            SamarasimhaClause::LordsSeeTheirHouses => "LORDS_SEE_THEIR_HOUSES",
            SamarasimhaClause::KaryeshaInLagnaSeesLagnesha => "KARYESHA_IN_LAGNA_SEES_LAGNESHA",
            SamarasimhaClause::KaryeshaElsewhereSeesLagnesha => "KARYESHA_ELSEWHERE_SEES_LAGNESHA",
            SamarasimhaClause::MoonSeesLagnesha => "MOON_SEES_LAGNESHA",
        }
    }

    /// Every clause, in the passage's order.
    pub const ALL: [SamarasimhaClause; 4] = [
        SamarasimhaClause::LordsSeeTheirHouses,
        SamarasimhaClause::KaryeshaInLagnaSeesLagnesha,
        SamarasimhaClause::KaryeshaElsewhereSeesLagnesha,
        SamarasimhaClause::MoonSeesLagnesha,
    ];
}

/// The link between the lagnesha and the karyesha, read.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// // A question of the 7th, Scorpio: Venus the lagnesha, Mars the karyesha.
/// let answer = read(&sky, Question::about(7), PrashnaRules::default())?;
/// let link = answer.link.ok_or(teistro_core::Error::internal("a house was asked"))?;
/// // Samarasimha's conditions are reported in his order, never weighed.
/// let order: Vec<usize> = link.samarasimha.iter().filter_map(|clause| teistro_prashna::SamarasimhaClause::ALL.iter().position(|one| one == clause)).collect();
/// assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaLink"))]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// Which link.
    pub kind: LinkKind,
    /// What it says of the matter, under the question's tense.
    pub meaning: Meaning,
    /// The yoga that made it: Ithasala, Nakta, Yamaya, Manau or Ishrafa.
    pub yoga: Option<YearYoga>,
    /// The third graha: the one carrying or gathering the light, or the
    /// malefic destroying it.
    pub through: Option<Graha>,
    /// The Kamboola's grade, when the Moon joins the Ithasala (v. 21).
    pub kamboola: Option<KamboolaGrade>,
    /// v. 96: whether the Moon looks at neither lord, by Tajika's sign
    /// aspect, the conjunction included; `None` where [`MoonScope`] does
    /// not reach the question. Reported, never weighed.
    pub moonless: Option<bool>,
    /// Samarasiṁha's conditions that hold, in his order.
    pub samarasimha: Vec<SamarasimhaClause>,
}

/// The lagnesha's own yogas read as time (n179, crux C375): the graha it
/// is separating from tells the past, the one it is joined with the
/// present, the one it will join the future.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// // Taurus rising: Venus, whose yogas are read as time.
/// assert_eq!(answer.tri_kala.lagnesha, Graha::Venus);
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TriKala {
    /// The lagnesha.
    pub lagnesha: Graha,
    /// In Ishrafa with it, inside the orb: separating.
    pub past: Vec<Graha>,
    /// In Vartamana or Poorna Ithasala with it: joined.
    pub present: Vec<Graha>,
    /// In Bhavishyat Ithasala with it, or aspecting it with the faster
    /// still behind and outside the orb: coming to it.
    pub future: Vec<Graha>,
}

/// Reads the link from the yogas the pair hold.
pub(crate) fn link(
    links: &YearYogas,
    sky: &AnnualSky,
    lagna: Rashi,
    tense: Tense,
    rules: LinkRules,
) -> Result<Link, Error> {
    let held = |yoga: YearYoga| links.held.iter().find(|one| one.yoga == yoga);
    let found = |kind, meaning, yoga: YearYoga| Link {
        kind,
        meaning,
        yoga: Some(yoga),
        through: held(yoga).and_then(|one| one.through),
        kamboola: None,
        moonless: None,
        samarasimha: Vec::new(),
    };
    let mut link = if links.same_lord {
        unread(LinkKind::SameLord)
    } else if held(YearYoga::Manau).is_some() {
        found(LinkKind::Destroyed, Meaning::Fails, YearYoga::Manau)
    } else if held(YearYoga::Ithasala).is_some() {
        found(LinkKind::Direct, Meaning::Promised, YearYoga::Ithasala)
    } else if let Some(yoga) = [YearYoga::Nakta, YearYoga::Yamaya]
        .into_iter()
        .find(|&yoga| held(yoga).is_some())
    {
        found(LinkKind::Through, Meaning::Promised, yoga)
    } else if held(YearYoga::Ishrafa).is_some() {
        let meaning = match tense {
            Tense::Unspecified => Meaning::FailsOrPast,
            Tense::Future => Meaning::Fails,
            Tense::Past => Meaning::Past,
        };
        found(LinkKind::Separated, meaning, YearYoga::Ishrafa)
    } else {
        unread(LinkKind::Unlinked)
    };
    if held(YearYoga::Kamboola).is_some() {
        link.kamboola = Some(kamboola_grade(
            links.lagnesha,
            links.karyesha,
            sky,
            rules.offices,
        )?);
    }
    let sees = |from: Rashi, to: Rashi| Drishti::between_signs(from, to).is_aspect();
    let (lagnesha, karyesha) = (sky.sign_of(links.lagnesha), sky.sign_of(links.karyesha));
    let moon = sky.sign_of(Graha::Moon);
    let reached = match rules.moon_scope {
        MoonScope::Every => true,
        MoonScope::Gain => links.house.get() == 11,
    };
    link.moonless = reached.then(|| !sees(moon, lagnesha) && !sees(moon, karyesha));
    link.samarasimha = SamarasimhaClause::ALL
        .into_iter()
        .filter(|clause| match clause {
            SamarasimhaClause::LordsSeeTheirHouses => {
                sees(lagnesha, lagna) && sees(karyesha, links.sign)
            }
            SamarasimhaClause::KaryeshaInLagnaSeesLagnesha => {
                karyesha == lagna && sees(karyesha, lagnesha)
            }
            SamarasimhaClause::KaryeshaElsewhereSeesLagnesha => {
                karyesha != lagna && sees(karyesha, lagnesha)
            }
            SamarasimhaClause::MoonSeesLagnesha => sees(moon, lagnesha),
        })
        .collect();
    Ok(link)
}

/// The readings a link is given under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct LinkRules {
    pub(crate) offices: OfficeRules,
    pub(crate) moon_scope: MoonScope,
}

/// A link the text does not read.
const fn unread(kind: LinkKind) -> Link {
    Link {
        kind,
        meaning: Meaning::Unread,
        yoga: None,
        through: None,
        kamboola: None,
        moonless: None,
        samarasimha: Vec::new(),
    }
}

/// The lagnesha's yogas with each other of the seven, as time.
pub(crate) fn tri_kala(lagnesha: Graha, sky: &AnnualSky) -> Result<TriKala, Error> {
    let mut read = TriKala {
        lagnesha,
        past: Vec::new(),
        present: Vec::new(),
        future: Vec::new(),
    };
    for other in SEVEN.into_iter().filter(|&other| other != lagnesha) {
        let pair = between(lagnesha, other, sky)?;
        let when = match pair.yoga {
            Some(Yoga::Ishrafa) => &mut read.past,
            Some(Yoga::IthasalaVartamana | Yoga::IthasalaPoorna) => &mut read.present,
            Some(Yoga::IthasalaBhavishyat) => &mut read.future,
            // Aspecting, the faster still behind, the orb not yet reached.
            None if pair.drishti.is_aspect() && !pair.within && pair.apart_deg > 0.0 => {
                &mut read.future
            }
            None => continue,
        };
        when.push(other);
    }
    Ok(read)
}
