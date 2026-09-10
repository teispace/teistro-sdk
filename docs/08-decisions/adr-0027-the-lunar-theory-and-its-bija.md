# ADR-0027: The lunar theory, its bija correction, and the tier bounds that follow

Status: accepted (maintainer, 2026-09-10)
Date: 2026-09-10
Question: Q37
Amends: ADR-0013 (which named ELP/MPP02) and ADR-0021's `reference`
size budget. ADR-0008's three-tier shape is unchanged.

## Context

Phase 3 needs a lunar theory, and the Moon is the body that decides the
tiers: panchanga publishes instants, and one arcsecond of lunar
longitude is **2.25 seconds of tithi boundary**, so a second-accurate
almanac wants the Moon to 0.445 arcseconds
(`03-design/lunar-accuracy-measured.md`).

Three findings, each measured rather than argued, closed off the plan as
written.

1. **ELP/MPP02 cannot be obtained.** The address it is published at no
   longer serves it; the Internet Archive holds the directory listing
   and none of the six files; IMCCE and CDS carry only ELP2000-82B; and
   the two public re-implementations that do carry data are GPL-3.0 and
   EUPL-1.2, which `deny.toml` refuses everywhere in the workspace under
   ADR-0019. It is a removal, not an outage: the directory already
   answered 404 in the August 2022 crawl.
2. **ELP2000-82B alone misses by a wide margin.** Ported faithfully and
   measured against the engine in its own frame, it is 0.254 arcseconds
   over 1980 to 2020 — which is the check that the port is right — and
   **19.4 arcseconds over the 1800 to 2400 that `standard` claims**, or
   44 seconds of tithi. Its own reader says why: "Constants fitted to
   JPL's ephemerides DE200/LE200". Truncation is irrelevant to this: the
   ladder converges at 49 KB, and the remaining 36 733 terms buy
   nothing.
3. **Replacing it with a fitted table is far dearer than ADR-0021
   assumes.** An analytic theory costs the same whatever span it is
   asked for; a Chebyshev table costs one block per interval. The
   cheapest fit reaching 0.445 arcseconds over 1800 to 2400 is **3.76 MB
   for the Moon alone**, against ADR-0021's budget of about 1 MB for
   every body.

## Decision

**1. The built-in lunar theory is ELP2000-82B with a quadratic bija
correction to its mean longitude.**

The drift is not noise. A lunar theory ages through the tidal
acceleration its source ephemeris assumed, and that enters the mean
longitude as a term in the square of the time. Fitted over 1900 to 2100
and then scored on spans it never saw:

| degree | cost | 1900–2100 | 1800–2400 | 1700–2500 |
|---:|---:|---:|---:|---:|
| uncorrected | — | 1.69″ | 19.40″ | 29.74″ |
| 1 | 16 B | 0.867″ | 17.42″ | 27.37″ |
| **2** | **24 B** | **0.261″** | **3.00″** | **4.46″** |
| 3 | 32 B | 0.261″ | 3.08″ | 4.63″ |
| 4 | 40 B | 0.260″ | 4.53″ | 9.10″ |

Two independent things choose the square. It is fitted over two
centuries and repairs eight, which is what a fact about the theory does
and a curve fitted to a window does not. And the quartic is better on
its own span and **worse off it**, which is what overfitting looks like
from the outside. The model the evidence picks is the one the physics
predicts.

The tradition has both the idea and the name. A *bija* is a seed
correction that re-anchors an old theory to the present sky, and this
SDK already computes one for the Surya Siddhanta; the built-in
ephemeris's is the same device applied to the same kind of problem.

**2. The tier bounds are what was measured, per body, and no rounder.**

| tier | Moon | planets | span | size |
|---|---|---|---|---|
| `compact` | as `standard` — the Moon's cost is the theory, not the table | 1 arcminute | 1800–2400 | about 110 KB |
| `standard` | 0.26″ over 1900–2100, 3.0″ over 1800–2400 | 1″ for Mercury to Saturn; **Uranus 4.7″ and Neptune 6.6″** | 1800–2400 | about 420 KB |
| `reference` | by refit, if ADR-0021's fitter meets its target | by refit | 1800–2400 | **budget raised to 4 MB** |

`standard` no longer claims "2 arcsec Moon" or "1 arcsec planets"
without qualification. Both were falsified: the Moon by 19.4 arcseconds
uncorrected, and the outer planets by VSOP87's own age, which no
truncation can mend. The published figures are per body and per span,
which is what a consumer can actually rely on.

**3. `reference` is where second-accuracy over six centuries lives.**
The bija makes `standard` second-accurate over the two centuries it is
fitted to and minute-accurate over six. A consumer who needs a second at
the edges of the span takes `reference`, and its cost is now stated
honestly.

**4. The bija's provenance is declared, not silent.** Its coefficients
are fitted against the reference ephemeris named in the manifest, and
the fit's span and residual travel with them. Refitting against a
different or later ephemeris is a calculation-version change under
ADR-0020 like any other numeric change.

## Consequences

- The Moon costs about 50 KB and 24 bytes rather than 3.76 MB. The
  built-in ephemeris stays a module a wasm consumer can afford, which is
  the point of a `compact` tier.
- **The accuracy document gains rows per body, per span and per tier**,
  and stops carrying a single number that was true of neither the Moon
  nor the outer planets.
- ADR-0021's `reference` budget moves from about 1 MB to 4 MB. The
  degradation ladder it fixes in advance — split per body so a panchanga
  consumer ships the Sun and Moon only — becomes the likely shape rather
  than the fallback, because the Moon is where the megabytes are.
- The SDK's built-in ephemeris is calibrated to a modern reference and
  says so. Agreement with that reference proves consistency, not
  correctness (ADR-0021), so the bija is refitted against JPL Horizons
  or CSPICE before v1 and the figure republished if it moves.
- A consumer who wants the theory as its authors published it can have
  it: the bija is a declared, inspectable correction with a knob, not a
  silent adjustment baked into the tables.

## Alternatives considered

**ELP/MPP02, as ADR-0013 chose.** Unobtainable under the licence rules;
the only public copies are copyleft and transformed. If the publication
returns, its published 2.4 m over a century is 0.003 seconds of tithi
and it would replace this decision outright.

**A Chebyshev refit for the Moon at `standard`.** Rejected on the
measurement: 3.76 MB against 49 KB, for accuracy that only the
`reference` tier's consumers need.

**Narrowing `standard` to 1900 to 2100.** Would let the uncorrected
theory claim 1.69 arcseconds and needs nothing built. Rejected because
the bija costs 24 bytes and gives a better figure over a wider span; a
tier that quietly shrank its range would be solving a documentation
problem with a capability cut.

**Shipping ELP2000-82B uncorrected and stating 19.4 arcseconds.**
Honest, and 44 seconds of tithi is visibly wrong in a printed almanac
where boundaries are quoted to the minute.

## Evidence

`03-design/lunar-accuracy-measured.md` (the budget, the sourcing routes,
the theory measured, the three routes priced, the out-of-sample check);
`03-design/builtin-ephemeris-measured.md` (the planets, the truncation
curve and the theory floor); `crates/ephemeris-builtin/data/` for the
recorded tables; `elp82b.f` for the theory's own statement of its fit.
