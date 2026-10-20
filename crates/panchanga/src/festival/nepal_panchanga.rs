//! The days Nepal's national panchanga prints that no text in hand gives a
//! table for (`03-design/festival-rules.md` §10, crux C600).
//!
//! Each rule is fitted: a window and a table chosen because they give the
//! days the Nepal Panchanga Nirnayak Samiti prints, checked against its
//! VS 2082 and 2083 editions (`cargo xtask festival`, `festival-measured.md`
//! §6). A fitted rule says so in its source and names the page it was read
//! from, so a reader who keeps the day by another authority sees where it
//! came from and can replace it by key.
//!
//! Four fits cover them all:
//!
//! - **udaya**: the day whose sunrise holds the tithi, the earlier when
//!   both sunrises do, as [`FestivalRule::udaya`] reads a rite of the
//!   daylight (C197);
//! - **udaya, the later**: the later of two sunrises, and the earlier day
//!   only when no sunrise holds the tithi (C610);
//! - **the share of a window**, the maintainer's rule (C601): the day
//!   holding more of the window, in time; the earlier on a tie, both
//!   holding it wholly counted as one; and the earlier when neither holds
//!   it (C603). Purvahna is the daylight's first six muhurtas, its first
//!   two fifths (C602);
//! - **on a vara**: the udaya day, kept only when it is the vara the rule
//!   names, and not at all otherwise (C618).
//!
//! The pradosha fast and the weekday fasts are kept in every month, the
//! adhika included, on a day of either paksha (§3.4) or the new moon.

use teistro_core::catalogue::{Masa, Tithi, Vara};
use teistro_core::settings::LunarMonth as Convention;

use super::{
    APARAHNA, Case, Choice, DayPart, FestivalRule, FollowingRule, Guard, MADHYAHNA, Occurs,
    Predicate, Which, Window, case,
};

/// Purvahna: the daylight's first six muhurtas, its first two fifths
/// (C602).
const PURVAHNA: Window = Window::DayMuhurta { from: 1, to: 6 };

/// The last fifth of the daylight.
const SAYAHNA: Window = Window::Part {
    part: DayPart::Sayahna,
};

/// How a fitted rule decides between its two days.
#[derive(Clone, Copy)]
enum Fit {
    /// The day whose sunrise holds the tithi, the earlier when both do.
    Udaya,
    /// The later of two sunrises, the earlier day when none holds it.
    UdayaLater,
    /// The day holding more of a window, in time.
    Share(Window),
    /// The udaya day, kept only when it is this vara (C618).
    OnVara(Vara),
}

impl Fit {
    /// The window, the guards and the fallback the fit stands for, and
    /// the words its source ends with.
    fn decide(self) -> (Window, Vec<Guard>, Choice, String) {
        use Choice::{Earlier, Later};
        match self {
            Fit::Udaya => (
                Window::Sunrise,
                vec![Guard::new([case(Case::LaterOnly)], Later)],
                Earlier,
                "the day whose sunrise holds the tithi, the earlier when both do (C197)".to_owned(),
            ),
            Fit::UdayaLater => (
                Window::Sunrise,
                vec![Guard::new([case(Case::Neither)], Earlier)],
                Later,
                "the day whose sunrise holds the tithi, the later when both do and the earlier when neither does (C610)".to_owned(),
            ),
            Fit::Share(window) => (
                window,
                vec![
                    Guard::new([case(Case::LaterOnly)], Later),
                    Guard::new([case(Case::Both)], Earlier),
                    Guard::new([Predicate::Greater { day: Which::Later }], Later),
                ],
                Earlier,
                format!(
                    "{}: the day holding more of it, in time; the earlier on a tie, both holding it wholly being one, and when neither holds it (C601, C603)",
                    Fit::name(window)
                ),
            ),
            Fit::OnVara(vara) => {
                let on = |day| Predicate::OnVara { day, vara };
                (
                    Window::Sunrise,
                    vec![
                        Guard::new([case(Case::LaterOnly), on(Which::Later)], Later),
                        Guard::new([case(Case::LaterOnly)], Choice::None),
                        Guard::new([on(Which::Earlier)], Earlier),
                    ],
                    Choice::None,
                    format!(
                        "the day whose sunrise holds the tithi, the earlier when both do, kept only when that day is {} (C618)",
                        vara.key()
                    ),
                )
            }
        }
    }

