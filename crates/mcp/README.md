# teistro-mcp

The Teistro SDK as tools an agent calls: a Model Context Protocol server
over stdio or Streamable HTTP (`docs/03-design/mcp-server.md`). A model asked for a
nakshatra answers from memory with no ayanamsha named; a tool answers
with the settings hash, the input hash and every convention applied.

Each release carries the binary for every platform the SDK ships, as
`teistro-mcp-{version}-{platform}.tar.gz` beside its digest in
`checksums.txt`, its bill of materials and its build attestation
(`gh attestation verify <file> --repo teispace/teistro-sdk`). Unpack it
and point the host at `teistro-mcp/teistro-mcp`. Or build it from a
tagged checkout:

```sh
cargo install --git https://github.com/teispace/teistro-sdk --tag vX.Y.Z teistro-mcp
```

A host's configuration names the program and its options. Through npm,
which installs the prebuilt program for the host it runs on
(`@teistro/mcp` and one `@teistro/mcp-<platform>` package):

```json
{ "mcpServers": { "teistro": { "command": "npx",
                               "args": ["-y", "@teistro/mcp", "--ephemeris", "BUILTIN"] } } }
```

or the unpacked program itself:

```json
{ "mcpServers": { "teistro": { "command": "/opt/teistro-mcp/teistro-mcp",
                               "args": ["--pack", "/opt/packs/readings.tpack"] } } }
```

`TEISTRO_MCP` names a program for the npm launcher to run instead of the
one npm installed.

For Claude Desktop on macOS or Windows, each release carries
`teistro-mcp-{version}.mcpb`: open it and Claude Desktop installs the
server, asking which ephemeris to compute with.

```sh
teistro-mcp                       # the built-in ephemeris
teistro-mcp --ephemeris SURYA_SIDDHANTA
teistro-mcp --plugin /opt/adapters/libteimeris_teistro.so   # a real engine, the built-in behind it
teistro-mcp --http 127.0.0.1:8080                            # Streamable HTTP at /mcp
```

A host starts it as a child process and speaks either revision:

- **2026-07-28**, stateless: each request carries
  `_meta["io.modelcontextprotocol/protocolVersion"]` and
  `_meta["io.modelcontextprotocol/clientCapabilities"]` (`{}` for none),
  each result names the server in `_meta`, and `server/discover`
  answers the revisions, the capabilities and the server's identity;
- **2025-11-25**, after `initialize`, for the rest of the process.

Each record tool takes `{request, profile, settings, locale}`. `request`
is the record its binding sends, refused by the same reader under the
same field names; `settings` is a patch over the profile, and
`settings.describe` answers its JSON Schema with every knob's
documentation. An answer is `{value, provenance}`. A refusal is a tool
error whose structured content is the SDK's error record, naming the
field, the range and a hint.

`chart.found` takes a whole chart request: the births (`instant` or
`instants`), the place and clock, the sections as `true` flags, and a
record for each table read off the charts, under the names every
binding writes (`kp`, `fortitudes`, `varsha`, …). It answers the chart
documents as `charts` and each table with a row a chart.

`almanac.days` takes a range of days: `first` and optional `last`, the
place and clock, and beside them a `muhurta` search, a `festivals`
reckoning, and `true` flags for the lunar `years`, the `eclipses` and
each day's `nepalSambat` date. It answers every day's panchanga, the
days' own hashes, and each section asked as its own
`{value, provenance}`.

`time.resolve`, `time.civil`, `time.convert` and `calendar.convert`
resolve a civil time in a zone to an instant, read an instant on a
zone's clock, carry an instant between UT1, TT and UTC with what was
applied, and write a date in another calendar with its weekday.

`--plugin PATH` loads an engine's adapter (`--plugin-config JSON` for
its own options) and computes with it first, the `--ephemeris` engine
behind it. `engine.manifest` answers what the engine offers,
`engine.call` calls one of its operations by name, and each operation
is also a tool of its own, `engine.<name>`, whose arguments are its
parameters. Every engine answer is sealed with a provenance naming the
engine.

