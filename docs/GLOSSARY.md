# Glossary

Terms as this project uses them. Where astrology uses several names for one
thing, the SDK picks one canonical key and lists the others as aliases in the
locale packs, never in code.

| term | meaning here |
|---|---|
| ayanamsha | the offset between the tropical and sidereal zodiacs at an instant; a numbered catalogue entry (Lahiri, Raman, Krishnamurti, ...), value computed by the `astro` layer or a provider override |
| calculation version | an integer in every result that bumps whenever any numeric output changes for identical input; the cache key with the input and settings hashes (ADR-0020) |
| confidence mark | V (verified against the baseline engine or a text), T (traditional, awaiting a citation) or S (shape only) on every table row; only V ships (ADR-0018) |
| clean room | the policy in `CLEAN_ROOM.md`: what may be taken from each source by rank, and what may never be |
| crux | an open verification item in `01-research/feature-universe/19-verification-cruxes.md`: a text that disagrees with itself, a third party that disagrees with the baseline engine, or a convention the SDK had to choose |
| BS | Bikram Sambat, the official calendar of Nepal; table-driven for the officially published span |
| binding | a generated language package that exposes the SDK's API idiomatically in that language |
| capability | a declaration by a provider (ephemeris, calendar, timezone, locale data) of what it can compute; the SDK validates settings against it |
| base locale | `en-Latn`, the locale whose sources define every key and parameter; every other locale is validated against it (Teistro Intl) |
| namespace (intl) | one JSON file per locale, `sdk.entity`, `sdk.reason` and the like; keys are dotted paths inside it; consumers own every namespace not prefixed `sdk.` |
| pack (`.tpack`) | a compiled namespace of one locale: a sorted key table over a byte arena with a checksum, a content hash and the locale's metadata; what a runtime loads |
| context (intl) | a closed set of values a message may select on, declared in `_meta.json` (`gender: [m, f, n]`) and typed in the generated accessors |
| typed accessor | generated per binding from the base locale: a function per message with typed parameters, keys only, text from packs |
| content pack | a data bundle of interpretation text with citations, versioned and loadable at runtime; separate from locale packs |
| context | the SDK's per-consumer handle holding settings, providers and caches; no global state |
| core | the language-native library every binding wraps |
| dasha | a planetary period system; the SDK has a registry of systems, each a plug-in |
| deliberate difference | a registry row that explains a known divergence between the SDK and a golden-vector source (a convention the SDK does not copy, or a defect the source has), so the harness reports it instead of failing (`05-testing/01-golden-vectors.md`) |
| fixture | one golden-vector file: an input, a complete settings profile with its hash, provenance, and the recorded outputs of one source under one profile (`fixtures/README.md`) |
| golden vector | a recorded output of another implementation or a text that the SDK must reproduce within the tolerance for the provider class, cited to its source (ADR-0022) |
| drik | computation by observed astronomy (Swiss Ephemeris, Teimeris, JPL); opposed to Surya Siddhanta or other siddhantic models |
| footedness | the odd or even classification of signs in groups of three from Aries, used for the duration count in rashi dashas; a distinct type from sign parity, which governs the sequence direction |
| entity | a named astrological object with a stable key: graha, rashi, nakshatra, tithi, karana, yoga, vara, varga, dignity, and so on |
| ephemeris provider | the consumer-supplied implementation of the ephemeris port |
| IDL | the machine-readable description of the API from which every binding and the API reference are generated |
| kernel | one implementation of the algorithmic shape a family of variants shares (udu dashas, rashi dashas, vargas, bala schemes, rules); the variants are rows of data over it (ADR-0017) |
| key | a stable, language-neutral identifier for an entity, state or rule; engine output carries keys, never display strings |
| mark and continue | the rule that an unsourced or unverified variant is registered but not implemented and is refused with a reason, never silently replaced by a default |
| Nas | the canonical angle type: an `i64` count of nanoarcseconds; every classification is integer arithmetic on it (ADR-0016) |
| locale pack | a data bundle mapping keys to display strings for one language and script, plus numerals, plural rules and formatting patterns |
| module | an independently shippable unit of the SDK with an explicit dependency list; the tree-shaking unit |
| panchanga | the five limbs of the Hindu day (tithi, vara, nakshatra, yoga, karana) and the derived timings |
| port | an interface the SDK depends on but does not implement: ephemeris, calendar, timezone, locale data, geo |
| profile | a named bundle of settings representing a school or a product default (for example "Nepali default") |
| resolution (calendar) | whether a date came from an authority's table, from computation, or from a table that disagrees with computation (`divergent`), reported on every date |
| row | one variant of a family expressed as data over a kernel, with citations and a confidence mark |
| settings snapshot | the complete set of computational choices a result was produced under; stored with the result |
| siddhanta | the computational model of planetary motion: drik or Surya Siddhanta |
| varga | a divisional chart (D-n) |
