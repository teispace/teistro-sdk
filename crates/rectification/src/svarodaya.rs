//! The nadis and the tattvas of the Shiva Svarodaya (`03-design/rectification.md`,
//! step 7, X12), read in the 1899 Khemraj print and checked against the
//! 1919 and 1931 prints (`rectification-sources.md` §3).
//!
//! - **The nadi at sunrise** goes by the tithi then, from pratipada in runs
//!   of three days: the Moon's first in the bright fortnight, the Sun's in
//!   the dark (v. 62).
//! - **The turns:** each nadi runs two and a half ghatis, the two in turn
//!   through the sixty ghatis of day and night, twenty-four turns with no
//!   new start at sunset (v. 63). A ghati is a sixtieth of the day from
//!   sunrise to sunrise.
//! - **The tattvas** rise afresh inside each turn in the order air, fire,
//!   earth, water (vv. 71, 72), at v. 197's palas (air 20, fire 30, earth
//!   50, water 40), and ether's 10 flow at the junction (v. 154), so they
//!   close the turn. The sushumna is the junction itself, a moment
//!   (vv. 132, 154), which is every turn's end.
//! - **The sex** is the nadi's: the Moon's female, the Sun's male (v. 60).
//!
//! **No verse reads a nadi or a tattva at a birth**: v. 60's sex belongs to
//! the nadi, and the sex rules that read a tattva are about a question or a
//! conception (vv. 294, 299). So this is a report and never a bar, labelled
//! an application the text does not make.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Tithi;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use crate::baseline::{Sex, Tattva};
use crate::{Day, Sky, Window};

/// The turns of a day and night: sixty ghatis in turns of two and a half
/// (v. 63).
pub const TURNS: u8 = 24;

/// The palas of one turn: two and a half ghatis of sixty (v. 197's sum).
pub const TURN_PALAS: f64 = 150.0;

/// The tattvas of one turn in the order they rise, each with its palas:
/// air, fire, earth and water (v. 71) at v. 197's palas, and ether at the
/// junction (v. 154), last.
pub const ORDER: [(Tattva, f64); 5] = [
    (Tattva::Vayu, 20.0),
    (Tattva::Agni, 30.0),
    (Tattva::Prithvi, 50.0),
    (Tattva::Jala, 40.0),
    (Tattva::Akasha, 10.0),
];

/// One of the two nadis that alternate through the day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Nadi {
    /// The Moon's, the left (ida): female (v. 60).
    Moon,
    /// The Sun's, the right (pingala): male (v. 60).
    Sun,
}

impl Nadi {
    /// The other nadi, which the next turn runs.
    #[must_use]
    pub const fn other(self) -> Nadi {
        match self {
            Nadi::Moon => Nadi::Sun,
            Nadi::Sun => Nadi::Moon,
        }
    }

    /// The sex v. 60 gives the nadi.
    #[must_use]
    pub const fn sex(self) -> Sex {
        match self {
            Nadi::Moon => Sex::Female,
            Nadi::Sun => Sex::Male,
        }
    }

    /// The nadi rising at a sunrise whose tithi is `tithi` (v. 62): in
    /// runs of three days from pratipada, the Moon's first in the bright
    /// fortnight and the Sun's in the dark.
    #[must_use]
    pub const fn at_sunrise(tithi: Tithi) -> Nadi {
        let index = tithi as u16;
        let bright = index < 15;
        let run = (index % 15) / 3;
        let first = if bright { Nadi::Moon } else { Nadi::Sun };
        if run % 2 == 0 { first } else { first.other() }
    }
}

/// One stretch of a day under one nadi and one tattva.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SvarodayaRun {
    /// Where it starts.
    pub from: JulianDay<Utc>,
    /// Where it ends.
    pub to: JulianDay<Utc>,
    /// The nadi flowing.
    pub nadi: Nadi,
    /// Its turn in the day, 0 the one rising at sunrise, to 23.
    pub turn: u8,
    /// The tattva flowing in it.
    pub tattva: Tattva,
    /// The sex v. 60 gives the nadi.
    pub sex: Sex,
}

