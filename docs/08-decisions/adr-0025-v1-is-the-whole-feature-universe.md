# ADR-0025: v1.0 is the whole feature universe, not baseline parity

Status: accepted (maintainer, 2026-09-10)
Date: 2026-09-10
Question: Q4 (revised)
Supersedes: the v1-scope clause of ADR-0005. ADR-0005's modularity
decision — module families as crates, profiles, size gates — stands
unchanged and is what makes this scope shippable.

## Context

ADR-0005 set v1.0 at baseline parity, with Western and Hellenistic
"designed in Phase 0 and kept honest by the chart model" but shipped in
v1.x. That was the right call against the evidence of 2026-09-04, which
was a survey of the products an astrologer buys.

The survey missed the tier a developer buys from
(`01-research/competitive-analysis/02-developer-market.md`, 2026-09-10).
Three facts from it bear on the scope:

1. The developer market is majority-Western. The leading modern library
   is Western-only and sells a hosted API in front of its own AGPL.
2. A competitor of this SDK's exact shape — Apache-2.0, pure Rust,
   seventeen crates — shipped Vedic *and* Western *and* ten more
   traditions at once, took 866 stars in five weeks, and then stalled
   with 840 KB of repository behind twelve traditions. Breadth without
   depth did not hold the position. Depth without breadth would not have
   taken it.
3. Splitting v1 by tradition splits the claim. "The astrology SDK" and
   "the Vedic SDK that will do Western later" are different products to
   a developer choosing a dependency, and only the first ends the search.

The architecture was already built for this. The fifteen capability areas
of `01-research/feature-universe/00-taxonomy.md` are shared machinery, and
a tradition is a profile over them: `western` and `hellenistic` are
already modules in `02-architecture/01-module-catalog.md`. Nothing here
adds a mechanism; it moves work across a release line.

## Decision

**v1.0 ships the whole feature universe.** Western foundations and
Hellenistic time lords move from v1.x into v1, alongside the Vedic
depth that was already scoped. The maintainer accepted the longer road
explicitly (2026-09-10): "we will deploy everything we discussed and plan
in first version, although it takes longer time."

What moves into v1:

| module | area | what it brings |
|---|---|---|
| `western` | B, E, G, I, K, M | the tropical frame as a first-class peer of the sidereal one, Ptolemaic aspects with orb models, midpoints, Arabic parts, secondary and tertiary progressions, solar arc directions, solar and lunar returns, synastry, composite and Davison, declinations and parallels, harmonics |
| `hellenistic` | E, F, H, I, J | sect, terms and faces, dignity scores and almutens, lots, zodiacal releasing, annual profections, firdaria, decennials, horary considerations |

What does not change: the exit criteria of Phases 0 to 6 and 8, the
baseline migration, the evidence discipline of ADR-0018 (a Western rule
carries a citation exactly as a Vedic one does), and ADR-0005's
modularity — a consumer who wants only `panchanga` still ships only
`panchanga`, which is precisely what makes a larger v1 cost a Vedic
consumer nothing.

## Consequences

- **v1.0 is later.** Accepted deliberately. The release is gated by
  exits, not dates, and this adds two module families' worth of exits.
- **The profile set grows.** `western` and `hellenistic` join
  `panchanga`, `kundali`, `baseline-parity` and `full` as named
  profiles; the size gate per profile per platform now proves the claim
  that breadth is free to whoever does not use it.
- **The conformance corpus must cover Western.** The current corpus is
  Vedic. Western and Hellenistic fixtures have to be sourced or computed
  and recorded before their exits can be gated, and the oracle for them
  is not Teimeris. This is the largest new unknown and is tracked as a
  crux, not assumed away.
- **The rule engine's tradition-neutrality stops being theoretical.** It
  was specified as neutral and has only ever run Vedic rules. Western
  configurations and horary considerations are the falsification.
- **The claim changes.** The SDK is no longer "a Vedic SDK with Western
  designed in"; it is an astrology SDK. That is the claim that ends a
  developer's search, and it is only worth making once it is true.

## Alternatives considered

**Keep ADR-0005's split** — fastest to the baseline migration, and the
depth would still be unmatched; concedes the larger half of the developer
market until v1.x and invites the comparison to be made against a
half-product.

**Core Western in v1, Hellenistic in v1.x** — the middle path, and the
one this record initially recommended. Rejected by the maintainer: the
Hellenistic time lords share the time-lord registry with the dashas, so
splitting them costs a second pass over the same kernel later, and the
tradition-neutrality of the rule engine is proven better by two unlike
traditions than by one and a half.

## Evidence

`01-research/competitive-analysis/02-developer-market.md` (measured, not
quoted: registry and repository figures for the competing SDK);
`01-research/feature-universe/00-taxonomy.md` (the fifteen areas and
traditions-as-profiles); `01-research/feature-universe/15-western-modern.md`
and `16-hellenistic-medieval.md`; `02-architecture/01-module-catalog.md`.