Each tool's `inputSchema` is the record's JSON Schema, derived from the
types its reader reads. The list is lean by default: the records a
request carries by name (`chart.found`'s `kp`, `almanac.days`'s
`muhurta`) are each one line, and `schema.describe` answers a tool's
input and output schemas in full, or one part's:

```json
{"name": "schema.describe", "arguments": {"tool": "chart.found", "part": "kp"}}
```

`--schemas full` lists every schema inline, answers included, for a
client that validates structured content.

Beside the tools, the server answers resources, prompts and
completions:

- **Resources:** `teistro://catalogue` and `teistro://catalogue/{kind}`
  (each kind whole, members, attributes and sources), `teistro://profiles/{id}`,
  `teistro://settings/schema`, `teistro://document/schema`, and
  `teistro://tools/{tool}/schema`.
- **Prompts:** `birth-chart`, `day-panchanga` and `match` take a date,
  a time, a zone and a place as text and answer the call to make,
  already resolved and checked.
- **Completions:** a template's argument and a prompt's chart sections
  and festival packs.

A call whose `_meta` carries a `progressToken` hears
`notifications/progress` while it computes, and `notifications/cancelled`
stops a running call at its next engine request; a cancelled call is not
answered. A program embedding the server sends progress through
`Server::with_notify` and cancels from another thread through
`Server::interrupt`.

A request is bounded, and a request past a bound is refused naming its
field, the bound and the option moving it: a message's bytes
(`--max-message-bytes`), any array's members (`--max-items`) and the
days a record's ranges span (`--max-days`). Each takes `none`, and
`--help` gives the defaults. A program embedding the server sets them with
`Server::with_limits`:

```rust
use teistro_mcp::{Engine, Limits, Server};

let server = Server::new(Engine::Builtin)
    .with_limits(Limits { days: Some(732), ..Limits::default() });
```

Over HTTP (`--http ADDRESS`) the server answers 2026-07-28 statelessly:
each message is a POST to `/mcp`, answered as JSON, or as an event stream
with its progress first when it carries a `progressToken`; closing the
connection cancels the call. `MCP-Protocol-Version`, `Mcp-Method` and
`Mcp-Name` are checked against the body (`-32020` when they disagree),
a Base64-encoded `Mcp-Name` is decoded first, a browser's `Origin` must
be a loopback one or one named by `--allow-origin`, each peer address is
held to `--http-rate` messages a second (`429` past it), and each of `--http-workers` threads keeps a server
of its own. It checks no credentials: bind it to loopback, or put
whatever authorizes callers in front of it. A program embedding the
server serves the same way with `teistro_mcp::http::serve`.

`--pack PATH` (repeatable) loads an interpretation or locale pack into
every context, and every answer's provenance names the packs that shaped
its words. `--layouts PATH` and `--dashas PATH` (each a JSON array,
repeatable) register chart layouts and dasha systems of the operator's
own, which a chart request then names by key like the catalogue's;
`settings.describe` lists them. A program embedding the server adds tools of its own:

```rust
use serde_json::json;
use teistro_mcp::{Engine, Server, Tool};

let tool = Tool::new(
    "acme.greet",
    "Greets a querent by name.",
    json!({ "type": "object", "properties": { "name": { "type": "string" } } }),
    |call| Ok(json!({ "greeting": call.argument::<String>("name")? })),
)
.read_only();
let server = Server::new(Engine::Builtin).with_tool(tool)?;
# Ok::<(), teistro::Error>(())
```

A name under a namespace the SDK lists tools in (`chart.`, `engine.`, …)
is refused, so an SDK upgrade never shadows a program's tool; a tool
made `in_context` reads `profile`, `settings` and `locale` and reaches
that context through `Call::context`.

No tool reaches the file system or the network: the ephemeris, the
plugin and the packs are chosen on the command line and never by a tool
argument.
