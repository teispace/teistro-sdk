//! Panchaka, the muhurta yogas and the disha shool: what a day is said to
//! be, as opposed to what it measures.
//!
//! Each of these is an **interval** and not a flag. The corpus records
//! them as flags, and a flag is what an interval reduces to — "some
//! interval contains sunrise" — so reporting the interval is strictly
//! more information at no cost, since the limb spans it is cut from are
//! already computed.
//!
//! The muhurta yogas are the one limb the corpus cannot settle. Five of
//! them fire thirteen times on twelve of the fifty-five recorded days,
//! which cannot derive a table of seven varas by twenty-seven
//! nakshatras, and the recording engine's positives match no published
//! table under any rotation of either index
//! (`03-design/panchanga-day-conventions.md` §8). Two things the corpus
//! does settle and this module keeps:
//!
//! 1. **A vara-and-nakshatra yoga is read at sunrise.** Reading any
//!    nakshatra of the day into the Amrit Siddhi pairs fires it on four
//!    days the engine leaves clear; reading only the sunrise one agrees
//!    on all fifty-five.
//! 2. **Tripushkar's classical rule holds** on the corpus's one instance
//!    with no false positives.
//!
//! So the tables ship as data with their own confidence marks, chosen by
//! `panchanga.muhurta_tables`, and a yoga that fires says what made it.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{
    AgniVasa, Direction, Graha, MuhurtaYoga as Kind, Nakshatra, Panchaka, Rashi, Tithi, TithiClass,
    Vara,
};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::settings::{AmritKalam, PanchakaStart, Varjyam};

use crate::span::Span;

/// The tables the SDK ships, by the key `panchanga.muhurta_tables` names.
pub const CLASSICAL: &str = "CLASSICAL";

/// What made a muhurta yoga hold.
///
/// ```
/// # use teistro_core::catalogue::{Nakshatra, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_panchanga::span::Span;
/// #
/// # // A Sunday from 100.25 to 101.25: Hasta at sunrise, Chitra from 100.7,
/// # // and Shukla Saptami all the day.
/// # let day = Interval::literal(100.25, 101.25);
/// # let star = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// # let nakshatras: Vec<_> =
/// #     [star(Nakshatra::Hasta, 99.8, 100.7), star(Nakshatra::Chitra, 100.7, 101.7)].into_iter().flatten().collect();
/// # let tithis: Vec<_> = Span::new(Tithi::ShuklaSaptami, Interval::literal(100.0, 101.3), day).into_iter().collect();
///
/// use teistro_panchanga::omen::{CLASSICAL, yogas};
///
/// use teistro_panchanga::omen::YogaCause;
///
/// // Hasta on a Sunday makes Amrit Siddhi.
/// let found = yogas(CLASSICAL, Vara::Ravivara, &nakshatras, &tithis)?;
/// assert_eq!(found[0].because, YogaCause::VaraNakshatra { vara: Vara::Ravivara, nakshatra: Nakshatra::Hasta });
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "cause", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YogaCause {
    /// The vara and the nakshatra the Moon was in.
    VaraNakshatra {
        /// The day's vara.
        vara: Vara,
        /// The nakshatra that makes it.
        nakshatra: Nakshatra,
    },
    /// The vara, the tithi's class and the nakshatra's number of feet.
    VaraTithiNakshatra {
        /// The day's vara.
        vara: Vara,
        /// The tithi that makes it.
        tithi: Tithi,
        /// The nakshatra that makes it.
        nakshatra: Nakshatra,
    },
}

/// A muhurta yoga that held, and for how long.
///
/// ```
/// # use teistro_core::catalogue::{Nakshatra, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_panchanga::span::Span;
/// #
/// # // A Sunday from 100.25 to 101.25: Hasta at sunrise, Chitra from 100.7,
/// # // and Shukla Saptami all the day.
/// # let day = Interval::literal(100.25, 101.25);
/// # let star = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// # let nakshatras: Vec<_> =
/// #     [star(Nakshatra::Hasta, 99.8, 100.7), star(Nakshatra::Chitra, 100.7, 101.7)].into_iter().flatten().collect();
/// # let tithis: Vec<_> = Span::new(Tithi::ShuklaSaptami, Interval::literal(100.0, 101.3), day).into_iter().collect();
///
/// use teistro_panchanga::omen::{CLASSICAL, yogas};
///
/// use teistro_core::catalogue::MuhurtaYoga;
///
/// // It holds while the sunrise nakshatra does.
/// let found = yogas(CLASSICAL, Vara::Ravivara, &nakshatras, &tithis)?;
/// assert_eq!((found[0].yoga, found[0].at), (MuhurtaYoga::AmritSiddhi, Interval::literal(100.25, 100.7)));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PanchangaMuhurtaYoga"))]
pub struct MuhurtaYoga {
    /// Which yoga.
    pub yoga: Kind,
    /// While it held, clipped to the day.
    pub at: Interval,
    /// What made it, so a reader can see why.
    pub because: YogaCause,
}

/// What a day is said to be.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// # // Wednesday 25 November 2026.
/// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
///
/// use teistro_core::catalogue::Vara;
/// use teistro_panchanga::omen::disha_shool;
///
/// assert_eq!(day.omens.disha_shool, disha_shool(Vara::Budhavara));
/// // Every window is cut to the day.
/// assert!(day.omens.varjyam.iter().all(|span| span.inside.from.get() >= day.day.sunrise.get()));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Omens {
    /// Panchaka, while the Moon is in the last five nakshatras.
    pub panchaka: Vec<Span<Panchaka>>,
    /// The muhurta yogas that held.
    pub yogas: Vec<MuhurtaYoga>,
    /// The varjyam: each nakshatra's four ghatis of poison, the window
    /// itself in `whole` and the part inside the day in `inside`, placed
    /// by `panchanga.varjyam` (crux C165).
    pub varjyam: Vec<Span<Nakshatra>>,
    /// The amrita kalam: each nakshatra's four auspicious ghatis, placed
    /// by `panchanga.amrit_kalam` (crux C677), `whole` and `inside` as the
    /// varjyam's.
    pub amrit_kalam: Vec<Span<Nakshatra>>,
    /// The direction not to travel in, which is the vara's.
    pub disha_shool: Direction,
}

/// The direction a vara's travel is blocked in.
///
/// Monday and Saturday east, Tuesday and Wednesday north, Thursday
/// south, Friday and Sunday west — the corpus's own table, on all 55
/// days, and Muhurta Chintamani's (*Yatra Prakarana* v. 10; the 1906
/// Venkateshwar Press edition, p. 111), which gives the nakshatra's
/// beside it ([`nakshatra_shool`]).
///
/// ```
/// use teistro_core::catalogue::{Direction, Vara};
/// use teistro_panchanga::omen::disha_shool;
///
/// assert_eq!(disha_shool(Vara::Ravivara), Direction::West);
/// assert_eq!(disha_shool(Vara::Guruvara), Direction::South);
/// ```
#[must_use]
pub const fn disha_shool(vara: Vara) -> Direction {
    match vara {
        Vara::Somavara | Vara::Shanivara => Direction::East,
        Vara::Mangalavara | Vara::Budhavara => Direction::North,
        Vara::Guruvara => Direction::South,
        _ => Direction::West,
    }
}

/// The ghati, of a star sixty ghatis long, each nakshatra's varjyam
/// begins after, Ashwini first: Muhurta Chintamani's (Vivaha Prakarana
/// vv. 49 to 51), read off Mahidhara Sharma's print and its table and
/// checked against the Piyushadhara's quotation of Kashyapa.
const VARJYAM_MUHURTA_CHINTAMANI: [u8; 27] = [
    50, 24, 30, 40, 14, 21, 30, 20, 32, 30, 20, 18, 21, 20, 14, 14, 10, 14, 56, 24, 20, 10, 10, 18,
    16, 24, 30,
];

/// Raman's *Muhurtha* (ch. II, p. 15) as printed: Muhurta Chintamani's
/// but for Bharani 4, Uttara Phalguni 1, Mula 20, Purva Ashadha 20 and
/// Uttara Bhadrapada 30 (crux C165).
const VARJYAM_RAMAN: [u8; 27] = [
    50, 4, 30, 40, 14, 21, 30, 20, 32, 30, 20, 1, 21, 20, 14, 14, 10, 14, 20, 20, 20, 10, 10, 18,
    16, 30, 30,
];

