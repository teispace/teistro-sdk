//! The corpus check: a provider's positions against the conformance
//! corpus's recorded charts, under the band `tolerances.json` gives its
//! provider class (ADR-0022).
//!
//! Every recorded birth is founded by the SDK under the
//! `conformance-baseline` profile — the recording engine's defaults — over
//! the provider being checked, and every position the recording and the
//! founded chart both carry is compared: each graha's sidereal and
//! tropical longitude, latitude, speed and distance, and the lagna. The
//! band is the corpus's own, keyed by field pattern and provider class and
//! never per fixture, so a provider is held to what its class promises and
//! not to a number chosen for it here. A field the file does not list is
//! compared exactly, as the file says.
//!
//! The answer is a [`CorpusReport`] in the corpus's own report format
//! (`schema/report.schema.json`): every field compared is listed with how
//! far it missed, because a report that lists only failures cannot be told
//! from one that compared nothing.
//!
//! ```no_run
//! use std::path::Path;
//! use teistro_ephemeris_kit::corpus::{self, Corpus};
//! use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
//!
//! let corpus = Corpus::open(Path::new("fixtures"))?;
//! let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
//! let report = corpus::run(&corpus, &open, "builtin-compact")?;
//! println!("{}", report.markdown());
//! # Ok::<(), String>(())
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro::{ChartFoundation, Context, Ephemeris, UtcOffset};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::{ChartKind, Graha};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};

use crate::sdk_only::Open;

/// The profile the corpus's charts reproduce under.
///
/// ```
/// use teistro_core::settings::Profile;
/// use teistro_ephemeris_kit::corpus::PROFILE;
///
/// // A profile the SDK ships.
/// assert!(Profile::shipped(PROFILE).is_some());
/// ```
pub const PROFILE: &str = "conformance-baseline";

/// Where the recorded charts are, under the corpus's root.
const CHARTS: &str = "baseline/charts";

/// The report format this writes, as the corpus names it.
pub(crate) const REPORT_SCHEMA: &str = "teistro-conformance/report/1";

/// The one central tolerance file, as the corpus writes it.
#[derive(Clone, Debug, Deserialize)]
struct TolerancesFile {
    provider_classes: BTreeMap<String, String>,
    fields: BTreeMap<String, BTreeMap<String, Value>>,
}

/// The corpus's tolerance bands, by field pattern and provider class.
///
/// ```
/// # const TOLERANCES: &str = r#"{
/// #     "provider_classes": {"builtin-compact": "the compact tier", "same-ephemeris": "the recording engine's own"},
/// #     "fields": {
/// #         "positions.bodies.*.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.01, "same-ephemeris": 1e-6},
/// #         "positions.bodies.MOON.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.02, "same-ephemeris": 1e-6}
/// #     }
/// # }"#;
///
/// use teistro_ephemeris_kit::corpus::Tolerances;
///
/// let tolerances = Tolerances::from_json(TOLERANCES)?;
/// assert_eq!(tolerances.classes.len(), 2);
/// # Ok::<(), String>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Tolerances {
    /// Every provider class the corpus names, with what it means.
    pub classes: BTreeMap<String, String>,
    /// Each field pattern's band per class.
    bands: Vec<(String, BTreeMap<String, f64>)>,
}

impl Tolerances {
    /// Reads the file, refusing a band that is not a number.
    ///
    /// ```
    /// use teistro_ephemeris_kit::corpus::Tolerances;
    ///
    /// // A band that is not a number is refused, naming it.
    /// let text = r#"{"provider_classes": {"de": "a DE reader"}, "fields": {"jpl.*": {"de": "tight"}}}"#;
    /// assert!(Tolerances::from_json(text).unwrap_err().contains("`jpl.*`.de"));
    /// ```
    ///
    /// # Errors
    ///
    /// JSON that is not the tolerance file's shape.
    pub fn from_json(text: &str) -> Result<Tolerances, String> {
        let file: TolerancesFile =
            serde_json::from_str(text).map_err(|why| format!("tolerances.json: {why}"))?;
        let mut bands = Vec::new();
        for (pattern, entry) in file.fields {
            let mut by_class = BTreeMap::new();
            for (class, value) in entry {
                if class == "unit" {
                    continue;
                }
                let band = value.as_f64().ok_or_else(|| {
                    format!("tolerances.json: `{pattern}`.{class} is not a number")
                })?;
                by_class.insert(class, band);
            }
            bands.push((pattern, by_class));
        }
        Ok(Tolerances {
            classes: file.provider_classes,
            bands,
        })
    }

    /// The band a field is held to under a class: the most specific pattern
    /// that matches it, or `0` — exact — when none does.
    ///
    /// ```
    /// # const TOLERANCES: &str = r#"{
    /// #     "provider_classes": {"builtin-compact": "the compact tier", "same-ephemeris": "the recording engine's own"},
    /// #     "fields": {
    /// #         "positions.bodies.*.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.01, "same-ephemeris": 1e-6},
    /// #         "positions.bodies.MOON.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.02, "same-ephemeris": 1e-6}
    /// #     }
    /// # }"#;
    ///
    /// use teistro_ephemeris_kit::corpus::Tolerances;
    ///
    /// let tolerances = Tolerances::from_json(TOLERANCES)?;
    /// // The most specific pattern wins; a field none names is exact.
    /// assert_eq!(tolerances.band("positions.bodies.MOON.sidereal_longitude_deg", "builtin-compact"), 0.02);
    /// assert_eq!(tolerances.band("positions.bodies.SUN.sidereal_longitude_deg", "builtin-compact"), 0.01);
    /// assert_eq!(tolerances.band("positions.bodies.SUN.latitude_deg", "builtin-compact"), 0.0);
    /// # Ok::<(), String>(())
    /// ```
    #[must_use]
    pub fn band(&self, path: &str, class: &str) -> f64 {
        self.bands
            .iter()
            .filter(|(pattern, _)| matches(pattern, path))
            .max_by_key(|(pattern, _)| pattern.chars().filter(|c| *c != '*').count())
            .and_then(|(_, by_class)| by_class.get(class).copied())
            .unwrap_or(0.0)
    }

    /// Refuses a class the corpus does not name, naming the ones it does.
    ///
    /// ```
    /// # const TOLERANCES: &str = r#"{
    /// #     "provider_classes": {"builtin-compact": "the compact tier", "same-ephemeris": "the recording engine's own"},
    /// #     "fields": {
    /// #         "positions.bodies.*.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.01, "same-ephemeris": 1e-6},
    /// #         "positions.bodies.MOON.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.02, "same-ephemeris": 1e-6}
    /// #     }
    /// # }"#;
    ///
    /// use teistro_ephemeris_kit::corpus::Tolerances;
    ///
    /// let tolerances = Tolerances::from_json(TOLERANCES)?;
    /// assert!(tolerances.class("same-ephemeris").is_ok());
    /// assert!(tolerances.class("acme").unwrap_err().ends_with("it names builtin-compact, same-ephemeris"));
    /// # Ok::<(), String>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A class that is not a key of the file's `provider_classes`.
    pub fn class(&self, class: &str) -> Result<(), String> {
        if self.classes.contains_key(class) {
            return Ok(());
        }
        let known: Vec<&str> = self.classes.keys().map(String::as_str).collect();
        Err(format!(
            "`{class}` is not a provider class of the corpus; it names {}",
            known.join(", ")
        ))
    }
}

