# `teistro-mcp`: the SDK as tools an agent calls

Status: `building`, decided 2026-10-10; §6 steps 1 to 3 and §7 P1 to P6,
P8 and P10 built, P7 and P9 open, §6 step 5 (packaging) not built. Closes Q35 (`QUESTIONS.md`), which the
maintainer deferred to the end of the plan and then handed over to be
researched and decided (2026-10-07). The order of work is §6.

A model asked for a nakshatra answers from memory, with no ayanamsha
named. A tool that computes it answers with the settings hash, the input
hash and every convention applied (ADR-0020), which is the part a model
drops. This page is how the SDK becomes such a tool without becoming a
second product.

## 1. The protocol, as it stands

The Model Context Protocol's current revision is **2026-07-28**. It
removed the session: there is no `initialize` handshake, every request
carries its protocol version, client identity and capabilities in
`_meta` (`io.modelcontextprotocol/protocolVersion`, `…/clientInfo`,
`…/clientCapabilities`), a server **must** answer `server/discover`, and
every result carries `resultType` (`"complete"` for an ordinary one). A
version the server does not speak is `UnsupportedProtocolVersionError`,
code −32022, with the versions it does. `tools/list` results are
cacheable and carry `ttlMs` and `cacheScope`, and should list tools in a
deterministic order. `ping`, logging and the server-initiated requests
went; logs go to stderr on stdio. A tool failure the model can act on is
a result with `isError: true`, not a JSON-RPC error; an unknown tool or a
malformed request is −32602.

Clients still speak **2025-11-25**, the handshake revision: `initialize`,
`notifications/initialized`, `ping`, and results with none of the new
fields. The revision calls a server that answers both *dual-era* and
allows it: a request carrying the modern `_meta` is served statelessly,
and `initialize` selects the legacy revision for the rest of the stdio
process.

Sources: the specification's changelog, its versioning page, and its
`server/discover` and tools pages (modelcontextprotocol.io,
`specification/2026-07-28`).

## 2. Decisions

**D1. A local stdio binary first, dual-era.** `teistro-mcp` runs as a
child process of the agent's host, as every desktop and coding agent
starts a local server, and speaks both 2026-07-28 and 2025-11-25. A
hosted HTTP service is not this project's to run; because the modern
revision is stateless, the same dispatcher answers Streamable HTTP when
someone wants to host one, so nothing here forecloses it.

**D2. It calls the Rust façade, not the C ABI.** The server is Rust and
the façade is the SDK's Rust API (`rust-consumer-surface.md`); going
through `ts_*` would mean decoding blobs back into the values the façade
already returns. Q35's "a sixth emitter" was written when the C ABI was
the only surface; it is the right shape for a server in another
language and the wrong one for a Rust server.

**D3. A tool is a request record the façade already reads.** Every area
that a binding reaches with one JSON record (`ResearchRequest`,
`PakshiRequest`, `RashifalBatch`, `NumerologyRequest`, `NaamRequest`)
is a tool whose arguments are that record, refused by the same reader
under the same field names. So the tool list is not a second list of
the SDK's operations to keep equal by hand: a tool is a row naming an
existing reader, and a gate (§5) holds the rows to the façade's request
types both ways.

**D4. Every answer is the envelope.** `structuredContent` is the
answer's `{value, provenance}`, exactly the canonical JSON a binding
reads, and the text content is the same JSON, as the revision advises
for older clients. Nothing is summarised: a summary is a reading, and
the readings are the SDK's to make (`interpret`).

**D5. Each call names its settings.** A stateless protocol cannot rely
on a context opened earlier, so every tool takes optional `profile`,
`settings` (a patch over the profile, as `ContextBuilder::settings_json`
reads it) and `locale` beside its record. The server keeps the contexts
it has built keyed by those three, which is a cache and not state: no
answer depends on what was asked before. `settings.describe` answers
the shipped profiles and the settings patch's JSON Schema, derived by
schemars from the patch type itself, so every knob comes with the
documentation its field carries; an agent finds a setting rather than
guessing one, and the `settings` argument of every tool is checked by
the same schema.

