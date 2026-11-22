//! The root settings, cited knob by knob, and the shipped profiles as
//! patches over it. Only a resolved `Settings` is ever hashed; a profile
//! id is a name for a patch, never a hash input.

use std::collections::{BTreeMap, BTreeSet};

use crate::catalogue::{
    Ayanamsha, BalaScheme, Calendar, DashaFamily, DashaSystem, Era, HouseSystem, Mark, Source, Star,
};

use super::knobs::{
    AfterCycle, AmritKalam, AntardashaPlace, AshtakavargaGoodFrom, AshtottariGrouping,
    AyanamshaBasis, AyurdayaReadings, AyurdayaSpan, Balance, Benefics, BhavaBalaDivision, BhavaDig,
    BhavaDrishti, BhavaSpecialRules, BirthPeriod, BrahmaRule, Centre, CharaKarakas, Cheshta,
    DayBoundary, DayLordDay, DecennialDivision, DeltaT, DigKendras, Drekkana, Drik, DstGap,
    DstOverlap, DualLord, DurmuhurtaTable, EclipseShadow, EclipseVedha, EightKarakaOrder,
    Ekadhipatya, FestivalObserver, FirdariaNodes, FrameBias, GhatiReckoning, GrahaArudhaException,
    GrahaDivision, HoraReckoning, IshtaKashta, KaalaLords, KalachakraAfterNinth, KalachakraBalance,
    KalachakraMembership, KiratYeleEpoch, Kranti, LuminaryCheshta, LunarMonth, MaranaSaturn,
    MaranaVenus, MoonBinduFromJupiter, MoonEvents, MoonPaksha, Naisargika, NaisargikaDivision,
    NakshatraScheme, Nathonnatha, Node, NodeAspects, NodeCoLordship, NodeObstruction, NodeRulers,
    NodeVedha, Nutation, OverridePolicy, PanchakaName, PanchakaStart, PolarDayPolicy, PolarPolicy,
    Positions, PreDawnNight, Precession, PunyaKalaTable, PushkaraBhaga, RashiStart,
    ReleasingSharedSign, RequiredRupas, RetrogradeRejection, RituReckoning, RulingCount,
    SamvatsaraCount, Saptavargaja, SayanadiGhatis, SayanadiNodes, SeedOverflow, SevenKarakaOrder,
    ShantaSign, ShivaVasaReading, Shodhana, SiderealTime, SolarMonthStart, StrongestGraha,
    SunAyana, Sunrise, SuryaSunrise, SuryaTrig, Tier, TransitCentre, UnattestedDn, UnknownTime,
    Varjyam, Vimshopaka, YearLength, Yuddha, Zodiac,
};
use super::{
    Aspect, AyanamshaChoice, Calendars, Citation, Dasha, Day, Diagnostics, Dut1, Festivals, Frame,
    Gochar, HorizonModel, Houses, Jaimini, Kp, LagnaTable, Layer, Origins, Output, Panchanga,
    Precision, Provider, Rashifal, Resolved, SCHEMA, ScoreBands, Settings, SettingsPatch,
    Siddhanta, State, Strength, SuryaBija, SuryaPlanets, Time, Vargas,
};
use crate::quantity::Depth;

/// A profile's id.
///
/// ```
/// use teistro_core::settings::ProfileId;
///
/// let id: ProfileId = serde_json::from_str(r#""nepali-drik""#).unwrap();
/// assert_eq!(id, ProfileId::new("nepali-drik"));
/// assert_eq!(id.to_string(), "nepali-drik");
/// ```
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct ProfileId(String);

impl ProfileId {
    /// An id.
    ///
    /// ```
    /// use teistro_core::settings::ProfileId;
    ///
    /// assert_eq!(ProfileId::new("mine").as_str(), "mine");
    /// ```
    #[must_use]
    pub fn new(id: &str) -> ProfileId {
        ProfileId(id.to_string())
    }

    /// The id.
    ///
    /// ```
    /// use teistro_core::settings::{DEFAULT_PROFILE, Profile};
    ///
    /// assert_eq!(Profile::shipped(DEFAULT_PROFILE).unwrap().id.as_str(), DEFAULT_PROFILE);
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for ProfileId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A named, versioned patch over a base.
///
/// ```
/// use teistro_core::settings::{Profile, SettingsPatch};
///
/// let committee = Profile::shipped("nepali-committee").unwrap();
/// // One level of base at most, and a citation for what it sets.
/// assert!(committee.base.as_ref().is_none_or(|base| Profile::shipped(base.as_str()).is_some()));
/// assert!(!committee.sources.is_empty());
/// assert!(committee.resolve(&SettingsPatch::default()).is_ok());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    /// The id.
    pub id: ProfileId,
    /// Bumps when a default changes; a major release.
    pub version: u16,
    /// The base, one level; `None` means the root.
    pub base: Option<ProfileId>,
    /// What this profile sets over its base.
    pub patch: SettingsPatch,
    /// Why each default.
    pub sources: Vec<Citation>,
    /// The profile's confidence.
    pub mark: Mark,
}

/// The profile a context uses when the consumer names none: the texts as
/// read, with nothing of any one country's practice in it (ADR-0024). A
/// binding's constructor and the C ABI's `ts_context_new` both fall back
/// to it.
///
/// It is the one profile chosen by not choosing, so it is the one that
/// has to need the least defending: every value in it is cited to a text
/// or to the root, and nothing is inherited from an implementation's
/// convenience. The choice matters — the centre alone moves the Moon by
/// up to 39 arcminutes, which changes a Vimshottari mahadasha lord in one
/// of the six charts the corpus records both ways.
///
/// ```
/// use teistro_core::settings::{DEFAULT_PROFILE, SHIPPED_PROFILES};
///
/// assert_eq!(DEFAULT_PROFILE, "parashari-classical");
/// assert!(SHIPPED_PROFILES.contains(&DEFAULT_PROFILE));
/// ```
pub const DEFAULT_PROFILE: &str = "parashari-classical";

/// The ids of the shipped profiles.
///
/// ```
/// use teistro_core::settings::{Profile, SettingsPatch, SHIPPED_PROFILES};
///
/// // Every one resolves, with nothing patched.
/// for id in SHIPPED_PROFILES {
///     assert!(Profile::shipped(id).unwrap().resolve(&SettingsPatch::default()).is_ok(), "{id}");
/// }
/// ```
pub const SHIPPED_PROFILES: [&str; 11] = [
    "nepali-drik",
    "nepali-committee",
    "parashari-classical",
    "surya-siddhanta",
    "mithila-makaranda",
    "tamil-vakya",
    "ketaki",
    "tilak-revati",
    "kp-default",
    "western-tropical-default",
    "conformance-baseline",
];

/// The names a shipped profile answers to beside its id, each `(alias, id)`.
///
/// An alias opens exactly the profile it names: the same settings, so the
/// same hash, and the answer's `provenance.profile` and a context's
/// `profile()` say the id and never the alias. The name a stored document
/// or an older caller holds keeps working; nothing writes it back.
///
/// ```
/// use teistro_core::settings::{PROFILE_ALIASES, Profile};
///
/// for (alias, id) in PROFILE_ALIASES {
///     assert_eq!(Profile::shipped(alias), Profile::shipped(id));
/// }
/// ```
pub const PROFILE_ALIASES: [(&str, &str); 1] = [("nepali-default", "nepali-drik")];

const BASELINE: Source = Source::new("baseline-engine", "persisted chart settings");

/// The SDK's own decisions, where no text decides a convention.
const fn decided(reference: &'static str) -> Source {
    Source::new("teistro-sdk design", reference)
}

const BPHS: &str = "Brihat Parashara Hora Shastra";
const SURYA: &str = "Surya Siddhanta (Burgess 1860)";
const RAMAN: &str = "B.V. Raman, Graha and Bhava Balas";
const KP_VI: &str = "K.S. Krishnamurti, KP Reader VI";