    /// The window's name in a source.
    fn name(window: Window) -> &'static str {
        match window {
            Window::DayMuhurta { from: 1, to: 6 } => {
                "purvahna, the daylight's first six muhurtas (C602)"
            }
            Window::Part { part } => match part {
                DayPart::Pratah => "pratah",
                DayPart::Sangava => "sangava",
                DayPart::Madhyahna => "madhyahna",
                DayPart::Aparahna => "aparahna",
                DayPart::Sayahna => "sayahna",
            },
            Window::Pradosha => "pradosha",
            Window::Sunrise
            | Window::Sunset
            | Window::Nishitha
            | Window::NightMuhurta { .. }
            | Window::DayMuhurta { .. } => "the window",
        }
    }
}

/// The fitted rules: key, month (amanta, the nija month), tithi, fit, and
/// where the day is printed.
const RULES: [(&str, Masa, Tithi, Fit, &str); 58] = [
    (
        "CHAITE_DASHAIN",
        Masa::Chaitra,
        Tithi::ShuklaAshtami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 25: चैतेदशैं",
    ),
    (
        "PARASHURAMA_JAYANTI",
        Masa::Vaishakha,
        Tithi::ShuklaTritiya,
        Fit::Share(Window::Pradosha),
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: परशुरामजयन्ती",
    ),
    (
        "SHANKARACHARYA_JAYANTI",
        Masa::Vaishakha,
        Tithi::ShuklaPanchami,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: आद्यजगत्गुरु शङ्कराचार्यजयन्ती",
    ),
    (
        "GANGA_SAPTAMI",
        Masa::Vaishakha,
        Tithi::ShuklaSaptami,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: गंगासप्तमी",
    ),
    (
        "SITA_NAVAMI",
        Masa::Vaishakha,
        Tithi::ShuklaNavami,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: सीताजयन्ती; VS 2082 is a day early until GREATER (C601)",
    ),
    (
        "KURMA_JAYANTI",
        Masa::Vaishakha,
        Tithi::Purnima,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: कूर्मजयन्ती",
    ),
    (
        "DASHAHARA_SNANA_ARAMBHA",
        Masa::Jyeshtha,
        Tithi::ShuklaPratipada,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: दशहरास्नानारम्भ; VS 2083 prints it in the adhika month (C606)",
    ),
    (
        "KUMARA_JATRA",
        Masa::Jyeshtha,
        Tithi::ShuklaSaptami,
        Fit::Share(PURVAHNA),
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 7: कुमारयात्रा",
    ),
    (
        "GANGA_DASHAHARA",
        Masa::Jyeshtha,
        Tithi::ShuklaDashami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: गङ्गादशहरा; VS 2083 prints it in the adhika month (C606)",
    ),
    (
        "NIRJALA_EKADASHI",
        Masa::Jyeshtha,
        Tithi::ShuklaEkadashi,
        Fit::UdayaLater,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 7: निर्जला एकादशीव्रत",
    ),
    (
        "ANALA_PUNHI",
        Masa::Jyeshtha,
        Tithi::Purnima,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2083 p. 5: अनलापुन्हि; the print names only the adhika month's full moon (C607)",
    ),
    (
        "DILA_CHARHE",
        Masa::Jyeshtha,
        Tithi::KrishnaChaturdashi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 6 and VS 2083 p. 8: (दिल्ला चःह्रे पूजा)",
    ),
    (
        "DURVA_ASHTAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaAshtami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 12: दूर्वाष्टमीव्रत; the third of three tithis a source gives; VS 2083 prints Shravana's dark 8th (C608, C620)",
    ),
    (
        "AJA_EKADASHI",
        Masa::Shravana,
        Tithi::KrishnaEkadashi,
        Fit::UdayaLater,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: अजाएकादशीव्रत",
    ),
    (
        "CHATHA",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturthi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 11: चन्द्रदोषदर्शन (चथा); VS 2083 p. 13: चन्द्रदर्शनदोष (चथा); the Newar name of the 4th's night",
    ),
    (
        "GANESHA_CHATURTHI",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturthi,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: गणेशचतुर्थीव्रत",
    ),
    (
        "RISHI_PANCHAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaPanchami,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: ऋषिपञ्चमीव्रत",
    ),
    (
        "GAURA_SAPTAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaSaptami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 12: गौरासप्तमी; VS 2083 prints Shravana's dark 7th, inside the solar Bhadra (C608)",
    ),
    (
        "HILE_JATRA",
        Masa::Bhadrapada,
        Tithi::ShuklaNavami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: रामेछाप चिसापानी हिलेजात्रा; the print's Hile jatra (Ramechhap) is Shravana's dark 9th (C609)",
    ),
    (
        "VAMANA_DWADASHI",
        Masa::Bhadrapada,
        Tithi::ShuklaDwadashi,
        Fit::Share(PURVAHNA),
        "Nepal's national panchanga, VS 2082 p. 11: वामनद्वादशी; VS 2083 p. 13: वामनजयन्ती",
    ),
    (
        "PRATIPADA_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaPratipada,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: प्रतिपदाश्राद्ध",
    ),
    (
        "DWITIYA_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaDvitiya,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: द्वितीयाश्राद्ध",
    ),
    (
        "TRITIYA_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaTritiya,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12: तृतीयाश्राद्ध (गातिला); VS 2083 p. 14: तृतीयाश्राद्ध",
    ),
    (
        "CHATURTHI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaChaturthi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: चतुर्थीश्राद्ध",
    ),
    (
        "PANCHAMI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaPanchami,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: पञ्चमीश्राद्ध",
    ),
    (
        "SHASHTHI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaShashthi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: षष्ठीश्राद्ध",
    ),
    (
        "SAPTAMI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaSaptami,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: सप्तमीश्राद्ध",
    ),
    (
        "ASHTAMI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaAshtami,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: अष्टमीश्राद्ध",
    ),
    (
        "NAVAMI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaNavami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: नवमीश्राद्ध, मातृनवमीश्राद्ध",
    ),
    (
        "DASHAMI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaDashami,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: दशमीश्राद्ध",
    ),
    (
        "EKADASHI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaEkadashi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: एकादशीश्राद्ध",
    ),
    (
        "DWADASHI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaDwadashi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: द्वादशीश्राद्ध",
    ),
    (
        "TRAYODASHI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaTrayodashi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: त्रयोदशीश्राद्ध",
    ),
    (
        "CHATURDASHI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaChaturdashi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: शस्त्रादिबाट निधन भएकाहरूको विश्वेदेवासहित एकोदिष्ट विधिले चतुर्दशीश्राद्ध",
    ),
    (
        "AMAVASYA_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::Amavasya,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: औंसीश्राद्ध, पितृविसर्जन",
    ),
    (
        "UPANGA_LALITA_VRATA",
        Masa::Ashwina,
        Tithi::ShuklaPanchami,
        Fit::Share(PURVAHNA),
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: उपाङ्गललिताव्रत; VS 2082 is a day late until DAY_MUHURTA (C602)",
    ),
    (
        "KUCHHI_BHOYE",
        Masa::Ashwina,
        Tithi::ShuklaAshtami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: (कूछि भ्वय्); the earlier of VS 2083's two sunrises (C683)",
    ),
    (
        "YAMAPANCHAKA_ARAMBHA",
        Masa::Ashwina,
        Tithi::KrishnaTrayodashi,
        Fit::Udaya,
        "kept in Nepal, though the national panchanga prints it in neither VS 2082 nor 2083 (C613)",
    ),
    (
        "DHANVANTARI_JAYANTI",
        Masa::Ashwina,
        Tithi::KrishnaTrayodashi,
        Fit::Share(PURVAHNA),
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: धन्वन्तरी जयन्ती",
    ),
    (
        "MHA_PUJA",
        Masa::Kartika,
        Tithi::ShuklaPratipada,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: म्हपूजा; VS 2083 is a day early until GREATER (C601)",
    ),
    (
        "HARIBODHINI_EKADASHI",
        Masa::Kartika,
        Tithi::ShuklaEkadashi,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: हरिबोधिनीएकादशीव्रत",
    ),
    (
        "PASHUPATI_HALESI_MELA",
        Masa::Kartika,
        Tithi::KrishnaTrayodashi,
        Fit::UdayaLater,
        "Nepal's national panchanga, VS 2082 p. 16: हलेसीमहादेवमेला; VS 2083 p. 18: हलेसीमहादेव मेला",
    ),
    (
        "BALA_CHATURDASHI",
        Masa::Kartika,
        Tithi::KrishnaChaturdashi,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 16 and VS 2083 p. 18: बालाचतुर्दशी; a rite of the morning, kept on the sunrise tithi, where purvahna read as six muhurtas gives VS 2082 a day early (C602)",
    ),
    (
        "DATTATREYA_JAYANTI",
        Masa::Margashirsha,
        Tithi::ShuklaChaturdashi,
        Fit::UdayaLater,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: दत्तात्रेयजयन्ती",
    ),
    (
        "PISHACHAMOCHANA_SHRADDHA",
        Masa::Margashirsha,
        Tithi::ShuklaChaturdashi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: काशीमा पिशाचमोचन तीर्थश्राद्ध; a shraddha, kept in aparahna as Dharmasindhu p. 265 keeps shraddha, where purvahna gave both years a day late (C602)",
    ),
    (
        "GAIDU_PUJA",
        Masa::Margashirsha,
        Tithi::Purnima,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: गैडुपूजा",
    ),
    (
        "PAUSHA_PUTRADA_EKADASHI",
        Masa::Pausha,
        Tithi::ShuklaEkadashi,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 19: पुत्रदाएकादशीव्रत; VS 2083 p. 21: स्मार्तहरुको पुत्रदाएकादशीव्रत",
    ),
    (
        "LAI_CHARHE",
        Masa::Pausha,
        Tithi::KrishnaChaturdashi,
        Fit::Share(SAYAHNA),
        "Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 22: (लैचःह्रेपूजा)",
    ),
    (
        "TILAKUNDA_CHATURTHI",
        Masa::Magha,
        Tithi::ShuklaChaturthi,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: तिलकुन्द चौथी",
    ),
    (
        "CHAMPA_SHASHTHI",
        Masa::Magha,
        Tithi::ShuklaShashthi,
        Fit::Udaya,
        "kept in Nepal, though the national panchanga prints it in neither VS 2082 nor 2083 (C613); the print names this day Skanda shashthi (C612)",
    ),
    (
        "BHIMA_EKADASHI",
        Masa::Magha,
        Tithi::ShuklaEkadashi,
        Fit::Share(PURVAHNA),
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: भीमा एकादशीव्रत",
    ),
    (
        "VIJAYA_EKADASHI",
        Masa::Magha,
        Tithi::KrishnaEkadashi,
        Fit::UdayaLater,
        "Nepal's national panchanga, VS 2082 p. 22 and VS 2083 p. 24: विजया एकादशीव्रत; the later sunrise, which VS 2083 prints (C610)",
    ),
    (
        "DVAPARA_YUGADI",
        Masa::Magha,
        Tithi::Amavasya,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 22 and VS 2083 p. 24: द्वापरयुगादि",
    ),
    (
        "PAHAN_CHARHE",
        Masa::Phalguna,
        Tithi::KrishnaChaturdashi,
        Fit::Share(Window::Pradosha),
        "Nepal's national panchanga, VS 2082 p. 24 and VS 2083 p. 26: (पाशाचःह्रेपूजा)",
    ),
    (
        "GHODE_JATRA",
        Masa::Phalguna,
        Tithi::Amavasya,
        Fit::Share(PURVAHNA),
        "Nepal's national panchanga, VS 2082 p. 24 and VS 2083 p. 26: घोडेजात्रा",
    ),
    (
        "MATSYANARAYANA_MELA",
        Masa::Chaitra,
        Tithi::ShuklaTritiya,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 25 and VS 2083 p. 27: मत्स्यनारायणमेला",
    ),
    (
        "MATSYA_JAYANTI",
        Masa::Chaitra,
        Tithi::ShuklaTritiya,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 25 and VS 2083 p. 27: मत्स्यजयन्ती",
    ),
    (
        "VAISHAKHA_SNANA_ARAMBHA",
        Masa::Chaitra,
        Tithi::Purnima,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 25: वैशाखस्नान सुरु",
    ),
];

