//! The *Vakyakarana*'s true Sun and true Moon (I.4b–14a), and the
//! almanac makers' correction to the Moon, the *manyadi* (the edition's
//! introduction, p. xvi), all in whole arithmetic: an [`Arc`] counts
//! quarter-seconds, so a day read as degrees (I.4b) is the moment's
//! [`PARTS_PER_DAY`] unchanged.
//!
//! - **The Sun** (I.4b–5a): the days, nadis and vinadis since the Mesha
//!   sankranti read as degrees, minutes and seconds, less the minutes of
//!   the mnemonics *Bhupajna* … for each ten days ([`SUN_DEDUCTIONS`]),
//!   by the text's interpolation rule (I.6–8): [`true_sun`].
//! - **The Moon** (I.9–14a): the Kali days less 16,00,984, divided out by
//!   12,372, then 3,031, then 248; the quotients times their dhruvas, plus
//!   the epoch's, plus the remainder's *candravakya* ([`CANDRA_VAKYAS`]),
//!   is the uncorrected Moon, and the quotients' vinadis times the day's
//!   motion beyond 13° 11′ correct it: [`true_moon`].
//! - **The manyadi**: the Moon's place at the true sunrise of a place in
//!   the Chola country, with a bija, which the almanacs add though the
//!   text does not ([`MANYADI_MINUTES`], [`almanac_moon`]).

use super::{KaliMoment, PARTS_PER_DAY};

/// Quarter-seconds in a degree.
pub const ARC_PER_DEGREE: i64 = 14_400;

/// Quarter-seconds in a minute of arc.
const ARC_PER_MINUTE: i64 = 240;

/// Quarter-seconds in a sign.
const ARC_PER_SIGN: i64 = 30 * ARC_PER_DEGREE;

/// Quarter-seconds in the circle.
pub const ARC_PER_CIRCLE: i64 = 360 * ARC_PER_DEGREE;

/// A sidereal longitude of the *Vakyakarana*, in quarter-seconds of arc
/// from the first point of Mesha, within the circle.
///
/// ```
/// use teistro_siddhanta::vakya::Arc;
///
/// let moon = Arc::from_rashi(4, 11, 17, 32);
/// assert_eq!(moon.rashi(), (4, 11, 17, 32));
/// assert!((moon.degrees() - 131.292_222).abs() < 1e-6);
/// assert_eq!(Arc::from_rashi(12, 0, 0, 0), Arc::from_rashi(0, 0, 0, 0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arc {
    quarter_seconds: i64,
}

impl Arc {
    /// The arc of so many quarter-seconds, brought within the circle.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::{ARC_PER_CIRCLE, Arc};
    ///
    /// assert_eq!(Arc::from_quarter_seconds(-1).quarter_seconds(), ARC_PER_CIRCLE - 1);
    /// ```
    #[must_use]
    pub const fn from_quarter_seconds(quarter_seconds: i64) -> Arc {
        Arc {
            quarter_seconds: quarter_seconds.rem_euclid(ARC_PER_CIRCLE),
        }
    }

    /// The arc of signs, degrees, minutes and seconds, as the text writes
    /// a place.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::Arc;
    ///
    /// assert_eq!(Arc::from_rashi(0, 12, 3, 0).quarter_seconds(), (12 * 60 + 3) * 240);
    /// ```
    #[must_use]
    pub const fn from_rashi(signs: i64, degrees: i64, minutes: i64, seconds: i64) -> Arc {
        Arc::from_quarter_seconds(
            ((signs * 30 + degrees) * 60 + minutes) * ARC_PER_MINUTE + seconds * 4,
        )
    }

    /// The quarter-seconds from the first point of Mesha.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::Arc;
    ///
    /// assert_eq!(Arc::from_rashi(0, 0, 1, 0).quarter_seconds(), 240);
    /// ```
    #[must_use]
    pub const fn quarter_seconds(self) -> i64 {
        self.quarter_seconds
    }

