//! Every number the text supplies, cited by chapter and verse (Burgess,
//! 1860), and the overlay a tradition's bija corrections take.

use core::fmt;

use crate::equation::Epicycle;
use crate::mean::{Cycle, Motion};

/// The bodies the text gives a motion of their own, in the text's order
/// (I.29 to 34). Rahu and Ketu are the Moon's node and its opposite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Planet {
    /// The Sun.
    Sun,
    /// The Moon.
    Moon,
    /// Mars.
    Mars,
    /// Mercury; its own motion is its conjunction (sighrocca).
    Mercury,
    /// Jupiter.
    Jupiter,
    /// Venus; its own motion is its conjunction (sighrocca).
    Venus,
    /// Saturn.
    Saturn,
}

impl Planet {
    /// The seven, in the text's order.
    pub const ALL: [Planet; 7] = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mars,
        Planet::Mercury,
        Planet::Jupiter,
        Planet::Venus,
        Planet::Saturn,
    ];

    /// The five star planets, which take the sighra equation.
    pub const STARS: [Planet; 5] = [
        Planet::Mars,
        Planet::Mercury,
        Planet::Jupiter,
        Planet::Venus,
        Planet::Saturn,
    ];

    /// Whether the planet's mean place is the Sun's (Mercury and Venus,
    /// whose own revolutions are those of their conjunction; I.29).
    #[must_use]
    pub const fn is_inferior(self) -> bool {
        matches!(self, Planet::Mercury | Planet::Venus)
    }

    /// The index in the seven-entry tables.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The index in the five-entry star-planet tables; `None` for the Sun
    /// and the Moon.
    #[must_use]
    pub const fn star_index(self) -> Option<usize> {
        match self {
            Planet::Sun | Planet::Moon => None,
            Planet::Mars => Some(0),
            Planet::Mercury => Some(1),
            Planet::Jupiter => Some(2),
            Planet::Venus => Some(3),
            Planet::Saturn => Some(4),
        }
    }

    /// The name as the text's tradition writes it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Planet::Sun => "Surya",
            Planet::Moon => "Chandra",
            Planet::Mars => "Mangala",
            Planet::Mercury => "Budha",
            Planet::Jupiter => "Guru",
            Planet::Venus => "Shukra",
            Planet::Saturn => "Shani",
        }
    }
}

impl fmt::Display for Planet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The text's numbers. Each field names its verse; the derived fields
/// name the verses they combine.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Parameters {
    /// Civil days in an age (yuga): I.37.
    pub yuga_civil_days: u64,
    /// Solar years in an age: I.15 to 17.
    pub yuga_years: u32,
    /// Ages in an aeon (kalpa): I.19 to 21.
    pub kalpa_yugas: u32,
    /// Civil days from the aeon's start to the start of the Kali age: the
    /// elapsed years of I.22 to 24 (six Manus with their twilights, twenty-
    /// seven ages and three quarters of the twenty-eighth, less the
    /// 17 064 000 years of creation) turned into days by I.37.
    pub elapsed_days_at_kali: u64,
    /// The epoch: the start of the Kali age at midnight on the meridian of
    /// Lanka, as a Julian day in Universal Time (the days of I.48 to 53 are
    /// reckoned to midnight at Lanka; I.62 places Lanka on the meridian of
    /// Ujjain, 75°47′ east).
    pub epoch_jd_ut: f64,
    /// The meridian's longitude east, degrees: I.62 with Ujjain's modern
    /// longitude.
    pub meridian_deg: f64,
    /// The mean motions of the seven, in [`Planet`] order: I.29 to 34.
    pub motions: [Motion; 7],
    /// The Moon's apsis: I.34.
    pub moon_apsis: Motion,
    /// The Moon's node, retrograde: I.34.
    pub moon_node: Motion,
    /// The apsides of the Sun and the five star planets per aeon, in
    /// [`Planet`] order without the Moon (Sun, Mars, Mercury, Jupiter,
    /// Venus, Saturn): I.41 to 42.
    pub apsides: [Motion; 6],
    /// The nodes of the five star planets per aeon, retrograde, in
    /// [`Planet::STARS`] order: I.43 to 44.
    pub nodes: [Motion; 5],
    /// The manda (apsis) epicycles of the seven: II.34 to 37.
    pub manda: [Epicycle; 7],
    /// The sighra (conjunction) epicycles of the five star planets, in
    /// [`Planet::STARS`] order: II.36 to 37.
    pub sighra: [Epicycle; 5],
    /// The sine of the greatest declination, in the table's units of a
    /// radius of 3438 (24 degrees): II.28.
    pub obliquity_sine: u32,
    /// The libration of the equinoxes per age: III.9.
    pub ayana_revolutions_per_yuga: u32,
    /// The libration's extent in degrees, three tenths of its argument's
    /// reduced arc: III.10 to 12.
    pub ayana_extent_deg: u32,
    /// The extreme latitudes in minutes of arc, in the order Moon, Mars,
    /// Mercury, Jupiter, Venus, Saturn: I.68 to 70 (an eightieth of the
    /// circle's minutes for the Moon, and its ninths for the others; or,
    /// as verse 70 states them, 27, 9, 12, 6, 12 and 12 times ten).
    pub extreme_latitudes_arcmin: [u32; 6],
    /// The times of rising of the first three signs at Lanka, in
    /// respirations (a minute of arc of the equator each): III.44, which
    /// III.42 to 43 derive from the sine table and the obliquity.
    pub lanka_rising_asu: [u32; 3],
}

