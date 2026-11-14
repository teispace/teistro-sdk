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
//! A day kept by a star or a tithi of a solar month is a third form: Onam
//! on Thiruvonam in Chingam, Srimanta Sankardeva's birthday on the bright
//! 10th in Ahin. The almanac's lunar month cannot say it, for the star or
//! the tithi of one solar month falls in either of two lunar months.
//!
//! The churches' moveable feasts are kept by a count from Easter Sunday,
//! which the computus dates in its own calendar (`teistro_calendar::easter`):
//! Good Friday two days before it, Pentecost forty-nine after.

use serde::{Deserialize, Serialize};
use teistro_calendar::easter::{Computus, easter};
use teistro_calendar::gregorian::gregorian_from_fixed;
use teistro_calendar::hijri::HijriOffset;
use teistro_calendar::tibetan::{self, SkippedDay, TIBETAN_RANGE};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay, shipped};
use teistro_core::catalogue::{Calendar, Nakshatra, Tithi, Vara};
use teistro_core::error::Error;
use teistro_core::interval::Interval;

use super::{FestivalDay, FestivalMeta, Observances, SunEntry, Window};
use crate::span::Span;

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

/// The days kept in Nepal on a Bikram Sambat date that the national
/// panchanga prints in neither edition, each on the date its keeper names:
/// key, month, day, and the source read for it (C1600, C1601). Each also
/// gives every day a patro printed in Nepal verified for it from VS 2070
/// to 2083, by a back-test kept outside the repository.
const KEPT_DAYS: [(&str, u8, i8, &str); 8] = [
    (
        "GEN_Z_MARTYRS_DAY",
        5,
        23,
        "Government of Nepal, Ministry of Home Affairs, public holidays of 2083 (Nepal Gazette vol. 75 no. 67 part 5, Falgun 18 2082) p. 6, §8 ग, among the days kept nationally with offices open: \"जेनजी सहिद दिवस - भदौ २३ गते\"; one notice, so one year read (C1600)",
    ),
    (
        "BP_SMRITI_DIWAS",
        4,
        6,
        "B.P. Koirala died on Shrawan 6, VS 2039, and the day is kept each Shrawan 6, as Onlinekhabar reports on VS 2081 Shrawan 6 (the 42nd), VS 2082 Shrawan 6 (the 43rd) and VS 2083 Shrawan 6 (\"हरेक वर्ष साउन ६ गते\"), and Hamro Patro on 20 July 2026, the Nepali Congress's 44th (rank 3, C1601)",
    ),
    (
        "KAMAIYA_MUKTI_DIWAS",
        4,
        2,
        "the Government of Nepal declared the kamaiyas free on Shrawan 2, VS 2057, as Onlinekhabar (VS 2081 Shrawan 3, under कमैया मुक्ति दिवस; VS 2083 Shrawan 2, the 26th year) and Kantipur (19 July 2026) report; the day is that date (rank 3, C1601)",
    ),
    (
        "MADAN_ASHRIT_SMRITI_DIWAS",
        2,
        3,
        "CPN-UML keeps Madan Bhandari's and Jivraj Ashrit's memorial day on Jestha 3, as Kantipur (VS 2082 Jestha 3, the 32nd) and Onlinekhabar (VS 2083 Jestha 3, the 33rd) report it kept that day (rank 3, C1601)",
    ),
    (
        "NATIONAL_RECONCILIATION_DAY",
        9,
        16,
        "राष्ट्रिय एकता तथा मेलमिलाप दिवस, kept on Poush 16, the day B.P. Koirala returned from exile in VS 2033, as Kantipur (VS 2081 Poush 16) and Onlinekhabar (VS 2082 Poush 16: \"प्रत्येक वर्ष मनाइने पुस १६ गतेको मेलमिलाप दिवस\") report (rank 3, C1601)",
    ),
    (
        "NATIONAL_SCIENCE_DAY",
        6,
        1,
        "Onlinekhabar, VS 2081 Asoj 1 (Sanat Kumar Sharma's blog): Asoj 1, Amrit Science Campus's founding day, is decided as राष्ट्रिय विज्ञान तथा प्रविधि दिवस, first kept on Asoj 1, VS 2070 (rank 3, C1601)",
    ),
    (
        "NATIONAL_TEA_DAY",
        1,
        15,
        "Onlinekhabar, VS 2079 Baisakh 15: the National Tea and Coffee Development Board keeps the 26th राष्ट्रिय चिया दिवस that day (rank 3, C1601)",
    ),
    (
        "NEPAL_NURSING_DAY",
        10,
        15,
        "Onlinekhabar, VS 2082 Magh 15: \"आज माघ १५, नेपाली नर्स दिवस\" (rank 3, C1601)",
    ),
];

/// The United Nations' international days a patro printed in Nepal keeps
/// and the national panchanga prints in neither edition, each on the date
/// the UN's list gives: key, month, day, and the list's entry as printed
/// (C1602). Each gives every day the patro verified for it.
const UN_DAYS: [(&str, u8, i8, &str); 20] = [
    (
        "INTERNATIONAL_MOTHER_LANGUAGE_DAY",
        2,
        21,
        "\"International Mother Language Day\", 21 February, A/RES/56/262",
    ),
    (
        "WORLD_HEALTH_DAY",
        4,
        7,
        "\"World Health Day [WHO]\", 7 April, WHA/A.2/Res.35",
    ),
    (
        "EARTH_DAY",
        4,
        22,
        "\"International Mother Earth Day\", 22 April, A/RES/63/278",
    ),
    (
        "INTERNATIONAL_DAY_OF_FAMILIES",
        5,
        15,
        "\"International Day of Families\", 15 May, A/RES/47/237",
    ),
    (
        "INTERNATIONAL_DAY_OF_UNITED_NATIONS_PEACEKEEPERS",
        5,
        29,
        "\"International Day of UN Peacekeepers\", 29 May, A/RES/57/129",
    ),
    (
        "INTERNATIONAL_DAY_OF_YOGA",
        6,
        21,
        "\"International Day of Yoga\", 21 June, A/RES/69/131",
    ),
    (
        "INTERNATIONAL_YOUTH_DAY",
        8,
        12,
        "\"International Youth Day\", 12 August, A/RES/54/120",
    ),
    (
        "INTERNATIONAL_DAY_OF_OLDER_PERSONS",
        10,
        1,
        "\"International Day of Older Persons\", 1 October, A/RES/45/106",
    ),
    (
        "INTERNATIONAL_DAY_OF_NON_VIOLENCE",
        10,
        2,
        "\"International Day of Non-Violence\", 2 October, A/RES/61/271",
    ),
    (
        "WORLD_TEACHERS_DAY",
        10,
        5,
        "\"World Teachers' Day [UNESCO]\", 5 October, 27 C/INF.7",
    ),
    (
        "WORLD_MENTAL_HEALTH_DAY",
        10,
        10,
        "\"World Mental Health Day [WHO]\", 10 October, no resolution printed",
    ),
    (
        "WORLD_FOOD_DAY",
        10,
        16,
        "\"World Food Day [FAO]\", 16 October, A/RES/35/70",
    ),
    (
        "INTERNATIONAL_DAY_FOR_THE_ERADICATION_OF_POVERTY",
        10,
        17,
        "\"International Day for the Eradication of Poverty\", 17 October, A/RES/47/196",
    ),
    (
        "WORLD_CHILDREN_S_DAY",
        11,
        20,
        "\"World Children's Day\", 20 November, A/RES/836(IX)",
    ),
    (
        "INTERNATIONAL_DAY_FOR_THE_ELIMINATION_OF_VIOLENCE_AGAINST_WOMEN",
        11,
        25,
        "\"International Day for the Elimination of Violence against Women\", 25 November, A/RES/54/134",
    ),
    (
        "WORLD_AIDS_DAY",
        12,
        1,
        "\"World AIDS Day\", 1 December, no resolution printed",
    ),
    (
        "INTERNATIONAL_VOLUNTEER_DAY",
        12,
        5,
        "\"International Volunteer Day for Economic and Social Development\", 5 December, A/RES/40/212",
    ),
    (
        "INTERNATIONAL_ANTI_CORRUPTION_DAY",
        12,
        9,
        "\"International Anti-Corruption Day\", 9 December, A/RES/58/4",
    ),
    (
        "INTERNATIONAL_MOUNTAIN_DAY",
        12,
        11,
        "\"International Mountain Day\", 11 December, A/RES/57/245",
    ),
    (
        "INTERNATIONAL_MIGRANTS_DAY",
        12,
        18,
        "\"International Migrants Day\", 18 December, A/RES/55/93",
    ),
];

