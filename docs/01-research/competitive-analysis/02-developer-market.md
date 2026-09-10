# The developer market: libraries, engines and hosted APIs

Status: `research`, 2026-09-10. `01-products.md` surveyed the products an
*astrologer* buys. This page surveys what a *developer* buys, which is the
market the SDK actually enters. It was written because the first survey
missed that tier entirely, and because one entrant has since taken the
position the matrix called open.

Every figure below was read from the registry or the API on 2026-09-10, not
from a vendor's page, and is dated so it can be re-measured.

## Why this tier exists at all

Two costs stand between a team and an astrology feature, and the whole
hosted-API industry is priced against them.

1. **The ephemeris licence.** Swiss Ephemeris is AGPL-3.0 or a paid
   Professional Licence (CHF 750 for the first, CHF 1550 unlimited on the
   older list; CHF 700 one-off on the current one). Almost every product in
   `01-products.md` and almost every hosted API runs on it. Vendors
   advertise against the licence directly — one competitor's blog is titled
   for the question of whether the AGPL is a problem for an astrology SaaS.
2. **The domain layer.** The baseline engine is about 155 000 lines of
   astrology across seven packages, and it is not the outlier: it is what
   one team needed for one market. Every team that wants charts rebuilds
   some fraction of it.

A hosted API removes both by moving them behind a network call, and charges
rent for it. That is the business the SDK displaces — not by hosting, but by
making both costs zero at the library level.

## The tiers

| tier | who | what they sell | licence |
|---|---|---|---|
| ephemeris engines | Swiss Ephemeris, XALEN, Roxy's engine, Moshier ports | raw positions | AGPL or paid; Apache-2.0; MIT |
| Western libraries | Kerykeion, Immanuel, flatlib and its forks | chart data, aspects, SVG | AGPL-3.0; MIT-class |
| Vedic libraries | PyJHora, VedAstro, jyotishganit | Vedic depth in one language | mixed |
| general SDKs | **the position this SDK is for** | every tradition, every language | — |
| hosted APIs | Prokerala, AstrologyAPI, VedicRishi, RoxyAPI, Vedika, DivineAPI, FreeAstrologyAPI, Astrologer API | endpoints, per call | closed |

## Kerykeion — the leading modern library

Source: its GitHub page, fetched 2026-09-10.

Python, AGPL-3.0. Natal, synastry, transit, composite, solar and lunar
return charts; all Swiss house systems except Gauquelin; tropical plus 47
named sidereal modes and a custom mode; geocentric and heliocentric; 23
fixed stars; SVG rendering in two styles with six themes; ten languages;
JSON output; and an **AI Context Serializer** that emits XML "optimised for
LLM consumption". It is the engine behind a paid hosted API on RapidAPI,
which exists so that closed-source consumers can avoid its own AGPL.

For the SDK, four readings:

- Its sidereal-mode and house-system counts are the field's bar, and the
  catalogue already meets them: 47 ayanamshas, 22 house systems.
- **SVG rendering is what a developer expects a chart library to do.** The
  decided line — the SDK ships geometry and never draws
  (`feature-universe/18`) — stays right for a multi-language core, but it
  means the first-party rendering package it names is not optional
  garnish. It is what the comparison will be made on.
- The AGPL-plus-paid-API shape is the same trap as the ephemeris, one
  layer up. Apache-2.0 answers both at once.
- An LLM-facing serialisation is a shipped feature of the leading library,
  not a speculation. This does not reopen Q35, which the maintainer
  deferred; it is evidence for the discussion when it happens. The
  capability itself is already P0 as the dossier in `feature-universe/18`,
  and is independent of any MCP decision.

## XALEN — the position was contested

Source: `github.com/vedika-io/xalen-ephemeris` and the crates.io API, both
read 2026-09-10.

Pure-Rust, Apache-2.0, 866 stars, 122 forks. It describes itself as an
ephemeris for "Vedic, Western, Chinese and 9 world traditions" and
publishes seventeen crates: `xalen-ephem`, `-coords`, `-time`, `-houses`,
`-ayanamsa`, `-stars`, `-chart`, `-vedic`, `-western`, `-chinese`,
`-iching`, `-lalkitab`, `-numerology`, `-world`, `-ffi`, `-wasm`. VSOP87A
planets, truncated ELP2000-82 Moon, Meeus reductions with IAU 2006
precession and IAU 2000B nutation, validated against DE440.

**This is the shape of this SDK, from another team, shipped first.** The
matrix's fourth conclusion — "nobody offers an SDK across platforms … the
position is open" — was true when it was written on 2026-09-04 and is not
true now. That sentence has been corrected in `00-matrix.md`.

What the measurements say about how firmly it is held:

