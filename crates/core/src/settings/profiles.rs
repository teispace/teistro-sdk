//! The root settings, cited knob by knob, and the shipped profiles as
//! patches over it. Only a resolved `Settings` is ever hashed; a profile
//! id is a name for a patch, never a hash input.

use std::collections::{BTreeMap, BTreeSet};

use crate::catalogue::{
    Ayanamsha, BalaScheme, Calendar, DashaSystem, Era, HouseSystem, Mark, Source,
};

use super::knobs::{
    AfterCycle, AshtottariGrouping, AyanamshaBasis, Balance, Benefics, BhavaDig, BhavaDrishti,
    BhavaSpecialRules, BirthPeriod, Centre, CharaKarakas, Cheshta, DayBoundary, DeltaT, DigKendras,
    Drekkana, Drik, DstGap, DstOverlap, DualLord, Ekadhipatya, GhatiReckoning, HoraReckoning,
    IshtaKashta, KaalaLords, KalachakraAfterNinth, KalachakraBalance, KalachakraMembership, Kranti,
    LuminaryCheshta, LunarMonth, MoonEvents, Naisargika, NakshatraScheme, Nathonnatha, Node,
    NodeAspects, NodeCoLordship, OverridePolicy, PolarDayPolicy, PolarPolicy, Positions,
    PreDawnNight, RashiStart, RequiredRupas, Saptavargaja, SayanadiGhatis, SayanadiNodes,
    SeedOverflow, ShantaSign, Shodhana, SunAyana, Sunrise, Tier, UnattestedDn, UnknownTime,
    Vimshopaka, YearLength, Yuddha, Zodiac,
};
use super::{
    Aspect, Calendars, Citation, Dasha, Day, Diagnostics, Frame, Houses, Jaimini, Output,
    Panchanga, Precision, Provider, Resolved, SCHEMA, Settings, SettingsPatch, Siddhanta, State,
    Strength, Time, Vargas,
};
use crate::quantity::Depth;

/// A profile's id.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct ProfileId(String);

impl ProfileId {
    /// An id.
    #[must_use]
    pub fn new(id: &str) -> ProfileId {
        ProfileId(id.to_string())
    }

