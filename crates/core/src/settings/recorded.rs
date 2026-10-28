//! The settings a baseline engine's stored snapshot means
//! (`03-design/compat-shim.md` §4): one table, [`ROWS`], from a stored
//! field and value to what it becomes, and [`recorded`], which folds a
//! snapshot through it over `conformance-baseline`.
//!
//! A snapshot is the settings an application on the baseline engine kept
//! beside each chart, in the engine's own field names. The table is the
//! one place they are mapped: a field it does not know is refused, never
//! skipped, and so is a value it does not know. What the snapshot does not
//! hold is the baseline's fixed convention, which `conformance-baseline`
//! already carries (guide §5.1), so a stored default translates to that
//! profile unchanged.
//!
//! ```
//! use teistro_core::settings::{Node, recorded::{Snapshot, recorded}};
//!
//! let snapshot = Snapshot::from_json(r#"{"ayanamsaId": 3, "nodeType": "true", "engineVersion": "1.8.0"}"#)?;
//! let translated = recorded(&snapshot)?;
//! assert_eq!(translated.natal.profile.as_str(), "conformance-baseline");
//! assert_eq!(translated.natal.settings.frame.node, Some(Node::True));
//! // The version is read and kept aside, as a note.
//! assert_eq!(translated.notes[0].field, "engineVersion");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::knobs::{
    Balance, Centre, CharaKarakas, Ekadhipatya, Node, PolarPolicy, Shodhana, Sunrise, Zodiac,
};
use super::profiles::conformance_baseline;
use super::{
    Profile, ProfileId, Resolved, Settings, SettingsPatch, Siddhanta, SunriseConvention, SuryaBija,
    SuryaPlanets, SuryaSunrise,
};
use crate::catalogue::{Ayanamsha, DashaSystem, HouseSystem};
use crate::error::{Error, Status};
use crate::quantity::Depth;

/// The profile a translation's natal settings are laid over: the baseline
/// engine's defaults, knob by knob (guide §5.1).
///
/// ```
/// use teistro_core::settings::{Profile, recorded::PROFILE};
///
/// assert!(Profile::shipped(PROFILE).is_some());
/// ```
pub const PROFILE: &str = "conformance-baseline";

/// The profile the KP context opens: the baseline casts KP under
/// Krishnamurti's ayanamsha and Placidus, on its own frame (§4.2).
///
/// ```
/// use teistro_core::settings::{Profile, recorded::KP_PROFILE};
///
/// assert!(Profile::shipped(KP_PROFILE).is_some());
/// ```
pub const KP_PROFILE: &str = "kp-default";

/// Where an unknown field or value is explained.
const TABLE: &str = "03-design/compat-shim.md §4.2";

/// Where a language paired with another script is explained.
const PAIRINGS: &str = "03-design/compat-shim.md §8";

/// One value of a snapshot as it was stored: what a JSON column holds.
///
/// ```
/// use teistro_core::settings::recorded::Stored;
///
/// let stored: Stored = serde_json::from_str(r#"{"language": "ne", "timeFormat24h": true}"#)?;
/// assert!(matches!(stored, Stored::Fields(ref fields) if fields["timeFormat24h"] == Stored::Flag(true)));
/// // Nothing stored reads as absent, and writes back as null.
/// assert_eq!(serde_json::from_str::<Stored>("null")?, Stored::Absent);
/// assert_eq!(serde_json::to_string(&Stored::Absent)?, "null");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stored {
    /// Nothing: the engine's default stands.
    Absent,
    /// A flag.
    Flag(bool),
    /// A whole number.
    Number(i64),
    /// Text.
    Text(String),
    /// A nested record: the locale, the per-system dasha depths.
    Fields(BTreeMap<String, Stored>),
}

impl Stored {
    /// The text, when the value is text.
    fn text(&self) -> Option<&str> {
        match self {
            Stored::Text(text) => Some(text),
            _ => None,
        }
    }

    /// The value as one column of the recording engine's canonical form
    /// writes it, which spells every value as text: a value a row names as
    /// text for that field (`nodeType=true`) stays text, and otherwise a
    /// flag or a number is itself and anything else is text.
    fn of_canonical(field: &str, text: &str) -> Stored {
        let named = ROWS.iter().any(|row| {
            row.names(field) && matches!(row.value, Value::Text(named) if named == text)
        });
        if named {
            return Stored::Text(text.to_owned());
        }
        match text {
            "true" => Stored::Flag(true),
            "false" => Stored::Flag(false),
            _ => text
                .parse()
                .map_or_else(|_| Stored::Text(text.to_owned()), Stored::Number),
        }
    }
}

