//! Rules that hold on exactly the same charts, probable duplicates, and
//! rules that hold on none or on all of them (`05-testing/01-quality-bar.md`,
//! "rule fixtures").
//!
//! Every shipped rule is evaluated over a sky the SDK computes: charts
//! spread over two centuries at six places, no two sharing an instant.
//! Rules that hold on the same charts, and on enough of them that chance
//! is not the reason, form a group, and a group has one of three
//! explanations:
//!
//! - **two texts state one condition**: its rules cite different texts,
//!   which is agreement between the sources and the reason the SDK ships
//!   both;
//! - **one aspect read from both grahas**: two rules of one text place
//!   each graha in the sign seventh from the other's, under the other's
//!   aspect, and both cast only the seventh, so each holds exactly where
//!   the other does;
//! - **one text says it twice**: listed in [`REVIEWED`] with where.
//!
//! Anything else is a rule copied where it should have differed, and the
//! gate fails naming it; so does a [`REVIEWED`] entry that no longer
//! co-fires. A rule written as conditions that holds on no chart is listed
//! in [`FIXTURES`] with an instant and a place where it does, which the
//! pass evaluates, or in [`UNSEEN`] with why it cannot; one unlisted fails,
//! and so does a rule holding on every chart, since nothing then shows it
//! can fail, unless [`ALWAYS`] says it names grahas rather than asks. The pass computes each chart's whole reading, its panchanga
//! included, so it runs in verify rather than fast-check.
//!
//! `cargo xtask cofiring` writes the page; `check-cofiring` regenerates it
//! in memory and fails on any difference. `cargo xtask cofiring-probe` finds
//! a fixture: given instants narrowed elsewhere on placements alone, it says
//! which the SDK holds a rule on.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use teistro::rules::{Body, BodyRef, BodySubject, Condition, House, Rule, SignRef};
use teistro::{ChartRequest, Context, Ephemeris, RuleRequest, ShippedRules, UtcOffset};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};

use crate::generated::{Output, check, write};
use crate::measure::count;

const PAGE: &str = "docs/03-design/rule-cofiring-measured.md";

/// How many places.
const PLACE_COUNT: u32 = 6;

/// The latitudes and longitudes the sky is computed at: Kathmandu, the
/// equator, the tropics, two temperate cities and the southern hemisphere.
const PLACES: [(f64, f64); PLACE_COUNT as usize] = [
    (27.7172, 85.324),
    (0.0, -78.5),
    (13.08, 80.27),
    (40.71, -74.0),
    (51.5, -0.13),
    (-33.87, 151.21),
];

/// Charts at each place.
const PER_PLACE: u32 = 1000;

/// The first instant, 1900-01-01 0h UT, and the span the charts cover.
const FIRST_JD: f64 = 2_415_020.5;
const SPAN_DAYS: f64 = 73_049.0;

/// The fewest charts a group must hold on to be listed. Below it, nested
/// conjunctions of five and six grahas (each holding on a chart or two of
/// the sky) meet by chance; every group above it is an equivalence.
const SUPPORT: usize = 5;