    /// The signs, degrees, minutes and whole seconds, the quarters cast
    /// out.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::Arc;
    ///
    /// assert_eq!(Arc::from_quarter_seconds(14_400 * 31 + 3).rashi(), (1, 1, 0, 0));
    /// ```
    #[must_use]
    pub const fn rashi(self) -> (i64, i64, i64, i64) {
        let seconds = self.quarter_seconds / 4;
        let minutes = seconds / 60;
        let degrees = minutes / 60;
        (degrees / 30, degrees % 30, minutes % 60, seconds % 60)
    }

    /// The arc in degrees.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::Arc;
    ///
    /// assert_eq!(Arc::from_rashi(6, 0, 0, 0).degrees(), 180.0);
    /// ```
    #[must_use]
    pub fn degrees(self) -> f64 {
        // Below 5,184,000, so exact as an i32.
        f64::from(i32::try_from(self.quarter_seconds).unwrap_or(i32::MAX)) / 14_400.0
    }

    /// The arc forward from `from` to this one, within the circle.
    ///
    /// ```
    /// use teistro_siddhanta::vakya::Arc;
    ///
    /// let (a, b) = (Arc::from_rashi(11, 29, 0, 0), Arc::from_rashi(0, 1, 0, 0));
    /// assert_eq!(b.since(a), Arc::from_rashi(0, 2, 0, 0));
    /// ```
    #[must_use]
    pub const fn since(self, from: Arc) -> Arc {
        Arc::from_quarter_seconds(self.quarter_seconds - from.quarter_seconds)
    }

    /// This arc and another.
    const fn plus(self, other: Arc) -> Arc {
        Arc::from_quarter_seconds(self.quarter_seconds + other.quarter_seconds)
    }
}

/// `numerator / denominator` to the nearest whole, a half rounded up,
/// for a positive denominator: the text's rounding of a proportion.
pub(super) const fn rounded(numerator: i64, denominator: i64) -> i64 {
    (2 * numerator + denominator).div_euclid(2 * denominator)
}

/// The minutes of arc deducted from the days since the Mesha sankranti,
/// read as degrees, to give the true Sun, for 0, 10, 20 … 370 days (I.5A,
/// the mnemonics beginning *Bhupajna*; the first, at the sankranti, is
/// none): the Sun's equation of centre and its mean motion's shortfall
/// from a degree a day, together.
///
/// ```
/// use teistro_siddhanta::vakya::SUN_DEDUCTIONS;
///
/// // Greatest at 200 days, 6° 56′ (416′), the apogee's quarter past.
/// let (at, most) = SUN_DEDUCTIONS.iter().enumerate().max_by_key(|&(_, m)| *m).ok_or("a table")?;
/// assert_eq!((at, *most), (20, 416));
/// # Ok::<(), &str>(())
/// ```
pub const SUN_DEDUCTIONS: [u16; 38] = [
    0, 14, 32, 54, 78, 105, 133, 163, 194, 224, 254, 284, 311, 335, 358, 376, 391, 403, 411, 415,
    416, 412, 406, 398, 386, 374, 361, 347, 334, 322, 311, 303, 297, 295, 296, 301, 309, 322,
];

// A day read as a degree (I.4b) is the same count of parts.
const _: () = assert!(ARC_PER_DEGREE == PARTS_PER_DAY);

/// The parts of a day in the Sun table's ten-day steps.
const SUN_STEP: i64 = 10 * PARTS_PER_DAY;

/// The minutes the Sun table gives at a step, none past its end.
fn sun_deduction_at(step: i64) -> i64 {
    usize::try_from(step)
        .ok()
        .and_then(|step| SUN_DEDUCTIONS.get(step))
        .map_or(0, |&minutes| i64::from(minutes))
}

