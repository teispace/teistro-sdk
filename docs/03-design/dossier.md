# The dossier: a chart as text for a language model

Status: `building`, 2026-10-18. `teistro::dossier::dossier(&document,
&options, &[])` writes the fifteen sections below (`crates/serial/src/dossier.rs`,
`crates/sdk/tests/dossier.rs`), and a chart request's `dossier` record
carries each chart's text across the C boundary in section 137, read by
every binding as `chart.dossier`. The feature register
asks for it twice (`18-interpretation-rendering.md`: "LLM prompt
dossiers" and "compact text dossier for prompts, presets, byte-stable",
both P0), and the module catalogue lists it as `serial`'s later row.

## What it is

A chart **document** rendered as plain text that a language model reads
in a prompt: every section the document carries, one banner each, rows of
pipe-separated fields under a header line, and a legend that says what
each field is. It is the same chart the JSON says, in a form that spends
far fewer bytes on punctuation and keys and that a model reads row by row.

It renders **what was computed and nothing else**. A dossier never asks
the ephemeris anything, never reads the clock and never reads a locale
pack, so the same document under the same options is the same bytes on
every machine and in every binding.

## What the reference did, and what it taught

The baseline engine ships one (its serialiser, 1.4.0). Four of its
lessons are taken whole and one is turned round:

- **Date-free by default.** Its 1.2.0 removed a wall-clock default after
  stored text was found carrying the day it was computed on. Here no
  option reads a clock: without an instant the dasha section is the
  complete timeline to its depth; with one (`at`) the running periods are
  marked and only their branch is expanded below the first level.
- **The legend scoped to the sections present.** A dossier of three
  sections does not pay for the legend of twenty.
- **Every column documented.** Its 1.4.0 found six sections whose columns
  no legend line explained, and added a test to keep it so. Here the test
  is not needed: a section declares its columns once, as `(name,
  meaning)` pairs, and **both** the header line and the legend are
  written from that one table, so a column without a meaning does not
  compile.
- **No lite preset.** It retired its cheaper tier after the tier dropped
  the D9 and the D10 a marriage or a career question needs. The presets
  here choose sections, never thin them.

What is turned round is the **codes**. It writes two-letter codes (`Ju`,
`Sg`) with a legend to decode them. Here a member is written as its
catalogue key, bare (`JUPITER`, `SAGITTARIUS`, `ASHWINI`): the spelling
every binding already answers, no decoding step, and no collision of the
kind two letters invite (`Sa` for Saturn beside `Sg`, `Sc`, `Si`). A key
costs a few bytes more than a code; the measurement on this page counts
them.

## The shape

