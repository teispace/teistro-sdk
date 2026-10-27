//! Observances kept on a date of a calendar rather than a tithi
//! (`03-design/festival-rule-language.md` §3.6).
//!
//! Nepal's committee prints the sankrantis on the first day of each
//! Bikram Sambat month, whose months are the Sun's signs under the punya
//! convention the calendar already applies (C619), and the national days
//! on fixed Bikram Sambat dates: पृथ्वीजयन्ती on Poush 27, the earthquake
//! safety day on Magh 2 (VS 2082 p. 20). A solar day is so a civil date in
//! a solar calendar, and two forms say every such rule: a day of a month,
//! counted back from its end when negative, and the nth weekday of a
//! month.
//!
//! The churches' moveable feasts are kept by a count from Easter Sunday,
//! which the computus dates in its own calendar (`teistro_calendar::easter`):
//! Good Friday two days before it, Pentecost forty-nine after.

use serde::{Deserialize, Serialize};
use teistro_calendar::easter::{Computus, easter};
use teistro_calendar::gregorian::gregorian_from_fixed;
use teistro_calendar::tibetan::{self, SkippedDay, TIBETAN_RANGE};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay, shipped};
use teistro_core::catalogue::{Calendar, Nakshatra, Vara};
use teistro_core::error::Error;

use super::{FestivalDay, Observances, SunEntry};

/// The festivals Nepal's national panchanga prints on a day of a Bikram
/// Sambat month, each read off both editions with its continuation
/// letters followed: key, month, day (negative counts back from the
/// month's last), and where the day is printed. Each also gives every day
/// a patro printed in Nepal verified for it, by a back-test kept outside
/// the repository (`festival-classification.md`).
const SOLAR_DAYS: [(&str, u8, i8, &str); 31] = [
    (
        "TOKHA_SAPAN_TIRTHA_SNAN",
        1,
        1,
        "VS 2082 p. 2 and VS 2083 p. 2: \"टोखा सपन तीर्थस्नान\" on Baisakh 1",
    ),
    (
        "YOSIN_PATAN",
        1,
        1,
        "VS 2082 p. 2 and VS 2083 p. 2: \"भ.पु विश्वध्वजपतन\" on Baisakh 1",
    ),
    (
        "BHAKTAPUR_MAHAKALI_MAHALAXMI_JATRA",
        1,
        2,
        "VS 2082 p. 2 and VS 2083 p. 2: \"भ.पु.महाकाली महालक्ष्मी यात्रा\" on Baisakh 2",
    ),
    (
        "TOKHA_CHANDESHWARI_JATRA",
        1,
        2,
        "VS 2082 p. 2 and VS 2083 p. 2: \"टोखा चण्डेश्वरी मूलरथयात्रा\" on Baisakh 2",
    ),
    (
        "THIMI_BALKUMARI_JATRA",
        1,
        2,
        "VS 2082 p. 2 and VS 2083 p. 2: \"मध्यपुरठिमी बालकुमारी यात्रा\" on Baisakh 2, continuing its row",
    ),
    (
        "BHAKTAPUR_BRAHMAYANI_JATRA",
        1,
        3,
        "VS 2082 p. 2 and VS 2083 p. 2: \"भ.पु.ब्रह्मायणीयात्रा\" on Baisakh 3",
    ),
    (
        "BHAKTAPUR_CHHUMA_GANESH_JATRA",
        1,
        4,
        "VS 2082 p. 2 and VS 2083 p. 2: \"भ.पु छुमा गणेशयात्रा\" on Baisakh 4",
    ),
    (
        "BISKET_JATRA_ENDS",
        1,
        5,
        "VS 2082 p. 2 and VS 2083 p. 3: \"भ.पु.भैरवभद्रकाली रथयात्रा समापन\" on Baisakh 5",
    ),
    (
        "KANDAKARAK_PUJA",
        4,
        1,
        "VS 2082 p. 8 and VS 2083 p. 9: \"कण्डारकपूजा\" on Shrawan 1, the day of श्रावणसङ्क्रान्ति",
    ),
    (
        "KHEER_KHANE_DIN",
        4,
        15,
        "VS 2082 p. 9 and VS 2083 p. 10: \"खीरखाने दिन\" on Shrawan 15",
    ),
    (
        "KIRTIPUR_BAGH_BHAIRAV_JATRA",
        5,
        1,
        "VS 2082 p. 10 and VS 2083 p. 11: \"कीर्तिपुरमा बाघभैरवयात्रा\" on Bhadra 1, continuing its row",
    ),
    (
        "VISHWAKARMA_PUJA",
        6,
        1,
        "VS 2082 p. 12 and VS 2083 p. 13: \"विश्वकर्मापूजा\" on Asoj 1, the day of आश्विनसङ्क्रान्ति",
    ),
    (
        "MAGHI_THARU",
        10,
        1,
        "VS 2082 p. 20 and VS 2083 p. 21: \"माघीपर्व\" on Magh 1, the day of माघसङ्क्रान्ति",
    ),
    (
        "MAKAR_SNAN",
        10,
        1,
        "VS 2082 p. 20 and VS 2083 p. 21: \"मकरस्नान\" on Magh 1, continuing its row",
    ),
    (
        "DEVGHAT_MAKAR_SNAN",
        10,
        1,
        "VS 2082 p. 20 and VS 2083 p. 21: \"हरिहरक्षेत्र देवघाटस्थित प्रयागमा\" on Magh 1, continuing its row",
    ),
    (
        "BISKET_JATRA_BEGINS",
        12,
        -4,
        "VS 2082 p. 26 and VS 2083 p. 27: \"भ.पु.भैरवभद्रकाली रथयात्रारम्भ\" on Chaitra's fourth-last day, its 27th of 30",
    ),
    (
        "CHHANDA_DIWAS",
        1,
        8,
        "VS 2082 p. 2 and VS 2083 p. 3: \"छन्द दिवस\" on Baisakh 8",
    ),
    (
        "KIRAT_SAMAJ_SUDHAR_DIWAS",
        1,
        24,
        "VS 2082 p. 3 and VS 2083 p. 4: \"किराँत समाजसुधार दिवस\" on Baisakh 24",
    ),
    (
        "LAW_DAY",
        1,
        26,
        "VS 2082 p. 3 and VS 2083 p. 4: \"कानुनदिवस\" on Baisakh 26",
    ),
    (
        "REPUBLIC_DAY",
        2,
        15,
        "VS 2082 p. 5 and VS 2083 p. 5: \"गणतन्त्र दिवस\" on Jestha 15",
    ),
    (
        "CASTE_BASED_DISCRIMINATION_AND_UNTOUCHABILITY_ELIMINATION_DAY",
        2,
        21,
        "VS 2082 p. 5 and VS 2083 p. 6: \"जातीय भेदभाव तथा छुवाछुत उन्मूलन राष्ट्रिय दिवस\" on Jestha 21",
    ),
    (
        "NATIONAL_PADDY_DAY",
        3,
        15,
        "VS 2082 p. 7 and VS 2083 p. 7: \"धानदिवस\" on Asar 15",
    ),
    (
        "BHANU_JAYANTI",
        3,
        29,
        "VS 2082 p. 8 and VS 2083 p. 8: \"भानुजयन्ती\" on Asar 29",
    ),
    (
        "NATIONAL_DAY_AGAINST_HUMAN_TRAFFICKING",
        5,
        20,
        "VS 2082 p. 11 and VS 2083 p. 12: \"मानव बेचबिखन विरुद्धको राष्ट्रिय दिवस\" on Bhadra 20",
    ),
    (
        "CIVIL_SERVICE_DAY",
        5,
        22,
        "VS 2082 p. 11 and VS 2083 p. 12: \"निजामती सेवादिवस\" on Bhadra 22",
    ),
    (
        "NATIONAL_CHILDREN_S_DAY",
        5,
        29,
        "VS 2082 p. 12 and VS 2083 p. 13: \"बालदिवस\" on Bhadra 29",
    ),
    (
        "VASTU_DIWAS",
        6,
        1,
        "VS 2082 p. 12 and VS 2083 p. 13: \"वास्तुदिवस\" on Asoj 1",
    ),
    (
        "FALGUNANDA_JAYANTI",
        7,
        25,
        "VS 2082 p. 16 and VS 2083 p. 17: \"फाल्गुनन्दजयन्ती\" on Kartik 25",
    ),
    (
        "LEKHNATH_PAUDYAL_JAYANTI",
        9,
        15,
        "VS 2082 p. 19 and VS 2083 p. 20: \"कवि शिरोमणि लेखनाथ जयन्ती\" on Poush 15",
    ),
    (
        "ARANIKO_MEMORIAL_DAY",
        9,
        23,
        "VS 2082 p. 20: \"अरनिको स्मृति दिवस\", and VS 2083 p. 20: \"अरनिको स्मृतिदिवस\", continuing its row, on Poush 23",
    ),
    (
        "ELECTION_DAY",
        11,
        7,
        "VS 2082 p. 23 and VS 2083 p. 23: \"निर्वाचन दिवस\" on Phalgun 7",
    ),
];

