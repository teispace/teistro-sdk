//! Settings and profiles (`docs/03-design/settings-and-profiles.md`): one
//! complete value every module reads, built by applying an explicit
//! patch to a named, versioned profile; a canonical serialisation and a
//! hash that go into every result; coherence validated once on the
//! resolved value, every finding returned.
//!
//! ```
//! use teistro_core::settings::{Profile, SettingsPatch, knobs::Node};
//! use teistro_core::catalogue::Ayanamsha;
//!
//! let profile = Profile::shipped("nepali-default").expect("shipped");
//! let mut patch = SettingsPatch::default();
//! patch.frame.node = Some(Node::True);
//! patch.frame.ayanamsha = Some(Ayanamsha::Raman.into());
//! let resolved = profile.resolve(&patch).expect("coherent");
//! assert_eq!(resolved.settings.frame.node, Node::True);
//! assert_eq!(resolved.settings.hash().to_string().len(), 64);
//! assert!(resolved.warnings.is_empty());
//! ```

pub mod knobs;
mod profiles;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::catalogue::{
    Ayanamsha, BalaScheme, Calendar, DashaSystem, Era, HouseSystem, Mark, Source,
};
use crate::envelope::{CALCULATION_VERSION, Hash, Provenance, Version};
use crate::error::{Error, Status};
use crate::quantity::Depth;
pub use knobs::{
    AfterCycle, AshtottariGrouping, AyanamshaBasis, Balance, Benefics, BhavaDig, BhavaDrishti,
    BhavaSpecialRules, BirthPeriod, BrahmaRule, Centre, CharaKarakas, Cheshta, DayBoundary, DeltaT,
    DigKendras, Drekkana, Drik, DstGap, DstOverlap, DualLord, Ekadhipatya, GhatiReckoning,
    GrahaArudhaException, HoraReckoning, IshtaKashta, KaalaLords, KalachakraAfterNinth,
    KalachakraBalance, KalachakraMembership, Kranti, LuminaryCheshta, LunarMonth, MoonEvents,
    Naisargika, NakshatraScheme, Nathonnatha, Node, NodeAspects, NodeCoLordship, OverridePolicy,
    PolarDayPolicy, PolarPolicy, Positions, PreDawnNight, RashiStart, RequiredRupas, Saptavargaja,
    SayanadiGhatis, SayanadiNodes, SeedOverflow, ShantaSign, Shodhana, SunAyana, Sunrise, Tier,
    UnattestedDn, UnknownTime, Vimshopaka, YearLength, Yuddha, Zodiac,
};
pub use profiles::{DEFAULT_PROFILE, Profile, ProfileId, SHIPPED_PROFILES, root};

/// The settings document's schema version; a later knob appends and an
/// old document still hashes the same under the old schema.
pub const SCHEMA: u16 = 1;

/// Which ayanamsha: a catalogued one, or a custom definition.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AyanamshaChoice {
    /// A catalogued ayanamsha.
    Catalogued {
        /// The member.
        id: Ayanamsha,
    },
    /// A custom ayanamsha: a value at an epoch and a yearly rate.
    Custom {
        /// The epoch, a Julian day in TT.
        epoch_jd_tt: f64,
        /// The value at the epoch, degrees.
        value_deg: f64,
        /// The rate, degrees per Julian year.
        rate_deg_per_year: f64,
    },
}

impl From<Ayanamsha> for AyanamshaChoice {
    fn from(id: Ayanamsha) -> AyanamshaChoice {
        AyanamshaChoice::Catalogued { id }
    }
}

/// The sunrise convention, with a custom altitude when asked.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SunriseConvention {
    /// One of the named conventions.
    Named {
        /// The convention.
        which: Sunrise,
    },
    /// The centre of the disc at a given altitude.
    Custom {
        /// The altitude of the centre, degrees; negative below the horizon.
        altitude_deg: f64,
    },
    /// A named convention that refracts, refracted by the given air rather
    /// than by the almanac's 34 arcminutes (`03-design/horizon-atmosphere.md`).
    ///
    /// ```
    /// use teistro_core::settings::{Atmosphere, Sunrise, SunriseConvention};
    ///
    /// // The engines' own convention: the standard air at the place.
    /// let engines = SunriseConvention::Atmospheric {
    ///     which: Sunrise::UpperLimbRefraction,
    ///     air: Atmosphere::STANDARD,
    /// };
    /// // A weather report's.
    /// let observed = SunriseConvention::Atmospheric {
    ///     which: Sunrise::UpperLimbRefraction,
    ///     air: Atmosphere::given(987.0, 4.5),
    /// };
    /// # let _ = (engines, observed);
    /// ```
    Atmospheric {
        /// The convention, which must refract.
        which: Sunrise,
        /// The air.
        air: Atmosphere,
    },
}

impl From<Sunrise> for SunriseConvention {
    fn from(which: Sunrise) -> SunriseConvention {
        SunriseConvention::Named { which }
    }
}

impl Sunrise {
    /// Whether the convention refracts, and so has an air to give it.
    #[must_use]
    pub const fn refracts(self) -> bool {
        matches!(
            self,
            Sunrise::UpperLimbRefraction | Sunrise::LowerLimbRefraction
        )
    }
}

/// The air a refracted horizon is seen through, as a convention names it:
/// each part given, or left to the engines' standard at the place
/// (`03-design/horizon-atmosphere.md` §5).
///
/// Left out, the pressure is the ICAO standard atmosphere's at the
/// observer's height and the temperature is 15 °C, which is what the
/// engines assume; so [`Atmosphere::STANDARD`] names their convention, and
/// a weather report gives both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Atmosphere {
    /// The pressure at the observer, hectopascals (millibars); absent for
    /// the standard atmosphere's at the observer's height.
    pub pressure_hpa: Option<f64>,
    /// The temperature at the observer, degrees Celsius; absent for 15 °C.
    pub temperature_c: Option<f64>,
}

/// The pressures an air may name, hectopascals: the highest recorded at
/// sea level is 1084, and 100 is far above any observer.
pub const AIR_PRESSURE_HPA: core::ops::RangeInclusive<f64> = 100.0..=1100.0;

