# Teistro Intl: the localisation standard

Status: `draft`, revised 2026-09-05 after spike 4, whose engine, sources,
packs and generators settled the conventions in
`03-design/intl-engine-and-packs.md` (the stable `MessageFormat 2`
syntax, no parameter sidecar, entities selecting on their own gender,
source text in packs, keys-only accessors). One opinionated, centralised
system for every string the SDK renders and for consumer applications that
want one i18n system end to end. Shaped by ICU4X's data model, Unicode
MessageFormat 2.0, next-intl's typed messages and slang's generated
accessors, tooling and runtime overrides.

## The one-paragraph version

Text lives in JSON files, one per locale per namespace, under `i18n/`.
Values use MessageFormat 2.0 syntax. The base locale (`en`) is the source
of truth for keys, parameters and structure; every other locale is
validated against it. A CLI compiles the files into versioned binary packs,
generates typed accessors for every binding so a key or a parameter error
is a compile error, extracts templates for new languages, and reports
coverage. At runtime a context loads packs (baked into the package, or
supplied as bytes), resolves a locale with a declared fallback chain, and
renders messages with formatting functions that understand the SDK's own
types: entities, angles, calendar dates, ghati-pala. Consumers add a
language by authoring files and loading the pack; they never touch the SDK.

## Layering

```
 computation (L2)  ──►  keys + numbers + structured results, never text
 interpret (L3)    ──►  narrative plans: (message key, slots)
 intl (L3)         ──►  rendered text in a locale, from packs, via MF2
 serial (L3)       ──►  JSON (keys), dossier text, report blocks
```

No computation crate depends on `intl`.

## Conventions (the opinionated part)

| convention | rule |
|---|---|
| directory | `i18n/<locale>/<namespace>.json`; `i18n/<locale>/_meta.json` holds locale metadata (direction, numbering system, date patterns per calendar, fallback chain, honorific default) |
| locale tags | BCP-47 with script: `en-Latn`, `ne-Deva-NP`, `hi-Deva-IN`, `sa-Deva`; the file directory uses the full tag |
| base locale | `en-Latn` is the source locale: complete by definition, keys and parameter names are defined there; a gate refuses a key present in another locale but not in the base |
| namespaces | one file per namespace; SDK namespaces are prefixed `sdk.` (`sdk.entity`, `sdk.state`, `sdk.rule`, `sdk.reason`, `sdk.ui`, `sdk.calendar`, `sdk.glyph`); consumers own everything else; namespace names are lowercase with dots for grouping |
| keys | nested JSON objects; a full key is the namespace plus the dotted path; segments are `camelCase` identifiers (`[a-z][A-Za-z0-9]*`); entity keys inside `sdk.entity` mirror the catalogue keys (`graha.MARS`) |
| values | MessageFormat 2.0 strings, or structured objects for entities (`{ "short": ..., "name": ..., "prose": ..., "gender": "m" }`) validated by the schema for that namespace |
| parameters | declared inline by MF2 usage; the type follows from the function (`{$count :integer}`, `{$graha :entity kind=graha}`) or from a context (`:string` on a variable named like a context, or selected with its values); a bare `{$name}` is text; no sidecar |
| plural and ordinal | `.input {$count :integer} .match $count one {{...}} * {{...}}` with CLDR categories and exact numeric keys (`1 2 3 4 *` for Nepali ordinals); ordinal via `:integer select=ordinal` |
| select and contexts | `.input {$gender :string} .match $gender m {{...}} f {{...}} * {{...}}`; a context declared in `_meta.json` (`contexts.gender: [m, f, n]`) becomes a typed enum in the generated accessors; an entity selects on its own gender and key (`.input {$rashi :entity kind=rashi} .match $rashi f {{...}} * {{...}}`) |
| rich text | MF2 markup `{#link href=$url}...{/link}`; renderers per binding; plain-text rendering strips markup |
| linked messages | the `:msg` function (`{sdk.ui.appName :msg}`) resolves another message; an SDK extension to MF2, documented as such |
| escaping | MF2 rules (`{{`, `}}` and `\{`) only; no other syntax |
| fallback | `_meta.json` `fallback: ["ne-Deva-NP", "en-Latn"]`; SDK-shipped locales must be complete (`strict`, gated); consumer packs may be partial (`base`) |
| forbidden | string concatenation of translated fragments in code; locale-specific logic in computation crates; hard-coded strings in bindings |

