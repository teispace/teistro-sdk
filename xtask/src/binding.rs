//! What the bindings' gates share: running a step and reporting it,
//! building the library each one loads, and writing the blob fixtures
//! their decoders read. `check-c`, `check-node` and `check-dart` differ
//! only in the toolchain they drive, so everything else is here once.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::platform::Platform;

/// The crate that builds the SDK's shared library, as Cargo names its
/// artefacts.
pub(crate) const LIBRARY_STEM: &str = "teistro_ffi";

/// How a C compiler is pointed at the SDK's **shared** library in `dir`.
///
/// On Unix the two arguments a consumer writes, `-L <dir>
/// -lteistro_ffi`; on Windows the import library by path. A shared
/// library resolves its own imports, so neither needs anything else —
/// not `-lm`, not the Windows import libraries, whatever the toolchain
/// reports for the *static* library.
///
/// Windows is not a variation for neatness. `-l<stem>` makes the linker
/// search, and beside `teistro_ffi.dll.lib` in the same directory sits
/// `teistro_ffi.lib` — the **static** library — which GNU ld finds
/// first. That silently turned `check-c` into a static link of Rust's
/// whole standard library, built for the MSVC ABI, driven by the
/// runner's MinGW gcc: `undefined reference to __chkstk`, to
/// `__imp_NtReadFile`, and to `??_7type_info@@6B@`, which lives in the
/// MSVC C++ runtime that MinGW does not have. No set of `-l` flags
/// closes that — the two ABIs do not meet — and chasing it with flags
/// is what three failed attempts at this were. The import library is
/// what a Windows consumer links under either compiler, so it is what
/// the gates link.
pub(crate) fn shared_link(platform: &Platform, dir: &Path) -> Vec<OsString> {
    match platform.import_library(LIBRARY_STEM) {
        Some(import) => vec![dir.join(import).into_os_string()],
        None => vec![
            OsString::from("-L"),
            dir.as_os_str().to_os_string(),
            OsString::from(format!("-l{LIBRARY_STEM}")),
        ],
    }
}

/// Whether a C consumer on this platform can link the **static**
/// library, and what they must link beside it.
///
/// `libteistro_ffi.a` carries Rust's standard library as object code,
/// and an archive records nothing about what it needs, so the caller's
/// link line has to say.
///
/// **`-lm`**, because the astronomy calls `sin`, `atan2` and the rest,
/// and on Linux the maths functions live in a separate `libm` the
/// linker will not pull in by itself. On macOS they are in libSystem
/// and the flag is a harmless no-op — which is exactly why this was
/// missing for two days: every gate that ran locally passed.
///
/// **Refused on Windows.** The Rust target is `x86_64-pc-windows-msvc`,
/// so `teistro_ffi.lib` is MSVC-ABI object code that wants the MSVC C++
/// runtime; the `cc` on the runner, and on most Windows machines
/// without Visual Studio's environment loaded, is MinGW's gcc. Linking
/// the two is not a missing flag, and a gate that pretends otherwise
/// fails with a page of mangled symbols. `cl` links it; `gcc` does not.
///
/// Stated on `bindings/c/README.md`, so the line a consumer is told to
/// run is the line the gates run.
pub(crate) fn static_link(platform: &Platform) -> StaticLink {
    if platform.is_windows() {
        StaticLink::Refused(
            "teistro_ffi.lib is MSVC-ABI object code; link it with `cl`, \
             or link teistro_ffi.dll.lib against the DLL instead",
        )
    } else {
        StaticLink::With(&["-lm"])
    }
}

/// What [`static_link`] answers: the libraries to link beside the static
/// one, or why this platform's C compiler cannot link it at all.
pub(crate) enum StaticLink {
    /// Linkable, with these libraries beside it.
    With(&'static [&'static str]),
    /// Not linkable by this platform's `cc`, for this reason.
    Refused(&'static str),
}

/// The Python interpreter, as the environment names it.
pub(crate) fn python() -> String {
    std::env::var("PYTHON").unwrap_or_else(|_| String::from("python3")) // lint: python-runs-in-utf8-mode the one reader
}

/// A Python command, **in UTF-8 mode**.
///
/// Four gates run Python -- the binding's tests, its examples, the
/// parity runner and the installed package's consumer -- and each of
/// them needs the same two answers: which interpreter, and that it must
/// be in UTF-8 mode. Each used to answer the first for itself and none
/// answered the second.
///
/// `PYTHONUTF8`, because this SDK answers in Devanagari when it is asked
/// to and every one of those programs prints what it returns. The
/// Windows console's default encoding is cp1252, and `print` of a Nepali
/// string raises `UnicodeEncodeError` there before a character reaches
/// the screen -- `'charmap' codec can't encode characters in position
/// 0-5`, those six being `सोमबार`, and later `'\u2609'`, the Sun. PEP
/// 540's UTF-8 mode is the documented answer and becomes Python's
/// default in 3.15; `bindings/python/README.md` tells a Windows consumer
/// the same.
///
/// Found twice, in two gates, because the fix went into one of them.
/// `check-lints`'s `python-runs-in-utf8-mode` now holds the class: no
/// module may build a Python command of its own.
pub(crate) fn python_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program); // lint: python-runs-in-utf8-mode the one builder
    command.env("PYTHONUTF8", "1");
    command
}

