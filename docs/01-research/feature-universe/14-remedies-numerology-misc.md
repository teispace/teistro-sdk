# Remedies, numerology, Lal Kitab, Pancha Pakshi, namakarana and other applications

Status: `research`, 2026-09-04. Checked against the baseline engine's jataka package and
the Astro-Vision product line (GemFinder, DigiTell numerology, NameFinder,
Panchapakshi, Parihara).

## Remedies

| feature | inputs | baseline | field | tier |
|---|---|---|---|---|
| functional benefic and malefic classification per lagna (trikona and dusthana lordships) | lordships | yes | all | P0 |
| gemstone recommendation with safety rules (lagna lord, never a functional malefic, maraka lords only if trikona lords), wearing weekday, hora, finger, metal | | yes | Astro-Vision GemFinder, PL | P0 |
| mantras and japa counts, donations, fasting, deity worship per planet and per dosha | tables | yes | all | P0 |
| dosha-specific rituals (pooja, ritual, mantra) | | yes | | P0 |
| priorities and reason codes in four languages | | yes | | P0 |
| Lal Kitab remedies (upaya) by planet and house | Lal Kitab chart | yes | some | P0 |
| Parihara (Kerala practice), rudraksha, yantra, colour and direction guidance | tables | partial (lucky elements) | Astro-Vision | P1 |

Remedies are a rule pack over chart state (Principle 6). The SDK ships the
engine and the baseline engine's tables as a pack; consumers can ship their own.

## Numerology

| feature | baseline | tier |
|---|---|---|
| Pythagorean (with master numbers), Chaldean (no 9, compound numbers): destiny, name, soul urge, personality, birth day, life path | yes | P0 |
| name compatibility with the chart's ruling planet (mulanka mapping) | yes | P0 |
| Lo Shu grid, personal year and month cycles, pinnacles and challenges | no | P1 |
| Indian numerology (Vedic) variants and akshara-based numerology | partial | P1 |

## Lal Kitab

| feature | baseline | tier |
|---|---|---|
| house-number based chart, pakka ghar, exalted and debilitated houses, sleeping planets, three rinas (debts), Varshkundali year cycle, seven yogas, remedies | yes | P0 |
| Lal Kitab teva (traditional chart drawing) geometry | no | P1 |

## Pancha Pakshi

| feature | baseline | tier |
|---|---|---|
| five birds by nakshatra and paksha, day and night yamas, activity cycles (ruling, eating, walking, sleeping, dying), favourable windows | yes | P0 |
| detailed Tamil Pancha Pakshi with sub-activities and relationships between birds | partial | P1 |

## Namakarana (naming)

| feature | baseline | tier |
|---|---|---|
| Moon nakshatra pada syllable (akshar) in Devanagari and Latin | yes | P0 |
| name validation by phonetic matching (variants, leading clusters, Levenshtein) | yes | P0 |
| name corpus suggestions by gender and akshar (offline) and an LLM path (stays in the application, not the SDK) | yes | P0 (offline) |
| numerology compatibility of the chosen name | yes | P0 |

## Research and statistics

| feature | baseline | tier |
|---|---|---|
| batch chart computation, frequency statistics of placements and rules over a set | yes | P0 |
| synastry and composites (see matching) | yes | P0 |
| rule search over a chart database (yoga search) | partial | P0 (batch rule evaluation) |

## Rashifal (sign forecasts)

| feature | baseline | tier |
|---|---|---|
| transit context for a period (snapshot, ingresses, stations), per-rashi overlays, scoring with weights, life-area mapping, lucky elements | yes | P0 |
| text generation | application concern (LLM) | out of scope; the SDK gives the scored context and deterministic fallback text via the interpretation packs |

## Closing checklist

- Every table in this page becomes a data pack with citations; the engines
  are small and generic.
- Namakarana's Devanagari transliteration and phonetic matching are
  localisation-layer concerns; design them there (`platform/04`).
