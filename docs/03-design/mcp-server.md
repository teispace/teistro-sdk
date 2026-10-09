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
| `chart.found` | a chart record: when, where, the sections and each family's record | step 2 |
| `almanac.days` | a panchanga record | step 2 |
| `time.resolve`, `calendar.convert` | a civil time and zone; a date | step 3 |
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
ABI and parity suites hold. Next come a chart record read from JSON and
`chart.found`.

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
- **The tool table and the façade agree.** Every request type with a
  `from_json` is either a tool or listed as not yet one with the step
  that makes it one, both ways, as `UNNAMED` holds the catalogue.

## 6. Order of work

1. The crate and the binary: the dual-era dispatcher over stdio, the
   step 1 tools, D5's settings, D7's errors, and the first two gates.
2. The chart record in the façade, the boundary moved onto it, and
   `chart.found` and `almanac.days`; the third gate.
3. Time, calendar and the engine passthrough.
4. Packaging with the release (Phase 9): the binary in the release
   assets, signed, with an install check and a page in the site's
   guides.

## Sources

- Model Context Protocol, specification revision 2026-07-28:
  changelog, versioning and compatibility, `server/discover`, tools
  (<https://modelcontextprotocol.io/specification/2026-07-28>).
- ADR-0004 (bindings), ADR-0020 (provenance), ADR-0029 (the engine
  plugged in like a transport); `QUESTIONS.md` Q35.