/// One text stating one condition twice, each with where it does.
const REVIEWED: [(&[&str], &str); 91] = [
    (
        &[
            "SARAVALI_SATURN_WITH_BOTH_LUMINARIES",
            "SARAVALI_TOGETHER_SUN_MOON_SATURN",
        ],
        "Saravali reads Saturn with both luminaries twice: ch. 10 vv. 6 and 7 among the deaths \
         of children, ch. 16 v. 5 among the sets of three",
    ),
    (
        &[
            "BPHS_ARMY_CHIEF_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA",
            "BPHS_KING_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA",
        ],
        "BPHS gives malefics third and sixth from the Atmakaraka twice: ch. 39 v. 10 makes the \
         native a king, ch. 40 v. 7 an army chief",
    ),
    (
        &[
            "SARAVALI_MERCURY_AND_JUPITER_IN_THE_SEVENTH",
            "SARAVALI_MERCURY_JUPITER_IN_BHAVA_7",
        ],
        "Saravali reads Mercury and Jupiter in the seventh in ch. 34 v. 47 (one wife), and again \
         among two grahas in an angle",
    ),
    (
        &[
            "SARAVALI_MERCURY_AND_VENUS_IN_THE_SEVENTH",
            "SARAVALI_MERCURY_VENUS_IN_BHAVA_7",
        ],
        "Saravali reads Mercury and Venus in the seventh in ch. 34, and again among two grahas \
         in an angle",
    ),
    (
        &[
            "PHALADEEPIKA_KETU_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
            "PHALADEEPIKA_RAHU_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        ],
        "Phaladeepika ch. 14 vv. 13 and 15 give each node its own death; the nodes stand \
         opposite and aspect by the seventh alone, so one in the eighth or the second puts both \
         on it",
    ),
    (
        &["SARAVALI_MOON_IN_TAURUS", "SARAVALI_RAJA_MOON_EXALTED"],
        "Saravali reads the Moon in Taurus in ch. 23 among her signs, and again in ch. 35 v. 106 \
         as the exalted Moon that makes a king even when waning",
    ),
    (
        &[
            "PARIJATA_DARIDRA_SATURN_MOON_SUN_IN_KENDRAS",
            "PARIJATA_ROGA_SATURN_MOON_SUN_IN_KENDRAS_IDIOT",
        ],
        "Jataka Parijata gives Saturn, the Moon and the Sun in kendras twice: ch. 6 v. 4 \
         among the poverty yogas, ch. 6 v. 81 among the diseases of the mind",
    ),
    (
        &[
            "PARIJATA_MARANA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DEATH_BY_DROPSY",
            "PARIJATA_ROGA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DROPSY",
        ],
        "Jataka Parijata gives Saturn in Cancer and the Moon in Capricorn twice: ch. 5A v. 3 \
         as the manner of death, ch. 6 v. 97 as the disease",
    ),
    (
        &[
            "PARIJATA_JANMA_JARAJA_LORDS_OF_ONE_AND_FOUR_EXCHANGED",
            "PARIJATA_VIYONI_LORDS_OF_ONE_AND_FOUR_EXCHANGED",
        ],
        "Jataka Parijata ch. 3 gives the first and fourth lords exchanged twice: v. 5 a birth \
         of another kind, v. 54 the child of another man, the same figure read of a human chart",
    ),
    (
        &[
            "PARIJATA_JANMA_KANINA_LUMINARIES_TOGETHER",
            "PARIJATA_TOGETHER_SUN_MOON",
        ],
        "Jataka Parijata gives the Sun and the Moon together twice: ch. 3 v. 51 among the \
         parentages, the child born before marriage, and ch. 8 among the conjunctions",
    ),
    (
        &[
            "SARAVALI_MARANA_VENUS_IN_THE_EIGHTH_DEATH_BY_THIRST",
            "SARAVALI_VENUS_IN_BHAVA_8",
        ],
        "Saravali gives Venus in the eighth twice: ch. 46 v. 1 as the manner of death, and among \
         the grahas in the houses",
    ),
    (
        &[
            "BPHS_MARS_IN_THIRD_DEATH_BY_WOUNDS",
            "BPHS_SAHAJA_MARS_IN_THE_THIRD",
        ],
        "BPHS gives the figure twice: ch. 44 v. 25-31 (marana) and ch. 14 v. 14 (house-based)",
    ),
    (
        &[
            "SARAVALI_MARANA_SUN_IN_THE_EIGHTH_DEATH_BY_FIRE",
            "SARAVALI_SUN_IN_BHAVA_8",
        ],
        "Saravali gives the Sun in the eighth twice: ch. 46 v. 1 as the manner of death, and \
         among the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_JUPITER_IN_BHAVA_8",
            "SARAVALI_MARANA_JUPITER_IN_THE_EIGHTH_DEATH_BY_DISEASE",
        ],
        "Saravali gives Jupiter in the eighth twice: ch. 46 v. 1 as the manner of death, and \
         among the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_MARANA_SATURN_IN_THE_EIGHTH_DEATH_BY_HUNGER",
            "SARAVALI_SATURN_IN_BHAVA_8",
        ],
        "Saravali gives Saturn in the eighth twice: ch. 46 v. 1 as the manner of death, and \
         among the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_MARANA_MARS_IN_THE_EIGHTH_DEATH_BY_A_WEAPON",
            "SARAVALI_MARS_IN_BHAVA_8",
        ],
        "Saravali gives Mars in the eighth twice: ch. 46 v. 1 as the manner of death, and among \
         the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_MARANA_MERCURY_IN_THE_EIGHTH_DEATH_BY_FEVER",
            "SARAVALI_MERCURY_IN_BHAVA_8",
        ],
        "Saravali gives Mercury in the eighth twice: ch. 46 v. 1 as the manner of death, and \
         among the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_BHANGA_SATURN_RISING",
            "SARAVALI_SATURN_IN_BHAVA_1",
        ],
        "Saravali gives Saturn rising twice: ch. 38 v. 12 among what cancels a raja yoga, and \
         among the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_MARANA_MOON_IN_THE_EIGHTH_DEATH_BY_WATER",
            "SARAVALI_MOON_IN_BHAVA_8",
        ],
        "Saravali gives the Moon in the eighth twice: ch. 46 v. 1 as the manner of death, and \
         among the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_VENUS",
            "SARAVALI_TOGETHER_SUN_MARS_MERCURY_VENUS",
        ],
        "Saravali gives the Sun, Mars, Mercury and Venus in one sign twice: ch. 20 v. 13 among \
         the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_JUPITER",
            "SARAVALI_TOGETHER_SUN_MARS_MERCURY_JUPITER",
        ],
        "Saravali gives the Sun, Mars, Mercury and Jupiter in one sign twice: ch. 20 v. 4, 12 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_MARANA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DROPSY",
            "SARAVALI_SATURN_IN_CANCER_ASPECTED_BY_MOON",
        ],
        "Saravali gives Saturn in Cancer with the Moon in Capricorn twice: ch. 46 v. 4 as the \
         manner of death, and among the grahas aspected",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY",
            "SARAVALI_TOGETHER_SUN_MOON_MARS_MERCURY",
        ],
        "Saravali gives the Sun, the Moon, Mars and Mercury in one sign twice: ch. 20 v. 2, 16 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "BPHS_BHAGYA_SATURN_SECOND_SUN_SEVENTH_FROM_SATURN",
            "BPHS_SATURN_IN_SECOND_ASPECTED_BY_SUN",
        ],
        "BPHS gives the figure twice: ch. 20 v. 23-24 (arishta-father) and ch. 42 v. 18 (daridra)",
    ),
    (
        &[
            "BPHS_KALATRA_SECOND_AND_ELEVENTH_LORDS_EXCHANGED",
            "BPHS_LABHA_SECOND_AND_ELEVENTH_LORDS_EXCHANGED",
        ],
        "BPHS gives the figure twice: ch. 18 v. 29 (kalatra) and ch. 22 v. 9 (dhana)",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MERCURY_JUPITER_SATURN",
            "SARAVALI_TOGETHER_SUN_MERCURY_JUPITER_SATURN",
        ],
        "Saravali gives the Sun, Mercury, Jupiter and Saturn in one sign twice: ch. 20 v. 4 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MOON_MERCURY_SATURN",
            "SARAVALI_TOGETHER_SUN_MOON_MERCURY_SATURN",
        ],
        "Saravali gives the Sun, the Moon, Mercury and Saturn in one sign twice: ch. 20 v. 3 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_MOON_MARS_MERCURY_VENUS",
            "SARAVALI_TOGETHER_MOON_MARS_MERCURY_VENUS",
        ],
        "Saravali gives the Moon, Mars, Mercury and Venus in one sign twice: ch. 20 v. 16 among \
         the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_JUPITER_VENUS_SATURN",
            "SARAVALI_TOGETHER_SUN_JUPITER_VENUS_SATURN",
        ],
        "Saravali gives the Sun, Jupiter, Venus and Saturn in one sign twice: ch. 20 v. 16 among \
         the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_VENUS",
            "SARAVALI_TOGETHER_SUN_MOON_MARS_VENUS",
        ],
        "Saravali gives the Sun, the Moon, Mars and Venus in one sign twice: ch. 20 v. 16 among \
         the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_JUPITER_VENUS",
            "SARAVALI_TOGETHER_SUN_MARS_MERCURY_JUPITER_VENUS",
        ],
        "Saravali gives the Sun, Mars, Mercury, Jupiter and Venus in one sign twice: ch. 20 v. 9 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY_VENUS",
            "SARAVALI_TOGETHER_SUN_MOON_MARS_MERCURY_VENUS",
        ],
        "Saravali gives the Sun, the Moon, Mars, Mercury and Venus in one sign twice: ch. 20 v. \
         8 among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_VENUS_SATURN",
            "SARAVALI_TOGETHER_SUN_MARS_MERCURY_VENUS_SATURN",
        ],
        "Saravali gives the Sun, Mars, Mercury, Venus and Saturn in one sign twice: ch. 20 v. 2, \
         6 among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MOON_VENUS_SATURN",
            "SARAVALI_TOGETHER_SUN_MOON_VENUS_SATURN",
        ],
        "Saravali gives the Sun, the Moon, Venus and Saturn in one sign twice: ch. 20 v. 12 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_MOON_MARS_JUPITER_VENUS",
            "SARAVALI_TOGETHER_MOON_MARS_JUPITER_VENUS",
        ],
        "Saravali gives the Moon, Mars, Jupiter and Venus in one sign twice: ch. 20 v. 16 among \
         the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_MOON_MARS_MERCURY_JUPITER",
            "SARAVALI_TOGETHER_MOON_MARS_MERCURY_JUPITER",
        ],
        "Saravali gives the Moon, Mars, Mercury and Jupiter in one sign twice: ch. 20 v. 13 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_MOON_MARS_MERCURY_SATURN",
            "SARAVALI_TOGETHER_MOON_MARS_MERCURY_SATURN",
        ],
        "Saravali gives the Moon, Mars, Mercury and Saturn in one sign twice: ch. 20 v. 14 among \
         the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_MARS_MERCURY_JUPITER_SATURN",
            "SARAVALI_TOGETHER_MARS_MERCURY_JUPITER_SATURN",
        ],
        "Saravali gives Mars, Mercury, Jupiter and Saturn in one sign twice: ch. 20 v. 6, 14 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "SARAVALI_PRAVRAJYA_SUN_MARS_JUPITER_VENUS_SATURN",
            "SARAVALI_TOGETHER_SUN_MARS_JUPITER_VENUS_SATURN",
        ],
        "Saravali gives the Sun, Mars, Jupiter, Venus and Saturn in one sign twice: ch. 20 v. 7 \
         among the ascetics, and among its sets of grahas together",
    ),
    (
        &[
            "BPHS_DUAL_THIRD_PLACE_OF_DEATH",
            "BPHS_STRI_SEVENTH_MOVABLE",
        ],
        "BPHS reads a movable lagna twice: ch. 44 v. 34 by its dual third, the place of death, \
         and ch. 80 v. 17 by its movable seventh, in a woman's chart",
    ),
    (
        &[
            "PHALADEEPIKA_AVA",
            "PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_A_DUSTHANA",
        ],
        "Phaladeepika gives the lagna lord in a dusthana twice: ch. 6 vv. 57 to 69's Ava yoga, \
         and ch. 16 v. 4 among the effects of the houses",
    ),
    (
        &[
            "PARIJATA_A_BENEFIC_FALLEN",
            "PARIJATA_JANMA_A_BENEFIC_DEBILITATED",
        ],
        "Jataka Parijata gives a benefic debilitated twice: ch. 6 v. 35, and ch. 3 v. 69 among \
         the circumstances of birth",
    ),
    (
        &[
            "BPHS_KARAKAMSHA_KETU_ASPECTING_THE_FIFTH_FROM_IT",
            "BPHS_KARAKAMSHA_RAHU_FIFTH_FROM_IT",
        ],
        "BPHS ch. 33 gives one figure two ways: Ketu aspecting the fifth from the Karakamsha (v. \
         37) is Rahu in it (v. 39), Ketu's seventh being Rahu's sign",
    ),
    (
        &[
            "BPHS_STRI_VENUS_EIGHTH",
            "BPHS_VENUS_IN_EIGHTH_DEATH_BY_HUNGER",
        ],
        "BPHS gives Venus in the eighth twice: ch. 44 vv. 35-36 as the manner of death, and ch. \
         80 v. 31 in a woman's chart",
    ),
    (
        &["BPHS_STRI_SUN_EIGHTH", "BPHS_SUN_IN_EIGHTH_DEATH_BY_FIRE"],
        "BPHS gives the Sun in the eighth twice: ch. 44 vv. 35-36 as the manner of death, and ch. \
         80 v. 26 in a woman's chart",
    ),
    (
        &[
            "BPHS_JUPITER_IN_EIGHTH_DEATH_BY_DISEASE",
            "BPHS_STRI_JUPITER_EIGHTH",
        ],
        "BPHS gives Jupiter in the eighth twice: ch. 44 vv. 35-36 as the manner of death, and ch. \
         80 v. 30 in a woman's chart",
    ),
    (
        &[
            "BPHS_SATURN_IN_EIGHTH_DEATH_BY_THIRST",
            "BPHS_STRI_SATURN_EIGHTH",
        ],
        "BPHS gives Saturn in the eighth twice: ch. 44 vv. 35-36 as the manner of death, and ch. \
         80 v. 32 in a woman's chart",
    ),
    (
        &[
            "BPHS_MARS_IN_EIGHTH_DEATH_BY_WEAPONS",
            "BPHS_STRI_MARS_EIGHTH",
        ],
        "BPHS gives Mars in the eighth twice: ch. 44 vv. 35-36 as the manner of death, and ch. 80 \
         v. 28 in a woman's chart",
    ),
    (
        &[
            "BPHS_MERCURY_IN_EIGHTH_DEATH_BY_FEVER",
            "BPHS_STRI_MERCURY_EIGHTH",
        ],
        "BPHS gives Mercury in the eighth twice: ch. 44 vv. 35-36 as the manner of death, and ch. \
         80 v. 29 in a woman's chart",
    ),
    (
        &[
            "BPHS_MOON_IN_EIGHTH_DEATH_BY_WATER",
            "BPHS_STRI_MOON_EIGHTH",
        ],
        "BPHS gives the Moon in the eighth twice: ch. 44 vv. 35-36 as the manner of death, and \
         ch. 80 v. 27 in a woman's chart",
    ),
    (
        &[
            "SARAVALI_MARS_IN_BHAVA_7",
            "SARAVALI_STRI_MARS_SEVENTH_WIDOWED_A_BRIDE",
        ],
        "Saravali gives Mars in the seventh twice: ch. 45 v. 16 in a woman's chart, and among the \
         grahas in the houses",
    ),
    (
        &[
            "BPHS_KARAKAMSHA_KETU_IN_IT",
            "BPHS_KARAKAMSHA_RAHU_SEVENTH_FROM_IT",
        ],
        "BPHS ch. 33 gives one figure two ways, the nodes standing opposite: Ketu in the \
         Karakamsha (v. 18) is Rahu in its seventh (v. 48)",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_CANCER",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_FIRST",
        ],
        "Saravali reads the first decanate of Cancer rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 42 by the death the 22nd drekkana from it, Aquarius's first, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_SCORPIO",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_SECOND",
        ],
        "Saravali reads the second decanate of Scorpio rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 27 by the death the 22nd drekkana from it, Gemini's second, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_LEO",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_FIRST",
        ],
        "Saravali reads the first decanate of Leo rising twice: among the rising decanates, and \
         in ch. 46 vv. 22, 44 by the death the 22nd drekkana from it, Pisces's first, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_LIBRA",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_SECOND",
        ],
        "Saravali reads the second decanate of Libra rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 25 by the death the 22nd drekkana from it, Taurus's second, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_GEMINI",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_FIRST",
        ],
        "Saravali reads the first decanate of Gemini rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 40 by the death the 22nd drekkana from it, Capricorn's first, \
         gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_SAGITTARIUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_SECOND",
        ],
        "Saravali reads the second decanate of Sagittarius rising twice: among the rising \
         decanates, and in ch. 46 vv. 22, 30 by the death the 22nd drekkana from it, Cancer's \
         second, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_LIBRA",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_FIRST",
        ],
        "Saravali reads the first decanate of Libra rising twice: among the rising decanates, and \
         in ch. 46 vv. 22, 25 by the death the 22nd drekkana from it, Taurus's first, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_SCORPIO",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_THIRD",
        ],
        "Saravali reads the third decanate of Scorpio rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 28 by the death the 22nd drekkana from it, Gemini's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_SCORPIO",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_FIRST",
        ],
        "Saravali reads the first decanate of Scorpio rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 27 by the death the 22nd drekkana from it, Gemini's first, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_GEMINI",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_SECOND",
        ],
        "Saravali reads the second decanate of Gemini rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 41 by the death the 22nd drekkana from it, Capricorn's second, \
         gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_VIRGO",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_ARIES_SECOND",
        ],
        "Saravali reads the second decanate of Virgo rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 24 by the death the 22nd drekkana from it, Aries's second, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_LEO",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_THIRD",
        ],
        "Saravali reads the third decanate of Leo rising twice: among the rising decanates, and \
         in ch. 46 vv. 22, 45 by the death the 22nd drekkana from it, Pisces's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_TAURUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_FIRST",
        ],
        "Saravali reads the first decanate of Taurus rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 38 by the death the 22nd drekkana from it, Sagittarius's first, \
         gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_CANCER",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_THIRD",
        ],
        "Saravali reads the third decanate of Cancer rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 43 by the death the 22nd drekkana from it, Aquarius's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_VIRGO",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_ARIES_THIRD",
        ],
        "Saravali reads the third decanate of Virgo rising twice: among the rising decanates, and \
         in ch. 46 vv. 22, 24 by the death the 22nd drekkana from it, Aries's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_GEMINI",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_THIRD",
        ],
        "Saravali reads the third decanate of Gemini rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 41 by the death the 22nd drekkana from it, Capricorn's third, \
         gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_LIBRA",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_THIRD",
        ],
        "Saravali reads the third decanate of Libra rising twice: among the rising decanates, and \
         in ch. 46 vv. 22, 26 by the death the 22nd drekkana from it, Taurus's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_CANCER",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_SECOND",
        ],
        "Saravali reads the second decanate of Cancer rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 43 by the death the 22nd drekkana from it, Aquarius's second, \
         gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_LEO",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_SECOND",
        ],
        "Saravali reads the second decanate of Leo rising twice: among the rising decanates, and \
         in ch. 46 vv. 22, 45 by the death the 22nd drekkana from it, Pisces's second, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_CAPRICORN",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_SECOND",
        ],
        "Saravali reads the second decanate of Capricorn rising twice: among the rising \
         decanates, and in ch. 46 vv. 22, 31 by the death the 22nd drekkana from it, Leo's \
         second, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_SAGITTARIUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_FIRST",
        ],
        "Saravali reads the first decanate of Sagittarius rising twice: among the rising \
         decanates, and in ch. 46 vv. 22, 29 by the death the 22nd drekkana from it, Cancer's \
         first, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_SAGITTARIUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_THIRD",
        ],
        "Saravali reads the third decanate of Sagittarius rising twice: among the rising \
         decanates, and in ch. 46 vv. 22, 30 by the death the 22nd drekkana from it, Cancer's \
         third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_TAURUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_THIRD",
        ],
        "Saravali reads the third decanate of Taurus rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 39 by the death the 22nd drekkana from it, Sagittarius's third, \
         gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_TAURUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_SECOND",
        ],
        "Saravali reads the second decanate of Taurus rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 38 by the death the 22nd drekkana from it, Sagittarius's second, \
         gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_ARIES",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_THIRD",
        ],
        "Saravali reads the third decanate of Aries rising twice: among the rising decanates, and \
         in ch. 46 vv. 22, 37 by the death the 22nd drekkana from it, Scorpio's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_CAPRICORN",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_FIRST",
        ],
        "Saravali reads the first decanate of Capricorn rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 31 by the death the 22nd drekkana from it, Leo's first, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_PISCES",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_THIRD",
        ],
        "Saravali reads the third decanate of Pisces rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 35 by the death the 22nd drekkana from it, Libra's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_AQUARIUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_SECOND",
        ],
        "Saravali reads the second decanate of Aquarius rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 33 by the death the 22nd drekkana from it, Virgo's second, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_CAPRICORN",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_THIRD",
        ],
        "Saravali reads the third decanate of Capricorn rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 32 by the death the 22nd drekkana from it, Leo's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_PISCES",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_SECOND",
        ],
        "Saravali reads the second decanate of Pisces rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 35 by the death the 22nd drekkana from it, Libra's second, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_AQUARIUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_FIRST",
        ],
        "Saravali reads the first decanate of Aquarius rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 33 by the death the 22nd drekkana from it, Virgo's first, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_ARIES",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_FIRST",
        ],
        "Saravali reads the first decanate of Aries rising twice: among the rising decanates, and \
         in ch. 46 vv. 22, 36 by the death the 22nd drekkana from it, Scorpio's first, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_3_OF_AQUARIUS",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_THIRD",
        ],
        "Saravali reads the third decanate of Aquarius rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 34 by the death the 22nd drekkana from it, Virgo's third, gives",
    ),
    (
        &[
            "SARAVALI_DECANATE_1_OF_PISCES",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_FIRST",
        ],
        "Saravali reads the first decanate of Pisces rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 35 by the death the 22nd drekkana from it, Libra's first, gives",
    ),
    (
        &[
            "SARAVALI_MERCURY_VENUS_IN_BHAVA_1",
            "SARAVALI_STRI_VENUS_AND_MERCURY_RISING",
        ],
        "Saravali gives Mercury and Venus in the first twice: ch. 45 v. 26 in a woman's chart, \
         and among the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_DECANATE_2_OF_ARIES",
            "SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_SECOND",
        ],
        "Saravali reads the second decanate of Aries rising twice: among the rising decanates, \
         and in ch. 46 vv. 22, 36 by the death the 22nd drekkana from it, Scorpio's second, gives",
    ),
    (
        &[
            "PARIJATA_JANMA_THREE_DEBILITATED",
            "PARIJATA_YOGA_THREE_DEBILITATED",
        ],
        "Jataka Parijata gives three grahas debilitated twice: ch. 8 v. 111 among the counts of \
         grahas, and ch. 3 v. 70 among the circumstances of birth",
    ),
    (
        &[
            "SARAVALI_MOON_VENUS_IN_BHAVA_1",
            "SARAVALI_STRI_MOON_AND_VENUS_RISING",
        ],
        "Saravali gives the Moon and Venus in the first twice: ch. 45 v. 25 in a woman's chart, \
         and among the grahas in the houses",
    ),
    (
        &[
            "SARAVALI_MOON_MERCURY_IN_BHAVA_1",
            "SARAVALI_STRI_MOON_AND_MERCURY_RISING",
        ],
        "Saravali gives the Moon and Mercury in the first twice: ch. 45 v. 25 in a woman's chart, \
         and among the grahas in the houses",
    ),
];