/// The days Nepal's national panchanga prints on a Gregorian date, each
/// on the same date in both editions and on every day a patro printed in
/// Nepal verified for it: key, month, day, and where the day is printed.
const GREGORIAN_DAYS: [(&str, u8, i8, &str); 20] = [
    (
        "NEW_YEAR_S_DAY_AD",
        1,
        1,
        "VS 2082 p. 19: \"सन् २०२६ प्रारम्भ\", and VS 2083 p. 20: \"सन् २०२७ प्रारम्भ\", on 1 January",
    ),
    (
        "INTERNATIONAL_WOMEN_S_DAY",
        3,
        8,
        "VS 2082 p. 24 and VS 2083 p. 24: \"अन्तर्राष्ट्रिय नारी दिवस\" on 8 March",
    ),
    (
        "WORLD_WATER_DAY",
        3,
        22,
        "VS 2082 p. 25 and VS 2083 p. 25: \"जलदिवस\" on 22 March",
    ),
    (
        "WORLD_TUBERCULOSIS_DAY",
        3,
        24,
        "VS 2082 p. 25: \"विश्व क्षयरोगदिवस\", and VS 2083 p. 26: \"विश्वक्षयरोग दिवस\", on 24 March",
    ),
    (
        "INTERNATIONAL_LABOUR_DAY",
        5,
        1,
        "VS 2082 p. 3: \"मजदूरदिवस\", and VS 2083 p. 3: \"मजदूर दिवस\", on 1 May",
    ),
    (
        "WORLD_PRESS_FREEDOM_DAY",
        5,
        3,
        "VS 2082 p. 3 and VS 2083 p. 4: \"अन्तर्राष्ट्रिय प्रेस स्वतन्त्रता दिवस\" on 3 May",
    ),
    (
        "WORLD_RED_CROSS_DAY",
        5,
        8,
        "VS 2082 p. 3 and VS 2083 p. 4: \"रेडक्रस दिवस\" on 8 May",
    ),
    (
        "WORLD_NO_TOBACCO_DAY",
        5,
        31,
        "VS 2082 p. 5: \"धूम्रपानरहितदिवस\", and VS 2083 p. 5: \"धूम्रपानरहित दिवस\", on 31 May",
    ),
    (
        "WORLD_ENVIRONMENT_DAY",
        6,
        5,
        "VS 2082 p. 5: \"विश्व वातावरण दिवस\", continuing its row, and VS 2083 p. 6: \"विश्व वातावरण दिवस\", on 5 June",
    ),
    (
        "INTERNATIONAL_SPORTS_JOURNALISTS_DAY",
        7,
        2,
        "VS 2082 p. 7 and VS 2083 p. 8: \"खेलकूद पत्रकार दिवस\" on 2 July",
    ),
    (
        "WORLD_POPULATION_DAY",
        7,
        11,
        "VS 2082 p. 8 and VS 2083 p. 8: \"विश्व जनसङ्ख्या दिवस\" on 11 July",
    ),
    (
        "INTERNATIONAL_DAY_OF_THE_WORLD_S_INDIGENOUS_PEOPLES",
        8,
        9,
        "VS 2082 p. 9: \"विश्व आदिवासी दिवस\", continuing its row, and VS 2083 p. 10: \"विश्व आदिबासी दिवस\", on 9 August",
    ),
    (
        "INTERNATIONAL_LITERACY_DAY",
        9,
        8,
        "VS 2082 p. 12 and VS 2083 p. 12: \"शिक्षा तथा साक्षरता दिवस\" on 8 September",
    ),
    (
        "INTERNATIONAL_DAY_OF_PEACE",
        9,
        21,
        "VS 2082 p. 12 and VS 2083 p. 13: \"विश्वशान्तिदिवस\" on 21 September",
    ),
    (
        "WORLD_TOURISM_DAY",
        9,
        27,
        "VS 2082 p. 13 and VS 2083 p. 14: \"विश्व पर्यटन दिवस\" on 27 September",
    ),
    (
        "UNITED_NATIONS_DAY",
        10,
        24,
        "VS 2082 p. 15 and VS 2083 p. 15: \"संयुक्तराष्ट्रसंघ दिवस\" on 24 October",
    ),
    (
        "WORLD_DIABETES_DAY",
        11,
        14,
        "VS 2082 p. 16 and VS 2083 p. 17: \"विश्व मधुमेह दिवस\" on 14 November",
    ),
    (
        "INTERNATIONAL_DAY_OF_PERSONS_WITH_DISABILITIES",
        12,
        3,
        "VS 2082 p. 17 and VS 2083 p. 18: \"विश्व अपाङ्ग दिवस\" on 3 December",
    ),
    (
        "HUMAN_RIGHTS_DAY",
        12,
        10,
        "VS 2082 p. 18: \"मानवअधिकार दिवस\", and VS 2083 p. 19: \"मानव अधिकार दिवस\", on 10 December",
    ),
    (
        "CHRISTMAS",
        12,
        25,
        "VS 2082 p. 19 and VS 2083 p. 20: \"क्रिसमस डे\" on 25 December",
    ),
];