**D6. Nothing reaches the file system or the network.** A provider
plugin is a library the process loads, so it is chosen by the server's
command line (`--plugin PATH`, with `--ephemeris` the fallback behind
it), never by a tool argument: an argument naming a path would let a
model load code. The built-in ephemeris is the default. Every tool is annotated read-only, idempotent and closed
world.

**D7. A refusal is a tool error the model can correct.** The SDK's
refusals already name their field, the range and a hint
(`developer-experience-mandate`); the server returns them as `isError`
results whose structured content is `{status, field, message, hint}`,
so a model retries with the field fixed. JSON-RPC errors are kept for
the protocol's own failures.

## 3. The tools, by step

| tool | the record | built |
|---|---|---|
| `research.study` | `ResearchRequest` | step 1, built |
| `almanac.pakshi` | `PakshiRequest` | step 1, built |
| `chart.rashifal` | `RashifalBatch` | step 1, built |
| `numerology.profile` | `NumerologyRequest` | step 1, built |
| `matching.naam` | `NaamRequest` | step 1, built |
| `settings.describe` | none: the knobs and the profiles | step 1, built |
| `chart.found` | `FoundRequest`: when, where, the sections and each family's record | step 2, built |
| `almanac.days` | `DaysRequest`: a range of days, the place and clock, and what is asked beside them | step 2, built |
| `time.resolve` | `ResolveRequest`: a civil date and time and a zone | step 3, built |
| `time.civil` | `CivilRequest`: an instant, a zone and a calendar | step 3, built |
| `time.convert` | `ScaleRequest`: an instant and two scales | step 3, built |
| `calendar.convert` | `CalendarRequest`: a date and the calendar wanted | step 3, built |
| `engine.manifest`, `engine.call` | `ManifestRequest`, `EngineCall`: the engine passthrough (ADR-0030) | step 3, built |
| `engine.<name>` | one per operation a `--plugin` engine's manifest lists | step 3, built |

Step 2 needed the chart's composition in the façade. Until it was built,
`ts_chart_found` composed a chart's families inside the boundary crate
(`crates/ffi/src/chart.rs`), beside the blob it encodes, so a `chart.found`
tool would have been a second copy of that composition. The
composition moved into `teistro` behind a chart record that reads from
JSON, the boundary reads its C request into the same record, and both
the boundary and the server call it, which removed the copy the
boundary kept.

**Built so far (2026-10-10).** `teistro::ChartRecords` holds the
records a chart request carries beside its sections, each read by its
own reader. `ChartArea::compose` founds the charts, widens the request
by what the records read off them, and answers every section as
`Composed`. Today that covers the rules and plans, Sade Sati, transits,
hits, dignities, fortitudes, lots, considerations, perfection,
matching, SVG, KP, prashna, remedies, Lal Kitab, rectification, the
annual charts (`varsha`) and the Western tables. The boundary reads its
C strings into the records and encodes what `compose` answers, so it
composes no section of its own; the blob did not move a byte, which the
ABI and parity suites hold.

`ChartRecords::read` is the one place a record's name meets its reader:
`ChartRecords::NAMES` lists the names every binding writes, and the
boundary reads each C field through it, so a record's reader is named
once. `teistro::FoundRequest` reads a whole chart request from JSON,
spelt as the other request records are: `instant` or `instants`, the
place and clock, the catalogue keys, a `true` flag for each section and
each record by its name. `chart.found` is that record: it answers the
chart documents as `charts` and each table a row a chart, with the
batch's provenance as the answer's.

`teistro::DaysRequest` reads a range of days the same way: `first` and
optional `last` in a `calendar`, the place and clock, a `muhurta` and a
`festivals` record by their names, and `true` flags for the lunar
`years`, the `eclipses` and each day's `nepalSambat` date.
`AlmanacAnswer::sections` writes every section asked beside the days in
full and seals each over what it writes; the boundary's
`ts_panchanga_days` encodes those sections and `almanac.days` answers
them, so the two share one writer. `almanac.days` answers the days,
their own hashes and each section as its own `{value, provenance}`,
with the days' provenance as the answer's; a section not asked is left
out. The third gate holds the table (§5).

