# Adding a language

Status: `draft`, revised 2026-09-04 for Teistro Intl; becomes a worked
example when the CLI exists (Phase 1).

## For a consumer (no SDK changes)

1. Pick the locale tag with its script: `ta-Taml-IN`, `en-Latn-NP`.
2. `teistro-intl extract --locale <tag> --namespaces sdk.entity,sdk.rule,...`
   produces `i18n/<tag>/` with every key and the base-locale text beside
   it, plus a `_meta.json` template.
3. Translate. Keep placeholders and selectors intact; add the plural and
   select cases your language needs with MF2 `.match`; fill `_meta.json`
   (direction, numbering system, date patterns per calendar, fallback
   chain, honorific default, contexts).
4. `teistro-intl validate` refuses missing keys (unless the fallback chain
   covers them and partial coverage is declared), placeholder mismatches,
   uncovered selector cases, unknown keys and MF2 syntax errors.
5. `teistro-intl build --locales <tag> --namespaces ...` produces `.tpack`
   blobs sliced to what you need.
6. `teistro-intl gen --target ts` (or `dart`, `py`, `rs`, `java`) if you
   also localise your own namespaces; the SDK's namespaces already have
   generated accessors in every binding.
7. Load at runtime: `context.intl.loadPack(bytes)`; the SDK's packs are
   untouched and lookups fall back per key along your declared chain.
   `context.intl.overrides({...})` patches individual messages without a
   rebuild.
8. `teistro-intl report` shows coverage; render a full chart in the new
   locale with the snapshot tool and review it with a native reader.

## For the SDK team (shipping a new core language)

Everything above, plus: the sources go under `i18n/<tag>/`, the locale
joins the shipped list (`strict` completeness gated for every key), gets a
baked feature in the binding packaging, has a native reviewer sign-off
recorded in the pull request, and appears in the coverage report in the
docs.

## Interpretation packs

Same pipeline under `packs/interpret/<tag>/`; every entry carries at least
one citation; the pack manifest carries a licence; a pack may ship
separately from the SDK and may be commercial.
