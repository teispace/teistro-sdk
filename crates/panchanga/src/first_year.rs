//! The first year after a death (`03-design/shraddha.md` §3.3): the
//! sixteen shraddhas of the *Dharmasindhu* (pp. 329–331), the sapindana,
//! and the first abdika, each on its day.
//!
//! Every day is the festival evaluator's, through rules kept every month,
//! and the eleventh day is the ashauch's day one plus ten, so a monthly,
//! an anniversary and an ashauch never judge a day differently.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{Kind, Nakshatra, Tithi, Vara, write_in_full};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::LunarMonth as Convention;
use teistro_core::time::LocalClock;

use crate::ashauch::{self, DayOneBy, NightRule};
use crate::festival::{
    Anniversary, AnniversaryKind, FestivalDay, FestivalRule, FirstYearAdhika, InAdhika, InSamsarpa,
    JanmaOptions, Observance, Origin, OriginFound, Window, anniversaries, by_share, observances,
};
use crate::limb::Member;

/// The last monthly's number: the twelfth closes the year.
const LAST_MONTHLY: u8 = 12;

/// When the first abdika is kept (DS p. 330, two views).
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # use teistro_panchanga::first_year::FirstYear;
/// #
/// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
/// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
/// # let death = JulianDay::literal(sunrise.get() + 0.25);
///
/// use teistro_panchanga::first_year::Rite;
///
/// // Nepal's view keeps an abdika on the eleventh day, and the
/// // anniversary is the second.
/// let nepal = almanac.first_year(&FirstYear::nepal("parent", death), &kathmandu)?.value;
/// let abdikas: Vec<_> = nepal.rites.iter().filter(|r| r.rite == Rite::Abdika).collect();
/// assert_eq!(abdikas.last().and_then(|r| r.year), Some(2));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum FirstAbdika {
    /// At the second year's start, the death's first anniversary: "आब्दिकं
    /// वत्सरे गते".
    #[default]
    SecondYear,
    /// On the eleventh day with the first monthly, the anniversary being
    /// the second: the view Dhakal records for Nepal.
    EleventhDay,
}

/// The day the sapindana is kept on (DS p. 331).
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # use teistro_panchanga::first_year::FirstYear;
/// #
/// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
/// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
/// # let death = JulianDay::literal(sunrise.get() + 0.25);
///
/// use teistro_panchanga::first_year::SapindanaAt;
///
/// let asked = FirstYear { sapindana: SapindanaAt::EleventhDay, ..FirstYear::dharmasindhu("parent", death) };
/// let schedule = almanac.first_year(&asked, &kathmandu)?.value;
/// assert_eq!(schedule.sapindana, schedule.eleventh_day);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SapindanaAt {
    /// The eleventh day.
    EleventhDay,
    /// The twelfth day.
    #[default]
    TwelfthDay,
    /// The traipakshika's day.
    Traipakshika,
    /// The year's end, the first anniversary's day: "पूर्णे संवत्सरे".
    YearEnd,
}

/// How far short an *ūna* rite is kept (DS p. 330).
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::first_year::{FirstYear, UnaDay};
///
/// // Madhava's day before, unless a request names a tithi.
/// let asked = FirstYear::dharmasindhu("parent", JulianDay::literal(2_461_370.0));
/// assert_eq!(asked.una, UnaDay::DayBefore);
/// assert_eq!(serde_json::to_string(&UnaDay::TwoTithis)?, r#""TWO_TITHIS""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum UnaDay {
    /// The day before the day it is short of: Madhava's.
    #[default]
    DayBefore,
    /// The tithi one short, its last occurrence before.
    OneTithi,
    /// The tithi two short.
    TwoTithis,
    /// The tithi three short.
    ThreeTithis,
}

impl UnaDay {
    /// The tithis short, none for the day before.
    const fn tithis(self) -> Option<u32> {
        match self {
            UnaDay::DayBefore => None,
            UnaDay::OneTithi => Some(1),
            UnaDay::TwoTithis => Some(2),
            UnaDay::ThreeTithis => Some(3),
        }
    }
}

