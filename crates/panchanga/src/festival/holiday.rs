//! A government's holidays as its notice lists them
//! (`03-design/nepal-gazette-holidays.md`, SR-123): each clause's words,
//! the festivals it names by their keys, whom it is for, and the days it
//! prints.
//!
//! A clause's words are kept apart from its festivals' names. Nepal's VS
//! 2083 notice gives Dhanya Purnima's day as "धान्य पूर्णिमा - किराँत
//! समुदायको उधौली पर्व, योमरी पुन्हि, ज्यापू दिवस": one holiday naming four
//! festivals, each of which has its own name in every locale. A reader
//! shows the notice as the notice, and each festival by its key.
//!
//! The days are the notice's own, in the Bikram Sambat calendar. A clause
//! that gives none ("on the day of Eid") has none: the festival's rule
//! gives the day, and the notice moves nothing.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDay;
use teistro_core::catalogue::{Calendar, HolidayScope};

/// A clause's words, as printed and in English.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HolidayNotice {
    /// As the notice prints it, in Nepali.
    pub ne: String,
    /// In English.
    pub en: String,
}

/// One holiday of a notice.
///
/// ```
/// use teistro_core::catalogue::HolidayScope;
/// use teistro_panchanga::festival::HolidaySet;
///
/// let holidays = HolidaySet::NepalGazette.holidays();
/// let poush_9 = holidays.iter().find(|holiday| holiday.festivals.len() == 4).ok_or("listed")?;
/// assert_eq!(poush_9.festivals, ["DHANYA_PURNIMA", "UDHAULI", "YOMARI_PUNHI", "JYAPU_DIWAS"]);
/// assert_eq!(poush_9.scope, HolidayScope::National);
/// assert_eq!(poush_9.first.as_ref().map(|day| (day.year, day.month, day.day)), Some((2083, 9, 9)));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Holiday {
    /// The set it is of ([`HolidaySet::key`]).
    pub set: String,
    /// Its clause's words.
    pub notice: HolidayNotice,
    /// The festivals the clause names, by key, in its order.
    pub festivals: Vec<String>,
    /// Whom it is for.
    pub scope: HolidayScope,
    /// Where it is kept, for a holiday the clause limits to a part of the
    /// country no ISO code names (`KATHMANDU_VALLEY`, `HILLS`, `TERAI`);
    /// empty when the clause names districts no key does, which its
    /// notice lists.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<String>,
    /// For a holiday of a community or of a religion's followers, the
    /// community the clause names (`NEWAR`, `KIRAT`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub communities: Vec<String>,
    /// The first day the notice prints; absent when it gives the day as
    /// the festival's own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first: Option<CalendarDay>,
    /// The last day, for a holiday of several days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last: Option<CalendarDay>,
    /// The notice and its clause.
    pub source: String,
}

/// A government's list of holidays the SDK ships.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HolidaySet {
    /// Nepal's public, festival and substitute holidays of VS 2083, as
    /// the Ministry of Home Affairs' notice in the Nepal Gazette (vol. 75
    /// no. 67, part 5, Falgun 18, 2082) lists them.
    NepalGazette,
}

impl HolidaySet {
    /// Every set the SDK ships.
    pub const ALL: [HolidaySet; 1] = [HolidaySet::NepalGazette];