/// Rules the sky never holds, each with an instant (a Julian day in UT)
/// and a place, its latitude and longitude, where it does: rare rules,
/// shown alive. Each was found at Kathmandu's latitude, at Kathmandu itself
/// or on the meridian that puts the lagna the rule asks for on the horizon,
/// except an eclipse, found where it is seen with the luminary just risen.
const FIXTURES: [(&str, f64, f64, f64); 230] = [
    // Brihat Jataka ch. 23's figures the computed sky misses: v. 12's two
    // eclipses at a place that sees the luminary just risen in the rising
    // sign, the Moon's in 1846 at 60° S and the Sun's in 1969 at 25° N, and
    // v. 7's two placements on an early day, each read back through
    // `cofiring-probe`.
    (
        "BJ_ANISHTA_ECLIPSED_MOON_RISING_SATURN_AND_MARS_IN_THE_TRINES",
        2_392_651.059_3,
        -60.0,
        78.0,
    ),
    (
        "BJ_ANISHTA_ECLIPSED_SUN_RISING_SATURN_AND_MARS_IN_THE_TRINES",
        2_440_476.221_8,
        25.0,
        -171.0,
    ),
    (
        "BJ_ANISHTA_MOON_RISING_SATURN_AND_MARS_ABOUT_SUN_SEVENTH",
        2_379_626.930_6,
        27.7172,
        85.324,
    ),
    (
        "BJ_ANISHTA_MOON_TENTH_MARS_SEVENTH_SATURN_SECOND_FROM_THE_SUN",
        2_379_033.125,
        27.7172,
        85.324,
    ),
    // Brihat Jataka ch. 11's figures the computed sky misses: an early day of
    // 1800 to 2400 with each, narrowed on placements and read back through
    // `cofiring-probe` (Venus at home in the fourth on 6 of 43 so narrowed).
    (
        "BJ_RAJA_VENUS_AT_HOME_IN_THE_FOURTH",
        2_409_098.236_1,
        27.7172,
        85.324,
    ),
    (
        "BJ_RAJA_ARIES_SUN_AND_MOON_RISING_FIGURE",
        2_406_723.513_9,
        27.7172,
        85.324,
    ),
    (
        "BJ_RAJA_ARIES_SUN_RISING_FIGURE",
        2_380_808.472_2,
        27.7172,
        85.324,
    ),
    (
        "BJ_RAJA_CANCER_JUPITER_RISING_FIGURE",
        2_387_738.736_1,
        27.7172,
        85.324,
    ),
    (
        "BJ_RAJA_CAPRICORN_MARS_AND_MOON_FIGURE",
        2_381_415.652_7,
        27.7172,
        85.324,
    ),
    (
        "BJ_RAJA_CAPRICORN_MARS_AND_SATURN_FIGURE",
        2_405_512.638_9,
        27.7172,
        85.324,
    ),
    (
        "BJ_RAJA_SAGITTARIUS_MOON_AND_JUPITER_FIGURE",
        2_406_714.506_9,
        27.7172,
        85.324,
    ),
    (
        "BJ_RAJA_TAURUS_MOON_RISING_FIGURE",
        2_384_950.194_4,
        27.7172,
        85.324,
    ),
    (
        "ARISHTA_JUPITER_LAGNA_FOUR_IN_SECOND",
        2_415_023.166_67,
        27.7172,
        172.13,
    ),
    (
        "ARISHTA_NODES_WITH_LUMINARIES_LAGNA_AFFLICTED",
        2_415_345.25,
        27.7172,
        -86.883,
    ),
    (
        "ARISHTA_SATURN_LAGNA_MOON_EIGHTH_JUPITER_THIRD",
        2_416_294.083_33,
        27.7172,
        105.938,
    ),
    (
        "BJ_ECLIPSED_MOON_IN_LAGNA_MARS_EIGHTH",
        2_425_046.833_33,
        27.7172,
        167.283,
    ),
    (
        "BJ_SATURN_SUN_MOON_MARS_IN_ORDER",
        2_416_033.75,
        27.7172,
        121.864,
    ),
    ("BPHS_BRAHMIN_CURSE_2", 2_431_221.0, 27.7172, 153.649),
    ("BPHS_BRAHMIN_CURSE_6", 2_386_282.5, 27.7172, 59.955),
    ("BPHS_BROTHER_CURSE_1", 2_382_500.5, 27.7172, -124.453),
    ("BPHS_BROTHER_CURSE_10", 2_500_859.0, 27.7172, 40.202),
    ("BPHS_BROTHER_CURSE_2", 2_403_064.0, 27.7172, 13.969),
    ("BPHS_BROTHER_CURSE_3", 2_385_689.5, 27.7172, 176.183),
    ("BPHS_BROTHER_CURSE_4", 2_381_655.5, 27.7172, -59.659),
    ("BPHS_BROTHER_CURSE_7", 2_431_492.0, 27.7172, -60.936),
    ("BPHS_BROTHER_CURSE_9", 2_399_106.75, 27.7172, -166.358),
    (
        "BPHS_CANCER_RISING_WITH_JUPITER_AND_MOON_LIMITLESS",
        2_404_926.5,
        27.7172,
        152.741,
    ),
    (
        "BJ_AYUR_CANCER_RISING_WITH_JUPITER_AND_MOON_BEYOND_RECKONING",
        2_404_926.5,
        27.7172,
        152.741,
    ),
    ("BPHS_MATERNAL_UNCLE_CURSE_2", 2_380_555.0, 27.7172, 173.046),
    (
        "BPHS_MOON_AND_A_BENEFIC_IN_ARUDHA_LAGNA_AND_JUPITER_SECOND",
        2_436_428.666_67,
        27.7172,
        79.059,
    ),
    ("BPHS_MOTHER_CURSE_12", 2_434_156.0, 27.7172, 72.443),
    ("BPHS_MOTHER_CURSE_13", 2_584_735.5, 27.7172, -9.26),
    ("BPHS_MOTHER_CURSE_5", 2_410_500.5, 27.7172, -49.856),
    ("BPHS_MOTHER_CURSE_6", 2_385_569.0, 27.7172, 177.583),
    ("BPHS_MOTHER_CURSE_8", 2_477_457.0, 27.7172, -90.153),
    (
        "BPHS_NO_SON_STRONG_MALEFICS_IN_THE_FIFTH",
        2_514_807.5,
        27.7172,
        -2.923,
    ),
    (
        "BPHS_SATURN_IN_A_FIFTH_OF_HIS_OWN_WITH_THE_LUMINARIES_ELEVENTH",
        2_415_964.166_67,
        27.7172,
        -95.118,
    ),
    ("BPHS_SERPENT_CURSE_5", 2_379_840.5, 27.7172, -84.943),
    ("BPHS_SPIRIT_CURSE_1", 2_383_249.5, 27.7172, -42.061),
    ("BPHS_SPIRIT_CURSE_8", 2_417_856.5, 27.7172, -121.56),
    ("BPHS_SPIRIT_CURSE_9", 2_380_791.0, 27.7172, 109.151),
    (
        "BPHS_THE_SUN_IN_LEO_AS_THE_FIFTH_WITH_THREE_ELEVENTH",
        2_524_474.333_33,
        27.7172,
        17.863,
    ),
    ("BPHS_WIFE_CURSE_10", 2_451_130.5, 27.7172, -113.242),
    ("BPHS_WIFE_CURSE_11", 2_420_394.5, 27.7172, -103.057),
    ("BPHS_WIFE_CURSE_7", 2_384_731.5, 27.7172, 103.036),
    ("BPHS_WIFE_CURSE_8", 2_400_560.0, 27.7172, 162.235),
    ("NABHASA_CHAKRA", 2_412_258.5, 27.7172, 65.74),
    ("NABHASA_CHATRA", 2_378_584.5, 27.7172, -158.98),
    ("NABHASA_GADA", 2_430_110.0, 27.7172, -93.752),
    ("NABHASA_GOLA", 2_437_699.083_33, 27.7172, -37.399),
    ("NABHASA_KAMALA", 2_420_757.166_67, 27.7172, 119.593),
    ("NABHASA_NALA", 2_420_757.166_67, 27.7172, 59.593),
    ("NABHASA_SAKATA", 2_418_976.25, 27.7172, -8.53),
    ("NABHASA_SAMUDRA", 2_412_258.5, 27.7172, 97.14),
    ("NABHASA_VIHAGA", 2_418_976.25, 27.7172, 81.47),
    ("PARASHARA_MATSYA", 2_418_336.0, 27.7172, 112.354),
    (
        "PARIJATA_JUPITER_EIGHTH_IN_A_SIGN_OF_MARS_ASPECTED_BY_FOUR",
        2_428_308.916_67,
        27.7172,
        -89.235,
    ),
    (
        "PARIJATA_ARIES_END_RISING_WITH_JUPITER_OR_VENUS_MOON_MID_TAURUS_MARS_SIMHASANA",
        2_379_731.991_82,
        27.7172,
        -107.86,
    ),
    (
        "PARIJATA_JUPITER_SATURN_ONE_NAVAMSHA_IN_THE_NINTH_OR_TENTH_SUN_RISING",
        2_429_878.5,
        27.7172,
        82.5,
    ),
    (
        "PARIJATA_JUPITER_GOPURA_IN_A_KENDRA_VENUS_PARAVATA_IN_A_TRINE_CANCER_RISING",
        2_473_202.25,
        27.7172,
        -68.985,
    ),
    (
        "PARIJATA_SAGITTARIUS_RISING_IN_ARIES_NAVAMSHA_WITH_JUPITER_VENUS_SEVENTH_MOON_IN_VIRGO",
        2_380_915.0,
        27.7172,
        41.087,
    ),
    // Jataka Parijata ch. 7's garland from the second, the seven in the
    // second to the eighth one to a house, and Sarada, the Sun strong in Leo
    // with the tenth lord in the fifth and Mars trine to Mercury, and Kurma,
    // every graha in a place of its class and a sign or navamsha it is
    // dignified in, which 28 days of 1801 to 2398 come near and one, in 2263,
    // reaches; each read back through `cofiring-probe` at Kathmandu.
    (
        "PARIJATA_MALIKA_FROM_THE_SECOND",
        2_379_544.972_2,
        27.7172,
        85.324,
    ),
    ("PARIJATA_SARADA", 2_390_777.222_2, 27.7172, 85.324),
    ("PARIJATA_KURMA", 2_547_710.312_5, 27.7172, 85.324),
    // The garland from the fifth on: one sky of 1802 with the seven in Gemini
    // to Scorpio, one to a sign, read at the longitudes that raise the lagna
    // each garland needs.
    (
        "PARIJATA_MALIKA_FROM_THE_FIFTH",
        2_379_544.784_7,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_MALIKA_FROM_THE_SEVENTH",
        2_379_544.784_7,
        27.7172,
        45.0,
    ),
    (
        "PARIJATA_MALIKA_FROM_THE_EIGHTH",
        2_379_544.784_7,
        27.7172,
        12.0,
    ),
    (
        "PARIJATA_MALIKA_FROM_THE_NINTH",
        2_379_544.784_7,
        27.7172,
        -22.0,
    ),
    (
        "PARIJATA_MALIKA_FROM_THE_ELEVENTH",
        2_379_544.784_7,
        27.7172,
        -92.0,
    ),
    // All five great men at once: Mars in Aries, Venus in Libra and Saturn in
    // Capricorn angular to an Aries lagna, Mercury in Virgo and Jupiter in
    // Sagittarius angular to a Gemini Moon; 9 days of 1800 to 2400 have it.
    (
        "PHALADEEPIKA_MAHAPURUSHA_FIVE",
        2_545_713.0,
        27.7172,
        85.324,
    ),
    // v. 70's two, which only Leo, Virgo, Sagittarius and Pisces rising can
    // hold (C471): of 4820 two-hourly instants of 1801 to 2398 at Kathmandu
    // with every lord placed as the verse asks, 108 also have the strengths
    // for the Duryoga and 13 for the king. A Leo lagna on 1823-01-15 and a
    // Virgo lagna on 1964-02-14.
    (
        "PHALADEEPIKA_DURYOGA_OF_THE_LORDS",
        2_386_911.083_3,
        27.7172,
        85.324,
    ),
    (
        "PHALADEEPIKA_KING_BY_THE_LORDS",
        2_438_440.166_8,
        27.7172,
        85.324,
    ),
    // Phaladeepika ch. 7's figures the computed sky misses, each found by
    // narrowing instants on placements and asking `cofiring-probe`.
    // The Sun and Mars in Aries, the Moon and Jupiter in Cancer and Saturn in
    // Capricorn, angular to one another: 47 quarter-days of 1801 to 2398 have
    // five so, and an angular lagna 51 of their 216 twenty-minute steps here.
    (
        "PHALADEEPIKA_MAHARAJA_FIVE_DIGNIFIED_IN_KENDRAS",
        2_426_839.013_9,
        27.7172,
        85.324,
    ),
    // The full Moon and three strong grahas in their own navamshas, each
    // aspected by a benefic: 3 of 9000 instants narrowed on placements.
    (
        "PHALADEEPIKA_MAHARAJA_FULL_MOON_THREE_IN_OWN_NAVAMSHAS",
        2_379_392.291_7,
        27.7172,
        85.324,
    ),
    // With Venus opposite the Moon the Sun is her temporary enemy, so only
    // Mercury in the fourth or tenth from her, the Moon in his navamsha, makes
    // its lord a great friend: 1865 of the 2057 half-hours so placed hold it.
    (
        "PHALADEEPIKA_MAHARAJA_MOON_IN_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_VENUS",
        2_381_876.416_7,
        27.7172,
        85.324,
    ),
    // Mercury, Jupiter and Venus bright in kendras at night, the one state of
    // the verse's three they can share: 25 of 1800 narrowed instants.
    (
        "PHALADEEPIKA_MAHARAJA_NIGHT_MOON_IN_CANCER_TENTH",
        2_383_287.25,
        27.7172,
        85.324,
    ),
    // The Sun and the Moon in Sagittarius, Saturn rising and Mars exalted: 15
    // of 1800 narrowed instants.
    (
        "PHALADEEPIKA_MAHARAJA_SAGITTARIUS_FIGURE",
        2_381_414.333_3,
        27.7172,
        85.324,
    ),
    // Venus rising in Ashvini under the aspect of three of the Moon, Mars,
    // Jupiter and Saturn: 5 of 1800 narrowed instants.
    (
        "PHALADEEPIKA_MAHARAJA_VENUS_IN_ASHVINI_RISING",
        2_380_836.402_8,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_MARS_OR_SATURN_TRINE_FROM_VENUS_OR_THE_MOON",
        2_417_139.0,
        27.7172,
        -89.502,
    ),
    (
        "PARIJATA_SATURN_IN_A_JUPITER_NAVAMSHA_LAGNA_LORD_EXALTED",
        2_416_949.0,
        27.7172,
        48.39,
    ),
    (
        "PARIJATA_SUN_SATURN_MARS_IN_LAGNA_WANING_MOON_IN_A_VENUS_SEVENTH",
        2_425_223.25,
        27.7172,
        169.281,
    ),
    (
        "PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER_VENUS_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "PARIJATA_TOGETHER_SUN_MOON_MARS_JUPITER_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "PARIJATA_TOGETHER_SUN_MOON_MARS_JUPITER_VENUS_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_JUPITER_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "PHALADEEPIKA_WEAK_MOON_IN_A_DUSTHANA",
        2_450_123.749_13,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_JUPITER_IN_EIGHTH_IN_A_SIGN_OF_MARS",
        2_428_308.916_67,
        27.7172,
        -89.235,
    ),
    (
        "SARAVALI_MARS_SUN_SATURN_IN_TAURUS_AS_EIGHTH",
        2_451_702.416_67,
        27.7172,
        -97.355,
    ),
    (
        "SARAVALI_RAJA_ALL_VARGOTTAMA_OR_AT_HOME",
        2_414_577.972_2,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_ARIES_RISING_WITH_MOON_MARS_AND_JUPITER",
        2_382_242.534_7,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_BENEFICS_IN_THE_FIRST_FOURTH_SEVENTH_MALEFICS_THIRD_NINTH_ELEVENTH",
        2_383_809.312_5,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_EXALTED_SUN_TENTH_WITH_JUPITER_VENUS_NINTH",
        2_386_658.715_3,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_FULL_MOON_IN_A_TAURUS_NAVAMSHA_ASPECTED_BY_ALL",
        2_409_719.152_8,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_JUPITER_AND_MOON_FIFTH_THREE_IN_OWN_NAVAMSHAS",
        2_385_461.138_9,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_JUPITER_FOURTH_AT_HOME_FULL_MOON_NINTH",
        2_554_453.493_1,
        27.7172,
        85.324,
    ),
    ("SARAVALI_RAJA_KALASHA", 2_387_002.743_1, 27.7172, 85.324),
    (
        "SARAVALI_RAJA_LUMINARIES_EARLY_SAGITTARIUS_SATURN_RISING",
        2_387_615.888_9,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_MERCURY_RISING_JUPITER_SEVENTH_MOON_FOURTH_IN_CANCER",
        2_436_292.555_6,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_MOON_AT_HIGHEST_EXALTATION_ASPECTED_BY_VENUS",
        2_384_649.555_6,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_SUN_AT_HIGHEST_EXALTATION_RISING",
        2_378_971.5,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_TAURUS_RISING_JUPITER_AND_MOON",
        2_391_343.694_4,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_VENUS_MERCURY_MARS_RISING_MOON_AND_JUPITER_SEVENTH",
        2_392_442.493_1,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_VENUS_MOON_SUN_THIRD_MARS_SEVENTH_JUPITER_NINTH",
        2_413_189.319_4,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_VENUS_RISING_IN_A_SIGN_OF_JUPITER_MERCURY_EXALTED",
        2_404_003.687_5,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_MOON_SIGN_LORD_IN_AN_UPACHAYA_FROM_THE_MOON",
        2_379_895.258_6,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_BENEFICS_SECOND_MOON_AND_MARS_SEVENTH",
        2_400_648.319_4,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_CAPRICORN_MARS_RISING_SATURN_NINTH_LUMINARIES_SEVENTH",
        2_412_303.069_4,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_CAPRICORN_RISING_MARS_RISING_BENEFICS_FOURTH",
        2_395_064.222_2,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_CAPRICORN_SATURN_RISING_SUN_SEVENTH_VENUS_EIGHTH",
        2_480_990.020_8,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_MARS_TENTH_MOON_AND_VENUS_NINTH_EXALTED_SUN_ELEVENTH",
        2_525_076.659_7,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_MOON_AND_JUPITER_IN_THE_FIRST_FIVE_DEGREES_OF_CANCER",
        2_378_916.256_9,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_SATURN_AND_MOON_RISING_JUPITER_SEVENTH_MERCURY_EXALTED",
        2_411_977.493_1,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_BRIGHT_LAGNA_LORD_MALEFICS_ELEVENTH_BENEFICS_IN_KENDRAS",
        2_387_779.645_8,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_RAJA_MERCURY_AND_SATURN_IN_KENDRAS_FROM_VENUS_THE_REST_THIRD_OR_ELEVENTH",
        2_403_424.361_1,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_TOGETHER_MOON_MARS_MERCURY_JUPITER_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "SARAVALI_TOGETHER_MOON_MARS_MERCURY_JUPITER_VENUS_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "SARAVALI_TOGETHER_SUN_MOON_MARS_JUPITER_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "SARAVALI_TOGETHER_SUN_MOON_MARS_JUPITER_VENUS_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    (
        "SARAVALI_TOGETHER_SUN_MOON_MARS_MERCURY_JUPITER_SATURN",
        2_437_699.083_33,
        27.7172,
        -37.399,
    ),
    // Jataka Parijata ch. 7 vv. 1 to 127: found by sign day by day, 1800 to
    // 2398, and confirmed every ten minutes of the day at Kathmandu.
    (
        "PARIJATA_RAJA_SUN_MOON_MERCURY_VENUS_TENTH_IN_FRIENDS_NAVAMSHAS",
        2_527_530.693_2,
        27.7172,
        85.324,
    ),
    ("PARIJATA_BUDHA", 2_393_193.083_3, 27.7172, 85.324),
    (
        "PARIJATA_RAJA_FIVE_EXALTED_JUPITER_RISING",
        2_413_666.743_1,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_FULL_MOON_NINTH_SATURN_TENTH_MARS_SECOND",
        2_379_244.368_1,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_FULL_MOON_RISING_IN_PISCES_MARS_EXALTED_SATURN_AQUARIUS",
        2_417_158.881_9,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_LUMINARIES_EARLY_SAGITTARIUS_STRONG_SATURN_RISING_MARS_EXALTED",
        2_387_615.888_9,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_MARS_CAPRICORN_SUN_ARIES_JUPITER_AQUARIUS",
        2_390_285.263_9,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_MERCURY_RISING_JUPITER_SEVENTH_BRIGHT_MOON_FOURTH_IN_CANCER_VENUS_TENTH",
        2_436_292.555_6,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_MOON_RISING_JUPITER_FOURTH_VENUS_TENTH_SATURN_DIGNIFIED",
        2_380_915.597_2,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_ONE_EACH_IN_THE_FIRST_SECOND_THIRD_FOURTH_FIFTH_SEVENTH_NINTH",
        2_381_967.229_2,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_SATURN_RISING_IN_AQUARIUS_FOUR_EXALTED",
        2_406_725.388_9,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_STRONG_MARS_RISING_IN_CAPRICORN_SATURN_NINTH_OR_TWELFTH_LUMINARIES_SEVENTH",
        2_383_452.027_8,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_SUN_AND_MERCURY_FOURTH_SATURN_AND_MOON_TENTH_MARS_RISING",
        2_385_710.263_9,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_SUN_IN_A_TRIKONA_IN_PISCES_MOON_AND_JUPITER_IN_CANCER",
        2_378_944.263_9,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_SUN_RISING_IN_SAGITTARIUS_MARS_AND_MOON_TENTH_VENUS_ELEVENTH_OR_TWELFTH",
        2_387_253.534_7,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_TAURUS_MOON_RISING_ASPECTED_BY_ALL",
        2_454_074.937_5,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_RAJA_VENUS_IN_ASHWINI_RISING",
        2_380_836.388_9,
        27.7172,
        85.324,
    ),
    // Brihat Jataka ch. 12: the Nabhasa yogas BPHS ch. 35 states alike hold
    // where BPHS's do.
    ("BJ_NABHASA_NALA", 2_420_757.166_67, 27.7172, 59.593),
    ("BJ_NABHASA_GADA", 2_430_110.0, 27.7172, -93.752),
    ("BJ_NABHASA_SAKATA", 2_418_976.25, 27.7172, -8.53),
    ("BJ_NABHASA_VIHAGA", 2_418_976.25, 27.7172, 81.47),
    ("BJ_NABHASA_KAMALA", 2_420_757.166_67, 27.7172, 119.593),
    ("BJ_NABHASA_CHHATRA", 2_378_584.5, 27.7172, -158.98),
    ("BJ_NABHASA_SAMUDRA", 2_412_258.5, 27.7172, 97.14),
    ("BJ_NABHASA_CHAKRA", 2_412_258.5, 27.7172, 65.74),
    ("BJ_NABHASA_GOLA", 2_437_699.083_33, 27.7172, -37.399),
    // Brihat Jataka ch. 15 v. 2: the leader combust, found by sign day by
    // day, 1800 to 2398, and confirmed at Kathmandu.
    (
        "BJ_PRAVRAJYA_MARS_COMBUST",
        2_379_202.944_4,
        27.7172,
        85.324,
    ),
    ("BJ_PRAVRAJYA_SATURN_COMBUST", 2_379_857.0, 27.7172, 85.324),
    // Jataka Parijata chs. 5A and 6: found by sign day by day, 1800 to
    // 2398, or drawn over 1900 to 2100, and confirmed at Kathmandu.
    (
        "PARIJATA_ANGA_LAST_NAVAMSHA_OF_CAPRICORN_RISING_ASPECTED_BY_SATURN_MOON_SUN",
        2_378_699.097_2,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_ANGA_LAST_NAVAMSHA_OF_CAPRICORN_RISING_MALEFICS_IN_ITS_THREE_DREKKANAS",
        2_426_740.569_4,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_ANGA_LEO_RISING_WITH_THE_LUMINARIES_ASPECTED_BY_SATURN_AND_MARS",
        2_384_931.5,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_ANGA_LUMINARIES_TWELFTH_MALEFICS_SIXTH_AND_EIGHTH_BOTH_EYES",
        2_379_584.604_2,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_ANGA_MARS_IN_THE_FIRST_DREKKANA_OF_THE_FIFTH_ASPECTED_BY_SATURN_SUN_MOON",
        2_403_740.666_7,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_ANGA_MARS_IN_THE_FIRST_DREKKANA_OF_THE_LAGNA_ASPECTED_BY_SATURN_SUN_MOON",
        2_403_741.0,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_ANGA_MOON_SIXTH_SUN_EIGHTH_SATURN_TWELFTH_MARS_SECOND",
        2_395_967.888_9,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_ANGA_MOON_TENTH_MARS_SEVENTH_SUN_SECOND",
        2_378_835.375,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_DARIDRA_LAGNA_LORD_TWELFTH_MARS_WITH_THE_MOON_IN_A_CRUEL_TENTH",
        2_378_631.5,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_DARIDRA_NIGHT_MOVABLE_LAGNA_WEAK_BENEFICS_IN_KENDRAS_AND_TRIKONAS",
        2_452_872.162_6,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_DARIDRA_SUN_AT_HIS_DEEPEST_FALL",
        2_415_318.5,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_DARIDRA_SUN_IN_ARIES_IN_A_LIBRA_NAVAMSHA_ASPECTED_BY_A_MALEFIC",
        2_378_617.5,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_DARIDRA_VENUS_FIRST_JUPITER_FIFTH_MARS_ELEVENTH_MOON_THIRD_ALL_FALLEN",
        2_416_028.5,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_DARIDRA_VENUS_JUPITER_MOON_MARS_FALLEN_IN_FOUR_GOOD_HOUSES",
        2_416_028.826_4,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_MARANA_MOON_IN_VIRGO_SEVENTH_WITH_A_MALEFIC_VENUS_IN_ARIES_SUN_RISING",
        2_380_041.5,
        27.7172,
        85.324,
    ),
    (
        "BJ_MARANA_MOON_IN_VIRGO_SEVENTH_WITH_A_MALEFIC_VENUS_IN_ARIES_SUN_RISING",
        2_380_041.5,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_MARANA_SATURN_SECOND_MOON_FOURTH_MARS_TENTH_WORMS_IN_A_WOUND",
        2_378_648.506_9,
        27.7172,
        85.324,
    ),
    (
        "BJ_MARANA_SATURN_SECOND_MOON_FOURTH_MARS_TENTH_WORMS_IN_A_WOUND",
        2_378_648.506_9,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_MARANA_SUN_AND_MARS_SEVENTH_SATURN_EIGHTH_WANING_MOON_FOURTH_BIRDS",
        2_386_221.965_3,
        27.7172,
        85.324,
    ),
    (
        "BJ_MARANA_SUN_AND_MARS_SEVENTH_SATURN_EIGHTH_WANING_MOON_FOURTH_BIRDS",
        2_386_221.965_3,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_MARANA_SUN_FIRST_MARS_FIFTH_SATURN_EIGHTH_MOON_NINTH_A_FALL",
        2_378_894.5,
        27.7172,
        85.324,
    ),
    (
        "BJ_MARANA_SUN_FIRST_MARS_FIFTH_SATURN_EIGHTH_MOON_NINTH_A_FALL",
        2_378_894.5,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_MARANA_SUN_FOURTH_MARS_TENTH_ASPECTED_BY_THE_WANING_MOON_IMPALED",
        2_380_352.222_2,
        27.7172,
        85.324,
    ),
    (
        "BJ_MARANA_SUN_FOURTH_MARS_TENTH_ASPECTED_BY_THE_WANING_MOON_IMPALED",
        2_380_352.222_2,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_PRESHYA_VENUS_NINTH_MOON_SEVENTH_JUPITER_SECOND_OR_FIRST_MARS_EIGHTH_FIXED_LAGNA",
        2_380_934.722_2,
        27.7172,
        85.324,
    ),
    (
        "PARIJATA_VENUS_FALLEN_OR_SATURNIAN_IN_A_DUSTHANA_ASPECTED_BY_SATURN_LUMINARIES_SEVENTH",
        2_378_875.979_2,
        27.7172,
        85.324,
    ),
    (
        "BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_VENUS_AND_THE_MOON",
        2_380_000.25,
        27.7172,
        29.306,
    ),
    (
        "PHALADEEPIKA_KALATRA_VENUS_AND_MOON_SEVENTH_FROM_MARS_AND_SATURN",
        2_406_841.5,
        27.7172,
        -21.251,
    ),
    (
        "PHALADEEPIKA_PUTRA_SUN_FIFTH_IN_A_SIGN_OF_FEW_CHILDREN_SATURN_EIGHTH_MARS_RISING",
        2_436_432.5,
        27.7172,
        -26.638,
    ),
    (
        "PHALADEEPIKA_PUTRA_VENUS_AND_MERCURY_SEVENTH_JUPITER_FIFTH_MALEFIC_FOURTH",
        2_379_520.5,
        27.7172,
        -67.337,
    ),
    (
        "PARIJATA_GARBHA_MALE_HERMAPHRODITE_SATURN_WITH_MERCURY_SIXTH",
        2_413_883.5,
        27.7172,
        -64.469,
    ),
    (
        "PARIJATA_GARBHA_MANY_LAGNA_IN_SAGITTARIUS_LAST_NAVAMSHA",
        2_380_862.5,
        27.7172,
        -59.061,
    ),
    (
        "PARIJATA_GARBHA_TRIPLETS_TWO_BOYS_AND_A_GIRL",
        2_404_407.5,
        27.7172,
        -147.429,
    ),
    (
        "PARIJATA_GARBHA_TRIPLETS_TWO_GIRLS_AND_A_BOY",
        2_382_381.5,
        27.7172,
        -38.206,
    ),
    (
        "PARIJATA_GARBHA_TWIN_BOYS_FIVE_IN_GEMINI_OR_SAGITTARIUS_NAVAMSHAS",
        2_411_498.5,
        27.7172,
        68.733,
    ),
    (
        "PARIJATA_GARBHA_TWIN_GIRLS_FIVE_IN_VIRGO_OR_PISCES_NAVAMSHAS",
        2_407_511.5,
        27.7172,
        38.386,
    ),
    (
        "PARIJATA_RAJA_FIVE_IN_OWN_SIGNS",
        2_384_900.5,
        27.7172,
        4.173,
    ),
    // BPHS chs. 12 to 23 and Saravali chs. 20, 38, 39, 44 and 46: found by
    // sign day by day, 1800 to 2398, and confirmed at the place each names.
    (
        "BPHS_BHAGYA_SUN_DEEP_EXALTED_NINTH_LORD_ELEVENTH",
        2_378_971.5,
        27.7172,
        -145.651,
    ),
    (
        "BPHS_BHAGYA_VENUS_DEEP_EXALTED_WITH_THE_NINTH_LORD_SATURN_THIRD",
        2_385_162.5,
        27.7172,
        9.244,
    ),
    (
        "BPHS_DHANA_JUPITER_ELEVENTH_VENUS_SECOND_BENEFICS_WITH_THE_SECOND_LORD_AND_IN_THE_TWELFTH",
        2_387_114.5,
        27.7172,
        108.579,
    ),
    (
        "BPHS_DHANA_SECOND_AND_ELEVENTH_LORDS_IN_DUSTHANAS_MARS_ELEVENTH_RAHU_SECOND",
        2_381_651.5,
        27.7172,
        126.853,
    ),
    (
        "BPHS_KARMA_SUN_ELEVENTH_WITH_RAHU_MARS_AND_SATURN",
        2_390_045.5,
        27.7172,
        167.89,
    ),
    (
        "BPHS_PUTRA_JUPITER_DEEP_EXALTED_SECOND_LORD_WITH_RAHU_NINTH_LORD_NINTH",
        2_400_571.5,
        27.7172,
        -126.567,
    ),
    (
        "BPHS_PUTRA_MALEFICS_FOURTH_AND_SIXTH_FIFTH_LORD_DEEP_EXALTED_WITH_THE_LAGNA_LORD",
        2_388_627.5,
        27.7172,
        -43.29,
    ),
    (
        "BPHS_PUTRA_NINTH_LORD_RISING_FIFTH_LORD_DEBILITATED_KETU_AND_MERCURY_FIFTH",
        2_398_851.5,
        27.7172,
        -8.867,
    ),
    (
        "BPHS_PUTRA_SIX_GRAHAS_FIFTH_ITS_LORD_TWELFTH_LAGNA_LORD_AND_MOON_STRONG",
        2_540_998.5,
        27.7172,
        -34.632,
    ),
    (
        "BPHS_ROGA_FIFTH_LORD_SIXTH_SIXTH_LORD_WITH_JUPITER_TWELFTH_LORD_RISING",
        2_384_552.5,
        27.7172,
        -56.53,
    ),
    (
        "BPHS_VYAYA_RAHU_TWELFTH_WITH_MARS_SATURN_AND_THE_SUN_ITS_LORD_WITH_THE_SUN",
        2_414_974.5,
        27.7172,
        146.319,
    ),
    (
        "SARAVALI_AYUR_CANCER_RISING_WITH_JUPITER_AND_MOON_UNLIMITED",
        2_404_926.5,
        27.7172,
        158.281,
    ),
    (
        "SARAVALI_BHANGA_AQUARIUS_RISING_THREE_DEBILITATED_JUPITER_WITH_THE_SUN_NONE_EXALTED",
        2_385_812.5,
        27.7172,
        141.59,
    ),
    (
        "SARAVALI_BHANGA_LAGNA_LORD_HOSTILE_TO_THE_MOON_SUN_THIRD_MARS_SIXTH_SATURN_SEVENTH",
        2_393_713.5,
        27.7172,
        15.17,
    ),
    (
        "SARAVALI_BHANGA_MALEFICS_IN_KENDRAS_FALLEN_OR_HOSTILE_BENEFICS_HIDDEN",
        2_431_765.5,
        27.7172,
        -106.764,
    ),
    (
        "SARAVALI_BHANGA_SUN_DEEPLY_DEBILITATED_OTHERS_DIGNIFIED",
        2_380_254.5,
        27.7172,
        -96.651,
    ),
    (
        "SARAVALI_BHANGA_SUN_IN_THE_TENTH_DEGREE_OF_LIBRA",
        2_378_792.5,
        27.7172,
        -95.678,
    ),
    ("SARAVALI_FIVE_DEBILITATED", 2_455_125.5, 27.7172, -90.773),
    (
        "SARAVALI_FIVE_IN_MOOLATRIKONA",
        2_562_831.5,
        27.7172,
        -47.561,
    ),
    ("SARAVALI_FIVE_IN_OWN_SIGNS", 2_394_381.5, 27.7172, 19.537),
    (
        "SARAVALI_FOUR_IN_MOOLATRIKONA",
        2_430_250.5,
        27.7172,
        -53.554,
    ),
    (
        "SARAVALI_MARANA_MARS_TENTH_MOON_FOURTH_SATURN_SECOND_WORMS_IN_A_WOUND",
        2_378_648.5,
        27.7172,
        103.298,
    ),
    (
        "SARAVALI_MARANA_PISCES_RISING_SUN_MOON_SEVENTH_WITH_A_MALEFIC_VENUS_IN_ARIES",
        2_380_041.5,
        27.7172,
        91.114,
    ),
    (
        "SARAVALI_MARANA_SUN_AND_MARS_SEVENTH_SATURN_EIGHTH_MOON_FOURTH_BIRDS",
        2_386_221.5,
        27.7172,
        -91.278,
    ),
    (
        "SARAVALI_MARANA_SUN_FOURTH_MARS_TENTH_WITH_THE_WANING_MOON_ASPECTED_BY_SATURN_WOOD",
        2_391_264.5,
        27.7172,
        12.237,
    ),
    (
        "SARAVALI_MARANA_SUN_MARS_SATURN_MOON_IN_THE_LAGNA_EIGHTH_AND_TRINES_A_FALL",
        2_378_894.5,
        27.7172,
        96.063,
    ),
    (
        "SARAVALI_MARANA_WANING_MOON_EIGHTH_MARS_TENTH_SUN_FIRST_SATURN_FOURTH_CLUBBED",
        2_378_597.5,
        27.7172,
        96.517,
    ),
    (
        "SARAVALI_MARANA_WANING_MOON_TENTH_MARS_NINTH_SUN_FIRST_SATURN_FIFTH_SMOKE_OR_FIRE",
        2_385_009.5,
        27.7172,
        91.388,
    ),
    (
        "SARAVALI_PRAVRAJYA_MOON_MARS_MERCURY_JUPITER_SATURN",
        2_379_849.5,
        27.7172,
        -57.476,
    ),
    (
        "SARAVALI_PRAVRAJYA_STRONG_MOON_TENTH_IN_A_BENEFICS_NAVAMSHA_ASPECTED_BY_STRONG_SATURN",
        2_389_559.5,
        27.7172,
        92.185,
    ),
    (
        "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_JUPITER_VENUS_SATURN",
        2_386_257.5,
        27.7172,
        106.689,
    ),
    (
        "SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY_JUPITER_SATURN",
        2_379_849.5,
        27.7172,
        -57.476,
    ),
    (
        "SARAVALI_RAJA_SUN_VIRGO_MARS_JUPITER_SCORPIO_MOON_ARIES_UNASPECTED",
        2_480_300.5,
        27.7172,
        -63.669,
    ),
    ("SARAVALI_SIX_DEBILITATED", 2_558_865.5, 27.7172, -98.605),
    // Saravali ch. 20's strong sets and strongest leaders: drawn over 1900
    // to 2100, Brihat Jataka ch. 15 v. 2's own instants, or the half hours
    // about a day the sign search found, at Kathmandu.
    (
        "SARAVALI_PRAVRAJYA_MARS_STRONGEST_ASPECTED_BY_ANOTHER",
        2_447_915.521_5,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_PRAVRAJYA_MARS_STRONGEST_COMBUST",
        2_379_202.944_4,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_PRAVRAJYA_SATURN_STRONGEST_ASPECTED_BY_ANOTHER",
        2_478_116.825_8,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_PRAVRAJYA_SATURN_STRONGEST_COMBUST",
        2_379_857.0,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_PRAVRAJYA_STRONG_SUN_MARS_JUPITER_SATURN",
        2_451_658.099_5,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_PRAVRAJYA_STRONG_MARS_MERCURY_JUPITER_VENUS_SATURN",
        2_386_253.708_3,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_PRAVRAJYA_STRONG_MOON_MARS_MERCURY_JUPITER_VENUS_SATURN",
        2_386_256.75,
        27.7172,
        85.324,
    ),
    (
        "SARAVALI_PRAVRAJYA_STRONG_SUN_MARS_MERCURY_JUPITER_VENUS_SATURN",
        2_386_253.708_3,
        27.7172,
        85.324,
    ),
    // Saravali ch. 45, Jataka Parijata ch. 3 vv. 61 to 79 and BPHS ch. 80: the
    // half hours about a day the sign search found, at the place it names.
    (
        "BPHS_STRI_MOON_VENUS_SATURN_AND_MARS_RISING_FIFTH_TOUCHED_BY_A_MALEFIC",
        2_381_300.5,
        27.7172,
        146.992,
    ),
    (
        "PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_VENUS_AND_THE_MOON",
        2_380_000.5,
        27.7172,
        -60.941,
    ),
    (
        "SARAVALI_STRI_STRONG_MERCURY_JUPITER_VENUS_WITH_THE_MOON_IN_AN_EVEN_LAGNA",
        2_382_682.5,
        27.7172,
        56.166,
    ),
];