/// A death's first year, as a request asks it.
///
/// ```
/// use teistro_panchanga::first_year::{FirstAbdika, FirstYear, SapindanaAt};
///
/// let asked: FirstYear = serde_json::from_str(r#"{"key": "GRANDFATHER", "at": 2461051.677083}"#)?;
/// assert_eq!((asked.first_abdika, asked.sapindana), (FirstAbdika::SecondYear, SapindanaAt::TwelfthDay));
/// assert_eq!(FirstYear::nepal("GRANDFATHER", asked.at).first_abdika, FirstAbdika::EleventhDay);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FirstYearRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FirstYear {
    /// The consumer's own key, `"GRANDFATHER"`.
    pub key: String,
    /// The death, a UTC Julian day.
    pub at: JulianDay<Utc>,
    /// Which day a death at night belongs to, as the ashauch counts it.
    #[serde(default = "thirds")]
    pub night: NightRule,
    /// When the first abdika is kept.
    #[serde(default)]
    pub first_abdika: FirstAbdika,
    /// The anniversary's month when it is doubled (C666).
    #[serde(default)]
    pub first_year_adhika: FirstYearAdhika,
    /// The sapindana's day.
    #[serde(default)]
    pub sapindana: SapindanaAt,
    /// How far short the *ūna* rites are kept.
    #[serde(default)]
    pub una: UnaDay,
    /// The rite's time in place of both forms' (kutapa to rauhina for the
    /// ekoddishta, aparahna for the parvana).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<Window>,
}

const fn thirds() -> NightRule {
    NightRule::Thirds
}

impl FirstYear {
    /// The *Dharmasindhu*'s defaults for a death at `at`.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::first_year::{FirstAbdika, FirstYear, SapindanaAt};
    ///
    /// let asked = FirstYear::dharmasindhu("parent", JulianDay::literal(2_461_370.0));
    /// assert_eq!((asked.first_abdika, asked.sapindana), (FirstAbdika::SecondYear, SapindanaAt::TwelfthDay));
    /// assert!(asked.window.is_none());
    /// ```
    #[must_use]
    pub fn dharmasindhu(key: &str, at: JulianDay<Utc>) -> FirstYear {
        FirstYear {
            key: key.to_owned(),
            at,
            night: NightRule::Thirds,
            first_abdika: FirstAbdika::SecondYear,
            first_year_adhika: FirstYearAdhika::Adhika,
            sapindana: SapindanaAt::TwelfthDay,
            una: UnaDay::DayBefore,
            window: None,
        }
    }

    /// Dhakal's readings for Nepal: the first abdika on the eleventh day,
    /// and the anniversary, the barkhi, in the ordinary month.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::festival::FirstYearAdhika;
    /// use teistro_panchanga::first_year::{FirstAbdika, FirstYear};
    ///
    /// let asked = FirstYear::nepal("parent", JulianDay::literal(2_461_370.0));
    /// assert_eq!((asked.first_abdika, asked.first_year_adhika), (FirstAbdika::EleventhDay, FirstYearAdhika::Nija));
    /// ```
    #[must_use]
    pub fn nepal(key: &str, at: JulianDay<Utc>) -> FirstYear {
        FirstYear {
            first_abdika: FirstAbdika::EleventhDay,
            first_year_adhika: FirstYearAdhika::Nija,
            ..FirstYear::dharmasindhu(key, at)
        }
    }

    /// Refuses an empty key.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::first_year::FirstYear;
    ///
    /// let at = JulianDay::literal(2_461_370.0);
    /// assert!(FirstYear::dharmasindhu("parent", at).check().is_ok());
    /// assert_eq!(FirstYear::dharmasindhu(" ", at).check().unwrap_err().field(), Some("key"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming `key`.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("a first year needs a key").with_field("key"));
        }
        Ok(())
    }

    /// The days the schedule reads: from two days before the death to
    /// fourteen lunations after it, which holds a first anniversary kept
    /// in an adhika thirteenth month and the day after it.
    ///
    /// ```
    /// use teistro_panchanga::first_year::FirstYear;
    ///
    /// // Fourteen lunations hold a thirteenth month and the day after.
    /// let (back, ahead) = FirstYear::reach();
    /// assert_eq!(back, 2);
    /// assert!(ahead as f64 > 14.0 * 29.53);
    /// ```
    #[must_use]
    pub const fn reach() -> (i64, i64) {
        (2, 420)
    }

    /// The window a form is judged at.
    fn window(&self, form: RiteForm) -> Window {
        self.window.unwrap_or_else(|| form.kind().window())
    }
}