/// **The root's citations**, one for every knob, beside the values in
/// [`root`]: why each default is what it is (SR-110). An explanation of
/// any shipped profile reads a knob the profile does not set from here,
/// so `every_knob_of_the_root_is_cited` holds this list to
/// [`Settings::knobs`] both ways.
pub(crate) const ROOT_SOURCES: &[Citation] = &[
    Citation::new(
        "frame.zodiac",
        Source::new(
            SURYA,
            "chs. I and II: longitudes counted from the start of Ashvini, a point among the stars; ADR-0024",
        ),
    ),
    Citation::new(
        "frame.ayanamsha",
        Source::new(
            "Calendar Reform Committee, Government of India (1955)",
            "Report: Lahiri's ayanamsha, adopted for the Indian Astronomical Ephemeris and the Rashtriya Panchang",
        ),
    ),
    Citation::new(
        "frame.ayanamsha_basis",
        Source::new(
            "Indian Astronomical Ephemeris",
            "the nirayana longitude is the apparent longitude less the true ayanamsha, the mean plus the nutation in longitude; the Rashtriya Panchang's 109 sankrantis of 2018 to 2027 meet it within 32 seconds; SR-100, C1750",
        ),
    ),
    Citation::new(
        "frame.node",
        Source::new(
            SURYA,
            "ch. II corrects the planets by their equations and gives the node its mean place alone; the conformance corpus's engine reads the mean node too",
        ),
    ),
    Citation::new(
        "frame.centre",
        Source::new(
            "ADR-0024",
            "the classical graha sphuta is computed from the Earth's centre, as every published table is; the lambana is an eclipse correction, not a natal one",
        ),
    ),
    Citation::new(
        "frame.positions",
        Source::new(
            "Indian Astronomical Ephemeris",
            "apparent places, light time, aberration and deflection applied, as the ephemeris tabulates and a drik panchanga reads them",
        ),
    ),
    Citation::new(
        "frame.siddhanta",
        Source::new(
            "ADR-0029",
            "modern astronomy unless the settings name a text's, which the provider then supplies; docs/03-design/classical-chart.md",
        ),
    ),
    Citation::new(
        "frame.nakshatra_scheme",
        Source::new(
            SURYA,
            "II.64: a nakshatra is 800 minutes of arc, twenty-seven to the circle",
        ),
    ),
    Citation::new(
        "houses.placement_system",
        Source::new(
            BPHS,
            "the rashi chakra: each sign from the lagna's is one bhava; docs/03-design/astro-house-systems.md",
        ),
    ),
    Citation::new(
        "houses.chalit_system",
        Source::new(
            BPHS,
            "the Sripati bhava: the angles trisected, each madhya its bhava's middle; ADR-0024, docs/03-design/chart-bhava-chalit.md",
        ),
    ),
    Citation::new(
        "houses.module_overrides",
        Source::new(
            "K.S. Krishnamurti, KP Reader I",
            "the cusps by Placidus's system, which KP reads its sub lords from; docs/03-design/kp.md",
        ),
    ),
    Citation::new(
        "houses.polar_policy",
        decided(
            "docs/03-design/astro-house-systems.md §4: a system with no solution inside the polar circle is refused, not replaced unasked",
        ),
    ),
    Citation::new(
        "day.sunrise",
        Source::new(
            "The Astronomical Almanac",
            "sunrise as the upper limb on the horizon with 34 minutes of refraction, which Drik Panchang and the Swiss Ephemeris default to; docs/03-design/sunrise-convention.md",
        ),
    ),
    Citation::new(
        "day.moonrise",
        Source::new(
            "U.S. Naval Observatory, Rise, Set, and Twilight Definitions",
            "moonrise as the Moon's upper limb on the horizon with 34 minutes of refraction, her semidiameter and horizontal parallax at her distance, as the Explanatory Supplement (§9.33) and Meeus (ch. 15) define it; docs/03-design/moonrise-convention.md, C2000",
        ),
    ),
    Citation::new(
        "day.day_boundary",
        Source::new(
            SURYA,
            "ch. I: the civil (savana) day runs from sunrise to sunrise",
        ),
    ),
    Citation::new(
        "day.polar_day_policy",
        Source::new(
            "ADR-0024",
            "the texts define no sunrise where the Sun does not rise, so such a day is reported undefined rather than synthesised",
        ),
    ),
    Citation::new(
        "day.ghati_reckoning",
        Source::new(
            SURYA,
            "I.11 and 12: sixty nadis (ghatis) to the day, twenty-four minutes each",
        ),
    ),
    Citation::new(
        "day.hora_reckoning",
        Source::new(
            "baseline-engine",
            "the fixtures' hora lords: the day and the night each in twelve proportional horas",
        ),
    ),
    Citation::new(
        "panchanga.centre",
        decided(
            "docs/03-design/panchanga-day.md §10: every published panchanga's limbs are geocentric; the corpus measures the difference at five classifications in 136",
        ),
    ),
    Citation::new(
        "panchanga.moon_events",
        decided(
            "docs/03-design/panchanga-day.md: the Moon's rise and set over the sunrise-to-sunrise window the limbs run over; registry entry 18",
        ),
    ),
    Citation::new(
        "panchanga.muhurta_tables",
        Source::new(
            "Muhurta Chintamani",
            "the yogas read at the star and tithi of sunrise, as the almanacs print them; docs/03-design/day-markers.md §2",
        ),
    ),
    Citation::new(
        "panchanga.panchaka_start",
        Source::new(
            "Muhurta Chintamani",
            "Nakshatra Prakarana v. 48: the Moon in Aquarius or Pisces; crux C158",
        ),
    ),
    Citation::new(
        "panchanga.panchaka_name",
        decided(
            "docs/03-design/printed-conventions.md: the almanacs' weekday name, flagged in every answer's applied conventions; crux C821",
        ),
    ),
    Citation::new(
        "panchanga.shiva_vasa",
        Source::new(
            "Meru Tantra",
            "the table Surya Panchanga VS 2082 p. 60 prints, (2t + 5) mod 7 in every cell; crux C750",
        ),
    ),
    Citation::new(
        "panchanga.varjyam",
        Source::new(
            "Muhurta Chintamani",
            "Vivaha Prakarana vv. 49 to 51, the vishaghati table; crux C165",
        ),
    ),
    Citation::new(
        "panchanga.amrit_kalam",
        Source::new(
            "B.V. Raman, Muhurtha",
            "the printed table, which gives Drik Panchang's every window at Delhi over a week of June 2025 to the minute; crux C677",
        ),
    ),
    Citation::new(
        "panchanga.lagna_table",
        decided(
            "the ascendant a chart of the same context has; crux C680, docs/03-design/amrit-kalam-and-lagnas.md §2",
        ),
    ),
    Citation::new(
        "panchanga.durmuhurta",
        Source::new(
            "Muhurta Chintamani",
            "Vivaha Prakarana vv. 50 to 52 (1906 ed., p. 89); crux C398",
        ),
    ),
    Citation::new(
        "panchanga.punya_kala",
        Source::new(
            "Dharmasindhu",
            "pp. 2 and 3: the ghatis before and after each sankranti; crux C752",
        ),
    ),
    Citation::new(
        "panchanga.ritu",
        Source::new(
            SURYA,
            "XIV.10: the seasons by two solar months each from Makara; Nepal's daily panchanga over 341 days; crux C178",
        ),
    ),
    Citation::new(
        "panchanga.solar_month_start",
        Source::new(
            "Dharmasindhu",
            "the sankranti's punya kala places the solar month's first day; Sewell and Dikshit §28; crux C186",
        ),
    ),
    Citation::new(
        "panchanga.eclipse_shadow",
        Source::new(
            "Danjon (1951)",
            "the Earth's shadow enlarged for its atmosphere, as NASA's eclipse catalogue and the French almanac compute it; docs/03-design/eclipses.md",
        ),
    ),
    Citation::new(
        "panchanga.eclipse_horizon",
        Source::new(
            "Dharmasindhu",
            "an eclipse counts where the eye sees it: the upper limb lifted by refraction; docs/03-design/eclipses.md §5",
        ),
    ),
    Citation::new(
        "panchanga.eclipse_vedha",
        Source::new(
            "Dharmasindhu",
            "p. 28: four praharas before a solar eclipse, three before a lunar one; crux C192",
        ),
    ),
    Citation::new(
        "time.dst_gap",
        decided(
            "docs/03-design/time-and-timezone.md §4: a civil time a zone's clocks skipped is refused with both offsets rather than moved unasked",
        ),
    ),
    Citation::new(
        "time.dst_overlap",
        decided(
            "docs/03-design/time-and-timezone.md §4: a civil time a zone's clocks show twice takes the earlier, as the baseline engine does",
        ),
    ),
    Citation::new(
        "time.unknown_time",
        decided(
            "docs/03-design/time-and-timezone.md §4: a birth without a time is refused, naming the fallbacks, rather than given one unasked",
        ),
    ),
    Citation::new(
        "time.delta_t",
        Source::new(
            "Espenak and Meeus (2006)",
            "the observed table where it runs, their polynomials beyond it; docs/03-design/time-and-timezone.md",
        ),
    ),
    Citation::new(
        "dasha.balance",
        Source::new(
            BPHS,
            "ch. 46: the first dasha's balance is the part of the birth nakshatra yet to run, by the Moon's longitude; the corpus's engine agrees",
        ),
    ),
    Citation::new(
        "dasha.year_length",
        decided(
            "crux C6: 365.25 days for the nakshatra dashas, as the corpus's engine runs them; 360 for the ayurdaya, releasing and decennial families, as their texts count years",
        ),
    ),
    Citation::new(
        "dasha.depth",
        decided(
            "three levels, the mahadasha to the pratyantardasha, the depth an almanac prints; any system goes to six on request",
        ),
    ),
    Citation::new(
        "dasha.seed_overflow",
        decided(
            "a conditional dasha seeded outside its cycle wraps to its start, as the corpus's engine does",
        ),
    ),
    Citation::new(
        "dasha.ashtottari_grouping",
        Source::new(
            BPHS,
            "ch. 46 vv. 17 to 20: from Ardra in fours and threes alternately; crux C5",
        ),
    ),
    Citation::new(
        "dasha.birth_period",
        Source::new(
            "baseline-engine",
            "the birth mahadasha's antardashas compressed into its balance on all 148 recorded answers; crux C48",
        ),
    ),
    Citation::new(
        "dasha.after_cycle",
        Source::new(
            "baseline-engine",
            "no period past the ninth mahadasha on every recorded answer; crux C48",
        ),
    ),
    Citation::new(
        "dasha.kalachakra_membership",
        Source::new(
            "baseline-engine",
            "the recording engine's pada tables, measured on every recorded answer; crux C54",
        ),
    ),
    Citation::new(
        "dasha.kalachakra_balance",
        Source::new(
            "baseline-engine",
            "the unelapsed part of the pada of the first sign's years; crux C55",
        ),
    ),
    Citation::new(
        "dasha.kalachakra_after_ninth",
        Source::new(
            "baseline-engine",
            "the nine signs reversed after the ninth; crux C56",
        ),
    ),
    Citation::new(
        "dasha.shanta_sign",
        Source::new(
            BPHS,
            "ch. 47 vv. 5 and 6: a friend's or a great friend's sign; crux C79",
        ),
    ),
    Citation::new(
        "dasha.dual_lord",
        Source::new(BPHS, "ch. 46 vv. 158 to 166; crux C51"),
    ),
    Citation::new(
        "dasha.rashi_start",
        Source::new(BPHS, "ch. 46 vv. 174 and 179 to 184; crux C53"),
    ),
    Citation::new(
        "dasha.releasing_shared_sign",
        Source::new(
            "Vettius Valens, Anthologies",
            "IV.4: activity from the sign immediately following a shared one; crux C223",
        ),
    ),
    Citation::new(
        "dasha.firdaria_nodes",
        Source::new(
            "al-Biruni, Book of Instruction",
            "§438's table, with Abu Ma'shar: the Head and the Tail after the seven; crux C225",
        ),
    ),
    Citation::new(
        "dasha.decennial_division",
        Source::new(
            "Vettius Valens, Anthologies",
            "VI.6's tables: in proportion to the minor years at every level; crux C228",
        ),
    ),
    Citation::new(
        "dasha.karaka_division",
        Source::new(
            BPHS,
            "ch. 51 vv. 1 and 2: in proportion, from the period's own lord; crux C737",
        ),
    ),
    Citation::new(
        "dasha.strongest_graha",
        Source::new(
            BPHS,
            "ch. 46 v. 208 names no measure; the shadbala total every shadbala chapter ranks by; crux C735",
        ),
    ),
    Citation::new(
        "dasha.ayurdaya_place",
        Source::new(
            BPHS,
            "ch. 51 v. 14: one body per share the verse names; crux C741",
        ),
    ),
    Citation::new(
        "dasha.naisargika_division",
        Source::new(
            BPHS,
            "ch. 51 vv. 13 to 16: shares among the seven by their signs and shadbala; crux C743",
        ),
    ),
    Citation::new(
        "dasha.ayurdaya_span",
        Source::new(
            BPHS,
            "ch. 43 vv. 30 to 32: the span of the strongest of the lagna, the Sun and the Moon; crux C742",
        ),
    ),
    Citation::new(
        "dasha.ayurdaya_readings",
        Source::new(BPHS, "ch. 43's reductions; crux C742"),
    ),
    Citation::new(
        "jaimini.chara_karakas",
        Source::new(
            BPHS,
            "ch. 32: seven karakas, the Sun to Saturn; the eight with Rahu is the knob's other value",
        ),
    ),
    Citation::new(
        "jaimini.eight_karaka_order",
        Source::new(BPHS, "ch. 32 vv. 13 to 17, the verses' order; crux C101"),
    ),
    Citation::new(
        "jaimini.node_co_lordship",
        Source::new(
            BPHS,
            "ch. 46 v. 171 read with the chapter's own lordship; crux C126",
        ),
    ),
    Citation::new(
        "jaimini.brahma",
        Source::new(
            BPHS,
            "ch. 46 vv. 170 to 173 as read; the translator's note is the other value; crux C128",
        ),
    ),
    Citation::new(
        "jaimini.graha_arudha_exception",
        Source::new(
            BPHS,
            "ch. 29 vv. 6 and 7: the count, and no move; crux C132",
        ),
    ),
    Citation::new(
        "gochar.node_vedha",
        Source::new(
            "Phaladeepika",
            "ch. 26 v. 2: the nodes like the Sun; crux C136",
        ),
    ),
    Citation::new(
        "gochar.node_obstruction",
        Source::new(
            "Phaladeepika",
            "ch. 26 vv. 3 to 8: planets obstruct, and the nodes never each other; cruxes C137, C140",
        ),
    ),
    Citation::new(
        "gochar.ashtakavarga_good_from",
        Source::new(
            "Phaladeepika",
            "ch. 23 v. 11: four bindus a fear, five wealth; crux C141",
        ),
    ),
    Citation::new(
        "gochar.transit_centre",
        decided("each transit from the chart's own centre, so one chart reads one sky; crux C778"),
    ),
    Citation::new(
        "kp.ruling_count",
        Source::new(KP_VI, "these five planets; crux C150"),
    ),
    Citation::new(
        "kp.day_lord_day",
        Source::new(KP_VI, "the day from sunrise to the next sunrise; crux C151"),
    ),
    Citation::new(
        "kp.node_rulers",
        Source::new(
            KP_VI,
            "a node in a ruler's sign, or conjoined as the worked example takes Ketu; crux C152",
        ),
    ),
    Citation::new(
        "kp.retrograde_rejection",
        Source::new(
            KP_VI,
            "rulers in a retrograde planet's constellation; crux C153",
        ),
    ),
    Citation::new(
        "aspect.node_aspects",
        Source::new(
            BPHS,
            "ch. 26: special drishtis for Mars, Jupiter and Saturn, none for the nodes beyond the seventh",
        ),
    ),
    Citation::new(
        "aspect.drishti_table",
        Source::new(BPHS, "ch. 26: the sphuta drishti by the angular distance"),
    ),
    Citation::new(
        "state.combustion_orbs",
        Source::new(
            SURYA,
            "IX.6 to 8: the degrees of time, with the inner orb the corpus brackets; docs/03-design/state-tables-measured.md §3",
        ),
    ),
    Citation::new(
        "state.sayanadi_ghatis",
        Source::new(
            BPHS,
            "ch. 45 vv. 30 to 37: the ghatis of birth elapsed; crux C78",
        ),
    ),
    Citation::new(
        "state.sayanadi_nodes",
        Source::new(
            BPHS,
            "ch. 45 v. 30 and ch. 3's order of the nine: Rahu 8, Ketu 9; crux C78",
        ),
    ),
    Citation::new(
        "state.pushkara_bhaga",
        Source::new(
            "Vidyamadhaviyam",
            "ch. 1 v. 18 (Shama Sastry, Mysore 1923, pp. 80 and 81): one degree a triplicity, every graha; crux C767",
        ),
    ),
    Citation::new(
        "state.marana_saturn",
        Source::new(
            "Jataka Parijata",
            "ch. 17 vv. 34 and 35 (Sastri, vol. II p. 982): janmaga, the lagna's sign; crux C768",
        ),
    ),
    Citation::new(
        "state.marana_venus",
        Source::new(
            "Jataka Parijata",
            "ch. 17 vv. 34 and 35: the sixth sign; crux C768",
        ),
    ),
    Citation::new(
        "strength.bala_scheme",
        Source::new(BPHS, "ch. 27: the six components of the Shadbala"),
    ),
    Citation::new(
        "strength.ekadhipatya",
        Source::new(BPHS, "ch. 68; crux C60"),
    ),
    Citation::new(
        "strength.shodhana",
        Source::new(
            BPHS,
            "chs. 67 to 69: each graha's own Ashtakavarga; crux C59",
        ),
    ),
    Citation::new(
        "strength.moon_bindu_from_jupiter",
        Source::new(BPHS, "ch. 66 and Varahamihira: the 12th; crux C144"),
    ),
    Citation::new(
        "strength.vimshopaka",
        Source::new(BPHS, "ch. 7 vv. 21 to 27; crux C63"),
    ),
    Citation::new(
        "strength.saptavargaja",
        Source::new(
            RAMAN,
            "the compound relationship's figures, Example 56; crux C64",
        ),
    ),
    Citation::new(
        "strength.nathonnatha",
        Source::new(RAMAN, "from midnight, as BPHS ch. 27 vv. 8 and 9; crux C65"),
    ),
    Citation::new(
        "strength.pre_dawn_night",
        Source::new(
            RAMAN,
            "a birth before sunrise in the night that began the evening before; crux C65",
        ),
    ),
    Citation::new(
        "strength.sun_ayana",
        Source::new(
            RAMAN,
            "the Sun's Ayana bala doubled, as BPHS ch. 27 v. 17; crux C66",
        ),
    ),
    Citation::new(
        "strength.moon_paksha",
        Source::new(RAMAN, "the Moon's Paksha bala doubled; crux C66"),
    ),
    Citation::new(
        "strength.luminary_cheshta",
        Source::new(
            RAMAN,
            "after Sripati: no Cheshta bala for the luminaries; crux C66",
        ),
    ),
    Citation::new(
        "strength.kranti",
        Source::new(
            RAMAN,
            "after Sripati: the Hindu table of declinations, 24 degrees; crux C66",
        ),
    ),
    Citation::new(
        "strength.kaala_lords",
        Source::new(
            RAMAN,
            "Arts. 58 to 62: the Abda and Masa lords from the ahargana, as BPHS ch. 27 v. 13; crux C67",
        ),
    ),
    Citation::new(
        "strength.dig",
        Source::new(RAMAN, "the four angles, as BPHS ch. 27 v. 7; crux C68"),
    ),
    Citation::new(
        "strength.drik",
        Source::new(
            RAMAN,
            "after Sripati: a quarter of each drishti received; crux C69",
        ),
    ),
    Citation::new(
        "strength.naisargika",
        Source::new(
            RAMAN,
            "the natural strengths exact, as BPHS ch. 27 v. 14; crux C71",
        ),
    ),
    Citation::new(
        "strength.required_rupas",
        Source::new(
            RAMAN,
            "Art. 122, after Sripati: the Sun's five rupas; crux C71",
        ),
    ),
    Citation::new(
        "strength.drekkana",
        Source::new(RAMAN, "after Sripati: male, neuter and female; crux C72"),
    ),
    Citation::new(
        "strength.benefics",
        Source::new(
            RAMAN,
            "the Moon by her phase and Mercury by his company; crux C69",
        ),
    ),
    Citation::new(
        "strength.cheshta",
        Source::new(
            RAMAN,
            "Sripati's mean elements, the only ones a source gives; crux C70",
        ),
    ),
    Citation::new(
        "strength.yuddha",
        Source::new(
            RAMAN,
            "after Sripati: grahas at war gain and lose the Yuddha bala; crux C70",
        ),
    ),
    Citation::new(
        "strength.ishta_kashta",
        Source::new(RAMAN, "after Sripati: the square roots; crux C76"),
    ),
    Citation::new(
        "strength.bhava_division",
        Source::new(
            RAMAN,
            "Art. 29: the bhavas are the chalit's, from the angles; crux C73",
        ),
    ),
    Citation::new(
        "strength.bhava_dig",
        Source::new(RAMAN, "after Sripati: the bhava's sign class; crux C73"),
    ),
    Citation::new(
        "strength.bhava_drishti",
        Source::new(
            RAMAN,
            "after Sripati: the sphuta drishti a bhava receives; crux C74",
        ),
    ),
    Citation::new(
        "strength.bhava_special_rules",
        Source::new(
            RAMAN,
            "after Sripati: none of BPHS ch. 27 vv. 30 and 31's additions; crux C75",
        ),
    ),
    Citation::new(
        "vargas.unattested_dn",
        decided(
            "a varga no text attests is divided cyclically (parivritti), the rule the attested equal vargas follow; docs/03-design/varga-kernel.md",
        ),
    ),
    Citation::new(
        "calendars.civil_calendar",
        Source::new(
            "ADR-0024",
            "no one country's practice in the default: the Gregorian calendar",
        ),
    ),
    Citation::new(
        "calendars.lunar_month",
        Source::new(
            "ADR-0024",
            "the amanta month, the astrological one, from new moon to new moon",
        ),
    ),
    Citation::new(
        "calendars.samvatsara",
        Source::new(
            SURYA,
            "I.55: the Jovian year by Jupiter's mean sign, read as Nepal's committee reads it; cruxes C180, C184",
        ),
    ),
    Citation::new(
        "calendars.eras",
        Source::new(
            "ADR-0024",
            "the three pan-Indic eras, Vikrama, Shaka and Kali",
        ),
    ),
    Citation::new(
        "provider.overrides",
        Source::new(
            "ADR-0013",
            "a provider's own implementation where it declares one, the SDK's elsewhere",
        ),
    ),
    Citation::new(
        "provider.tier",
        Source::new("ADR-0028", "the built-in ephemeris's standard tier"),
    ),
    Citation::new(
        "provider.cache_cells",
        decided(
            "docs/03-design/batch-and-parallelism-measured.md: what a fifty-day almanac range needs, measured",
        ),
    ),
    Citation::new(
        "output.precision",
        Source::new(
            "ADR-0011",
            "nine decimals of a degree, eight of a day and three of a score: below every input's own precision",
        ),
    ),
    Citation::new(
        "frame.precession",
        Source::new(
            "Vondrák, Capitaine and Wallace (2011)",
            "A&A 534, A22: valid over ±200 000 years, so at every catalogued epoch; crux C1752",
        ),
    ),
    Citation::new(
        "frame.nutation",
        Source::new(
            "McCarthy and Luzum (2003)",
            "IAU 2000B, the SDK's own series; docs/03-design/astro-timescales-and-frames.md",
        ),
    ),
    Citation::new(
        "frame.frame_bias",
        Source::new(
            "IERS Conventions (2010)",
            "§5.4.4: the IAU 2006 frame bias between the GCRS and the dynamical frame of J2000",
        ),
    ),
    Citation::new(
        "day.horizon",
        Source::new(
            "ADR-0024",
            "what every almanac and the SDK have reckoned by: Bennett's refraction, the actual disc of the IAU 2015 radii, the body's own latitude, no dip and the ideal horizon; docs/03-design/horizon-atmosphere.md",
        ),
    ),
    Citation::new(
        "time.dut1",
        Source::new(
            "IERS Bulletin A",
            "leap seconds keep UTC within 0.9 s of UT1, so UT1 is taken as UTC: what a civil clock reads and every almanac assumes",
        ),
    ),
    Citation::new(
        "time.sidereal_time",
        Source::new(
            "IERS Conventions (2010)",
            "eq. 5.32: the Earth rotation angle with the IAU 2006 polynomial, the SDK's own",
        ),
    ),
    Citation::new(
        "vargas.schemes",
        Source::new(
            BPHS,
            "ch. 6: Parashara's division for every chart no scheme is named for; docs/03-design/varga-schemes.md",
        ),
    ),
    Citation::new(
        "rashifal.score_bands",
        Source::new(
            "Tithipatra (SR-124)",
            "75, 50 and 30, the consumer's bands, since no text bands a score; docs/03-design/rashifal.md, The score band",
        ),
    ),
    Citation::new(
        "jaimini.seven_karaka_order",
        Source::new(
            "Jaimini Sutras (B. Suryanarain Rao)",
            "1.1.13 to 18: Atma, Amatya, Bhratri, Matri, Putra, Gnati and Dara, the ranks' names when seven are counted; crux C2061",
        ),
    ),
    Citation::new(
        "calendars.marginal_month_minutes",
        Source::new(
            "Nepal Panchanga Nirnayak Samiti",
            "covers every month the committee and a drik almanac decided differently in AD 1900 to 2100, the widest by 6.3 hours",
        ),
    ),
    Citation::new(
        "calendars.kirat_yele",
        Source::new(
            "Kantipur, 15 January 2026",
            "KIRATI, the count printed today: 5086 from Magh 1, 2082 BS; docs/calendars/bikram-sambat.md, Kirat Yele Sambat",
        ),
    ),
    Citation::new(
        "festivals.observer",
        Source::new(
            "Nepal Panchanga Nirnayak Samiti",
            "EVERY: the committee prints both the Smartas' and the Vaishnavas' Ekadashi where they part (VS 2082 p. 6, VS 2083 p. 8; crux C688), and no shipped profile is a sect's; docs/03-design/festival-traditions.md §6, crux C2147",
        ),
    ),
];