/// An observance on a date of a calendar.
///
/// ```
/// use teistro_panchanga::festival::{Dated, DatedRule};
/// use teistro_core::catalogue::Calendar;
///
/// let maghe = DatedRule::nepal_panchanga().into_iter().find(|rule| rule.key == "MAKARA_SANKRANTI").ok_or("shipped")?;
/// assert_eq!(maghe.on, Dated::CivilDate { calendar: Calendar::BikramSambat, month: 10, day: 1, skipped: None });
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(transform = super::variants_as_fields))]
#[serde(rename_all = "camelCase")]
pub struct DatedRule {
    /// The rule's key in its pack, `"MAKARA_SANKRANTI"`.
    pub key: String,
    /// Where the rule is stated.
    pub source: String,
    /// The date, written as one of [`Dated::KEYS`] beside the key.
    #[serde(flatten)]
    pub on: Dated,
}

/// The date a [`DatedRule`] keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Dated {
    /// A day of a month of `calendar`, 1 for the first; a negative day
    /// counts back from the month's end, -1 for its last.
    CivilDate {
        /// The calendar the date is in.
        calendar: Calendar,
        /// The month, 1 for the first.
        month: u8,
        /// The day; never 0.
        day: i8,
        /// Which neighbour keeps the date when its number is skipped, in
        /// the calendar that skips numbers, the Tibetan; left out, the
        /// day before ([`SkippedDay`]). Refused for any other calendar.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skipped: Option<SkippedDay>,
    },
    /// The `nth` `weekday` of a month of `calendar`; a negative `nth`
    /// counts back from the month's end, -1 for its last.
    CivilWeekday {
        /// The calendar the month is in.
        calendar: Calendar,
        /// The month, 1 for the first.
        month: u8,
        /// The weekday.
        weekday: Vara,
        /// Which of them; never 0, at most 5 either way.
        nth: i8,
    },
    /// The civil day, local midnight to the next, in which the sidereal
    /// Sun enters `nakshatra` in the almanac's own sky, or the day `days`
    /// civil days after it: भूमिरज on the Sun's entry into Ardra, and the
    /// भूमि पूजा three days on (C696).
    SunEnters {
        /// The nakshatra entered.
        nakshatra: Nakshatra,
        /// How many civil days after the entry's, 0 for that day, at most
        /// [`Dated::MOST_DAYS`].
        #[serde(default)]
        days: u8,
    },
    /// The day `days` days after Easter Sunday by `computus`, before it
    /// when negative: Good Friday at -2, Pentecost at 49. A year the
    /// computus does not reckon (the Gregorian before 1583) keeps none.
    Easter {
        /// Which churches' reckoning of Easter.
        computus: Computus,
        /// The days from Easter Sunday, 0 for that day, at most
        /// [`Dated::EASTER_DAYS`] either way.
        #[serde(default)]
        days: i16,
    },
}

