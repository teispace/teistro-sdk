//! The root settings, cited knob by knob, and the shipped profiles as
//! patches over it. Only a resolved `Settings` is ever hashed; a profile
//! id is a name for a patch, never a hash input.

use std::collections::{BTreeMap, BTreeSet};

use crate::catalogue::{
    Ayanamsha, BalaScheme, Calendar, DashaSystem, Era, HouseSystem, Mark, Source,
};

use super::knobs::{
    AyanamshaBasis, Balance, Centre, CharaKarakas, DayBoundary, DeltaT, DstGap, DstOverlap,
    Ekadhipatya, GhatiReckoning, HoraReckoning, LunarMonth, NakshatraScheme, Node, NodeAspects,
    NodeCoLordship, OverridePolicy, PolarDayPolicy, PolarPolicy, Positions, SeedOverflow, Sunrise,
    Tier, UnattestedDn, UnknownTime, YearLength, Zodiac,
};
use super::{
    Aspect, Calendars, Citation, Dasha, Day, Diagnostics, Frame, Houses, Jaimini, Output,
    Precision, Provider, Resolved, SCHEMA, Settings, SettingsPatch, Siddhanta, State, Strength,
    Time, Vargas,
};
use crate::quantity::Depth;

/// A profile's id.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
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

/// The ids of the shipped profiles.
pub const SHIPPED_PROFILES: [&str; 5] = [
    "nepali-default",
    "parashari-classical",
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
        },
        strength: Strength {
            bala_scheme: BalaScheme::Parashara,
            ekadhipatya: Ekadhipatya::Classical,
        },
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
                    "synthesised polar days; fixtures/README.md",
                ),
            ),
            Citation::new("calendars.civil_calendar", BASELINE),
        ],
        mark: Mark::Traditional,
    }
}

fn parashari_classical() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.houses.chalit_system = Some(HouseSystem::Sripati);
    patch.day.ghati_reckoning = Some(GhatiReckoning::Proportional);
    patch.jaimini.chara_karakas = Some(CharaKarakas::Eight);
    patch.state.combustion_orbs = Some(String::from("SURYA_SIDDHANTA"));
    Profile {
        id: ProfileId::new("parashari-classical"),
        version: 1,
        base: Some(ProfileId::new("nepali-default")),
        patch,
        sources: vec![
            Citation::new(
                "houses.chalit_system",
                Source::new("BPHS", "the Sripati bhava"),
            ),
            Citation::new(
                "jaimini.chara_karakas",
                Source::new("Jaimini Sutras", "1.1.10-18"),
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

fn conformance_baseline() -> Profile {
    let mut patch = SettingsPatch::default();
    patch.frame.centre = Some(Centre::Topocentric);
    patch.houses.chalit_system = Some(HouseSystem::Vehlow);
    patch.day.polar_day_policy = Some(PolarDayPolicy::NearestEvent);
    patch.day.sunrise = Some(Sunrise::UpperLimbRefraction.into());
    patch.time.dst_gap = Some(DstGap::Error);
    patch.time.dst_overlap = Some(DstOverlap::Earlier);
    patch.provider.overrides = Some(OverridePolicy::PreferNative);
    patch.calendars.civil_calendar = Some(Calendar::BikramSambat);
    Profile {
        id: ProfileId::new("conformance-baseline"),
        version: 1,
        base: None,
        patch,
        sources: vec![Citation::new(
            "*",
            Source::new(
                "baseline-engine",
                "fixtures/README.md, the baseline conventions",
            ),
        )],
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