fn depth_all(depth: u8) -> BTreeMap<DashaSystem, Depth> {
    DashaSystem::ALL
        .iter()
        .map(|d| (*d, Depth::try_new(depth).unwrap_or(Depth::MIN)))
        .collect()
}

/// KP Reader VI's readings of the four forks the baseline engine takes
/// otherwise (cruxes C150 to C153; `03-design/kp.md`).
fn kp_readers() -> Kp {
    Kp {
        ruling_count: RulingCount::Five,
        day_lord_day: DayLordDay::Sunrise,
        node_rulers: NodeRulers::SignOrConjoined,
        retrograde_rejection: RetrogradeRejection::Star,
    }
}

/// The Jaimini readers: seven karakas, the verses' order and rules as read.
fn jaimini() -> Jaimini {
    Jaimini {
        chara_karakas: CharaKarakas::Seven,
        // The verses' order (BPHS ch. 32 vv. 13 to 17; crux C101).
        eight_karaka_order: EightKarakaOrder::Parashara,
        node_co_lordship: NodeCoLordship::None,
        // BPHS ch. 46 vv. 170 to 173 as read; the translator's note is
        // the knob's other value (`03-design/jaimini-significators.md`).
        brahma: BrahmaRule::Verses,
        // BPHS ch. 29 vv. 6 and 7 state the count and no move
        // (`03-design/graha-arudhas.md`).
        graha_arudha_exception: GrahaArudhaException::None,
        // The Jaimini Sutras 1.1.13 to 18, the seven the recording engine
        // names and its corpus reproduces (crux C2061).
        seven_karaka_order: SevenKarakaOrder::Jaimini,
    }
}

