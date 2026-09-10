# How accurate the Moon has to be, measured

Status: `generated` by `cargo xtask moon`. Do not edit. The rates are
the SDK's own catalogued extremes (`astro::events::quantity_least_rate`,
the table the search grid is sized from) and the arithmetic over them is
exact; the Sun's error is measured and recorded in
`crates/ephemeris-builtin/data/vsop87-floor.json`.

Phase 3 must choose a lunar theory, and the research page says the tiers
are chosen by Moon accuracy first without saying what accuracy. This is
the number. Panchanga publishes instants, so an error in the Moon's
longitude is not an error in a position — it is an error in a *time*,
and this page converts one into the other.

## What an arcsecond of lunar error costs

A boundary is where a quantity crosses a line of its lattice. An error
of `e` degrees in the quantity moves the crossing by `e / rate` days, so
one arcsecond moves it by `24 / rate` seconds. The rate is the
**quantity's**, which is why the same lunar error is worth more in a
tithi than in a yoga: a tithi runs on the Moon less the Sun and a yoga
on the Moon plus it. The rate used is the slowest the quantity ever
moves, because that is the worst case and the Moon's daily motion swings
by a third between perigee and apogee.

| limb | quantity | member width | slowest rate | one arcsecond is | one second needs | one minute needs |
|---|---|---:|---:|---:|---:|---:|
| Tithi | Moon and Sun | 12.0° | 10.670°/day | 2.25 s | 0.445″ | 26.67″ |
| Karana | Moon and Sun | 6.0° | 10.670°/day | 2.25 s | 0.445″ | 26.67″ |
| Nakshatra | Moon alone | 13.3° | 11.700°/day | 2.05 s | 0.488″ | 29.25″ |
| Yoga | Moon and Sun | 13.3° | 12.650°/day | 1.90 s | 0.527″ | 31.62″ |

## What that asks of a theory

Reading the table the other way: to hold **every** panchanga boundary to
one second of clock time, the Moon's longitude must be right to
**0.445″**, and to hold it to one minute, to **26.67″**. The tithi
is the binding limb, because its quantity is the slowest.

**The Sun is already spent, and it is cheap.** The floor measurement
puts VSOP87's Sun at 0.148″ against the engine, which is 0.33 s of
tithi boundary on its own. That leaves the Moon essentially the whole
budget rather than half of it, and it means a lunar theory good to a
tenth of an arcsecond is not wasted on a Sun good to a seventh.

## The candidates

| theory | fitted to | published accuracy | as tithi boundary | source |
|---|---|---|---:|---|
| ELP/MPP02 | DE405 and DE406 | 2.4 m over a century about J2000; 1.4 km over five millennia | 0.003 s and 1.7 s | **cannot be obtained** |
| ELP2000-82B | DE200 and LE200 | stated as the theory's own; unmeasured here | unmeasured | CDS VI/79 and IMCCE, both reachable |

A metre at the Moon's mean distance subtends 5.37e-4″, which is how
the first row's seconds are reached.

## The finding that blocks the choice

**ELP/MPP02 cannot be obtained.** ADR-0013 and the research page name it
as the SDK's lunar theory. Every route to it was tried on 2026-09-10,
and the result is not a broken link but a removal.

| route | result |
|---|---|
| `cyrano-se.obspm.fr`, the address the theory is published at | no answer over FTP or HTTPS |
| the same path in the Internet Archive | the directory listing is captured, naming `ELP_MAIN.S1/2/3` and `ELP_PERT.S1/2/3`; **not one of the six files was ever captured**, and the directory itself already answered 404 in the August 2022 crawl while its five siblings did not |
| `ftp.imcce.fr/pub/ephem/moon/` | `elp82b` and nothing later |
| CDS catalogue VI/79 | ELP2000-82B as well |
| the two public re-implementations that carry data files | **excluded.** One is GPL-3.0 and the other EUPL-1.2, and `deny.toml` refuses copyleft everywhere in the workspace (ADR-0019). The GPL one's files are transformed besides — fourteen files under names of its own, where the publication has six — so they are that project's derived work and not the published series |

So the choice is between a theory that cannot be had and one that can,
and the second has a question over it.

