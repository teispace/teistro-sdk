//! The house and business rites and the marriage by Daivajna Rama's
//! *Muhurta Chintamani* (1600): beginning a house (griharambha), which is
//! the laying of the foundation, shilanyas; the ground's worship before
//! digging (bhumi puja); the house's propitiation (vastu shanti); opening
//! a shop (vipani); the marriage (vivaha); and the thread ceremony
//! (upanayana, Nepal's bratabandha) — each a value of [`ActivityRules`]
//! like Raman's.
//!
//! The verses are read in two printings with Hindi commentaries: Mahidhara's
//! (Bombay, 1905) and Pyarelal Mishra's (1939), with *Muhurta Martanda*
//! (Khemraj, 1904) where it says the same in other words. Each clause is
//! encoded as the verse words it, and cites its chapter and verse.
//!
//! Unlike Raman's rites these carry no general shuddhi under them (crux
//! C1130): *Muhurta Chintamani* names what each rite avoids in its own
//! verse, so a limb the verse leaves out is left middling. A caller who
//! keeps a general shuddhi lays these grades over it with
//! [`Graded::over`].
//!
//! The verses avoid and do not rank: none says a placement **must** not
//! be, so nothing here bars (crux C205), and what a verse avoids weighs
//! against the time. What a verse asks that the SDK does not judge — a
//! door's facing, Rahu's mouth for the digging, the native's birth stars
//! — is listed in `unjudged`. `saait-measured.md` holds the rites to the
//! days Nepal's panchanga committee decided for two years.

use teistro_core::catalogue::{Graha, Masa, Nakshatra, Rashi, Tithi, Vara};

use crate::activity::{ActivityRules, Hours, MonthRule, MonthWithSun, Pada, Unjudged, Unwanted};
use crate::day::DayRules;
use crate::grade::{Grade, Graded};
use crate::instant::MALEFICS;
use crate::rites::{BENEFICS, MOVABLE, benefic_varas, both, bright, dark, good, unjudged};
use crate::season::BlackoutKind;
use crate::tara::ChandraBala;

/// The rikta tithis, the 4th, 9th and 14th of both halves.
fn rikta() -> Vec<Tithi> {
    both(&[4, 9, 14])
}

/// The fixed (dhruva) stars: Rohini and the three Uttaras (ch. II v. 3).
const DHRUVA: [Nakshatra; 4] = [
    Nakshatra::Rohini,
    Nakshatra::UttaraPhalguni,
    Nakshatra::UttaraAshadha,
    Nakshatra::UttaraBhadrapada,
];

/// The soft stars, which the chapter also calls the friendly (mridu,
/// maitra): Mrigashira, Revati, Chitra and Anuradha (ch. II v. 7).
const MRIDU: [Nakshatra; 4] = [
    Nakshatra::Mrigashira,
    Nakshatra::Revati,
    Nakshatra::Chitra,
    Nakshatra::Anuradha,
];

/// The swift stars (kshipra, laghu): Hasta, Ashwini and Pushya, with
/// Abhijit (ch. II v. 6), which is not one of the twenty-seven a day's
/// star is read among.
const KSHIPRA: [Nakshatra; 3] = [Nakshatra::Hasta, Nakshatra::Ashwini, Nakshatra::Pushya];

/// The movable stars (chara): Swati, Punarvasu, Shravana, Dhanishtha and
/// Shatabhisha (ch. II v. 4).
const CHARA: [Nakshatra; 5] = [
    Nakshatra::Swati,
    Nakshatra::Punarvasu,
    Nakshatra::Shravana,
    Nakshatra::Dhanishtha,
    Nakshatra::Shatabhisha,
];

/// The stars of the panchaka, the Moon in Aquarius and Pisces (ch. II
/// v. 48): Dhanishtha's second half, by its quarters, and the four stars
/// after it whole.
const PANCHAKA_STARS: [Nakshatra; 4] = [
    Nakshatra::Shatabhisha,
    Nakshatra::PurvaBhadrapada,
    Nakshatra::UttaraBhadrapada,
    Nakshatra::Revati,
];

/// The quarters of Dhanishtha in the panchaka.
const PANCHAKA_PADAS: [Pada; 2] = [
    Pada {
        nakshatra: Nakshatra::Dhanishtha,
        pada: 3,
    },
    Pada {
        nakshatra: Nakshatra::Dhanishtha,
        pada: 4,
    },
];

/// The lunar months a house is begun in, each while the Sun stands in a
/// sign (ch. XII v. 15): Phalguna with the Sun in Aquarius, Shravana in
/// Leo or Cancer, Pausha in Capricorn, Vaishakha in Taurus or Aries,
/// Margashirsha in Libra or Scorpio.
const HOUSE_MONTHS: [MonthWithSun; 8] = [
    MonthWithSun {
        masa: Masa::Phalguna,
        sun: Rashi::Aquarius,
    },
    MonthWithSun {
        masa: Masa::Shravana,
        sun: Rashi::Leo,
    },
    MonthWithSun {
        masa: Masa::Shravana,
        sun: Rashi::Cancer,
    },
    MonthWithSun {
        masa: Masa::Pausha,
        sun: Rashi::Capricorn,
    },
    MonthWithSun {
        masa: Masa::Vaishakha,
        sun: Rashi::Taurus,
    },
    MonthWithSun {
        masa: Masa::Vaishakha,
        sun: Rashi::Aries,
    },
    MonthWithSun {
        masa: Masa::Margashirsha,
        sun: Rashi::Libra,
    },
    MonthWithSun {
        masa: Masa::Margashirsha,
        sun: Rashi::Scorpio,
    },
];