    /// Its key.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            HolidaySet::NepalGazette => "NEPAL_GAZETTE",
        }
    }

    /// The year of the calendar its days are in that it lists.
    #[must_use]
    pub const fn year(self) -> i32 {
        match self {
            HolidaySet::NepalGazette => 2083,
        }
    }

    /// The days of the festivals its holidays of several days are granted
    /// as, by key, as the notice's year reckons them: Dashain's and
    /// Tihar's days the pack keys apart from the festivals the clause
    /// names. A day inside its holiday's span is granted as the festival's.
    ///
    /// ```
    /// use teistro_panchanga::festival::HolidaySet;
    ///
    /// let days = HolidaySet::NepalGazette.festival_days();
    /// assert!(days.iter().any(|(key, day)| key == "KUKUR_TIHAR" && (day.month, day.day) == (7, 22)));
    /// ```
    #[must_use]
    pub fn festival_days(self) -> Vec<(String, CalendarDay)> {
        match self {
            HolidaySet::NepalGazette => NEPAL_2083
                .iter()
                .flat_map(|row| row.days_of)
                .map(|&(key, (month, day))| {
                    (
                        key.to_owned(),
                        CalendarDay {
                            calendar: Calendar::BikramSambat,
                            year: self.year(),
                            month,
                            day,
                            era: None,
                            leap: None,
                        },
                    )
                })
                .collect(),
        }
    }

    /// Its holidays, in the notice's order.
    ///
    /// ```
    /// use teistro_core::catalogue::HolidayScope;
    /// use teistro_panchanga::festival::HolidaySet;
    ///
    /// let holidays = HolidaySet::NepalGazette.holidays();
    /// assert_eq!(holidays.len(), 45);
    /// assert!(holidays.iter().any(|holiday| holiday.scope == HolidayScope::Women));
    /// ```
    #[must_use]
    pub fn holidays(self) -> Vec<Holiday> {
        match self {
            HolidaySet::NepalGazette => NEPAL_2083
                .iter()
                .map(|row| row.holiday(self, NEPAL_GAZETTE))
                .collect(),
        }
    }
}

/// Whom Nepal's notice gives `key` a holiday for, and where it says so:
/// the broadest of the clauses naming it (a nationwide clause over one for
/// some), `None` for a key no clause names.
///
/// ```
/// use teistro_core::catalogue::HolidayScope;
/// use teistro_panchanga::festival::nepal_gazette_scope;
///
/// let (scope, source) = nepal_gazette_scope("TAMU_LHOSAR").ok_or("named")?;
/// assert_eq!(scope, HolidayScope::National);
/// assert!(source.ends_with("clause 2.1 (ट)"));
/// assert_eq!(nepal_gazette_scope("GAURA_PARVA").map(|(scope, _)| scope), Some(HolidayScope::Concerned));
/// assert_eq!(nepal_gazette_scope("NO_SUCH_DAY"), None);
/// # Ok::<(), &str>(())
/// ```
#[must_use]
pub fn nepal_gazette_scope(key: &str) -> Option<(HolidayScope, String)> {
    let granted = |row: &&Row| row.festivals.contains(&key) || row.grants_as_its_festival(key);
    let naming = NEPAL_2083.iter().filter(granted);
    let row = naming
        .clone()
        .find(|row| row.scope == HolidayScope::National)
        .or_else(|| naming.clone().next())?;
    let within = if row.festivals.contains(&key) {
        ""
    } else {
        ", a day of the festival it grants"
    };
    Some((
        row.scope,
        format!("{NEPAL_GAZETTE}, clause {}{within}", row.clause),
    ))
}

/// The notice Nepal's VS 2083 holidays are of.
const NEPAL_GAZETTE: &str = "Nepal Gazette vol. 75 no. 67, part 5 (Falgun 18, 2082): the Ministry of Home Affairs' notice of the public, festival and substitute holidays of VS 2083";

/// A day of the notice: month and day.
type On = Option<(u8, u8)>;

/// One holiday as the table writes it.
struct Row {
    clause: &'static str,
    scope: HolidayScope,
    festivals: &'static [&'static str],
    regions: &'static [&'static str],
    communities: &'static [&'static str],
    first: On,
    last: On,
    /// For a holiday of several days granted as one festival's (Dashain's,
    /// Tihar's), that festival's other days the pack keys apart, with the
    /// day it reckons each in the notice's year: a day inside `first` to
    /// `last` is granted as the festival's.
    days_of: &'static [(&'static str, (u8, u8))],
    ne: &'static str,
    en: &'static str,
}

impl Row {
    /// Whether the holiday grants `key`'s day as its festival's: one of
    /// [`Row::days_of`] whose day lies between its first and last.
    fn grants_as_its_festival(&self, key: &str) -> bool {
        let (Some(first), Some(last)) = (self.first, self.last) else {
            return false;
        };
        self.days_of
            .iter()
            .any(|&(held, day)| held == key && (first..=last).contains(&day))
    }

