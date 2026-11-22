//! The façade's integration tests, one binary: each file of `tests/` is a
//! module here rather than a test target of its own, so the workspace links
//! the SDK once for them instead of once per file, and their tests run
//! side by side. A file keeps its own target only when it measures the
//! process (memory, time, a lazily loaded locale) or a gate runs it by name;
//! those are listed in `Cargo.toml`. A module's features are the
//! `required-features` its target had.

// What the corpus tests share; every module that reads it needs the
// built-in ephemeris.
#[cfg(feature = "builtin-ephemeris")]
mod common;

#[cfg(feature = "builtin-ephemeris")]
mod andhra_pradesh;
#[cfg(feature = "builtin-ephemeris")]
mod anniversaries;
#[cfg(feature = "builtin-ephemeris")]
mod anniversary;
#[cfg(feature = "builtin-ephemeris")]
mod annual_chart;
#[cfg(feature = "builtin-ephemeris")]
mod antiscia;
#[cfg(feature = "builtin-ephemeris")]
mod ashauch;
#[cfg(feature = "builtin-ephemeris")]
mod ashtakavarga;
#[cfg(feature = "builtin-ephemeris")]
mod assam;
#[cfg(feature = "builtin-ephemeris")]
mod baseline_chara_karakas;
#[cfg(feature = "builtin-ephemeris")]
mod bhava_bala;
#[cfg(feature = "builtin-ephemeris")]
mod bihar;
#[cfg(feature = "builtin-ephemeris")]
mod cells;
#[cfg(feature = "builtin-ephemeris")]
mod changes;
#[cfg(feature = "builtin-ephemeris")]
mod chart_disagreements;
#[cfg(all(feature = "builtin-ephemeris", feature = "tajika"))]
mod chart_zone;
mod classical_chart;
#[cfg(all(
    feature = "builtin-ephemeris",
    feature = "kp",
    feature = "prashna",
    feature = "tajika",
    feature = "western"
))]
mod compose;
#[cfg(feature = "builtin-ephemeris")]
mod composites;
#[cfg(feature = "builtin-ephemeris")]
mod consumer_dasha;
#[cfg(feature = "builtin-ephemeris")]
mod coverage_edges;
#[cfg(feature = "builtin-ephemeris")]
mod custom_ayanamsha;
#[cfg(feature = "builtin-ephemeris")]
mod dasha;
#[cfg(feature = "builtin-ephemeris")]
mod dasha_phala;
#[cfg(feature = "builtin-ephemeris")]
mod day_markers;
#[cfg(feature = "builtin-ephemeris")]
mod days_request;
#[cfg(feature = "builtin-ephemeris")]
mod death_time;
#[cfg(feature = "builtin-ephemeris")]
mod declinations;
#[cfg(feature = "builtin-ephemeris")]
mod digest;
#[cfg(feature = "builtin-ephemeris")]
mod dignities;
#[cfg(feature = "builtin-ephemeris")]
mod disagreements;
#[cfg(feature = "builtin-ephemeris")]
mod dossier;
#[cfg(feature = "builtin-ephemeris")]
mod eclipses;
mod engine_request;
#[cfg(feature = "builtin-ephemeris")]
mod every_profile;
#[cfg(feature = "builtin-ephemeris")]
mod festival;
#[cfg(feature = "builtin-ephemeris")]
mod festival_calendar;
#[cfg(feature = "builtin-ephemeris")]
mod festival_groups;
#[cfg(feature = "builtin-ephemeris")]
mod festival_meta;
#[cfg(feature = "builtin-ephemeris")]
mod festival_observer;
#[cfg(feature = "builtin-ephemeris")]
mod festival_rank;
#[cfg(all(feature = "builtin-ephemeris", feature = "chart", feature = "muhurta"))]
mod finders;
#[cfg(feature = "builtin-ephemeris")]
mod first_year;
#[cfg(feature = "builtin-ephemeris")]
mod fixed_points;
#[cfg(feature = "builtin-ephemeris")]
mod gaudiya;
#[cfg(feature = "builtin-ephemeris")]
mod geo;
#[cfg(feature = "builtin-ephemeris")]
mod gochar;
#[cfg(feature = "builtin-ephemeris")]
mod gujarat;
#[cfg(feature = "builtin-ephemeris")]
mod harmonics;
#[cfg(feature = "builtin-ephemeris")]
mod hits;
#[cfg(feature = "builtin-ephemeris")]
mod homa_ahuti;
mod in_era;
#[cfg(feature = "builtin-ephemeris")]
mod ishta_devata;
#[cfg(feature = "builtin-ephemeris")]
mod jaimini;
#[cfg(feature = "builtin-ephemeris")]
mod jain;
#[cfg(feature = "builtin-ephemeris")]
mod janma;
#[cfg(feature = "builtin-ephemeris")]
mod karnataka;
#[cfg(feature = "builtin-ephemeris")]
mod kerala;
#[cfg(feature = "builtin-ephemeris")]
mod ketaki;
#[cfg(feature = "builtin-ephemeris")]
mod kp;
#[cfg(all(feature = "builtin-ephemeris", feature = "lalkitab"))]
mod lalkitab;
#[cfg(feature = "builtin-ephemeris")]
mod lilith;
#[cfg(feature = "builtin-ephemeris")]
mod mahadvadashi;
#[cfg(feature = "builtin-ephemeris")]
mod masa_pravesha;
#[cfg(feature = "builtin-ephemeris")]
mod matching;
#[cfg(feature = "builtin-ephemeris")]
mod midpoints;
mod mithila;
#[cfg(feature = "builtin-ephemeris")]
mod moment;
#[cfg(feature = "builtin-ephemeris")]
mod month_margin;
#[cfg(feature = "builtin-ephemeris")]
mod moonrise;
#[cfg(feature = "builtin-ephemeris")]
mod muhurta;
#[cfg(all(feature = "builtin-ephemeris", feature = "western"))]
mod mundane;
#[cfg(feature = "chart")]
mod naming;
#[cfg(feature = "builtin-ephemeris")]
mod nepal_gazette;
#[cfg(feature = "builtin-ephemeris")]
mod next;
#[cfg(feature = "builtin-ephemeris")]
mod odisha;
#[cfg(feature = "builtin-ephemeris")]
mod outer_planets;
#[cfg(all(feature = "builtin-ephemeris", feature = "pakshi"))]
mod pakshi;
#[cfg(feature = "builtin-ephemeris")]
mod panchanga_reach;
#[cfg(feature = "builtin-ephemeris")]
mod parana;
#[cfg(feature = "builtin-ephemeris")]
mod parana_bar;
#[cfg(feature = "builtin-ephemeris")]
mod plans;
#[cfg(feature = "builtin-ephemeris")]
mod polar_day;
#[cfg(feature = "builtin-ephemeris")]
mod prashna;
#[cfg(feature = "builtin-ephemeris")]
mod printed_day;
#[cfg(feature = "builtin-ephemeris")]
mod progressions;
#[cfg(feature = "builtin-ephemeris")]
mod published;
#[cfg(feature = "builtin-ephemeris")]
mod punya_kala;
#[cfg(feature = "builtin-ephemeris")]
mod rashifal;
#[cfg(feature = "builtin-ephemeris")]
mod receipt;
#[cfg(all(feature = "builtin-ephemeris", feature = "rectification"))]
mod rectify;
#[cfg(feature = "builtin-ephemeris")]
mod remedies;
#[cfg(feature = "builtin-ephemeris")]
mod render;
#[cfg(all(feature = "builtin-ephemeris", feature = "research"))]
mod research;
#[cfg(feature = "builtin-ephemeris")]
mod returns;
#[cfg(feature = "builtin-ephemeris")]
mod ritu;
#[cfg(feature = "builtin-ephemeris")]
mod rules;
#[cfg(feature = "builtin-ephemeris")]
mod sade_sati;
#[cfg(feature = "builtin-ephemeris")]
mod sait;
#[cfg(feature = "builtin-ephemeris")]
mod samvatsara;
#[cfg(feature = "builtin-ephemeris")]
mod sankalpa;
#[cfg(feature = "builtin-ephemeris")]
mod sarvatobhadra;
#[cfg(feature = "builtin-ephemeris")]
mod sealing;
#[cfg(feature = "builtin-ephemeris")]
mod sentences;
mod settings_effects;
mod settings_explain;
#[cfg(feature = "builtin-ephemeris")]
mod shadbala;
#[cfg(feature = "builtin-ephemeris")]
mod sidecar;
mod signed_pack;
#[cfg(feature = "builtin-ephemeris")]
mod sky_pose;
#[cfg(feature = "builtin-ephemeris")]
mod sky_view;
#[cfg(feature = "builtin-ephemeris")]
mod synastry;
#[cfg(feature = "builtin-ephemeris")]
mod tamil_nadu;
#[cfg(feature = "builtin-ephemeris")]
mod telangana;
#[cfg(feature = "builtin-ephemeris")]
mod terrain_horizon;
#[cfg(all(feature = "builtin-ephemeris", feature = "chart"))]
mod threads;
#[cfg(feature = "builtin-ephemeris")]
mod time_lords;
mod time_request;
#[cfg(feature = "builtin-ephemeris")]
mod tithi_pravesha;
#[cfg(feature = "builtin-ephemeris")]
mod transit_centre;
#[cfg(feature = "builtin-ephemeris")]
mod turns;
#[cfg(feature = "builtin-ephemeris")]
mod twilight;
#[cfg(feature = "builtin-ephemeris")]
mod uttar_pradesh;
#[cfg(feature = "builtin-ephemeris")]
mod vaiseshikamsa;
#[cfg(feature = "builtin-ephemeris")]
mod vakya;
#[cfg(feature = "builtin-ephemeris")]
mod varga_schemes;
#[cfg(feature = "builtin-ephemeris")]
mod verify;
#[cfg(feature = "builtin-ephemeris")]
mod vimshopaka;
#[cfg(feature = "builtin-ephemeris")]
mod vrata_parana;
#[cfg(feature = "builtin-ephemeris")]
mod western_aspects;
#[cfg(feature = "builtin-ephemeris")]
mod western_houses;
#[cfg(feature = "builtin-ephemeris")]
mod year_rulers;
mod zone_pack;
#[cfg(all(feature = "builtin-ephemeris", feature = "full"))]
mod zone_requests;