/// The nadi and the tattva at an instant, and the day they are counted in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Svarodaya {
    /// The sunrise the turns are counted from.
    pub sunrise: JulianDay<Utc>,
    /// The sunrise that ends the day.
    pub next_sunrise: JulianDay<Utc>,
    /// The tithi at the sunrise, which gives its nadi.
    pub tithi: Tithi,
    /// The nadi rising at the sunrise.
    pub sunrise_nadi: Nadi,
    /// The run the instant falls in.
    pub run: SvarodayaRun,
    /// The turn's junctions, where the sushumna flows for a moment: its
    /// start and its end.
    pub junctions: [JulianDay<Utc>; 2],
}

/// The tithi at an instant: the Moon's elongation from the Sun in twelfths
/// of a sign.
fn tithi_at(sky: &dyn Sky, at: JulianDay<Utc>) -> Result<Tithi, Error> {
    let elongation = (sky.moon_deg(at)? - sky.sun_deg(at)?).rem_euclid(360.0);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "an elongation in [0, 360) over 12 is 0 to 29"
    )]
    let index = (elongation / 12.0).floor() as usize;
    Tithi::ALL
        .get(index)
        .copied()
        .ok_or_else(|| Error::internal("an elongation outside the thirty tithis"))
}

/// Every run of one day, from its sunrise to the next, in order: the
/// twenty-four turns, each cut into [`ORDER`]'s five.
fn day_runs(day: &Day, first: Nadi) -> Vec<SvarodayaRun> {
    let sunrise = day.sunrise.get();
    let turn_days = (day.next_sunrise.get() - sunrise) / f64::from(TURNS);
    let mut runs = Vec::with_capacity(usize::from(TURNS) * ORDER.len());
    for turn in 0..TURNS {
        let nadi = if turn % 2 == 0 { first } else { first.other() };
        let start = sunrise + f64::from(turn) * turn_days;
        let mut palas = 0.0;
        for (tattva, span) in ORDER {
            let from = start + turn_days * palas / TURN_PALAS;
            palas += span;
            // The last run closes on the turn's own end, so the day's runs
            // tile it with no gap a rounding could open.
            let to = if palas >= TURN_PALAS {
                sunrise + f64::from(turn + 1) * turn_days
            } else {
                start + turn_days * palas / TURN_PALAS
            };
            runs.push(SvarodayaRun {
                from: JulianDay::literal(from),
                to: JulianDay::literal(to),
                nadi,
                turn,
                tattva,
                sex: nadi.sex(),
            });
        }
    }
    runs
}

/// The nadi and the tattva at an instant, counted in the day it belongs
/// to by the sky's own rule.
///
/// ```
/// use teistro_core::catalogue::{Tithi, Vara};
/// use teistro_core::error::Error;
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_rectification::svarodaya::{Nadi, svarodaya};
/// use teistro_rectification::{Day, Sky, Tattva};
///
/// // A day of exactly 24 hours from sunrise at 2460000.25, and a Moon
/// // 5° past the Sun: shukla pratipada.
/// struct Still;
/// impl Sky for Still {
///     fn ascendant_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> { Ok(0.0) }
///     fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> { Ok(10.0) }
///     fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> { Ok(15.0) }
///     fn day(&self, _: JulianDay<Utc>) -> Result<Day, Error> {
///         Ok(Day::literal(2_460_000.25, 2_460_000.75, 2_460_001.25, Vara::Somavara))
///     }
/// }
/// // Five minutes after sunrise: the Moon's nadi, in its air (the turn's
/// // first 20 palas of 150, eight minutes of the hour).
/// let read = svarodaya(&Still, JulianDay::literal(2_460_000.25 + 5.0 / 1440.0))?;
/// assert_eq!(read.tithi, Tithi::ShuklaPratipada);
/// assert_eq!((read.run.nadi, read.run.tattva), (Nadi::Moon, Tattva::Vayu));
/// # Ok::<(), Error>(())
/// ```
///
/// # Errors
///
/// Whatever the sky cannot answer, unchanged.
pub fn svarodaya(sky: &dyn Sky, at: JulianDay<Utc>) -> Result<Svarodaya, Error> {
    let day = sky.day(at)?;
    let tithi = tithi_at(sky, day.sunrise)?;
    let sunrise_nadi = Nadi::at_sunrise(tithi);
    let runs = day_runs(&day, sunrise_nadi);
    let run = runs
        .iter()
        .find(|run| at.get() < run.to.get())
        .or(runs.last())
        .copied()
        .ok_or_else(|| Error::internal("a day with no turns"))?;
    let turn_days = (day.next_sunrise.get() - day.sunrise.get()) / f64::from(TURNS);
    let start = day.sunrise.get() + f64::from(run.turn) * turn_days;
    Ok(Svarodaya {
        sunrise: day.sunrise,
        next_sunrise: day.next_sunrise,
        tithi,
        sunrise_nadi,
        run,
        junctions: [
            JulianDay::literal(start),
            JulianDay::literal(start + turn_days),
        ],
    })
}

