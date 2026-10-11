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
    /// The container image the row is built and tested inside, on its
    /// runner, where the runner's own system is not the platform's: a
    /// musl row runs in Alpine on the Linux runner of its architecture,
    /// still natively. `None` where the runner is the platform.
    pub(crate) container: Option<&'static str>,
    /// The oldest glibc a build for this platform must load on, which
    /// `package` holds every shipped file to (`floor.rs`). `None` where
    /// the C library is not glibc.
    ///
    /// 2.28 is RHEL 8's and Debian 10's, the oldest still supported in
    /// 2026, and the floor Node 22's own Linux builds and `manylinux_2_28`
    /// wheels already ask of a consumer, so the SDK asks nothing more.
    pub(crate) glibc_floor: Option<(u32, u32)>,
    /// The platform tag of this row's Python wheel (`wheel.rs`): the
    /// oldest system the library loads on, in the words pip matches a
    /// host against. A Linux row's is its glibc floor, as `manylinux`
    /// spells it; a macOS row's is the deployment target `rustc` builds
    /// for by default, which `wheel.rs` reads back out of the library.
    pub(crate) wheel_tag: &'static str,
}

/// Every platform the release matrix builds.
///
/// The order is the order artefacts are listed in and the order the
/// matrix runs in: the two Linux architectures first, because they are
/// the pair Phase 1's determinism criterion compares, then macOS, then
/// Windows, x64 then Arm, then the two musl rows.
pub(crate) const PLATFORMS: [Platform; 8] = [
    Platform {
        triple: "x86_64-unknown-linux-gnu",
        os: "linux",
        cpu: "x64",
        libc: Some("glibc"),
        runner: "ubuntu-latest",
        container: None,
        glibc_floor: Some((2, 28)),
        wheel_tag: "manylinux_2_28_x86_64",
    },
    Platform {
        triple: "aarch64-unknown-linux-gnu",
        os: "linux",
        cpu: "arm64",
        libc: Some("glibc"),
        runner: "ubuntu-24.04-arm",
        container: None,
        glibc_floor: Some((2, 28)),
        wheel_tag: "manylinux_2_28_aarch64",
    },
    Platform {
        triple: "aarch64-apple-darwin",
        os: "darwin",
        cpu: "arm64",
        libc: None,
        runner: "macos-latest",
        container: None,
        glibc_floor: None,
        wheel_tag: "macosx_11_0_arm64",
    },
    Platform {
        triple: "x86_64-apple-darwin",
        os: "darwin",
        cpu: "x64",
        libc: None,
        // Intel macOS. Not `macos-13`: GitHub retired that image, and a
        // retired label does not fail, it queues -- eleven dispatches of
        // the verify matrix on 2026-09-12 never reached a conclusion
        // because this one row waited for a runner that does not exist.
        // `check-lints`'s `runner-matches-the-platform-table` holds the
        // workflows to this field now, because the correction had to be
        // made in three places and one of them was missed.
        runner: "macos-15-intel",
        container: None,
        glibc_floor: None,
        wheel_tag: "macosx_10_12_x86_64",
    },
    Platform {
        triple: "x86_64-pc-windows-msvc",
        os: "win32",
        cpu: "x64",
        libc: None,
        runner: "windows-latest",
        container: None,
        glibc_floor: None,
        wheel_tag: "win_amd64",
    },
    // Windows on Arm, built and run natively on GitHub's Arm image rather
    // than cross-compiled from x64, for the reason every row is native.
    Platform {
        triple: "aarch64-pc-windows-msvc",
        os: "win32",
        cpu: "arm64",
        libc: None,
        runner: "windows-11-arm",
        container: None,
        glibc_floor: None,
        wheel_tag: "win_arm64",
    },
    // musl, built and run in Alpine on the Linux runner of each
    // architecture: the container is the platform, the runner only hosts
    // it, so nothing is cross-compiled. musl has no symbol versions, so
    // there is no floor to read back; the wheel asks for musl 1.2, which
    // every Alpine since 3.13 carries.
    Platform {
        triple: "x86_64-unknown-linux-musl",
        os: "linux",
        cpu: "x64",
        libc: Some("musl"),
        runner: "ubuntu-latest",
        container: Some(ALPINE),
        glibc_floor: None,
        wheel_tag: "musllinux_1_2_x86_64",
    },
    Platform {
        triple: "aarch64-unknown-linux-musl",
        os: "linux",
        cpu: "arm64",
        libc: Some("musl"),
        runner: "ubuntu-24.04-arm",
        container: Some(ALPINE),
        glibc_floor: None,
        wheel_tag: "musllinux_1_2_aarch64",
    },
];