/// Which rite.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # use teistro_panchanga::first_year::FirstYear;
/// #
/// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
/// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
/// # let death = JulianDay::literal(sunrise.get() + 0.25);
/// # let schedule = almanac.first_year(&FirstYear::dharmasindhu("parent", death), &kathmandu)?.value;
///
/// use teistro_panchanga::first_year::Rite;
///
/// // The first monthly, then the other eleven months' monthlies.
/// assert_eq!(schedule.rites.iter().filter(|r| r.rite == Rite::AdyaMasika).count(), 1);
/// assert!(schedule.rites.iter().filter(|r| r.rite == Rite::Masika).count() >= 11);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Rite {
    /// The first monthly, on the eleventh day.
    AdyaMasika,
    /// Short of the first month.
    UnaMasika,
    /// A monthly, at its month's death tithi.
    Masika,
    /// At three fortnights.
    Traipakshika,
    /// Short of the sixth month.
    UnaShanmasika,
    /// Short of the year.
    UnaAbdika,
    /// A yearly shraddha.
    Abdika,
    /// The joining with the ancestors.
    Sapindana,
}

/// How a rite is performed.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # use teistro_panchanga::first_year::FirstYear;
/// #
/// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
/// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
/// # let death = JulianDay::literal(sunrise.get() + 0.25);
/// # let schedule = almanac.first_year(&FirstYear::dharmasindhu("parent", death), &kathmandu)?.value;
///
/// use teistro_panchanga::first_year::{Rite, RiteForm};
///
/// // The first monthly falls before the sapindana and keeps the
/// // ekoddishta; a monthly after it is done again by the parvana, drawn
/// // forward to the sapindana's day.
/// let first = schedule.rites.iter().find(|r| r.rite == Rite::AdyaMasika).ok_or("the first monthly")?;
/// assert_eq!((first.form, first.drawn_to.as_ref()), (Some(RiteForm::Ekoddishta), None));
/// let last = schedule.rites.iter().rfind(|r| r.rite == Rite::Masika).ok_or("a monthly")?;
/// assert_eq!((last.form, last.drawn_to.as_ref()), (Some(RiteForm::Parvana), Some(&schedule.sapindana)));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiteForm {
    /// For the one dead, at kutapa to rauhina: before the sapindana.
    Ekoddishta,
    /// For three generations, at aparahna: after it.
    Parvana,
}

impl RiteForm {
    const fn kind(self) -> AnniversaryKind {
        match self {
            RiteForm::Ekoddishta => AnniversaryKind::Ekoddishta,
            RiteForm::Parvana => AnniversaryKind::Parvana,
        }
    }
}

/// What DS bars an *ūna* rite's day for, found at its sunrise.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # use teistro_panchanga::first_year::FirstYear;
/// #
/// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
/// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
/// # let death = JulianDay::literal(sunrise.get() + 0.25);
/// # let schedule = almanac.first_year(&FirstYear::dharmasindhu("parent", death), &kathmandu)?.value;
///
/// use teistro_panchanga::first_year::{Rite, UnaBar};
///
/// // Only an ūna rite's day carries bars.
/// let una = [Rite::UnaMasika, Rite::UnaShanmasika, Rite::UnaAbdika];
/// assert!(schedule.rites.iter().all(|r| r.bars.is_empty() || una.contains(&r.rite)));
/// assert_eq!(serde_json::to_string(&UnaBar::Chaturdashi)?, r#""CHATURDASHI""#);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum UnaBar {
    /// A nandā tithi: the 1st, 6th or 11th of either half.
    Nanda,
    /// Friday, Bhrigu's day.
    Friday,
    /// The 14th of either half.
    Chaturdashi,
    /// Krittika.
    Krittika,
}

