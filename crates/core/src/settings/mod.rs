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
use crate::envelope::Hash;
use crate::error::{Error, Status};
use crate::quantity::Depth;
pub use knobs::{
    AyanamshaBasis, Balance, Centre, CharaKarakas, DayBoundary, DeltaT, DstGap, DstOverlap,
    Ekadhipatya, GhatiReckoning, HoraReckoning, LunarMonth, NakshatraScheme, Node, NodeAspects,
    NodeCoLordship, OverridePolicy, PolarDayPolicy, PolarPolicy, Positions, SeedOverflow, Sunrise,
    Tier, UnattestedDn, UnknownTime, YearLength, Zodiac,
};
pub use profiles::{Profile, ProfileId, SHIPPED_PROFILES, root};

/// The settings document's schema version; a later knob appends and an
/// old document still hashes the same under the old schema.
pub const SCHEMA: u16 = 1;

/// Which ayanamsha: a catalogued one, or a custom definition.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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
}

impl From<Sunrise> for SunriseConvention {
    fn from(which: Sunrise) -> SunriseConvention {
        SunriseConvention::Named { which }
    }
}

/// Which Surya Siddhanta model, when the siddhanta knob is classical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[serde(deny_unknown_fields)]
pub struct Precision {
    /// Decimals of a degree value beside its exact integer.
    pub angle_decimals: u8,
    /// Decimals of a Julian day.
    pub instant_decimals: u8,
    /// Decimals of a score.
    pub score_decimals: u8,
}

macro_rules! group {
    ($(#[$m:meta])* $name:ident, $patch:ident { $( $(#[$fm:meta])* $field:ident : $ty:ty ),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $( $(#[$fm])* pub $field: $ty ),+
        }

        #[doc = concat!("The patch of `", stringify!($name), "`: every knob optional.")]
        #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
        /// Modern or classical astronomy.
        siddhanta: Siddhanta,
        /// Twenty-seven or twenty-eight nakshatras.
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
        seed_overflow: SeedOverflow,
    }
);

group!(
    /// Jaimini conventions.
    Jaimini, JaiminiPatch {
        /// Seven or eight chara karakas.
        chara_karakas: CharaKarakas,
        /// The nodes' co-lordship.
        node_co_lordship: NodeCoLordship,
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
    }
);

group!(
    /// Strength.
    Strength, StrengthPatch {
        /// The bala scheme.
        bala_scheme: BalaScheme,
        /// The Ashtakavarga reduction rule.
        ekadhipatya: Ekadhipatya,
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
        civil_calendar: Calendar,
        /// The lunar month system.
        lunar_month: LunarMonth,
        /// The era numbers a date carries.
        eras: BTreeSet<Era>,
    }
);

group!(
    /// The provider.
    Provider, ProviderPatch {
        /// The override policy (ADR-0013).
        overrides: OverridePolicy,
        /// The built-in ephemeris tier.
        tier: Tier,
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

/// A patch: every group's knobs optional.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SettingsPatch {
    /// The frame.
    pub frame: FramePatch,
    /// House systems.
    pub houses: HousesPatch,
    /// The local day.
    pub day: DayPatch,
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
    /// every knob present. The JSON layer's map is ordered, so a pass
    /// through `Value` sorts every object.
    #[must_use]
    pub fn canonical_json(&self) -> String {
        serde_json::to_value(self)
            .and_then(|value| serde_json::to_string(&value))
            .unwrap_or_default()
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
        let settings: Settings = serde_json::from_str(json).map_err(|e| Diagnostics {
            items: vec![Diagnostic::error("json", e.to_string(), &[])],
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
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The settings cannot be used.
    Error,
    /// Worth a look; recorded in provenance.
    Warning,
}

/// One coherence finding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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
    /// The warnings, for provenance.
    pub warnings: Vec<Diagnostic>,
    /// The profile it came from.
    pub profile: ProfileId,
}

/// The coherence rules, every finding returned.
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
    if let SunriseConvention::Custom { altitude_deg } = s.day.sunrise {
        if !altitude_deg.is_finite() || altitude_deg.abs() > 90.0 {
            out.push(Diagnostic::error(
                "sunrise-altitude",
                "a custom sunrise altitude is a finite number of degrees within -90 to 90",
                &["day.sunrise"],
            ));
        }
    }
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
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    fn shipped(id: &str) -> Profile {
        Profile::shipped(id).unwrap_or_else(|| panic!("{id}"))
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
        let classical = shipped("parashari-classical");
        assert_eq!(
            classical.base.as_ref().map(ProfileId::as_str),
            Some("nepali-default")
        );
        let resolved = classical
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(resolved.settings.houses.chalit_system, HouseSystem::Sripati);
        assert_eq!(resolved.settings.frame.zodiac, Zodiac::Sidereal);
        assert_eq!(resolved.profile.as_str(), "parashari-classical");
    }
}