impl Parameters {
    /// The text as Burgess (1860) prints it.
    pub const TEXT: Parameters = Parameters {
        yuga_civil_days: 1_577_917_828,
        yuga_years: 4_320_000,
        kalpa_yugas: 1000,
        // 1 955 880 000 years / 4 320 000 years per age = 452.75 ages of
        // 1 577 917 828 days, an exact whole number.
        elapsed_days_at_kali: 714_402_296_627,
        epoch_jd_ut: 588_465.5 - (75.0 + 47.0 / 60.0) / 360.0,
        meridian_deg: 75.0 + 47.0 / 60.0,
        motions: [
            Motion::direct(4_320_000, Cycle::Yuga),
            Motion::direct(57_753_336, Cycle::Yuga),
            Motion::direct(2_296_832, Cycle::Yuga),
            Motion::direct(17_937_060, Cycle::Yuga),
            Motion::direct(364_220, Cycle::Yuga),
            Motion::direct(7_022_376, Cycle::Yuga),
            Motion::direct(146_568, Cycle::Yuga),
        ],
        moon_apsis: Motion::direct(488_203, Cycle::Yuga),
        moon_node: Motion::retrograde(232_238, Cycle::Yuga),
        apsides: [
            Motion::direct(387, Cycle::Kalpa),
            Motion::direct(204, Cycle::Kalpa),
            Motion::direct(368, Cycle::Kalpa),
            Motion::direct(900, Cycle::Kalpa),
            Motion::direct(535, Cycle::Kalpa),
            Motion::direct(39, Cycle::Kalpa),
        ],
        nodes: [
            Motion::retrograde(214, Cycle::Kalpa),
            Motion::retrograde(488, Cycle::Kalpa),
            Motion::retrograde(174, Cycle::Kalpa),
            Motion::retrograde(903, Cycle::Kalpa),
            Motion::retrograde(662, Cycle::Kalpa),
        ],
        manda: [
            Epicycle::new(14 * 60, 13 * 60 + 40),
            Epicycle::new(32 * 60, 31 * 60 + 40),
            Epicycle::new(75 * 60, 72 * 60),
            Epicycle::new(30 * 60, 28 * 60),
            Epicycle::new(33 * 60, 32 * 60),
            Epicycle::new(12 * 60, 11 * 60),
            Epicycle::new(49 * 60, 48 * 60),
        ],
        sighra: [
            Epicycle::new(235 * 60, 232 * 60),
            Epicycle::new(133 * 60, 132 * 60),
            Epicycle::new(70 * 60, 72 * 60),
            Epicycle::new(262 * 60, 260 * 60),
            Epicycle::new(39 * 60, 40 * 60),
        ],
        obliquity_sine: 1397,
        ayana_revolutions_per_yuga: 600,
        ayana_extent_deg: 27,
        extreme_latitudes_arcmin: [270, 90, 120, 60, 120, 120],
        lanka_rising_asu: [1670, 1795, 1935],
    };