/// Whether a field pattern matches a path: segment by segment, a `*`
/// standing for any run of characters inside one segment.
///
/// ```
/// use teistro_ephemeris_kit::corpus::matches;
///
/// assert!(matches("positions.bodies.*.speed_deg_per_day", "positions.bodies.MOON.speed_deg_per_day"));
/// // A `*` stays inside its segment.
/// assert!(!matches("positions.*", "positions.bodies.MOON"));
/// assert!(matches("jpl.geometric.*_km", "jpl.geometric.distance_km"));
/// ```
#[must_use]
pub fn matches(pattern: &str, path: &str) -> bool {
    let patterns: Vec<&str> = pattern.split('.').collect();
    let segments: Vec<&str> = path.split('.').collect();
    patterns.len() == segments.len()
        && patterns
            .iter()
            .zip(&segments)
            .all(|(pattern, segment)| glob(pattern, segment))
}

/// A single segment against a pattern whose `*` matches any run.
fn glob(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, rest)) => {
            let Some(after) = text.strip_prefix(head) else {
                return false;
            };
            (0..=after.len())
                .filter(|at| after.is_char_boundary(*at))
                .any(|at| glob(rest, &after[at..]))
        }
    }
}

/// The corpus a run is made against: its root, its version, the commit
/// it was checked out at when that can be read, and its tolerances.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::Corpus;
/// #
/// # const TOLERANCES: &str = r#"{
/// #     "provider_classes": {"builtin-compact": "the compact tier", "same-ephemeris": "the recording engine's own"},
/// #     "fields": {
/// #         "positions.bodies.*.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.01, "same-ephemeris": 1e-6},
/// #         "positions.bodies.MOON.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.02, "same-ephemeris": 1e-6}
/// #     }
/// # }"#;
/// # // A corpus of no recorded charts, written where the example can.
/// # let dir = std::env::temp_dir().join(format!("teistro-kit-doc-{}-corpus", std::process::id()));
/// # std::fs::create_dir_all(dir.join("baseline/charts"))?;
/// # std::fs::write(dir.join("corpus.json"), r#"{"version": "0.0.1"}"#)?;
/// # std::fs::write(dir.join("tolerances.json"), TOLERANCES)?;
/// # let corpus = Corpus::open(&dir)?;
///
/// // Not a checkout, so no commit to cite.
/// assert_eq!((corpus.version.as_str(), corpus.commit.as_deref()), ("0.0.1", None));
/// # std::fs::remove_dir_all(&dir)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Corpus {
    root: PathBuf,
    /// The corpus's own version, from `corpus.json`.
    pub version: String,
    /// The commit it is checked out at, from its git metadata; `None` when
    /// it is not a checkout, and then the report cannot be cited.
    pub commit: Option<String>,
    /// Its tolerance bands.
    pub tolerances: Tolerances,
}

impl Corpus {
    /// Opens the corpus at its root (the SDK's `fixtures/`).
    ///
    /// ```
    /// # use teistro_ephemeris_kit::corpus::Corpus;
    /// #
    /// # const TOLERANCES: &str = r#"{
    /// #     "provider_classes": {"builtin-compact": "the compact tier", "same-ephemeris": "the recording engine's own"},
    /// #     "fields": {
    /// #         "positions.bodies.*.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.01, "same-ephemeris": 1e-6},
    /// #         "positions.bodies.MOON.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.02, "same-ephemeris": 1e-6}
    /// #     }
    /// # }"#;
    /// # // A corpus of no recorded charts, written where the example can.
    /// # let dir = std::env::temp_dir().join(format!("teistro-kit-doc-{}-open", std::process::id()));
    /// # std::fs::create_dir_all(dir.join("baseline/charts"))?;
    /// # std::fs::write(dir.join("corpus.json"), r#"{"version": "0.0.1"}"#)?;
    /// # std::fs::write(dir.join("tolerances.json"), TOLERANCES)?;
    /// # let corpus = Corpus::open(&dir)?;
    ///
    /// assert!(corpus.tolerances.class("builtin-compact").is_ok());
    /// // A root without the corpus's files is refused, saying where to get them.
    /// let empty = dir.join("baseline");
    /// assert!(Corpus::open(&empty).unwrap_err().contains("git submodule update --init"));
    /// # std::fs::remove_dir_all(&dir)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A root without `corpus.json` or `tolerances.json`, or either not
    /// the corpus's shape.
    pub fn open(root: &Path) -> Result<Corpus, String> {
        let read = |name: &str| {
            std::fs::read_to_string(root.join(name)).map_err(|why| {
                format!(
                    "{}: {why}. The corpus is a submodule; `git submodule update --init`",
                    root.join(name).display()
                )
            })
        };
        let manifest: Value = serde_json::from_str(&read("corpus.json")?)
            .map_err(|why| format!("corpus.json: {why}"))?;
        let version = manifest
            .get("version")
            .and_then(Value::as_str)
            .ok_or("corpus.json names no version")?
            .to_owned();
        Ok(Corpus {
            root: root.to_path_buf(),
            version,
            commit: commit_of(root),
            tolerances: Tolerances::from_json(&read("tolerances.json")?)?,
        })
    }

    /// A file of the corpus, by its path under the root.
    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// A JSON file of the corpus, by its path under the root.
    pub(crate) fn read(&self, relative: &str) -> Result<Value, String> {
        let path = self.path(relative);
        let text =
            std::fs::read_to_string(&path).map_err(|why| format!("{}: {why}", path.display()))?;
        serde_json::from_str(&text).map_err(|why| format!("{relative}: {why}"))
    }

    /// The report of a run over this corpus: what it compared, by whom.
    pub(crate) fn report(
        &self,
        identity: &teistro_port_ephemeris::Identity,
        class: &str,
        settings_hash: Option<String>,
        results: Vec<FixtureResult>,
    ) -> CorpusReport {
        CorpusReport {
            schema: REPORT_SCHEMA,
            corpus: CorpusId {
                version: self.version.clone(),
                commit: self.commit.clone(),
            },
            implementation: Implementation {
                name: format!("teistro over {}", identity.name),
                version: identity.version.clone(),
                binding: "rust",
                provider_class: class.to_owned(),
                platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
                settings_hash,
            },
            counts: Counts::of(&results),
            results,
        }
    }

    /// The recorded charts, in name order.
    ///
    /// # Errors
    ///
    /// A directory that cannot be read.
    fn charts(&self) -> Result<Vec<(String, Value)>, String> {
        let directory = self.root.join(CHARTS);
        let entries = std::fs::read_dir(&directory)
            .map_err(|why| format!("{}: {why}", directory.display()))?;
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let name = path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let text = std::fs::read_to_string(&path)
                    .map_err(|why| format!("{}: {why}", path.display()))?;
                let chart = serde_json::from_str(&text).map_err(|why| format!("{name}: {why}"))?;
                Ok((name, chart))
            })
            .collect()
    }
}

/// The commit a checkout's `HEAD` names: a submodule's `.git` is a file
/// pointing at its git directory, and a submodule's `HEAD` is detached, so
/// it is the commit itself; a branch is followed one reference deep.
fn commit_of(root: &Path) -> Option<String> {
    let dot_git = root.join(".git");
    let git_dir = if dot_git.is_file() {
        let pointer = std::fs::read_to_string(&dot_git).ok()?;
        root.join(pointer.trim().strip_prefix("gitdir:")?.trim())
    } else {
        dot_git
    };
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    let commit = match head.strip_prefix("ref:") {
        None => head.to_owned(),
        Some(reference) => std::fs::read_to_string(git_dir.join(reference.trim()))
            .ok()?
            .trim()
            .to_owned(),
    };
    (commit.len() >= 7 && commit.chars().all(|c| c.is_ascii_hexdigit())).then_some(commit)
}