impl core::fmt::Display for Stored {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&crate::envelope::canonical_json(self))
    }
}

/// A stored settings snapshot: the baseline engine's own field names, each
/// with what was stored.
///
/// ```
/// use teistro_core::settings::recorded::{Snapshot, Stored};
///
/// let snapshot = Snapshot::from_json(r#"{"houseSystem": "placidus", "suryaBija": null}"#)?;
/// assert_eq!(snapshot.fields["houseSystem"], Stored::Text("placidus".into()));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Snapshot {
    /// Every field, by its stored name.
    pub fields: BTreeMap<String, Stored>,
}

impl Snapshot {
    /// Reads a snapshot from its JSON: one object, a field to each key.
    ///
    /// ```
    /// use teistro_core::error::Status;
    /// use teistro_core::settings::recorded::Snapshot;
    ///
    /// assert!(Snapshot::from_json(r#"{"nodeType": "mean"}"#).is_ok());
    /// // Not an object, or a value no column holds, is refused naming the snapshot.
    /// let refused = Snapshot::from_json("[1]").unwrap_err();
    /// assert_eq!((refused.status, refused.field()), (Status::InvalidArg, Some("snapshot")));
    /// assert!(Snapshot::from_json(r#"{"dashaMaxLevel": 2.5}"#).is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `snapshot` for text that is not one JSON object of
    /// flags, whole numbers, text and nested objects.
    pub fn from_json(json: &str) -> Result<Snapshot, Error> {
        serde_json::from_str(json).map_err(|why| {
            Error::invalid_arg(format!("a stored settings snapshot: {why}")).with_field("snapshot")
        })
    }

    /// Reads a snapshot from the recording engine's canonical form, the
    /// text its settings hash is taken over: `field=value` columns joined
    /// by `|` (the corpus's `settings_hash.canonical`).
    ///
    /// ```
    /// use teistro_core::settings::recorded::{Snapshot, Stored};
    ///
    /// let snapshot = Snapshot::from_canonical("ayanamsaId=27|houseSystem=whole-sign|useTopocentric=false")?;
    /// assert_eq!(snapshot.fields["ayanamsaId"], Stored::Number(27));
    /// assert_eq!(snapshot.fields["useTopocentric"], Stored::Flag(false));
    /// // A column with no value is refused.
    /// assert!(Snapshot::from_canonical("ayanamsaId").is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `snapshot` for a column with no `=`.
    pub fn from_canonical(text: &str) -> Result<Snapshot, Error> {
        let fields = text
            .split('|')
            .map(|column| {
                column
                    .split_once('=')
                    .map(|(field, value)| (field.to_owned(), Stored::of_canonical(field, value)))
                    .ok_or_else(|| {
                        Error::invalid_arg(format!("the canonical column `{column}` has no value"))
                            .with_field("snapshot")
                    })
            })
            .collect::<Result<_, _>>()?;
        Ok(Snapshot { fields })
    }

    /// Every stored value that is not absent, by its path: a nested
    /// record's under its field (`locale.language`), in path order.
    fn leaves(&self) -> Vec<(String, &Stored)> {
        fn walk<'s>(path: String, value: &'s Stored, out: &mut Vec<(String, &'s Stored)>) {
            match value {
                Stored::Absent => {}
                Stored::Fields(fields) => {
                    for (key, nested) in fields {
                        walk(format!("{path}.{key}"), nested, out);
                    }
                }
                _ => out.push((path, value)),
            }
        }
        let mut out = Vec::new();
        for (field, value) in &self.fields {
            walk(field.clone(), value, &mut out);
        }
        out
    }
}

