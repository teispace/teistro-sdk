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
//!   names, and not at all otherwise (C618);
//! - **the earlier alone**: the earlier day only when it alone holds the
//!   tithi at a window, the later otherwise, as the full-moon fast is kept
//!   at sunset (C200).
//!
//! Two rules are not fitted but read (`BY_TEXT`): *Dharmasindhu* gives
//! Kojagara and Nrisimha jayanti that last decision, at nishitha and at
//! sunset, and the print agrees with it in both editions.
//!
//! Most of the days are the plain udaya rule (`AT_SUNRISE`): the
//! festivals of a patro printed in Nepal that its own sunrise rule
//! reproduces, each gated by the committee's print of both years.
//!
//! The pradosha fast and the weekday fasts are kept in every month, the
//! adhika included, on a day of either paksha (§3.4) or the new moon.

use teistro_core::catalogue::{Masa, Tithi, Vara};
use teistro_core::settings::LunarMonth as Convention;

use super::{
    APARAHNA, Case, Choice, DayPart, FestivalRule, FollowingRule, Guard, InAdhika, MADHYAHNA,
    Occurs, Predicate, Which, Window, by_share, case,
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
    /// The earlier day when the tithi holds its last day muhurta wholly,
    /// the muhurta before sunset; the later otherwise (C684).
    Crescent,
    /// The earlier day only when it alone holds the tithi at a window, the
    /// later otherwise: the full-moon fast's decision at sunset (C200).
    EarlierAlone(Window),
    /// The earlier day whenever it holds the tithi at a window, alone or
    /// with the later, the later otherwise: the Sankashta chaturthi's
    /// decision at moonrise.
    EarlierHolds(Window),
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
            Fit::Share(window) => {
                let (decide, otherwise) = by_share();
                (window, decide, otherwise, format!(
                    "{}: the day holding more of it, in time; the earlier on a tie, both holding it wholly being one, and when neither holds it (C601, C603)",
                    Fit::name(window)
                ))
            }
            Fit::Crescent => (
                Window::DayMuhurta { from: 15, to: 15 },
                vec![Guard::new([Predicate::Wholly { day: Which::Earlier }], Earlier)],
                Later,
                "the earlier day when the tithi holds its last muhurta before sunset wholly, the later otherwise (C684)".to_owned(),
            ),
            Fit::EarlierAlone(window) => (
                window,
                vec![Guard::new([case(Case::EarlierOnly)], Earlier)],
                Later,
                format!(
                    "{}: the earlier day only when it alone holds the tithi there, the later otherwise",
                    Fit::name(window)
                ),
            ),
            Fit::EarlierHolds(window) => (
                window,
                vec![
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new([case(Case::Both)], Earlier),
                ],
                Later,
                format!(
                    "{}: the earlier day when it holds the tithi there, alone or with the later, the later otherwise",
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
            Window::Moonrise => "moonrise",
            Window::Sunset => "sunset",
            Window::Nishitha => "nishitha",
            Window::Sunrise | Window::NightMuhurta { .. } | Window::DayMuhurta { .. } => {
                "the window"
            }
        }
    }
}

/// The fitted rules: key, month (amanta, the nija month), tithi, fit, and
/// where the day is printed.
const RULES: [(&str, Masa, Tithi, Fit, &str); 67] = [
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
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: सीताजयन्ती",
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
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: दशहरास्नानारम्भ; VS 2083 prints it in the adhika month, as *Dharmasindhu* p. 38 keeps Dashahara in a doubled Jyeshtha (C606)",
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
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: गङ्गादशहरा; VS 2083 prints it in the adhika month, as *Dharmasindhu* p. 38 keeps Dashahara in a doubled Jyeshtha (C606)",
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
        "Nepal's national panchanga, VS 2083 p. 5: अनलापुन्हि, the adhika month's full moon; VS 2082, with no adhika month, prints none (C607)",
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
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: उपाङ्गललिताव्रत",
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
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: म्हपूजा",
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
        "GYALPO_LHOSAR",
        Masa::Phalguna,
        Tithi::ShuklaPratipada,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 23: ग्याल्पो ल्होसार on 18 February 2026, and VS 2083 p. 25: \"चन्द्रोदय, ग्याल्पो ल्होसार\" on 9 March 2027, a lunation after the Tibetan Phugpa new year",
    ),
    (
        "SONAM_LHOSAR",
        Masa::Magha,
        Tithi::ShuklaPratipada,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 21: सोनम ल्होसार/तामाङ ल्होसार, and VS 2083 p. 23: सोनम ल्होसार/तामाङ ल्होछार",
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
    (
        "SI_PUNHI",
        Masa::Magha,
        Tithi::Purnima,
        Fit::EarlierAlone(Window::Sunset),
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: (सिपून्हि), on the day of पूर्णिमाव्रत; a patro printed in Nepal gives Magha's bright 5th instead, which neither edition prints",
    ),
    (
        "HAYAGRIVA_JAYANTI",
        Masa::Shravana,
        Tithi::Purnima,
        Fit::EarlierAlone(Window::Sunset),
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: हयग्रीवोत्पत्ति, on the day of पूर्णिमाव्रत",
    ),
    (
        "SHIVA_PAVITRAROPAN",
        Masa::Shravana,
        Tithi::Purnima,
        Fit::EarlierAlone(Window::Sunset),
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: श्रीशिवजीमा पवित्रारोपण, on the day of पूर्णिमाव्रत",
    ),
    (
        "VARAHA_JAYANTI",
        Masa::Shravana,
        Tithi::ShuklaTritiya,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: वराहजयन्ती; a patro printed in Nepal gives Bhadrapada's bright 3rd in most years, a lunation later",
    ),
    (
        "KALKI_JAYANTI",
        Masa::Shravana,
        Tithi::ShuklaShashthi,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: कल्कीजयन्ती; a patro printed in Nepal gives the bright 5th, a day before both printed days",
    ),
    (
        "BAHIDYO_BOYEGU",
        Masa::Shravana,
        Tithi::ShuklaDwadashi,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 9: बहिद्यब्वोय; VS 2083 p. 11: बहिद्यब्बोय",
    ),
    (
        "LALITPUR_NARSINGH_JATRA",
        Masa::Shravana,
        Tithi::KrishnaPanchami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: ल.पु. नृसिंहयात्रा (उपाकलं सायाः)",
    ),
];