/// The Alpine image the musl rows run in: its Node is 22, the floor every
/// package gate runs on, and its musl is 1.2.
pub(crate) const ALPINE: &str = "alpine:3.21";

impl Platform {
    /// The platform this build is running on.
    ///
    /// Every gate that builds for the host asks this rather than testing
    /// `cfg!` itself, so there is one answer to "what is this machine"
    /// and one place to correct it.
    pub(crate) fn host() -> Self {
        // musl first: a musl Linux is a Linux, so the glibc test below
        // would claim it.
        let triple = if cfg!(all(
            target_os = "linux",
            target_env = "musl",
            target_arch = "x86_64"
        )) {
            "x86_64-unknown-linux-musl"
        } else if cfg!(all(
            target_os = "linux",
            target_env = "musl",
            target_arch = "aarch64"
        )) {
            "aarch64-unknown-linux-musl"
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            "x86_64-unknown-linux-gnu"
        } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            "aarch64-unknown-linux-gnu"
        } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            "aarch64-apple-darwin"
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            "x86_64-apple-darwin"
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            "x86_64-pc-windows-msvc"
        } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
            "aarch64-pc-windows-msvc"
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
                container: None,
                glibc_floor: None,
                wheel_tag: "unknown",
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

    /// The file name Cargo gives a program built from `name`: `name.exe`
    /// on Windows.
    pub(crate) fn program(&self, name: &str) -> String {
        if self.is_windows() {
            format!("{name}.exe")
        } else {
            name.to_string()
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

    /// The directory a Python virtual environment puts its executables
    /// in: `Scripts` on Windows and `bin` everywhere else.
    ///
    /// A platform fact, so it is a row of this table rather than a
    /// `cfg!` in the gate that makes a virtual environment. `check-c`
    /// had been failing in the same job for long enough that nothing had
    /// ever reached the `.venv/bin` this hard-coded, and `pip` is not
    /// there on Windows.
    pub(crate) fn venv_bin(&self) -> &'static str {
        if self.is_windows() { "Scripts" } else { "bin" }
    }

    /// The npm package that carries this platform's addon.
    pub(crate) fn npm_package(&self) -> String {
        format!("{NPM_SCOPE}-{}", self.name())
    }

    /// The npm package that carries this platform's agent server.
    pub(crate) fn npm_mcp_package(&self) -> String {
        format!("{NPM_MCP}-{}", self.name())
    }
}

/// The scoped name of the Node package; the platform packages are this
/// name with the platform appended, which is what the loader resolves.
pub(crate) const NPM_SCOPE: &str = "@teistro/sdk";

/// The scoped name of the agent server's npm launcher, which a host runs
/// as `npx -y @teistro/mcp`; its platform packages are this name with the
/// platform appended, which is what the launcher resolves.
pub(crate) const NPM_MCP: &str = "@teistro/mcp";

/// The scoped name of the wasm package: the same layer over the wasm
/// module, one package for every host. It matches the release's
/// `@teistro/sdk-*` glob like a platform package, and is published with
/// them (`03-design/wasm-binding.md`).
pub(crate) const NPM_WASM: &str = "@teistro/sdk-wasm";

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
    fn a_virtual_environment_is_laid_out_by_the_operating_system() {
        let linux = Platform::by_name("linux-x64").expect("a shipped platform");
        assert_eq!(linux.venv_bin(), "bin");
        let windows = Platform::by_name("win32-x64").expect("a shipped platform");
        assert_eq!(windows.venv_bin(), "Scripts");
    }

    #[test]
    fn a_linux_wheel_is_tagged_with_its_glibc_floor() {
        for platform in PLATFORMS {
            if let Some((major, minor)) = platform.glibc_floor {
                let cpu = platform.triple.split('-').next().unwrap_or_default();
                assert_eq!(
                    platform.wheel_tag,
                    format!("manylinux_{major}_{minor}_{cpu}"),
                    "{}",
                    platform.name()
                );
            }
        }
    }

    #[test]
    fn a_musl_row_is_named_and_tagged_for_musl() {
        for platform in PLATFORMS.iter().filter(|p| p.libc == Some("musl")) {
            let cpu = platform.triple.split('-').next().unwrap_or_default();
            assert_eq!(platform.wheel_tag, format!("musllinux_1_2_{cpu}"));
            assert!(platform.name().ends_with("-musl"), "{}", platform.name());
            assert!(platform.container.is_some(), "{}", platform.name());
            assert_eq!(platform.glibc_floor, None, "{}", platform.name());
        }
    }

    #[test]
    fn platform_packages_are_scoped() {
        let mac = Platform::by_name("darwin-arm64").expect("a shipped platform");
        assert_eq!(mac.npm_package(), "@teistro/sdk-darwin-arm64");
    }
}