/// How many ghatis a varjyam lasts, of a star sixty long (*kṛta*, v. 51).
const VARJYAM_GHATIS: f64 = 4.0;

/// The ghati a nakshatra's varjyam begins after under a table, of a star
/// sixty ghatis long, or `None` under `NONE`.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_core::settings::Varjyam;
/// use teistro_panchanga::omen::varjyam_start;
///
/// // Bharani, where Raman's print departs from Muhurta Chintamani.
/// assert_eq!(varjyam_start(Varjyam::MuhurtaChintamani, Nakshatra::Bharani), Some(24));
/// assert_eq!(varjyam_start(Varjyam::Raman, Nakshatra::Bharani), Some(4));
/// ```
#[must_use]
pub fn varjyam_start(table: Varjyam, nakshatra: Nakshatra) -> Option<u8> {
    let starts = match table {
        Varjyam::MuhurtaChintamani => &VARJYAM_MUHURTA_CHINTAMANI,
        Varjyam::Raman => &VARJYAM_RAMAN,
        // `NONE`, and any table a later version adds that this one
        // cannot know.
        _ => return None,
    };
    Nakshatra::ALL
        .iter()
        .zip(starts)
        .find_map(|(member, start)| (*member == nakshatra).then_some(*start))
}

/// The varjyam windows of a day, from its nakshatra spans.
///
/// Verse 51 scales the table to the star's own span ("multiplied by the
/// star's extent and divided by sixty they become the true ones"), and
/// the four ghatis are scaled with the start, as Ojha counts them (crux
/// C394), so a window is the sixtieths `start` to `start + 4` of the
/// star. A window is reported when any of it falls in the day; one
/// cannot fall outside its star, so clipping to the star's part of the
/// day clips it to the day.
///
/// ```
/// # use teistro_core::catalogue::{Nakshatra, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_panchanga::span::Span;
/// #
/// # // A Sunday from 100.25 to 101.25: Hasta at sunrise, Chitra from 100.7,
/// # // and Shukla Saptami all the day.
/// # let day = Interval::literal(100.25, 101.25);
/// # let star = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// # let nakshatras: Vec<_> =
/// #     [star(Nakshatra::Hasta, 99.8, 100.7), star(Nakshatra::Chitra, 100.7, 101.7)].into_iter().flatten().collect();
/// # let tithis: Vec<_> = Span::new(Tithi::ShuklaSaptami, Interval::literal(100.0, 101.3), day).into_iter().collect();
///
/// use teistro_core::settings::Varjyam;
/// use teistro_panchanga::omen::varjyam;
///
/// // Hasta's falls before sunrise; Chitra's is its 20th to 24th sixtieths.
/// let found = varjyam(&nakshatras, Varjyam::MuhurtaChintamani)?;
/// assert_eq!(found.len(), 1);
/// assert_eq!(found[0].member, Nakshatra::Chitra);
/// assert!((found[0].whole.from.get() - (100.7 + 20.0 / 60.0)).abs() < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// None in practice: a window is a fraction of an interval that exists.
pub fn varjyam(
    nakshatras: &[Span<Nakshatra>],
    table: Varjyam,
) -> Result<Vec<Span<Nakshatra>>, Error> {
    ghati_windows(nakshatras, |star| varjyam_start(table, star))
}

/// The ghati, of a star sixty ghatis long, each nakshatra's amrita kalam
/// begins after under Raman's table, Ashwini first: the table printed in
/// his translation of the *Praśna Mārga* (ch. 2 v. 21), the one the
/// published panchangas print (crux C677).
const AMRITA_RAMAN: [u8; 27] = [
    42, 48, 54, 52, 38, 35, 54, 44, 56, 54, 44, 42, 45, 44, 38, 38, 28, 38, 44, 48, 44, 34, 34, 42,
    40, 48, 54,
];

/// The ghati a nakshatra's amrita kalam begins after, of a star sixty
/// ghatis long: Raman's table, or under `PRASNA_MARGA` the end of the
/// varjyam `table` places (C678), or `None` under `NONE`, under a
/// varjyam of `NONE`, and where the varjyam ends the star and leaves no
/// four ghatis after it.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_core::settings::{AmritKalam, Varjyam};
/// use teistro_panchanga::omen::amrit_kalam_start;
///
/// let raman = amrit_kalam_start(AmritKalam::Raman, Varjyam::MuhurtaChintamani, Nakshatra::Revati);
/// assert_eq!(raman, Some(54));
/// let adjacent = amrit_kalam_start(AmritKalam::PrasnaMarga, Varjyam::MuhurtaChintamani, Nakshatra::Revati);
/// assert_eq!(adjacent, Some(34));
/// ```
#[must_use]
pub fn amrit_kalam_start(reading: AmritKalam, table: Varjyam, nakshatra: Nakshatra) -> Option<u8> {
    match reading {
        AmritKalam::Raman => Nakshatra::ALL
            .iter()
            .zip(&AMRITA_RAMAN)
            .find_map(|(member, start)| (*member == nakshatra).then_some(*start)),
        AmritKalam::PrasnaMarga => varjyam_start(table, nakshatra)
            .map(|visha| visha + 4)
            .filter(|start| f64::from(*start) + VARJYAM_GHATIS <= 60.0),
        // `NONE`, and any reading a later version adds that this one
        // cannot know.
        _ => None,
    }
}

/// The amrita kalam windows of a day, from its nakshatra spans: four
/// ghatis of each star from the reading's start, scaled to the star's
/// own span as the varjyam is (C679).
///
/// ```
/// # use teistro_core::catalogue::{Nakshatra, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_panchanga::span::Span;
/// #
/// # // A Sunday from 100.25 to 101.25: Hasta at sunrise, Chitra from 100.7,
/// # // and Shukla Saptami all the day.
/// # let day = Interval::literal(100.25, 101.25);
/// # let star = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// # let nakshatras: Vec<_> =
/// #     [star(Nakshatra::Hasta, 99.8, 100.7), star(Nakshatra::Chitra, 100.7, 101.7)].into_iter().flatten().collect();
/// # let tithis: Vec<_> = Span::new(Tithi::ShuklaSaptami, Interval::literal(100.0, 101.3), day).into_iter().collect();
///
/// use teistro_core::settings::{AmritKalam, Varjyam};
/// use teistro_panchanga::omen::amrit_kalam;
///
/// // Hasta's from its 45th sixtieth, scaled to its 0.9 days.
/// let found = amrit_kalam(&nakshatras, AmritKalam::Raman, Varjyam::MuhurtaChintamani)?;
/// assert_eq!(found.len(), 1);
/// assert!((found[0].whole.from.get() - (99.8 + 0.9 * 45.0 / 60.0)).abs() < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// None in practice: a window is a fraction of an interval that exists.
pub fn amrit_kalam(
    nakshatras: &[Span<Nakshatra>],
    reading: AmritKalam,
    table: Varjyam,
) -> Result<Vec<Span<Nakshatra>>, Error> {
    ghati_windows(nakshatras, |star| amrit_kalam_start(reading, table, star))
}

/// Each star's four ghatis from the ghati `start_of` gives it, of a star
/// sixty long, scaled to the star's span and kept when any of it falls
/// in the day: the varjyam's arithmetic and the amrita kalam's.
fn ghati_windows(
    nakshatras: &[Span<Nakshatra>],
    start_of: impl Fn(Nakshatra) -> Option<u8>,
) -> Result<Vec<Span<Nakshatra>>, Error> {
    let mut found = Vec::new();
    for span in nakshatras {
        let Some(start) = start_of(span.member) else {
            continue;
        };
        let start = f64::from(start);
        let whole = Interval::new(
            span.whole.at_fraction(start / 60.0)?,
            span.whole.at_fraction((start + VARJYAM_GHATIS) / 60.0)?,
        )?;
        found.extend(Span::new(span.member, whole, span.inside));
    }
    Ok(found)
}

