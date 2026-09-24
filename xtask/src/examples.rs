//! The shared examples: every binding carries the same programs, and each
//! program prints the same lines in every binding.
//!
//! An example is what a reader copies, so it is held to two bars. Each
//! binding's own gate runs its examples against the library the build
//! produced (`check-node`, `check-python`, `check-dart`), which says they
//! work. `check-parity` runs them all and compares what they print, which
//! says they are **one** set of examples and not three: the same names in
//! every binding, and the same output from each, line for line.
//!
//! Line for line and not byte for byte, because a line's ending is the
//! platform's and not the binding's: on Windows Python's `print` writes
//! `\r\n` where Node writes `\n`, and the first verify run of this gate
//! failed every example on win32 with every line alike.
//!
//! The second bar was a sentence until it was measured. On 2026-09-22 four
//! of the eleven printed alike and seven differed — a full catalogue key
//! against a bare one, `False` against `false`, `2200000.0` against
//! `2200000`, an error's class name — and one of the seven differed
//! because the bindings did: a provider written in a binding refused a
//! whole batch for one instant outside its coverage where a native one
//! refused a cell.
//!
//! What a language may print differently is kept out of the output rather
//! than excused: an example asserts its own column type or error class and
//! prints the fact it demonstrates.
//!
//! The Rust façade's examples (`crates/sdk/examples`) are the fourth set.
//! On 2026-09-24 one of its ten printed what the bindings print, and
//! asking why the rest did not found the SDK's own differences again: a
//! Rust provider was asked for instants outside its coverage where a
//! foreign one was not, a Rust grid of positions carried no provenance,
//! the bindings' birth chart bypassed the profile's topocentric Moon and
//! no binding could say which ayanamsha a chart applied. What is left is
//! [`EXCUSED`]: every difference by name, each with the item that removes
//! it, and the list fails both ways — an excuse that no longer excuses
//! anything is stale.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A binding that carries the shared examples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Binding {
    /// `bindings/node/example/*.mjs`.
    Node,
    /// `bindings/python/example/*.py`.
    Python,
    /// `bindings/dart/example/*.dart`.
    Dart,
    /// `crates/sdk/examples/*.rs`, the façade's own.
    Rust,
}

/// A difference the comparison excuses: in `example`, as `binding` prints
/// it against the others — the whole example, where `line` is `None` and
/// one side does not have it, or one line of its output — and the item in
/// `docs/STATUS.md` that removes it.
pub(crate) struct Excused {
    example: &'static str,
    binding: Binding,
    line: Option<usize>,
    reason: &'static str,
}

impl Excused {
    /// The excuse as a report line: what it excuses, and why.
    pub(crate) fn describe(&self) -> String {
        format!(
            "{} in {}{}: {}",
            self.example,
            self.binding.name(),
            self.line
                .map_or_else(String::new, |line| format!(" at line {line}")),
            self.reason
        )
    }
}

/// Every difference [`differences`] excuses. Exhaustive and refused both
/// ways: a difference not listed fails, and so does an entry that no
/// longer excuses one.
pub(crate) const EXCUSED: [Excused; 8] = [
    Excused {
        example: "annual_chart",
        binding: Binding::Rust,
        line: None,
        reason: "the year's chart, its office-bearers, sahams and dashas are composed at the C boundary \
                 and not in the façade, so Rust has no one call to write it with (STATUS 2g)",
    },
    Excused {
        example: "phala",
        binding: Binding::Rust,
        line: None,
        reason: "loading the readings corpus from disk is shown in Rust alone (STATUS 2g)",
    },
    Excused {
        example: "readings",
        binding: Binding::Rust,
        line: None,
        reason: "loading the readings corpus from disk is shown in Rust alone (STATUS 2g)",
    },
    Excused {
        example: "birth_chart",
        binding: Binding::Rust,
        line: Some(2),
        reason: "a zone's source is `iana` in the bindings and `IANA` in Rust (STATUS 2f)",
    },
    Excused {
        example: "birth_chart",
        binding: Binding::Rust,
        line: Some(23),
        reason: "a zone warning is `time-unknown-fallback` in the bindings and `TIME_UNKNOWN_FALLBACK` \
                 in Rust (STATUS 2f)",
    },
    Excused {
        example: "your_own_ephemeris",
        binding: Binding::Rust,
        line: Some(10),
        reason: "a cell's status is `out-of-range` in the bindings and `OUT_OF_RANGE` in Rust (STATUS 2f)",
    },
    Excused {
        example: "your_own_ephemeris",
        binding: Binding::Rust,
        line: Some(20),
        reason: "a status is `unsupported` in the bindings and `UNSUPPORTED` in Rust (STATUS 2f)",
    },
    Excused {
        example: "interpretation",
        binding: Binding::Rust,
        line: Some(465),
        reason: "a binding's refusal names the C argument, `interpret_json.readings`, where the \
                 caller wrote `interpret.readings` (STATUS 2f)",
    },
];

