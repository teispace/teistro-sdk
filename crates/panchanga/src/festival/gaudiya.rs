//! The Gaudiya Vaishnavas' festivals (`03-design/festival-rules.md`
//! §8.10, SR-17): Janmashtami, Rama Navami and Nrisimha Chaturdashi as
//! *Hari Bhakti Vilasa* keeps them, and Radhashtami and Gaura Purnima as a
//! published Gaudiya calendar does.
//!
//! The Vaishnava's vedha is the Smarta's turned round. A tithi touched at
//! sunrise by the one before it is *pierced* (*viddha*) and is left, as a
//! pierced Ekadashi is (HBV 15.358, 12.352's commentary naming Rama
//! Navami and Nrisimha Chaturdashi with it). For every tithi but the 11th
//! the touch that pierces is at sunrise, not at arunodaya: the commentary
//! on 15.361 refutes those who carry the Ekadashi's arunodaya over to
//! Janmashtami (C1120). So each rule reads the tithi at sunrise: the day
//! whose sunrise holds it is pure, and of two such days the earlier.
//!
//! Each rule cites its verse in Haridas Shastri's edition (vol. 2,
//! Vrindavan 1986, the Internet Archive's `hbv_vol2`), as the Gaudiya
//! Ekadashi does ([`EkadashiRule::hari_bhakti_vilasa`]). Radhashtami and
//! Gaura Purnima are not in the text: their days are a convention, the
//! Vaishnava vedha carried to them as the published calendar keeps them
//! (C1123).
//!
//! [`EkadashiRule::hari_bhakti_vilasa`]: super::EkadashiRule::hari_bhakti_vilasa

use teistro_core::catalogue::{Masa, Nakshatra, Tithi};

use super::{
    Case, Choice, Convention, FestivalMeta, FestivalRule, Guard, InAdhika, Predicate, VrataParana,
    Which, Window, case, joined,
};

/// The general Vaishnava reading: the day whose sunrise the tithi holds,
/// the earlier of two, and the later when no sunrise holds it, since the
/// earlier is then pierced (HBV 15.358, 15.365; 14.460–461).
fn pure_at_sunrise() -> Vec<Guard> {
    vec![Guard::new(
        [Predicate::Wholly {
            day: Which::Earlier,
        }],
        Choice::Earlier,
    )]
}

/// A Gaudiya rule kept in the nija month, read at sunrise.
fn rule(
    key: &str,
    source: &str,
    (month, tithi): (Masa, Tithi),
    decide: Vec<Guard>,
    otherwise: Choice,
    parana: Vec<VrataParana>,
) -> FestivalRule {
    FestivalRule {
        in_month: None,
        barred: None,
        key: key.to_owned(),
        source: source.to_owned(),
        month: Some(month),
        convention: Convention::Amanta,
        occurs: tithi.into(),
        in_adhika: InAdhika::Nija,
        at: Window::Sunrise,
        decide,
        otherwise,
        parana,
        meta: FestivalMeta::default(),
    }
}

