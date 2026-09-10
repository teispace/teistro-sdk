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

## The theory's own floor

Every figure above is truncation against the whole theory, so the last
row is zero by construction. This is what the whole theory itself costs,
measured against Teimeris in the frame VSOP87 is stated in —
`HELIOCENTRIC/J2000/ECLIPTIC/TROPICAL/GEOMETRIC` — at 5479 instants 40
days apart, on the engine's `compatible` profile. Recorded by
`teistro-ephemeris-teimeris-vsop-floor` into
`crates/ephemeris-builtin/data/vsop87-floor.json`; the number, not the
code that produced it.

| body | scatter ″ | worst ″ | radius (relative) | 1800 ″ | 2100 ″ | 2400 ″ |
|---|---:|---:|---:|---:|---:|---:|
| Mercury | 0.248 | 0.373 | 2.8e-7 | 0.027 | 0.094 | 0.373 |
| Venus | 0.064 | 0.128 | 3.0e-8 | 0.127 | 0.055 | 0.034 |
| Mars | 0.833 | 1.07 | 3.8e-7 | 0.008 | 0.028 | 0.839 |
| Jupiter | 0.432 | 0.835 | 2.8e-7 | 0.200 | 0.422 | 0.696 |
| Saturn | 0.440 | 0.767 | 3.8e-7 | 0.142 | 0.346 | 0.762 |
| Uranus | 3.68 | 4.73 | 9.6e-6 | 0.780 | 0.033 | 4.08 |
| Neptune | 3.99 | 6.57 | 2.5e-6 | 2.05 | 2.30 | 6.57 |

VSOP87's own documentation states a precision of one arcsecond for every
planet over this span, and a relative precision per body that puts
Neptune near a tenth of an arcsecond. **Measured against a modern
ephemeris it does not hold for the outer two.** Mercury to Saturn stay
inside 1.07 arcseconds; Uranus and Neptune reach 6.57.

The two diagnostics say what kind of difference it is. The heliocentric
**radius** agrees to about one part in ten million for the six inner
bodies and to one part in a hundred thousand for Uranus, so the orbit is
not what disagrees — the body sits at a different place along it. And
the **trend** grows from 1800 towards 2400 for every body rather than
staying flat, which is a fit drifting from its epoch and not a rotation
between frames. VSOP87 was fitted to DE200, published in 1981; the
engine answers from a modern one.

**This falsifies part of the plan.** `standard` claims one arcsecond for
the planets, and no truncation can deliver that for Uranus or Neptune:
the theory is the limit, not the table. Either `standard` states a bound
per body, or the outer planets come from the `reference` tier's refit,
which ADR-0021 already sizes at 0.02 arcseconds for them. The choice
belongs in the design page; what this page establishes is that the
single-number claim is not available.


## What this does not measure

**Which threshold `standard` should take** is now decidable and is not
decided here. The floor above says what the theory costs; the sweep says
what each truncation costs; the design page picks the pair. What this
page refuses to do is pick it in passing.

**The Moon.** ELP/MPP02 is a separate ingestion, and the research page
says the tiers are chosen by Moon accuracy first, because nakshatra and
tithi boundaries are what a consumer feels. Every figure here is planets
only, and every tier boundary is provisional until the Moon is measured
beside them.

**Any range but 1800 to 2400**, which is `standard`'s own span. The
trend column shows the disagreement growing towards 2400 for every body,
so a tier claiming a wider range has to be swept over that range rather
than inheriting these figures.

**Pluto, the nodes and the apogees**, which have no VSOP87 series at
all: Pluto is fitted from a public-domain kernel and the rest are mean
elements (ADR-0021).


## What the claims measure to

| proposed rule | verdict | measured |
|---|---|---|
| `compact` holds 1 arcminute in tens of KB | **holds** | 59 KB at threshold 3e-6 |
| `standard` holds 1 arcsecond in a few hundred KB | **holds** | 372 KB at threshold 3e-8 |
| `full` is a few MB | **holds** | 919 KB |
| `standard` holds 1 arcsecond for **every** planet, the theory included | falsified | Neptune is 6.57 arcseconds from the engine with every term kept |