/// The temperatures an air may name, degrees Celsius: the recorded
/// extremes at the surface are −89 and 57.
pub const AIR_TEMPERATURE_C: core::ops::RangeInclusive<f64> = -90.0..=60.0;

/// The standard atmosphere's temperature at sea level, and the engines'
/// at every height, degrees Celsius.
pub const STANDARD_TEMPERATURE_C: f64 = 15.0;

impl Atmosphere {
    /// The engines' standard air at the place: both parts left out.
    pub const STANDARD: Atmosphere = Atmosphere {
        pressure_hpa: None,
        temperature_c: None,
    };

    /// An air with both parts given, as a weather report gives them.
    #[must_use]
    pub const fn given(pressure_hpa: f64, temperature_c: f64) -> Atmosphere {
        Atmosphere {
            pressure_hpa: Some(pressure_hpa),
            temperature_c: Some(temperature_c),
        }
    }

    /// The air at a height above sea level: what is given, and the
    /// standard for what is not.
    ///
    /// The pressure is the ICAO standard atmosphere's,
    /// `1013.25 · (1 − 0.0065 h / 288.15)^5.25588` hPa, whose exponent is
    /// `g₀M / (R L)`.
    ///
    /// ```
    /// use teistro_core::quantity::Altitude;
    /// use teistro_core::settings::Atmosphere;
    ///
    /// let sea = Atmosphere::STANDARD.at(Altitude::try_new(0.0).unwrap());
    /// assert_eq!((sea.pressure_hpa, sea.temperature_c), (1013.25, 15.0));
    /// let kathmandu = Atmosphere::STANDARD.at(Altitude::try_new(1400.0).unwrap());
    /// assert!((kathmandu.pressure_hpa - 856.0).abs() < 0.5);
    /// ```
    #[must_use]
    pub fn at(self, height: crate::quantity::Altitude) -> Air {
        const SEA_LEVEL_HPA: f64 = 1013.25;
        const SEA_LEVEL_K: f64 = 288.15;
        const LAPSE_K_PER_M: f64 = 0.0065;
        const EXPONENT: f64 = 5.25588;
        Air {
            pressure_hpa: self.pressure_hpa.unwrap_or_else(|| {
                SEA_LEVEL_HPA
                    * crate::math::powf(1.0 - LAPSE_K_PER_M * height.get() / SEA_LEVEL_K, EXPONENT)
            }),
            temperature_c: self.temperature_c.unwrap_or(STANDARD_TEMPERATURE_C),
        }
    }

    /// Whether each part given lies in its range (`AIR_PRESSURE_HPA`,
    /// `AIR_TEMPERATURE_C`).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.pressure_hpa
            .is_none_or(|p| AIR_PRESSURE_HPA.contains(&p))
            && self
                .temperature_c
                .is_none_or(|t| AIR_TEMPERATURE_C.contains(&t))
    }
}

/// An air resolved at a place: the pressure and the temperature a
/// refraction is computed from.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Air {
    /// Hectopascals.
    pub pressure_hpa: f64,
    /// Degrees Celsius.
    pub temperature_c: f64,
}

/// Which Surya Siddhanta model, when the siddhanta knob is classical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Siddhanta {
    /// Modern astronomy.
    Drik,
    /// Surya Siddhanta, with or without the bija corrections.
    Surya {
        /// Whether the bija corrections apply.
        bija: bool,
    },
}

/// The rounding contract of serialised output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Precision {
    /// Decimals of a degree value beside its exact integer.
    pub angle_decimals: u8,
    /// Decimals of a Julian day.
    pub instant_decimals: u8,
    /// Decimals of a score.
    pub score_decimals: u8,
}

/// How many ephemeris cells a context remembers by default.
///
/// A fifty-day almanac range asks for 53 935 cells of which 21 446 are
/// distinct (`03-design/batch-and-parallelism-measured.md`), so this
/// holds such a range whole with room over; a cell and its key are about
/// a hundred bytes, which puts it at roughly six megabytes. The port's
/// own default is this number — one value, in the place a consumer sets
/// it.
pub const DEFAULT_CACHE_CELLS: u32 = 65_536;

