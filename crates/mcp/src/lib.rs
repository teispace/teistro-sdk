//! The Teistro SDK as tools an agent calls: a Model Context Protocol
//! server answering every record entry point of the Rust façade
//! (`03-design/mcp-server.md`).
//!
//! The server is **dual-era** (D1). A request carrying the modern
//! revision's `_meta` (`io.modelcontextprotocol/protocolVersion` naming
//! [`MODERN`]) is served statelessly; `initialize` selects the handshake
//! revision, [`LEGACY`], for the rest of the process. Either way a tool is
//! a record the façade already reads (D3), answered as the envelope the
//! bindings read (D4) under the profile, settings and locale the call
//! names (D5), and a refusal is a tool error naming its field (D7).
//!
//! [`Server::handle`] takes one JSON-RPC message and gives the reply, so
//! the transport is the caller's: the binary speaks stdio, and the same
//! server answers any transport that delivers a message at a time.
//!
//! ```
//! use teistro_mcp::{Engine, Server};
//!
//! let mut server = Server::new(Engine::None);
//! let reply = server
//!     .handle(r#"{"jsonrpc":"2.0","id":1,"method":"server/discover"}"#)
//!     .expect("a request is answered");
//! assert!(reply.contains(r#""supportedVersions":["2026-07-28","2025-11-25"]"#));
//! ```

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use serde_json::{Map, Value, json};
use teistro::records::{Answered, Record};
use teistro::{Context, Ephemeris, Error};
use teistro_core::envelope::canonical_json;
use teistro_core::settings::DEFAULT_PROFILE;
use teistro_port_ephemeris::EphemerisProvider;
#[cfg(not(target_family = "wasm"))]
pub use teistro_port_ephemeris::load::Adapter;
use teistro_port_ephemeris::native::NativeFunction;

mod completion;
mod prompts;
mod resources;
mod schemas;
mod tools;

pub use schemas::Detail;

/// The stateless revision the server speaks.
pub const MODERN: &str = "2026-07-28";
/// The handshake revision the server speaks after `initialize`.
pub const LEGACY: &str = "2025-11-25";
/// Every revision the server speaks, newest first.
pub const SUPPORTED: [&str; 2] = [MODERN, LEGACY];

const VERSION_META: &str = "io.modelcontextprotocol/protocolVersion";
const SERVER_INFO_META: &str = "io.modelcontextprotocol/serverInfo";

/// How long a client may keep a list or a resource: each changes only with the
/// binary, so a day.
const TTL_MS: u64 = 86_400_000;

/// The contexts kept before the cache starts again. A context is the
/// profile resolved and the locale engine loaded, which is what a call
/// would otherwise pay each time; an agent asks under a handful of
/// settings, so a small bound keeps every one it uses.
const CONTEXTS: usize = 16;

const PARSE_ERROR: i64 = -32_700;
const INVALID_REQUEST: i64 = -32_600;
const METHOD_NOT_FOUND: i64 = -32_601;
const INVALID_PARAMS: i64 = -32_602;
const UNSUPPORTED_VERSION: i64 = -32_022;
/// What the handshake revision answered for a resource it does not
/// have; the stateless revision answers [`INVALID_PARAMS`].
const RESOURCE_NOT_FOUND: i64 = -32_002;

const INSTRUCTIONS: &str = "Every tool computes; nothing is recalled. An answer is \
`{value, provenance}`: the provenance names the settings hash, the input hash and every \
convention applied, so a number quoted from it can be reproduced. Each call names its own \
`profile`, `settings` (a patch over the profile) and `locale`; `settings.describe` answers \
the profiles and every setting with its documentation. A refusal is a tool error naming the \
field, the accepted range and a hint: fix that field and call again.";

/// The ephemeris every context computes with, chosen on the server's
/// command line and never by a tool argument (D6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    /// The analytic ephemeris the SDK carries.
    Builtin,
    /// The Surya Siddhanta's own astronomy.
    SuryaSiddhanta,
    /// None: calendars and numerology answer, a position refuses.
    None,
}

impl Engine {
    /// The names [`Engine::from_name`] reads, as every binding names them.
    pub const NAMES: [&'static str; 3] = ["BUILTIN", "SURYA_SIDDHANTA", "NONE"];

    /// The engine called `name`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Engine> {
        match name {
            "BUILTIN" => Some(Engine::Builtin),
            "SURYA_SIDDHANTA" => Some(Engine::SuryaSiddhanta),
            "NONE" => Some(Engine::None),
            _ => None,
        }
    }