/// What a report says of the corpus it was made against.
///
/// ```
/// use teistro_ephemeris_kit::corpus::CorpusId;
///
/// // A commit that is not known is left out of the report.
/// let id = CorpusId { version: "0.12.0".to_owned(), commit: None };
/// assert_eq!(serde_json::to_string(&id)?, r#"{"version":"0.12.0"}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CorpusId {
    /// The corpus's version.
    pub version: String,
    /// Its commit, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

/// What a report says of the implementation it scores.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
/// #
/// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
/// # let field = |path: &str, difference: f64, tolerance: f64| Field {
/// #     path: path.to_owned(),
/// #     expected: 10.0,
/// #     found: 10.0 + difference,
/// #     difference,
/// #     tolerance,
/// #     within: difference <= tolerance,
/// # };
/// # let results = vec![
/// #     FixtureResult {
/// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
/// #         outcome: Outcome::Pass,
/// #         reason: None,
/// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
/// #     },
/// #     FixtureResult {
/// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
/// #         outcome: Outcome::Fail,
/// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
/// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
/// #     },
/// # ];
/// # let report = CorpusReport {
/// #     schema: "teistro-conformance/report/1",
/// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
/// #     implementation: Implementation {
/// #         name: "teistro over acme".to_owned(),
/// #         version: "1.0".to_owned(),
/// #         binding: "rust",
/// #         provider_class: "builtin-compact".to_owned(),
/// #         platform: "macos-aarch64".to_owned(),
/// #         settings_hash: None,
/// #     },
/// #     counts: Counts::of(&results),
/// #     results,
/// # };
///
/// let json = serde_json::to_value(&report.implementation)?;
/// assert_eq!(json["provider_class"], "builtin-compact");
/// assert!(json.get("settings_hash").is_none());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Implementation {
    /// The SDK and the provider it computed with.
    pub name: String,
    /// The provider's version.
    pub version: String,
    /// Which binding ran.
    pub binding: &'static str,
    /// The provider class, a key of the corpus's `provider_classes`.
    pub provider_class: String,
    /// The platform, as the Rust target names it.
    pub platform: String,
    /// The hash of the settings the charts were founded under; `None` when
    /// the provider was asked directly, through no context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_hash: Option<String>,
}

/// A fixture's outcome, as the corpus's report spells it.
///
/// ```
/// use teistro_ephemeris_kit::corpus::Outcome;
///
/// assert_eq!(serde_json::to_string(&Outcome::Skip)?, r#""skip""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// Every field compared was within its band.
    Pass,
    /// Some field was not.
    Fail,
    /// Not compared, with a reason: a skip is not a pass.
    Skip,
}

/// One field compared.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
/// #
/// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
/// # let field = |path: &str, difference: f64, tolerance: f64| Field {
/// #     path: path.to_owned(),
/// #     expected: 10.0,
/// #     found: 10.0 + difference,
/// #     difference,
/// #     tolerance,
/// #     within: difference <= tolerance,
/// # };
/// # let results = vec![
/// #     FixtureResult {
/// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
/// #         outcome: Outcome::Pass,
/// #         reason: None,
/// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
/// #     },
/// #     FixtureResult {
/// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
/// #         outcome: Outcome::Fail,
/// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
/// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
/// #     },
/// # ];
/// # let report = CorpusReport {
/// #     schema: "teistro-conformance/report/1",
/// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
/// #     implementation: Implementation {
/// #         name: "teistro over acme".to_owned(),
/// #         version: "1.0".to_owned(),
/// #         binding: "rust",
/// #         provider_class: "builtin-compact".to_owned(),
/// #         platform: "macos-aarch64".to_owned(),
/// #         settings_hash: None,
/// #     },
/// #     counts: Counts::of(&results),
/// #     results,
/// # };
///
/// // Each field compared is listed, inside its band or not.
/// let missed = &report.results[1].fields[0];
/// assert!(!missed.within && missed.difference > missed.tolerance);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Field {
    /// The field's path in the recorded chart.
    pub path: String,
    /// What the corpus records.
    pub expected: f64,
    /// What the provider gave.
    pub found: f64,
    /// How far apart they are; for a longitude, the shorter way round.
    pub difference: f64,
    /// The band it was held to.
    pub tolerance: f64,
    /// Whether it was within the band.
    pub within: bool,
}

/// One recorded chart's result.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
/// #
/// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
/// # let field = |path: &str, difference: f64, tolerance: f64| Field {
/// #     path: path.to_owned(),
/// #     expected: 10.0,
/// #     found: 10.0 + difference,
/// #     difference,
/// #     tolerance,
/// #     within: difference <= tolerance,
/// # };
/// # let results = vec![
/// #     FixtureResult {
/// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
/// #         outcome: Outcome::Pass,
/// #         reason: None,
/// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
/// #     },
/// #     FixtureResult {
/// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
/// #         outcome: Outcome::Fail,
/// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
/// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
/// #     },
/// # ];
/// # let report = CorpusReport {
/// #     schema: "teistro-conformance/report/1",
/// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
/// #     implementation: Implementation {
/// #         name: "teistro over acme".to_owned(),
/// #         version: "1.0".to_owned(),
/// #         binding: "rust",
/// #         provider_class: "builtin-compact".to_owned(),
/// #         platform: "macos-aarch64".to_owned(),
/// #         settings_hash: None,
/// #     },
/// #     counts: Counts::of(&results),
/// #     results,
/// # };
///
/// let failed = &report.results[1];
/// assert_eq!(failed.outcome, Outcome::Fail);
/// assert!(failed.reason.as_deref().is_some_and(|why| why.starts_with("outside its band")));
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FixtureResult {
    /// The recorded chart.
    pub fixture: String,
    /// What came of it.
    pub outcome: Outcome,
    /// Why it was skipped or failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Every field compared.
    pub fields: Vec<Field>,
}

/// How many fixtures came out each way.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
/// #
/// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
/// # let field = |path: &str, difference: f64, tolerance: f64| Field {
/// #     path: path.to_owned(),
/// #     expected: 10.0,
/// #     found: 10.0 + difference,
/// #     difference,
/// #     tolerance,
/// #     within: difference <= tolerance,
/// # };
/// # let results = vec![
/// #     FixtureResult {
/// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
/// #         outcome: Outcome::Pass,
/// #         reason: None,
/// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
/// #     },
/// #     FixtureResult {
/// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
/// #         outcome: Outcome::Fail,
/// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
/// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
/// #     },
/// # ];
/// # let report = CorpusReport {
/// #     schema: "teistro-conformance/report/1",
/// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
/// #     implementation: Implementation {
/// #         name: "teistro over acme".to_owned(),
/// #         version: "1.0".to_owned(),
/// #         binding: "rust",
/// #         provider_class: "builtin-compact".to_owned(),
/// #         platform: "macos-aarch64".to_owned(),
/// #         settings_hash: None,
/// #     },
/// #     counts: Counts::of(&results),
/// #     results,
/// # };
///
/// assert_eq!(report.counts, Counts { passed: 1, failed: 1, skipped: 0 });
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Counts {
    /// Passed.
    pub passed: usize,
    /// Failed.
    pub failed: usize,
    /// Skipped.
    pub skipped: usize,
}