    fn holiday(&self, set: HolidaySet, notice: &str) -> Holiday {
        let owned = |keys: &[&str]| keys.iter().map(|&key| key.to_owned()).collect();
        let day = |on: On| {
            on.map(|(month, day)| CalendarDay {
                calendar: Calendar::BikramSambat,
                year: set.year(),
                month,
                day,
                era: None,
                leap: None,
            })
        };
        Holiday {
            set: set.key().to_owned(),
            notice: HolidayNotice {
                ne: self.ne.to_owned(),
                en: self.en.to_owned(),
            },
            festivals: owned(self.festivals),
            scope: self.scope,
            regions: owned(self.regions),
            communities: owned(self.communities),
            first: day(self.first),
            last: day(self.last),
            source: format!("{notice}, clause {}", self.clause),
        }
    }
}

/// A holiday the whole country keeps on one printed day.
const fn national(
    clause: &'static str,
    festivals: &'static [&'static str],
    first: On,
    (ne, en): (&'static str, &'static str),
) -> Row {
    Row {
        clause,
        scope: HolidayScope::National,
        festivals,
        regions: &[],
        communities: &[],
        first,
        last: None,
        days_of: &[],
        ne,
        en,
    }
}

/// A holiday for some, by its scope, with the region or community the
/// clause names.
#[expect(
    clippy::too_many_arguments,
    reason = "one argument a column of the notice's table"
)]
const fn limited(
    clause: &'static str,
    scope: HolidayScope,
    festivals: &'static [&'static str],
    regions: &'static [&'static str],
    communities: &'static [&'static str],
    first: On,
    ne: &'static str,
    en: &'static str,
) -> Row {
    Row {
        clause,
        scope,
        festivals,
        regions,
        communities,
        first,
        last: None,
        days_of: &[],
        ne,
        en,
    }
}

/// The days the Kathmandu valley's jatra holidays are for (clause 5).
const VALLEY: &[&str] = &["KATHMANDU_VALLEY"];

