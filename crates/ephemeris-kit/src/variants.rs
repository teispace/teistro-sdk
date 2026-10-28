//! The variant check: the corpus's recorded variants — each chart cast
//! again under another of the recording engine's settings — founded under
//! the settings the corpus's manifest stored for them, translated by the
//! compatibility shim (`03-design/compat-shim.md`, item 2).
//!
//! No variant's settings are written here. The manifest stores each
//! variant profile's settings as the recording engine's own canonical
//! snapshot, [`recorded`] reads that snapshot into a profile and a patch,
//! and the chart is founded under what it gives. A profile the shim
//! refuses (a correction the SDK decided not to carry) is reported apart,
//! with the shim's reason: it is not a skip, because nothing was attempted
//! and failed, and not a pass, because nothing was compared.
//!
//! Compared are the positions and the lagna as [`corpus::run`] compares
//! them, the day's sunrises and sunset against the recorded day nearest
//! each, and the selected division's cusps, each under the band the
//! corpus's tolerance file gives the field.
//!
//! ```no_run
//! use std::path::Path;
//! use teistro_ephemeris_kit::corpus::Corpus;
//! use teistro_ephemeris_kit::variants;
//! use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
//!
//! let corpus = Corpus::open(Path::new("fixtures"))?;
//! let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
//! let run = variants::run(&corpus, &open, "builtin-compact")?;
//! println!("{}", run.report.markdown());
//! # Ok::<(), String>(())
//! ```
//!
//! [`recorded`]: teistro::settings::recorded::recorded

use std::collections::BTreeMap;

use serde_json::Value;
use teistro::settings::recorded::{Recorded, Snapshot, recorded};
use teistro::{ChartFoundation, Context, DayState, Ephemeris};

use crate::corpus::{
    self, Compared, Corpus, CorpusReport, Divergence, FULL_TIER_TRUNCATION, FixtureResult,
    MOON_TOPOCENTRIC_SPEED,
};
use crate::sdk_only::Open;

/// The corpus's manifest, under its root: every fixture with its profile,
/// and every profile with its stored settings.
const MANIFEST: &str = "baseline/manifest.json";

/// The profile a chart's own recording is cast under, which
/// [`corpus::run`] checks.
const DEFAULT: &str = "default";

/// Where the variants part from the SDK, and why; each measured on
/// 2026-10-28 over every built-in tier. Two are the charts' own
/// ([`corpus::KNOWN`]), met again; the others are a variant's.
///
/// ```
/// use teistro_ephemeris_kit::variants::KNOWN;
///
/// // Each says why, and none is named twice.
/// assert!(KNOWN.iter().all(|d| !d.why.is_empty() && !d.fields.is_empty()));
/// let names: std::collections::BTreeSet<_> = KNOWN.iter().map(|d| d.name).collect();
/// assert_eq!(names.len(), KNOWN.len());
/// ```
pub const KNOWN: [Divergence; 5] = [
    MOON_TOPOCENTRIC_SPEED,
    FULL_TIER_TRUNCATION,
    Divergence {
        name: "surya-text-theory",
        fields: &["positions.bodies.*.*"],
        fixtures: &[
            "c001-kathmandu-1990-04-14--surya",
            "c009-mumbai-1969-07-20--surya",
        ],
        classes: &[],
        why: "the recording engine's Surya Siddhanta is not the SDK's reading of the text: \
              in the text's own zodiac the star planets stand 0.04 to 0.95 degrees apart, \
              the Moon 3.5 and 3.7, the node 0.015 and the lagna 0.66 and 1.37, and the \
              recording gives no latitude or distance (it writes 0); its sunrise, the \
              modern one, reproduces to 1.5e-5 days (`03-design/compat-shim.md` §8)",
    },
    Divergence {
        name: "true-node",
        fields: &["positions.bodies.RAHU.*", "positions.bodies.KETU.*"],
        fixtures: &[
            "c001-kathmandu-1990-04-14--true-node",
            "c009-mumbai-1969-07-20--true-node",
            "c018-sydney-2000-09-15--true-node",
            "c033-cape-town-1990-02-11--true-node",
            "c040-buenos-aires-1986-06-22--true-node",
            "c052-kathmandu-2020-04-13--true-node",
        ],
        classes: &["builtin-compact", "builtin-standard", "builtin-full"],
        why: "the true node is the osculating node of each tier's own lunar theory, whose \
              truncated short-period terms the node's place and speed magnify: 1.2e-2 \
              degrees and 1.5e-2 degrees a day from the recording at worst on the compact \
              tier, 4e-3 degrees a day on the others, where the place is inside the band",
    },
    Divergence {
        name: "fixed-sunrise-altitude-at-64-north",
        fields: &["foundation.*.sunrise_jd", "foundation.*.sunset_jd"],
        fixtures: &["c027-reykjavik-1975-06-21--temporal"],
        classes: &["builtin-full"],
        why: "the recording engine puts the Sun's centre at a fixed -0.8333 degrees and the \
              SDK's upper limb with refraction at the Sun's own semidiameter (C776), \
              0.004 degrees higher at the June aphelion; at Reykjavik on the solstice the \
              Sun climbs 2 degrees an hour, so the sunrise is 7 seconds later and the \
              sunset 7 earlier on every tier, 8.4e-5 days, outside the full tier's band \
              of 5e-5 alone; the geometric variant of the same day reproduces to 1e-6",
    },
];