/// The fitted rules kept in every month, the adhika included, as the
/// committee prints them in both editions: key, what is kept, fit, and
/// where the days are printed (§3.4, §3.5).
const EVERY_MONTH: [(&str, Occurs, Fit, &str); 6] = [
    (
        "PRADOSHA_VRATA",
        Occurs::DayOfPaksha { day_of_paksha: 13 },
        Fit::Share(Window::Pradosha),
        "Nepal's national panchanga, every paksha of VS 2082 (pp. 2–25) and VS 2083 (pp. 2–26): प्रदोषव्रत, named शनिप्रदोषव्रत on a Saturday and सोमप्रदोषव्रत on a Monday",
    ),
    (
        "MANGALA_CHATURTHI",
        Occurs::DayOfPaksha { day_of_paksha: 4 },
        Fit::OnVara(Vara::Mangalavara),
        "Nepal's national panchanga, VS 2083 pp. 3, 4, 12, 13, 21 and 22: मंगलचौथीव्रत; Dharmasindhu p. 20 names the 4th on a Tuesday",
    ),
    (
        "RAVI_SAPTAMI",
        Occurs::DayOfPaksha { day_of_paksha: 7 },
        Fit::OnVara(Vara::Ravivara),
        "Nepal's national panchanga, VS 2082 pp. 2, 3, 12, 21 and 22, VS 2083 pp. 6, 7 and 16: रविसप्तमीव्रत; Dharmasindhu p. 20 names the 7th on a Sunday",
    ),
    (
        "BHAUMASHTAMI",
        Occurs::DayOfPaksha { day_of_paksha: 8 },
        Fit::OnVara(Vara::Mangalavara),
        "Nepal's national panchanga, VS 2082 pp. 4, 5, 13, 14 and 23, VS 2083 pp. 17, 18, 25 and 26: भौमाष्टमीव्रत",
    ),
    (
        "BUDHASHTAMI",
        Occurs::DayOfPaksha { day_of_paksha: 8 },
        Fit::OnVara(Vara::Budhavara),
        "Nepal's national panchanga, VS 2082 pp. 15, 16 and 24, VS 2083 pp. 8 and 9: बुधाष्टमीव्रत",
    ),
    (
        "SOMAVATI_AMAVASYA",
        Occurs::Tithi {
            tithi: Tithi::Amavasya,
        },
        Fit::OnVara(Vara::Somavara),
        "Nepal's national panchanga, VS 2083 pp. 6 and 16: सोमबारे औसी; Dharmasindhu p. 20 names the new moon on a Monday; VS 2083 p. 24 does not name Monday 8 March 2027's new moon, a day it prints as दर्शश्राद्ध",
    ),
];

