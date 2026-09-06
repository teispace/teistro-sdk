# Teistro Intl: the engine, the sources and the packs

Status: `draft`, revised 2026-09-06 when the twelve-hour clock and the
day periods were added (§5); revised the same day when the typed
accessors reached the bindings (§9); written 2026-09-05 from spike 4
(`spikes/04-teistro-intl/README.md`); revised 2026-09-06 when the `intl`
crate (`crates/intl`, `teistro-intl`) and its command line were built
from the spike: the SDK's catalogue became the authority for every entity
key, the locale bundle and the Rust accessors were added, the sources
moved to `i18n/` at the repository root and are gated by `cargo xtask
check-intl`. Derives from `02-architecture/03-localization-architecture.md`,
ADR-0010, ADR-0020 and ADR-0023.

## 1. Purpose and scope

One localisation standard for every string the SDK renders and for
consumer applications that adopt it: text in JSON sources under `i18n/`,
`MessageFormat 2` syntax with a fixed function set bound to the SDK's
types, the base locale as the source of truth for keys and parameters,
validation before build, compiled packs a runtime loads with nothing
else, and typed accessors generated for every binding. This page settles
the source conventions, the grammar subset and the functions, the
selection rules, the validation gates, the pack container and the
accessor shapes.

## 2. Inputs, settings and ports

Inputs are the source tree, the locale selected explicitly by the
application (the engine never reads the environment), a key and its
parameters. The engine reads no settings knob; it reads the locale's
metadata. CLDR plural rules and locale parsing come from ICU4X
(`icu_plurals`, `icu_locale_core`, `fixed_decimal`); nothing of CLDR is
reimplemented.

## 3. The sources

```text
i18n/<locale>/_meta.json           the locale's metadata
i18n/<locale>/<namespace>.json     one file per namespace, nested objects
```

- **Locale tags** carry a script: `en-Latn`, `ne-Deva-NP`; the directory
  name is the tag and `_meta.json` repeats it.
- **The base locale** is `en-Latn`: complete by definition, the source of
  every key and every parameter. A key present elsewhere and absent
  here is an error.
- **Namespaces** are lowercase dotted words; the SDK's are prefixed
  `sdk.`; the entity namespace `sdk.entity` holds records, every other
  namespace holds messages. The SDK ships four locales: `en-Latn` (the
  base) and `ne-Deva-NP` at `strict` completeness, `hi-Deva-IN` and
  `sa-Deva` at `base` completeness until their messages are translated;
  the last two came whole from the baseline engine's name tables
  (`teistro-intl migrate baseline`, below), the first two gained the same
  tables under their hand-shaped records.
- **Keys** are dotted paths; a segment is a `camelCase` identifier, a
  catalogue key (`UPPER_SNAKE`) or a catalogue kind's name
  (`avastha_baladi`); a full key is the namespace plus the path
  (`sdk.reason.strength.rank`, `sdk.entity.graha.SUN`). Inside
  `sdk.entity` the path is a catalogue key (`graha.SUN`, `point.LAGNA`,
  `chara_karaka.ATMAKARAKA`), resolved against `teistro_core`'s
  catalogue; any other is refused.
- **`migrate baseline`** imports the baseline engine's entity name tables
  once: the engine's exporter (in its own repository, beside its
  golden-vector exporter) writes every entity type with its entities in
  index order and their names in `sa`, `ne`, `en` and `hi` (primary,
  abbreviation, transliteration, synonyms) to one document
  (`fixtures/baseline/names.json`); the command maps twenty of its forty
  types onto catalogue kinds (with the spelling aliases the two do not
  share: `VISHKAMBA`, the deities' display names, the chara karakas'
  initials, the Lagna as a point) and writes records whose `name` and
  `prose` are the primary name, `short` the abbreviation where the engine
  has one, `iast` the language's transliteration or else the Sanskrit
  one, the glyph the engine's symbol or the catalogue skeleton's and the
  gender the skeleton's; a record a locale already has is kept unless
  `--overwrite`. Every written key resolves in the catalogue; a key that
  does not is reported and skipped; the twenty types the catalogue has no
  kind for (the jagradadi, deeptadi and lajjitadi avasthas, the yoni
  genders, the anga labels, the natural and temporary friendships, the
  combust statuses, the ayurdaya methods and tiers, the maraka triggers,
  the dasha levels, the two named muhurtas, vashya, paya and disha, the
  nakshatra vargas, the transit events, the muhurta activities and the
  Nepali rite names) are reported for the catalogue's growth.
