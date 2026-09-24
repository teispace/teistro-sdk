//! The shared examples: every binding carries the same programs, and each
//! program prints the same bytes in every binding.
//!
//! An example is what a reader copies, so it is held to two bars. Each
//! binding's own gate runs its examples against the library the build
//! produced (`check-node`, `check-python`, `check-dart`), which says they
//! work. `check-parity` runs them all and compares what they print, which
//! says they are **one** set of examples and not three: the same names in
//! every binding, and the same output from each, byte for byte.
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
//! than excused here: an example asserts its own column type or error
//! class and prints the fact it demonstrates, so there is no list of
//! permitted differences to maintain. The Rust façade's examples are its
//! own set (`crates/sdk/examples`), printed with Rust's own formatting
//! and run by `check-rust`, and are not compared here.

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
}

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
    /// The three, in the order the gates report them.
    pub(crate) const ALL: [Binding; 3] = [Binding::Node, Binding::Python, Binding::Dart];

    /// The binding's name, for a report line.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Binding::Node => "Node",
            Binding::Python => "Python",
            Binding::Dart => "Dart",
        }
    }

    /// The package the binding's examples belong to.
    const fn package(self) -> &'static str {
        match self {
            Binding::Node => "bindings/node",
            Binding::Python => "bindings/python",
            Binding::Dart => "bindings/dart",
        }
    }

    /// The directory, relative to the repository.
    pub(crate) fn directory(self) -> String {
        format!("{}/example", self.package())
    }

    /// The extension an example has in this binding.
    const fn extension(self) -> &'static str {
        match self {
            Binding::Node => "mjs",
            Binding::Python => "py",
            Binding::Dart => "dart",
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
/// one binding has and another does not, and an example whose output
/// differs, at its first differing line.
///
/// Every set is compared against the first, as `check-parity` compares
/// its reports, so a machine with two of the three toolchains still gates
/// the pair it has.
pub(crate) fn differences(sets: &[(Binding, Vec<Ran>)]) -> usize {
    let Some(((first, reference), rest)) = sets.split_first() else {
        return 0;
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
            if mine.output == yours.output {
                continue;
            }
            found += 1;
            let mut left = mine.output.lines();
            let mut right = yours.output.lines();
            let mut line = 1;
            loop {
                match (left.next(), right.next()) {
                    (Some(a), Some(b)) if a == b => line += 1,
                    (a, b) => {
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
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{Binding, Ran, differences};

    fn ran(name: &str, output: &str) -> Ran {
        Ran {
            name: name.to_owned(),
            output: output.to_owned(),
        }
    }

    /// Alike is byte for byte, and a missing example is a difference in
    /// either direction.
    #[test]
    fn examples_differ_by_name_or_by_output() {
        let node = vec![ran("a", "x\ny\n"), ran("b", "z\n")];
        let alike = vec![ran("a", "x\ny\n"), ran("b", "z\n")];
        assert_eq!(
            differences(&[(Binding::Node, node), (Binding::Dart, alike)]),
            0
        );

        let node = vec![ran("a", "x\ny\n"), ran("b", "z\n")];
        let python = vec![ran("a", "x\nY\n"), ran("c", "z\n")];
        // `a` differs at line 2, `b` is Node's alone and `c` Python's.
        assert_eq!(
            differences(&[(Binding::Node, node), (Binding::Python, python)]),
            3
        );

        // A trailing line more is a difference, not a prefix that agrees.
        let short = vec![ran("a", "x\n")];
        let long = vec![ran("a", "x\ny\n")];
        assert_eq!(
            differences(&[(Binding::Node, short), (Binding::Dart, long)]),
            1
        );
    }
}