/// The rest of the United Nations' international days a patro printed in
/// Nepal keeps and the national panchanga prints in neither edition, each
/// on the date the UN's list gives: key, month, day, and the list's entry
/// as printed (C1700). Each gives every day the patro verified for it but
/// World Remembrance and Reconciliation Day, two of whose four are 9 May
/// (C1702).
const MORE_UN_DAYS: [(&str, u8, i8, &str); 40] = [
    (
        "INTERNATIONAL_DAY_OF_EDUCATION",
        1,
        24,
        "\"International Day of Education\", 24 January, A/RES/73/25",
    ),
    (
        "WORLD_WETLANDS_DAY",
        2,
        2,
        "\"World Wetlands Day\", 2 February, A/RES/75/317",
    ),
    (
        "WORLD_RADIO_DAY",
        2,
        13,
        "\"World Radio Day\", 13 February, A/RES/67/124",
    ),
    (
        "WORLD_DAY_OF_SOCIAL_JUSTICE",
        2,
        20,
        "\"World Day of Social Justice\", 20 February, A/RES/62/10",
    ),
    (
        "INTERNATIONAL_DAY_OF_HAPPINESS",
        3,
        20,
        "\"International Day of Happiness\", 20 March, A/RES/66/281",
    ),
    (
        "INTERNATIONAL_DAY_FOR_THE_ELIMINATION_OF_RACIAL_DISCRIMINATION",
        3,
        21,
        "\"International Day for the Elimination of Racial Discrimination\", 21 March, A/RES/2142 (XXI)",
    ),
    (
        "WORLD_METEOROLOGICAL_DAY",
        3,
        23,
        "\"World Meteorological Day [WMO]\", 23 March, WMO/EC-XII/Res.6",
    ),
    (
        "WORLD_AUTISM_AWARENESS_DAY",
        4,
        2,
        "\"World Autism Awareness Day\", 2 April, A/RES/62/139",
    ),
    (
        "INTERNATIONAL_DAY_OF_SPORT_FOR_DEVELOPMENT_AND_PEACE",
        4,
        6,
        "\"International Day of Sport for Development and Peace\", 6 April, A/RES/67/296",
    ),
    (
        "INTERNATIONAL_WELLNESS_DAY",
        4,
        15,
        "\"International Wellness Day\", 15 April, A/RES/80/249",
    ),
    (
        "WORLD_BOOK_AND_COPYRIGHT_DAY",
        4,
        23,
        "\"World Book and Copyright Day [UNESCO]\", 23 April, 28 C/Resolution 3.18",
    ),
    (
        "WORLD_MALARIA_DAY",
        4,
        25,
        "\"World Malaria Day [WHO]\", 25 April, no resolution printed",
    ),
    (
        "WORLD_INTELLECTUAL_PROPERTY_DAY",
        4,
        26,
        "\"World Intellectual Property Day [WIPO]\", 26 April, no resolution printed",
    ),
    (
        "WORLD_REMEMBRANCE_AND_RECONCILIATION_DAY",
        5,
        8,
        "\"Time of Remembrance and Reconciliation for Those Who Lost Their Lives During the Second World War\", 8 May, A/RES/59/26; the resolution names 8 and 9 May, and the list and this rule keep the first (C1702)",
    ),
    (
        "WORLD_TELECOMMUNICATION_AND_INFORMATION_SOCIETY_DAY",
        5,
        17,
        "\"World Telecommunication and Information Society Day\", 17 May, A/RES/60/252",
    ),
    (
        "WORLD_BEE_DAY",
        5,
        20,
        "\"World Bee Day [FAO]\", 20 May, A/RES/72/211",
    ),
    (
        "WORLD_CULTURE_DAY",
        5,
        21,
        "\"World Day for Cultural Diversity for Dialogue and Development\", 21 May, A/RES/57/249",
    ),
    (
        "INTERNATIONAL_DAY_FOR_BIOLOGICAL_DIVERSITY",
        5,
        22,
        "\"International Day for Biological Diversity\", 22 May, A/RES/55/201",
    ),
    (
        "GLOBAL_DAY_OF_PARENTS",
        6,
        1,
        "\"Global Day of Parents\", 1 June, A/RES/66/292",
    ),
    (
        "WORLD_BICYCLE_DAY",
        6,
        3,
        "\"World Bicycle Day\", 3 June, A/RES/72/272",
    ),
    (
        "WORLD_OCEANS_DAY",
        6,
        8,
        "\"World Oceans Day\", 8 June, A/RES/63/111",
    ),
    (
        "WORLD_DAY_AGAINST_CHILD_LABOUR",
        6,
        12,
        "\"World Day Against Child Labour\", 12 June, no resolution printed",
    ),
    (
        "WORLD_BLOOD_DONOR_DAY",
        6,
        14,
        "\"World Blood Donor Day [WHO]\", 14 June, WHA Resolution 58.13",
    ),
    (
        "WORLD_DAY_TO_COMBAT_DESERTIFICATION_AND_DROUGHT",
        6,
        17,
        "\"World Day to Combat Desertification and Drought\", 17 June, A/RES/49/115",
    ),
    (
        "WORLD_REFUGEE_DAY",
        6,
        20,
        "\"World Refugee Day\", 20 June, A/RES/55/76",
    ),
    (
        "INTERNATIONAL_DAY_AGAINST_DRUG_ABUSE_AND_ILLICIT_TRAFFICKING",
        6,
        26,
        "\"International Day against Drug Abuse and Illicit Trafficking\", 26 June, A/RES/42/112",
    ),
    (
        "MICRO_SMALL_AND_MEDIUM_SIZED_ENTERPRISES_DAY",
        6,
        27,
        "\"Micro-, Small and Medium-sized Enterprises Day\", 27 June, A/RES/71/279",
    ),
    (
        "INTERNATIONAL_DAY_OF_SIGN_LANGUAGES",
        9,
        23,
        "\"International Day of Sign Languages\", 23 September, A/RES/72/161",
    ),
    (
        "INTERNATIONAL_DAY_FOR_THE_TOTAL_ELIMINATION_OF_NUCLEAR_WEAPONS",
        9,
        26,
        "\"International Day for the Total Elimination of Nuclear Weapons\", 26 September, A/RES/68/32",
    ),
    (
        "WORLD_POST_DAY",
        10,
        9,
        "\"World Post Day\", 9 October, no resolution printed",
    ),
    (
        "INTERNATIONAL_DAY_FOR_DISASTER_RISK_REDUCTION",
        10,
        13,
        "\"International Day for Disaster Risk Reduction\", 13 October, A/RES/64/200",
    ),
    (
        "INTERNATIONAL_DAY_OF_RURAL_WOMEN",
        10,
        15,
        "\"International Day of Rural Women\", 15 October, A/RES/62/136",
    ),
    (
        "WORLD_DEVELOPMENT_INFORMATION_DAY",
        10,
        24,
        "\"World Development Information Day\", 24 October, A/RES/3038 (XXVII)",
    ),
    (
        "WORLD_CITIES_DAY",
        10,
        31,
        "\"World Cities Day\", 31 October, A/RES/68/239",
    ),
    (
        "INTERNATIONAL_DAY_FOR_TOLERANCE",
        11,
        16,
        "\"International Day for Tolerance [UNESCO]\", 16 November, 28 C/Resolution 5.61",
    ),
    (
        "WORLD_TOILET_DAY",
        11,
        19,
        "\"World Toilet Day\", 19 November, A/RES/67/291",
    ),
    (
        "WORLD_TELEVISION_DAY",
        11,
        21,
        "\"World Television Day\", 21 November, A/RES/51/205",
    ),
    (
        "INTERNATIONAL_DAY_FOR_THE_ABOLITION_OF_SLAVERY",
        12,
        2,
        "\"International Day for the Abolition of Slavery\", 2 December, A/RES/317(IV)",
    ),
    (
        "WORLD_SOIL_DAY",
        12,
        5,
        "\"World Soil Day\", 5 December, A/RES/68/232",
    ),
    (
        "INTERNATIONAL_CIVIL_AVIATION_DAY",
        12,
        7,
        "\"International Civil Aviation Day\", 7 December, A/RES/51/33",
    ),
];

