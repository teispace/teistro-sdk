//! A call to one of the engine's own operations, read whole from one JSON
//! record (`03-design/mcp-server.md`, step 3; ADR-0029's passthrough):
//! what [`EngineArea::call`](crate::EngineArea::call) takes, so the agent
//! server's `engine.call` reads it and nothing else composes the request.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use teistro_core::error::Error;
use teistro_core::strict;

/// One of the engine's own operations, by the name its manifest gives,
/// with its arguments keyed by parameter name.
///
/// ```
/// use teistro::EngineCall;
///
/// let call = EngineCall::from_json(r#"{"name": "tp_sum", "arguments": {"values": [1, 2]}}"#)?;
/// assert_eq!(call.name, "tp_sum");
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineCall {
    /// The operation's name, as `engine.manifest` lists it.
    pub name: String,
    /// Its arguments, keyed by the parameter names the manifest gives;
    /// none when it takes none.
    #[serde(default)]
    pub arguments: Map<String, Value>,
}

impl EngineCall {
    /// What a JSON engine call is, for a caller choosing it by name.
    pub const DESCRIPTION: &'static str = "`request` is a call to one of the engine's own operations: `name`, as `engine.manifest` lists it, and `arguments`, an object keyed by the parameter names the manifest gives. The answer is the engine's own JSON, unread by the SDK, with the provenance naming the engine. An engine that offers no operations of its own (the built-in one) refuses with how to load one that does.";

    /// The call read and checked; a refusal names the field written.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read.
    pub fn from_json(text: &str) -> Result<EngineCall, Error> {
        strict::read(text, "")
    }
}

/// The request `engine.manifest` reads: nothing, refused if anything is
/// written, so a misplaced argument is said rather than ignored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestRequest {}

impl ManifestRequest {
    /// What a JSON manifest request is, for a caller choosing it by name.
    pub const DESCRIPTION: &'static str = "`request` is `{}`. The answer is everything the engine says it offers: its name and version, and each of its own operations with its parameters, what each is for and what it answers, which `engine.call` calls by name.";

    /// The request read and checked.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not `{}`.
    pub fn from_json(text: &str) -> Result<ManifestRequest, Error> {
        strict::read(text, "")
    }
}