Step 3's time and calendar tools read a record each, spelt as the others
are: a zone as `ZoneSpec` serialises, which is how a stored chart keeps
it, and a scale by its key. They answer without an envelope, as
numerology does, because each answer already names what produced it:
the zone resolution carries the database's version and what each policy
did, and a conversion carries the Delta T with its model and source.
The engine passthrough has one loader. `load::Adapter` in the port
(feature `load`, native only) opens an adapter, and each provider it
binds holds the library open itself, so the C boundary lost its
separate keep-alive slot and a handle and its contexts drop in any
order. `ts_provider_load` and `--plugin` both call it. The server
computes with the plugin first and `--ephemeris` behind it, lists each
operation the engine's manifest names as `engine.<name>` with its
supplied parameters as the input schema (an out parameter is the
engine's to fill; a name an SDK tool holds stays reachable through
`engine.call`), and seals every engine answer with a provenance naming
the engine and the operation as its step. `crates/test-adapter` is the
analytic test provider as a real shared library, so the loader, the
boundary and the server each load one on every push, where before only
a machine with Teimeris built did.

## 4. What is not a tool

The lifetime plumbing (`ts_context_new`, the frees, `last_error`), the
pack loader (it takes bytes a model has no business choosing), the
plugin loader (D6), and the blob views. A layout of one's own and a
dasha system of one's own are context configuration, reached through
`settings` when they become knobs and not before.

## 5. Gates

- **Both eras, end to end.** An integration test starts the binary,
  speaks 2026-07-28 (`server/discover`, `tools/list`, a call, an
  unsupported version) and then, in a fresh process, 2025-11-25
  (`initialize`, `notifications/initialized`, `tools/list`, a call), and
  holds every result to the revision's shape.
- **A tool answers as the façade does.** Each tool is called with the
  record the shared example of its area sends, and its structured
  content is compared with the façade's own answer to the bit, input
  hash included.
- **The tool table and the façade agree.** Every JSON record reader in
  the SDK's crates (an `impl T` holding `pub fn from_json`) is reached by
  a tool: called where a tool reads its request (`teistro::records`, a
  chart request's records, a range of days, the server's arguments),
  carried inside a record that is, or listed as no tool's with why. Both
  ways: a listed type a tool reads by its own call, or one with no
  reader any more, fails. Built as the `every-reader-reaches-a-tool`
  lint, which found two readers carried rather than called on its first
  run and was shown red by dropping the days request from its list.

## 6. Order of work

1. The crate and the binary: the dual-era dispatcher over stdio, the
   step 1 tools, D5's settings, D7's errors, and the first two gates.
2. The chart record in the façade, the boundary moved onto it, and
   `chart.found` and `almanac.days`; the third gate.
3. Time, calendar and the engine passthrough.
4. The rest of the protocol (§7): each tool's schemas first, then
   resources, completions and prompts, the limits, progress and
   cancellation, and the HTTP transport.
5. Packaging with the release (Phase 9): the binary in the release
   assets, signed, with an install check and a page in the site's
   guides.

## 7. The rest of the protocol

Tools are where the protocol starts, not where the server stops. Each
of the 2026-07-28 revision's features (§1 sources, its changelog) is
taken up below or declined with its reason, so the server tracks the
revision rather than a subset of it.

**P1. Each tool states its record and its answer.** A tool's
`inputSchema` gives `request` the record's own JSON Schema, derived by
schemars from the reader's types (a `schema` feature on `teistro`, as
the chart document's types carry one), and its `outputSchema` gives the
envelope with the answer's schema where the answer's types carry one.
The schema is never stricter than the reader (`document-schema.md`'s
rule): a gate sends every tool's example through both and holds that
what the reader takes the schema takes. A model then fills fields it
reads rather than fields it guesses from prose.