/// What ch. I vv. 46–47 forbids beginning or consecrating a house in, and
/// a marriage: Jupiter or Venus set, and a month short or added.
const SET_AND_UNEVEN_MONTHS: [BlackoutKind; 4] = [
    BlackoutKind::GuruAsta,
    BlackoutKind::ShukraAsta,
    BlackoutKind::AdhikaMasa,
    BlackoutKind::KshayaMasa,
];

/// The thread ceremony's tithis (ch. V vv. 40, 48 and 54): v. 40's good
/// numbers of the bright half and of the dark half's first third, the
/// galagrahas, the anadhyayas and the dark half past its first third
/// rejected, the bright 6th alone left middling.
fn upanayana_tithis() -> Graded<Tithi> {
    let good_numbers = [2, 3, 5, 10, 11, 12];
    let mut good_tithis: Vec<Tithi> = good_numbers.iter().filter_map(|n| bright(*n)).collect();
    good_tithis.extend([2, 3, 5].iter().filter_map(|n| dark(*n)));
    // v. 48's galagrahas (13, 14, 15, 1, 7, 8, 9, 4) and v. 54's
    // anadhyayas (14, 15, 1, 8), of both halves; the 15th is the full
    // Moon and the new.
    let mut rejected = both(&[1, 4, 7, 8, 9, 13, 14]);
    rejected.extend([Tithi::Purnima, Tithi::Amavasya]);
    // v. 48's "कृष्णे": the dark half past its first third.
    rejected.extend((6..=14).filter_map(dark));
    Graded {
        best: good_tithis,
        middling: Vec::new(),
        rejected,
        otherwise: Grade::Middling,
    }
}

/// What the thread ceremony's verses ask that a muhurta request does not
/// carry or the rule language does not yet express.
fn upanayana_unjudged() -> Vec<Unjudged> {
    vec![
        unjudged(
            "the boy's age: the 8th year from conception or birth for a brahmin, the 5th for one who wants learning, the 6th and 11th for a kshatriya, the 8th and 12th for a vaishya, and twice the time as the secondary season (v. 39)",
            "the boy's age, which a muhurta request does not carry: a consumer keeps the days that fall in it",
        ),
        unjudged(
            "the lagna's lord in the 6th or 8th (v. 41)",
            "an unwanted placement names grahas, not a lord that turns with the lagna",
        ),
        unjudged(
            "malefics in the 3rd, 6th and 11th, and the full Moon in Taurus or Cancer in the lagna (v. 42)",
            "they favour the time and are not graded",
        ),
        unjudged(
            "the lord of the boy's Veda and varna, its weekday and strength, and Jupiter or Venus in an enemy's sign, debilitated or defeated (vv. 43–44)",
            "the boy's Veda, which a muhurta request does not carry, and a graha's dignity, which is not graded",
        ),
        unjudged(
            "the boy's birth star, month and lagna, for an eldest son (v. 45)",
            "the boy's chart, which a muhurta request does not carry",
        ),
        unjudged(
            "Jupiter's transit from the boy's Moon sign: good in the 9th, 5th, 11th, 2nd and 7th, propitiated in the 10th, 6th, 3rd and 1st, bad in the 4th, 8th and 12th, an exalted or own-sign Jupiter excepted (vv. 46–47)",
            "the boy's chart, which a muhurta request does not carry",
        ),
        unjudged(
            "thunder before the morning twilight (v. 48)",
            "a day's weather is not a time",
        ),
        unjudged(
            "the lagna's navamsha and the Moon's, with their fruits (vv. 49–53)",
            "a navamsha's grade is not yet expressed",
        ),
        unjudged(
            "the bright 10th of Ashadha, 2nd of Jyeshtha, 11th of Pausha and 12th of Magha, and the day of the Sun's sidereal sankranti, as anadhyayas (v. 54)",
            "a tithi's grade that turns on the lunar month, and the sankranti's day, are not yet expressed; the four months' other anadhyayas are graded",
        ),
        unjudged(
            "the stars of each Veda (v. 57), and Punarvasu for a brahmin (the Piyushadhara on v. 40)",
            "the boy's Veda and varna, which a muhurta request does not carry",
        ),
        unjudged(
            "the mother in her courses after the nandi shraddha (v. 58)",
            "not a time",
        ),
        unjudged(
            "Jupiter in Leo or Capricorn, and the Sun with Jupiter (ch. I v. 48)",
            "a graha's sign closing the season is not yet expressed",
        ),
        bala_vriddha(),
    ]
}

