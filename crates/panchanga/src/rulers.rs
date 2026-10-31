//! The year's office-bearers: the raja, the mantri and the lords of the
//! crops, the rains and the rest (`03-design/year-rulers.md`).
//!
//! Each office goes to the lord of a weekday: the raja to the weekday of
//! Chaitra Shukla Pratipada, every other office to the weekday of one of
//! the Sun's ingresses into a sign or, for the lord of the clouds, into
//! Ardra. The texts agree on which ingress makes which office and differ
//! on which offices there are, and on which day an event's weekday is:
//!
//! - *Jyotirnibandha* (Shivaraja, Anandashrama Sanskrit Series 85, 1919),
//!   p. 29, "atha rājādīnāṃ nirṇayaḥ" vv. 1–10: six offices, and four
//!   readings of the raja's day (C1070);
//! - *Narada Samhita* ch. 2 vv. 1–3 (Ram Janma Mishra's edition, Kashi
//!   Sanskrit Series 40, 1984, pp. 6–7): five, among them the lord of the
//!   dry goods at Makara, whom *Jyotirnibandha* does not name;
//! - the nine of the southern panchangas (*navanāyaka*), which add the
//!   commander at Simha and the lord of prices at Mithuna, printed every
//!   year and cited here to no verse (C1071).
//!
//! So every office is answered, each naming the texts that hold it, and
//! each office's weekday is read under every rule the texts give for
//! it, each named (C1072): nothing is chosen silently.

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Longitudes, Search};
use teistro_core::catalogue::{Graha, Vara};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Ut1, Utc};
use teistro_port_ephemeris::{Body, Lattice, Quantity};
use teistro_time::local_day::LocalDay;

use crate::limb::{Sidereal, Zodiac};

/// How far past the year's opening its last ingress, Makara's, can lie,
/// days. The opening new moon has the Sun in Meena, at least 330°, so
/// Makara's 270° is at most 300° on: about 305 days at the Sun's slowest.
const YEAR_REACH_DAYS: f64 = 330.0;

/// How far past the opening Ardra's entry can lie, days: at most about
/// 97° on from Meena's start.
const ARDRA_REACH_DAYS: f64 = 110.0;

/// How far past the opening the pratipada can end, days: a tithi is
/// never as long as two.
const PRATIPADA_REACH_DAYS: f64 = 2.0;

/// How close a crossing's reported boundary is to a target to be it,
/// degrees.
const SAME_LINE_DEG: f64 = 1e-6;

/// One of the year's offices.
///
/// ```
/// use teistro_panchanga::rulers::{YearOffice, RulerText};
///
/// // The lord of the crops is the weekday's lord at Karka's ingress, and
/// // every text names him.
/// assert_eq!(YearOffice::Sasyadhipati.ingress_deg(), Some(90.0));
/// assert_eq!(YearOffice::Sasyadhipati.texts().len(), 3);
/// // The raja is read from the year's first day, not from an ingress.
/// assert_eq!(YearOffice::Raja.ingress_deg(), None);
/// assert_eq!(YearOffice::Senadhipati.texts(), &[RulerText::Navanayaka]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YearOffice {
    /// The king, the year's lord (*abdapa*, *varṣeśa*): the lord of
    /// Chaitra Shukla Pratipada's weekday.
    Raja,
    /// The minister (*mantrī*; *bhūpati* in *Narada Samhita*): Mesha's
    /// ingress.
    Mantri,
    /// The commander (*senādhipati*): Simha's ingress.
    Senadhipati,
    /// The lord of the crops (*sasyādhipa*), *Jyotirnibandha*'s first
    /// lord of grain (*agradhānyeśa*): Karka's ingress.
    Sasyadhipati,
    /// The lord of grain (*dhānyādhipa*), *Jyotirnibandha*'s later lord
    /// of grain (*paścimadhānyeśa*): Dhanu's ingress.
    Dhanyadhipati,
    /// The lord of prices (*arghādhipati*): Mithuna's ingress.
    Arghadhipati,
    /// The lord of the clouds (*meghādhipa*): the Sun's entry into Ardra.
    Meghadhipati,
    /// The lord of juices (*rasādhipa*): Tula's ingress.
    Rasadhipati,
    /// The lord of dry goods (*nīrasapati*): Makara's ingress.
    Nirasadhipati,
}