/// The days printed on another rule's day: key, the rule it keeps the
/// day of, and where.
const SAME_DAY: [(&str, &str, &str); 13] = [
    (
        "SETO_MACHHINDRANATH_JATRA",
        "CHAITE_DASHAIN",
        "Nepal's national panchanga, VS 2082 p. 25: श्रीश्वेतमत्स्येन्द्रनाथ रथयात्रा: the day of CHAITE_DASHAIN",
    ),
    (
        "GOSAIKUNDA_SNANA_ARAMBHA",
        "DASHAHARA_SNANA_ARAMBHA",
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: गोसाइकुण्डस्नानारम्भ: the day of DASHAHARA_SNANA_ARAMBHA",
    ),
    (
        "GOSAIKUNDA_SNANA_SAMAPTI",
        "GANGA_DASHAHARA",
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: गोसाइँकुण्ड स्नानसमाप्ति: the day of GANGA_DASHAHARA",
    ),
    (
        "HATYAMOCHANA_SNANA",
        "GANGA_DASHAHARA",
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: हत्यामोचन तीर्थस्नान: the day of GANGA_DASHAHARA",
    ),
    (
        "AMUKTABHARANA_SAPTAMI",
        "GAURA_SAPTAMI",
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 12: अमुक्ताभरणसप्तमी: the day of GAURA_SAPTAMI",
    ),
    (
        "SORHA_SHRADDHA_ARAMBHA",
        "PRATIPADA_SHRADDHA",
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: सोह्रश्राद्ध सुरु: the day of PRATIPADA_SHRADDHA",
    ),
    (
        "KALI_YUGADI",
        "TRAYODASHI_SHRADDHA",
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: कलियुगादि: the day of TRAYODASHI_SHRADDHA; the yugadi shraddha in the afternoon, as the 13th's shraddha",
    ),
    (
        "LAXMI_PRASAD_DEVKOTA_JAYANTI",
        "LAKSHMI_PUJA",
        "kept in Nepal, though the national panchanga prints it in neither VS 2082 nor 2083 (C613): the day of LAKSHMI_PUJA; the poet's birthday, kept on Lakshmi puja's day",
    ),
    (
        "NEPAL_SAMBAT_NEW_YEAR",
        "MHA_PUJA",
        "Nepal's national panchanga, VS 2082 p. 15: ने.सं.११४६ प्रारम्भ; VS 2083 p. 17: ने.सं.११४७ प्रारम्भ: the day of MHA_PUJA; Kachhala's first day, which the print opens on Mha puja's day",
    ),
    (
        "BHISHMA_PANCHAKA_ARAMBHA",
        "HARIBODHINI_EKADASHI",
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: भीष्मपञ्चकारम्भ: the day of HARIBODHINI_EKADASHI",
    ),
    (
        "TULASI_VIVAHA",
        "HARIBODHINI_EKADASHI",
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: तुलसीविवाह: the day of HARIBODHINI_EKADASHI",
    ),
    (
        "NUWAKOT_DUPCHESHWAR_MELA",
        "DATTATREYA_JAYANTI",
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: नुवाकोट दुप्चेश्वरमेला: the day of DATTATREYA_JAYANTI; VS 2083 prints it on the full moon, the day after (C611)",
    ),
    (
        "SHILA_CHARHE",
        "SHIVARATRI",
        "Nepal's national panchanga, VS 2082 p. 22 and VS 2083 p. 24: (शिलाचःह्रेपूजा): the day of SHIVARATRI; the Newar name of Shivaratri's night (C614)",
    ),
];

