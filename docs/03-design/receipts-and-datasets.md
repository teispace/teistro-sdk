# Receipts, a deterministic verify, and research datasets (SR-51, SR-57)

Status: `built`, 2026-11-02 (designed the same day, before the code).
SR-51 asks for a receipt
naming what was computed and a `verify(receipt)` that computes it again
and says whether it held, usable as the attester of an Open Knowledge
Format (OKF) v0.2 *Attested Computation*. SR-57 asks for a versioned
dataset format for studies: a reliability rating per record and a
settings snapshot per study, feeding `research.study`.

The two share one piece: **a settings snapshot**, the profile, the whole
canonical settings document and its hash, which a receipt carries so it
can be computed again and a study carries so it is run under what its
author ran it under. Both are re-run on a context **derived** from the
caller's: the caller's ephemeris and locale state, the snapshot's
settings (§1.4).

## 1. Receipts (SR-51)

### 1.1 What the envelope already had, and what was missing

Every sealed answer's provenance (`core/src/envelope.rs`) carries the
SDK, calculation and catalogue versions, the profile, `settings_hash`,
`input_hash`, the provider's identity, the packs, the time tables and
`content_hash`, the hash of the value. Missing, as the consumer read it:

- **the record's name.** `input_hash` is a hash of the request as each
  area chose to write it; nothing says which operation it was the input
  of, so nothing can run it again;
- **the request itself.** A hash cannot be re-run;
- **the settings themselves.** `settings_hash` names settings a verifier
  may not have;
- **an answer without provenance.** A record that reads no sky
  (`calendar.convert`, `numerology.profile`) is answered with no
  provenance at all, so it had nothing to receipt;
- **`verify`.** Nothing computed an answer again and compared it.

### 1.2 The receipt

`verify.issue({record, request})` answers the record exactly as
`records::record(name).answer` does, and beside it a receipt
(`core/src/receipt.rs`, `teistro.receipt/1`):

| field | what it is |
|---|---|
| `format` | `teistro.receipt/1`; another is refused at read |
| `record` | the record's name, `almanac.digest` |
| `request` | the request as the record read it, a JSON value, written canonically when hashed or sent |
| `requestDigest` | the hash of the request's canonical JSON |
| `settings` | the settings snapshot: `{profile, settings, hash}`, the whole canonical settings document, kept as it was carried, and its hash (§3) |
| `locale` | the locale the context answered in, since a record may answer in words |
| `packs` | the runtime packs loaded, as the provenance stamps them |
| `provider` | `{name, version, dataVersion}` of the ephemeris, or null where the answer read none |
| `tzdb` | the zone database's version |
| `sdkVersion`, `calculationVersion`, `catalogueVersion` | as the provenance carries them; `sdkVersion` keeps a pre-release label, `{major, minor, patch, pre: "rc.1"}` (C1177) |
| `outputDigest` | the hash of the answer's **value**, canonical JSON (C1170) |
| `digest` | the seal: the hash of the receipt's canonical JSON with `digest` left out |

Every hash is the existing `content_hash` over the existing canonical
form (`envelope::canonical_json`); the receipt adds no second writer.

### 1.3 `verify.receipt`

`verify.receipt(receipt)` answers `{verdict, record, moved, broken,
issued, now, reason}`. The steps, in order:

1. **Held together.** The request's digest, the snapshot's hash and the
   seal are computed again from the receipt itself. A field edited after
   issue breaks at least the seal: the verdict is `BROKEN` and `broken`
   names each digest that failed. Nothing is re-run. Only a digest that
   does not match the bytes the receipt carries is `BROKEN`: settings
   from a build with a knob this one lacks, or without one it has, hold
   to their hash and come to step 2 (C1176).
2. **Can it be re-run here.** `NOT_RERUN` with `reason: {cause,
   detail}` when this build carries no such record (`RECORD`); the
   receipt names a provider and the context has none or another one
   (`PROVIDER`); the receipt's settings are not this build's to read, a
   knob it does not have or one it has that they lack, named in `detail`
   (`SETTINGS`, C1176), or need the classical provider opened another way
   (`SETTINGS`, §1.4); its locale is not answered here
   (`LOCALE`); it names a runtime pack not loaded here (`PACKS`).
3. **Re-run.** The record is answered on the derived context with the
   receipt's request. A refusal `OUT_OF_RANGE` (an instant past the
   provider's data, the consumer's "data range", or a table's years or a
   count's bound, which the status does not tell apart) is `NOT_RERUN`
   with `OUT_OF_RANGE` and the refusal's message; any other
   refusal is `NOT_RERUN` with `REFUSED`.
4. **Compared.** `HELD` when the output digest is the receipt's,
   `DIFFERS` when it is not. Either way `moved` lists every field that
   moved between `issued` and `now` with both values: `output`,
   `sdkVersion`, `calculationVersion`, `catalogueVersion`, `provider`,
   `tzdb`. A different SDK that computes the same value is `HELD` with
   `sdkVersion` in `moved` (C1171).

