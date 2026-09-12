//! The engine's own operations at the boundary: what the port reaches of
//! a provider beyond its eight operations
//! (`crates/port-ephemeris/src/native.rs`).
//!
//! Two entry points and no more, because the SDK holds no list of an
//! engine's operations. One asks the engine what it offers and the other
//! relays a call by the name that answer gives, so an operation the
//! engine gains after this library ships is callable through it without
//! a new release — which is the whole requirement, and the reason
//! neither of these has a signature that mentions any engine.
//!
//! Every binding is generated from the API description, so these two
//! cross into C, Node, Dart and Python without any of them being
//! written by hand; a proxy over them in each language is what turns
//! `call("tm_eclipse_when", …)` into `engine.tm_eclipse_when(…)`.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::Status;
use teistro_core::error::Error;
use teistro_port_ephemeris::Native;

use crate::context::TsContext;
use crate::string::TsString;
use crate::support::{text, with_context, write_plain};

/// The engine of a context, or the refusal that says why there is none.
fn native_of(context: &TsContext) -> Result<Native<'_>, Error> {
    let provider = context
        .provider()
        .ok_or_else(crate::support::no_ephemeris)?;
    if !provider.capabilities().native {
        return Err(Error::unsupported(format!(
            "{} describes no operations of its own",
            provider.capabilities().identity.name
        ))
        .with_hint("an adapter offers these by answering the port's native manifest"));
    }
    Ok(Native::new(provider))
}

/// What the context's engine says it offers beyond this library's own
/// operations: its manifest, as the engine wrote it.
///
/// The document names each operation, its parameters and the role of
/// each — which of them a caller supplies and which the engine fills.
/// Read it once and cache it against the engine's version; it changes
/// when the engine does and not when this library does.
///
/// `CAPABILITY` when the context has no ephemeris, `UNSUPPORTED` when the
/// engine describes nothing of its own.
///
/// # Safety
///
/// `context` must be a live handle; `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_ephemeris_manifest(
    context: *const TsContext,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        let manifest = native_of(ctx)?
            .manifest_json()
            .map_err(|error| Error::new(Status::Provider, error.to_string()))?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", TsString::from_string(manifest)) }
    })
}

/// Calls one of the engine's own operations by the name its manifest
/// gives.
///
/// `arguments_json` is a JSON object keyed by the parameter names the
/// manifest names, holding the ones it marks as a caller's to supply.
/// The answer is the engine's own JSON, unread by this library.
///
/// `CAPABILITY` when the context has no ephemeris, `UNSUPPORTED` when the
/// engine describes nothing of its own or names no such operation, and
/// `PROVIDER` when the engine itself refuses.
///
/// # Safety
///
/// `context` must be a live handle; `function` and `arguments_json` must
/// be NUL-terminated; `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_ephemeris_call(
    context: *const TsContext,
    function: *const c_char,
    arguments_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let name = unsafe { text(function, "function") }?;
        // SAFETY: the entry point's contract.
        let arguments = unsafe { text(arguments_json, "arguments_json") }?;
        let answer = native_of(ctx)?
            .call_json(name, arguments)
            .map_err(|error| Error::new(Status::Provider, error.to_string()))?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", TsString::from_string(answer)) }
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::expect_used,
        clippy::unwrap_used,
        reason = "a test fails by panicking"
    )]

    use super::native_of;
    use crate::context::TsContext;
    use teistro_core::Status;
    use teistro_port_ephemeris::test_provider::TestProvider;
    use teistro_port_ephemeris::{Capabilities, EphemerisProvider};

    fn with_test_provider() -> TsContext {
        TsContext::build(
            None,
            None,
            teistro::Ephemeris::Provider(Box::new(TestProvider::new())),
            None,
        )
        .expect("the default profile")
    }

    #[test]
    fn the_manifest_crosses_the_boundary_as_the_engine_wrote_it() {
        let context = with_test_provider();
        let native = native_of(&context).expect("the test provider describes itself");
        let manifest = native.manifest_json().expect("it answers");
        assert!(manifest.contains("tp_echo"), "{manifest}");
        // Parsed the same either side: the boundary relays, it does not
        // reinterpret.
        assert_eq!(
            native.manifest().unwrap(),
            serde_json::from_str(&manifest).unwrap()
        );
    }

    #[test]
    fn a_call_crosses_and_the_answer_comes_back_unread() {
        let context = with_test_provider();
        let native = native_of(&context).unwrap();
        let answer = native
            .call_json("tp_echo", r#"{"value":3.25}"#)
            .expect("it answers");
        assert_eq!(answer, r#"{"value":3.25}"#);
    }

    #[test]
    fn a_context_without_an_ephemeris_says_so_rather_than_failing_the_call() {
        let context = TsContext::build(None, None, teistro::Ephemeris::None, None)
            .expect("the default profile");
        let refused = native_of(&context).expect_err("there is no engine");
        assert_eq!(refused.status, Status::Capability);
        assert!(refused.hint().is_some(), "and says what to do about it");
    }

    #[test]
    fn an_engine_that_describes_nothing_is_unsupported_and_named() {
        struct Quiet;
        impl EphemerisProvider for Quiet {
            fn capabilities(&self) -> Capabilities {
                Capabilities {
                    native: false,
                    ..TestProvider::new().capabilities()
                }
            }
            fn positions(
                &self,
                request: &teistro_port_ephemeris::PositionRequest<'_>,
            ) -> Result<
                teistro_port_ephemeris::PositionColumns,
                teistro_port_ephemeris::ProviderError,
            > {
                TestProvider::new().positions(request)
            }
        }
        let context = TsContext::build(
            None,
            None,
            teistro::Ephemeris::Provider(Box::new(Quiet)),
            None,
        )
        .unwrap();
        let refused = native_of(&context).expect_err("it describes nothing");
        assert_eq!(refused.status, Status::Unsupported);
        assert!(
            format!("{refused}").contains("test-provider"),
            "the refusal names the engine: {refused}"
        );
    }

    #[test]
    fn an_operation_the_engine_does_not_have_is_the_engines_refusal() {
        let context = with_test_provider();
        let native = native_of(&context).unwrap();
        let refused = native
            .call_json("tm_eclipse_when", "{}")
            .expect_err("this engine names no such function");
        assert!(
            format!("{refused}").contains("tm_eclipse_when"),
            "{refused}"
        );
    }
}
