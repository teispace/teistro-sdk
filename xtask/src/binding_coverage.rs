//! Each binding's own layer under its own coverage tool
//! (`03-design/binding-coverage.md`): the reports `check-node`,
//! `check-python`, `check-dart` and `check-java` write when
//! [`VARIABLE`] names a directory, read into the crates' record.
//!
//! Three tools write LCOV, whose `SF:` opens a file and whose `LF`/`LH`
//! and `BRF`/`BRH` count its lines and branches; JaCoCo writes XML whose
//! report-level `LINE` and `BRANCH` counters are its last of each type.
//! Each report sits beside a `.tool` file naming the tool and its version
//! that counted it, since each counts its own way.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::coverage::Counts;

/// The variable a binding check reads: a directory to write its report
/// into, or unset to run as before.
pub(crate) const VARIABLE: &str = "TEISTRO_BINDING_COVERAGE";

/// The directory a run asked reports in, absolute under `root`, created;
/// none when [`VARIABLE`] is unset or empty.
///
/// # Errors
///
/// The directory cannot be created.
pub(crate) fn asked(root: &Path) -> Result<Option<PathBuf>, String> {
    let Some(dir) = std::env::var_os(VARIABLE).filter(|dir| !dir.is_empty()) else {
        return Ok(None);
    };
    let dir = root.join(dir);
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Some(dir))
}

/// Names the tool that counted `binding`'s report in `dir`: `name` and
/// the first line `version` prints.
///
/// # Errors
///
/// The tool does not answer, or the file cannot be written.
pub(crate) fn name_tool(
    dir: &Path,
    binding: &str,
    name: &str,
    version: &mut Command,
) -> Result<(), String> {
    let out = version
        .output()
        .map_err(|e| format!("`{name}` did not say its version: {e}"))?;
    let said = String::from_utf8_lossy(if out.stdout.is_empty() {
        &out.stderr
    } else {
        &out.stdout
    })
    .lines()
    .next()
    .unwrap_or_default()
    .trim()
    .to_owned();
    if !out.status.success() || said.is_empty() {
        return Err(format!("`{name}` did not say its version"));
    }
    let file = dir.join(format!("{binding}.tool"));
    fs::write(&file, format!("{name} {said}\n")).map_err(|e| format!("{}: {e}", file.display()))
}

/// JaCoCo's pin: its version and each jar's SHA-256 and download.
const JACOCO: &str = "xtask/jacoco.txt";
/// Where the pinned jars are kept once fetched.
const JACOCO_JARS: &str = "target/jacoco";

/// The pinned JaCoCo, fetched and verified, and the directory its report
/// goes to.
pub(crate) struct Jacoco {
    agent: PathBuf,
    cli: PathBuf,
    dir: PathBuf,
}

impl Jacoco {
    /// The pin's jars, fetched once into [`JACOCO_JARS`] and held to their
    /// SHA-256 each time; the tool named in `dir`.
    ///
    /// # Errors
    ///
    /// The pin unreadable or incomplete, a download failing, or a jar
    /// that is not the one pinned.
    pub(crate) fn fetched(root: &Path, dir: &Path) -> Result<Jacoco, String> {
        let pin = fs::read_to_string(root.join(JACOCO)).map_err(|e| format!("{JACOCO}: {e}"))?;
        let (version, jars) = pinned(&pin)?;
        let store = root.join(JACOCO_JARS);
        fs::create_dir_all(&store).map_err(|e| format!("{}: {e}", store.display()))?;
        let mut fetch = |role: &str| -> Result<PathBuf, String> {
            let (sha256, url) = jars
                .get(role)
                .ok_or(format!("{JACOCO} pins no `{role}` jar"))?;
            let file = store.join(url.rsplit('/').next().unwrap_or(role));
            if !file.is_file() {
                let fetched = Command::new("curl")
                    .args([
                        "--fail",
                        "--silent",
                        "--show-error",
                        "--location",
                        "--output",
                    ])
                    .arg(&file)
                    .arg(url)
                    .status()
                    .is_ok_and(|status| status.success());
                if !fetched {
                    let _ = fs::remove_file(&file);
                    return Err(format!("{url} could not be fetched"));
                }
            }
            let bytes = fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let got = crate::hashes::hex(&<sha2::Sha256 as sha2::Digest>::digest(&bytes));
            if &got != sha256 {
                let _ = fs::remove_file(&file);
                return Err(format!(
                    "{} is {got}, not the {sha256} {JACOCO} pins; removed",
                    file.display()
                ));
            }
            Ok(file)
        };
        let (agent, cli) = (fetch("agent")?, fetch("cli")?);
        let tool = dir.join("java.tool");
        fs::write(&tool, format!("jacoco {version}\n"))
            .map_err(|e| format!("{}: {e}", tool.display()))?;
        Ok(Jacoco {
            agent,
            cli,
            dir: dir.to_path_buf(),
        })
    }