/// The *Vakyakarana*'s true Sun at a moment (I.4b–5a): the time since the
/// year's Mesha sankranti read as an arc, a day a degree, less the
/// [`SUN_DEDUCTIONS`] interpolated by the text's rule (I.6–8).
///
/// ```
/// use teistro_siddhanta::vakya::{self, KaliMoment, PARTS_PER_DAY};
///
/// // Example 2: at the end of Kali day 18,44,004, 180 days 19 nadis 41
/// // vinadis after the Mesha sankranti, the true Sun is 5s 23° 28′ 33″.
/// let sun = vakya::true_sun(KaliMoment::from_parts(1_844_004 * PARTS_PER_DAY));
/// assert_eq!(sun.rashi(), (5, 23, 28, 33));
/// ```
#[must_use]
pub fn true_sun(moment: KaliMoment) -> Arc {
    let since = moment.parts() - super::mesha_sankranti(super::year_at(moment)).parts();
    let step = since.div_euclid(SUN_STEP);
    let (last, next) = (sun_deduction_at(step), sun_deduction_at(step + 1));
    let within = since - step * SUN_STEP;
    let deduction =
        last * ARC_PER_MINUTE + rounded((next - last) * ARC_PER_MINUTE * within, SUN_STEP);
    // A day is a degree: PARTS_PER_DAY parts are ARC_PER_DEGREE quarter-seconds.
    Arc::from_quarter_seconds(since - deduction)
}