/// The direction a nakshatra blocks travel in, if it blocks one: the
/// nakshatra shool.
///
/// Muhurta Chintamani, *Yatra Prakarana* v. 10, in the verse that gives
/// the vara's: न पूर्वदिशि शाक्रभे, not east in Jyeshtha (Indra's);
/// न चाजपदभे … यमदिशि, not south in Purva Bhadrapada (Ajapada's);
/// न पाशिदिशि धातृभे, not west in Rohini (Dhatri's, Brahma's); and
/// … अर्यमर्क्षे … न सौम्यककुभि, not north in Uttara Phalguni
/// (Aryaman's). The edition's Hindi glosses the last as Bharani, which
/// is Yama's star and not Aryaman's; the Sanskrit is followed (crux C402).
///
/// ```
/// use teistro_core::catalogue::{Direction, Nakshatra};
/// use teistro_panchanga::omen::nakshatra_shool;
///
/// assert_eq!(nakshatra_shool(Nakshatra::Jyeshtha), Some(Direction::East));
/// assert_eq!(nakshatra_shool(Nakshatra::UttaraPhalguni), Some(Direction::North));
/// assert_eq!(nakshatra_shool(Nakshatra::Ashwini), None);
/// ```
#[must_use]
pub const fn nakshatra_shool(nakshatra: Nakshatra) -> Option<Direction> {
    match nakshatra {
        Nakshatra::Jyeshtha => Some(Direction::East),
        Nakshatra::PurvaBhadrapada => Some(Direction::South),
        Nakshatra::Rohini => Some(Direction::West),
        Nakshatra::UttaraPhalguni => Some(Direction::North),
        _ => None,
    }
}

/// The nakshatra shool's spans over a day: while the Moon is in one of
/// the four nakshatras that block a direction, that direction.
///
/// ```
/// use teistro_core::catalogue::{Direction, Nakshatra};
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::omen::nakshatra_shools;
/// use teistro_panchanga::span::Span;
///
/// // Rohini blocks the west while the Moon is in it; Mrigashira nothing.
/// let day = Interval::literal(100.25, 101.25);
/// let star = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// let nakshatras: Vec<_> =
///     [star(Nakshatra::Rohini, 99.9, 100.6), star(Nakshatra::Mrigashira, 100.6, 101.6)].into_iter().flatten().collect();
/// let shools = nakshatra_shools(&nakshatras);
/// assert_eq!(shools.len(), 1);
/// assert_eq!((shools[0].member, shools[0].whole), (Direction::West, Interval::literal(99.9, 100.6)));
/// ```
#[must_use]
pub fn nakshatra_shools(nakshatras: &[Span<Nakshatra>]) -> Vec<Span<Direction>> {
    nakshatras
        .iter()
        .filter_map(|span| nakshatra_shool(span.member).map(|direction| span.map(|_| direction)))
        .collect()
}

/// The direction the Moon dwells in from a sign, the chandra vasa:
/// Aries, Leo and Sagittarius east; Taurus, Virgo and Capricorn south;
/// Gemini, Libra and Aquarius west; Cancer, Scorpio and Pisces north.
///
/// The directions of the signs Muhurta Chintamani's commentary gives
/// beside *Yatra Prakarana* v. 34 (the 1906 edition, p. 120: १।५।९ पूर्व,
/// २।६।१० दक्षिण, ३।७।११ पश्चिम, ४।८।१२ उत्तर), read for the sign the Moon
/// is in (crux C403). A sign's number from Aries, counted from zero,
/// taken modulo four, is the direction's place in that cycle.
///
/// ```
/// use teistro_core::catalogue::{Direction, Rashi};
/// use teistro_panchanga::omen::chandra_vasa;
///
/// assert_eq!(chandra_vasa(Rashi::Leo), Direction::East);
/// assert_eq!(chandra_vasa(Rashi::Capricorn), Direction::South);
/// assert_eq!(chandra_vasa(Rashi::Aquarius), Direction::West);
/// assert_eq!(chandra_vasa(Rashi::Pisces), Direction::North);
/// ```
#[must_use]
pub const fn chandra_vasa(sign: Rashi) -> Direction {
    match sign.id() % 4 {
        0 => Direction::East,
        1 => Direction::South,
        2 => Direction::West,
        _ => Direction::North,
    }
}

/// The natal Moon sign a Moon in `sign` is eighth from, the one whose
/// native has chandrashtama while the Moon is there.
///
/// Kalaprakasika (N. P. Subramania Iyer's translation, 1917, p. 51 and
/// p. 167) defines chandrashtama as the Moon's passage through the eighth
/// sign from the one it held at birth, and works one: a Moon born in
/// Aries makes the days the Moon spends in Scorpio. Counting inclusively,
/// the eighth sign is seven on, so the natal sign is seven back, which
/// is five on (crux C404).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_panchanga::omen::chandrashtama_of;
///
/// // Kalaprakasika's own example, read backwards.
/// assert_eq!(chandrashtama_of(Rashi::Scorpio), Rashi::Aries);
/// assert_eq!(chandrashtama_of(Rashi::Aries), Rashi::Virgo);
/// ```
#[must_use]
pub fn chandrashtama_of(sign: Rashi) -> Rashi {
    Rashi::from_id((sign.id() + 5) % 12).unwrap_or(sign)
}

/// The spans of a day over which each natal Moon sign has chandrashtama,
/// from the signs the Moon stood in: the member is the **natal** sign,
/// and the intervals are the Moon's own run through the eighth from it.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::omen::chandrashtama;
/// use teistro_panchanga::span::Span;
///
/// // The Moon in Scorpio all day: those born with it in Aries.
/// let day = Interval::literal(100.25, 101.25);
/// let signs: Vec<_> = Span::new(Rashi::Scorpio, Interval::literal(99.5, 101.8), day).into_iter().collect();
/// assert_eq!(chandrashtama(&signs)[0].member, Rashi::Aries);
/// ```
#[must_use]
pub fn chandrashtama(moon_signs: &[Span<Rashi>]) -> Vec<Span<Rashi>> {
    moon_signs
        .iter()
        .map(|span| span.map(chandrashtama_of))
        .collect()
}

/// The Moon's direction over a day, by the sign it stood in.
///
/// ```
/// use teistro_core::catalogue::{Direction, Rashi};
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::omen::chandra_vasas;
/// use teistro_panchanga::span::Span;
///
/// // Leo, then Virgo from 100.8: east, then south.
/// let day = Interval::literal(100.25, 101.25);
/// let sign = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// let signs: Vec<_> = [sign(Rashi::Leo, 98.5, 100.8), sign(Rashi::Virgo, 100.8, 103.1)].into_iter().flatten().collect();
/// let directions: Vec<_> = chandra_vasas(&signs).iter().map(|span| span.member).collect();
/// assert_eq!(directions, [Direction::East, Direction::South]);
/// ```
#[must_use]
pub fn chandra_vasas(moon_signs: &[Span<Rashi>]) -> Vec<Span<Direction>> {
    moon_signs
        .iter()
        .map(|span| span.map(chandra_vasa))
        .collect()
}

/// Where fire dwells for a homa, the agni vasa, from the tithi and the
/// vara: Muhurta Chintamani, Nakshatra-prakarana v. 36 (crux C748).
///
/// The verse adds one to the tithi, counted from Shukla Pratipada as 1 to
/// Amavasya as 30, adds the vara, Sunday as 1, and divides by four: a
/// remainder of three or nothing is the earth, where a homa brings ease;
/// one is the sky and two below the earth, where it costs life and
/// wealth. The commentary's worked case is a Thursday's Krishna Dvitiya,
/// 17 + 1 + 5 = 23, three over, on the earth. The verse is asked of a
/// kamya homa; the nitya and naimittika homas are done whatever it says.
///
/// ```
/// use teistro_core::catalogue::{AgniVasa, Tithi, Vara};
/// use teistro_panchanga::omen::agni_vasa;
///
/// // The commentary's own case.
/// assert_eq!(agni_vasa(Tithi::KrishnaDvitiya, Vara::Guruvara), AgniVasa::Prithvi);
/// // A Sunday's Shukla Pratipada: 1 + 1 + 1 = 3, the earth.
/// assert_eq!(agni_vasa(Tithi::ShuklaPratipada, Vara::Ravivara), AgniVasa::Prithvi);
/// // A Monday's: 4, nothing over, the earth; a Tuesday's: 5, one, the sky.
/// assert_eq!(agni_vasa(Tithi::ShuklaPratipada, Vara::Mangalavara), AgniVasa::Svarga);
/// assert!(!AgniVasa::Patala.attributes().favourable);
/// ```
#[must_use]
pub const fn agni_vasa(tithi: Tithi, vara: Vara) -> AgniVasa {
    // Both ids count from zero: the tithi's number is one on, and so is
    // the vara's, so the verse's sum is the ids and three.
    match (tithi.id() + vara.id() + 3) % 4 {
        1 => AgniVasa::Svarga,
        2 => AgniVasa::Patala,
        _ => AgniVasa::Prithvi,
    }
}

