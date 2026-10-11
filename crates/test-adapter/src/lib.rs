//! The analytic [`TestProvider`] as a loadable adapter (ADR-0029).
//!
//! A real engine's adapter is built in its own repository against data
//! a checkout does not have, so without this nothing in CI ever opened a
//! shared library: the loader, `ts_provider_load` and the agent server's
//! `--plugin` were tested only where a maintainer had built Teimeris.
//! This is the same three symbols over the provider the SDK is already
//! tested against, with its two native operations (`tp_echo`, `tp_sum`),
//! so the whole seam runs on every push.
//!
//! The configuration is empty or `{}`; anything else is refused as
//! `INVALID_ARG` with the text it was given, which is how a test sees an
//! adapter's own refusal pass through the loader unchanged.

#![allow(
    unsafe_code,
    reason = "`export_provider!` expands the plugin's `no_mangle` C entry points here"
)]

use std::path::PathBuf;

use teistro_port_ephemeris::{ProviderError, TestProvider};

/// Opens the provider from the adapter's configuration.
///
/// # Errors
///
/// `INVALID_ARG` on any configuration but empty or `{}`.
pub fn open(config: &str) -> Result<TestProvider, ProviderError> {
    match config.trim() {
        "" | "{}" => Ok(TestProvider::new()),
        other => Err(ProviderError::invalid(format!(
            "the test adapter takes no options; it was given `{other}`"
        ))),
    }
}

teistro_port_ephemeris::export_provider!(TestProvider, open);

/// Where cargo put this adapter's shared library, for a test that loads
/// it: beside the running test binary, in the `deps` directory a
/// dev-dependency is built into, under the platform's library name with
/// or without cargo's hash; the newest when there are several.
///
/// A test crate names this crate as a dev-dependency, which is what makes
/// cargo build the library before the test runs, and calls this to find
/// it.
///
/// # Errors
///
/// What was looked for and where, when the library is not there.
pub fn library() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|why| format!("no test binary path: {why}"))?;
    let stem = format!("{}teistro_test_adapter", std::env::consts::DLL_PREFIX);
    let suffix = std::env::consts::DLL_SUFFIX;
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for dir in exe.ancestors().skip(1).take(2) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let ours = name
                .strip_prefix(stem.as_str())
                .and_then(|rest| rest.strip_suffix(suffix))
                .is_some_and(|hash| hash.is_empty() || hash.starts_with('-'));
            if ours && let Ok(modified) = entry.metadata().and_then(|meta| meta.modified()) {
                found.push((modified, entry.path()));
            }
        }
    }
    found
        .into_iter()
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
        .ok_or_else(|| {
            format!(
                "no `{stem}*{suffix}` beside `{}`; name `teistro-test-adapter` as a dev-dependency so cargo builds it",
                exe.display()
            )
        })
}