macro_rules! group {
    ($(#[$m:meta])* $name:ident, $patch:ident { $( $(#[$fm:meta])* $field:ident : $ty:ty ),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $( $(#[$fm])* pub $field: $ty ),+
        }

        #[doc = concat!("The patch of `", stringify!($name), "`: every knob optional.")]
        #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[serde(default, deny_unknown_fields)]
        pub struct $patch {
            $( $(#[$fm])* pub $field: Option<$ty> ),+
        }

        impl $name {
            /// Applies every knob the patch sets.
            pub fn apply(&mut self, patch: &$patch) {
                $( if let Some(value) = &patch.$field { self.$field = value.clone(); } )+
            }

            /// The knob names, in document order.
            pub const KNOBS: &'static [&'static str] = &[ $( stringify!($field) ),+ ];
        }

        impl $patch {
            /// Whether the patch sets nothing.
            #[must_use]
            pub fn is_empty(&self) -> bool {
                true $( && self.$field.is_none() )+
            }
        }
    };
}

group!(
    /// The frame positions are expressed in.
    Frame, FramePatch {
        /// Tropical or sidereal.
        zodiac: Zodiac,
        /// Which ayanamsha, when sidereal.
        ayanamsha: AyanamshaChoice,
        /// Mean or with nutation.
        ayanamsha_basis: AyanamshaBasis,
        /// Which node.
        node: Node,
        /// Geocentric or topocentric.
        centre: Centre,
        /// Apparent or true.
        positions: Positions,
        /// Modern or classical astronomy: which the chart asks for. The
        /// context holds it to the provider it opens, refusing a modern
        /// engine under the Surya Siddhanta, since only the text defines
        /// the text's chart (`03-design/classical-chart.md`).
        siddhanta: Siddhanta,
        /// Twenty-seven or twenty-eight nakshatras.
        /// lint: knob-has-a-reader — `panchanga` and `core::angle` both divide
        /// by twenty-seven today; the twenty-eighth (Abhijit) is unequal and
        /// wants its own measurement before either reads this.
        nakshatra_scheme: NakshatraScheme,
    }
);

group!(
    /// House systems and their policy.
    Houses, HousesPatch {
        /// The system placements use.
        placement_system: HouseSystem,
        /// The Bhava-Chalit system.
        chalit_system: HouseSystem,
        /// Per-module overrides (`kp` to Placidus).
        module_overrides: BTreeMap<String, HouseSystem>,
        /// What to do where the system has no solution.
        polar_policy: PolarPolicy,
    }
);

group!(
    /// The local day.
    Day, DayPatch {
        /// The sunrise convention.
        sunrise: SunriseConvention,
        /// When the day begins.
        day_boundary: DayBoundary,
        /// A day without a sunrise.
        polar_day_policy: PolarDayPolicy,
        /// How ghatis are counted.
        ghati_reckoning: GhatiReckoning,
        /// How the planetary hours are counted.
        hora_reckoning: HoraReckoning,
    }
);

group!(
    /// The daily panchanga.
    ///
    /// Three knobs, each one a difference the conformance corpus
    /// measured (`03-design/panchanga-day.md` §2). The window the day's
    /// limbs run over is `day.day_boundary`, and the arcs its periods
    /// divide are the `day.sunrise` ones, so neither is repeated here.
    Panchanga, PanchangaPatch {
        /// Which centre the day's limbs are computed from. An almanac is
        /// geocentric wherever the chart is: the lunar parallax reaches a
        /// degree, which is two hours of tithi boundary.
        centre: Centre,
        /// Which window the Moon's rise and set are found in.
        moon_events: MoonEvents,
        /// The muhurta yoga tables' key, as `aspect.drishti_table` is.
        muhurta_tables: String,
    }
);

group!(
    /// Time resolution.
    Time, TimePatch {
        /// A civil time in a DST gap.
        dst_gap: DstGap,
        /// A civil time in a DST overlap.
        dst_overlap: DstOverlap,
        /// A birth without a time.
        unknown_time: UnknownTime,
        /// Which Delta T.
        delta_t: DeltaT,
    }
);

group!(
    /// Dasha computation.
    Dasha, DashaPatch {
        /// How the first period is balanced.
        balance: Balance,
        /// The year length per system.
        year_length: BTreeMap<DashaSystem, YearLength>,
        /// The default depth per system.
        depth: BTreeMap<DashaSystem, Depth>,
        /// A seed outside a conditional cycle.
        /// lint: knob-has-a-reader — `teistro_dasha::Rules::of`, which is handed the `dasha` group and reads it as `settings.seed_overflow`.
        seed_overflow: SeedOverflow,
        /// How Ashtottari's lords share the nakshatras (crux C5).
        /// lint: knob-has-a-reader — `teistro_dasha::Rules::of`, which is handed the `dasha` group and reads it as `settings.ashtottari_grouping`.
        ashtottari_grouping: AshtottariGrouping,
        /// How the birth period is divided among its sub-periods (crux C48).
        /// lint: knob-has-a-reader — `teistro_dasha::Rules::of`, which is handed the `dasha` group and reads it as `settings.birth_period`.
        birth_period: BirthPeriod,
        /// What a dasha answers past the end of its cycle (crux C48).
        after_cycle: AfterCycle,
        /// Which pada table a nakshatra takes in the Kalachakra dasha (crux C54).
        kalachakra_membership: KalachakraMembership,
        /// How the Kalachakra balance at birth is taken (crux C55).
        kalachakra_balance: KalachakraBalance,
        /// What follows the ninth Kalachakra mahadasha (crux C56).
        kalachakra_after_ninth: KalachakraAfterNinth,
        /// Which friendly signs make a dasha favourable (crux C79).
        shanta_sign: ShantaSign,
        /// How a rashi dasha finds the stronger lord of a dual-lorded sign
        /// (crux C51).
        dual_lord: DualLord,
        /// Where the rashi dashas that start from a stronger sign begin (crux
        /// C53).
        rashi_start: RashiStart,
    }
);

group!(
    /// Jaimini conventions.
    Jaimini, JaiminiPatch {
        /// Seven or eight chara karakas: which ranks the Atmakaraka the
        /// karakamsha is read from.
        chara_karakas: CharaKarakas,
        /// The nodes' co-lordship of Scorpio and Aquarius: which lords of
        /// the 6th, 8th and 12th the Brahma graha is chosen among (C126).
        node_co_lordship: NodeCoLordship,
        /// Which rule finds the Brahma graha (crux C128).
        brahma: BrahmaRule,
        /// Whether a graha's arudha moves as a bhava's does (crux C132).
        graha_arudha_exception: GrahaArudhaException,
    }
);

group!(
    /// Aspects.
    Aspect, AspectPatch {
        /// The nodes' aspects.
        node_aspects: NodeAspects,
        /// The sphuta drishti table's key.
        drishti_table: String,
    }
);

group!(
    /// Planetary state.
    State, StatePatch {
        /// The combustion orb table's key.
        combustion_orbs: String,
        /// Which count of the ghatis of birth the Sayanadi avasthas add (crux C78).
        sayanadi_ghatis: SayanadiGhatis,
        /// The numbers Rahu and Ketu multiply by in the Sayanadi avasthas (crux C78).
        sayanadi_nodes: SayanadiNodes,
    }
);

group!(
    /// Strength.
    Strength, StrengthPatch {
        /// Which components the Shadbala counts: the six of BPHS ch. 27.
        bala_scheme: BalaScheme,
        /// How the Ashtakavarga's Ekadhipatya reduction treats a co-ruled sign
        /// beside an occupied one (crux C60).
        ekadhipatya: Ekadhipatya,
        /// Where the Ashtakavarga's reductions and pindas are made (crux C59).
        shodhana: Shodhana,
        /// How the Vimshopaka scores a graha in a varga (crux C63).
        vimshopaka: Vimshopaka,
        /// How the Shadbala's Saptavargaja scores a varga (crux C64).
        saptavargaja: Saptavargaja,
        /// How the Shadbala's Nathonnatha measures the hour (crux C65).
        nathonnatha: Nathonnatha,
        /// Which night a birth before sunrise is measured in (crux C65).
        pre_dawn_night: PreDawnNight,
        /// Where the Sun's Ayana bala is counted (crux C66).
        sun_ayana: SunAyana,
        /// The Sun's and the Moon's Cheshta balas (crux C66).
        luminary_cheshta: LuminaryCheshta,
        /// The declination the Ayana bala reads (crux C66).
        kranti: Kranti,
        /// Whose weekdays the Abda and Masa lords are (crux C67).
        kaala_lords: KaalaLords,
        /// The kendras the Dig bala measures from (crux C68).
        dig: DigKendras,
        /// How the Drik bala weighs the drishtis received (crux C69).
        drik: Drik,
        /// The natural strengths (crux C71).
        naisargika: Naisargika,
        /// The rupas a graha's Shadbala must reach (crux C71).
        required_rupas: RequiredRupas,
        /// Which decanate gives each gender its Drekkana bala (crux C72).
        drekkana: Drekkana,
        /// Which grahas are benefics for Paksha and Drik (crux C69).
        benefics: Benefics,
        /// The mean elements the Cheshta bala reads (crux C70).
        cheshta: Cheshta,
        /// Whether grahas at war gain and lose the Yuddha bala (crux C70).
        yuddha: Yuddha,
        /// How the Ishta and Kashta phalas are read (crux C76).
        ishta_kashta: IshtaKashta,
        /// How the Bhava bala's Dig bala reads a bhava's sign class (crux C73).
        bhava_dig: BhavaDig,
        /// How the Bhava bala weighs the drishtis a bhava receives (crux C74).
        bhava_drishti: BhavaDrishti,
        /// Whether the Bhava bala adds the special rules (crux C75).
        bhava_special_rules: BhavaSpecialRules,
    }
);

group!(
    /// Divisional charts.
    Vargas, VargasPatch {
        /// The convention for an unattested D-N.
        unattested_dn: UnattestedDn,
    }
);

group!(
    /// Calendars.
    Calendars, CalendarsPatch {
        /// The civil calendar of a request's dates.
        ///
        /// Read by `ts_chart_found`, which is what the deferral this knob
        /// carried was waiting for: "a binding builds a chart from a
        /// settings document alone".
        civil_calendar: Calendar,
        /// The lunar month system.
        lunar_month: LunarMonth,
        /// The era numbers a date carries.
        /// lint: knob-has-a-reader — as `civil_calendar` above — `crates/calendar` renders the eras a
        /// caller asks for rather than the ones the settings name.
        eras: BTreeSet<Era>,
    }
);

group!(
    /// The provider.
    Provider, ProviderPatch {
        /// The override policy (ADR-0013).
        overrides: OverridePolicy,
        /// The built-in ephemeris tier.
        /// lint: knob-has-a-reader — Phase 3's built-in ephemeris, which is what has tiers to choose
        /// between; a provider a caller supplies declares its own.
        tier: Tier,
        /// How many ephemeris cells a context remembers, so that a batch
        /// asks for each of them once.
        ///
        /// Nought is off. Any other number is the capacity, and past it
        /// the memo stops admitting and keeps serving what it holds, so
        /// the memory is a number chosen here rather than a function of
        /// how long a batch ran. One knob and not two, so that no pair of
        /// settings can disagree about whether the memo exists.
        ///
        /// It is honoured only over a provider that declares
        /// `deterministic`: a memo answers a repeat with the first
        /// answer, which is right exactly when the provider would have
        /// answered the same again.
        /// [`DEFAULT_CACHE_CELLS`] is what a fifty-day almanac range
        /// needs, measured (`03-design/batch-and-parallelism-measured.md`).
        cache_cells: u32,
    }
);

group!(
    /// Output.
    Output, OutputPatch {
        /// The rounding contract.
        precision: Precision,
    }
);

/// Every knob, complete; built only by resolving a profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// The document's schema version.
    pub schema: u16,
    /// The frame.
    pub frame: Frame,
    /// House systems.
    pub houses: Houses,
    /// The local day.
    pub day: Day,
    /// The daily panchanga.
    pub panchanga: Panchanga,
    /// Time resolution.
    pub time: Time,
    /// Dashas.
    pub dasha: Dasha,
    /// Jaimini.
    pub jaimini: Jaimini,
    /// Aspects.
    pub aspect: Aspect,
    /// State.
    pub state: State,
    /// Strength.
    pub strength: Strength,
    /// Vargas.
    pub vargas: Vargas,
    /// Calendars.
    pub calendars: Calendars,
    /// The provider.
    pub provider: Provider,
    /// Output.
    pub output: Output,
}