/// What an example needs from the machine to run: the library the build
/// produced and the interpreter Python is run with. Node's examples load
/// the addon from the package and read neither.
pub(crate) struct Runtime {
    /// The SDK's shared library, where `binding::library` builds it.
    library: PathBuf,
    /// The Python interpreter, as the environment names it.
    python: String,
}

impl Runtime {
    /// The library and the interpreter every gate uses.
    pub(crate) fn of(root: &Path) -> Runtime {
        Runtime {
            library: root
                .join("target/release")
                .join(crate::binding::library_artefact()),
            python: crate::binding::python(),
        }
    }
}

/// One example that ran, and what it printed.
pub(crate) struct Ran {
    /// Its file name without the extension, which is what the bindings
    /// share: `birth_chart` is `birth_chart.mjs`, `.py` and `.dart`.
    pub(crate) name: String,
    /// Its standard output.
    pub(crate) output: String,
}

impl Binding {
    /// The four, in the order the gates report them.
    pub(crate) const ALL: [Binding; 4] =
        [Binding::Node, Binding::Python, Binding::Dart, Binding::Rust];

    /// The binding's name, for a report line.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Binding::Node => "Node",
            Binding::Python => "Python",
            Binding::Dart => "Dart",
            Binding::Rust => "Rust",
        }
    }

    /// The package the binding's examples belong to.
    const fn package(self) -> &'static str {
        match self {
            Binding::Node => "bindings/node",
            Binding::Python => "bindings/python",
            Binding::Dart => "bindings/dart",
            Binding::Rust => "crates/sdk",
        }
    }

    /// The directory, relative to the repository.
    pub(crate) fn directory(self) -> String {
        match self {
            Binding::Rust => format!("{}/examples", self.package()),
            _ => format!("{}/example", self.package()),
        }
    }

    /// The one file in the directory that is not an example: Rust's parity
    /// runner, which `check-parity` runs as one of four runners and which
    /// prints `key<TAB>value` rather than anything a reader copies. The
    /// bindings keep theirs beside the package instead.
    const fn runner(self) -> Option<&'static str> {
        match self {
            Binding::Rust => Some("parity"),
            _ => None,
        }
    }

    /// The extension an example has in this binding.
    const fn extension(self) -> &'static str {
        match self {
            Binding::Node => "mjs",
            Binding::Python => "py",
            Binding::Dart => "dart",
            Binding::Rust => "rs",
        }
    }

    /// Every example, in name order; an error when there are none, which
    /// is a directory emptied by mistake rather than a binding without
    /// examples.
    ///
    /// **Every** file there is found, so an example added to the
    /// directory is gated by having been added: the failure a list here
    /// would eventually have is that someone writes an example and forgets
    /// to list it.
    pub(crate) fn examples(self, root: &Path) -> Result<Vec<PathBuf>, ()> {
        let directory = self.directory();
        let mut found: Vec<PathBuf> = std::fs::read_dir(root.join(&directory))
            .map_err(|e| println!("FAIL  {directory}: {e}"))?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|kind| kind == self.extension())
            })
            .filter(|path| {
                self.runner()
                    .is_none_or(|runner| path.file_stem().is_none_or(|stem| stem != runner))
            })
            .collect();
        found.sort();
        if found.is_empty() {
            println!("FAIL  {directory} holds no examples");
            return Err(());
        }
        Ok(found)
    }

    /// The command that runs one example against the library this build
    /// produced, from where the binding expects to be run.
    fn command(self, root: &Path, example: &Path, runtime: &Runtime) -> Command {
        let package = root.join(self.package());
        match self {
            Binding::Node => {
                let mut command = Command::new("node");
                command.arg(example).current_dir(root);
                command
            }
            Binding::Python => {
                let mut command = crate::binding::python_command(&runtime.python);
                command
                    .arg(example)
                    .env("TEISTRO_LIBRARY", &runtime.library)
                    .env("PYTHONPATH", &package)
                    .current_dir(&package);
                command
            }
            Binding::Dart => {
                let mut command = Command::new("dart");
                command
                    .arg("run")
                    .arg(example)
                    .env("TEISTRO_LIBRARY", &runtime.library)
                    .current_dir(&package);
                command
            }
            // Release, because the built-in ephemeris is a truncated VSOP87
            // and ELP2000 and a debug build of it computes a year of the
            // sky slowly enough to notice -- which is also what the README
            // tells a reader to do. Rust composes the crates, so there is
            // no library to load.
            Binding::Rust => {
                let mut command = Command::new(crate::binding::cargo());
                command
                    .args(["run", "--quiet", "--release", "-p", "teistro", "--example"])
                    .arg(example.file_stem().unwrap_or_default())
                    .current_dir(root);
                command
            }
        }
    }

    /// Runs every example and keeps what each printed.
    ///
    /// Each must exit cleanly; one that does not is reported with its
    /// program and its error output, and the run stops there, since an
    /// example that fails is a finding on its own and comparing the rest
    /// would bury it.
    pub(crate) fn run(self, root: &Path, runtime: &Runtime) -> Result<Vec<Ran>, ()> {
        let directory = self.directory();
        let mut ran = Vec::new();
        for example in self.examples(root)? {
            let name = example
                .file_stem()
                .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
            let mut command = self.command(root, &example, runtime);
            let output = command.output().map_err(|e| {
                println!(
                    "FAIL  {directory}/{name} did not start (`{}`): {e}",
                    command.get_program().to_string_lossy()
                );
            })?;
            if !output.status.success() {
                println!(
                    "FAIL  {directory}/{name} did not run (`{}`)\n{}{}",
                    command.get_program().to_string_lossy(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                );
                return Err(());
            }
            ran.push(Ran {
                name,
                output: String::from_utf8_lossy(&output.stdout).into_owned(),
            });
        }
        println!(
            "ok    {directory}: {} run",
            crate::measure::plural(ran.len(), "example")
        );
        Ok(ran)
    }
}