/// The agni vasa over a day, one span for each tithi it held, all read
/// with the day's own vara, which turns at sunrise.
///
/// ```
/// use teistro_core::catalogue::{AgniVasa, Tithi, Vara};
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::omen::agni_vasas;
/// use teistro_panchanga::span::Span;
///
/// // A Thursday that turns from Krishna Pratipada to Dvitiya at 100.6.
/// let day = Interval::literal(100.25, 101.25);
/// let tithi = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// let tithis: Vec<_> =
///     [tithi(Tithi::KrishnaPratipada, 99.6, 100.6), tithi(Tithi::KrishnaDvitiya, 100.6, 101.6)].into_iter().flatten().collect();
/// let vasas: Vec<_> = agni_vasas(&tithis, Vara::Guruvara).iter().map(|span| span.member).collect();
/// assert_eq!(vasas, [AgniVasa::Patala, AgniVasa::Prithvi]);
/// ```
#[must_use]
pub fn agni_vasas(tithis: &[Span<Tithi>], vara: Vara) -> Vec<Span<AgniVasa>> {
    tithis
        .iter()
        .map(|span| span.map(|tithi| agni_vasa(tithi, vara)))
        .collect()
}

/// Whose mouth a homa's oblation falls in, by the Moon's nakshatra counted
/// from the Sun's in threes: Muhurta Chintamani, Nakshatra-prakarana v. 35
/// (crux C751). The Sun's own star and the two after it give the Sun,
/// the next three Mercury, and so on to Ketu; the verse bars a malefic's
/// share, which the reader judges.
pub const HOMA_AHUTI: [Graha; 9] = [
    Graha::Sun,
    Graha::Mercury,
    Graha::Venus,
    Graha::Saturn,
    Graha::Moon,
    Graha::Mars,
    Graha::Jupiter,
    Graha::Rahu,
    Graha::Ketu,
];

/// The graha whose mouth the oblation falls in with the Sun in one
/// nakshatra and the Moon in another: the Moon's star counted from the
/// Sun's, the Sun's own the first, in threes ([`HOMA_AHUTI`]).
///
/// ```
/// use teistro_core::catalogue::{Graha, Nakshatra};
/// use teistro_panchanga::omen::ahuti;
///
/// // The Moon in the Sun's own star: the first three, the Sun's.
/// assert_eq!(ahuti(Nakshatra::Rohini, Nakshatra::Rohini), Graha::Sun);
/// // The Moon four stars on, the fourth: Mercury's three.
/// assert_eq!(ahuti(Nakshatra::Ashwini, Nakshatra::Rohini), Graha::Mercury);
/// // The star before the Sun's is the 27th, the last three, Ketu's.
/// assert_eq!(ahuti(Nakshatra::Bharani, Nakshatra::Ashwini), Graha::Ketu);
/// ```
#[must_use]
pub fn ahuti(sun: Nakshatra, moon: Nakshatra) -> Graha {
    let count = (moon.id() + 27 - sun.id()) % 27;
    HOMA_AHUTI
        .get(usize::from(count / 3))
        .copied()
        .unwrap_or(Graha::Ketu)
}

/// A day's homa ahuti: one span for each stretch the Moon's nakshatra and
/// the Sun's both held, its `whole` where the two stays overlap and its
/// `inside` the part in the day ([`ahuti`]).
///
/// ```
/// use teistro_core::catalogue::{Graha, Nakshatra};
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::Span;
/// use teistro_panchanga::omen::homa_ahuti;
///
/// let day = Interval::literal(100.25, 101.25);
/// let star = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// // The Moon changes star in the afternoon, the Sun stays in Rohini.
/// let moon: Vec<_> = [star(Nakshatra::Rohini, 99.9, 100.7), star(Nakshatra::Mrigashira, 100.7, 101.7)].into_iter().flatten().collect();
/// let sun: Vec<_> = star(Nakshatra::Rohini, 95.0, 108.0).into_iter().collect();
/// let shares = homa_ahuti(&moon, &sun, day);
/// assert_eq!(shares.iter().map(|s| s.member).collect::<Vec<_>>(), [Graha::Sun, Graha::Sun]);
/// assert_eq!(shares[1].whole, Interval::literal(100.7, 101.7));
/// ```
#[must_use]
pub fn homa_ahuti(
    moon: &[Span<Nakshatra>],
    sun: &[Span<Nakshatra>],
    window: Interval,
) -> Vec<Span<Graha>> {
    moon.iter()
        .flat_map(|m| {
            sun.iter().filter_map(move |s| {
                let whole = m.whole.clipped_to(s.whole)?;
                Span::new(ahuti(s.member, m.member), whole, window)
            })
        })
        .collect()
}

/// The directions Rahu walks over the daylight, one an eighth of it from
/// sunrise: *Jyotirnibandha* (Anandashrama 85, p. 337), Rahu-bhumi v. 1,
/// "इन्द्रवायुयमे रुद्रे तोयाग्निशशिराक्षसे । यामार्धमुदितो राहुर्भ्रमत्येव
/// दिगष्टके", that is Indra's east, Vayu's northwest, Yama's south,
/// Rudra's northeast, Varuna's west, Agni's southeast, the Moon's north
/// and the Rakshasa's southwest, half a yama each (crux C749).
pub const RAHU_WALK: [Direction; 8] = [
    Direction::East,
    Direction::Northwest,
    Direction::South,
    Direction::Northeast,
    Direction::West,
    Direction::Southeast,
    Direction::North,
    Direction::Southwest,
];

/// Rahu's direction over a day's daylight, the eighths of sunrise to
/// sunset in [`RAHU_WALK`]'s order; the verse names no night, so the
/// night has none, and a polar day or night with no daylight none at all.
///
/// ```
/// use teistro_core::catalogue::Direction;
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::omen::rahu_vasas;
///
/// // Twelve hours of daylight from 6 a.m.: an hour and a half each.
/// let (daylight, day) = (Interval::literal(100.25, 100.75), Interval::literal(100.25, 101.25));
/// let walk = rahu_vasas(daylight, day);
/// assert_eq!(walk.len(), 8);
/// assert_eq!((walk[0].member, walk[0].whole), (Direction::East, Interval::literal(100.25, 100.25 + 0.0625)));
/// assert_eq!(walk[7].member, Direction::Southwest);
/// assert!(rahu_vasas(Interval::literal(100.25, 100.25), day).is_empty());
/// ```
#[must_use]
pub fn rahu_vasas(daylight: Interval, window: Interval) -> Vec<Span<Direction>> {
    if daylight.is_empty() {
        return Vec::new();
    }
    daylight
        .divided(8)
        .zip(RAHU_WALK)
        .filter_map(|(part, direction)| Span::new(direction, part, window))
        .collect()
}

/// The panchaka a nakshatra carries, if it is one of the last five.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Panchaka};
/// use teistro_panchanga::omen::panchaka_of;
///
/// assert_eq!(panchaka_of(Nakshatra::Dhanishtha), Some(Panchaka::Mrityu));
/// assert_eq!(panchaka_of(Nakshatra::Revati), Some(Panchaka::Roga));
/// assert_eq!(panchaka_of(Nakshatra::Ashwini), None);
/// ```
#[must_use]
pub fn panchaka_of(nakshatra: Nakshatra) -> Option<Panchaka> {
    Panchaka::ALL
        .iter()
        .copied()
        .find(|panchaka| panchaka.attributes().nakshatra == nakshatra)
}