    fn entry(self) -> Ephemeris {
        match self {
            Engine::Builtin => Ephemeris::Builtin,
            Engine::SuryaSiddhanta => Ephemeris::SuryaSiddhanta,
            Engine::None => Ephemeris::None,
        }
    }
}

/// Which revision a request is answered under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Era {
    Modern,
    Legacy,
}

/// A failure of the protocol itself, answered as a JSON-RPC error.
#[derive(Debug)]
struct Fault {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl Fault {
    fn new(code: i64, message: impl Into<String>) -> Fault {
        Fault {
            code,
            message: message.into(),
            data: None,
        }
    }
}

/// What a context is built from: the profile, the canonical settings
/// patch and the locale tag, empty where the call named none.
type ContextKey = (String, String, String);

/// Opens the engine a context computes with first: one provider per
/// context, all over the one engine.
pub type OpenEngine = Box<dyn Fn() -> Result<Box<dyn EphemerisProvider>, Error> + Send + Sync>;

/// The engine ahead of the server's own, and the operations of its own
/// its manifest named when the server started: each is a tool,
/// `engine.<name>` (ADR-0030).
struct Plugin {
    name: String,
    open: OpenEngine,
    operations: Vec<NativeFunction>,
}

/// The server: the engine it was started with, the adapter loaded ahead
/// of it, whether `initialize` chose the handshake revision, and the
/// contexts it has built.
///
/// The contexts are a cache and not state: no answer depends on what was
/// asked before (D5).
pub struct Server {
    engine: Engine,
    plugin: Option<Plugin>,
    legacy: bool,
    detail: Detail,
    /// The tool list, built on the first `tools/list`: it changes only
    /// with the binary and the engine, both fixed for the process.
    listed: Option<Map<String, Value>>,
    contexts: HashMap<ContextKey, Context>,
}

impl core::fmt::Debug for Server {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Server")
            .field("engine", &self.engine)
            .field("plugin", &self.plugin.as_ref().map(|p| p.name.as_str()))
            .field("legacy", &self.legacy)
            .field("detail", &self.detail)
            .field("listed", &self.listed.is_some())
            .field("contexts", &self.contexts.len())
            .finish()
    }
}

impl Server {
    /// A server computing with `engine`.
    #[must_use]
    pub fn new(engine: Engine) -> Server {
        Server {
            engine,
            plugin: None,
            legacy: false,
            detail: Detail::Lean,
            listed: None,
            contexts: HashMap::new(),
        }
    }

    /// The same server, its tool list stating each record's schema in
    /// `detail`: [`Detail::Lean`] unless a client validating structured
    /// content asks for [`Detail::Full`].
    #[must_use]
    pub fn with_detail(self, detail: Detail) -> Server {
        Server {
            detail,
            listed: None,
            ..self
        }
    }

    /// A server computing with the engine `open` gives first and
    /// `fallback` where it does not answer, the chain every binding
    /// builds (ADR-0029): a loaded adapter ([`Server::with_plugin`]) or
    /// a provider a program embedding the server brings. Each operation
    /// of the engine's own is listed as a tool, `engine.<name>`, with its
    /// parameters as the input schema; one whose name an SDK tool already
    /// holds is reached through `engine.call` instead.
    ///
    /// # Errors
    ///
    /// `open`'s own refusal, or a manifest the engine wrote that does not
    /// parse.
    pub fn with_engine(fallback: Engine, open: OpenEngine) -> Result<Server, Error> {
        let provider = open()?;
        let name = provider.capabilities().identity.name;
        let operations = match provider.native() {
            Some(native) => native.manifest()?.functions,
            None => Vec::new(),
        };
        Ok(Server {
            plugin: Some(Plugin {
                name,
                open,
                operations: tools::reachable(operations),
            }),
            ..Server::new(fallback)
        })
    }

    /// A server computing with the engine in `adapter` first and
    /// `fallback` behind it: [`Server::with_engine`] over the adapter's
    /// providers.
    ///
    /// # Errors
    ///
    /// A vtable the adapter wrote that does not bind, or a manifest the
    /// engine wrote that does not parse.
    #[cfg(not(target_family = "wasm"))]
    pub fn with_plugin(fallback: Engine, adapter: Adapter) -> Result<Server, Error> {
        Server::with_engine(
            fallback,
            Box::new(move || {
                let provider: Box<dyn EphemerisProvider> = Box::new(adapter.provider()?);
                Ok(provider)
            }),
        )
    }