impl FestivalRule {
    /// The days Nepal's national panchanga prints beyond [`FestivalRule::nepal`],
    /// each rule fitted to the committee's VS 2082 and 2083 editions and
    /// citing the page it was read from (C600). Its keys do not repeat
    /// `nepal()`'s, so the two compose as one pack.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let fitted = FestivalRule::nepal_panchanga();
    /// let sita = fitted.iter().find(|rule| rule.key == "SITA_NAVAMI").ok_or("shipped")?;
    /// assert!(sita.source.contains("C601"));
    /// let nepal = FestivalRule::nepal();
    /// assert!(nepal.iter().all(|rule| fitted.iter().all(|other| other.key != rule.key)));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn nepal_panchanga() -> Vec<FestivalRule> {
        let yearly = RULES.iter().map(|&(key, month, tithi, fit, printed)| {
            (key, Some(month), tithi.into(), fit, printed)
        });
        let monthly = EVERY_MONTH
            .iter()
            .map(|&(key, occurs, fit, printed)| (key, None, occurs, fit, printed));
        yearly
            .chain(monthly)
            .map(|(key, month, occurs, fit, printed)| {
                let (at, decide, otherwise, reading) = fit.decide();
                FestivalRule {
                    key: key.to_owned(),
                    source: format!("{printed}; fitted: {reading}"),
                    month,
                    convention: Convention::Amanta,
                    occurs,
                    in_adhika: month.is_none(),
                    at,
                    decide,
                    otherwise,
                }
            })
            .collect()
    }
}