    /// The JVM option that runs the agent, writing its execution data.
    pub(crate) fn agent_option(&self) -> String {
        format!(
            "-javaagent:{}=destfile={}",
            self.agent.display(),
            self.dir.join("java.exec").display()
        )
    }

    /// The XML report of the run over the module's own classes, its tests
    /// left out by not being given.
    pub(crate) fn report(&self, root: &Path, classes: &Path) -> Result<(), ()> {
        let java = root.join("bindings/java");
        crate::binding::step(
            crate::binding::java_command()
                .arg("-jar")
                .arg(&self.cli)
                .arg("report")
                .arg(self.dir.join("java.exec"))
                .arg("--classfiles")
                .arg(classes)
                .arg("--sourcefiles")
                .arg(java.join("src"))
                .arg("--sourcefiles")
                .arg(java.join("generated"))
                .arg("--quiet")
                .arg("--xml")
                .arg(self.dir.join("java.xml")),
            "the Java binding's coverage written",
            "the Java binding's coverage was not written",
        )
    }
}

/// The pin's version and each role's SHA-256 and download.
fn pinned(text: &str) -> Result<(String, BTreeMap<String, (String, String)>), String> {
    let mut version = None;
    let mut jars = BTreeMap::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        match fields.as_slice() {
            ["version", v] => version = Some((*v).to_owned()),
            [role, sha, url] if sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()) => {
                jars.insert((*role).to_owned(), ((*sha).to_owned(), (*url).to_owned()));
            }
            _ => {
                return Err(format!(
                    "{JACOCO}: `{line}` is neither a version nor a pinned jar"
                ));
            }
        }
    }
    Ok((version.ok_or(format!("{JACOCO} names no version"))?, jars))
}

/// A report's format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    Lcov,
    Jacoco,
}

/// A binding, its report's file and format, and the fragment of a source
/// path that marks a file of its layer.
struct Binding {
    name: &'static str,
    report: &'static str,
    format: Format,
    layer: &'static str,
}

/// The bindings measured, in the record's order.
const BINDINGS: [Binding; 4] = [
    Binding {
        name: "dart",
        report: "dart.lcov",
        format: Format::Lcov,
        layer: "lib/",
    },
    Binding {
        name: "java",
        report: "java.xml",
        format: Format::Jacoco,
        layer: "",
    },
    Binding {
        name: "node",
        report: "node.lcov",
        format: Format::Lcov,
        layer: "lib/",
    },
    Binding {
        name: "python",
        report: "python.lcov",
        format: Format::Lcov,
        layer: "teistro/",
    },
];

/// A binding's floors, whole percents, and the tool that counted them; a
/// tool that counts no branch has no branch floor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Floors {
    pub(crate) counted_by: String,
    pub(crate) lines: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) branches: Option<u8>,
}

/// What a run measured of a binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Measured {
    pub(crate) counted_by: String,
    pub(crate) lines: Counts,
    pub(crate) branches: Option<Counts>,
}