/// Rules the sky never holds and no instant searched does, each with why.
const UNSEEN: [(&str, &str); 83] = [
    (
        "ARISHTA_ALL_IN_APOKLIMAS_WITHOUT_STRENGTH",
        "It asks all seven below their required Shadbala at once, the Moon among them, and her Shadbala rarely falls below her six rupas: no instant from 1900 to 2100 with the seven in apoklimas has every one weak.",
    ),
    (
        "BJ_RAJA_AQUARIUS_ARIES_TAURUS_FIGURE",
        "It puts the Sun in Aries and Mercury in Gemini, two signs on, more than 30 degrees from the Sun, and Mercury never stands 28 degrees from him.",
    ),
    (
        "BJ_RAJA_CAPRICORN_SATURN_RISING_FIGURE",
        "No sky from 1801 to 2398 has it; the nearest a search from 1000 BC found is in 483 BC.",
    ),
    (
        "BJ_RAJA_CAPRICORN_SATURN_RISING_LORDS_AT_HOME_FIGURE",
        "It puts the Sun in Leo and Mercury in Gemini, two signs back, more than 30 degrees from the Sun, and Mercury never stands 28 degrees from him.",
    ),
    (
        "BJ_RAJA_JUPITER_RISING_MOON_TENTH",
        "It puts Mercury in the second and the Sun in the fourth, more than 30 degrees apart, and Mercury never stands 28 degrees from the Sun.",
    ),
    (
        "BJ_RAJA_LIBRA_TAURUS_FIGURE",
        "No sky from 1000 BC to AD 2400 has its seven grahas so with Libra or Taurus rising, searched day by day and every ten minutes of a day its slow grahas allow.",
    ),
    (
        "BJ_RAJA_MARS_AND_SATURN_RISING",
        "No sky from 1000 BC to AD 2400 has its seven grahas in those houses, searched day by day and every ten minutes of a day its slow grahas allow.",
    ),
    (
        "BJ_RAJA_PISCES_MOON_RISING_FIGURE",
        "No sky from 1801 to 2398 has it; the nearest a search from 1000 BC found is in 980 BC.",
    ),
    (
        "BJ_RAJA_SATURN_RISING_LUMINARIES_TENTH",
        "No sky from 1801 to 2398 has it; the nearest a search from 1000 BC found is in AD 1250.",
    ),
    (
        "BJ_RAJA_TAURUS_MOON_RISING_OTHERS_ELEVENTH",
        "No sky from 1000 BC to AD 2400 has its seven grahas in those houses, searched day by day and every ten minutes of a day its slow grahas allow.",
    ),
    (
        "BJ_RAJA_VIRGO_MERCURY_RISING_FIFTH_AND_FOURTH_FIGURE",
        "Venus three signs from Mercury needs both near their greatest elongations on either side of the Sun, and no sky from 1000 BC to AD 2400 has it with the other five so.",
    ),
    (
        "BJ_RAJA_VIRGO_MERCURY_RISING_PISCES_FIGURE",
        "Venus in Gemini three signs from Mercury in Virgo needs both near their greatest elongations on either side of the Sun, and no sky from 1000 BC to AD 2400 has it with the other five so.",
    ),
    (
        "BPHS_BRAHMIN_CURSE_1",
        "Of the 529 half-days from 1800 to 2398 with Jupiter, Mars and Saturn together and Rahu in a sign of Jupiter, the ninth lord, counted from a lagna four signs behind them, stands in the fifth itself or in the third or fourth, never the eighth.",
    ),
    (
        "BPHS_DEBILITATED_GRAHAS_SIXTH_EIGHTH_THIRD_AND_LAGNA_LORD_DIGNIFIED",
        "Only Leo and Aquarius rising have a sign of debilitation in each of the third, sixth and eighth, and in both the graha debilitated in the third is the lagna lord, the Sun in Libra or Saturn in Aries, whom the verse wants exalted or in his own sign.",
    ),
    (
        "BPHS_LAGNA_LORD_NEUTRAL_OF_THE_SUN_MEDIUM",
        "No lagna lord regards the Sun as a neutral: the Moon, Mars, Mercury and Jupiter count him a friend, Venus and Saturn an enemy, and for Leo rising the lord is the Sun himself.",
    ),
    (
        "BPHS_MATERNAL_UNCLE_CURSE_1",
        "It asks Mercury, Jupiter, Mars and Rahu in one sign with Saturn four signs behind them: of the 361 half-days from 1800 to 2398 with the four together, none has Saturn so.",
    ),
    (
        "BPHS_MATERNAL_UNCLE_CURSE_4",
        "With the Moon, Mercury and Mars in the fifth, the sixth or the twelfth lord is one of them under ten lagnas, and under Virgo and Pisces the Sun in the lagna would stand four signs from Mercury; the translation's fourth lord fails the same way (C390).",
    ),
    (
        "BPHS_MERCURY_IN_A_FIFTH_OF_HIS_OWN_WITH_THREE_ELEVENTH",
        "It asks Mars and Jupiter together in the sign opposite Mercury in his own, so both near opposition to the Sun, with the Moon there too: no day from 1800 to 2400, the span the built-in ephemeris covers, has the slower grahas so.",
    ),
    (
        "BPHS_MOTHER_CURSE_7",
        "Only Cancer and Scorpio rising leave its five lords' places consistent: of the 78 half-days from 1800 to 2398 with Jupiter and Saturn in Cancer and Mars in Sagittarius, none has Venus in Aquarius, and of the 54 with Mars and Jupiter in Aries and Saturn in Gemini, none has Mercury in Scorpio.",
    ),
    (
        "BPHS_MOTHER_CURSE_9",
        "The thin Moon in the seventh keeps the Sun and Mercury between the fourth and the tenth, and Saturn, Rahu and Ketu stand in the fifth, the fourth and the tenth, so Mars alone is left to hem the lagna, which needs a malefic on each side (C388).",
    ),
    (
        "BPHS_SERPENT_CURSE_7",
        "It asks six grahas in the fifth, Rahu, Jupiter, Saturn and Mars among them: no half-day from 1800 to 2398, the span the built-in ephemeris covers, has even the Sun, Mars, Jupiter, Saturn and Rahu in one sign.",
    ),
    (
        "BPHS_SIX_GRAHAS_EXALTED",
        "Mercury exalted in Virgo and Venus in Pisces would stand opposite, which the two never do, so the six are the Sun in Aries, the Moon in Taurus, Mars in Capricorn, Jupiter in Cancer, Venus in Pisces and Saturn in Libra: no day from 1800 to 2400 has the slower grahas so.",
    ),
    (
        "BPHS_SPIRIT_CURSE_3",
        "It asks the Sun in the twelfth and Mercury in the fifth, five signs apart, and Mercury keeps within a sign of the Sun (C390).",
    ),
    (
        "BPHS_SPIRIT_CURSE_5",
        "The lagna lord in the eighth is none of the five in the lagna, and the Sun or Mercury there would stand five signs from Venus, so it is Mars, with Aries or Scorpio rising: no half-day from 1800 to 2398 has Jupiter, Saturn and Rahu together in either.",
    ),
    (
        "PARASHARA_KOORMA",
        "It asks the Sun, Mars and Saturn each in its own or exaltation sign in the lagna, third and eleventh, with a benefic in each of the fifth, sixth and seventh: no day from 1800 to 2400 has the grahas other than the Moon so.",
    ),
    (
        "PARIJATA_MOON_IN_LEO_SUN_AND_SATURN_EIGHTH_VENUS_ASPECTING",
        "Venus casts only the seventh aspect and never stands seventh from the Sun. The verse holds under the partial aspects Jataka Parijata ch. 2 counts, which the condition language does not read.",
    ),
    (
        "PARIJATA_SATURN_DEVALOKA_MARS_PARAVATA_JUPITER_SIMHASANA_RISING",
        "Saturn in seven or eight of his own vargas of the ten and Mars in six meet on about thirty days from 1800 to 2398 (the half-daily sky), and Jupiter then holds at most four of his, never the five the verse asks.",
    ),
    (
        "PARIJATA_WANING_MOON_EIGHTH_LORD_WITH_A_MALEFIC_WEAK_LAGNA",
        "A lagna's strength is its bhava bala, which has no required figure for it to fall short \
         of, so no chart carries a weak lagna.",
    ),
    (
        "PHALADEEPIKA_MAHARAJA_PISCES_FIGURE",
        "Of the 68 days from 1801 to 2398 with Venus, Jupiter and Saturn in Pisces, the Moon stands in Taurus for 164 hours, and in none of them does Mars aspect the Sun.",
    ),
    (
        "SARAVALI_BHANGA_JUPITER_AND_MOON_IN_CANCER_MERCURY_AND_SATURN_IN_LIBRA",
        "Four days from 1800 to 2400, all in 1895, have Jupiter in Cancer, Mercury and Saturn in Libra and the Sun, Mars and Venus as the rule asks, and on them the Moon is in Libra to Sagittarius, never Cancer.",
    ),
    (
        "SARAVALI_MARS_SATURN_AND_SUN_IN_THE_SECOND_UNDER_A_WEAK_MOON",
        "A Moon aspecting the second while the Sun stands in it is opposite the Sun, near full, and no instant from 1900 to 2100 has her Shadbala below her six rupas there.",
    ),
    (
        "SARAVALI_WEAK_MOON_IN_THE_SECOND_ASPECTED_BY_MERCURY",
        "Mercury casts only the seventh aspect and keeps near the Sun, so the Moon he aspects is near full, and no instant from 1900 to 2100 has her Shadbala below her six rupas there.",
    ),
    (
        "SARAVALI_RAJA_ARIES_RISING_JUPITER_AND_SUN_IN_LEO_FIGURE",
        "No sky from 1801 to 2398 has its seven grahas in those signs, searched four times a day.",
    ),
    (
        "SARAVALI_RAJA_CANCER_MOON_RISING_FIGURE",
        "No sky from 1801 to 2398 has its seven grahas in those houses with Cancer rising, searched four times a day.",
    ),
    (
        "SARAVALI_RAJA_EXALTED_MOON_RISING_FIGURE",
        "No sky from 1801 to 2398 has its six grahas in those signs, searched four times a day.",
    ),
    (
        "SARAVALI_RAJA_LEO_SUN_RISING_FIGURE",
        "No sky from 1801 to 2398 has its five grahas in those signs, searched four times a day.",
    ),
    (
        "SARAVALI_RAJA_SATURN_LATE_CAPRICORN_FIGURE",
        "No sky from 1801 to 2398 has its seven grahas in those signs with Saturn in the latter half of Capricorn, searched four times a day.",
    ),
    (
        "SARAVALI_RAJA_VIRGO_MERCURY_RISING_MOON_ELEVENTH",
        "No sky from 1801 to 2398 has its six grahas in those signs, searched four times a day.",
    ),
    (
        "SARAVALI_RAJA_ARIES_FIGURE_JUPITER_NINTH_IN_OWN_NAVAMSHA",
        "No sky from 1801 to 2398 has its seven grahas in those signs, searched every ten minutes of a day they allow.",
    ),
    (
        "SARAVALI_RAJA_BENEFICS_FIFTH_SUN_SIXTH_MARS_EXALTED_SATURN_NINTH",
        "No sky from 1801 to 2398 has Venus, Mercury and Jupiter together with the Sun a sign on, Mars exalted and Saturn four signs on from them, searched every ten minutes of a day they allow.",
    ),
    (
        "SARAVALI_RAJA_VIRGO_MERCURY_RISING_JUPITER_PISCES_SATURN_SIXTH",
        "Venus in Sagittarius three signs from Mercury in Virgo needs both near their greatest elongations on either side of the Sun, and no sky from 1801 to 2398 has it with Jupiter in Pisces and Saturn in Aquarius.",
    ),
    (
        "SARAVALI_RAJA_MOON_TENTH_VENUS_NINTH_OTHERS_ELEVENTH",
        "No sky from 1801 to 2398 has Mars, Mercury, Jupiter and Saturn in the Sun's sign with Venus two signs and the Moon one sign behind it, searched every ten minutes of a day they allow.",
    ),
    (
        "SARAVALI_RAJA_JUPITER_SECOND_MARS_FOURTH_SUN_TENTH_MOON_RISING_THE_REST_IN_ENEMY_SIGNS",
        "No sky from 1801 to 2398 holds it: the 164 instants at Kathmandu that place Jupiter, Mars, the Sun and the Moon as it asks, searched every ten minutes of a day they allow, never also put Mercury, Venus and Saturn all in an enemy's sign.",
    ),
    (
        "SARAVALI_RAJA_VENUS_AND_MERCURY_IN_KENDRAS_FROM_SATURN_THE_REST_EXALTED",
        "No sky from 1801 to 2398 has the Sun, the Moon, Mars, Jupiter and Saturn exalted at once, searched every ten minutes of a day they allow.",
    ),
    (
        "PARIJATA_RAJA_LAGNA_LORD_IN_AN_UPACHAYA_FROM_THE_MOON_BENEFICS_IN_KENDRAS_MALEFICS_WEAK",
        "It asks the Sun, Mars and Saturn all below their required Shadbala: of 2,000 instants drawn from 1900 to 2100 the Sun is weak on 482, Mars on 54, the two together on 12 and all three on none, and none of the 16,848 instants every twenty minutes of the 300 days from 1800 to 2398 that place the rest has them weak together.",
    ),
    (
        "PARIJATA_RAJA_SIX_EXALTED",
        "As BPHS_SIX_GRAHAS_EXALTED: Mercury exalted in Virgo and Venus in Pisces would stand opposite, which the two never do, so the six are the Sun, the Moon, Mars, Jupiter, Venus and Saturn, and no day from 1800 to 2398 has the slower of them exalted together.",
    ),
    (
        "PARIJATA_RAJA_LIBRA_FIGURE",
        "It fixes six grahas' signs and the lagna: the Moon, Jupiter and Venus in Sagittarius, Mercury in Libra rising, Mars in Virgo and Saturn in Capricorn. No day from 1800 to 2398 has the grahas in those signs, searched day by day.",
    ),
    (
        "PARIJATA_RAJA_MERCURY_VIRGO_MOON_PISCES_MARS_GEMINI_JUPITER_SAGITTARIUS_SATURN_CAPRICORN",
        "No day from 1800 to 2398 has Mercury, the Moon, Mars, Jupiter and Saturn in Virgo, Pisces, Gemini, Sagittarius and Capricorn at once, searched day by day by sign.",
    ),
    (
        "BJ_NABHASA_VAJRA",
        "Benefics in the 1st and 7th and malefics in the 4th and 10th, all seven in the kendras: the Sun, a malefic, would stand three or nine signs from Venus, a benefic, and Venus is never more than two signs from the Sun (her greatest elongation 47.28 degrees, day by day 1800 to 2398). Ch. 12 v. 6 says so itself.",
    ),
    (
        "BJ_NABHASA_YAVA",
        "Vajra's figure turned about, malefics in the 1st and 7th and benefics in the 4th and 10th, all seven in the kendras: the Sun and Venus would again stand three or nine signs apart, which her greatest elongation of 47.28 degrees, day by day 1800 to 2398, never allows.",
    ),
    (
        "PARIJATA_MARANA_WANING_MOON_EIGHTH_MARS_TENTH_SATURN_FIRST_SUN_FOURTH_CLUBBED",
        "The Moon in the 8th and the Sun in the 4th stand five signs apart, so the Moon is 120 to 180 degrees past the Sun and waxing; the verse asks her waning.",
    ),
    (
        "BJ_MARANA_WANING_MOON_EIGHTH_MARS_TENTH_SATURN_FIRST_SUN_FOURTH_CLUBBED",
        "The Moon in the 8th and the Sun in the 4th stand five signs apart, so the Moon is 120 to 180 degrees past the Sun and waxing; the verse asks her waning.",
    ),
    (
        "PARIJATA_MARANA_WANING_MOON_TENTH_MARS_NINTH_SATURN_FIRST_SUN_FIFTH_SMOKE_OR_FIRE",
        "The Moon in the 10th and the Sun in the 5th stand six signs apart, 120 to 180 degrees past the Sun, so she is waxing; of 83 days 1800 to 2398 holding everything else, none has her waning.",
    ),
    (
        "BJ_MARANA_WANING_MOON_TENTH_MARS_NINTH_SATURN_FIRST_SUN_FIFTH_SMOKE_OR_FIRE",
        "The Moon in the 10th and the Sun in the 5th stand six signs apart, 120 to 180 degrees past the Sun, so she is waxing; of 83 days 1800 to 2398 holding everything else, none has her waning.",
    ),
    (
        "PARIJATA_MARANA_WANING_MOON_TENTH_SUN_SEVENTH_MARS_FOURTH_DEATH_IN_FILTH",
        "The Moon in the 10th and the Sun in the 7th stand four signs apart, so the Moon is past the Sun by under 180 degrees and waxing; the verse asks her waning.",
    ),
    (
        "BJ_MARANA_WANING_MOON_TENTH_SUN_SEVENTH_MARS_FOURTH_DEATH_IN_FILTH",
        "The Moon in the 10th and the Sun in the 7th stand four signs apart, so the Moon is past the Sun by under 180 degrees and waxing; the verse asks her waning.",
    ),
    (
        "PARIJATA_PRESHYA_NIGHT_SUN_TENTH_MOON_SEVENTH_SATURN_FOURTH_MARS_THIRD_JUPITER_SECOND",
        "The verse asks a birth at night (the Sanskrit's nisayam) with the Sun in the 10th, which is above the horizon: no night birth has the Sun there.",
    ),
    (
        "PARIJATA_ROGA_MARS_WITH_MERCURY_SIXTH_IN_A_MALEFICS_NAVAMSHA_ASPECTED_BY_MOON_AND_VENUS",
        "Venus aspects only the 7th from her, so she would stand opposite Mars and so opposite Mercury, with whom he stands; Mercury and Venus are never more than about 76 degrees apart.",
    ),
    (
        "PARIJATA_JUPITER_WITH_SATURN_MARS_RAHU_ASPECTED_BY_VENUS_AND_MERCURY",
        "No day 1800 to 2398 has Jupiter with Saturn, Mars and Rahu aspected by Venus and Mercury, searched by sign day by day; leaving out any one condition leaves at most 26 days.",
    ),
    (
        "PARIJATA_DARIDRA_FIVE_IN_THE_EIGHTH_SIXTH_TWELFTH_FIFTH_TENTH_TWELFTH_LORD_FALLEN_YET_STRONGER",
        "Of the days 1800 to 2398 placing the five grahas so, the twelfth lord is debilitated on 66 and combust as well on none, searched by sign day by day; the translator's own example figure cannot occur.",
    ),
    (
        "PARIJATA_ROGA_SATURN_WITH_MARS_SIXTH_ASPECTED_BY_SUN_AND_RAHU_LAGNA_LORD_WEAK",
        "Of the days 1800 to 2398 holding everything else, 383 by sign, none has Rahu aspecting Saturn and Mars in the 6th.",
    ),
    (
        "PARIJATA_ANGA_MERCURY_IN_A_TRIKONA_THE_REST_WEAK",
        "It asks every graha but Mercury weak, and the Sun, Mars and Saturn are never weak together: of 2,000 instants drawn over 1900 to 2100 the Sun is weak on 482, Mars on 54, both on 12, and the three on none.",
    ),
    (
        "PARIJATA_LOKA_MOKSHA_PISCES_RISING_IN_A_BENEFICS_NAVAMSHA_WITH_JUPITER_THE_REST_WEAK",
        "It asks every graha but Jupiter weak, and the Sun, Mars and Saturn are never weak together: of 2,000 instants drawn over 1900 to 2100 the Sun is weak on 482, Mars on 54, both on 12, and the three on none.",
    ),
    (
        "BJ_LOKA_MOKSHA_PISCES_RISING_IN_A_BENEFICS_NAVAMSHA_WITH_JUPITER_THE_REST_WEAK",
        "It asks every graha but Jupiter weak, and the Sun, Mars and Saturn are never weak together: of 2,000 instants drawn over 1900 to 2100 the Sun is weak on 482, Mars on 54, both on 12, and the three on none.",
    ),
    (
        "PARIJATA_MOON_AND_MERCURY_TENTH_AFFLICTED_IN_FALLEN_NAVAMSHAS",
        "It holds on none of 24,848 instants drawn over 1900 to 2100 at Kathmandu, nor on any ten minutes of the days a sign-level search of 1800 to 2398 found for it: the Moon and Mercury in the 10th, each in its fallen navamsha, with a malefic, aspected by one, together.",
    ),
    (
        "PARIJATA_ROGA_SATURN_SIXTH_WITH_GULIKA_ASPECTED_BY_SUN_MARS_RAHU",
        "It holds on none of 24,848 instants drawn over 1900 to 2100 at Kathmandu, nor on any ten minutes of the days a sign-level search of 1800 to 2398 found with Saturn so aspected: Gulika, which moves through the day, must stand in the 6th with him as well.",
    ),
    (
        "BJ_STRI_SATURN_MIDDLING_THE_SOFT_WEAK_THE_HARD_STRONG_ODD_LAGNA",
        "It asks the Moon, Venus and Mercury below their required Shadbala together while the Sun, Mars and Jupiter pass theirs, and the Moon rarely falls below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds it.",
    ),
    (
        "SARAVALI_STRI_SATURN_MIDDLING_THE_SOFT_WEAK_THE_HARD_STRONG_ODD_LAGNA",
        "Brihat Jataka ch. 24's figure, its conditions copied: of the same 9,000 instants, none holds it.",
    ),
    (
        "PARIJATA_VIYONI",
        "It asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them, and her Shadbala rarely falls below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds it.",
    ),
    (
        "PARIJATA_VIYONI_LAGNA_ASPECTED_BY_VENUS_COW",
        "It is a figure within the viyoni birth of v. 2, which asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them and rarely below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds v. 2.",
    ),
    (
        "PARIJATA_VIYONI_LAGNA_WITH_OR_ASPECTED_BY_SATURN_BUFFALO",
        "It is a figure within the viyoni birth of v. 2, which asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them and rarely below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds v. 2.",
    ),
    (
        "PARIJATA_VIYONI_NODE_RISING_SHEEP_OR_GOAT",
        "It is a figure within the viyoni birth of v. 2, which asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them and rarely below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds v. 2.",
    ),
    (
        "PARIJATA_VIYONI_MALEFICS_RISING_OTHER_KIND",
        "It is a figure within the viyoni birth of v. 2, which asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them and rarely below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds v. 2.",
    ),
    (
        "PARIJATA_GARBHA_TRIPLETS_THREE_BOYS",
        "The lagna and all seven must stand in two navamsha signs with Mercury in one of them: no day from 1800 to 2398 holds it at the level of signs, and none of 9,000 instants drawn over 1900 to 2100.",
    ),
    (
        "PARIJATA_GARBHA_TRIPLETS_THREE_GIRLS",
        "The lagna and all seven must stand in two navamsha signs with Mercury in one of them: no day from 1800 to 2398 holds it at the level of signs, and none of 9,000 instants drawn over 1900 to 2100.",
    ),
    (
        "PARIJATA_RAJA_FOUR_EXALTED_AND_STRONG_IN_KENDRAS",
        "Only the Sun, Mars, Jupiter and Saturn are exalted in signs square to one another, Aries, Capricorn, Cancer and Libra, and no day from 1800 to 2398 has the four exalted together.",
    ),
    (
        "SARAVALI_AYUR_PISCES_RISING_LAST_NAVAMSHA_BENEFICS_IN_TAURUS_THE_REST_EXALTED",
        "It asks Jupiter in Taurus, Saturn in Libra, the Sun in Aries, Mars in Capricorn and Mercury and Venus in Taurus: the slow pair stands so on 1,808 days from 1800 to 2398, and on none of them are the other four placed so.",
    ),
    (
        "SARAVALI_SIX_EXALTED",
        "Six exalted asks Jupiter in Cancer with Saturn in Libra, which holds on 1,339 days from 1800 to 2398, and no day of them has four more exalted.",
    ),
    (
        "SARAVALI_SIX_IN_MOOLATRIKONA",
        "No day from 1800 to 2398 has six of the seven in their moolatrikona spans.",
    ),
    (
        "SARAVALI_SEVEN_IN_MOOLATRIKONA",
        "Jupiter and Saturn are both in moolatrikona on 102 days from 1800 to 2398, and on none of them are the other five.",
    ),
    (
        "SARAVALI_SIX_IN_OWN_SIGNS",
        "No day from 1800 to 2398 has six of the seven in their own signs outside the moolatrikona span.",
    ),
    (
        "SARAVALI_SEVEN_IN_OWN_SIGNS",
        "No day from 1800 to 2398 has the seven in their own signs outside the moolatrikona span.",
    ),
    (
        "SARAVALI_MARANA_WANING_MOON_TENTH_SUN_SEVENTH_MARS_FOURTH_FILTH",
        "It asks a waning Moon in the tenth with the Sun in the seventh, which puts her three signs ahead of him and so always waxing.",
    ),
];