/// One rite of the first year.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # use teistro_panchanga::first_year::FirstYear;
/// #
/// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
/// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
/// # let death = JulianDay::literal(sunrise.get() + 0.25);
/// # let schedule = almanac.first_year(&FirstYear::dharmasindhu("parent", death), &kathmandu)?.value;
///
/// use std::collections::BTreeSet;
///
/// // The sixteen, each in its place.
/// let places: BTreeSet<u8> = schedule.rites.iter().filter_map(|r| r.place).collect();
/// assert_eq!(places, (1..=16).collect());
/// assert!(schedule.rites.iter().all(|r| !r.clause.is_empty()));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FirstYearRite {
    /// Which rite.
    pub rite: Rite,
    /// **Its place in DS's sixteen**, 1 for the first monthly to 16 for
    /// the ūnābdika; a doubled month's two monthlies share it. None for
    /// the abdika and the sapindana, which are not among them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<u8>,
    /// **The month a monthly closes**, 1 for the first to 12; None for
    /// every other rite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<u8>,
    /// **Which yearly shraddha**, 1 for the first; None for every other
    /// rite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u8>,
    /// Whether it is kept in an adhika lunation: the first of a doubled
    /// month's two monthlies.
    pub adhika: bool,
    /// Its own day.
    pub day: CalendarDate,
    /// How it is performed on that day; none for the sapindana.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form: Option<RiteForm>,
    /// The sapindana's day, when the rite is first done there by the
    /// ekoddishta and again on its own day by the parvana.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drawn_to: Option<CalendarDate>,
    /// What DS bars an *ūna* rite's day for; empty for every other rite.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bars: Vec<UnaBar>,
    /// The festival evaluator's judgement of a day kept on a tithi; none
    /// for one counted in days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observance: Option<Observance>,
    /// Where the rule is stated.
    pub clause: String,
}

/// A death's first year.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # use teistro_panchanga::first_year::FirstYear;
/// #
/// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
/// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
/// # let death = JulianDay::literal(sunrise.get() + 0.25);
/// # let schedule = almanac.first_year(&FirstYear::dharmasindhu("parent", death), &kathmandu)?.value;
///
/// use teistro_calendar::CalendarSystem;
///
/// // The eleventh day is the count's first plus ten.
/// let day = |date| Gregorian.fixed_of(date);
/// assert_eq!(day(&schedule.day_one)?.plus_days(10), day(&schedule.eleventh_day)?);
/// assert_eq!(schedule.key, "parent");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FirstYearSchedule {
    /// The request's key.
    pub key: String,
    /// The death's lunar date.
    pub origin: OriginFound,
    /// The first day of the count, as the ashauch's.
    pub day_one: CalendarDate,
    /// Why it is that day.
    pub day_one_by: DayOneBy,
    /// The eleventh day.
    pub eleventh_day: CalendarDate,
    /// The sapindana's day.
    pub sapindana: CalendarDate,
    /// Every rite, in the order of their own days, then of DS's sixteen.
    pub rites: Vec<FirstYearRite>,
}