impl Settings {
    /// Every knob of every group, as `("group", "knob")`, in document
    /// order.
    ///
    /// One list, so that a rule about the knobs has a single source of
    /// truth rather than a copy — `cargo xtask check-lints` uses it for
    /// the `knob-has-a-reader` rule. The group names are the field names
    /// a settings document uses, and
    /// `every_group_of_the_document_is_in_the_knob_paths` holds this
    /// list to the document itself, so a group added without a line here
    /// fails rather than going unwatched.
    #[must_use]
    pub fn knob_paths() -> Vec<(&'static str, &'static str)> {
        let groups: [(&str, &[&str]); 14] = [
            ("frame", Frame::KNOBS),
            ("houses", Houses::KNOBS),
            ("day", Day::KNOBS),
            ("panchanga", Panchanga::KNOBS),
            ("time", Time::KNOBS),
            ("dasha", Dasha::KNOBS),
            ("jaimini", Jaimini::KNOBS),
            ("aspect", Aspect::KNOBS),
            ("state", State::KNOBS),
            ("strength", Strength::KNOBS),
            ("vargas", Vargas::KNOBS),
            ("calendars", Calendars::KNOBS),
            ("provider", Provider::KNOBS),
            ("output", Output::KNOBS),
        ];
        groups
            .into_iter()
            .flat_map(|(group, knobs)| knobs.iter().map(move |knob| (group, *knob)))
            .collect()
    }
}

/// A patch: every group's knobs optional.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct SettingsPatch {
    /// The frame.
    pub frame: FramePatch,
    /// House systems.
    pub houses: HousesPatch,
    /// The local day.
    pub day: DayPatch,
    /// The daily panchanga.
    pub panchanga: PanchangaPatch,
    /// Time resolution.
    pub time: TimePatch,
    /// Dashas.
    pub dasha: DashaPatch,
    /// Jaimini.
    pub jaimini: JaiminiPatch,
    /// Aspects.
    pub aspect: AspectPatch,
    /// State.
    pub state: StatePatch,
    /// Strength.
    pub strength: StrengthPatch,
    /// Vargas.
    pub vargas: VargasPatch,
    /// Calendars.
    pub calendars: CalendarsPatch,
    /// The provider.
    pub provider: ProviderPatch,
    /// Output.
    pub output: OutputPatch,
}