*Built.* Every record tool states both; a part whose reader reads no
single type (`chart.found`'s `rules`, a dasha year) is written by hand
beside the type, and a gate lists any left unstated, so none is. The
lint that holds a serialised type to carrying a schema also holds a
field serialised `with` a module to naming the schema it writes. The
whole list measured nearly a megabyte, most of it `chart.found` and its
answer, which is more than a model should read to pick a tool. So the
list is **lean** by default: each record a request carries by name
(`chart.found`'s `kp`, `almanac.days`'s `muhurta`) is one line naming
`schema.describe`, the definitions nothing then reaches are dropped,
and no `outputSchema` is listed (the revision makes it optional).
`schema.describe` answers a tool's input and output schemas in full, or
one part's with the definitions it reaches. `--schemas full` lists
everything inline for a client that validates structured content. The
lean list is about a fourteenth of the full one, a gate holds it under
a quarter, and the server builds it once and keeps it. Every example
holds to both details, and every chart record its reader takes holds to
the full schema and to the part `schema.describe` answers.

**P2. Resources for what an agent reads rather than computes.** The
catalogue (`teistro://catalogue/{kind}`, a resource template), the
shipped profiles and the settings schema (`teistro://profiles/{id}`,
`teistro://settings/schema`), and the chart document's JSON Schema.
Each is a `CacheableResult` with `ttlMs` and `cacheScope: "public"`:
nothing in them varies by caller.

*Built.* `resources/list` names the settings schema, the document
schema, the catalogue's index, each shipped profile and each catalogue
kind, every one written once per process; the build script carries
`catalogue/catalogue.json` in a kind at a time, so a kind's text is
the file's own. `resources/templates/list` names three templates,
`teistro://catalogue/{kind}`, `teistro://profiles/{id}` and
`teistro://tools/{tool}/schema` (what `schema.describe` answers, kept
once a tool). A resource the server lacks is refused, never an empty
`contents`: `-32602` with the URI under 2026-07-28 and `-32002` under
2025-11-25, as each revision says. Every list is one page, so a cursor
is refused. Gated: every listed resource reads as its list gives it,
every kind the binary compiles is served under its name and number and
none other, and each resource is the answer its tool gives.

**P3. Completions.** `completion/complete` answers a resource
template's `{kind}` and `{id}` and a prompt's arguments from the
catalogue and the profiles, so a client offers the keys the reader
takes.

*Built.* One table holds each template's argument and the values it
takes, and both the template list and the completion read it, so a
completion cannot offer what a template does not take. Values rank by
prefix, then substring, then letters in order, ignoring case. A
prompt's argument completes from its table where one holds it (a
chart's sections, a list completing its last item from those not yet
named; the festival packs) and offers nothing for a date, a time or a
place. Gated: every value a template's completion offers reads.

**P4. Prompts as worked requests.** A prompt is a request the SDK
answers well, written out: a birth chart with its sections, a day's
panchanga at a place, a match between two births. Its arguments are
the fields a person supplies (date, time, place); its messages name
the tool and the record, so an agent learns the record from one that
works.

*Built.* `birth-chart`, `day-panchanga` and `match` read a date, a
time, a zone (an IANA name, a fixed offset or LMT) and a place as text.
`prompts/get` resolves each civil time through `time.resolve` and reads
the record it writes with the tool's own reader before answering, so a
prompt is never a call that fails; its messages give the call as JSON
and link the tool's schemas. A refusal names the prompt's argument, not
the record's field: a thirteenth month is `date`, a latitude of 97 is
`latitude`. Gated: the call each prompt writes is made and answers, a
birth chart's instant is the one `time.resolve` answers, and each
required argument left out is refused by its name.

**P5. Limits.** A request's text, a batch's length and a range of days
are bounded, and the bound is a refusal naming the field and the
limit, not a slow answer: a server a model drives must not be made to
compute a century of days by one typo.