/// Every reading a request carries, as before the knob: Nepal's committee
/// prints both observers' Ekadashi where they part (C688), and no shipped
/// profile is a sect's (`03-design/festival-traditions.md` §6, C2147).
const fn festivals() -> Festivals {
    Festivals {
        observer: FestivalObserver::Every,
    }
}

/// The civil calendar, the lunar month and the year's names: the
/// Surya Siddhanta's Jovian count (I.55), read as Nepal's committee reads
/// it (C180, C184; `03-design/samvatsara-measured.md`).
fn calendars() -> Calendars {
    Calendars {
        civil_calendar: Calendar::Gregorian,
        lunar_month: LunarMonth::Amanta,
        samvatsara: SamvatsaraCount::Barhaspatya,
        eras: [Era::Vikrama, Era::Shaka, Era::Kali].into_iter().collect(),
        marginal_month_minutes: crate::settings::DEFAULT_MARGINAL_MONTH_MINUTES,
        // The count printed today (`calendars/bikram-sambat.md` §Kirat
        // Yele Sambat).
        kirat_yele: KiratYeleEpoch::Kirati,
    }
}

/// The root's daily panchanga: the knobs every profile starts from.
fn root_panchanga() -> Panchanga {
    Panchanga {
        // An almanac is geocentric wherever the chart is; the daily
        // limbs of every published panchanga are, and the corpus
        // measures the difference at five classifications in 136
        // (`03-design/panchanga-day.md` §10).
        centre: Centre::Geocentric,
        moon_events: MoonEvents::Window,
        muhurta_tables: String::from("CLASSICAL"),
        panchaka_start: PanchakaStart::Aquarius,
        // Printed conventions, each flagged in every answer's
        // `applied_conventions`; the default answers and refuses nothing
        // (`03-design/printed-conventions.md`, C821, C750).
        panchaka_name: PanchakaName::Weekday,
        shiva_vasa: ShivaVasaReading::Printed,
        // Muhurta Chintamani's table, the rank-1 text (crux C165).
        varjyam: Varjyam::MuhurtaChintamani,
        // The table the published panchangas print, which gives Drik
        // Panchang's every window over a week at Delhi to the minute
        // (crux C677).
        amrit_kalam: AmritKalam::Raman,
        // The ascendant a chart of the same context has (crux C680).
        lagna_table: LagnaTable::Chart,
        // Muhurta Chintamani's, the Sanskrit Raman's list departs from
        // (crux C398).
        durmuhurta: DurmuhurtaTable::MuhurtaChintamani,
        // The Dharmasindhu's, the rank-1 nibandha the solar month's
        // day is already placed by (crux C752).
        punya_kala: PunyaKalaTable::Dharmasindhu,
        ritu: RituReckoning::Solar,
        solar_month_start: SolarMonthStart::Punyakala,
        eclipse_shadow: EclipseShadow::Danjon,
        eclipse_horizon: Sunrise::UpperLimbRefraction.into(),
        eclipse_vedha: EclipseVedha::Dharmasindhu,
    }
}

/// The root's day: its sunrise, its moonrise and the horizon they share.
fn root_day() -> Day {
    Day {
        // The observable rise: the upper limb on the horizon with
        // standard refraction, which Drik Panchang and Swiss
        // Ephemeris default to and astronomical almanacs print, and
        // which the eclipse horizon already reads
        // (`03-design/sunrise-convention.md`).
        sunrise: Sunrise::UpperLimbRefraction.into(),
        // The almanacs' moonrise: the Moon's upper limb on the horizon
        // with 34 arcminutes of refraction, her semidiameter and
        // horizontal parallax applied (USNO, *Rise, Set, and Twilight
        // Definitions*; *Explanatory Supplement* §9.33; Meeus ch. 15's
        // h₀ = 0.7275π − 34′). Its own knob, so a day that opens at
        // another sunrise does not move its Moon (SR-102,
        // `03-design/moonrise-convention.md`).
        moonrise: Sunrise::UpperLimbRefraction.into(),
        day_boundary: DayBoundary::Sunrise,
        polar_day_policy: PolarDayPolicy::Undefined,
        ghati_reckoning: GhatiReckoning::Civil,
        hora_reckoning: HoraReckoning::Proportional,
        // Bennett's formula for a named air (Meeus eq. 16.4), the IAU
        // 2015 radii, the body's own latitude, no dip and the ideal
        // horizon: what every almanac and the SDK have reckoned by.
        horizon: HorizonModel::STANDARD,
    }
}

/// The root: every knob set and cited; never selected directly.
///
/// ```
/// use teistro_core::settings::{DEFAULT_PROFILE, Profile, SettingsPatch, root};
///
/// // Every profile is a patch over the root.
/// let resolved = Profile::shipped(DEFAULT_PROFILE).unwrap().resolve(&SettingsPatch::default()).unwrap();
/// assert_eq!(resolved.settings.schema, root().schema);
/// ```
#[must_use]
pub fn root() -> Settings {
    Settings {
        schema: SCHEMA,
        frame: Frame {
            zodiac: Zodiac::Sidereal,
            ayanamsha: Ayanamsha::Lahiri.into(),
            // The true value, nutation included, which the apparent
            // positions below carry: a nirayana longitude is the apparent
            // one less the true ayanamsha, as the Indian Astronomical
            // Ephemeris and the Rashtriya Panchang tabulate it ("True
            // Ayanamsa = Mean Ayanamsa + Nutation in Longitude"), and as
            // the Swiss Ephemeris computes a sidereal position (the mean
            // equinox less the mean value, the same angle). Less the mean
            // value an apparent longitude keeps the nutation, up to 17.2
            // arcseconds; the Rashtriya Panchang's 109 sankrantis of
            // 2018 to 2027 meet the true basis within 32 seconds and miss
            // the mean one by up to 7.7 minutes
            // (`calendars/regional-solar.md`; SR-100, crux C1750).
            ayanamsha_basis: AyanamshaBasis::True,
            node: Node::Mean,
            centre: Centre::Geocentric,
            positions: Positions::Apparent,
            siddhanta: Siddhanta::Drik,
            nakshatra_scheme: NakshatraScheme::TwentySeven,
            // Vondrák et al. (2011): valid at every catalogued epoch (C1752).
            precession: Precession::Vondrak2011,
            // IAU 2000B (McCarthy and Luzum 2003), the SDK's own series.
            nutation: Nutation::Iau2000B,
            // IAU 2006 (IERS Conventions 2010, §5.4.4).
            frame_bias: FrameBias::Iau2006,
        },
        houses: Houses {
            placement_system: HouseSystem::WholeSign,
            chalit_system: HouseSystem::Sripati,
            module_overrides: [(String::from("kp"), HouseSystem::Placidus)]
                .into_iter()
                .collect(),
            polar_policy: PolarPolicy::Error,
        },
        day: root_day(),
        panchanga: root_panchanga(),
        time: root_time(),
        dasha: Dasha {
            balance: Balance::Spatial,
            year_length: year_lengths(),
            depth: depth_all(3),
            seed_overflow: SeedOverflow::WrapToStart,
            // BPHS ch. 46 vv. 17 to 20: four and three alternately from
            // Ardra over the twenty-eight with Abhijit (crux C5).
            ashtottari_grouping: AshtottariGrouping::FourAndThree,
            // The corpus's readings, measured on every recorded answer
            // (`03-design/dasha-measured.md`); the others are knob values
            // because neither reading has a rank-1 text (crux C48).
            birth_period: BirthPeriod::Compressed,
            after_cycle: AfterCycle::End,
            // The recording engine's Kalachakra, measured on every recorded
            // answer (`03-design/kalachakra-measured.md`); the sources read
            // give the other values (cruxes C54 to C56).
            kalachakra_membership: KalachakraMembership::Listed,
            kalachakra_balance: KalachakraBalance::FirstSign,
            kalachakra_after_ninth: KalachakraAfterNinth::Reverse,
            // BPHS ch. 47 vv. 5 and 6, the "Shant" sign set against the
            // inimical ones (crux C79).
            shanta_sign: ShantaSign::Friendly,
            // BPHS ch. 46 vv. 158 to 166 and 179 to 184 (cruxes C51, C53).
            dual_lord: DualLord::Bphs,
            rashi_start: RashiStart::Stronger,
            releasing_shared_sign: ReleasingSharedSign::Next,
            firdaria_nodes: FirdariaNodes::End,
            decennial_division: DecennialDivision::Proportional,
            // BPHS ch. 51 vv. 1 and 2, the rule for a dasha whose years
            // add to a total, as the Karaka dasha's do (crux C737).
            karaka_division: GrahaDivision::Proportional,
            // The plain sum of the six strengths: v. 208 names no measure,
            // and the total is the one every shadbala chapter ranks by
            // (crux C735).
            strongest_graha: StrongestGraha::ShadbalaTotal,
            ayurdaya_place: AntardashaPlace::Share,
            naisargika_division: NaisargikaDivision::Shares,
            ayurdaya_span: AyurdayaSpan::Strongest,
            ayurdaya_readings: AyurdayaReadings::Bphs,
        },
        jaimini: jaimini(),
        // Phaladeepika ch. 26 read whole: the nodes "like the Sun", and
        // "planets" obstructing (`03-design/gochar.md`, C136 and C137).
        gochar: Gochar {
            node_vedha: NodeVedha::LikeTheSun,
            node_obstruction: NodeObstruction::NotEachOther,
            ashtakavarga_good_from: AshtakavargaGoodFrom::Five,
            // Each transit from the chart's own centre (C778).
            transit_centre: TransitCentre::AsChart,
        },
        // No text bands a score; Tithipatra's thresholds, 75, 50 and 30
        // (SR-124; `03-design/rashifal.md`, "The score band").
        rashifal: Rashifal {
            score_bands: ScoreBands::DEFAULT,
        },
        kp: kp_readers(),
        aspect: Aspect {
            node_aspects: NodeAspects::None,
            drishti_table: String::from("PARASHARA"),
        },
        state: State {
            combustion_orbs: String::from("BPHS"),
            // BPHS ch. 45 vv. 30 to 37: the ghatis of birth, and ch. 3's order
            // of the nine grahas for the two v. 30 does not number.
            sayanadi_ghatis: SayanadiGhatis::Elapsed,
            sayanadi_nodes: SayanadiNodes::NineGrahaOrder,
            // Vidyamadhaviyam ch. 1 v. 18, which defines the bhaga and the
            // navamsha together for every graha, and Jataka Parijata ch. 17
            // vv. 34 and 35 read by their own list of houses (C768).
            pushkara_bhaga: PushkaraBhaga::Vidyamadhaviya,
            marana_saturn: MaranaSaturn::First,
            marana_venus: MaranaVenus::Sixth,
        },
        strength: strength(),
        vargas: Vargas {
            unattested_dn: UnattestedDn::Cyclic,
            // BPHS ch. 6 for every chart: no chart named here.
            schemes: BTreeMap::new(),
        },
        calendars: calendars(),
        festivals: festivals(),
        provider: Provider {
            overrides: OverridePolicy::PreferNative,
            tier: Tier::Standard,
            cache_cells: crate::settings::DEFAULT_CACHE_CELLS,
        },
        output: Output {
            precision: Precision {
                angle_decimals: 9,
                instant_decimals: 8,
                score_decimals: 3,
            },
        },
    }
}