/// The months of the thread ceremony: the five from Magha, which the
/// Piyushadhara on ch. V v. 40 gives as common to all three twice-born
/// after shutting the Sun's southern course ("माघादिषु च पंचसु", Vasishtha
/// and Kashyapa), its Ashadha rejected; Pausha joins them while the Sun is
/// in Capricorn, the northern course's part of that month (crux C1960).
const UPANAYANA_MONTHS: [Masa; 5] = [
    Masa::Magha,
    Masa::Phalguna,
    Masa::Chaitra,
    Masa::Vaishakha,
    Masa::Jyeshtha,
];

/// Jupiter's and Venus's infancy and old age, which ch. I v. 47 forbids
/// with their setting.
fn bala_vriddha() -> Unjudged {
    unjudged(
        "Jupiter's and Venus's infancy and old age (bala, vriddha), ch. I v. 47",
        "the days either side of an asta the texts count differ by body and side, and are not yet sourced",
    )
}

/// A day's limbs graded by a verse alone: no yoga or karana graded, and
/// the native's chandrabala Raman's, which the chapters leave to the
/// general rules.
fn verse_day(
    tithis: Graded<Tithi>,
    nakshatras: Graded<Nakshatra>,
    varas: Graded<Vara>,
) -> DayRules {
    DayRules {
        tithis,
        nakshatras,
        yogas: Graded::none(),
        karanas: Graded::none(),
        varas,
        chandrabala: ChandraBala::raman(),
    }
}

/// The house's beginning, the election ch. XII vv. 13–18 give and both
/// [`ActivityRules::chintamani_griharambha`] and
/// [`ActivityRules::chintamani_bhumi_puja`] hold, with what each asks
/// beyond it.
fn house_beginning(beyond: Vec<Unjudged>) -> ActivityRules {
    // v. 18: "भौमार्करिक्तामाद्यूने" — without Tuesday and Sunday, the
    // riktas, the new Moon and the first; both commentaries read the
    // first as the pratipada, of either half (crux C1132).
    let mut tithis = rikta();
    tithis.push(Tithi::Amavasya);
    tithis.extend(bright(1));
    tithis.extend(dark(1));
    // v. 15's stars good; v. 18's "विपञ्चके", without the panchaka, rejects
    // four of them and two quarters of a fifth, and the worse grade wins
    // (crux C1131).
    let mut best: Vec<Nakshatra> = DHRUVA.into();
    best.extend(MRIDU);
    best.extend([
        Nakshatra::Shatabhisha,
        Nakshatra::Swati,
        Nakshatra::Dhanishtha,
        Nakshatra::Hasta,
        Nakshatra::Pushya,
    ]);
    let day = verse_day(
        Graded::rejecting(tithis),
        Graded {
            best,
            middling: Vec::new(),
            rejected: PANCHAKA_STARS.into(),
            otherwise: Grade::Middling,
        },
        Graded::rejecting(vec![Vara::Ravivara, Vara::Mangalavara]),
    );
    let mut unjudged_list = vec![
        unjudged(
            "the door's facing by the month (v. 15) and by the tithi (v. 17)",
            "a request does not carry a facing",
        ),
        unjudged(
            "\"some\" who key the months on the Sun's sign from the dark half's first (v. 16), and Narada's and Vasishtha's months the commentary adds",
            "the months are v. 15's; a caller who keeps another reading sets `months` (crux C1134)",
        ),
        unjudged(
            "malefics in the 3rd, 6th and 11th (v. 18)",
            "they favour the time and are not graded",
        ),
        unjudged(
            "a graha's star and weekday at the beginning, with their fruits (vv. 26–28)",
            "not yet expressed",
        ),
        unjudged(
            "the native's birth star and sign",
            "the chapter's tara and the house's own reckoning (aya, vyaya) need the owner's chart",
        ),
        bala_vriddha(),
    ];
    unjudged_list.extend(beyond);
    ActivityRules {
        day,
        months: MonthRule::Lunar {
            months: Graded {
                best: Vec::new(),
                middling: Vec::new(),
                rejected: Vec::new(),
                otherwise: Grade::Rejected,
            },
            with_sun: HOUSE_MONTHS.into(),
        },
        // v. 18: "चरोने अङ्गे", a lagna not movable.
        lagnas: Graded::rejecting(MOVABLE.into()),
        hours: Hours::Any,
        shool: Vec::new(),
        padas: PANCHAKA_PADAS.into(),
        heeds: SET_AND_UNEVEN_MONTHS.into(),
        // v. 18: "व्यष्टान्त्यस्थैः शुभैः", the benefics out of the 8th and
        // 12th.
        unwanted: vec![Unwanted::of(BENEFICS.into(), vec![8, 12])],
        vrisha_vastu: true,
        pradosha: false,
        bars: Vec::new(),
        unjudged: unjudged_list,
        baseline: None,
    }
}

