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
    AgniVasa, AshubhaYoga, BhadraVasa, DayMarker, Direction, Graha, Karana, MuhurtaYoga as Kind,
    Nakshatra, Panchaka, Rashi, ShivaVasa, Tithi, TithiClass, Vara,
};
use teistro_core::envelope::Convention;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::{
    AmritKalam, PanchakaStart, Panchanga as PanchangaSettings, ShivaVasaReading, Varjyam,
};

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

/// Where Bhadra dwells while the Moon stands in `sign`: Muhurta
/// Chintamani, Shubhashubha-prakarana v. 45, "कुम्भकर्कद्वये मर्त्ये
/// स्वर्गेऽब्जेऽजात्त्रयेऽलिगे । स्त्रीधनुर्जूकनक्रेऽधो भद्रा तत्रैव
/// तत्फलम्" (crux C820).
///
/// The Moon (*abja*) in Aquarius and Pisces, Cancer and Leo puts her among
/// men; in the three from Aries and in Scorpio, in heaven; in Virgo,
/// Sagittarius, Libra and Capricorn, below. "Her result is there": the
/// *Piyushadhara* reads the verse as the exception to Bhadra's censure,
/// so she harms the earth's rites only while she dwells on it, and
/// quotes two more verses with the same signs.
///
/// ```
/// use teistro_core::catalogue::{BhadraVasa, Rashi};
/// use teistro_panchanga::omen::bhadra_vasa;
///
/// assert_eq!(bhadra_vasa(Rashi::Cancer), BhadraVasa::Martya);
/// assert_eq!(bhadra_vasa(Rashi::Scorpio), BhadraVasa::Svarga);
/// assert_eq!(bhadra_vasa(Rashi::Capricorn), BhadraVasa::Patala);
/// // Only the earth's is harmful.
/// assert!(BhadraVasa::Martya.attributes().harmful);
/// assert!(!BhadraVasa::Svarga.attributes().harmful);
/// ```
#[must_use]
pub const fn bhadra_vasa(sign: Rashi) -> BhadraVasa {
    match sign {
        Rashi::Cancer | Rashi::Leo | Rashi::Aquarius | Rashi::Pisces => BhadraVasa::Martya,
        Rashi::Aries | Rashi::Taurus | Rashi::Gemini | Rashi::Scorpio => BhadraVasa::Svarga,
        _ => BhadraVasa::Patala,
    }
}

/// Where Bhadra dwells over a day: one span for each stretch the vishti
/// karana and a sign of the Moon's both held, its `whole` where the two
/// overlap and its `inside` the part in the day ([`bhadra_vasa`]). A
/// Bhadra the Moon changes sign in is two spans, one a loka; a day with
/// no vishti has none.
///
/// ```
/// use teistro_core::catalogue::{BhadraVasa, Karana, Rashi};
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::omen::bhadra_vasas;
/// use teistro_panchanga::span::Span;
///
/// // Vishti from 100.4 to 100.9; the Moon leaves Gemini for Cancer at 100.6.
/// let day = Interval::literal(100.25, 101.25);
/// let karana = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// let sign = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// let karanas: Vec<_> = [karana(Karana::Vanija, 99.9, 100.4), karana(Karana::Vishti, 100.4, 100.9)].into_iter().flatten().collect();
/// let signs: Vec<_> = [sign(Rashi::Gemini, 98.4, 100.6), sign(Rashi::Cancer, 100.6, 102.9)].into_iter().flatten().collect();
///
/// let lokas = bhadra_vasas(&karanas, &signs, day);
/// assert_eq!(lokas.iter().map(|s| s.member).collect::<Vec<_>>(), [BhadraVasa::Svarga, BhadraVasa::Martya]);
/// assert_eq!(lokas[1].whole, Interval::literal(100.6, 100.9));
/// ```
#[must_use]
pub fn bhadra_vasas(
    karanas: &[Span<Karana>],
    moon_signs: &[Span<Rashi>],
    window: Interval,
) -> Vec<Span<BhadraVasa>> {
    karanas
        .iter()
        .filter(|karana| karana.member == Karana::Vishti)
        .flat_map(|vishti| {
            moon_signs.iter().filter_map(move |sign| {
                let whole = vishti.whole.clipped_to(sign.whole)?;
                Span::new(bhadra_vasa(sign.member), whole, window)
            })
        })
        .collect()
}

/// The stars that end at a junction of a water and a fire sign, Revati,
/// Ashlesha and Jyeshtha, and those that begin at one, Ashwini, Magha and
/// Mula: together the six ganda mula stars.
const JUNCTION_ENDS: [Nakshatra; 3] = [Nakshatra::Revati, Nakshatra::Ashlesha, Nakshatra::Jyeshtha];
const JUNCTION_BEGINS: [Nakshatra; 3] = [Nakshatra::Ashwini, Nakshatra::Magha, Nakshatra::Mula];

/// How far gandanta reaches on each side of a junction, in days: two
/// ghatis, a ghati a sixtieth of a day (Muhurta Chintamani,
/// Vivaha-prakarana v. 41).
pub const GANDANTA_REACH_DAYS: f64 = 2.0 / 60.0;

/// The junction a star touches, if it is one of the six: the instant
/// its span ends for Revati, Ashlesha and Jyeshtha, begins for Ashwini,
/// Magha and Mula.
fn junction(star: &Span<Nakshatra>) -> Option<JulianDay<Utc>> {
    if JUNCTION_ENDS.contains(&star.member) {
        Some(star.whole.to)
    } else if JUNCTION_BEGINS.contains(&star.member) {
        Some(star.whole.from)
    } else {
        None
    }
}