    /// The motion of a planet's own revolutions (the conjunction's for
    /// Mercury and Venus).
    #[must_use]
    #[allow(
        clippy::indexing_slicing,
        reason = "the index is the planet's discriminant, inside the seven by construction"
    )]
    pub const fn motion(&self, planet: Planet) -> Motion {
        self.motions[planet.index()]
    }

    /// The apsis of the Sun or a star planet; the Moon's is
    /// [`Parameters::moon_apsis`].
    #[must_use]
    pub const fn apsis(&self, planet: Planet) -> Motion {
        match planet {
            Planet::Sun => self.apsides[0],
            Planet::Moon => self.moon_apsis,
            Planet::Mars => self.apsides[1],
            Planet::Mercury => self.apsides[2],
            Planet::Jupiter => self.apsides[3],
            Planet::Venus => self.apsides[4],
            Planet::Saturn => self.apsides[5],
        }
    }

    /// The node of a planet other than the Sun: the Moon's from I.34, a
    /// star planet's from I.43 to 44.
    #[must_use]
    #[allow(
        clippy::indexing_slicing,
        reason = "the star index is inside the five by construction"
    )]
    pub const fn node(&self, planet: Planet) -> Option<Motion> {
        match planet {
            Planet::Sun => None,
            Planet::Moon => Some(self.moon_node),
            star => match star.star_index() {
                Some(index) => Some(self.nodes[index]),
                None => None,
            },
        }
    }

    /// The extreme latitude of a planet other than the Sun, minutes of
    /// arc (I.68 to 70).
    #[must_use]
    pub const fn extreme_latitude_arcmin(&self, planet: Planet) -> Option<u32> {
        let index = match planet {
            Planet::Sun => return None,
            Planet::Moon => 0,
            Planet::Mars => 1,
            Planet::Mercury => 2,
            Planet::Jupiter => 3,
            Planet::Venus => 4,
            Planet::Saturn => 5,
        };
        #[allow(
            clippy::indexing_slicing,
            reason = "an index inside the six by construction"
        )]
        Some(self.extreme_latitudes_arcmin[index])
    }

    /// The manda epicycle of a planet.
    #[must_use]
    #[allow(
        clippy::indexing_slicing,
        reason = "the index is the planet's discriminant, inside the seven by construction"
    )]
    pub const fn manda_epicycle(&self, planet: Planet) -> Epicycle {
        self.manda[planet.index()]
    }

    /// The sighra epicycle of a star planet.
    #[must_use]
    #[allow(
        clippy::indexing_slicing,
        reason = "the star index is inside the five by construction"
    )]
    pub const fn sighra_epicycle(&self, planet: Planet) -> Option<Epicycle> {
        match planet.star_index() {
            Some(index) => Some(self.sighra[index]),
            None => None,
        }
    }

    /// The civil days of a cycle.
    #[must_use]
    pub const fn cycle_days(&self, cycle: Cycle) -> u64 {
        match cycle {
            Cycle::Yuga => self.yuga_civil_days,
            Cycle::Kalpa => self.yuga_civil_days * self.kalpa_yugas as u64,
        }
    }

    /// The same parameters with a tradition's bija corrections applied to
    /// the revolution counts.
    #[must_use]
    pub fn with_bija(&self, bija: &Bija) -> Parameters {
        let mut params = self.clone();
        let adjust = |motion: &mut Motion, delta: i64| {
            motion.revolutions = motion.revolutions.saturating_add_signed(delta);
        };
        for (planet, delta) in [
            (Planet::Moon, bija.moon),
            (Planet::Mars, bija.mars),
            (Planet::Mercury, bija.mercury),
            (Planet::Jupiter, bija.jupiter),
            (Planet::Venus, bija.venus),
            (Planet::Saturn, bija.saturn),
        ] {
            if let Some(motion) = params.motions.get_mut(planet.index()) {
                adjust(motion, delta);
            }
        }
        adjust(&mut params.moon_apsis, bija.moon_apsis);
        adjust(&mut params.moon_node, bija.moon_node);
        params
    }
}