/// Which stored values a row takes.
///
/// ```
/// use teistro_core::settings::recorded::{ROWS, Value};
///
/// // The node is one of two words; the ayanamsha any number.
/// assert!(ROWS.iter().any(|row| row.field == "nodeType" && row.value == Value::Text("true")));
/// assert!(ROWS.iter().any(|row| row.field == "ayanamsaId" && matches!(row.value, Value::Each(_))));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    /// This text.
    Text(&'static str),
    /// This flag.
    Flag(bool),
    /// This number.
    Number(i64),
    /// Any value, which the row reads: what it accepts, in words.
    Each(&'static str),
}

impl Value {
    fn takes(self, stored: &Stored) -> bool {
        match (self, stored) {
            (Value::Text(want), Stored::Text(got)) => want == got,
            (Value::Flag(want), Stored::Flag(got)) => want == *got,
            (Value::Number(want), Stored::Number(got)) => want == *got,
            (Value::Each(_), _) => true,
            _ => false,
        }
    }
}

/// The ephemeris a translation needs at the front of its chain.
///
/// ```
/// use teistro_core::settings::recorded::RequiredEphemeris;
///
/// assert_eq!(serde_json::to_string(&RequiredEphemeris::SuryaSiddhanta)?, r#""SURYA_SIDDHANTA""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequiredEphemeris {
    /// The Surya Siddhanta, which the text's chart is computed by.
    SuryaSiddhanta,
}

/// A row's reading of a stored value: the natal patch it writes to, the
/// profile's own settings, the value's path and the value. `Err` says why
/// the value is refused.
pub type Reader = fn(&mut SettingsPatch, &Settings, &str, &Stored) -> Result<(), &'static str>;

/// What a stored value becomes.
///
/// ```
/// use teistro_core::settings::SettingsPatch;
/// use teistro_core::settings::recorded::{Becomes, ROWS};
///
/// // A row's knobs are written into a patch, which is how the table is
/// // printed: each row's patch as JSON.
/// let sidereal = ROWS.iter().find(|row| row.field == "zodiacMode").map(|row| row.becomes);
/// let mut patch = SettingsPatch::default();
/// if let Some(Becomes::Settings(write)) = sidereal {
///     write(&mut patch);
/// }
/// assert!(!patch.is_empty());
/// ```
#[derive(Clone, Copy, Debug)]
pub enum Becomes {
    /// Knobs laid over the profile.
    Settings(fn(&mut SettingsPatch)),
    /// Knobs read from the value: the member a number names, a depth.
    Read(Reader),
    /// Knobs, and the ephemeris they need at the front of the chain.
    Ephemeris(fn(&mut SettingsPatch), RequiredEphemeris),
    /// The locale a language is, with its script.
    Locale(&'static str),
    /// The locale's script, which must be the language's locale's.
    Script,
    /// Read and set aside: what it means, and why nothing computes from it.
    Note(&'static str),
    /// Refused: the crux or guide anchor that explains it, and why.
    Refused {
        /// The crux or the page and section.
        anchor: &'static str,
        /// Why, in one sentence.
        why: &'static str,
    },
}

/// One row of the table: a stored field, the values it takes, and what
/// they become.
///
/// ```
/// use teistro_core::settings::recorded::{Becomes, ROWS, Value};
///
/// let bija = ROWS.iter().find(|row| row.field == "suryaBija" && row.value == Value::Flag(true));
/// assert!(matches!(bija.map(|row| row.becomes), Some(Becomes::Refused { anchor: "C777", .. })));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Row {
    /// The stored field: its name, a nested field's path
    /// (`locale.language`), or `field.*` for every key of a stored record.
    pub field: &'static str,
    /// Which values the row takes.
    pub value: Value,
    /// What they become.
    pub becomes: Becomes,
}

impl Row {
    fn names(&self, path: &str) -> bool {
        match self.field.strip_suffix(".*") {
            Some(record) => path
                .strip_prefix(record)
                .is_some_and(|key| key.starts_with('.')),
            None => self.field == path,
        }
    }
}

/// The 18 dasha systems by the names the baseline engine stores them
/// under. Listed one by one rather than derived by a case rule, so a
/// spelling the SDK writes otherwise cannot be mapped wrongly.
///
/// ```
/// use teistro_core::catalogue::DashaSystem;
/// use teistro_core::settings::recorded::DASHA_SYSTEMS;
///
/// assert_eq!(DASHA_SYSTEMS.len(), 18);
/// assert!(DASHA_SYSTEMS.contains(&("niryana-shoola", DashaSystem::NiryanaShoola)));
/// ```
pub const DASHA_SYSTEMS: [(&str, DashaSystem); 18] = [
    ("vimshottari", DashaSystem::Vimshottari),
    ("yogini", DashaSystem::Yogini),
    ("tribhagi", DashaSystem::Tribhagi),
    ("chara", DashaSystem::Chara),
    ("narayana", DashaSystem::Narayana),
    ("ashtottari", DashaSystem::Ashtottari),
    ("kalachakra", DashaSystem::Kalachakra),
    ("dwisaptati-sama", DashaSystem::DwisaptatiSama),
    ("shatabdika", DashaSystem::Shatabdika),
    ("chaturashiti-sama", DashaSystem::ChaturashitiSama),
    ("dwadashottari", DashaSystem::Dwadashottari),
    ("panchottari", DashaSystem::Panchottari),
    ("shoola", DashaSystem::Shoola),
    ("niryana-shoola", DashaSystem::NiryanaShoola),
    ("padanadhamsa", DashaSystem::Padanadhamsa),
    ("mandooka", DashaSystem::Mandooka),
    ("drig", DashaSystem::Drig),
    ("trikona", DashaSystem::Trikona),
];

/// The house system a stored name stands for: the member whose key,
/// lower-cased with hyphens, is the name. The baseline engine offers the
/// catalogue's 22 systems under exactly those names.
///
/// ```
/// use teistro_core::catalogue::HouseSystem;
/// use teistro_core::settings::recorded::house_system;
///
/// assert_eq!(house_system("equal-mc"), Some(HouseSystem::EqualMc));
/// assert_eq!(house_system("EQUAL_MC"), None);
/// ```
#[must_use]
pub fn house_system(name: &str) -> Option<HouseSystem> {
    HouseSystem::ALL
        .into_iter()
        .find(|system| system.key().to_ascii_lowercase().replace('_', "-") == name)
}

/// The number a stored value holds.
fn number(stored: &Stored) -> Result<i64, &'static str> {
    match stored {
        Stored::Number(number) => Ok(*number),
        _ => Err("not a number"),
    }
}

