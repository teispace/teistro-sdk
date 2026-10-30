//! The corpus and JPL checks over tiny corpora written in the test: a
//! manifest, a tolerance file, a few recorded charts and a states file of
//! ten targets, each broken in turn. The runs are judged, reported and
//! written as a real corpus's are, and each malformed part is refused
//! naming what is wrong.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they wrote"
)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{Value, json};
use teistro_ephemeris_kit::corpus::{
    self, Corpus, CorpusId, CorpusReport, Counts, Divergence, FixtureResult, Implementation,
    Outcome,
};
use teistro_ephemeris_kit::jpl;
use teistro_ephemeris_kit::runner;
use teistro_port_ephemeris::{
    Body, Capabilities, Cell, CellStatus, EphemerisProvider, PositionColumns, PositionRequest,
    ProviderError, TestProvider,
};

/// The astronomical unit in kilometres, as the kit converts.
const AU_KM: f64 = 149_597_870.7;

/// The ten JPL targets, in the order the kit asks for them.
const TARGETS: [(&str, Body); 10] = [
    ("10", Body::Sun),
    ("301", Body::Moon),
    ("199", Body::Mercury),
    ("299", Body::Venus),
    ("4", Body::Mars),
    ("5", Body::Jupiter),
    ("6", Body::Saturn),
    ("7", Body::Uranus),
    ("8", Body::Neptune),
    ("9", Body::Pluto),
];

/// The instants recorded: one passed, one Pluto is refused at, one the
/// Moon is a degree off at, and one past the provider's range.
const INSTANTS: [f64; 4] = [2_451_545.0, 2_451_700.0, 2_451_800.0, 2_460_000.0];

const TOLERANCES: &str = r#"{
    "provider_classes": {"wide": "anything near", "exact": "nothing but the recording"},
    "fields": {
        "positions.bodies.*.sidereal_longitude_deg": {"unit": "deg", "wide": 400.0},
        "positions.bodies.*.tropical_longitude_deg": {"unit": "deg", "wide": 400.0},
        "positions.bodies.*.latitude_deg": {"unit": "deg", "wide": 400.0},
        "positions.bodies.*.speed_deg_per_day": {"unit": "deg/day", "wide": 400.0},
        "positions.bodies.*.distance_au": {"unit": "au", "wide": 400.0},
        "jpl.geometric.*.direction_deg": {"unit": "deg", "wide": 1e-6},
        "jpl.geometric.outer.*.direction_deg": {"unit": "deg", "wide": 1e-6},
        "jpl.geometric.*.distance_km": {"unit": "km", "wide": 1e-3}
    }
}"#;

/// A fresh directory of the test's own.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("kit-corpus")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("baseline/charts")).unwrap();
    dir
}