/// The day's markers from the Moon alone (`03-design/day-markers.md`
/// §3), in the order they begin, each over its whole stretch clipped to
/// `window`:
///
/// - [`DayMarker::GandaMula`] over each of the six junction stars the
///   day held, the whole star as the almanacs mark it (crux C1051);
/// - [`DayMarker::Gandanta`] over two ghatis each side of each junction
///   the day's stars touch, one span a junction, so the same span on
///   each day it reaches (Muhurta Chintamani, Vivaha-prakarana v. 41);
/// - [`DayMarker::Vinchudo`] over each stretch the Moon is in Scorpio, a
///   Gujarati printed reading (crux C1051).
///
/// ```
/// use teistro_core::catalogue::{DayMarker, Nakshatra, Rashi};
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::omen::{GANDANTA_REACH_DAYS, day_markers};
/// use teistro_panchanga::span::Span;
///
/// // A day from 100.25 to 101.25: Revati until 100.5, then Ashwini, the
/// // Moon in Pisces then Aries.
/// let day = Interval::literal(100.25, 101.25);
/// let stars: Vec<_> = [
///     Span::new(Nakshatra::Revati, Interval::literal(99.5, 100.5), day),
///     Span::new(Nakshatra::Ashwini, Interval::literal(100.5, 101.5), day),
/// ]
/// .into_iter()
/// .flatten()
/// .collect();
/// let signs: Vec<_> = [
///     Span::new(Rashi::Pisces, Interval::literal(98.0, 100.5), day),
///     Span::new(Rashi::Aries, Interval::literal(100.5, 102.8), day),
/// ]
/// .into_iter()
/// .flatten()
/// .collect();
/// let markers = day_markers(&stars, &signs, day);
/// let kinds: Vec<_> = markers.iter().map(|m| m.member).collect();
/// assert_eq!(kinds, [DayMarker::GandaMula, DayMarker::Gandanta, DayMarker::GandaMula]);
/// // One gandanta, two ghatis each side of the junction at 100.5.
/// let gandanta = markers[1].whole;
/// assert_eq!((gandanta.from.get(), gandanta.to.get()), (100.5 - GANDANTA_REACH_DAYS, 100.5 + GANDANTA_REACH_DAYS));
/// ```
#[must_use]
pub fn day_markers(
    nakshatras: &[Span<Nakshatra>],
    moon_signs: &[Span<Rashi>],
    window: Interval,
) -> Vec<Span<DayMarker>> {
    let mut found: Vec<Span<DayMarker>> = nakshatras
        .iter()
        .filter(|star| junction(star).is_some())
        .filter_map(|star| Span::new(DayMarker::GandaMula, star.whole, window))
        .collect();
    let mut junctions: Vec<JulianDay<Utc>> = nakshatras.iter().filter_map(junction).collect();
    junctions.sort_by(|a, b| a.get().total_cmp(&b.get()));
    // Revati's end and Ashwini's start are one instant when the day holds
    // both.
    junctions.dedup_by(|a, b| (a.get() - b.get()).abs() < 1e-9);
    found.extend(junctions.into_iter().filter_map(|at| {
        let reach = Interval::new(
            at.plus_days(-GANDANTA_REACH_DAYS).ok()?,
            at.plus_days(GANDANTA_REACH_DAYS).ok()?,
        )
        .ok()?;
        Span::new(DayMarker::Gandanta, reach, window)
    }));
    found.extend(
        moon_signs
            .iter()
            .filter(|sign| sign.member == Rashi::Scorpio)
            .filter_map(|sign| Span::new(DayMarker::Vinchudo, sign.whole, window)),
    );
    // Stable, so markers beginning together keep the order above.
    found.sort_by(|a, b| a.whole.from.get().total_cmp(&b.whole.from.get()));
    found
}

/// Where Shiva dwells on a tithi, as the almanacs print it: the tithi
/// *t*, 1 from Shukla Pratipada to 30 at Amavasya, as (2*t* + 5) mod 7,
/// the member whose `remainder` that is. **A printed convention**: no
/// text read gives it (crux C750); the Surya Panchanga's table (VS 2082,
/// p. 60) follows it in all thirty tithis.
///
/// ```
/// use teistro_core::catalogue::{ShivaVasa, Tithi};
/// use teistro_panchanga::omen::shiva_vasa;
///
/// // Shukla Pratipada: 2 + 5 = 7, nothing over, the cremation ground.
/// assert_eq!(shiva_vasa(Tithi::ShuklaPratipada), ShivaVasa::Shmashana);
/// // Shukla Panchami: 15, one over, Kailasa; Amavasya: 65, two, beside Gauri.
/// assert_eq!(shiva_vasa(Tithi::ShuklaPanchami), ShivaVasa::Kailasa);
/// assert_eq!(shiva_vasa(Tithi::Amavasya), ShivaVasa::Gauri);
/// ```
#[must_use]
pub fn shiva_vasa(tithi: Tithi) -> ShivaVasa {
    // The id counts from nought, so the tithi's number is one on and
    // 2t + 5 is twice the id and seven, which leaves what twice the id
    // leaves.
    let remainder = (2 * tithi.id()) % 7;
    ShivaVasa::ALL
        .iter()
        .copied()
        .find(|member| u16::from(member.attributes().remainder) == remainder)
        .unwrap_or(ShivaVasa::Shmashana)
}

/// The Shiva vasa over a day under `reading`, one span for each tithi it
/// held; none under `NONE`.
///
/// ```
/// use teistro_core::catalogue::{ShivaVasa, Tithi};
/// use teistro_core::interval::Interval;
/// use teistro_core::settings::ShivaVasaReading;
/// use teistro_panchanga::omen::shiva_vasas;
/// use teistro_panchanga::span::Span;
///
/// let day = Interval::literal(100.25, 101.25);
/// let tithis: Vec<_> = Span::new(Tithi::ShuklaDvitiya, Interval::literal(100.0, 101.1), day).into_iter().collect();
/// assert_eq!(shiva_vasas(&tithis, ShivaVasaReading::Printed)[0].member, ShivaVasa::Gauri);
/// assert!(shiva_vasas(&tithis, ShivaVasaReading::None).is_empty());
/// ```
#[must_use]
pub fn shiva_vasas(tithis: &[Span<Tithi>], reading: ShivaVasaReading) -> Vec<Span<ShivaVasa>> {
    match reading {
        ShivaVasaReading::None => Vec::new(),
        _ => tithis.iter().map(|span| span.map(shiva_vasa)).collect(),
    }
}

/// The name a panchaka beginning on `vara` is printed with, or `None` for
/// Wednesday and Thursday, which the almanacs leave unnamed. **A printed
/// convention** (crux C821): Sunday Roga, Monday Raja, Tuesday Agni,
/// Friday Chora, Saturday Mrityu. Muhurta Chintamani's weekday list
/// (Vivaha-prakarana v. 72) is the bana's and differs.
///
/// ```
/// use teistro_core::catalogue::{Panchaka, Vara};
/// use teistro_panchanga::omen::panchaka_by_vara;
///
/// assert_eq!(panchaka_by_vara(Vara::Ravivara), Some(Panchaka::Roga));
/// assert_eq!(panchaka_by_vara(Vara::Shanivara), Some(Panchaka::Mrityu));
/// assert_eq!(panchaka_by_vara(Vara::Budhavara), None);
/// ```
#[must_use]
pub const fn panchaka_by_vara(vara: Vara) -> Option<Panchaka> {
    match vara {
        Vara::Ravivara => Some(Panchaka::Roga),
        Vara::Somavara => Some(Panchaka::Raja),
        Vara::Mangalavara => Some(Panchaka::Agni),
        Vara::Shukravara => Some(Panchaka::Chora),
        Vara::Shanivara => Some(Panchaka::Mrityu),
        _ => None,
    }
}

