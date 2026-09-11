# The completion, measured

Status: `generated` by `cargo xtask agreement`, gated by `check-agreement`. Do not edit. The numbers are read from `crates/ephemeris-builtin/data/stack-agreement-<tier>.json`, recorded by `teistro-ephemeris-teimeris-stack-agreement` against the reference engine — the number, not the code that produced it.

Every other measurement in this directory isolates one thing: a truncation against the whole theory, a theory against the engine in its own frame, the Moon against a tithi. **This one measures what a consumer receives.** A position in the frame a chart is actually cast in — in the equinox of date, sidereal by Lahiri, apparent — with the SDK's own completion doing precession, light time, deflection, aberration, nutation and the ayanamsha over the built-in ephemeris, against the same frame from the reference engine.

It is **not** two runs of one pipeline, and the next section says why that could not be arranged and what follows from it. Read the geocentric table for the ephemeris and the topocentric one for the chart.

The frame is `OF_DATE/ECLIPTIC/SIDEREAL(LAHIRI)/APPARENT, from both centres`, on the engine's `compatible` profile, at 5479 instants 40 days apart from JD 2378497.0 to 2597641.0 (1800 to 2400).

## `compact`

**The two sides are not one pipeline, and cannot be made one.** The engine answers the whole frame in a single native call — its step list is `positions:Native` and nothing else — while the built-in ephemeris is completed by the SDK's own steps. Forcing the SDK's everywhere was tried and is refused: the engine's native positions are *apparent*, the SDK can add corrections and never remove them, so a geometric frame comes back `Unsupported { step: "corrections" }`.

So both centres are recorded, and the pair is the point. **Geocentric isolates the ephemeris**: the instants are TT, so nothing in that path needs Delta T. **Topocentric is what a chart receives**, and it carries the two sides' Delta T models as well, through the observer's own rotation.

### From the Earth's centre — the ephemeris

