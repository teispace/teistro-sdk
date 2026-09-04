# Localisation and internationalisation

Status: `research`, revised 2026-09-04 after Q10 (one opinionated,
centralised standard in the manner of next-intl and slang). Feeds
`02-architecture/03-localization-architecture.md`, which specifies the
standard, named **Teistro Intl**. Sources: the baseline engine localisation code,
ICU4X's data-provider design, Unicode MessageFormat 2.0 (stable in CLDR
47), CLDR plural rules and numbering systems, the next-intl message
documentation and the slang package documentation (both read on
2026-09-04).

## The brief

The baseline engine's localisation works but its architecture is poor: four-language
name blocks inside computation results, several egress points with
different rules, no message syntax, string replacement for templates,
hand-written transliteration, and no tooling. The SDK needs the opposite:
**one standard way**, opinionated, typed, tool-driven, extensible by
consumers without touching the core, scalable to any number of languages,
and usable by consumer applications for their own text so a Teispace
product has one i18n system end to end.

## What next-intl and slang get right (and what to adopt)

| property | next-intl (Next.js) | slang (Flutter) | adopt |
|---|---|---|---|
| message files | one JSON per locale, nested objects, namespaces by nesting | `<namespace>_<locale>.json` (or YAML, CSV, ARB); nested objects | nested JSON per locale per namespace, YAML accepted as source |
| message syntax | ICU MessageFormat 1 (arguments, plural, selectordinal, select, rich tags, escaping) | `$param` or `{param}` interpolation; plural by node keywords (`one`, `other`); contexts for gender with generated enums; linked translations `@:path`; rich text modifier | Unicode MessageFormat 2.0 syntax (the successor of ICU MF1, standard, with `.match` selectors for plural, ordinal and select, markup for rich text, functions for formatting); linked messages and typed contexts as SDK conventions on top |
| typing | TypeScript augmentation of a global `Messages` type so keys and params are checked | code generation of typed accessors (`t.login.success`, `t.hello(name:)`), contexts as enums, interfaces for polymorphic groups | code generation of typed accessors in every binding from the base locale; contexts as enums; parameter types declared in the message |
| scoping | `useTranslations('About')` gives a scoped `t` | `t.<namespace>.<path>` | scoped accessors per namespace and module |
| rich text | `t.rich` maps tags to components; `t.markup` for HTML | `(rich)` modifier to TextSpan | MF2 markup (`{#b}...{/b}`) rendered by binding-specific renderers (React nodes, Flutter spans, plain text elsewhere) |
| formatting | `Intl` based named formats for numbers, dates, lists | `intl` package NumberFormat and DateFormat via typed parameters | MF2 functions bound to SDK types: `:number`, `:integer`, `:date` (calendar-aware), `:time`, `:dms`, `:zodiac`, `:ghati`, `:entity`, `:list` |
| fallback | default-locale fallback for incomplete translations; `t.has()` | strategies `none`, `base_locale`, `base_locale_empty_string`; `(fallback)` per map | declared fallback chain per locale; `strict` for SDK-shipped packs (gate), `base` for consumer packs; `has()` |
| lazy loading | message splitting by route or namespace | deferred loading of secondary locales on web; `lazy: false` option | packs sliced by namespace and locale; lazy load per namespace |
| runtime overrides | not a focus | `translation_overrides: true` and `LocaleSettings.overrideTranslations()` for backend-driven content | first-class: load and override packs at runtime; a consumer adds a language without rebuilding |
| tooling | linting and platform integrations (Crowdin) recommended | CLI: `analyze` (missing and unused), `apply`, `clean`, `edit move/copy/delete/add`, `normalize`, `stats`, `outdated <key>`, `migrate arb`; config `base_locale`, `fallback_strategy`, `key_case`, `param_case`, `pluralization.auto`, `obfuscation`, `autodoc` | a single CLI (`teistro-intl`) with the slang command set plus `validate`, `build`, `gen`, `extract`, `diff`, `report`, `export xliff`, `import xliff`, `migrate` from the baseline engine's TypeScript records and from ICU MF1 and ARB |
| conventions | keys cannot contain `.`; select values alphanumeric | key case transforms; sanitisation of reserved words | one key grammar, enforced |
| RTL | documented patterns | Flutter handles | locale metadata carries direction; renderers honour it |

## What neither has, and the SDK needs

1. **Engine keys as first-class values.** An entity key (`graha:MARS`) is
   not a string to translate; it is a value that renders through
   `{$graha :entity form=prose}` to the right name form, gender and
   script. The message system must know the SDK's catalogue.
2. **Structured name forms.** An entity has several forms (short, name,
   prose, with honorific, glyph) and grammatical gender; a pack entry for
   an entity is an object, not a string, and the accessor is typed.
3. **Numbering systems and scripts** as locale metadata (Devanagari digits
   for `ne` and `hi`, Latin for `en`), applied by every formatting function.
4. **Calendar-aware dates**: a date formats in the requesting calendar (BS
   with गते, AD, Indian lunisolar) through the SDK's calendar module.
5. **Transliteration** as data (Devanagari to IAST, ISO 15919,
   Harvard-Kyoto, popular; and back), with akshar segmentation for naming.
6. **Interpretation packs with citations and licences**, versioned
   separately from UI text, possibly commercial.
7. **Cross-language parity**: the same pack renders byte-identically in
   Node, Python, Dart and Rust (one engine, one set of tests).
8. **Pack slicing** so a Nepali panchanga app ships kilobytes, and a
   build-time report of what shipped.
9. **Grammatical agreement beyond plural**: gender, honorific level,
   Sanskrit dual, case marking of interpolated names (a name in Nepali may
   need a postposition that depends on the noun); handled by `select` on
   typed contexts and by name forms, not by string concatenation.

## ICU4X as the data engine

CLDR plural rules, numbering systems, locale canonicalisation and fallback,
list formatting and the calendar arithmetic come from ICU4X crates; the SDK
does not reimplement CLDR. ICU4X does not yet ship MessageFormat 2, so the
SDK implements the MF2 subset it needs (placeholders, `.match` with plural,
ordinal and select, markup, a fixed function set with options), keeping the
syntax standard so MF2 editors and translation platforms can be used. This
is a small, fuzzed parser and evaluator; JavaScript's `messageformat` 4.0
exists as a reference implementation of the syntax.

## Requirements the architecture must meet (unchanged, restated)

Keys only in the engine; a message syntax with plural, select and markup;
numerals and scripts; transliteration; calendar and date formatting;
grammatical agreement; a pack authoring workflow with validation, build,
diff and extraction; consumer-authored languages at runtime; interpretation
packs separate from locale packs; size by slicing; and typed accessors in
every binding so a wrong key or a missing parameter is a compile error, not
a runtime string.

## Open items for the design

- The base (source) locale: recommended `en` for authoring and validation,
  with `ne` complete and reviewed as the product's first language (Q20).
- Source format: JSON canonical, YAML accepted (Q20).
- Whether consumer applications are expected to use Teistro Intl for their
  own UI text (recommended yes; Q21).