impl FirstYearSchedule {
    /// Where the answer names catalogue members, and of which kind: a
    /// dotted path from the schedule. What a boundary writes in full.
    pub const MEMBERS: [(&'static str, Kind); 24] = [
        ("origin.tithi", Kind::Tithi),
        ("origin.month", Kind::Masa),
        ("origin.nakshatra", Kind::Nakshatra),
        ("origin.candidates", Kind::Tithi),
        ("dayOne.calendar", Kind::Calendar),
        ("dayOne.era.era", Kind::Era),
        ("eleventhDay.calendar", Kind::Calendar),
        ("eleventhDay.era.era", Kind::Era),
        ("sapindana.calendar", Kind::Calendar),
        ("sapindana.era.era", Kind::Era),
        ("rites.day.calendar", Kind::Calendar),
        ("rites.day.era.era", Kind::Era),
        ("rites.drawnTo.calendar", Kind::Calendar),
        ("rites.drawnTo.era.era", Kind::Era),
        ("rites.observance.day.calendar", Kind::Calendar),
        ("rites.observance.day.era.era", Kind::Era),
        ("rites.observance.month", Kind::Masa),
        ("rites.observance.extents.day.calendar", Kind::Calendar),
        ("rites.observance.extents.day.era.era", Kind::Era),
        ("rites.observance.parana.day.calendar", Kind::Calendar),
        ("rites.observance.parana.day.era.era", Kind::Era),
        ("rites.observance.parana.reading.nakshatra", Kind::Nakshatra),
        ("rites.observance.parana.reading.tithi", Kind::Tithi),
        (
            "rites.observance.decidedBy.guard.nakshatra",
            Kind::Nakshatra,
        ),
    ];

    /// The answer as JSON, every catalogue member written as its full key.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
    /// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// # let provider = TestProvider;
    /// # let resolved = Profile::shipped(DEFAULT_PROFILE)
    /// #     .ok_or("a shipped profile")?
    /// #     .resolve(&SettingsPatch::default())
    /// #     .map_err(|d| format!("{d:?}"))?;
    /// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
    /// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
    /// # let clock = UtcOffset::literal(5, 45, 0);
    /// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # use teistro_panchanga::first_year::FirstYear;
    /// #
    /// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
    /// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
    /// # let death = JulianDay::literal(sunrise.get() + 0.25);
    /// # let schedule = almanac.first_year(&FirstYear::dharmasindhu("parent", death), &kathmandu)?.value;
    ///
    /// let json = schedule.in_full()?;
    /// assert!(json["origin"]["tithi"].as_str().is_some_and(|key| key.starts_with("tithi.")));
    /// assert_eq!(json["dayOne"]["calendar"], "calendar.GREGORIAN");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An internal error if the answer does not serialise.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self)
            .map_err(|why| Error::internal(format!("a first year did not serialise: {why}")))?;
        for (path, kind) in Self::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

/// One rite as found under one form, before the sapindana sorts them.
struct Found {
    rite: Rite,
    place: Option<u8>,
    month: Option<u8>,
    year: Option<u8>,
    adhika: bool,
    index: usize,
    observance: Option<Observance>,
    clause: &'static str,
}

/// The run's index of a date.
fn index_of(days: &[FestivalDay], date: &CalendarDate) -> Result<usize, Error> {
    days.iter()
        .position(|day| day.date == *date)
        .ok_or_else(|| short("an observance's day"))
}

fn short(what: &str) -> Error {
    Error::internal(format!(
        "the first year's run of days does not hold {what}; build it over FirstYear::reach"
    ))
}

/// A tithi's occurrences after `at`, judged at `window`, every lunation.
fn every(
    tithi: Tithi,
    window: Window,
    at: JulianDay<Utc>,
    days: &[FestivalDay],
) -> Result<Vec<Observance>, Error> {
    let (decide, otherwise) = by_share();
    let rule = FestivalRule {
        key: String::from("FIRST_YEAR"),
        source: String::from("Dharmasindhu p. 330"),
        month: None,
        convention: Convention::Amanta,
        occurs: tithi.into(),
        in_adhika: InAdhika::Also,
        at: window,
        decide,
        otherwise,
        parana: Vec::new(),
    };
    let found = observances(core::slice::from_ref(&rule), days)?;
    if let Some(unjudged) = found
        .unjudged
        .iter()
        .find(|unjudged| unjudged.tithi.from.get() > at.get())
    {
        return Err(Error::new(
            Status::OutOfRange,
            format!(
                "the first year's {tithi:?} from {} could not be judged: {}",
                unjudged.tithi.from.get(),
                unjudged.why
            ),
        ));
    }
    Ok(found
        .observances
        .into_iter()
        .filter(|observance| observance.tithi.from.get() > at.get())
        .collect())
}

/// The tithi `ahead` places on from `tithi`, around the circle.
fn ahead(tithi: Tithi, ahead: u32) -> Tithi {
    Tithi::of(u32::from(tithi.id()) + ahead)
}

/// What DS bars on a day, at its sunrise.
fn bars(day: &FestivalDay) -> Vec<UnaBar> {
    let at = day.sunrise.get();
    let tithi = day
        .tithi
        .iter()
        .find(|span| span.whole.from.get() <= at && at < span.whole.to.get())
        .map(|span| span.member);
    let nakshatra = day
        .nakshatra
        .iter()
        .find(|span| span.whole.from.get() <= at && at < span.whole.to.get())
        .map(|span| span.member);
    let number = tithi.map(|tithi| tithi.id() % 15 + 1);
    let mut found = Vec::new();
    if matches!(number, Some(1 | 6 | 11)) {
        found.push(UnaBar::Nanda);
    }
    if day.vara == Vara::Shukravara {
        found.push(UnaBar::Friday);
    }
    if number == Some(14) {
        found.push(UnaBar::Chaturdashi);
    }
    if nakshatra == Some(Nakshatra::Krittika) {
        found.push(UnaBar::Krittika);
    }
    found
}

const ADYA: &str =
    "Dharmasindhu p. 330: the first monthly on the eleventh day, \"आद्यं स्यादेकादशेऽहनि\"";
const MASIKA: &str = "Dharmasindhu p. 330: each monthly at its month's start, on the death tithi; p. 331: a month doubled in the year, its monthly twice";
const TRAIPAKSHIKA: &str = "Dharmasindhu p. 330: the traipakshika on the death tithi three fortnights on, \"त्रैपक्षिकं त्रिपक्षे अतीते मृताहे\"";
const UNA_BEFORE: &str = "Dharmasindhu p. 330: Madhava's ūna rite on the day before the death tithi's, \"मृताहात् पूर्वेद्युः\"";
const UNA_TITHIS: &str = "Dharmasindhu p. 330: the ūna rite one, two or three tithis short";
const ABDIKA: &str =
    "Dharmasindhu p. 330: the first abdika at the second year's start, \"आब्दिकं वत्सरे गते\"";
const ABDIKA_ELEVENTH: &str = "Dharmasindhu p. 330 (and Dhakal for Nepal): the first abdika on the eleventh day with the first monthly";
const SAPINDANA: &str = "Dharmasindhu p. 331: the sapindana, the sixteen drawn forward to it by the ekoddishta and those after it done again in their own times by the parvana";

impl Found {
    /// A rite on the run's `index`, outside the sixteen and unnumbered.
    const fn at(rite: Rite, index: usize, clause: &'static str) -> Found {
        Found {
            rite,
            place: None,
            month: None,
            year: None,
            adhika: false,
            index,
            observance: None,
            clause,
        }
    }

    /// A rite kept on a tithi's judged day.
    fn kept(
        rite: Rite,
        observance: Observance,
        days: &[FestivalDay],
        clause: &'static str,
    ) -> Result<Found, Error> {
        let index = index_of(days, &observance.day)?;
        Ok(Found {
            adhika: observance.adhika,
            observance: Some(observance),
            ..Found::at(rite, index, clause)
        })
    }
}

/// DS's place of a month's monthly among the sixteen.
const fn place(month: u8) -> u8 {
    match month {
        2 => 3,
        3..=6 => month + 2,
        _ => month + 3,
    }
}

/// One form's reading of the year: its tithis judged at one window.
struct Reading<'r> {
    asked: &'r FirstYear,
    days: &'r [FestivalDay],
    tithi: Tithi,
    window: Window,
}

/// The schedule's rites under one form, and the two days a sapindana can
/// be kept on besides the eleventh and twelfth.
struct Pass {
    rites: Vec<Found>,
    traipakshika: usize,
    anniversary: usize,
}

impl Reading<'_> {
    /// The tithi `ahead` places on from the death's, every lunation after
    /// the death.
    fn every(&self, ahead_by: u32) -> Result<Vec<Observance>, Error> {
        every(
            ahead(self.tithi, ahead_by),
            self.window,
            self.asked.at,
            self.days,
        )
    }

