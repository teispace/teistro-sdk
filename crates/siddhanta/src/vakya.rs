//! The Sun of the *Vakyakarana*, the manual the Tamil vakya almanacs
//! compute from (`docs/calendars/regional-solar.md`, "The two Tamil schools").
//!
//! The *Vakyakarana* (Kanchi, c. 1300; edited by T. S. Kuppanna Sastri
//! and K. V. Sarma, Madras, 1962) belongs to the Aryabhata school. It
//! gives the Sun in three rules and two tables, all whole arithmetic:
//!
//! - **the year** (I.2–3): the Kali days to the Sun's entry into Mesha
//!   are the Kali years elapsed times 365, plus a quarter of them, plus
//!   five times them less 1237 over 576, counted from the mean sunrise at
//!   the Ujjain meridian on the epoch's first day, a Friday
//!   ([`mesha_sankranti`]);
//! - **the months** (I.4): the twelve sankranti vakyas give the days,
//!   nadis and vinadis from the Mesha sankranti to each sign's
//!   ([`SANKRANTI_VAKYAS`], [`sankranti`]);
//! - **the day's length** (III.7–8): the cara-intervals 20, 16 and 6⅔
//!   vinadis for each sign of the sayana Sun's bhuja, times the place's
//!   equinoctial shadow (palabha), give the cara, and thirty nadis less
//!   the cara is the daytime ([`cara_vinadis`], [`day_length_vinadis`]),
//!   with the precession of III.1 ([`precession_degrees`]).
//!
//! The almanacs make none of the corrections from the Ujjain mean sunrise
//! to a place's sunrise (the edition's introduction, p. xvi): a moment's
//! fraction of its Kali day is read as the nadis after the day's sunrise.
//!
//! ```
//! use teistro_siddhanta::vakya;
//!
//! // The edition's Example 1: Saka 1869 gone is Kali 5048, and the year's
//! // Mesha sankranti falls 18,43,823 days 40 nadis 18¾ vinadis from the epoch,
//! // which the edition rounds to 19.
//! let mesha = vakya::mesha_sankranti(vakya::kali_years(1869));
//! assert_eq!(mesha.day(), 1_843_824);
//! assert_eq!(mesha.since_sunrise(), (40, 18, 3));
//! ```

/// Parts of a day the arithmetic counts in: a quarter of a vinadi
/// (1/14,400 of a day), the least unit that both the year rule's 576ths
/// and the vakyas' vinadis divide.
pub const PARTS_PER_DAY: i64 = 14_400;

/// Parts in a vinadi (a sixtieth of a nadi, which is a sixtieth of a day).
const PARTS_PER_VINADI: i64 = 4;

/// Vinadis in a day.
pub const VINADIS_PER_DAY: i64 = 3_600;

/// The Kali years elapsed when Saka 0 began: the Saka year plus 3179
/// (I.2).
pub const SAKA_TO_KALI: i64 = 3_179;

/// The Kali years elapsed at the start of a Saka year counted as elapsed
/// years, as the Tamil solar year is (I.2).
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// assert_eq!(vakya::kali_years(1869), 5048);
/// ```
#[must_use]
pub const fn kali_years(saka_elapsed: i64) -> i64 {
    saka_elapsed + SAKA_TO_KALI
}

/// The days, nadis and vinadis from the Mesha sankranti to each sign's
/// sankranti, Mesha's first (I.4). The text gives them as the remainders
/// after casting out sevens, the week-days and times of the mnemonics
/// beginning *Srirgunamitra* (2-55-32, 6-19-44, …, 1-15-31); the whole
/// days are the commentary's (*kulina* 31 … the year's 365), and the
/// last, the next Mesha's, is the year itself: 365 days 15 nadis 31
/// vinadis, which is the year rule's 365 + ¼ + 5/576 days to the vinadi.
///
/// ```
/// use teistro_siddhanta::vakya::SANKRANTI_VAKYAS;
///
/// // Each remainder after casting out sevens is the mnemonic's week-day count.
/// let weekdays: Vec<u16> = SANKRANTI_VAKYAS.iter().skip(1).map(|&(days, _, _)| days % 7).collect();
/// assert_eq!(weekdays, [2, 6, 2, 6, 2, 4, 6, 1, 2, 4, 5]);
/// ```
pub const SANKRANTI_VAKYAS: [(u16, u8, u8); 12] = [
    (0, 0, 0),
    (30, 55, 32),
    (62, 19, 44),
    (93, 56, 22),
    (125, 24, 34),
    (156, 26, 44),
    (186, 54, 6),
    (216, 48, 13),
    (246, 18, 37),
    (275, 39, 30),
    (305, 6, 46),
    (334, 55, 10),
];