impl YearOffice {
    /// Every office, in the order a southern panchanga prints them.
    pub const ALL: [YearOffice; 9] = [
        YearOffice::Raja,
        YearOffice::Mantri,
        YearOffice::Senadhipati,
        YearOffice::Sasyadhipati,
        YearOffice::Dhanyadhipati,
        YearOffice::Arghadhipati,
        YearOffice::Meghadhipati,
        YearOffice::Rasadhipati,
        YearOffice::Nirasadhipati,
    ];

    /// The Sun's sidereal longitude whose crossing makes the office,
    /// degrees, or `None` for the raja, whose day is the year's first.
    /// Ardra begins at the end of the fifth nakshatra, 66°40′.
    ///
    /// ```
    /// use teistro_panchanga::rulers::YearOffice;
    ///
    /// assert_eq!(YearOffice::Nirasadhipati.ingress_deg(), Some(270.0));
    /// let ardra = YearOffice::Meghadhipati.ingress_deg().unwrap_or_default();
    /// assert!((ardra - 66.0 - 2.0 / 3.0).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn ingress_deg(self) -> Option<f64> {
        match self {
            YearOffice::Raja => None,
            YearOffice::Mantri => Some(0.0),
            YearOffice::Arghadhipati => Some(60.0),
            YearOffice::Meghadhipati => Some(5.0 * 360.0 / 27.0),
            YearOffice::Sasyadhipati => Some(90.0),
            YearOffice::Senadhipati => Some(120.0),
            YearOffice::Rasadhipati => Some(180.0),
            YearOffice::Dhanyadhipati => Some(240.0),
            YearOffice::Nirasadhipati => Some(270.0),
        }
    }

    /// The texts that name the office, as [`RulerText`] lists them.
    ///
    /// ```
    /// use teistro_panchanga::rulers::{RulerText, YearOffice};
    ///
    /// // *Jyotirnibandha* has no lord of dry goods, *Narada Samhita* no lord of the clouds.
    /// assert!(!YearOffice::Nirasadhipati.texts().contains(&RulerText::Jyotirnibandha));
    /// assert!(!YearOffice::Meghadhipati.texts().contains(&RulerText::NaradaSamhita));
    /// ```
    #[must_use]
    pub fn texts(self) -> &'static [RulerText] {
        use RulerText::{Jyotirnibandha, NaradaSamhita, Navanayaka};
        match self {
            YearOffice::Raja
            | YearOffice::Mantri
            | YearOffice::Sasyadhipati
            | YearOffice::Rasadhipati => &[Jyotirnibandha, NaradaSamhita, Navanayaka],
            YearOffice::Meghadhipati | YearOffice::Dhanyadhipati => &[Jyotirnibandha, Navanayaka],
            YearOffice::Nirasadhipati => &[NaradaSamhita, Navanayaka],
            YearOffice::Senadhipati | YearOffice::Arghadhipati => &[Navanayaka],
        }
    }

    /// The rules an office's weekday is read under, the first being the
    /// one the texts' own words read most plainly (C1072).
    ///
    /// ```
    /// use teistro_panchanga::rulers::{RulerDayRule, YearOffice};
    ///
    /// assert_eq!(YearOffice::Raja.rules()[0], RulerDayRule::Sunrise);
    /// assert_eq!(YearOffice::Mantri.rules(), &[RulerDayRule::Daylight, RulerDayRule::Vara]);
    /// ```
    #[must_use]
    pub fn rules(self) -> &'static [RulerDayRule] {
        match self {
            YearOffice::Raja => &[
                RulerDayRule::Sunrise,
                RulerDayRule::NewMoon,
                RulerDayRule::Garga,
            ],
            _ => &[RulerDayRule::Daylight, RulerDayRule::Vara],
        }
    }
}