/// Every run of nadi and tattva inside a window, in order and clipped to
/// it: what a consumer sets beside the purifier's runs.
///
/// # Errors
///
/// Whatever the sky cannot answer, unchanged.
pub fn svarodaya_runs(sky: &dyn Sky, window: Window) -> Result<Vec<SvarodayaRun>, Error> {
    let (from, to) = (window.from.get(), window.to.get());
    let mut runs = Vec::new();
    let mut day = sky.day(window.from)?;
    loop {
        let first = Nadi::at_sunrise(tithi_at(sky, day.sunrise)?);
        for run in day_runs(&day, first) {
            let (a, b) = (run.from.get().max(from), run.to.get().min(to));
            if b > a {
                runs.push(SvarodayaRun {
                    from: JulianDay::literal(a),
                    to: JulianDay::literal(b),
                    ..run
                });
            }
        }
        if day.next_sunrise.get() >= to {
            return Ok(runs);
        }
        day = sky.day(day.next_sunrise)?;
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, reason = "tests")]
mod tests {
    use teistro_core::catalogue::Vara;

    use super::*;

    const SUNRISE: f64 = 2_460_000.25;
    const MINUTE: f64 = 1.0 / 1440.0;

    /// Days of exactly 24 hours from 6:00, and a Moon `elongation` past a
    /// still Sun.
    struct Still {
        elongation: f64,
    }

    impl Sky for Still {
        fn ascendant_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> {
            Ok(0.0)
        }
        fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> {
            Ok(100.0)
        }
        fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> {
            Ok(100.0 + self.elongation)
        }
        fn day(&self, at: JulianDay<Utc>) -> Result<Day, Error> {
            let sunrise = SUNRISE + (at.get() - SUNRISE).floor();
            Ok(Day::literal(
                sunrise,
                sunrise + 0.5,
                sunrise + 1.0,
                Vara::Somavara,
            ))
        }
    }

    fn at(minutes: f64) -> JulianDay<Utc> {
        JulianDay::literal(SUNRISE + minutes * MINUTE)
    }

    /// v. 62: from pratipada in runs of three days, the Moon's first in the
    /// bright fortnight and the Sun's in the dark.
    #[test]
    fn the_sunrise_nadi_runs_three_days_at_a_time() {
        let read: Vec<Nadi> = Tithi::ALL.iter().map(|t| Nadi::at_sunrise(*t)).collect();
        let (m, s) = (Nadi::Moon, Nadi::Sun);
        assert_eq!(read[..15], [m, m, m, s, s, s, m, m, m, s, s, s, m, m, m]);
        assert_eq!(read[15..], [s, s, s, m, m, m, s, s, s, m, m, m, s, s, s]);
    }