    /// The id.
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
pub const DEFAULT_PROFILE: &str = "parashari-classical";

/// The ids of the shipped profiles.
pub const SHIPPED_PROFILES: [&str; 6] = [
    "nepali-default",
    "parashari-classical",
    "surya-siddhanta",
    "kp-default",
    "western-tropical-default",
    "conformance-baseline",
];

const BASELINE: Source = Source::new("baseline-engine", "persisted chart settings");

fn depth_all(depth: u8) -> BTreeMap<DashaSystem, Depth> {
    DashaSystem::ALL
        .iter()
        .map(|d| (*d, Depth::try_new(depth).unwrap_or(Depth::MIN)))
        .collect()
}

/// The root: every knob set and cited; never selected directly.
#[must_use]
pub fn root() -> Settings {
    Settings {
        schema: SCHEMA,
        frame: Frame {
            zodiac: Zodiac::Sidereal,
            ayanamsha: Ayanamsha::Lahiri.into(),
            ayanamsha_basis: AyanamshaBasis::Mean,
            node: Node::Mean,
            centre: Centre::Geocentric,
            positions: Positions::Apparent,
            siddhanta: Siddhanta::Drik,
            nakshatra_scheme: NakshatraScheme::TwentySeven,
        },
        houses: Houses {
            placement_system: HouseSystem::WholeSign,
            chalit_system: HouseSystem::Sripati,
            module_overrides: [(String::from("kp"), HouseSystem::Placidus)]
                .into_iter()
                .collect(),
            polar_policy: PolarPolicy::Error,
        },
        day: Day {
            sunrise: Sunrise::CentreNoRefraction.into(),
            day_boundary: DayBoundary::Sunrise,
            polar_day_policy: PolarDayPolicy::Undefined,
            ghati_reckoning: GhatiReckoning::Civil,
            hora_reckoning: HoraReckoning::Proportional,
        },
        panchanga: Panchanga {
            // An almanac is geocentric wherever the chart is; the daily
            // limbs of every published panchanga are, and the corpus
            // measures the difference at five classifications in 136
            // (`03-design/panchanga-day.md` §10).
            centre: Centre::Geocentric,
            moon_events: MoonEvents::Window,
            muhurta_tables: String::from("CLASSICAL"),
        },
        time: Time {
            dst_gap: DstGap::Error,
            dst_overlap: DstOverlap::Earlier,
            unknown_time: UnknownTime::Refuse,
            delta_t: DeltaT::TableThenModel,
        },
        dasha: Dasha {
            balance: Balance::Spatial,
            year_length: DashaSystem::ALL
                .iter()
                .map(|d| (*d, YearLength::Julian36525))
                .collect(),
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
        },
        jaimini: Jaimini {
            chara_karakas: CharaKarakas::Seven,
            node_co_lordship: NodeCoLordship::None,
        },
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
        },
        strength: strength(),
        vargas: Vargas {
            unattested_dn: UnattestedDn::Cyclic,
        },
        calendars: Calendars {
            civil_calendar: Calendar::Gregorian,
            lunar_month: LunarMonth::Amanta,
            eras: [Era::Vikrama, Era::Shaka, Era::Kali].into_iter().collect(),
        },
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

/// The strength measures' defaults: BPHS wherever a rank-1 text decides.
fn strength() -> Strength {
    Strength {
        bala_scheme: BalaScheme::Parashara,
        // BPHS chs. 67 to 69, a rank-1 text, which the conformance
        // corpus's engine reads otherwise (`03-design/ashtakavarga-measured.md`).
        ekadhipatya: Ekadhipatya::Bphs,
        shodhana: Shodhana::EachGraha,
        vimshopaka: Vimshopaka::Bphs,
        // BPHS ch. 27 wherever it decides (`03-design/shadbala-measured.md`).
        saptavargaja: Saptavargaja::Compound,
        nathonnatha: Nathonnatha::Midnight,
        pre_dawn_night: PreDawnNight::PreviousEvening,
        sun_ayana: SunAyana::Doubled,
        luminary_cheshta: LuminaryCheshta::AyanaAndPaksha,
        kranti: Kranti::True,
        kaala_lords: KaalaLords::Ahargana,
        dig: DigKendras::Angles,
        drik: Drik::QuarterWithJupiterMercury,
        naisargika: Naisargika::Exact,
        required_rupas: RequiredRupas::Bphs,
        drekkana: Drekkana::MaleFemaleNeuter,
        benefics: Benefics::Conditional,
        // The only mean elements a source gives (crux C70).
        cheshta: Cheshta::Sripati,
        yuddha: Yuddha::Sripati,
        // BPHS ch. 28 vv. 2 to 6.
        ishta_kashta: IshtaKashta::Rays,
        // BPHS ch. 27 vv. 26 to 31 as translated (`03-design/bhava-bala-measured.md`).
        bhava_dig: BhavaDig::Bphs,
        bhava_drishti: BhavaDrishti::QuarterOfDig,
        bhava_special_rules: BhavaSpecialRules::Bphs,
    }
}

fn nepali_default() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.centre = Some(Centre::Topocentric);
    patch.houses.chalit_system = Some(HouseSystem::Vehlow);
    patch.day.polar_day_policy = Some(PolarDayPolicy::NearestEvent);
    patch.calendars.civil_calendar = Some(Calendar::BikramSambat);
    patch.calendars.eras = Some(
        [Era::Vikrama, Era::Shaka, Era::Kali, Era::NepalSambat]
            .into_iter()
            .collect(),
    );
    Profile {
        id: ProfileId::new("nepali-default"),
        version: 1,
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
        ],
        mark: Mark::Traditional,
    }
}

/// The texts as read, and nothing else (ADR-0024).
///
/// It patches the root rather than another profile, which is what makes
/// "nothing of one country's practice in it" true rather than stated: on
/// `nepali-default` it inherited that engine's topocentric centre,
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
        // 2: the base moved from `nepali-default` to the root, so the
        // centre became geocentric, the civil calendar Gregorian, the
        // eras the three pan-Indic ones and a polar day undefined
        // (ADR-0024).
        version: 2,
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
    patch.frame.siddhanta = Some(Siddhanta::Surya { bija: false });
    patch.frame.ayanamsha = Some(Ayanamsha::Suryasiddhanta.into());
    let burgess = |what: &'static str| Source::new("Surya Siddhanta", what);
    Profile {
        id: ProfileId::new("surya-siddhanta"),
        version: 1,
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
        ],
        mark: Mark::Traditional,
    }
}