/// The rules a text gives, which the committee's print follows in both
/// editions: key, month (amanta, the nija month), tithi, the decision,
/// and the text with the page it was read from.
const BY_TEXT: [(&str, Masa, Tithi, Fit, &str); 4] = [
    (
        "NARASIMHA_JAYANTI",
        Masa::Vaishakha,
        Tithi::ShuklaChaturdashi,
        Fit::EarlierAlone(Window::Sunset),
        "Dharmasindhu p. 37, read off the page image: \"वैशाखशुक्लचतुर्दशी नृसिंहजयन्ती सा सूर्यास्तमयकालव्यापिनी ग्राह्या दिनद्वये तद्व्याप्तौ तदव्याप्तौ वा परैव\"; Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: नृसिंहजयन्ती",
    ),
    (
        "KOJAGRAT_PURNIMA",
        Masa::Ashwina,
        Tithi::Purnima,
        Fit::EarlierAlone(Window::Nishitha),
        "Dharmasindhu p. 74, read off the page image: \"सा पूर्वत्रैव निशीथव्याप्तौ पूर्वा, उत्तरदिने एव दिनद्वयेऽपि वा निशीथव्याप्तौ दिनद्वये निशीथास्पर्शे वा उत्तरैव\", pradosha being only some authorities' reading (C617); Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: कोजाग्रतव्रत",
    ),
    (
        "GOVATSA_DWADASHI",
        Masa::Ashwina,
        Tithi::KrishnaDwadashi,
        Fit::EarlierHolds(Window::Pradosha),
        "Dharmasindhu p. 76, read off the OCR'd page: \"आश्विनकृष्णद्वादशी गोवत्सद्वादशी सा प्रदोषव्यापिनी ग्राह्या दिनद्वये तदव्याप्तौ परा … उभयत्र तद्व्याप्तौ पूर्वेति बहवः\", the earlier as most authorities hold; Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: गोवत्सद्वादशी, printed both years beside रमाएकादशीव्रत on the day whose sunrise holds the 11th",
    ),
    (
        "KASHI_BADA_GANESH_UTSAV",
        Masa::Pausha,
        Tithi::KrishnaChaturthi,
        Fit::EarlierHolds(Window::Moonrise),
        "Dharmasindhu p. 9, the Sankashta chaturthi: the dark 4th at moonrise (चन्द्रोदयव्यापिनी), the earlier day, joined to the 3rd, when both hold it; Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 22: काशीमा बडागणेश उत्सव, the purnimanta Magha's dark 4th, printed both years on the day whose sunrise holds the 3rd",
    ),
];