/// The root's time conventions.
fn root_time() -> Time {
    Time {
        dst_gap: DstGap::Error,
        dst_overlap: DstOverlap::Earlier,
        unknown_time: UnknownTime::Refuse,
        delta_t: DeltaT::TableThenModel,
        // UT1 taken as UTC, within the 0.9 s leap seconds keep them:
        // what a civil clock reads and every almanac assumes.
        dut1: Dut1::None,
        // IERS Conventions 2010, eq. 5.32: the SDK's own.
        sidereal_time: SiderealTime::Iers2010,
    }
}

/// The strength measures' defaults: BPHS wherever a rank-1 text decides,
/// and Sripati's Shadbala and Bhava bala as Raman works them.
fn strength() -> Strength {
    Strength {
        bala_scheme: BalaScheme::Parashara,
        // BPHS chs. 67 to 69, a rank-1 text, which the conformance
        // corpus's engine reads otherwise (`03-design/ashtakavarga-measured.md`).
        ekadhipatya: Ekadhipatya::Bphs,
        shodhana: Shodhana::EachGraha,
        // BPHS ch. 66 and Varahamihira, and the corpus's engine; Phaladeepika
        // alone reads the 2nd (C144).
        moon_bindu_from_jupiter: MoonBinduFromJupiter::Twelfth,
        vimshopaka: Vimshopaka::Bphs,
        // Sripati's Shadbala and Bhava bala, as B.V. Raman works them in
        // *Graha and Bhava Balas*: the one reading with a worked example,
        // which the SDK reproduces to 0.2 virupas, and the one printed
        // tables and software give. BPHS ch. 27 as the Sanskrit reads it
        // stands a median 40 virupas from it and is `ShadbalaRules::BPHS`
        // (`03-design/shadbala-measured.md`, cruxes C64 to C75).
        saptavargaja: Saptavargaja::Compound,
        nathonnatha: Nathonnatha::Midnight,
        pre_dawn_night: PreDawnNight::PreviousEvening,
        sun_ayana: SunAyana::Doubled,
        moon_paksha: MoonPaksha::Doubled,
        luminary_cheshta: LuminaryCheshta::None,
        kranti: Kranti::HinduTable,
        kaala_lords: KaalaLords::Ahargana,
        dig: DigKendras::Angles,
        drik: Drik::Quarter,
        naisargika: Naisargika::Exact,
        required_rupas: RequiredRupas::Sripati,
        drekkana: Drekkana::MaleNeuterFemale,
        benefics: Benefics::Conditional,
        // The only mean elements a source gives (crux C70).
        cheshta: Cheshta::Sripati,
        yuddha: Yuddha::Sripati,
        ishta_kashta: IshtaKashta::SquareRoots,
        // The bhavas are the chalit's, from the angles (Raman, Art. 29).
        bhava_division: BhavaBalaDivision::Chalit,
        bhava_dig: BhavaDig::Sripati,
        bhava_drishti: BhavaDrishti::Sphuta,
        bhava_special_rules: BhavaSpecialRules::None,
    }
}

fn nepali_drik() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.centre = Some(Centre::Topocentric);
    patch.houses.chalit_system = Some(HouseSystem::Vehlow);
    patch.day.polar_day_policy = Some(PolarDayPolicy::NearestEvent);
    // Nepal's daily print: the disc's centre with standard refraction,
    // a median 0.4 minutes off over 333 days (C39).
    patch.day.sunrise = Some(Sunrise::CentreRefraction.into());
    patch.calendars.civil_calendar = Some(Calendar::BikramSambat);
    patch.calendars.lunar_month = Some(LunarMonth::Purnimanta);
    patch.panchanga.solar_month_start = Some(SolarMonthStart::BikramSambat);
    patch.panchanga.eclipse_vedha = Some(EclipseVedha::FixedHours);
    patch.calendars.eras = Some(
        [Era::Vikrama, Era::Shaka, Era::Kali, Era::NepalSambat]
            .into_iter()
            .collect(),
    );
    Profile {
        id: ProfileId::new("nepali-drik"),
        // 2: the lunar month leads purnimanta, as Nepal names it.
        // 3: the season follows the civil calendar's solar months.
        // 4: an eclipse's vedha is the committee's fixed hours.
        // 6: the ayanamsha basis became `TRUE`, the root's (SR-100).
        // 7: the text's equations take the true sine (C1950).
        // 8: the Moon rises by her own convention, the root's upper limb
        //    with refraction, no longer the day's sunrise (SR-102).
        version: 8,
        base: None,
        patch,
        sources: vec![
            Citation::new("frame.centre", BASELINE),
            Citation::new(
                "houses.chalit_system",
                Source::new(
                    "baseline-engine",
                    "measured Vehlow while documented Sripati; the deliberate-difference registry",
                ),
            ),
            Citation::new(
                "day.polar_day_policy",
                Source::new(
                    "baseline-engine",
                    "synthesised polar days; docs/05-testing/01-golden-vectors.md",
                ),
            ),
            Citation::new("calendars.civil_calendar", BASELINE),
            Citation::new("calendars.eras", BASELINE),
            Citation::new(
                "day.sunrise",
                Source::new(
                    "Nepal Panchanga Nirnayak Vikas Samiti",
                    "measured: Nepal's daily sunrise is a modern sunrise of the disc's centre with standard refraction, a median 0.4 minutes off over 333 days; docs/03-design/nepal-day-measured.md, C39",
                ),
            ),
            Citation::new(
                "calendars.lunar_month",
                Source::new(
                    "Nepal Panchanga Nirnayak Vikas Samiti",
                    "a month from full moon to full moon: Gai Jatra, the day after Shravana's full moon, is Bhadra Krishna Pratipada; docs/03-design/nepal-month-measured.md",
                ),
            ),
            Citation::new(
                "panchanga.solar_month_start",
                Source::new(
                    "Nepal's daily panchanga",
                    "the season changes on the Bikram Sambat month's first day on all 341 recorded days; docs/03-design/ritu-measured.md",
                ),
            ),
            Citation::new(
                "panchanga.eclipse_vedha",
                Source::new(
                    "Nepal Panchanga Nirnayak Vikas Samiti",
                    "no food from nine hours before a lunar eclipse's first seen moment and twelve before a solar one's: 2025-09-07, 2026-03-03 (the Moon rising eclipsed) and 2022-10-25 (the Sun setting eclipsed), as the press reported the committee; docs/03-design/muhurta.md §4.1.1",
                ),
            ),
            Citation::new(
                "day.moonrise",
                Source::new(
                    "Nepal Panchanga Nirnayak Vikas Samiti",
                    "the root's upper limb with refraction, kept though the day's sunrise is the disc's centre: the committee timed the 2026-03-03 vedha from a moonrise at 18:03, which the upper limb with refraction gives (18:03.7) and the centre with refraction (18:05.0) does not; docs/03-design/moonrise-convention.md §5, C2000",
                ),
            ),
        ],
        mark: Mark::Traditional,
    }
}

/// Nepal's national panchanga as its committee computes it: Nepal's
/// civil practice (`nepali-drik`) over the Surya Siddhanta with the
/// committee's bija, in the text's own zodiac.
///
/// The committee requires its panchanga makers to compute by the Surya
/// Siddhanta, and its printed Sun and Moon are the text's, the Moon's
/// apsis carrying a bija of four revolutions an age
/// (`calendars/bikram-sambat.md`, R2); Nepal's daily panchanga prints
/// its tithi, nakshatra and yoga ends from the same, where the modern
/// sky under Lahiri parts from them by hours
/// (`03-design/nepal-day-measured.md`). The zodiac is the text's own,
/// which the text **defines**; the chart is geocentric, as the text's
/// is. The provider is declared, never chosen by the profile
/// (ADR-0029): open the context over `SURYA_SIDDHANTA`.
///
/// The day begins at a **modern** sunrise beside the text's sky, the
/// disc's centre with standard refraction: the text's own sunrise has no
/// equation of time and stands up to twenty minutes from the print's,
/// and with this one every printed vriddhi and kshaya agrees (C39). The
/// modern sunrise comes from the chain's next modern entry, or the
/// built-in ephemeris. The committee's five star planets and its node
/// are modern positions under Lahiri (C38), and so is its daily lagna
/// table (C680): the profile takes the planets from that modern entry
/// (`frame.siddhanta.planets`) and the table from the sphere
/// (`panchanga.lagna_table`).
fn nepali_committee() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.centre = Some(Centre::Geocentric);
    patch.frame.siddhanta = Some(Siddhanta::Surya {
        bija: SuryaBija::NepalCommittee,
        sunrise: SuryaSunrise::Modern,
        planets: SuryaPlanets::Modern {
            ayanamsha: Ayanamsha::Lahiri.into(),
        },
        trig: SuryaTrig::Exact,
    });
    patch.day.sunrise = Some(Sunrise::CentreRefraction.into());
    patch.frame.ayanamsha = Some(Ayanamsha::Suryasiddhanta.into());
    patch.panchanga.lagna_table = Some(LagnaTable::Sphere {
        ayanamsha: Ayanamsha::Lahiri.into(),
    });
    let committee = |what: &'static str| Source::new("Nepal Panchanga Nirnayak Vikas Samiti", what);
    Profile {
        id: ProfileId::new("nepali-committee"),
        // 6: the ayanamsha basis became `TRUE`, the root's (SR-100).
        // 7: the text's equations take the true sine (C1950).
        // 8: the Moon rises by `nepali-drik`'s convention, the upper limb
        //    with refraction (SR-102).
        version: 8,
        base: Some(ProfileId::new("nepali-drik")),
        patch,
        sources: vec![
            Citation::new(
                "frame.siddhanta",
                committee(
                    "measured: the Rashtriya Panchangam 2082 and 2083 print the text's Moon with its apsis four revolutions an age fewer, and modern star planets in Lahiri's zodiac (a root mean square of 9 to 33 arcminutes over 49 printed mornings, the text's 94 to 459), computed with the true sine rather than the text's table of sines (over 730 printed days the Sun's mean error falls from 2.0 to 0.7 arcseconds and every limb's mean end error to the print's rounding, 0.26 minutes); docs/calendars/bikram-sambat.md R2, docs/03-design/nepal-day-measured.md, docs/03-design/committee-star-planets.md and docs/03-design/committee-trig-measured.md, C1950",
                ),
            ),
            Citation::new(
                "frame.ayanamsha",
                committee(
                    "measured: Nepal's daily nakshatra and yoga ends are the text's in its own zodiac; docs/03-design/nepal-day-measured.md",
                ),
            ),
            Citation::new(
                "frame.centre",
                Source::new(
                    "Surya Siddhanta",
                    "Burgess 1860: the text's places are geocentric",
                ),
            ),
            Citation::new(
                "day.sunrise",
                committee(
                    "measured: Nepal's daily sunrise is a modern sunrise of the disc's centre with standard refraction, within 1.5 minutes on 320 of 333 days and 1.7 on every one; docs/03-design/nepal-day-measured.md, C39",
                ),
            ),
            Citation::new(
                "panchanga.lagna_table",
                committee(
                    "measured: the Rashtriya Panchangam 2082's daily lagna starts at Kathmandu are the sphere's ascendant in Lahiri's zodiac, each printed minute 9 to 101 seconds before the cut; the text's own rising times miss by up to four and a half minutes; docs/03-design/amrit-kalam-and-lagnas.md §2.2, C680",
                ),
            ),
        ],
        mark: Mark::Traditional,
    }
}