/// A text, or a body of practice, that names the year's offices.
///
/// ```
/// use teistro_panchanga::rulers::RulerText;
///
/// assert_eq!(serde_json::to_string(&RulerText::NaradaSamhita).unwrap(), r#""NARADA_SAMHITA""#);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RulerText {
    /// *Jyotirnibandha* p. 29 vv. 7–10, quoting *Jyotiṣaphalodaya*: the
    /// raja, the mantri and the lords of the crops, the clouds, juices
    /// and grain.
    Jyotirnibandha,
    /// *Narada Samhita* ch. 2 vv. 1–3: the raja, the mantri and the lords
    /// of the crops, juices and dry goods.
    NaradaSamhita,
    /// The nine (*navanāyaka*) of the southern panchangas, which add the
    /// commander and the lord of prices; no verse read (C1071).
    Navanayaka,
}

/// Which day an event's weekday is read from.
///
/// ```
/// use teistro_panchanga::rulers::RulerDayRule;
///
/// assert_eq!(serde_json::to_string(&RulerDayRule::NewMoon).unwrap(), r#""NEW_MOON""#);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RulerDayRule {
    /// The raja: the weekday at the sunrise that Chaitra Shukla
    /// Pratipada runs at; of two such sunrises the earlier, and when the
    /// pratipada runs at none, the earlier day, the one it began in
    /// (*Jyotirnibandha* vv. 2–4, "many say"; *Narada Samhita* 2.1).
    Sunrise,
    /// The raja: the weekday holding the new moon itself, "the junction
    /// of the new moon and the pratipada" (*Jyotirnibandha* v. 1, from
    /// *Ratnāvalī*).
    NewMoon,
    /// The raja: the day of the new moon if it falls before that day's
    /// midday, else the next (*Jyotirnibandha* v. 6, Garga).
    Garga,
    /// An ingress: the day whose daylight holds it, and for one at night
    /// the day that begins at the next sunrise, "the day (*ahan*) of the
    /// entry" read as the daylight. The printed southern panchangas.
    Daylight,
    /// An ingress: the weekday, sunrise to sunrise, holding its instant.
    Vara,
}

/// An office's weekday and its lord under one rule.
///
/// ```
/// use teistro_core::catalogue::{Graha, Vara};
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::rulers::{RulerDayRule, RulerReading};
///
/// let reading = RulerReading {
///     rule: RulerDayRule::Vara,
///     vara: Vara::Guruvara,
///     lord: Graha::Jupiter,
///     day: JulianDay::literal(2_461_237.5),
/// };
/// assert_eq!(reading.lord, reading.vara.attributes().lord);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RulerReading {
    /// The rule.
    pub rule: RulerDayRule,
    /// The weekday the rule reads.
    pub vara: Vara,
    /// Its lord, who holds the office.
    pub lord: Graha,
    /// The sunrise that opened that weekday, a UTC Julian day.
    pub day: JulianDay<Utc>,
}

/// One office of the year and who holds it.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::rulers::{YearOffice, YearRuler};
///
/// // An office with no reading has no lord to name.
/// let event = JulianDay::literal(2_461_118.9);
/// let office = YearOffice::Raja;
/// let ruler = YearRuler { office, event, texts: office.texts().to_vec(), readings: Vec::new() };
/// assert_eq!(ruler.lord(), None);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YearRuler {
    /// The office.
    pub office: YearOffice,
    /// The event that makes it: the new moon that opened the year for the
    /// raja, the Sun's ingress for the rest, a UTC Julian day.
    pub event: JulianDay<Utc>,
    /// The texts that name the office.
    pub texts: Vec<RulerText>,
    /// Its weekday and lord under each rule [`YearOffice::rules`] gives, in
    /// that order.
    pub readings: Vec<RulerReading>,
}