/// The days kept on a Gregorian date by a body other than the UN that a
/// patro printed in Nepal keeps and the national panchanga prints in
/// neither edition, each on the date the keeping body's own page gives:
/// key, month, day, and the body, its page and the line read (C1701).
/// Each gives every day the patro verified for it.
const ORGANISERS_DAYS: [(&str, u8, i8, &str); 16] = [
    (
        "INTERNATIONAL_CUSTOMS_DAY",
        1,
        26,
        "the World Customs Organization, wcoomd.org newsroom, January 2023: \"On 26 January, the Secretariat of the World Customs Organization (WCO) celebrated\" the day, the anniversary of the Customs Co-operation Council's first session on 26 January 1953",
    ),
    (
        "WORLD_CANCER_DAY",
        2,
        4,
        "the Union for International Cancer Control, worldcancerday.org/about-us: \"an international day observed every 4 February\"",
    ),
    (
        "INTERNATIONAL_CHILDHOOD_CANCER_DAY",
        2,
        15,
        "Childhood Cancer International, internationalchildhoodcancerday.org: \"On February 15th International Childhood Cancer Day is a global\" day",
    ),
    (
        "WORLD_CONSUMER_RIGHTS_DAY",
        3,
        15,
        "Consumers International, consumersinternational.org/what-we-do/world-consumer-rights-day: \"The consumer movement marks 15th March with World Consumer Rights Day every year\"",
    ),
    (
        "WORLD_ORAL_HEALTH_DAY",
        3,
        20,
        "FDI World Dental Federation, worldoralhealthday.org: \"On the 20 March every year, we ask the world to unite\"",
    ),
    (
        "WORLD_THEATRE_DAY",
        3,
        27,
        "the International Theatre Institute, world-theatre-day.org: \"World Theatre Day celebrated annually on the 27th March\"",
    ),
    (
        "INTERNATIONAL_DANCE_DAY",
        4,
        29,
        "the International Theatre Institute, international-dance-day.org: \"Created 1982 by ITI and Int. Dance Committee\", \"International Dance Day 29th April\"",
    ),
    (
        "INTERNATIONAL_NURSES_DAY",
        5,
        12,
        "the International Council of Nurses, icn.ch/how-we-do-it/campaigns/international-nurses-day: \"IND is celebrated around the world every May 12\"",
    ),
    (
        "INTERNATIONAL_DAY_OF_ACTION_FOR_WOMEN_S_HEALTH",
        5,
        28,
        "the May 28 campaign, may28.org, whose page is titled \"May 28 – International Day of Action for Women's Health\"",
    ),
    (
        "WORLD_MUSIC_DAY",
        6,
        21,
        "the French Ministry of Culture, fetedelamusique.culture.gouv.fr: \"Fête de la musique 21 juin\"",
    ),
    (
        "WORLD_PHYSIOTHERAPY_DAY",
        9,
        8,
        "World Physiotherapy, world.physio/wptday: \"World PT Day takes place every year on 8 September\"",
    ),
    (
        "WORLD_ALZHEIMER_S_DAY",
        9,
        21,
        "Alzheimer's Disease International, alzint.org/get-involved/world-alzheimers-month: \"is on 21 September every year\"",
    ),
    (
        "WORLD_RABIES_DAY",
        9,
        28,
        "the Global Alliance for Rabies Control, rabiesalliance.org/world-rabies-day: \"Every year on 28 September\"",
    ),
    (
        "WORLD_HEART_DAY",
        9,
        29,
        "the World Heart Federation, world-heart-federation.org/world-heart-day: \"Each year on World Heart Day (29 September)\"",
    ),
    (
        "WORLD_ANIMAL_DAY",
        10,
        4,
        "World Animal Day, worldanimalday.org, whose page is titled \"World Animal Day | October 4th\"",
    ),
    (
        "GLOBAL_HANDWASHING_DAY",
        10,
        15,
        "the Global Handwashing Partnership, globalhandwashing.org/global-handwashing-day: \"October 15 is Global Handwashing Day\"",
    ),
];