impl Counts {
    /// How many of the results came out each way.
    ///
    /// ```
    /// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
    /// #
    /// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
    /// # let field = |path: &str, difference: f64, tolerance: f64| Field {
    /// #     path: path.to_owned(),
    /// #     expected: 10.0,
    /// #     found: 10.0 + difference,
    /// #     difference,
    /// #     tolerance,
    /// #     within: difference <= tolerance,
    /// # };
    /// # let results = vec![
    /// #     FixtureResult {
    /// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
    /// #         outcome: Outcome::Pass,
    /// #         reason: None,
    /// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
    /// #     },
    /// #     FixtureResult {
    /// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
    /// #         outcome: Outcome::Fail,
    /// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
    /// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
    /// #     },
    /// # ];
    /// # let report = CorpusReport {
    /// #     schema: "teistro-conformance/report/1",
    /// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
    /// #     implementation: Implementation {
    /// #         name: "teistro over acme".to_owned(),
    /// #         version: "1.0".to_owned(),
    /// #         binding: "rust",
    /// #         provider_class: "builtin-compact".to_owned(),
    /// #         platform: "macos-aarch64".to_owned(),
    /// #         settings_hash: None,
    /// #     },
    /// #     counts: Counts::of(&results),
    /// #     results,
    /// # };
    ///
    /// assert_eq!(Counts::of(&report.results[..1]).passed, 1);
    /// assert_eq!(Counts::of(&[]), Counts::default());
    /// ```
    #[must_use]
    pub fn of(results: &[FixtureResult]) -> Counts {
        let mut counts = Counts::default();
        for result in results {
            match result.outcome {
                Outcome::Pass => counts.passed += 1,
                Outcome::Fail => counts.failed += 1,
                Outcome::Skip => counts.skipped += 1,
            }
        }
        counts
    }
}

/// One implementation's score against one version of the corpus, in the
/// corpus's report format.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
/// #
/// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
/// # let field = |path: &str, difference: f64, tolerance: f64| Field {
/// #     path: path.to_owned(),
/// #     expected: 10.0,
/// #     found: 10.0 + difference,
/// #     difference,
/// #     tolerance,
/// #     within: difference <= tolerance,
/// # };
/// # let results = vec![
/// #     FixtureResult {
/// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
/// #         outcome: Outcome::Pass,
/// #         reason: None,
/// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
/// #     },
/// #     FixtureResult {
/// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
/// #         outcome: Outcome::Fail,
/// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
/// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
/// #     },
/// # ];
/// # let report = CorpusReport {
/// #     schema: "teistro-conformance/report/1",
/// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
/// #     implementation: Implementation {
/// #         name: "teistro over acme".to_owned(),
/// #         version: "1.0".to_owned(),
/// #         binding: "rust",
/// #         provider_class: "builtin-compact".to_owned(),
/// #         platform: "macos-aarch64".to_owned(),
/// #         settings_hash: None,
/// #     },
/// #     counts: Counts::of(&results),
/// #     results,
/// # };
///
/// let json = serde_json::to_value(&report)?;
/// assert_eq!(json["schema"], "teistro-conformance/report/1");
/// assert_eq!(json["results"][1]["outcome"], "fail");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CorpusReport {
    /// The report format.
    pub schema: &'static str,
    /// The corpus.
    pub corpus: CorpusId,
    /// The implementation.
    pub implementation: Implementation,
    /// Each recorded chart.
    pub results: Vec<FixtureResult>,
    /// The counts.
    pub counts: Counts,
}

/// The worst a field pattern missed by, as a share of its band.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
/// #
/// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
/// # let field = |path: &str, difference: f64, tolerance: f64| Field {
/// #     path: path.to_owned(),
/// #     expected: 10.0,
/// #     found: 10.0 + difference,
/// #     difference,
/// #     tolerance,
/// #     within: difference <= tolerance,
/// # };
/// # let results = vec![
/// #     FixtureResult {
/// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
/// #         outcome: Outcome::Pass,
/// #         reason: None,
/// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
/// #     },
/// #     FixtureResult {
/// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
/// #         outcome: Outcome::Fail,
/// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
/// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
/// #     },
/// # ];
/// # let report = CorpusReport {
/// #     schema: "teistro-conformance/report/1",
/// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
/// #     implementation: Implementation {
/// #         name: "teistro over acme".to_owned(),
/// #         version: "1.0".to_owned(),
/// #         binding: "rust",
/// #         provider_class: "builtin-compact".to_owned(),
/// #         platform: "macos-aarch64".to_owned(),
/// #         settings_hash: None,
/// #     },
/// #     counts: Counts::of(&results),
/// #     results,
/// # };
///
/// // The worst sidereal longitude, the Moon's of 2350.
/// let worst = report.worst();
/// assert_eq!(worst.len(), 1);
/// assert_eq!((worst[0].field.as_str(), worst[0].fixture.as_str()), ("sidereal_longitude_deg", "c046-kathmandu-2350-01-01"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Worst {
    /// The field, with the body in place of `*`.
    pub field: String,
    /// The largest difference.
    pub difference: f64,
    /// Its band.
    pub tolerance: f64,
    /// The recorded chart it was found in.
    pub fixture: String,
}

