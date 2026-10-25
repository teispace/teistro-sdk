//! The engine's own operations, reached through the SDK without the SDK
//! knowing what they are.
//!
//! The port names eight operations. A real engine names far more —
//! Teimeris's public header names 161 functions, 57 structs and 40 enums
//! — and a consumer who wants an eclipse, a star, an orbit or the
//! engine's own calendar grid has, until now, had to open a second
//! handle to the same engine and manage it themselves. That is the
//! largest dead end in the project
//! (`03-design/batch-and-parallelism-measured.md` §6).
//!
//! # The requirement, which is stricter than a passthrough
//!
//! An operation the engine gains **after the SDK ships** must be
//! callable without an SDK release. So the SDK holds **no list of engine
//! operations**. It holds a relay and a reader for the manifest the
//! engine ships, and a function that appears in that manifest is
//! callable the day it appears.
//!
//! # Why it is possible
//!
//! Because an engine can describe itself, and Teimeris already does:
//! `tools/idl/teimeris.idl`, extracted from its own headers, is the
//! source every Teimeris binding is generated from. It carries every
//! signature **and a role for every one of its 657 parameters** —
//! `handle`, `value`, `struct_in`, `scalar_out`, `struct_out`,
//! `array_in`/`array_len`, `array_out`/`array_cap`, `string_in` and the
//! rest. A role vocabulary is what a generic caller needs, and it is the
//! same vocabulary this SDK's own API description arrived at
//! independently.
//!
//! # Where the marshalling lives, and why it is not here
//!
//! In the **adapter**, generated from the engine's own manifest at build
//! time. That is not an implementation convenience, it is the safety
//! argument: 82 of Teimeris's parameters are doubles passed by value,
//! across 54 of its functions, and eight more functions return one.
//! Doubles do not travel in the same registers as integers on any
//! platform this SDK ships to, so a dispatcher that treated every
//! argument as a machine word would put them in the wrong place and be
//! wrong **silently** — the right number of bytes, read from the wrong
//! register. A generated dispatcher writes a real, typed C call per
//! function and cannot make that mistake. The SDK's side is a relay that
//! never sees a register.
//!
//! # What crosses
//!
//! JSON, both ways. Not because it is fast — it is not — but because
//! this is the path for what the port does *not* cover, so it is never
//! the hot loop, and because the alternative is a binary encoding that
//! the SDK would have to understand, which is exactly the knowledge that
//! must not live here for the requirement above to hold. Everything the
//! SDK does in anger goes through the eight typed operations.

use serde::{Deserialize, Serialize};

use crate::error::ProviderError;
use crate::provider::EphemerisProvider;

/// What a parameter is for, which is what a caller needs to marshal it.
///
/// The vocabulary an engine's manifest uses. **`Other` is not a
/// fallback, it is the requirement**: an engine that gains a role this
/// SDK has never heard of must still describe itself to a consumer who
/// can read it, and a manifest that failed to parse because of one
/// unknown word would break every function in it, including the ones the
/// SDK does understand.
///
/// ```
/// use teistro_port_ephemeris::native::Role;
///
/// // A role this SDK has never heard of is kept, and read as an input.
/// let unknown: Role = serde_json::from_str(r#""callback""#).unwrap();
/// assert_eq!(unknown, Role::Other(String::from("callback")));
/// assert!(unknown.is_supplied());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// An input, where the manifest says no more than that: the
    /// direction a JSON dispatcher needs, which is all the Teimeris
    /// adapter's manifest gives, every array carrying its own length.
    In,
    /// An output the engine writes, likewise.
    Out,
    /// Passed by value.
    Value,
    /// The engine's own context or object.
    Handle,
    /// A handle the call writes.
    HandleOut,
    /// A struct passed in by pointer.
    StructIn,
    /// A struct the call writes.
    StructOut,
    /// The size of the struct the call writes.
    OutStructSize,
    /// A scalar the call writes.
    ScalarOut,
    /// An array passed in.
    ArrayIn,
    /// How many elements that array has.
    ArrayLen,
    /// An array the call writes.
    ArrayOut,
    /// How many elements it may write.
    ArrayCap,
    /// A second array written alongside the first.
    ArrayOutParallel,
    /// A string passed in.
    StringIn,
    /// A string the call writes.
    StringOut,
    /// How many bytes it may write.
    StringCap,
    /// Something the engine does not describe further.
    Opaque,
    /// A role this SDK does not know, kept as the engine spelled it.
    #[serde(untagged)]
    Other(String),
}