impl Dated {
    /// The key each kind of date is written under, in the order of the
    /// variants: a [`DatedRule`] gives exactly one.
    pub const KEYS: [&'static str; 4] = ["civilDate", "civilWeekday", "sunEnters", "easter"];

    /// The most days a rule counts from the Sun's entry: a fortnight,
    /// which is about the Sun's stay in a nakshatra, past which the next
    /// entry names the day better than a count.
    pub const MOST_DAYS: u8 = 15;

    /// The most days a rule counts from Easter, either way: ten weeks,
    /// which reach the Eastern churches' Sunday of the Publican and the
    /// Pharisee (-70), the Triodion's first, and the Western Sacred Heart
    /// (68), the farthest feasts either calendar keeps by Easter.
    pub const EASTER_DAYS: i16 = 70;

    /// The calendar and month a civil form names; none for the Sun's
    /// entry and for a count from Easter, which no month dates.
    fn civil(self) -> Option<(Calendar, u8)> {
        match self {
            Dated::CivilDate {
                calendar, month, ..
            }
            | Dated::CivilWeekday {
                calendar, month, ..
            } => Some((calendar, month)),
            Dated::SunEnters { .. } | Dated::Easter { .. } => None,
        }
    }

    /// Whether `date`, of a month `length` days long, falling on `vara`,
    /// is the date this names.
    fn holds(self, date: &CalendarDate, length: u8, vara: Vara) -> bool {
        if self.civil().map(|(_, month)| month) != Some(date.month) {
            return false;
        }
        let (day, length) = (i16::from(date.day), i16::from(length));
        match self {
            Dated::CivilDate { day: wanted, .. } => {
                let wanted = i16::from(wanted);
                day == if wanted > 0 {
                    wanted
                } else {
                    length + 1 + wanted
                }
            }
            Dated::CivilWeekday {
                weekday,
                nth: wanted,
                ..
            } => {
                let nth = if wanted > 0 {
                    (day - 1) / 7 + 1
                } else {
                    -((length - day) / 7 + 1)
                };
                vara == weekday && nth == i16::from(wanted)
            }
            Dated::SunEnters { .. } | Dated::Easter { .. } => false,
        }
    }
}

impl DatedRule {
    /// Whether the rule is kept on the Sun's entry into a nakshatra, which
    /// the days are read for only when a rule asks.
    #[must_use]
    pub fn reads_sun_entry(&self) -> bool {
        matches!(self.on, Dated::SunEnters { .. })
    }

    /// How many civil days the rule counts from the day it is kept by,
    /// which the days before a range are founded to reach.
    #[must_use]
    pub fn reach(&self) -> u8 {
        match self.on {
            Dated::SunEnters { days, .. } => days,
            // Easter is reckoned from the year alone, so its count needs
            // no day before the range.
            Dated::CivilDate { .. } | Dated::CivilWeekday { .. } | Dated::Easter { .. } => 0,
        }
    }

