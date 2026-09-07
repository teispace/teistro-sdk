# `teistro-chart`

Status: `built` (the day and the bhavas), 2026-09-07. The design is
[`docs/03-design/chart-foundation.md`](../../docs/03-design/chart-foundation.md);
the measurement behind the bhavas is
[`docs/03-design/chart-bhava-chalit.md`](../../docs/03-design/chart-bhava-chalit.md).

What every module above a chart starts from. Two of its parts are built,
and they are the two the design named as easy to get wrong.

| module | what it settles |
|---|---|
| [`day`](src/day.rs) | the day an instant belongs to, which is **not** its civil date: a panchanga day runs sunrise to sunrise, so an instant before the civil date's sunrise belongs to the day that began the morning before, and the vara, the hora, the ishtakaal and the lagna's anchor move back with it |
| [`bhava`](src/bhava.rs) | the twelve bhavas of a chart with their madhya as well as their sandhi, and where a graha falls between them — carrying the chalit that placed it, because the methods disagree |

## Why a placement carries its method

Over the corpus's 55 charts the four named bhava chalit methods put a
graha in a different bhava between 10% and 51% of the time. Sripati and
Porphyry are built on the *same cusps* and disagree half the time,
because one reads a cusp as a house's middle and the other as its edge.
A bhava number without its method is not a reproducible fact.

`astro::houses` returns cusps; this crate turns them into bhavas, and it
is the only place that does. Everything above asks for a `Placement`
rather than comparing longitudes to cusps itself.

## Against the corpus

`tests/baseline_bhavas.rs` is the first thing to read the corpus's
`houses.bhava_chalit` section, recorded in spike 1 and with nothing to
compare against until now:

| what | result |
|---|---|
| the madhya and sandhi of all 55 charts | 1320 compared, worst 1.7e-13° |
| the arc, the lagna's anchor and day-or-night, over 50 comparable charts | all of them, 20 of them births before sunrise |
| the ishtakaal under both reckonings | 100 readings, worst 3 pala — the engine's night is `24h − daylight` (entry 15) |
| bhayat and bhabhoga against the Moon's nakshatra transit | 110 readings, worst 0.39 minutes |
| every graha's bhava | 495 placements, all of them |
| the engine's own list of grahas the chalit moves | 107, right in both the chalit and the whole-sign reading |
| Sripati against Vehlow | 21.8%, which is what `cargo xtask chalit` measures independently from the SDK's own cusps |

The comparison is against **Vehlow**, because that is what the recording
engine computes whatever its label says — entry 14 of the
deliberate-difference registry. The test asserts that difference rather
than avoiding it.

## Two things the corpus corrected

**Bhayat and bhabhoga are not the day's part.** The design page said they
were the elapsed and total of the part the birth falls in. They are the
duration of the *Moon's traversal of its nakshatra* and the elapsed part
of it — which the corpus settles to within 0.39 minutes over all 55
charts. They belong to `dasha`, and the page is corrected.

**The engine's night is `24h − daylight`**, not sunset to the next
sunrise: true of all 110 nights the corpus records, and up to 1.80
minutes from the real interval. It reaches the proportional ishtakaal,
which spreads thirty ghatis over the night; the SDK divides the night it
actually has and is up to 3 palas from the engine. Entry 15 of the
deliberate-difference registry.

## Still to come

The foundation itself: the positions in both frames, the lagna and its
anchor, the birth timing, and the provenance that stamps them. The design
page settles all of it; `day` and `bhava` are what it is built from.
