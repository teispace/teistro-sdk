# Prashna and horary

Status: `research`, 2026-09-04. Checked against the baseline engine's prashna module
(query-moment chart, void-of-course Moon, yes/no score, timing unit, topic,
arudha), JHora's prashna entry modes (1–108, 1–249, 1–1800), Shri Jyoti Star
(KP and Prashna Marga), Kala's prashna module and Solar Fire's horary page.

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| chart at the moment of the query with all natal machinery | time, place | | yes | all | P0 |
| number-based prashna: 1–108 (nakshatra pada), 1–249 (KP sub), 1–1800 | number | lagna set from the number's arc | partial | JHora, PyJHora, SJS | P0 |
| Prashna Marga techniques: arudha (querent's chosen point), chhatra rashi, sprashtanga, Prashna lagna versus Arudha lagna comparison | | | partial (arudha) | SJS, Kala | P1 |
| Moon void-of-course (Vedic sign-based and Western aspect-based) | Moon aspects before sign exit | Ptolemaic (the baseline engine), Lilly | yes | all | P0 |
| yes/no scoring from lagna lord, Moon, benefics in kendra, applying aspects | | scoring schemes are school-specific: rule pack | yes | The baseline engine, some tools | P0 |
| timing of the event from lagna quality (movable, fixed, dual) and house counts | | | yes | | P0 |
| topic detection from the strongest planet's house | | | yes | | P0 |
| KP horary: ruling planets, sub-lord of the queried cusp, significators, fructification timing | KP module | | partial | all KP tools | P0 |
| Ashtamangala prashna (Kerala) with cowrie counts | | | no | rare | P2 |
| Western horary: Lilly's considerations before judgement (early or late ascendant, Moon void, Moon in via combusta, Saturn in 7th, ...), planetary hour ruler agreement, receptions, translation and collection of light, frustration, prohibition, refranation | tropical chart, dignity tables | | no | Solar Fire, Delphic Oracle | P1 (`western`) |
| horary chart search over a database | | | no | Solar Fire | P2 |

## Closing checklist

- Treat the yes/no score as a rule pack with a named scheme so alternative
  schemes can be added without engine changes.
- KP horary requires the KP ayanamsha; the profile enforces it.
