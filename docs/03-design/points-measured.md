# The derived points, measured

Status: `generated` by `cargo xtask points` over the conformance
corpus's `foundation.upagrahas` and `houses.special_lagnas`,
2026-09-07. Do not edit: `check-points` regenerates this page and
fails on any difference. The design written from it is
[`derived-points.md`](derived-points.md).

## 1. What the corpus records

A derived point is a function of things the corpus already holds — the
Sun, the Moon, the lagna, the sunrise and the time since it — and the
corpus holds the **answer** beside them. So nothing here is a comparison
within a tolerance: a proposed formula reproduces a recorded value or it
does not.

71 fixtures carry the eight special lagnas (`avayogi_point_deg`, `ghati_lagna_deg`, `hora_lagna_deg`, `pranapada_lagna_deg`, `sree_lagna_deg`, `varnada_lagna_deg`, `yogi_nakshatra_index`, `yogi_point_deg`) and 71 of them also
carry the seven upagrahas. The project's own research page marks
every one of these formulas **verify**
(`01-research/feature-universe/01-vedic-parashari-core.md` §E);
this pass is that verification.

## 2. The five upagrahas the Sun casts are one chain

Each is defined from the one before it, and the chain begins at the Sun:
Dhuma is 133°20′ ahead of it, Vyatipata is Dhuma reflected about the
start of the zodiac, Parivesha is Vyatipata opposed, Indrachapa is
Parivesha reflected, and Upaketu is 16°40′ past Indrachapa.

| proposed rule | verdict | measured |
|---|---|---|
| `DHUMA` is where the chain puts it | **holds** | worst 0.000000″ over 71 |
| `INDRACHAPA` is where the chain puts it | **holds** | worst 0.000000″ over 71 |
| `PARIVESH` is where the chain puts it | **holds** | worst 0.000000″ over 71 |
| `UPAKETU` is where the chain puts it | **holds** | worst 0.000000″ over 71 |
| `VYATIPATA` is where the chain puts it | **holds** | worst 0.000000″ over 71 |

Every one is exact — not near, **exact**, to the last bit of a double,
on every fixture that records it. The research page's "verify" is
verified, and the chain is worth keeping as a chain rather than five
offsets from the Sun, because two of its steps are reflections and a
reflection does not compose into an offset.

## 3. The Yogi and the Avayogi

The Yogi point is the two luminaries added together and advanced by
93°20′ — seven nakshatras — and the Avayogi is 186°40′ past it,
which is fourteen more.

| proposed rule | verdict | measured |
|---|---|---|
| the Yogi point is the Sun and the Moon together, and 93°20′ on | **holds** | worst 0.000000″ over 71 |
| the Avayogi is 186°40′ past the Yogi | **holds** | worst 0.000000″ over 71 |
| the recorded nakshatra index is the Yogi point's own | **holds** | 0 of 71 disagree |

Both exact, and the nakshatra the engine records beside the Yogi is the
one its own longitude falls in, which settles that the recorded index is
a rendering of the point and not a separate reading.

## 4. The Sree lagna is the Moon's nakshatra fraction on the lagna

The Moon stands some fraction of the way through its nakshatra. The Sree
lagna is the lagna advanced by that fraction — of what, is the
question, and the corpus answers it over 71 fixtures.

| proposed rule | verdict | measured |
|---|---|---|
| the Sree lagna is the lagna, advanced by the fraction of a **circle** | **holds** | worst 0.0000° |
| the Sree lagna is the lagna, advanced by the fraction of a **sign** | falsified | worst 176.2918° |
| the Sree lagna is the start of the lagna's sign, by the fraction of a circle | falsified | worst 29.8410° |
| the Sree lagna is the start of the lagna's sign, by the fraction of a sign | falsified | worst 177.4227° |

Of a **circle**, exactly. That is a strong result for a small formula:
the three readings that are wrong are wrong by tens of degrees, so
nothing here is a matter of taste.

## 5. Three lagnas the clock drives, and one time under them