- **Messages** are `MessageFormat 2` source strings.
- **`sdk.calendar`** holds what the date functions read: per calendar
  key (`GREGORIAN`, `BIKRAM_SAMBAT`, ...) a `monthName` and `monthShort`
  message selecting on `$month`, and `date.numeric`, `date.long` and
  `date.full` patterns over `$year`, `$month`, `$day`, `$monthName`,
  `$weekday`, `$weekdayName` and `$era`; `weekdayName` and
  `weekdayShort` selecting on `$weekday` (0 Sunday to 6 Saturday);
  `time.numeric` and `time.long` over `$hour`, `$minute`, `$second`;
  `datetime.join` over `$date` and `$time`; `ghati.numeric` and
  `ghati.long` over `$ghati`, `$pala`, `$vipala`; `duration.day`,
  `hour`, `minute` and `second` selecting on `$n`. A calendar that
  shares another's names links them (`{sdk.calendar.GREGORIAN.monthName
  :msg}`). Era names are `sdk.entity.era` records.
- **Entity records** are objects with named forms (`short`, `name`,
  `prose`, `iast`, and any a locale adds; `name` is required), an
  optional `gender` from the `gender` context, and an optional `glyph`.
  An object is a record when it has a string `name`; otherwise it is a
  group.
- **Source order** is kept and followed by the generators (the Sun
  precedes the Moon in an enum); packs are sorted by key.
- **Metadata** (`_meta.json`): `locale`, `direction`, `numberingSystem`
  (CLDR: `latn`, `deva`, `beng`, `gujr`, `orya`, `taml`, `tibt`,
  `arab`), `grouping` (sizes from the right, the last repeating; `[3, 2]`
  is the Indian grouping), `decimal`, `group`, `fallback` (nearest
  first, ending in the base locale for a non-base locale),
  `completeness` (`strict` for SDK-shipped locales, gated; `base` for
  consumer packs), `contexts` (each a closed value set, `gender: [m, f,
  n]`), `termStyle`, and `listPatterns` by type (`and`, `or`) with `pair`,
  `middle` and `end` templates.
- **No parameter sidecar.** Types follow from the function applied to a
  variable or from a context; see section 5.

## 4. The grammar

The engine implements the stable `MessageFormat 2` grammar (Unicode LDML
47, `message.abnf`) in full: simple messages; complex messages with
`.input` and `.local` declarations, a quoted pattern or a `.match` with
one or more selector variables and variants keyed by literals or `*`;
expressions with a literal or variable operand, a function with options
(literal or variable values) and attributes; markup open, close and
standalone with options; quoted and unquoted literals; the escapes
`\\`, `\{`, `\|`, `\}`; and the data-model checks (duplicate
declaration, missing selector annotation, variant key mismatch,
duplicate variant, missing fallback variant, duplicate option and
attribute names). Selectors are declared variables:

```text
.input {$count :integer}
.match $count
0   {{No planet conjoins {$graha}}}
one {{One planet conjoins {$graha}}}
*   {{{$count} planets conjoin {$graha}}}
```

Errors carry a byte offset. Serialisation back to source round-trips
(a property test over generated trees), and arbitrary input never
panics (two property tests).

## 5. The functions and the types they imply

| function | operand | options | selection keys | parameter type |
|---|---|---|---|---|
| `:string` | any | | the text | `string`, or the context whose name or values it selects with |
| `:integer` | number | `select=ordinal`, `useGrouping=false`, `minimumIntegerDigits` | exact numeric match, then the plural category | integer |
| `:number` | number | `minimumFractionDigits`, `maximumFractionDigits`, `useGrouping=false`, `minimumIntegerDigits`, `select=ordinal` | as `:integer`, on the visible digits | number |
| `:dms` | degrees | `precision=deg\|min\|sec`, `signed=true` | | number |
| `:zodiac` | ecliptic longitude | `form=degree-in-sign\|sign-degree`, `precision`, `signNames=name\|short\|glyph\|iast` | the sign's key | number |
| `:entity` | catalogue key | `form=short\|name\|prose\|iast\|glyph`, `kind=<catalogue kind>` | the bare key, the full key, the gender | a member of the kind (the catalogue's enum in Rust), or any catalogue key |
| `:list` | list | `type=and\|or` | | list |
| `:msg` | a key literal | | | none |
| `:date` | a calendar date | `calendar=<calendar key>`, `style=numeric\|long\|full`, `pattern=<message key>` | | date |
| `:time` | a time of day | `style=numeric\|long`, `hour12=true`, `pattern` | | time |
| `:datetime` | a date with a time | `calendar`, `style`, `hour12=true`, `pattern` | | date and time |
| `:ghati` | a ghati-pala count | `style=numeric\|long`, `pattern` | | ghati |
| `:duration` | number | `unit=day\|hour\|minute\|second` | | number |

Numbers render in the locale's numbering system, grouping and
separators; angles in the same digits. `:zodiac` looks the sign name up
in `sdk.entity.rashi` along the fallback chain. `:msg` renders another
key with the same parameters, eight levels deep at most. An unknown
function, a missing parameter or a missing entity yields a warning in
the result and the standard fallback text (`{$name}`, the key); the
worst case renders the key itself, never a blank. `:entity` checks its
key against the catalogue at render (`teistro_core::key::resolve`) and
warns on one the catalogue lacks; `:zodiac` takes the sign keys from the
catalogue's `rashi` kind. Selection follows the specification: for each
selector the function's keys in preference order, variants filtered and
sorted by rank, `*` last. `:date` renders a `CalendarDate` in its own
calendar or the one `calendar=` names (converted through the shipped
calendars' fixed day; an unknown or unshipped target warns and leaves
the date), through the locale's `sdk.calendar` pattern for the style or
the message `pattern=` names, with the era's year and short form when
the date carries an era; `:time` a `ClockTime`; `:datetime` both, joined
by `datetime.join`; `:ghati` a `Ghati` (the time crate's ghati-pala
count); `:duration` a count of a unit through the unit's plural message.
A time pattern is given six parameters whatever the clock: `hour` (0 to
23), `minute`, `second`, `hour12` (1 to 12, so midnight and noon are
twelve), `dayPeriod` (the locale's word for the part of the day) and
`meridiem` (its am or pm). `hour12=true` chooses the locale's
`sdk.calendar.time.<style>12` pattern instead of `<style>`, and each
locale writes the one its readers expect: English by am and pm (`6:15
am`), Nepali by the part of the day (`बिहान ६:१५`). The parts are
`morning` from 4 to 11, `afternoon` from 12 to 15, `evening` from 16 to
19 and `night` from 20 to 3, the same ranges in every locale until the
sources can carry ranges of their own (§13); a locale whose day divides
elsewhere writes its pattern on `meridiem` or on `hour`. These are the
engine's own parameters, so validation lets a locale use one the base
locale's pattern does not (`engine_params`).

A locale that declares no pattern gets the built-in default (the ISO
order for a date, `HH:MM` for a time or `H:MM am` on a twelve-hour
clock, `GG-PP` for a ghati, the number and the unit's name for a
duration) with a warning that names the missing key. A value of the wrong kind warns and renders the fallback
text; a date, time or ghati offers no selection keys.

## 6. The engine's API

`Intl::new(locales)` over sources rebuilt from packs, `Intl::from_tree(&tree)`
over a loaded `i18n/` root (`Tree::load`, `sdk_root()` for the SDK's
own), `set_locale(tag)`, `has(key)`, `render(key, &params) -> Rendered {
text, parts, resolved_from, is_fallback, warnings }`,
`render_typed(&message)` for a `TypedMessage` (a generated accessor: its
key and parameters checked by the compiler), `render_source(source,
&params)` for tools. Parameters (`Value`) are `Str`, `Int`, `Num`,
`Entity(key)` or `List`; `Value::catalogued(Graha::Sun)` is the typed
entity, `Value::entity("graha.SUN")` the textual one. Resolution walks
the current locale then its declared fallbacks; every result says which
locale answered. Messages are parsed once per key and cached. The
measurements are in §10.

The runtime API (`runtime`): `load_pack(&bytes)` takes a `.tpack` or a
`.tbundle` after construction, adding a locale (its metadata from the
file, its plural rules from ICU4X) or a namespace, or replacing entries
already loaded under the same key, and returns the record the provenance
envelope keeps (locale, namespaces, entries, replaced, SHA-256);
`set_override(locale, key, source)` patches one message in memory,
checked as it is set, standing before the locale's own entry and before
any fallback until `clear_override` or `clear_overrides`; `report()`
lists every locale with its coverage of the base keys, the files loaded
and the overrides in force. A render says when an override answered
(`Rendered::is_override`) as it says which locale did. The parse cache
forgets what the runtime API replaces, so a replaced or overridden
message renders anew at once.

## 7. Validation

The gates, in one report with a coverage table and diagnostics sorted
errors first: every message parses (offset reported); metadata is sound
(numbering system known, contexts distinct, fallbacks loaded, the chain
ending in the base, list patterns holding `{0}` and `{1}`); no key
outside the base; strict locales complete; a translation uses only the
base message's parameters with agreeing types (a warning when it drops
one, or adds markup the base lacks); every selector key is valid for
its type (a plural category the locale produces, a value of the
context, an entity key or a gender); `:msg` targets and the
`pattern=` a date, time or ghati function names exist in the base; a
translation that links another message with `:msg` forwards every
parameter and is not asked to name them; a matcher cannot select on a
date, time or ghati; the catalogue is the authority for entities: an entity record's key and an
`:entity` literal must be catalogue keys (an error naming the nearest
key), `kind=` must name a catalogue kind (an error), and a catalogue key
the entity namespace does not describe yet is a warning, since its
record can come later; entity genders are context values and forms match
the base (a warning for a missing form). The report also counts, per
closed catalogue kind, how many members the base locale describes: the
signs and the nakshatras are complete, nine grahas and the Lagna are
described, the rest await the migration of the name tables; reported,
not gated. Fourteen mistakes are proven caught by the tests.

## 8. Packs

`.tpack`, one locale and one namespace per file: magic `TPK1`, format
version 2, locale, namespace, entry count, arena length, a CRC32 of the
body, a SHA-256 of the body for the provenance envelope (ADR-0020), the
locale's metadata as JSON (or none, when the pack's bundle carries it),
a key table (16 bytes an entry: key offset and length, kind, value
offset and length) sorted by key, and a byte arena. Messages are source
text; entities are length-prefixed forms, gender and glyph. Reads are
zero-copy and bounds-checked; parsing verifies the checksum and the key
order and decodes values on access.

`.tbundle`, one locale with every namespace: magic `TPB1`, format
version, locale, namespace count, a CRC32 and a SHA-256 of the payload;
the payload is the metadata once, an index (namespace name and length),
and the namespaces' packs without metadata, each verified on its own
terms. The bundle is smaller than the separate packs by the metadata it
pays once (§10). `locales_from_packs` rebuilds locales from any mix of
packs and bundles, a pack without metadata being refused unless its
locale is already known; an engine over packs renders the same bits as
one over sources (tested). Per-namespace slicing stays a build option
(`build` against `build --bundle`). Interpretation packs use the same
container with citation fields and a licence.

## 9. Typed accessors

One model from the base locale (namespaces as groups, messages with
typed parameters in name order, entity kinds with their keys in source
order, contexts, the forms every entity has), one emitter per target:

- TypeScript: string unions for contexts and entity kinds
  (`GrahaKey = 'graha.SUN' | …`), `MessageKey`, an `EntityForms`
  interface, a `Renderer` interface, and `messages(renderer)` returning
  nested accessors: `t.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER',
  bhava: 7 })`; `t.sdk.entity.graha.SUN()`.
- Dart: enums with a `key` field, `EntityForms`, a `Renderer` interface,
  and one class per group: `Messages(renderer).sdk.reason.grahaInBhava(
  graha: GrahaKey.jupiter, bhava: 7)`.

- Rust: an enum per context with its `key()`, and a struct per message
  with typed fields (`i64`, `f64`, `String`, the context's enum, the
  catalogue's own enum for `kind=graha`, `Vec<Value>` for a list)
  implementing `TypedMessage { KEY, params() }`, in modules that follow
  the key's segments: `sdk::reason::GrahaInBhava { bhava: 7, graha:
  Graha::Jupiter }`, rendered by `Intl::render_typed`. Entities need no
  accessor: the catalogue's enums are the typed keys. The SDK's own
  namespaces are generated into `teistro_intl::messages` by `cargo xtask
  gen intl` and held by `check-intl`; a consumer runs `teistro-intl gen
  --target rs` on its own `i18n/`.

The generated code holds keys and parameter shapes only, never text
(tested), so a message changes or is overridden at runtime without
regenerating. The TypeScript and Dart surfaces compile clean and reject
wrong usages (six in TypeScript, five in Dart) as compile errors (spike
4's harnesses); the Rust surface compiles as part of the crate. Python
and Java follow the same model with their bindings.

**In the bindings.** `cargo xtask gen intl` writes the accessors into
both packages beside the Rust ones, and `check-intl` holds all three to
the sources: `bindings/node/lib/messages.js` with its declarations
`messages.d.ts` (a `.js` and a `.d.ts` rather than a `.ts`, because the
package ships no compiler), and `bindings/dart/lib/src/messages.dart`.
Each accessor wraps its parameters as the engine's JSON takes them
(`{"$entity": "graha.JUPITER"}`, `{"$date": {...}}`), so a caller passes
a key or a value and never a tagged object. A context reaches them
through the layer:

```js
ctx.messages.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 });
ctx.entity('graha.SUN').name;
```

```dart
ctx.messages.sdk.reason.grahaInBhava(graha: GrahaKey.jupiter, bhava: 7);
ctx.entity('graha.SUN').name;
```

An entity's forms come from a boundary entry point of their own,
`ts_intl_entity`, which hands back every form the locale gives (`name`,
`prose`, `iast`, `short` and any it adds), the glyph and the gender as a
JSON object lent until the next call; a key the locale chain does not
carry is `UNSUPPORTED` naming the locale that was asked. In Dart the
accessors are their own entry point (`package:teistro/messages.dart`),
because the locale's names are its own and one of them (`Gender`) is a
word the catalogue uses too.

## 10. Performance budget and benchmark

Budget: a render under 5 µs, a pack lookup under 1 µs, a pack of ten
thousand entries verified under 5 ms, a locale loaded under 1 ms per
thousand entries. The benchmark is `crates/intl/benches/intl.rs`
(criterion, over the SDK's own sources), measured in one session on
2026-09-06 (release, Apple Silicon; rows compare within the table):

| operation | measured |
|---|---:|
| render: a literal | 0.34 µs |
| render: an ordinal select with an entity | 2.28 µs |
| render: an entity and a zodiac angle | 2.01 µs |
| parse: a matcher with two declarations and three variants | 2.21 µs |
| pack: build the Nepali entity namespace | 7.50 µs (49 entities) |
| pack: parse and verify it | 1.42 µs (6.2 KB) |
| pack: look up `graha.SUN` | 0.50 µs |
| bundle: parse and verify the Nepali locale | 2.92 µs (8.4 KB, two namespaces) |
| engine: build from every pack, plural rules included | 68 µs (four packs) |

Sizes (`teistro-intl report`): the English entity namespace 49 entries, 7 097 source bytes, 4 989 pack bytes; its reason namespace 13 entries, 1 464 against 1 965 (the metadata is the overhead of a small pack); the Nepali entity namespace 8 298 against 6 215, its reason namespace 1 944 against 2 470; the English bundle 6 692 bytes against 6 954 for its two packs, the Nepali 8 404 against 8 685. The generated surfaces: TypeScript 109 lines (6.3 KB), Dart 257 (8.5 KB), Rust 286 (9.3 KB), no message text in any (tested).

## 11. Tests

Parser unit tests per construct with error offsets; the round-trip
property test; robustness property tests; evaluation tests in both
languages for every function and selector kind, including fallback,
missing keys and missing parameters; the catalogued value and the
catalogue's sign keys; the generated typed messages rendering the same
bits as their keys; validation tests for every gate, the catalogue's
among them; pack round trip, corruption and truncation; the bundle's
round trip, its size against the separate packs, a lone metadata-less
pack refused, corruption and truncation; generator tests for key
coverage and the absence of text; the date functions (`tests/dates.rs`:
the architecture page's Bikram Sambat example `२०८१/०५/१९ गते`, Gregorian
dates in three styles and both locales, a Julian date through the linked
patterns, an era's year and short form, `pattern=`, the conversion
`calendar=BIKRAM_SAMBAT` against the calendar crate's own answer, an
unknown calendar warned, times, a datetime, ghatis, durations
pluralised, the defaults with their warning when a locale declares no
patterns, the number options, the analysis's types and links); the
command line's value parsing and `extract`; the runtime API: an override standing before the entry and the
fallback, refused when it does not parse, cleared back, a pack adding a
namespace, a replacement rendering anew, a bundle adding a selectable
locale that falls back to the base, the report; the language harnesses
under `spikes/04-teistro-intl/harness/` (44 tests and a doctest in the
crate). To come: fuzzing under
`cargo-fuzz` and cross-binding rendering parity on a snapshot set.

## 12. Localisation

This page is the localisation layer; the SDK's namespaces are
`sdk.entity`, `sdk.state`, `sdk.rule`, `sdk.reason`, `sdk.ui`,
`sdk.calendar` and `sdk.glyph`, with `sdk.entity` and `sdk.reason` shaped
here.

## 13. Open questions

- Bidi isolation around interpolated values in right-to-left locales
  (the grammar's bidi marks are accepted; rendering policy is open).
- Whether `:entity` form names are a closed set per catalogue kind or
  open per locale (open in the spike).
- The composite provider's precedence rules once a binding loads packs
  from several places (baked, blob, filesystem): the Rust runtime API
  loads in call order, later entries replacing earlier ones.
- The twenty baseline entity types without a catalogue kind, and the
  synonyms the engine's names carry (not a form yet).
- The date functions' next steps: day-period ranges per locale (the
  ranges are the same in every locale today, which suits English and
  Nepali and will not suit every language), abbreviated month names for
  Nepali (the locale links the full names), `:duration` over several
  units at once, and the `zone` option once the time crate's zoned
  instants cross the port.
- Transliteration, XLIFF, and `migrate baseline` for the four launch
  languages' name tables into `sdk.entity`.
