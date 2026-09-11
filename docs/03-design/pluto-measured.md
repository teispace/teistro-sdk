# Pluto, measured

Status: `generated` by `cargo xtask pluto`, gated by `check-pluto`. Do not edit. The numbers are read from `crates/ephemeris-builtin/data/pluto-fit.json`, recorded by `teistro-ephemeris-teimeris-pluto-fit` against the reference engine — the number, not the code that produced it.

PLUTO is the one body in the built-in ephemeris with **no published series to truncate** (ADR-0008). Every other body's tier is a threshold on an existing theory; Pluto's is a choice of how finely to fit it. This page measures that choice.

What is fitted is the **heliocentric** vector, in `HELIOCENTRIC/J2000/ECLIPTIC/RECTANGULAR, fitted; the error is the geocentric angle`. Pluto's motion about the Sun is slow and smooth — one circuit in 248 years — and a Chebyshev series is cheap on a smooth function. Its *geocentric* motion is that same slow curve with the Earth's whole annual orbit superimposed, and fitting that would pay for the Earth again at every one of Pluto's blocks, when the Earth already has a theory.

> a fit is exact at its own nodes, so every probe is off them; the error is the angle a chart would show, rebuilt from the fitted heliocentric vector and the engine's own Earth

## The ladder

Every combination of interval length and degree, fitted over the whole span and then probed **off** its own nodes — a Chebyshev fit is exact at its nodes, so measuring there would report zero however bad the fit is.

| interval | terms | blocks | bytes | worst | mean |
|---:|---:|---:|---:|---:|---:|
| 40 y | 6 | 15 | 2.1 KB | 81.7″ | 17.7″ |
| 40 y | 8 | 15 | 2.8 KB | 64.7″ | 18.1″ |
| 40 y | 10 | 15 | 3.5 KB | 32.1″ | 5.99″ |
| 40 y | 12 | 15 | 4.2 KB | 10.3″ | 3.61″ |
| 40 y | 14 | 15 | 4.9 KB | 3.31″ | 0.922″ |
| 40 y | 16 | 15 | 5.6 KB | 2.11″ | 0.441″ |
| 20 y | 6 | 30 | 4.2 KB | 13.6″ | 4.41″ |
| 20 y | 8 | 30 | 5.6 KB | 2.96″ | 0.796″ |
| 20 y | 10 | 30 | 7.0 KB | 0.867″ | 0.149″ |
| 20 y | 12 | 30 | 8.4 KB | 0.225″ | 0.091″ |
| 20 y | 14 | 30 | 9.8 KB | 0.102″ | 0.029″ |
| 20 y | 16 | 30 | 11.2 KB | 0.063″ | 0.015″ |
| 10 y | 6 | 60 | 8.4 KB | 0.752″ | 0.152″ |
| 10 y | 8 | 60 | 11.2 KB | 0.090″ | 0.023″ |
| 10 y | 10 | 60 | 14.1 KB | 0.059″ | 0.0095″ |
| 10 y | 12 | 60 | 16.9 KB | 0.052″ | 0.018″ |
| 10 y | 14 | 60 | 19.7 KB | 0.055″ | 0.018″ |
| 10 y | 16 | 60 | 22.5 KB | 0.054″ | 0.013″ |
| 5 y | 6 | 120 | 16.9 KB | 0.059″ | 0.014″ |
| 5 y | 8 | 120 | 22.5 KB | 0.061″ | 0.014″ |
| 5 y | 10 | 120 | 28.1 KB | 0.063″ | 0.0087″ |
| 5 y | 12 | 120 | 33.8 KB | 0.068″ | 0.015″ |
| 5 y | 14 | 120 | 39.4 KB | 0.053″ | 0.013″ |
| 5 y | 16 | 120 | 45.0 KB | 0.034″ | 0.0069″ |
| 2 y | 6 | 300 | 42.2 KB | 0.046″ | 0.0074″ |
| 2 y | 8 | 300 | 56.2 KB | 0.026″ | 0.0059″ |
| 2 y | 10 | 300 | 70.3 KB | 0.011″ | 0.0019″ |
| 2 y | 12 | 300 | 84.4 KB | 0.0061″ | 0.0011″ |
| 2 y | 14 | 300 | 98.4 KB | 0.0061″ | 0.0009″ |
| 2 y | 16 | 300 | 112 KB | 0.0061″ | 0.0009″ |
| 1 y | 6 | 600 | 84.4 KB | 0.0061″ | 0.0013″ |
| 1 y | 8 | 600 | 112 KB | 0.0061″ | 0.0009″ |
| 1 y | 10 | 600 | 141 KB | 0.0061″ | 0.0009″ |
| 1 y | 12 | 600 | 169 KB | 0.0061″ | 0.0009″ |
| 1 y | 14 | 600 | 197 KB | 0.0061″ | 0.0009″ |
| 1 y | 16 | 600 | 225 KB | 0.0061″ | 0.0009″ |