impl CorpusReport {
    /// The misses against the known divergences, both ways.
    ///
    /// ```
    /// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
    /// #
    /// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
    /// # let field = |path: &str, difference: f64, tolerance: f64| Field {
    /// #     path: path.to_owned(),
    /// #     expected: 10.0,
    /// #     found: 10.0 + difference,
    /// #     difference,
    /// #     tolerance,
    /// #     within: difference <= tolerance,
    /// # };
    /// # let results = vec![
    /// #     FixtureResult {
    /// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
    /// #         outcome: Outcome::Pass,
    /// #         reason: None,
    /// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
    /// #     },
    /// #     FixtureResult {
    /// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
    /// #         outcome: Outcome::Fail,
    /// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
    /// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
    /// #     },
    /// # ];
    /// # let report = CorpusReport {
    /// #     schema: "teistro-conformance/report/1",
    /// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
    /// #     implementation: Implementation {
    /// #         name: "teistro over acme".to_owned(),
    /// #         version: "1.0".to_owned(),
    /// #         binding: "rust",
    /// #         provider_class: "builtin-compact".to_owned(),
    /// #         platform: "macos-aarch64".to_owned(),
    /// #         settings_hash: None,
    /// #     },
    /// #     counts: Counts::of(&results),
    /// #     results,
    /// # };
    ///
    /// use teistro_ephemeris_kit::corpus::KNOWN;
    ///
    /// // 2350 is past the Delta T table: the miss is a known one.
    /// let judged = report.against(&KNOWN);
    /// assert_eq!((judged.explained, judged.unexplained.len()), (1, 0));
    /// assert_eq!(judged.explained_by[0].0.name, "delta-t-beyond-the-table");
    /// ```
    #[must_use]
    pub fn against<'k>(&self, known: &'k [Divergence]) -> Judged<'k> {
        let applicable: Vec<&Divergence> = known
            .iter()
            .filter(|divergence| divergence.applies_to(&self.implementation.provider_class))
            .collect();
        let mut used = vec![0_usize; applicable.len()];
        let mut unexplained = Vec::new();
        let mut explained = 0;
        for result in &self.results {
            if result.outcome == Outcome::Skip {
                unexplained.push(format!(
                    "{}: skipped, {}",
                    result.fixture,
                    result.reason.as_deref().unwrap_or("with no reason")
                ));
            }
            for field in result.fields.iter().filter(|field| !field.within) {
                match applicable
                    .iter()
                    .position(|divergence| divergence.explains(&result.fixture, &field.path))
                {
                    Some(at) => {
                        if let Some(slot) = used.get_mut(at) {
                            *slot += 1;
                        }
                        explained += 1;
                    }
                    None => unexplained.push(format!("{}: {}", result.fixture, field.path)),
                }
            }
        }
        let by: Vec<(&Divergence, usize)> = applicable.into_iter().zip(used).collect();
        Judged {
            unexplained,
            idle: by
                .iter()
                .filter(|(_, count)| *count == 0)
                .map(|(divergence, _)| *divergence)
                .collect(),
            explained,
            explained_by: by.into_iter().filter(|(_, count)| *count > 0).collect(),
        }
    }

    /// Whether every fixture passed and none was skipped.
    ///
    /// ```
    /// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
    /// #
    /// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
    /// # let field = |path: &str, difference: f64, tolerance: f64| Field {
    /// #     path: path.to_owned(),
    /// #     expected: 10.0,
    /// #     found: 10.0 + difference,
    /// #     difference,
    /// #     tolerance,
    /// #     within: difference <= tolerance,
    /// # };
    /// # let results = vec![
    /// #     FixtureResult {
    /// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
    /// #         outcome: Outcome::Pass,
    /// #         reason: None,
    /// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
    /// #     },
    /// #     FixtureResult {
    /// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
    /// #         outcome: Outcome::Fail,
    /// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
    /// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
    /// #     },
    /// # ];
    /// # let report = CorpusReport {
    /// #     schema: "teistro-conformance/report/1",
    /// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
    /// #     implementation: Implementation {
    /// #         name: "teistro over acme".to_owned(),
    /// #         version: "1.0".to_owned(),
    /// #         binding: "rust",
    /// #         provider_class: "builtin-compact".to_owned(),
    /// #         platform: "macos-aarch64".to_owned(),
    /// #         settings_hash: None,
    /// #     },
    /// #     counts: Counts::of(&results),
    /// #     results,
    /// # };
    ///
    /// // One failed, so not passed, though every miss is explained.
    /// assert!(!report.passed());
    /// ```
    #[must_use]
    pub fn passed(&self) -> bool {
        self.counts.failed == 0 && self.counts.skipped == 0
    }

    /// The largest miss of each kind of field, across the corpus: the
    /// measurement a provisional band is settled from.
    ///
    /// ```
    /// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
    /// #
    /// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
    /// # let field = |path: &str, difference: f64, tolerance: f64| Field {
    /// #     path: path.to_owned(),
    /// #     expected: 10.0,
    /// #     found: 10.0 + difference,
    /// #     difference,
    /// #     tolerance,
    /// #     within: difference <= tolerance,
    /// # };
    /// # let results = vec![
    /// #     FixtureResult {
    /// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
    /// #         outcome: Outcome::Pass,
    /// #         reason: None,
    /// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
    /// #     },
    /// #     FixtureResult {
    /// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
    /// #         outcome: Outcome::Fail,
    /// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
    /// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
    /// #     },
    /// # ];
    /// # let report = CorpusReport {
    /// #     schema: "teistro-conformance/report/1",
    /// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
    /// #     implementation: Implementation {
    /// #         name: "teistro over acme".to_owned(),
    /// #         version: "1.0".to_owned(),
    /// #         binding: "rust",
    /// #         provider_class: "builtin-compact".to_owned(),
    /// #         platform: "macos-aarch64".to_owned(),
    /// #         settings_hash: None,
    /// #     },
    /// #     counts: Counts::of(&results),
    /// #     results,
    /// # };
    ///
    /// let worst = &report.worst()[0];
    /// assert!((worst.difference - 3e-2).abs() < 1e-12);
    /// assert_eq!(worst.tolerance, 2e-2);
    /// ```
    #[must_use]
    pub fn worst(&self) -> Vec<Worst> {
        let mut worst: BTreeMap<String, Worst> = BTreeMap::new();
        for result in &self.results {
            for field in &result.fields {
                let kind = field
                    .path
                    .rsplit_once('.')
                    .map_or(field.path.as_str(), |(_, last)| last)
                    .to_owned();
                let entry = worst.entry(kind.clone()).or_insert_with(|| Worst {
                    field: kind,
                    difference: -1.0,
                    tolerance: field.tolerance,
                    fixture: String::new(),
                });
                if field.difference > entry.difference {
                    entry.difference = field.difference;
                    entry.tolerance = field.tolerance;
                    entry.fixture.clone_from(&result.fixture);
                }
            }
        }
        worst.into_values().collect()
    }

    /// The report for a reader: the counts, the worst of each field
    /// against its band, and every field that missed.
    ///
    /// ```
    /// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
    /// #
    /// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
    /// # let field = |path: &str, difference: f64, tolerance: f64| Field {
    /// #     path: path.to_owned(),
    /// #     expected: 10.0,
    /// #     found: 10.0 + difference,
    /// #     difference,
    /// #     tolerance,
    /// #     within: difference <= tolerance,
    /// # };
    /// # let results = vec![
    /// #     FixtureResult {
    /// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
    /// #         outcome: Outcome::Pass,
    /// #         reason: None,
    /// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
    /// #     },
    /// #     FixtureResult {
    /// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
    /// #         outcome: Outcome::Fail,
    /// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
    /// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
    /// #     },
    /// # ];
    /// # let report = CorpusReport {
    /// #     schema: "teistro-conformance/report/1",
    /// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
    /// #     implementation: Implementation {
    /// #         name: "teistro over acme".to_owned(),
    /// #         version: "1.0".to_owned(),
    /// #         binding: "rust",
    /// #         provider_class: "builtin-compact".to_owned(),
    /// #         platform: "macos-aarch64".to_owned(),
    /// #         settings_hash: None,
    /// #     },
    /// #     counts: Counts::of(&results),
    /// #     results,
    /// # };
    ///
    /// let page = report.markdown();
    /// assert!(page.starts_with("corpus 0.12.0 against teistro over acme 1.0 as `builtin-compact`: 1 passed, 1 failed, 0 skipped"));
    /// assert!(page.contains("| sidereal_longitude_deg | 3.000e-2 | 2.0e-2 | 150% |"));
    /// ```
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut out = format!(
            "corpus {} against {} {} as `{}`: {} passed, {} failed, {} skipped\n\n\
             | field | worst | band | share | in |\n|---|---:|---:|---:|---|\n",
            self.corpus.version,
            self.implementation.name,
            self.implementation.version,
            self.implementation.provider_class,
            self.counts.passed,
            self.counts.failed,
            self.counts.skipped,
        );
        for worst in self.worst() {
            let share = if worst.tolerance > 0.0 {
                format!("{:.0}%", 100.0 * worst.difference / worst.tolerance)
            } else {
                String::from("exact")
            };
            let _ = writeln!(
                out,
                "| {} | {:.3e} | {:.1e} | {share} | {} |",
                worst.field, worst.difference, worst.tolerance, worst.fixture
            );
        }
        for result in &self.results {
            if let Some(reason) = &result.reason {
                let _ = writeln!(out, "\n{} {:?}: {reason}", result.fixture, result.outcome);
            }
        }
        out
    }

    /// Writes the report as the corpus's JSON to `<dir>/<name>.json`.
    ///
    /// ```
    /// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
    /// #
    /// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
    /// # let field = |path: &str, difference: f64, tolerance: f64| Field {
    /// #     path: path.to_owned(),
    /// #     expected: 10.0,
    /// #     found: 10.0 + difference,
    /// #     difference,
    /// #     tolerance,
    /// #     within: difference <= tolerance,
    /// # };
    /// # let results = vec![
    /// #     FixtureResult {
    /// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
    /// #         outcome: Outcome::Pass,
    /// #         reason: None,
    /// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
    /// #     },
    /// #     FixtureResult {
    /// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
    /// #         outcome: Outcome::Fail,
    /// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
    /// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
    /// #     },
    /// # ];
    /// # let report = CorpusReport {
    /// #     schema: "teistro-conformance/report/1",
    /// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
    /// #     implementation: Implementation {
    /// #         name: "teistro over acme".to_owned(),
    /// #         version: "1.0".to_owned(),
    /// #         binding: "rust",
    /// #         provider_class: "builtin-compact".to_owned(),
    /// #         platform: "macos-aarch64".to_owned(),
    /// #         settings_hash: None,
    /// #     },
    /// #     counts: Counts::of(&results),
    /// #     results,
    /// # };
    ///
    /// let dir = std::env::temp_dir().join(format!("teistro-kit-doc-{}-report", std::process::id()));
    /// let path = report.write(&dir, "acme-corpus")?;
    /// assert_eq!(path, dir.join("acme-corpus.json"));
    /// std::fs::remove_dir_all(&dir)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// When the directory cannot be created or the file written.
    pub fn write(&self, dir: &Path, name: &str) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{name}.json"));
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&path, format!("{json}\n"))?;
        Ok(path)
    }
}