`issued` and `now` are the two sides' versions, provider and zone
database, so a `DIFFERS` says what changed between the run and the
re-run without the caller diffing two documents.

### 1.4 The derived context

A receipt is re-run under **its own** settings, not the verifier's.
`Context::with_settings(snapshot, locale)` builds a context from the
caller's: the same ephemeris behind the same cache (the core now holds
it in an `Arc`), the same layouts, dasha systems, zone database, packs
and name books, and the snapshot's settings, checked for coherence as a
build checks them and held to the provider as a build holds them.

One case cannot be derived. A classical provider (the Surya Siddhanta)
is opened for the settings' `frame.siddhanta` and `time.delta_t`, so
settings that differ there need the provider opened again, which only
the caller's chain can do: the derived context is refused with
`UNSUPPORTED` at that knob, and `verify` answers `NOT_RERUN` with
`SETTINGS` (C1173). A modern provider answers the same whatever the
settings, and is shared.

### 1.5 The seal is a digest, not a signature

The seal catches a field changed after issue. It does not stop someone
who changes a field and seals again: there is no secret. What catches
that is the re-run: a resealed output, request or setting gives
`DIFFERS` (or a refusal), and a resealed version or provider is reported
in `moved`. A signature needs a key the maintainer holds (the PGP key
waits on them, HANDOFF), and is a later layer over the same seal (C1172).

### 1.6 As an OKF attester

OKF v0.2 §10 describes an *Attested Computation* concept: `runtime`,
typed `parameters`, an `executor` that runs the computation and returns
a **receipt** whose fields `executor.receipt` declares, and an
`attester`, deterministic code with no model in it that takes a receipt
and returns a verdict, run on the consumer's side. The specification
leaves the receipt's wire format and the verdict's spelling open (§12,
"considered and deferred").

The SDK fills the two roles:

- **executor**: `verify.issue`, on any binding or as the MCP tool. A
  concept's `parameters` are the record's request fields, and its
  `executor.receipt` lists the receipt's fields in §1.2;
- **attester**: `verify.receipt`. A consumer refuses to show an answer
  whose attestation is not `HELD` (OKF §10.5, step 6); `NOT_RERUN`
  says what the attester lacked, which OKF leaves to the consumer.

OKF's `stale_after` is the concept's, not the receipt's, and is not
read here.

### 1.7 Determinism

A re-run answers the same bytes when the record is a function of its
request, settings, locale, packs, provider and zone database, which is
what the receipt names. A record that reads the wall clock would not
be; none of the records does. A context's runtime layouts and dasha
systems are not named: a receipt of a request naming one is re-run
only where the verifier registered the same one, and moves otherwise.

## 2. Research datasets (SR-57)

### 2.1 The reliability rating

The rating is the Rodden Rating, the scale Lois Rodden kept for her
birth-data collections and Astro-Databank uses, which grades **the
source** of a birth time, not its precision:

| rating | the source |
|---|---|
| `AA` | a birth certificate, civil register or written family record |
| `A` | the native, kin or an associate, from memory |
| `B` | a biography or autobiography |
| `C` | caution: no source, an ambiguous one, or one not credible |
| `DD` | dirty data: two or more unsubstantiated, conflicting sources |
| `X` | the date without a time |
| `XX` | the date itself questioned |

The scale is the one research already uses: Sutton (2010) excludes `B`
and worse from research and rates family records `AA`, and the
grading is the source's, so an `AA` time is still rounded where its
clerk rounded it. `X` and `XX` are not grades of a time but statements
that there is none, so **no study admits them**; the rest are ordered
`AA` > `A` > `B` > `C` > `DD`, and a study names the worst it admits,
`minimumRating` (C1174). A study whose `minimumRating` is `X` or `XX` is
refused.

### 2.2 The format

`crates/research/src/dataset.rs`, `teistro.dataset/1`, with its JSON
Schema generated from the types into
`crates/research/schema/dataset.schema.json` by `cargo xtask
document-schema`, as the chart document's is:

```
{format, name, description?,
 records: [{id, birth: {instant, latitudeDeg, longitudeDeg, altitudeM?,
            utcOffsetSeconds, uncertaintyMinutes?}, rating, source?, event?}],
 studies: [{id, settings: {profile, settings, hash}, minimumRating,
            groups?: {recordId: group}, request}]}
```

A study's `request` is `research.study`'s request without `births`,
`subjects` and `design`, which the dataset supplies (C1175): the births
of the records it admits, in the dataset's order; for `COUNTS` and
`COMPARE`, the design from `groups`, every admitted record named; for
`TIMED`, each admitted record's `event` as its subject's.

### 2.3 Loading