    /// v. 63 and v. 197: twenty-four turns alternating, each cut 20, 30,
    /// 50, 40 and 10 palas of 150, tiling the day to its last bit.
    #[test]
    fn a_day_is_twenty_four_turns_of_five_tattvas() {
        let day = Still { elongation: 5.0 }.day(at(0.0)).expect("a day");
        let runs = day_runs(&day, Nadi::Moon);
        assert_eq!(runs.len(), 120);
        assert_eq!(runs[0].from, day.sunrise);
        assert_eq!(runs[119].to, day.next_sunrise);
        for pair in runs.windows(2) {
            assert_eq!(pair[0].to, pair[1].from, "no gap and no overlap");
        }
        for (k, run) in runs.iter().enumerate() {
            let (tattva, palas) = ORDER[k % 5];
            assert_eq!(run.tattva, tattva);
            let minutes = (run.to.get() - run.from.get()) / MINUTE;
            assert!(
                (minutes - palas * 60.0 / TURN_PALAS).abs() < 1e-6,
                "{k}: {minutes}"
            );
            let nadi = if run.turn % 2 == 0 {
                Nadi::Moon
            } else {
                Nadi::Sun
            };
            assert_eq!((run.nadi, run.sex), (nadi, nadi.sex()));
        }
    }

    /// Half an hour into the first turn is 75 palas: earth (50 to 100).
    /// Rama Prasad's "a ghari each" would make it fire, the second ghati;
    /// the Sanskrit's five within the turn (vv. 64, 72) make it earth.
    #[test]
    fn the_tattvas_are_inside_the_turn_not_a_ghati_each() {
        let sky = Still { elongation: 5.0 };
        let read = svarodaya(&sky, at(30.0)).expect("a reading");
        assert_eq!(read.run.tattva, Tattva::Prithvi);
        assert_ne!(read.run.tattva, Tattva::Agni);
        // Air's 20 palas are the turn's first eight minutes.
        let air = svarodaya(&sky, at(5.0)).expect("air");
        assert_eq!(air.run.tattva, Tattva::Vayu);
        let fire = svarodaya(&sky, at(10.0)).expect("fire");
        assert_eq!(fire.run.tattva, Tattva::Agni);
        // Ether closes the turn, at the junction (v. 154).
        let late = svarodaya(&sky, at(59.0)).expect("ether");
        assert_eq!(
            (late.run.tattva, late.run.nadi),
            (Tattva::Akasha, Nadi::Moon)
        );
        assert!((late.junctions[1].get() - at(60.0).get()).abs() < 1e-9);
        // The next turn is the Sun's, and so on through the night.
        assert_eq!(
            svarodaya(&sky, at(61.0)).expect("turn 1").run.nadi,
            Nadi::Sun
        );
        let night = svarodaya(&sky, at(23.0 * 60.0 + 1.0)).expect("turn 23");
        assert_eq!((night.run.turn, night.run.nadi), (23, Nadi::Sun));
    }

    /// The dark fortnight's fourth day (krishna chaturthi) starts the Moon's.
    #[test]
    fn the_tithi_at_sunrise_starts_the_day() {
        let sky = Still {
            elongation: 180.0 + 3.0 * 12.0 + 1.0,
        };
        let read = svarodaya(&sky, at(1.0)).expect("a reading");
        assert_eq!(read.tithi, Tithi::KrishnaChaturthi);
        assert_eq!((read.sunrise_nadi, read.run.nadi), (Nadi::Moon, Nadi::Moon));
        assert_eq!(read.run.sex, Sex::Female);
    }

    /// A window across a sunrise is tiled by the runs of both days, each
    /// day's nadi read from its own sunrise.
    #[test]
    fn a_window_across_sunrise_is_tiled_by_both_days() {
        let sky = Still { elongation: 5.0 };
        let window = Window::between(at(-90.0), at(90.0)).expect("a window");
        let runs = svarodaya_runs(&sky, window).expect("runs");
        assert_eq!(runs.first().expect("a run").from, window.from);
        assert_eq!(runs.last().expect("a run").to, window.to);
        for pair in runs.windows(2) {
            assert_eq!(pair[0].to, pair[1].from);
        }
        // Before sunrise: the day before's last turns, the 23rd the Sun's.
        assert_eq!((runs[0].turn, runs[0].nadi), (22, Nadi::Moon));
        let first = runs
            .iter()
            .find(|run| run.from.get() >= SUNRISE)
            .expect("after sunrise");
        assert_eq!(
            (first.turn, first.nadi, first.tattva),
            (0, Nadi::Moon, Tattva::Vayu)
        );
    }
}