/// A moment counted from the *Vakyakarana*'s epoch, in quarter-vinadis
/// ([`PARTS_PER_DAY`] to a day) from the mean sunrise at the Ujjain
/// meridian that began the first Kali day.
///
/// ```
/// use teistro_siddhanta::vakya::{KaliMoment, PARTS_PER_DAY};
///
/// let moment = KaliMoment::from_parts(PARTS_PER_DAY + PARTS_PER_DAY / 2);
/// assert_eq!(moment.day(), 2);
/// assert_eq!(moment.since_sunrise(), (30, 0, 0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KaliMoment {
    parts: i64,
}

impl KaliMoment {
    /// The moment so many quarter-vinadis after the epoch.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::KaliMoment;
    ///
    /// assert_eq!(KaliMoment::from_parts(4).parts(), 4);
    /// ```
    #[must_use]
    pub const fn from_parts(parts: i64) -> KaliMoment {
        KaliMoment { parts }
    }

    /// The quarter-vinadis after the epoch.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::KaliMoment;
    ///
    /// assert_eq!(KaliMoment::from_parts(-3).parts(), -3);
    /// ```
    #[must_use]
    pub const fn parts(self) -> i64 {
        self.parts
    }

    /// The Kali day the moment falls in, from sunrise to sunrise, the
    /// epoch's Friday being day 1 (the edition's Example 1 counts the
    /// day that holds a night-time sankranti "to the end of the day").
    ///
    /// ```
    /// use teistro_siddhanta::vakya::KaliMoment;
    ///
    /// assert_eq!(KaliMoment::from_parts(0).day(), 1);
    /// assert_eq!(KaliMoment::from_parts(-1).day(), 0);
    /// ```
    #[must_use]
    pub const fn day(self) -> i64 {
        self.parts.div_euclid(PARTS_PER_DAY) + 1
    }

    /// The vinadis after the sunrise that began [`KaliMoment::day`], with
    /// any quarter-vinadis as a fraction.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::KaliMoment;
    ///
    /// assert_eq!(KaliMoment::from_parts(4 * 61 + 2).vinadis_after_sunrise(), 61.5);
    /// ```
    #[must_use]
    pub fn vinadis_after_sunrise(self) -> f64 {
        let parts = self.parts.rem_euclid(PARTS_PER_DAY);
        // Below 14,400, so the conversion is exact.
        f64::from(u16::try_from(parts).unwrap_or(u16::MAX)) / 4.0
    }

    /// The nadis, vinadis and quarter-vinadis after the day's sunrise, as
    /// an almanac prints a moment (to the vinadi, the quarters aside).
    ///
    /// ```
    /// use teistro_siddhanta::vakya::KaliMoment;
    ///
    /// assert_eq!(KaliMoment::from_parts(4 * 61 + 2).since_sunrise(), (1, 1, 2));
    /// ```
    #[must_use]
    pub const fn since_sunrise(self) -> (i64, i64, i64) {
        let parts = self.parts.rem_euclid(PARTS_PER_DAY);
        let vinadis = parts / PARTS_PER_VINADI;
        (vinadis / 60, vinadis % 60, parts % PARTS_PER_VINADI)
    }
}

/// The Sun's entry into Mesha that opens a Kali year (I.2–3): the years
/// elapsed times 365, plus a quarter of them, plus five times them less
/// 1237 over 576, in days from the epoch. The 1237/576 days (2 days 8
/// nadis 51¼ vinadis) are the true year's start before the mean year's,
/// the Sun's equation of centre at Mesha, constant since the school holds
/// the apsis fixed (the edition's introduction, p. xiii).
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// // Kali 5048: 18,43,823 days 40 nadis 18¾ vinadis (Example 1), a Sunday:
/// // day 1,843,824 leaves 3 after casting out sevens, counted from Friday.
/// let mesha = vakya::mesha_sankranti(5048);
/// assert_eq!((mesha.day(), mesha.day() % 7), (1_843_824, 3));
/// ```
#[must_use]
pub const fn mesha_sankranti(kali_years: i64) -> KaliMoment {
    // 365 + 1/4 + 5/576 days a year, less 1237/576: (210,389 y - 1237)/576
    // days, and 14,400 / 576 = 25 parts to a 576th.
    KaliMoment::from_parts(25 * (210_389 * kali_years - 1_237))
}