/// The days the committee prints on the udaya rule, each read off both
/// editions: key, month (amanta, the nija month), tithi, and where the day
/// is printed. Each also gives every day a patro printed in Nepal verified
/// for it from 2013 to 2027, by a back-test kept outside the repository
/// (`festival-classification.md`).
const AT_SUNRISE: [(&str, Masa, Tithi, &str); 164] = [
    (
        "SAMVATSARA_ARAMBHA",
        Masa::Chaitra,
        Tithi::ShuklaPratipada,
        "Nepal's national panchanga, VS 2082 p. 24 and VS 2083 p. 27: वत्सरारम्भ",
    ),
    (
        "GAURI_VRAT",
        Masa::Chaitra,
        Tithi::ShuklaTritiya,
        "Nepal's national panchanga, VS 2082 p. 25 and VS 2083 p. 27: गौरीव्रत",
    ),
    (
        "MAHAVIR_JAYANTI",
        Masa::Chaitra,
        Tithi::ShuklaTrayodashi,
        "Nepal's national panchanga, VS 2082 p. 25: महावीरजयन्ती",
    ),
    (
        "BALAJU_BAISDHARA_MELA",
        Masa::Chaitra,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 25: बालाजु बाइसधारा मेला",
    ),
    (
        "HANUMAN_JAYANTI",
        Masa::Chaitra,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 25: हनुमानजयन्ती",
    ),
    (
        "BISHALNAGAR_VAISHNAVI_DEVI_JATRA",
        Masa::Chaitra,
        Tithi::KrishnaPratipada,
        "Nepal's national panchanga, VS 2082 p. 26: विशालनगरमा वैष्णवीदेवीयात्रा",
    ),
    (
        "CHANGUNARAYAN_CHHINNAMASTA_KILESHWOR_MULRATH_JATRA",
        Masa::Chaitra,
        Tithi::Amavasya,
        "Nepal's national panchanga, VS 2082 p. 2 and VS 2083 p. 2: चाँगुनारायण छिन्नमस्ता कीलेश्वर मूलरथयात्रा",
    ),
    (
        "MATA_TIRTHA_AUNSI",
        Masa::Chaitra,
        Tithi::Amavasya,
        "Nepal's national panchanga, VS 2082 p. 2 and VS 2083 p. 2: मातातीर्थस्नान",
    ),
    (
        "VARUTHINI_EKADASHI",
        Masa::Chaitra,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 2: बरुथिनीएकादशीव्रत; VS 2082 p. 26: बरुथिनी एकादशीव्रत; the Smartas' day, as the print names it where the Vaishnavas' differs (C688)",
    ),
    (
        "RATO_MACHHINDRANATH_RATHAROHAN",
        Masa::Vaishakha,
        Tithi::ShuklaPratipada,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: ल.पु.मत्स्येन्द्रनाथ रथारोहण",
    ),
    (
        "AKSHAYA_TRITIYA",
        Masa::Vaishakha,
        Tithi::ShuklaTritiya,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: अक्षयतृतीया",
    ),
    (
        "TRETAYUGADI",
        Masa::Vaishakha,
        Tithi::ShuklaTritiya,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: त्रेतायुगादि",
    ),
    (
        "RATO_MACHHINDRANATH_RATH_JATRA_BEGINS",
        Masa::Vaishakha,
        Tithi::ShuklaChaturthi,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: ल.पु.श्रीमत्स्येन्द्रनाथ रथयात्रारम्भ",
    ),
    (
        "PASHUPATINATH_DAMAN_PUSHPARPAN",
        Masa::Vaishakha,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: श्रीपशुपतिनाथमा दमनार्पण",
    ),
    (
        "MOHINI_EKADASHI",
        Masa::Vaishakha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 3: मोहिनी एकादशीव्रत; VS 2083 p. 3: मोहिनीएकादशीव्रत",
    ),
    (
        "BAISAKH_SNAN_ENDS",
        Masa::Vaishakha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: वैशाखस्नानसमाप्ति",
    ),
    (
        "BUDDHA_JAYANTI",
        Masa::Vaishakha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: गौतमबुद्धजयन्ती",
    ),
    (
        "CHANDI_PURNIMA",
        Masa::Vaishakha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: चण्डीपूर्णिमा",
    ),
    (
        "GORAKHNATH_JAYANTI",
        Masa::Vaishakha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: गोरखनाथजयन्ती",
    ),
    (
        "UBHAULI",
        Masa::Vaishakha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: उभौलीपर्व",
    ),
    (
        "APARA_EKADASHI",
        Masa::Vaishakha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 4 and VS 2083 p. 4: अपराएकादशीव्रत",
    ),
    (
        "SHANI_JAYANTI",
        Masa::Vaishakha,
        Tithi::Amavasya,
        "Nepal's national panchanga, VS 2082 p. 4 and VS 2083 p. 4: शनिजयन्ती",
    ),
    (
        "BHAKTAPUR_CHANDI_BHAGAWATI_JATRA",
        Masa::Jyeshtha,
        Tithi::ShuklaShashthi,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 7: भ.पु.चण्डीभगवतीयात्रा",
    ),
    (
        "KUMAR_SHASHTHI",
        Masa::Jyeshtha,
        Tithi::ShuklaShashthi,
        "Nepal's national panchanga, VS 2082 p. 5: कुमारषष्ठी (सिथिनख); VS 2083 p. 7: कुमारषष्ठी",
    ),
    (
        "SITHI_NAKHA",
        Masa::Jyeshtha,
        Tithi::ShuklaShashthi,
        "Nepal's national panchanga, VS 2082 p. 5: कुमारषष्ठी (सिथिनख); VS 2083 p. 7: (सिथिनख)",
    ),
    (
        "VAYU_ASHTAMI",
        Masa::Jyeshtha,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 5: वायुअष्टमी; VS 2083 p. 7: वायु अष्टमी",
    ),
    (
        "GANGA_PUJA",
        Masa::Jyeshtha,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: रामेश्वरगङ्गापूजा; in VS 2083 on the adhika Jyeshtha's bright 10th, as Dashahara (C606, C691)",
    ),
    (
        "GURU_PADMASAMBHAVA_JAYANTI",
        Masa::Jyeshtha,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: वज्रगुरु पद्मसम्भवजयन्ती; in VS 2083 on the adhika Jyeshtha's bright 10th, as Dashahara (C606, C691)",
    ),
    (
        "RAMESHWAR_PUJA",
        Masa::Jyeshtha,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 5: रामेश्वरगङ्गापूजा; in VS 2083 on the adhika Jyeshtha's bright 10th, as Dashahara (C606, C691)",
    ),
    (
        "JYA_PUNHI",
        Masa::Jyeshtha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 7: (ज्याः पुन्हि)",
    ),
    (
        "KABIR_JAYANTI",
        Masa::Jyeshtha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 7: कवीरजयन्ती",
    ),
    (
        "MASTA_PURNIMA",
        Masa::Jyeshtha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 7: मष्टपूर्णिमा",
    ),
    (
        "PANAUTI_SNAN_MELA",
        Masa::Jyeshtha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 7: पनौतीस्नान, on the full moon's day, not the day before when पूर्णिमाव्रत is kept",
    ),
    (
        "PANAUTI_JATRA",
        Masa::Jyeshtha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 5: पनौती भैरवभद्रकाली इन्द्रेश्वर रथयात्रा; VS 2083 p. 7: पनौती भैरवभद्रकाली इन्द्रेश्वररथयात्रा, the text after Ⓑ that ends the full moon's row",
    ),
    (
        "BHAL_BHAL_ASHTAMI",
        Masa::Jyeshtha,
        Tithi::KrishnaAshtami,
        "Nepal's national panchanga, VS 2082 p. 6 and VS 2083 p. 8: भलभलअष्टमी",
    ),
    (
        "DEOPATAN_TRISHUL_JATRA",
        Masa::Jyeshtha,
        Tithi::KrishnaAshtami,
        "Nepal's national panchanga, VS 2082 p. 6 and VS 2083 p. 8: देउपाटनमा त्रिशूलयात्रा",
    ),
    (
        "NAKWA_DISHI",
        Masa::Jyeshtha,
        Tithi::KrishnaDashami,
        "Nepal's national panchanga, VS 2082 p. 6 and VS 2083 p. 8: नक्वादिशी",
    ),
    (
        "YOGINI_EKADASHI",
        Masa::Jyeshtha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 6: स्मार्तहरूको योगिनी एकादशीव्रत; VS 2083 p. 8: स्मार्तहरुको योगिनी एकादशीव्रत; the Smartas' day, the Vaishnavas' printed the day after (C688)",
    ),
    (
        "JAGANNATH_RATH_JATRA",
        Masa::Ashadha,
        Tithi::ShuklaDvitiya,
        "Nepal's national panchanga, VS 2082 p. 7 and VS 2083 p. 9: श्रीजगन्नाथरथयात्रा",
    ),
    (
        "DEOPATAN_GANGAMAI_RATH_JATRA",
        Masa::Ashadha,
        Tithi::ShuklaSaptami,
        "Nepal's national panchanga, VS 2082 p. 7 and VS 2083 p. 9: देउपाटनमा गङ्गामाई रथयात्रा",
    ),
    (
        "SURYA_PUJA",
        Masa::Ashadha,
        Tithi::ShuklaSaptami,
        "Nepal's national panchanga, VS 2082 p. 7 and VS 2083 p. 9: सूर्यपूजा",
    ),
    (
        "CHATURMAS_BEGINS",
        Masa::Ashadha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 7 and VS 2083 p. 9: चतुर्मासव्रत सुरु",
    ),
    (
        "HARISHAYANI_EKADASHI",
        Masa::Ashadha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 7: हरिशयनीएकादशीव्रत; VS 2083 p. 9: हरिशयनी एकादशीव्रत",
    ),
    (
        "DAKSHINAMURTI_PUJA",
        Masa::Ashadha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 7 and VS 2083 p. 9: दक्षिणामूर्तिपूजा",
    ),
    (
        "DILLA_PUNHI",
        Masa::Ashadha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 7 and VS 2083 p. 9: (दिल्लापून्हि)",
    ),
    (
        "GURU_PURNIMA",
        Masa::Ashadha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 7 and VS 2083 p. 9: गुरुपूर्णिमा",
    ),
    (
        "KAMIKA_EKADASHI",
        Masa::Ashadha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 8 and VS 2083 p. 10: कामिकाएकादशीव्रत",
    ),
    (
        "GUNLA_BEGINS",
        Masa::Shravana,
        Tithi::ShuklaPratipada,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: गुँलाधर्मारम्भ",
    ),
    (
        "DHAPAKHEL_NAGDAHA",
        Masa::Shravana,
        Tithi::ShuklaPanchami,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: धापाखेल नागदह",
    ),
    (
        "NAG_PANCHAMI",
        Masa::Shravana,
        Tithi::ShuklaPanchami,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: नागपञ्चमी",
    ),
    (
        "TULSIDAS_JAYANTI",
        Masa::Shravana,
        Tithi::ShuklaSaptami,
        "Nepal's national panchanga, VS 2082 p. 9: गोस्वामी तुलसीदासजयन्ती; VS 2083 p. 11: गोस्वामी तुलसीदास जयन्ती",
    ),
    (
        "YALA_PANCHADAN",
        Masa::Shravana,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 9: (यल पञ्चदान); VS 2083 p. 11: (यँलपञ्चदान)",
    ),
    (
        "SHRAWAN_PUTRADA_EKADASHI",
        Masa::Shravana,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: पुत्रदाएकादशीव्रत",
    ),
    (
        "KWATI_PUNHI",
        Masa::Shravana,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: व्याँजानके गुँ पुन्हि",
    ),
    (
        "SANSKRIT_DAY",
        Masa::Shravana,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: संस्कृतदिवस",
    ),
    (
        "GAI_JATRA",
        Masa::Shravana,
        Tithi::KrishnaPratipada,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: गाईजात्रा",
    ),
    (
        "ROPAI_JATRA",
        Masa::Shravana,
        Tithi::KrishnaDvitiya,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: रोपाईजात्रा",
    ),
    (
        "CHANDRA_SHASHTHI",
        Masa::Shravana,
        Tithi::KrishnaShashthi,
        "Nepal's national panchanga, VS 2083 p. 12: चन्द्रषष्ठी",
    ),
    (
        "GUGA_NAVAMI",
        Masa::Shravana,
        Tithi::KrishnaNavami,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: गूँगानवमी",
    ),
    (
        "KUSHE_AUNSI",
        Masa::Shravana,
        Tithi::Amavasya,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: कुशेऔंसी, बाबुको मुख हेर्नेदिन",
    ),
    (
        "MOTIRAM_JAYANTI",
        Masa::Shravana,
        Tithi::Amavasya,
        "Nepal's national panchanga, VS 2082 p. 10: मोतिजयन्ती; VS 2083 p. 12: मोतीजयन्ती",
    ),
    (
        "GUNLA_ENDS",
        Masa::Bhadrapada,
        Tithi::ShuklaPratipada,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: गुँलाधर्मसमाप्ति",
    ),
    (
        "DAR_KHANE_DIN",
        Masa::Bhadrapada,
        Tithi::ShuklaDvitiya,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: दरखाने दिन",
    ),
    (
        "KOKHAJA_BIYE",
        Masa::Bhadrapada,
        Tithi::ShuklaPanchami,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: (क्वखजाविय)",
    ),
    (
        "SURYA_SHASHTHI",
        Masa::Bhadrapada,
        Tithi::ShuklaShashthi,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: सूर्यषष्ठी",
    ),
    (
        "KAYA_ASHTAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: काय अष्टमी",
    ),
    (
        "ADUKHA_NAVAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: अदुःखनवमीव्रत",
    ),
    (
        "HARIPARIVARTINI_EKADASHI",
        Masa::Bhadrapada,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: हरिपरिवर्तिनी एकादशीव्रत",
    ),
    (
        "INDRADHWAJA_UTTHAN",
        Masa::Bhadrapada,
        Tithi::ShuklaDwadashi,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: इन्द्रध्वजोत्थान (मतविए)",
    ),
    (
        "ANANTA_CHATURDASHI",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturdashi,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: अनन्त चतुर्दशीव्रत",
    ),
    (
        "INDRA_JATRA",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturdashi,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: कुमारीइन्द्रयात्रा",
    ),
    (
        "BHAKTAPUR_SIDDHA_POKHARI_MELA",
        Masa::Bhadrapada,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: भ.पु.सिद्धपोखरीमेला",
    ),
    (
        "CHONAM_PARVA",
        Masa::Bhadrapada,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 11: चेपाङचोनामपर्व; VS 2083 p. 13: चेपाङ-चोनामपर्व",
    ),
    (
        "INDRADAHA_SNAN",
        Masa::Bhadrapada,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: इन्द्रदहस्नान",
    ),
    (
        "PUNCHALI_BHUJA",
        Masa::Bhadrapada,
        Tithi::KrishnaPratipada,
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: पुञ्चलीभूजा",
    ),
    (
        "GATILA",
        Masa::Bhadrapada,
        Tithi::KrishnaTritiya,
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: (गातिला)",
    ),
    (
        "JITIYA",
        Masa::Bhadrapada,
        Tithi::KrishnaAshtami,
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: (जितियापर्व)",
    ),
    (
        "INDIRA_EKADASHI",
        Masa::Bhadrapada,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: इन्दिराएकादशीव्रत",
    ),
    (
        "MAGHA_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::KrishnaTrayodashi,
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: मघाश्राद्ध",
    ),
    (
        "PACHALI_BHAIRAV_JATRA",
        Masa::Ashwina,
        Tithi::ShuklaPanchami,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: पचलीभैरवयात्रा",
    ),
    (
        "BILVA_NIMANTRAN",
        Masa::Ashwina,
        Tithi::ShuklaShashthi,
        "Nepal's national panchanga, VS 2082 p. 13: विल्व निमन्त्रण; VS 2083 p. 15: बिल्वनिमन्त्रण",
    ),
    (
        "PHULPATI",
        Masa::Ashwina,
        Tithi::ShuklaSaptami,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: फूलपाती",
    ),
    (
        "MAHA_ASHTAMI",
        Masa::Ashwina,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 13: महाअष्टमीव्रत; VS 2083 p. 15: महाष्टमीव्रत",
    ),
    (
        "BUDDHAVATAR",
        Masa::Ashwina,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: बौद्धावतार",
    ),
    (
        "MAHA_NAVAMI",
        Masa::Ashwina,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: महानवमी",
    ),
    (
        "SYAKWA_TYAKWA",
        Masa::Ashwina,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: (स्याक्वत्याक्व)",
    ),
    (
        "CHAALA",
        Masa::Ashwina,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2083 p. 15: (चालं)",
    ),
    (
        "DEVI_VISARJAN",
        Masa::Ashwina,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: देवी विसर्जन",
    ),
    (
        "KHADGA_JATRA",
        Masa::Ashwina,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: खड्गयात्रा",
    ),
    (
        "ANNAPURNA_JATRA",
        Masa::Ashwina,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 13: अन्नपूर्णयात्रा; VS 2083 p. 15: अन्नपूर्णायात्रा",
    ),
    (
        "PAPANKUSHA_EKADASHI",
        Masa::Ashwina,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: पापाङ्कुशा एकादशीव्रत",
    ),
    (
        "KHANCHI_SARAI_PARVA",
        Masa::Ashwina,
        Tithi::ShuklaTrayodashi,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: खाँचीमा सरायपर्व",
    ),
    (
        "AKHILBALI_PURTI",
        Masa::Ashwina,
        Tithi::ShuklaChaturdashi,
        "Nepal's national panchanga, VS 2082 p. 13: अखिलबलिपूर्ति (सिघया); VS 2083 p. 15: अखिलबलिपूर्ति",
    ),
    (
        "SHREEGHA_JATRA",
        Masa::Ashwina,
        Tithi::ShuklaChaturdashi,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: (सिघया), the Newar name (C692)",
    ),
    (
        "ARGHA_SARAI_PARVA",
        Masa::Ashwina,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 13: अर्घासरायपर्व; VS 2083 p. 15: अर्घामा सरायपर्व",
    ),
    (
        "KARTIK_SNAN_BEGINS",
        Masa::Ashwina,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 13: कार्तिकस्नान; VS 2083 p. 15: कार्तिकस्नान आरम्भ",
    ),
    (
        "KATIN_PUNHI",
        Masa::Ashwina,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: कतिं पुन्हि",
    ),
    (
        "RADHASHTAMI",
        Masa::Ashwina,
        Tithi::KrishnaAshtami,
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: राधाष्टमी",
    ),
    (
        "RAMA_EKADASHI",
        Masa::Ashwina,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: रमाएकादशीव्रत",
    ),
    (
        "DHANTERAS",
        Masa::Ashwina,
        Tithi::KrishnaDwadashi,
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: धन्तेरस",
    ),
    (
        "KAG_TIHAR",
        Masa::Ashwina,
        Tithi::KrishnaTrayodashi,
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: कागतिहार",
    ),
    (
        "SWONTI_CHARHE",
        Masa::Ashwina,
        Tithi::KrishnaTrayodashi,
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: (स्वन्तिचह्रेपूजा)",
    ),
    (
        "KUKUR_TIHAR",
        Masa::Ashwina,
        Tithi::KrishnaChaturdashi,
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: कुकुरतिहार",
    ),
    (
        "NARAK_CHATURDASHI",
        Masa::Ashwina,
        Tithi::KrishnaChaturdashi,
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: नरक चतुर्दशी",
    ),
    (
        "GOVARDHAN_PUJA",
        Masa::Kartika,
        Tithi::ShuklaPratipada,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: गोवर्धनपूजा",
    ),
    (
        "HALI_TIHAR",
        Masa::Kartika,
        Tithi::ShuklaPratipada,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: हलितिहार",
    ),
    (
        "CHHATH",
        Masa::Kartika,
        Tithi::ShuklaShashthi,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: छठपर्व",
    ),
    (
        "GOPASHTAMI",
        Masa::Kartika,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 15: गोपअष्टमी; VS 2083 p. 17: गोपाष्टमी",
    ),
    (
        "MUKHA_ASHTAMI",
        Masa::Kartika,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 15: मुखअष्टमी; VS 2083 p. 17: मुखाष्टमी",
    ),
    (
        "SILANGA_MAHARUDRA_KOTIHOM_MELA",
        Masa::Kartika,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: बैतडी सिलङ्गामा महारुद्र कोटीहोममेला",
    ),
    (
        "HETAUDA_KUSHMANDA_SAROBAR_MELA",
        Masa::Kartika,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: हेटौडा कुष्माण्ड सरोवरमेला",
    ),
    (
        "KUSHMANDA_NAVAMI",
        Masa::Kartika,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: कुष्माण्डनवमी",
    ),
    (
        "SAKWO_CHANGU_WONE",
        Masa::Kartika,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 15: साँखुचाँगुमेला (सकोचंगुवने); VS 2083 p. 17: साँखुचाँगुमेला(सकोचंगुवने)",
    ),
    (
        "SATYAYUGADI",
        Masa::Kartika,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: सत्ययुगादि",
    ),
    (
        "BALAMBU_MAHALAXMI_JATRA",
        Masa::Kartika,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: बलम्बु महालक्ष्मीयात्रा",
    ),
    (
        "FULKHARKA_GANGAJAMUNA_MELA",
        Masa::Kartika,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: धादिङ फूलखर्क गङ्गाजमुनामेला",
    ),
    (
        "CHANGUNARAYAN_AKHANDA_DEEP_DARSHAN",
        Masa::Kartika,
        Tithi::ShuklaDwadashi,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: चाँगुनारायण अखण्डदीपदर्शन",
    ),
    (
        "SHIVA_BAIKUNTHA_CHATURDASHI",
        Masa::Kartika,
        Tithi::ShuklaChaturdashi,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: बैकुण्ठ चतुर्दशीव्रत",
    ),
    (
        "CHATURMAS_ENDS",
        Masa::Kartika,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 15: चतुर्मासव्रत समाप्ति; VS 2083 p. 17: कार्तिकस्नान चतुर्मासव्रत समाप्ति",
    ),
    (
        "GURU_NANAK_JAYANTI",
        Masa::Kartika,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: गुरुनानकजयन्ती",
    ),
    (
        "KARTIK_SNAN_ENDS",
        Masa::Kartika,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: कार्तिकस्नान चतुर्मासव्रत समाप्ति",
    ),
    (
        "THANKOT_MAHALAXMI_JATRA",
        Masa::Kartika,
        Tithi::KrishnaPratipada,
        "Nepal's national panchanga, VS 2082 p. 16 and VS 2083 p. 18: थानकोट महालक्ष्मीयात्रा",
    ),
    (
        "GUHYESHWARI_JATRA",
        Masa::Kartika,
        Tithi::KrishnaDashami,
        "Nepal's national panchanga, VS 2082 p. 16 and VS 2083 p. 18: गुह्येश्वरीयात्रा",
    ),
    (
        "UTPATTIKA_EKADASHI",
        Masa::Kartika,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 16 and VS 2083 p. 18: उत्पत्तिका एकादशीव्रत",
    ),
    (
        "BALA_CHATURDASHI_BALA",
        Masa::Kartika,
        Tithi::KrishnaChaturdashi,
        "Nepal's national panchanga, VS 2082 p. 16 and VS 2083 p. 18: बालाचतुर्दशी",
    ),
    (
        "SHADANANDA_JAYANTI",
        Masa::Margashirsha,
        Tithi::ShuklaPanchami,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: आबाल ब्रह्मचारी षडानन्दजयन्ती",
    ),
    (
        "VIVAH_PANCHAMI",
        Masa::Margashirsha,
        Tithi::ShuklaPanchami,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: विवाहपञ्चमी",
    ),
    (
        "BAKHUMADA",
        Masa::Margashirsha,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: (वखुमद)",
    ),
    (
        "PANGA_VAISHNAVI_DEVI_JATRA",
        Masa::Margashirsha,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: पाङ्गा वैष्णवदेवीयात्रा",
    ),
    (
        "KIRTIPUR_INDRAYANI_JATRA",
        Masa::Margashirsha,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: कीर्तिपुरमा इन्द्राणीयात्रा",
    ),
    (
        "GITA_JAYANTI",
        Masa::Margashirsha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: गीताजयन्ती",
    ),
    (
        "MOKSHADA_EKADASHI",
        Masa::Margashirsha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: मोक्षदा एकादशीव्रत",
    ),
    (
        "DHANYA_PURNIMA",
        Masa::Margashirsha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: धान्यपूर्णिमा",
    ),
    (
        "UDHAULI",
        Masa::Margashirsha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: उधौलीपर्व",
    ),
    (
        "YOMARI_PUNHI",
        Masa::Margashirsha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 17: (योमरीपून्हि, थिंलापून्हि); VS 2083 p. 19: (योमरीपून्हि)",
    ),
    (
        "SAFALA_EKADASHI",
        Masa::Margashirsha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 18 and VS 2083 p. 20: सफला एकादशीव्रत",
    ),
    (
        "GAYA_MAUNI_AUNSI",
        Masa::Margashirsha,
        Tithi::Amavasya,
        "Nepal's national panchanga, VS 2082 p. 18: गयामौनी औसी; VS 2083 p. 20: गयामौनी औंसी",
    ),
    (
        "TOL_LHOSAR",
        Masa::Pausha,
        Tithi::ShuklaPratipada,
        "Nepal's national panchanga, VS 2082 p. 19 and VS 2083 p. 21: तोल ल्होसार",
    ),
    (
        "SETO_MACHHINDRANATH_SNAN",
        Masa::Pausha,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 19: श्रीश्वेतमत्स्येन्द्रनाथस्नान (जनबहाद्यन्हव); VS 2083 p. 21: श्रीश्वेतमत्स्येन्द्रनाथस्नान (जनबाद्यन्हव)",
    ),
    (
        "MUSYADULI",
        Masa::Pausha,
        Tithi::ShuklaDwadashi,
        "Nepal's national panchanga, VS 2082 p. 19 and VS 2083 p. 21: मुस्याडुली",
    ),
    (
        "CHANGUNARAYAN_KALASH_JATRA",
        Masa::Pausha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 19 and VS 2083 p. 21: (चाँगुनारायण कलशयात्रा)",
    ),
    (
        "MAGH_SNAN_BEGINS",
        Masa::Pausha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 19 and VS 2083 p. 21: माघस्नानसुरु",
    ),
    (
        "SWASTHANI_VRAT_BEGINS",
        Masa::Pausha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 19 and VS 2083 p. 21: श्रीस्वस्थानीव्रत सुरु",
    ),
    (
        "SHATTILA_EKADASHI",
        Masa::Pausha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 22: षट्तिला एकादशीव्रत",
    ),
    (
        "ARYAGHAT_MADHAV_NARAYAN_MELA",
        Masa::Pausha,
        Tithi::Amavasya,
        "Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 22: आर्यघाटमा माधवनारायण मेला",
    ),
    (
        "SHREE_BALLABH_JAYANTI",
        Masa::Magha,
        Tithi::ShuklaPratipada,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: श्रीवल्लभजयन्ती",
    ),
    (
        "BASANTA_PANCHAMI",
        Masa::Magha,
        Tithi::ShuklaPanchami,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: वसन्तपञ्चमी",
    ),
    (
        "SKANDA_SHASHTHI",
        Masa::Magha,
        Tithi::ShuklaShashthi,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: स्कन्दषष्ठी",
    ),
    (
        "ACHALA_SAPTAMI",
        Masa::Magha,
        Tithi::ShuklaSaptami,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: अचलासप्तमी",
    ),
    (
        "BHISHMASHTAMI",
        Masa::Magha,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: भीष्माष्टमी",
    ),
    (
        "DRONA_NAVAMI",
        Masa::Magha,
        Tithi::ShuklaNavami,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: द्रोणनवमी",
    ),
    (
        "SHALYA_DASHAMI",
        Masa::Magha,
        Tithi::ShuklaDashami,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: शल्यदशमी",
    ),
    (
        "BHISHMA_DWADASHI",
        Masa::Magha,
        Tithi::ShuklaDwadashi,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: भीष्मद्वादशी",
    ),
    (
        "CHANGU_MADHAV_NARAYAN_MELA",
        Masa::Magha,
        Tithi::ShuklaDwadashi,
        "Nepal's national panchanga, VS 2082 p. 21: चाँगुमाधवनारायणमेला; VS 2083 p. 23: चाँगुनारायणमेला",
    ),
    (
        "WATU_MAGH_JATRA",
        Masa::Magha,
        Tithi::ShuklaDwadashi,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: वटुमाघयात्रा",
    ),
    (
        "MAGH_SNAN_ENDS",
        Masa::Magha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: माघस्नान समाप्ति",
    ),
    (
        "SWASTHANI_VRAT_ENDS",
        Masa::Magha,
        Tithi::Purnima,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: श्रीस्वस्थानीव्रतसमाप्ति",
    ),
    (
        "CHIR_SWAYEGU",
        Masa::Phalguna,
        Tithi::ShuklaAshtami,
        "Nepal's national panchanga, VS 2082 p. 23: चिरोत्थान; VS 2083 p. 25: चिरस्वाय",
    ),
    (
        "AMALAKI_EKADASHI",
        Masa::Phalguna,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 23 and VS 2083 p. 25: आमलकी एकादशीव्रत",
    ),
    (
        "GOVINDA_DWADASHI",
        Masa::Phalguna,
        Tithi::ShuklaDwadashi,
        "Nepal's national panchanga, VS 2082 p. 23 and VS 2083 p. 25: गोविन्दद्वादशी",
    ),
    (
        "NALA_MACHHINDRANATH_RATH_JATRA",
        Masa::Phalguna,
        Tithi::KrishnaTritiya,
        "Nepal's national panchanga, VS 2082 p. 24: नालायात्रा; VS 2083 p. 26: नालायात्रा, नाला मत्स्येन्द्रनाथ यात्रा",
    ),
    (
        "SHITALASHTAMI",
        Masa::Phalguna,
        Tithi::KrishnaAshtami,
        "Nepal's national panchanga, VS 2082 p. 24 and VS 2083 p. 26: शीतलाष्टमी",
    ),
    (
        "PAPMOCHANI_EKADASHI",
        Masa::Phalguna,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 24 and VS 2083 p. 26: पापमोचनी एकादशीव्रत",
    ),
    (
        "KAMADA_EKADASHI",
        Masa::Chaitra,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 25: कामदाएकादशीव्रत; the Smartas' day, as Varuthini and Yogini ekadashi (C688)",
    ),
];