    /// Refuses a rule that could not be kept: an empty key, a calendar
    /// the SDK does not ship, month 0, a day of 0 or past 32 either way,
    /// an nth of 0 or past 5 either way, a count from the Sun's entry
    /// past [`Dated::MOST_DAYS`], or one from Easter past
    /// [`Dated::EASTER_DAYS`] either way.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the field that is wrong.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("a festival rule needs a key").with_field("key"));
        }
        let (form, calendar, month) = match self.on {
            Dated::CivilDate {
                calendar, month, ..
            } => ("civilDate", calendar, month),
            Dated::CivilWeekday {
                calendar, month, ..
            } => ("civilWeekday", calendar, month),
            Dated::SunEnters { days, .. } if days > Dated::MOST_DAYS => {
                return Err(Error::invalid_arg(format!(
                    "{}: a rule counts at most {} days from the Sun's entry, not {days}; past a fortnight, name the next entry",
                    self.key,
                    Dated::MOST_DAYS
                ))
                .with_field("sunEnters.days"));
            }
            Dated::Easter { days, .. }
                if days.unsigned_abs() > Dated::EASTER_DAYS.unsigned_abs() =>
            {
                return Err(Error::invalid_arg(format!(
                    "{}: a rule counts at most {} days either way from Easter, not {days}",
                    self.key,
                    Dated::EASTER_DAYS
                ))
                .with_field("easter.days"));
            }
            Dated::SunEnters { .. } | Dated::Easter { .. } => return Ok(()),
        };
        if shipped(calendar).is_none() {
            return Err(Error::invalid_arg(format!(
                "{}: the SDK does not ship the `{}` calendar",
                self.key,
                calendar.key()
            ))
            .with_field(format!("{form}.calendar")));
        }
        let tibetan = calendar == Calendar::TibetanPhugpa;
        // A Tibetan date is kept by `tibetan::observed`; a weekday of a
        // Tibetan month has no source that says which month of a doubled
        // pair, or which weeks, count.
        if tibetan && form == "civilWeekday" {
            return Err(Error::unsupported(format!(
                "{}: a weekday of a Tibetan month is not reckoned; keep the rule on a date (docs/calendars/tibetan.md)",
                self.key
            ))
            .with_field("civilWeekday.calendar"));
        }
        if let Dated::CivilDate {
            skipped: Some(_), ..
        } = self.on
            && !tibetan
        {
            return Err(Error::invalid_arg(format!(
                "{}: `{}` numbers no day twice and skips none, so a skipped date needs no neighbour",
                self.key,
                calendar.key()
            ))
            .with_field("civilDate.skipped"));
        }
        if month == 0 {
            return Err(
                Error::invalid_arg(format!("{}: months are counted from 1", self.key))
                    .with_field(format!("{form}.month")),
            );
        }
        match self.on {
            Dated::CivilDate { day, .. }
                if tibetan && (day == 0 || day.unsigned_abs() > 30) =>
            {
                Err(Error::invalid_arg(format!(
                    "{}: a Tibetan month numbers its days 1 to 30, or back from -1, not {day}",
                    self.key
                ))
                .with_field("civilDate.day"))
            }
            Dated::CivilDate { day, .. } if day == 0 || day.unsigned_abs() > 32 => {
                Err(Error::invalid_arg(format!(
                    "{}: a day is counted from 1, or back from the month's end from -1, not {day}",
                    self.key
                ))
                .with_field("civilDate.day"))
            }
            Dated::CivilWeekday { nth, .. } if nth == 0 || nth.unsigned_abs() > 5 => {
                Err(Error::invalid_arg(format!(
                    "{}: a month holds a weekday at most five times; count from 1, or back from -1, not {nth}",
                    self.key
                ))
                .with_field("civilWeekday.nth"))
            }
            _ => Ok(()),
        }
    }

    /// The days Nepal's national panchanga prints on a Bikram Sambat date:
    /// the twelve sankrantis on the first of each month, the national days
    /// it names, and the festivals it keeps on a date;
    /// and the two it keeps on the Sun's entry into Ardra, भूमिरज and the
    /// भूमि पूजा three days on.
    #[must_use]
    pub fn nepal_panchanga() -> Vec<DatedRule> {
        let bs = |key: &str, month: u8, day: i8, source: &str| DatedRule {
            key: key.to_owned(),
            source: source.to_owned(),
            on: Dated::CivilDate {
                calendar: Calendar::BikramSambat,
                month,
                day,
                skipped: None,
            },
        };
        let sankranti = |key: &str, month: u8, printed: &str| {
            bs(
                key,
                month,
                1,
                &format!(
                    "Nepal's national panchanga, VS 2082: \"{printed}\" on the first of the Bikram Sambat month, the day the Sun enters its sign under the punya convention (C619)"
                ),
            )
        };
        let mut rules = vec![
            sankranti("MESHA_SANKRANTI", 1, "वैशाखसंक्रान्ति, नववर्षारम्भ"),
            sankranti("VRISHABHA_SANKRANTI", 2, "ज्येष्ठसंक्रान्ति"),
            sankranti("MITHUNA_SANKRANTI", 3, "आषाढसंक्रान्ति"),
            sankranti("KARKATA_SANKRANTI", 4, "श्रावणसङ्क्रान्ति, दक्षिणायनारम्भ"),
            sankranti("SIMHA_SANKRANTI", 5, "भाद्रसङ्क्रान्ति"),
            sankranti("KANYA_SANKRANTI", 6, "आश्विनसङ्क्रान्ति"),
            sankranti("TULA_SANKRANTI", 7, "कार्तिकसङ्क्रान्ति"),
            sankranti("VRISHCHIKA_SANKRANTI", 8, "मार्गसङ्क्रान्ति"),
            sankranti("DHANU_SANKRANTI", 9, "पौषसङ्क्रान्ति"),
            sankranti("MAKARA_SANKRANTI", 10, "माघसङ्क्रान्ति, उत्तरायण सुरु"),
            sankranti("KUMBHA_SANKRANTI", 11, "फाल्गुनसङ्क्रान्ति"),
            sankranti("MINA_SANKRANTI", 12, "चैत्रसङ्क्रान्ति"),
            bs(
                "LOKTANTRA_DIWAS",
                1,
                11,
                "Nepal's national panchanga, VS 2082 p. 2: \"लोकतन्त्र दिवस\" on Baisakh 11",
            ),
            bs(
                "CONSTITUTION_DAY",
                6,
                3,
                "Nepal's national panchanga, VS 2082 p. 12: \"संविधानदिवस\" on Asoj 3",
            ),
            bs(
                "PRITHVI_JAYANTI",
                9,
                27,
                "Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 21: \"पृथ्वीजयन्ती, राष्ट्रिय एकता दिवस\" on Poush 27",
            ),
            bs(
                "EARTHQUAKE_SAFETY_DAY",
                10,
                2,
                "Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 21: \"राष्ट्रिय भूकम्प सुरक्षा दिवस\" on Magh 2",
            ),
            bs(
                "TAMU_LHOSAR",
                9,
                15,
                "Nepal's national panchanga, VS 2082 p. 19 and VS 2083 p. 20: \"तमुल्होछार\" on Poush 15",
            ),
            bs(
                "MARTYRS_DAY",
                10,
                16,
                "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 22: \"सहिद दिवस\" on Magh 16",
            ),
            bs(
                "DEMOCRACY_DAY",
                11,
                7,
                "Nepal's national panchanga, VS 2082 p. 23 and VS 2083 p. 23: \"राष्ट्रिय प्रजातन्त्र दिवस\" on Phalgun 7",
            ),
            bs(
                "WALKE_PARVA",
                5,
                1,
                "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 11: \"वल्केपर्व\" on Bhadra 1, the day of भाद्रसङ्क्रान्ति",
            ),
        ];
        rules.extend(Self::on_printed_dates());
        rules.extend(Self::on_the_suns_entry());
        rules
    }

    /// The festivals and civil days both editions print on one date of a
    /// Bikram Sambat month ([`SOLAR_DAYS`]) or of the Gregorian year
    /// ([`GREGORIAN_DAYS`]).
    fn on_printed_dates() -> impl Iterator<Item = DatedRule> {
        let printed = |calendar| {
            move |&(key, month, day, printed): &(&str, u8, i8, &str)| DatedRule {
                key: key.to_owned(),
                source: format!("Nepal's national panchanga, {printed}"),
                on: Dated::CivilDate {
                    calendar,
                    month,
                    day,
                    skipped: None,
                },
            }
        };
        SOLAR_DAYS
            .iter()
            .map(printed(Calendar::BikramSambat))
            .chain(GREGORIAN_DAYS.iter().map(printed(Calendar::Gregorian)))
    }

    /// The days Nepal's national panchanga keeps on the Sun's entry into
    /// Ardra: भूमिरज on the entry's civil day and the भूमि पूजा three days
    /// on (C696).
    fn on_the_suns_entry() -> Vec<DatedRule> {
        let ardra = |key: &str, days: u8, printed: &str| DatedRule {
            key: key.to_owned(),
            source: format!(
                "Nepal's national panchanga, VS 2082 p. 6 and VS 2083 p. 7: {printed}; the side column prints \"आर्द्रा १ मा सूर्य\" on Asar 8 at 14:58 and 21:11, the Sun's entry into Ardra to the minute (C696)"
            ),
            on: Dated::SunEnters {
                nakshatra: Nakshatra::Ardra,
                days,
            },
        };
        vec![
            ardra("BHUMIRAJ", 0, "\"भूमिरज\" on Asar 8"),
            ardra(
                "BHUMI_PUJA",
                3,
                "\"भूमिपूजा\" on Asar 11, three days after भूमिरज",
            ),
        ]
    }
}