/// The Moon's places on the days after a khanda's beginning, from the
/// Moon at its apogee, days 1 to 248: the 248 *candravakyas* ascribed to
/// Vararuci, beginning *girnah sreyah* (0s 12° 3′), in signs, degrees and
/// minutes, a whole circle cast out of each.
///
/// Transcribed from the edition's Appendix II (pp. 125–134), where each
/// sentence is printed beside its value. Day 148, *samprataṃ raviḥ*, is
/// printed 4 26 17; the sentence reads 4s 26° 27′ by the *katapayadi*
/// count, and the edition's rule for checking the vakyas (p. 134: a vakya
/// and the one 248 less its number make the last, 0s 27° 44′) holds with
/// 27 and fails with 17, so it ships as 27 (crux C1290). Every other pair
/// of the 124 the rule pairs holds as printed.
///
/// ```
/// use teistro_siddhanta::vakya::CANDRA_VAKYAS;
///
/// // girnah sreyah: gi 3, (r)na 0, (s)re 2, ya 1, read backwards, 12° 3′.
/// assert_eq!(CANDRA_VAKYAS[0], (0, 12, 3));
/// // bhavet sukham, the last: the 248 days' motion, 27° 44′.
/// assert_eq!(CANDRA_VAKYAS[247], (0, 27, 44));
/// ```
pub const CANDRA_VAKYAS: [(u8, u8, u8); 248] = [
    // p. 125, from day 1.
    (0, 12, 3),
    (0, 24, 9),
    (1, 6, 22),
    (1, 18, 44),
    (2, 1, 19),
    (2, 14, 9),
    (2, 27, 13),
    (3, 10, 33),
    (3, 24, 9),
    (4, 7, 58),
    (4, 21, 58),
    (5, 6, 8),
    (5, 20, 25),
    (6, 4, 44),
    (6, 19, 2),
    (7, 3, 15),
    (7, 17, 22),
    (8, 1, 17),
    (8, 15, 1),
    (8, 28, 29),
    (9, 11, 42),
    (9, 24, 40),
    (10, 7, 23),
    // p. 126, from day 24.
    (10, 19, 52),
    (11, 2, 10),
    (11, 14, 19),
    (11, 26, 24),
    (0, 8, 26),
    (0, 20, 30),
    (1, 2, 38),
    (1, 14, 55),
    (1, 27, 23),
    (2, 10, 4),
    (2, 23, 0),
    (3, 6, 12),
    (3, 19, 39),
    (4, 3, 21),
    (4, 17, 15),
    (5, 1, 20),
    (5, 15, 33),
    (5, 29, 51),
    (6, 14, 10),
    (6, 28, 27),
    (7, 12, 37),
    (7, 26, 39),
    (8, 10, 30),
    (8, 24, 7),
    (9, 7, 29),
    (9, 20, 35),
    (10, 3, 26),
    (10, 16, 2),
    // p. 127, from day 52.
    (10, 28, 26),
    (11, 10, 40),
    (11, 22, 46),
    (0, 4, 49),
    (0, 16, 52),
    (0, 28, 58),
    (1, 11, 10),
    (1, 23, 31),
    (2, 6, 5),
    (2, 18, 52),
    (3, 1, 55),
    (3, 15, 14),
    (3, 28, 47),
    (4, 12, 35),
    (4, 26, 34),
    (5, 10, 44),
    (5, 24, 59),
    (6, 9, 18),
    (6, 23, 36),
    (7, 7, 51),
    (7, 21, 58),
    (8, 5, 55),
    (8, 19, 40),
    (9, 3, 10),
    (9, 16, 25),
    (9, 29, 24),
    (10, 12, 8),
    (10, 24, 39),
    // p. 128, from day 80.
    (11, 6, 58),
    (11, 19, 8),
    (0, 1, 13),
    (0, 13, 15),
    (0, 25, 19),
    (1, 7, 27),
    (1, 19, 43),
    (2, 2, 10),
    (2, 14, 49),
    (2, 27, 43),
    (3, 10, 53),
    (3, 24, 18),
    (4, 7, 58),
    (4, 21, 52),
    (5, 5, 56),
    (5, 20, 8),
    (6, 4, 26),
    (6, 18, 45),
    (7, 3, 2),
    (7, 17, 13),
    (8, 1, 17),
    (8, 15, 8),
    (8, 28, 47),
    (9, 12, 10),
    (9, 25, 18),
    (10, 8, 11),
    (10, 20, 48),
    (11, 3, 14),
    // p. 129, from day 108.
    (11, 15, 28),
    (11, 27, 36),
    (0, 9, 39),
    (0, 21, 41),
    (1, 3, 46),
    (1, 15, 58),
    (1, 28, 18),
    (2, 10, 50),
    (2, 23, 36),
    (3, 6, 37),
    (3, 19, 54),
    (4, 3, 26),
    (4, 17, 12),
    (5, 1, 11),
    (5, 15, 19),
    (5, 29, 34),
    (6, 13, 52),
    (6, 28, 10),
    (7, 12, 25),
    (7, 26, 33),
    (8, 10, 32),
    (8, 24, 18),
    (9, 7, 50),
    (9, 21, 7),
    (10, 4, 8),
    // p. 130, from day 133.
    (10, 16, 54),
    (10, 29, 26),
    (11, 11, 46),
    (11, 23, 58),
    (0, 6, 3),
    (0, 18, 5),
    (1, 0, 8),
    (1, 12, 16),
    (1, 24, 30),
    (2, 6, 56),
    (2, 19, 33),
    (3, 2, 26),
    (3, 15, 34),
    (3, 28, 57),
    (4, 12, 36),
    (4, 26, 27),
    (5, 10, 31),
    (5, 24, 42),
    (6, 8, 59),
    (6, 23, 18),
    (7, 7, 36),
    (7, 21, 48),
    (8, 5, 52),
    (8, 19, 46),
    (9, 3, 26),
    // p. 131, from day 158.
    (9, 16, 51),
    (10, 0, 1),
    (10, 12, 55),
    (10, 25, 34),
    (11, 8, 1),
    (11, 20, 17),
    (0, 2, 25),
    (0, 14, 29),
    (0, 26, 31),
    (1, 8, 36),
    (1, 20, 46),
    (2, 3, 5),
    (2, 15, 36),
    (2, 28, 20),
    (3, 11, 19),
    (3, 24, 34),
    (4, 8, 4),
    (4, 21, 49),
    (5, 5, 46),
    (5, 19, 53),
    (6, 4, 8),
    (6, 18, 26),
    (7, 2, 45),
    (7, 17, 0),
    (8, 1, 10),
    // p. 132, from day 183.
    (8, 15, 9),
    (8, 28, 57),
    (9, 12, 30),
    (9, 25, 49),
    (10, 8, 52),
    (10, 21, 39),
    (11, 4, 13),
    (11, 16, 34),
    (11, 28, 46),
    (0, 10, 52),
    (0, 22, 55),
    (1, 4, 58),
    (1, 17, 4),
    (1, 29, 18),
    (2, 11, 42),
    (2, 24, 18),
    (3, 7, 9),
    (3, 20, 15),
    (4, 3, 37),
    (4, 17, 14),
    (5, 1, 5),
    (5, 15, 7),
    (5, 29, 17),
    (6, 13, 34),
    (6, 27, 53),
    // p. 133, from day 208.
    (7, 12, 11),
    (7, 26, 24),
    (8, 10, 29),
    (8, 24, 23),
    (9, 8, 5),
    (9, 21, 32),
    (10, 4, 44),
    (10, 17, 40),
    (11, 0, 21),
    (11, 12, 49),
    (11, 25, 6),
    (0, 7, 14),
    (0, 19, 18),
    (1, 1, 20),
    (1, 13, 25),
    (1, 25, 34),
    (2, 7, 52),
    (2, 20, 21),
    (3, 3, 4),
    (3, 16, 2),
    (3, 29, 15),
    (4, 12, 43),
    (4, 26, 27),
    (5, 10, 22),
    (5, 24, 29),
    (6, 8, 42),
    (6, 23, 0),
    // p. 134, from day 235.
    (7, 7, 19),
    (7, 21, 36),
    (8, 5, 46),
    (8, 19, 46),
    (9, 3, 35),
    (9, 17, 11),
    (10, 0, 31),
    (10, 13, 35),
    (10, 26, 25),
    (11, 9, 0),
    (11, 21, 22),
    (0, 3, 35),
    (0, 15, 41),
    (0, 27, 44),
];

