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
//! Some rules are not fitted but read (`BY_TEXT`): *Dharmasindhu* gives
//! Kojagara and Nrisimha jayanti that last decision, at nishitha and at
//! sunset, and the print agrees with it in both editions; the
//! *Nirnayasindhu* gives the Mahalakshmi vrata's two days and the Vat
//! Savitri fast's, and Indra's pole is lowered on the dark 4th (C1470).
//!
//! *Dharmasindhu* p. 56 bars the Durva fast once Agastya has risen, and
//! keeps it on the dark 8th before ([`Barred`], C608); the Gaura parva's
//! 5th and 7th move with it, as the committee prints (C686).
//!
//! Most of the days are the plain udaya rule (`AT_SUNRISE`): the
//! festivals of a patro printed in Nepal that its own sunrise rule
//! reproduces, each gated by the committee's print of both years.
//!
//! The pradosha fast and the weekday fasts are kept in every month, the
//! adhika included, on a day of either paksha (§3.4) or the new moon.

use teistro_core::catalogue::{Calendar, Masa, Nakshatra, Paksha, Tithi, Vara};
use teistro_core::settings::LunarMonth as Convention;

use super::{
    APARAHNA, Barred, Case, Choice, DayPart, Edge, EkadashiDecision, FestivalRule, FollowingRule,
    Guard, InAdhika, MADHYAHNA, Occurs, Predicate, Which, Window, by_share, case,
};

/// Purvahna: the daylight's first six muhurtas, its first two fifths
/// (C602).
const PURVAHNA: Window = Window::DayMuhurta { from: 1, to: 6 };

/// The last fifth of the daylight.
const SAYAHNA: Window = Window::Part {
    part: DayPart::Sayahna,
};

/// How a fitted rule decides between its two days; the Mithila and West
/// Bengal packs read their rules through the same fits.
#[derive(Clone, Copy)]
pub(super) enum Fit {
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
    /// The later day only when it alone holds the tithi at a window, the
    /// earlier otherwise: Kali Puja's new moon at nishitha in Bengal.
    LaterAlone(Window),
    /// The earlier day whenever the tithi touches a window of it at all,
    /// wholly or in part, alone or with the later; the later otherwise:
    /// Gathamuga charhe's evening rite, kept on the first pradosha the
    /// fourteenth reaches (rc.8 audit, a community rule).
    EarlierAny(Window),
    /// The earlier day when both hold a window wholly, or neither does;
    /// the later when it holds it wholly or alone while the earlier holds
    /// it in part at most: *Dharmasindhu* p. 55's chaturthi, "the later
    /// when the earlier holds madhyahna in part and the later wholly", and
    /// so for the other months' (“एवं मासान्तरेऽपि”).
    LaterWhollyOrAlone(Window),
    /// The earlier day when the tithi holds a window of it wholly, the
    /// later otherwise: the Vat Savitri fast's three muhurtas before
    /// sunset (*Dharmasindhu* p. 41).
    EarlierWholly(Window),
    /// At moonrise: the later day when the tithi lasts this many ghatis
    /// past its moonrise, the earlier otherwise: the Mahalakshmi vrata's
    /// close (*Nirnayasindhu* pp. 300–301).
    LaterPastMoonrise(u8),
    /// At sunrise: the later day when the tithi lasts this many ghatis
    /// past its sunrise, the earlier otherwise: the Bali pratipada's
    /// cows' worship (*Dharmasindhu* p. 78).
    LaterPastSunrise(u8),
    /// The fortnight's Ekadashi, decided as Nepal's calendars print it
    /// ([`EkadashiDecision::NEPAL_SMARTA`]) rather than by a window; only
    /// [`Fit::rule`] carries the decision, so a rule takes it from there.
    Ekadashi,
}