/// The Sun's entry into a sign, 0 for Mesha, in a Kali year (I.4): the
/// year's Mesha sankranti plus the sign's vakya. `None` for a sign
/// outside 0 to 11.
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// // Kali 5098 (1997): Kataka at 32 nadis 43 vinadis after sunrise,
/// // as the Manonmani Vilasam almanac prints it for 16 July 1997.
/// let kataka = vakya::sankranti(5098, 3).ok_or("a sign")?;
/// assert_eq!(kataka.since_sunrise().0, 32);
/// assert_eq!(kataka.since_sunrise().1, 43);
/// assert_eq!(vakya::sankranti(5098, 12), None);
/// # Ok::<(), &str>(())
/// ```
#[must_use]
pub fn sankranti(kali_years: i64, sign: u8) -> Option<KaliMoment> {
    let &(days, nadis, vinadis) = SANKRANTI_VAKYAS.get(usize::from(sign))?;
    let vinadis = i64::from(days) * VINADIS_PER_DAY + i64::from(nadis) * 60 + i64::from(vinadis);
    Some(KaliMoment::from_parts(
        mesha_sankranti(kali_years).parts() + vinadis * PARTS_PER_VINADI,
    ))
}

/// The precession in degrees for a Kali year (III.1): the years elapsed
/// less 3600, less a 121st of themselves, over 60. The verse goes on to
/// reflect a total past 24° about 24° (an oscillation the commentary
/// calls wrong); this is the unreflected total, which fits the printed
/// day lengths of a vakya almanac better (crux C1203).
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// // Example 15: Kali 5048 gives 23° 56'.
/// let degrees = vakya::precession_degrees(5048);
/// assert!((degrees - (23.0 + 56.0 / 60.0)).abs() < 1.0 / 60.0);
/// ```
#[must_use]
pub fn precession_degrees(kali_years: i64) -> f64 {
    // (y - 3600) × 120/121 / 60 = 2 (y - 3600) / 121, exact in i32 for any
    // year an almanac can name.
    let excess = i32::try_from(kali_years - 3_600).unwrap_or(i32::MAX);
    2.0 * f64::from(excess) / 121.0
}

/// An equinoctial shadow (palabha): the noon shadow of a twelve-angula
/// gnomon at an equinox, in angulas and sixtieths (vyangulas), which is
/// how the *Vakyakarana* names a place's latitude (III.6).
///
/// ```
/// use teistro_siddhanta::vakya::Palabha;
///
/// // Pudukkottai's 2 angulas 12 vyangulas (Example 19).
/// assert_eq!(Palabha::from_vyangulas(132).angulas(), 2.2);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Palabha {
    vyangulas: u16,
}

impl Palabha {
    /// A shadow of so many vyangulas (sixtieths of an angula).
    ///
    /// ```
    /// use teistro_siddhanta::vakya::Palabha;
    ///
    /// assert_eq!(Palabha::from_vyangulas(135).vyangulas(), 135);
    /// ```
    #[must_use]
    pub const fn from_vyangulas(vyangulas: u16) -> Palabha {
        Palabha { vyangulas }
    }

    /// The shadow in vyangulas.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::Palabha;
    ///
    /// assert_eq!(Palabha::from_vyangulas(132).vyangulas(), 132);
    /// ```
    #[must_use]
    pub const fn vyangulas(self) -> u16 {
        self.vyangulas
    }

    /// The shadow in angulas.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::Palabha;
    ///
    /// assert_eq!(Palabha::from_vyangulas(135).angulas(), 2.25);
    /// ```
    #[must_use]
    pub fn angulas(self) -> f64 {
        f64::from(self.vyangulas) / 60.0
    }
}

/// The cara-intervals of III.7, in vinadis per angula of shadow, for the
/// first, second and third sign of the bhuja.
const CARA_INTERVALS: [f64; 3] = [20.0, 16.0, 20.0 / 3.0];

