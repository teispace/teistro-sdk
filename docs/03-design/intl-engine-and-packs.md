# Teistro Intl: the engine, the sources and the packs

Status: `draft`, written 2026-09-05 from spike 4
(`spikes/04-teistro-intl/README.md`); revised in Phase 1 when the `intl`
crate and the `teistro-intl` CLI are built. Derives from
`02-architecture/03-localization-architecture.md`, ADR-0010, ADR-0020 and
ADR-0023. Names are the spike's; Phase 1 renames into the SDK's
catalogue without changing the shapes.

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
  namespace holds messages.
- **Keys** are dotted paths; a segment is a `camelCase` identifier or a
  catalogue key (`UPPER_SNAKE`); a full key is the namespace plus the
  path (`sdk.reason.strength.rank`, `sdk.entity.graha.SUN`).
- **Messages** are `MessageFormat 2` source strings.
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
| `:integer` | number | `select=ordinal` | exact numeric match, then the plural category | integer |
| `:number` | number | `minimumFractionDigits`, `maximumFractionDigits`, `select=ordinal` | as `:integer`, on the visible digits | number |
| `:dms` | degrees | `precision=deg\|min\|sec`, `signed=true` | | number |
| `:zodiac` | ecliptic longitude | `form=degree-in-sign\|sign-degree`, `precision`, `signNames=name\|short\|glyph\|iast` | the sign's key | number |
| `:entity` | catalogue key | `form=short\|name\|prose\|iast\|glyph`, `kind=<group>` | the bare key, the full key, the gender | entity key of the kind, or any entity key |
| `:list` | list | `type=and\|or` | | list |
| `:msg` | a key literal | | | none |

Numbers render in the locale's numbering system, grouping and
separators; angles in the same digits. `:zodiac` looks the sign name up
in `sdk.entity.rashi` along the fallback chain. `:msg` renders another
key with the same parameters, eight levels deep at most. An unknown
function, a missing parameter or a missing entity yields a warning in
the result and the standard fallback text (`{$name}`, the key); the
worst case renders the key itself, never a blank. Selection follows the
specification: for each selector the function's keys in preference
order, variants filtered and sorted by rank, `*` last. Phase 1 adds
`:date`, `:time`, `:datetime` (calendar-aware), `:ghati` and
`:duration`.

## 6. The engine's API

`Intl::new(locales)`, `set_locale(tag)`, `has(key)`, `render(key,
params) -> Rendered { text, parts, resolved_from, is_fallback, warnings }`,
`render_source(source, params)` for tools. Parameters are `Str`, `Int`,
`Num`, `Entity(key)` or `List`. Resolution walks the current locale then
its declared fallbacks; every result says which locale answered.
Messages are parsed once per key and cached. Measured: 0.5 µs for a
literal, 2.2 to 2.7 µs for a message with an entity, an ordinal select
or a `:zodiac`; 5 µs to parse a five-variant matcher.

## 7. Validation

The gates, in one report with a coverage table and diagnostics sorted
errors first: every message parses (offset reported); metadata is sound
(numbering system known, contexts distinct, fallbacks loaded, the chain
ending in the base, list patterns holding `{0}` and `{1}`); no key
outside the base; strict locales complete; a translation uses only the
base message's parameters with agreeing types (a warning when it drops
one, or adds markup the base lacks); every selector key is valid for
its type (a plural category the locale produces, a value of the
context, an entity key or a gender); `:msg` targets and `:entity`
literals exist and `kind=` names a group; entity genders are context
values and forms match the base (a warning for a missing form). Twelve
mistakes are proven caught by the spike's tests.

## 8. Packs

`.tpack`, one locale and one namespace per file: magic `TPK1`, format
version, locale, namespace, entry count, arena length, a CRC32 of the
body, a SHA-256 of the body for the provenance envelope (ADR-0020), the
locale's metadata as JSON, a key table (16 bytes an entry: key offset
and length, kind, value offset and length) sorted by key, and a byte
arena. Messages are source text; entities are length-prefixed forms,
gender and glyph. Reads are zero-copy and bounds-checked; parsing
verifies the checksum and the key order (1.4 µs for a 6 KB pack) and
decodes values on access (0.46 µs a lookup). A runtime rebuilds locales
from packs alone (65 µs for four packs with plural rules); an engine
over packs renders the same bits as one over sources (tested).

Phase 1 adds the bundle: one file per locale with every namespace behind
one metadata block, because the metadata is the overhead that makes a
small namespace's pack larger than its JSON; per-namespace slicing stays
a build option. Interpretation packs use the same container with
citation fields and a licence.

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

The generated code holds keys and parameter shapes only, never text
(tested), so a message changes or is overridden at runtime without
regenerating. Both surfaces compile clean and reject wrong usages (six
in TypeScript, five in Dart) as compile errors. Python, Rust and Java
follow the same model in Phase 1.

## 10. Performance budget and benchmark

Budget: a render under 5 µs, a pack lookup under 1 µs, a pack of ten
thousand entries verified under 5 ms, a locale loaded under 1 ms per
thousand entries. The benchmark is the CLI's `report`, whose rows the
spike's result page quotes.

## 11. Tests

Parser unit tests per construct with error offsets; the round-trip
property test; robustness property tests; evaluation tests in both
languages for every function and selector kind, including fallback,
missing keys and missing parameters; validation tests for every gate;
pack round trip, corruption and truncation; generator tests for key
coverage and the absence of text; the language harnesses under
`spikes/04-teistro-intl/harness/`. Phase 1 adds fuzzing under
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
- The bundle container's index and the composite provider's precedence
  rules (Phase 1, with the runtime overrides API).