/// Writes one depth to each system named, over the profile's depths.
fn depths(
    patch: &mut SettingsPatch,
    settings: &Settings,
    stored: &Stored,
    systems: impl IntoIterator<Item = DashaSystem>,
) -> Result<(), &'static str> {
    let depth = number(stored)?;
    let depth = u8::try_from(depth)
        .ok()
        .filter(|depth| (1..=5).contains(depth))
        .and_then(|depth| Depth::try_new(depth).ok())
        .ok_or("not a depth the baseline engine stores, 1 to 5")?;
    let map = patch
        .dasha
        .depth
        .get_or_insert_with(|| settings.dasha.depth.clone());
    for system in systems {
        map.insert(system, depth);
    }
    Ok(())
}

/// The table, in the order a snapshot is folded through it: a later row
/// wins where two write one knob, so a per-system depth stands over the
/// overall one. Each field's stored values are the inventory's
/// (`01-research/baseline-engine/00-inventory.md`, "Settings snapshot").
///
/// ```
/// use teistro_core::settings::recorded::{Becomes, ROWS};
///
/// // Every row says what it becomes; the refused ones say where it is explained.
/// let refused = ROWS.iter().filter(|row| matches!(row.becomes, Becomes::Refused { .. })).count();
/// assert_eq!(refused, 3);
/// ```
pub const ROWS: &[Row] = &[
    Row {
        field: "ayanamsaId",
        value: Value::Each("the number a catalogued ayanamsha's `swiss_mode` is"),
        becomes: Becomes::Read(|patch, _, _, stored| {
            let mode = number(stored)?;
            let member = Ayanamsha::ALL
                .into_iter()
                .find(|member| i64::from(member.attributes().swiss_mode) == mode)
                .ok_or("no catalogued ayanamsha has this `swiss_mode`")?;
            patch.frame.ayanamsha = Some(member.into());
            Ok(())
        }),
    },
    Row {
        field: "zodiacMode",
        value: Value::Text("sidereal"),
        becomes: Becomes::Settings(|patch| patch.frame.zodiac = Some(Zodiac::Sidereal)),
    },
    Row {
        field: "zodiacMode",
        value: Value::Text("tropical"),
        becomes: Becomes::Note(
            "the recording engine casts the same sidereal chart under it: the corpus's four \
             tropical variants equal their default charts field for field",
        ),
    },
    Row {
        field: "nodeType",
        value: Value::Text("mean"),
        becomes: Becomes::Settings(|patch| patch.frame.node = Some(Node::Mean)),
    },
    Row {
        field: "nodeType",
        value: Value::Text("true"),
        becomes: Becomes::Settings(|patch| patch.frame.node = Some(Node::True)),
    },
    Row {
        field: "useTopocentric",
        value: Value::Flag(true),
        becomes: Becomes::Settings(|patch| patch.frame.centre = Some(Centre::Topocentric)),
    },
    Row {
        field: "useTopocentric",
        value: Value::Flag(false),
        becomes: Becomes::Settings(|patch| patch.frame.centre = Some(Centre::Geocentric)),
    },
    Row {
        field: "houseSystem",
        value: Value::Each("the 22 names, a house system's key lower-cased with hyphens"),
        becomes: Becomes::Read(|patch, settings, _, stored| {
            let system = stored
                .text()
                .and_then(house_system)
                .ok_or("not one of the 22 house systems")?;
            patch.houses.placement_system = Some(system);
            // The recording engine divides a quadrant system's undefined
            // polar arc as Porphyry does, silently (measured on the
            // corpus's Tromsø variant, `03-design/compat-shim.md` §4.2);
            // the profile's own whole-sign has no such arc, so it keeps
            // the profile's policy.
            if system != settings.houses.placement_system {
                patch.houses.polar_policy = Some(PolarPolicy::FallbackPorphyry);
            }
            Ok(())
        }),
    },
    Row {
        field: "siddhanta",
        value: Value::Text("drik"),
        becomes: Becomes::Settings(|patch| patch.frame.siddhanta = Some(Siddhanta::Drik)),
    },
    Row {
        field: "siddhanta",
        value: Value::Text("surya"),
        // What the recording engine computes under it, measured on the
        // corpus's two surya variants (`03-design/compat-shim.md` §4.2):
        // the text's places, geocentric whatever `useTopocentric` stores
        // and in the text's own zodiac whatever `ayanamsaId` stores, under
        // the modern sunrise. This row is read after both, so it wins.
        becomes: Becomes::Ephemeris(
            |patch| {
                patch.frame.siddhanta = Some(Siddhanta::Surya {
                    bija: SuryaBija::None,
                    sunrise: SuryaSunrise::Modern,
                    planets: SuryaPlanets::Text,
                });
                patch.frame.centre = Some(Centre::Geocentric);
                patch.frame.ayanamsha = Some(Ayanamsha::Suryasiddhanta.into());
            },
            RequiredEphemeris::SuryaSiddhanta,
        ),
    },
    Row {
        field: "suryaBija",
        value: Value::Flag(false),
        becomes: Becomes::Note(
            "the text's own revolutions, which `siddhanta: surya` already asks for",
        ),
    },
    Row {
        field: "suryaBija",
        value: Value::Flag(true),
        becomes: Becomes::Refused {
            anchor: "C777",
            why: "the baseline's bija re-anchors the mean node alone, and no named set of revolutions does that",
        },
    },
    Row {
        field: "sunriseMode",
        value: Value::Text("apparent-refraction"),
        becomes: Becomes::Settings(|patch| {
            patch.day.sunrise = Some(Sunrise::UpperLimbRefraction.into());
        }),
    },
    Row {
        field: "sunriseMode",
        value: Value::Text("geometric"),
        becomes: Becomes::Settings(|patch| {
            patch.day.sunrise = Some(Sunrise::CentreNoRefraction.into());
        }),
    },
    // C776: the two modes whose names say a limb the altitude does not
    // mean, translated by the altitude the baseline drives the centre to.
    Row {
        field: "sunriseMode",
        value: Value::Text("upper-limb"),
        becomes: Becomes::Settings(|patch| {
            patch.day.sunrise = Some(SunriseConvention::Custom {
                altitude_deg: -0.5667,
            });
        }),
    },
    Row {
        field: "sunriseMode",
        value: Value::Text("lower-limb"),
        becomes: Becomes::Settings(|patch| {
            patch.day.sunrise = Some(SunriseConvention::Custom { altitude_deg: -1.1 });
        }),
    },
    Row {
        field: "dashaBalanceMethod",
        value: Value::Text("spatial"),
        becomes: Becomes::Settings(|patch| patch.dasha.balance = Some(Balance::Spatial)),
    },
    Row {
        field: "dashaBalanceMethod",
        value: Value::Text("temporal"),
        becomes: Becomes::Settings(|patch| patch.dasha.balance = Some(Balance::Temporal)),
    },
    Row {
        field: "dashaMaxLevel",
        value: Value::Each("1 to 5, each of the 18 systems"),
        becomes: Becomes::Read(|patch, settings, _, stored| {
            depths(
                patch,
                settings,
                stored,
                DASHA_SYSTEMS.map(|(_, system)| system),
            )
        }),
    },
    Row {
        field: "dashaDepthBySystem.*",
        value: Value::Each("1 to 5, the system its key names"),
        becomes: Becomes::Read(|patch, settings, path, stored| {
            let system = path
                .rsplit_once('.')
                .and_then(|(_, name)| DASHA_SYSTEMS.iter().find(|(stored, _)| *stored == name))
                .map(|(_, system)| *system)
                .ok_or("not one of the 18 systems the baseline engine names")?;
            depths(patch, settings, stored, [system])
        }),
    },
    Row {
        field: "charaKarakaSystem",
        value: Value::Number(7),
        becomes: Becomes::Settings(|patch| {
            patch.jaimini.chara_karakas = Some(CharaKarakas::Seven);
        }),
    },
    Row {
        field: "charaKarakaSystem",
        value: Value::Number(8),
        becomes: Becomes::Settings(|patch| {
            patch.jaimini.chara_karakas = Some(CharaKarakas::Eight);
        }),
    },
    Row {
        field: "ekadhipatyaMethod",
        value: Value::Text("classical"),
        becomes: Becomes::Settings(|patch| {
            patch.strength.ekadhipatya = Some(Ekadhipatya::EmptyToZero);
            patch.strength.shodhana = Some(Shodhana::Sarva);
        }),
    },
    Row {
        field: "ekadhipatyaMethod",
        value: Value::Text("zero"),
        becomes: Becomes::Refused {
            anchor: "C60",
            why: "no text gives it and the baseline's own documentation calls it legacy",
        },
    },
    Row {
        field: "ekadhipatyaMethod",
        value: Value::Text("transfer"),
        becomes: Becomes::Refused {
            anchor: "C60",
            why: "no text gives it and the baseline's own documentation calls it legacy",
        },
    },
    Row {
        field: "locale.language",
        value: Value::Text("ne"),
        becomes: Becomes::Locale("ne-Deva-NP"),
    },
    Row {
        field: "locale.language",
        value: Value::Text("en"),
        becomes: Becomes::Locale("en-Latn"),
    },
    Row {
        field: "locale.language",
        value: Value::Text("hi"),
        becomes: Becomes::Locale("hi-Deva-IN"),
    },
    Row {
        field: "locale.language",
        value: Value::Text("sa"),
        becomes: Becomes::Locale("sa-Deva"),
    },
    Row {
        field: "locale.script",
        value: Value::Each("the language's locale's script"),
        becomes: Becomes::Script,
    },
    Row {
        field: "locale.numeralSystem",
        value: Value::Each("any"),
        becomes: Becomes::Note("presentation: numbers are the application's to format"),
    },
    Row {
        field: "locale.calendarSystem",
        value: Value::Each("any"),
        becomes: Becomes::Note("presentation: dates are the application's to format"),
    },
    Row {
        field: "locale.timeFormat24h",
        value: Value::Each("any"),
        becomes: Becomes::Note("presentation: the clock is the application's to format"),
    },
    Row {
        field: "engineVersion",
        value: Value::Each("any"),
        becomes: Becomes::Note("kept for the stored chart's comparison (guide §8.3)"),
    },
    Row {
        field: "swissephVersion",
        value: Value::Each("any"),
        becomes: Becomes::Note("kept for the stored chart's comparison (guide §8.3)"),
    },
    Row {
        field: "serializerVersion",
        value: Value::Each("any"),
        becomes: Becomes::Note("kept for the stored chart's comparison (guide §8.3)"),
    },
];