/// The cara (III.7), the daytime's difference from thirty nadis, in
/// vinadis: for each sign of the sayana Sun's bhuja, its cara-interval
/// times the shadow, a part sign in proportion; negative while the
/// sayana Sun is in its first six signs (the day longer than thirty
/// nadis), positive in the other six.
///
/// ```
/// use teistro_siddhanta::vakya::{self, Palabha};
///
/// // Example 20: the Sun at 4s 22° 30' sayana, a shadow of 3 angulas: −72.
/// let cara = vakya::cara_vinadis(142.5, Palabha::from_vyangulas(180));
/// assert!((cara + 72.0).abs() < 1e-9);
/// ```
#[must_use]
pub fn cara_vinadis(sayana_degrees: f64, shadow: Palabha) -> f64 {
    let longitude = sayana_degrees.rem_euclid(360.0);
    let within = longitude.rem_euclid(180.0);
    let bhuja = if within > 90.0 {
        180.0 - within
    } else {
        within
    };
    let mut cara = 0.0;
    let mut left = bhuja;
    for interval in CARA_INTERVALS {
        let part = left.min(30.0);
        cara += interval * part / 30.0;
        left -= part;
    }
    let cara = cara * shadow.angulas();
    if longitude < 180.0 { -cara } else { cara }
}