/// Rules that hold on every chart by design, each with why: a rule that
/// names grahas rather than asking whether, and says what it says in its
/// participants.
const ALWAYS: [(&str, &str); 1] = [(
    "BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH",
    "BPHS names every chart's wealth-givers, the lords of the ninth and fifth and the grahas \
     joining either, and the rule says which they are in its participants.",
)];

/// The instants of the `place`-th place: evenly through the span, each
/// place offset by a fraction of a step so no two places share a sky, and
/// an hour of the day that walks the golden ratio so the lagna visits
/// every sign.
fn instants(place: u32) -> Vec<JulianDay<Utc>> {
    const GOLDEN: f64 = 0.618_033_988_749_894_9;
    let step = SPAN_DAYS / f64::from(PER_PLACE);
    let offset = f64::from(place) * step / f64::from(PLACE_COUNT);
    (0..PER_PLACE)
        .map(|at| {
            let at = f64::from(at);
            let day = (at * step + offset).floor();
            JulianDay::<Utc>::literal(FIRST_JD + day + (at * GOLDEN).fract())
        })
        .collect()
}

/// What the sky gave: the shipped rules, for each that held at least
/// once the charts it held on, and the [`FIXTURES`] that held at their
/// instants.
struct Sky {
    rules: Vec<Rule>,
    held: BTreeMap<String, Vec<u32>>,
    fixed: BTreeSet<String>,
}