/// A way the SDK is known to part from the corpus, with why: the fields it
/// covers, the recorded charts it holds in (none named is every chart), and
/// the provider classes it holds under (none named is every class).
///
/// Each was measured before it was written down, and the list is held both
/// ways by [`CorpusReport::against`]: a miss no entry explains fails, and so
/// does an entry that explains nothing in a run it applies to — a
/// divergence the SDK has stopped having is a line that must go.
///
/// ```
/// use teistro_ephemeris_kit::corpus::Divergence;
///
/// let divergence = Divergence {
///     name: "acme-moon",
///     fields: &["positions.bodies.MOON.*"],
///     fixtures: &[],
///     classes: &["builtin-compact"],
///     why: "measured: the Moon's theory, not its truncation",
/// };
/// assert!(divergence.applies_to("builtin-compact") && !divergence.applies_to("builtin-full"));
/// assert!(divergence.explains("c001", "positions.bodies.MOON.distance_au"));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Divergence {
    /// A short name, for a report.
    pub name: &'static str,
    /// The field patterns it covers, as `tolerances.json` writes them.
    pub fields: &'static [&'static str],
    /// The recorded charts it holds in; empty for every chart.
    pub fixtures: &'static [&'static str],
    /// The provider classes it holds under; empty for every class.
    pub classes: &'static [&'static str],
    /// Why, measured.
    pub why: &'static str,
}

impl Divergence {
    /// Whether it applies to a run of a class.
    ///
    /// ```
    /// use teistro_ephemeris_kit::corpus::KNOWN;
    ///
    /// // Past the Delta T table every class misses alike.
    /// let beyond = KNOWN.iter().find(|d| d.name == "delta-t-beyond-the-table").ok_or("listed")?;
    /// assert!(beyond.applies_to("builtin-compact") && beyond.applies_to("same-ephemeris"));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn applies_to(&self, class: &str) -> bool {
        self.classes.is_empty() || self.classes.contains(&class)
    }

    /// Whether it explains one field's miss in one recorded chart.
    ///
    /// ```
    /// use teistro_ephemeris_kit::corpus::KNOWN;
    ///
    /// let beyond = KNOWN.iter().find(|d| d.name == "delta-t-beyond-the-table").ok_or("listed")?;
    /// assert!(beyond.explains("c046-kathmandu-2350-01-01", "positions.bodies.SUN.tropical_longitude_deg"));
    /// // Only in the charts it names.
    /// assert!(!beyond.explains("c001-kathmandu-2000-01-01", "positions.bodies.SUN.tropical_longitude_deg"));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn explains(&self, fixture: &str, path: &str) -> bool {
        (self.fixtures.is_empty() || self.fixtures.contains(&fixture))
            && self.fields.iter().any(|pattern| matches(pattern, path))
    }
}

/// The recording engine's topocentric Moon speed, which a variant run
/// meets too.
pub const MOON_TOPOCENTRIC_SPEED: Divergence = Divergence {
    name: "moon-topocentric-speed",
    fields: &["positions.bodies.MOON.speed_deg_per_day"],
    fixtures: &[],
    classes: &["builtin-standard", "builtin-full"],
    why: "the recording engine's topocentric Moon speed is not the derivative of its own \
          topocentric places, by 1.1e-3 to 1.6e-3 degrees a day at Kiritimati measured \
          over Teimeris itself, and the SDK's is: 3e-3 apart at worst",
};

/// The full tier's truncated series, which a variant run meets too.
pub const FULL_TIER_TRUNCATION: Divergence = Divergence {
    name: "full-tier-truncation",
    fields: &[
        "positions.bodies.SATURN.distance_au",
        "positions.bodies.JUPITER.distance_au",
        "positions.bodies.MERCURY.distance_au",
        "positions.bodies.MERCURY.speed_deg_per_day",
    ],
    fixtures: &[],
    classes: &["builtin-full"],
    why: "the full tier's truncated series carry the outer planets' distances to 4e-6 \
          AU (0.09 arcseconds at Saturn's) and Mercury's speed to 2e-4 degrees a day, \
          inside its arcsecond class; the corpus's provisional band asks 1e-6 AU and 1e-4",
};

/// Where the SDK parts from corpus 0.11.0, and why; each measured on
/// 2026-09-25 over every built-in tier and over Teimeris, the corpus's own
/// ephemeris (`docs/03-design/ephemeris-port-and-adapters.md` §9).
///
/// ```
/// use teistro_ephemeris_kit::corpus::KNOWN;
///
/// // Each says why, and none is named twice.
/// assert!(KNOWN.iter().all(|d| !d.why.is_empty() && !d.fields.is_empty()));
/// let names: std::collections::BTreeSet<_> = KNOWN.iter().map(|d| d.name).collect();
/// assert_eq!(names.len(), KNOWN.len());
/// ```
pub const KNOWN: [Divergence; 6] = [
    Divergence {
        name: "delta-t-beyond-the-table",
        fields: &["positions.bodies.*.*"],
        fixtures: &["c046-kathmandu-2350-01-01", "c048-kathmandu-2399-12-30"],
        classes: &[],
        why: "past the IERS table the SDK's Delta T is its own model's and the recording \
              engine extrapolates its own, and in 2350 and 2399 the two are minutes apart: \
              every body moves with the instant, whatever the ephemeris",
    },
    Divergence {
        name: "delta-t-before-the-table",
        fields: &[
            "positions.bodies.MOON.sidereal_longitude_deg",
            "positions.bodies.MOON.tropical_longitude_deg",
        ],
        fixtures: &[
            "c012-varanasi-1850-02-05",
            "c023-london-1830-06-26",
            "c047-london-1800-01-02",
        ],
        classes: &["builtin-full"],
        why: "before 1880 the SDK's Delta T is 1.8 to 5.2 seconds below the recording \
              engine's (13.7 against 18.9 in 1800), and the Moon moves half an arcsecond a \
              second: 8e-4 degrees in 1800, inside every tier's band but the full one's",
    },
    Divergence {
        name: "sidereal-time-before-1850",
        fields: &["positions.bodies.LAGNA.sidereal_longitude_deg"],
        fixtures: &["c023-london-1830-06-26", "c047-london-1800-01-02"],
        classes: &["same-ephemeris"],
        why: "the lagna of 1800 and 1830 stands 2.2e-4 and 2.6e-5 degrees from the \
              recording engine's over its own ephemeris, a difference in the Earth's \
              rotation before the modern era rather than in any position",
    },
    Divergence {
        name: "lagna-at-65-north",
        fields: &["positions.bodies.LAGNA.sidereal_longitude_deg"],
        fixtures: &["c043-fairbanks-2015-06-21"],
        classes: &["same-ephemeris"],
        why: "at Fairbanks the ascendant turns three times as fast as at the equator, and \
              two implementations' last digits part by 1.08e-6 degrees against a band of 1e-6",
    },
    MOON_TOPOCENTRIC_SPEED,
    FULL_TIER_TRUNCATION,
];

