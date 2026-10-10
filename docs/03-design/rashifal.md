# Rashifal: a period read for each sign (the `rashifal` module)

Status: `built`, 2026-10-08; every step of the order of work built.
Written from the text and from the baseline engine's code before any of
this module's code; the building is expected to correct it.

A rashifal reads one period of the sky for each of the twelve signs,
each taken as a reader's janma rāśi: the gochar from that sign, Saturn's
standing from it, and what changes during the period. It needs no natal
chart, so one snapshot serves all twelve.

Phase 8 lists it fourth (`07-roadmap/00-roadmap.md`, Track B): the
feature universe marks it P0 for consumer apps
(`01-research/feature-universe/11-transits-gochar.md`), and the baseline
engine serves it daily, weekly, monthly and yearly.

## What the text decides

Phaladeepika ch. 26, already read for `gochar.md` §1 and built in
`teistro-gochar`, is the whole of the doctrine:

- **v. 1:** count from the Moon's sign. A rashifal's sign *is* that
  sign, so `Reference::moon(rashi)` for each of the twelve.
- **v. 2:** the good houses for each graha.
- **vv. 3–8:** each good house's vedha house, and the two mutual
  exemptions (the Sun and Saturn, the Moon and Mercury).
- **v. 25:** which decanate bears a transit's fruit.

The gochar settings group's knobs (C136, C137, C140) apply unchanged.
The text gives **no score**: it says which transits are good, which are
obstructed and which are not good. A number from 0 to 100 is the
baseline's and nothing else's.

Saturn's 12th, 1st and 2nd from the Moon are Sade Sati's three phases,
and the 4th and the 8th are its smaller spells (Kantaka and Ashtama
Shani, C149). `teistro_gochar::sade_sati` already holds both, with its
`Phase` and `DEFAULT_SPELLS`, so a rashifal reads Saturn's house through
them rather than through a table of its own.

## What the baseline does

The baseline engine's rashifal package was read in full on 2026-10-08,
its constants, its scoring and the orchestration that calls it. It
does this:

- **The instant.** It is 06:00 local clock time on the reference day,
  though every comment says sunrise.
- **The reference day.** It is the period's middle day, first day plus
  ⌊(n − 1)/2⌋. For a week (Sunday to Saturday) that is the Wednesday;
  for a 31-day month the 16th; for a common year July 2.
- **The place.** It is pinned to Kathmandu for every reader worldwide.
  The positions are geocentric with the mean node, and the zodiac is the
  reader's.
- **The tables.** All nine good-house tables match v. 2, and so do the
  Sun's, the Moon's, Mars's, Saturn's, Mercury's and Jupiter's vedha.
  Three tables do not:
  - **Venus** reads 11→6 and 12→3, two houses exchanged against v. 8's
    11→3 and 12→6.
  - **The nodes** take Saturn's 3→12 and 6→9 with the Sun's 10→4 and
    11→5, where "like the Sun" gives 3→9 and 6→12.
  - **The exemptions** cover only the Sun and the Moon, which the text
    never exempts, and omit both pairs it does.
- **The events.** It scans for ingresses and stations in fixed steps,
  and that scan misses some:
  - It never finds a node's ingress.
  - A weekly reading never finds Saturn's, because the 7-day step is
    longer than the 6-day window.
  - Every scan skips its last partial step.
  - Its dates are UTC days.
- **The score.** Each graha is weighted Jupiter 20, Saturn 20, Rahu 12,
  Ketu 10, the Sun 8, Mars 8, Venus 7, Mercury 6 and the Moon 5. The
  Moon's weight is 2 for a week, 1 for a month and 0 for a year. A
  transit then counts:
  - when good, its weight, times 0.75 when retrograde;
  - when obstructed, 0.15 of its weight;
  - when not good, −0.6 of its weight, times 1.25 when retrograde.

  Saturn's standing adds −8, −15 or −5 for Sade Sati and −6 for the 4th
  or the 8th. The reference day's panchanga adds a tithi group's score,
  a yoga nature's score and 2 for each active muhurta yoga, though the
  baseline reads that panchanga on the wrong day. Each event adds +0.5
  of the graha's weight for an ingress into a good house, −0.3 for any
  other house, and −0.15 for a station. The sum is mapped linearly from
  [−0.6·Σw − 25, Σw + 10] onto 0–100 and clamped.
- **Life areas.** Eight are scored, each from a fixed house map, and the
  overall area counts the 1st and 4th houses twice.
- **Key influences.** Five grahas with weight 8 or more are named, and
  the stable sort always drops Mars.
- **Lucky elements.** A colour, number, direction and day are looked up
  by the sign's lord.

No baseline test pins a table or a score.

## The design

A new crate, `teistro-rashifal`, depending on `teistro-core` and
`teistro-gochar`. It is pure: it reads no clock and no ephemeris.