/// An observance a [`DatedRule`] gave.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DatedObservance {
    /// The rule's key.
    pub rule: String,
    /// The day, in the range's calendar.
    pub day: CalendarDate,
    /// The same day in the rule's calendar; for a rule kept on the Sun's
    /// entry, the day the Sun entered, in the range's calendar.
    pub on: CalendarDate,
    /// What a published entry did to it; absent when none touched it,
    /// so the day is the rule's (`03-design/published-overrides.md`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<super::Published>,
}

/// The observances each dated rule gives over `days`, in the order of
/// the days, then of the rules.
///
/// # Errors
///
/// A rule's own refusal ([`DatedRule::check`]), or a day its calendar
/// cannot convert.
pub fn dated(rules: &[DatedRule], days: &[FestivalDay]) -> Result<Observances, Error> {
    for rule in rules {
        rule.check()?;
    }
    let mut answer = Observances::default();
    for (at, day) in days.iter().enumerate() {
        let fixed = system(day.date.calendar)?.fixed_of(&day.date)?;
        for rule in rules {
            let on = match rule.on {
                Dated::SunEnters {
                    nakshatra,
                    days: count,
                } => entered(days, at, nakshatra, count)?.map(|from| from.date.clone()),
                Dated::Easter {
                    computus,
                    days: count,
                } => from_easter(fixed, computus, count)?,
                Dated::CivilDate {
                    calendar: Calendar::TibetanPhugpa,
                    month,
                    day: wanted,
                    skipped,
                } => {
                    let on = system(Calendar::TibetanPhugpa)?.date_of(fixed)?;
                    keeps_tibetan(fixed, on.year, month, wanted, skipped.unwrap_or_default())
                        .then_some(on)
                }
                Dated::CivilDate { calendar, .. } | Dated::CivilWeekday { calendar, .. } => {
                    let calendar = system(calendar)?;
                    let on = calendar.date_of(fixed)?;
                    let length = calendar.month_length(on.year, on.month)?;
                    rule.on.holds(&on, length, day.vara).then_some(on)
                }
            };
            if let Some(on) = on {
                answer.dated.push(DatedObservance {
                    rule: rule.key.clone(),
                    day: day.date.clone(),
                    on,
                    published: None,
                });
            }
        }
    }
    Ok(answer)
}

/// Easter Sunday in `computus`'s own calendar, when `fixed` is `count`
/// days after it; none in a year the computus does not reckon, whose
/// churches kept no such Easter.
///
/// # Errors
///
/// None for a day the range could hold; the calendar's own refusal past
/// its years.
fn from_easter(
    fixed: FixedDay,
    computus: Computus,
    count: i16,
) -> Result<Option<CalendarDate>, Error> {
    let sunday = fixed.plus_days(-i64::from(count));
    let (year, ..) = gregorian_from_fixed(sunday);
    if year < computus.first_year() {
        return Ok(None);
    }
    if easter(year, computus)? != sunday {
        return Ok(None);
    }
    let calendar = match computus {
        Computus::Gregorian => Calendar::Gregorian,
        Computus::Julian => Calendar::Julian,
    };
    system(calendar)?.date_of(sunday).map(Some)
}

/// The day `count` days before `days[at]`, when the Sun entered
/// `nakshatra` in it. A day the count reaches back past is none: the
/// caller founds the days a rule counts from before its range.
///
/// # Errors
///
/// `INVALID_ARG` for a day whose entry was not read
/// (`Almanac::sun_entries`).
fn entered(
    days: &[FestivalDay],
    at: usize,
    nakshatra: Nakshatra,
    count: u8,
) -> Result<Option<&FestivalDay>, Error> {
    let Some(from) = at
        .checked_sub(usize::from(count))
        .and_then(|from| days.get(from))
    else {
        return Ok(None);
    };
    match from.sun_entry {
        SunEntry::Entered(into) => Ok((into == nakshatra).then_some(from)),
        SunEntry::Stayed => Ok(None),
        SunEntry::Unsearched => Err(Error::invalid_arg(format!(
            "a rule reads the Sun's entry on {}, which nothing read; fill the days' entries first (`Almanac::sun_entries`)",
            from.date
        ))),
    }
}

/// Whether `fixed`, a day of Tibetan year `year`, is the day a date's
/// observance falls on (`tibetan::observed`). A day may keep the year
/// before's date, a skipped 30th of month 12 kept on the day after being
/// the next year's first, so that year is asked too. The rule's month and
/// day are checked, so the one refusal left is a day past the range's
/// edge, which keeps nothing.
fn keeps_tibetan(fixed: FixedDay, year: i32, month: u8, day: i8, skipped: SkippedDay) -> bool {
    // -1 is the 30th: a month's days are numbered to 30, a skipped one
    // missing.
    let Ok(number) = u8::try_from(if day > 0 { day } else { 31 + day }) else {
        return false;
    };
    [year - 1, year]
        .into_iter()
        .filter(|year| (TIBETAN_RANGE.0..=TIBETAN_RANGE.1).contains(year))
        .any(|year| tibetan::observed(year, month, number, skipped).is_ok_and(|kept| kept == fixed))
}