/// A profile and the settings patch laid over it: what a context opens
/// with.
///
/// ```
/// use teistro_core::settings::recorded::{Snapshot, recorded};
///
/// let translated = recorded(&Snapshot::default())?;
/// // Opened as a context would be, it is the profile itself.
/// let natal = translated.natal.resolve()?;
/// assert_eq!(natal.profile.as_str(), "conformance-baseline");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Opening {
    /// The profile.
    pub profile: ProfileId,
    /// The patch over it.
    pub settings: SettingsPatch,
}

impl Opening {
    /// The settings a context opened this way holds.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `profile` for a profile the SDK does not ship, and
    /// the patched profile's coherence errors.
    pub fn resolve(&self) -> Result<Resolved, Error> {
        let profile = Profile::shipped(self.profile.as_str()).ok_or_else(|| {
            Error::invalid_arg(format!("no shipped profile is `{}`", self.profile))
                .with_field("profile")
        })?;
        Ok(profile.resolve(&self.settings)?)
    }
}

/// A stored field read and set aside: nothing computes from it.
///
/// ```
/// use teistro_core::settings::recorded::{Snapshot, Stored, recorded};
///
/// let snapshot = Snapshot::from_json(r#"{"locale": {"numeralSystem": "devanagari"}}"#)?;
/// let note = &recorded(&snapshot)?.notes[0];
/// assert_eq!((note.field.as_str(), &note.value), ("locale.numeralSystem", &Stored::Text("devanagari".into())));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Note {
    /// The stored field's path.
    pub field: String,
    /// What was stored.
    pub value: Stored,
    /// Why nothing computes from it.
    pub why: &'static str,
}