    /// The reply to one JSON-RPC message, or `None` for a notification.
    pub fn handle(&mut self, message: &str) -> Option<String> {
        let message: Value = match serde_json::from_str(message) {
            Ok(message) => message,
            Err(why) => {
                return Some(failure(
                    &Value::Null,
                    &Fault::new(PARSE_ERROR, format!("the message is not JSON: {why}")),
                ));
            }
        };
        let Some(object) = message.as_object() else {
            return Some(failure(
                &Value::Null,
                &Fault::new(
                    INVALID_REQUEST,
                    "a message is one JSON object; batches are not part of the protocol",
                ),
            ));
        };
        let id = object.get("id");
        let Some(method) = object.get("method").and_then(Value::as_str) else {
            // A response to a request the server never sends, or nothing.
            return id.map(|id| {
                failure(
                    id,
                    &Fault::new(INVALID_REQUEST, "a request names its `method`"),
                )
            });
        };
        let empty = Value::Object(Map::new());
        let params = object.get("params").unwrap_or(&empty);
        // A notification asks for nothing back: `initialized` and
        // `cancelled` change nothing here, and every call finishes before
        // the next message is read.
        let id = id?;
        Some(match self.request(method, params) {
            Ok(result) => success(id, &result),
            Err(fault) => failure(id, &fault),
        })
    }

    fn request(&mut self, method: &str, params: &Value) -> Result<Value, Fault> {
        match method {
            "initialize" => Ok(self.initialize()),
            "ping" => Ok(json!({})),
            "server/discover" => Ok(discover()),
            "tools/list" => {
                let era = self.era(params)?;
                first_page(params)?;
                let detail = self.detail;
                let operations = self.plugin.as_ref().map_or(&[][..], |p| &p.operations);
                let list = self
                    .listed
                    .get_or_insert_with(|| tools::list(operations, detail))
                    .clone();
                Ok(complete(era, cacheable(era, list)))
            }
            "tools/call" => {
                let era = self.era(params)?;
                Ok(complete(era, self.call(params)?))
            }
            "resources/list" => {
                let era = self.era(params)?;
                first_page(params)?;
                Ok(complete(era, cacheable(era, resources::list())))
            }
            "resources/templates/list" => {
                let era = self.era(params)?;
                first_page(params)?;
                Ok(complete(era, cacheable(era, resources::templates())))
            }
            "prompts/list" => {
                let era = self.era(params)?;
                first_page(params)?;
                Ok(complete(era, cacheable(era, prompts::list())))
            }
            "prompts/get" => {
                let era = self.era(params)?;
                let mut ask = |tool: &str, request: &Value| -> Result<Value, Error> {
                    let record = teistro::records::record(tool).ok_or_else(|| {
                        Error::internal(format!("this build carries no `{tool}`"))
                    })?;
                    let mut arguments = Map::new();
                    arguments.insert(String::from("request"), request.clone());
                    let mut answer = self.answer(record, &arguments)?;
                    Ok(answer.get_mut("value").map(Value::take).unwrap_or_default())
                };
                let written = prompts::get(params, &mut ask).map_err(|refused| Fault {
                    code: INVALID_PARAMS,
                    message: refused.message,
                    data: refused
                        .argument
                        .map(|argument| json!({ "argument": argument })),
                })?;
                Ok(complete(era, written))
            }
            "completion/complete" => {
                let era = self.era(params)?;
                let completed = completion::complete(params).map_err(|refused| Fault {
                    code: INVALID_PARAMS,
                    message: refused.message,
                    data: Some(json!({ "field": refused.field })),
                })?;
                Ok(complete(era, completed))
            }
            "resources/read" => {
                let era = self.era(params)?;
                let uri = params.get("uri").and_then(Value::as_str).ok_or_else(|| {
                    Fault::new(
                        INVALID_PARAMS,
                        "`resources/read` names its resource in `uri`",
                    )
                })?;
                let contents = resources::read(uri).ok_or_else(|| Fault {
                    code: match era {
                        Era::Modern => INVALID_PARAMS,
                        Era::Legacy => RESOURCE_NOT_FOUND,
                    },
                    message: format!(
                        "no resource `{uri}`: `resources/list` and \
                         `resources/templates/list` name every one"
                    ),
                    data: Some(json!({ "uri": uri })),
                })?;
                Ok(complete(era, cacheable(era, contents)))
            }
            _ => Err(Fault::new(
                METHOD_NOT_FOUND,
                format!(
                    "no method `{method}`: the server answers `server/discover`, `tools/list`, \
                     `tools/call`, `resources/list`, `resources/templates/list` and \
                     `resources/read`, `prompts/list`, `prompts/get` and \
                     `completion/complete`, and `initialize` and `ping` under {LEGACY}"
                ),
            )),
        }
    }