- **`RashiReading`** is one sign's reading of one snapshot:
  - the sign;
  - its `GocharReading` (v. 2's houses, vv. 3–8's obstructions and
    v. 25's fruition, all from `gochar`);
  - Saturn's house from the sign, with the Sade Sati phase when it is
    the 12th, 1st or 2nd, and whether it is one of the smaller spells
    the rules count (C149's 4th and 8th by default);
  - each event of the period, with its house from the sign and whether
    v. 2 calls that house good. An event is a fact about the period, so
    it is not obstructed.
- **`rashifal(snapshot, events, rules) -> [RashiReading; 12]`** builds
  all twelve from one set of nine transits and one event list.
- **`baseline_score(reading, panchanga, period) -> BaselineScore`**
  reproduces the formula above number for number. It returns the
  overall score, the eight areas, the key influences and the lucky
  elements, and is reached only when asked. It is `BASELINE` and
  unsourced (C361).

The façade, `sdk.chart().rashifal(&RashifalRequest)`, takes:

- **`place` and `offset`.** These are required. A reader's own place is
  the default a consumer gives, and Kathmandu is one value of it, not
  the SDK's.
- **`first` and `last`.** The period's first and last civil days,
  inclusive. The SDK does not decide what a week or a month is: a
  consumer passes Sunday to Saturday, or a Bikram Sambat month through
  `sdk.calendar`.
- **`snapshot`.** Sunrise at the place on the reference day by default
  (C358). `Snapshot::Clock(h, m)` is the baseline's 06:00.
- **`events`.** The grahas whose ingresses and stations are found. All
  but the Moon by default (C360).
- **`spells`.** Saturn's smaller spells, C149's 4th and 8th by default.
  The gochar group's knobs come from the context's settings, as
  `sdk.chart().gochar` reads them.

It founds the nine longitudes once at the snapshot, through
`Founder::longitudes`. It finds each ingress and station exactly with
the hits machinery `sdk.chart().hits` already uses, and finds them from local
midnight of the first day to local midnight after the last, so an
ingress on the last evening is the period's. A batch form,
`sdk.chart().rashifal_many`, takes many periods under one founder.

## The forks (cruxes)

| crux | question | default | why |
|---|---|---|---|
| C358 | the instant a period is read at | sunrise at the place on the reference day | the roadmap's "the snapshot at sunrise, not 06:00", and the baseline's own comments; its 06:00 clock is `Snapshot::Clock(6, 0)` |
| C359 | a long period's reference day | the middle day, first + ⌊(n − 1)/2⌋ | it is what the baseline's midpoint by date comes to for every length, and no text reads a period at another day; a caller wanting the whole span reads its days, the batch form being the cheap way |
| C360 | which grahas' events a period reports | every graha but the Moon, the nodes included | the Moon changes sign about every 2.5 days, so a week would be a list of its ingresses; it is one knob away. The nodes' ingresses are the slowest events after Saturn's and Jupiter's, and the baseline's omission is a defect |
| C361 | the baseline's score | `baseline_score`, `BASELINE`, over the SDK's own reading | no text scores a rashifal. The score is reproduced formula for formula (its double count and its five key influences included, since a consumer of its numbers sees them), but over the text's tables, exact events and the reference day's panchanga. The measured page counts each difference that moves a verdict or a score: the three tables, the missed events, 06:00 against sunrise and the panchanga's day |

## Tests

- Each of the twelve readings agrees with `gochar(Reference::moon(r), …)`
  called alone, and Saturn's house with the Sade Sati kernel's.
- An event's house from each sign is counted from the event's sign. The
  twelve houses of one event are its sign counted from each.
- The reference day for a day, a 7-day week, months of 28 to 31 days and
  years of 365 and 366: the values in "What the baseline does".
- `baseline_score` held to hand-computed cases: an all-good sky, an
  all-bad one with Sade Sati at its peak, and a node retrograde (the
  mean node always is). Also the double-counted overall area, and the
  key influences that drop Mars.
- Through the façade:
  - the snapshot is the day's sunrise to the second;
  - a node's ingress inside a week is found;
  - a Saturn ingress on a period's last day is found, where the baseline
    misses both;
  - the batch form agrees with one call per period.

## Order of work

1. The kernel: the crate, `rashifal` and `baseline_score`, with the
   tests above: **done**. `teistro_gochar::good_house` is new, for an
   event's house. `RashifalRules::default()` is the text read whole
   (`GocharRules::TEXT`) with C149's spells; a façade reads the gochar
   group from its settings.
2. The façade: the request, sunrise, the events, the batch form:
   **done**. A station is counted in the sign the graha stands in; the
   panchanga the baseline score reads is the reference day's at
   sunrise, whatever the snapshot.
3. The measured page, `rashifal-measured.md`, under `check-rashifal`.
   It reads a year of days at Kathmandu and counts each of C361's
   differences: **done**. `baseline::baseline_gochar` re-judges a reading
   by the baseline engine's own vedha tables (D1 to D3), so its score can
   be read over its own verdicts as well as the text's. A year from July
   2026 holds the nodes' ingress and Saturn's; two years cost twice the
   almanac's days for no row the year lacks.
4. Every binding: a call of its own, with the parity runners: **done**.
   A rashifal is read for periods, not for a chart, so it crosses as its
   own boundary call, `ts_rashifal`, rather than as a chart section: a
   batch of periods in, each period's answer out as canonical JSON, the
   baseline's scores beside the reading when a baseline period is asked.
   A refusal names the period, `rashifal.periods[i]`, and the field in
   it.