impl SettingsPatch {
    /// Whether the patch sets nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frame.is_empty()
            && self.houses.is_empty()
            && self.day.is_empty()
            && self.time.is_empty()
            && self.dasha.is_empty()
            && self.jaimini.is_empty()
            && self.aspect.is_empty()
            && self.state.is_empty()
            && self.strength.is_empty()
            && self.vargas.is_empty()
            && self.calendars.is_empty()
            && self.provider.is_empty()
            && self.output.is_empty()
    }
}

impl Settings {
    /// Applies a patch, knob by knob, and returns the result.
    #[must_use]
    pub fn patched(mut self, patch: &SettingsPatch) -> Settings {
        self.frame.apply(&patch.frame);
        self.houses.apply(&patch.houses);
        self.day.apply(&patch.day);
        self.panchanga.apply(&patch.panchanga);
        self.time.apply(&patch.time);
        self.dasha.apply(&patch.dasha);
        self.jaimini.apply(&patch.jaimini);
        self.aspect.apply(&patch.aspect);
        self.state.apply(&patch.state);
        self.strength.apply(&patch.strength);
        self.vargas.apply(&patch.vargas);
        self.calendars.apply(&patch.calendars);
        self.provider.apply(&patch.provider);
        self.output.apply(&patch.output);
        self
    }

    /// The canonical document: keys in code-point order, no whitespace,
    /// every knob present ([`crate::envelope::canonical_json`]).
    #[must_use]
    pub fn canonical_json(&self) -> String {
        crate::envelope::canonical_json(self)
    }

    /// The hash of the canonical document.
    #[must_use]
    pub fn hash(&self) -> Hash {
        Hash::of(self.canonical_json().as_bytes())
    }

    /// A settings document parsed and checked for coherence.
    ///
    /// # Errors
    ///
    /// A document that does not parse, or one with coherence errors.
    pub fn from_json(json: &str) -> Result<Resolved, Diagnostics> {
        // The message names the knob that failed, `houses.chalit_system`,
        // since a diagnostic's fields are the static knob names.
        let settings: Settings =
            crate::strict::deserialize_str(json, "").map_err(|e| Diagnostics {
                items: vec![Diagnostic::error("json", e.message, &[])],
            })?;
        if settings.schema != SCHEMA {
            return Err(Diagnostics {
                items: vec![Diagnostic {
                    severity: Severity::Error,
                    rule: "schema",
                    message: format!("settings schema {} is not {SCHEMA}", settings.schema),
                    fields: vec!["schema"],
                }],
            });
        }
        settings.validated(ProfileId::new("document"))
    }

    /// Runs the coherence rules and splits them into errors and warnings.
    ///
    /// # Errors
    ///
    /// The diagnostics when any rule is an error.
    pub fn validated(self, profile: ProfileId) -> Result<Resolved, Diagnostics> {
        let diagnostics = coherence(&self);
        if diagnostics.iter().any(|d| d.severity == Severity::Error) {
            return Err(Diagnostics { items: diagnostics });
        }
        Ok(Resolved {
            settings: self,
            warnings: diagnostics,
            profile,
        })
    }
}

/// How bad a finding is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    /// The settings cannot be used.
    Error,
    /// Worth a look; recorded in provenance.
    Warning,
}

/// One coherence finding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Diagnostic {
    /// Error or warning.
    pub severity: Severity,
    /// The rule's id.
    pub rule: &'static str,
    /// What is wrong.
    pub message: String,
    /// The knobs involved.
    pub fields: Vec<&'static str>,
}

impl Diagnostic {
    fn error(
        rule: &'static str,
        message: impl Into<String>,
        fields: &[&'static str],
    ) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            rule,
            message: message.into(),
            fields: fields.to_vec(),
        }
    }

    fn warning(
        rule: &'static str,
        message: impl Into<String>,
        fields: &[&'static str],
    ) -> Diagnostic {
        Diagnostic {
            severity: Severity::Warning,
            rule,
            message: message.into(),
            fields: fields.to_vec(),
        }
    }
}

/// Every finding of a validation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Diagnostics {
    /// The findings, in rule order.
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    /// The errors.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter().filter(|d| d.severity == Severity::Error)
    }
}

impl core::fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, d) in self.items.iter().enumerate() {
            if i > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{} [{}]: {}", d.rule, d.fields.join(", "), d.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

impl From<Diagnostics> for Error {
    fn from(diagnostics: Diagnostics) -> Error {
        let mut error = Error::new(Status::InvalidArg, diagnostics.to_string());
        if let Some(first) = diagnostics.errors().next() {
            if let Some(field) = first.fields.first() {
                error = error.with_field(*field);
            }
        }
        error
    }
}

/// A resolved, validated settings value.
#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    /// The settings.
    pub settings: Settings,
    /// The warnings, which every result's provenance carries
    /// ([`Resolved::provenance`]).
    pub warnings: Vec<Diagnostic>,
    /// The profile it came from.
    pub profile: ProfileId,
}