fn kp_default() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.ayanamsha = Some(Ayanamsha::Krishnamurti.into());
    patch.frame.node = Some(Node::True);
    patch.houses.placement_system = Some(HouseSystem::Placidus);
    patch.houses.chalit_system = Some(HouseSystem::Placidus);
    patch.houses.polar_policy = Some(PolarPolicy::FallbackPorphyry);
    patch.dasha.year_length = Some(
        DashaSystem::ALL
            .iter()
            .map(|d| (*d, YearLength::Julian36525))
            .collect(),
    );
    Profile {
        id: ProfileId::new("kp-default"),
        version: 1,
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
    patch.calendars.eras = Some(BTreeSet::new());
    Profile {
        id: ProfileId::new("western-tropical-default"),
        version: 1,
        base: None,
        patch,
        sources: vec![Citation::new(
            "frame.zodiac",
            Source::new("convention", "Western practice"),
        )],
        mark: Mark::Traditional,
    }
}

/// The recording engine's defaults exactly, so that the corpus's charts
/// reproduce (`05-testing/01-golden-vectors.md`).
fn conformance_baseline() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.centre = Some(Centre::Topocentric);
    // The engine has no basis knob and applies the nutated value.
    // Measured: over the corpus's 55 charts the mean basis is up to
    // 18.46 arcseconds from the value the engine recorded and the true
    // basis is within 0.0086, two thousand times closer (entry 16).
    patch.frame.ayanamsha_basis = Some(AyanamshaBasis::True);
    patch.houses.chalit_system = Some(HouseSystem::Vehlow);
    patch.day.polar_day_policy = Some(PolarDayPolicy::NearestEvent);
    patch.day.sunrise = Some(Sunrise::UpperLimbRefraction.into());
    patch.time.dst_gap = Some(DstGap::Error);
    patch.time.dst_overlap = Some(DstOverlap::Earlier);
    patch.provider.overrides = Some(OverridePolicy::PreferNative);
    patch.calendars.civil_calendar = Some(Calendar::BikramSambat);
    // The engine's daily moonrise and moonset are the first at or after
    // local civil midnight, inside a section every other field of which
    // is bounded by sunrise (entry 18).
    patch.panchanga.moon_events = Some(MoonEvents::CivilDay);
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
    patch.strength.bhava_dig = Some(BhavaDig::WholeSign);
    patch.strength.bhava_drishti = Some(BhavaDrishti::QuarterOfDig);
    patch.strength.bhava_special_rules = Some(BhavaSpecialRules::None);
    // The engine's rashi dashas (cruxes C51, C53).
    patch.dasha.dual_lord = Some(DualLord::Kendra);
    patch.dasha.rashi_start = Some(RashiStart::Lagna);
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
        version: 10,
        base: None,
        patch,
        sources: vec![
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
        ],
        mark: Mark::Verified,
    }
}

impl Profile {
    /// A shipped profile by id.
    #[must_use]
    pub fn shipped(id: &str) -> Option<Profile> {
        match id {
            "nepali-default" => Some(nepali_default()),
            "parashari-classical" => Some(parashari_classical()),
            "surya-siddhanta" => Some(surya_siddhanta()),
            "kp-default" => Some(kp_default()),
            "western-tropical-default" => Some(western_tropical_default()),
            "conformance-baseline" => Some(conformance_baseline()),
            _ => None,
        }
    }

    /// Resolves the profile with a request's patch: root, then the base,
    /// then this profile, then the patch; validated once.
    ///
    /// # Errors
    ///
    /// The coherence errors, every one.
    pub fn resolve(&self, patch: &SettingsPatch) -> Result<Resolved, Diagnostics> {
        let mut settings = root();
        if let Some(base) = self
            .base
            .as_ref()
            .and_then(|b| Profile::shipped(b.as_str()))
        {
            settings = settings.patched(&base.patch);
        }
        settings = settings.patched(&self.patch).patched(patch);
        settings.validated(self.id.clone())
    }
}
