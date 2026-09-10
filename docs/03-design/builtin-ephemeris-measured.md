# The built-in ephemeris, measured

Status: `generated` by `cargo xtask vsop` over the published VSOP87A
series. Do not edit. There is no `check-vsop` yet, and the reason is
recorded below rather than left as an omission: the gate arrives with
the truncated tables, which are the artefact that can be regenerated
without the 5.7 MB of source files.

The source is VSOP87A — heliocentric rectangular coordinates in the
ecliptic and equinox of J2000, Bretagnon and Francou (1988), as
published in CDS catalogue VI/81. The whole theory for the eight planets
is **39 198 terms**, which at 24 bytes a term is **919 KB**.

The sweep is 7305 instants, every 30 days from 1800 to 2400, against
every one of 13 amplitude thresholds. Each error is the angle between
the geocentric direction the full theory gives and the one the truncated
theory gives, in arcseconds — the Earth truncated with the body,
because the Earth's series is subtracted from every other and its error
is common to the whole chart.

## Worst geocentric error by threshold

| threshold (AU) | Sun | Mercury | Venus | Mars | Jupiter | Saturn | Uranus | Neptune | terms | bytes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1e-4 | 37.39 | 67.75 | 123 | 195 | 35.32 | 16.18 | 7.90 | 4.17 | 622 | 15 KB |
| 3e-5 | 31.00 | 52.96 | 102 | 136 | 12.22 | 9.23 | 4.16 | 2.14 | 961 | 23 KB |
| 1e-5 | 14.65 | 23.71 | 64.84 | 42.53 | 6.40 | 3.99 | 1.66 | 0.795 | 1562 | 37 KB |
| 3e-6 | 5.48 | 9.13 | 23.25 | 19.56 | 2.13 | 1.28 | 0.709 | 0.275 | 2503 | 59 KB |
| 1e-6 | 2.78 | 4.73 | 9.13 | 6.18 | 0.987 | 0.619 | 0.263 | 0.145 | 3863 | 91 KB |
| 3e-7 | 0.905 | 1.40 | 3.22 | 3.00 | 0.346 | 0.232 | 0.084 | 0.044 | 6199 | 145 KB |
| 1e-7 | 0.504 | 0.609 | 1.78 | 1.28 | 0.122 | 0.082 | 0.041 | 0.021 | 9537 | 224 KB |
| 3e-8 | 0.124 | 0.226 | 0.561 | 0.500 | 0.054 | 0.033 | 0.014 | 0.007 | 15860 | 372 KB |
| 1e-8 | 0.071 | 0.099 | 0.287 | 0.242 | 0.018 | 0.009 | 0.003 | 0.002 | 21949 | 514 KB |
| 3e-9 | 0.017 | 0.028 | 0.075 | 0.097 | 0.004 | 0.002 | 8.1e-4 | 5.5e-4 | 26639 | 624 KB |
| 1e-9 | 0.009 | 0.012 | 0.032 | 0.034 | 0.002 | 7.3e-4 | 3.6e-4 | 2.8e-4 | 29902 | 701 KB |
| 1e-10 | 5.7e-6 | 0.001 | 2.8e-5 | 1.4e-5 | 1.4e-6 | 5.9e-7 | 3.3e-7 | 1.9e-7 | 36164 | 848 KB |
| 0 (whole theory) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 39198 | 919 KB |

## What the claims measure to

| proposed rule | verdict | measured |
|---|---|---|
| `compact` holds 1 arcminute in tens of KB | **holds** | 59 KB at threshold 3e-6 |
| `standard` holds 1 arcsecond in a few hundred KB | **holds** | 372 KB at threshold 3e-8 |
| `full` is a few MB | **holds** | 919 KB |


## What this does not measure, and why it matters

**This is truncation error, not accuracy.** The last column is the whole
theory compared against itself, so it reads zero by construction.
VSOP87's own departure from reality is not zero: its authors put it near
an arcsecond for the inner planets over the span this sweep covers. So
the total error a consumer sees is this table's figure **plus** a theory
floor this pass cannot see, and at the tight end of the ladder the two
are of the same order.

That has a consequence for the tiers. `standard` reaching 0.561 at a
threshold of 3e-8 is buying precision below the floor: the threshold
above it is a third of the size and still inside the arcsecond the
theory itself can promise. **Which of them is the right `standard`
cannot be decided from this page.** It needs the floor measured against
Teimeris, which is the next pass. Choosing now would be doing by
intuition the thing this pass exists to prevent.

**The Moon is not here.** ELP/MPP02 is a separate ingestion, and the
research page says the tiers are chosen by Moon accuracy first, because
nakshatra and tithi boundaries are what a consumer feels. Every figure
above is planets only, and the tier boundaries are provisional until the
Moon is measured beside them.

**The range is 1800 to 2400**, which is `standard`'s own span. VSOP87 is
published as valid far wider and its error grows towards the edges, so a
tier claiming a wider range has to be swept over that range rather than
inheriting this one's figures.

