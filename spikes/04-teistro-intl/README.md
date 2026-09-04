# Spike 4: Teistro Intl

**Question.** Does one opinionated localisation standard hold up on real
content in the product's two first languages: JSON sources per locale
per namespace, `MessageFormat 2` syntax with a fixed function set bound
to the SDK's types, the base locale as the source of truth, validation
that catches a translator's mistakes, compiled packs a runtime can load
with nothing else, and typed accessors generated for TypeScript and Dart
so a wrong key or parameter is a compile error? And what does each piece
cost? **Result: yes, on every count, with four conventions changed by
what was found; the standard goes into
`docs/03-design/intl-engine-and-packs.md`.**

## The slice

```text
intl/                 teistro-spike-intl        the engine and the CLI: mf2 (data model, parser, checks,
                                                serialiser), source (the i18n/ conventions), analysis
                                                (signatures), render (evaluation, the SDK functions, ICU4X
                                                plurals, numbering systems, fallback), validate (the gates),
                                                pack (the .tpack container), generate (TypeScript, Dart)
i18n/en-Latn/         _meta.json, sdk.entity.json, sdk.reason.json   the base locale: 49 entities (grahas,
                                                rashis, nakshatras with short, name, prose, IAST, glyph,
                                                gender), 13 messages exercising every function and selector
i18n/ne-Deva-NP/      the same in Nepali, Devanagari digits, Indian grouping, a fallback chain to the base
harness/ts/           the generated sdk.ts, a runtime check, six wrong usages, a runner over tsc
harness/dart/         the generated lib/sdk.dart, a runtime check, five wrong usages, a runner over dart analyze
results/              intl.json (validation, sizes, generated line counts, timings), harness.json
```