    /// The monthlies, numbered 2 to 12: a lunation after an adhika one is
    /// the doubled month's second, and keeps the number.
    fn monthlies(&self) -> Result<Vec<(u8, Observance)>, Error> {
        let mut month = 1;
        let mut after_adhika = false;
        let mut monthly = Vec::new();
        for observance in self.every(0)? {
            if !after_adhika {
                month += 1;
            }
            if month > LAST_MONTHLY {
                break;
            }
            after_adhika = observance.adhika;
            monthly.push((month, observance));
        }
        if monthly.last().map(|(month, _)| *month) != Some(LAST_MONTHLY) {
            return Err(short("the twelfth monthly"));
        }
        Ok(monthly)
    }

    /// An ūna rite short of `of`: the day before, or the tithi so many
    /// short at its last occurrence before.
    fn una(&self, rite: Rite, place: u8, of: &Observance) -> Result<Found, Error> {
        let found = match self.asked.una.tithis() {
            None => {
                let index = index_of(self.days, &of.day)?
                    .checked_sub(1)
                    .ok_or_else(|| short("an ūna rite"))?;
                Found::at(rite, index, UNA_BEFORE)
            }
            Some(short_by) => {
                let found = self
                    .every(30 - short_by)?
                    .into_iter()
                    .rfind(|observance| observance.tithi.from.get() < of.tithi.from.get())
                    .ok_or_else(|| short("an ūna rite's tithi"))?;
                Found {
                    adhika: false,
                    ..Found::kept(rite, found, self.days, UNA_TITHIS)?
                }
            }
        };
        Ok(Found {
            place: Some(place),
            ..found
        })
    }