/// Each binding's report in `dir`, read; a binding without one is absent.
///
/// # Errors
///
/// A report unreadable, in the wrong shape, without its `.tool` file, or
/// counting none of its layer.
pub(crate) fn measured(dir: &Path) -> Result<BTreeMap<String, Measured>, String> {
    let mut run = BTreeMap::new();
    for binding in &BINDINGS {
        let path = dir.join(binding.report);
        if !path.is_file() {
            continue;
        }
        let shown = path.display();
        let text = fs::read_to_string(&path).map_err(|e| format!("{shown}: {e}"))?;
        let tool = path.with_extension("tool");
        let counted_by = fs::read_to_string(&tool)
            .map_err(|e| format!("{}: {e}; each report names its tool", tool.display()))?
            .trim()
            .to_owned();
        let (lines, branches) = match binding.format {
            Format::Lcov => lcov(&text, binding.layer),
            Format::Jacoco => jacoco(&text),
        }
        .map_err(|why| format!("{shown}: {why}"))?;
        if lines.count == 0 {
            return Err(format!(
                "{shown}: no line of the `{}` layer counted",
                binding.name
            ));
        }
        run.insert(
            binding.name.to_owned(),
            Measured {
                counted_by,
                lines,
                branches,
            },
        );
    }
    Ok(run)
}