impl Role {
    /// Whether the caller supplies this parameter.
    ///
    /// The out roles are the engine's to fill and a caller does not
    /// name them, nor a capacity, which follows from the out it sizes. A
    /// length is the caller's, as the C signature takes it. An unknown role is treated as an input, because a
    /// caller that supplies it gets an error from the engine and one
    /// that does not gets silence.
    ///
    /// ```
    /// use teistro_port_ephemeris::native::Role;
    ///
    /// assert!(Role::In.is_supplied() && Role::ArrayIn.is_supplied());
    /// // Outs and the capacities that size them are the engine's to write.
    /// assert!(!Role::Out.is_supplied() && !Role::ArrayCap.is_supplied());
    /// ```
    #[must_use]
    pub fn is_supplied(&self) -> bool {
        !matches!(
            self,
            Role::Out
                | Role::HandleOut
                | Role::StructOut
                | Role::OutStructSize
                | Role::ScalarOut
                | Role::ArrayOut
                | Role::ArrayCap
                | Role::ArrayOutParallel
                | Role::StringOut
                | Role::StringCap
        )
    }
}

/// One parameter of an engine's function.
///
/// ```
/// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
///
/// let provider = TestProvider::new();
/// let native = provider.native().unwrap();
/// let manifest = native.manifest()?;
/// let value = &manifest.function("tp_echo").unwrap().params[0];
/// assert_eq!((value.name.as_str(), value.kind.as_deref()), ("value", Some("double")));
/// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NativeParam {
    /// The name the engine gives it, which is the key a caller uses.
    pub name: String,
    /// What it is for.
    pub role: Role,
    /// The engine's own spelling of its type, for a caller that wants to
    /// show it; the SDK does not interpret it. Read from `type` too, the
    /// key the Teimeris manifest writes.
    #[serde(default, alias = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Whether a caller may leave it out: an input the engine reads as
    /// null when absent.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub optional: bool,
    /// What it means, when the engine says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

/// One operation an engine offers beyond the port's eight.
///
/// ```
/// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
///
/// let provider = TestProvider::new();
/// let native = provider.native().unwrap();
/// let manifest = native.manifest()?;
/// let sum = manifest.function("tp_sum").unwrap();
/// assert_eq!(sum.params.len(), 3);
/// assert_eq!(sum.returns.as_deref(), Some("object"));
/// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NativeFunction {
    /// The name to call it by.
    pub name: String,
    /// Its parameters, in the engine's own order.
    #[serde(default)]
    pub params: Vec<NativeParam>,
    /// What it answers with, in the engine's spelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    /// What it does, when the engine says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Whether the call changes the engine's own state (a data path, a
    /// cache), when the engine says; a caller that cannot tell reads it
    /// as possibly so.
    #[serde(
        default,
        rename = "mutatesEngineState",
        skip_serializing_if = "Option::is_none"
    )]
    pub mutates_engine_state: Option<bool>,
}

impl NativeFunction {
    /// The parameters a caller supplies, in order.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = provider.native().unwrap();
    /// let manifest = native.manifest()?;
    /// // The caller gives the array and its length; the total is the engine's.
    /// let names: Vec<&str> = manifest.function("tp_sum").unwrap().supplied().map(|param| param.name.as_str()).collect();
    /// assert_eq!(names, ["values", "count"]);
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    pub fn supplied(&self) -> impl Iterator<Item = &NativeParam> {
        self.params.iter().filter(|param| param.role.is_supplied())
    }
}