/// A printed convention: a reading the almanacs print and no text read
/// gives, chosen by its knob, and named in the provenance of every
/// answer whose settings apply it (`03-design/printed-conventions.md`).
///
/// ```
/// use teistro_panchanga::omen::PRINTED_CONVENTIONS;
///
/// assert!(PRINTED_CONVENTIONS.iter().any(|c| c.knob == "panchanga.shiva_vasa"));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PrintedConvention {
    /// The knob that chooses it, by its full key.
    pub knob: &'static str,
    /// The knob's value in a day's settings, by its key.
    pub value: fn(&PanchangaSettings) -> &'static str,
    /// The crux that records the prints and the texts searched.
    pub crux: &'static str,
    /// Where it is printed.
    pub printed: &'static str,
}

/// Every printed convention the SDK ships, in the order an answer's
/// provenance names them.
pub const PRINTED_CONVENTIONS: [PrintedConvention; 2] = [
    PrintedConvention {
        knob: "panchanga.panchaka_name",
        value: |settings| settings.panchaka_name.key(),
        crux: "C821",
        printed: "the panchaka named by the vara it begins on, as Indian almanacs print it",
    },
    PrintedConvention {
        knob: "panchanga.shiva_vasa",
        value: |settings| settings.shiva_vasa.key(),
        crux: "C750",
        printed: "Shiva's abode by the tithi, as the Surya Panchanga (VS 2082, p. 60) prints it",
    },
];

/// The printed conventions a day's settings apply, as provenance entries:
/// each knob not set to `NONE`, with its value and why it is flagged.
///
/// ```
/// use teistro_core::settings::{DEFAULT_PROFILE, PanchakaName, Profile, SettingsPatch};
/// use teistro_panchanga::omen::applied_conventions;
///
/// let resolved = Profile::shipped(DEFAULT_PROFILE).ok_or("shipped")?.resolve(&SettingsPatch::default()).map_err(|d| format!("{d:?}"))?;
/// let mut panchanga = resolved.settings.panchanga.clone();
/// assert_eq!(applied_conventions(&panchanga).len(), 2);
/// panchanga.panchaka_name = PanchakaName::None;
/// assert_eq!(applied_conventions(&panchanga)[0].knob, "panchanga.shiva_vasa");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn applied_conventions(settings: &PanchangaSettings) -> Vec<Convention> {
    PRINTED_CONVENTIONS
        .iter()
        .filter_map(|convention| {
            let value = (convention.value)(settings);
            (value != "NONE").then(|| Convention {
                knob: convention.knob.to_owned(),
                value: value.to_owned(),
                reason: format!(
                    "a printed convention with no text behind it: {} ({})",
                    convention.printed, convention.crux
                ),
            })
        })
        .collect()
}

/// The tithis v. 8 of Muhurta Chintamani's Shubhashubha-prakarana makes
/// inauspicious on a weekday, as their numbers in either paksha:
/// *sūryeśa-pañcāgni-rasāṣṭa-nandā* the dagdha, *vedāṅga-saptāśvi-gajāṅka-
/// śailāḥ* the visha and *sūryāṅga-saptoraga-go-digīśāḥ* the hutashana,
/// Sunday to Saturday, as the commentary reads each word as a number
/// (crux C760).
const fn tithi_rows(vara: Vara) -> [(AshubhaYoga, u8); 3] {
    let [dagdha, visha, hutashana] = match vara {
        Vara::Ravivara => [12, 4, 12],
        Vara::Somavara => [11, 6, 6],
        Vara::Mangalavara => [5, 7, 7],
        Vara::Budhavara => [3, 2, 8],
        Vara::Guruvara => [6, 8, 9],
        Vara::Shukravara => [8, 9, 10],
        _ => [9, 7, 11],
    };
    [
        (AshubhaYoga::Dagdha, dagdha),
        (AshubhaYoga::Visha, visha),
        (AshubhaYoga::Hutashana, hutashana),
    ]
}

/// The inauspicious yogas a tithi makes on a weekday, by v. 8 (C760): a
/// Sunday's Dvadashi is both dagdha and hutashana, and most pairs are
/// neither. The tithi is read by its number in the paksha, which the
/// commentary's Guru says holds in both.
///
/// ```
/// use teistro_core::catalogue::{AshubhaYoga, Tithi, Vara};
/// use teistro_panchanga::omen::tithi_yogas;
///
/// // The commentary's first case, a Sunday's Dvadashi, either paksha.
/// assert_eq!(tithi_yogas(Tithi::KrishnaDwadashi, Vara::Ravivara), [AshubhaYoga::Dagdha, AshubhaYoga::Hutashana]);
/// // A Wednesday's Dvitiya is visha, its Tritiya dagdha.
/// assert_eq!(tithi_yogas(Tithi::ShuklaDvitiya, Vara::Budhavara), [AshubhaYoga::Visha]);
/// assert_eq!(tithi_yogas(Tithi::ShuklaTritiya, Vara::Budhavara), [AshubhaYoga::Dagdha]);
/// assert!(tithi_yogas(Tithi::ShuklaPratipada, Vara::Ravivara).is_empty());
/// ```
#[must_use]
pub fn tithi_yogas(tithi: Tithi, vara: Vara) -> Vec<AshubhaYoga> {
    let number = tithi.attributes().number;
    tithi_rows(vara)
        .into_iter()
        .filter(|(_, n)| *n == number)
        .map(|(yoga, _)| yoga)
        .collect()
}