/// A corpus of the files given, beside its manifest and tolerances.
fn corpus(name: &str, files: &[(&str, String)]) -> PathBuf {
    let root = scratch(name);
    std::fs::write(root.join("corpus.json"), r#"{"version": "9.9.9"}"#).unwrap();
    std::fs::write(root.join("tolerances.json"), TOLERANCES).unwrap();
    for (path, text) in files {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    root
}

/// Where a body stands, for ever: its longitude, latitude and distance.
fn fixed(body: Body) -> (f64, f64, f64) {
    let at = TARGETS.iter().position(|(_, b)| *b == body).unwrap();
    let index = f64::from(u8::try_from(at).unwrap());
    (30.0 * index + 5.0, index - 4.0, 1.0 + 0.5 * index)
}

/// A provider whose bodies stand still, in the frame it is asked for,
/// which gives no Pluto between JD 2451600 and 2451750.
struct Still;

impl EphemerisProvider for Still {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            jd_range: (2_451_000.0, 2_452_000.0),
            bodies: TARGETS.iter().map(|(_, body)| *body).collect(),
            native_frame: jpl::GEOMETRIC,
            ..TestProvider::new().capabilities()
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let mut columns =
            PositionColumns::new(request.jds.len(), request.bodies.len(), request.frame);
        for (row, jd) in request.jds.iter().enumerate() {
            for (column, body) in request.bodies.iter().enumerate() {
                let (lon, lat, dist) = fixed(*body);
                let cell = if *body == Body::Pluto && (2_451_600.0..2_451_750.0).contains(jd) {
                    Cell {
                        status: CellStatus::UnsupportedBody,
                        ..Cell::EMPTY
                    }
                } else {
                    Cell {
                        lon,
                        lat,
                        dist,
                        status: CellStatus::Ok,
                        ..Cell::EMPTY
                    }
                };
                columns.set_at(row, column, cell);
            }
        }
        Ok(columns)
    }
}

/// A state row for a body, in km, with a velocity and a light time.
fn state(body: Body, moved_deg: f64) -> Vec<f64> {
    let (lon, lat, dist) = fixed(body);
    let (lon, lat) = ((lon + moved_deg).to_radians(), lat.to_radians());
    let km = dist * AU_KM;
    vec![
        km * lat.cos() * lon.cos(),
        km * lat.cos() * lon.sin(),
        km * lat.sin(),
        0.0,
        0.0,
        0.0,
        0.0,
    ]
}

/// The states file, with `change` applied to its JSON.
fn states(change: impl FnOnce(&mut Value)) -> String {
    let series: Vec<Value> = TARGETS
        .iter()
        .map(|(target, body)| {
            let rows: Vec<Vec<f64>> = INSTANTS
                .iter()
                .map(|jd| {
                    let moved = if *body == Body::Moon && (*jd - 2_451_800.0).abs() < 1e-9 {
                        1.0
                    } else {
                        0.0
                    };
                    state(*body, moved)
                })
                .collect();
            json!({"target": target, "kernel": "de440", "epoch": "tdb", "frame": "ECLIPJ2000",
                "correction": "NONE", "rows": rows})
        })
        .collect();
    let instants: Vec<Value> = INSTANTS.iter().map(|jd| json!({"jd": jd})).collect();
    let mut value = json!({
        "schema": "teistro-conformance/jpl-cspice-states/1",
        "instants": instants,
        "series": series
    });
    change(&mut value);
    value.to_string()
}

const STATES: &str = "jpl/cspice/states.json";

/// The divergences the geometric run is judged against: one explaining
/// its miss, one of another class, one that explains nothing.
const MOON: Divergence = Divergence {
    name: "moon-a-degree-off",
    fields: &["jpl.geometric.MOON.direction_deg"],
    fixtures: &["jd-2451800"],
    classes: &["wide"],
    why: "written so",
};
const ELSEWHERE: Divergence = Divergence {
    name: "another-class",
    fields: &["jpl.geometric.SUN.direction_deg"],
    fixtures: &[],
    classes: &["exact"],
    why: "not this run's",
};
const IDLE: Divergence = Divergence {
    name: "explains-nothing",
    fields: &["jpl.geometric.SUN.distance_km"],
    fixtures: &[],
    classes: &[],
    why: "nothing missed it",
};

#[test]
fn a_geometric_run_passes_skips_fails_and_leaves_out_by_instant() {
    let root = corpus("jpl", &[(STATES, states(|_| {}))]);
    let corpus = Corpus::open(&root).unwrap();
    let run = jpl::geometric(&corpus, &Still, "wide").unwrap();
    assert_eq!(run.outside, [2_460_000.0]);
    let report = &run.report;
    let outcomes: Vec<(&str, Outcome)> = report
        .results
        .iter()
        .map(|result| (result.fixture.as_str(), result.outcome))
        .collect();
    assert_eq!(
        outcomes,
        [
            ("jd-2451545", Outcome::Pass),
            ("jd-2451700", Outcome::Skip),
            ("jd-2451800", Outcome::Fail),
        ]
    );
    assert_eq!(
        report.results[1].reason.as_deref(),
        Some("inside its range, the provider gave no PLUTO")
    );
    assert_eq!(
        report.results[2].reason.as_deref(),
        Some("outside its band: jpl.geometric.MOON.direction_deg")
    );
    // The outer planets' directions are filed under their own band.
    let paths: Vec<&str> = report.results[0]
        .fields
        .iter()
        .map(|field| field.path.as_str())
        .collect();
    assert_eq!(paths.len(), 20);
    assert!(paths.contains(&"jpl.geometric.outer.PLUTO.direction_deg"));
    assert!(paths.contains(&"jpl.geometric.PLUTO.distance_km"));
    assert!(paths.contains(&"jpl.geometric.MARS.direction_deg"));
    assert_eq!(
        (
            report.counts.passed,
            report.counts.failed,
            report.counts.skipped
        ),
        (1, 1, 1)
    );
    assert!(!report.passed());
    assert_eq!(report.implementation.provider_class, "wide");
    assert_eq!(report.corpus.version, "9.9.9");

    // Judged: the skip and the miss are unexplained until a divergence
    // names the miss, and one that explains nothing is idle.
    let bare = report.against(&[]);
    assert_eq!(
        bare.unexplained,
        [
            "jd-2451700: skipped, inside its range, the provider gave no PLUTO",
            "jd-2451800: jpl.geometric.MOON.direction_deg",
        ]
    );
    assert!(!bare.holds());
    let judged = report.against(&[MOON, ELSEWHERE, IDLE]);
    assert_eq!(judged.explained, 1);
    assert_eq!(judged.explained_by, [(&MOON, 1)]);
    assert_eq!(judged.idle, [&IDLE]);
    assert_eq!(judged.unexplained.len(), 1);

    // The worst of each kind of field, and the report as text and as a file.
    let worst = report.worst();
    let direction = worst.iter().find(|w| w.field == "direction_deg").unwrap();
    assert_eq!(direction.fixture, "jd-2451800");
    // A degree of longitude at the Moon's latitude here, three degrees
    // south: near enough the cosine of it.
    let expected = 3.0_f64.to_radians().cos();
    assert!(
        (direction.difference - expected).abs() < 1e-6,
        "{}",
        direction.difference
    );
    let text = report.markdown();
    assert!(text.starts_with(
        "corpus 9.9.9 against test-provider 1 as `wide`: 1 passed, 1 failed, 1 skipped\n"
    ));
    assert!(text.contains("\njd-2451700 Skip: inside its range, the provider gave no PLUTO"));
    let out = root.join("out");
    let written = report.write(&out, "geometric").unwrap();
    assert_eq!(written, out.join("geometric.json"));
    let back: Value = serde_json::from_str(&std::fs::read_to_string(written).unwrap()).unwrap();
    assert_eq!(back["counts"]["failed"], 1);
}

#[test]
fn an_exact_class_holds_every_field_to_nothing() {
    let root = corpus("jpl-exact", &[(STATES, states(|_| {}))]);
    let corpus = Corpus::open(&root).unwrap();
    let run = jpl::geometric(&corpus, &Still, "exact").unwrap();
    assert!(
        run.report
            .results
            .iter()
            .all(|r| r.fields.iter().all(|f| f.tolerance == 0.0))
    );
    assert!(run.report.markdown().contains("| exact |"));
}

#[test]
fn a_states_file_is_refused_for_each_way_it_is_malformed() {
    let refused =
        |name: &str, text: Option<String>, class: &str, provider: &dyn EphemerisProvider| {
            let files: Vec<(&str, String)> = text.into_iter().map(|text| (STATES, text)).collect();
            let root = corpus(name, &files);
            jpl::geometric(&Corpus::open(&root).unwrap(), provider, class).unwrap_err()
        };
    let cases: [(&str, Option<String>, &str); 9] = [
        ("absent", None, "the corpus is older than 0.12.0"),
        (
            "not-json",
            Some(String::from("{")),
            "jpl/cspice/states.json: ",
        ),
        (
            "schema",
            Some(states(|v| {
                v["schema"] = json!("teistro-conformance/other/1");
            })),
            "is `teistro-conformance/other/1`, and this reads",
        ),
        (
            "kernel",
            Some(states(|v| v["series"][0]["kernel"] = json!("de430"))),
            "has no geometric ECLIPJ2000 series for 10",
        ),
        (
            "epoch",
            Some(states(|v| v["series"][0]["epoch"] = json!("utc"))),
            "has no geometric ECLIPJ2000 series for 10",
        ),
        (
            "frame",
            Some(states(|v| v["series"][0]["frame"] = json!("J2000"))),
            "has no geometric ECLIPJ2000 series for 10",
        ),
        (
            "rows",
            Some(states(|v| {
                v["series"][1]["rows"].as_array_mut().unwrap().pop();
            })),
            "target 301 has 3 rows for 4 instants",
        ),
        (
            "short",
            Some(states(|v| v["series"][2]["rows"][0] = json!([1.0, 2.0]))),
            "a row of target 199 has no position",
        ),
        (
            "class",
            Some(states(|_| {})),
            "`acme` is not a provider class",
        ),
    ];
    for (name, text, saying) in cases {
        let class = if name == "class" { "acme" } else { "wide" };
        let why = refused(name, text, class, &Still);
        assert!(why.contains(saying), "{name}: {why}");
    }
    // A provider that answers only its canonical frame refuses the request.
    let why = refused(
        "canonical",
        Some(states(|_| {})),
        "wide",
        &TestProvider::new(),
    );
    assert!(
        why.starts_with("test-provider refused the geometric request"),
        "{why}"
    );
}

// ── The recorded charts ──────────────────────────────────────────────────

fn chart(latitude: &Value, offset: i64, jd: f64) -> String {
    json!({
        "input": {
            "place": {"latitude": latitude, "longitude": 85.3, "altitude_m": 1300.0},
            "resolved": {"jd_ut": jd, "tz_offset_min": offset}
        },
        "positions": {"bodies": {
            "SUN": {"sidereal_longitude_deg": 256.0, "tropical_longitude_deg": 280.0,
                "latitude_deg": 0.0, "speed_deg_per_day": 1.0, "distance_au": 0.98},
            "RAHU": {"sidereal_longitude_deg": 100.0, "distance_au": 1.0},
            "LAGNA": {"sidereal_longitude_deg": 10.0}
        }}
    })
    .to_string()
}

fn open() -> Box<dyn EphemerisProvider> {
    Box::new(TestProvider::new())
}

#[test]
fn each_recorded_chart_is_founded_compared_or_skipped_for_its_reason() {
    let root = corpus(
        "charts",
        &[
            (
                "baseline/charts/a-kathmandu.json",
                chart(&json!(27.7), 345, 2_451_545.0),
            ),
            (
                "baseline/charts/b-no-latitude.json",
                chart(&Value::Null, 0, 2_451_545.0),
            ),
            (
                "baseline/charts/c-past-the-pole.json",
                chart(&json!(95.0), 0, 2_451_545.0),
            ),
            (
                "baseline/charts/d-zone.json",
                chart(&json!(27.7), 100_000, 2_451_545.0),
            ),
            (
                "baseline/charts/e-out-of-range.json",
                chart(&json!(27.7), 0, 100.0),
            ),
            ("baseline/charts/notes.txt", String::from("not a chart")),
        ],
    );
    let corpus = Corpus::open(&root).unwrap();
    let report = corpus::run(&corpus, &open, "wide").unwrap();
    let fixtures: Vec<&str> = report.results.iter().map(|r| r.fixture.as_str()).collect();
    assert_eq!(
        fixtures,
        [
            "a-kathmandu",
            "b-no-latitude",
            "c-past-the-pole",
            "d-zone",
            "e-out-of-range"
        ]
    );
    let founded = &report.results[0];
    assert_eq!(founded.outcome, Outcome::Pass, "{:?}", founded.reason);
    let paths: Vec<&str> = founded.fields.iter().map(|f| f.path.as_str()).collect();
    // A node is a direction: its distance is a convention, not compared.
    assert!(paths.contains(&"positions.bodies.RAHU.sidereal_longitude_deg"));
    assert!(!paths.contains(&"positions.bodies.RAHU.distance_au"));
    assert!(paths.contains(&"positions.bodies.SUN.distance_au"));
    assert!(paths.contains(&"positions.bodies.LAGNA.sidereal_longitude_deg"));
    let reason = |at: usize| report.results[at].reason.clone().unwrap_or_default();
    assert_eq!(
        reason(1),
        "the recorded chart has no `/input/place/latitude`"
    );
    assert_ne!(reason(2), "");
    assert_eq!(reason(3), "a zone offset of 100000 minutes");
    assert!(
        reason(4).starts_with("the SDK could not found it"),
        "{}",
        reason(4)
    );
    assert!(
        report.results[1..]
            .iter()
            .all(|r| r.outcome == Outcome::Skip)
    );

    // Held to nothing, the founded chart misses every field it compares.
    let exact = corpus::run(&corpus, &open, "exact").unwrap();
    assert_eq!(exact.results[0].outcome, Outcome::Fail);
    assert!(
        exact.results[0]
            .reason
            .as_deref()
            .is_some_and(|r| r.starts_with("outside its band: positions.bodies.SUN."))
    );

    // The runner reads the same corpus, writes its report and judges it.
    let out = root.join("out");
    let args: Vec<String> = [
        "--corpus",
        root.to_str().unwrap(),
        "--class",
        "wide",
        "--out",
        out.to_str().unwrap(),
    ]
    .map(String::from)
    .to_vec();
    // The skips are unexplained, so the provider's own run is not held,
    // whether or not the report is written.
    assert!(!runner::charts("test-provider", &open, &args));
    assert!(out.join("test-provider-corpus.json").is_file());
    assert!(!runner::charts("test-provider", &open, &args[..4]));
    let mut reduced = args.clone();
    reduced.push(String::from("--native-frame-only"));
    // Reduced to its native frame, the run is reported and not judged.
    assert!(runner::charts("test-provider", &open, &reduced));
    assert!(out.join("test-provider-native-frame-corpus.json").is_file());
    // A report that cannot be written fails the run.
    let blocked = root.join("blocked");
    std::fs::write(&blocked, "a file where the directory would be").unwrap();
    let mut unwritable = args[..4].to_vec();
    unwritable.extend([String::from("--out"), blocked.display().to_string()]);
    assert!(!runner::charts("test-provider", &open, &unwritable));
}

#[test]
fn a_chart_that_does_not_parse_or_no_chart_directory_fails_the_run() {
    let root = corpus(
        "broken",
        &[("baseline/charts/broken.json", String::from("{"))],
    );
    let why = corpus::run(&Corpus::open(&root).unwrap(), &open, "wide").unwrap_err();
    assert!(why.starts_with("broken: "), "{why}");
    std::fs::remove_dir_all(root.join("baseline")).unwrap();
    let why = corpus::run(&Corpus::open(&root).unwrap(), &open, "wide").unwrap_err();
    assert!(why.contains("baseline/charts"), "{why}");
}

#[test]
fn a_corpus_is_opened_with_its_version_tolerances_and_commit() {
    // No manifest, a manifest that does not parse, one with no version,
    // and no tolerances.
    let root = scratch("open");
    let why = Corpus::open(&root).unwrap_err();
    assert!(why.contains("corpus.json") && why.contains("git submodule update --init"));
    std::fs::write(root.join("corpus.json"), "{").unwrap();
    assert!(
        Corpus::open(&root)
            .unwrap_err()
            .starts_with("corpus.json: ")
    );
    std::fs::write(root.join("corpus.json"), "{}").unwrap();
    assert_eq!(
        Corpus::open(&root).unwrap_err(),
        "corpus.json names no version"
    );
    std::fs::write(root.join("corpus.json"), r#"{"version": "1.0.0"}"#).unwrap();
    assert!(Corpus::open(&root).unwrap_err().contains("tolerances.json"));
    std::fs::write(root.join("tolerances.json"), TOLERANCES).unwrap();
    let opened = Corpus::open(&root).unwrap();
    assert_eq!(
        (opened.version.as_str(), opened.commit.as_deref()),
        ("1.0.0", None)
    );

    // A checkout on a branch: the reference is followed.
    let commit = "0123456789abcdef0123456789abcdef01234567";
    let git = root.join(".git");
    std::fs::create_dir_all(git.join("refs/heads")).unwrap();
    std::fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    std::fs::write(git.join("refs/heads/main"), format!("{commit}\n")).unwrap();
    assert_eq!(Corpus::open(&root).unwrap().commit.as_deref(), Some(commit));
    // A reference to nothing, and a head that is no commit, name none.
    std::fs::write(git.join("HEAD"), "ref: refs/heads/gone\n").unwrap();
    assert_eq!(Corpus::open(&root).unwrap().commit, None);
    std::fs::write(git.join("HEAD"), "not-a-commit\n").unwrap();
    assert_eq!(Corpus::open(&root).unwrap().commit, None);

    // A submodule: `.git` is a file naming the git directory, whose head
    // is detached.
    std::fs::remove_dir_all(&git).unwrap();
    let modules = root.join("modules/corpus");
    std::fs::create_dir_all(&modules).unwrap();
    std::fs::write(modules.join("HEAD"), format!("{commit}\n")).unwrap();
    std::fs::write(&git, "gitdir: modules/corpus\n").unwrap();
    assert_eq!(Corpus::open(&root).unwrap().commit.as_deref(), Some(commit));
    std::fs::write(&git, "not a pointer\n").unwrap();
    assert_eq!(Corpus::open(&root).unwrap().commit, None);
}

#[test]
fn a_kit_run_that_cannot_write_its_results_fails() {
    let root = scratch("results");
    let blocked = root.join("blocked");
    std::fs::write(&blocked, "a file where the directory would be").unwrap();
    let provider = TestProvider::new();
    let code = runner::run("test-provider", &provider, Some(&blocked), Vec::new);
    assert_eq!(code, ExitCode::FAILURE);
}

#[test]
fn a_run_that_misses_nothing_passes_and_holds_against_no_divergence() {
    let results = vec![FixtureResult {
        fixture: String::from("c001-kathmandu-2000-01-01"),
        outcome: Outcome::Pass,
        reason: None,
        fields: Vec::new(),
    }];
    let report = CorpusReport {
        schema: "teistro-conformance/report/1",
        corpus: CorpusId {
            version: String::from("9.9.9"),
            commit: None,
        },
        implementation: Implementation {
            name: String::from("teistro over test-provider"),
            version: String::from("1"),
            binding: "rust",
            provider_class: String::from("wide"),
            platform: String::from("test"),
            settings_hash: None,
        },
        counts: Counts::of(&results),
        results,
    };
    assert!(report.passed());
    let judged = report.against(&[]);
    assert!(judged.holds(), "{judged:?}");
    assert_eq!(judged.explained, 0);
}

#[test]
fn a_divergence_explains_its_fields_in_the_fixtures_it_names_or_in_any() {
    // Naming no fixture, it explains its field in every one.
    assert!(
        corpus::MOON_TOPOCENTRIC_SPEED
            .explains("any-fixture", "positions.bodies.MOON.speed_deg_per_day")
    );
    // Naming some, only in those.
    let beyond = corpus::KNOWN
        .iter()
        .find(|d| d.name == "delta-t-beyond-the-table")
        .unwrap();
    let field = "positions.bodies.SUN.tropical_longitude_deg";
    assert!(beyond.explains("c046-kathmandu-2350-01-01", field));
    assert!(!beyond.explains("c001-kathmandu-2000-01-01", field));
}