/// More days kept in Nepal on a Bikram Sambat date that the national
/// panchanga prints in neither edition, each on the date its keeper or the
/// press names, every source opened on 2026-11-14: key, month, day, and the
/// source read (C1800, C1802). Each gives every day a patro printed in Nepal
/// verified for it from VS 2070 to 2083.
const MORE_KEPT_DAYS: [(&str, u8, i8, &str); 11] = [
    (
        "NATIONAL_MAGAR_DAY",
        11,
        15,
        "Kantipur, VS 2081 Falgun 15 (Nawalparasi): the Nepal Magar Sangh's forerunner first met in Tanahun on Falgun 14 to 16, VS 2039, and \"फागुन १५ लाई संघको स्थापना दिवसका रूपमा मनाउने गरिन्छ\", \"प्रत्येक वर्ष फागुन १५ गते यो दिवस मनाइँदै आएको छ\" (rank 3, C1800)",
    ),
    (
        "PASANG_LHAMU_SMRITI_DIWAS",
        1,
        10,
        "The Himalayan Times, 22 April 2016, \"Pasang Lhamu Sherpa Memorial Day observed\": the Pasang Lhamu Sherpa Mountaineering Foundation, its chairman says, has been \"annually marking the 10th of the Nepali month Baishakh as the Pasang Lhamu Memorial Day\", the day she died on Everest in VS 2050 (rank 3, C1800)",
    ),
    (
        "SOCIAL_SERVICE_DAY",
        6,
        6,
        "Onlinekhabar, VS 2080 Asoj 4: the Social Welfare Council's staff mark the 44th सामाजिक सेवा दिवस, \"असोज ६ गते पर्ने सामाजिक सेवा दिवसको पूर्वसन्ध्यामा\" (rank 3, C1800)",
    ),
    (
        "NATIONAL_BHAKKA_DAY",
        9,
        29,
        "Hamro Patro, \"राष्ट्रिय भक्का दिवस\": the Koch Rajbanshi and the other indigenous communities of the eastern Terai \"पुस २९ गतेलाई राष्ट्रिय भक्का दिवसका रूपमा मनाउने गरेका छन्\" (rank 3, C1800)",
    ),
    (
        "NATIONAL_FLAG_DAY",
        9,
        1,
        "Republica, 16 December (the 7th National Flag Day): \"National Flag Day is celebrated on Poush 1 of the Nepali Calendar every year\" (rank 3, C1800)",
    ),
    (
        "NATIONAL_YOGA_DAY",
        10,
        1,
        "Onlinekhabar, VS 2079 Magh 1, reporting the cabinet's decision of Shrawan 28, VS 2072: \"माघ १ गते राष्ट्रिय योग दिवस मनाउने औपचारिक निर्णय\" (rank 3, C1800)",
    ),
    (
        "NATIONAL_JOURNALISM_DAY",
        1,
        24,
        "Kantipur, VS 2082 Baisakh 24 (the 10th): Gorkhapatra's first weekly issue came out on Baisakh 24, VS 1958, and \"सरकारले २०७३ सालमा पत्रकारिता दिवस राष्ट्रिय रुपमै मनाउन मान्यता दिएपछि यो दिवस वैशाख २४ गते मनाउन थालिएको हो\" (rank 3, C1800, C1801)",
    ),
    (
        "NATIONAL_FILM_DAY",
        6,
        15,
        "Kantipur, VS 2082 Asar 16, reporting the cabinet's decision of Asar 13, VS 2082, the day Aama was first shown: \"प्रत्येक वर्ष असोज १५ गतेलाई 'राष्ट्रिय चलचित्र दिवस' मनाइने भएको छ\"; kept from VS 2082, the rule answering every year (rank 3, C1800, C1801)",
    ),
    (
        "LANGUAGE_DAY",
        1,
        24,
        "Kantipur, VS 2083 Baisakh 23: Bagmati Province keeps its language day on Baisakh 24, the day it began to use its official-language act in VS 2080, \"उक्त दिनलाई भाषा दिवस मनाउँदै आएको छ\", and gives a public holiday across the province by its notice of Chaitra 23, VS 2082 (rank 3, a province's day, C1800, C1802)",
    ),
    (
        "BODE_JIBRO_CHHEDNE_JATRA",
        1,
        2,
        "Kantipur, VS 2079 Chaitra 25: \"जिल्लाको प्रसिद्ध बिस्केट जात्राअन्तर्गत प्रत्येक वैशाख २ मा मध्यपुर थिमि बोडेमा जिब्रो छेड्ने जात्रा मनाइन्छ\"; and VS 2083 Baisakh 2: \"प्रत्येक वर्ष २ बैशाखमा\" (rank 3, C1800)",
    ),
    (
        "SINDOOR_JATRA",
        1,
        2,
        "Kantipur, VS 2083 Baisakh 2: \"भक्तपुरको बिस्काः जात्राअन्तर्गत २ बैशाखमा थिमीमा सिन्दूर जात्रा गर्ने प्रचलन रहेको छ\" (rank 3, C1800)",
    ),
];

/// The days other bodies, Nepal's press and custom keep on a Gregorian
/// date that the national panchanga prints in neither edition (or, for
/// Sagarmatha Day, in VS 2083 alone), each on the date its source names,
/// every source opened on 2026-11-14: key, month, day, and the source read
/// (C1803, C1804). Each gives every day a patro printed in Nepal verified
/// for it from VS 2070 to 2083.
const KEPT_GREGORIAN_DAYS: [(&str, u8, i8, &str); 15] = [
    (
        "NATIONAL_DRESS_DAY",
        1,
        1,
        "Kantipur, VS 2082 Poush 17 (1 January 2026, the 13th): youth organisations keep राष्ट्रिय पोसाक एवं टोपी दिवस, begun in VS 2070, \"जनवरी १ तारिखका दिन\"; one day, which the patro lists under two names (rank 3, C1803)",
    ),
    (
        "NATIONAL_TOPI_DAY",
        1,
        1,
        "Kantipur, VS 2082 Poush 17 (1 January 2026, the 13th): youth organisations keep राष्ट्रिय पोसाक एवं टोपी दिवस, begun in VS 2070, \"जनवरी १ तारिखका दिन\"; one day, which the patro lists under two names (rank 3, C1803)",
    ),
    (
        "VALENTINE_S_DAY",
        2,
        14,
        "a custom no body keeps, as Kantipur reports it kept in Nepal, VS 2082 Falgun 2: \"संसारभर फेब्रुअरी १४ लाई भ्यालेन्टाइन्स डेका रूपमा मनाइँदै आइएको छ\" (rank 3, C1804)",
    ),
    (
        "PI_DAY",
        3,
        14,
        "the International Day of Mathematics, idm314.org: \"Each year on March 14\", and \"March 14 is already celebrated in many countries as Pi Day\"; Pi Day itself has no keeping body (rank 2 for the date, C1803)",
    ),
    (
        "APRIL_FOOLS_DAY",
        4,
        1,
        "a custom no body keeps, as Kantipur reports it kept in Nepal, VS 2082 Chaitra 18: \"अप्रिल १ अर्थात् 'अप्रिल फूल'को अवसर\" (rank 3, C1804)",
    ),
    (
        "WORLD_HERITAGE_DAY",
        4,
        18,
        "the International Council on Monuments and Sites, icomos.org agenda \"18 April - Celebrating International Day for Monuments and Sites\": \"Every year, since 1982, the 18th of April is celebrated as the International Day for Monuments and Sites\", which the UNESCO General Conference adopted at its 22nd session, 1983 (rank 2, C1803)",
    ),
    (
        "WORLD_HYPERTENSION_DAY",
        5,
        17,
        "the World Hypertension League, whleague.org/about-us/world-hypertension-day: the day on \"May 17, 2026\"; and The Week, 16 May 2024: \"Every year, May 17th is observed as World Hypertension Day\", \"since 2006\" (rank 2 for one year and rank 3, C1803)",
    ),
    (
        "SAGARMATHA_DAY",
        5,
        29,
        "Kantipur, VS 2082 Jestha 15: Everest was first climbed on 29 May 1953, and \"सोही दिनलाई स्मरण गर्दै हरेक वर्ष मे २९ का दिन अन्तरराष्टिय सगरमाथा दिवस मनाउन थालिएको हो\"; the national panchanga prints it in VS 2083 alone (rank 3, C1803)",
    ),
    (
        "INTERNATIONAL_OLYMPIC_DAY",
        6,
        23,
        "the Japanese Olympic Committee, joc.or.jp/english/olympicdayevents: \"The IOC recognizes this memorable day - 23 June - as Olympic Day\", the day the IOC was founded in 1894 (rank 2, C1803)",
    ),
    (
        "WORLD_PHARMACISTS_DAY",
        9,
        25,
        "the International Pharmaceutical Federation, \"Preparing for World Pharmacists Day\" (fip.org): its 2009 Council accepted \"to annually celebrate a World Pharmacists Day, to be organised on September 25 (the day that FIP was founded in 1912)\" (rank 2, C1803)",
    ),
    (
        "NON_RESIDENT_NEPALI_DAY",
        10,
        11,
        "the Non-Resident Nepali Association, nrna.org (AGM 2024): established \"on 11 October, 2003\"; its Australian council, nrn.org.au: \"11th October is also declared as NRN day by Nepal Government\" (rank 2, C1803)",
    ),
    (
        "WORLD_STANDARDS_DAY",
        10,
        14,
        "the International Organization for Standardization, committee.iso.org/world-standards-day: \"Each year on 14 October, the members of the IEC, ISO and ITU celebrate World Standards Day\" (rank 2, C1803)",
    ),
    (
        "WORLD_RADIOGRAPHY_DAY",
        11,
        8,
        "the International Society of Radiographers and Radiological Technologists, isrrt.org/world-radiography-day, the day \"created to mark the anniversary of the discovery of x-rays by Wilhelm Roentgen in 1895\", \"Nov 8th\"; and the Society of Radiographers, sor.org: \"World Radiography Day 8 November marks the anniversary of the discovery of x-radiation\" (rank 2, C1803)",
    ),
    (
        "WORLD_FREEDOM_DAY",
        11,
        9,
        "the President of the United States, Proclamation 10856 of 8 November 2024 (89 FR, 14 November 2024): \"do hereby proclaim November 9, 2024, as World Freedom Day\", the Berlin Wall's fall, as each year's proclamation since 2001 does (rank 1, one proclamation read, C1803)",
    ),
    (
        "INTERNATIONAL_STUDENTS_DAY",
        11,
        17,
        "the European Students' Union, esu-online.org, 16 November 2013: \"November 17th known as the International Day of Students\", the events of Prague in November 1939 \"first commemorated on the 17th of November 1941\" (rank 2, C1803)",
    ),
];