impl FestivalRule {
    /// The `GAUDIYA` pack's festivals, each citing its verse: Janmashtami,
    /// Rama Navami and Nrisimha Chaturdashi after *Hari Bhakti Vilasa*,
    /// then Radhashtami and Gaura Purnima as a published Gaudiya calendar
    /// keeps them (a convention, C1123).
    ///
    /// ```
    /// use teistro_panchanga::festival::{FestivalRule, Window};
    ///
    /// let rules = FestivalRule::hari_bhakti_vilasa();
    /// let janmashtami = rules.iter().find(|rule| rule.key == "JANMASHTAMI_VAISHNAVA").ok_or("shipped")?;
    /// // The Vaishnava's Janmashtami is read at sunrise; the Smarta's, at nishitha.
    /// assert_eq!(janmashtami.at, Window::Sunrise);
    /// assert!(janmashtami.source.starts_with("Hari Bhakti Vilasa 15.358"));
    /// assert!(rules.iter().all(|rule| rule.check().is_ok()));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn hari_bhakti_vilasa() -> Vec<FestivalRule> {
        use Choice::{Earlier, Later};
        use Which::{Earlier as E, Later as L};
        let both = || case(Case::Both);
        let janmashtami = {
            let mut decide = vec![
                Guard::new(
                    [both(), joined(E, Nakshatra::Rohini, Some(Window::Nishitha))],
                    Earlier,
                ),
                Guard::new(
                    [
                        both(),
                        Predicate::Stands {
                            day: L,
                            nakshatra: Nakshatra::Rohini,
                            at: Window::Nishitha,
                        },
                    ],
                    Later,
                ),
            ];
            decide.extend(pure_at_sunrise());
            decide
        };
        vec![
            rule(
                "JANMASHTAMI_VAISHNAVA",
                "Hari Bhakti Vilasa 15.358–374: the 8th pierced by the 7th is left \"as a pierced Ekadashi is\" (15.358), though Rohini join it (15.359–369), and pierced at sunrise, not at arunodaya (15.361's commentary, C1120); pure on both days the earlier is fasted (15.374), unless Rohini, absent at its midnight, stands with the 9th at the later's (15.374's commentary); no sunrise holding it, the 9th (15.365); parana at the end of the 8th and of Rohini, by day when one spans the night (15.385–386, 15.397–402)",
                (Masa::Shravana, Tithi::KrishnaAshtami),
                janmashtami,
                Later,
                vec![VrataParana::TithiEnd {
                    nakshatra: Some(Nakshatra::Rohini),
                    yamas: None,
                }],
            ),
            rule(
                "RAMA_NAVAMI_VAISHNAVA",
                "Hari Bhakti Vilasa 14.249–253: the 9th pierced by the 8th is left by the Vaishnava, the fast on the 9th and the parana on the 10th (14.252); when no sunrise holds the 9th the pierced day is taken, since the parana must fall on the 10th (14.253, C1122)",
                (Masa::Chaitra, Tithi::ShuklaNavami),
                vec![Guard::new([case(Case::LaterOnly)], Later)],
                Earlier,
                vec![VrataParana::NextTithi],
            ),
            rule(
                "NRISIMHA_CHATURDASHI",
                "Hari Bhakti Vilasa 14.455–461: the bright 14th of Vaishakha, fasted and the Lord worshipped at evening (14.457); the 14th pierced by the 13th is not for a Vaishnava (14.460), even joined with Svati or a Tuesday (14.461), so the later day when no sunrise holds it",
                (Masa::Vaishakha, Tithi::ShuklaChaturdashi),
                pure_at_sunrise(),
                Later,
                Vec::new(),
            ),
            rule(
                "RADHASHTAMI_VAISHNAVA",
                "a convention, not a text's rule (C1123): the bright 8th of Bhadrapada on the day a published Gaudiya calendar keeps it, the Vaishnava vedha of Hari Bhakti Vilasa 15.358 and 12.352's commentary carried to it: the day whose sunrise holds the 8th, the earlier of two, else the later",
                (Masa::Bhadrapada, Tithi::ShuklaAshtami),
                pure_at_sunrise(),
                Later,
                Vec::new(),
            ),
            rule(
                "GAURA_PURNIMA",
                "a convention, not a text's rule (C1123): Phalguna's full moon, Chaitanya's birth \"at the evening of Phalguna's full moon\" (Chaitanya Charitamrita Adi 13.89), on the day a published Gaudiya calendar keeps it, the Vaishnava vedha of Hari Bhakti Vilasa 15.358 and 12.352's commentary carried to it: the day whose sunrise holds the full moon, the earlier of two, else the later",
                (Masa::Phalguna, Tithi::Purnima),
                pure_at_sunrise(),
                Later,
                Vec::new(),
            ),
        ]
    }
}