impl YearRuler {
    /// The lord under the office's first rule, the plainest reading.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Vara};
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::rulers::{RulerDayRule, YearOffice, RulerReading, YearRuler};
    ///
    /// let day = JulianDay::literal(2_461_118.5);
    /// let reading = RulerReading { rule: RulerDayRule::Daylight, vara: Vara::Mangalavara, lord: Graha::Mars, day };
    /// let ruler = YearRuler { office: YearOffice::Mantri, event: day, texts: Vec::new(), readings: vec![reading] };
    /// assert_eq!(ruler.lord(), Some(Graha::Mars));
    /// ```
    #[must_use]
    pub fn lord(&self) -> Option<Graha> {
        self.readings.first().map(|reading| reading.lord)
    }
}

/// An office's weekday under one rule, from the day (sunrise to sunrise)
/// that holds its event.
///
/// `pratipada_ends` is when the year's first tithi ends, which only
/// [`RulerDayRule::Sunrise`] reads: a pratipada ending before the next sunrise
/// runs at none, and the day it began in is the raja's.
///
/// ```
/// use teistro_core::catalogue::{Calendar, Graha, Vara};
/// use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// use teistro_core::settings::Sunrise;
/// use teistro_panchanga::rulers::{RulerDayRule, read};
/// use teistro_time::local_day::{DayState, LocalDay};
///
/// // A Thursday, sunrise to sunrise.
/// let jd = JulianDay::literal;
/// let thursday = LocalDay {
///     place: Place::new(Latitude::literal(17.385), Longitude::literal(78.487), Altitude::literal(500.0)),
///     date: teistro_calendar::CalendarDate::defined(Calendar::Gregorian, 2026, 7, 16),
///     vara: Vara::Guruvara,
///     sunrise: jd(2_461_237.5),
///     sunset: jd(2_461_238.0),
///     next_sunrise: jd(2_461_238.5),
///     state: DayState::Normal,
///     convention: Sunrise::CentreNoRefraction.into(),
///     model: String::new(),
/// };
/// // An ingress in the night: Thursday's by the weekday, Friday's by the daylight.
/// let night = jd(2_461_238.2);
/// assert_eq!(read(RulerDayRule::Vara, night, &thursday, None).lord, Graha::Jupiter);
/// let daylight = read(RulerDayRule::Daylight, night, &thursday, None);
/// assert_eq!((daylight.vara, daylight.day), (Vara::Shukravara, thursday.next_sunrise));
/// ```
#[must_use]
pub fn read(
    rule: RulerDayRule,
    event: JulianDay<Utc>,
    holding: &LocalDay,
    pratipada_ends: Option<JulianDay<Utc>>,
) -> RulerReading {
    let midday = f64::midpoint(holding.sunrise.get(), holding.sunset.get());
    let next = match rule {
        RulerDayRule::NewMoon | RulerDayRule::Vara => false,
        // The pratipada runs at the next sunrise unless it ended first.
        RulerDayRule::Sunrise => {
            pratipada_ends.is_none_or(|ends| ends.get() >= holding.next_sunrise.get())
        }
        RulerDayRule::Garga => event.get() >= midday,
        RulerDayRule::Daylight => event.get() >= holding.sunset.get(),
    };
    let (vara, day) = if next {
        (following(holding.vara), holding.next_sunrise)
    } else {
        (holding.vara, holding.sunrise)
    };
    RulerReading {
        rule,
        vara,
        lord: vara.attributes().lord,
        day,
    }
}

/// The weekday after another.
fn following(vara: Vara) -> Vara {
    let weekday = (u16::from(vara.attributes().weekday) + 1) % 7;
    Vara::from_id(weekday).unwrap_or(vara)
}