/// The days other bodies keep on a weekday of a Gregorian month that the
/// national panchanga prints in neither edition: key, month, weekday, which
/// of the month's, and the source read on 2026-11-14 (C1805). Each gives the
/// one day a patro printed in Nepal verified for it, in VS 2079.
const KEPT_WEEKDAYS: [(&str, u8, Vara, i8, &str); 3] = [
    (
        "WORLD_FIRST_AID_DAY",
        9,
        Vara::Shanivara,
        2,
        "the Red Cross of Monaco, a society of the International Federation of Red Cross and Red Crescent Societies, croix-rouge.mc: World First Aid Day, \"organised annually by the International Federation of Red Cross and Red Crescent Societies (IFRC)\", \"on the second Saturday of September\" (rank 2, C1805)",
    ),
    (
        "WORLD_SMILE_DAY",
        10,
        Vara::Shukravara,
        1,
        "the World Smile Foundation, worldsmile.org/about: \"Harvey Ball launched World Smile Day, on the first Friday in October, 1999\"; and The Vindicator, 1 October 2010, under \"World Smile Day | First Friday in October\" (rank 2, C1805)",
    ),
    (
        "WORLD_SIGHT_DAY",
        10,
        Vara::Guruvara,
        2,
        "the International Agency for the Prevention of Blindness, iapb.world/world_sight_day: \"Every October, on the second Thursday, World Sight Day casts a spotlight on the importance of eye care\" (rank 2, C1805)",
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
    /// What the rule's festival is, as sourced (`03-design/festival-metadata.md`);
    /// a field no source gives stays unset, and a rule with none leaves it out.
    #[serde(default, skip_serializing_if = "FestivalMeta::is_empty")]
    pub meta: FestivalMeta,
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
    /// The day in which the sidereal Sun enters `nakshatra` in the
    /// almanac's own sky, or the day `days` civil days after it: भूमिरज on
    /// the civil day of the Sun's entry into Ardra, and the भूमि पूजा three
    /// days on (C696); Vishu on the day after the sunrise day of its entry
    /// into Ashwini, the first of Mesha (C1301).
    SunEnters {
        /// The nakshatra entered.
        nakshatra: Nakshatra,
        /// How many civil days after the entry's, 0 for that day, at most
        /// [`Dated::MOST_DAYS`].
        #[serde(default)]
        days: u8,
        /// Which day holds the entry: the civil day, midnight to midnight,
        /// when left out, or the day from sunrise to the next.
        #[serde(default, skip_serializing_if = "DayReckoning::is_civil")]
        reckoning: DayReckoning,
    },
    /// The day `nakshatra` holds in a month of `calendar`, a solar
    /// calendar's for the days Kerala keeps by the star of a Malayalam
    /// month: Onam on Thiruvonam (Shravana) in Chingam, Sree Narayana Guru's
    /// birthday on Chatayam (Shatabhisha) in it (C1304). An occurrence of
    /// the star is kept on the first day whose sunrise it holds, or, when
    /// it holds no sunrise, on the day it runs in; a month holding two
    /// occurrences keeps the one `twice` names. `days` counts civil days
    /// from that day, as the Onam days around Thiruvonam are kept.
    NakshatraInMonth {
        /// The calendar the month is in.
        calendar: Calendar,
        /// The month, 1 for the first.
        month: u8,
        /// The nakshatra.
        nakshatra: Nakshatra,
        /// Which of two occurrences in one month keeps the day.
        twice: super::Which,
        /// How many civil days after the star's day, before it when
        /// negative, 0 for that day; at most [`Dated::MOST_DAYS`] either
        /// way.
        #[serde(default)]
        days: i8,
    },
    /// The day `tithi` holds in a month of `calendar`, as Assam keeps
    /// Sankardeva's birthday on the bright 10th in Ahin (C1361): the day
    /// holding more of the window `at`, the earlier on a tie, or the day
    /// the tithi runs in when no day holds any; `twice` and `days` as for
    /// [`Dated::NakshatraInMonth`].
    TithiInMonth {
        /// The calendar the month is in.
        calendar: Calendar,
        /// The month, 1 for the first.
        month: u8,
        /// The tithi.
        tithi: Tithi,
        /// Which of two occurrences in one month keeps the day.
        twice: super::Which,
        /// The window judged: the sunrise when left out; never the
        /// moonrise (C1362).
        #[serde(default = "sunrise")]
        at: Window,
        /// How many civil days after the tithi's day, before it when
        /// negative, 0 for that day; at most [`Dated::MOST_DAYS`] either
        /// way.
        #[serde(default)]
        days: i8,
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

/// Which day holds an instant, for a rule kept on the day of the Sun's
/// entry ([`Dated::SunEnters`]).
///
/// ```
/// use teistro_panchanga::festival::DayReckoning;
///
/// assert_eq!(DayReckoning::default(), DayReckoning::Civil);
/// assert_eq!(serde_json::to_string(&DayReckoning::Sunrise)?, r#""SUNRISE""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum DayReckoning {
    /// The civil day, local midnight to the next: the day a patro prints
    /// the entry on (C696).
    #[default]
    Civil,
    /// The day from its sunrise to the next: the day a Kerala almanac
    /// reckons the sankranti in, whose next day is Vishu (C1301).
    Sunrise,
}

impl DayReckoning {
    /// Whether this is the civil day, which a rule leaves out.
    #[must_use]
    pub const fn is_civil(&self) -> bool {
        matches!(self, DayReckoning::Civil)
    }
}

/// The window a tithi's day in a month is judged at when a rule names
/// none: the sunrise.
const fn sunrise() -> Window {
    Window::Sunrise
}

impl Dated {
    /// The key each kind of date is written under, in the order of the
    /// variants: a [`DatedRule`] gives exactly one.
    pub const KEYS: [&'static str; 6] = [
        "civilDate",
        "civilWeekday",
        "sunEnters",
        "nakshatraInMonth",
        "tithiInMonth",
        "easter",
    ];

    /// The most days a solar month runs, which a rule kept on a star or a
    /// tithi in one reads around its day to find the month's other
    /// occurrence.
    pub const MONTH_DAYS: u8 = 32;

    /// The most days a rule counts from the Sun's entry, or either way
    /// from a star's day in a month: a fortnight, which is about the
    /// Sun's stay in a nakshatra, past which the next entry names the day
    /// better than a count.
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
            Dated::SunEnters { .. }
            | Dated::NakshatraInMonth { .. }
            | Dated::TithiInMonth { .. }
            | Dated::Easter { .. } => None,
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
            Dated::SunEnters { .. }
            | Dated::NakshatraInMonth { .. }
            | Dated::TithiInMonth { .. }
            | Dated::Easter { .. } => false,
        }
    }
}

