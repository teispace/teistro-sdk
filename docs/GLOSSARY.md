# Glossary

Terms as this project uses them. Where astrology uses several names for one
thing, the SDK picks one canonical key and lists the others as aliases in the
locale packs, never in code.

| term | meaning here |
|---|---|
| ayanamsha | the offset between the tropical and sidereal zodiacs at an instant; a numbered catalogue entry (Lahiri, Raman, Krishnamurti, ...), value computed by the ephemeris provider |
| BS | Bikram Sambat, the official calendar of Nepal; table-driven for the officially published span |
| binding | a generated language package that exposes the SDK's API idiomatically in that language |
| capability | a declaration by a provider (ephemeris, calendar, timezone, locale data) of what it can compute; the SDK validates settings against it |
| content pack | a data bundle of interpretation text with citations, versioned and loadable at runtime; separate from locale packs |
| context | the SDK's per-consumer handle holding settings, providers and caches; no global state |
| core | the language-native library every binding wraps |
| dasha | a planetary period system; the SDK has a registry of systems, each a plug-in |
| drik | computation by observed astronomy (Swiss Ephemeris, Teimeris, JPL); opposed to Surya Siddhanta or other siddhantic models |
| entity | a named astrological object with a stable key: graha, rashi, nakshatra, tithi, karana, yoga, vara, varga, dignity, and so on |
| ephemeris provider | the consumer-supplied implementation of the ephemeris port |
| IDL | the machine-readable description of the API from which every binding and the API reference are generated |
| key | a stable, language-neutral identifier for an entity, state or rule; engine output carries keys, never display strings |
| locale pack | a data bundle mapping keys to display strings for one language and script, plus numerals, plural rules and formatting patterns |
| module | an independently shippable unit of the SDK with an explicit dependency list; the tree-shaking unit |
| panchanga | the five limbs of the Hindu day (tithi, vara, nakshatra, yoga, karana) and the derived timings |
| port | an interface the SDK depends on but does not implement: ephemeris, calendar, timezone, locale data, geo |
| profile | a named bundle of settings representing a school or a product default (for example "Nepali default") |
| settings snapshot | the complete set of computational choices a result was produced under; stored with the result |
| siddhanta | the computational model of planetary motion: drik or Surya Siddhanta |
| varga | a divisional chart (D-n) |