impl Resolved {
    /// The provenance every result computed under these settings starts
    /// from: the versions, the profile, the settings' hash, the input's,
    /// and **the settings' warnings**.
    ///
    /// One constructor, because the warnings were computed at resolution
    /// and reached no result: each producer built its own stamp from the
    /// profile and the hash, and none carried them, so a warning such as
    /// `siddhanta-topocentric` was said to nobody. A warning crosses as
    /// its rule in the envelope's spelling (`SIDDHANTA_TOPOCENTRIC`), with
    /// its message and the knobs it names as slots.
    ///
    /// ```
    /// use teistro_core::envelope::{Hash, Version};
    /// use teistro_core::settings::{Profile, SettingsPatch};
    ///
    /// let mut patch = SettingsPatch::default();
    /// patch.frame.centre = Some(teistro_core::settings::Centre::Topocentric);
    /// patch.frame.siddhanta = Some(teistro_core::settings::Siddhanta::Surya { bija: false });
    /// let resolved = Profile::shipped("parashari-classical").unwrap().resolve(&patch).unwrap();
    /// let stamp = resolved.provenance(Version::new(0, 1, 0), Hash::of(b"input"));
    /// assert_eq!(stamp.profile, "parashari-classical");
    /// assert!(stamp.warnings.iter().any(|w| w.code == "SIDDHANTA_TOPOCENTRIC"));
    /// ```
    #[must_use]
    pub fn provenance(&self, sdk_version: Version, input_hash: Hash) -> Provenance {
        let mut provenance = Provenance::new(
            sdk_version,
            CALCULATION_VERSION,
            crate::catalogue::SCHEMA_VERSION,
            self.profile.as_str(),
            self.settings.hash(),
            input_hash,
        );
        for warning in &self.warnings {
            provenance.warn(
                &warning.rule.to_uppercase().replace('-', "_"),
                None,
                vec![
                    (String::from("message"), warning.message.clone()),
                    (String::from("fields"), warning.fields.join(",")),
                ],
            );
        }
        provenance
    }
}

/// The coherence rules, every finding returned.
/// What the settings refuse in a sunrise convention: a custom altitude
/// that is not one, an air given to a convention that does not refract,
/// and an air outside what the Earth's surface has seen.
fn sunrise_coherence(convention: SunriseConvention, out: &mut Vec<Diagnostic>) {
    if let SunriseConvention::Custom { altitude_deg } = convention {
        if !altitude_deg.is_finite() || altitude_deg.abs() > 90.0 {
            out.push(Diagnostic::error(
                "sunrise-altitude",
                "a custom sunrise altitude is a finite number of degrees within -90 to 90",
                &["day.sunrise"],
            ));
        }
    }
    if let SunriseConvention::Atmospheric { which, air } = convention {
        if !which.refracts() {
            out.push(Diagnostic::error(
                "sunrise-air-refracts",
                format!(
                    "{} does not refract, so it has no air to give; name UPPER_LIMB_REFRACTION or LOWER_LIMB_REFRACTION",
                    which.key()
                ),
                &["day.sunrise"],
            ));
        }
        if !air.is_valid() {
            out.push(Diagnostic::error(
                "sunrise-air",
                format!(
                    "the air's pressure is {} to {} hPa and its temperature {} to {} °C, or left out for the standard at the place",
                    AIR_PRESSURE_HPA.start(),
                    AIR_PRESSURE_HPA.end(),
                    AIR_TEMPERATURE_C.start(),
                    AIR_TEMPERATURE_C.end()
                ),
                &["day.sunrise"],
            ));
        }
    }
}

fn coherence(s: &Settings) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    if let AyanamshaChoice::Custom {
        epoch_jd_tt,
        value_deg,
        rate_deg_per_year,
    } = s.frame.ayanamsha
    {
        if !(epoch_jd_tt.is_finite() && value_deg.is_finite() && rate_deg_per_year.is_finite()) {
            out.push(Diagnostic::error(
                "custom-ayanamsha-finite",
                "a custom ayanamsha needs a finite epoch, value and rate",
                &["frame.ayanamsha"],
            ));
        }
    }
    sunrise_coherence(s.day.sunrise, &mut out);
    if s.frame.nakshatra_scheme == NakshatraScheme::TwentyEight {
        let seeded: Vec<&DashaSystem> = s
            .dasha
            .depth
            .keys()
            .filter(|d| d.attributes().family == crate::catalogue::DashaFamily::Udu)
            .collect();
        if !seeded.is_empty() {
            out.push(Diagnostic::error(
                "nakshatra-scheme-dasha",
                format!(
                    "the twenty-eight nakshatra scheme has no seed map for {}",
                    seeded
                        .iter()
                        .map(|d| d.key())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &["frame.nakshatra_scheme", "dasha.depth"],
            ));
        }
    }
    if s.houses.polar_policy == PolarPolicy::Clamp
        && s.houses.placement_system == HouseSystem::WholeSign
    {
        out.push(Diagnostic::warning(
            "polar-policy-unused",
            "the clamp policy never applies to whole-sign placements",
            &["houses.polar_policy", "houses.placement_system"],
        ));
    }
    if matches!(s.frame.siddhanta, Siddhanta::Surya { .. }) && s.frame.centre == Centre::Topocentric
    {
        out.push(Diagnostic::warning(
            "siddhanta-topocentric",
            "the classical model is geocentric; the topocentric correction is applied on top and stamped",
            &["frame.siddhanta", "frame.centre"],
        ));
    }
    let root = profiles::root();
    for (system, length) in &s.dasha.year_length {
        if root
            .dasha
            .year_length
            .get(system)
            .is_some_and(|classical| classical != length)
        {
            out.push(Diagnostic::warning(
                "year-length-convention",
                format!(
                    "{system} runs at {length}, not its classical year; recorded as a convention"
                ),
                &["dasha.year_length"],
            ));
        }
    }
    if s.frame.zodiac == Zodiac::Tropical && s.frame.ayanamsha != root.frame.ayanamsha {
        out.push(Diagnostic::warning(
            "ayanamsha-ignored",
            "the ayanamsha is ignored under the tropical zodiac",
            &["frame.zodiac", "frame.ayanamsha"],
        ));
    }
    out
}

/// A citation on a profile's defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Citation {
    /// What the citation is for.
    pub knob: &'static str,
    /// The source.
    pub source: Source,
}

impl Citation {
    /// A citation.
    #[must_use]
    pub const fn new(knob: &'static str, source: Source) -> Citation {
        Citation { knob, source }
    }
}