impl DatedRule {
    /// Whether the rule is kept on the civil day of the Sun's entry into a
    /// nakshatra, which the days are read for only when a rule asks
    /// ([`FestivalDay::sun_entry`]).
    #[must_use]
    pub fn reads_sun_entry(&self) -> bool {
        matches!(
            self.on,
            Dated::SunEnters {
                reckoning: DayReckoning::Civil,
                ..
            }
        )
    }

    /// Whether the rule is kept on the sunrise day of the Sun's entry into
    /// a nakshatra ([`FestivalDay::sunrise_entry`]), read only when a rule
    /// asks.
    #[must_use]
    pub fn reads_sunrise_entry(&self) -> bool {
        matches!(
            self.on,
            Dated::SunEnters {
                reckoning: DayReckoning::Sunrise,
                ..
            }
        )
    }

    /// How many civil days the rule counts from the day it is kept by,
    /// which the days before a range are founded to reach; a star in a
    /// month adds the month's length, over which its other occurrence is
    /// sought.
    #[must_use]
    pub fn reach(&self) -> u8 {
        match self.on {
            Dated::SunEnters { days, .. } => days,
            // The month's other occurrence may lie a month either side.
            Dated::NakshatraInMonth { days, .. } | Dated::TithiInMonth { days, .. } => {
                Dated::MONTH_DAYS.saturating_add(days.unsigned_abs())
            }
            // Easter is reckoned from the year alone, so its count needs
            // no day before the range.
            Dated::CivilDate { .. } | Dated::CivilWeekday { .. } | Dated::Easter { .. } => 0,
        }
    }

    /// Refuses a count of days past its form's limit.
    fn check_count(&self) -> Result<(), Error> {
        match self.on {
            Dated::SunEnters { days, .. } if days > Dated::MOST_DAYS => {
                Err(Error::invalid_arg(format!(
                    "{}: a rule counts at most {} days from the Sun's entry, not {days}; past a fortnight, name the next entry",
                    self.key,
                    Dated::MOST_DAYS
                ))
                .with_field("sunEnters.days"))
            }
            Dated::Easter { days, .. }
                if days.unsigned_abs() > Dated::EASTER_DAYS.unsigned_abs() =>
            {
                Err(Error::invalid_arg(format!(
                    "{}: a rule counts at most {} days either way from Easter, not {days}",
                    self.key,
                    Dated::EASTER_DAYS
                ))
                .with_field("easter.days"))
            }
            Dated::NakshatraInMonth { days, .. } if days.unsigned_abs() > Dated::MOST_DAYS => {
                Err(Error::invalid_arg(format!(
                    "{}: a rule counts at most {} days either way from a star's day, not {days}",
                    self.key,
                    Dated::MOST_DAYS
                ))
                .with_field("nakshatraInMonth.days"))
            }
            Dated::TithiInMonth { days, .. } if days.unsigned_abs() > Dated::MOST_DAYS => {
                Err(Error::invalid_arg(format!(
                    "{}: a rule counts at most {} days either way from a tithi's day, not {days}",
                    self.key,
                    Dated::MOST_DAYS
                ))
                .with_field("tithiInMonth.days"))
            }
            Dated::CivilDate { .. }
            | Dated::CivilWeekday { .. }
            | Dated::SunEnters { .. }
            | Dated::NakshatraInMonth { .. }
            | Dated::TithiInMonth { .. }
            | Dated::Easter { .. } => Ok(()),
        }
    }

