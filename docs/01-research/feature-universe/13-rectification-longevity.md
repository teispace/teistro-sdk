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

### What BPHS ch. 43 settles, and the order of work (2026-09-16)

The chapter was read whole. It has four parts, and they are two kinds of
thing:

1. **Rules with a class of life** (vv. 51 to 78): the seven classes and their
   spans, and some twenty combinations. **Built** as `Outcome::LifeClass` and
   21 rules (crux C102).
2. **The three pairs** (vv. 33 to 50): the lagna and eighth lords, Saturn and
   the Moon, the lagna and the hora lagna, each pair's modalities giving long,
   medium or short life; the majority, or the lagna pair, or Saturn and the
   Moon where the Moon is in the lagna or seventh; 120, 108, 96 years and the
   rest by how many pairs agree; the degrees of the contributors rectifying
   it; Saturn lowering the class and Jupiter raising it. A computation, not a
   rule. The translator works an example (born 21 May 1944, 19:01:15 IWT,
   13°40′ N 79°20′ E): short life from two pairs, 36 years, rectified to
   31.18. **Next.**
3. **Pindayu, Nisargayu and Amsayu** (vv. 4 to 22) with their reductions —
   half for combustion (not Venus or Saturn), a third in an enemy's sign (not
   retrograde), the visible-half losses from full to a sixth, the malefic
   rising by the lagna's degrees, only the largest reduction applying, and
   only the strongest of several in one house losing — and the lagna's own
   contribution by rasi or navamsha as its lord or the navamsha lord is
   stronger; chosen by whichever of the lagna, the Sun and the Moon is
   strongest, averaged on a tie (vv. 30 to 32). The same example gives every
   graha's basic years (the Sun 17.5642, the Moon 24.6247 …) and a Pindayu of
   82.2502. The translator's graded visible-half loss is his own refinement
   of the verse's flat fractions and is a reading, not the verse. **After the
   three pairs.**
4. **Longevity for other beings** (vv. 23 to 29): a scale factor by species.
   Out of scope.

## Closing checklist

- Keep the disclaimer policy as a property of the result (a flag that the
  interpretation layer must honour), not only as text.
- Confirm the Pindayu and Amsayu harana rules with citations.