/// A variant profile the shim refused, with the variants it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// The manifest's name for the profile.
    pub profile: String,
    /// Its recorded variants, none of them founded.
    pub fixtures: Vec<String>,
    /// The shim's reason, naming the field, the value and its anchor.
    pub why: String,
}

/// What a variant run found: the report of every variant founded, and
/// every profile refused.
#[derive(Clone, Debug, PartialEq)]
pub struct VariantRun {
    /// The variants founded, in the corpus's report format. Its settings
    /// hash is `None`: each profile is founded under settings of its own.
    pub report: CorpusReport,
    /// The profiles the shim refused, in name order.
    pub refused: Vec<Refusal>,
}

/// Every variant profile's recorded fixtures, by profile, from the
/// manifest: the fixture's name (its file's stem) and its path under the
/// corpus root.
fn variants(manifest: &Value) -> Result<BTreeMap<String, Vec<(String, String)>>, String> {
    let fixtures = manifest
        .get("fixtures")
        .and_then(Value::as_array)
        .ok_or("the manifest lists no fixtures")?;
    let mut by_profile: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for fixture in fixtures {
        let text = |key: &str| {
            fixture
                .get(key)
                .and_then(Value::as_str)
                .ok_or_else(|| format!("a manifest fixture has no `{key}`"))
        };
        let profile = text("profile")?;
        if profile == DEFAULT {
            continue;
        }
        let file = text("file")?;
        let name = file
            .rsplit('/')
            .next()
            .and_then(|leaf| leaf.strip_suffix(".json"))
            .unwrap_or(file);
        by_profile
            .entry(profile.to_owned())
            .or_default()
            .push((name.to_owned(), format!("baseline/{file}")));
    }
    Ok(by_profile)
}

/// A variant profile's stored settings: the canonical snapshot the
/// manifest hashed them from.
fn snapshot(manifest: &Value, profile: &str) -> Result<Snapshot, String> {
    let canonical = manifest
        .pointer(&format!("/profiles/{profile}/settings_hash/canonical"))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("the manifest stores no settings for `{profile}`"))?;
    Snapshot::from_canonical(canonical).map_err(|why| format!("`{profile}`: {why}"))
}

/// The natal context a translation opens over the provider, behind the
/// ephemeris it asks for when it asks for one.
fn context(translated: &Recorded, open: Open<'_>) -> Result<Context, String> {
    let natal = &translated.natal;
    let patch = serde_json::to_string(&natal.settings).map_err(|why| why.to_string())?;
    let chain = translated
        .ephemeris
        .map(Ephemeris::from)
        .into_iter()
        .chain([Ephemeris::Provider(open())]);
    Context::builder()
        .profile(natal.profile.as_str())
        .settings_json(patch)
        .ephemeris(chain)
        .build()
        .map_err(|why| why.to_string())
}

/// The recorded day groups a founded day's events are matched in.
const DAYS: [&str; 3] = ["previous_day", "sunrise", "next_day"];

/// The founded day's sunrise, sunset and next sunrise, each against the
/// recorded event of its kind nearest it, within half a day; nothing for
/// a polar day, whose bounds a policy made.
fn day_events(recorded: &Value, founded: &ChartFoundation) -> Vec<Compared> {
    let day = &founded.day.day;
    if day.state != DayState::Normal {
        return Vec::new();
    }
    let events = [
        ("sunrise", day.sunrise),
        ("sunset", day.sunset),
        ("sunrise", day.next_sunrise),
    ];
    events
        .into_iter()
        .filter_map(|(kind, at)| {
            let at = at.get();
            DAYS.into_iter()
                .filter_map(|group| {
                    recorded
                        .pointer(&format!("/foundation/{group}/{kind}_jd"))
                        .and_then(Value::as_f64)
                        .map(|expected| (group, expected))
                })
                .filter(|(_, expected)| (expected - at).abs() < 0.5)
                .min_by(|(_, a), (_, b)| (a - at).abs().total_cmp(&(b - at).abs()))
                .map(|(group, expected)| {
                    (format!("foundation.{group}.{kind}_jd"), expected, at, false)
                })
        })
        .collect()
}