The hora, ghati and pranapada lagnas are one rule at three speeds: start
at the Sun **at birth**, advance at the rule's rate for the time since
sunrise. The pranapada adds one thing more — nothing if the Sun stands
in a movable sign, 240° in a fixed one and 120° in a dual one — and
that is the whole difference between them.

| proposed rule | verdict | measured |
|---|---|---|
| the hora lagna advances 30° an hour from the Sun over the recorded ishtakaal | falsified | 25 of 71 disagree; worst 0.8163° |
| the pranapada lagna advances 60° an hour from the Sun over the recorded ishtakaal | falsified | 25 of 71 disagree; worst 1.6326° |
| the ghati lagna advances 75° an hour from the Sun over the recorded ishtakaal | falsified | 25 of 71 disagree; worst 2.0408° |
| the three are built on one elapsed time, not three | **holds** | 0 of 71 disagree |
| and over that time every one of them is exact | **holds** | 0 of 213 disagree |

Each is exact on most fixtures and out on the rest by an amount
proportional to its own rate — 0.8163° at 30° an hour, 1.6326° at
60°, 2.0408° at 75°. **That proportionality is the finding.** A
wrong rule would be wrong by its own kind of amount; three rules
wrong in proportion to their rates are three right rules reading
one wrong clock. The last two claims close it: on every fixture the
three imply the same elapsed time as each other to under a
hundredth of a minute, and against that time each is exact.

So the disagreement is one quantity and not three. The elapsed time the
engine's points are built on differs from the ishtakaal the same fixture
records by at most **1.633 minutes**, on 25 of 71 fixtures; on the rest
it agrees exactly.

| fixture | the engine's elapsed time, less the ishtakaal it records |
|---|---|
| c035-new-york-2021-03-14 | +1.633 min |
| c024-london-1947-05-18 | +1.433 min |
| c036-new-york-2021-11-07 | -1.182 min |
| c037-new-york-2021-11-07 | -1.182 min |
| c005-kathmandu-1910-03-21 | +1.128 min |
| c055-kathmandu-2024-04-09 | +1.074 min |
| c001-kathmandu-1990-04-14 | +1.044 min |
| c001-kathmandu-1990-04-14--fagan-bradley | +1.044 min |

The SDK uses the ishtakaal it computes, which is the one it can explain.
A harness comparing these three points against the corpus allows for the
bracket above, and the deliberate-difference registry carries it.

## 6. Gulika begins Saturn's eighth and Mandi ends it

The arc a birth falls in — the daylight, or the night that
follows it — is cut into eight, and the catalogue already carries
which of those eighths is Saturn's on each day of the week: the
`GULIKA_KAALA` row, measured against the recorded panchanga on all
55 days (`panchanga-day-conventions.md`). For a night birth the
sequence begins five weekdays on, which is the walk the choghadiya
already take.

What the research page could not settle is **where inside that eighth**
each point is read, and whether Gulika and Mandi are two names for one
thing or two readings of it
(`01-research/feature-universe/01-vedic-parashari-core.md` §E, marked
"verify"). This pass proposed no answer: it tried all twenty-four
candidate instants against each recorded value and read the rule off the
result.

**They are two readings of one portion, and it is start against end
rather than start against middle.** Gulika is the ascendant where
Saturn's eighth begins and Mandi where it ends, on every fixture, to the
precision of the ascendant itself — which the first claim measures and
the other four inherit. The ascendant is the SDK's own
`astro::houses::houses_at` over the fixture's instant, place and
recorded ayanamsha, so nothing here needs an ephemeris.

| proposed rule | verdict | measured |
|---|---|---|
| this pass computes the same ascendant the fixture records | **holds** | worst 0.003344° |
| `GULIKA` is the ascendant somewhere inside an eighth of its arc | **holds** | 0 of 71 disagree |
| `MANDI` is the ascendant somewhere inside an eighth of its arc | **holds** | 0 of 71 disagree |
| `GULIKA` is the ascendant at the start of Saturn's eighth | **holds** | 0 of 71 disagree; worst 0.0030° |
| `MANDI` is the ascendant at the end of Saturn's eighth | **holds** | 0 of 71 disagree; worst 0.0033° |

