# Jaimini's significators, measured

Status: `generated` by `cargo xtask jaimini` over the recorded births.
Do not edit: `check-jaimini` regenerates this page and fails on any
difference.

The design this measures is `jaimini-significators.md`. The corpus
records the chara karakas and neither the karakamsha nor the Brahma
graha, so the one input it records is held, and the rest counted.

## 1. The Atmakaraka the karakamsha is read from

Every recorded birth (55 of them), founded under `conformance-baseline`
and asked for its significators through `sdk.chart().jaimini`, against
the Atmakaraka the recording ranks: 54 of 55 agree under seven karakas
and 54 of 55 under eight. Where they part, the recorded Atmakaraka
stands on a sign's edge, where the two ephemerides put it on either
side:

- `c051-kathmandu-2020-04-13` (system7): the recording ranks SUN at 359.999995° and the SDK MARS with SUN at 0.000009°, 0.02″ and 0.03″ from a sign's edge
- `c051-kathmandu-2020-04-13` (system8): the recording ranks SUN at 359.999995° and the SDK RAHU with SUN at 0.000009°, 0.02″ and 0.03″ from a sign's edge

## 2. The karakamsha's houses, in two charts (C130)

A graha's house counted from the karakamsha in the rasi chart and
in the navamsha: 446 of 495 differ, and on
25 of 55 births every graha's does. The schools
that read one chart and those that read the other answer
differently almost always, which is why both are reported.

## 3. The Brahma graha, under each rule

Over the 55 births, each rule under each reading of the dual lords:

| rule | Scorpio's and Aquarius's lords | found | none, and why | Saturn or a node passed it on |
|---|---|---:|---|---:|
| `VERSES` | `NONE` | 23 | 29 no lord qualifies, 3 no planet in the 6th | 4 |
| `VERSES` | `STRONGER_LORD` | 22 | 29 no lord qualifies, 4 no planet in the 6th | 6 |
| `VERSES` | `BOTH` | 24 | 24 no lord qualifies, 7 no planet in the 6th | 10 |
| `TRANSLATORS_NOTE` | `NONE` | 48 | 7 no planet qualifies | 0 |
| `TRANSLATORS_NOTE` | `STRONGER_LORD` | 48 | 7 no planet qualifies | 0 |
| `TRANSLATORS_NOTE` | `BOTH` | 48 | 7 no planet qualifies | 0 |

## 4. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| the SDK ranks the recording's Atmakaraka under seven karakas | falsified | 1 of 55 disagree |
| the SDK ranks the recording's Atmakaraka under eight karakas | falsified | 1 of 55 disagree |
| every Atmakaraka the SDK and the recording part on stands within an arc-second of a sign's edge | **holds** | 0 of 2 disagree |
| the karakamsha's houses are the same in the rasi chart and the navamsha | falsified | 446 of 495 disagree |
| the verses find a Brahma for every birth (C128) | falsified | 32 of 55 disagree |
| the translator's note finds a Brahma for every birth | falsified | 7 of 55 disagree |

Five of the six claims are falsified. A chart the verses find no Brahma
for has no Sthira dasa under them, and says so by name; the note's rule
is the knob that supplies one (`jaimini.brahma`).