/// The selected division's recorded cusps against the founded bhavas'
/// boundaries, each indexed in its path.
fn cusps(recorded: &Value, founded: &ChartFoundation) -> Vec<Compared> {
    let Some(cusps) = recorded
        .pointer("/houses/selected/cusps")
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    cusps
        .iter()
        .zip(founded.houses.sandhi)
        .enumerate()
        .filter_map(|(index, (expected, found))| {
            expected.as_f64().map(|expected| {
                (
                    format!("houses.selected.cusps[{index}]"),
                    expected,
                    found,
                    true,
                )
            })
        })
        .collect()
}

/// Every value a variant and its founded chart both carry.
fn fields(recorded: &Value, founded: &ChartFoundation) -> Vec<Compared> {
    let mut out = corpus::positions(recorded, founded);
    out.extend(day_events(recorded, founded));
    out.extend(cusps(recorded, founded));
    out
}

/// Runs the variant check against a provider of a class.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::Corpus;
/// #
/// # const TOLERANCES: &str = r#"{"provider_classes": {"builtin-compact": "the compact tier"}, "fields": {}}"#;
/// # // A corpus of one refused variant profile, written where the example can.
/// # let dir = std::env::temp_dir().join(format!("teistro-kit-doc-{}-variants", std::process::id()));
/// # std::fs::create_dir_all(dir.join("baseline"))?;
/// # std::fs::write(dir.join("corpus.json"), r#"{"version": "0.0.1"}"#)?;
/// # std::fs::write(dir.join("tolerances.json"), TOLERANCES)?;
/// # std::fs::write(dir.join("baseline/manifest.json"), r#"{
/// #     "profiles": {"surya-bija": {"settings_hash": {"canonical": "siddhanta=surya|suryaBija=true"}}},
/// #     "fixtures": [{"profile": "surya-bija", "file": "variants/c001--surya-bija.json"}]
/// # }"#)?;
/// # let corpus = Corpus::open(&dir)?;
///
/// use teistro_ephemeris_kit::variants;
/// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
///
/// // The bija correction is refused (C777): reported apart, nothing founded.
/// let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
/// let run = variants::run(&corpus, &open, "builtin-compact")?;
/// assert!(run.report.results.is_empty());
/// assert_eq!((run.refused[0].profile.as_str(), run.refused[0].fixtures.as_slice()), ("surya-bija", ["c001--surya-bija".to_owned()].as_slice()));
/// assert!(run.refused[0].why.contains("C777"));
/// assert!(variants::run(&corpus, &open, "acme").is_err());
/// # std::fs::remove_dir_all(&dir)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// A class the corpus does not name, a manifest or recorded variant that
/// cannot be read, or a context the SDK cannot open under a translation.
pub fn run(corpus: &Corpus, open: Open<'_>, class: &str) -> Result<VariantRun, String> {
    corpus.tolerances.class(class)?;
    let manifest = corpus.read(MANIFEST)?;
    let identity = open().capabilities().identity;
    let mut results: Vec<FixtureResult> = Vec::new();
    let mut refused = Vec::new();
    for (profile, fixtures) in variants(&manifest)? {
        let translated = match recorded(&snapshot(&manifest, &profile)?) {
            Ok(translated) => translated,
            Err(refusal) => {
                refused.push(Refusal {
                    fixtures: fixtures.into_iter().map(|(name, _)| name).collect(),
                    profile,
                    why: refusal.to_string(),
                });
                continue;
            }
        };
        let sdk = context(&translated, open).map_err(|why| format!("`{profile}`: {why}"))?;
        for (name, path) in fixtures {
            let chart = corpus.read(&path)?;
            results.push(corpus::compare(
                &sdk,
                &corpus.tolerances,
                class,
                (&name, &chart),
                fields,
            ));
        }
    }
    Ok(VariantRun {
        report: corpus.report(&identity, class, None, results),
        refused,
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking, and index what they built"
    )]

    use serde_json::json;
    use teistro::catalogue::ChartKind;
    use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    use teistro::settings::PolarDayPolicy;
    use teistro::{ChartFoundation, Context, DayState, Ephemeris, PolarKind, UtcOffset};

    use super::{Compared, cusps, day_events, snapshot, variants};

    /// Nothing compared.
    const NONE: [Compared; 0] = [];

    /// A chart founded over the test provider: Kathmandu, 1990-04-14.
    fn founded() -> ChartFoundation {
        let sdk = Context::builder()
            .profile(crate::corpus::PROFILE)
            .ephemeris([Ephemeris::Test])
            .build()
            .unwrap();
        let place = Place::new(
            Latitude::try_new(27.7172).unwrap(),
            Longitude::try_new(85.324).unwrap(),
            Altitude::try_new(1400.0).unwrap(),
        );
        sdk.chart()
            .found(
                JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
                &place,
                UtcOffset::try_from_seconds(20_700).unwrap(),
                ChartKind::Natal,
            )
            .unwrap()
            .value
    }

    #[test]
    fn a_manifest_gives_each_variant_profile_its_fixtures_by_stem() {
        let manifest = json!({"fixtures": [
            {"profile": "default", "file": "charts/c001.json"},
            {"profile": "temporal", "file": "variants/c001--temporal.json"},
            {"profile": "temporal", "file": "variants/c007--temporal.json"},
        ]});
        let found = variants(&manifest).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(
            found["temporal"],
            [
                (
                    "c001--temporal".to_owned(),
                    "baseline/variants/c001--temporal.json".to_owned()
                ),
                (
                    "c007--temporal".to_owned(),
                    "baseline/variants/c007--temporal.json".to_owned()
                ),
            ]
        );
        assert!(variants(&json!({})).unwrap_err().contains("no fixtures"));
        let unnamed = json!({"fixtures": [{"file": "variants/c001--temporal.json"}]});
        assert!(variants(&unnamed).unwrap_err().contains("`profile`"));
        let fileless = json!({"fixtures": [{"profile": "temporal"}]});
        assert!(variants(&fileless).unwrap_err().contains("`file`"));
    }

    #[test]
    fn a_profile_s_settings_are_its_canonical_snapshot() {
        let manifest = json!({"profiles": {
            "temporal": {"settings_hash": {"canonical": "dashaBalanceMethod=temporal"}},
            "broken": {"settings_hash": {"canonical": "dashaBalanceMethod"}},
        }});
        assert_eq!(snapshot(&manifest, "temporal").unwrap().fields.len(), 1);
        assert!(
            snapshot(&manifest, "placidus")
                .unwrap_err()
                .contains("no settings")
        );
        assert!(
            snapshot(&manifest, "broken")
                .unwrap_err()
                .starts_with("`broken`")
        );
    }

    #[test]
    fn a_day_event_meets_the_recorded_event_of_its_kind_nearest_it() {
        let mut chart = founded();
        let day = chart.day.day.clone();
        let recorded = json!({"foundation": {
            "previous_day": {"sunrise_jd": day.sunrise.get() - 1.0, "sunset_jd": day.sunset.get() - 1.0},
            "sunrise": {"sunrise_jd": day.sunrise.get() + 1e-4, "sunset_jd": day.sunset.get()},
            "next_day": {"sunrise_jd": day.next_sunrise.get()},
        }});
        let events = day_events(&recorded, &chart);
        let paths: Vec<&str> = events.iter().map(|(path, ..)| path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "foundation.sunrise.sunrise_jd",
                "foundation.sunrise.sunset_jd",
                "foundation.next_day.sunrise_jd"
            ]
        );
        // Nothing recorded within half a day is nothing compared.
        assert_eq!(day_events(&json!({}), &chart), NONE);
        // A polar day's bounds are a policy's, and are not compared.
        chart.day.day.state = DayState::Polar {
            kind: PolarKind::Day,
            policy: PolarDayPolicy::NearestEvent,
        };
        assert_eq!(day_events(&recorded, &chart), NONE);
    }

    #[test]
    fn a_recorded_cusp_meets_the_bhava_boundary_of_its_index() {
        let chart = founded();
        let recorded = json!({"houses": {"selected": {"cusps": [chart.houses.sandhi[0], null]}}});
        let compared = cusps(&recorded, &chart);
        assert_eq!(compared.len(), 1);
        assert_eq!(compared[0].0, "houses.selected.cusps[0]");
        assert!(compared[0].3, "a cusp is an angle");
        assert_eq!(cusps(&json!({}), &chart), NONE);
    }
}