/// The events a year's offices are read from: each office's, in
/// [`YearOffice::ALL`]'s order, and when the year's first tithi ended.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::rulers::{YearEvents, YearOffice};
///
/// let opened = JulianDay::literal(2_461_118.56);
/// let events = YearEvents { offices: vec![(YearOffice::Raja, opened)], pratipada_ends: JulianDay::literal(2_461_119.45) };
/// assert!(events.pratipada_ends.get() > events.offices[0].1.get());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct YearEvents {
    /// Each office and its event: the opening new moon for the raja, the
    /// Sun's ingress for the rest.
    pub offices: Vec<(YearOffice, JulianDay<Utc>)>,
    /// When the pratipada that the opening began ended.
    pub pratipada_ends: JulianDay<Utc>,
}

/// The events of the year a new moon opened, found in a sky: the Sun's
/// first crossing of each office's longitude after the opening, which is
/// this year's since the opening has the Sun in Meena, and the end of
/// the first tithi.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::rulers::{YearOffice, events};
/// use teistro_panchanga::year::chaitra_opening;
///
/// let opened = chaitra_opening(&longitudes, zodiac, window.from)?;
/// let year = events(&longitudes, zodiac, opened)?;
/// // Every office, the raja at the opening and the rest in the Sun's order.
/// assert_eq!(year.offices.len(), YearOffice::ALL.len());
/// assert_eq!(year.offices[0], (YearOffice::Raja, opened));
/// let at = |office| year.offices.iter().find(|(o, _)| *o == office).map(|(_, t)| t.get());
/// assert!(at(YearOffice::Mantri) < at(YearOffice::Arghadhipati));
/// assert!(at(YearOffice::Arghadhipati) < at(YearOffice::Meghadhipati));
/// assert!(at(YearOffice::Dhanyadhipati) < at(YearOffice::Nirasadhipati));
/// assert!(year.pratipada_ends.get() > opened.get());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The provider's refusal, or `NOT_CONVERGED` when an office's crossing
/// is not where the year's sky puts it.
pub fn events<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    opened: JulianDay<Utc>,
) -> Result<YearEvents, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let after = |days: f64| JulianDay::<Ut1>::literal(opened.get() + days);
    let from = after(0.0);
    let crossings = |quantity: Quantity, lattice: Lattice, reach: f64| {
        Search::new(&source, quantity, lattice).between(from, after(reach))
    };
    let sun = Quantity::Longitude(Body::Sun);
    let signs = crossings(sun, Lattice::SIGNS, YEAR_REACH_DAYS)?;
    let ardra = YearOffice::Meghadhipati.ingress_deg().unwrap_or_default();
    let clouds = crossings(sun, Lattice::single(ardra), ARDRA_REACH_DAYS)?;
    let mut offices = Vec::with_capacity(YearOffice::ALL.len());
    for office in YearOffice::ALL {
        let Some(target) = office.ingress_deg() else {
            offices.push((office, opened));
            continue;
        };
        let list = if office == YearOffice::Meghadhipati {
            &clouds
        } else {
            &signs
        };
        let found = list
            .iter()
            .find(|event| same_line(event.boundary_deg, target))
            .ok_or_else(|| {
                Error::new(
                    Status::NotConverged,
                    format!("no ingress at {target}° within a year of the opening at {opened}"),
                )
            })?;
        offices.push((office, JulianDay::try_new(found.instant.get())?));
    }
    let tithi = crossings(
        Quantity::ELONGATION,
        Lattice::single(12.0),
        PRATIPADA_REACH_DAYS,
    )?;
    let ends = tithi.first().ok_or_else(|| {
        Error::new(
            Status::NotConverged,
            format!("the pratipada after the opening at {opened} did not end within two days"),
        )
    })?;
    Ok(YearEvents {
        offices,
        pratipada_ends: JulianDay::try_new(ends.instant.get())?,
    })
}

/// Whether a crossing's boundary is a target line, either side of 0°.
fn same_line(boundary: f64, target: f64) -> bool {
    let apart = (boundary - target).rem_euclid(360.0);
    apart < SAME_LINE_DEG || apart > 360.0 - SAME_LINE_DEG
}