/// The ekadashis kept in an adhika month alone, whichever month is
/// doubled, on the udaya rule as the committee prints them: key, tithi,
/// and where the day is printed (§3.7, §3.10). VS 2082, with no adhika
/// month, prints neither.
const IN_ADHIKA: [(&str, Tithi, &str); 2] = [
    (
        "PADMINI_EKADASHI",
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2083 p. 5: पद्मिनी एकादशीव्रत, the adhika Jyeshtha's bright 11th",
    ),
    (
        "PARAMA_EKADASHI",
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2083 p. 6: परमा एकादशीव्रत, the adhika Jyeshtha's dark 11th",
    ),
];

/// The fitted rules kept in every month, the adhika included, as the
/// committee prints them in both editions: key, what is kept, fit, and
/// where the days are printed (§3.4, §3.5).
const EVERY_MONTH: [(&str, Occurs, Fit, &str); 7] = [
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
    (
        "CHANDRODAYA",
        Occurs::Tithi {
            tithi: Tithi::ShuklaDvitiya,
        },
        Fit::Crescent,
        "Nepal's national panchanga, the first row of every bright half of VS 2082 (pp. 3–25) and VS 2083 (pp. 3–27): चन्द्रोदय, the first evening the crescent is named, the day after the new moon's; 18 February 2026 is not printed, its 2nd beginning 26 minutes before sunset, and every printed day's began at least 166 minutes before",
    ),
];