/// Cargo, as the environment names it.
pub(crate) fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}

/// A tool as this platform spells it, or `None` when the machine has
/// none.
///
/// **Windows spells some tools `.cmd`.** `npm`, `npx` and `tsc` are
/// batch shims there, and `Command::new` goes through `CreateProcess`,
/// which appends `.exe` and nothing else — so a Windows runner with npm
/// installed answered "no `npm` on this machine", and `check-package`
/// skipped the Node packages on the platform whose packaging is least
/// like the others'. A skip that says the machine lacks a tool it has is
/// worse than a failure.
///
/// A tool that is an `.exe`, or any tool on a Unix, resolves on the
/// first candidate, so this costs one spawn where nothing is wrong.
pub(crate) fn tool(name: &str, version: &str) -> Option<String> {
    [name.to_owned(), format!("{name}.cmd")]
        .into_iter()
        .find(|candidate| Command::new(candidate).arg(version).output().is_ok())
}

/// Whether a tool is on this machine, which decides whether a gate runs
/// or says why it is skipping (ADR-0014: the fast check stays Rust-only).
///
/// The same question as [`tool`] without the answer's spelling, for a
/// gate that only has to decide whether to run.
pub(crate) fn present(tool_name: &str, version: &str) -> bool {
    tool(tool_name, version).is_some()
}

/// Runs a step and reports it: `Ok(())` when it passed, `Err(())` when it
/// did not, with the line the gate prints either way.
pub(crate) fn step(command: &mut Command, passed: &str, failed: &str) -> Result<(), ()> {
    let status = command.status();
    if status.is_ok_and(|s| s.success()) {
        if !passed.is_empty() {
            println!("ok    {passed}");
        }
        Ok(())
    } else {
        println!("FAIL  {failed}");
        Err(())
    }
}

/// The file name this platform gives the SDK's shared library.
pub(crate) fn library_artefact() -> String {
    Platform::host().shared(LIBRARY_STEM)
}

/// Builds a crate in release, quietly.
pub(crate) fn build(root: &Path, package: &str, what: &str) -> Result<(), ()> {
    step(
        Command::new(cargo())
            .args(["build", "--quiet", "--release", "-p", package])
            .current_dir(root),
        "",
        &format!("{what} did not build"),
    )
}

/// The shared library, built and its path returned.
pub(crate) fn library(root: &Path) -> Result<PathBuf, ()> {
    build(root, "teistro-ffi", "the library")?;
    Ok(root.join("target/release").join(library_artefact()))
}

/// Writes the result blobs a binding's decoders are tested against.
pub(crate) fn blob_fixtures(root: &Path, into: &Path) -> Result<(), ()> {
    step(
        Command::new(cargo())
            .args([
                "run",
                "--quiet",
                "-p",
                "teistro-ffi",
                "--example",
                "blob_fixtures",
                "--",
            ])
            .arg(into)
            .current_dir(root),
        "",
        "the blob fixtures did not build",
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{LIBRARY_STEM, StaticLink, shared_link, static_link};
    use crate::platform::PLATFORMS;

    /// The defect this pair of functions exists to stop: a link line
    /// that means the shared library and names the static one. On
    /// Windows the two live in the same directory under names a
    /// searching linker cannot tell apart by intent, so the check is
    /// that no shared link line mentions the static library's file name
    /// anywhere in it.
    #[test]
    fn a_shared_link_never_names_the_static_library() {
        for platform in PLATFORMS {
            let line = shared_link(&platform, Path::new("/lib"));
            let joined = line
                .iter()
                .map(|argument| argument.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(
                !joined.contains(&platform.static_library(LIBRARY_STEM)),
                "the shared link line for {} names the static library: {joined}",
                platform.name()
            );
            match platform.import_library(LIBRARY_STEM) {
                // A platform that ships an import library links through
                // it, by path. `-l` would search, and searching is the
                // defect: it finds the static library first.
                Some(import) => assert!(
                    joined.contains(&import),
                    "{} ships {import} and the shared link line does not use it: {joined}",
                    platform.name()
                ),
                None => assert!(
                    joined.contains(&format!("-l{LIBRARY_STEM}")),
                    "the shared link line for {} names no library at all: {joined}",
                    platform.name()
                ),
            }
        }
    }

    /// Windows is the exception and is the only one: the release builds
    /// `*-windows-msvc` and the gate's `cc` is MinGW's gcc, so the
    /// archive cannot be linked there. Every other platform's `cc` and
    /// Rust target share an ABI, and a refusal there would be a gate
    /// quietly proving less than it says.
    #[test]
    fn only_windows_refuses_the_static_link() {
        for platform in PLATFORMS {
            match static_link(&platform) {
                StaticLink::With(beside) => {
                    assert!(
                        !platform.is_windows(),
                        "Windows cannot link an MSVC archive with MinGW's gcc"
                    );
                    assert!(
                        beside.contains(&"-lm"),
                        "{} links an archive of the astronomy without libm",
                        platform.name()
                    );
                }
                StaticLink::Refused(why) => {
                    assert!(
                        platform.is_windows(),
                        "{} refuses the static link, and only Windows should: {why}",
                        platform.name()
                    );
                    assert!(!why.is_empty(), "a refusal with no reason");
                }
            }
        }
    }
}