```text
# teistro dossier 1
== LEGEND ==
# Each section opens `== NAME ==`; a `#` line lists its columns, `|`-separated.
# `-` is a value the chart does not have.
# GRAHAS: graha|sign|deg|nakshatra|pada|house|speed|motion
#   deg: degrees in the sign, two decimals. speed: degrees a day.
…
== CHART ==
kind|NATAL
instant|1990-04-12T07:35:00Z|2447993.816
…
== GRAHAS ==
# graha|sign|deg|nakshatra|pada|house|speed|motion
LAGNA|CANCER|12.40|PUSHYA|3|1|-|-
SUN|PISCES|28.72|REVATI|4|9|0.98|DIRECT
…
```

- The first line names the format and its **version**, `1`. A change to
  any byte the format writes — a column, a legend line, a number's
  decimals — raises it, so a store that keys on the text knows its text
  is stale.
- A **section** is a banner, an optional header line, and rows. A row is
  fields joined by `|`; a field never holds a `|` or a line break.
- **Numbers** have a fixed number of decimals per column, written by
  Rust's formatter, which is exact, so the bytes are the same everywhere.
  Instants are written as the civil date and time in the clock the
  request named, and as the Julian day, so a model can read either.
- **Missing** is `-`.

## The sections

In this order, each present when the document carries it:

| banner | from | rows |
|---|---|---|
| `CHART` | the foundation | kind, instant, clock, place, zodiac and ayanamsha, the day it belongs to, the lagna |
| `NOW` | the request's `at` | the instant, the native's age in years |
| `GRAHAS` | the foundation, and the state when present | the lagna, then the nine (and the outer three and Lilith when placed): sign, degrees, nakshatra, pada, house, speed, motion; with the state, dignity, friendship, combustion and the Baladi age |
| `HOUSES` | `houses` | each house's cusp under the system computed |
| `VARGAS` | `vargas` | one row a graha, one column a divisional chart, the sign |
| `DASHAS` | `dashas` | each system: date-free, the complete timeline to the depth; dated, the running periods marked `*` and their branch expanded |
| `PANCHANGA` | `panchanga` | tithi, paksha, nakshatra, yoga, karana, vara, sunrise and sunset |
| `ASHTAKAVARGA` | `ashtakavarga` | the sarva row, then each graha's bhinna row, twelve signs from Aries |
| `SHADBALA` | `shadbala` | each graha's six strengths, total, the required, the ratio |
| `BHAVA_BALA` | `bhava_bala` | each house's strength |
| `VIMSHOPAKA` | `vimshopaka` | each graha under the four schemes |
| `JAIMINI` | `jaimini` | the chara karakas, the karakamsha |
| `POINTS` | `points` | each upagraha and special lagna: sign, degrees |
| `ASPECTS` | `aspects` | each graha's drishti onto houses |
| `AVAKAHADA` | `avakahada` | the Moon's star and pada, the naming syllable, the koota readings of one native |

A section a consumer adds (below) comes after these, in the order added.

## Options and presets

`DossierOptions`:

- `sections`: the sections to write, by key; **every one the document
  carries** when left out. A section named that the document does not
  carry is refused by its key, with the request flag that would compute
  it as the hint (`shadbala` asked of a chart founded without
  `shadbala: true`), because a silently empty section reads to a model
  as "no strength".
- `at`: the "now" a dated dossier is read at; none, date-free.
- `dasha_depth`: how many levels a timeline is written to, at most the
  depth computed; the computed depth when left out.
- `utc_offset_seconds`: the clock instants are written in; UTC when left
  out, and the chart request's own for its `dossier` record.

Two presets name section lists, and nothing else:

- `FULL`: every section the document carries.
- `FAMILY`: `CHART`, `GRAHAS`, `DASHAS`, `PANCHANGA`, for a prompt that
  holds several people's charts; one legend for the group, so the legend
  can be written on its own (`dossier_legend`).

## Extending it

A **section** is a trait, `DossierSection`: a key, a banner, its columns
with their meanings, and the rows it writes for a document. The built-in
sections are implementations of it, and a consumer's section — a pack's
yogas, a house's own reading — is another, passed with the options. Its
columns go into the legend by the same table, so a consumer's section is
documented the way a built-in one is.

## Across the boundary

A chart request's `dossier` record (`{sections?, preset?, at?,
dashaDepth?, utcOffsetSeconds?}`, `DossierOptions::from_json`) writes
each chart's dossier on the request's own clock, and section 137,
`dossier`, carries them as a JSON array of strings, one a chart
(`tests/abi.rs` holds it to the façade's own). A section the record
names that one request flag computes — `SHADBALA`, `HOUSES` and the rest
— **widens the request** with that flag, so naming it is asking for it;
`VARGAS` and `DASHAS` name no list, so the request must carry one, and
`NOW` needs `at`; each is refused by `dossier.sections[i]` otherwise.
Every binding reads `chart.dossier` as a string (Java's
`chart.dossier()`, an `Optional<String>`), its sections a closed type in
Node (`DossierSection`), Python (`DossierSection`, a `Literal`) and Dart
(the `DossierSection` enum); Java's generated `DossierOptions` takes them
as keys, refused by the SDK when unknown. The parity example asks every
binding for one dated dossier of five sections and prints its length and
hash, so the six runners are held to the same bytes. The agent server
reads the record, so a model can ask a chart for its own dossier.

## What is decided and what is not

| | |
|---|---|
| **decided** | bare catalogue keys, not codes; presets choose sections and never thin them, and a named section the document lacks is refused; one column table writes the header and the legend; date-free unless `at` |
| **a setting** | `sections`, `preset`, `at`, `dasha_depth`, `utc_offset_seconds` |
| **not built** | the gochar overlay (a transit section from the `gochar` record's readings); the yogas and doshas of a rules reading as a built-in section; a localised legend |

## What will be measured

The bytes of each section's dossier text against the canonical JSON of
the same section, over the conformance corpus's charts, recorded on a
generated page, so "far fewer bytes" is a measured number and not a
sentence.
