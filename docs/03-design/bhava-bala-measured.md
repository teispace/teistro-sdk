# The Bhava bala, measured

Status: `generated` by `cargo xtask bhava-bala` over the conformance
corpus's `baseline/bhava-bala`, 2026-09-15. Do not edit:
`check-bhava-bala` regenerates this page and fails on any difference.
The design it measures is [`strength-schemes.md`](strength-schemes.md).

The corpus records the recording engine's Bhava bala for 71 charts,
every house's lord's strength, Dig, drishti and total, computed from the
recorded lagna and houses and the engine's Shadbala of the same chart.
This page measures the built module against it:
`teistro_strength::bhava_bala` under the engine's reading, and then each
other reading one fork at a time.

## The engine's reading, reproduced

| proposed rule | verdict | measured |
|---|---|---|
| a house is aspected by every graha's seventh and by Mars's fourth and eighth, Jupiter's fifth and ninth and Saturn's third and tenth, counted in whole-sign houses, the nodes the seventh alone | **holds** | 0 of 852 disagree |
| the lord's strength is the lord's whole Shadbala, within the engine's hundredths | **holds** | 0 of 852 disagree; worst 5.0e-3 |
| Dig is ten virupas a house from the house the sign's class makes weakest, Cancer and Scorpio insects, Sagittarius human and Capricorn quadruped throughout, within the engine's hundredths | **holds** | 0 of 852 disagree; worst 0.0e0 |
| the drishti bala is a quarter of the Dig added when only the Moon, Mercury, Jupiter or Venus aspect the house and taken away when only the others do, with Jupiter's and Mercury's Drik bala added when they aspect it, within the engine's hundredths | **holds** | 0 of 852 disagree; worst 0.0e0 |
| the whole is the three together, with no special rules, within the engine's hundredths | **holds** | 0 of 852 disagree; worst 5.0e-3 |

## The other readings, one fork at a time

Each switches one fork from the engine's reading and counts the houses
whose total moves by more than half a virupa.

| proposed rule | verdict | measured |
|---|---|---|
| Sripati's classes: Scorpio the only insect and Cancer watery, Sagittarius and Capricorn split at their halves (B.V. Raman, Arts. 126–131) | falsified | 208 of 852 disagree; worst gap 60.00 |
| the verses' classes and arc: Cancer and Scorpio insects, the halves split, the Dig a third of the arc between madhyas (vv. 26 to 28) | falsified | 137 of 852 disagree; worst gap 60.00 |
| Sripati's drishti: the sphuta drishti on the bhava madhya, Jupiter's and Mercury's in full and a quarter of each other graha's (Arts. 132–133) | falsified | 846 of 852 disagree; worst gap 193.86 |
| the special rules: a rupa for each Jupiter and Mercury in the house and less one for each Sun, Mars and Saturn, and 15 for a sign rising the way the hour favours (vv. 30 and 31) | falsified | 472 of 852 disagree; worst gap 165.00 |

## What it means for the module

**The engine's reading is settled**, so the conformance profile takes
it, the engine's rounding to hundredths left to the caller.

**Every other fork moves real houses.** The verses as translated are the
default (C73 to C75): the Dig by the madhyas' arc with Cancer an insect
and the two halves split, a quarter of it for a one-sided drishti, and
the special rules, whose twilight the verses leave undefined and the
module takes as two ghatis either side of the night (C75). Sripati's
reading is the other published one, and B.V. Raman's worked Standard
Horoscope holds the module to it house by house
(`crates/strength/tests/sripati.rs`).