    /// Every rite, the anniversary judged as `anniversary` asks.
    fn pass(
        &self,
        anniversary: &Anniversary,
        origin: &OriginFound,
        eleventh: usize,
    ) -> Result<Pass, Error> {
        let mut rites = vec![Found {
            place: Some(1),
            month: Some(1),
            ..Found::at(Rite::AdyaMasika, eleventh, ADYA)
        }];
        let first_abdika_eleventh = self.asked.first_abdika == FirstAbdika::EleventhDay;
        if first_abdika_eleventh {
            rites.push(Found {
                year: Some(1),
                ..Found::at(Rite::Abdika, eleventh, ABDIKA_ELEVENTH)
            });
        }
        let monthly = self.monthlies()?;
        let first_of = |wanted: u8| {
            monthly
                .iter()
                .find(|(month, _)| *month == wanted)
                .map(|(_, observance)| observance)
                .ok_or_else(|| short("a monthly"))
        };
        let traipakshika = self
            .every(15)?
            .into_iter()
            .nth(1)
            .ok_or_else(|| short("the traipakshika"))?;
        let anniversary = anniversaries(&[(anniversary.clone(), origin.clone())], self.days, &[])?
            .anniversaries
            .into_iter()
            .find(|kept| kept.count == Some(1))
            .ok_or_else(|| short("the first anniversary"))?
            .observance;
        rites.push(self.una(Rite::UnaMasika, 2, first_of(2)?)?);
        let traipakshika = Found {
            place: Some(4),
            ..Found::kept(Rite::Traipakshika, traipakshika, self.days, TRAIPAKSHIKA)?
        };
        let traipakshika_index = traipakshika.index;
        rites.push(traipakshika);
        rites.push(self.una(Rite::UnaShanmasika, 9, first_of(7)?)?);
        rites.push(self.una(Rite::UnaAbdika, 16, &anniversary)?);
        for (month, observance) in monthly {
            rites.push(Found {
                place: Some(place(month)),
                month: Some(month),
                ..Found::kept(Rite::Masika, observance, self.days, MASIKA)?
            });
        }
        let anniversary = Found {
            year: Some(if first_abdika_eleventh { 2 } else { 1 }),
            ..Found::kept(Rite::Abdika, anniversary, self.days, ABDIKA)?
        };
        let anniversary_index = anniversary.index;
        rites.push(anniversary);
        Ok(Pass {
            rites,
            traipakshika: traipakshika_index,
            anniversary: anniversary_index,
        })
    }
}