/// What a stored snapshot means to the SDK: a natal context, a KP context,
/// the ephemeris the natal one needs at the front of its chain, the
/// locale, and the fields read and set aside.
///
/// Transits need no context of their own: `conformance-baseline` casts
/// them from the Earth's centre, as the baseline does (crux C778).
///
/// ```
/// use teistro_core::settings::recorded::{RequiredEphemeris, Snapshot, recorded};
///
/// let snapshot = Snapshot::from_json(r#"{"siddhanta": "surya", "locale": {"language": "ne", "script": "Deva"}}"#)?;
/// let translated = recorded(&snapshot)?;
/// assert_eq!(translated.ephemeris, Some(RequiredEphemeris::SuryaSiddhanta));
/// assert_eq!(translated.locale, Some("ne-Deva-NP"));
/// assert_eq!(translated.kp.profile.as_str(), "kp-default");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Recorded {
    /// The chart's context: [`PROFILE`] and the snapshot's knobs.
    pub natal: Opening,
    /// The KP context: [`KP_PROFILE`] on the natal frame's node, centre
    /// and ayanamsha basis and its polar day, as the baseline casts KP.
    pub kp: Opening,
    /// The ephemeris the natal context needs at the front of its chain,
    /// when it needs one.
    pub ephemeris: Option<RequiredEphemeris>,
    /// The locale the snapshot's language is, when it stores one.
    pub locale: Option<&'static str>,
    /// The fields read and set aside, in table order.
    pub notes: Vec<Note>,
}

