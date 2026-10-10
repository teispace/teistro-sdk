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
//!     .handle(r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{
//!         "io.modelcontextprotocol/protocolVersion":"2026-07-28",
//!         "io.modelcontextprotocol/clientCapabilities":{}}}}"#)
//!     .expect("a request is answered");
//! assert!(reply.contains(r#""supportedVersions":["2026-07-28","2025-11-25"]"#));
//! ```

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use serde_json::{Map, Value, json};
use teistro::records::{Answered, Record};
use teistro::{Context, Ephemeris, Error, Loaded};
use teistro_core::envelope::canonical_json;
use teistro_core::settings::DEFAULT_PROFILE;
use teistro_core::strict;
use teistro_port_ephemeris::EphemerisProvider;
#[cfg(not(target_family = "wasm"))]
pub use teistro_port_ephemeris::load::Adapter;
use teistro_port_ephemeris::native::NativeFunction;

mod completion;
#[cfg(not(target_family = "wasm"))]
pub mod http;
mod limits;
mod prompts;
mod resources;
mod schemas;
mod tool;
mod tools;
mod watch;

pub use limits::Limits;
pub use schemas::Detail;
pub use tool::{Call, Handler, Tool};
pub use watch::{Interrupt, Notify};

/// The stateless revision the server speaks.
pub const MODERN: &str = "2026-07-28";
/// The handshake revision the server speaks after `initialize`.
pub const LEGACY: &str = "2025-11-25";
/// Every revision the server speaks, newest first.
pub const SUPPORTED: [&str; 2] = [MODERN, LEGACY];

const VERSION_META: &str = "io.modelcontextprotocol/protocolVersion";
const SERVER_INFO_META: &str = "io.modelcontextprotocol/serverInfo";
const CAPABILITIES_META: &str = "io.modelcontextprotocol/clientCapabilities";
/// Where a request carries the token its progress is reported under.
const PROGRESS_META: &str = "progressToken";
const SUBSCRIPTION_META: &str = "io.modelcontextprotocol/subscriptionId";

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

/// The reply to a message the strict reader refused: not JSON, answered
/// with a null id; or a key given twice, answered under the id a lenient
/// reading finds (none for a notification) and naming the key's path.
fn unreadable(message: &[u8], twice: Option<&Error>) -> Option<(String, Option<i64>)> {
    let lenient = match serde_json::from_slice::<Value>(message) {
        Ok(lenient) => lenient,
        Err(why) => {
            return Some(coded(
                &Value::Null,
                &Fault::new(PARSE_ERROR, format!("the message is not JSON: {why}")),
            ));
        }
    };
    let field = twice.and_then(Error::field).unwrap_or_default();
    let code = if field.starts_with("params") {
        INVALID_PARAMS
    } else {
        INVALID_REQUEST
    };
    let fault = Fault {
        code,
        message: format!("`{field}` is given twice; give each key once"),
        data: Some(json!({ "field": field })),
    };
    match lenient.get("id") {
        Some(id) => Some(coded(id, &fault)),
        None if lenient.get("method").is_some() => None,
        None => Some(coded(&Value::Null, &fault)),
    }
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

/// A pack loaded into every context the server builds: its bytes, and
/// what its first load reported.
struct Pack {
    bytes: Arc<[u8]>,
    loaded: Loaded,
}

/// What the operator adds to every context the server builds: its own
/// layouts and dasha systems, registered, and its packs, loaded after.
#[derive(Default)]
struct Own {
    layouts: Vec<teistro::Layout>,
    dashas: Vec<teistro::DashaDefinition>,
    packs: Vec<Pack>,
}

impl Own {
    /// A context built by `builder` with everything added in order: the
    /// one place the order lives.
    fn building(&self, builder: teistro::ContextBuilder) -> Result<Context, Error> {
        let builder = self
            .layouts
            .iter()
            .fold(builder, |builder, layout| builder.layout(layout.clone()));
        let builder = self.dashas.iter().fold(builder, |builder, definition| {
            builder.dasha_system(definition.clone())
        });
        let context = builder.build()?;
        for pack in &self.packs {
            context.intl().load_pack(&pack.bytes)?;
        }
        Ok(context)
    }

    /// What `settings.describe` says the operator added.
    fn described(&self) -> tools::Added<'_> {
        tools::Added {
            packs: self.packs.iter().map(|pack| &pack.loaded).collect(),
            layouts: self
                .layouts
                .iter()
                .map(|layout| layout.key.as_str())
                .collect(),
            dashas: self
                .dashas
                .iter()
                .map(teistro::DashaDefinition::key)
                .collect(),
        }
    }
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
    limits: Limits,
    /// What a transport cancels a running call through, and what the
    /// engine's watch reads.
    watch: Arc<watch::Shared>,
    /// The tool list, built on the first `tools/list`: it changes only
    /// with the binary and the engine, both fixed for the process.
    listed: Option<Map<String, Value>>,
    /// The layouts, dasha systems and packs every context adds.
    own: Own,
    /// The tools of the embedding program's own.
    tools: Vec<Tool>,
    contexts: HashMap<ContextKey, Context>,
}

