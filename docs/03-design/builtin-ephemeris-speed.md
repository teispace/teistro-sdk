# The built-in ephemeris, made fast

Status: `decided`, 2026-10-21. Answers P2-6 of the consumer brief: the
built-in almanac within 3× of Teimeris for one day and a month grid, at
the accuracy its pages already hold.

## What the time went to

A month grid over the built-in took 460 ms on an M2 Pro, against 22 ms
over Teimeris (`05-testing/app-speed-measured.md`). Sampled with
`sample(1)`, 80% of it was `libm`'s `sin`, `cos`, `sincos` and
`rem_pio2` under ELP2000-82B: an almanac asks for the Moon several
hundred times a day, every sunrise, tithi end and nakshatra end being a
root search, and each answer was the whole series again, 1139 terms at the
standard tier, each with a sine for the value and a cosine for the rate.
Once the Moon was fitted, the same profile put 70% of what was left in
`eraEpv00` (the Earth's barycentric state, which the completion's
geometric-to-apparent step asks for at every instant) and most of the
rest in VSOP87A's Earth, which is the Sun.

## What was done

Each of the three is answered from a **fitted segment**: the quantity
evaluated at the nine Chebyshev–Lobatto points of a two-day segment, the
interpolating series through them, and every instant inside the segment
answered from that series (`teistro_core::chebyshev`).

| quantity | where | fitted from | components |
|---|---|---|---:|
| the Moon | `ephemeris-builtin::fitted::moon` | ELP2000-82B's positions | 3 |
| the Earth, heliocentric | `ephemeris-builtin::fitted::earth` | VSOP87A's positions | 3 |
| the Earth's state for aberration | `astro::sky::earth_at_fitted` | `eraEpv00` with the rotation to date | 6 |

- **A fixed grid.** Segments are two days from J2000, so an instant's
  answer depends on the instant alone, never on what was asked before it
  or on what a thread holds. Two days is a power of two, so an instant's
  place in its segment is exact. The content hash of an answer is
  therefore still a function of the request.
- **Continuous at the seams.** The Lobatto points include both ends, so
  neighbouring segments pass through the same value at the instant they
  share.
- **The rate is the fit's derivative.** It is exact for the series
  returned, so position and rate never disagree, and the nodes need only
  positions: for the Moon, half the series' work.
- **A thread's own store.** 256 segments a thread, directly mapped by
  the segment's number, allocated on the first instant asked: no lock,
  nothing shared, and a segment evicted and fitted again is the same bits.
  256 is a year's range resident at once. The first record was taken at
  64, and a year of almanac days, whose lockstep rounds sweep every day of
  the range, refitted every segment each round and took 8.0 s, twice its
  time with no fit. About 57 KB a thread for the Moon and the Earth each,
  113 KB for the Earth's state.
- **Only where a theory is summed.** The fitted Earth state is used by the
  completion's geometric-to-apparent step, which runs only for a provider
  that answers geometric positions; Teimeris answers apparent ones, and
  its answers are unchanged to the bit. The other planets are not fitted:
  a chart asks for each once, where a fit would cost nine evaluations.
- **An opt-out.** `Builtin::with_series()` sums every theory at every
  instant, as before, for a consumer checking the fit or reproducing the
  theories to the last bit. The provider's `data_version` ends in
  `+ fitted` when it fits, so a cached answer and a chart's provenance
  tell the two apart.

## How close

Over the provider's six centuries, 12 000 instants including every
segment's seams, the fit against the theories summed at the instant
(`ephemeris-builtin/tests/fitted.rs`, `astro::sky` tests):

| quantity | position | distance | rate |
|---|---:|---:|---:|
| the Moon | 1.1e-5″ | 6.5e-12 of itself | 3.6e-7 °/day |
| the Sun | 7.1e-7″ | 6.9e-14 | 2.8e-9 °/day |
| Mars | 1.6e-6″ | 4.4e-12 | 6.6e-9 °/day |
| the Earth's state | 1e-11 of itself | — | — |

None of these is the fit's own error. A Julian day near 2.4 million is an
`f64` good to 40 microseconds, in which the Moon moves 2e-5″: the theory
cannot be asked more finely than that, and the same floor appears however
many points a segment is fitted through (degree 6 to 20, segments of one
to sixteen days, every tier measured). The tiers' own truncation of the
Moon is 18″, 0.71″ and 0″, and the theories stand 20″ to 30″ from a
modern ephemeris, so the fit is five orders of magnitude inside every
accuracy figure the built-in publishes.

**A defect found on the way.** The fit's derivative was compared with
the series' analytic rate, and they differed by 4e-5 of the Moon's
velocity. The analytic rate of a perturbation term turned only with
Delaunay's arguments: it left out the figure terms' ζ and the planetary
terms' longitudes, and paired table 1's three Delaunay multipliers with
the wrong arguments. Corrected, the analytic rate is within 1e-8 to 3e-8
of the series' own derivative (a twice-extrapolated central difference),
and the fit's derivative within 6e-9, the difference's own floor. The
Moon's speed was within the provider kit's bound of 0.002 °/day before
and after; it is now right.

## The trigonometry

The SDK's trigonometry is the pure-Rust `libm` so that the same request
gives the same bits on every platform (ADR-0022). Measured on four
million arguments of the size ELP's angles reach: the platform's
`sin_cos` on Apple silicon takes 9.5 ns against `libm`'s 20 ns, and
differs from it by one ulp on 8.4% of arguments. A twofold speed is not
worth a determinism the content hashes rest on, and the fit has already
cut the calls the trigonometry gets by about thirty times. `libm` stays.

## What it bought

Medians on the M2 Pro, one thread, a fresh context each run, from the
gated record (`05-testing/app-speed-measured.md`):

| request | before | after | Teimeris | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|
| one almanac day | 32.3 ms | 2.34 ms | 1.33 ms | 1.8× |
| a month grid | 462 ms | 38.5 ms | 19.9 ms | 1.9× |
| a year of almanac days | 3805 ms | 348 ms | 173 ms | 2.0× |
| a year of festivals | 1645 ms | 125 ms | 55.9 ms | 2.2× |
| a 90-day muhurta search | 3045 ms | 492 ms | 143 ms | 3.5× |
| a birth chart | 2.61 ms | 1.54 ms | 1.04 ms | 1.5× |

The acceptance was 3× for a day and a month grid, and both hold. The
muhurta search is past 3× and has not been profiled since the fit; it is
the next place to look.