/// The days printed on another rule's day: key, the rule it keeps the
/// day of, and where.
const SAME_DAY: [(&str, &str, &str); 15] = [
    (
        "FAGU_PURNIMA",
        "HOLIKA",
        "Government of Nepal, Ministry of Home Affairs, public holidays of 2082 and 2083: Fagu Purnima, for the 56 hill and mountain districts, on 2082/11/18 and 2083/12/07, the days VS 2082 p. 23 and VS 2083 p. 25 print पहाडी जिल्लामा होली (C693)",
    ),
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
    (
        "ARJUNDHARA_MELA",
        "PASHUPATI_HALESI_MELA",
        "Nepal's national panchanga, VS 2082 p. 16: सताक्षीधाम तथा अर्जुनधारा मेला; VS 2083 p. 18: झापा सताक्षीधाम तथा अर्जुनधारामेला, each continuing the row of हलेसीमहादेवमेला: the day of PASHUPATI_HALESI_MELA; a patro printed in Nepal gives Bala chaturdashi's day, the day after, which VS 2083 does not print",
    ),
];

/// The days printed the day after another rule's day: key, the rule, and
/// where.
const DAY_AFTER: [(&str, &str, &str); 1] = [(
    "KOKA_KAUSHIKI_SANGAM_MELA",
    "DHANYA_PURNIMA",
    "Nepal's national panchanga, VS 2082 p. 18: कोशिकोका सङ्गममा मेला; VS 2083 p. 20: कोशीकोका सङ्गममा मेला: the day after DHANYA_PURNIMA, as a patro printed in Nepal gives it on all 13 of its verified days from 2013 to 2025 (C690)",
)];

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
        // The rules a doubled Jyeshtha keeps in its adhika month (C606),
        // and the one the print names only there (C607); a monthly rule
        // is kept in an adhika month too.
        fn in_adhika(key: &str, month: Option<Masa>) -> InAdhika {
            match key {
                "DASHAHARA_SNANA_ARAMBHA"
                | "GANGA_DASHAHARA"
                | "GANGA_PUJA"
                | "RAMESHWAR_PUJA"
                | "GURU_PADMASAMBHAVA_JAYANTI" => InAdhika::First,
                "ANALA_PUNHI" | "PADMINI_EKADASHI" | "PARAMA_EKADASHI" => InAdhika::Only,
                _ if month.is_none() => InAdhika::Also,
                _ => InAdhika::Nija,
            }
        }
        let yearly = RULES.iter().map(|&(key, month, tithi, fit, printed)| {
            (key, Some(month), tithi.into(), fit, printed, "fitted")
        });
        let at_sunrise = AT_SUNRISE.iter().map(|&(key, month, tithi, printed)| {
            (
                key,
                Some(month),
                tithi.into(),
                Fit::Udaya,
                printed,
                "fitted",
            )
        });
        let adhika = IN_ADHIKA
            .iter()
            .map(|&(key, tithi, printed)| (key, None, tithi.into(), Fit::Udaya, printed, "fitted"));
        let monthly = EVERY_MONTH
            .iter()
            .map(|&(key, occurs, fit, printed)| (key, None, occurs, fit, printed, "fitted"));
        let by_text = BY_TEXT.iter().map(|&(key, month, tithi, fit, read)| {
            (key, Some(month), tithi.into(), fit, read, "the text's rule")
        });
        yearly
            .chain(at_sunrise)
            .chain(adhika)
            .chain(monthly)
            .chain(by_text)
            .map(|(key, month, occurs, fit, printed, how)| {
                let (at, decide, otherwise, reading) = fit.decide();
                FestivalRule {
                    key: key.to_owned(),
                    source: format!("{printed}; {how}: {reading}"),
                    month,
                    convention: Convention::Amanta,
                    occurs,
                    in_adhika: in_adhika(key, month),
                    at,
                    decide,
                    otherwise,
                    parana: Vec::new(),
                    meta: super::FestivalMeta::default(),
                }
            })
            .collect()
    }
}