/// What an engine says it offers.
///
/// Unknown fields are kept rather than refused: a manifest is the
/// engine's document, and an engine that adds to it must not stop
/// working with an SDK that has not been told.
///
/// ```
/// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
///
/// let provider = TestProvider::new();
/// let native = provider.native().unwrap();
/// let manifest = native.manifest()?;
/// assert_eq!((manifest.engine.as_str(), manifest.version.as_str()), ("test-provider", "1"));
/// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NativeManifest {
    /// The engine's name.
    #[serde(default)]
    pub engine: String,
    /// The engine's version, which is what a caller caches against.
    #[serde(default)]
    pub version: String,
    /// The operations, in the engine's own order.
    #[serde(default)]
    pub functions: Vec<NativeFunction>,
}

impl NativeManifest {
    /// One operation by name.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = provider.native().unwrap();
    /// let manifest = native.manifest()?;
    /// assert!(manifest.function("tp_echo").is_some());
    /// assert!(manifest.function("tp_nope").is_none());
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    #[must_use]
    pub fn function(&self, name: &str) -> Option<&NativeFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// How many operations it names.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = provider.native().unwrap();
    /// let manifest = native.manifest()?;
    /// assert_eq!(manifest.len(), 2);
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Whether it names none.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = provider.native().unwrap();
    /// let manifest = native.manifest()?;
    /// assert!(!manifest.is_empty());
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }
}

/// The engine's own operations, over a provider that offers them.
///
/// ```
/// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
///
/// let provider = TestProvider::new();
/// let native = provider.native().expect("the test provider describes itself");
/// let manifest = native.manifest().expect("it parses");
/// assert!(manifest.function("tp_echo").is_some());
///
/// let answer = native
///     .call("tp_echo", &serde_json::json!({ "value": 6.0 }))
///     .expect("it answers");
/// assert_eq!(answer["value"], 6.0);
/// ```
#[derive(Clone, Copy)]
pub struct Native<'p> {
    provider: &'p dyn EphemerisProvider,
}

impl core::fmt::Debug for Native<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Native")
            .field("engine", &self.provider.capabilities().identity.name)
            .finish()
    }
}