/// What [`CorpusReport::against`] found.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
/// #
/// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
/// # let field = |path: &str, difference: f64, tolerance: f64| Field {
/// #     path: path.to_owned(),
/// #     expected: 10.0,
/// #     found: 10.0 + difference,
/// #     difference,
/// #     tolerance,
/// #     within: difference <= tolerance,
/// # };
/// # let results = vec![
/// #     FixtureResult {
/// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
/// #         outcome: Outcome::Pass,
/// #         reason: None,
/// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
/// #     },
/// #     FixtureResult {
/// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
/// #         outcome: Outcome::Fail,
/// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
/// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
/// #     },
/// # ];
/// # let report = CorpusReport {
/// #     schema: "teistro-conformance/report/1",
/// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
/// #     implementation: Implementation {
/// #         name: "teistro over acme".to_owned(),
/// #         version: "1.0".to_owned(),
/// #         binding: "rust",
/// #         provider_class: "builtin-compact".to_owned(),
/// #         platform: "macos-aarch64".to_owned(),
/// #         settings_hash: None,
/// #     },
/// #     counts: Counts::of(&results),
/// #     results,
/// # };
///
/// use teistro_ephemeris_kit::corpus::KNOWN;
///
/// // The one divergence that applies to the compact tier explained a miss.
/// let judged = report.against(&KNOWN);
/// assert!(judged.idle.is_empty());
/// assert_eq!(judged.explained_by[0].1, 1);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Judged<'k> {
    /// Each miss no divergence explains, as `fixture: path`.
    pub unexplained: Vec<String>,
    /// Each divergence that applies to the run and explained nothing.
    pub idle: Vec<&'k Divergence>,
    /// How many misses the divergences explained.
    pub explained: usize,
    /// How many misses each divergence explained, those that explained one.
    pub explained_by: Vec<(&'k Divergence, usize)>,
}

impl Judged<'_> {
    /// Whether every miss is explained and every divergence explained one.
    ///
    /// ```
    /// # use teistro_ephemeris_kit::corpus::{CorpusId, CorpusReport, Counts, Field, FixtureResult, Implementation, Outcome};
    /// #
    /// # // Two recorded charts: one inside its band, one whose Moon missed in 2350.
    /// # let field = |path: &str, difference: f64, tolerance: f64| Field {
    /// #     path: path.to_owned(),
    /// #     expected: 10.0,
    /// #     found: 10.0 + difference,
    /// #     difference,
    /// #     tolerance,
    /// #     within: difference <= tolerance,
    /// # };
    /// # let results = vec![
    /// #     FixtureResult {
    /// #         fixture: "c001-kathmandu-2000-01-01".to_owned(),
    /// #         outcome: Outcome::Pass,
    /// #         reason: None,
    /// #         fields: vec![field("positions.bodies.SUN.sidereal_longitude_deg", 1e-5, 1e-2)],
    /// #     },
    /// #     FixtureResult {
    /// #         fixture: "c046-kathmandu-2350-01-01".to_owned(),
    /// #         outcome: Outcome::Fail,
    /// #         reason: Some("outside its band: positions.bodies.MOON.sidereal_longitude_deg".to_owned()),
    /// #         fields: vec![field("positions.bodies.MOON.sidereal_longitude_deg", 3e-2, 2e-2)],
    /// #     },
    /// # ];
    /// # let report = CorpusReport {
    /// #     schema: "teistro-conformance/report/1",
    /// #     corpus: CorpusId { version: "0.12.0".to_owned(), commit: None },
    /// #     implementation: Implementation {
    /// #         name: "teistro over acme".to_owned(),
    /// #         version: "1.0".to_owned(),
    /// #         binding: "rust",
    /// #         provider_class: "builtin-compact".to_owned(),
    /// #         platform: "macos-aarch64".to_owned(),
    /// #         settings_hash: None,
    /// #     },
    /// #     counts: Counts::of(&results),
    /// #     results,
    /// # };
    ///
    /// use teistro_ephemeris_kit::corpus::KNOWN;
    ///
    /// assert!(report.against(&KNOWN).holds());
    /// // Held to no divergences, the Moon's miss is unexplained.
    /// let bare = report.against(&[]);
    /// assert!(!bare.holds());
    /// assert_eq!(bare.unexplained, ["c046-kathmandu-2350-01-01: positions.bodies.MOON.sidereal_longitude_deg"]);
    /// ```
    #[must_use]
    pub fn holds(&self) -> bool {
        self.unexplained.is_empty() && self.idle.is_empty()
    }
}

/// A recorded birth: its instant, place and zone.
pub(crate) fn birth(chart: &Value) -> Result<(JulianDay<Utc>, Place, UtcOffset), String> {
    let number = |pointer: &str| {
        chart
            .pointer(pointer)
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("the recorded chart has no `{pointer}`"))
    };
    let latitude =
        Latitude::try_new(number("/input/place/latitude")?).map_err(|why| why.to_string())?;
    let longitude =
        Longitude::try_new(number("/input/place/longitude")?).map_err(|why| why.to_string())?;
    let altitude = Altitude::try_new(number("/input/place/altitude_m").unwrap_or(0.0))
        .map_err(|why| why.to_string())?;
    let jd = number("/input/resolved/jd_ut")?;
    let minutes = chart
        .pointer("/input/resolved/tz_offset_min")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let offset = i32::try_from(minutes * 60)
        .ok()
        .and_then(|seconds| UtcOffset::try_from_seconds(seconds).ok())
        .ok_or_else(|| format!("a zone offset of {minutes} minutes"))?;
    Ok((
        JulianDay::<Utc>::literal(jd),
        Place::new(latitude, longitude, altitude),
        offset,
    ))
}

/// The recorded bodies the chart founds, by the key the corpus files them
/// under, and whether each is **placed**: the nodes are directions, which
/// the port gives no distance (`Body::is_placed`), so a distance an engine
/// reports for one — the Moon's mean distance, by one convention — is a
/// convention and not a position, and is not compared.
const GRAHAS: [(&str, Graha, bool); 9] = [
    ("SUN", Graha::Sun, true),
    ("MOON", Graha::Moon, true),
    ("MARS", Graha::Mars, true),
    ("MERCURY", Graha::Mercury, true),
    ("JUPITER", Graha::Jupiter, true),
    ("VENUS", Graha::Venus, true),
    ("SATURN", Graha::Saturn, true),
    ("RAHU", Graha::Rahu, false),
    ("KETU", Graha::Ketu, false),
];

/// One value both the recording and the founded chart carry: its path in
/// the recording, what was recorded, what was founded, and whether it is
/// an angle compared the shorter way round.
pub(crate) type Compared = (String, f64, f64, bool);