`Dataset::from_json` reads strictly and refuses by the field: an
unknown key, a format that is not `teistro.dataset/1`
(`SCHEMA_VERSION`), an empty `records`, a record id given twice or
empty, a latitude, longitude, offset, instant or uncertainty out of
range (`records[3].birth.latitudeDeg`), a study id given twice, a
`minimumRating` of `X` or `XX`, a group naming no record, a snapshot
whose hash is not its settings' (`studies[0].settings.hash`), and a
request that carries what the dataset supplies (`studies[0].request.births`).

`Dataset::study_request(id)` is the `research.study` request the study
means, and `ResearchArea::dataset` runs it on a context derived under
the study's snapshot (§1.4), answering the study's result beside the
records admitted and those set aside with their rating.

## 3. The settings snapshot

`core/src/settings/snapshot.rs`: `{profile, settings, hash}`. The
document is kept as it was carried, a JSON object, and `hash` is taken
over its own canonical JSON, never over this build's re-serialised
`Settings`: a snapshot written by a later build, with a knob this one
does not have, still holds to its hash here (C1176). `resolved()` holds
`hash` to the document, then reads it as this build's settings, refusing
an unknown or missing knob by its field under `settings`, and runs the
coherence rules, so a snapshot whose document was edited is refused at
`hash` before anything runs.

A study's `request` is checked for what it must not carry when the
dataset is loaded, and read as `research.study` reads it only when the
study is run, so a refusal of the request itself is named under
`research` (`research.rules`), as the record names it.

## 4. The boundary

| record | boundary | answer |
|---|---|---|
| `verify.issue` | `ts_verify_issue` | `{answer, receipt}`: `answer` is the record's own answer, `{value, provenance}` or the value alone |
| `verify.receipt` | `ts_verify_receipt` | `{verdict, record, moved, broken, issued, now, reason}` |

Both in every binding as `sdk.verify.issue` and `sdk.verify.receipt`,
and as MCP tools by the records table. A receipt crosses as JSON and is
handed back whole.

What is Rust only for now: the dataset. `Dataset`, `Dataset::study_request`
and `ResearchArea::dataset` have no record, so no C function, binding or
MCP tool; a binding runs a study's request through `research.study`.

## 5. Measured

- **A receipt's size.** The `requests.json` example receipt
  (`calendar.convert`, no provider, no packs) is **6 550 bytes** of
  canonical JSON; almost all of it is the settings document, which is
  carried whole so a verifier can re-run without the issuer's profile.
- **Its cost.** Issuing adds two `content_hash` passes over canonical
  JSON (the request and the value) and one over the receipt; checking
  re-runs the record once on a derived context, which shares the
  ephemeris and its cache. Under the test profile, every example record
  in `crates/sdk/tests/requests.json` (41, leaving out the engine's two
  and the receipts' own) issued and checked again took **77.6 s** in
  all, not timed record by record; a single sankalpa issued and
  checked ran inside a test of 2.4 s, context build included. Nothing
  was timed under a release profile.
- **The dataset schema** generated from the types is **7 149 bytes**.

## 6. Cruxes

| id | crux | decided |
|---|---|---|
| C1170 | what the output digest covers | the value only; the provenance carries the issuer's versions, which differ from a verifier's by design, and those are compared field by field in `moved` |
| C1171 | a different SDK version that computes the same value | `HELD`, with `sdkVersion` in `moved`: the verdict is about the value |
| C1172 | a seal anyone can recompute | a digest catches an edit; a resealed edit is caught by the re-run or reported in `moved`; a signature waits on the maintainer's key |
| C1173 | a receipt whose settings need the classical provider opened another way | `NOT_RERUN`, `SETTINGS`; a modern provider is shared |
| C1174 | which reliability scale, and what a study admits | Rodden's AA–DD by a study's `minimumRating`; `X` and `XX` never |
| C1175 | what a dataset study's request is | `research.study`'s request less what the dataset supplies |
| C1176 | a receipt whose settings another build wrote | the document is kept as carried and hashed as carried: `NOT_RERUN`, `SETTINGS`, naming the knob, never `BROKEN` |
| C1177 | a pre-release SDK version | kept as written, `1.0.0-rc.1`, and ordered by semantic-version precedence |

## Sources

- Open Knowledge Format v0.2, `GoogleCloudPlatform/knowledge-catalog`,
  `okf/SPEC.md`, §2 (terminology), §10 (attested computations), §12
  (deferred); read 2026-11-02.
- Lois M. Rodden, the AstroData series and *Data News*; the scale as
  Astro-Databank states it (Wikipedia, "Astrodatabank", the rating
  table; read 2026-11-02). `X` and `XX` as Astrotheme states them
  ("Birth time in astrology"; read 2026-11-02).
- D. Sutton, *Birth Data, Data Rating and Astro-Research* (2010), light.org:
  family records rated `AA`, `B` excluded from research, `AA` times
  rounded.
