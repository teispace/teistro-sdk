# Fuzzing

Status: `built`, 2026-10-12; the `mf2` and `blob` targets 2026-10-23.
The quality bar's fuzzing row ([`01-quality-bar.md`](01-quality-bar.md)):
pack parsers, the MF2 engine, the blob decoders and every C ABI entry
point survive arbitrary bytes, with a committed corpus, a nightly smoke
run and a weekly long run.

## 1. What is fuzzed, and why there

A binding hands the library bytes it did not check. Everything a caller
can say crosses the C ABI, so the ABI is the surface: a target there
reaches the JSON readers, the pack loader, the key parser and the
computation behind each, through the same guards a binding goes
through. Fuzzing the crates beneath instead would test code no caller
reaches as written, and miss the boundary's own copying and checks.

| target | entry points | input |
|---|---|---|
| `json` | every entry point that reads a caller's JSON: the 26 sections of `ts_chart_request`, `ts_research`, `ts_pakshi`, `ts_rashifal`, `ts_numerology_profile`, `ts_naam_milan`, `ts_panchanga_days`'s muhurta and festivals, `ts_intl_render`'s parameters, and the context's settings, layouts and dashas | a selector byte names the slot; the rest is the text, cut at its first NUL as a C caller's string is |
| `pack` | `ts_intl_load_pack` | the bytes, as a caller's buffer |
| `key` | `ts_key_parse`, `ts_intl_render`'s key, `ts_intl_entity` | the text |
| `mf2` | the MF2 parser and formatter, through `ts_intl_render` | a selector byte picks the base locale or Nepali; the rest is a message's source, set as an override under a key no locale has and rendered with one parameter of every type the engine formats |
| `blob` | the result-blob reader (`teistro_idl::blob::Reader`) | the bytes as a blob, read under the schema its header names: every section by its kind, every field of each |

The MF2 engine and the blob reader have targets of their own because
the ABI targets reach them only through text the library already
checked. A pack's messages are its translators' words, which a fuzzer
reaching them through a pack would first have to get past the pack's
own checks. A blob is bytes the library wrote and a binding reads, so a
reader must survive bytes cut short or altered between the two. The
reader is the reference every generated binding decoder is held to, and
`every_blob_the_library_writes_reads_whole` holds it to one blob of
each schema the library writes.

The generated decoders in Node, Python, Dart and Java are out of
libFuzzer's reach, so each binding's blob test corrupts what the library
wrote instead (`blob.test.mjs`, `test_blob.py`, `blob_test.dart`,
`BindingTest.java`). Every binding runs the same mutator over one blob of
each schema (`crates/ffi/examples/blob_fixtures.rs`): a seeded xorshift32
stream picks, two thousand times a blob, one of four mutations: a few
bytes flipped past the header; one aligned word set to an edge value
(0, 1, 3, 4, 8, 2^31 − 1, 2^31, 2^32 − 1 or the blob's length); the blob
cut short; or the blob grown with zeros. The last two mend the header's
length so the table of contents is read. A mutant is decoded or refused
as the binding's blob error (`BlobFormatError`, `BlobError`,
`FormatException`, `BlobFormatException`), and anything else thrown is a
finding. Each schema must take both ways out, so a mutator that never got
past the header cannot pass. Its first run found every decoder but
Java's throwing the runtime's own errors (a `RangeError`, a
`struct.error`, a `UnicodeDecodeError`) where a section, a column or a
text ran outside its bounds or off its alignment. All four now hold one
contract, which Java's reader already mostly kept:
- the table of contents fits the blob;
- a fixed section holds a slot for each field;
- a column's offset is inside its section, starts on a multiple of its
  width, and holds every row;
- a text's count fits its section and is UTF-8.

A chart request's other fields are fixed at a request that founds one
chart, so every section's reader runs against a real sky; the context is
built once per process with the compact built-in ephemeris.

## 2. What a finding is

The ABI catches a panic and answers `INTERNAL`, which a binding would
survive. Under the fuzzer the panic hook aborts first, so **any panic is
a finding**, caught or not; and so is any answer outside the status
codes, a hang past libFuzzer's timeout, and an allocation past its RSS
limit. A refusal is the expected answer to most inputs and is not a
finding.