/// The texts as read, and nothing else (ADR-0024).
///
/// It patches the root rather than another profile, which is what makes
/// "nothing of one country's practice in it" true rather than stated: on
/// `nepali-drik` it inherited that engine's topocentric centre,
/// Nepal's civil calendar and its synthesised polar days, none of which
/// is in any text. Everything not patched here comes from [`root`],
/// which is cited knob by knob.
fn parashari_classical() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.houses.chalit_system = Some(HouseSystem::Sripati);
    patch.day.ghati_reckoning = Some(GhatiReckoning::Proportional);
    patch.jaimini.chara_karakas = Some(CharaKarakas::Eight);
    patch.state.combustion_orbs = Some(String::from("SURYA_SIDDHANTA"));
    Profile {
        id: ProfileId::new("parashari-classical"),
        // 2: the base moved from `nepali-default`, as `nepali-drik` was then
        // called, to the root, so the
        // centre became geocentric, the civil calendar Gregorian, the
        // eras the three pan-Indic ones and a polar day undefined
        // (ADR-0024).
        // 4: the ayanamsha basis became `TRUE`, the root's (SR-100).
        version: 4,
        base: None,
        patch,
        sources: vec![
            Citation::new(
                "houses.chalit_system",
                Source::new("BPHS", "the Sripati bhava"),
            ),
            Citation::new(
                "day.ghati_reckoning",
                Source::new(
                    "BPHS",
                    "the day divided by its own arcs, as the ishta-kaal is reckoned",
                ),
            ),
            Citation::new(
                "jaimini.chara_karakas",
                Source::new("Jaimini Sutras", "1.1.10-18"),
            ),
            Citation::new(
                "state.combustion_orbs",
                Source::new("Surya Siddhanta", "the text's own orbs where it gives them"),
            ),
        ],
        mark: Mark::Traditional,
    }
}

/// The texts as read, over the **Surya Siddhanta's own astronomy**: the
/// chart the classical panchangas of Nepal and much of India reckon, in
/// every part the text defines (`03-design/classical-chart.md`).
///
/// Based on `parashari-classical`, whose rules are the texts' and whose
/// combustion orbs are already the Surya Siddhanta's, it patches only what
/// the text itself settles: the astronomy, which a context holds to its
/// provider, and the zodiac, the text's own precession (III.9 to 12)
/// rather than the catalogue member of that name. Everything else the
/// text needs the root already has — a geocentric chart, the centre of the
/// Sun on the geometric horizon, the mean node, whole-sign houses and a
/// Sripati chalit, which are built from the text's two angles. The
/// provider is declared, never chosen by the profile (ADR-0029): open the
/// context over `SURYA_SIDDHANTA`.
fn surya_siddhanta() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.siddhanta = Some(Siddhanta::surya(SuryaBija::None));
    patch.frame.ayanamsha = Some(Ayanamsha::Suryasiddhanta.into());
    // The text's sunrise: the centre on the geometric horizon (III.34 to
    // 35), whatever the root's modern one is.
    patch.day.sunrise = Some(Sunrise::CentreNoRefraction.into());
    // And its moonrise: ch. X times the Moon's rising by her ascensional
    // difference, the centre on the same geometric horizon.
    patch.day.moonrise = Some(Sunrise::CentreNoRefraction.into());
    let burgess = |what: &'static str| Source::new("Surya Siddhanta", what);
    Profile {
        id: ProfileId::new("surya-siddhanta"),
        // 3: the ayanamsha basis became `TRUE`, the root's (SR-100).
        // 4: the moonrise its own knob, the text's horizon as before (SR-102).
        version: 4,
        base: Some(ProfileId::new("parashari-classical")),
        patch,
        sources: vec![
            Citation::new(
                "frame.siddhanta",
                burgess("Burgess 1860, the text's own mean motions and equations, without bija"),
            ),
            Citation::new(
                "frame.ayanamsha",
                burgess("Burgess 1860, III.9 to 12: the libration of the equinoxes"),
            ),
            Citation::new(
                "day.sunrise",
                burgess(
                    "Burgess 1860, III.34 and 35: the centre of the Sun on the geometric horizon",
                ),
            ),
            Citation::new(
                "day.moonrise",
                burgess(
                    "Burgess 1860, ch. X: the Moon's rising from her ascensional difference, the centre on the geometric horizon, as the Sun's in III.34 to 35",
                ),
            ),
        ],
        mark: Mark::Traditional,
    }
}

/// Mithila's almanacs as they are computed: the Surya Siddhanta under
/// the Makarandasarini's (1478) bija, the day counted from the text's
/// sunrise carried by the bhujantara (`03-design/makaranda.md`).
///
/// The Makaranda is the Surya Siddhanta in tables: its Sun is the text's,
/// its Moon's equation and tithi corrections are the text's rules
/// tabulated, and its daily-motion tables step by the text's motions with
/// whole revolutions in an age added or taken away (`Bija::MAKARANDA`,
/// C1221). The Vishwavidyalaya Panchang of Darbhanga says it is made from
/// the Makaranda on the Surya Siddhanta; its printed days over eight
/// years from 1980 to 2022, at the place it states (85°30′ E, 26°35′ N),
/// are the acceptance values, held to measured bounds by
/// `crates/siddhanta/tests/mithila.rs`: minutes where the text without
/// the bija is a quarter of an hour off and the modern sky hours (C1225,
/// C1226). Its star planets are the text's four steps, which the
/// Makaranda's three fold: the print's Mars to Saturn on five Thursdays
/// of 2022 are the model's within 1.4′ (C1227). Based on
/// `surya-siddhanta`, which already has the text's zodiac and its sunrise
/// on the geometric horizon. The provider is
/// declared, never chosen by the profile (ADR-0029): open the context
/// over `SURYA_SIDDHANTA`.
fn mithila_makaranda() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.siddhanta = Some(Siddhanta::Surya {
        bija: SuryaBija::Makaranda,
        sunrise: SuryaSunrise::Bhujantara,
        planets: SuryaPlanets::Text,
        trig: SuryaTrig::Table,
    });
    let makaranda = |what: &'static str| Source::new("Makarandasarini", what);
    Profile {
        id: ProfileId::new("mithila-makaranda"),
        // 2: the ayanamsha basis became `TRUE`, the root's (SR-100).
        version: 2,
        base: Some(ProfileId::new("surya-siddhanta")),
        patch,
        sources: vec![Citation::new(
            "frame.siddhanta",
            makaranda(
                "Makaranda, Makarandasarini (1478; Varanasi print 1923): the bija read from its daily-motion tables and footers, the bhujantara sunrise measured against 162 printed days of the Vishwavidyalaya Panchang, Darbhanga, 1980 to 2022, and its star planets against 25 printed places of 2022; docs/03-design/makaranda.md",
            ),
        )],
        mark: Mark::Traditional,
    }
}

/// Tamil Nadu's vakya almanacs as they are computed: the *Vakyakarana*'s
/// Sun and Moon, the Moon with the almanac makers' manyadi, joined
/// between sunrises by proportion (`docs/calendars/vakya.md`).
///
/// The Manonmani Vilasam suddha vakya almanac's printed tithi and
/// nakshatra ends, 127 of them over 1956, 1998 and 2015, are the model's within
/// 10 vinadis (`crates/siddhanta/tests/vakya.rs`), and its Vinayakar
/// Chathurthi of 2023 is the 18 September Tamil Nadu declared, where the
/// modern sky gives the 19th. The day is counted from the Sun's upper
/// limb rising with standard refraction, the Tamil solar calendars'
/// sunrise. Based on `surya-siddhanta`, whose zodiac, star planets and
/// angles stand. The provider is declared, never chosen by the profile
/// (ADR-0029): open the context over `SURYA_SIDDHANTA`.
fn tamil_vakya() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.siddhanta = Some(Siddhanta::Vakya);
    patch.day.sunrise = Some(Sunrise::UpperLimbRefraction.into());
    // The Moon by the same modern horizon, rather than the text's that
    // `surya-siddhanta` gives it: the almanacs' (SR-102).
    patch.day.moonrise = Some(Sunrise::UpperLimbRefraction.into());
    let vakyakarana = |what: &'static str| Source::new("Vakyakarana", what);
    Profile {
        id: ProfileId::new("tamil-vakya"),
        // 2: the ayanamsha basis became `TRUE`, the root's (SR-100).
        // 3: the moonrise its own knob, the upper limb as before (SR-102).
        version: 3,
        base: Some(ProfileId::new("surya-siddhanta")),
        patch,
        sources: vec![
            Citation::new(
                "day.sunrise",
                vakyakarana(
                    "the almanac's nadis are counted from the modern upper-limb sunrise at Chennai, where it is printed; docs/calendars/vakya.md, cruxes C1207 and C1294",
                ),
            ),
            Citation::new(
                "day.moonrise",
                Source::new(
                    "U.S. Naval Observatory, Rise, Set, and Twilight Definitions",
                    "the almanacs' upper limb with refraction, as at the root, in place of the text's centre that surya-siddhanta gives the Moon: no Tamil almanac's moonrise was measured; docs/03-design/moonrise-convention.md §5",
                ),
            ),
            Citation::new(
                "frame.siddhanta",
                vakyakarana(
                    "Vakyakarana, ed. Kuppanna Sastri and Sarma (Madras, 1962): the Sun of I.4-5, the Moon of I.9-14 and the 248 candravakyas of Appendix II, the almanacs' manyadi from the introduction, p. xvi; measured against 127 printed ends of the Manonmani Vilasam suddha vakya panchangam, 1956, 1998 and 2015; docs/calendars/vakya.md",
                ),
            ),
        ],
        mark: Mark::Traditional,
    }
}

/// Maharashtra's Ketaki almanacs as they compute today: a modern sky in
/// Ketkar's Chitra-paksha zodiac, under his conventions
/// (`03-design/ketaki.md` §3).
///
/// V. B. Ketkar's *Ketakī Grahagaṇitam* (1899) and *Jyotirgaṇitam* (1898)
/// set the zodiac's origin opposite Chitra and give its value by a linear
/// rule, the catalogue's `KETAKI` (C2180). The sixteen almanacs the
/// Calendar Reform Committee found on his method in 1955 (Date's,
/// Nirnaysagar, Chitrashala, Vaijayanti and others) computed from modern
/// tables and printed that value within four arcseconds, so the modern
/// layer is the engine's sky less `KETAKI`: no provider of its own, over
/// any modern engine. Every other convention the texts state is already
/// the root's: amanta months, a day from sunrise, Bhaskara's adhika and
/// kshaya rule, the mean node, a geocentric chart, and the mean value
/// subtracted on the true basis, which gives star-referred longitudes as
/// Ketkar's mean tables did (C2181). Only the year's name is patched: the
/// southern cycle the Deccan's almanacs count. The day starts at the
/// root's upper-limb sunrise with refraction until a printed Ketaki
/// sunrise is read (C2189). The 1899 karana itself is a later provider
/// (`ketaki-grahaganita`, phase K4).
fn ketaki() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.ayanamsha = Some(Ayanamsha::Ketaki.into());
    patch.calendars.samvatsara = Some(SamvatsaraCount::Chandramana);
    Profile {
        id: ProfileId::new("ketaki"),
        version: 1,
        base: None,
        patch,
        sources: vec![
            Citation::new(
                "frame.ayanamsha",
                Source::new(
                    "V. B. Ketkar, Ketakī Grahagaṇitam (Poona, 1899); Jyotirgaṇitam, 2nd ed. (Poona, 1937)",
                    "the origin opposite Chitra (KGG madhyama v. 3); 22°08′33″ at the Mesha sankranti of Shaka 1800 plus (n/70)° − (n/50)′ a year (KGG spashta v. 6, its example Shaka 1815 → 22°21′6″; JG1937 p. 7, 22°8′33.04″); the 1955 survey's sixteen Ketkar almanacs print 23°12′06″ to 23°12′10″ for Shaka 1876 (Report of the Calendar Reform Committee, Annexure VI); docs/01-research/ketaki.md §2, C2180",
                ),
            ),
            Citation::new(
                "frame.ayanamsha_basis",
                Source::new(
                    "V. B. Ketkar, Jyotirgaṇitam, 2nd ed. (Poona, 1937)",
                    "p. 346: Spica's longitude on 1 January 1892, 22°20′2.9″ after 180°, is its geometric longitude on the mean equinox (22°20′03.4″), not its apparent one (22°19′46.1″): a mean value, which the root's true basis applies to apparent positions; docs/01-research/ketaki.md §2.2, C2181",
                ),
            ),
            Citation::new(
                "calendars.lunar_month",
                Source::new(
                    "V. B. Ketkar, Jyotirgaṇitam, 2nd ed. (Poona, 1937)",
                    "ch. 1 p. 7: months from new moon to new moon, a month with no sankranti adhika and one with two kshaya (Bhaskara's rule); the root's amanta month and adhika rule",
                ),
            ),
            Citation::new(
                "calendars.eras",
                Source::new(
                    "V. B. Ketkar, Ketakī Grahagaṇitam (Poona, 1899)",
                    "madhyama v. 1: the Shalivahana Shaka, elapsed, from Chaitra shukla 1, which the root's eras already count; whether today's heirs also print the Vikram year is C2186",
                ),
            ),
            Citation::new(
                "calendars.samvatsara",
                Source::new(
                    "the Maharashtra almanacs in Ketkar's tradition",
                    "the sixty years named by the Shaka year in order, Chaitradi, the cycle of the Deccan's luni-solar almanacs; to confirm on a printed Ketaki page, C2186",
                ),
            ),
            Citation::new(
                "day.day_boundary",
                Source::new(
                    "V. B. Ketkar, Jyotirgaṇitam, 2nd ed. (Poona, 1937)",
                    "ch. 1 pp. 7–8: the tithi, nakshatra and yoga current at sunrise are written for the day",
                ),
            ),
        ],
        mark: Mark::Traditional,
    }
}