/// The context every chart of the page is cast in: the built-in ephemeris,
/// as a consumer's default.
fn context() -> Result<Context, String> {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|e| e.to_string())
}

/// A chart at a latitude and longitude, its clock UTC.
fn request((latitude, longitude): (f64, f64)) -> Result<ChartRequest, String> {
    let place = Place::new(
        Latitude::try_new(latitude).map_err(|e| e.to_string())?,
        Longitude::try_new(longitude).map_err(|e| e.to_string())?,
        Altitude::try_new(0.0).map_err(|e| e.to_string())?,
    );
    Ok(ChartRequest::at(place, UtcOffset::UTC))
}

fn sky() -> Result<Sky, String> {
    let sdk = context()?;
    let set = RuleRequest::shipped(ShippedRules::ALL)
        .rule_set()
        .map_err(|e| e.to_string())?;
    let mut held: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    let mut chart = 0_u32;
    for (place, at) in PLACES.into_iter().zip(0..) {
        let read = sdk
            .chart()
            .readings_with_rules(&instants(at), &request(place)?, &set)
            .map_err(|e| e.to_string())?
            .value;
        for (_, reading) in &read {
            for present in &reading.present {
                held.entry(present.rule.key.clone())
                    .or_default()
                    .push(chart);
            }
            chart += 1;
        }
    }
    let mut fixed = BTreeSet::new();
    for (key, jd, latitude, longitude) in FIXTURES {
        let read = sdk
            .chart()
            .readings_with_rules(
                &[JulianDay::literal(jd)],
                &request((latitude, longitude))?,
                &set,
            )
            .map_err(|e| e.to_string())?
            .value;
        if read
            .iter()
            .any(|(_, reading)| reading.present.iter().any(|p| p.rule.key == key))
        {
            fixed.insert(key.to_owned());
        }
    }
    Ok(Sky {
        rules: set.rules().to_vec(),
        held,
        fixed,
    })
}