/// A stored field or value the translation refuses.
///
/// ```
/// use teistro_core::error::{Error, Status};
/// use teistro_core::settings::recorded::{Snapshot, recorded};
///
/// let refused = recorded(&Snapshot::from_json(r#"{"suryaBija": true}"#)?).unwrap_err();
/// assert_eq!((refused.field.as_str(), refused.anchor, refused.decided), ("suryaBija", "C777", true));
/// // As an error: a decision is `UNSUPPORTED`, naming the field and the anchor.
/// let error = Error::from(refused);
/// assert_eq!((error.status, error.field(), error.hint()), (Status::Unsupported, Some("snapshot.suryaBija"), Some("C777")));
/// # Ok::<(), Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    /// The stored field's path.
    pub field: String,
    /// What was stored.
    pub value: Stored,
    /// The crux, or the page and section, that explains it.
    pub anchor: &'static str,
    /// Why, in one sentence.
    pub why: &'static str,
    /// Whether a row of the table refuses it, as a decision; otherwise the
    /// table does not know the field or the value.
    pub decided: bool,
}

impl Refused {
    fn unknown(field: &str, value: &Stored, why: &'static str) -> Refused {
        Refused {
            field: field.to_owned(),
            value: value.clone(),
            anchor: TABLE,
            why,
            decided: false,
        }
    }
}

impl core::fmt::Display for Refused {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "the stored `{}: {}` is refused ({}): {}",
            self.field, self.value, self.anchor, self.why
        )
    }
}

impl std::error::Error for Refused {}