| measure | value on 2026-09-10 |
|---|---|
| repository size | 840 KB, for eighteen crates over twelve traditions |
| analytic range claimed | AD 1600 to 2100 |
| last commit | 2026-07-02 — over two months quiet |
| last crates.io release | 2026-06-02 |
| headline crate `xalen-ephemeris` | version 0.0.0, a placeholder |
| best-adopted crate `xalen-time` | 1 143 downloads all-time |
| Python and Node packages | announced; not on PyPI or npm |

866 stars in five weeks, then stall. Breadth was declared before depth
existed: twelve traditions in 840 KB is a README's worth of each, and the
registry numbers show nobody has come to depend on it. The position is
vacant again — but it has been proven to be a position somebody wants, and
proven takeable by whoever ships something installable.

## The hosted APIs

Prokerala, AstrologyAPI, VedicRishi, RoxyAPI, Vedika, DivineAPI,
FreeAstrologyAPI and Kerykeion's Astrologer API. Credit-wallet or
per-request pricing, roughly ₹1 000 (about US$12) a month for 100 000
credits at the cheap end, into the hundreds of dollars a month at volume,
with per-endpoint credit costs that make a bill hard to predict — a kundli
call costing fifty times a panchanga call is a documented complaint. Most
now advertise both Vedic and Western, and most run on Swiss Ephemeris;
independent engines are the exception rather than the rule.

For the SDK: these are the incumbent answer to "we need charts by Friday",
and their weaknesses are structural rather than incidental — a network hop
on every chart, no offline or on-device use, no determinism guarantee
across calls, no settings snapshot a consumer can pin, per-call cost that
scales with success, and the consumer's birth data leaving their
infrastructure. Each is something a library simply does not have.

## What this changes, and what it does not

**It does not change the architecture.** The fifteen capability areas and
"a tradition is a profile over shared machinery" (`feature-universe/00`)
are exactly what XALEN's twelve traditions in 840 KB failed to have, and
what a hosted API cannot expose. The evidence discipline — a citation per
rule, marks V/T/S, unsourced variants refused (Q28) — is the thing none of
this tier does at all, and it is the moat.

**It does change three judgements.**

1. **The ephemeris is no longer the differentiator.** Phase 3's value is
   removing the licence and the files, and a permissively-licensed Rust
   VSOP87-and-ELP engine now exists from someone else. What is still ours
   to claim is the range, the three size tiers, analytic speeds, and byte
   determinism across architectures — none of which the competing engine
   claims. Phase 3's accuracy document should measure against XALEN as
   well as Teimeris, since it is Apache-2.0 and installable, which makes it
   a legitimate second oracle rather than only a rival.
2. **Nothing of ours is installable, and the names are unclaimed.**
   crates.io returns zero for `teistro`; npm returns zero for `teistro`.
   The names decided in Q16 are all free and squattable, and 128 000 lines
   of working Rust are reachable by nobody. A competitor with a tenth of
   the substance took the attention with a `cargo add` line.
3. **Rendering geometry is a headline feature, not a Phase 9 tail.** It is
   already P0 in `feature-universe/18` while the roadmap carries it in
   Phase 9; the gap between those two should be closed deliberately rather
   than by default.

## Gaps found against the field, 2026-09-10

Small, concrete, and none of them architectural. Recorded here so they are
scheduled rather than remembered.

| gap | against | note |
|---|---|---|
| D81, D108, D144 not in `varga.yaml` | JHora | the `Scheme` algebra already expresses them; this is catalogue rows, not code |
| named varga variants (six hora, four D-3, three D-9 and D-30, two D-108) | JHora | the algebra takes them; the rows and their citations do not exist yet |
| a custom D-N a consumer names at runtime | JHora (1 to 300) | the algebra is general; nothing exposes it as a consumer-supplied scheme |
| sahamas as catalogued points | JHora's 36 | `point.yaml` has 47 points and no sahama; they are in `feature-universe` prose only |
| KP subs beyond four levels | JHora's five | the baseline engine stops at four |
| Nepali lotus (Ashtadala Padma) layout | the baseline engine's market | `feature-universe/18` lists North, South, East Indian and Bengali |
| an atlas | every product in `01-products.md` | no roadmap entry; decide explicitly whether it is a port, an optional data package, or out of scope |

## Sources

Read 2026-09-10: `github.com/g-battaglia/kerykeion`;
`github.com/vedika-io/xalen-ephemeris` and the GitHub and crates.io APIs;
`astro.com/swisseph` licence and price pages; `vedika.io` and `roxyapi.com`
comparison articles; `vedicastrologer.org/jh/features.htm`. Vendor claims
are recorded as claims; registry and API figures are measurements.
