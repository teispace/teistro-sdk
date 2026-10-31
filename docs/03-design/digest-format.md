# The N-day digest

Status: `built`, 2026-10-30. Design for the consumer's SR-08 (= A2-10),
written before the code. Cruxes C871 to C873.

A home-screen widget or a watch complication runs in a small process: an iOS
widget extension has about 30 MB of memory and a limited reload budget, so
it cannot run the engine. The consumer asks for one record that writes N
days (60 by default) for one place and profile, with every limb and its
times, the named windows, the Moon's phase, rise and set, and the day's
observances, in a compact, versioned, documented format that is streamable,
at most a few hundred KB for 60 days, deterministic, and read by one reader
with no dependencies that Swift and Kotlin both call.

## 1. Does the existing blob fit?

**The layout does; the `panchanga` schema does not.** The `TSRB` result blob
(`ffi-abi-and-api-description.md` §3.4) already gives a header, a table of
contents, a versioned layout, generated decoders in Node, Python, Dart and
Java, a mutator in each binding's tests, the fuzz target and a reference
reader. A second format would repeat all of it. So the digest is a new
schema over the same layout, `digest`, id 6.

The `panchanga` schema cannot be that schema: it is what a binding needs,
not what a widget needs. It has 39 sections, Julian days as `f64`, and a
JSON section for what a binding's caller may ask beside the days. Its
names are catalogue ids, which a process without the locale engine cannot
print.

What the layout lacks, this page adds on top of it, without changing it:

| need | where | why not elsewhere |
|---|---|---|
| a digest of the contents | the last section, `seal` | the header's three reserved words are 12 bytes and a binding reads them as zero |
| a format version | `summary.format_major`, `format_minor` | the layout's own version describes the layout, and every schema shares it |
| names with no engine | `names` and `text` | a span's name must print with no locale pack |

## 2. The format

Little-endian, as every blob. Sections, in order:

| id | section | kind | holds |
|---|---|---|---|
| 1 | `summary` | fixed | `format_major`, `format_minor`, `day_count`, `base_unix_seconds`, the first day's `utc_offset_seconds`, `calendar`, the place |
| 2 | `days` | columns | one row a day: date, vara, polar state, sunrise, sunset, first moonrise and moonset, phase, lunar month, and where its rows are |
| 3 | `spans` | columns | `kind`, `name`, `from`, `to` |
| 4 | `observances` | columns | `name`, `flags` |
| 5 | `names` | columns | where each key and name is in `text` |
| 6 | `text` | bytes | UTF-8 |
| 7 | `locale` | bytes | the tag `names` is in |
| 8 | `provenance_json` | bytes | the days' provenance, canonical |
| 10 | `clock` | columns | one row a day, the offset the day was read on (format 1.1) |
| 9 | `seal` | bytes | SHA-256 of every byte before it, 64 lowercase hexadecimal digits |

The sections are in that order in the bytes. `clock` has the higher id and
stands before `seal`, since the seal ends the file (§4). `idl/api.json` holds
each field's type and meaning, generated from `digest_schema()`, so this page
does not repeat them.

- **Time.** Every time is a signed 32-bit count of whole seconds from
  `base_unix_seconds`, the first day's local midnight. Signed, since the
  tithi running at the first midnight began before it. `i32::MIN` is an
  instant the day lacks. Seconds, since no widget shows less, and four bytes
  against an `f64`'s eight.
- **The clock.** Times are instants, so they read the same under any offset,
  and `base_unix_seconds` stands on the first day's. To print a day's times a
  widget adds that day's offset from `clock`, which a request by `zone` moves
  across a clock change (SR-73, C1010). It is the offset at the day's opening,
  as `Panchanga.clock` is, so a day whose clock changes before sunrise reads
  the new offset. An instant before the change on that day is an hour out when
  printed this way; a process that must print those holds a zone database.