impl From<Refused> for Error {
    fn from(refused: Refused) -> Error {
        let status = if refused.decided {
            Status::Unsupported
        } else {
            Status::InvalidArg
        };
        Error::new(status, refused.to_string())
            .with_field(format!("snapshot.{}", refused.field))
            .with_hint(refused.anchor)
    }
}

/// The row that takes a stored value at a path, with its place in the
/// table.
fn row_of(path: &str, value: &Stored) -> Result<(usize, &'static Row), Refused> {
    let named: Vec<(usize, &Row)> = ROWS
        .iter()
        .enumerate()
        .filter(|(_, row)| row.names(path))
        .collect();
    if named.is_empty() {
        return Err(Refused::unknown(
            path,
            value,
            "a field the table does not know",
        ));
    }
    named
        .into_iter()
        .find(|(_, row)| row.value.takes(value))
        .ok_or_else(|| Refused::unknown(path, value, "a value the table does not know"))
}

/// Translates a stored snapshot into the SDK's settings: [`ROWS`] folded
/// over [`PROFILE`], in table order.
///
/// ```
/// use teistro_core::settings::recorded::{Snapshot, recorded};
///
/// // The corpus's default snapshot is the profile, knob for knob.
/// let stored = Snapshot::from_canonical(
///     "ayanamsaId=1|dashaBalanceMethod=spatial|ekadhipatyaMethod=classical|engineVersion=1.8.0|houseSystem=whole-sign|nodeType=mean|siddhanta=drik|sunriseMode=apparent-refraction|swissephVersion=2.10.03|useTopocentric=true|zodiacMode=sidereal",
/// )?;
/// let translated = recorded(&stored)?;
/// let profile = teistro_core::settings::Profile::shipped("conformance-baseline").unwrap();
/// assert_eq!(translated.natal.resolve()?.settings, profile.settings());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// [`Refused`] for a field or a value the table does not know, a value a
/// row cannot read, a row's refusal, or a language paired with a script
/// its locale does not have.
pub fn recorded(snapshot: &Snapshot) -> Result<Recorded, Refused> {
    let base = conformance_baseline().settings();
    let mut read = snapshot
        .leaves()
        .into_iter()
        .map(|(path, value)| row_of(&path, value).map(|(index, row)| (index, row, path, value)))
        .collect::<Result<Vec<_>, _>>()?;
    read.sort_by_key(|(index, ..)| *index);
    let mut natal = SettingsPatch::default();
    let mut ephemeris = None;
    let mut locale = None;
    let mut script = None;
    let mut notes = Vec::new();
    for (_, row, path, value) in read {
        match row.becomes {
            Becomes::Settings(write) => write(&mut natal),
            Becomes::Read(reader) => reader(&mut natal, &base, &path, value)
                .map_err(|why| Refused::unknown(&path, value, why))?,
            Becomes::Ephemeris(write, needs) => {
                write(&mut natal);
                ephemeris = Some(needs);
            }
            Becomes::Locale(tag) => locale = Some(tag),
            Becomes::Script => {
                script = Some(
                    value
                        .text()
                        .ok_or_else(|| Refused::unknown(&path, value, "not a script subtag"))?,
                );
            }
            Becomes::Note(why) => notes.push(Note {
                field: path,
                value: value.clone(),
                why,
            }),
            Becomes::Refused { anchor, why } => {
                return Err(Refused {
                    field: path,
                    value: value.clone(),
                    anchor,
                    why,
                    decided: true,
                });
            }
        }
    }
    if let (Some(tag), Some(script)) = (locale, script) {
        if tag.split('-').nth(1) != Some(script) {
            return Err(Refused {
                field: String::from("locale.script"),
                value: Stored::Text(script.to_owned()),
                anchor: PAIRINGS,
                why: "the SDK ships no locale pairing that language with this script",
                decided: false,
            });
        }
    }
    let settings = base.patched(&natal);
    let mut kp = SettingsPatch::default();
    kp.frame.node = Some(settings.frame.node);
    kp.frame.centre = Some(settings.frame.centre);
    kp.frame.ayanamsha_basis = Some(settings.frame.ayanamsha_basis);
    kp.day.polar_day_policy = Some(settings.day.polar_day_policy);
    Ok(Recorded {
        natal: Opening {
            profile: ProfileId::new(PROFILE),
            settings: natal,
        },
        kp: Opening {
            profile: ProfileId::new(KP_PROFILE),
            settings: kp,
        },
        ephemeris,
        locale,
        notes,
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests;