/// A bija (seed) correction: the whole revolutions per age a tradition
/// adds to or takes from the text's counts. The text has none; the later
/// commentators supply sets that differ, so no set ships until it is
/// cited (`docs/calendars/bikram-sambat.md`), and a consumer with its
/// tradition's set applies it through [`Parameters::with_bija`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Bija {
    /// The Moon's revolutions.
    pub moon: i64,
    /// The Moon's apsis.
    pub moon_apsis: i64,
    /// The Moon's node.
    pub moon_node: i64,
    /// Mars.
    pub mars: i64,
    /// Mercury's conjunction.
    pub mercury: i64,
    /// Jupiter.
    pub jupiter: i64,
    /// Venus's conjunction.
    pub venus: i64,
    /// Saturn.
    pub saturn: i64,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn the_elapsed_days_are_the_texts_years_turned_into_days() {
        let p = &Parameters::TEXT;
        // Six Manus of 71 ages with seven twilights of a Krita age each,
        // twenty-seven ages, and the Krita, Treta and Dvapara of the
        // twenty-eighth, less the years of creation (I.22 to 24).
        let krita = 1_728_000u64;
        let treta = 1_296_000u64;
        let dvapara = 864_000u64;
        let yuga = u64::from(p.yuga_years);
        let years = 6 * 71 * yuga + 7 * krita + 27 * yuga + krita + treta + dvapara - 17_064_000;
        assert_eq!(years, 1_955_880_000);
        // Ages elapsed times the days of an age, exactly.
        assert_eq!(years * p.yuga_civil_days / yuga, p.elapsed_days_at_kali);
        assert_eq!(years * p.yuga_civil_days % yuga, 0);
        assert_eq!(p.cycle_days(Cycle::Kalpa), 1_577_917_828_000);
        assert!((p.epoch_jd_ut - 588_465.289_490_740_7).abs() < 1e-9);
    }

    #[test]
    fn bija_moves_the_counts_it_names_and_nothing_else() {
        let bija = Bija {
            moon: -1,
            saturn: 12,
            ..Bija::default()
        };
        let p = Parameters::TEXT.with_bija(&bija);
        assert_eq!(p.motion(Planet::Moon).revolutions, 57_753_335);
        assert_eq!(p.motion(Planet::Saturn).revolutions, 146_580);
        assert_eq!(p.motion(Planet::Sun), Parameters::TEXT.motion(Planet::Sun));
        assert_eq!(p.moon_node, Parameters::TEXT.moon_node);
        assert_eq!(
            Parameters::TEXT.with_bija(&Bija::default()),
            Parameters::TEXT
        );
        let json = serde_json::to_string(&bija).unwrap();
        assert_eq!(serde_json::from_str::<Bija>(&json).unwrap(), bija);
        assert!(serde_json::from_str::<Bija>("{\"sun\": 1}").is_err());
    }

    #[test]
    fn planets_index_their_tables() {
        for (i, planet) in Planet::ALL.iter().enumerate() {
            assert_eq!(planet.index(), i);
        }
        for (i, planet) in Planet::STARS.iter().enumerate() {
            assert_eq!(planet.star_index(), Some(i));
            assert!(Parameters::TEXT.sighra_epicycle(*planet).is_some());
        }
        assert_eq!(Planet::Sun.star_index(), None);
        assert!(Parameters::TEXT.sighra_epicycle(Planet::Moon).is_none());
        assert!(Planet::Venus.is_inferior() && !Planet::Mars.is_inferior());
        assert_eq!(Planet::Jupiter.to_string(), "Guru");
        assert_eq!(Parameters::TEXT.apsis(Planet::Saturn).revolutions, 39);
        assert_eq!(Parameters::TEXT.apsis(Planet::Moon).revolutions, 488_203);
        assert_eq!(Parameters::TEXT.manda_epicycle(Planet::Sun).odd_arcmin, 820);
        assert_eq!(Parameters::TEXT.node(Planet::Sun), None);
        assert_eq!(
            Parameters::TEXT.node(Planet::Moon),
            Some(Parameters::TEXT.moon_node)
        );
        assert_eq!(
            Parameters::TEXT.node(Planet::Saturn).map(|n| n.revolutions),
            Some(662)
        );
        assert_eq!(Parameters::TEXT.extreme_latitude_arcmin(Planet::Sun), None);
        assert_eq!(
            Parameters::TEXT.extreme_latitude_arcmin(Planet::Moon),
            Some(270)
        );
        assert_eq!(
            Parameters::TEXT.extreme_latitude_arcmin(Planet::Jupiter),
            Some(60)
        );
        // I.68 to 69: an eightieth of the circle's minutes, and its ninths.
        assert_eq!(21_600 / 80, 270);
        assert_eq!([270 * 3 / 9, 270 * 4 / 9, 270 * 2 / 9], [90, 120, 60]);
    }
}