/// The Tilak panchang's zodiac: Ketaki's conventions with the origin at
/// the yogatara of Revati, ζ Piscium, held at 0° (`03-design/ketaki.md` §3,
/// C2183, C2184).
///
/// This is not Ketaki. Ketkar's first edition of *Jyotirgaṇitam* (1898)
/// printed an 18° panchang type beside the 22° one: 18°10′25″ in Shaka
/// 1800 is ζ Piscium's own tropical longitude then. His note of 1928
/// dropped it (*Jyotirgaṇitam*, 1937, pp. (21)–(22)); the Sangli
/// conference took the Revati yogatara as the origin with Tilak's
/// consent, and the Tilak panchang of Pune and Nagpur follows it. The two
/// origins stand 3°58′ apart, so the two schools' sankrantis fall about
/// four days apart. The star is held as an anchored ayanamsha, which
/// meets the Nagpur Tilak Panchang's 19°13′51″ for 1954 within 24″, until
/// Daftari's *Karaṇakalpalatā*, the method it names, gives a rule
/// (C2184).
fn tilak_revati() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.ayanamsha = Some(AyanamshaChoice::Anchored {
        star: Star::Revati,
        longitude_deg: 0.0,
    });
    Profile {
        id: ProfileId::new("tilak-revati"),
        version: 1,
        base: Some(ProfileId::new("ketaki")),
        patch,
        sources: vec![Citation::new(
            "frame.ayanamsha",
            Source::new(
                "Mahārāṣṭrīya pañcāṅgaikya maṇḍaḷa, pratham adhiveśanācā vṛttānta (Pune, 1926); Report of the Calendar Reform Committee (New Delhi, 1955)",
                "pp. 1–3, 8: the Sangli resolution takes the Revati yogatara as the zodiac's origin, and the Zeta (Tilak) panchang is one of the four parties; Annexure VI: the Nagpur Tilak Panchang prints 19°13′51″ for Shaka 1875–76, ζ Piscium at 0° within 24″; docs/01-research/ketaki.md §2.3, C2183, C2184",
            ),
        )],
        mark: Mark::Traditional,
    }
}

/// Every system's year: 365.25 days, but releasing's and the decennials'
/// 360, the year Valens distributes in (*Anthologies* IV.9 and VI.5,
/// `03-design/hellenistic-time-lords.md`, `hellenistic-decennials.md`), and
/// the ayurdaya and Naisargika dashas' 360, the year their spans are
/// counted in (*Jataka Parijata* ch. 5 v. 34, which turns them into solar
/// years by 360 over 365).
fn year_lengths() -> BTreeMap<DashaSystem, YearLength> {
    DashaSystem::ALL
        .iter()
        .map(|system| {
            let length = match (system, system.attributes().family) {
                (_, DashaFamily::Releasing | DashaFamily::Decennials)
                | (DashaSystem::Aayu | DashaSystem::Ashtakavarga | DashaSystem::Naisargika, _) => {
                    YearLength::Savana360
                }
                _ => YearLength::Julian36525,
            };
            (*system, length)
        })
        .collect()
}

fn kp_default() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.ayanamsha = Some(Ayanamsha::Krishnamurti.into());
    patch.frame.node = Some(Node::True);
    patch.houses.placement_system = Some(HouseSystem::Placidus);
    patch.houses.chalit_system = Some(HouseSystem::Placidus);
    patch.houses.polar_policy = Some(PolarPolicy::FallbackPorphyry);
    patch.dasha.year_length = Some(year_lengths());
    Profile {
        id: ProfileId::new("kp-default"),
        // 3: the ayanamsha basis became `TRUE`, the root's (SR-100).
        version: 3,
        base: None,
        patch,
        sources: vec![
            Citation::new(
                "frame.ayanamsha",
                Source::new("Krishnamurti", "the readers"),
            ),
            Citation::new(
                "houses.placement_system",
                Source::new(
                    "Krishnamurti",
                    "the readers: Placidus cusps as house starts",
                ),
            ),
            Citation::new(
                "houses.chalit_system",
                Source::new(
                    "Krishnamurti",
                    "the readers: the Placidus cusps are the bhavas, so the chalit is the placement",
                ),
            ),
            Citation::new("houses.polar_policy", POLAR_PORPHYRY),
            Citation::new("frame.node", TRUE_NODE),
            Citation::new(
                "dasha.year_length",
                Source::new(
                    "K.S. Krishnamurti, KP Reader IV",
                    "Vimshottari's years of 365.25 days; releasing's, the decennials' and the ayurdaya dashas' 360, as the root counts them",
                ),
            ),
        ],
        mark: Mark::Traditional,
    }
}

fn western_tropical_default() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.zodiac = Some(Zodiac::Tropical);
    patch.frame.node = Some(Node::True);
    patch.houses.placement_system = Some(HouseSystem::Placidus);
    patch.houses.chalit_system = Some(HouseSystem::Placidus);
    patch.houses.polar_policy = Some(PolarPolicy::FallbackPorphyry);
    patch.day.day_boundary = Some(DayBoundary::Midnight);
    // Its day runs from midnight, so a polar day is reckoned the same
    // way: under the base's UNDEFINED no chart could be founded inside a
    // polar day or night at all, though nothing Western reads a sunrise
    // to found one (`progressed-angles-measured.md`, Tromsø).
    patch.day.polar_day_policy = Some(PolarDayPolicy::CivilMidnight);
    patch.calendars.eras = Some(BTreeSet::new());
    Profile {
        id: ProfileId::new("western-tropical-default"),
        // 4: the ayanamsha basis became `TRUE`, the root's (SR-100).
        version: 4,
        base: None,
        patch,
        sources: vec![
            Citation::new(
                "frame.zodiac",
                Source::new("convention", "Western practice"),
            ),
            Citation::new("frame.node", TRUE_NODE),
            Citation::new(
                "houses.placement_system",
                Source::new(
                    "convention",
                    "Western practice: Placidus, the system of the published tables of houses",
                ),
            ),
            Citation::new(
                "houses.chalit_system",
                Source::new(
                    "convention",
                    "Western practice: no chalit apart from the placement",
                ),
            ),
            Citation::new("houses.polar_policy", POLAR_PORPHYRY),
            Citation::new(
                "day.day_boundary",
                Source::new(
                    "convention",
                    "Western practice: the civil day from midnight",
                ),
            ),
            Citation::new(
                "day.polar_day_policy",
                Source::new(
                    "teistro-sdk design",
                    "a day from midnight reckons a polar day so too; nothing Western reads a sunrise to found a chart (docs/03-design/progressed-angles-measured.md, Tromso)",
                ),
            ),
            Citation::new(
                "calendars.eras",
                Source::new("convention", "Western practice: no Indian era numbers"),
            ),
        ],
        mark: Mark::Traditional,
    }
}

/// The recording engine's defaults exactly, so that the corpus's charts
/// reproduce (`05-testing/01-golden-vectors.md`).
pub(super) fn conformance_baseline() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.centre = Some(Centre::Topocentric);
    // The engine has no basis knob and applies the nutated value, the
    // root's own since SR-100; kept, so that its measurement stays cited.
    // Measured: over the corpus's 55 charts the mean basis is up to
    // 18.46 arcseconds from the value the engine recorded and the true
    // basis is within 0.0086, two thousand times closer (entry 16).
    patch.frame.ayanamsha_basis = Some(AyanamshaBasis::True);
    patch.houses.chalit_system = Some(HouseSystem::Vehlow);
    patch.day.polar_day_policy = Some(PolarDayPolicy::NearestEvent);
    patch.day.sunrise = Some(Sunrise::UpperLimbRefraction.into());
    // The engine's moonrise is the upper limb with 34′ of refraction
    // whatever its sunrise mode (a fixed geometric altitude of the
    // centre, +0.125°, from a mean parallax and semidiameter), so a
    // snapshot's sunrise mode leaves it (`03-design/moonrise-convention.md`
    // §5).
    patch.day.moonrise = Some(Sunrise::UpperLimbRefraction.into());
    patch.time.dst_gap = Some(DstGap::Error);
    patch.time.dst_overlap = Some(DstOverlap::Earlier);
    patch.provider.overrides = Some(OverridePolicy::PreferNative);
    patch.calendars.civil_calendar = Some(Calendar::BikramSambat);
    // The engine's daily moonrise and moonset are the first at or after
    // local civil midnight, inside a section every other field of which
    // is bounded by sunrise (entry 18).
    patch.panchanga.moon_events = Some(MoonEvents::CivilDay);
    // The engine's panchaka begins with Dhanishtha rather than at its
    // third quarter (crux C158).
    patch.panchanga.panchaka_start = Some(PanchakaStart::Dhanishtha);
    // The engine's Ashtakavarga reductions and pindas (cruxes C59, C60).
    patch.strength.ekadhipatya = Some(Ekadhipatya::EmptyToZero);
    // The recording engine's Ashtottari: three nakshatras each, the three
    // before Ardra wrapped to the start (crux C5).
    patch.dasha.ashtottari_grouping = Some(AshtottariGrouping::ThreeEach);
    patch.strength.shodhana = Some(Shodhana::Sarva);
    patch.strength.vimshopaka = Some(Vimshopaka::SaptavargajaVirupas);
    // The engine's Shadbala, component by component (cruxes C64 to C71).
    patch.strength.saptavargaja = Some(Saptavargaja::Natural);
    patch.strength.nathonnatha = Some(Nathonnatha::Arc);
    patch.strength.pre_dawn_night = Some(PreDawnNight::SameEvening);
    patch.strength.sun_ayana = Some(SunAyana::NotInKaala);
    patch.strength.moon_paksha = Some(MoonPaksha::Doubled);
    patch.strength.luminary_cheshta = Some(LuminaryCheshta::AyanaAndElongation);
    patch.strength.kranti = Some(Kranti::Ecliptic);
    patch.strength.kaala_lords = Some(KaalaLords::Sankranti);
    patch.strength.dig = Some(DigKendras::LagnaProjection);
    patch.strength.drik = Some(Drik::Full);
    patch.strength.naisargika = Some(Naisargika::Hundredths);
    patch.strength.required_rupas = Some(RequiredRupas::Sripati);
    patch.strength.drekkana = Some(Drekkana::MaleFemaleNeuter);
    patch.strength.benefics = Some(Benefics::Fixed);
    patch.strength.cheshta = Some(Cheshta::RecordingEngine);
    patch.strength.yuddha = Some(Yuddha::None);
    patch.strength.ishta_kashta = Some(IshtaKashta::ShadbalaCheshta);
    // The engine's Bhava bala (cruxes C73 to C75).
    patch.strength.bhava_division = Some(BhavaBalaDivision::Placement);
    patch.strength.bhava_dig = Some(BhavaDig::WholeSign);
    patch.strength.bhava_drishti = Some(BhavaDrishti::QuarterOfDig);
    patch.strength.bhava_special_rules = Some(BhavaSpecialRules::None);
    // The engine's rashi dashas (cruxes C51, C53).
    patch.dasha.dual_lord = Some(DualLord::Kendra);
    patch.dasha.rashi_start = Some(RashiStart::Lagna);
    // The engine ranks eight chara karakas: every profile of the corpus's
    // manifest records `chara_karaka: 8`.
    patch.jaimini.chara_karakas = Some(CharaKarakas::Eight);
    // And names the eighth rank the Pitrikaraka (crux C101).
    patch.jaimini.eight_karaka_order = Some(EightKarakaOrder::PitrikarakaLast);
    // The engine's KP ruling planets: seven, the sub lords too, and the
    // civil weekday's lord (cruxes C150, C151).
    patch.kp.ruling_count = Some(RulingCount::WithSubs);
    patch.kp.day_lord_day = Some(DayLordDay::Civil);
    // A birth from its place, a transit from the Earth's centre (C778).
    patch.gochar.transit_centre = Some(TransitCentre::Geocentric);
    Profile {
        id: ProfileId::new("conformance-baseline"),
        // 2: the ayanamsha basis became `TRUE`, which is what the engine
        // applies (entry 16 of the deliberate-difference registry).
        // 3: the Moon's rise and set became the civil day's (entry 18).
        // 4: the Ashtakavarga became the engine's (cruxes C59, C60).
        // 5: the Vimshopaka became the engine's (crux C63).
        // 6: the Shadbala became the engine's (cruxes C64 to C71).
        // 7: the Shadbala's four further forks became the engine's (C69, C70,
        // C72).
        // 8: the Bhava bala became the engine's (cruxes C73 to C75).
        // 9: the Ishta and Kashta phalas became the engine's (crux C76).
        // 10: the rashi dashas' dual lord and start became the engine's
        // (cruxes C51, C53).
        // 12: the chara karakas became the engine's eight, and the KP ruling
        // planets its seven by the civil weekday (cruxes C150, C151).
        // 13: the least of eight karakas became the engine's Pitrikaraka
        // (crux C101).
        // 14: a transit became the engine's, cast from the Earth's centre
        // (crux C778).
        // 15: the moonrise its own knob, the engine's upper limb with
        // refraction as before (SR-102).
        version: 15,
        base: None,
        patch,
        sources: conformance_sources(),
        mark: Mark::Verified,
    }
}