**ELP2000-82B says of itself: "Constants fitted to JPL's ephemerides
DE200/LE200".** That is the header of `elp82b.f`, the reader its own
authors publish, and not an inference from outside. It is the same fit
whose age the planetary floor measurement caught: VSOP87 was fitted to
DE200 in 1981 and drifts against a modern ephemeris by 4.7 arcseconds at
Uranus and 6.6 at Neptune, while holding 0.148″ at the Sun.

Whether the Moon inherits that drift is **not known and must not be
assumed either way**. A tenth of the planets' worst would still be
inside the second-accurate budget; a half of it would not. The
measurement is the one the planets had — the whole theory against the
engine in the isolated frame — and the harness for it exists.

## What ELP2000-82B actually costs

Measured against the engine in the frame the theory is stated in —
geocentric, ecliptic, J2000, geometric — on the engine's `compatible`
profile. The whole theory is 37 872 terms.

**Truncation is not what limits it.** The ladder converges long before
the terms run out, so the Moon is cheap and the theory is the whole
cost:

| threshold | terms | table | worst |
|---:|---:|---:|---:|
| 1 | 160 | 9 KB | 31.67″ |
| 0.3 | 268 | 14 KB | 23.47″ |
| 0.1 | 416 | 21 KB | 21.34″ |
| 0.03 | 722 | 33 KB | 19.68″ |
| 0.01 | 1139 | 49 KB | 19.29″ |
| 0.001 | 3410 | 123 KB | 19.29″ |
| 0 (whole theory) | 37 872 | 1084 KB | 19.29″ |

**What limits it is distance from its own epoch.** Every row below keeps
every term:

| span | worst | as tithi boundary | radius | verdict |
|---|---:|---:|---:|---|
| 1980 to 2020 | 0.254″ | 0.57 s | 1.3e-7 | **second-accurate** |
| 1900 to 2100 | 1.685″ | 3.79 s | 5.7e-7 | minute-accurate |
| 1850 to 2150 | 3.310″ | 7.45 s | 1.1e-6 | minute-accurate |
| 1800 to 2400 | 19.398″ | 43.63 s | 6.2e-6 | minute-accurate |
| 1700 to 2500 | 29.742″ | 66.90 s | 9.6e-6 | neither |

The first row is the check that the port is faithful rather than the
finding: at the theory's own epoch it agrees with a modern ephemeris to
a quarter of an arcsecond, which is what a correct reading of a
DE200-fitted theory should give. Everything after it is the fit ageing
— and it ages fast, because a lunar theory's mean longitude carries
the tidal acceleration its source ephemeris assumed, and an error there
grows with the square of the time.

## The three ways out, priced

The theory is 19.4″ over the span `standard` claims, and that is 44
seconds of tithi against a budget of one. There are three ways out and
all three are now measured rather than argued.

### Correct the theory's mean longitude

The drift is not noise. A lunar theory ages mainly through the tidal
acceleration its source ephemeris assumed, which enters the mean
longitude as a term in the **square** of the time — so if that is what
the difference is, a polynomial of two or three coefficients removes
most of it. The tradition has the same idea and the same name for it: a
*bija*, a seed correction that re-anchors an old theory to the present
sky, which this SDK already computes for the Surya Siddhanta.

| degree | cost | before | after | as tithi |
|---:|---:|---:|---:|---:|
| 1 | 16 bytes | 19.40″ | 8.48″ | 19.1 s |
| 2 | 24 bytes | 19.40″ | 3.20″ | 7.2 s |
| 3 | 32 bytes | 19.40″ | 3.24″ | 7.3 s |
| 4 | 40 bytes | 19.40″ | 3.23″ | 7.3 s |

**It saturates at the square**, which is the physics rather than a
coincidence: degree three and four buy nothing over degree two, so what
the polynomial is removing is the tidal term and not a curve fitted to
noise. 24 bytes take the Moon from 19.40″ to 3.20″ — 44 seconds of
tithi to 7.2 — over six centuries. What remains is the periodic part
of the difference, which no polynomial can reach.

The best of them is degree 2 at 3.20″.

### Replace the theory with a fitted table

ADR-0021 names a Chebyshev refit from a modern kernel as the `reference`
tier. It has an arithmetic problem the ADR does not price: **an analytic
theory costs the same whatever span it is asked for, and a fitted table
costs one block per interval.** The Moon circles in 27 days where
Jupiter takes twelve years, so the Moon is where that bites.