/// The panchaka spans of a day, from its nakshatra spans.
///
/// The window a panchaka runs in is the nakshatra's own, cut at the
/// Moon's entry into Aquarius where `start` says it begins there (crux
/// C158): only Dhanishtha straddles that entry, so `moon_signs` is asked
/// for the signs over a Dhanishtha span and nothing else. The corpus
/// cannot separate the two readings — they differ only in the first half
/// of Dhanishtha, and no recorded day has the Moon there — so the SDK
/// follows Raman's *Muhurtha* (ch. IV), the one text in hand, and names
/// the recording engine's whole nakshatra as the other.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Panchaka, Rashi};
/// use teistro_core::interval::Interval;
/// use teistro_core::settings::PanchakaStart;
/// use teistro_panchanga::omen::panchaka;
/// use teistro_panchanga::span::Span;
///
/// // Dhanishtha to 100.9, then Shatabhisha; the Moon enters Aquarius at 100.5.
/// let day = Interval::literal(100.25, 101.25);
/// let star = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// let nakshatras: Vec<_> =
///     [star(Nakshatra::Dhanishtha, 100.0, 100.9), star(Nakshatra::Shatabhisha, 100.9, 101.9)].into_iter().flatten().collect();
/// let signs = |_| {
///     let sign = |member, from, to| Span::new(member, Interval::literal(from, to), day);
///     Ok([sign(Rashi::Capricorn, 98.0, 100.5), sign(Rashi::Aquarius, 100.5, 102.8)].into_iter().flatten().collect())
/// };
///
/// // From the Moon's entry into Aquarius, as Raman has it.
/// let found = panchaka(&nakshatras, PanchakaStart::Aquarius, signs)?;
/// assert_eq!((found[0].member, found[0].whole), (Panchaka::Mrityu, Interval::literal(100.5, 100.9)));
/// assert_eq!(found[1].member, Panchaka::Agni);
/// // From Dhanishtha's start, the signs unasked.
/// let whole = panchaka(&nakshatras, PanchakaStart::Dhanishtha, |_| Ok(Vec::new()))?;
/// assert_eq!(whole[0].whole, Interval::literal(100.0, 100.9));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// Whatever `moon_signs` returns.
pub fn panchaka<F>(
    nakshatras: &[Span<Nakshatra>],
    start: PanchakaStart,
    mut moon_signs: F,
) -> Result<Vec<Span<Panchaka>>, Error>
where
    F: FnMut(Interval) -> Result<Vec<Span<Rashi>>, Error>,
{
    let mut found = Vec::new();
    for span in nakshatras {
        let Some(kind) = panchaka_of(span.member) else {
            continue;
        };
        if span.member != Nakshatra::Dhanishtha || start == PanchakaStart::Dhanishtha {
            found.push(span.map(|_| kind));
            continue;
        }
        for sign in moon_signs(span.inside)? {
            if sign.member != Rashi::Aquarius {
                continue;
            }
            let Some(whole) = span.whole.clipped_to(sign.whole) else {
                continue;
            };
            found.extend(Span::new(kind, whole, span.inside));
        }
    }
    Ok(found)
}

/// The panchaka an instant's four numbers leave, or `None` when the
/// remainder is a good one (crux C159).
///
/// Raman's *Muhurtha* (ch. III, p. 20) sums the tithi's number (from
/// Shukla Pratipada, 1 to 30), the vara's (Sunday 1), the nakshatra's
/// (from Ashwini) and the lagna's (from Aries) and divides by nine: 1
/// leaves mrityu, 2 agni, 4 raja, 6 chora and 8 roga, each member's
/// `remainder`; 3, 5, 7 and 0 are good. It is a clause of an **instant**,
/// since the lagna is one, and not the almanac's panchaka, which is the
/// Moon's run through the last nakshatras and carries the recording
/// engine's kind for each.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Panchaka, Rashi, Tithi, Vara};
/// use teistro_panchanga::omen::panchaka_remainder;
///
/// // 1 + 1 + 1 + 1 = 4 leaves raja; one sign on, 5 is good.
/// let raja = panchaka_remainder(Tithi::ShuklaPratipada, Vara::Ravivara, Nakshatra::Ashwini, Rashi::Aries);
/// assert_eq!(raja, Some(Panchaka::Raja));
/// assert_eq!(panchaka_remainder(Tithi::ShuklaPratipada, Vara::Ravivara, Nakshatra::Ashwini, Rashi::Taurus), None);
/// ```
#[must_use]
pub fn panchaka_remainder(
    tithi: Tithi,
    vara: Vara,
    nakshatra: Nakshatra,
    lagna: Rashi,
) -> Option<Panchaka> {
    let sum = u32::from(tithi.id())
        + u32::from(vara.attributes().weekday)
        + u32::from(nakshatra.id())
        + u32::from(lagna.id())
        + 4;
    let remainder = sum % 9;
    Panchaka::ALL
        .iter()
        .copied()
        .find(|panchaka| u32::from(panchaka.attributes().remainder) == remainder)
}

/// One row of a muhurta yoga table.
struct Row {
    /// Which yoga the row makes.
    yoga: Kind,
    /// The vara it needs.
    vara: Vara,
    /// The nakshatra it needs.
    nakshatra: Nakshatra,
}

/// The vara-and-nakshatra rows of the classical tables.
///
/// Marked traditional: the corpus cannot derive them and the recording
/// engine's own do not match any published set
/// (`03-design/panchanga-day-conventions.md` §8). They are here so that a
/// caller gets an answer with a stated provenance rather than none, and
/// `panchanga.muhurta_tables` is where another tradition's rows go.
const VARA_NAKSHATRA: [Row; 12] = [
    // Amrit Siddhi: one pair per vara.
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Ravivara,
        nakshatra: Nakshatra::Hasta,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Somavara,
        nakshatra: Nakshatra::Mrigashira,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Mangalavara,
        nakshatra: Nakshatra::Ashwini,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Budhavara,
        nakshatra: Nakshatra::Anuradha,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Guruvara,
        nakshatra: Nakshatra::Pushya,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Shukravara,
        nakshatra: Nakshatra::Revati,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Shanivara,
        nakshatra: Nakshatra::Rohini,
    },
    // Sarvartha Siddhi: a few nakshatras per vara. Only the rows the
    // published tables agree on are here; the rest await a citation.
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Ravivara,
        nakshatra: Nakshatra::Pushya,
    },
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Ravivara,
        nakshatra: Nakshatra::Hasta,
    },
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Somavara,
        nakshatra: Nakshatra::Shravana,
    },
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Budhavara,
        nakshatra: Nakshatra::Rohini,
    },
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Guruvara,
        nakshatra: Nakshatra::Revati,
    },
];

/// The nakshatras of three feet, which a Bhadra tithi on a Sunday,
/// Tuesday or Saturday makes Tripushkar in.
const THREE_FOOTED: [Nakshatra; 6] = [
    Nakshatra::Krittika,
    Nakshatra::Punarvasu,
    Nakshatra::UttaraPhalguni,
    Nakshatra::Vishakha,
    Nakshatra::UttaraAshadha,
    Nakshatra::PurvaBhadrapada,
];

/// The nakshatras of two feet, which the same combination makes
/// Dwipushkar in. No recorded day carries one, so the rule ships
/// untested and says so.
const TWO_FOOTED: [Nakshatra; 4] = [
    Nakshatra::Mrigashira,
    Nakshatra::Chitra,
    Nakshatra::Dhanishtha,
    Nakshatra::Anuradha,
];

/// The varas a pushkara yoga needs.
const PUSHKARA_VARAS: [Vara; 3] = [Vara::Ravivara, Vara::Mangalavara, Vara::Shanivara];