/// Every position both the recording and the founded chart carry.
pub(crate) fn positions(recorded: &Value, founded: &ChartFoundation) -> Vec<Compared> {
    let mut out = Vec::new();
    let recorded_at = |key: &str, name: &str| {
        recorded
            .pointer(&format!("/positions/bodies/{key}/{name}"))
            .and_then(Value::as_f64)
    };
    for (key, graha, placed) in GRAHAS {
        let Some(found) = founded.graha(graha) else {
            continue;
        };
        let fields = [
            ("sidereal_longitude_deg", found.longitude_deg, true),
            ("tropical_longitude_deg", found.tropical_deg, true),
            ("latitude_deg", found.latitude_deg, false),
            ("speed_deg_per_day", found.speed_deg_per_day, false),
            ("distance_au", found.distance_au, false),
        ];
        for (name, value, angle) in fields {
            if name == "distance_au" && !placed {
                continue;
            }
            if let Some(expected) = recorded_at(key, name) {
                out.push((
                    format!("positions.bodies.{key}.{name}"),
                    expected,
                    value,
                    angle,
                ));
            }
        }
    }
    if let Some(expected) = recorded_at("LAGNA", "sidereal_longitude_deg") {
        out.push((
            String::from("positions.bodies.LAGNA.sidereal_longitude_deg"),
            expected,
            founded.lagna_deg,
            true,
        ));
    }
    out
}

/// One recorded chart against the provider, on the values `read` finds in
/// both.
pub(crate) fn compare(
    sdk: &Context,
    tolerances: &Tolerances,
    class: &str,
    (name, chart): (&str, &Value),
    read: fn(&Value, &ChartFoundation) -> Vec<Compared>,
) -> FixtureResult {
    let skipped = |reason: String| FixtureResult {
        fixture: name.to_owned(),
        outcome: Outcome::Skip,
        reason: Some(reason),
        fields: Vec::new(),
    };
    let (instant, place, offset) = match birth(chart) {
        Ok(birth) => birth,
        Err(why) => return skipped(why),
    };
    let founded = match sdk.chart().found(instant, &place, offset, ChartKind::Natal) {
        Ok(sealed) => sealed.value,
        Err(why) => return skipped(format!("the SDK could not found it: {why}")),
    };
    let fields: Vec<Field> = read(chart, &founded)
        .into_iter()
        .map(|(path, expected, found, angle)| {
            let difference = if angle {
                difference_deg(found, expected).abs()
            } else {
                (found - expected).abs()
            };
            // An element of a recorded array is held to its array's band.
            let banded = path
                .split_once('[')
                .map_or(path.as_str(), |(array, _)| array);
            let tolerance = tolerances.band(banded, class);
            Field {
                within: difference <= tolerance,
                path,
                expected,
                found,
                difference,
                tolerance,
            }
        })
        .collect();
    let missed: Vec<&str> = fields
        .iter()
        .filter(|field| !field.within)
        .map(|field| field.path.as_str())
        .collect();
    FixtureResult {
        fixture: name.to_owned(),
        outcome: if missed.is_empty() {
            Outcome::Pass
        } else {
            Outcome::Fail
        },
        reason: (!missed.is_empty()).then(|| format!("outside its band: {}", missed.join(", "))),
        fields,
    }
}

/// Runs the corpus check against a provider of a class.
///
/// ```
/// # use teistro_ephemeris_kit::corpus::Corpus;
/// #
/// # const TOLERANCES: &str = r#"{
/// #     "provider_classes": {"builtin-compact": "the compact tier", "same-ephemeris": "the recording engine's own"},
/// #     "fields": {
/// #         "positions.bodies.*.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.01, "same-ephemeris": 1e-6},
/// #         "positions.bodies.MOON.sidereal_longitude_deg": {"unit": "deg", "builtin-compact": 0.02, "same-ephemeris": 1e-6}
/// #     }
/// # }"#;
/// # // A corpus of no recorded charts, written where the example can.
/// # let dir = std::env::temp_dir().join(format!("teistro-kit-doc-{}-run", std::process::id()));
/// # std::fs::create_dir_all(dir.join("baseline/charts"))?;
/// # std::fs::write(dir.join("corpus.json"), r#"{"version": "0.0.1"}"#)?;
/// # std::fs::write(dir.join("tolerances.json"), TOLERANCES)?;
/// # let corpus = Corpus::open(&dir)?;
///
/// use teistro_ephemeris_kit::corpus;
/// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
///
/// // No recorded charts: an empty report, founded under the corpus's profile.
/// let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
/// let report = corpus::run(&corpus, &open, "builtin-compact")?;
/// assert!(report.results.is_empty() && report.implementation.settings_hash.is_some());
/// assert!(corpus::run(&corpus, &open, "acme").is_err());
/// # std::fs::remove_dir_all(&dir)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// A class the corpus does not name, recorded charts that cannot be read,
/// or a context the SDK cannot build under the corpus's profile.
pub fn run(corpus: &Corpus, open: Open<'_>, class: &str) -> Result<CorpusReport, String> {
    corpus.tolerances.class(class)?;
    let provider = open();
    let identity = provider.capabilities().identity;
    let sdk = Context::builder()
        .profile(PROFILE)
        .ephemeris([Ephemeris::Provider(provider)])
        .build()
        .map_err(|why| format!("the `{PROFILE}` context: {why}"))?;
    let results: Vec<FixtureResult> = corpus
        .charts()?
        .into_iter()
        .map(|(name, chart)| compare(&sdk, &corpus.tolerances, class, (&name, &chart), positions))
        .collect();
    Ok(corpus.report(
        &identity,
        class,
        Some(sdk.settings_hash().to_string()),
        results,
    ))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::{GRAHAS, Tolerances, matches};
    use teistro_port_ephemeris::Body;

    /// The bodies whose distance is not compared are the port's
    /// directions, and nothing else.
    #[test]
    fn a_distance_is_compared_for_exactly_the_placed_bodies() {
        for (key, _, placed) in GRAHAS {
            let body = match key {
                "RAHU" | "KETU" => Body::MeanNode,
                "SUN" => Body::Sun,
                "MOON" => Body::Moon,
                "MARS" => Body::Mars,
                "MERCURY" => Body::Mercury,
                "JUPITER" => Body::Jupiter,
                "VENUS" => Body::Venus,
                _ => Body::Saturn,
            };
            assert_eq!(placed, body.is_placed(), "{key}");
        }
        assert!(!Body::TrueNode.is_placed());
    }

    #[test]
    fn a_pattern_matches_segment_by_segment() {
        assert!(matches(
            "positions.bodies.*.sidereal_longitude_deg",
            "positions.bodies.MOON.sidereal_longitude_deg"
        ));
        assert!(!matches(
            "positions.bodies.*.sidereal_longitude_deg",
            "positions.bodies.MOON.extra.sidereal_longitude_deg"
        ));
        assert!(matches(
            "foundation.*_day.*_jd",
            "foundation.panchanga_day.sunrise_jd"
        ));
        assert!(matches(
            "houses.bhava_chalit.bhava_*",
            "houses.bhava_chalit.bhava_12"
        ));
        assert!(!matches("houses.*.mc", "houses.whole.ascendant"));
    }

    #[test]
    fn a_band_is_the_most_specific_patterns_and_an_unlisted_field_is_exact() {
        let tolerances = Tolerances::from_json(
            r#"{"provider_classes": {"a": "", "b": ""},
                "fields": {
                    "x.*.y": {"unit": "deg", "a": 1.0, "b": 2.0},
                    "x.MOON.y": {"unit": "deg", "a": 0.5, "b": 3.0}
                }}"#,
        )
        .unwrap();
        assert!((tolerances.band("x.SUN.y", "a") - 1.0).abs() < f64::EPSILON);
        assert!((tolerances.band("x.MOON.y", "a") - 0.5).abs() < f64::EPSILON);
        assert!(tolerances.band("x.SUN.z", "a").abs() < f64::EPSILON);
        assert!(tolerances.class("a").is_ok());
        let unknown = tolerances.class("builtin-tiny").unwrap_err();
        assert!(unknown.contains("a, b"), "{unknown}");
    }
}