Work in proportion to what was asked is not a hang. The SDK computes
whatever it is asked, and bounding a caller is the host's job, as the
shipped server does (`teistro_mcp::Limits`, `03-design/mcp-server.md`
P5). So the `json` target skips a record that server would refuse for its
size, such as two centuries of daily readings, and a timeout still
counts as a finding on every record that server would accept.

## 3. Corpus and cadence

- The seed corpus is committed under `fuzz/corpus/<target>/`: each JSON
  slot seeded with the requests the ABI's own tests send, so the fuzzer
  starts past the parser; the pack seeded with the shipped packs; `mf2`
  with shipped English and Nepali messages using each function and
  shape, and messages naming every parameter; `blob` with one blob of
  each schema, written by `TEISTRO_FUZZ_SEEDS=corpus/blob cargo test
  every_blob` in `fuzz/` and the positions blob of `blob_fixtures`.
- **Nightly smoke** (`verify.yml`): each target for 60 seconds from the
  committed corpus. A crash fails verify and uploads its input.
- **Weekly long run** (`fuzz.yml`): each target for an hour; new coverage
  is offered back as an artefact, not committed by the job.
- A crash input, once fixed, is added to the corpus and to a regression
  test in the crate it fixed, so the plain test suite holds it without
  the fuzzer.

The fuzz crate is outside the workspace, as the adapters are: it needs
nightly and `libfuzzer-sys`, which the workspace builds without.

## 4. Findings

| found | input | what broke | fixed |
|---|---|---|---|
| 2026-10-12, the first two minutes of `json` | `progressions_json` asking `{"at": 1.24e30}` | the progressed chart's civil day reached `gregorian_from_fixed`, whose year arithmetic overflowed before `date_of` checked the year it made | every shipped calendar refuses a day outside its range before converting it (`OUT_OF_RANGE`, field `fixed`); `crates/calendar/tests/range.rs` holds it for each |
| 2026-10-12, five minutes of `json` | `ts_rashifal` with one period from 1837 to 2037, `WEEKLY` | nothing: 73 000 days of readings take minutes because 73 000 were asked for | not a defect. The target holds records to the shipped server's bounds (§2), and the input is in the corpus; `a_record_past_the_servers_bounds_is_skipped` holds it |
| 2026-10-23, verify's nightly minute of `json` | `hits_json` asking `{"from": 2460676.5, "to": 6021641.5, "grahas": ["SATURN"]}` | nothing in the SDK: 9,750 years of Saturn's hits take minutes because they were asked for. The harness read a chart section's text as a whole record, where the server's bounds find no `hits` window, so it did not skip a record the server refuses | the harness reads a chart section under its own name in the chart's record, as the server does; `a_transit_window_past_the_servers_bounds_is_skipped` holds it |
| 2026-10-23, verify's minute of `json` on `8557e728` | `sade_sati_json` asking `{"from":2460676.5,"to":246434429.0,"reckoning":"SIGN"}` | the server bounded no window of Saturn's spells: the search runs on to the ephemeris's edge, so 670,000 years asked the built-in's whole six centuries (six seconds in release, past the timeout under the fuzzer), and an ephemeris reaching millennia would hold a worker for minutes | `Limits::years` (`--max-years`, 150 by default) bounds a `sadeSati` window in years, and the harness names a section as the chart record spells it; `a_window_of_saturns_spells_is_bounded_in_years` and `a_window_of_spells_past_the_servers_bounds_is_skipped` hold it |
| 2026-10-23, the first seconds of `mf2` | `{$x :number maximumFractionDigits=87777777777}` | the formatter wrote that many fraction digits, and the allocation aborted | a digit count past ECMA-402's range (100 fraction digits, 21 integer) is ignored with a warning; `a_digit_count_past_the_range_is_ignored_with_a_warning` holds it, and five minutes more found nothing |
| 2026-10-23, five minutes of `blob` | — | nothing in 327 000 runs | — |