    /// The handshake: the legacy revision, for the rest of the process.
    fn initialize(&mut self) -> Value {
        self.legacy = true;
        json!({
            "protocolVersion": LEGACY,
            "capabilities": capabilities(),
            "serverInfo": server_info(),
            "instructions": INSTRUCTIONS,
        })
    }

    /// The revision a request is answered under: the one its `_meta`
    /// names, or the handshake's once `initialize` chose it.
    fn era(&self, params: &Value) -> Result<Era, Fault> {
        let requested = params
            .get("_meta")
            .and_then(|meta| meta.get(VERSION_META))
            .and_then(Value::as_str);
        match requested {
            Some(MODERN) => Ok(Era::Modern),
            Some(LEGACY) => Ok(Era::Legacy),
            None if self.legacy => Ok(Era::Legacy),
            _ => Err(Fault {
                code: UNSUPPORTED_VERSION,
                message: match requested {
                    Some(version) => format!("the server does not speak revision {version}"),
                    None => format!(
                        "the request names no revision: send `_meta.{VERSION_META}`, or \
                         `initialize` first"
                    ),
                },
                data: Some(json!({ "supported": SUPPORTED, "requested": requested })),
            }),
        }
    }

    fn call(&mut self, params: &Value) -> Result<Map<String, Value>, Fault> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| Fault::new(INVALID_PARAMS, "`tools/call` names its tool in `name`"))?;
        let empty = Map::new();
        let arguments = match params.get("arguments") {
            None | Some(Value::Null) => &empty,
            Some(Value::Object(arguments)) => arguments,
            Some(_) => return Err(Fault::new(INVALID_PARAMS, "`arguments` is an object")),
        };
        let outcome = if name == tools::DESCRIBE {
            tools::describe(arguments)
        } else if name == schemas::DESCRIBE {
            schemas::describe(arguments)
        } else if let Some(operation) = self.operation(name) {
            let request = json!({ "name": operation, "arguments": arguments }).to_string();
            self.engine_answer(request)
        } else {
            let record = teistro::records::record(name).ok_or_else(|| {
                Fault::new(
                    INVALID_PARAMS,
                    format!("no tool `{name}`: `tools/list` names every one"),
                )
            })?;
            self.answer(record, arguments)
        };
        Ok(match outcome {
            Ok(structured) => tool_result(structured, false),
            Err(refusal) => tool_result(refused(&refusal), true),
        })
    }

    /// A record tool's answer: the arguments read, the context they name,
    /// and the record answered under it as the envelope.
    fn answer(&mut self, record: Record, arguments: &Map<String, Value>) -> Result<Value, Error> {
        let asked = tools::Arguments::read(arguments)?;
        let context = self.context(&asked)?;
        envelope(record.answer(context, &asked.request)?)
    }

    /// The engine's own operations listed as tools; none without a
    /// plugin.
    fn operations(&self) -> &[NativeFunction] {
        self.plugin
            .as_ref()
            .map_or(&[], |plugin| plugin.operations.as_slice())
    }

    /// The engine operation the tool `name` calls, if it is one.
    fn operation(&self, name: &str) -> Option<String> {
        let wanted = name.strip_prefix(tools::ENGINE_PREFIX)?;
        self.operations()
            .iter()
            .find(|operation| operation.name == wanted)
            .map(|operation| operation.name.clone())
    }

    /// An `engine.<name>` tool's answer: `engine.call` under the default
    /// context, the tool's arguments being the operation's own.
    fn engine_answer(&mut self, request: String) -> Result<Value, Error> {
        let record = teistro::records::record("engine.call")
            .ok_or_else(|| Error::internal("this build carries no `engine.call`"))?;
        let asked = tools::Arguments {
            request,
            profile: None,
            settings: None,
            locale: None,
        };
        let context = self.context(&asked)?;
        envelope(record.answer(context, &asked.request)?)
    }

    fn context(&mut self, asked: &tools::Arguments) -> Result<&Context, Error> {
        let key: ContextKey = (
            asked
                .profile
                .clone()
                .unwrap_or_else(|| DEFAULT_PROFILE.to_owned()),
            asked.settings.clone().unwrap_or_default(),
            asked.locale.clone().unwrap_or_default(),
        );
        if !self.contexts.contains_key(&key) && self.contexts.len() >= CONTEXTS {
            self.contexts.clear();
        }
        Ok(match self.contexts.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let (profile, settings, locale) = entry.key();
                let mut chain = Vec::with_capacity(2);
                if let Some(plugin) = &self.plugin {
                    chain.push(Ephemeris::Provider((plugin.open)()?));
                }
                if self.engine != Engine::None || chain.is_empty() {
                    chain.push(self.engine.entry());
                }
                let mut builder = Context::builder()
                    .profile(profile.as_str())
                    .ephemeris(chain);
                if !settings.is_empty() {
                    builder = builder.settings_json(settings.as_str());
                }
                if !locale.is_empty() {
                    builder = builder.locale(locale.as_str());
                }
                entry.insert(builder.build()?)
            }
        })
    }
}