/// The muhurta yogas of a day.
///
/// ```
/// # use teistro_core::catalogue::{Nakshatra, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_panchanga::span::Span;
/// #
/// # // A Sunday from 100.25 to 101.25: Hasta at sunrise, Chitra from 100.7,
/// # // and Shukla Saptami all the day.
/// # let day = Interval::literal(100.25, 101.25);
/// # let star = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// # let nakshatras: Vec<_> =
/// #     [star(Nakshatra::Hasta, 99.8, 100.7), star(Nakshatra::Chitra, 100.7, 101.7)].into_iter().flatten().collect();
/// # let tithis: Vec<_> = Span::new(Tithi::ShuklaSaptami, Interval::literal(100.0, 101.3), day).into_iter().collect();
///
/// use teistro_panchanga::omen::{CLASSICAL, yogas};
///
/// use teistro_core::catalogue::MuhurtaYoga;
///
/// // Hasta on a Sunday: Amrit Siddhi and Sarvartha Siddhi.
/// let kinds: Vec<_> = yogas(CLASSICAL, Vara::Ravivara, &nakshatras, &tithis)?.iter().map(|y| y.yoga).collect();
/// assert_eq!(kinds, [MuhurtaYoga::AmritSiddhi, MuhurtaYoga::SarvarthaSiddhi]);
/// // A Bhadra tithi on a Sunday in two-footed Chitra: Dwipushkar.
/// let chitra = &nakshatras[1..];
/// let dvitiya: Vec<_> = Span::new(Tithi::ShuklaDvitiya, Interval::literal(100.7, 101.3), day).into_iter().collect();
/// let found = yogas(CLASSICAL, Vara::Ravivara, chitra, &dvitiya)?;
/// assert_eq!(found.iter().map(|y| y.yoga).collect::<Vec<_>>(), [MuhurtaYoga::Dwipushkar]);
/// assert_eq!(yogas("ACME", Vara::Ravivara, chitra, &dvitiya).unwrap_err().field(), Some("panchanga.muhurta_tables"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `UNSUPPORTED` for a table key the SDK does not ship, naming the ones
/// it does.
pub fn yogas(
    tables: &str,
    vara: Vara,
    nakshatras: &[Span<Nakshatra>],
    tithis: &[Span<Tithi>],
) -> Result<Vec<MuhurtaYoga>, Error> {
    if !tables.eq_ignore_ascii_case(CLASSICAL) {
        return Err(Error::unsupported(format!(
            "no muhurta yoga table `{tables}`; the SDK ships `{CLASSICAL}`"
        ))
        .with_field("panchanga.muhurta_tables"));
    }
    // Measured: a vara-and-nakshatra yoga is read at the nakshatra the
    // day opens in, not at any nakshatra of the day.
    let Some(at_sunrise) = nakshatras.first() else {
        return Ok(Vec::new());
    };
    let mut found: Vec<MuhurtaYoga> = VARA_NAKSHATRA
        .iter()
        .filter(|row| row.vara == vara && row.nakshatra == at_sunrise.member)
        .map(|row| MuhurtaYoga {
            yoga: row.yoga,
            at: at_sunrise.inside,
            because: YogaCause::VaraNakshatra {
                vara,
                nakshatra: at_sunrise.member,
            },
        })
        .collect();
    found.extend(pushkara(vara, at_sunrise, tithis));
    Ok(found)
}