71 fixtures carry both points and an arc to divide. The tables
below are the derivation the rule was read off, one row per vara
and half of the day; an eighth's end is the next one's beginning,
and the pass names the earlier of the two.

**GULIKA**

| vara | half | where the corpus puts it |
|---|---|---|
| Sunday | night | the 2 eighth, at its end (10) |
| Sunday | day | the 6 eighth, at its end (5) |
| Monday | night | the 1 eighth, at its end (3) |
| Monday | day | the 5 eighth, at its end (1) |
| Tuesday | night | the 1 eighth, at its start (4) |
| Tuesday | day | the 4 eighth, at its end (9) |
| Wednesday | night | the 6 eighth, at its end (4) |
| Thursday | night | the 5 eighth, at its end (3) |
| Thursday | day | the 2 eighth, at its end (6) |
| Friday | night | the 4 eighth, at its end (13) |
| Friday | day | the 1 eighth, at its end (1) |
| Saturday | night | the 3 eighth, at its end (9) |
| Saturday | day | the 1 eighth, at its start (3) |

**MANDI**

| vara | half | where the corpus puts it |
|---|---|---|
| Sunday | night | the 3 eighth, at its end (10) |
| Sunday | day | the 7 eighth, at its end (5) |
| Monday | night | the 2 eighth, at its end (3) |
| Monday | day | the 6 eighth, at its end (1) |
| Tuesday | night | the 1 eighth, at its end (4) |
| Tuesday | day | the 5 eighth, at its end (9) |
| Wednesday | night | the 7 eighth, at its end (4) |
| Thursday | night | the 6 eighth, at its end (3) |
| Thursday | day | the 3 eighth, at its end (6) |
| Friday | night | the 5 eighth, at its end (13) |
| Friday | day | the 2 eighth, at its end (1) |
| Saturday | night | the 4 eighth, at its end (9) |
| Saturday | day | the 1 eighth, at its end (3) |

## 7. The Varnada, which this pass refuses

The Varnada is counted from the lagna and the hora lagna, forward from
Aries when a sign is odd and backward from Pisces when it is even, the
two counts added when the signs share a parity and subtracted when they
do not, and the total counted off again from whichever end the lagna's
parity chooses.

| proposed rule | verdict | measured |
|---|---|---|
| the recorded Varnada is a whole sign and not a longitude | **holds** | 0 of 71 disagree |
| the received rule over the lagna and the hora lagna reproduces it | falsified | 31 of 71 disagree |

Every recorded value is a **sign** rather than a longitude, which is a
fact about the field worth having. The rule is not: it is wrong on 31 of
71. Nor is any of the readings this pass constructed from the same parts
— the parity taken the other way about, the counts subtracted rather
than added, the final count anchored on the hora lagna instead of the
lagna — the best of them wrong on 31.

That is crux **C22** met in the data: five published schools disagree
over the Varnada and this engine follows one of them. Naming which would
take the school's own text, so `crates/points` **ships no Varnada** and
the crux stays open. A guess here would be a wrong sign in a chart,
silently.

## 8. What this pass decides

- **The five solar upagrahas ship as a chain**, exact on every
fixture that records them. The research page's "verify" is
verified.
- **The Yogi and the Avayogi ship**, exact, and the nakshatra
beside the Yogi is a rendering of it rather than a second
reading.
- **The Sree lagna is the fraction of a circle**, not of a sign.
The wrong readings are wrong by tens of degrees.
- **The hora, ghati and pranapada lagnas are one rule at three
rates**, from the Sun **at birth** and not at sunrise, with the
pranapada's modality shift. They are built on one elapsed time,
which differs from the recorded ishtakaal by at most 1.633
minutes — a registry entry, not a rule.
- **Gulika begins Saturn's eighth and Mandi ends it.** Two
readings of one portion, not two names for one point and not
start against middle, derived from the corpus rather than
proposed. The eighth's index the catalogue already carries, and
a night birth walks it five weekdays on.
- **The Varnada does not ship.** No reading of the received rule
reproduces the engine's, which is crux C22 met in the data.
