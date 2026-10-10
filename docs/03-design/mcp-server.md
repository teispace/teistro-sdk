# `teistro-mcp`: the SDK as tools an agent calls

Status: `decided`, 2026-10-10; step 1 built. Closes Q35 (`QUESTIONS.md`), which the
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
command line (`--ephemeris`), never by a tool argument: an argument
naming a path would let a model load code. The built-in ephemeris is
the default. Every tool is annotated read-only, idempotent and closed
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
| `engine.manifest`, `engine.call` | the engine passthrough (ADR-0029) | step 3 |

Step 2 needs the chart's composition in the façade. Today
`ts_chart_found` composes a chart's families inside the boundary crate
(`crates/ffi/src/chart.rs`), beside the blob it encodes; a `chart.found`
tool written now would be a second copy of that composition. The
composition moves into `teistro` behind a chart record that reads from
JSON, the boundary reads its C request into the same record, and both
the boundary and the server call it, which removes the copy the
boundary keeps today.

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
The engine passthrough is next: the plugin loader moves out of the C
boundary into the port, so the SDK, the boundary and this server load
an adapter the same way, and `--plugin` names one.

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

**P2. Resources for what an agent reads rather than computes.** The
catalogue (`teistro://catalogue/{kind}`, a resource template), the
shipped profiles and the settings schema (`teistro://profiles/{id}`,
`teistro://settings/schema`), and the chart document's JSON Schema.
Each is a `CacheableResult` with `ttlMs` and `cacheScope: "public"`:
nothing in them varies by caller.

**P3. Completions.** `completion/complete` answers a resource
template's `{kind}` and `{id}` and a prompt's arguments from the
catalogue and the profiles, so a client offers the keys the reader
takes.

**P4. Prompts as worked requests.** A prompt is a request the SDK
answers well, written out: a birth chart with its sections, a day's
panchanga at a place, a match between two births. Its arguments are
the fields a person supplies (date, time, place); its messages name
the tool and the record, so an agent learns the record from one that
works.

**P5. Limits.** A request's text, a batch's length and a range of days
are bounded, and the bound is a refusal naming the field and the
limit, not a slow answer: a server a model drives must not be made to
compute a century of days by one typo.

**P6. Cancellation and progress.** `notifications/cancelled` stops a
batch between items, and a call carrying a `progressToken` hears
`notifications/progress` per item on its own response stream.

**P7. Long work as a task.** A research study over a large corpus is
the case for the `io.modelcontextprotocol/tasks` extension: the call
answers a task handle, `tasks/get` polls it. Taken up when a study is
measured to outlast a client's timeout, not before.

**P8. Streamable HTTP, stateless.** The modern revision has no session,
so the same dispatcher answers HTTP POST: `Mcp-Method`/`Mcp-Name`
headers checked against the body (`HeaderMismatchError`), `Origin`
validated, bound to loopback unless told otherwise, and authorization
left to the host in front of it (the server holds no secret).

**P9. Extended where the SDK is.** The server reaches what a context
registers, not only what ships: a provider plugin by `--ephemeris`, an
interpretation pack by `--pack`, a plugin's own functions through
`engine.call` (step 3), and a context's registered dasha systems and
layouts by their keys. A program embedding `teistro_mcp::Server` adds
tools of its own beside the records (`Server::with_tool`), answered and
listed as the SDK's are, so a product built on the SDK serves its own
operations without forking the server. A new SDK record is one row in
`teistro::records`, and the server lists it with nothing else changed.

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