*Built.* `Limits` bounds a message's bytes (`--max-message-bytes`),
the members of any array in a record (`--max-items`) and the days a
record's ranges span (`--max-days`), summed over every object holding
`first` and `last` as dates; each is `None`, or `none` on the command
line, for unbounded, and `Limits::default` states the shipped bounds.
They were chosen by timing the release binary: the shipped batch is
found in seconds and the shipped year of days in well under a minute,
where a century of days would take the better part of half an hour. A
window of life (progressions' `from` and `to`)
is not a range of days and is not counted, since it spans decades by
design. A record past a bound is a `LIMIT` naming the field by its path
(`periods[1].last`, `instants`), the bound and the option; a prompt
writes no such call and names its own argument. The transport stops
reading a line a byte past the message bound and refuses it with a null
id, then reads the next. Gated: each record reading days answers at the
bound and is refused a day past it, the ranges are summed, a window of
life passes, an array is named by its path, and a line past the bound
is refused while the next one answers.

**P6. Cancellation and progress.** `notifications/cancelled` stops a
batch between items, and a call carrying a `progressToken` hears
`notifications/progress` per item on its own response stream.

*Built.* The watch sits on the engine rather than in the SDK's loops:
`ContextBuilder::wrapping` lays a provider over whichever one a chain
opens, under whatever the settings name, and the server's own forwards
every request after a check. Every computation reaches the engine, so a
batch of charts, a range of days, a study and an adapter's own
operations all stop at their next position once cancelled, with no
hook threaded through any loop. The stdio transport reads its input on
a thread of its own and hands a `notifications/cancelled` to
`Server::interrupt`'s handle as it arrives; a cancelled call's answer
is not sent, and a request cancelled before it begins is not answered.
A call whose `_meta` carries a `progressToken` hears
`notifications/progress` while it computes: the engine requests
answered so far, with no `total`, since a call does not know its
count beforehand, and at most one every quarter of a second. Over
Streamable HTTP the 2026-07-28 revision cancels by closing the stream,
which P8 maps to the same handle. Gated through the binary: progress
increases under its token before the answer and an untokened call hears
none, a call cancelled mid-computation is never answered and stops
sooner than a whole call takes, and a cancellation before a request
begins is spent on it once.

**Conformance.** Read against the revision's own pages: a modern request
missing its revision or its client's capabilities is `-32602` naming the
`_meta` field (an unknown revision stays `-32022`), and every modern
result carries `_meta["io.modelcontextprotocol/serverInfo"]`, with the
server's description and website. `server/discover` names its revision
as every request does, a request whose id is null or neither a string
nor an integer is `-32600`, and `subscriptions/listen` is acknowledged
first with an empty filter, since nothing the server lists changes
while it runs, then closed gracefully under the request's id (over HTTP,
both on one event stream). A plugin operation whose name the revision's
tool-name rule refuses is reached through `engine.call` only, and the
lists' cache lifetime falls from a day to an hour once a plugin is
loaded. Gated in `tests/eras.rs`.

Two of the revision's recommendations are declined on purpose. The
stdio transport answers every request it read before its input closed
rather than stopping the running one, since a host that pipes its
requests and closes its end is waiting for the answers. And the HTTP
transport implements no authorization: it checks no credentials,
warns when bound beyond loopback, and leaves callers to whatever stands
in front.

**P7. Long work as a task.** A research study over a large corpus is
the case for the `io.modelcontextprotocol/tasks` extension: the call
answers a task handle, `tasks/get` polls it. Taken up when a study is
measured to outlast a client's timeout, not before.

**P8. Streamable HTTP, stateless.** The modern revision has no session,
so the same dispatcher answers HTTP POST: `Mcp-Method`/`Mcp-Name`
headers checked against the body (`HeaderMismatchError`), `Origin`
validated, bound to loopback unless told otherwise, and authorization
left to the host in front of it (the server holds no secret).
`teistro::Context` is neither `Send` nor `Sync`, so a concurrent HTTP
transport builds a context per worker thread.