/// A profile's confidence, for the root and the shipped ones.
pub type ProfileMark = Mark;

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    fn shipped(id: &str) -> Profile {
        Profile::shipped(id).unwrap_or_else(|| panic!("{id}"))
    }

    /// The rules a settings document breaks with this sunrise convention.
    fn sunrise_rules(convention: SunriseConvention) -> Vec<&'static str> {
        let mut patch = SettingsPatch::default();
        patch.day.sunrise = Some(convention);
        match shipped(DEFAULT_PROFILE).resolve(&patch) {
            Ok(_) => Vec::new(),
            Err(diagnostics) => diagnostics.errors().map(|d| d.rule).collect(),
        }
    }

    #[test]
    fn an_air_is_named_in_json_with_either_part_left_to_the_standard() {
        let atmospheric = |json: &str| -> SunriseConvention {
            crate::strict::deserialize_str(json, "")
                .unwrap_or_else(|e| panic!("{json}: {}", e.message))
        };
        let standard =
            atmospheric(r#"{"kind":"ATMOSPHERIC","which":"UPPER_LIMB_REFRACTION","air":{}}"#);
        assert_eq!(
            standard,
            SunriseConvention::Atmospheric {
                which: Sunrise::UpperLimbRefraction,
                air: Atmosphere::STANDARD
            }
        );
        let partial = atmospheric(
            r#"{"kind":"ATMOSPHERIC","which":"LOWER_LIMB_REFRACTION","air":{"temperature_c":-4.5}}"#,
        );
        let SunriseConvention::Atmospheric { air, .. } = partial else {
            panic!("{partial:?}");
        };
        assert_eq!((air.pressure_hpa, air.temperature_c), (None, Some(-4.5)));
        // A part the document does not know is refused, not ignored.
        assert!(
            crate::strict::deserialize_str::<SunriseConvention>(
                r#"{"kind":"ATMOSPHERIC","which":"UPPER_LIMB_REFRACTION","air":{"humidity":0.4}}"#,
                ""
            )
            .is_err()
        );
        // And it writes back as it reads.
        let written = serde_json::to_string(&partial).unwrap();
        assert_eq!(atmospheric(&written), partial);
    }

    #[test]
    fn an_air_resolves_at_the_place_as_the_standard_atmosphere_does() {
        let at = |metres: f64| crate::quantity::Altitude::try_new(metres).unwrap();
        let sea = Atmosphere::STANDARD.at(at(0.0));
        assert_eq!((sea.pressure_hpa, sea.temperature_c), (1013.25, 15.0));
        // The ICAO atmosphere's tabulated pressures, 898.76 hPa at 1000 m,
        // 795.01 at 2000 m and 540.48 at 5000 m, are at geopotential
        // heights; an observer's height is geometric, which puts the
        // formula a few hundredths under the table at these heights (and
        // within 0.15 hPa of Teimeris's own to 8848 m, 0.006′ of
        // refraction).
        for (metres, hpa) in [(1000.0, 898.76), (2000.0, 795.01), (5000.0, 540.48)] {
            let air = Atmosphere::STANDARD.at(at(metres));
            assert!((air.pressure_hpa - hpa).abs() < 0.3, "{metres} m: {air:?}");
            assert_eq!(
                air.temperature_c.to_bits(),
                STANDARD_TEMPERATURE_C.to_bits()
            );
        }
        // What is given is kept at any height.
        let given = Atmosphere::given(987.0, 4.5).at(at(3000.0));
        assert_eq!((given.pressure_hpa, given.temperature_c), (987.0, 4.5));
        let half = Atmosphere {
            pressure_hpa: None,
            temperature_c: Some(30.0),
        }
        .at(at(0.0));
        assert_eq!((half.pressure_hpa, half.temperature_c), (1013.25, 30.0));
    }

    #[test]
    fn the_settings_refuse_an_air_that_cannot_be_applied() {
        let air = |which, air| SunriseConvention::Atmospheric { which, air };
        assert!(sunrise_rules(air(Sunrise::UpperLimbRefraction, Atmosphere::STANDARD)).is_empty());
        assert!(
            sunrise_rules(air(
                Sunrise::LowerLimbRefraction,
                Atmosphere::given(1100.0, -90.0)
            ))
            .is_empty()
        );
        assert_eq!(
            sunrise_rules(air(Sunrise::CentreNoRefraction, Atmosphere::STANDARD)),
            ["sunrise-air-refracts"]
        );
        for wrong in [
            Atmosphere::given(99.0, 15.0),
            Atmosphere::given(1101.0, 15.0),
            Atmosphere::given(1013.25, 61.0),
            Atmosphere::given(1013.25, -91.0),
            Atmosphere::given(f64::NAN, 15.0),
        ] {
            assert_eq!(
                sunrise_rules(air(Sunrise::UpperLimbRefraction, wrong)),
                ["sunrise-air"],
                "{wrong:?}"
            );
        }
    }

    #[test]
    fn every_shipped_profile_resolves_and_hashes_stably() {
        let mut hashes = BTreeMap::new();
        for id in SHIPPED_PROFILES {
            let resolved = shipped(id)
                .resolve(&SettingsPatch::default())
                .unwrap_or_else(|e| panic!("{id}: {e}"));
            assert!(
                resolved.warnings.is_empty(),
                "{id}: {:?}",
                resolved.warnings
            );
            let json = resolved.settings.canonical_json();
            assert!(!json.contains(' '), "{id}: canonical form has whitespace");
            assert!(
                json.starts_with("{\"aspect\":{\"drishti_table\""),
                "{id}: canonical form is not key-sorted: {}",
                &json[..40]
            );
            let back: Settings = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(back, resolved.settings);
            assert_eq!(
                back.canonical_json(),
                json,
                "{id}: canonical form is a fixed point"
            );
            hashes.insert(id, resolved.settings.hash().to_string());
        }
        assert_eq!(hashes.len(), SHIPPED_PROFILES.len());
        let distinct: BTreeSet<&String> = hashes.values().collect();
        assert_eq!(distinct.len(), SHIPPED_PROFILES.len(), "{hashes:?}");
    }

    #[test]
    fn patches_apply_in_order_and_change_the_hash() {
        let base = shipped("nepali-default")
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        let mut patch = SettingsPatch::default();
        patch.frame.node = Some(Node::True);
        let patched = shipped("nepali-default")
            .resolve(&patch)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_ne!(base.settings.hash(), patched.settings.hash());
        assert_eq!(patched.settings.frame.node, Node::True);
        assert_eq!(patched.settings.frame.zodiac, base.settings.frame.zodiac);
        let again = patched.settings.clone().patched(&SettingsPatch::default());
        assert_eq!(again, patched.settings);
        assert!(SettingsPatch::default().is_empty());
        assert!(!patch.is_empty());
    }

    #[test]
    fn coherence_rules_fire_with_fields() {
        let mut patch = SettingsPatch::default();
        patch.frame.ayanamsha = Some(AyanamshaChoice::Custom {
            epoch_jd_tt: f64::NAN,
            value_deg: 23.0,
            rate_deg_per_year: 0.0139,
        });
        patch.frame.nakshatra_scheme = Some(NakshatraScheme::TwentyEight);
        patch.houses.polar_policy = Some(PolarPolicy::Clamp);
        let error = shipped("nepali-default").resolve(&patch).unwrap_err();
        let rules: Vec<&str> = error.items.iter().map(|d| d.rule).collect();
        assert!(rules.contains(&"custom-ayanamsha-finite"), "{rules:?}");
        assert!(rules.contains(&"nakshatra-scheme-dasha"), "{rules:?}");
        assert!(rules.contains(&"polar-policy-unused"), "{rules:?}");
        assert_eq!(error.errors().count(), 2);
        let as_error: Error = error.into();
        assert_eq!(as_error.status, Status::InvalidArg);
        assert_eq!(as_error.field(), Some("frame.ayanamsha"));

        let mut warned = SettingsPatch::default();
        warned.dasha.year_length = Some(
            [(DashaSystem::Vimshottari, YearLength::Savana360)]
                .into_iter()
                .collect(),
        );
        warned.frame.siddhanta = Some(Siddhanta::Surya { bija: true });
        let resolved = shipped("nepali-default")
            .resolve(&warned)
            .unwrap_or_else(|e| panic!("{e}"));
        let rules: Vec<&str> = resolved.warnings.iter().map(|d| d.rule).collect();
        assert_eq!(rules, ["siddhanta-topocentric", "year-length-convention"]);
    }

    #[test]
    fn documents_round_trip_and_refuse_other_schemas() {
        let resolved = shipped("kp-default")
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        let json = resolved.settings.canonical_json();
        let back = Settings::from_json(&json).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back.settings, resolved.settings);
        assert_eq!(back.settings.houses.placement_system, HouseSystem::Placidus);
        assert_eq!(
            back.settings.houses.module_overrides.get("kp"),
            Some(&HouseSystem::Placidus)
        );
        let other = json.replacen("\"schema\":1", "\"schema\":2", 1);
        assert!(Settings::from_json(&other).is_err());
        assert!(Settings::from_json("{\"schema\":1}").is_err());
        assert!(
            shipped("nepali-default")
                .sources
                .iter()
                .any(|c| c.knob == "houses.chalit_system")
        );
    }

    #[test]
    fn profiles_inherit_and_are_listed() {
        for id in SHIPPED_PROFILES {
            let profile = shipped(id);
            assert_eq!(profile.id.as_str(), id);
            assert!(profile.version >= 1);
        }
        assert!(Profile::shipped("nowhere").is_none());
        // The default patches the root and nothing else, which is what
        // makes "the texts as read, with nothing of one country's
        // practice in it" true rather than stated (ADR-0024).
        let classical = shipped(DEFAULT_PROFILE);
        assert_eq!(classical.base, None);
        let resolved = classical
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(resolved.profile.as_str(), "parashari-classical");
        // What the texts give it.
        assert_eq!(resolved.settings.houses.chalit_system, HouseSystem::Sripati);
        assert_eq!(resolved.settings.jaimini.chara_karakas, CharaKarakas::Eight);
        assert_eq!(resolved.settings.state.combustion_orbs, "SURYA_SIDDHANTA");
        // What it must not have taken from anywhere else. The centre is
        // the one that moves numbers: up to 39 arcminutes of Moon, which
        // changes a mahadasha lord in one of the six charts the corpus
        // records both ways.
        assert_eq!(resolved.settings.frame.zodiac, Zodiac::Sidereal);
        assert_eq!(resolved.settings.frame.centre, Centre::Geocentric);
        assert_eq!(
            resolved.settings.calendars.civil_calendar,
            Calendar::Gregorian
        );
        assert!(!resolved.settings.calendars.eras.contains(&Era::NepalSambat));
        assert_eq!(
            resolved.settings.day.polar_day_policy,
            PolarDayPolicy::Undefined
        );

        // The product's charts keep every one of them, cited to the
        // engine whose charts they reproduce.
        let nepali = shipped("nepali-default");
        let product = nepali
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(product.settings.frame.centre, Centre::Topocentric);
        assert_eq!(
            product.settings.calendars.civil_calendar,
            Calendar::BikramSambat
        );
        assert!(product.settings.calendars.eras.contains(&Era::NepalSambat));

        // A change of defaults is a change of hash, which is what makes
        // it visible in every result computed under it.
        assert_ne!(resolved.settings.hash(), product.settings.hash());
    }

    #[test]
    fn every_group_of_the_document_is_in_the_knob_paths() {
        // The list `knob_paths` holds is checked against the document
        // itself, so a group added to `Settings` without a line there
        // fails here rather than going unwatched by the lint that reads
        // it.
        let document = serde_json::to_value(root()).expect("a settings document");
        let object = document.as_object().expect("an object");
        let paths = Settings::knob_paths();
        let mut groups: Vec<&str> = paths.iter().map(|(group, _)| *group).collect();
        groups.sort_unstable();
        groups.dedup();
        for key in object.keys() {
            // `schema` is the document's version and not a knob.
            if key == "schema" {
                continue;
            }
            assert!(groups.contains(&key.as_str()), "{key} is not in knob_paths");
        }
        assert_eq!(
            groups.len(),
            object.len() - 1,
            "knob_paths names a group the document does not have"
        );
        // And every knob of every group is a field of that group.
        for (group, knob) in &paths {
            let fields = object
                .get(*group)
                .and_then(serde_json::Value::as_object)
                .unwrap_or_else(|| panic!("{group} is a group"));
            assert!(fields.contains_key(*knob), "{group}.{knob}");
        }
        assert_eq!(
            paths.len(),
            object
                .iter()
                .filter(|(key, _)| *key != "schema")
                .filter_map(|(_, value)| value.as_object())
                .map(serde_json::Map::len)
                .sum::<usize>(),
            "every knob of the document is named once"
        );
    }
}