/// Why a group co-fires, or that nothing explains it; a group with several
/// explanations is filed under the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Why {
    Texts,
    Mutual,
    Reviewed(usize),
    Unexplained,
}

/// `(graha, sign, aspecting graha)` when a rule is exactly "the graha in
/// the sign under that graha's aspect".
fn aspected_in_sign(rule: &Rule) -> Option<(Graha, Rashi, Graha)> {
    let [
        Condition::PlanetInSign {
            planet: SignRef::Of(BodyRef::Body(Body::Graha(graha))),
            signs,
        },
        Condition::PlanetAspectsPlanet {
            from: BodySubject::Ref(BodyRef::Body(Body::Graha(from))),
            target: SignRef::Of(BodyRef::Body(Body::Graha(target))),
        },
    ] = rule.conditions.as_slice()
    else {
        return None;
    };
    let [sign] = signs.as_slice() else {
        return None;
    };
    (graha == target).then_some((*graha, *sign, *from))
}

/// Whether two rules of one text are one aspect read from both grahas:
/// each in the sign seventh from the other's, under the other's aspect,
/// both casting only the seventh (the Sun, the Moon, Mercury and Venus).
fn mutual(one: &Rule, two: &Rule) -> bool {
    const SEVENTH_ONLY: [Graha; 4] = [Graha::Sun, Graha::Moon, Graha::Mercury, Graha::Venus];
    let (Some((a, sign_a, by_a)), Some((b, sign_b, by_b))) =
        (aspected_in_sign(one), aspected_in_sign(two))
    else {
        return false;
    };
    by_a == b
        && by_b == a
        && SEVENTH_ONLY.contains(&a)
        && SEVENTH_ONLY.contains(&b)
        && House::between(sign_a, sign_b).get() == 7
}

/// Why `keys` co-fire: within each text the rules must be one rule, a
/// mutual pair or a reviewed entry.
fn why(keys: &[&str], rules: &BTreeMap<&str, &Rule>) -> Why {
    let mut by_text: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for key in keys {
        let text = rules.get(key).map_or("", |rule| rule.source.text.as_str());
        by_text.entry(text).or_default().push(key);
    }
    let mut verdict = Why::Texts;
    for same in by_text.values().filter(|same| same.len() > 1) {
        let paired = match same.as_slice() {
            [one, two] => rules
                .get(one)
                .zip(rules.get(two))
                .is_some_and(|(a, b)| mutual(a, b)),
            _ => false,
        };
        let found = if paired {
            Why::Mutual
        } else {
            REVIEWED
                .iter()
                .position(|(listed, _)| listed.iter().copied().eq(same.iter().copied()))
                .map_or(Why::Unexplained, Why::Reviewed)
        };
        verdict = verdict.max(found);
    }
    verdict
}

/// One listed group: how many charts it holds on, its rules and why.
struct Group<'a> {
    support: usize,
    keys: Vec<&'a str>,
    why: Why,
}

/// The groups at or above [`SUPPORT`], most charts first within each
/// explanation, and how many fell below it.
fn groups(sky: &Sky) -> (Vec<Group<'_>>, usize) {
    let rules: BTreeMap<&str, &Rule> = sky.rules.iter().map(|r| (r.key.as_str(), r)).collect();
    let mut by_charts: BTreeMap<&[u32], Vec<&str>> = BTreeMap::new();
    for (key, charts) in &sky.held {
        by_charts
            .entry(charts.as_slice())
            .or_default()
            .push(key.as_str());
    }
    let mut chance = 0;
    let mut listed = Vec::new();
    for (charts, keys) in by_charts {
        if keys.len() < 2 {
            continue;
        }
        if charts.len() < SUPPORT {
            chance += 1;
            continue;
        }
        let why = why(&keys, &rules);
        listed.push(Group {
            support: charts.len(),
            keys,
            why,
        });
    }
    listed.sort_by(|a, b| (a.why, b.support, &a.keys).cmp(&(b.why, a.support, &b.keys)));
    (listed, chance)
}

/// The table rows of the groups `keep` admits, with the reviewed reason
/// when `reason` is asked.
fn rows(groups: &[Group<'_>], keep: impl Fn(Why) -> bool, reason: bool) -> String {
    let mut out = String::new();
    for group in groups.iter().filter(|g| keep(g.why)) {
        let keys: Vec<String> = group.keys.iter().map(|k| format!("`{k}`")).collect();
        let _ = write!(out, "| {} | {} |", group.support, keys.join("<br>"));
        if reason {
            let said = match group.why {
                Why::Reviewed(at) => REVIEWED.get(at).map_or("", |(_, why)| why),
                _ => "**unexplained**",
            };
            let _ = write!(out, " {said} |");
        }
        out.push('\n');
    }
    out
}

/// The rules written in the condition language: a rule computed in code
/// has no conditions for this evaluation to read, and `check-doshas`
/// measures it.
fn written(sky: &Sky) -> impl Iterator<Item = &Rule> {
    sky.rules.iter().filter(|r| !r.conditions.is_empty())
}

/// Refuses a written rule the sky never holds with neither a fixture nor
/// a reason, a listed one the sky holds or the catalogue lacks, a fixture
/// that does not hold at its instant, and a rule that holds on every
/// chart, which has no negative fixture.
fn unseen_refusals(sky: &Sky) -> Result<(), String> {
    let listed: Vec<&str> = FIXTURES
        .iter()
        .map(|(key, ..)| *key)
        .chain(UNSEEN.iter().map(|(key, _)| *key))
        .collect();
    let charts = (PER_PLACE * PLACE_COUNT) as usize;
    let mut found = Vec::new();
    for rule in written(sky) {
        let key = rule.key.as_str();
        if !sky.held.contains_key(key) && !listed.contains(&key) {
            found.push(format!(
                "`{key}` holds on no chart of the sky; find an instant and a place where it holds \
                 (`cargo xtask cofiring-probe` asks the SDK of candidate instants) and list it in \
                 FIXTURES, or say in UNSEEN why none can"
            ));
        }
        let always = ALWAYS.iter().any(|(listed, _)| *listed == key);
        let everywhere = sky.held.get(key).is_some_and(|on| on.len() == charts);
        if everywhere && !always {
            found.push(format!(
                "`{key}` holds on every chart of the sky, so nothing shows it can fail; fix \
                 it, or list it in ALWAYS with why it names rather than asks"
            ));
        }
        if always && !everywhere {
            found.push(format!(
                "`{key}` is listed in ALWAYS but fails on a chart of the sky; remove it"
            ));
        }
    }
    for (at, key) in listed.iter().enumerate() {
        if listed[..at].contains(key) {
            found.push(format!("`{key}` is listed twice in FIXTURES and UNSEEN"));
        } else if !written(sky).any(|r| r.key == *key) {
            found.push(format!(
                "`{key}` is listed in FIXTURES or UNSEEN but is no rule written in the condition \
                 language; remove it"
            ));
        } else if sky.held.contains_key(*key) {
            found.push(format!(
                "`{key}` now holds in the sky, so it needs no fixture; remove it from FIXTURES or \
                 UNSEEN"
            ));
        }
    }
    for (key, _) in ALWAYS {
        if !written(sky).any(|r| r.key == key) {
            found.push(format!(
                "`{key}` is listed in ALWAYS but is no rule written in the condition language; \
                 remove it"
            ));
        }
    }
    for (key, ..) in FIXTURES {
        if !sky.fixed.contains(key) {
            found.push(format!(
                "`{key}` does not hold at its fixture's instant; find it another, or move it to \
                 UNSEEN with why"
            ));
        }
    }
    if found.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} finding(s) in xtask/src/cofiring.rs's FIXTURES, UNSEEN and ALWAYS:\n{}",
            found.len(),
            found.join("\n")
        ))
    }
}