impl ActivityRules {
    /// Beginning a house, griharambha — Nepal's laying of the foundation
    /// (shilanyas, "जग राख्ने") — by *Muhurta Chintamani*, Vastu-prakarana
    /// (ch. XII vv. 13–18), heeding ch. I vv. 46–47.
    ///
    /// - **The bull** (vv. 13–14): the day's star counted from the Sun's
    ///   over the twenty-eight with Abhijit, "seven bad, eleven good, ten
    ///   bad" ([`ActivityRules::vrisha_vastu`]).
    /// - **Months** (v. 15): Phalguna with the Sun in Aquarius, Shravana
    ///   in Cancer or Leo, Pausha in Capricorn, Vaishakha in Aries or
    ///   Taurus, Margashirsha in Libra or Scorpio; the rest rejected.
    /// - **Stars** (v. 15): the fixed and the soft, Shatabhisha, Swati,
    ///   Dhanishtha, Hasta and Pushya good; (v. 18) the panchaka's
    ///   rejected, which takes Shatabhisha, Uttara Bhadrapada and Revati
    ///   back out of v. 15's and Dhanishtha's last two quarters with them
    ///   (crux C1131).
    /// - **Tithis** (v. 18): the riktas, the new Moon and the pratipada
    ///   rejected (crux C1132).
    /// - **Varas** (v. 18): Sunday and Tuesday rejected.
    /// - **Lagnas** (v. 18): movable rejected.
    /// - **Against** (v. 18): a benefic in the 8th or 12th.
    /// - **Heeds** (ch. I vv. 46–47): Jupiter or Venus set, and an adhika
    ///   or kshaya month.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Vara};
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let house = ActivityRules::chintamani_griharambha();
    /// house.check()?;
    /// assert!(house.vrisha_vastu);
    /// // v. 15 names Revati good and v. 18's panchaka rejects it: the worse wins.
    /// assert_eq!(house.day.nakshatras.grade(&Nakshatra::Revati), Grade::Rejected);
    /// assert_eq!(house.day.varas.grade(&Vara::Ravivara), Grade::Rejected);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn chintamani_griharambha() -> ActivityRules {
        house_beginning(vec![unjudged(
            "the first stone laid from the south-east (agneya) corner, clockwise (*Muhurta Martanda*, Vastu v. 16)",
            "a corner is not a time",
        )])
    }

    /// The ground's worship before the digging, bhumi puja, by *Muhurta
    /// Chintamani*: the house's beginning
    /// ([`ActivityRules::chintamani_griharambha`], ch. XII vv. 13–18),
    /// whose first act the digging is, with Rahu's mouth (v. 19) asked
    /// beside it.
    ///
    /// ```
    /// use teistro_muhurta::ActivityRules;
    ///
    /// let puja = ActivityRules::chintamani_bhumi_puja();
    /// puja.check()?;
    /// // The same election as the house's beginning; only what is asked beyond it differs.
    /// let house = ActivityRules::chintamani_griharambha();
    /// assert_eq!((&puja.day, &puja.months, &puja.lagnas), (&house.day, &house.months, &house.lagnas));
    /// assert!(puja.unjudged.iter().any(|u| u.what.contains("Rahu")));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn chintamani_bhumi_puja() -> ActivityRules {
        house_beginning(vec![unjudged(
            "the digging begun in the corner behind Rahu's mouth, which turns with the Sun's sign, from Leo for a house (v. 19)",
            "a corner is not a time",
        )])
    }

    /// The house's propitiation, vastu shanti — the vastu puja and the
    /// offering to the spirits (bhutabali) before entering — by *Muhurta
    /// Chintamani*, Grihapravesha-prakarana (ch. XIII v. 3), heeding
    /// ch. I vv. 46–47, whose "भवनारम्भप्रतिष्ठे" forbids a house's
    /// consecration as its beginning (crux C1133).
    ///
    /// - **Stars**: the soft, the fixed, the swift and the movable, and
    ///   Mula, good. Nothing else is named.
    ///
    /// The rest of vv. 3–4 — the lagna's houses, the tithis and varas, and
    /// Chaitra — is the entry's, which follows the worship.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let shanti = ActivityRules::chintamani_vastu_shanti();
    /// shanti.check()?;
    /// assert_eq!(shanti.day.nakshatras.grade(&Nakshatra::Mula), Grade::Best);
    /// assert_eq!(shanti.day.nakshatras.grade(&Nakshatra::Bharani), Grade::Middling);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn chintamani_vastu_shanti() -> ActivityRules {
        let mut best: Vec<Nakshatra> = MRIDU.into();
        best.extend(DHRUVA);
        best.extend(KSHIPRA);
        best.extend(CHARA);
        best.push(Nakshatra::Mula);
        ActivityRules {
            day: verse_day(Graded::none(), good(best, Grade::Middling), Graded::none()),
            months: MonthRule::Any,
            lagnas: Graded::none(),
            hours: Hours::Any,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: SET_AND_UNEVEN_MONTHS.into(),
            unwanted: Vec::new(),
            vrisha_vastu: false,
            pradosha: false,
            bars: Vec::new(),
            unjudged: vec![
                unjudged(
                    "the entry's lagna, tithis, varas and months (vv. 3–4)",
                    "the entry's, which follows the worship; a caller holding both at once lays the entry's rules over these",
                ),
                unjudged(
                    "Abhijit among the swift stars (ch. II v. 6)",
                    "not one of the twenty-seven a day's star is read among",
                ),
                bala_vriddha(),
            ],
            baseline: None,
        }
    }

    /// Opening a shop or a business, vipani ("दोकान खोल्ने"), by *Muhurta
    /// Chintamani*, Nakshatra-prakarana (ch. II v. 17, its second half).
    ///
    /// - **Stars**: the friendly (maitra, the soft, ch. II v. 7), the
    ///   fixed and the swift good.
    /// - **Tithis**: the riktas rejected.
    /// - **Varas**: Tuesday rejected.
    /// - **Lagnas**: Aquarius rejected.
    /// - **Against**: a malefic in the 12th or 8th.
    ///
    /// Chapter I vv. 46–47 does not name it, so no blackout is heeded.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Rashi, Vara};
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let shop = ActivityRules::chintamani_vipani();
    /// shop.check()?;
    /// assert_eq!(shop.day.nakshatras.grade(&Nakshatra::Anuradha), Grade::Best);
    /// assert_eq!(shop.day.varas.grade(&Vara::Mangalavara), Grade::Rejected);
    /// assert_eq!(shop.lagnas.grade(&Rashi::Aquarius), Grade::Rejected);
    /// assert!(shop.heeds.is_empty());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn chintamani_vipani() -> ActivityRules {
        let mut best: Vec<Nakshatra> = MRIDU.into();
        best.extend(DHRUVA);
        best.extend(KSHIPRA);
        ActivityRules {
            day: verse_day(
                Graded::rejecting(rikta()),
                good(best, Grade::Middling),
                Graded::rejecting(vec![Vara::Mangalavara]),
            ),
            months: MonthRule::Any,
            lagnas: Graded::rejecting(vec![Rashi::Aquarius]),
            hours: Hours::Any,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: Vec::new(),
            // "व्ययाष्टरहितैः पापैः": the malefics out of the 12th and 8th.
            unwanted: vec![Unwanted::of(MALEFICS.into(), vec![8, 12])],
            vrisha_vastu: false,
            pradosha: false,
            bars: Vec::new(),
            unjudged: vec![
                unjudged(
                    "the Moon and Venus in the lagna, and benefics in the 2nd, 11th and 10th",
                    "they favour the time and are not graded",
                ),
                unjudged(
                    "Abhijit among the swift stars (ch. II v. 6)",
                    "not one of the twenty-seven a day's star is read among",
                ),
            ],
            baseline: None,
        }
    }

    /// The marriage, vivaha, by *Muhurta Chintamani*, Vivaha-prakarana
    /// (ch. VI vv. 13 and 55), heeding ch. I vv. 46–47.
    ///
    /// - **Months** (v. 13): the Sun in Gemini, Aquarius, Capricorn,
    ///   Scorpio, Taurus or Aries; the rest shut. A gate, which grades
    ///   none of the six above another.
    /// - **Stars** (v. 55): Mrigashira, Hasta, Mula, Anuradha, Magha,
    ///   Rohini, Revati, the three Uttaras and Swati good.
    /// - **Tithis** (v. 55): the riktas and the new Moon rejected.
    /// - **Varas** (v. 55, "शुभेऽह्नि"): Monday, Wednesday, Thursday and
    ///   Friday good.
    /// - **Heeds** (ch. I vv. 46–47): Jupiter or Venus set, and an adhika
    ///   or kshaya month.
    ///
    /// The doshas of the lagna (vv. 44–45 and on) — kartari, the Moon
    /// joined, a malefic in the lagna — are the general clauses every
    /// window carries, and weigh against it; none bars. Where this parts
    /// from the committee's marriage days is `saait-measured.md`'s and
    /// crux C206's.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Tithi};
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let marriage = ActivityRules::chintamani_vivaha();
    /// marriage.check()?;
    /// assert_eq!(marriage.day.nakshatras.grade(&Nakshatra::Magha), Grade::Best);
    /// assert_eq!(marriage.day.tithis.grade(&Tithi::Amavasya), Grade::Rejected);
    /// assert!(marriage.bars.is_empty());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn chintamani_vivaha() -> ActivityRules {
        let mut tithis = rikta();
        tithis.push(Tithi::Amavasya);
        let mut best: Vec<Nakshatra> = vec![
            Nakshatra::Mrigashira,
            Nakshatra::Hasta,
            Nakshatra::Mula,
            Nakshatra::Anuradha,
            Nakshatra::Magha,
            Nakshatra::Rohini,
            Nakshatra::Revati,
        ];
        best.extend([
            Nakshatra::UttaraPhalguni,
            Nakshatra::UttaraAshadha,
            Nakshatra::UttaraBhadrapada,
            Nakshatra::Swati,
        ]);
        ActivityRules {
            day: verse_day(
                Graded::rejecting(tithis),
                good(best, Grade::Middling),
                good(benefic_varas(), Grade::Middling),
            ),
            months: MonthRule::Solar {
                signs: Graded::admitting(vec![
                    Rashi::Gemini,
                    Rashi::Aquarius,
                    Rashi::Capricorn,
                    Rashi::Scorpio,
                    Rashi::Taurus,
                    Rashi::Aries,
                ]),
            },
            lagnas: Graded::none(),
            hours: Hours::Any,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: SET_AND_UNEVEN_MONTHS.into(),
            unwanted: Vec::new(),
            vrisha_vastu: false,
            pradosha: false,
            bars: Vec::new(),
            unjudged: vec![
                unjudged(
                    "the Sun in Gemini only in Ashadha's first third (v. 13)",
                    "a sign's grade that turns on the lunar month is not yet expressed; Gemini is admitted whole",
                ),
                unjudged(
                    "a star free of vedha, the pierced pairs of v. 56",
                    "a graha's piercing of the day's star is not yet expressed",
                ),
                unjudged(
                    "the couple's birth month, star and tithi, eldest children, and Jupiter's and the Sun's strength for each (vv. 12, 14–15)",
                    "the couple's charts, which a muhurta request does not carry",
                ),
                unjudged(
                    "the lagna's own purity and the ten doshas (vv. 44 on)",
                    "graded only as the general clauses every window carries",
                ),
                bala_vriddha(),
            ],
            baseline: None,
        }
    }

    /// The thread ceremony, upanayana — Nepal's bratabandha ("व्रतबन्ध") —
    /// by *Muhurta Chintamani*, Samskara-prakarana (ch. V vv. 40–42, 48,
    /// 54 and 55), heeding ch. I v. 47, which names the mauñji among the
    /// rites a set Jupiter or Venus and an uneven month forbid.
    ///
    /// - **Stars** (v. 40, "क्षिप्रध्रुवाहिचरमूलमृदुत्रिपूर्वारौद्रे"): the
    ///   swift, the fixed, Ashlesha, the movable, Mula, the soft, the three
    ///   Purvas and Ardra good — twenty-two; the other five, Bharani,
    ///   Krittika, Magha, Vishakha and Jyeshtha, rejected, as the
    ///   Piyushadhara on v. 40 reads the verse ("अर्थात् भरणीकृत्तिकामघा-
    ///   विशाखाज्येष्ठासु न कार्यम्", crux C1967).
    /// - **Varas** (v. 40, "अर्कविद्गुरुसितेन्दुदिने"): Sunday, Wednesday,
    ///   Thursday, Friday and Monday good; (v. 48, "शनौ") Saturday
    ///   rejected; Tuesday unnamed, middling.
    /// - **Tithis** (v. 40, "द्वित्रीषुरुद्ररविदिक्प्रमिते तिथौ च
    ///   कृष्णादिमत्रिलवकेऽपि"): the 2nd, 3rd, 5th, 11th, 12th and 10th good,
    ///   of the bright half and of the dark half's first third. Rejected:
    ///   (v. 48) the dark half past its first third, its 6th to the new
    ///   Moon; the galagrahas, the 13th, 14th, 15th and 1st, the 7th, 8th
    ///   and 9th and the 4th, of either half (the Piyushadhara on v. 48,
    ///   after Guru); and (v. 54) the anadhyayas the 14th, the full and new
    ///   Moon, the 1st and the 8th. The bright 6th alone is unnamed.
    /// - **Months**: the verses name none. The Piyushadhara on v. 40 shuts
    ///   the southern course, so the northern course's six months are
    ///   praised; it gives the five lunar months from Magha as the common
    ///   season of every varna and sets Ashadha aside. Read as the five,
    ///   with Pausha admitted while the Sun is in Capricorn, the northern
    ///   course's part of it, and Ashadha rejected (crux C1960).
    /// - **Hours** (v. 40, "न चापराह्णे"; v. 48, "निश्यपराह्णके"): before
    ///   the afternoon, the daylight's last third, and not at night
    ///   ([`Hours::BeforeAfternoon`]).
    /// - **Pradosha** (vv. 48, 55): a day whose 12th, 6th or 3rd gives way
    ///   before half the night, a prahara and a half, or a prahara, counts
    ///   against the time ([`ActivityRules::pradosha`]).
    /// - **Against** (v. 41): Venus, Jupiter and the Moon in the 6th or 8th,
    ///   the Moon and Venus in the 12th, a malefic in the lagna, the 5th or
    ///   the 8th, "अधमाः"; (v. 42) a benefic in the 8th, 6th or 12th.
    /// - **Heeds** (ch. I v. 47): Jupiter or Venus set, and an adhika or
    ///   kshaya month.
    ///
    /// The verses avoid and do not rank, so nothing bars (crux C205, C1962).
    /// What turns on the boy — his age, his Veda's lord and stars, his
    /// birth star and month, Jupiter's transit from his Moon — is listed in
    /// `unjudged`.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Tithi, Vara};
    /// use teistro_muhurta::{ActivityRules, Grade, Hours};
    ///
    /// let thread = ActivityRules::chintamani_upanayana();
    /// thread.check()?;
    /// assert_eq!(thread.day.nakshatras.grade(&Nakshatra::Ardra), Grade::Best);
    /// assert_eq!(thread.day.nakshatras.grade(&Nakshatra::Bharani), Grade::Rejected);
    /// // The dark half's first third keeps its 2nd, 3rd and 5th; its 6th is past it.
    /// assert_eq!(thread.day.tithis.grade(&Tithi::KrishnaPanchami), Grade::Best);
    /// assert_eq!(thread.day.tithis.grade(&Tithi::KrishnaShashthi), Grade::Rejected);
    /// assert_eq!(thread.day.varas.grade(&Vara::Shanivara), Grade::Rejected);
    /// assert_eq!(thread.hours, Hours::BeforeAfternoon);
    /// assert!(thread.pradosha && thread.bars.is_empty());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn chintamani_upanayana() -> ActivityRules {
        let mut best: Vec<Nakshatra> = KSHIPRA.into();
        best.extend(DHRUVA);
        best.push(Nakshatra::Ashlesha);
        best.extend(CHARA);
        best.push(Nakshatra::Mula);
        best.extend(MRIDU);
        best.extend([
            Nakshatra::PurvaPhalguni,
            Nakshatra::PurvaAshadha,
            Nakshatra::PurvaBhadrapada,
            Nakshatra::Ardra,
        ]);
        ActivityRules {
            day: verse_day(
                upanayana_tithis(),
                good(best, Grade::Rejected),
                Graded {
                    best: vec![
                        Vara::Ravivara,
                        Vara::Budhavara,
                        Vara::Guruvara,
                        Vara::Shukravara,
                        Vara::Somavara,
                    ],
                    middling: Vec::new(),
                    rejected: vec![Vara::Shanivara],
                    otherwise: Grade::Middling,
                },
            ),
            months: MonthRule::Lunar {
                months: Graded::admitting(UPANAYANA_MONTHS.into()),
                with_sun: vec![MonthWithSun {
                    masa: Masa::Pausha,
                    sun: Rashi::Capricorn,
                }],
            },
            lagnas: Graded::none(),
            hours: Hours::BeforeAfternoon,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: SET_AND_UNEVEN_MONTHS.into(),
            unwanted: vec![
                // v. 41: "कवीज्यचन्द्रलग्नपा रिपौ मृतौ व्रतेऽधमाः । व्ययेऽब्जभार्गवौ
                // तथा तनौ मृतौ सुते खलाः".
                Unwanted::of(vec![Graha::Venus, Graha::Jupiter, Graha::Moon], vec![6, 8]),
                Unwanted::of(vec![Graha::Moon, Graha::Venus], vec![12]),
                Unwanted::of(MALEFICS.into(), vec![1, 5, 8]),
                // v. 42: "व्रतबन्धेऽष्टषड्रिःफवर्जिताः शोभनाः शुभाः".
                Unwanted::of(BENEFICS.into(), vec![6, 8, 12]),
            ],
            vrisha_vastu: false,
            pradosha: true,
            bars: Vec::new(),
            unjudged: upanayana_unjudged(),
            baseline: None,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{CHARA, DHRUVA, KSHIPRA, MRIDU};
    use crate::activity::{ActivityRules, MonthRule, MonthWithSun, vrisha_count, vrisha_grade};
    use crate::grade::Grade;
    use crate::rites::{bright, dark};
    use crate::season::BlackoutKind;
    use teistro_core::catalogue::{Masa, Nakshatra, Rashi, Tithi, Vara};

    #[test]
    fn every_rite_checks() {
        for rules in [
            ActivityRules::chintamani_griharambha(),
            ActivityRules::chintamani_bhumi_puja(),
            ActivityRules::chintamani_vastu_shanti(),
            ActivityRules::chintamani_vipani(),
            ActivityRules::chintamani_vivaha(),
            ActivityRules::chintamani_upanayana(),
        ] {
            assert_eq!(rules.check(), Ok(()));
            assert!(rules.bars.is_empty(), "a verse avoids and does not bar");
        }
    }

    #[test]
    fn the_groups_are_the_chapters() {
        // Twenty-seven less Abhijit: no star in two groups.
        let mut all: Vec<Nakshatra> = DHRUVA.into();
        all.extend(MRIDU);
        all.extend(KSHIPRA);
        all.extend(CHARA);
        let count = all.len();
        all.sort_by_key(|n| n.id());
        all.dedup();
        assert_eq!(all.len(), count);
    }

    #[test]
    fn the_bull_is_seven_eleven_ten() {
        let grades: Vec<Grade> = (1..=28).map(vrisha_grade).collect();
        let best = grades.iter().filter(|g| **g == Grade::Best).count();
        assert_eq!(best, 11);
        assert_eq!(
            grades
                .iter()
                .take(7)
                .filter(|g| **g == Grade::Rejected)
                .count(),
            7
        );
        assert_eq!(
            grades
                .iter()
                .skip(18)
                .filter(|g| **g == Grade::Rejected)
                .count(),
            10
        );
    }

    #[test]
    fn the_bull_counts_abhijit() {
        // The Sun in Shravana's start, the Moon in U.Ashadha's first quarter:
        // Shravana (23rd of 28), Abhijit, then on round to U.Ashadha (21st)
        // is the 27th.
        assert_eq!(vrisha_count(282.0, 268.0), 27);
        // The Sun in Ashwini, the Moon in Shravana, Abhijit counted between:
        // the 23rd.
        assert_eq!(vrisha_count(1.0, 282.0), 23);
        // Without Abhijit the Moon in Revati from the Sun in Ashwini is the
        // 27th; with it the 28th.
        assert_eq!(vrisha_count(1.0, 359.0), 28);
    }

    #[test]
    fn the_house_months_need_the_sun() {
        let house = ActivityRules::chintamani_griharambha();
        assert!(matches!(
            &house.months,
            MonthRule::Lunar { months, with_sun }
                if months.grade(&Masa::Phalguna) == Grade::Rejected
                    && with_sun.iter().any(|w| w.masa == Masa::Phalguna && w.sun == Rashi::Aquarius)
                    && !with_sun.iter().any(|w| w.masa == Masa::Kartika)
        ));
    }

    #[test]
    fn the_house_avoids_the_first_of_both_halves() {
        let house = ActivityRules::chintamani_griharambha();
        for tithi in [bright(1), dark(1), Some(Tithi::Amavasya)]
            .into_iter()
            .flatten()
        {
            assert_eq!(house.day.tithis.grade(&tithi), Grade::Rejected, "{tithi:?}");
        }
        assert_eq!(
            house.day.nakshatras.grade(&Nakshatra::Dhanishtha),
            Grade::Best
        );
        assert_eq!(house.day.nakshatras.grade(&Nakshatra::Swati), Grade::Best);
        assert!(house.heeds.contains(&BlackoutKind::GuruAsta));
    }

    #[test]
    fn the_shop_is_open_on_sunday() {
        let shop = ActivityRules::chintamani_vipani();
        assert_eq!(shop.day.varas.grade(&Vara::Ravivara), Grade::Middling);
        assert_eq!(shop.day.tithis.grade(&Tithi::Amavasya), Grade::Middling);
    }

    #[test]
    fn the_thread_ceremonys_tithis_are_the_verses() {
        let thread = ActivityRules::chintamani_upanayana();
        let grade = |t: Option<Tithi>| t.map(|t| thread.day.tithis.grade(&t));
        // v. 40's numbers, of the bright half and the dark half's first third.
        for n in [2, 3, 5, 10, 11, 12] {
            assert_eq!(grade(bright(n)), Some(Grade::Best), "bright {n}");
        }
        for n in [2, 3, 5] {
            assert_eq!(grade(dark(n)), Some(Grade::Best), "dark {n}");
        }
        // The dark half's 10th to 12th are v. 40's numbers past its first third.
        for n in [10, 11, 12] {
            assert_eq!(grade(dark(n)), Some(Grade::Rejected), "dark {n}");
        }
        // Thirty tithis: nine good, one unnamed, twenty rejected.
        let all: Vec<Grade> = (0..30)
            .filter_map(Tithi::from_id)
            .map(|t| thread.day.tithis.grade(&t))
            .collect();
        let count = |g: Grade| all.iter().filter(|x| **x == g).count();
        assert_eq!(
            (
                count(Grade::Best),
                count(Grade::Middling),
                count(Grade::Rejected)
            ),
            (9, 1, 20)
        );
        assert_eq!(grade(bright(6)), Some(Grade::Middling));
    }

    #[test]
    fn the_thread_ceremonys_stars_and_months() {
        let thread = ActivityRules::chintamani_upanayana();
        let best = (0..27)
            .filter_map(Nakshatra::from_id)
            .filter(|n| thread.day.nakshatras.grade(n) == Grade::Best)
            .count();
        assert_eq!(best, 22);
        assert_eq!(
            thread.day.nakshatras.grade(&Nakshatra::Jyeshtha),
            Grade::Rejected
        );
        assert!(matches!(
            &thread.months,
            MonthRule::Lunar { months, with_sun }
                if with_sun == &[MonthWithSun { masa: Masa::Pausha, sun: Rashi::Capricorn }]
                    && months.grade(&Masa::Jyeshtha) == Grade::Middling
                    && months.grade(&Masa::Ashadha) == Grade::Rejected
                    && months.grade(&Masa::Pausha) == Grade::Rejected
        ));
        assert_eq!(thread.day.varas.grade(&Vara::Mangalavara), Grade::Middling);
        assert!(thread.heeds.contains(&BlackoutKind::ShukraAsta));
    }

    #[test]
    fn the_marriage_shuts_the_other_months() {
        let marriage = ActivityRules::chintamani_vivaha();
        assert!(matches!(
            &marriage.months,
            MonthRule::Solar { signs }
                if signs.grade(&Rashi::Gemini) == Grade::Middling
                    && signs.grade(&Rashi::Cancer) == Grade::Rejected
        ));
        assert_eq!(
            marriage.day.varas.grade(&Vara::Mangalavara),
            Grade::Middling
        );
    }
}