Arcseconds of **geocentric angle**, which is what a chart shows: each probe rebuilds the geocentric vector from the fitted heliocentric one and the engine's own Earth, so the figure is what a consumer would see rather than what the fitter would like to report.

## What each tier takes

**The cheapest rung meeting the tier's own bound**, and not the best rung that fits the budget. Pluto is small enough at every tier that a richer rung would always be affordable, which is exactly why the rule has to be written down: a tier that quietly spent a consumer's bytes on the strength of nothing stated is how a budget stops meaning anything.

| tier | bound | interval | terms | blocks | bytes | worst |
|---|---:|---:|---:|---:|---:|---:|
| `compact` | 60.0″ | 40 y | 10 | 15 | 3.5 KB | 32.1″ |
| `standard` | 1.00″ | 20 y | 10 | 30 | 7.0 KB | 0.867″ |
| `full` | 0.0092″ | 2 y | 12 | 300 | 84.4 KB | 0.0061″ |

`compact` and `standard` carry the bounds their own manifest states — an arcminute and an arcsecond. `full` cannot say "the theory as published", because for a fitted body there is no published theory, so its bound is the floor the ladder itself reaches.

## The floor, and what it is not

The ladder stops improving at **0.0061″**. nine of its 36 rungs reach that figure and none beats it, the cheapest at 84.4 KB: past that point a finer fit buys nothing at all.

**What the floor is has not been established here, and guessing would be worse than saying so.** A representation error falls without limit as the fit is refined, so a floor is something else: the reference's own resolution for this body, or the precision of the round trip through the port's spherical cell, or the probe's own arithmetic. It bounds what this page can claim — no rung can be shown to be better than 0.0061″ by this measurement, whatever it may be in truth — and it is the reason `full`'s bound is stated against the ladder rather than against a number chosen in advance.

## The claims this decides

| proposed rule | verdict | measured |
|---|---|---|
| `compact`: Pluto inside 60.0″, the tier's own bound | **holds** | worst 32.1″ in 3.5 KB |
| `standard`: Pluto inside 1.00″, the tier's own bound | **holds** | worst 0.867″ in 7.0 KB |
| `full`: Pluto inside 0.0092″, the tier's own bound | **holds** | worst 0.0061″ in 84.4 KB |

They hold by construction — the rung was chosen to meet the bound — so the row that would matter is a falsified one, which would mean no rung on the ladder could meet a tier's promise and the promise had to move.

## What this does not measure

**Correctness.** The fit is against the reference engine, so it proves consistency and not truth (ADR-0021). The refit against JPL Horizons or CSPICE before v1 is the one the Moon's bija already owes, and every figure here is republished if it moves.

**Any span but 2378497.0 to 2597641.0.** A Chebyshev table does not degrade outside its fit, it diverges, so the evaluator refuses rather than extrapolates and a wider tier is refitted rather than reused.

**The rate.** The table's derivative is analytic and exact for the series it holds, and is tested as such; what is *not* measured here is the rate against the engine's, which belongs beside every other body's in `03-design/completion-measured.md`.