impl core::fmt::Debug for Server {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Server")
            .field("engine", &self.engine)
            .field("plugin", &self.plugin.as_ref().map(|p| p.name.as_str()))
            .field("legacy", &self.legacy)
            .field("detail", &self.detail)
            .field("limits", &self.limits)
            .field("watch", &self.watch)
            .field("listed", &self.listed.is_some())
            .field("layouts", &self.own.layouts.len())
            .field("dashas", &self.own.dashas.len())
            .field("packs", &self.own.packs.len())
            .field("tools", &self.tools)
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
            limits: Limits::default(),
            watch: watch::Shared::new(),
            listed: None,
            own: Own::default(),
            tools: Vec::new(),
            contexts: HashMap::new(),
        }
    }

    /// The same server, `tool` listed and answered beside the SDK's
    /// records, in name order, under both revisions and both transports.
    /// A refusal is the tool's result with `isError`, as a record's is;
    /// the request bounds ([`Limits`]) hold for its arguments too.
    ///
    /// Call it after [`Server::with_engine`] or [`Server::with_plugin`],
    /// which build a server afresh.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `tool.name` for a name outside the protocol's
    /// rules, one already listed, or one under a namespace the SDK lists
    /// tools in (`chart.`, `engine.`, …), so an SDK upgrade can never
    /// quietly shadow a program's tool; naming the schema for one that is
    /// not an object schema, or that declares a context argument of a
    /// tool in context.
    pub fn with_tool(mut self, tool: Tool) -> Result<Server, Error> {
        let detail = self.detail;
        let operations = self.plugin.as_ref().map_or(&[][..], |p| &p.operations);
        let sdk: Vec<String> = tools::list(operations, &[], detail)
            .get("tools")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|listed| listed.get("name").and_then(Value::as_str))
            .map(str::to_owned)
            .collect();
        tool.check(&sdk, &self.tools)?;
        self.tools.push(tool);
        self.listed = None;
        Ok(self)
    }

    /// The same server, `bytes` loaded into every context it builds after
    /// the packs given before it, as `sdk.intl().load_pack` loads one: its
    /// entries laid over what stands, a later pack over an earlier one,
    /// and a locale it brings one a call may name. Every answer's
    /// provenance names the packs that shaped its words.
    ///
    /// The bytes are verified now, by loading them after the packs before
    /// them, so a file that is not a pack fails the server's start rather
    /// than a call.
    ///
    /// # Errors
    ///
    /// `PACK` naming `bytes`: bytes that are not a pack, a pack built for
    /// another catalogue, or one without metadata for a locale nothing
    /// loaded yet.
    pub fn with_pack(mut self, bytes: impl Into<Arc<[u8]>>) -> Result<Server, Error> {
        let bytes = bytes.into();
        let probe = self.own.building(Context::builder())?;
        let loaded = probe.intl().load_pack(&bytes)?;
        self.own.packs.push(Pack { bytes, loaded });
        // A context built before the pack would answer without it.
        self.contexts.clear();
        Ok(self)
    }

    /// What each pack reported when it was loaded, in the order given.
    #[must_use]
    pub fn packs(&self) -> Vec<&Loaded> {
        self.own.packs.iter().map(|pack| &pack.loaded).collect()
    }

    /// The same server, every context it builds drawing in `layout` too
    /// ([`teistro::ContextBuilder::layout`]): a regional chart the SDK does
    /// not ship, which `chart.found` draws when a request names its key
    /// (`chart_layout.ACME_ODIA`, or bare).
    ///
    /// Checked now, by building a context with it and every registration
    /// before it, so a layout the registry refuses fails the server's
    /// start rather than a call.
    ///
    /// ```
    /// use teistro_mcp::{Engine, Server};
    ///
    /// let mut layout = teistro::Layouts::new().get("EAST_INDIAN").cloned().expect("shipped");
    /// layout.key = String::from("ACME_ODIA");
    /// let server = Server::new(Engine::None).with_layout(layout)?;
    /// # let _ = server;
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// What the context's builder refuses, naming the layout by its place
    /// among the server's (`layouts[1].shape`): a row the shipped rules
    /// refuse, or a key the SDK ships or was given before.
    pub fn with_layout(mut self, layout: teistro::Layout) -> Result<Server, Error> {
        self.own.layouts.push(layout);
        self.registered()
    }

    /// The same server, every context it builds reading `definition` too
    /// ([`teistro::ContextBuilder::dasha_system`]): a dasha system of the
    /// operator's own, which `chart.found` computes when a request names
    /// its key (`dasha_system.ACME_SAPTAKA`, or bare).
    ///
    /// Checked now, as [`Server::with_layout`] checks a layout.
    ///
    /// # Errors
    ///
    /// What the context's builder refuses, naming the system by its place
    /// among the server's (`dashas[0].span`): a definition the shipped
    /// rules refuse, or a key the catalogue has or was given before.
    pub fn with_dasha_system(
        mut self,
        definition: impl Into<teistro::DashaDefinition>,
    ) -> Result<Server, Error> {
        self.own.dashas.push(definition.into());
        self.registered()
    }

    /// The server once a context builds with what it registers, every
    /// cached context dropped since none was built with the latest.
    fn registered(mut self) -> Result<Server, Error> {
        self.own.building(Context::builder())?;
        self.contexts.clear();
        Ok(self)
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

    /// The same server, each request held to `limits` rather than the
    /// shipped [`Limits::default`].
    #[must_use]
    pub fn with_limits(self, limits: Limits) -> Server {
        Server { limits, ..self }
    }

    /// The same server, sending each progress notification to `notify`
    /// as a line of JSON while a call carrying a `progressToken` runs.
    /// Without one, a token is accepted and nothing is sent.
    #[must_use]
    pub fn with_notify(self, notify: impl Fn(String) + Send + Sync + 'static) -> Server {
        self.watch.set_notify(Arc::new(notify));
        self
    }

    /// A handle that cancels a request while the server computes it, for
    /// the thread a transport reads its input on. A
    /// `notifications/cancelled` the server itself reads does the same.
    #[must_use]
    pub fn interrupt(&self) -> Interrupt {
        self.watch.interrupt()
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
        self.handle_bytes(message.as_bytes())
    }

    /// The reply to one JSON-RPC message as the bytes a transport read,
    /// or `None` for a notification. A message longer than the limit is
    /// refused unread, so a transport may stop reading one a byte past
    /// it.
    pub fn handle_bytes(&mut self, message: &[u8]) -> Option<String> {
        self.handle_coded(message).map(|(reply, _)| reply)
    }

    /// [`Server::handle_bytes`], with the code of the error the reply
    /// carries, which an HTTP transport answers under its own status.
    pub(crate) fn handle_coded(&mut self, message: &[u8]) -> Option<(String, Option<i64>)> {
        if let Some(most) = self
            .limits
            .message_bytes
            .filter(|most| message.len() > *most)
        {
            return Some(coded(
                &Value::Null,
                &Fault {
                    code: INVALID_REQUEST,
                    message: limits::too_long(most),
                    data: Some(json!({ "limit": most })),
                },
            ));
        }
        // Read strictly: a key given twice is one a lenient reader would
        // silently take the last of, so the server and its caller would
        // read two different requests.
        let read = std::str::from_utf8(message)
            .map_err(|_| None)
            .and_then(|text| strict::parse(text, "").map_err(Some));
        let message = match read {
            Ok(message) => message,
            Err(twice) => return unreadable(message, twice.as_ref()),
        };
        let Some(object) = message.as_object() else {
            return Some(coded(
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
                coded(
                    id,
                    &Fault::new(INVALID_REQUEST, "a request names its `method`"),
                )
            });
        };
        let empty = Value::Object(Map::new());
        let params = object.get("params").unwrap_or(&empty);
        // A notification asks for nothing back; a cancellation read here
        // is of a request not yet begun, or of one a transport's own
        // thread has already stopped.
        let Some(id) = id else {
            if method == "notifications/cancelled" {
                if let Some(request) = params.get("requestId") {
                    self.watch.interrupt().cancel(request);
                }
            }
            return None;
        };
        // An id is a string or an integer, never null (JSON-RPC 2.0 as
        // the revision restricts it).
        if !(id.is_string() || id.is_i64() || id.is_u64()) {
            return Some(coded(
                &Value::Null,
                &Fault::new(
                    INVALID_REQUEST,
                    "a request's `id` is a string or an integer",
                ),
            ));
        }
        let token = params.get("_meta").and_then(|meta| meta.get(PROGRESS_META));
        if let watch::Begun::Cancelled = self.watch.begin(id, token) {
            return None;
        }
        let answered = if method == "subscriptions/listen" {
            self.listen(id, params)
        } else {
            self.request(method, params)
        };
        let reply = match answered {
            Ok(result) => (success(id, &result), None),
            Err(fault) => coded(id, &fault),
        };
        // A cancelled call's answer is not sent: its caller has stopped
        // listening for it.
        (!self.watch.end()).then_some(reply)
    }

    fn request(&mut self, method: &str, params: &Value) -> Result<Value, Fault> {
        match method {
            "initialize" => Ok(self.initialize()),
            "ping" => Ok(json!({})),
            "server/discover" => {
                let era = self.era(params)?;
                Ok(complete(era, self.discover()))
            }
            "tools/list" => {
                let era = self.era(params)?;
                first_page(params)?;
                let detail = self.detail;
                let operations = self.plugin.as_ref().map_or(&[][..], |p| &p.operations);
                let list = self
                    .listed
                    .get_or_insert_with(|| tools::list(operations, &self.tools, detail))
                    .clone();
                Ok(complete(era, self.cacheable(era, list)))
            }
            "tools/call" => {
                let era = self.era(params)?;
                Ok(complete(era, self.call(params)?))
            }
            "resources/list" => {
                let era = self.era(params)?;
                first_page(params)?;
                Ok(complete(era, self.cacheable(era, resources::list())))
            }
            "resources/templates/list" => {
                let era = self.era(params)?;
                first_page(params)?;
                Ok(complete(era, self.cacheable(era, resources::templates())))
            }
            "prompts/list" => {
                let era = self.era(params)?;
                first_page(params)?;
                Ok(complete(era, self.cacheable(era, prompts::list())))
            }
            "prompts/get" => {
                let era = self.era(params)?;
                let limits = self.limits;
                let mut ask = |tool: &str, request: &Value| -> Result<Value, Error> {
                    let record = teistro::records::record(tool).ok_or_else(|| {
                        Error::internal(format!("this build carries no `{tool}`"))
                    })?;
                    let mut arguments = Map::new();
                    arguments.insert(String::from("request"), request.clone());
                    let mut answer = self.answer(record, &arguments)?;
                    Ok(answer.get_mut("value").map(Value::take).unwrap_or_default())
                };
                let written = prompts::get(params, &mut ask, &limits).map_err(|refused| Fault {
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
                Ok(complete(era, self.cacheable(era, contents)))
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
            Some(MODERN) => {
                // The revision requires every request to declare what its
                // client can do, and the server relies on nothing it does
                // not declare.
                let declared = params
                    .get("_meta")
                    .and_then(|meta| meta.get(CAPABILITIES_META))
                    .is_some_and(Value::is_object);
                if declared {
                    Ok(Era::Modern)
                } else {
                    Err(Fault {
                        code: INVALID_PARAMS,
                        message: format!(
                            "a {MODERN} request declares its client's capabilities in \
                             `_meta.{CAPABILITIES_META}`, `{{}}` for none"
                        ),
                        data: Some(json!({ "field": format!("_meta.{CAPABILITIES_META}") })),
                    })
                }
            }
            Some(LEGACY) => Ok(Era::Legacy),
            None if self.legacy => Ok(Era::Legacy),
            None => Err(Fault {
                code: INVALID_PARAMS,
                message: format!(
                    "the request names no revision: send `_meta.{VERSION_META}`, or \
                     `initialize` first"
                ),
                data: Some(json!({
                    "field": format!("_meta.{VERSION_META}"),
                    "supported": SUPPORTED,
                })),
            }),
            Some(version) => Err(Fault {
                code: UNSUPPORTED_VERSION,
                message: format!("the server does not speak revision {version}"),
                data: Some(json!({ "supported": SUPPORTED, "requested": version })),
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
            tools::describe(arguments, &self.own.described())
        } else if name == schemas::DESCRIBE {
            schemas::describe(arguments)
        } else if let Some(at) = self.tools.iter().position(|tool| tool.name == name) {
            if let Err(refusal) = self.limits.check_fields(arguments) {
                return Ok(tool_result(refused(&refusal), true));
            }
            self.own(at, arguments)
        } else if let Some(operation) = self.operation(name) {
            if let Err(refusal) = self.limits.check_fields(arguments) {
                return Ok(tool_result(refused(&refusal), true));
            }
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

    /// A tool of the program's own answered: in the context its scope
    /// names when it reads one. A panic inside its handler is the tool's
    /// failure, and the contexts it unwound through are not reused.
    fn own(&mut self, at: usize, arguments: &Map<String, Value>) -> Result<Value, Error> {
        let Some(tool) = self.tools.get(at).cloned() else {
            return Err(Error::internal("a tool the server lists is gone"));
        };
        let watch = Arc::clone(&self.watch);
        let (context, rest) = if tool.context {
            let (scope, rest) = tools::Arguments::scope(arguments)?;
            (Some(self.context(&scope)?), rest)
        } else {
            (None, arguments.clone())
        };
        let call = Call {
            name: &tool.name,
            arguments: &rest,
            context,
            watch: &watch,
        };
        let answered =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (tool.handler)(&call)));
        answered.unwrap_or_else(|_| {
            self.contexts.clear();
            Err(Error::internal(format!(
                "the tool `{}` failed inside",
                tool.name
            )))
        })
    }

    /// A record tool's answer: the arguments read, the context they name,
    /// and the record answered under it as the envelope.
    fn answer(&mut self, record: Record, arguments: &Map<String, Value>) -> Result<Value, Error> {
        if let Some(request) = arguments.get("request") {
            self.limits.check(request)?;
        }
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
                let watch = Arc::clone(&self.watch);
                let mut builder = Context::builder()
                    .profile(profile.as_str())
                    .ephemeris(chain)
                    .wrapping(move |opened| -> Box<dyn EphemerisProvider> {
                        Box::new(watch::Watched::new(opened, watch))
                    });
                if !settings.is_empty() {
                    builder = builder.settings_json(settings.as_str());
                }
                // The locale after the packs, since a pack may bring it.
                let context = self.own.building(builder)?;
                if !locale.is_empty() {
                    context.intl().set_locale(locale.as_str())?;
                }
                entry.insert(context)
            }
        })
    }
}

impl Server {
    /// The modern revision's discovery answer.
    fn discover(&self) -> Map<String, Value> {
        let answer = json!({
            "resultType": "complete",
            "supportedVersions": SUPPORTED,
            "capabilities": capabilities(),
            "instructions": INSTRUCTIONS,
            "ttlMs": self.ttl_ms(),
            "cacheScope": "public",
        });
        answer.as_object().cloned().unwrap_or_default()
    }

    /// How long a caller may keep what the server lists: a day for the
    /// shipped server, whose lists change only with the binary, and an
    /// hour once a plugin is loaded, since a host in front may outlive a
    /// restart under another one.
    fn ttl_ms(&self) -> u64 {
        if self.plugin.is_some() {
            TTL_MS / 24
        } else {
            TTL_MS
        }
    }

    /// A subscription (`subscriptions/listen`): nothing the server lists
    /// changes while it runs, so it honours no notification type, says so
    /// in the acknowledgment sent first, and ends the subscription at once
    /// with its graceful closure, rather than holding open a stream that
    /// would never carry anything. Both carry the request's id as the
    /// subscription's.
    fn listen(&self, id: &Value, params: &Value) -> Result<Value, Fault> {
        let era = self.era(params)?;
        if era == Era::Legacy {
            return Err(Fault::new(
                METHOD_NOT_FOUND,
                format!("`subscriptions/listen` is a {MODERN} method"),
            ));
        }
        self.watch.say(
            json!({
                "jsonrpc": "2.0",
                "method": "notifications/subscriptions/acknowledged",
                "params": {
                    "_meta": { SUBSCRIPTION_META: id },
                    "notifications": {},
                },
            })
            .to_string(),
        );
        let mut closed = Map::new();
        closed.insert(String::from("_meta"), json!({ SUBSCRIPTION_META: id }));
        Ok(complete(era, closed))
    }

    /// A result every caller may keep for [`Server::ttl_ms`]. The
    /// handshake revision has no caching fields.
    fn cacheable(&self, era: Era, mut result: Map<String, Value>) -> Map<String, Value> {
        if era == Era::Modern {
            result.insert(String::from("ttlMs"), json!(self.ttl_ms()));
            result.insert(String::from("cacheScope"), json!("public"));
        }
        result
    }
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
        "description": "The Teistro astrology SDK as tools: charts, almanacs, dashas, \
                        matching and more, each answer carrying its settings and inputs",
        "websiteUrl": "https://github.com/teispace/teistro-sdk",
    })
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
        // Every modern result says which server answered it, since no
        // handshake said so first.
        if let Value::Object(meta) = result
            .entry(String::from("_meta"))
            .or_insert_with(|| Value::Object(Map::new()))
        {
            meta.entry(SERVER_INFO_META).or_insert_with(server_info);
        }
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

/// A refusal and its code.
fn coded(id: &Value, fault: &Fault) -> (String, Option<i64>) {
    (failure(id, fault), Some(fault.code))
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