/// The Kali days to the Moon's epoch, the day its khandas count from
/// (I.9).
pub const MOON_EPOCH_DAYS: i64 = 1_600_984;

/// The three khandas, longest first, each in days with the Moon's motion
/// over it, a whole number of circles cast out (I.9–11): 12,372 days and
/// 9s 27° 48′ 10″, 3,031 days and 11s 7° 31′ 1″, 248 days and 0s 27° 44′ 6″.
pub const KHANDAS: [(i64, Arc); 3] = [
    (12_372, Arc::from_rashi(9, 27, 48, 10)),
    (3_031, Arc::from_rashi(11, 7, 31, 1)),
    (248, Arc::from_rashi(0, 27, 44, 6)),
];

/// The Moon at the epoch (I.11): 7s 2° 0′ 7″.
pub const MOON_EPOCH_DHRUVA: Arc = Arc::from_rashi(7, 2, 0, 7);

/// The daily motion the correction of I.12–14a measures from: 13° 11′.
const MOTION_BASE: Arc = Arc::from_rashi(0, 13, 11, 0);

/// A candravakya's place, the zeroth being the khanda's start.
fn vakya_arc(day: i64) -> Arc {
    let Some(&(signs, degrees, minutes)) = usize::try_from(day - 1)
        .ok()
        .and_then(|index| CANDRA_VAKYAS.get(index))
    else {
        return Arc::from_quarter_seconds(0);
    };
    Arc::from_rashi(signs.into(), degrees.into(), minutes.into(), 0)
}

/// The Moon's place from the *Vakyakarana*'s tables after so many Kali
/// days, before the almanacs' [`MANYADI_MINUTES`]: the khandas' dhruvas
/// and the remainder's candravakya (I.9–11), corrected by the vinadis of
/// I.12–14a (thirty-two for each 248-day khanda less eight for each of
/// 3,031) times the day's motion beyond 13° 11′.
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// // Example 3: Kali day 18,44,004 gives quotients 19, 2 and 7 and the
/// // 154th vakya, the uncorrected Moon 4s 11° 14′ 1″, and 208 vinadis
/// // at a motion of 14° 12′ add 211″: 4s 11° 17′ 32″.
/// assert_eq!(vakya::true_moon(1_844_004).rashi(), (4, 11, 17, 32));
/// ```
#[must_use]
pub fn true_moon(kali_days: i64) -> Arc {
    let [(long, long_arc), (middle, middle_arc), (short, short_arc)] = KHANDAS;
    let days = kali_days - MOON_EPOCH_DAYS;
    let (first, rest) = (days.div_euclid(long), days.rem_euclid(long));
    let (second, rest) = (rest / middle, rest % middle);
    let (third, vakya) = (rest / short, rest % short);
    let dhruva = Arc::from_quarter_seconds(
        MOON_EPOCH_DHRUVA.quarter_seconds()
            + first * long_arc.quarter_seconds()
            + second * middle_arc.quarter_seconds()
            + third * short_arc.quarter_seconds(),
    );
    let uncorrected = dhruva.plus(vakya_arc(vakya));
    // The day's motion: this vakya less the one before, the last of the
    // previous khanda before the first.
    let before = if vakya == 0 {
        vakya_arc(short - 1).since(vakya_arc(short))
    } else {
        vakya_arc(vakya - 1)
    };
    let motion = vakya_arc(vakya).since(before);
    let vinadis = 32 * third - 8 * second;
    // Degrees of excess motion times vinadis are seconds of arc.
    let excess = motion.quarter_seconds() - MOTION_BASE.quarter_seconds();
    Arc::from_quarter_seconds(
        uncorrected.quarter_seconds() + rounded(excess * vinadis, ARC_PER_DEGREE / 4),
    )
}