/// An LCOV report's lines and branches over the files whose path holds
/// `layer` and no test directory; no branch at all is none measured.
fn lcov(text: &str, layer: &str) -> Result<(Counts, Option<Counts>), String> {
    let (mut lines, mut branches, mut any_branch) = (Counts::default(), Counts::default(), false);
    let mut counted = false;
    for line in text.lines().map(str::trim) {
        if let Some(path) = line.strip_prefix("SF:") {
            let path = path.replace('\\', "/");
            counted = path.contains(layer) && !path.contains("/test");
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if !counted || !matches!(key, "LF" | "LH" | "BRF" | "BRH") {
            continue;
        }
        let value: u64 = value
            .parse()
            .map_err(|_| format!("`{line}` is not a count"))?;
        match key {
            "LF" => lines.count += value,
            "LH" => lines.covered += value,
            "BRF" => {
                branches.count += value;
                any_branch = true;
            }
            _ => branches.covered += value,
        }
    }
    if lines.covered > lines.count || branches.covered > branches.count {
        return Err(String::from("more covered than counted"));
    }
    Ok((
        lines,
        (any_branch && branches.count > 0).then_some(branches),
    ))
}

/// A JaCoCo XML report's own `LINE` and `BRANCH` counters: the last of
/// each type, since the report's come after every package's.
fn jacoco(text: &str) -> Result<(Counts, Option<Counts>), String> {
    let counter = |kind: &str| -> Result<Option<Counts>, String> {
        let open = format!("<counter type=\"{kind}\"");
        let Some(at) = text.rfind(&open) else {
            return Ok(None);
        };
        let tag = &text[at..];
        let tag = &tag[..tag.find("/>").ok_or("an unclosed counter")?];
        let read = |name: &str| -> Result<u64, String> {
            let key = format!("{name}=\"");
            let from = tag
                .find(&key)
                .ok_or(format!("a {kind} counter without `{name}`"))?
                + key.len();
            let rest = &tag[from..];
            rest[..rest.find('"').unwrap_or(rest.len())]
                .parse()
                .map_err(|_| format!("a {kind} counter's `{name}` is not a count"))
        };
        let (missed, covered) = (read("missed")?, read("covered")?);
        Ok(Some(Counts {
            count: missed + covered,
            covered,
        }))
    };
    let lines = counter("LINE")?.ok_or("no LINE counter: not a JaCoCo report")?;
    Ok((lines, counter("BRANCH")?))
}

/// The floors a run supports: each raised to the run's whole percent and
/// none lowered, but a binding another tool counted re-based to the run.
pub(crate) fn raised(
    old: &BTreeMap<String, Floors>,
    run: &BTreeMap<String, Measured>,
) -> BTreeMap<String, Floors> {
    let mut floors = old.clone();
    for (name, measured) in run {
        let rebased = Floors {
            counted_by: measured.counted_by.clone(),
            lines: measured.lines.floor(),
            branches: measured.branches.map(Counts::floor),
        };
        let floor = floors
            .entry(name.clone())
            .or_insert_with(|| rebased.clone());
        if floor.counted_by != measured.counted_by {
            *floor = rebased;
            continue;
        }
        floor.lines = floor.lines.max(rebased.lines);
        floor.branches = match (floor.branches, rebased.branches) {
            (Some(old), Some(new)) => Some(old.max(new)),
            (old, new) => old.or(new),
        };
    }
    floors
}

/// What a run says against the floors: a binding under either, a floored
/// binding without a report, a report without a floor, and a report
/// another tool counted.
pub(crate) fn findings(
    floors: &BTreeMap<String, Floors>,
    run: &BTreeMap<String, Measured>,
) -> Vec<String> {
    let mut found = Vec::new();
    for (name, floor) in floors {
        let Some(measured) = run.get(name) else {
            found.push(format!(
                "the `{name}` binding has floors but the run wrote no report for it"
            ));
            continue;
        };
        if measured.counted_by != floor.counted_by {
            found.push(format!(
                "the `{name}` binding was counted by `{}` and its floors by `{}`: record the run \
                 with `cargo xtask coverage --bindings DIR`",
                measured.counted_by, floor.counted_by
            ));
            continue;
        }
        let mut under = |kind: &str, counts: Counts, floor: u8| {
            if counts.percent() < f64::from(floor) {
                found.push(format!(
                    "the `{name}` binding covers {:.2}% of its {kind}, under its floor of \
                     {floor}%: add the tests the change left out",
                    counts.percent()
                ));
            }
        };
        under("lines", measured.lines, floor.lines);
        match (floor.branches, measured.branches) {
            (Some(branch), Some(counts)) => under("branches", counts, branch),
            (Some(branch), None) => found.push(format!(
                "the `{name}` binding has a branch floor of {branch}% but the run counted no branch"
            )),
            (None, _) => {}
        }
    }
    for name in run.keys().filter(|name| !floors.contains_key(*name)) {
        found.push(format!(
            "the `{name}` binding has no floors; record them with `cargo xtask coverage \
             --bindings DIR`"
        ));
    }
    found
}

/// Each binding floor of `now` lower than `main`'s counted by the same
/// tool, as sentences.
pub(crate) fn lowered(
    main: &BTreeMap<String, Floors>,
    now: &BTreeMap<String, Floors>,
) -> Vec<String> {
    let mut found = Vec::new();
    for (name, was) in main {
        let Some(now) = now.get(name).filter(|now| now.counted_by == was.counted_by) else {
            continue;
        };
        if now.lines < was.lines {
            found.push(format!(
                "the `{name}` binding's line floor fell from {}% on main to {}%",
                was.lines, now.lines
            ));
        }
        if let (Some(was), Some(now)) = (was.branches, now.branches)
            && now < was
        {
            found.push(format!(
                "the `{name}` binding's branch floor fell from {was}% on main to {now}%"
            ));
        }
    }
    found
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn counts(count: u64, covered: u64) -> Counts {
        Counts { count, covered }
    }

    fn measured(lines: Counts, branches: Option<Counts>) -> Measured {
        Measured {
            counted_by: "node 26".into(),
            lines,
            branches,
        }
    }

    /// Only the layer's files count, never a test file, and a report with
    /// no branch record measures no branch.
    /// The pin names a version and each jar by its SHA-256.
    #[test]
    fn the_jacoco_pin_reads_its_version_and_jars() {
        let sha = "ab".repeat(32);
        let (version, jars) = pinned(&format!(
            "# x\n\nversion 1.2\nagent {sha} https://h/a.jar\n"
        ))
        .unwrap();
        assert_eq!(version, "1.2");
        assert_eq!(jars["agent"], (sha.clone(), "https://h/a.jar".into()));
        assert!(
            pinned("agent zz https://h/a.jar")
                .unwrap_err()
                .contains("neither")
        );
        assert!(
            pinned(&format!("agent {sha} u"))
                .unwrap_err()
                .contains("no version")
        );
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let (_, shipped) = pinned(&fs::read_to_string(repo.join(JACOCO)).unwrap()).unwrap();
        assert_eq!(shipped.keys().collect::<Vec<_>>(), ["agent", "cli"]);
    }

    #[test]
    fn lcov_counts_the_layer_s_files() {
        let text = "TN:\nSF:/w/bindings/node/lib/index.js\nLF:10\nLH:8\nBRF:4\nBRH:3\nend_of_record\n\
                    SF:/w/bindings/node/test/a.test.mjs\nLF:50\nLH:50\nBRF:2\nBRH:2\nend_of_record\n\
                    SF:C:\\w\\bindings\\node\\lib\\keys.js\nLF:5\nLH:5\nend_of_record\n\
                    SF:/w/bindings/node/bench/x.mjs\nLF:9\nLH:0\nend_of_record\n";
        assert_eq!(
            lcov(text, "lib/").unwrap(),
            (counts(15, 13), Some(counts(4, 3)))
        );
        let lines_only = "SF:/w/lib/a.dart\nDA:1,1\nLF:3\nLH:2\nend_of_record\n";
        assert_eq!(lcov(lines_only, "lib/").unwrap(), (counts(3, 2), None));
        assert!(lcov("SF:/w/lib/a.js\nLF:x\n", "lib/").is_err());
        assert!(lcov("SF:/w/lib/a.js\nLF:1\nLH:2\n", "lib/").is_err());
    }

    /// The report's own counters are the last of each type.
    #[test]
    fn jacoco_reads_the_report_s_own_counters() {
        let text = r#"<report name="x"><package name="p"><class name="c">
            <counter type="LINE" missed="9" covered="1"/></class>
            <counter type="LINE" missed="9" covered="1"/></package>
            <counter type="INSTRUCTION" missed="5" covered="5"/>
            <counter type="BRANCH" missed="2" covered="6"/>
            <counter type="LINE" missed="20" covered="80"/></report>"#;
        assert_eq!(jacoco(text).unwrap(), (counts(100, 80), Some(counts(8, 6))));
        let no_branch = r#"<report><counter type="LINE" missed="0" covered="4"/></report>"#;
        assert_eq!(jacoco(no_branch).unwrap(), (counts(4, 4), None));
        assert!(jacoco("<report/>").is_err());
        assert!(jacoco(r#"<counter type="LINE" missed="a" covered="1"/>"#).is_err());
        assert!(jacoco(r#"<counter type="LINE" covered="1"/>"#).is_err());
        assert!(jacoco(r#"<counter type="LINE" missed="1" covered="1""#).is_err());
    }

    /// A directory's reports are read with their tools; a report without
    /// its tool, or counting nothing of its layer, is refused.
    #[test]
    fn a_directory_s_reports_are_read_with_their_tools() {
        let dir = std::env::temp_dir().join(format!("teistro-bcov-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("node.lcov"),
            "SF:/w/lib/a.js\nLF:4\nLH:3\nend_of_record\n",
        )
        .unwrap();
        assert!(
            super::measured(&dir)
                .unwrap_err()
                .contains("names its tool")
        );
        fs::write(dir.join("node.tool"), "node 26.12.0\n").unwrap();
        let run = super::measured(&dir).unwrap();
        assert_eq!(run.len(), 1);
        assert_eq!(run["node"].counted_by, "node 26.12.0");
        assert_eq!(run["node"].lines, counts(4, 3));
        fs::write(dir.join("python.lcov"), "SF:tests/a.py\nLF:4\nLH:3\n").unwrap();
        fs::write(dir.join("python.tool"), "coverage.py 7\n").unwrap();
        assert!(
            super::measured(&dir)
                .unwrap_err()
                .contains("no line of the `python` layer")
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    /// A floor rises and never falls under one tool; another tool re-bases
    /// it; a tool that starts counting branches gains a branch floor.
    #[test]
    fn a_binding_floor_rises_under_its_tool() {
        let old: BTreeMap<String, Floors> = [
            (
                "node".into(),
                Floors {
                    counted_by: "node 26".into(),
                    lines: 80,
                    branches: Some(70),
                },
            ),
            (
                "dart".into(),
                Floors {
                    counted_by: "dart 3".into(),
                    lines: 60,
                    branches: None,
                },
            ),
            (
                "java".into(),
                Floors {
                    counted_by: "jacoco 0.8.11".into(),
                    lines: 90,
                    branches: Some(90),
                },
            ),
        ]
        .into();
        let mut run: BTreeMap<String, Measured> = [
            (
                "node".into(),
                measured(counts(100, 79), Some(counts(10, 9))),
            ),
            (
                "dart".into(),
                Measured {
                    counted_by: "dart 3".into(),
                    ..measured(counts(10, 7), Some(counts(2, 1)))
                },
            ),
            (
                "java".into(),
                Measured {
                    counted_by: "jacoco 0.8.12".into(),
                    ..measured(counts(10, 5), None)
                },
            ),
        ]
        .into();
        let floors = raised(&old, &run);
        assert_eq!(
            floors["node"],
            Floors {
                counted_by: "node 26".into(),
                lines: 80,
                branches: Some(90)
            }
        );
        assert_eq!(
            floors["dart"],
            Floors {
                counted_by: "dart 3".into(),
                lines: 70,
                branches: Some(50)
            }
        );
        assert_eq!(
            floors["java"],
            Floors {
                counted_by: "jacoco 0.8.12".into(),
                lines: 50,
                branches: None
            }
        );
        run.insert(
            "python".into(),
            Measured {
                counted_by: "coverage.py 7".into(),
                ..measured(counts(3, 2), None)
            },
        );
        assert_eq!(raised(&old, &run)["python"].lines, 66);
        // Java's floor fell only by a new tool's count, which re-bases it.
        assert!(lowered(&old, &floors).is_empty());
        let fell: BTreeMap<String, Floors> = [(
            "node".into(),
            Floors {
                counted_by: "node 26".into(),
                lines: 79,
                branches: Some(60),
            },
        )]
        .into();
        assert_eq!(
            lowered(&old, &fell),
            [
                "the `node` binding's line floor fell from 80% on main to 79%",
                "the `node` binding's branch floor fell from 70% on main to 60%"
            ]
        );
    }

    #[test]
    fn a_run_under_or_beside_its_floors_is_refused() {
        let floors: BTreeMap<String, Floors> = [
            (
                "node".into(),
                Floors {
                    counted_by: "node 26".into(),
                    lines: 80,
                    branches: Some(70),
                },
            ),
            (
                "dart".into(),
                Floors {
                    counted_by: "dart 3".into(),
                    lines: 60,
                    branches: None,
                },
            ),
            (
                "java".into(),
                Floors {
                    counted_by: "jacoco 1".into(),
                    lines: 60,
                    branches: Some(10),
                },
            ),
            (
                "python".into(),
                Floors {
                    counted_by: "coverage.py 7".into(),
                    lines: 60,
                    branches: Some(10),
                },
            ),
        ]
        .into();
        let run: BTreeMap<String, Measured> = [
            (
                "node".into(),
                measured(counts(100, 79), Some(counts(10, 6))),
            ),
            (
                "java".into(),
                Measured {
                    counted_by: "jacoco 2".into(),
                    ..measured(counts(1, 1), None)
                },
            ),
            (
                "python".into(),
                Measured {
                    counted_by: "coverage.py 7".into(),
                    ..measured(counts(1, 1), None)
                },
            ),
            ("wasm".into(), measured(counts(1, 1), None)),
        ]
        .into();
        let found = findings(&floors, &run).join("\n");
        assert!(
            found.contains("`node` binding covers 79.00% of its lines"),
            "{found}"
        );
        assert!(
            found.contains("`node` binding covers 60.00% of its branches"),
            "{found}"
        );
        assert!(
            found.contains("`dart` binding has floors but the run wrote no report"),
            "{found}"
        );
        assert!(
            found.contains("`java` binding was counted by `jacoco 2`"),
            "{found}"
        );
        assert!(
            found.contains(
                "`python` binding has a branch floor of 10% but the run counted no branch"
            ),
            "{found}"
        );
        assert!(found.contains("`wasm` binding has no floors"), "{found}");
        let at: BTreeMap<String, Measured> =
            [("node".into(), measured(counts(10, 8), Some(counts(10, 7))))].into();
        let node: BTreeMap<String, Floors> = floors
            .into_iter()
            .filter(|(name, _)| name == "node")
            .collect();
        assert!(findings(&node, &at).is_empty());
    }
}
