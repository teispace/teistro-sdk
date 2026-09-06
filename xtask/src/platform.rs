//! The platforms the SDK ships a build for, as one table.
//!
//! Four things need the same facts and used to spell them out apart: what
//! Cargo calls the target, what Node calls the platform and the
//! architecture, what file name an operating system gives a library, and
//! which runner builds it. A row here is the only place any of that is
//! written, so adding a platform is adding a row.

/// One platform: a Rust target, the names npm knows it by, and the runner
/// that builds it.
///
/// The npm names are Node's own `process.platform` and `process.arch`,
/// because the loader picks a package with them at runtime and npm picks
/// one with them at install time; using anything else would mean a table
/// to translate between the two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Platform {
    /// Cargo's target triple, which names the build.
    pub(crate) triple: &'static str,
    /// Node's `process.platform`: the operating system.
    pub(crate) os: &'static str,
    /// Node's `process.arch`: the architecture.
    pub(crate) cpu: &'static str,
    /// The C library the build links against, which npm matches against
    /// the host so that a glibc build is not installed on musl. `None`
    /// where the operating system has only one.
    pub(crate) libc: Option<&'static str>,
    /// The GitHub runner that builds this platform in the release matrix.
    pub(crate) runner: &'static str,
}

/// Every platform the release matrix builds.
///
/// The order is the order artefacts are listed in and the order the
/// matrix runs in: the two Linux architectures first, because they are
/// the pair Phase 1's determinism criterion compares, then macOS, then
/// Windows. A musl row is the next one to add; the field is already
/// carried so that adding it changes this table and nothing else.
pub(crate) const PLATFORMS: [Platform; 5] = [
    Platform {
        triple: "x86_64-unknown-linux-gnu",
        os: "linux",
        cpu: "x64",
        libc: Some("glibc"),
        runner: "ubuntu-latest",
    },
    Platform {
        triple: "aarch64-unknown-linux-gnu",
        os: "linux",
        cpu: "arm64",
        libc: Some("glibc"),
        runner: "ubuntu-24.04-arm",
    },
    Platform {
        triple: "aarch64-apple-darwin",
        os: "darwin",
        cpu: "arm64",
        libc: None,
        runner: "macos-latest",
    },
    Platform {
        triple: "x86_64-apple-darwin",
        os: "darwin",
        cpu: "x64",
        libc: None,
        runner: "macos-13",
    },
    Platform {
        triple: "x86_64-pc-windows-msvc",
        os: "win32",
        cpu: "x64",
        libc: None,
        runner: "windows-latest",
    },
];

impl Platform {
    /// The platform this build is running on.
    ///
    /// Every gate that builds for the host asks this rather than testing
    /// `cfg!` itself, so there is one answer to "what is this machine"
    /// and one place to correct it.
    pub(crate) fn host() -> Self {
        let triple = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            "x86_64-unknown-linux-gnu"
        } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            "aarch64-unknown-linux-gnu"
        } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            "aarch64-apple-darwin"
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            "x86_64-apple-darwin"
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            "x86_64-pc-windows-msvc"
        } else {
            // A platform the SDK does not ship for still builds and tests;
            // the shape of its file names is the only thing needed, and
            // the two Unix conventions cover every remaining target.
            return Self {
                triple: "unknown",
                os: if cfg!(target_os = "macos") {
                    "darwin"
                } else if cfg!(target_os = "windows") {
                    "win32"
                } else {
                    "linux"
                },
                cpu: "unknown",
                libc: None,
                runner: "none",
            };
        };
        Self::by_triple(triple).unwrap_or_else(|| unreachable!("the triple came from the table"))
    }

    /// The row for a Cargo target triple, when the SDK ships that target.
    pub(crate) fn by_triple(triple: &str) -> Option<Self> {
        PLATFORMS.into_iter().find(|p| p.triple == triple)
    }

    /// The row for an artefact name (`linux-x64`, `darwin-arm64`), when
    /// the SDK ships it.
    pub(crate) fn by_name(name: &str) -> Option<Self> {
        PLATFORMS.into_iter().find(|p| p.name() == name)
    }

    /// What this platform is called in an artefact's name, in a package's
    /// name and in a report: `<os>-<cpu>`, with the C library appended
    /// where an operating system has more than one and this row is not
    /// the usual one.
    pub(crate) fn name(&self) -> String {
        match self.libc {
            Some("musl") => format!("{}-{}-musl", self.os, self.cpu),
            _ => format!("{}-{}", self.os, self.cpu),
        }
    }

    /// Whether this is Windows, which names libraries its own way.
    pub(crate) fn is_windows(&self) -> bool {
        self.os == "win32"
    }

    /// The file name Cargo gives a shared library built from `stem` (a
    /// crate name with underscores, as Cargo writes it).
    pub(crate) fn shared(&self, stem: &str) -> String {
        match self.os {
            "win32" => format!("{stem}.dll"),
            "darwin" => format!("lib{stem}.dylib"),
            _ => format!("lib{stem}.so"),
        }
    }

    /// The file name Cargo gives a static library built from `stem`.
    pub(crate) fn static_library(&self, stem: &str) -> String {
        if self.is_windows() {
            format!("{stem}.lib")
        } else {
            format!("lib{stem}.a")
        }
    }

    /// The import library a Windows consumer links against, which no
    /// other platform has.
    pub(crate) fn import_library(&self, stem: &str) -> Option<String> {
        self.is_windows().then(|| format!("{stem}.dll.lib"))
    }

    /// The npm package that carries this platform's addon.
    pub(crate) fn npm_package(&self) -> String {
        format!("{NPM_SCOPE}-{}", self.name())
    }
}

/// The scoped name of the Node package; the platform packages are this
/// name with the platform appended, which is what the loader resolves.
pub(crate) const NPM_SCOPE: &str = "@teistro/sdk";

#[cfg(test)]
mod tests {
    use super::{PLATFORMS, Platform};

    #[test]
    fn every_platform_is_named_once() {
        let mut names: Vec<String> = PLATFORMS.iter().map(Platform::name).collect();
        names.sort();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "two platforms share a name");
    }

    #[test]
    fn file_names_follow_the_operating_system() {
        let linux = Platform::by_name("linux-x64").expect("a shipped platform");
        assert_eq!(linux.shared("teistro_ffi"), "libteistro_ffi.so");
        assert_eq!(linux.static_library("teistro_ffi"), "libteistro_ffi.a");
        assert_eq!(linux.import_library("teistro_ffi"), None);

        let mac = Platform::by_name("darwin-arm64").expect("a shipped platform");
        assert_eq!(mac.shared("teistro_ffi"), "libteistro_ffi.dylib");

        let windows = Platform::by_name("win32-x64").expect("a shipped platform");
        assert_eq!(windows.shared("teistro_ffi"), "teistro_ffi.dll");
        assert_eq!(windows.static_library("teistro_ffi"), "teistro_ffi.lib");
        assert_eq!(
            windows.import_library("teistro_ffi").as_deref(),
            Some("teistro_ffi.dll.lib")
        );
    }

    #[test]
    fn the_host_names_this_machine() {
        let host = Platform::host();
        assert!(
            host.shared("x").contains('x'),
            "the host still names libraries"
        );
        if host.triple != "unknown" {
            assert_eq!(
                Platform::by_triple(host.triple).map(|p| p.name()),
                Some(host.name()),
                "the host is a row of the table"
            );
        }
    }

    #[test]
    fn platform_packages_are_scoped() {
        let mac = Platform::by_name("darwin-arm64").expect("a shipped platform");
        assert_eq!(mac.npm_package(), "@teistro/sdk-darwin-arm64");
    }
}