/// The nakshatra v. 9 makes Yamaghanta on a weekday, *maghā-viśākhā-
/// śiva-mūla-vahni-brāhmaṁ karo 'rkādi*: Magha on a Sunday, Vishakha,
/// Ardra, Mula, Krittika, Rohini, and Hasta on a Saturday (C760).
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Vara};
/// use teistro_panchanga::omen::yamaghanta;
///
/// assert_eq!(yamaghanta(Vara::Ravivara), Nakshatra::Magha);
/// assert_eq!(yamaghanta(Vara::Mangalavara), Nakshatra::Ardra);
/// assert_eq!(yamaghanta(Vara::Shanivara), Nakshatra::Hasta);
/// ```
#[must_use]
pub const fn yamaghanta(vara: Vara) -> Nakshatra {
    match vara {
        Vara::Ravivara => Nakshatra::Magha,
        Vara::Somavara => Nakshatra::Vishakha,
        Vara::Mangalavara => Nakshatra::Ardra,
        Vara::Budhavara => Nakshatra::Mula,
        Vara::Guruvara => Nakshatra::Krittika,
        Vara::Shukravara => Nakshatra::Rohini,
        _ => Nakshatra::Hasta,
    }
}

/// A day's inauspicious yogas (vv. 8–9, C760): one span for each tithi
/// the day held that makes one with its vara, and one for the nakshatra
/// that makes Yamaghanta, each over that limb's own span and in the order
/// they begin. The vara is the day's own, which turns at sunrise.
/// Vasishtha, quoted under v. 9, lifts them at night; that exception is
/// the reader's, and the spans run as the limbs do.
///
/// ```
/// use teistro_core::catalogue::{AshubhaYoga, Nakshatra, Tithi, Vara};
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::omen::ashubha_yogas;
/// use teistro_panchanga::span::Span;
///
/// // A Sunday that turns from Ekadashi to Dvadashi at 100.6, in Magha all day.
/// let day = Interval::literal(100.25, 101.25);
/// let span = |member, from, to| Span::new(member, Interval::literal(from, to), day);
/// let tithis: Vec<_> =
///     [span(Tithi::ShuklaEkadashi, 99.6, 100.6), span(Tithi::ShuklaDwadashi, 100.6, 101.6)].into_iter().flatten().collect();
/// let stars: Vec<_> = Span::new(Nakshatra::Magha, Interval::literal(99.9, 101.4), day).into_iter().collect();
/// let found: Vec<_> = ashubha_yogas(&tithis, &stars, Vara::Ravivara).iter().map(|s| s.member).collect();
/// assert_eq!(found, [AshubhaYoga::Yamaghanta, AshubhaYoga::Dagdha, AshubhaYoga::Hutashana]);
/// ```
#[must_use]
pub fn ashubha_yogas(
    tithis: &[Span<Tithi>],
    nakshatras: &[Span<Nakshatra>],
    vara: Vara,
) -> Vec<Span<AshubhaYoga>> {
    let mut found: Vec<Span<AshubhaYoga>> = tithis
        .iter()
        .flat_map(|span| {
            tithi_yogas(span.member, vara)
                .into_iter()
                .map(move |yoga| span.map(|_| yoga))
        })
        .chain(
            nakshatras
                .iter()
                .filter(|span| span.member == yamaghanta(vara))
                .map(|span| span.map(|_| AshubhaYoga::Yamaghanta)),
        )
        .collect();
    // Stable, so a tithi's two yogas keep the verse's order.
    found.sort_by(|a, b| a.whole.from.get().total_cmp(&b.whole.from.get()));
    found
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

/// Which pushkara yoga a nakshatra makes with a Bhadra tithi on a
/// Sunday, Tuesday or Saturday, if any: Muhurta Chintamani,
/// Nakshatra-prakarana v. 49 (1954, p. 86), *dvīśāryamājacaraṇāditivahni-
/// vaiśve* the triple, *vasutakṣacāndre* the double (crux C1050).
///
/// The verse's six and three are the nine stars a sign boundary cuts, and
/// the cut is the rule: a star whose four quarters fall three in one sign
/// and one in the next (Krittika, Punarvasu, Uttara Phalguni, Vishakha,
/// Uttara Ashadha, Purva Bhadrapada) makes Tripushkar, and one cut two and
/// two (Mrigashira, Chitra, Dhanishtha) Dwipushkar. Each star is four of
/// the zodiac's 108 quarters and a sign nine, so the star counted from
/// nought begins `4 × id mod 9` quarters into its sign, and a start of six,
/// seven or eight leaves three, two or one quarter there.
///
/// ```
/// use teistro_core::catalogue::{MuhurtaYoga, Nakshatra};
/// use teistro_panchanga::omen::pushkara_of;
///
/// assert_eq!(pushkara_of(Nakshatra::Vishakha), Some(MuhurtaYoga::Tripushkar));
/// assert_eq!(pushkara_of(Nakshatra::Chitra), Some(MuhurtaYoga::Dwipushkar));
/// // Anuradha lies whole in Scorpio, so it makes neither.
/// assert_eq!(pushkara_of(Nakshatra::Anuradha), None);
/// ```
#[must_use]
pub const fn pushkara_of(nakshatra: Nakshatra) -> Option<Kind> {
    match (4 * nakshatra.id()) % 9 {
        6 | 8 => Some(Kind::Tripushkar),
        7 => Some(Kind::Dwipushkar),
        _ => None,
    }
}

/// The varas a pushkara yoga needs.
const PUSHKARA_VARAS: [Vara; 3] = [Vara::Ravivara, Vara::Mangalavara, Vara::Shanivara];

/// The table key of Muhurta Chintamani's full tables
/// (`03-design/day-markers.md` §2).
pub const MUHURTA_CHINTAMANI: &str = "MUHURTA_CHINTAMANI";

/// Each vara's Amrit Siddhi star and the tithi, by its number in either
/// paksha, that takes it away: Narada's pairs as the Piyushadhara quotes
/// them under Muhurta Chintamani, Shubhashubha-prakarana vv. 20–21 (1954,
/// p. 20), *hastarkṣaṁ sūryavāre …*, and the verses' own exceptions,
/// *varjayet sarvakāryeṣu hastārkaṁ pañcamītithau …* (crux C1050).
const AMRIT_SIDDHI_MUHURTA_CHINTAMANI: [(Vara, Nakshatra, u8); 7] = [
    (Vara::Ravivara, Nakshatra::Hasta, 5),
    (Vara::Somavara, Nakshatra::Mrigashira, 6),
    (Vara::Mangalavara, Nakshatra::Ashwini, 7),
    (Vara::Budhavara, Nakshatra::Anuradha, 8),
    (Vara::Guruvara, Nakshatra::Pushya, 9),
    (Vara::Shukravara, Nakshatra::Revati, 10),
    (Vara::Shanivara, Nakshatra::Rohini, 11),
];

/// Each vara's Sarvartha Siddhi stars: Muhurta Chintamani,
/// Shubhashubha-prakarana vv. 28–29 (1954, p. 22), *sūrye'rkamūlottara-
/// puṣyadāsraṁ …*, which the Piyushadhara says no rule of the Anandadi
/// yogas generates. Friday's are the printed text's, *śukre'ntya-
/// bhāgyāśvyaditiśravobham*; the edition's footnote reads Anuradha
/// (*maitra*) for Purva Phalguni (*bhāgya*), and the 1906 print's gloss
/// keeps Purva Phalguni (crux C1050).
const SARVARTHA_SIDDHI_MUHURTA_CHINTAMANI: [(Vara, &[Nakshatra]); 7] = [
    (
        Vara::Ravivara,
        &[
            Nakshatra::Hasta,
            Nakshatra::Mula,
            Nakshatra::UttaraPhalguni,
            Nakshatra::UttaraAshadha,
            Nakshatra::UttaraBhadrapada,
            Nakshatra::Pushya,
            Nakshatra::Ashwini,
        ],
    ),
    (
        Vara::Somavara,
        &[
            Nakshatra::Shravana,
            Nakshatra::Rohini,
            Nakshatra::Mrigashira,
            Nakshatra::Pushya,
            Nakshatra::Anuradha,
        ],
    ),
    (
        Vara::Mangalavara,
        &[
            Nakshatra::Ashwini,
            Nakshatra::UttaraBhadrapada,
            Nakshatra::Krittika,
            Nakshatra::Ashlesha,
        ],
    ),
    (
        Vara::Budhavara,
        &[
            Nakshatra::Rohini,
            Nakshatra::Anuradha,
            Nakshatra::Hasta,
            Nakshatra::Krittika,
            Nakshatra::Mrigashira,
        ],
    ),
    (
        Vara::Guruvara,
        &[
            Nakshatra::Revati,
            Nakshatra::Anuradha,
            Nakshatra::Ashwini,
            Nakshatra::Punarvasu,
            Nakshatra::Pushya,
        ],
    ),
    (
        Vara::Shukravara,
        &[
            Nakshatra::Revati,
            Nakshatra::PurvaPhalguni,
            Nakshatra::Ashwini,
            Nakshatra::Punarvasu,
            Nakshatra::Shravana,
        ],
    ),
    (
        Vara::Shanivara,
        &[Nakshatra::Shravana, Nakshatra::Rohini, Nakshatra::Swati],
    ),
];

/// The vara-and-star yogas named for Pushya: Guru Pushya, Pushya on a
/// Thursday, which Muhurta Chintamani names (*gurupuṣya*, Shubhashubha-
/// prakarana vv. 21–22, 1954 p. 20), and Ravi Pushya, Pushya on a Sunday,
/// one of v. 28's Sunday stars, named so in the almanacs (crux C1050).
const PUSHYA_YOGAS: [(Vara, Kind); 2] = [
    (Vara::Guruvara, Kind::GuruPushya),
    (Vara::Ravivara, Kind::RaviPushya),
];

/// The muhurta yogas of a day.
///
/// Under `CLASSICAL` a vara-and-star yoga is read at the star the day
/// opens in, which the corpus measured, and a pushkara yoga at the
/// tithi and star of sunrise. Under `MUHURTA_CHINTAMANI` every yoga holds
/// while its limbs do inside the vara: each star the day held, the full
/// tables of vv. 20–29, Amrit Siddhi lost while its tithi runs, Guru and
/// Ravi Pushya, and a pushkara yoga wherever a Bhadra tithi and its star
/// overlap (`03-design/day-markers.md` §2).
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
/// use teistro_panchanga::omen::{CLASSICAL, MUHURTA_CHINTAMANI, yogas};
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
/// // Muhurta Chintamani's reads each star of the day: Hasta's two, and in
/// // two-footed Chitra under the Saptami, a Bhadra tithi, the Dwipushkar
/// // the classical reading at sunrise misses.
/// let full = yogas(MUHURTA_CHINTAMANI, Vara::Ravivara, &nakshatras, &tithis)?;
/// assert_eq!(
///     full.iter().map(|y| y.yoga).collect::<Vec<_>>(),
///     [MuhurtaYoga::AmritSiddhi, MuhurtaYoga::SarvarthaSiddhi, MuhurtaYoga::Dwipushkar]
/// );
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
    if tables.eq_ignore_ascii_case(MUHURTA_CHINTAMANI) {
        return Ok(muhurta_chintamani(vara, nakshatras, tithis));
    }
    if !tables.eq_ignore_ascii_case(CLASSICAL) {
        return Err(Error::unsupported(format!(
            "no muhurta yoga table `{tables}`; the SDK ships `{CLASSICAL}` and `{MUHURTA_CHINTAMANI}`"
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
    if let Some(tithi) = tithis.first() {
        found.extend(pushkara(vara, at_sunrise, tithi));
    }
    Ok(found)
}

/// Dwipushkar or Tripushkar: a Bhadra tithi on a Sunday, Tuesday or
/// Saturday in a star a sign boundary cuts ([`pushkara_of`]), while both
/// hold.
fn pushkara(vara: Vara, star: &Span<Nakshatra>, tithi: &Span<Tithi>) -> Option<MuhurtaYoga> {
    if !PUSHKARA_VARAS.contains(&vara) || tithi.member.attributes().class != TithiClass::Bhadra {
        return None;
    }
    let yoga = pushkara_of(star.member)?;
    Some(MuhurtaYoga {
        yoga,
        at: tithi.inside.clipped_to(star.inside)?,
        because: YogaCause::VaraTithiNakshatra {
            vara,
            tithi: tithi.member,
            nakshatra: star.member,
        },
    })
}

/// The yogas of Muhurta Chintamani's tables over every star and tithi
/// of the day, in the order they begin.
fn muhurta_chintamani(
    vara: Vara,
    nakshatras: &[Span<Nakshatra>],
    tithis: &[Span<Tithi>],
) -> Vec<MuhurtaYoga> {
    let mut found = Vec::new();
    for star in nakshatras {
        let because = YogaCause::VaraNakshatra {
            vara,
            nakshatra: star.member,
        };
        let held = |yoga, at| MuhurtaYoga { yoga, at, because };
        for (_, _, losing) in AMRIT_SIDDHI_MUHURTA_CHINTAMANI
            .iter()
            .filter(|(v, n, _)| *v == vara && *n == star.member)
        {
            let lost: Vec<Interval> = tithis
                .iter()
                .filter(|tithi| tithi.member.attributes().number == *losing)
                .map(|tithi| tithi.inside)
                .collect();
            found.extend(
                without(star.inside, &lost)
                    .into_iter()
                    .map(|at| held(Kind::AmritSiddhi, at)),
            );
        }
        let sarvartha = SARVARTHA_SIDDHI_MUHURTA_CHINTAMANI
            .iter()
            .any(|(v, stars)| *v == vara && stars.contains(&star.member));
        if sarvartha {
            found.push(held(Kind::SarvarthaSiddhi, star.inside));
        }
        if star.member == Nakshatra::Pushya {
            found.extend(
                PUSHYA_YOGAS
                    .iter()
                    .filter(|(v, _)| *v == vara)
                    .map(|(_, yoga)| held(*yoga, star.inside)),
            );
        }
        found.extend(
            tithis
                .iter()
                .filter_map(|tithi| pushkara(vara, star, tithi)),
        );
    }
    // Stable, so one star's yogas keep the tables' order.
    found.sort_by(|a, b| a.at.from.get().total_cmp(&b.at.from.get()));
    found
}

/// `whole` less every interval of `cut`: the pieces left, in order, each
/// of some length.
fn without(whole: Interval, cut: &[Interval]) -> Vec<Interval> {
    let mut left = vec![whole];
    for gap in cut {
        left = left
            .into_iter()
            .flat_map(|piece| {
                if piece.overlaps(*gap) {
                    [
                        Interval::new(piece.from, gap.from).ok(),
                        Interval::new(gap.to, piece.to).ok(),
                    ]
                } else {
                    [Some(piece), None]
                }
            })
            .flatten()
            .filter(|piece| !piece.is_empty())
            .collect();
    }
    left
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

    /// The tables against the commentary's other witnesses, each in its
    /// own order (Shubhashubha-prakarana vv. 8–9, C760): Narada's dagdha
    /// verse, Brihaspati's visha verse, Guru's hutashana rule (the sixth
    /// tithi onward with Monday onward, Sunday's the twelfth) and Garga's
    /// Yamaghanta.
    #[test]
    fn the_inauspicious_yogas_agree_with_the_commentarys_witnesses() {
        use Vara::*;
        use teistro_core::catalogue::{AshubhaYoga, Nakshatra, Tithi, Vara};
        let held = |yoga, number: u8, vara| {
            let tithi = Tithi::ALL
                .into_iter()
                .find(|t| t.attributes().number == number)
                .unwrap();
            super::tithi_yogas(tithi, vara).contains(&yoga)
        };
        // "ekādaśī cendu-vāre dvādaśī cārka-vāsare, ṣaṣṭhī bṛhaspater …"
        let narada = [
            (Somavara, 11),
            (Ravivara, 12),
            (Guruvara, 6),
            (Budhavara, 3),
            (Shukravara, 8),
            (Shanivara, 9),
            (Mangalavara, 5),
        ];
        // "ṣaṣṭhī śaśini navamī ca śukre budhe dvitīyā tapane caturthī, jīve 'ṣṭamī sauri-kuje 'hni saptamī"
        let brihaspati = [
            (Somavara, 6),
            (Shukravara, 9),
            (Budhavara, 2),
            (Ravivara, 4),
            (Guruvara, 8),
            (Shanivara, 7),
            (Mangalavara, 7),
        ];
        for (yoga, list) in [
            (AshubhaYoga::Dagdha, narada),
            (AshubhaYoga::Visha, brihaspati),
        ] {
            for (vara, number) in list {
                assert!(held(yoga, number, vara), "{yoga:?} {vara:?} {number}");
                // One tithi number a weekday for each yoga.
                let others = (1..=15)
                    .filter(|n| *n != number && held(yoga, *n, vara))
                    .count();
                assert_eq!(others, 0, "{yoga:?} {vara:?}");
            }
        }
        // Guru: "ṣaṣṭhyādi-tithayaḥ sapta candravārādibhir yutāḥ".
        let monday_on = [
            Somavara,
            Mangalavara,
            Budhavara,
            Guruvara,
            Shukravara,
            Shanivara,
            Ravivara,
        ];
        for (number, vara) in (6..=12).zip(monday_on) {
            assert!(
                held(AshubhaYoga::Hutashana, number, vara),
                "{vara:?} {number}"
            );
        }
        // Garga: "maghā viśākhā cārdrā ca mūlarkṣaṁ ca kṛttikā, rohiṇī hasta-devaṁ yamaghaṇṭāḥ kramād raveḥ".
        let garga = [
            Nakshatra::Magha,
            Nakshatra::Vishakha,
            Nakshatra::Ardra,
            Nakshatra::Mula,
            Nakshatra::Krittika,
            Nakshatra::Rohini,
            Nakshatra::Hasta,
        ];
        let sunday_on = [
            Ravivara,
            Somavara,
            Mangalavara,
            Budhavara,
            Guruvara,
            Shukravara,
            Shanivara,
        ];
        for (vara, star) in sunday_on.into_iter().zip(garga) {
            assert_eq!(super::yamaghanta(vara), star, "{vara:?}");
        }
        // A tithi's number holds in both pakshas.
        assert_eq!(
            super::tithi_yogas(Tithi::ShuklaPanchami, Mangalavara),
            super::tithi_yogas(Tithi::KrishnaPanchami, Mangalavara)
        );
    }
    #[test]
    fn shiva_vasa_reproduces_the_surya_panchanga_table() {
        use teistro_core::catalogue::ShivaVasa::{
            Bhojana, Gauri, Kailasa, Krida, Sabha, Shmashana, Vrishabha,
        };
        // Surya Panchanga VS 2082, p. 60: the Shukla tithis 1 to 15 down
        // the rows.
        let rows = [
            Shmashana, Gauri, Sabha, Krida, Kailasa, Vrishabha, Bhojana, Shmashana, Gauri, Sabha,
            Krida, Kailasa, Vrishabha, Bhojana, Shmashana,
        ];
        for (row, printed) in (1u16..).zip(rows) {
            let tithi = Tithi::from_id(row - 1).unwrap();
            assert_eq!(super::shiva_vasa(tithi), printed, "Shukla {row}");
        }
        // Its Krishna column: the 7th in the first row, the 1st to 6th in
        // the second to seventh, the 8th and Amavasya in the ninth, the
        // 9th to 14th in the tenth to fifteenth.
        let krishna = |k: u16| Tithi::from_id(14 + k).unwrap();
        assert_eq!(super::shiva_vasa(krishna(7)), rows[0]);
        for k in 1..=6 {
            assert_eq!(
                super::shiva_vasa(krishna(k)),
                rows[usize::from(k)],
                "Krishna {k}"
            );
        }
        assert_eq!(super::shiva_vasa(krishna(8)), rows[8]);
        assert_eq!(super::shiva_vasa(Tithi::Amavasya), rows[8]);
        for k in 9..=14 {
            assert_eq!(
                super::shiva_vasa(krishna(k)),
                rows[usize::from(k)],
                "Krishna {k}"
            );
        }
    }

    #[test]
    fn bhadra_dwells_by_the_verses_signs() {
        use teistro_core::catalogue::BhadraVasa::{Martya, Patala, Svarga};
        // v. 45: Aquarius and Pisces, Cancer and Leo among men; the three
        // from Aries and Scorpio in heaven; Virgo, Sagittarius, Libra and
        // Capricorn below.
        let by_sign = [
            Svarga, Svarga, Svarga, Martya, Martya, Patala, Patala, Svarga, Patala, Patala, Martya,
            Martya,
        ];
        for (id, loka) in (0u16..).zip(by_sign) {
            assert_eq!(
                super::bhadra_vasa(Rashi::from_id(id).unwrap()),
                loka,
                "{id}"
            );
        }
        // No vishti, no Bhadra.
        let calm = [span(teistro_core::catalogue::Karana::Bava)];
        assert_eq!(
            super::bhadra_vasas(&calm, &[span(Rashi::Leo)], window()),
            Vec::new()
        );
    }

    #[test]
    fn a_panchaka_is_named_by_the_vara_it_begins_on() {
        use Vara::{Budhavara, Guruvara, Mangalavara, Ravivara, Shanivara, Shukravara, Somavara};
        let named = [
            (Ravivara, Some(Panchaka::Roga)),
            (Somavara, Some(Panchaka::Raja)),
            (Mangalavara, Some(Panchaka::Agni)),
            (Budhavara, None),
            (Guruvara, None),
            (Shukravara, Some(Panchaka::Chora)),
            (Shanivara, Some(Panchaka::Mrityu)),
        ];
        for (vara, name) in named {
            assert_eq!(super::panchaka_by_vara(vara), name, "{vara:?}");
        }
    }

    #[test]
    fn a_convention_set_to_none_is_not_named() {
        use teistro_core::settings::{
            DEFAULT_PROFILE, PanchakaName, Profile, SettingsPatch, ShivaVasaReading,
        };
        let mut panchanga = Profile::shipped(DEFAULT_PROFILE)
            .unwrap()
            .resolve(&SettingsPatch::default())
            .unwrap()
            .settings
            .panchanga;
        let applied = super::applied_conventions(&panchanga);
        assert_eq!(
            applied.iter().map(|c| c.value.as_str()).collect::<Vec<_>>(),
            ["WEEKDAY", "PRINTED"]
        );
        assert!(applied.iter().all(|c| c.reason.contains("no text")));
        panchanga.panchaka_name = PanchakaName::None;
        panchanga.shiva_vasa = ShivaVasaReading::None;
        assert_eq!(super::applied_conventions(&panchanga), Vec::new());
        // Every knob the table names is a key the settings answer to.
        for convention in super::PRINTED_CONVENTIONS {
            let (group, knob) = convention.knob.split_once('.').unwrap();
            assert_eq!(group, "panchanga");
            assert!(
                serde_json::to_value(&panchanga)
                    .unwrap()
                    .get(knob)
                    .is_some(),
                "{knob}"
            );
        }
    }

    /// A day of the given stars and tithis, each `(member, from, to)`, in a
    /// window from 100.25 to 101.25.
    fn spans<T: Copy>(rows: &[(T, f64, f64)]) -> Vec<Span<T>> {
        let day = Interval::literal(100.25, 101.25);
        rows.iter()
            .filter_map(|(member, from, to)| Span::new(*member, Interval::literal(*from, *to), day))
            .collect()
    }

    /// Each star is four padas and each sign nine, so the padas a sign
    /// boundary leaves a star on either side say how many feet it has in
    /// its first sign: the generator `pushkara_of` encodes is checked
    /// here against that count, star by star.
    #[test]
    fn pushkara_is_the_star_a_sign_boundary_cuts() {
        use super::pushkara_of;
        for star in &Nakshatra::ALL {
            let first = 4 * star.id();
            let feet_before_boundary = (0..4).take_while(|p| (first + p) / 9 == first / 9).count();
            let expected = match feet_before_boundary {
                1 | 3 => Some(Kind::Tripushkar),
                2 => Some(Kind::Dwipushkar),
                _ => None,
            };
            assert_eq!(pushkara_of(*star), expected, "{star:?}");
        }
        let two: Vec<_> = Nakshatra::ALL
            .iter()
            .filter(|n| pushkara_of(**n) == Some(Kind::Dwipushkar))
            .copied()
            .collect();
        assert_eq!(
            two,
            [
                Nakshatra::Mrigashira,
                Nakshatra::Chitra,
                Nakshatra::Dhanishtha
            ]
        );
        assert_eq!(
            Nakshatra::ALL
                .iter()
                .filter(|n| pushkara_of(**n) == Some(Kind::Tripushkar))
                .count(),
            6
        );
        assert_eq!(pushkara_of(Nakshatra::Anuradha), None, "whole in Scorpio");
    }

    #[test]
    fn without_leaves_the_pieces_a_cut_does_not_touch() {
        use super::without;
        let whole = Interval::literal(10.0, 20.0);
        assert_eq!(without(whole, &[]), [whole]);
        assert_eq!(
            without(whole, &[Interval::literal(12.0, 14.0)]),
            [Interval::literal(10.0, 12.0), Interval::literal(14.0, 20.0)]
        );
        // A cut over an end, one over all of it, one outside it.
        assert_eq!(
            without(whole, &[Interval::literal(5.0, 11.0)]),
            [Interval::literal(11.0, 20.0)]
        );
        assert_eq!(without(whole, &[Interval::literal(0.0, 30.0)]).len(), 0);
        assert_eq!(without(whole, &[Interval::literal(20.0, 25.0)]), [whole]);
        // Two cuts.
        assert_eq!(
            without(
                whole,
                &[Interval::literal(12.0, 13.0), Interval::literal(15.0, 16.0)]
            ),
            [
                Interval::literal(10.0, 12.0),
                Interval::literal(13.0, 15.0),
                Interval::literal(16.0, 20.0)
            ]
        );
    }

    /// A Thursday in Pushya: Muhurta Chintamani's tables give Amrit
    /// Siddhi, Sarvartha Siddhi and Guru Pushya over the star, and its
    /// Amrit Siddhi is lost while a Navami runs (vv. 20-22).
    #[test]
    fn muhurta_chintamani_reads_every_star_and_loses_amrit_siddhi_in_its_tithi() {
        use super::MUHURTA_CHINTAMANI;
        let stars = spans(&[
            (Nakshatra::Punarvasu, 99.9, 100.5),
            (Nakshatra::Pushya, 100.5, 101.6),
        ]);
        let tithis = spans(&[
            (Tithi::ShuklaAshtami, 99.8, 100.8),
            (Tithi::ShuklaNavami, 100.8, 101.0),
            (Tithi::ShuklaDashami, 101.0, 102.0),
        ]);
        let found = yogas(MUHURTA_CHINTAMANI, Vara::Guruvara, &stars, &tithis).unwrap();
        let of = |kind| -> Vec<Interval> {
            found
                .iter()
                .filter(|y| y.yoga == kind)
                .map(|y| y.at)
                .collect()
        };
        // Punarvasu, the day's first star, is Thursday's Sarvartha star too:
        // the classical reading at sunrise would stop there.
        assert_eq!(
            of(Kind::SarvarthaSiddhi),
            [
                Interval::literal(100.25, 100.5),
                Interval::literal(100.5, 101.25)
            ]
        );
        assert_eq!(of(Kind::GuruPushya), [Interval::literal(100.5, 101.25)]);
        assert_eq!(
            of(Kind::AmritSiddhi),
            [
                Interval::literal(100.5, 100.8),
                Interval::literal(101.0, 101.25)
            ],
            "lost while the Navami runs"
        );
        assert!(of(Kind::RaviPushya).is_empty(), "not a Sunday");
        // The same sky on a Sunday: Ravi Pushya and Sunday's Sarvartha star.
        let sunday = yogas(MUHURTA_CHINTAMANI, Vara::Ravivara, &stars, &tithis).unwrap();
        let kinds: Vec<_> = sunday.iter().map(|y| y.yoga).collect();
        assert_eq!(kinds, [Kind::SarvarthaSiddhi, Kind::RaviPushya]);
        // In order of beginning.
        assert!(
            found
                .windows(2)
                .all(|w| w[0].at.from.get() <= w[1].at.from.get())
        );
    }

    /// A pushkara yoga under Muhurta Chintamani's tables holds wherever a
    /// Bhadra tithi and its star overlap, not only at sunrise.
    #[test]
    fn muhurta_chintamani_finds_a_pushkara_after_sunrise() {
        use super::MUHURTA_CHINTAMANI;
        let stars = spans(&[
            (Nakshatra::Magha, 100.0, 100.6),
            (Nakshatra::Chitra, 100.6, 101.7),
        ]);
        let tithis = spans(&[
            (Tithi::ShuklaPratipada, 100.0, 100.9),
            (Tithi::ShuklaDvitiya, 100.9, 101.9),
        ]);
        let classical = yogas(CLASSICAL, Vara::Mangalavara, &stars, &tithis).unwrap();
        assert!(classical.is_empty(), "{classical:?}");
        let found = yogas(MUHURTA_CHINTAMANI, Vara::Mangalavara, &stars, &tithis).unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].yoga, Kind::Dwipushkar);
        assert_eq!(found[0].at, Interval::literal(100.9, 101.25));
        let error = yogas("NONE", Vara::Mangalavara, &stars, &tithis).unwrap_err();
        assert!(error.message.contains(MUHURTA_CHINTAMANI), "{error}");
    }

    /// Gandanta is one span a junction, the same span from either side of
    /// it, and ganda mula and vinchudo follow their stars and sign.
    #[test]
    fn day_markers_follow_the_junctions_and_scorpio() {
        use super::{GANDANTA_REACH_DAYS, day_markers};
        use teistro_core::catalogue::DayMarker;
        let day = Interval::literal(100.25, 101.25);
        // A day that holds Revati's end and Ashwini's start, and one that
        // holds only Ashwini, the junction at its sunrise's edge.
        let both = spans(&[
            (Nakshatra::Revati, 99.5, 100.5),
            (Nakshatra::Ashwini, 100.5, 101.5),
        ]);
        let after = spans(&[(Nakshatra::Ashwini, 100.5, 101.5)]);
        let gandanta = |markers: &[Span<DayMarker>]| -> Vec<Interval> {
            markers
                .iter()
                .filter(|m| m.member == DayMarker::Gandanta)
                .map(|m| m.whole)
                .collect()
        };
        let expected = Interval::literal(100.5 - GANDANTA_REACH_DAYS, 100.5 + GANDANTA_REACH_DAYS);
        assert_eq!(gandanta(&day_markers(&both, &[], day)), [expected]);
        assert_eq!(gandanta(&day_markers(&after, &[], day)), [expected]);
        // Ashlesha into Magha and Jyeshtha into Mula; Bharani touches none.
        let others = spans(&[
            (Nakshatra::Ashlesha, 99.9, 100.4),
            (Nakshatra::Magha, 100.4, 101.0),
            (Nakshatra::PurvaPhalguni, 101.0, 102.0),
        ]);
        let found = day_markers(&others, &[], day);
        assert_eq!(
            found
                .iter()
                .filter(|m| m.member == DayMarker::GandaMula)
                .count(),
            2
        );
        assert_eq!(gandanta(&found).len(), 1);
        assert_eq!(
            day_markers(&spans(&[(Nakshatra::Bharani, 99.0, 102.0)]), &[], day).len(),
            0
        );
        // Vinchudo is the Moon's time in Scorpio and nothing else.
        let signs = spans(&[(Rashi::Libra, 99.0, 100.7), (Rashi::Scorpio, 100.7, 103.0)]);
        let vinchudo: Vec<_> = day_markers(&[], &signs, day)
            .into_iter()
            .map(|m| (m.member, m.whole, m.inside))
            .collect();
        assert_eq!(
            vinchudo,
            [(
                DayMarker::Vinchudo,
                Interval::literal(100.7, 103.0),
                Interval::literal(100.7, 101.25)
            )]
        );
    }
}