impl<'p> Native<'p> {
    /// The engine's operations over a provider.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Native, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = Native::new(&provider);
    /// assert_eq!(native.call_json("tp_echo", r#"{"value": 1}"#)?, r#"{"value":1.0}"#);
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    #[must_use]
    pub const fn new(provider: &'p dyn EphemerisProvider) -> Native<'p> {
        Native { provider }
    }

    /// What the engine says it offers, parsed.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = provider.native().unwrap();
    /// let manifest = native.manifest()?;
    /// assert_eq!(manifest.functions.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["tp_echo", "tp_sum"]);
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The provider's refusal, or a manifest that is not the JSON this
    /// reads.
    pub fn manifest(&self) -> Result<NativeManifest, ProviderError> {
        let text = self.provider.native_manifest()?;
        serde_json::from_str(&text).map_err(|err| {
            ProviderError::invalid(format!("the engine's manifest does not parse: {err}"))
        })
    }

    /// The manifest as the engine wrote it, for a caller that wants to
    /// hand it on rather than read it — a binding building a proxy, or a
    /// consumer caching it against the engine's version.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = provider.native().unwrap();
    /// // The engine's own text, to hand on unread.
    /// assert_eq!(native.manifest_json()?, provider.native_manifest()?);
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The provider's refusal.
    pub fn manifest_json(&self) -> Result<String, ProviderError> {
        self.provider.native_manifest()
    }

    /// Calls one of the engine's own operations by name.
    ///
    /// The arguments are an object keyed by the parameter names the
    /// manifest gives. What comes back is whatever the engine's adapter
    /// answers with, unread by the SDK.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = provider.native().unwrap();
    /// let answer = native.call("tp_sum", &serde_json::json!({ "values": [0.5, 0.25] }))?;
    /// assert_eq!(answer["total"], 0.75);
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The provider's refusal — an unknown function, an argument it
    /// cannot marshal, or the engine's own failure — or an answer that
    /// is not JSON.
    pub fn call(
        &self,
        function: &str,
        arguments: &serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        let text = self.call_json(function, &arguments.to_string())?;
        serde_json::from_str(&text).map_err(|err| {
            ProviderError::invalid(format!("the engine's answer does not parse: {err}"))
        })
    }

    /// Calls an operation with the arguments already written as JSON, and
    /// answers with the engine's own JSON: the relay a binding uses,
    /// where parsing twice would be waste.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, ProviderError, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let native = provider.native().unwrap();
    /// assert!(matches!(native.call_json("tp_nope", "{}"), Err(ProviderError::Unsupported { .. })));
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Native::call`].
    pub fn call_json(&self, function: &str, arguments_json: &str) -> Result<String, ProviderError> {
        self.provider.native_call(function, arguments_json)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::*;
    use crate::test_provider::TestProvider;

    fn native() -> TestProvider {
        TestProvider::new()
    }

    #[test]
    fn a_provider_that_describes_itself_says_so_before_it_is_asked() {
        let provider = native();
        assert!(provider.capabilities().native);
        assert!(provider.native().is_some(), "asked once, not per operation");
    }

    #[test]
    fn the_manifest_names_what_the_engine_offers() {
        let native = native();
        let native = native.native().unwrap();
        let manifest = native.manifest().unwrap();
        assert_eq!(manifest.engine, "test-provider");
        assert_eq!(manifest.len(), 2);
        assert!(!manifest.is_empty());
        let echo = manifest.function("tp_echo").expect("it is named");
        assert_eq!(echo.params.len(), 1);
        assert_eq!(echo.params[0].role, Role::Value);
        assert_eq!(echo.params[0].kind.as_deref(), Some("double"));
        assert!(echo.doc.is_some(), "an engine that documents itself");
        assert!(manifest.function("tp_nothing").is_none());
    }

    #[test]
    fn a_caller_supplies_the_in_roles_and_not_the_out_ones() {
        let provider = native();
        let native = provider.native().unwrap();
        let manifest = native.manifest().unwrap();
        let sum = manifest.function("tp_sum").unwrap();
        let supplied: Vec<&str> = sum.supplied().map(|p| p.name.as_str()).collect();
        assert_eq!(
            supplied,
            ["values", "count"],
            "`total` is the engine's to fill"
        );
    }

    #[test]
    fn an_operation_answers_through_the_relay() {
        let provider = native();
        let native = provider.native().unwrap();
        let answer = native
            .call("tp_echo", &serde_json::json!({ "value": 6.0 }))
            .unwrap();
        assert_eq!(answer["value"], 6.0);
        let summed = native
            .call("tp_sum", &serde_json::json!({ "values": [1.0, 2.0, 3.5] }))
            .unwrap();
        assert_eq!(summed["total"], 6.5);
        assert_eq!(summed["count"], 3);
    }

    #[test]
    fn the_engines_refusals_are_the_engines_own() {
        let provider = native();
        let native = provider.native().unwrap();
        let unknown = native
            .call("tm_no_such_thing", &serde_json::json!({}))
            .expect_err("the engine names no such function");
        assert!(
            format!("{unknown}").contains("tm_no_such_thing"),
            "the refusal names what was asked for: {unknown}"
        );
        let wrong = native
            .call("tp_echo", &serde_json::json!({ "value": "six" }))
            .expect_err("a number was wanted");
        assert!(format!("{wrong}").contains("value"), "{wrong}");
    }

    #[test]
    fn a_provider_that_offers_nothing_says_nothing() {
        // The default: every provider written before this existed, and
        // every one that does not want it.
        struct Bare;
        impl EphemerisProvider for Bare {
            fn capabilities(&self) -> crate::capabilities::Capabilities {
                crate::capabilities::Capabilities {
                    native: false,
                    ..TestProvider::new().capabilities()
                }
            }
            fn positions(
                &self,
                request: &crate::provider::PositionRequest<'_>,
            ) -> Result<crate::columns::PositionColumns, ProviderError> {
                TestProvider::new().positions(request)
            }
        }
        let bare = Bare;
        assert!(bare.native().is_none());
        assert!(bare.native_manifest().is_err());
        assert!(bare.native_call("anything", "{}").is_err());
    }

    #[test]
    fn a_role_the_sdk_has_never_heard_of_does_not_break_the_manifest() {
        // The requirement, at the level of the vocabulary: an engine that
        // gains a role must still describe itself to a consumer who can
        // read it, and one unknown word must not take the whole manifest
        // down with it.
        let text = r#"{
          "engine": "future", "version": "9",
          "functions": [
            { "name": "f_new", "params": [
              { "name": "a", "role": "value" },
              { "name": "b", "role": "quaternion_in" }
            ] }
          ]
        }"#;
        let manifest: NativeManifest = serde_json::from_str(text).expect("it still parses");
        let function = manifest.function("f_new").unwrap();
        assert_eq!(function.params[0].role, Role::Value);
        assert_eq!(
            function.params[1].role,
            Role::Other(String::from("quaternion_in"))
        );
        // And an unknown role is treated as the caller's to supply, so
        // the function stays callable rather than becoming unreachable.
        assert!(function.params[1].role.is_supplied());
        assert_eq!(function.supplied().count(), 2);
    }

    /// The Teimeris adapter's manifest speaks `in`/`out`, `type`,
    /// `optional` and `mutatesEngineState`: each is read, and an out is
    /// not offered to a caller.
    #[test]
    fn the_teimeris_spelling_of_a_manifest_is_read() {
        let text = r#"{"engine":"teimeris","version":"0.1.0","functions":[
          {"name":"tm_julian_day","params":[
            {"name":"dt","role":"in","type":"tm_datetime","optional":true},
            {"name":"cal","role":"in","type":"tm_calendar"},
            {"name":"out_jd","role":"out","type":"double"}],
           "returns":"status","mutatesEngineState":false}]}"#;
        let manifest: NativeManifest = serde_json::from_str(text).unwrap();
        let function = manifest.function("tm_julian_day").unwrap();
        assert_eq!(function.params[0].kind.as_deref(), Some("tm_datetime"));
        assert!(function.params[0].optional && !function.params[1].optional);
        assert_eq!(function.mutates_engine_state, Some(false));
        let supplied: Vec<&str> = function.supplied().map(|p| p.name.as_str()).collect();
        assert_eq!(supplied, ["dt", "cal"]);
    }

    #[test]
    fn a_manifest_the_sdk_does_not_understand_is_a_refusal_and_not_a_panic() {
        struct Broken;
        impl EphemerisProvider for Broken {
            fn capabilities(&self) -> crate::capabilities::Capabilities {
                TestProvider::new().capabilities()
            }
            fn positions(
                &self,
                request: &crate::provider::PositionRequest<'_>,
            ) -> Result<crate::columns::PositionColumns, ProviderError> {
                TestProvider::new().positions(request)
            }
            fn native_manifest(&self) -> Result<String, ProviderError> {
                Ok(String::from("not json at all"))
            }
        }
        let broken = Broken;
        let native = broken.native().expect("it declares one");
        let refused = native.manifest().expect_err("it does not parse");
        assert!(format!("{refused}").contains("manifest"), "{refused}");
        // The raw manifest is still available, which is what a binding
        // relaying it to another reader would want.
        assert_eq!(native.manifest_json().unwrap(), "not json at all");
    }
}