/// A death's first year over consecutive sunrise days holding
/// [`FirstYear::reach`] around it, its origin already found at the
/// instant.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # use teistro_panchanga::first_year::FirstYear;
/// #
/// # // A death at noon on Wednesday 25 November 2026, in Kathmandu.
/// # let sunrise = almanac.day(&date(11, 25), &kathmandu)?.value.day.sunrise;
/// # let death = JulianDay::literal(sunrise.get() + 0.25);
///
/// use teistro_calendar::CalendarSystem;
/// use teistro_panchanga::festival::Origin;
/// use teistro_panchanga::first_year::first_year;
///
/// // The almanac's own steps: the origin at the death, then the days
/// // from two before it to the reach after.
/// let asked = FirstYear::dharmasindhu("parent", death);
/// let origin = almanac.origin(&Origin::Instant { at: death }, &kathmandu)?;
/// let (back, ahead) = FirstYear::reach();
/// let civil = Gregorian.fixed_of(&date(11, 25))?;
/// // A run holds at most a year, so the reach is founded in two that join.
/// let middle = ahead / 2;
/// let run = |from: i64, to: i64| -> Result<_, teistro_core::error::Error> {
///     let (from, to) = (Gregorian.date_of(civil.plus_days(from))?, Gregorian.date_of(civil.plus_days(to))?);
///     Ok(almanac.festival_days(&from, &to, &kathmandu)?.value)
/// };
/// let mut days = run(-back, middle)?;
/// days.extend(run(middle + 1, ahead)?);
/// let schedule = first_year(&asked, &origin, &days, &clock)?;
/// assert_eq!(schedule, almanac.first_year(&asked, &kathmandu)?.value);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// A request [`FirstYear::check`] refuses; a death on a polar day, or a
/// rite's occurrence the evaluator cannot judge (`OUT_OF_RANGE`); a run
/// that does not hold the year (internal).
pub fn first_year(
    asked: &FirstYear,
    origin: &OriginFound,
    days: &[FestivalDay],
    clock: &dyn LocalClock,
) -> Result<FirstYearSchedule, Error> {
    asked.check()?;
    let (first, day_one_by) = ashauch::day_one(days, asked.at, asked.night, clock)?;
    let anniversary = Anniversary {
        key: asked.key.clone(),
        origin: Origin::Instant { at: asked.at },
        kind: AnniversaryKind::Shraddha,
        window: asked.window,
        first_year_adhika: asked.first_year_adhika,
        // The first abdika is one rite, which `first_year_adhika` places.
        in_samsarpa: InSamsarpa::Nija,
        janma: JanmaOptions::default(),
    };
    let eleventh = index_of(days, &ashauch::day_after(days, first, 10)?.date)?;
    let reading = |form| Reading {
        asked,
        days,
        tithi: origin.tithi,
        window: asked.window(form),
    };
    let before = reading(RiteForm::Ekoddishta).pass(&anniversary, origin, eleventh)?;
    let sapindana = match asked.sapindana {
        SapindanaAt::EleventhDay => eleventh,
        SapindanaAt::TwelfthDay => eleventh + 1,
        SapindanaAt::Traipakshika => before.traipakshika,
        SapindanaAt::YearEnd => before.anniversary,
    };
    let after = reading(RiteForm::Parvana).pass(&anniversary, origin, eleventh)?;
    let kept = match anniversary.kept_as(origin) {
        AnniversaryKind::Parvana => RiteForm::Parvana,
        _ => RiteForm::Ekoddishta,
    };
    let date = |index: usize| {
        days.get(index)
            .map(|day| day.date.clone())
            .ok_or_else(|| short("a rite's day"))
    };
    let sapindana_day = date(sapindana)?;
    let mut rites = Vec::with_capacity(before.rites.len() + 1);
    for (early, late) in before.rites.into_iter().zip(after.rites) {
        // The anniversary keeps its own form (§3.1, C665); the sixteen,
        // and an abdika kept with the first of them, are drawn forward
        // when on or after the sapindana's day.
        let (found, form, drawn_to) = if early.rite == Rite::Abdika && early.observance.is_some() {
            (early, kept, None)
        } else if early.index >= sapindana {
            (late, RiteForm::Parvana, Some(sapindana_day.clone()))
        } else {
            (early, RiteForm::Ekoddishta, None)
        };
        let day = days.get(found.index).ok_or_else(|| short("a rite's day"))?;
        let una = matches!(
            found.rite,
            Rite::UnaMasika | Rite::UnaShanmasika | Rite::UnaAbdika
        );
        rites.push((
            found.index,
            FirstYearRite {
                rite: found.rite,
                place: found.place,
                month: found.month,
                year: found.year,
                adhika: found.adhika,
                day: day.date.clone(),
                form: Some(form),
                drawn_to,
                bars: if una { bars(day) } else { Vec::new() },
                observance: found.observance,
                clause: found.clause.to_owned(),
            },
        ));
    }
    rites.push((
        sapindana,
        FirstYearRite {
            rite: Rite::Sapindana,
            place: None,
            month: None,
            year: None,
            adhika: false,
            day: sapindana_day.clone(),
            form: None,
            drawn_to: None,
            bars: Vec::new(),
            observance: None,
            clause: SAPINDANA.to_owned(),
        },
    ));
    rites.sort_by_key(|(index, rite)| (*index, rite.place.unwrap_or(u8::MAX)));
    Ok(FirstYearSchedule {
        key: asked.key.clone(),
        origin: origin.clone(),
        day_one: date(first)?,
        day_one_by,
        eleventh_day: date(eleventh)?,
        sapindana: sapindana_day,
        rites: rites.into_iter().map(|(_, rite)| rite).collect(),
    })
}