## Formatting functions bound to SDK types

| function | input | options | example |
|---|---|---|---|
| `:number`, `:integer` | number | `numberingSystem` (defaults from `_meta`), `minimumFractionDigits`, `signDisplay`, `select=ordinal` | `{$score :number minimumFractionDigits=1}` |
| `:dms` | angle in degrees | `precision` (`deg`, `min`, `sec`), `signed` | `24°09′37″` in the locale's digits |
| `:zodiac` | ecliptic longitude | `form` (`sign-degree`, `degree-in-sign`), `signNames` (`name`, `short`, `glyph`) | `12°34′ वृश्चिक` |
| `:ghati` | ghati-pala pair | | `12-05` |
| `:date`, `:time`, `:datetime` | SDK instant or civil date | `calendar` (default from the context), `pattern` or `style`, `zone` | a BS date renders `२०८१/०५/१९ गते` |
| `:entity` | an entity key | `form` (`short`, `name`, `prose`, `glyph`), `case` (for languages with case marking, from the pack's forms) | `{$graha :entity form=prose}` |
| `:list` | list of strings or entities | `type` (`and`, `or`, `unit`), `style` | CLDR list patterns |
| `:duration` | days or minutes | `style` | |
| `:msg` | a message key | parameters forwarded | linked messages |

Functions are the extension seam for consumers too: a consumer can register
a formatting function in their own namespace at runtime (bindings expose a
registration API).

## Packs

`teistro-intl build` compiles a locale directory into `.tpack` files: one
per locale per namespace, or bundled per locale. Format: magic, format
version, pack manifest (locale, namespace, key count, content hash,
licence, the locale's metadata), a sorted key table and a byte arena
holding message source text (parsed once on load; spike 4 measured
parsing at 5 µs a message and found pre-parsed trees buy nothing while
source text keeps packs diffable and smaller) and entity records.
Zero-copy load with CRC and bounds checks. Interpretation packs use the
same container with citation fields and a licence.

Providers: baked (compiled into a binding for its default locales), blob
(bytes at runtime), filesystem (development), composite with the declared
fallback chain.

## Typed accessors (generated per binding)

From the base locale, `teistro-intl gen` emits:

| binding | shape |
|---|---|
| TypeScript | a `Messages` type and a typed `t` with scoped accessors: `t.sdk.rule.GAJA_KESARI.summary({ graha: 'JUPITER' })`; string-key overload `t('sdk.rule.GAJA_KESARI.summary', {...})` typed by template literal types; contexts as string unions; rich variants `t.rich(...)` |
| Dart | slang-style classes: `t.sdk.rule.gajaKesari.summary(graha: Graha.jupiter)`; contexts as enums; `TextSpan` variants |
| Python | generated classes with typed methods and `TypedDict` parameters; `py.typed` |
| Rust | an enum per namespace of message keys with typed parameter structs; `intl.render(Message::SdkRuleGajaKesariSummary { graha })` |
| Java | generated classes | 
| C | key ids only (numeric) with a generic render call |

Consumers run `teistro-intl gen --target ts` (or `dart`, `py`, `rs`) on
their own `i18n/` to get the same typed experience for their own
namespaces. The generated code contains no strings, only keys and
parameter shapes; text always comes from packs, so a consumer can override
a message at runtime without regenerating.

## Runtime API (same in every binding, names adjusted to the language)

```
intl = context.intl                       // bound to the context's packs
intl.setLocale("ne-Deva-NP")              // explicit; no device detection in the SDK
intl.loadPack(bytes)                      // add or override, returns the manifest
intl.overrides({ "sdk.ui.title": "..." }) // in-memory overrides (slang model)
intl.has("sdk.rule.GAJA_KESARI.full")     // availability under the fallback chain
t = intl.t("sdk.rule")                    // scoped accessor (generated typed variant preferred)
intl.format.dms(123.456)                  // formatting functions callable directly
intl.transliterate(text, "deva", "iast")  // data-driven transliteration
intl.report()                             // loaded packs, versions, coverage per namespace
```

Locale resolution is explicit and deterministic; the SDK never reads the
device or environment locale (an application does that and calls
`setLocale`). Every resolution reports which locale answered
(`resolved_from`, `is_fallback`), so an application can mark untranslated
content and the coverage report is derived from real lookups. The worst
case renders the key itself, never a blank.

## Axes that are not the language

| axis | examples | why separate |
|---|---|---|
| script | Devanagari, IAST, ISO 15919, Tamil, Bengali | the same Sanskrit term is written in several scripts; `sa-Latn` is derived from `sa-Deva` by the data-driven transliteration with hand overrides where the mechanical result is wrong, and checked in |
| numerals | Arabic, Devanagari, Tamil, Bengali, Gujarati, Odia, Tibetan | a Nepali reader may want either; one mapping at the formatting boundary |
| term style | vernacular, Sanskrit in IAST, Sanskrit in Devanagari, both ("Guru (Jupiter)") | a professional report and a consumer app want different registers in the same language; a `termStyle` option on `:entity` and in `_meta.json` |
| calendar preference and formats | AD, BS, Saka; 12- or 24-hour; DMS style | orthogonal to language |

Rules: no machine-translated stubs (a plausible wrong astrological term is
worse than a visible fallback; `extract` scaffolds with base text flagged
`untranslated`); the variables a message may select on (subject gender,
formality, plurality) are fixed in v1 because adding one later invalidates
every translation of that message; regional variants (`hi-IN`) are
allowed through BCP-47 and fall back to the base language, never required.

## Composers

A composer returns a narrative plan: an ordered list of (message key,
slots). Plans are language-neutral, testable and serialisable; `intl`
renders them. The baseline engine's composers port as plans plus messages. Consumers
may add composers (code) and messages (packs) independently.

## The CLI: `teistro-intl`

| command | job |
|---|---|
| `init` | scaffold `i18n/` with `_meta.json` for the base locale |
| `validate` | schema, base-locale completeness per shipped locale, placeholder and selector parity, unknown keys, catalogue references, citations for interpretation namespaces, MF2 syntax |
| `build` | compile to `.tpack`, sliced by `--namespaces` and `--locales`; emits a size report |
| `gen` | typed accessors for `--target ts,dart,py,rs,java` |
| `extract --locale <tag>` | a template tree for a new language with base text beside each key |
| `analyze` | missing and unused keys (scans generated-accessor usage in consumer code where a language allows) |
| `apply`, `clean`, `edit move/copy/delete/add`, `normalize`, `stats`, `outdated <key>` | the slang command set for maintenance |
| `diff <a> <b>` | changes between two pack versions, for translators and the changelog |
| `report` | coverage per locale and namespace, generated into the docs |
| `export xliff`, `import xliff` | round trip with translation platforms (XLIFF 2.1) |
| `migrate baseline` | one-time import of the baseline engine's TypeScript name tables and interpretation records into `i18n/` (four locales, citations preserved); `migrate icu-mf1`, `migrate arb` for consumers |

Every command is also a library function so the docs site and CI call it
without shelling out.

## Gates

Pack validation on every push; base-locale completeness for shipped
locales; generated accessors diffed; cross-binding rendering parity on a
snapshot set per language; MF2 parser fuzzed; size per profile per locale
reported and bounded.

## Adding a language, from outside

`extract`, translate, `validate`, `build`, `loadPack` at runtime (or
contribute upstream). The walkthrough is
`09-guidelines/02-adding-a-language.md`.