/// Dwipushkar and Tripushkar: a Bhadra tithi on a Sunday, Tuesday or
/// Saturday in a two- or three-footed nakshatra.
fn pushkara(vara: Vara, at_sunrise: &Span<Nakshatra>, tithis: &[Span<Tithi>]) -> Vec<MuhurtaYoga> {
    if !PUSHKARA_VARAS.contains(&vara) {
        return Vec::new();
    }
    let Some(tithi) = tithis.first() else {
        return Vec::new();
    };
    if tithi.member.attributes().class != TithiClass::Bhadra {
        return Vec::new();
    }
    let yoga = if THREE_FOOTED.contains(&at_sunrise.member) {
        Kind::Tripushkar
    } else if TWO_FOOTED.contains(&at_sunrise.member) {
        Kind::Dwipushkar
    } else {
        return Vec::new();
    };
    // It holds while both the tithi and the nakshatra do.
    let Some(at) = tithi.inside.clipped_to(at_sunrise.inside) else {
        return Vec::new();
    };
    vec![MuhurtaYoga {
        yoga,
        at,
        because: YogaCause::VaraTithiNakshatra {
            vara,
            tithi: tithi.member,
            nakshatra: at_sunrise.member,
        },
    }]
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{
        CLASSICAL, RAHU_WALK, agni_vasa, amrit_kalam, amrit_kalam_start, chandra_vasa,
        chandra_vasas, chandrashtama, chandrashtama_of, disha_shool, nakshatra_shool,
        nakshatra_shools, panchaka, panchaka_of, panchaka_remainder, varjyam, varjyam_start, yogas,
    };
    use teistro_core::catalogue::{
        AgniVasa, Direction, MuhurtaYoga as Kind, Nakshatra, Paksha, Panchaka, Rashi, Tithi, Vara,
    };
    use teistro_core::error::Error;
    use teistro_core::interval::Interval;
    use teistro_core::settings::{AmritKalam, PanchakaStart, Varjyam};

    use crate::span::Span;

    fn window() -> Interval {
        Interval::literal(100.0, 101.0)
    }

    fn span<T>(member: T) -> Span<T> {
        Span::new(member, window(), window()).expect("it fills the day")
    }

    #[test]
    fn the_last_five_nakshatras_carry_a_panchaka_and_no_others() {
        assert_eq!(panchaka_of(Nakshatra::Dhanishtha), Some(Panchaka::Mrityu));
        assert_eq!(panchaka_of(Nakshatra::Shatabhisha), Some(Panchaka::Agni));
        assert_eq!(
            panchaka_of(Nakshatra::PurvaBhadrapada),
            Some(Panchaka::Raja)
        );
        assert_eq!(
            panchaka_of(Nakshatra::UttaraBhadrapada),
            Some(Panchaka::Chora)
        );
        assert_eq!(panchaka_of(Nakshatra::Revati), Some(Panchaka::Roga));
        assert_eq!(panchaka_of(Nakshatra::Shravana), None, "the one before");
        assert_eq!(panchaka_of(Nakshatra::Ashwini), None);
        assert_eq!(
            Nakshatra::ALL
                .iter()
                .filter(|n| panchaka_of(**n).is_some())
                .count(),
            5
        );
    }

    /// A Moon that is never asked about: every span it is given is not
    /// Dhanishtha, or the start is the whole nakshatra.
    fn unasked(_: Interval) -> Result<Vec<Span<Rashi>>, Error> {
        Err(Error::internal("the Moon's signs were asked for"))
    }

    #[test]
    fn a_panchaka_span_is_the_nakshatras_own() {
        let spans = [span(Nakshatra::Revati), span(Nakshatra::Ashwini)];
        let found = panchaka(&spans, PanchakaStart::Aquarius, unasked).unwrap();
        assert_eq!(found.len(), 1, "only Revati of the two");
        assert_eq!(found[0].member, Panchaka::Roga);
        assert_eq!(found[0].inside, spans[0].inside);
        assert!(
            panchaka(
                &[span(Nakshatra::Ashwini)],
                PanchakaStart::Aquarius,
                unasked
            )
            .unwrap()
            .is_empty(),
            "{:?}",
            panchaka(
                &[span(Nakshatra::Ashwini)],
                PanchakaStart::Aquarius,
                unasked
            )
            .unwrap()
        );
    }

    /// Dhanishtha over the whole test day, the Moon entering Aquarius
    /// half way through it.
    fn dhanishtha_across_the_entry() -> (Span<Nakshatra>, Vec<Span<Rashi>>) {
        let entry = 100.5;
        let signs = vec![
            Span::new(Rashi::Capricorn, Interval::literal(98.0, entry), window()).unwrap(),
            Span::new(Rashi::Aquarius, Interval::literal(entry, 102.5), window()).unwrap(),
        ];
        (span(Nakshatra::Dhanishtha), signs)
    }

    #[test]
    fn under_raman_panchaka_begins_where_the_moon_enters_aquarius() {
        let (dhanishtha, signs) = dhanishtha_across_the_entry();
        let found = panchaka(&[dhanishtha], PanchakaStart::Aquarius, |within| {
            assert_eq!(within, dhanishtha.inside, "asked over the nakshatra's day");
            Ok(signs.clone())
        })
        .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].member, Panchaka::Mrityu);
        assert_eq!(found[0].inside, Interval::literal(100.5, 101.0));
        assert_eq!(found[0].whole, Interval::literal(100.5, 101.0));
        assert!(!found[0].began_before(), "it began inside the day");
    }

    #[test]
    fn under_the_recording_engine_panchaka_is_the_whole_of_dhanishtha() {
        let (dhanishtha, _) = dhanishtha_across_the_entry();
        let found = panchaka(&[dhanishtha], PanchakaStart::Dhanishtha, unasked).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].inside, dhanishtha.inside);
    }

    #[test]
    fn a_dhanishtha_wholly_in_capricorn_carries_no_panchaka_under_raman() {
        let (dhanishtha, signs) = dhanishtha_across_the_entry();
        let found = panchaka(&[dhanishtha], PanchakaStart::Aquarius, |_| {
            Ok(signs[..1].to_vec())
        })
        .unwrap();
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn the_remainder_reproduces_ramans_worked_example() {
        // Ch. III, p. 20: Aslesha (9), the 13th tithi, Virgo rising (6),
        // a Sunday (1) make 29, which leaves 2: agni, "not favourable".
        assert_eq!(
            panchaka_remainder(
                Tithi::ShuklaTrayodashi,
                Vara::Ravivara,
                Nakshatra::Ashlesha,
                Rashi::Virgo
            ),
            Some(Panchaka::Agni)
        );
    }

    #[test]
    fn every_remainder_names_its_kind_or_none() {
        // Walk the lagna through all twelve signs from a sum of 1+1+1+1 = 4:
        // the remainders run 4, 5, ... 15 by nine.
        let expected = [
            Some(Panchaka::Raja),   // 4
            None,                   // 5
            Some(Panchaka::Chora),  // 6
            None,                   // 7
            Some(Panchaka::Roga),   // 8
            None,                   // 0
            Some(Panchaka::Mrityu), // 1
            Some(Panchaka::Agni),   // 2
            None,                   // 3
            Some(Panchaka::Raja),   // 4
            None,                   // 5
            Some(Panchaka::Chora),  // 6
        ];
        for (lagna, kind) in Rashi::ALL.iter().zip(expected) {
            assert_eq!(
                panchaka_remainder(
                    Tithi::ShuklaPratipada,
                    Vara::Ravivara,
                    Nakshatra::Ashwini,
                    *lagna
                ),
                kind,
                "{lagna:?}"
            );
        }
    }

    #[test]
    fn the_thirtieth_tithi_counts_as_thirty() {
        // Amavasya is 30, not 15 of the dark half: 30 + 1 + 1 + 1 = 33,
        // which leaves 6, chora; counted as 15 it would be 18, which
        // leaves 0 and is good.
        assert_eq!(
            panchaka_remainder(
                Tithi::Amavasya,
                Vara::Ravivara,
                Nakshatra::Ashwini,
                Rashi::Aries
            ),
            Some(Panchaka::Chora)
        );
    }

    #[test]
    fn the_disha_shool_is_the_varas_own() {
        assert_eq!(disha_shool(Vara::Somavara), Direction::East);
        assert_eq!(disha_shool(Vara::Shanivara), Direction::East);
        assert_eq!(disha_shool(Vara::Mangalavara), Direction::North);
        assert_eq!(disha_shool(Vara::Budhavara), Direction::North);
        assert_eq!(disha_shool(Vara::Guruvara), Direction::South);
        assert_eq!(disha_shool(Vara::Shukravara), Direction::West);
        assert_eq!(disha_shool(Vara::Ravivara), Direction::West);
    }

    #[test]
    fn four_nakshatras_block_one_direction_each() {
        // Yatra Prakarana v. 10: Jyeshtha east, Purva Bhadrapada south,
        // Rohini west, Uttara Phalguni north, and no other star.
        assert_eq!(nakshatra_shool(Nakshatra::Jyeshtha), Some(Direction::East));
        assert_eq!(
            nakshatra_shool(Nakshatra::PurvaBhadrapada),
            Some(Direction::South)
        );
        assert_eq!(nakshatra_shool(Nakshatra::Rohini), Some(Direction::West));
        assert_eq!(
            nakshatra_shool(Nakshatra::UttaraPhalguni),
            Some(Direction::North)
        );
        // The Hindi gloss's Bharani is not the verse's Aryaman.
        assert_eq!(nakshatra_shool(Nakshatra::Bharani), None);
        assert_eq!(
            Nakshatra::ALL
                .iter()
                .filter(|n| nakshatra_shool(**n).is_some())
                .count(),
            4
        );
        let day = [span(Nakshatra::Mrigashira), span(Nakshatra::Rohini)];
        let found = nakshatra_shools(&day);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].member, Direction::West);
        assert_eq!(found[0].whole, day[1].whole);
    }

    #[test]
    fn the_moon_dwells_east_south_west_north_by_its_signs_triplicity() {
        let expected = [
            (Rashi::Aries, Direction::East),
            (Rashi::Taurus, Direction::South),
            (Rashi::Gemini, Direction::West),
            (Rashi::Cancer, Direction::North),
            (Rashi::Leo, Direction::East),
            (Rashi::Virgo, Direction::South),
            (Rashi::Libra, Direction::West),
            (Rashi::Scorpio, Direction::North),
            (Rashi::Sagittarius, Direction::East),
            (Rashi::Capricorn, Direction::South),
            (Rashi::Aquarius, Direction::West),
            (Rashi::Pisces, Direction::North),
        ];
        for (sign, direction) in expected {
            assert_eq!(chandra_vasa(sign), direction, "{sign:?}");
        }
        let found = chandra_vasas(&[span(Rashi::Cancer)]);
        assert_eq!(found[0].member, Direction::North);
    }

    #[test]
    fn chandrashtama_is_the_eighth_sign_counted_inclusively() {
        // Kalaprakasika p. 51: a Moon born in Aries has chandrashtama
        // while the Moon is in Scorpio, the eighth sign from Aries.
        assert_eq!(chandrashtama_of(Rashi::Scorpio), Rashi::Aries);
        // Every sign is the eighth from exactly one natal sign, and
        // counting eight from that sign, inclusively, gives it back:
        // natal + 7 ≡ moon (mod 12). Not the rival reading "eight on",
        // which would make Aries's Moon trouble in Sagittarius.
        for moon in Rashi::ALL {
            let natal = chandrashtama_of(moon);
            assert_eq!((natal.id() + 7) % 12, moon.id(), "{moon:?}");
        }
        assert_ne!(chandrashtama_of(Rashi::Sagittarius), Rashi::Aries);
        let found = chandrashtama(&[span(Rashi::Libra)]);
        assert_eq!(
            found[0].member,
            Rashi::Pisces,
            "Libra is eighth from Pisces"
        );
        assert_eq!(found[0].whole, window());
    }

    #[test]
    fn a_yoga_is_read_at_the_nakshatra_the_day_opens_in() {
        // Wednesday and Anuradha make Amrit Siddhi.
        let found = yogas(
            CLASSICAL,
            Vara::Budhavara,
            &[span(Nakshatra::Anuradha), span(Nakshatra::Jyeshtha)],
            &[],
        )
        .expect("a table the SDK ships");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].yoga, Kind::AmritSiddhi);

        // The same nakshatra second in the day makes nothing, which is
        // what the corpus falsified the other reading with.
        let later = yogas(
            CLASSICAL,
            Vara::Budhavara,
            &[span(Nakshatra::Vishakha), span(Nakshatra::Anuradha)],
            &[],
        )
        .expect("a table the SDK ships");
        assert!(later.is_empty(), "{later:?}");
    }

    #[test]
    fn tripushkar_needs_all_three_of_its_conditions() {
        let nakshatra = [span(Nakshatra::PurvaBhadrapada)];
        let bhadra = [span(Tithi::ShuklaSaptami)];
        let found =
            yogas(CLASSICAL, Vara::Shanivara, &nakshatra, &bhadra).expect("a table the SDK ships");
        assert!(found.iter().any(|yoga| yoga.yoga == Kind::Tripushkar));

        // A vara that is not one of the three.
        assert!(
            !yogas(CLASSICAL, Vara::Somavara, &nakshatra, &bhadra)
                .expect("a table")
                .iter()
                .any(|yoga| yoga.yoga == Kind::Tripushkar)
        );
        // A tithi that is not Bhadra.
        let nanda = [span(Tithi::ShuklaPratipada)];
        assert!(
            !yogas(CLASSICAL, Vara::Shanivara, &nakshatra, &nanda)
                .expect("a table")
                .iter()
                .any(|yoga| yoga.yoga == Kind::Tripushkar)
        );
        // A nakshatra of neither two nor three feet.
        assert!(
            !yogas(
                CLASSICAL,
                Vara::Shanivara,
                &[span(Nakshatra::Magha)],
                &bhadra
            )
            .expect("a table")
            .iter()
            .any(|yoga| yoga.yoga == Kind::Tripushkar)
        );
        // Two feet make the other one.
        assert!(
            yogas(
                CLASSICAL,
                Vara::Shanivara,
                &[span(Nakshatra::Chitra)],
                &bhadra
            )
            .expect("a table")
            .iter()
            .any(|yoga| yoga.yoga == Kind::Dwipushkar)
        );
    }

    #[test]
    fn a_table_the_sdk_does_not_ship_is_refused_by_name() {
        let error = yogas("SOMEONE_ELSES", Vara::Somavara, &[], &[]).expect_err("no such table");
        assert!(error.message.contains("SOMEONE_ELSES"), "{error}");
        assert!(error.message.contains(CLASSICAL), "it names what it has");
    }

    #[test]
    fn a_day_with_no_nakshatra_span_has_no_yogas() {
        assert!(
            yogas(CLASSICAL, Vara::Somavara, &[], &[])
                .expect("a table")
                .is_empty(),
            "{:?}",
            yogas(CLASSICAL, Vara::Somavara, &[], &[]).expect("a table")
        );
    }

    #[test]
    fn raman_differs_from_muhurta_chintamani_in_five_stars() {
        let differing: Vec<(Nakshatra, Option<u8>, Option<u8>)> = Nakshatra::ALL
            .iter()
            .map(|n| {
                (
                    *n,
                    varjyam_start(Varjyam::MuhurtaChintamani, *n),
                    varjyam_start(Varjyam::Raman, *n),
                )
            })
            .filter(|(_, ours, his)| ours != his)
            .collect();
        assert_eq!(
            differing,
            [
                (Nakshatra::Bharani, Some(24), Some(4)),
                (Nakshatra::UttaraPhalguni, Some(18), Some(1)),
                (Nakshatra::Mula, Some(56), Some(20)),
                (Nakshatra::PurvaAshadha, Some(24), Some(20)),
                (Nakshatra::UttaraBhadrapada, Some(24), Some(30)),
            ]
        );
        assert_eq!(varjyam_start(Varjyam::None, Nakshatra::Ashwini), None);
    }

    #[test]
    fn a_varjyam_is_four_sixtieths_of_its_own_star() {
        // Ashwini, sixty hours long from day 100: its window is the
        // star's hours 50 to 54, whatever part of it the day holds.
        let star = Interval::literal(100.0, 102.5);
        let day = Interval::literal(102.1, 103.0);
        let spans = [Span::new(Nakshatra::Ashwini, star, day).unwrap()];
        let found = varjyam(&spans, Varjyam::MuhurtaChintamani).unwrap();
        assert_eq!(found.len(), 1);
        let [window] = found.as_slice() else {
            unreachable!()
        };
        assert_eq!(window.member, Nakshatra::Ashwini);
        assert!((window.whole.from.get() - (100.0 + 50.0 / 24.0)).abs() < 1e-9);
        assert!((window.whole.to.get() - (100.0 + 54.0 / 24.0)).abs() < 1e-9);
        assert_eq!(window.inside.from.get(), 102.1, "clipped to the day");
        assert!(window.began_before());
    }

    #[test]
    fn a_varjyam_outside_the_day_and_none_are_not_reported() {
        // Mula's window is its last four sixtieths but one; a day
        // holding only the star's first half holds none of it.
        let star = Interval::literal(100.0, 101.0);
        let spans = [Span::new(Nakshatra::Mula, star, Interval::literal(99.5, 100.5)).unwrap()];
        let outside = varjyam(&spans, Varjyam::MuhurtaChintamani).unwrap();
        assert!(outside.is_empty(), "{outside:?}");
        let none = varjyam(&[span(Nakshatra::Ashwini)], Varjyam::None).unwrap();
        assert!(none.is_empty(), "{none:?}");
    }

    /// Raman's starts for the seven stars of 15 to 21 June 2025 at Delhi,
    /// read off Drik Panchang's windows as sixtieths of each star (C677),
    /// and the commentary's reading four ghatis past the chosen varjyam.
    #[test]
    fn an_amrita_kalam_starts_where_its_reading_places_it() {
        let raman = |star| amrit_kalam_start(AmritKalam::Raman, Varjyam::MuhurtaChintamani, star);
        for (star, printed) in [
            (Nakshatra::Shravana, 34),
            (Nakshatra::Dhanishtha, 34),
            (Nakshatra::Shatabhisha, 42),
            (Nakshatra::PurvaBhadrapada, 40),
            (Nakshatra::UttaraBhadrapada, 48),
            (Nakshatra::Revati, 54),
            (Nakshatra::Ashwini, 42),
        ] {
            assert_eq!(raman(star), Some(printed), "{star:?}");
        }
        let adjacent = |table, star| amrit_kalam_start(AmritKalam::PrasnaMarga, table, star);
        assert_eq!(
            adjacent(Varjyam::MuhurtaChintamani, Nakshatra::Ashwini),
            Some(54)
        );
        // Mula's varjyam ends at 60 under the text's table: no four
        // ghatis follow it in the star; Raman's places it at 20.
        assert_eq!(adjacent(Varjyam::MuhurtaChintamani, Nakshatra::Mula), None);
        assert_eq!(adjacent(Varjyam::Raman, Nakshatra::Mula), Some(24));
        assert_eq!(adjacent(Varjyam::None, Nakshatra::Ashwini), None);
        assert_eq!(
            amrit_kalam_start(
                AmritKalam::None,
                Varjyam::MuhurtaChintamani,
                Nakshatra::Ashwini
            ),
            None
        );
    }

    #[test]
    fn an_amrita_kalam_is_four_sixtieths_of_a_star_of_any_length() {
        // Revati, 64 hours long from day 100: Raman's window is its
        // sixtieths 54 to 58, so 4.27 hours rather than four.
        let star = Interval::literal(100.0, 100.0 + 64.0 / 24.0);
        let spans = [Span::new(Nakshatra::Revati, star, star).unwrap()];
        let found = amrit_kalam(&spans, AmritKalam::Raman, Varjyam::MuhurtaChintamani).unwrap();
        let [window] = found.as_slice() else {
            unreachable!("{found:?}")
        };
        let hours = |at: f64| (at - 100.0) * 24.0;
        assert!((hours(window.whole.from.get()) - 64.0 * 54.0 / 60.0).abs() < 1e-6);
        assert!((hours(window.whole.to.get()) - 64.0 * 58.0 / 60.0).abs() < 1e-6);
        assert_eq!(
            amrit_kalam(&spans, AmritKalam::None, Varjyam::MuhurtaChintamani).unwrap(),
            []
        );
    }

    /// v. 36 by its own numbers: the tithi from Shukla Pratipada as 1 to
    /// Amavasya as 30, read from each tithi's paksha and number rather
    /// than its id, Sunday as 1, so a slip in the ids' arithmetic shows.
    #[test]
    fn the_agni_vasa_is_the_verse_s_remainder_for_every_tithi_and_vara() {
        for tithi in Tithi::ALL {
            let held = tithi.attributes();
            let t = u16::from(held.number)
                + if held.paksha == Paksha::Krishna {
                    15
                } else {
                    0
                };
            for vara in Vara::ALL {
                let sum = t + 1 + u16::from(vara.attributes().weekday) + 1;
                let expected = match sum % 4 {
                    0 | 3 => AgniVasa::Prithvi,
                    1 => AgniVasa::Svarga,
                    _ => AgniVasa::Patala,
                };
                assert_eq!(agni_vasa(tithi, vara), expected, "{tithi:?} {vara:?}");
            }
        }
        // Each place comes round: 30 tithis by 7 varas reach all three.
        assert!(AgniVasa::ALL.iter().all(|place| {
            Tithi::ALL
                .iter()
                .any(|tithi| agni_vasa(*tithi, Vara::Ravivara) == *place)
        }));
    }

    /// The verse's eight directions are the eight, each once.
    #[test]
    fn rahu_walks_every_direction_once() {
        let mut walked = RAHU_WALK.to_vec();
        walked.sort_by_key(|direction| direction.id());
        assert_eq!(walked, Direction::ALL.to_vec());
    }
}