/// Nepal's VS 2083 notice, clause by clause in its order: the festivals
/// (2.1 nationwide, 2.2 for those they concern), women employees (3),
/// educational institutions (4), the valley's jatras (5), the days (6.1
/// nationwide, 6.2 for employees with disabilities), the birth
/// anniversaries (7.1 nationwide, 7.2 for a religion's followers) and the
/// days kept with offices open (8). Clause 1's Saturdays name no festival,
/// and clause 9 lets a mission abroad set its own.
const NEPAL_2083: [Row; 45] = [
    national(
        "2.1 (क)",
        &["MESHA_SANKRANTI"],
        Some((1, 1)),
        ("नव वर्ष", "New Year"),
    ),
    national(
        "2.1 (ख)",
        &["CHANDI_PURNIMA", "BUDDHA_JAYANTI", "UBHAULI"],
        Some((1, 18)),
        (
            "चण्डी पूर्णिमा (वैशाख पूर्णिमा) - बुद्ध जयन्ती, राई, लिम्बू, याक्खा, सुनुवारलगायतका किराँत समुदायको उभौली पर्व",
            "Chandi Purnima (the full moon of Baisakh): Buddha Jayanti, and Ubhauli of the Kirat community, the Rai, Limbu, Yakkha, Sunuwar and others",
        ),
    ),
    national(
        "2.1 (ग)",
        &["RAKSHABANDHAN"],
        Some((5, 12)),
        ("रक्षाबन्धन", "Raksha Bandhan"),
    ),
    national(
        "2.1 (घ)",
        &["JANMASHTAMI"],
        Some((5, 19)),
        ("श्रीकृष्ण जन्माष्टमी", "Shri Krishna Janmashtami"),
    ),
    national(
        "2.1 (ङ)",
        &["NAVARATRA_ARAMBHA"],
        Some((6, 25)),
        ("घटस्थापना", "Ghatasthapana"),
    ),
    Row {
        clause: "2.1 (च)",
        scope: HolidayScope::National,
        festivals: &["VIJAYA_DASHAMI", "PHULPATI"],
        regions: &[],
        communities: &[],
        first: Some((6, 31)),
        last: Some((7, 6)),
        // Mahashtami (Kartik 1) and Mahanavami (Kartik 3), Dashain's days.
        days_of: &[("MAHA_ASHTAMI", (7, 1)), ("MAHA_NAVAMI", (7, 3))],
        ne: "दशैं बिदा (फूलपातीदेखि द्वादशीसम्म)",
        en: "The Dashain holiday (from Phulpati to the 12th)",
    },
    Row {
        clause: "2.1 (छ)",
        scope: HolidayScope::National,
        festivals: &["LAKSHMI_PUJA", "YAMA_DWITIYA"],
        regions: &[],
        communities: &[],
        first: Some((7, 22)),
        last: Some((7, 26)),
        // Tihar's days the pack groups with its gazetted festivals: Kag
        // Tihar (Kartik 21, before the span), Kukur Tihar (Kartik 22, with
        // Lakshmi Puja) and Govardhan puja (Kartik 24). Mha puja is a
        // Newar festival in the pack's grouping, not one of these.
        days_of: &[
            ("KAG_TIHAR", (7, 21)),
            ("KUKUR_TIHAR", (7, 22)),
            ("GOVARDHAN_PUJA", (7, 24)),
        ],
        ne: "तिहार बिदा (लक्ष्मीपूजा देखि भाइटिकाको भोलिपल्टसम्म)",
        en: "The Tihar holiday (from Lakshmi Puja to the day after Bhai Tika)",
    },
    national("2.1 (ज)", &["CHHATH"], Some((7, 29)), ("छठ पर्व", "Chhath")),
    national(
        "2.1 (झ)",
        &["DHANYA_PURNIMA", "UDHAULI", "YOMARI_PUNHI", "JYAPU_DIWAS"],
        Some((9, 9)),
        (
            "धान्य पूर्णिमा - किराँत समुदायको उधौली पर्व, योमरी पुन्हि, ज्यापू दिवस",
            "Dhanya Purnima: Udhauli of the Kirat community, Yomari Punhi and Jyapu Day",
        ),
    ),
    national(
        "2.1 (ञ)",
        &["CHRISTMAS"],
        Some((9, 10)),
        (
            "इसाई धर्मावलम्बीको पर्व क्रिसमस डे, (डिसेम्बर २५)",
            "Christmas Day, the festival of the Christians (25 December)",
        ),
    ),
    national(
        "2.1 (ट)",
        &["TAMU_LHOSAR"],
        Some((9, 15)),
        ("तमू ल्होछार", "Tamu Lhochhar"),
    ),
    national(
        "2.1 (ठ)",
        &["MAGHI_THARU", "MAKARA_SANKRANTI"],
        Some((10, 1)),
        ("माघी पर्व/माघे सङ्क्रान्ति", "Maghi and Maghe Sankranti"),
    ),
    national(
        "2.1 (ड)",
        &["SONAM_LHOSAR"],
        Some((10, 24)),
        ("सोनम ल्होछार", "Sonam Lhochhar"),
    ),
    national(
        "2.1 (ढ)",
        &["SHIVARATRI"],
        Some((11, 22)),
        ("महाशिवरात्री", "Maha Shivaratri"),
    ),
    national(
        "2.1 (ण)",
        &["GYALPO_LHOSAR"],
        Some((11, 25)),
        ("ग्याल्पो ल्होसार", "Gyalpo Losar"),
    ),
    national(
        "2.1 (त)",
        &["EID_AL_FITR"],
        None,
        (
            "इस्लाम धर्मावलम्बीको पर्व ईद (ईद उल फित्र) का दिन",
            "On the day of Eid (Eid al-Fitr), the festival of the Muslims",
        ),
    ),
    national(
        "2.1 (थ)",
        &["EID_AL_ADHA"],
        None,
        (
            "इस्लाम धर्मावलम्बीको पर्व बकर ईद (ईद उल अजहा) का दिन",
            "On the day of Bakr Eid (Eid al-Adha), the festival of the Muslims",
        ),
    ),
    limited(
        "2.1 (द)",
        HolidayScope::Region,
        &["HOLI_HILLS"],
        &["HILLS"],
        &[],
        Some((12, 7)),
        "फागुपूर्णिमा - हिमाली र पहाडी ५६ जिल्लाहरूमा",
        "Fagu Purnima, in the 56 mountain and hill districts",
    ),
    limited(
        "2.1 (द)",
        HolidayScope::Region,
        &["HOLI_TERAI"],
        &["TERAI"],
        &[],
        Some((12, 8)),
        "फागुपूर्णिमा - तराईका जिल्ला झापा, मोरङ, सुनसरी, सप्तरी, सिराहा, उदयपुर, धनुषा, महोत्तरी, सर्लाही, रौतहट, बारा, पर्सा, नवलपरासी (बर्दघाट सुस्ता पूर्व), नवलपरासी (बर्दघाट सुस्ता पश्चिम), रुपन्देही, कपिलवस्तु, दाङ, बाँके, बर्दिया, कैलाली र कञ्चनपुरमा",
        "Fagu Purnima, in the Terai districts: Jhapa, Morang, Sunsari, Saptari, Siraha, Udayapur, Dhanusha, Mahottari, Sarlahi, Rautahat, Bara, Parsa, Nawalparasi (Bardaghat Susta East), Nawalparasi (Bardaghat Susta West), Rupandehi, Kapilvastu, Dang, Banke, Bardiya, Kailali and Kanchanpur",
    ),
    limited(
        "2.2 (क)",
        HolidayScope::Community,
        &["GAI_JATRA"],
        &[],
        &["NEWAR"],
        Some((5, 13)),
        "गाईजात्रा (देशभरका नेवार समुदायका लागि मात्र)",
        "Gai Jatra (for the Newar community across the country only)",
    ),
    limited(
        "2.2 (ख)",
        HolidayScope::Concerned,
        &["GAURA_PARVA"],
        &[],
        &[],
        Some((5, 19)),
        "गौरा पर्व",
        "Gaura Parva",
    ),
    limited(
        "2.2 (ग)",
        HolidayScope::Community,
        &["DURA_MHAIPRU_NAKUMA"],
        &[],
        &["DURA"],
        Some((9, 15)),
        "दुरा म्हैप्रु नकुमा (देशभरका दुरा समुदायका लागि मात्र)",
        "Dura Mhaipru Nakuma (for the Dura community across the country only)",
    ),
    limited(
        "2.2 (घ)",
        HolidayScope::Region,
        &["SIRUWA_PAVANI"],
        &[],
        &[],
        None,
        "सिरुवा पावनीको दिन - सो पर्व मनाउने झापा, मोरङ, सुनसरी, सिराहा र सप्तरी जिल्लामा",
        "On the day of Sirua Pavani, in the districts that keep it: Jhapa, Morang, Sunsari, Siraha and Saptari",
    ),
    limited(
        "3 (क)",
        HolidayScope::Women,
        &["HARITALIKA"],
        &[],
        &[],
        Some((5, 29)),
        "हरितालिका (तीज) व्रत",
        "The Haritalika (Teej) vrata",
    ),
    limited(
        "3 (ख)",
        HolidayScope::Women,
        &["JITIYA"],
        &[],
        &[],
        Some((6, 18)),
        "जितिया पर्व (जितिया पर्व मनाउने महिला कर्मचारीको लागि)",
        "Jitiya (for the women employees who keep it)",
    ),
    limited(
        "4",
        HolidayScope::Schools,
        &["BASANTA_PANCHAMI"],
        &[],
        &[],
        Some((10, 28)),
        "वसन्त पञ्चमी",
        "Basanta Panchami",
    ),
    limited(
        "5 (क)",
        HolidayScope::Region,
        &["GAI_JATRA"],
        VALLEY,
        &[],
        Some((5, 13)),
        "गाईजात्रा",
        "Gai Jatra, in the Kathmandu valley",
    ),
    limited(
        "5 (ख)",
        HolidayScope::Region,
        &["INDRA_JATRA"],
        VALLEY,
        &[],
        Some((6, 9)),
        "इन्द्रजात्रा",
        "Indra Jatra, in the Kathmandu valley",
    ),
    limited(
        "5 (ग)",
        HolidayScope::Region,
        &["BHOTO_JATRA"],
        VALLEY,
        &[],
        None,
        "मत्स्येन्द्रनाथको भोटो देखाउने जात्रा (सो जात्रा हुने दिन)",
        "The jatra showing Matsyendranath's vest (on the day of the jatra), in the Kathmandu valley",
    ),
    limited(
        "5 (घ)",
        HolidayScope::Region,
        &["GHODE_JATRA"],
        VALLEY,
        &[],
        Some((12, 23)),
        "घोडेजात्रा",
        "Ghode Jatra, in the Kathmandu valley",
    ),
    national(
        "6.1 (क)",
        &["INTERNATIONAL_LABOUR_DAY"],
        Some((1, 18)),
        ("विश्व मजदुर दिवस - (मे १)", "World Labour Day (1 May)"),
    ),
    national(
        "6.1 (ख)",
        &["REPUBLIC_DAY"],
        Some((2, 15)),
        ("गणतन्त्र दिवस", "Republic Day"),
    ),
    national(
        "6.1 (ग)",
        &["CONSTITUTION_DAY"],
        Some((6, 3)),
        (
            "संविधान दिवस (राष्ट्रिय दिवस)",
            "Constitution Day (the National Day)",
        ),
    ),
    national(
        "6.1 (घ)",
        &["MARTYRS_DAY"],
        Some((10, 16)),
        ("सहिद दिवस", "Martyrs' Day"),
    ),
    national(
        "6.1 (ङ)",
        &["DEMOCRACY_DAY"],
        Some((11, 7)),
        ("राष्ट्रिय प्रजातन्त्र दिवस", "National Democracy Day"),
    ),
    national(
        "6.1 (च)",
        &["INTERNATIONAL_WOMEN_S_DAY"],
        Some((11, 24)),
        (
            "अन्तर्राष्ट्रिय महिला दिवस (मार्च ८)",
            "International Women's Day (8 March)",
        ),
    ),
    limited(
        "6.2",
        HolidayScope::Disability,
        &["INTERNATIONAL_DAY_OF_PERSONS_WITH_DISABILITIES"],
        &[],
        &[],
        Some((8, 17)),
        "अन्तर्राष्ट्रिय अपाङ्गता दिवस (डिसेम्बर ३) (अपाङ्गता भएका नेपालभित्रका सम्पूर्ण कर्मचारीको लागि)",
        "International Day of Persons with Disabilities (3 December), for every employee in Nepal with a disability",
    ),
    national(
        "7.1 (क)",
        &["BUDDHA_JAYANTI"],
        Some((1, 18)),
        ("बुद्ध जयन्ती", "Buddha Jayanti"),
    ),
    national(
        "7.1 (ख)",
        &["PRITHVI_JAYANTI"],
        Some((9, 27)),
        (
            "पृथ्वी जयन्ती (राष्ट्रिय एकता दिवस)",
            "Prithvi Jayanti (National Unity Day)",
        ),
    ),
    limited(
        "7.2 (क)",
        HolidayScope::Religion,
        &["FALGUNANDA_JAYANTI"],
        &[],
        &["KIRAT"],
        Some((7, 25)),
        "फाल्गुनन्द जयन्ती (किराँत धर्मावलम्बी)",
        "Falgunanda Jayanti (for the followers of the Kirat religion)",
    ),
    limited(
        "7.2 (ख)",
        HolidayScope::Religion,
        &["MAWLID"],
        &[],
        &["MUSLIM"],
        None,
        "मोहम्मद जयन्तीका दिन (नेपाली मुस्लिम धर्मावलम्बी)",
        "On the day of Mohammad Jayanti (for Nepali Muslims)",
    ),
    limited(
        "7.2 (ग)",
        HolidayScope::Religion,
        &["GURU_NANAK_JAYANTI"],
        &[],
        &["SIKH"],
        None,
        "गुरु नानक जयन्तीका दिन (नेपाली सिख धर्मावलम्बी)",
        "On the day of Guru Nanak Jayanti (for Nepali Sikhs)",
    ),
    limited(
        "8 (क)",
        HolidayScope::OfficesOpen,
        &["CASTE_BASED_DISCRIMINATION_AND_UNTOUCHABILITY_ELIMINATION_DAY"],
        &[],
        &[],
        Some((2, 21)),
        "जातीय भेदभाव तथा छुवाछुत उन्मूलन राष्ट्रिय दिवस",
        "National Day for the Elimination of Caste Discrimination and Untouchability",
    ),
    limited(
        "8 (ख)",
        HolidayScope::OfficesOpen,
        &["CIVIL_SERVICE_DAY"],
        &[],
        &[],
        Some((5, 22)),
        "निजामती सेवा दिवस",
        "Civil Service Day",
    ),
    limited(
        "8 (ग)",
        HolidayScope::OfficesOpen,
        &["GEN_Z_MARTYRS_DAY"],
        &[],
        &[],
        Some((5, 23)),
        "जेनजी सहिद दिवस",
        "Gen Z Martyrs' Day",
    ),
];

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        reason = "a test indexes what it built and fails by panicking"
    )]

    use super::*;

    #[test]
    fn every_holiday_names_a_festival_and_a_dated_one_runs_forward() {
        for holiday in HolidaySet::NepalGazette.holidays() {
            assert!(!holiday.festivals.is_empty(), "{}", holiday.source);
            if let (Some(first), Some(last)) = (&holiday.first, &holiday.last) {
                assert!(
                    (first.month, first.day) < (last.month, last.day),
                    "{}",
                    holiday.source
                );
            }
            assert!(
                holiday.last.is_none() || holiday.first.is_some(),
                "{}",
                holiday.source
            );
        }
    }

    #[test]
    fn a_scope_beyond_the_country_says_whom_it_limits_to() {
        for holiday in HolidaySet::NepalGazette.holidays() {
            let limited = !holiday.regions.is_empty() || !holiday.communities.is_empty();
            let names = |held: bool| assert!(held, "{}", holiday.source);
            if matches!(
                holiday.scope,
                HolidayScope::Community | HolidayScope::Religion
            ) {
                names(!holiday.communities.is_empty());
            } else if holiday.scope == HolidayScope::Region {
                // Sirua Pavani's five districts have no key; its notice names them.
                names(!holiday.regions.is_empty() || holiday.festivals == ["SIRUWA_PAVANI"]);
            } else {
                names(!limited);
            }
        }
    }

    #[test]
    fn a_festival_s_day_inside_its_holiday_is_granted_and_one_outside_is_not() {
        // Kukur Tihar, Kartik 22, inside Tihar's Kartik 22 to 26.
        let (scope, source) = nepal_gazette_scope("KUKUR_TIHAR").unwrap();
        assert_eq!(scope, HolidayScope::National);
        assert!(source.ends_with("clause 2.1 (छ), a day of the festival it grants"));
        // Kag Tihar, Kartik 21, the day before it.
        assert_eq!(nepal_gazette_scope("KAG_TIHAR"), None);
        // Mha puja is no day of the clause's festival in the pack's grouping.
        assert_eq!(nepal_gazette_scope("MHA_PUJA"), None);
    }

    #[test]
    fn the_holidays_naming_more_than_one_festival_are_the_notices_five() {
        let several: Vec<Vec<String>> = HolidaySet::NepalGazette
            .holidays()
            .into_iter()
            .filter(|holiday| holiday.festivals.len() > 1)
            .map(|holiday| holiday.festivals)
            .collect();
        assert_eq!(
            several,
            [
                vec!["CHANDI_PURNIMA", "BUDDHA_JAYANTI", "UBHAULI"],
                vec!["VIJAYA_DASHAMI", "PHULPATI"],
                vec!["LAKSHMI_PUJA", "YAMA_DWITIYA"],
                vec!["DHANYA_PURNIMA", "UDHAULI", "YOMARI_PUNHI", "JYAPU_DIWAS"],
                vec!["MAGHI_THARU", "MAKARA_SANKRANTI"],
            ]
        );
    }
}