| body | worst | mean | 1800 | 2100 | 2400 | worst speed ("/day) | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|
| SUN | 5.45 | 1.16 | 0.164 | 0.206 | 0.516 | 0.186 | 1844 |
| MOON | 14.3 | 2.33 | 0.884 | 1.70 | 3.90 | 9.48 | 1864 |
| MERCURY | 8.49 | 1.41 | 2.30 | 0.966 | 2.01 | 3.82 | 2397 |
| VENUS | 21.8 | 1.68 | 1.17 | 0.205 | 0.048 | 1.15 | 2209 |
| MARS | 20.9 | 1.40 | 0.324 | 3.95 | 1.53 | 0.541 | 2224 |
| JUPITER | 2.47 | 0.522 | 0.059 | 0.850 | 0.006 | 90.0 | 2119 |
| SATURN | 1.78 | 0.438 | 1.02 | 0.238 | 0.965 | 2.55 | 1820 |
| MEAN_NODE | 0.985 | 0.326 | 0.330 | 0.182 | 0.985 | 0.001 | 2400 |
| TRUE_NODE | 2082 | 543 | 722 | 6.57 | 1508 | 103 | 2384 |

Arcseconds of ecliptic longitude, the shortest way round the circle.

### From the place — what a chart receives

| body | worst | mean | 1800 | 2100 | 2400 | worst speed ("/day) | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|
| SUN | 5.42 | 1.15 | 0.145 | 0.225 | 0.524 | 1.92 | 1844 |
| MOON | 119 | 23.9 | 0.401 | 20.3 | 92.9 | 751 | 2396 |
| MERCURY | 8.55 | 1.40 | 2.26 | 0.945 | 2.05 | 5.31 | 2397 |
| VENUS | 21.9 | 1.68 | 1.35 | 0.057 | 0.030 | 5.26 | 2209 |
| MARS | 20.9 | 1.40 | 0.463 | 4.11 | 1.48 | 3.15 | 2224 |
| JUPITER | 2.58 | 0.534 | 0.004 | 1.07 | 0.140 | 80.6 | 2119 |
| SATURN | 1.94 | 0.459 | 0.884 | 0.021 | 0.896 | 2.45 | 1820 |
| MEAN_NODE | 0.985 | 0.326 | 0.330 | 0.182 | 0.985 | 0.007 | 2400 |
| TRUE_NODE | 2082 | 543 | 722 | 6.57 | 1508 | 104 | 2384 |

Arcseconds of ecliptic longitude, the shortest way round the circle.

The Moon is the whole of the difference between the two tables: 14.3″ from the centre, 119″ from the place. Nothing about the ephemeris changed between them. Delta T did — at 2400 the two sides' models differ by enough Earth rotation to move the Moon about a hundred arcseconds, and the Moon is the one body near enough for the observer's position to matter. The trend says the same thing: 0.401″ at 1800, where Delta T is recorded, against 92.9″ at 2400, where it is extrapolated.

The geocentric Moon again as **seconds of tithi boundary**, which is what ADR-0027 argues in: worst 32, 1800 2, 2100 4, 2400 9. The recorded topocentric worst, for comparison, is 268 seconds.

**The bija, measured rather than declared.** From the centre, the Moon is 14.3″ with it and 29.5″ without — the correction removes 15.3″ of 29.5″, fitted over 1900 to 2100. The true node moves 2082″ against 2066″, which is the same shift carried through and not a correction of its own.

ADR-0027 argued the theory to be 19.4″ uncorrected over this span and about 3 corrected, from a floor measurement in ELP's own frame. Reaching 29.5″ and 14.3″ again through the whole completion is an independent path to the same two numbers.

It is a knob rather than a baked-in adjustment, so what it is worth is recorded beside what it was set to — where a `bija: true` field would have asserted only that somebody meant to switch it on.

**The true node is the open question on this page.** It reads 2082″ — identical from both centres, which is correct, because a node is a *direction* and the centre step leaves directions alone. But that is 146 times the Moon's own 14.3″, where the geometry predicts about eleven: a node is where the orbit crosses the ecliptic, so an error arrives there divided by the tangent of a five-degree inclination. Two candidates are already excluded by measurement. It is **not truncation** — the `full` tier, the theories entire, gives the same figure. It is **not the bija** — turning it off moves the node 15.6″, of 2082″. What is left is the latitude and the velocity the node is built from, and that is measured where those are rather than guessed at here.

## `standard`

**The two sides are not one pipeline, and cannot be made one.** The engine answers the whole frame in a single native call — its step list is `positions:Native` and nothing else — while the built-in ephemeris is completed by the SDK's own steps. Forcing the SDK's everywhere was tried and is refused: the engine's native positions are *apparent*, the SDK can add corrections and never remove them, so a geometric frame comes back `Unsupported { step: "corrections" }`.

So both centres are recorded, and the pair is the point. **Geocentric isolates the ephemeris**: the instants are TT, so nothing in that path needs Delta T. **Topocentric is what a chart receives**, and it carries the two sides' Delta T models as well, through the observer's own rotation.

### From the Earth's centre — the ephemeris

| body | worst | mean | 1800 | 2100 | 2400 | worst speed ("/day) | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|
| SUN | 0.228 | 0.075 | 0.156 | 0.066 | 0.068 | 0.014 | 1830 |
| MOON | 3.23 | 0.422 | 0.559 | 0.209 | 1.35 | 2.94 | 2389 |
| MERCURY | 0.850 | 0.146 | 0.172 | 0.088 | 0.058 | 3.23 | 1802 |
| VENUS | 1.71 | 0.149 | 0.121 | 0.103 | 0.004 | 0.328 | 2155 |
| MARS | 4.80 | 0.275 | 0.020 | 0.052 | 0.532 | 0.520 | 2399 |
| JUPITER | 1.12 | 0.408 | 0.273 | 0.234 | 0.601 | 90.0 | 2296 |
| SATURN | 1.09 | 0.349 | 0.222 | 0.017 | 0.685 | 2.56 | 2351 |
| MEAN_NODE | 0.985 | 0.326 | 0.330 | 0.182 | 0.985 | 0.001 | 2400 |
| TRUE_NODE | 2044 | 541 | 768 | 34.7 | 1559 | 34.4 | 2393 |

Arcseconds of ecliptic longitude, the shortest way round the circle.

### From the place — what a chart receives

| body | worst | mean | 1800 | 2100 | 2400 | worst speed ("/day) | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|
| SUN | 0.201 | 0.061 | 0.137 | 0.047 | 0.076 | 1.77 | 1830 |
| MOON | 117 | 23.1 | 1.86 | 18.4 | 95.4 | 750 | 2395 |
| MERCURY | 0.856 | 0.145 | 0.213 | 0.068 | 0.024 | 4.75 | 2379 |
| VENUS | 1.70 | 0.171 | 0.059 | 0.250 | 0.014 | 5.16 | 2155 |
| MARS | 4.84 | 0.331 | 0.159 | 0.218 | 0.490 | 3.00 | 2399 |
| JUPITER | 1.21 | 0.433 | 0.328 | 0.459 | 0.466 | 80.6 | 2296 |
| SATURN | 1.27 | 0.378 | 0.084 | 0.234 | 0.616 | 2.45 | 2333 |
| MEAN_NODE | 0.985 | 0.326 | 0.330 | 0.182 | 0.985 | 0.007 | 2400 |
| TRUE_NODE | 2044 | 541 | 768 | 34.7 | 1559 | 41.7 | 2393 |

Arcseconds of ecliptic longitude, the shortest way round the circle.

The Moon is the whole of the difference between the two tables: 3.23″ from the centre, 117″ from the place. Nothing about the ephemeris changed between them. Delta T did — at 2400 the two sides' models differ by enough Earth rotation to move the Moon about a hundred arcseconds, and the Moon is the one body near enough for the observer's position to matter. The trend says the same thing: 1.86″ at 1800, where Delta T is recorded, against 95.4″ at 2400, where it is extrapolated.

The geocentric Moon again as **seconds of tithi boundary**, which is what ADR-0027 argues in: worst 7, 1800 1, 2100 0, 2400 3. The recorded topocentric worst, for comparison, is 262 seconds.

**The bija, measured rather than declared.** From the centre, the Moon is 3.23″ with it and 19.3″ without — the correction removes 16.1″ of 19.3″, fitted over 1900 to 2100. The true node moves 2044″ against 2060″, which is the same shift carried through and not a correction of its own.

ADR-0027 argued the theory to be 19.4″ uncorrected over this span and about 3 corrected, from a floor measurement in ELP's own frame. Reaching 19.3″ and 3.23″ again through the whole completion is an independent path to the same two numbers.

It is a knob rather than a baked-in adjustment, so what it is worth is recorded beside what it was set to — where a `bija: true` field would have asserted only that somebody meant to switch it on.

**The true node is the open question on this page.** It reads 2044″ — identical from both centres, which is correct, because a node is a *direction* and the centre step leaves directions alone. But that is 633 times the Moon's own 3.23″, where the geometry predicts about eleven: a node is where the orbit crosses the ecliptic, so an error arrives there divided by the tangent of a five-degree inclination. Two candidates are already excluded by measurement. It is **not truncation** — the `full` tier, the theories entire, gives the same figure. It is **not the bija** — turning it off moves the node 16.4″, of 2044″. What is left is the latitude and the velocity the node is built from, and that is measured where those are rather than guessed at here.

## `full`

**The two sides are not one pipeline, and cannot be made one.** The engine answers the whole frame in a single native call — its step list is `positions:Native` and nothing else — while the built-in ephemeris is completed by the SDK's own steps. Forcing the SDK's everywhere was tried and is refused: the engine's native positions are *apparent*, the SDK can add corrections and never remove them, so a geometric frame comes back `Unsupported { step: "corrections" }`.

So both centres are recorded, and the pair is the point. **Geocentric isolates the ephemeris**: the instants are TT, so nothing in that path needs Delta T. **Topocentric is what a chart receives**, and it carries the two sides' Delta T models as well, through the observer's own rotation.

### From the Earth's centre — the ephemeris

| body | worst | mean | 1800 | 2100 | 2400 | worst speed ("/day) | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|
| SUN | 0.152 | 0.076 | 0.151 | 0.056 | 0.071 | 0.007 | 1814 |
| MOON | 2.93 | 0.416 | 0.549 | 0.085 | 1.46 | 2.93 | 2389 |
| MERCURY | 0.868 | 0.145 | 0.178 | 0.077 | 0.076 | 3.22 | 1840 |
| VENUS | 1.51 | 0.145 | 0.097 | 0.114 | 0.003 | 0.311 | 1896 |
| MARS | 4.84 | 0.276 | 0.042 | 0.011 | 0.526 | 0.520 | 2399 |
| JUPITER | 1.10 | 0.404 | 0.285 | 0.221 | 0.604 | 90.0 | 2296 |
| SATURN | 1.09 | 0.349 | 0.222 | 0.016 | 0.690 | 2.56 | 2351 |
| MEAN_NODE | 0.985 | 0.326 | 0.330 | 0.182 | 0.985 | 0.001 | 2400 |
| TRUE_NODE | 2044 | 541 | 767 | 34.2 | 1560 | 31.9 | 2393 |

Arcseconds of ecliptic longitude, the shortest way round the circle.

### From the place — what a chart receives

| body | worst | mean | 1800 | 2100 | 2400 | worst speed ("/day) | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|
| SUN | 0.137 | 0.061 | 0.131 | 0.038 | 0.079 | 1.78 | 1801 |
| MOON | 117 | 23.1 | 1.85 | 18.5 | 95.3 | 750 | 2395 |
| MERCURY | 0.793 | 0.143 | 0.219 | 0.056 | 0.043 | 4.74 | 1840 |
| VENUS | 1.53 | 0.166 | 0.083 | 0.261 | 0.015 | 5.20 | 2324 |
| MARS | 4.87 | 0.330 | 0.181 | 0.154 | 0.484 | 2.99 | 2399 |
| JUPITER | 1.20 | 0.430 | 0.341 | 0.446 | 0.470 | 80.6 | 2296 |
| SATURN | 1.26 | 0.377 | 0.083 | 0.233 | 0.621 | 2.45 | 2333 |
| MEAN_NODE | 0.985 | 0.326 | 0.330 | 0.182 | 0.985 | 0.007 | 2400 |
| TRUE_NODE | 2044 | 541 | 767 | 34.2 | 1560 | 41.4 | 2393 |

Arcseconds of ecliptic longitude, the shortest way round the circle.

The Moon is the whole of the difference between the two tables: 2.93″ from the centre, 117″ from the place. Nothing about the ephemeris changed between them. Delta T did — at 2400 the two sides' models differ by enough Earth rotation to move the Moon about a hundred arcseconds, and the Moon is the one body near enough for the observer's position to matter. The trend says the same thing: 1.85″ at 1800, where Delta T is recorded, against 95.3″ at 2400, where it is extrapolated.

The geocentric Moon again as **seconds of tithi boundary**, which is what ADR-0027 argues in: worst 7, 1800 1, 2100 0, 2400 3. The recorded topocentric worst, for comparison, is 263 seconds.

**The bija, measured rather than declared.** From the centre, the Moon is 2.93″ with it and 19.5″ without — the correction removes 16.5″ of 19.5″, fitted over 1900 to 2100. The true node moves 2044″ against 2061″, which is the same shift carried through and not a correction of its own.

ADR-0027 argued the theory to be 19.4″ uncorrected over this span and about 3 corrected, from a floor measurement in ELP's own frame. Reaching 19.5″ and 2.93″ again through the whole completion is an independent path to the same two numbers.

It is a knob rather than a baked-in adjustment, so what it is worth is recorded beside what it was set to — where a `bija: true` field would have asserted only that somebody meant to switch it on.

**The true node is the open question on this page.** It reads 2044″ — identical from both centres, which is correct, because a node is a *direction* and the centre step leaves directions alone. But that is 699 times the Moon's own 2.93″, where the geometry predicts about eleven: a node is where the orbit crosses the ecliptic, so an error arrives there divided by the tangent of a five-degree inclination. Two candidates are already excluded by measurement. It is **not truncation** — the `full` tier, the theories entire, gives the same figure. It is **not the bija** — turning it off moves the node 16.4″, of 2044″. What is left is the latitude and the velocity the node is built from, and that is measured where those are rather than guessed at here.

## One correction at a time

A disagreement in the apparent frame is five corrections at once, and a table of it names none of them. This ladder asks for one correction at a time, at `standard`, so a number can be attributed.

The first column is the check that makes the rest mean anything: **how far the engine's own answer moves** when only that correction is asked for. A flag the engine does not honour leaves it at zero, and a comparison against a correction the engine did not apply says nothing at all.

| correction | engine moves the Sun | Sun | Moon | Mars |
|---|---:|---:|---:|---:|
| geometric (of date) | 0.000 | 0.227 | 2.75 | 3.98 |
| light time only | 0.011 | 0.227 | 2.75 | 3.98 |
| deflection only | 0.000 | 0.228 | 2.75 | 3.98 |
| aberration only | 0.000 | 20.9 | 23.3 | 21.3 |
| nutation only | 18.9 | 0.225 | 3.22 | 4.79 |
| apparent, tropical | 39.6 | 0.226 | 3.22 | 4.79 |
| apparent, sidereal | 106035 | 0.228 | 3.23 | 4.80 |

Of the four single corrections the engine distinguishes `light time` and `nutation` — asking for `deflection` and `aberration` moves its own answer not at all. So those rows do not measure agreement: they measure the SDK applying a correction the engine did not, and are here to be read that way rather than mistaken for accuracy. **Only the rows the engine honours, and the two whole-frame rows at the foot, compare like with like.**

The last row runs the whole completion: `positions:Native`, `equinox:Sdk`, `obliquity:Sdk`, `corrections:Sdk`, `ayanamsha:Sdk`, `zodiac-shift:Sdk`. Each names who did the work — `Native` is the provider's own answer, `Sdk` is this crate's.

## The claims this decides

Each bound is one ADR-0027 argued from, not one chosen to be met, and each is decided on the **geocentric** table — the one that isolates the ephemeris. Deciding them on the topocentric table would be charging the lunar theory for a Delta T model six centuries out. A falsified row is the measurement doing its job: it says the tier cannot carry that claim, and the tier's published figure becomes the measured one.

| proposed rule | verdict | measured |
|---|---|---|
| `compact`: the retired single-number claim — one arcsecond for every planet | falsified | worst 21.8″ |
| `compact`: the Sun stays inside one arcsecond over the whole span | falsified | worst 5.45″ |
| `compact`: second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027) | falsified | worst 3.83 s of tithi |
| `compact`: minute-accurate over the whole six centuries (ADR-0027) | **holds** | worst 32 s of tithi |
| `compact`: the Moon meets the almanac's 0.445 arcseconds | falsified | worst 14.3″ |
| `compact`: the mean node is inside one arcsecond | **holds** | worst 0.985″ |
| `standard`: the retired single-number claim — one arcsecond for every planet | falsified | worst 4.80″ |
| `standard`: the Sun stays inside one arcsecond over the whole span | **holds** | worst 0.228″ |
| `standard`: second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027) | **holds** | worst 0.47 s of tithi |
| `standard`: minute-accurate over the whole six centuries (ADR-0027) | **holds** | worst 7.27 s of tithi |
| `standard`: the Moon meets the almanac's 0.445 arcseconds | falsified | worst 3.23″ |
| `standard`: the mean node is inside one arcsecond | **holds** | worst 0.985″ |
| `full`: the retired single-number claim — one arcsecond for every planet | falsified | worst 4.84″ |
| `full`: the Sun stays inside one arcsecond over the whole span | **holds** | worst 0.152″ |
| `full`: second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027) | **holds** | worst 0.19 s of tithi |
| `full`: minute-accurate over the whole six centuries (ADR-0027) | **holds** | worst 6.58 s of tithi |
| `full`: the Moon meets the almanac's 0.445 arcseconds | falsified | worst 2.93″ |
| `full`: the mean node is inside one arcsecond | **holds** | worst 0.985″ |

8 of 18 falsified:

- `compact`: the retired single-number claim — one arcsecond for every planet — worst 21.8″
- `compact`: the Sun stays inside one arcsecond over the whole span — worst 5.45″
- `compact`: second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027) — worst 3.83 s of tithi
- `compact`: the Moon meets the almanac's 0.445 arcseconds — worst 14.3″
- `standard`: the retired single-number claim — one arcsecond for every planet — worst 4.80″
- `standard`: the Moon meets the almanac's 0.445 arcseconds — worst 3.23″
- `full`: the retired single-number claim — one arcsecond for every planet — worst 4.84″
- `full`: the Moon meets the almanac's 0.445 arcseconds — worst 2.93″

**None of them is the completion.** What the completion decides is the Sun, the mean node and the frame the whole table is stated in; those are the rows that hold wherever the ephemeris under them is good enough to show it.

Of these, two still fail at `full`, the theories entire, so no table can mend them — they are the **age of a published theory**. VSOP87 was fitted to DE200 in 1981 and the engine answers from a modern ephemeris, which is why a planet drifts; ELP2000-82B carries the same era's tidal term, which is what the bija removes and what it cannot remove all of. Only a refit against a modern reference — `reference`, in ADR-0021's ladder — moves these.

- the retired single-number claim — one arcsecond for every planet
- the Moon meets the almanac's 0.445 arcseconds

The rest fail only at the lighter tiers, and that is the **truncation those tiers were chosen to buy** rather than a defect: `compact` is an arcminute in about 80 KB, and it says so.

- the Sun stays inside one arcsecond over the whole span
- second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027)

## What the recording says it measured

> the engine answers the whole frame natively (`positions:Native` alone) and the built-in ephemeris is completed by the SDK's own steps, so these are two pipelines and not one; geocentric isolates the ephemeris because the instants are TT and nothing in that path needs Delta T, while topocentric additionally carries the two sides' Delta T models

## What this does not measure

**Correctness.** Both sides are compared against the same reference engine, so agreement proves consistency and not truth (ADR-0021). The bija is refitted against JPL Horizons or CSPICE before v1 and every figure here republished if it moves.

**Any span but 1800 to 2400.** Every trend column grows towards the edges, so a tier claiming a wider range is swept over that range rather than inheriting these numbers.

**Latitude and distance.** The tables are ecliptic longitude, which is what a sign, a nakshatra, a tithi and a dasha are computed from. Latitude matters to a graha's war and to the true node, and is measured where those are.

**Pluto**, which has no series here at all and is fitted from a public-domain kernel separately.