fn system(calendar: Calendar) -> Result<&'static dyn CalendarSystem, Error> {
    shipped(calendar).ok_or_else(|| {
        Error::unsupported(format!(
            "the SDK does not ship the `{}` calendar",
            calendar.key()
        ))
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn date(month: u8, day: u8) -> CalendarDate {
        CalendarDate::defined(Calendar::BikramSambat, 2082, month, day)
    }

    #[test]
    fn each_kind_of_date_is_written_under_its_key_in_order() {
        let dates = [
            r#"{"calendar": "BIKRAM_SAMBAT", "month": 10, "day": 1}"#,
            r#"{"calendar": "GREGORIAN", "month": 11, "weekday": "GURUVARA", "nth": 4}"#,
            r#"{"nakshatra": "ARDRA"}"#,
            r#"{"computus": "JULIAN"}"#,
        ];
        for (key, date) in Dated::KEYS.iter().zip(dates) {
            let text = format!(r#"{{"key": "X", "source": "a test", "{key}": {date}}}"#);
            let rule: DatedRule = serde_json::from_str(&text).unwrap();
            let written = serde_json::to_value(rule.on).unwrap();
            let keys: Vec<&String> = written.as_object().unwrap().keys().collect();
            assert_eq!(keys, [key], "{text}");
        }
    }

    #[cfg(feature = "schema")]
    #[test]
    fn the_schema_writes_each_date_as_a_field_of_the_rule() {
        let schema = schemars::SchemaGenerator::default().into_root_schema_for::<DatedRule>();
        assert_eq!(schema.get("oneOf"), None);
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        let mut keys: Vec<&str> = properties.keys().map(String::as_str).collect();
        keys.sort_unstable();
        let mut expected = vec!["key", "source"];
        expected.extend(Dated::KEYS);
        expected.sort_unstable();
        assert_eq!(keys, expected);
        let required = schema.get("required").unwrap().as_array().unwrap();
        assert_eq!(required.len(), 2, "no date is required alone");
        let easter = properties
            .get("easter")
            .and_then(|f| f.get("description"))
            .and_then(serde_json::Value::as_str)
            .unwrap();
        assert!(
            easter.ends_with(
                "Exactly one of `civilDate`, `civilWeekday`, `sunEnters` or `easter` is given."
            ),
            "{easter}"
        );
    }

    fn civil(month: u8, day: i8) -> Dated {
        Dated::CivilDate {
            calendar: Calendar::BikramSambat,
            month,
            day,
            skipped: None,
        }
    }

    fn weekday(month: u8, weekday: Vara, nth: i8) -> Dated {
        Dated::CivilWeekday {
            calendar: Calendar::BikramSambat,
            month,
            weekday,
            nth,
        }
    }

    /// Good Friday and Easter by both reckonings in 2026, a week apart, and
    /// none before the Gregorian reckoning began.
    #[test]
    fn a_count_from_easter_keeps_the_day_and_names_the_sunday() {
        let fixed = teistro_calendar::gregorian::fixed_from_gregorian;
        let keeps = |computus, (year, month, day), count| {
            from_easter(fixed(year, month, day), computus, count)
                .unwrap()
                .map(|on| (on.calendar, on.year, on.month, on.day))
        };
        let (west, east) = (Computus::Gregorian, Computus::Julian);
        let sunday = Some((Calendar::Gregorian, 2026, 4, 5));
        assert_eq!(keeps(west, (2026, 4, 3), -2), sunday);
        assert_eq!(keeps(west, (2026, 4, 5), 0), sunday);
        assert_eq!(keeps(west, (2026, 5, 24), 49), sunday);
        assert_eq!(keeps(west, (2026, 4, 4), -2), None);
        // The Eastern Sunday is 12 April, 30 March in the Julian calendar.
        let julian = Some((Calendar::Julian, 2026, 3, 30));
        assert_eq!(keeps(east, (2026, 4, 10), -2), julian);
        assert_eq!(keeps(east, (2026, 4, 3), -2), None);
        // 1500's Western Easter was not reckoned, and the Eastern one was:
        // Julian 19 April, ten days behind once its 29 February had passed.
        assert_eq!(keeps(west, (1500, 4, 19), 0), None);
        assert_eq!(
            keeps(east, (1500, 4, 29), 0),
            Some((Calendar::Julian, 1500, 4, 19))
        );
    }

    #[test]
    fn a_count_from_easter_is_bounded_and_spelt_as_its_field() {
        let rule = |days| DatedRule {
            key: "X".to_owned(),
            source: "a test".to_owned(),
            on: Dated::Easter {
                computus: Computus::Julian,
                days,
            },
        };
        rule(-Dated::EASTER_DAYS).check().unwrap();
        rule(Dated::EASTER_DAYS).check().unwrap();
        for far in [-Dated::EASTER_DAYS - 1, Dated::EASTER_DAYS + 1, i16::MIN] {
            assert_eq!(rule(far).check().unwrap_err().field(), Some("easter.days"));
        }
        assert_eq!((rule(-2).reach(), rule(-2).reads_sun_entry()), (0, false));
        let spelt: DatedRule = serde_json::from_str(
            r#"{"key": "X", "source": "a test", "easter": {"computus": "JULIAN"}}"#,
        )
        .unwrap();
        assert_eq!(spelt, rule(0));
        let text = serde_json::to_string(&rule(-2)).unwrap();
        assert!(
            text.contains(r#""easter":{"computus":"JULIAN","days":-2}"#),
            "{text}"
        );
    }

    #[test]
    fn a_day_counts_from_the_first_or_back_from_the_last() {
        // A month of 30 days.
        assert!(civil(10, 1).holds(&date(10, 1), 30, Vara::Ravivara));
        assert!(!civil(10, 1).holds(&date(11, 1), 30, Vara::Ravivara));
        assert!(civil(12, -1).holds(&date(12, 30), 30, Vara::Ravivara));
        assert!(civil(12, -4).holds(&date(12, 27), 30, Vara::Ravivara));
        assert!(!civil(12, -4).holds(&date(12, 27), 31, Vara::Ravivara));
    }

    #[test]
    fn a_weekday_counts_by_the_weeks_from_either_end() {
        let saturday = Vara::Shanivara;
        // The 8th to the 14th hold each weekday's second.
        assert!(weekday(5, saturday, 2).holds(&date(5, 8), 31, saturday));
        assert!(weekday(5, saturday, 2).holds(&date(5, 14), 31, saturday));
        assert!(!weekday(5, saturday, 2).holds(&date(5, 15), 31, saturday));
        assert!(!weekday(5, saturday, 2).holds(&date(5, 8), 31, Vara::Ravivara));
        // In a 31-day month the 25th to the 31st hold each one's last.
        assert!(weekday(5, saturday, -1).holds(&date(5, 25), 31, saturday));
        assert!(!weekday(5, saturday, -1).holds(&date(5, 24), 31, saturday));
        assert!(weekday(5, saturday, -2).holds(&date(5, 24), 31, saturday));
    }

    #[test]
    fn a_rule_that_cannot_be_kept_is_refused_by_its_field() {
        let refused = |on| {
            DatedRule {
                key: "X".to_owned(),
                source: String::new(),
                on,
            }
            .check()
            .unwrap_err()
            .field()
            .map(str::to_owned)
        };
        assert_eq!(refused(civil(0, 1)).as_deref(), Some("civilDate.month"));
        assert_eq!(refused(civil(1, 0)).as_deref(), Some("civilDate.day"));
        assert_eq!(refused(civil(1, -33)).as_deref(), Some("civilDate.day"));
        assert_eq!(
            refused(weekday(1, Vara::Ravivara, 6)).as_deref(),
            Some("civilWeekday.nth")
        );
        assert_eq!(
            refused(weekday(1, Vara::Ravivara, 0)).as_deref(),
            Some("civilWeekday.nth")
        );
        for rule in DatedRule::nepal_panchanga() {
            rule.check().unwrap();
        }
        let tibetan = |month, day, skipped| Dated::CivilDate {
            calendar: Calendar::TibetanPhugpa,
            month,
            day,
            skipped,
        };
        let losar = DatedRule {
            key: String::from("LOSAR"),
            source: String::from("a test"),
            on: tibetan(1, 1, Some(SkippedDay::After)),
        };
        losar.check().unwrap();
        assert_eq!(
            refused(tibetan(1, 31, None)).as_deref(),
            Some("civilDate.day")
        );
        assert_eq!(
            refused(tibetan(1, -31, None)).as_deref(),
            Some("civilDate.day")
        );
        assert_eq!(
            refused(Dated::CivilWeekday {
                calendar: Calendar::TibetanPhugpa,
                month: 1,
                weekday: Vara::Ravivara,
                nth: 1
            })
            .as_deref(),
            Some("civilWeekday.calendar")
        );
        let skipped_bs = Dated::CivilDate {
            calendar: Calendar::BikramSambat,
            month: 1,
            day: 1,
            skipped: Some(SkippedDay::Before),
        };
        assert_eq!(refused(skipped_bs).as_deref(), Some("civilDate.skipped"));
    }

    /// Chokhor Duchen 2024 on the first of leap month 6's two 4ths, and
    /// Chotrul Duchen 2018, its 15th skipped, on either neighbour.
    #[test]
    fn a_tibetan_day_keeps_the_date_its_observance_falls_on() {
        let fixed = |year, month, date| {
            teistro_calendar::Gregorian
                .fixed_of(&CalendarDate::defined(
                    Calendar::Gregorian,
                    year,
                    month,
                    date,
                ))
                .unwrap()
        };
        let keeps = |at, year, month, day, skipped| keeps_tibetan(at, year, month, day, skipped);
        let before = SkippedDay::Before;
        assert!(keeps(fixed(2024, 7, 9), 2024, 6, 4, before));
        assert!(!keeps(fixed(2024, 7, 10), 2024, 6, 4, before));
        assert!(!keeps(fixed(2024, 8, 8), 2024, 6, 4, before));
        assert!(keeps(fixed(2018, 3, 1), 2018, 1, 15, before));
        assert!(!keeps(fixed(2018, 3, 2), 2018, 1, 15, before));
        assert!(keeps(fixed(2018, 3, 2), 2018, 1, 15, SkippedDay::After));
        // -1 is the 30th, kept on the last day of the month.
        let last = tibetan::observed(2026, 3, 30, before).unwrap();
        assert!(keeps(last, 2026, 3, -1, before));
        assert!(keeps(last, 2026, 3, 30, before));
    }

    #[test]
    fn a_rule_reads_as_its_json_record() {
        let rule: DatedRule = serde_json::from_str(
            r#"{"key": "SAFER_INTERNET_DAY", "source": "the organisers",
                "civilWeekday": {"calendar": "GREGORIAN", "month": 2, "weekday": "MANGALAVARA", "nth": 2}}"#,
        )
        .unwrap();
        assert_eq!(
            rule.on,
            Dated::CivilWeekday {
                calendar: Calendar::Gregorian,
                month: 2,
                weekday: Vara::Mangalavara,
                nth: 2
            }
        );
        let maghe = DatedRule::nepal_panchanga().into_iter().nth(9).unwrap();
        let text = serde_json::to_string(&maghe).unwrap();
        assert!(
            text.contains(r#""civilDate":{"calendar":"BIKRAM_SAMBAT","month":10,"day":1}"#),
            "{text}"
        );
    }
}