/// The *manyadi*, the minutes the vakya almanacs add to the Moon when the
/// Sun begins each sign, Mesha first: 21′ at Mesha, 15′ at Vrishabha, 10′
/// at Mithuna, and on to 26′ at Mina (the edition's introduction, p. xvi,
/// n. 1, which lists them from Vrishabha). The introduction reads them as
/// the bhujantara, half the cara and the desantara of a place in the Chola
/// country with a bija of 26′, put in by an almanac maker, probably
/// Munampannai Krishna Josyar, after about 1885; the text has none.
///
/// ```
/// use teistro_siddhanta::vakya::MANYADI_MINUTES;
///
/// // Least at Kataka's beginning, greatest at Makara's.
/// assert_eq!((MANYADI_MINUTES[3], MANYADI_MINUTES[9]), (7, 30));
/// ```
pub const MANYADI_MINUTES: [u8; 12] = [21, 15, 10, 7, 8, 11, 17, 23, 28, 30, 29, 26];

/// The manyadi for the Sun at a place: the sign's value carried toward
/// the next sign's by the Sun's progress through the sign, by the text's
/// rule for every table (I.6–8, n. 6).
///
/// ```
/// use teistro_siddhanta::vakya::{self, Arc};
///
/// assert_eq!(vakya::manyadi(Arc::from_rashi(9, 0, 0, 0)), Arc::from_rashi(0, 0, 30, 0));
/// // Half-way through Mina, half-way from 26′ to Mesha's 21′.
/// assert_eq!(vakya::manyadi(Arc::from_rashi(11, 15, 0, 0)), Arc::from_rashi(0, 0, 23, 30));
/// ```
#[must_use]
pub fn manyadi(sun: Arc) -> Arc {
    let sign = sun.quarter_seconds() / ARC_PER_SIGN;
    let minutes = |sign: i64| {
        usize::try_from(sign.rem_euclid(12))
            .ok()
            .and_then(|index| MANYADI_MINUTES.get(index))
            .map_or(0, |&minutes| i64::from(minutes))
    };
    let (last, next) = (minutes(sign), minutes(sign + 1));
    let within = sun.quarter_seconds() - sign * ARC_PER_SIGN;
    Arc::from_quarter_seconds(
        last * ARC_PER_MINUTE + rounded((next - last) * ARC_PER_MINUTE * within, ARC_PER_SIGN),
    )
}

/// The Moon a vakya almanac prints after so many Kali days, the moment
/// that day's sunrise: the [`true_moon`] with the [`manyadi`] for the
/// [`true_sun`] of that moment.
///
/// ```
/// use teistro_siddhanta::vakya;
///
/// // Example 3's day: the Sun in Kanya at 23° 28′, so Kanya's 11′ and
/// // 23/30 of the 6′ to Tula's 17′, 15′ 41″, are added.
/// let moon = vakya::almanac_moon(1_844_004);
/// assert_eq!(moon.since(vakya::true_moon(1_844_004)).rashi(), (0, 0, 15, 41));
/// ```
#[must_use]
pub fn almanac_moon(kali_days: i64) -> Arc {
    let sun = true_sun(KaliMoment::from_parts(kali_days * PARTS_PER_DAY));
    true_moon(kali_days).plus(manyadi(sun))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(
    clippy::indexing_slicing,
    reason = "the tests index the 248 vakyas by numbers the loops bound"
)]
mod tests {
    use super::*;