| interval | coefficients | worst | size over 1800 to 2400 |
|---:|---:|---:|---:|
| 4 days | 8 | 0.0003″ | 10.03 MB |
| 4 days | 10 | under 0.0001″ | 12.54 MB |
| 4 days | 12 | under 0.0001″ | 15.05 MB |
| 4 days | 14 | under 0.0001″ | 17.56 MB |
| 4 days | 16 | under 0.0001″ | 20.06 MB |
| 4 days | 20 | under 0.0001″ | 25.08 MB |
| 8 days | 8 | 0.0735″ | 5.02 MB |
| 8 days | 10 | 0.0020″ | 6.27 MB |
| 8 days | 12 | under 0.0001″ | 7.52 MB |
| 8 days | 14 | under 0.0001″ | 8.78 MB |
| 8 days | 16 | under 0.0001″ | 10.03 MB |
| 8 days | 20 | under 0.0001″ | 12.54 MB |
| 16 days | 8 | 24.2632″ | 2.51 MB |
| 16 days | 10 | 1.4941″ | 3.13 MB |
| 16 days | 12 | 0.1339″ | 3.76 MB |
| 16 days | 14 | 0.0114″ | 4.39 MB |
| 16 days | 16 | 0.0009″ | 5.02 MB |
| 16 days | 20 | under 0.0001″ | 6.27 MB |
| 32 days | 8 | 3323.4305″ | 1.25 MB |
| 32 days | 10 | 614.7278″ | 1.57 MB |
| 32 days | 12 | 122.4968″ | 1.88 MB |
| 32 days | 14 | 32.1237″ | 2.19 MB |
| 32 days | 16 | 8.2514″ | 2.51 MB |
| 32 days | 20 | 0.5152″ | 3.14 MB |

**The cheapest fit that reaches the second-accurate budget is 3.76 MB**
— 16-day blocks of 12 coefficients at 0.134″. ADR-0021 budgets about
1 MB for *every* body at this tier; the Moon alone is several times
that, so it is the ADR's ladder that has to move and not the
measurement.

The error is the fit's **representation** error against the theory it is
fitted to, which is what a table designer chooses. It is deliberately
not accuracy against an ephemeris: how smooth the Moon is, and so how
well a polynomial catches it, is the same question whichever modern
source it comes from.

### Narrow what `standard` claims

The span table above is this route's price list. The theory holds under
four seconds of tithi over 1900 to 2100 and under eight over 1850 to
2150, so a tier that claims three centuries rather than six needs
nothing built at all — it needs the claim to say so.

## What the claims measure to

| proposed rule | verdict | measured |
|---|---|---|
| the Moon decides the tiers, not the planets | **holds** | one arcsecond of Moon is 2.25 s of tithi; one arcsecond of a planet is a position nobody times |
| a second-accurate panchanga is reachable with an analytic theory | **holds** | ELP/MPP02's published 2.4 m is 0.003 s |
| the chosen theory can be obtained | falsified | ELP/MPP02 is published at an address that no longer serves it, was never captured by the archive, and reaches the public only through copyleft re-implementations that ADR-0019 refuses |
| `standard` holds 2 arcseconds for the Moon over 1800 to 2400 | falsified | ELP2000-82B, every term kept, is 19.40″ — 44 s of tithi |
| the Moon's table is a cost worth optimising | falsified | the theory's own accuracy is reached at 49 KB; the remaining 36 733 terms buy nothing |


## What this does not measure

**What a better lunar source would cost.** ELP/MPP02 cannot be had; what
remains is the `reference` tier's refit from a modern kernel, or
`ephemeris-de` reading one directly, and neither is sized here. That is
the next decision, and it is an ADR rather than a build step.

**The rate distribution.** The table uses the slowest the quantity ever
moves, which is the worst case and is what a bound needs. A typical
boundary moves less, and a page that wanted the typical figure would
have to sweep real rates rather than read the catalogued extreme.

**Latitude and distance.** Only longitude moves a panchanga boundary.
The Moon's latitude decides eclipses and its distance decides the
parallax that moves a rising, and both have budgets of their own that
this page does not set.