/// The daytime in vinadis (III.8): thirty nadis less the cara, for the
/// Sun at a sidereal longitude in a Kali year, its sayana longitude being
/// that plus the year's [`precession_degrees`].
///
/// ```
/// use teistro_siddhanta::vakya::{self, Palabha};
///
/// // A shadow of 2¼ angulas, the Sun entering Mesha in Kali 5098 (1997):
/// // 30 nadis 37 vinadis, as the Manonmani Vilasam almanac prints the day.
/// let day = vakya::day_length_vinadis(0.0, 5098, Palabha::from_vyangulas(135));
/// assert_eq!(day.round(), 1837.0);
/// ```
#[must_use]
pub fn day_length_vinadis(sidereal_degrees: f64, kali_years: i64, shadow: Palabha) -> f64 {
    let sayana = sidereal_degrees + precession_degrees(kali_years);
    1_800.0 - cara_vinadis(sayana, shadow)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn example_1_reproduces_the_kali_days_and_the_new_year() {
        // Saka 1869 gone: Kali days 18,43,823-40-19 to the true year, at
        // night, so the day holding it is 18,43,824, a Sunday, and the
        // New Year's day is Monday.
        let mesha = mesha_sankranti(kali_years(1869));
        assert_eq!(mesha.day(), 1_843_824);
        assert_eq!(mesha.since_sunrise(), (40, 18, 3));
        assert_eq!(mesha.day() % 7, 3);
    }

    #[test]
    fn the_year_rule_is_the_aryabhata_year() {
        // 365 days 15 nadis 31 vinadis 15 vipalas a year: the Mesha
        // sankrantis step by 365.2586805... days, in quarter-vinadis
        // 5,259,725 (365 × 14,400 + 3,725).
        for years in [0, 1, 4000, 5048, 5126] {
            let step = mesha_sankranti(years + 1).parts() - mesha_sankranti(years).parts();
            assert_eq!(step, 25 * 210_389);
        }
        // The last vakya is the year to the vinadi.
        assert_eq!(25 * 210_389 / PARTS_PER_VINADI, 365 * 3600 + 15 * 60 + 31);
    }

    #[test]
    fn the_vakyas_are_increasing_and_their_days_fit_the_mnemonics() {
        let mnemonic: [(u16, u8, u8); 11] = [
            (2, 55, 32),
            (6, 19, 44),
            (2, 56, 22),
            (6, 24, 34),
            (2, 26, 44),
            (4, 54, 6),
            (6, 48, 13),
            (1, 18, 37),
            (2, 39, 30),
            (4, 6, 46),
            (5, 55, 10),
        ];
        for (vakya, (weekday, nadis, vinadis)) in SANKRANTI_VAKYAS.iter().skip(1).zip(mnemonic) {
            assert_eq!((vakya.0 % 7, vakya.1, vakya.2), (weekday, nadis, vinadis));
        }
        assert!(
            SANKRANTI_VAKYAS
                .iter()
                .zip(SANKRANTI_VAKYAS.iter().skip(1))
                .all(|(a, b)| a < b)
        );
    }

    #[test]
    fn a_sankranti_is_the_mesha_moment_plus_its_vakya() {
        let mesha = mesha_sankranti(5048);
        let vrishabha = sankranti(5048, 1).unwrap_or(mesha);
        assert_eq!(
            vrishabha.parts() - mesha.parts(),
            (30 * 3600 + 55 * 60 + 32) * PARTS_PER_VINADI
        );
        assert_eq!(sankranti(5048, 0), Some(mesha));
        assert_eq!(sankranti(5048, 12), None);
    }

    #[test]
    fn printed_sankranti_times_of_the_manonmani_vilasam_almanac() {
        // (Kali year, sign, the nadis and vinadis printed for the Sun's entry).
        let printed: [(i64, u8, i64, i64); 4] = [
            (5098, 3, 32, 43),
            (5098, 4, 0, 55),
            (5098, 5, 3, 5),
            (5108, 6, 5, 40),
        ];
        for (year, sign, nadis, vinadis) in printed {
            let moment = sankranti(year, sign).unwrap_or(KaliMoment::from_parts(0));
            let at = moment.vinadis_after_sunrise();
            let printed_at = f64::from(u16::try_from(nadis * 60 + vinadis).unwrap_or(0));
            assert!(
                (at - printed_at).abs() <= 1.0,
                "{year} {sign}: {at} against {printed_at}"
            );
        }
    }

    #[test]
    fn a_moment_before_the_epoch_counts_back() {
        let moment = KaliMoment::from_parts(-PARTS_PER_DAY / 4);
        assert_eq!(moment.day(), 0);
        assert_eq!(moment.since_sunrise(), (45, 0, 0));
        assert!((moment.vinadis_after_sunrise() - 2_700.0).abs() < 1e-9);
    }

    #[test]
    fn cara_follows_the_quadrants() {
        let shadow = Palabha::from_vyangulas(60);
        // The full bhuja: 20 + 16 + 6⅔ vinadis an angula.
        assert!((cara_vinadis(90.0, shadow) + (42.0 + 2.0 / 3.0)).abs() < 1e-9);
        assert!((cara_vinadis(270.0, shadow) - (42.0 + 2.0 / 3.0)).abs() < 1e-9);
        // Symmetric about the solstices; nothing at the equinoxes.
        assert!((cara_vinadis(60.0, shadow) - cara_vinadis(120.0, shadow)).abs() < 1e-9);
        assert!(cara_vinadis(0.0, shadow).abs() < 1e-9);
        assert!(cara_vinadis(180.0, shadow).abs() < 1e-9);
        assert!((cara_vinadis(-90.0, shadow) - cara_vinadis(270.0, shadow)).abs() < 1e-9);
        // Example 21: a cara of -72 is a day of 31 nadis 12 vinadis.
        assert!(
            (1_800.0 - cara_vinadis(142.5, Palabha::from_vyangulas(180)) - 1_872.0).abs() < 1e-9
        );
    }

    #[test]
    fn the_precession_is_linear_and_unreflected() {
        assert!(precession_degrees(3600).abs() < 1e-12);
        // Example 16 reflects Kali 5060's 24° 8' to 23° 52'; unreflected it stays.
        let degrees = precession_degrees(5060);
        assert!((degrees - (24.0 + 8.0 / 60.0)).abs() < 1.0 / 60.0);
    }

    #[test]
    fn the_almanac_day_lengths_at_the_month_starts() {
        // The Manonmani Vilasam almanac's day lengths on the first days of
        // 1998's months, against a shadow of 2¼ angulas, within 2 vinadis.
        let shadow = Palabha::from_vyangulas(135);
        let printed: [(u8, i64); 12] = [
            (0, 30 * 60 + 36),
            (1, 31 * 60 + 16),
            (2, 31 * 60 + 34),
            (3, 31 * 60 + 25),
            (4, 30 * 60 + 50),
            (5, 30 * 60 + 7),
            (6, 29 * 60 + 24),
            (7, 28 * 60 + 45),
            (8, 28 * 60 + 26),
            (9, 28 * 60 + 36),
            (10, 29 * 60 + 8),
            (11, 29 * 60 + 52),
        ];
        for (sign, vinadis) in printed {
            let day = day_length_vinadis(30.0 * f64::from(sign), 5099, shadow);
            let printed = f64::from(u16::try_from(vinadis).unwrap_or(0));
            assert!(
                (day - printed).abs() <= 2.0,
                "sign {sign}: {day} against {printed}"
            );
        }
    }
}