/// The modern revision's discovery answer.
fn discover() -> Value {
    json!({
        "resultType": "complete",
        "supportedVersions": SUPPORTED,
        "capabilities": capabilities(),
        "instructions": INSTRUCTIONS,
        "ttlMs": TTL_MS,
        "cacheScope": "public",
        "_meta": { SERVER_INFO_META: server_info() },
    })
}

/// What the server offers: tools, resources and prompts, none of which
/// changes while it runs, so none sends a change, and completions of a
/// template's or a prompt's argument.
fn capabilities() -> Value {
    json!({
        "tools": { "listChanged": false },
        "resources": {},
        "prompts": {},
        "completions": {},
    })
}

fn server_info() -> Value {
    json!({
        "name": env!("CARGO_PKG_NAME"),
        "title": "Teistro",
        "version": env!("CARGO_PKG_VERSION"),
    })
}

/// A result every caller may keep a day: everything the server lists or
/// serves changes only with the binary, so a day. The handshake revision
/// has no caching fields.
fn cacheable(era: Era, mut result: Map<String, Value>) -> Map<String, Value> {
    if era == Era::Modern {
        result.insert(String::from("ttlMs"), json!(TTL_MS));
        result.insert(String::from("cacheScope"), json!("public"));
    }
    result
}

/// Every list fits one page, so a cursor is one the server never gave.
fn first_page(params: &Value) -> Result<(), Fault> {
    match params.get("cursor") {
        None | Some(Value::Null) => Ok(()),
        Some(cursor) => Err(Fault {
            code: INVALID_PARAMS,
            message: String::from("no page follows the first: every list is one page"),
            data: Some(json!({ "cursor": cursor })),
        }),
    }
}

/// A result as its revision writes it: the modern one marks it complete.
fn complete(era: Era, mut result: Map<String, Value>) -> Value {
    if era == Era::Modern {
        result.insert(String::from("resultType"), json!("complete"));
    }
    Value::Object(result)
}

/// An answer as the bindings read it: `{value, provenance}`, the
/// provenance absent where the area seals none.
fn envelope(answered: Answered) -> Result<Value, Error> {
    let mut envelope = Map::new();
    envelope.insert(String::from("value"), answered.value);
    if let Some(provenance) = answered.provenance {
        let provenance = serde_json::to_value(provenance)
            .map_err(|why| Error::internal(format!("a provenance serde cannot write: {why}")))?;
        envelope.insert(String::from("provenance"), provenance);
    }
    Ok(Value::Object(envelope))
}

/// A refusal as structured content: the SDK's own error record, its
/// status, field, message and hint (D7).
fn refused(refusal: &Error) -> Value {
    serde_json::to_value(refusal).unwrap_or_else(|_| json!({ "message": refusal.to_string() }))
}

/// A tool's result: the structured content, and the same JSON as text for
/// a client that reads only text.
fn tool_result(structured: Value, is_error: bool) -> Map<String, Value> {
    let mut result = Map::new();
    result.insert(
        String::from("content"),
        json!([{ "type": "text", "text": canonical_json(&structured) }]),
    );
    result.insert(String::from("structuredContent"), structured);
    result.insert(String::from("isError"), json!(is_error));
    result
}

fn success(id: &Value, result: &Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

fn failure(id: &Value, fault: &Fault) -> String {
    let mut error = Map::new();
    error.insert(String::from("code"), json!(fault.code));
    error.insert(String::from("message"), json!(fault.message));
    if let Some(data) = &fault.data {
        error.insert(String::from("data"), data.clone());
    }
    json!({ "jsonrpc": "2.0", "id": id, "error": error }).to_string()
}
