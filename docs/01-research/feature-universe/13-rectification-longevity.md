# Rectification and longevity

Status: `research`, 2026-09-04. Checked against the baseline engine's rectification
cascade (Bayesian stages: Tattwa prior, reported-time Gaussian, dasha-boundary
likelihood against life events, refinement passes, HPD intervals, hold-out),
its Ayurdaya and Maraka services, JHora, PL, SJS (rectification tools) and
Solar Fire (astro-lines rectification, life-event lists).

## Rectification methods in the field

| method | inputs | baseline | field | tier |
|---|---|---|---|---|
| event fitting against dasha boundaries (Vimshottari and others) with significations per event category | dated life events with precision and confidence | yes | PL, SJS, Kala (manual tools) | P0 |
| Tattwa and antara-tattwa cycle (sex of native, 90-minute cycles from sunrise) | sunrise, sex | yes (prior) | some Nepali and Indian practice | P0 |
| Pranapada lagna and the Moon (Pranapada in trine to Moon, or in a sign relative to the lagna by sex) | | no | classical | P1 |
| Kunda (Ravi–Chandra formula by sex) and Nisheka (conception) methods | | no | JHora (Kunda) | P1 |
| Ruling planets (KP) agreement with lagna and Moon sub-lords | KP | no | KP tools | P1 |
| navamsa lagna and D60 checks (varga consistency with known traits) | | no | practitioners | P2 |
| transit and progression fitting (Western): Solar arc directions to angles at event dates, primary directions | tropical | no | Solar Fire, Delphic Oracle | P1 (`western`) |
| astro-lines based (relocation lines through event places) | | no | Solar Fire | P2 |
| candidate scoring framework: uniform grid, log-likelihood combination, refinement, interval-first reporting with concentration and hold-out | | yes | unique to the baseline engine | P0 |

The baseline engine design is more rigorous than the field (interval-first with
validation); the SDK keeps it and makes stages pluggable so Pranapada, Kunda
and KP ruling planets become additional stages.

## Longevity

| feature | inputs | baseline | field | tier |
|---|---|---|---|---|
| Ayurdaya: Pindayu, Amsayu, Naisargikayu with haranas (reductions) and the choice of method by strongest of lagna, Sun, Moon | positions and dignities | yes (average of Pindayu and Amsayu headline, spread across three) | JHora (via Shadbala), PL | P0 |
| three pairs (Jaimini) longevity class (short, medium, long) | | yes | Kala, JHora | P0 |
| Maraka houses and lords (2nd, 7th and their lords, 8th lord, 12th, Saturn as Ayushkaraka) | | yes | all | P0 |
| longevity windows: dasha and antardasha intersections with maraka lords, plus Saturn transit of the sensitive point | dashas, transits | yes | | P0 |
| Balarishta and its cancellations | rules | yes | all | P0 |
| Niryana Shoola dasha and Sthira dasha for longevity timing | dashas | partial | JHora | P1 |
| ethical framing: presentation as vulnerability windows, never a date | interpretation policy | yes (disclaimer) | | P0 |

## Closing checklist

- Keep the disclaimer policy as a property of the result (a flag that the
  interpretation layer must honour), not only as text.
- Confirm the Pindayu and Amsayu harana rules with citations.