/// How many ways the bindings' examples differ, each printed: an example
/// one binding has and another does not, an example whose output differs,
/// at its first differing line, and an excuse that excused nothing.
///
/// Every set is compared against the first, as `check-parity` compares
/// its reports, so a machine with two of the four toolchains still gates
/// the pair it has. A difference in `excused` is not counted, and an entry
/// there for a binding that ran and was compared, which excused nothing,
/// is: a list of excuses that can go stale is a claim that rots.
pub(crate) fn differences(sets: &[(Binding, Vec<Ran>)], excused: &[Excused]) -> usize {
    let Some(((first, reference), rest)) = sets.split_first() else {
        return 0;
    };
    let mut used = vec![false; excused.len()];
    let mut excuse = |example: &str, bindings: [Binding; 2], line: Option<usize>| {
        let found = excused.iter().position(|entry| {
            entry.example == example && bindings.contains(&entry.binding) && entry.line == line
        });
        if let Some(at) = found
            && let Some(slot) = used.get_mut(at)
        {
            *slot = true;
        }
        found.is_some()
    };
    let mut found = 0;
    for (other, theirs) in rest {
        for (left, right, one, two) in [
            (reference, theirs, first, other),
            (theirs, reference, other, first),
        ] {
            for example in left
                .iter()
                .filter(|a| !right.iter().any(|b| b.name == a.name))
            {
                if excuse(&example.name, [*one, *two], None) {
                    continue;
                }
                found += 1;
                println!(
                    "FAIL  `{}` is a {} example and not a {} one",
                    example.name,
                    one.name(),
                    two.name()
                );
            }
        }
        for mine in reference {
            let Some(yours) = theirs.iter().find(|ran| ran.name == mine.name) else {
                continue;
            };
            let mut left = mine.output.lines();
            let mut right = yours.output.lines();
            let mut line = 1;
            loop {
                match (left.next(), right.next()) {
                    (None, None) => break,
                    (Some(a), Some(b)) if a == b => {}
                    (a, b) => {
                        if !excuse(&mine.name, [*first, *other], Some(line)) {
                            found += 1;
                            println!(
                                "FAIL  `{}` prints differently at line {line}\n      {:6} {}\n      {:6} {}",
                                mine.name,
                                first.name(),
                                a.unwrap_or("(nothing)"),
                                other.name(),
                                b.unwrap_or("(nothing)"),
                            );
                            break;
                        }
                    }
                }
                line += 1;
            }
        }
    }
    let compared: Vec<Binding> = sets.iter().map(|(binding, _)| *binding).collect();
    for (entry, used) in excused.iter().zip(used) {
        if !used && compared.contains(&entry.binding) {
            found += 1;
            println!(
                "FAIL  the excuse for `{}` in {}{} excused nothing; take it off the list",
                entry.example,
                entry.binding.name(),
                entry
                    .line
                    .map_or_else(String::new, |line| format!(" at line {line}")),
            );
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{Binding, Excused, Ran, differences};

    fn ran(name: &str, output: &str) -> Ran {
        Ran {
            name: name.to_owned(),
            output: output.to_owned(),
        }
    }

    /// Alike is line for line, and a missing example is a difference in
    /// either direction.
    #[test]
    fn examples_differ_by_name_or_by_output() {
        let node = vec![ran("a", "x\ny\n"), ran("b", "z\n")];
        let alike = vec![ran("a", "x\ny\n"), ran("b", "z\n")];
        assert_eq!(
            differences(&[(Binding::Node, node), (Binding::Dart, alike)], &[]),
            0
        );

        let node = vec![ran("a", "x\ny\n"), ran("b", "z\n")];
        let python = vec![ran("a", "x\nY\n"), ran("c", "z\n")];
        // `a` differs at line 2, `b` is Node's alone and `c` Python's.
        assert_eq!(
            differences(&[(Binding::Node, node), (Binding::Python, python)], &[]),
            3
        );

        // A trailing line more is a difference, not a prefix that agrees.
        let short = vec![ran("a", "x\n")];
        let long = vec![ran("a", "x\ny\n")];
        assert_eq!(
            differences(&[(Binding::Node, short), (Binding::Dart, long)], &[]),
            1
        );

        // A line's ending is the platform's: Python on Windows ends each
        // with `\r\n` where Node ends it with `\n`.
        let unix = vec![ran("a", "x\ny\n")];
        let windows = vec![ran("a", "x\r\ny\r\n")];
        assert_eq!(
            differences(&[(Binding::Node, unix), (Binding::Python, windows)], &[]),
            0
        );
    }

    /// An excuse excuses exactly what it names, and one that excuses
    /// nothing is itself a difference.
    #[test]
    fn an_excuse_is_used_or_it_fails() {
        let line = |line| Excused {
            example: "a",
            binding: Binding::Rust,
            line: Some(line),
            reason: "a test",
        };
        let node = || vec![ran("a", "x\ny\nz\n")];
        let rust = || vec![ran("a", "x\nY\nz\n"), ran("b", "only\n")];
        let absent = Excused {
            example: "b",
            binding: Binding::Rust,
            line: None,
            reason: "a test",
        };
        let sets = || [(Binding::Node, node()), (Binding::Rust, rust())];
        // Line 2 and the Rust-only example, each excused.
        assert_eq!(differences(&sets(), &[line(2), absent]), 0);
        // Without the line's excuse, the line is a difference.
        let absent = Excused {
            example: "b",
            binding: Binding::Rust,
            line: None,
            reason: "a test",
        };
        assert_eq!(differences(&sets(), &[absent]), 1);
        // An excuse for a line that agrees is stale, and so is one for a
        // binding that ran and has nothing to excuse.
        let absent = Excused {
            example: "b",
            binding: Binding::Rust,
            line: None,
            reason: "a test",
        };
        assert_eq!(differences(&sets(), &[line(2), line(3), absent]), 1);
        // An excuse for a binding this machine did not run is not stale.
        assert_eq!(
            differences(
                &[(Binding::Node, node()), (Binding::Dart, node())],
                &[line(2)]
            ),
            0
        );
    }
}