- **Span kinds.** `TsDigestSpan` in the IDL: tithi, nakshatra, yoga, karana,
  the three kaalas, choghadiya, hora, Abhijit, Bhadra, Brahma muhurta,
  durmuhurta, varjyam, amrita kalam, panchaka, and the Moon's and the Sun's
  signs. The numbers never change.
- **A limb's span is the whole limb**, `Span::whole`, so "the tithi ends at"
  reads off it even when the end is tomorrow. Consecutive days repeat a
  limb that crosses midnight; a widget showing one day needs that row.
- **Bhadra** is named by where she dwells, from `Panchanga::bhadra_vasa`, so
  a sign change inside a Vishti gives two rows.
- **A hora** is named by its lord. Abhijit, Brahma muhurta and a durmuhurta
  have no name of their own, written 65535.
- **The Moon's phase** is thousandths of a lunation at sunrise, read from
  the tithi that held sunrise and how far through it sunrise fell: a tithi
  is twelve degrees of elongation, so this needs no ephemeris call (C872).
- **Names** are the catalogue's full key (`tithi.SHUKLA_PRATIPADA`) or a
  rule's key, and the text in the context's locale, or the key where the
  locale has none. Each is written once, in the order first met, and a row
  holds a two-byte index.
- **Observances** are the festival answer's days, as the feed keeps them: a
  day a published set withdrew is left out, an added one is in.

## 3. Determinism

The same request writes the same bytes on every platform. No clock is read:
a digest has no stamp, and two runs differ only if the data does. Every
computation runs on `teistro_core::math`, one pure-Rust libm on every target
(`05-testing/01-quality-bar.md`, held by `uses-one-libm`), so an instant has
the same bits everywhere and the rounding to a second is the same. The only
floats written are the place's three, as given.
`a_digest_is_the_same_bytes_on_every_platform` pins the hash of a week's
sections, the provenance left out, so every platform the tests run on must
write them; `check-parity` holds every binding to one seal.

The bytes name the SDK's version, the profile and the settings through
`provenance_json`, so a new SDK writes new bytes even where no answer moved,
as every sealed answer does.

## 4. Versioning and the seal

`format_major` is raised when a reader of the last version would misread:
a field's meaning, a column's type, a section's removal. `format_minor` is
raised when a section is added, which goes before `seal`; a reader finds
sections by id and skips what it does not know, and the seal still covers
it. A new `TsDigestSpan` number is a minor version, and a reader shows what
it does not know by its name.

| format | adds |
|---|---|
| 1.0 | the sections of §2 but `clock` |
| 1.1 | `clock` (SR-73) |

The reader takes both. A 1.0 digest has no `clock`, and each `ts_digest_day`
then says the summary's offset, which was every day's. A digest of 1.1 or later
without `clock`, or with a row count other than `day_count`, is
`TS_DIGEST_E_MALFORMED`. The per-day figure is a section and not a column of
`days` by the rule above: a reader of 1.0 fixes the shape of `days` and would
misread a wider one.

The reader reads the version before the seal, so a later format is refused
as `TS_DIGEST_E_VERSION` rather than as damage, whatever it seals with.

`seal` is SHA-256 over every byte before it, in hexadecimal: text because
every binding's generated decoder reads a bytes section as UTF-8, and raw
bytes would refuse the digest there. It ends the file, so
`seal.offset + 64 == length`, and the header's length is under it: a cut
that mended the header still fails. SHA-256 and not a CRC, since the
digest's hash is the same one an envelope's `content_hash` uses, and a
reader can show the same value the provenance names; it is an integrity
check and not a signature. An app that takes a digest from a server
authenticates the transport as it would any file.

## 5. The reader

`bindings/c/digest/teistro_digest.{c,h}`: two files to drop into a project,
C11, `<stddef.h>` and `<stdint.h>` only, with no call to a C library and no
allocation. `check-c` compiles the object freestanding and refuses any
undefined symbol. A `ts_digest` is a fixed-size struct on the caller's
stack, holding pointers into the bytes, which it never copies or aligns.