impl FollowingRule {
    /// The days Nepal's national panchanga prints on another rule's day, or
    /// the day after it, beyond [`FollowingRule::nepal`]: each counts from a
    /// rule of [`FestivalRule::nepal`] or [`FestivalRule::nepal_panchanga`].
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
        let same_day = SAME_DAY
            .iter()
            .map(|&(key, after, source)| (key, after, 0, source));
        let day_after = DAY_AFTER
            .iter()
            .map(|&(key, after, source)| (key, after, 1, source));
        same_day
            .chain(day_after)
            .map(|(key, after, days, source)| FollowingRule {
                key: key.to_owned(),
                source: source.to_owned(),
                after: after.to_owned(),
                days,
                meta: super::FestivalMeta::default(),
            })
            .collect()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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

    #[test]
    fn the_earlier_alone_is_the_text_s_decision() {
        // Dharmasindhu p. 74: the earlier day only when nishitha holds the
        // full moon there alone; both days, or neither, give the later.
        let (at, decide, otherwise, reading) = Fit::EarlierAlone(Window::Nishitha).decide();
        assert_eq!(
            (at, decide, otherwise),
            (
                Window::Nishitha,
                vec![Guard::new([case(Case::EarlierOnly)], Choice::Earlier)],
                Choice::Later
            )
        );
        assert!(reading.starts_with("nishitha"));
        let kojagrat = FestivalRule::nepal_panchanga()
            .into_iter()
            .find(|rule| rule.key == "KOJAGRAT_PURNIMA")
            .unwrap();
        assert!(kojagrat.source.contains("Dharmasindhu p. 74"));
        assert!(kojagrat.source.contains("the text's rule: nishitha"));
        assert!(!kojagrat.source.contains("fitted"));
    }
}