    /// Refuses a rule that could not be kept: an empty key, a calendar
    /// the SDK does not ship, month 0, a day of 0 or past 32 either way,
    /// an nth of 0 or past 5 either way, a count from the Sun's entry
    /// past [`Dated::MOST_DAYS`], one from a star's or a tithi's day in a
    /// month past it either way, a tithi's day judged at the moonrise or
    /// at a muhurta past fifteen, or a count from Easter past
    /// [`Dated::EASTER_DAYS`] either way.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the field that is wrong.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("a festival rule needs a key").with_field("key"));
        }
        self.check_count()?;
        if let Dated::TithiInMonth { at, .. } = self.on {
            if at == Window::Moonrise {
                return Err(Error::invalid_arg(format!(
                    "{}: a tithi's day in a month is not judged at the moonrise, which no search finds for it",
                    self.key
                ))
                .with_field("tithiInMonth.at"));
            }
            super::check_window(&self.key, at, "tithiInMonth.at")?;
        }
        let (form, calendar, month) = match self.on {
            Dated::CivilDate {
                calendar, month, ..
            } => ("civilDate", calendar, month),
            Dated::CivilWeekday {
                calendar, month, ..
            } => ("civilWeekday", calendar, month),
            Dated::NakshatraInMonth {
                calendar, month, ..
            } => ("nakshatraInMonth", calendar, month),
            Dated::TithiInMonth {
                calendar, month, ..
            } => ("tithiInMonth", calendar, month),
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

    /// The twelve sankrantis Nepal's national panchanga prints, each on the
    /// first of its Bikram Sambat month, the day the Sun enters its sign
    /// under the punya convention the calendar applies (C619): Mesha's on
    /// Baisakh 1 to Mina's on Chaitra 1.
    ///
    /// ```
    /// use teistro_panchanga::festival::DatedRule;
    ///
    /// let sankrantis = DatedRule::sankrantis();
    /// assert_eq!(sankrantis.len(), 12);
    /// assert_eq!(sankrantis[9].key, "MAKARA_SANKRANTI");
    /// assert!(DatedRule::nepal_panchanga().starts_with(&sankrantis));
    /// ```
    #[must_use]
    pub fn sankrantis() -> Vec<DatedRule> {
        const PRINTED: [(&str, &str); 12] = [
            ("MESHA_SANKRANTI", "वैशाखसंक्रान्ति, नववर्षारम्भ"),
            ("VRISHABHA_SANKRANTI", "ज्येष्ठसंक्रान्ति"),
            ("MITHUNA_SANKRANTI", "आषाढसंक्रान्ति"),
            ("KARKATA_SANKRANTI", "श्रावणसङ्क्रान्ति, दक्षिणायनारम्भ"),
            ("SIMHA_SANKRANTI", "भाद्रसङ्क्रान्ति"),
            ("KANYA_SANKRANTI", "आश्विनसङ्क्रान्ति"),
            ("TULA_SANKRANTI", "कार्तिकसङ्क्रान्ति"),
            ("VRISHCHIKA_SANKRANTI", "मार्गसङ्क्रान्ति"),
            ("DHANU_SANKRANTI", "पौषसङ्क्रान्ति"),
            ("MAKARA_SANKRANTI", "माघसङ्क्रान्ति, उत्तरायण सुरु"),
            ("KUMBHA_SANKRANTI", "फाल्गुनसङ्क्रान्ति"),
            ("MINA_SANKRANTI", "चैत्रसङ्क्रान्ति"),
        ];
        (1..=12u8)
            .zip(PRINTED)
            .map(|(month, (key, printed))| DatedRule {
                key: key.to_owned(),
                source: format!(
                    "Nepal's national panchanga, VS 2082: \"{printed}\" on the first of the Bikram Sambat month, the day the Sun enters its sign under the punya convention (C619)"
                ),
                on: Dated::CivilDate {
                    calendar: Calendar::BikramSambat,
                    month,
                    day: 1,
                    skipped: None,
                },
                meta: super::FestivalMeta::default(),
            })
            .collect()
    }

    /// The days Nepal's national panchanga prints on a Bikram Sambat date:
    /// the twelve sankrantis on the first of each month, the national days
    /// it names, and the festivals it keeps on a date;
    /// the two it keeps on the Sun's entry into Ardra, भूमिरज and the
    /// भूमि पूजा three days on; and the civil days it does not print, on
    /// the dates their keepers name.
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
            meta: super::FestivalMeta::default(),
        };
        let mut rules = DatedRule::sankrantis();
        rules.extend([
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
            bs(
                "YOSIN_UTTHAN",
                12,
                -1,
                "Nepal's national panchanga, VS 2083 p. 27: \"भ.पु.विश्वध्वजोत्थान\" on Chaitra 30, the year's last day, VS 2082 printing it on no day; Bhaktapur raises the pole the felling (YOSIN_PATAN) lowers on Baisakh 1 on the year's last day, as Onlinekhabar (VS 2074 Chaitra 27: \"पुरानो वर्ष बिदाइ र नयाँ वर्षको आगमनको पूर्वसन्ध्यामा लिङ्गो उभ्याउने गरिन्छ\") and Kantipur (VS 2083 Baisakh 1: \"चैत मसान्तमा इन्द्रध्वजा सहितको लिंगो उठाउने\") report (rank 3, C1650)",
            ),
        ]);
        rules.extend(Self::on_printed_dates());
        rules.extend(Self::on_the_suns_entry());
        rules.extend(Self::on_kept_dates());
        rules
    }

    /// The days the national panchanga prints in neither edition, each
    /// on the date its keeper names: Nepal's own on a Bikram Sambat date
    /// ([`KEPT_DAYS`], [`MORE_KEPT_DAYS`]), the UN's international days
    /// on a Gregorian one ([`UN_DAYS`], [`MORE_UN_DAYS`]), other bodies'
    /// days on the date their own pages give ([`ORGANISERS_DAYS`]), other
    /// bodies' and custom's on a Gregorian date the press or a keeper names
    /// ([`KEPT_GREGORIAN_DAYS`]) or on a weekday of a Gregorian month
    /// ([`KEPT_WEEKDAYS`]).
    fn on_kept_dates() -> impl Iterator<Item = DatedRule> {
        let kept = |calendar, cited: fn(&str) -> String| {
            move |&(key, month, day, source): &(&str, u8, i8, &str)| DatedRule {
                key: key.to_owned(),
                source: cited(source),
                on: Dated::CivilDate {
                    calendar,
                    month,
                    day,
                    skipped: None,
                },
                meta: super::FestivalMeta::default(),
            }
        };
        KEPT_DAYS
            .iter()
            .map(kept(Calendar::BikramSambat, |source| {
                format!("{source}; the national panchanga prints it in neither VS 2082 nor 2083")
            }))
            .chain(UN_DAYS.iter().map(kept(Calendar::Gregorian, |entry| {
                format!(
                    "the United Nations' list of international days (un.org/en/observances/list-days-weeks, read 2026-11-13): {entry}"
                )
            })))
            .chain(MORE_UN_DAYS.iter().map(kept(Calendar::Gregorian, |entry| {
                format!(
                    "the United Nations' list of international days (un.org/en/observances/list-days-weeks, read 2026-11-14): {entry}"
                )
            })))
            .chain(ORGANISERS_DAYS.iter().map(kept(Calendar::Gregorian, |source| {
                format!(
                    "{source} (read 2026-11-14, rank 2); the national panchanga prints it in neither VS 2082 nor 2083"
                )
            })))
            .chain(
                MORE_KEPT_DAYS
                    .iter()
                    .map(kept(Calendar::BikramSambat, |source| {
                        format!(
                            "{source}; the national panchanga prints it in neither VS 2082 nor 2083"
                        )
                    })),
            )
            .chain(
                KEPT_GREGORIAN_DAYS
                    .iter()
                    .map(kept(Calendar::Gregorian, str::to_owned)),
            )
            .chain(
                KEPT_WEEKDAYS
                    .iter()
                    .map(|&(key, month, weekday, nth, source)| DatedRule {
                        key: key.to_owned(),
                        source: format!(
                            "{source}; the national panchanga prints it in neither VS 2082 nor 2083"
                        ),
                        on: Dated::CivilWeekday {
                            calendar: Calendar::Gregorian,
                            month,
                            weekday,
                            nth,
                        },
                        meta: super::FestivalMeta::default(),
                    }),
            )
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
                meta: super::FestivalMeta::default(),
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
                reckoning: DayReckoning::Civil,
            },
            meta: super::FestivalMeta::default(),
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
/// the days, then of the rules. A rule on a date of the tabular Hijri
/// calendar reads each day moved by `hijri` toward a sighting
/// (`03-design/published-overrides.md` §7), and its `on` is the moved
/// date; no other calendar moves.
///
/// # Errors
///
/// A rule's own refusal ([`DatedRule::check`]), or a day its calendar
/// cannot convert.
pub fn dated(
    rules: &[DatedRule],
    days: &[FestivalDay],
    hijri: HijriOffset,
) -> Result<Observances, Error> {
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
                    reckoning,
                } => {
                    entered(days, at, (nakshatra, reckoning), count)?.map(|from| from.date.clone())
                }
                Dated::Easter {
                    computus,
                    days: count,
                } => from_easter(fixed, computus, count)?,
                Dated::NakshatraInMonth {
                    calendar,
                    month,
                    nakshatra,
                    twice,
                    days: count,
                } => match at.checked_add_signed(-isize::from(count)) {
                    Some(from) => in_month(
                        days,
                        from,
                        (system(calendar)?, month),
                        (|day| day.nakshatra.as_slice(), nakshatra, Window::Sunrise),
                        twice,
                    )?,
                    None => None,
                },
                Dated::TithiInMonth {
                    calendar,
                    month,
                    tithi,
                    twice,
                    at: window,
                    days: count,
                } => match at.checked_add_signed(-isize::from(count)) {
                    Some(from) => in_month(
                        days,
                        from,
                        (system(calendar)?, month),
                        (|day| day.tithi.as_slice(), tithi, window),
                        twice,
                    )?,
                    None => None,
                },
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
                    let on = if calendar == Calendar::HijriTabular {
                        hijri.read(fixed)?
                    } else {
                        system(calendar)?.date_of(fixed)?
                    };
                    let calendar = system(calendar)?;
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