impl FollowingRule {
    /// The days Nepal's national panchanga prints on another rule's day,
    /// beyond [`FollowingRule::nepal`]: each counts no days from a rule of
    /// [`FestivalRule::nepal`] or [`FestivalRule::nepal_panchanga`].
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::nepal_panchanga();
    /// let new_year = rules.iter().find(|rule| rule.key == "NEPAL_SAMBAT_NEW_YEAR").ok_or("shipped")?;
    /// assert_eq!((new_year.after.as_str(), new_year.days), ("MHA_PUJA", 0));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn nepal_panchanga() -> Vec<FollowingRule> {
        SAME_DAY
            .iter()
            .map(|&(key, after, source)| FollowingRule {
                key: key.to_owned(),
                source: source.to_owned(),
                after: after.to_owned(),
                days: 0,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_fitted_rule_checks_and_keys_are_unique() {
        let mut keys = HashSet::new();
        let shipped = FestivalRule::nepal()
            .into_iter()
            .chain(FestivalRule::nepal_panchanga());
        for rule in shipped {
            rule.check().unwrap();
            assert!(
                rule.source.contains("p. ") || rule.source.contains("C613"),
                "{}",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        let following = FollowingRule::nepal()
            .into_iter()
            .chain(FollowingRule::nepal_panchanga());
        for rule in following {
            rule.check().unwrap();
            assert!(
                keys.contains(&rule.after),
                "{} follows {}",
                rule.key,
                rule.after
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
    }

    #[test]
    fn a_share_fit_is_the_maintainer_s_rule() {
        let (at, decide, otherwise, reading) = Fit::Share(APARAHNA).decide();
        assert_eq!((at, otherwise), (APARAHNA, Choice::Earlier));
        assert_eq!(
            decide,
            [
                Guard::new([case(Case::LaterOnly)], Choice::Later),
                Guard::new([case(Case::Both)], Choice::Earlier),
                Guard::new([Predicate::Greater { day: Which::Later }], Choice::Later),
            ]
        );
        assert!(reading.starts_with("aparahna"));
        let (at, decide, otherwise, _) = Fit::UdayaLater.decide();
        assert_eq!(
            (at, decide.len(), otherwise),
            (Window::Sunrise, 1, Choice::Later)
        );
        assert_eq!(
            (Fit::name(MADHYAHNA), Fit::name(SAYAHNA)),
            ("madhyahna", "sayahna")
        );
        assert!(Fit::name(PURVAHNA).contains("C602"));
    }
}