| status | when |
|---|---|
| `TS_DIGEST_E_ARG` | a null pointer |
| `TS_DIGEST_E_TRUNCATED` | fewer bytes than the header says |
| `TS_DIGEST_E_MAGIC` | not a `TSRB` blob |
| `TS_DIGEST_E_LAYOUT` | another layout version |
| `TS_DIGEST_E_SCHEMA` | another schema |
| `TS_DIGEST_E_VERSION` | another `format_major` |
| `TS_DIGEST_E_SEAL` | the bytes do not match the seal |
| `TS_DIGEST_E_MALFORMED` | sealed, and still not shaped as this page says |
| `TS_DIGEST_E_RANGE` | an index past the end |

**Streamable** (C871). The header and table sit first, and every row's
place follows from them, so a process mapping the file (`mmap`, or
`Data(contentsOf:options:.alwaysMapped)`) pages in only what it reads. The
seal is checked on open, which reads the file once; at this size that is
well under a frame. A forward-only stream cannot verify before it reads,
and the format does not try to serve one.

From Swift, add the two files to the extension's target and import the
header through its bridging header:

```swift
var digest = ts_digest()
let opened = bytes.withUnsafeBytes { ts_digest_open(&digest, $0.baseAddress, $0.count) }
guard opened == TS_DIGEST_OK else { return }
var day = ts_digest_day()
ts_digest_day_at(&digest, 0, &day)
```

From Kotlin, compile them with the NDK beside a JNI function that holds the
`ByteArray` for the call, or a direct `ByteBuffer` for as long as the digest
is read. The bytes must outlive the `ts_digest`.

## 6. The call

`AlmanacArea::digest(&DigestRequest)` answers the bytes, sealed: its
provenance is the days', its `content_hash` SHA-256 of the bytes. At the
boundary it is `ts_almanac_digest`, a JSON request and a blob; in every
binding `almanac.digest` answers the bytes, and each binding's generated
`digest` decoder reads them. The agent server's `almanac.digest` tool
answers `{format, days, size, seal, base64}`, since a tool's answer is text.

The request is `first`, `calendar`, `days` (60 when left out, 1 to 366),
`latitudeDeg`, `longitudeDeg`, `altitudeM`, `utcOffsetSeconds` or `zone` (a
`ZoneSpec`, as `almanac.days` takes it; both or neither is refused) and
optionally `festivals`, a festival request; none lists no observances.

## 7. Budgets and acceptance

| budget | ceiling | held by |
|---|---|---|
| 60 days at Kathmandu with every Nepal festival | 200 KB | `a_60_day_digest_keeps_its_size`, `crates/sdk/tests/digest.rs` |
| the reader's heap | none | `check-c`: no undefined symbol in the object |
| the reader's stack | one `ts_digest` and SHA-256's 512 bytes of schedule | by construction, `check-c`'s run under AddressSanitizer |

The consumer's own ceilings are "a few hundred KB" and 30 MB of process
memory. The reader takes the file's size and a few hundred bytes of stack.

- The C reader against the library's own: `check-c` compares a dump of every
  field through `teistro_digest.c` with one written through
  `teistro_idl::blob::Reader` over the same bytes, so a field the two read
  differently fails.
- Each status of §5 is provoked by name (`digest_read refuse`).
- Fuzzing: the `digest` target runs the reader over bytes with the seal
  mended, so a mutant reaches the checks behind it, and `check-c` runs the
  bindings' mutator over it, 20,000 rounds under AddressSanitizer and
  UndefinedBehaviorSanitizer, both ways out required. Every binding's
  decoder takes the digest through its own blob mutator, since it is one of
  `blob_fixtures`' schemas.
- The same request writes the same bytes; a day's rows and names read back
  equal the panchanga they came from.