/// The spans of one limb of a day, the nakshatras or the tithis.
type Limb<T> = fn(&FestivalDay) -> &[Span<T>];

/// Whether `member` of `limb` holds `day`'s sunrise.
fn at_sunrise<T: PartialEq>(day: &FestivalDay, limb: Limb<T>, member: &T) -> bool {
    crate::span::at(limb(day), day.sunrise).is_some_and(|span| span.member == *member)
}

/// Whether `days[at]` keeps an occurrence of `member` of `limb`: the
/// first day whose sunrise it holds, or, when it holds no sunrise, the
/// day it runs in. A day whose neighbour the decision needs and the days
/// do not hold keeps none: the caller founds days beside the range.
fn keeps_at_sunrise<T: PartialEq>(
    days: &[FestivalDay],
    at: usize,
    limb: Limb<T>,
    member: &T,
) -> bool {
    let Some(day) = days.get(at) else {
        return false;
    };
    if at_sunrise(day, limb, member) {
        return at
            .checked_sub(1)
            .and_then(|before| days.get(before))
            .is_some_and(|before| !at_sunrise(before, limb, member));
    }
    limb(day).iter().any(|span| span.member == *member)
        && days
            .get(at + 1)
            .is_some_and(|after| !at_sunrise(after, limb, member))
}

/// How much of `day`'s `window` the occurrence `whole` holds, in days; a
/// window that is an instant counts 1 when the occurrence holds it.
///
/// # Errors
///
/// None for the windows a rule's check lets through.
fn held(day: &FestivalDay, window: Window, whole: Interval) -> Result<f64, Error> {
    let window = day.window(window)?;
    if window.is_empty() {
        return Ok(if whole.contains(window.from) {
            1.0
        } else {
            0.0
        });
    }
    Ok(whole.clipped_to(window).map_or(0.0, Interval::days))
}

/// Whether `days[at]` keeps an occurrence of `member` of `limb` by
/// `window`: the day holding more of it than the day before and at least
/// as much as the day after, or, when none of the three holds any of it,
/// the first day it runs in. At the sunrise this is
/// [`keeps_at_sunrise`]. A day whose neighbour the decision needs and the
/// days do not hold keeps none.
///
/// # Errors
///
/// None for the windows a rule's check lets through.
fn keeps<T: PartialEq>(
    days: &[FestivalDay],
    at: usize,
    limb: Limb<T>,
    member: &T,
    window: Window,
) -> Result<bool, Error> {
    if window == Window::Sunrise {
        return Ok(keeps_at_sunrise(days, at, limb, member));
    }
    let neighbours = at
        .checked_sub(1)
        .and_then(|before| days.get(before))
        .zip(days.get(at + 1));
    let (Some(day), Some((before, after))) = (days.get(at), neighbours) else {
        return Ok(false);
    };
    for span in limb(day).iter().filter(|span| span.member == *member) {
        let here = held(day, window, span.whole)?;
        let earlier = held(before, window, span.whole)?;
        let later = held(after, window, span.whole)?;
        let kept = if here > 0.0 {
            here > earlier && here >= later
        } else {
            earlier <= 0.0 && later <= 0.0 && !before.sunrise_day().overlaps(span.whole)
        };
        if kept {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The day of `calendar`'s `month` that `days[at]` is, when it keeps
/// `member` of `limb` there by `window` and is the occurrence `twice`
/// names among the month's: a star's day or a tithi's in a solar month.
///
/// # Errors
///
/// A day the calendar cannot convert.
fn in_month<T: PartialEq>(
    days: &[FestivalDay],
    at: usize,
    (calendar, month): (&dyn CalendarSystem, u8),
    (limb, member, window): (Limb<T>, T, Window),
    twice: super::Which,
) -> Result<Option<CalendarDate>, Error> {
    let date_at = |index: usize| -> Result<Option<CalendarDate>, Error> {
        let Some(day) = days.get(index) else {
            return Ok(None);
        };
        let fixed = system(day.date.calendar)?.fixed_of(&day.date)?;
        calendar.date_of(fixed).map(Some)
    };
    if !keeps(days, at, limb, &member, window)? {
        return Ok(None);
    }
    let Some(on) = date_at(at)? else {
        return Ok(None);
    };
    if on.month != month {
        return Ok(None);
    }
    // The month's other occurrence, on the side `twice` passes over.
    let mut index = at;
    loop {
        index = match twice {
            super::Which::Earlier => match index.checked_sub(1) {
                Some(before) => before,
                None => break,
            },
            super::Which::Later => index + 1,
        };
        match date_at(index)? {
            Some(other) if other.year == on.year && other.month == on.month => {
                if keeps(days, index, limb, &member, window)? {
                    return Ok(None);
                }
            }
            _ => break,
        }
    }
    Ok(Some(on))
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
/// `nakshatra` in it, reckoned as `reckoning` says. A day the count
/// reaches back past is none: the caller founds the days a rule counts
/// from before its range.
///
/// # Errors
///
/// `INVALID_ARG` for a day whose entry was not read
/// (`Almanac::sun_entries`, `Almanac::sunrise_entries`).
fn entered(
    days: &[FestivalDay],
    at: usize,
    (nakshatra, reckoning): (Nakshatra, DayReckoning),
    count: u8,
) -> Result<Option<&FestivalDay>, Error> {
    let Some(from) = at
        .checked_sub(usize::from(count))
        .and_then(|from| days.get(from))
    else {
        return Ok(None);
    };
    let (entry, filled) = match reckoning {
        DayReckoning::Civil => (from.sun_entry, "Almanac::sun_entries"),
        DayReckoning::Sunrise => (from.sunrise_entry, "Almanac::sunrise_entries"),
    };
    match entry {
        SunEntry::Entered(into) => Ok((into == nakshatra).then_some(from)),
        SunEntry::Stayed => Ok(None),
        SunEntry::Unsearched => Err(Error::invalid_arg(format!(
            "a rule reads the Sun's entry on {}, which nothing read; fill the days' entries first (`{filled}`)",
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

pub(super) fn system(calendar: Calendar) -> Result<&'static dyn CalendarSystem, Error> {
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
            r#"{"calendar": "MALAYALAM_SOLAR", "month": 1, "nakshatra": "SHRAVANA", "twice": "LATER"}"#,
            r#"{"calendar": "ASSAM_SOLAR", "month": 6, "tithi": "SHUKLA_DASHAMI", "twice": "EARLIER"}"#,
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
        let mut expected = vec!["key", "meta", "source"];
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
                "Exactly one of `civilDate`, `civilWeekday`, `sunEnters`, `nakshatraInMonth`, `tithiInMonth` or `easter` is given."
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
            meta: super::FestivalMeta::default(),
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
                meta: super::FestivalMeta::default(),
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
            meta: super::FestivalMeta::default(),
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