    fn arc((signs, degrees, minutes): (u8, u8, u8)) -> Arc {
        Arc::from_rashi(signs.into(), degrees.into(), minutes.into(), 0)
    }

    #[test]
    fn the_vakyas_check_by_the_editions_rule() {
        // p. 134: the n-th vakya and the (248 - n)-th make the last, and
        // six signs and half the last is the 124th.
        let last = arc(CANDRA_VAKYAS[247]);
        for n in 1..248 {
            let pair = arc(CANDRA_VAKYAS[n - 1]).plus(arc(CANDRA_VAKYAS[247 - n]));
            assert_eq!(pair, last, "vakyas {n} and {}", 248 - n);
        }
        assert_eq!(
            arc(CANDRA_VAKYAS[123]),
            Arc::from_rashi(6, 0, 0, 0).plus(Arc::from_quarter_seconds(last.quarter_seconds() / 2))
        );
        // And the last is the shortest khanda's motion, to the minute.
        assert_eq!(
            last.since(KHANDAS[2].1).quarter_seconds(),
            ARC_PER_CIRCLE - 24
        );
    }

    #[test]
    fn every_day_moves_the_moon_as_the_moon_moves() {
        // A day's motion lies between 11° 50′ and 15° 20′, so no vakya
        // was read a sign or ten degrees wrong.
        let mut before = Arc::from_quarter_seconds(0);
        for (day, vakya) in CANDRA_VAKYAS.iter().enumerate() {
            let motion = arc(*vakya).since(before);
            assert!(
                (Arc::from_rashi(0, 11, 50, 0)..=Arc::from_rashi(0, 15, 20, 0)).contains(&motion),
                "day {}: {:?}",
                day + 1,
                motion.rashi()
            );
            before = arc(*vakya);
        }
    }

    #[test]
    fn the_moon_runs_on_across_a_khanda() {
        // The last day of a 248-day khanda and the first of the next are
        // a day's motion apart, the correction stepping with the quotient.
        let start = MOON_EPOCH_DAYS + 248;
        let step = true_moon(start).since(true_moon(start - 1)).degrees();
        assert!((11.5..15.5).contains(&step), "{step}");
        assert_eq!(true_moon(MOON_EPOCH_DAYS), MOON_EPOCH_DHRUVA);
        // Before the epoch the quotients count back.
        let back = true_moon(MOON_EPOCH_DAYS)
            .since(true_moon(MOON_EPOCH_DAYS - 1))
            .degrees();
        assert!((11.5..15.5).contains(&back), "{back}");
    }

    #[test]
    fn example_2_the_sun() {
        let sun = true_sun(KaliMoment::from_parts(1_844_004 * PARTS_PER_DAY));
        assert_eq!(sun.rashi(), (5, 23, 28, 33));
        // At the Mesha sankranti itself the Sun is at Mesha's first point.
        let mesha = super::super::mesha_sankranti(5048);
        assert_eq!(true_sun(mesha), Arc::from_quarter_seconds(0));
    }

    #[test]
    fn the_sun_meets_the_sankranti_vakyas_within_two_minutes() {
        // Two tables of the text, the sankranti vakyas and the Sun's
        // deductions, place the Sun at each sign's first point.
        for sign in 0..12_u8 {
            let moment = super::super::sankranti(5099, sign).unwrap_or(KaliMoment::from_parts(0));
            let off = true_sun(moment)
                .since(Arc::from_rashi(i64::from(sign), 0, 0, 0))
                .quarter_seconds();
            let off = if off > ARC_PER_CIRCLE / 2 {
                off - ARC_PER_CIRCLE
            } else {
                off
            };
            assert!(off.abs() <= 2 * ARC_PER_MINUTE, "sign {sign}: {off}");
        }
    }

    #[test]
    fn rounding_takes_a_half_up() {
        assert_eq!(rounded(7, 2), 4);
        assert_eq!(rounded(-7, 2), -3);
        assert_eq!(rounded(1, 3), 0);
    }
}