impl Fit {
    /// The window, the guards and the fallback the fit stands for, and
    /// the words its source ends with.
    pub(super) fn decide(self) -> (Window, Vec<Guard>, Choice, String) {
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
                Fit::at(window, decide, otherwise, "the day holding more of it, in time; the earlier on a tie, both holding it wholly being one, and when neither holds it (C601, C603)")
            }
            Fit::Crescent => Fit::crescent(),
            Fit::EarlierWholly(window) => Fit::at(
                window,
                vec![Guard::new([Predicate::Wholly { day: Which::Earlier }], Earlier)],
                Later,
                "the earlier day when the tithi holds it wholly, the later otherwise",
            ),
            Fit::LaterPastMoonrise(ghatis) => Fit::later_past(Edge::Moonrise, ghatis),
            Fit::LaterPastSunrise(ghatis) => Fit::later_past(Edge::Sunrise, ghatis),
            Fit::EarlierAlone(window) => Fit::at(
                window,
                vec![Guard::new([case(Case::EarlierOnly)], Earlier)],
                Later,
                "the earlier day only when it alone holds the tithi there, the later otherwise",
            ),
            Fit::EarlierHolds(window) => Fit::at(
                window,
                vec![
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new([case(Case::Both)], Earlier),
                ],
                Later,
                "the earlier day when it holds the tithi there, alone or with the later, the later otherwise",
            ),
            Fit::LaterAlone(window) => Fit::at(
                window,
                vec![Guard::new([case(Case::LaterOnly)], Later)],
                Earlier,
                "the later day only when it alone holds the tithi there, the earlier otherwise",
            ),
            Fit::EarlierAny(window) => Fit::at(
                window,
                vec![
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new([case(Case::Both)], Earlier),
                    Guard::new([case(Case::EqualParts)], Earlier),
                    Guard::new([case(Case::UnequalParts)], Earlier),
                ],
                Later,
                "the earlier day whenever the tithi touches it there, the later otherwise",
            ),
            Fit::LaterWhollyOrAlone(window) => Fit::at(
                window,
                vec![
                    Guard::new([case(Case::Both)], Earlier),
                    Guard::new([Predicate::Wholly { day: Which::Later }], Later),
                    Guard::new([case(Case::LaterOnly)], Later),
                ],
                Earlier,
                "the later day when it holds the tithi there wholly, or alone, and the earlier only in part; the earlier otherwise, both holding it wholly being the earlier",
            ),
            Fit::OnVara(vara) => Fit::on_vara(vara),
            Fit::Ekadashi => Fit::ekadashi(),
        }
    }

    /// A fit on one window: its guards and fallback, and its words after
    /// the window's name.
    fn at(
        window: Window,
        guards: Vec<Guard>,
        otherwise: Choice,
        words: &str,
    ) -> (Window, Vec<Guard>, Choice, String) {
        (
            window,
            guards,
            otherwise,
            format!("{}: {words}", Fit::name(window)),
        )
    }

    /// [`Fit::LaterPastMoonrise`]'s and [`Fit::LaterPastSunrise`]'s
    /// window, guards, fallback and words: the later day when the tithi
    /// lasts `ghatis` past `from` on it, the earlier otherwise.
    fn later_past(from: Edge, ghatis: u8) -> (Window, Vec<Guard>, Choice, String) {
        let (window, edge) = match from {
            Edge::Sunrise => (Window::Sunrise, "sunrise"),
            Edge::Sunset => (Window::Sunset, "sunset"),
            Edge::Moonrise => (Window::Moonrise, "moonrise"),
        };
        (
            window,
            vec![Guard::new(
                [Predicate::Lasts {
                    day: Which::Later,
                    from,
                    ghatis,
                }],
                Choice::Later,
            )],
            Choice::Earlier,
            format!(
                "{edge}: the later day when the tithi lasts {ghatis} ghatis past its {edge}, the earlier otherwise"
            ),
        )
    }

    /// [`Fit::Crescent`]'s window, guards, fallback and words.
    fn crescent() -> (Window, Vec<Guard>, Choice, String) {
        let (at, decide, otherwise, _) =
            Fit::EarlierWholly(Window::DayMuhurta { from: 15, to: 15 }).decide();
        (
            at,
            decide,
            otherwise,
            "the earlier day when the tithi holds its last muhurta before sunset wholly, the later otherwise (C684)".to_owned(),
        )
    }

    /// [`Fit::Ekadashi`]'s window, guards, fallback and words: the window
    /// gives only the extents an observance shows, and the decision's
    /// table reads no guard (`FestivalRule::ekadashi`).
    fn ekadashi() -> (Window, Vec<Guard>, Choice, String) {
        (
            Window::Sunrise,
            Vec::new(),
            Choice::Earlier,
            "the Smartas' Ekadashi as Nepal's calendars print it: Dharmasindhu pp. 11–12's householder, pierced at sunrise, but the pierced 11th alone in excess (S2) is kept on the later day, which the print keeps in all 7 such years of VS 2076 to 2083 (published-only, C2322)".to_owned(),
        )
    }

    /// [`Fit::OnVara`]'s window, guards, fallback and words.
    fn on_vara(vara: Vara) -> (Window, Vec<Guard>, Choice, String) {
        use Choice::{Earlier, Later};
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

    /// The amanta rule on `occurs` in `month` (every month when `None`)
    /// that this fit decides, its source `cited` and then the fit's words.
    pub(super) fn rule(
        self,
        key: &str,
        month: Option<Masa>,
        occurs: Occurs,
        in_adhika: InAdhika,
        cited: &str,
    ) -> FestivalRule {
        let (at, decide, otherwise, reading) = self.decide();
        FestivalRule {
            in_month: None,
            barred: None,
            key: key.to_owned(),
            source: format!("{cited}: {reading}"),
            month,
            convention: Convention::Amanta,
            occurs,
            in_adhika,
            at,
            decide,
            otherwise,
            parana: Vec::new(),
            ekadashi: matches!(self, Fit::Ekadashi).then_some(EkadashiDecision::NEPAL_SMARTA),
            meta: super::FestivalMeta::default(),
        }
    }

    /// The window's name in a source.
    fn name(window: Window) -> &'static str {
        match window {
            Window::DayMuhurta { from: 1, to: 6 } => {
                "purvahna, the daylight's first six muhurtas (C602)"
            }
            Window::DayMuhurta { from: 13, to: 15 } => {
                "the daylight's last three muhurtas, before sunset"
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
const RULES: [(&str, Masa, Tithi, Fit, &str); 70] = [
    (
        "CHAITE_DASHAIN",
        Masa::Chaitra,
        Tithi::ShuklaAshtami,
        Fit::UdayaLater,
        "Nepal's national panchanga, VS 2082 p. 25: चैतेदशैं; the 8th joined to the 9th (Nirnayasindhu p. 154) (rc.8 audit)",
    ),
    (
        "PARASHURAMA_JAYANTI",
        Masa::Vaishakha,
        Tithi::ShuklaTritiya,
        Fit::EarlierAlone(Window::Pradosha),
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: परशुरामजयन्ती; the night's first part, the earlier only when it alone holds it (Dharmasindhu p. 37; Nirnayasindhu p. 180) (rc.8 audit)",
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
        Fit::LaterWhollyOrAlone(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 3 and VS 2083 p. 3: गंगासप्तमी; madhyahna, the earlier when both, neither or each in part hold it (Nirnayasindhu p. 180; Dharmasindhu p. 37) (rc.8 audit)",
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
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 12: दूर्वाष्टमीव्रत; the third of three tithis a source gives (C620); barred once Agastya has risen, Dharmasindhu p. 56 and its note, \"पूर्वकृष्णाष्टम्यामेव कुर्यात्\", Agastya rising on the 21st day after the Sun enters Simha (p. 58), Bhadra 22 of the Bikram Sambat, so VS 2083 prints Shravana's dark 8th (C608)",
    ),
    (
        "CHATHA",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturthi,
        Fit::EarlierHolds(SAYAHNA),
        "Nepal's national panchanga, VS 2082 p. 11: चन्द्रदोषदर्शन (चथा); VS 2083 p. 13: चन्द्रदर्शनदोष (चथा); the Newar name of the 4th's night; a 4th begun by the sayahna puts the moon's fault on the earlier evening (Dharmasindhu p. 55; Nirnayasindhu p. 259) (rc.8 audit)",
    ),
    (
        "GANESHA_CHATURTHI",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturthi,
        Fit::LaterWhollyOrAlone(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: गणेशचतुर्थीव्रत; madhyahna, the later when it holds it wholly and the earlier in part (Dharmasindhu p. 55) (rc.8 audit)",
    ),
    (
        "RISHI_PANCHAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaPanchami,
        Fit::LaterWhollyOrAlone(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: ऋषिपञ्चमीव्रत; madhyahna, the earlier when both or neither hold it (Dharmasindhu p. 55) (rc.8 audit)",
    ),
    (
        "GAURA_SAPTAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaSaptami,
        Fit::LaterAlone(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 12: गौरासप्तमी; barred with DURVA_ASHTAMI, VS 2083 printing Shravana's dark 7th, inside the solar Bhadra (C608, C686); the 7th joined to the earlier, its puja at madhyahna (Nirnayasindhu pp. 260, 262) (rc.8 audit)",
    ),
    (
        "BIRUDA_PANCHAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaPanchami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 12: बिरुडापञ्चमी, the Gaura parva's first day; barred with DURVA_ASHTAMI, VS 2083 printing Shravana's dark 5th, the 1st of September (C686)",
    ),
    (
        "HILE_JATRA",
        Masa::Shravana,
        Tithi::KrishnaNavami,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: रामेछाप चिसापानी हिलेजात्रा; Shravana's dark 9th, the day the committee prints (C609) (rc.8 audit)",
    ),
    (
        "VAMANA_DWADASHI",
        Masa::Bhadrapada,
        Tithi::ShuklaDwadashi,
        Fit::LaterAlone(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 11: वामनद्वादशी; VS 2083 p. 13: वामनजयन्ती; madhyahna, the day joined to the 11th when both or neither hold it (Dharmasindhu p. 58) (rc.8 audit)",
    ),
    (
        "PRAUSHTHAPADI_SHRADDHA",
        Masa::Bhadrapada,
        Tithi::Purnima,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 11: प्रौष्ठपदीश्राद्ध; VS 2083 p. 13: प्रौष्ठपदी श्राद्ध: the full moon's shraddha, read at aparahna as the fortnight's shraddhas are (C2302)",
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
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: नवमीश्राद्ध, मातृनवमीश्राद्ध; a parvana shraddha, decided at aparahna as the other mahalayas (Dharmasindhu pp. 59, 266) (rc.8 audit)",
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
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: शस्त्रादिबाट निधन भएकाहरूको विश्वेदेवासहित एकोदिष्ट विधिले चतुर्दशीश्राद्ध; an ekoddishta, decided at madhyahna (Dharmasindhu p. 63; Nirnayasindhu p. 310) (rc.8 audit)",
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
        Fit::LaterAlone(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: उपाङ्गललिताव्रत; aparahna, the earlier unless the later alone holds it (Dharmasindhu p. 69) (rc.8 audit)",
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
        "NEPAL_SAMBAT_NEW_YEAR",
        Masa::Kartika,
        Tithi::ShuklaPratipada,
        Fit::Share(MADHYAHNA),
        "Nepal's national panchanga, VS 2082 p. 15: ने.सं.११४६ प्रारम्भ; VS 2083 p. 17: ने.सं.११४७ प्रारम्भ: Kachhala's first day, which the print opens on Mha puja's day, so read as MHA_PUJA is; in a doubled Kartika the year begins on the adhika month's first day, Dharmasindhu p. 31 (चैत्रस्य मलमासत्वे वत्सरारम्भनिमित्तकं ... मलमासप्रतिपद्येव कुर्यात्), as the Nepal Sambat calendar begins it, while Mha puja keeps the nija month (C2041)",
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
        Tithi::Purnima,
        Fit::EarlierAlone(Window::Pradosha),
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: दत्तात्रेयजयन्ती; Margashirsha's full moon at pradosha (Nirnayasindhu p. 424, 'pradoshavyapini') (rc.8 audit)",
    ),
    (
        "PISHACHAMOCHANA_SHRADDHA",
        Masa::Margashirsha,
        Tithi::ShuklaChaturdashi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: काशीमा पिशाचमोचन तीर्थश्राद्ध; a shraddha, kept in aparahna as Dharmasindhu keeps shraddha (the day's fourth fifth, p. 265; the day holding more of it, p. 266), where purvahna gave both years a day late (C602)",
    ),
    (
        "GAIDU_PUJA",
        Masa::Margashirsha,
        Tithi::Purnima,
        Fit::Udaya,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: गैडुपूजा",
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
        Fit::EarlierAlone(Window::Pradosha),
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: तिलकुन्द चौथी; a nakta fast at pradosha (Dharmasindhu pp. 6–7, 89; Nirnayasindhu p. 445) (rc.8 audit)",
    ),
    (
        "CHAMPA_SHASHTHI",
        Masa::Magha,
        Tithi::ShuklaShashthi,
        Fit::Udaya,
        "kept in Nepal, though the national panchanga prints it in neither VS 2082 nor 2083 (C613); the print names this day Skanda shashthi (C612)",
    ),
    (
        "DVAPARA_YUGADI",
        Masa::Magha,
        Tithi::Amavasya,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 22 and VS 2083 p. 24: द्वापरयुगादि; aparahna, kept with the new moon's shraddha (Dharmasindhu p. 94; Nirnayasindhu p. 175) (rc.8 audit)",
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
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 25 and VS 2083 p. 27: मत्स्यनारायणमेला; the fair is Matsya jayanti's day, aparahna (Nirnayasindhu p. 151), as the calendars print the two together (rc.8 audit)",
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
        Fit::LaterAlone(Window::Sunset),
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: कल्कीजयन्ती; a patro printed in Nepal gives the bright 5th, a day before both printed days; at evening, the earlier when the kala is equal (Nirnayasindhu p. 151) (rc.8 audit)",
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
    (
        "MATATI_CHARHE",
        Masa::Chaitra,
        Tithi::KrishnaChaturdashi,
        Fit::Share(SAYAHNA),
        "Nepal's national panchanga, VS 2082 p. 2 and VS 2083 p. 2: माताति चःह्रे पूजा; the printed charhe at sayahna, both editions' days (C614, C1812)",
    ),
    (
        "SITHI_CHARHE",
        Masa::Vaishakha,
        Tithi::KrishnaChaturdashi,
        Fit::Share(APARAHNA),
        "Nepal's national panchanga, VS 2082 p. 4 and VS 2083 p. 4: (सिथिचह्रेपूजा); the printed charhe at aparahna, Dila charhe's window, which gives both editions' days and more of a patro's verified days than sayahna (C614, C1812)",
    ),
    (
        "GATHAMUGA_CHARHE",
        Masa::Ashadha,
        Tithi::KrishnaChaturdashi,
        Fit::EarlierAny(Window::Pradosha),
        "Nepal's national panchanga, VS 2082 p. 8 and VS 2083 p. 10: घण्टाकर्ण चतुर्दशी, (गथां मुगः चःह्रे पूजा); the printed charhe at pradosha, Pahan charhe's window, which gives both editions' days and more of a patro's verified days than sayahna (C614, C1812); a community rule: the rite on the first pradosha the 14th touches, which every calendar of VS 2076–2083 keeps but one cal-C sunrise year (rc.8 audit)",
    ),
    (
        "JUGA_CHARHE",
        Masa::Shravana,
        Tithi::KrishnaChaturdashi,
        Fit::Share(SAYAHNA),
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: (जुगः चःह्रे पूजा); the printed charhe at sayahna, both editions' days (C614, C1812)",
    ),
    (
        "NALASWANE_CHARHE",
        Masa::Bhadrapada,
        Tithi::KrishnaChaturdashi,
        Fit::Share(SAYAHNA),
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: नलास्वनेचह्रेपूजा, the dark 14th's row continued after Ⓑ; the printed charhe at sayahna, both editions' days (C614, C1812)",
    ),
    (
        "DISHI_CHARHE",
        Masa::Margashirsha,
        Tithi::KrishnaChaturdashi,
        Fit::Share(SAYAHNA),
        "Nepal's national panchanga, VS 2082 p. 18 and VS 2083 p. 20: (दिशीचह्रेपूजा); the printed charhe at sayahna, both editions' days (C614, C1812)",
    ),
];

/// The rules a text gives, which the committee's print follows in both
/// editions: key, month (amanta, the nija month), tithi, the decision,
/// and the text with the page it was read from.
const BY_TEXT: [(&str, Masa, Tithi, Fit, &str); 9] = [
    (
        "GAI_PUJA",
        Masa::Kartika,
        Tithi::ShuklaPratipada,
        Fit::LaterPastSunrise(18),
        "Dharmasindhu p. 78: the Bali puja and the cows' play (गोक्रीडा) on the later day when Kartika's bright 1st holds nine muhurtas past its sunrise, since the Moon is not seen then, and less, on the 1st the new moon pierces, nine muhurtas being 18 ghatis; Nepal's national panchanga, VS 2082 p. 15: गाईपूजा on the 1st's later day, VS 2083 p. 16: गाईपूजा on the new moon's day the 1st begins on, and VS 2076 p. 14: गाईगोरुपूजा on the earlier day (C2301)",
    ),
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
    (
        "VAT_SAVITRI_VRAT",
        Masa::Vaishakha,
        Tithi::Amavasya,
        Fit::EarlierWholly(Window::DayMuhurta { from: 13, to: 15 }),
        "Nirnayasindhu p. 191: \"अमायां च तथा ज्येष्ठे\", the purnimanta Jyeshtha's new moon, and p. 193: \"अयं निर्णयोऽमायामपि ज्ञेयः\", the full-moon form's decision, which Dharmasindhu p. 41 gives as \"अस्तमयात्पूर्वं त्रिमुहूर्ताधिकव्यापिनी चतुर्दशीविद्धा ग्राह्या, त्रिमुहूर्तन्यूनत्वे परैव\", the fast being the one Nirnayasindhu p. 94 keeps on a day joined to the tithi before; Nepal's national panchanga, VS 2082 p. 4 and VS 2083 p. 4: वटसावित्रीव्रत",
    ),
    (
        "MAHALAXMI_VRAT_BEGINS",
        Masa::Bhadrapada,
        Tithi::ShuklaAshtami,
        Fit::LaterAlone(Window::Nishitha),
        "Nirnayasindhu p. 301: \"अर्धरात्रमतिक्रम्य वर्तते योत्तरा तिथिः। तदा तस्यां तिथौ कार्यं महालक्ष्मीव्रतं\", the 8th running past midnight; Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: महालक्ष्मीव्रतारम्भ",
    ),
    (
        "MAHALAXMI_VRAT_ENDS",
        Masa::Bhadrapada,
        Tithi::KrishnaAshtami,
        Fit::LaterPastMoonrise(6),
        "Nirnayasindhu p. 300: \"इयं चन्द्रोदयव्यापिनी ग्राह्या\" and \"परदिने चन्द्रोदयादूर्ध्वं त्रिमुहूर्तव्यापित्वे परैव कार्या, अन्यथा पूर्वैव\", three muhurtas being six ghatis; Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: महालक्ष्मीव्रत समाप्ति",
    ),
    (
        "INDRADHWAJA_PATAN",
        Masa::Bhadrapada,
        Tithi::KrishnaChaturthi,
        Fit::Udaya,
        "Toffin 1992, Contributions to Nepalese Studies p. 73, and Lewis 1984, The Tuladhars of Kathmandu p. 377: Indra's pole lowered on Yanla's dark 4th, read at sunrise, as a rite of the daylight is (C197); Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: इन्द्रध्वजपातन (नानिचायाः), printed in VS 2083 on the day before, whose sunrise holds the 3rd (C1470)",
    ),
];

/// The days the committee prints on the udaya rule, each read off both
/// editions: key, month (amanta, the nija month), tithi, and where the day
/// is printed. Each also gives every day a patro printed in Nepal verified
/// for it from 2013 to 2027, by a back-test kept outside the repository
/// (`festival-classification.md`).
const AT_SUNRISE: [(&str, Masa, Tithi, &str); 146] = [
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
];

/// The named ekadashis, each the fortnight's Ekadashi decided as Nepal's
/// calendars print it ([`Fit::Ekadashi`]): key, month (amanta, the nija
/// month), tithi, and where the day is printed. Over VS 2076 to 2083 the
/// table gives the printed day in 191 of the 192 Smarta ekadashis the
/// calendars agree on (`festival-rules.md` §10.7, C2322).
const EKADASHIS: [(&str, Masa, Tithi, &str); 24] = [
    (
        "NIRJALA_EKADASHI",
        Masa::Jyeshtha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 5 and VS 2083 p. 7: निर्जला एकादशीव्रत",
    ),
    (
        "AJA_EKADASHI",
        Masa::Shravana,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 10 and VS 2083 p. 12: अजाएकादशीव्रत",
    ),
    (
        "HARIBODHINI_EKADASHI",
        Masa::Kartika,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: हरिबोधिनीएकादशीव्रत",
    ),
    (
        "PAUSHA_PUTRADA_EKADASHI",
        Masa::Pausha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 19: पुत्रदाएकादशीव्रत; VS 2083 p. 21: स्मार्तहरुको पुत्रदाएकादशीव्रत",
    ),
    (
        "BHIMA_EKADASHI",
        Masa::Magha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 21 and VS 2083 p. 23: भीमा एकादशीव्रत",
    ),
    (
        "VIJAYA_EKADASHI",
        Masa::Magha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 22 and VS 2083 p. 24: विजया एकादशीव्रत; the 11th pierced at sunrise with it alone in excess, printed on the later day (C610, C2322)",
    ),
    (
        "VARUTHINI_EKADASHI",
        Masa::Chaitra,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 2: बरुथिनीएकादशीव्रत; VS 2082 p. 26: बरुथिनी एकादशीव्रत; the Smartas' day, as the print names it where the Vaishnavas' differs (C688)",
    ),
    (
        "MOHINI_EKADASHI",
        Masa::Vaishakha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 3: मोहिनी एकादशीव्रत; VS 2083 p. 3: मोहिनीएकादशीव्रत",
    ),
    (
        "APARA_EKADASHI",
        Masa::Vaishakha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 4 and VS 2083 p. 4: अपराएकादशीव्रत",
    ),
    (
        "YOGINI_EKADASHI",
        Masa::Jyeshtha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 6: स्मार्तहरूको योगिनी एकादशीव्रत; VS 2083 p. 8: स्मार्तहरुको योगिनी एकादशीव्रत; the Smartas' day, the Vaishnavas' printed the day after (C688)",
    ),
    (
        "HARISHAYANI_EKADASHI",
        Masa::Ashadha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 7: हरिशयनीएकादशीव्रत; VS 2083 p. 9: हरिशयनी एकादशीव्रत",
    ),
    (
        "KAMIKA_EKADASHI",
        Masa::Ashadha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 8 and VS 2083 p. 10: कामिकाएकादशीव्रत",
    ),
    (
        "SHRAWAN_PUTRADA_EKADASHI",
        Masa::Shravana,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 9 and VS 2083 p. 11: पुत्रदाएकादशीव्रत",
    ),
    (
        "HARIPARIVARTINI_EKADASHI",
        Masa::Bhadrapada,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 13: हरिपरिवर्तिनी एकादशीव्रत",
    ),
    (
        "INDIRA_EKADASHI",
        Masa::Bhadrapada,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: इन्दिराएकादशीव्रत",
    ),
    (
        "PAPANKUSHA_EKADASHI",
        Masa::Ashwina,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: पापाङ्कुशा एकादशीव्रत",
    ),
    (
        "RAMA_EKADASHI",
        Masa::Ashwina,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: रमाएकादशीव्रत",
    ),
    (
        "UTPATTIKA_EKADASHI",
        Masa::Kartika,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 16 and VS 2083 p. 18: उत्पत्तिका एकादशीव्रत",
    ),
    (
        "MOKSHADA_EKADASHI",
        Masa::Margashirsha,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 17 and VS 2083 p. 19: मोक्षदा एकादशीव्रत",
    ),
    (
        "SAFALA_EKADASHI",
        Masa::Margashirsha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 18 and VS 2083 p. 20: सफला एकादशीव्रत",
    ),
    (
        "SHATTILA_EKADASHI",
        Masa::Pausha,
        Tithi::KrishnaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 20 and VS 2083 p. 22: षट्तिला एकादशीव्रत",
    ),
    (
        "AMALAKI_EKADASHI",
        Masa::Phalguna,
        Tithi::ShuklaEkadashi,
        "Nepal's national panchanga, VS 2082 p. 23 and VS 2083 p. 25: आमलकी एकादशीव्रत",
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
/// doubled, each decided as [`EKADASHIS`]' are: key, tithi,
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

/// The rules kept in every month, the adhika included, by their text's
/// decision: key, what is kept, fit, and the text with what the calendars
/// print. Nepal's calendars print each in the years they print it at all.
const EVERY_MONTH_BY_TEXT: [(&str, Occurs, Fit, &str); 3] = [
    (
        "SHUKLA_ASHTAMI_VRATA",
        Occurs::Tithi {
            tithi: Tithi::ShuklaAshtami,
        },
        Fit::UdayaLater,
        "Dharmasindhu p. 10: for a vrata the bright 8th is the later (व्रतमात्रेऽष्टमी शुक्लपक्षे परा); Nepal's calendars print अष्टमी व्रत on it, 20 of 20 bright 8ths of VS 2076–2083 on this rule",
    ),
    (
        "KRISHNA_ASHTAMI_VRATA",
        Occurs::Tithi {
            tithi: Tithi::KrishnaAshtami,
        },
        Fit::Share(Window::Pradosha),
        "Dharmasindhu p. 10: some fast on every dark 8th for Kalabhairava (सर्वकृष्णाष्टमीषु कालभैरवोद्देशेन केचिदुपवसन्ति), the 8th holding the pradosha by the Kaustubha and by the practice of the learned (प्रदोषव्यापिनीति कौस्तुभे); Nepal's calendars print अष्टमी व्रत on it, 22 of 22 dark 8ths of VS 2076–2083 on this rule, and 15 on the earlier day the text gives a vrata otherwise",
    ),
    (
        "DARSHA_SHRADDHA",
        Occurs::Tithi {
            tithi: Tithi::Amavasya,
        },
        Fit::Share(APARAHNA),
        "Dharmasindhu p. 266: a parvana shraddha takes the tithi holding the aparahna, the fourth fifth of the day (पार्वणे त्वपराह्णव्यापिनी ग्राह्या); on both days or neither, or in equal parts, the earlier, and in unequal parts the greater. Nepal's calendars print दर्शश्राद्ध on every new moon of the years they print it, 33 of 34 days of VS 2080, 2082 and 2083 on this rule and 25 on the sunrise's; Nepal's national panchanga, VS 2083 p. 24, prints Monday 8 March 2027, the later day, where the earlier holds 93% of the aparahna and the later 55%",
    ),
];

/// The days kept on a nakshatra in a lunar month: key, month, what is
/// kept, fit, and where the days are printed.
const ON_A_STAR: [(&str, Masa, Occurs, Fit, &str); 1] = [(
    "RAMANUJACHARYA_JAYANTI",
    Masa::Vaishakha,
    Occurs::Nakshatra {
        nakshatra: Nakshatra::Ardra,
        paksha: Paksha::Shukla,
    },
    Fit::Udaya,
    "Nepal's calendars print श्री रामानुजाचार्य जयन्ती on Ardra in Vaishakha's bright half, every year of VS 2076 to 2083 they print it, 2021's on 16 May where Tamil Chittirai's Ardra was 18 April (C2330); the south keeps Thiruvathirai in Chittirai (`INDIA_KARNATAKA`, C1323)",
)];

/// The days printed on another rule's day: key, the rule it keeps the
/// day of, and where.
const SAME_DAY: [(&str, &str, &str); 26] = [
    (
        "KALARATRI",
        "MAHA_ASHTAMI",
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: कालरात्रि, in the Mahashtami's row: the day of MAHA_ASHTAMI (C2304)",
    ),
    (
        "AKASHDEEPADAN_ARAMBHA",
        "KARTIK_SNAN_BEGINS",
        "Nepal's national panchanga, VS 2082 p. 13 and VS 2083 p. 15: आकाशदीपदानारम्भ, in the row of कार्तिकस्नान आरम्भ: the day of KARTIK_SNAN_BEGINS (C2304)",
    ),
    (
        "YAMADEEPADAN",
        "DHANTERAS",
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: यमदीपदान, in the row of धनत्रयोदशी and धन्तेरस, and VS 2076 p. 14 the same: the day of DHANTERAS (C2304)",
    ),
    (
        "SUKHRATRI",
        "LAKSHMI_PUJA",
        "Nepal's national panchanga, VS 2082 p. 14 and VS 2083 p. 16: सुखरात्रि, in the row of लक्ष्मीपूजा, and VS 2076 p. 14 the same: the day of LAKSHMI_PUJA (C2304)",
    ),
    (
        "GORU_PUJA",
        "MHA_PUJA",
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: गोरुपूजा, in the row of म्हपूजा, and VS 2076 p. 14: (म्हपूजा), गाईगोरुपूजा, one row: the day of MHA_PUJA (C2301)",
    ),
    (
        "KIJA_PUJA",
        "YAMA_DWITIYA",
        "Nepal's national panchanga, VS 2082 p. 15 and VS 2083 p. 17: (किजापूजा), the Newar name, in the row of यमद्वितीया and भाइटीका, and VS 2076 p. 15 the same: the day of YAMA_DWITIYA (C2304)",
    ),
    (
        "SORHA_SHRADDHA_SAMAPTI",
        "AMAVASYA_SHRADDHA",
        "Nepal's national panchanga, VS 2083 p. 14: सोह्रश्राद्ध समाप्ति, in the row of औंसीश्राद्ध; VS 2082 p. 12 names no close: the day of AMAVASYA_SHRADDHA (C2304)",
    ),
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
        "GAURA_PARVA",
        "DURVA_ASHTAMI",
        "Government of Nepal, Ministry of Home Affairs, public holidays of 2082 and 2083: Gaura Parva, for the far-western province, on 2082/05/15 and 2083/05/19; Nepal's national panchanga, VS 2082 p. 11 and VS 2083 p. 12: गौरापर्व: the day of DURVA_ASHTAMI, the Gaura parva's 8th (C686)",
    ),
    (
        "NANICHAYA",
        "INDRADHWAJA_PATAN",
        "Nepal's national panchanga, VS 2082 p. 12 and VS 2083 p. 14: इन्द्रध्वजपातन (नानिचायाः): the day of INDRADHWAJA_PATAN, the Newar name the print gives the lowering's day",
    ),
    (
        "NEPALI_ARMY_DAY",
        "SHIVARATRI",
        "the Nepali Army keeps Army Day on Maha Shivaratri, as Spotlight Nepal (2020-02-21) and the Kathmandu Post (2016-03-08) report and the national panchanga prints in neither VS 2082 nor 2083 (C613, C685): the day of SHIVARATRI",
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
        "GHANTAKARNA_CHATURDASHI",
        "GATHAMUGA_CHARHE",
        "Nepal's national panchanga, VS 2082 p. 8 and VS 2083 p. 10: घण्टाकर्ण चतुर्दशी, in one row with (गथां मुगः चःह्रे पूजा): the day of GATHAMUGA_CHARHE (C614, C1812)",
    ),
    (
        "SHILA_CHARHE",
        "SHIVARATRI",
        "Nepal's national panchanga, VS 2082 p. 22 and VS 2083 p. 24: (शिलाचःह्रेपूजा): the day of SHIVARATRI; the Newar name of Shivaratri's night (C614)",
    ),
    (
        "JYAPU_DIWAS",
        "YOMARI_PUNHI",
        "the Jyapu Samaj keeps Jyapu Diwas on Yomari punhi, Margashirsha's full moon, as the Kathmandu Post (2022-12-08, 2023-12-26) and Radio Nepal (2025-12-04) report and the national panchanga prints in neither VS 2082 nor 2083 (C613, C685): the day of YOMARI_PUNHI",
    ),
    (
        "ARJUNDHARA_MELA",
        "PASHUPATI_HALESI_MELA",
        "Nepal's national panchanga, VS 2082 p. 16: सताक्षीधाम तथा अर्जुनधारा मेला; VS 2083 p. 18: झापा सताक्षीधाम तथा अर्जुनधारामेला, each continuing the row of हलेसीमहादेवमेला: the day of PASHUPATI_HALESI_MELA; a patro printed in Nepal gives Bala chaturdashi's day, the day after, which VS 2083 does not print",
    ),
];

/// The days counted from another rule's day: key, the rule, the days
/// after it (before it when negative), and where.
const COUNTED: [(&str, &str, i8, &str); 4] = [
    (
        "KOKA_KAUSHIKI_SANGAM_MELA",
        "DHANYA_PURNIMA",
        1,
        "Nepal's national panchanga, VS 2082 p. 18: कोशिकोका सङ्गममा मेला; VS 2083 p. 20: कोशीकोका सङ्गममा मेला: the day after DHANYA_PURNIMA, as a patro printed in Nepal gives it on all 13 of its verified days from 2013 to 2025 (C690)",
    ),
    (
        "CHHATH_NAHAY_KHAY",
        "CHHATH",
        -2,
        "Chhath's first day, the bath and the one meal (नहाय खाय), Kartika's bright 4th, two days before the evening arghya: the four days' order as Bihar keeps it, reported by Organiser (2024-11-08) and kept by every almanac read; the national panchanga prints no day before छठपर्व in VS 2082 or 2083 (C2306)",
    ),
    (
        "CHHATH_KHARNA",
        "CHHATH",
        -1,
        "Bihar's notification, Schedule-2: Kharna, the day before Chhath's evening arghya in every year listed; the national panchanga prints no day before छठपर्व in VS 2082 or 2083 (C2106, C2306)",
    ),
    (
        "CHHATH_SECOND_DAY",
        "CHHATH",
        1,
        "Bihar's notification, Schedule-2: Chhath's second day, the morning arghya, the day after the evening's in every year listed; the national panchanga prints no day after छठपर्व in VS 2082 or 2083 (C2106, C2306)",
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
        // The rules a doubled Jyeshtha keeps in its adhika month (C606),
        // the year a doubled Kartika begins there (C2041), and the one the
        // print names only there (C607); a monthly rule is kept in an
        // adhika month too.
        fn in_adhika(key: &str, month: Option<Masa>) -> InAdhika {
            match key {
                "DASHAHARA_SNANA_ARAMBHA"
                | "GANGA_DASHAHARA"
                | "GANGA_PUJA"
                | "RAMESHWAR_PUJA"
                | "GURU_PADMASAMBHAVA_JAYANTI"
                | "NEPAL_SAMBAT_NEW_YEAR" => InAdhika::First,
                "ANALA_PUNHI" | "PADMINI_EKADASHI" | "PARAMA_EKADASHI" => InAdhika::Only,
                _ if month.is_none() => InAdhika::Also,
                _ => InAdhika::Nija,
            }
        }
        // The Durva fast barred once Agastya has risen (C608), and the
        // Gaura parva's 5th and 7th with it (C686).
        fn barred(key: &str) -> Option<Barred> {
            let with = |instead| Barred::With {
                rule: "DURVA_ASHTAMI".to_owned(),
                instead,
            };
            match key {
                "DURVA_ASHTAMI" => Some(Barred::From {
                    calendar: Calendar::BikramSambat,
                    month: 5,
                    day: 22,
                    instead: Tithi::KrishnaAshtami,
                }),
                "GAURA_SAPTAMI" => Some(with(Tithi::KrishnaSaptami)),
                "BIRUDA_PANCHAMI" => Some(with(Tithi::KrishnaPanchami)),
                _ => None,
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
        let ekadashis = EKADASHIS.iter().map(|&(key, month, tithi, printed)| {
            (
                key,
                Some(month),
                tithi.into(),
                Fit::Ekadashi,
                printed,
                "the print's table",
            )
        });
        let adhika = IN_ADHIKA.iter().map(|&(key, tithi, printed)| {
            (
                key,
                None,
                tithi.into(),
                Fit::Ekadashi,
                printed,
                "the print's table",
            )
        });
        let monthly = EVERY_MONTH
            .iter()
            .map(|&(key, occurs, fit, printed)| (key, None, occurs, fit, printed, "fitted"));
        let monthly_by_text = EVERY_MONTH_BY_TEXT
            .iter()
            .map(|&(key, occurs, fit, read)| (key, None, occurs, fit, read, "the text's rule"));
        let by_text = BY_TEXT.iter().map(|&(key, month, tithi, fit, read)| {
            (key, Some(month), tithi.into(), fit, read, "the text's rule")
        });
        let on_a_star = ON_A_STAR.iter().map(|&(key, month, occurs, fit, printed)| {
            (key, Some(month), occurs, fit, printed, "fitted")
        });
        yearly
            .chain(at_sunrise)
            .chain(ekadashis)
            .chain(adhika)
            .chain(monthly)
            .chain(monthly_by_text)
            .chain(by_text)
            .chain(on_a_star)
            .map(|(key, month, occurs, fit, printed, how)| FestivalRule {
                barred: barred(key),
                ..fit.rule(
                    key,
                    month,
                    occurs,
                    in_adhika(key, month),
                    &format!("{printed}; {how}"),
                )
            })
            .collect()
    }
}

impl FollowingRule {
    /// The days Nepal's national panchanga prints on another rule's day, or
    /// a count of days from it, beyond [`FollowingRule::nepal`]: each counts
    /// from a rule of [`FestivalRule::nepal`] or
    /// [`FestivalRule::nepal_panchanga`]. Chhath's other three days are
    /// counted from its evening arghya (C2306).
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::nepal_panchanga();
    /// let yugadi = rules.iter().find(|rule| rule.key == "KALI_YUGADI").ok_or("shipped")?;
    /// assert_eq!((yugadi.after.as_str(), yugadi.days), ("TRAYODASHI_SHRADDHA", 0));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn nepal_panchanga() -> Vec<FollowingRule> {
        let same_day = SAME_DAY
            .iter()
            .map(|&(key, after, source)| (key, after, 0, source));
        let counted = COUNTED
            .iter()
            .map(|&(key, after, days, source)| (key, after, days, source));
        same_day
            .chain(counted)
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
    #![allow(clippy::unwrap_used, clippy::panic, reason = "tests fail by panicking")]

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
    fn every_named_ekadashi_and_only_one_is_decided_as_an_ekadashi() {
        let rules = FestivalRule::nepal_panchanga();
        let named: Vec<&str> = rules
            .iter()
            .filter(|rule| rule.ekadashi.is_some())
            .map(|rule| rule.key.as_str())
            .collect();
        assert_eq!(named.len(), EKADASHIS.len() + IN_ADHIKA.len());
        assert!(
            named.iter().all(|key| key.ends_with("_EKADASHI")),
            "{named:?}"
        );
        assert!(
            rules
                .iter()
                .filter(|rule| rule.ekadashi.is_some())
                .all(|rule| rule.ekadashi == Some(EkadashiDecision::NEPAL_SMARTA)
                    && rule.source.contains("C2322"))
        );
        // A decision on another tithi is refused, naming the field.
        let gita = rules
            .iter()
            .find(|rule| rule.key == "GITA_JAYANTI")
            .unwrap();
        assert!(gita.check().is_ok());
        let durva = rules
            .iter()
            .find(|rule| rule.key == "DURVA_ASHTAMI")
            .unwrap();
        let refused = FestivalRule {
            ekadashi: Some(EkadashiDecision::NEPAL_SMARTA),
            ..durva.clone()
        }
        .check()
        .unwrap_err();
        assert_eq!(refused.field(), Some("ekadashi"));
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

    /// The day a fit keeps when the tithi holds these shares of its window
    /// on the earlier and the later day, read as `observances` reads the
    /// guards: the first whose predicates all hold, else the fallback.
    fn kept(fit: Fit, earlier: f64, later: f64) -> Choice {
        let (_, decide, otherwise, _) = fit.decide();
        let is = super::super::case_of(earlier, later);
        let holds = |predicate: &Predicate| match *predicate {
            Predicate::Case { case } => case == is,
            Predicate::Wholly {
                day: Which::Earlier,
            } => earlier >= 1.0,
            Predicate::Wholly { day: Which::Later } => later >= 1.0,
            other => panic!("{other:?} is not read here"),
        };
        decide
            .iter()
            .find(|guard| guard.when.iter().all(holds))
            .map_or(otherwise, |guard| guard.choose)
    }

    #[test]
    fn the_earlier_any_keeps_the_earlier_whenever_it_is_touched() {
        use Choice::{Earlier, Later};
        // Gathamuga: the earlier day whenever pradosha holds the 14th at all.
        let fit = Fit::EarlierAny(Window::Pradosha);
        for (earlier, later, choice) in [
            (0.4, 0.0, Earlier),
            (1.0, 1.0, Earlier),
            (0.5, 0.5, Earlier),
            (0.2, 1.0, Earlier),
            (0.0, 1.0, Later),
            (0.0, 0.0, Later),
        ] {
            assert_eq!(kept(fit, earlier, later), choice, "{earlier}, {later}");
        }
        assert!(fit.decide().3.starts_with("pradosha"));
    }

    #[test]
    fn the_later_wholly_or_alone_is_the_text_s_decision() {
        use Choice::{Earlier, Later};
        // Dharmasindhu p. 55: the later day when it holds madhyahna wholly
        // and the earlier in part, or when it alone holds it; both wholly
        // give the earlier, and so does the earlier holding more of it.
        let fit = Fit::LaterWhollyOrAlone(MADHYAHNA);
        for (earlier, later, choice) in [
            (1.0, 1.0, Earlier),
            (0.3, 1.0, Later),
            (0.0, 0.6, Later),
            (1.0, 0.6, Earlier),
            (0.6, 0.3, Earlier),
            (0.4, 0.4, Earlier),
            (0.7, 0.0, Earlier),
            (0.0, 0.0, Earlier),
        ] {
            assert_eq!(kept(fit, earlier, later), choice, "{earlier}, {later}");
        }
        let ganesha = FestivalRule::nepal_panchanga()
            .into_iter()
            .find(|rule| rule.key == "GANESHA_CHATURTHI")
            .unwrap();
        assert!(ganesha.source.contains("p. 55"), "{}", ganesha.source);
    }
}