The message set is small on purpose and complete in kind: a literal, a
linked message (`:msg`), ordinal houses (`.match` on `:integer
select=ordinal`), cardinal plurals with an exact `0`, selection on an
entity's gender (`.match` on `:entity`), selection on a declared context
(the native's gender), `:zodiac` and `:dms` angles in the locale's
digits, `:list` over entities, markup, a `:number` with fraction digits,
and a nested group (`strength.score`, `strength.rank`).

## How to run

```sh
cargo run --release -p teistro-spike-intl -- validate
cargo run --release -p teistro-spike-intl -- render --locale ne-Deva-NP sdk.reason.grahaInBhava \
  --param graha=@graha.JUPITER --param bhava=7
cargo run --release -p teistro-spike-intl -- build --locales ne-Deva-NP --namespaces sdk.entity
cargo run --release -p teistro-spike-intl -- gen --target ts,dart
cargo run --release -p teistro-spike-intl -- report
(cd spikes/04-teistro-intl/harness/ts && npm install && node run.ts)
(cd spikes/04-teistro-intl/harness/dart && dart pub get && dart run bin/run.dart)
cargo test -p teistro-spike-intl
```

Every command is a library function with a thin shell; `report` runs
`validate`, builds every pack, generates both targets and times the
engine, and writes `results/intl.json`. The crate's 32 tests include a
400-case property test that any generated message serialises and parses
back to the same tree, and two that arbitrary text and every prefix of
a valid message parse or fail without panicking.

## Measurements

Validation of the two locales: 62 keys each, 0 errors, 0 warnings; the
gate test proves twelve kinds of mistake are caught (an unknown key, an
undeclared parameter, a plural category the locale never produces, a
context value outside the declared set, a syntax error with its offset,
a dangling `:msg`, an unknown entity, an unknown `kind`, a selector key
that is neither an entity nor a gender, a missing key under strict
completeness, an unknown numbering system, an entity gender outside the
context).

| locale | namespace | entries | JSON bytes | pack bytes |
|---|---|---:|---:|---:|
| en-Latn | sdk.entity | 49 | 7 080 | 4 989 |
| en-Latn | sdk.reason | 13 | 1 464 | 1 965 |
| ne-Deva-NP | sdk.entity | 49 | 8 281 | 6 215 |
| ne-Deva-NP | sdk.reason | 13 | 1 944 | 2 470 |

A pack carries its locale's metadata (about 450 bytes) plus a 16-byte
table row per entry, so a sliced Nepali entity pack is 6.2 KB and the
whole Nepali locale 8.7 KB. Generated accessors: TypeScript 106 lines
(6.2 KB), Dart 242 lines (8.3 KB), no message text in either (tested).

| measurement | median µs | p90 µs |
|---|---:|---:|
| parse: a matcher with two declarations and five variants | 4.96 | 5.04 |
| render: a literal | 0.46 | 0.50 |
| render: ordinal select with an entity (parse cached) | 2.67 | 2.75 |
| render: `:entity` and `:zodiac` (parse cached) | 2.17 | 2.21 |
| pack: build the Nepali entity namespace (49 entities) | 7.83 | 8.12 |
| pack: parse and verify it (CRC, key order) | 1.38 | 1.42 |
| pack: look up `graha.SUN` (binary search, zero copy) | 0.46 | 0.50 |
| engine: build from four packs, plural rules included | 64.71 | 68.17 |

Harness: TypeScript type-checks the generated surface and rejects all
six wrong usages (an unknown entity key, a missing parameter, a value
outside a context, a string for a number, an unknown message, a key
outside the union); Dart analyses clean and rejects all five (the union
case has no Dart equivalent; keys are enums); both runtime checks see
six calls with the exact keys and parameters. Apple Silicon laptop,
release build, Node 26, TypeScript 5.9, Dart 3.13.

## What the spike found

1. **The standard's syntax is the final one, not the draft.** The
   architecture page quoted `.match {$count :integer} one {{…}}` from
   the technical preview; the stable syntax (LDML 47) declares selectors
   first: `.input {$count :integer} .match $count one {{…}} * {{…}}`.
   The engine implements the stable grammar in full (declarations,
   matchers, quoted patterns, literals, options, attributes, markup,
   escapes, the data-model checks) and the page is corrected.
2. **No `_params` sidecar.** Every parameter's type follows from the
   function applied to it (`:integer`, `:number`, `:dms`, `:zodiac`,
   `:entity kind=graha`, `:list`) or from a context (`:string` on a
   variable named like a declared context, or selected with that
   context's values); only a bare `{$name}` is text. The sidecar is
   dropped from the conventions.
3. **Entities select on their own gender.** `:entity` as a selector
   offers the bare key, the full key and the entity's gender, so
   agreement with a sign or a planet (`वृश्चिक स्त्री राशि हो`) needs no
   extra parameter and no string surgery; declared contexts stay for
   facts about people. `kind=` on `:entity` narrows the generated type
   to `GrahaKey` and lets the validator check literal keys.
4. **Ordinals need exact keys as much as categories.** CLDR gives
   Nepali one ordinal category for 1 to 4 and another beyond, but the
   suffixes differ inside 1 to 4 (पहिलो, दोस्रो, तेस्रो, चौथो); MF2's
   exact numeric keys (`1 2 3 4 *`) rank above categories and express it
   exactly, where the category alone would have been wrong. English
   needed its four categories. The validator refuses a category the
   locale never produces (`two` in Nepali).
5. **Packs keep source text.** Parsing costs 5 µs a message and is
   cached per key, so pre-parsed trees buy nothing; source text keeps
   packs diffable and a third smaller than JSON for entities. A pack
   carries the locale metadata so a runtime needs nothing else, and that
   is the overhead that makes a 13-message pack larger than its JSON:
   the Phase 1 container bundles a locale's namespaces behind one
   metadata block and keeps per-namespace slicing as a build option.
6. **Verification is cheap when values are lazy.** Checking a 6 KB pack
   costs 1.4 µs (CRC32 and key order); the first draft decoded every
   entity during the order check and cost 20 µs, and probed values
   during binary search at 2 µs a lookup against 0.46 µs now.
7. **Typed accessors are keys and shapes only.** TypeScript's string
   unions and Dart's enums carry the catalogue; text stays in packs, so
   a translation can change or be overridden at runtime without
   regenerating. The generators share one model built from the base
   locale's signatures, in source order so the Sun precedes the Moon.
8. **ICU4X does the CLDR work.** `icu_plurals` with compiled data has
   the cardinal and ordinal rules for both locales, and `fixed_decimal`
   operands make the visible fraction digits decide (`1.00` is `other`
   in English), as the specification requires. Numbering systems and
   grouping are eleven lines over the locale's metadata.

## What is not covered

- `:date`, `:time`, `:datetime` (the calendar module, Phase 1),
  `:ghati` and `:duration`, transliteration, and the term-style axis
  beyond the `iast` form.
- Rich rendering: the engine emits parts with markup; the React and
  Flutter renderers over them are Phase 1 binding work.
- Python, Rust and Java accessors; the model is target-agnostic.
- The remaining CLI commands (`extract`, `analyze`, `diff`, `report` as
  coverage over real lookups, XLIFF, `migrate`), YAML sources, runtime
  overrides and the composite pack provider.
- The bundle container (finding 5) and pack licences.

## What changes in Phase 1

- The design page carries the grammar subset, the function set with
  its options, the selection rules, the source conventions without the
  sidecar, the validation gates, the pack layout and the accessor
  shapes from here.
- The engine's parse cache becomes the pack loader's job (parse on
  load, once), and the container gains the locale bundle.
- The four launch languages' name tables come in through `migrate
  baseline` into the entity namespace whose shape this spike fixed.