*Built.* `--http ADDRESS` serves `/mcp` from `teistro_mcp::http::serve`
on the standard library's sockets alone, so the transport adds no
dependency to audit. Each worker thread builds a server of its own and
answers one connection at a time; a connection the workers cannot take
is `503`. A POST carries one message: the revision header must be
2026-07-28 (another is `-32022`) and agree with the body, and
`Mcp-Method` and, for `tools/call`, `prompts/get` and `resources/read`,
`Mcp-Name` must say what the body does, or the request is `-32020`
under `400` before anything is computed. A notification is `202`, an
unknown method `404`, a foreign `Origin` `403`, `GET` and `DELETE` `405`
(a stateless server opens no stream and keeps no session), a body past
the message bound `413` unread. A call carrying a `progressToken`
answers as an event stream, its progress first and its answer last,
with `X-Accel-Buffering: no` so a proxy does not hold it; any other
answers as JSON. Each peer address may send fifty messages a second
with a burst of twice that unless `--http-rate` says otherwise, counted
once a message is read so its `429` reaches the peer; an `Mcp-Name` in
the revision's `=?base64?…?=` form is decoded before it is compared. A
worker watches the connection while its call
computes and stops the call through the same watch as P6 when the peer
closes, and a closure after the answer cancels nothing later. Gated in
`tests/http.rs` through the binary on a loopback port, each check shown
red by breaking it.

**P9. Extended where the SDK is.** The server reaches what a context
registers, not only what ships: a provider plugin by `--plugin`, an
interpretation pack by `--pack`, a plugin's own functions as tools of
their own and through `engine.call` (step 3), and a context's registered
dasha systems and layouts by their keys. A program embedding
`teistro_mcp::Server` adds tools of its own beside the records
(`Server::with_tool`), answered and listed as the SDK's are, so a
product built on the SDK serves its own operations without forking the
server. A new SDK record is one row in `teistro::records`, and the
server lists it with nothing else changed.

*Built: packs and a program's tools.* `Server::with_pack` (and
`--pack PATH`, repeatable, read once at start) loads a pack into every
context the server builds, in the order given, before the call's locale
is set, since a pack may bring the locale; a file that is not a pack
fails the start, and `settings.describe` lists each pack's locale,
namespaces, entries and digest. Building it found `Provenance.packs`
declared and never filled, so two consumers with different readings got
the same provenance for different words; `Context::pack_stamps` now
fills it on every answer stamped through the context and on every
interpreted chart, which is where a pack's words are said, by locale,
namespaces and digest.

`Server::with_tool` takes a `Tool` (a name, a description, an object
input schema and a handler, with a title, an output schema, read-only
annotations and `in_context` beside them). A name outside the
protocol's tool-name rule, one already listed, or one under a namespace
the SDK lists tools in (`chart.`, `engine.`, …, computed from the list
itself) is refused at registration, so an SDK upgrade can never quietly
shadow a program's tool. A tool `in_context` reads `profile`, `settings`
and `locale` as a record tool does and its handler reaches that context,
with the server's engine, watch and packs, through `Call::context`. The
request bounds hold for its arguments, a refusal its handler gives is
the tool's result with `isError`, and a handler that panics is the
tool's `INTERNAL` refusal while the server answers the next call.
Gated in `tests/packs.rs` and `tests/extend.rs`.

Not built: a registered dasha system or layout named in a request. The
JSON readers take catalogue members, so a registered key is refused
before any context sees it; the registration readers sit in the C
boundary and move into the façade first, as the chart's composition
did.

**P10. Quick by construction.** A context is built once per profile,
settings and locale, the settings keyed by their canonical JSON so two
spellings of one patch share it, and the cache is bounded. A tool list
and every resource is computed once per process. A batch is one call
(a grid is one request, `count-the-cost-the-resource-pays`), so an
agent asking for a year of charts pays for founding them, not for
thousands of round trips.

**Declined.** Sampling, roots and logging are deprecated in this
revision; the server needs none (D6: it reads no file the model
names). `input_required` (multi round-trip) has no use while every
request is complete in one record: a missing field is a refusal the
model corrects (D7), which costs one round trip either way.

## Sources

- Model Context Protocol, specification revision 2026-07-28:
  changelog, versioning and compatibility, `server/discover`, tools
  (<https://modelcontextprotocol.io/specification/2026-07-28>).
- ADR-0004 (bindings), ADR-0020 (provenance), ADR-0029 (the engine
  plugged in like a transport); `QUESTIONS.md` Q35.