/// The fixture rows, a row to a rule.
fn fixture_rows() -> String {
    let mut out = String::new();
    for (key, jd, latitude, longitude) in FIXTURES {
        let _ = writeln!(out, "| `{key}` | {jd} | {latitude}, {longitude} |");
    }
    out
}

/// The rows of the rules held on every chart by design, a row to a rule
/// with why.
fn always_rows() -> String {
    let mut out = String::new();
    for (key, why) in ALWAYS {
        let _ = writeln!(out, "| `{key}` | {why} |");
    }
    out
}

/// The unseen rows, a row to a rule with why.
fn unseen_rows() -> String {
    let mut out = String::new();
    for (key, why) in UNSEEN {
        let _ = writeln!(out, "| `{key}` | {why} |");
    }
    out
}

/// Refuses a reviewed entry that no longer co-fires and a group nothing
/// explains.
///
/// An entry is in use while some group holds all its keys. A group whose
/// condition two texts each give twice holds two entries, and its `why`
/// names only one of them, so the verdict cannot say which entries are in
/// use; a group whose text says it a third time is unexplained and refused
/// below.
fn refusals(groups: &[Group<'_>]) -> Result<(), String> {
    for (keys, _) in REVIEWED {
        if !groups
            .iter()
            .any(|g| keys.iter().all(|key| g.keys.contains(key)))
        {
            return Err(format!(
                "the reviewed group of `{}` no longer co-fires; remove it from REVIEWED in \
                 xtask/src/cofiring.rs",
                keys.first().copied().unwrap_or_default()
            ));
        }
    }
    let unexplained = groups.iter().filter(|g| g.why == Why::Unexplained).count();
    if unexplained > 0 {
        return Err(format!(
            "{unexplained} group(s) of one text co-fire with nothing to explain them, probable \
             duplicates; fix the rule, or list the group in REVIEWED with where the text says \
             it twice:\n{}",
            rows(groups, |w| w == Why::Unexplained, false)
        ));
    }
    Ok(())
}

fn page() -> Result<String, String> {
    let sky = sky()?;
    let (groups, chance) = groups(&sky);
    refusals(&groups)?;
    unseen_refusals(&sky)?;
    let of = |keep: &dyn Fn(Why) -> bool| groups.iter().filter(|g| keep(g.why)).count();
    let charts = PER_PLACE * PLACE_COUNT;
    Ok(format!(
        "# Rules that hold together, measured\n\n\
         Status: `generated` by `cargo xtask cofiring`; `check-cofiring` regenerates it in verify \
         and fails on any difference.\n\n\
         Two rules that hold on exactly the same charts are a duplicate unless something says \
         why ([`01-quality-bar.md`](../05-testing/01-quality-bar.md), \"rule fixtures\"). The \
         pass evaluates every shipped rule, {rules} of them, on {charts} charts the SDK \
         computes: {per} at each of {places} places from Quito to Sydney, from 1900 to 2100, no \
         two at one instant. {held} rules hold at least once and {never} never do.\n\n\
         Rules holding on the same {support} or more charts form {total} groups, and each is \
         explained. Below that, {chance} groups are nested conjunctions of five and six grahas, \
         each held on a chart or two, which is chance.\n\n\
         - **{texts} groups: two texts state one condition.** The rules cite different texts, \
         so the group is agreement between sources, and each rule carries its own reading.\n\
         - **{mutual} groups: one aspect read from both grahas.** Saravali reads a graha in \
         each sign under each aspect, a chapter to a graha, and two grahas that cast only the \
         seventh aspect each other exactly when each stands seventh from the other.\n\
         - **{reviewed} groups: one text says it twice**, where the table below says.\n\n\
         A group of one text that is none of these fails the gate, and so does a listed group \
         that no longer holds together.\n\n\
         ## One text, twice\n\n\
         | charts | rules | where |\n|---:|---|---|\n{reviewed_rows}\n\
         ## One aspect from both grahas\n\n\
         | charts | rules |\n|---:|---|\n{mutual_rows}\n\
         ## Two texts, one condition\n\n\
         | charts | rules |\n|---:|---|\n{texts_rows}\n\
         ## Never held\n\n\
         A rule that holds on none of the charts is not a duplicate of anything, but it is \
         either rare or dead, and every one is accounted for: it is listed with an instant and \
         a place where it holds, which the gate evaluates, or with why none can. The instants \
         were found by searching every two hours from 1900 to 2100 at Kathmandu's latitude, on \
         the meridian that puts the rule's lagna on the horizon, and from 1800 to 2400, the \
         span the built-in ephemeris covers, for a rule the two centuries lacked. {computed} \
         rules the sky never holds are computed in code rather than written as conditions, \
         which this evaluation does not read, and `check-doshas` measures them.\n\n\
         ### Held at a found instant\n\n\
         | rule | instant (JD, UT) | latitude, longitude |\n|---|---:|---|\n{fixture_rows}\n\
         ### Held nowhere searched\n\n\
         | rule | why |\n|---|---|\n{unseen_rows}\n\
         ## Held on every chart\n\n\
         A rule that holds on every chart of the sky fails the gate, unless it names grahas \
         rather than asking whether, and these are the ones that do.\n\n\
         | rule | why |\n|---|---|\n{always_rows}",
        rules = count(sky.rules.len()),
        charts = count(charts as usize),
        per = count(PER_PLACE as usize),
        places = PLACE_COUNT,
        held = count(sky.held.len()),
        never = count(sky.rules.len() - sky.held.len()),
        support = SUPPORT,
        total = groups.len(),
        texts = of(&|w| w == Why::Texts),
        mutual = of(&|w| w == Why::Mutual),
        reviewed = of(&|w| matches!(w, Why::Reviewed(_))),
        reviewed_rows = rows(&groups, |w| matches!(w, Why::Reviewed(_)), true),
        mutual_rows = rows(&groups, |w| w == Why::Mutual, false),
        texts_rows = rows(&groups, |w| w == Why::Texts, false),
        fixture_rows = fixture_rows(),
        unseen_rows = unseen_rows(),
        always_rows = always_rows(),
        computed = sky
            .rules
            .iter()
            .filter(|r| r.conditions.is_empty() && !sky.held.contains_key(&r.key))
            .count(),
    ))
}

fn outputs() -> Result<[Output; 1], String> {
    Ok([Output::new(PAGE, page()?)])
}

/// `cofiring-probe FILE KEYS [LATITUDE LONGITUDE]`: which of the instants in
/// `FILE`, one Julian day (UTC) a line, hold each of the comma-separated
/// rule `KEYS`, at Kathmandu unless a place is given. A fixture for a rule
/// the sky misses is found by narrowing the instants elsewhere, on
/// placements alone, and asking the SDK here which of them it holds on.
pub(crate) fn probe(args: &[String]) -> i32 {
    match probed(args) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("cofiring-probe: {error}");
            2
        }
    }
}

fn probed(args: &[String]) -> Result<(), String> {
    let usage = "usage: cofiring-probe FILE KEYS [LATITUDE LONGITUDE]";
    let [_, file, keys, ..] = args else {
        return Err(usage.to_owned());
    };
    let place = match args.get(3..) {
        Some([latitude, longitude]) => (
            latitude
                .parse()
                .map_err(|_| format!("latitude {latitude}"))?,
            longitude
                .parse()
                .map_err(|_| format!("longitude {longitude}"))?,
        ),
        Some([]) | None => PLACES[0],
        Some(_) => return Err(usage.to_owned()),
    };
    let keys: BTreeSet<&str> = keys.split(',').collect();
    let text = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let instants = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.trim()
                .parse()
                .map(JulianDay::literal)
                .map_err(|_| format!("{file}: `{line}` is not a Julian day"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let set = RuleRequest::shipped(ShippedRules::ALL)
        .rule_set()
        .map_err(|e| e.to_string())?;
    let read = context()?
        .chart()
        .readings_with_rules(&instants, &request(place)?, &set)
        .map_err(|e| e.to_string())?
        .value;
    let mut held: BTreeMap<&str, Vec<f64>> = keys.iter().map(|key| (*key, Vec::new())).collect();
    for ((_, reading), at) in read.iter().zip(&instants) {
        for present in &reading.present {
            if let Some(list) = held.get_mut(present.rule.key.as_str()) {
                list.push(at.get());
            }
        }
    }
    for (key, at) in held {
        let first: Vec<String> = at.iter().take(5).map(|jd| format!("{jd:.4}")).collect();
        println!(
            "{key}: {} of {} [{}]",
            at.len(),
            instants.len(),
            first.join(", ")
        );
    }
    Ok(())
}

pub(crate) fn generate(root: &Path) -> i32 {
    match outputs() {
        Ok(outputs) => write(root, &outputs),
        Err(error) => {
            eprintln!("cofiring: {error}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs() {
        Ok(outputs) => i32::from(check(root, &outputs, "cargo xtask cofiring") != 0),
        Err(error) => {
            eprintln!("cofiring: {error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    /// A rule of `text` reading `graha` in `sign` under `by`'s aspect.
    fn aspected(key: &str, text: &str, graha: &str, sign: &str, by: &str) -> Rule {
        serde_json::from_value(serde_json::json!({
            "key": key,
            "category": "test",
            "source": {"text": text},
            "conditions": [
                {"type": "planet-in-sign", "planet": graha, "signs": [sign]},
                {"type": "planet-aspects-planet", "from": by, "target": graha},
            ],
            "outcomes": [],
        }))
        .unwrap()
    }

    /// A mutual pair is recognised only for grahas casting the seventh
    /// alone, standing seventh from each other; anything else of one text
    /// is unexplained unless reviewed.
    #[test]
    fn a_group_of_one_text_is_explained_or_refused() {
        let rules = [
            aspected("A", "Saravali", "MOON", "CANCER", "MERCURY"),
            aspected("B", "Saravali", "MERCURY", "CAPRICORN", "MOON"),
            aspected("C", "Saravali", "MERCURY", "ARIES", "MOON"),
            aspected("D", "Saravali", "MARS", "CAPRICORN", "MOON"),
            aspected("E", "Phaladeepika", "MOON", "CANCER", "MERCURY"),
        ];
        let rules: BTreeMap<&str, &Rule> = rules.iter().map(|r| (r.key.as_str(), r)).collect();
        assert_eq!(why(&["A", "B"], &rules), Why::Mutual);
        assert_eq!(why(&["A", "B", "E"], &rules), Why::Mutual);
        assert_eq!(why(&["A", "E"], &rules), Why::Texts);
        assert_eq!(why(&["A", "C"], &rules), Why::Unexplained);
        assert_eq!(why(&["A", "D"], &rules), Why::Unexplained);
    }

    /// A rule the sky never holds is refused unless listed, a rule held on
    /// every chart unless it names, and a listed rule the sky holds or
    /// lacks is stale.
    #[test]
    fn every_rule_is_held_somewhere_and_fails_somewhere() {
        let rule = |key: &str| aspected(key, "Saravali", "MOON", "CANCER", "MERCURY");
        let charts = PER_PLACE * PLACE_COUNT;
        let mut rules: Vec<Rule> = [FIXTURES[0].0, UNSEEN[0].0, ALWAYS[0].0, "NEVER", "EVERY"]
            .into_iter()
            .map(rule)
            .collect();
        let mut held: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        held.insert(ALWAYS[0].0.to_owned(), (0..charts).collect());
        held.insert("EVERY".to_owned(), (0..charts).collect());
        let sky = Sky {
            rules: rules.clone(),
            held: held.clone(),
            fixed: [FIXTURES[0].0.to_owned()].into_iter().collect(),
        };
        let found = unseen_refusals(&sky).unwrap_err();
        assert!(found.contains("`NEVER` holds on no chart"), "{found}");
        assert!(found.contains("`EVERY` holds on every chart"), "{found}");
        assert!(
            !found.contains(&format!("`{}` holds on", FIXTURES[0].0)),
            "{found}"
        );
        assert!(
            !found.contains(&format!("`{}` holds on", UNSEEN[0].0)),
            "{found}"
        );
        assert!(
            !found.contains(&format!("`{}` holds on", ALWAYS[0].0)),
            "{found}"
        );
        // The other listed rules are no rules of this sky.
        assert!(
            found.contains(&format!("`{}` is listed", FIXTURES[1].0)),
            "{found}"
        );
        // A fixture that stops holding, and listed rules the sky now holds
        // or no longer always holds, are stale.
        rules.retain(|r| r.key != "NEVER" && r.key != "EVERY");
        held.insert(UNSEEN[0].0.to_owned(), vec![0]);
        held.insert(ALWAYS[0].0.to_owned(), vec![0]);
        let sky = Sky {
            rules,
            held,
            fixed: BTreeSet::new(),
        };
        let found = unseen_refusals(&sky).unwrap_err();
        assert!(
            found.contains(&format!("`{}` now holds in the sky", UNSEEN[0].0)),
            "{found}"
        );
        assert!(
            found.contains(&format!("`{}` is listed in ALWAYS but fails", ALWAYS[0].0)),
            "{found}"
        );
        assert!(
            found.contains(&format!("`{}` does not hold at its fixture", FIXTURES[0].0)),
            "{found}"
        );
    }
}