/// What `conformance-baseline`'s readings rest on: the baseline engine's
/// conventions, and each one measured or recorded against its corpus.
fn conformance_sources() -> Vec<Citation> {
    vec![
        Citation::new(
            "*",
            Source::new(
                "baseline-engine",
                "docs/05-testing/01-golden-vectors.md, the baseline conventions",
            ),
        ),
        Citation::new(
            "frame.ayanamsha_basis",
            Source::new(
                "baseline-engine",
                "measured: the recorded ayanamsha carries the nutation, 18.46\" against 0.0086\"",
            ),
        ),
        Citation::new(
            "strength.shodhana",
            Source::new(
                "baseline-engine",
                "measured: the recorded pindas are the reduced sum's and the raw bindus', 77 of 77",
            ),
        ),
        Citation::new(
            "strength.saptavargaja",
            Source::new(
                "baseline-engine",
                "measured: every recorded Shadbala component reproduces under the engine's reading, 497 of 497 cells each (docs/03-design/shadbala-measured.md)",
            ),
        ),
        Citation::new(
            "strength.vimshopaka",
            Source::new(
                "baseline-engine",
                "measured: the recorded scores are the Saptavargaja virupas over 45 by natural friendship, 93 of 93",
            ),
        ),
        Citation::new(
            "strength.ekadhipatya",
            Source::new(
                "baseline-engine",
                "measured: the recorded reductions zero an empty co-ruled sign, 77 of 77",
            ),
        ),
        Citation::new(
            "panchanga.moon_events",
            Source::new(
                "baseline-engine",
                "measured: 24 of the 108 recorded moon events fall outside the day's own window, and none before local midnight",
            ),
        ),
        Citation::new(
            "jaimini.chara_karakas",
            Source::new(
                "baseline-engine",
                "recorded: the corpus's manifest names eight chara karakas under all thirteen of its profiles",
            ),
        ),
        Citation::new(
            "jaimini.eight_karaka_order",
            Source::new(
                "baseline-engine",
                "recorded: the corpus names the least of eight karakas the Pitrikaraka, in every chart",
            ),
        ),
        Citation::new(
            "gochar.transit_centre",
            Source::new(
                "baseline-engine",
                "measured: its transits are cast geocentric under a topocentric birth (`03-design/ab-baseline.md` §10)",
            ),
        ),
        Citation::new(
            "kp.ruling_count",
            Source::new(
                "baseline-engine",
                "crux C150: the lagna's and the Moon's sub lords too, seven",
            ),
        ),
        Citation::new(
            "kp.day_lord_day",
            Source::new("baseline-engine", "crux C151: the civil weekday's lord"),
        ),
    ]
}

impl Profile {
    /// A shipped profile by id, or by one of its [`PROFILE_ALIASES`].
    ///
    /// ```
    /// use teistro_core::settings::Profile;
    ///
    /// assert_eq!(Profile::shipped("kp-default").map(|p| p.id.to_string()), Some(String::from("kp-default")));
    /// assert_eq!(Profile::shipped("nepali-default").map(|p| p.id.to_string()), Some(String::from("nepali-drik")));
    /// assert!(Profile::shipped("kp").is_none());
    /// ```
    #[must_use]
    pub fn shipped(id: &str) -> Option<Profile> {
        let id = PROFILE_ALIASES
            .iter()
            .find_map(|(alias, named)| (*alias == id).then_some(*named))
            .unwrap_or(id);
        match id {
            "nepali-drik" => Some(nepali_drik()),
            "nepali-committee" => Some(nepali_committee()),
            "parashari-classical" => Some(parashari_classical()),
            "surya-siddhanta" => Some(surya_siddhanta()),
            "mithila-makaranda" => Some(mithila_makaranda()),
            "tamil-vakya" => Some(tamil_vakya()),
            "ketaki" => Some(ketaki()),
            "tilak-revati" => Some(tilak_revati()),
            "kp-default" => Some(kp_default()),
            "western-tropical-default" => Some(western_tropical_default()),
            "conformance-baseline" => Some(conformance_baseline()),
            _ => None,
        }
    }

    /// A shipped profile by id or alias, or the refusal naming `profile`
    /// and listing the shipped ones: what a context's builder and the
    /// agent server's per-call scope each answer an unknown id with.
    ///
    /// ```
    /// use teistro_core::settings::Profile;
    ///
    /// assert_eq!(Profile::named("nepali-default")?.id.as_str(), "nepali-drik");
    /// assert_eq!(Profile::named("kp").unwrap_err().field(), Some("profile"));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` at `profile` for an id the SDK does not ship.
    pub fn named(id: &str) -> Result<Profile, crate::error::Error> {
        Profile::shipped(id).ok_or_else(|| unshipped(id))
    }

    /// Resolves the profile with a request's patch: root, then the base,
    /// then this profile, then the patch; validated once.
    ///
    /// ```
    /// use teistro_core::settings::{Node, Profile, SettingsPatch, SunriseConvention};
    ///
    /// let profile = Profile::shipped("nepali-drik").unwrap();
    /// let mut patch = SettingsPatch::default();
    /// // The patch is applied last, over the profile.
    /// patch.frame.node = Some(Node::Mean);
    /// assert_eq!(profile.resolve(&patch).unwrap().settings.frame.node, Node::Mean);
    /// // Every coherence error is returned, not the first alone.
    /// patch.day.sunrise = Some(SunriseConvention::Custom { altitude_deg: 95.0 });
    /// assert!(profile.resolve(&patch).unwrap_err().errors().count() >= 1);
    /// ```
    ///
    /// # Errors
    ///
    /// The coherence errors, every one.
    pub fn resolve(&self, patch: &SettingsPatch) -> Result<Resolved, Diagnostics> {
        let (settings, mut origins) = self.settings_noted();
        settings
            .laid(patch, &mut origins, Layer::Context, None)
            .validated_with(self.id.clone(), origins)
    }

    /// The profile's own settings, unvalidated: root, then the base, then
    /// this profile. What [`Profile::resolve`] lays a request's patch over.
    ///
    /// ```
    /// use teistro_core::settings::{Profile, SettingsPatch};
    ///
    /// let surya = Profile::shipped("surya-siddhanta").unwrap();
    /// // The base's knobs and the profile's own, with nothing patched.
    /// assert_eq!(surya.settings(), surya.resolve(&SettingsPatch::default()).unwrap().settings);
    /// ```
    #[must_use]
    pub fn settings(&self) -> Settings {
        self.settings_noted().0
    }

    /// [`Profile::settings`] with where each knob came from: the root's,
    /// the base's or this profile's, each with the citation that layer
    /// gives for it.
    fn settings_noted(&self) -> (Settings, Origins) {
        let mut origins = Origins::under(Layer::Root);
        let mut settings = root();
        if let Some(base) = self
            .base
            .as_ref()
            .and_then(|b| Profile::shipped(b.as_str()))
        {
            settings = settings.laid(&base.patch, &mut origins, Layer::Profile, Some(&base));
        }
        let settings = settings.laid(&self.patch, &mut origins, Layer::Profile, Some(self));
        (settings, origins)
    }

    /// The source this profile cites for `group.knob`, if it cites one.
    ///
    /// ```
    /// use teistro_core::settings::Profile;
    ///
    /// let committee = Profile::shipped("nepali-committee").unwrap();
    /// assert!(committee.cited("frame", "siddhanta").is_some());
    /// assert!(committee.cited("frame", "node").is_none());
    /// ```
    #[must_use]
    pub fn cited(&self, group: &str, knob: &str) -> Option<Source> {
        cited_in(&self.sources, group, knob)
    }
}

/// The source `sources` give for `group.knob`.
/// A citation for `*` covers every knob the profile cites no other
/// source for (`conformance-baseline`'s, the recording engine's
/// conventions).
fn cited_in(sources: &[Citation], group: &str, knob: &str) -> Option<Source> {
    let named = |cited: &&Citation| {
        cited
            .knob
            .split_once('.')
            .is_some_and(|(g, k)| g == group && k == knob)
    };
    sources
        .iter()
        .find(named)
        .or_else(|| sources.iter().find(|cited| cited.knob == "*"))
        .map(|cited| cited.source)
}

/// The root's citation for `group.knob` ([`ROOT_SOURCES`]).
pub(crate) fn root_source(group: &str, knob: &str) -> Option<Source> {
    cited_in(ROOT_SOURCES, group, knob)
}

/// Placidus and the quadrant systems have no cusps inside the polar
/// circle; Porphyry divides the arcs there instead.
const POLAR_PORPHYRY: Source = Source::new(
    "baseline-engine",
    "an undefined polar arc divided as Porphyry does, as its Tromso Placidus variant shows to 4e-8 degrees; docs/03-design/compat-shim.md",
);

/// The node a profile on modern tables reads: the readers and the Western
/// texts it follows name none (crux C1990, open).
const TRUE_NODE: Source = Source::new(
    "teistro-sdk design",
    "the osculating node of the modern ephemerides these profiles are read against; their sources name no node, so the choice is open (crux C1990) and MEAN is the knob's other value",
);

/// The refusal of a profile the SDK does not ship, out of line so
/// [`Profile::named`] stays one branch.
#[cold]
#[inline(never)]
fn unshipped(id: &str) -> crate::error::Error {
    crate::error::Error::unsupported(format!("no shipped profile `{id}`"))
        .with_field("profile")
        .with_hint(format!(
            "the shipped profiles are {}",
            SHIPPED_PROFILES.join(", ")
        ))
}
