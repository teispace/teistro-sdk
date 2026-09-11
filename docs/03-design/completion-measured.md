# The completion, measured

Status: `generated` by `cargo xtask agreement`, gated by `check-agreement`. Do not edit. The numbers are read from `crates/ephemeris-builtin/data/stack-agreement-<tier>.json`, recorded by `teistro-ephemeris-teimeris-stack-agreement` against the reference engine — the number, not the code that produced it.

Every other measurement in this directory isolates one thing: a truncation against the whole theory, a theory against the engine in its own frame, the Moon against a tithi. **This one measures what a consumer receives.** A position in the frame a chart is actually cast in — in the equinox of date, sidereal by Lahiri, apparent — with the SDK's own completion doing precession, light time, deflection, aberration, nutation and the ayanamsha over the built-in ephemeris, against the same frame from the reference engine.

It is **not** two runs of one pipeline, and the next section says why that could not be arranged and what follows from it. Read the geocentric table for the ephemeris and the topocentric one for the chart.

The frame is `OF_DATE/ECLIPTIC/SIDEREAL(LAHIRI)/APPARENT, from both centres`, on the engine's `compatible` profile, at 5479 instants 40 days apart from JD 2378497.0 to 2597641.0 (1800 to 2400).

## `compact`

**The two sides are not one pipeline, and cannot be made one.** The engine answers the whole frame in a single native call — its step list is `positions:Native` and nothing else — while the built-in ephemeris is completed by the SDK's own steps. Forcing the SDK's everywhere was tried and is refused: the engine's native positions are *apparent*, the SDK can add corrections and never remove them, so a geometric frame comes back `Unsupported { step: "corrections" }`.

So both centres are recorded, and the pair is the point. **Geocentric isolates the ephemeris**: the instants are TT, so nothing in that path needs Delta T. **Topocentric is what a chart receives**, and it carries the two sides' Delta T models as well, through the observer's own rotation.

### From the Earth's centre — the ephemeris

| body | worst | mean | 1800 | 2100 | 2400 | latitude | distance | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| SUN | 5.45 | 1.16 | 0.164 | 0.206 | 0.516 | 1.17 | 1.7e-5 | 1844 |
| MOON | 14.6 | 2.33 | 1.29 | 1.63 | 4.27 | 11.0 | 1.3e-4 | 1864 |
| MERCURY | 8.32 | 1.40 | 2.29 | 0.958 | 2.03 | 2.60 | 2.2e-4 | 2350 |
| VENUS | 21.4 | 1.67 | 1.42 | 0.376 | 0.008 | 6.09 | 1.7e-4 | 2209 |
| MARS | 21.1 | 1.40 | 0.331 | 4.04 | 1.51 | 4.33 | 9.8e-5 | 2224 |
| JUPITER | 2.61 | 0.516 | 0.063 | 1.02 | 0.002 | 0.817 | 1.7e-5 | 2119 |
| SATURN | 1.59 | 0.418 | 0.970 | 0.084 | 0.977 | 0.672 | 8.9e-6 | 2301 |
| URANUS | 4.85 | 1.02 | 0.892 | 0.058 | 3.88 | 0.317 | 1.1e-5 | 2334 |
| NEPTUNE | 6.76 | 2.59 | 1.98 | 2.27 | 6.71 | 0.169 | 4.3e-6 | 2399 |
| PLUTO | 33.3 | 8.73 | 13.4 | 7.58 | 4.49 | 6.43 | 1.4e-4 | 2260 |
| MEAN_NODE | 0.987 | 0.326 | 0.328 | 0.182 | 0.987 | — | — | 2400 |
| TRUE_NODE | 224 | 47.0 | 21.5 | 36.4 | 116 | — | — | 1955 |
| MEAN_APOGEE | 417 | 265 | 172 | 260 | 329 | — | — | 1805 |
| OSCULATING_APOGEE | 976 | 154 | 159 | 3.70 | 59.7 | 74.9 | 5.5e-4 | 2389 |

Arcseconds of ecliptic longitude, the shortest way round the circle, except the last two: **latitude** is the worst in arcseconds and **distance** the worst relative difference. A node and an apogee are *directions*, carrying a longitude and nothing else, so the SDK answers them with no latitude and no distance and those two columns would compare a declared zero against whatever the engine supplies — a convention, not an error, and left blank rather than dressed up as a measurement.

### From the place — what a chart receives

| body | worst | mean | 1800 | 2100 | 2400 | latitude | distance | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| SUN | 5.42 | 1.15 | 0.145 | 0.225 | 0.524 | 1.26 | 1.7e-5 | 1844 |
| MOON | 119 | 23.9 | 0.012 | 20.3 | 92.5 | 58.2 | 5.8e-4 | 2396 |
| MERCURY | 8.35 | 1.40 | 2.25 | 0.938 | 2.06 | 2.56 | 2.2e-4 | 2350 |
| VENUS | 21.5 | 1.67 | 1.60 | 0.229 | 0.026 | 6.05 | 1.7e-4 | 2209 |
| MARS | 21.1 | 1.40 | 0.470 | 4.20 | 1.47 | 4.24 | 9.8e-5 | 2224 |
| JUPITER | 2.72 | 0.527 | 0.007 | 1.25 | 0.137 | 0.812 | 1.7e-5 | 2119 |
| SATURN | 1.66 | 0.440 | 0.831 | 0.301 | 0.908 | 0.673 | 8.9e-6 | 2302 |
| URANUS | 5.02 | 1.04 | 1.15 | 0.292 | 3.92 | 0.377 | 1.1e-5 | 2330 |
| NEPTUNE | 6.89 | 2.60 | 1.78 | 2.53 | 6.87 | 0.271 | 4.3e-6 | 2395 |
| PLUTO | 33.4 | 8.73 | 13.6 | 7.79 | 4.58 | 6.45 | 1.4e-4 | 2260 |
| MEAN_NODE | 0.987 | 0.326 | 0.328 | 0.182 | 0.987 | — | — | 2400 |
| TRUE_NODE | 224 | 47.0 | 21.5 | 36.4 | 116 | — | — | 1955 |
| MEAN_APOGEE | 417 | 265 | 172 | 260 | 329 | — | — | 1805 |
| OSCULATING_APOGEE | 3963 | 1953 | 1295 | 250 | 721 | 2771 | 1.6e-2 | 1803 |

Arcseconds of ecliptic longitude, the shortest way round the circle, except the last two: **latitude** is the worst in arcseconds and **distance** the worst relative difference. A node and an apogee are *directions*, carrying a longitude and nothing else, so the SDK answers them with no latitude and no distance and those two columns would compare a declared zero against whatever the engine supplies — a convention, not an error, and left blank rather than dressed up as a measurement.

### The rates, from the centre

| body | longitude ("/day) | latitude ("/day) |
|---|---:|---:|
| SUN | 0.186 | 0.155 |
| MOON | 9.48 | 6.79 |
| MERCURY | 3.82 | 1.88 |
| VENUS | 1.15 | 0.618 |
| MARS | 0.541 | 0.443 |
| JUPITER | 90.0 | 1.09 |
| SATURN | 2.55 | 2.27 |
| URANUS | 5.19 | 2.71 |
| NEPTUNE | 1.57 | 0.942 |
| PLUTO | 0.562 | 0.660 |
| MEAN_NODE | 0.001 | — |
| TRUE_NODE | 102 | — |
| MEAN_APOGEE | 2.39 | — |
| OSCULATING_APOGEE | 732 | 1999 |

Worst disagreement in each rate, arcseconds a day. The Moon's latitude rate matters twice over: it is the out-of-plane motion, so it fixes the orbital plane the nodes are cut from.

The Moon is the whole of the difference between the two tables: 14.6″ from the centre, 119″ from the place. Nothing about the ephemeris changed between them. Delta T did — at 2400 the two sides' models differ by enough Earth rotation to move the Moon about a hundred arcseconds, and the Moon is the one body near enough for the observer's position to matter. The trend says the same thing: 0.012″ at 1800, where Delta T is recorded, against 92.5″ at 2400, where it is extrapolated.

The geocentric Moon again as **the time it costs a boundary**, which is what a consumer of an almanac feels and what Phase 3's exit asks be published. A tithi is twelve degrees of the Moon's elongation from the Sun and is crossed at 10.670 degrees a day; a nakshatra is 13°20' of the Moon's own longitude, crossed at its sidereal 13.176 — so the same arcsecond is worth less time at a nakshatra edge.

| boundary | worst | 1800 | 2100 | 2400 |
|---|---:|---:|---:|---:|
| tithi | 32.9 s | 2.9 s | 3.7 s | 9.6 s |
| nakshatra | 26.7 s | 2.4 s | 3.0 s | 7.8 s |

The recorded **topocentric** tithi worst, for comparison, is 268 seconds — the Delta T term again, and the reason the table above is the geocentric one.

**The bija, measured rather than declared.** From the centre, the Moon is 14.6″ with it and 29.5″ without — the correction removes 14.8″ of 29.5″, fitted over 1900 to 2100. The true node moves 224″ against 224″, which is the same shift carried through and not a correction of its own.

ADR-0027 argued the theory to be 19.4″ uncorrected over this span and about 3 corrected, from a floor measurement in ELP's own frame. Reaching 29.5″ and 14.6″ again through the whole completion is an independent path to the same two numbers.

It is a knob rather than a baked-in adjustment, so what it is worth is recorded beside what it was set to — where a `bija: true` field would have asserted only that somebody meant to switch it on.

**The true node, and why it is 224″ when the Moon is 14.6″.** A node is where the orbit meets the ecliptic, and near that meeting the Moon's latitude is level: it climbs at about the tangent of the orbit's 5.145-degree inclination, so an error across the orbit arrives along it divided by that tangent — an amplification of about 11.

**106″ of it is this tier's truncation**: `full`, the theories entire, reads 118″, so the rest of what follows is about the 118″ that remain there too. It is **not the bija**: turning it off moves the node 0.136″ of 224″. It is **not the precession model**: deriving the reference pole from the SDK's own model rather than the lunar theory's moved it 0.13 arcseconds. And it is **not the node's own arithmetic** — the table below shows each provider's node sitting on its own Moon's crossing.

| provider | year | crossings | node against its own crossing |
|---|---:|---:|---:|
| built-in | 1850 | 2 | 1.96″ |
| engine | 1850 | 2 | 1.59″ |
| built-in | 2000 | 1 | 0.166″ |
| engine | 2000 | 1 | 0.798″ |
| built-in | 2350 | 1 | 5.08″ |
| engine | 2350 | 1 | 1.98″ |

A month of six-hour steps at each epoch, with the crossing interpolated rather than sampled — at six hours the Moon moves three degrees, and comparing at the nearest sample would measure the step size and call it an error.

**Each node is where its own Moon crosses.** So neither is wrong about its own theory, and what the 224″ measures is the two theories disagreeing about the *plane* rather than about the node. Read backwards, 224″ of node is 20.1″ of orbital tilt — and the Moon's position agrees far better than that, to 11.0″ of latitude. A plane is fixed by a velocity as much as by a position, and the out-of-plane rate is where the disagreement is: 6.79″ a day against a rate whose own amplitude is 4277″, which tilts a 5.145-degree orbit by 29.4″. That is the number the node is amplifying, and it accounts for the tilt the node implies.

The mean node, a polynomial that does not depend on where the Moon is, is unaffected and reads 0.987″ at every tier.

**The two apses.** The osculating apogee reads 976″ and the mean apogee 417″, where the mean *node* beside them is 0.987″. All three are the Moon's, and two of the three are polynomials from one theory, so the gap is not the Moon.

The **osculating** apogee is the one built from a mass rather than from a theory — an osculating element is the two-body orbit matching a position and a velocity, and neither VSOP87 nor ELP2000-82B carries a mass (ADR-0008 refused this body until the constant was sourced). It is also the apse the Sun swings hardest: it wanders by tens of degrees over a month, so 976″ of disagreement is a small fraction of its own motion, and it is bounded by the same lunar velocity the true node is. Unlike the mean apogee it **does** answer to the tier, reading 976″ here against 561″ at `full`, because it is built from where the Moon is rather than from a polynomial.

The **mean** apogee is the one to explain, and this page does not. It is ELP's `W2` polynomial with half a turn added, exactly as the mean node is `W3`, and it is 422 times worse than that node. Its mean of 265″ against its worst of 417″ says a standing offset with an oscillation on top rather than a drift. The candidates are that the engine's mean apogee is not `W2 + 180°` but a series of its own, and that one of the two carries periodic terms the other does not; nothing here distinguishes them, and a page that picked one would be guessing. What can be said is what it is **not**: not the tier, since `full` reads 417″ for the same figure, which is what a polynomial untouched by truncation looks like; and not the Moon, whose own longitude agrees to 14.6″.

Their **latitudes** are a convention and not an error. This SDK answers a mean apogee as a *direction* on the ecliptic, latitude zero by construction; the engine places it on the Moon's own orbital plane, where it reaches the orbit's 5.145-degree inclination. The column reports the difference rather than hiding it, and the two are answering different questions rather than one of them answering it wrongly.

## `standard`

**The two sides are not one pipeline, and cannot be made one.** The engine answers the whole frame in a single native call — its step list is `positions:Native` and nothing else — while the built-in ephemeris is completed by the SDK's own steps. Forcing the SDK's everywhere was tried and is refused: the engine's native positions are *apparent*, the SDK can add corrections and never remove them, so a geometric frame comes back `Unsupported { step: "corrections" }`.

So both centres are recorded, and the pair is the point. **Geocentric isolates the ephemeris**: the instants are TT, so nothing in that path needs Delta T. **Topocentric is what a chart receives**, and it carries the two sides' Delta T models as well, through the observer's own rotation.

### From the Earth's centre — the ephemeris

| body | worst | mean | 1800 | 2100 | 2400 | latitude | distance | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| SUN | 0.228 | 0.075 | 0.156 | 0.066 | 0.068 | 0.058 | 3.9e-7 | 1830 |
| MOON | 2.75 | 0.336 | 0.151 | 0.280 | 1.73 | 1.75 | 1.0e-4 | 2389 |
| MERCURY | 0.285 | 0.078 | 0.187 | 0.081 | 0.040 | 0.125 | 2.0e-4 | 1833 |
| VENUS | 0.787 | 0.081 | 0.121 | 0.069 | 0.052 | 0.250 | 1.2e-4 | 2310 |
| MARS | 3.98 | 0.246 | 0.026 | 0.142 | 0.514 | 0.300 | 6.6e-5 | 2399 |
| JUPITER | 1.04 | 0.401 | 0.269 | 0.408 | 0.604 | 0.125 | 1.1e-5 | 2365 |
| SATURN | 0.834 | 0.327 | 0.170 | 0.339 | 0.696 | 0.060 | 5.5e-6 | 2399 |
| URANUS | 4.97 | 1.04 | 0.761 | 0.057 | 3.91 | 0.081 | 1.1e-5 | 2332 |
| NEPTUNE | 6.76 | 2.58 | 2.00 | 2.31 | 6.75 | 0.176 | 4.0e-6 | 2399 |
| PLUTO | 0.926 | 0.210 | 0.024 | 0.004 | 0.088 | 0.186 | 8.1e-6 | 1987 |
| MEAN_NODE | 0.987 | 0.326 | 0.328 | 0.182 | 0.987 | — | — | 2400 |
| TRUE_NODE | 119 | 38.6 | 68.2 | 8.35 | 65.0 | — | — | 2392 |
| MEAN_APOGEE | 417 | 265 | 172 | 260 | 329 | — | — | 1805 |
| OSCULATING_APOGEE | 565 | 107 | 131 | 48.8 | 28.6 | 50.4 | 1.8e-4 | 2016 |

Arcseconds of ecliptic longitude, the shortest way round the circle, except the last two: **latitude** is the worst in arcseconds and **distance** the worst relative difference. A node and an apogee are *directions*, carrying a longitude and nothing else, so the SDK answers them with no latitude and no distance and those two columns would compare a declared zero against whatever the engine supplies — a convention, not an error, and left blank rather than dressed up as a measurement.

### From the place — what a chart receives

| body | worst | mean | 1800 | 2100 | 2400 | latitude | distance | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| SUN | 0.201 | 0.061 | 0.137 | 0.047 | 0.076 | 0.148 | 1.6e-6 | 1830 |
| MOON | 117 | 23.1 | 1.44 | 18.3 | 95.1 | 53.9 | 5.7e-4 | 2395 |
| MERCURY | 0.334 | 0.076 | 0.228 | 0.061 | 0.007 | 0.164 | 2.0e-4 | 1856 |
| VENUS | 0.770 | 0.112 | 0.301 | 0.079 | 0.070 | 0.451 | 1.2e-4 | 2310 |
| MARS | 4.06 | 0.315 | 0.165 | 0.307 | 0.472 | 0.305 | 6.6e-5 | 2367 |
| JUPITER | 1.13 | 0.429 | 0.325 | 0.633 | 0.470 | 0.358 | 1.1e-5 | 2364 |
| SATURN | 1.03 | 0.358 | 0.031 | 0.556 | 0.627 | 0.155 | 5.5e-6 | 2399 |
| URANUS | 5.10 | 1.06 | 1.02 | 0.177 | 3.95 | 0.184 | 1.1e-5 | 2329 |
| NEPTUNE | 6.91 | 2.59 | 1.80 | 2.57 | 6.91 | 0.275 | 4.0e-6 | 2398 |
| PLUTO | 1.20 | 0.276 | 0.194 | 0.205 | 0.001 | 0.294 | 8.1e-6 | 1987 |
| MEAN_NODE | 0.987 | 0.326 | 0.328 | 0.182 | 0.987 | — | — | 2400 |
| TRUE_NODE | 119 | 38.6 | 68.2 | 8.35 | 65.0 | — | — | 2392 |
| MEAN_APOGEE | 417 | 265 | 172 | 260 | 329 | — | — | 1805 |
| OSCULATING_APOGEE | 3575 | 1953 | 1323 | 204 | 752 | 2766 | 1.6e-2 | 2108 |

Arcseconds of ecliptic longitude, the shortest way round the circle, except the last two: **latitude** is the worst in arcseconds and **distance** the worst relative difference. A node and an apogee are *directions*, carrying a longitude and nothing else, so the SDK answers them with no latitude and no distance and those two columns would compare a declared zero against whatever the engine supplies — a convention, not an error, and left blank rather than dressed up as a measurement.

### The rates, from the centre

| body | longitude ("/day) | latitude ("/day) |
|---|---:|---:|
| SUN | 0.014 | 0.004 |
| MOON | 2.94 | 2.36 |
| MERCURY | 3.23 | 2.00 |
| VENUS | 0.328 | 0.095 |
| MARS | 0.520 | 0.474 |
| JUPITER | 90.0 | 1.07 |
| SATURN | 2.56 | 2.28 |
| URANUS | 5.19 | 2.70 |
| NEPTUNE | 1.57 | 0.945 |
| PLUTO | 0.502 | 0.660 |
| MEAN_NODE | 0.001 | — |
| TRUE_NODE | 30.5 | — |
| MEAN_APOGEE | 2.39 | — |
| OSCULATING_APOGEE | 243 | 1999 |

Worst disagreement in each rate, arcseconds a day. The Moon's latitude rate matters twice over: it is the out-of-plane motion, so it fixes the orbital plane the nodes are cut from.

The Moon is the whole of the difference between the two tables: 2.75″ from the centre, 117″ from the place. Nothing about the ephemeris changed between them. Delta T did — at 2400 the two sides' models differ by enough Earth rotation to move the Moon about a hundred arcseconds, and the Moon is the one body near enough for the observer's position to matter. The trend says the same thing: 1.44″ at 1800, where Delta T is recorded, against 95.1″ at 2400, where it is extrapolated.

The geocentric Moon again as **the time it costs a boundary**, which is what a consumer of an almanac feels and what Phase 3's exit asks be published. A tithi is twelve degrees of the Moon's elongation from the Sun and is crossed at 10.670 degrees a day; a nakshatra is 13°20' of the Moon's own longitude, crossed at its sidereal 13.176 — so the same arcsecond is worth less time at a nakshatra edge.

| boundary | worst | 1800 | 2100 | 2400 |
|---|---:|---:|---:|---:|
| tithi | 6.2 s | 0.3 s | 0.6 s | 3.9 s |
| nakshatra | 5.0 s | 0.3 s | 0.5 s | 3.1 s |

The recorded **topocentric** tithi worst, for comparison, is 262 seconds — the Delta T term again, and the reason the table above is the geocentric one.

**The bija, measured rather than declared.** From the centre, the Moon is 2.75″ with it and 19.2″ without — the correction removes 16.4″ of 19.2″, fitted over 1900 to 2100. The true node moves 119″ against 105″, which is the same shift carried through and not a correction of its own.

ADR-0027 argued the theory to be 19.4″ uncorrected over this span and about 3 corrected, from a floor measurement in ELP's own frame. Reaching 19.2″ and 2.75″ again through the whole completion is an independent path to the same two numbers.

It is a knob rather than a baked-in adjustment, so what it is worth is recorded beside what it was set to — where a `bija: true` field would have asserted only that somebody meant to switch it on.

**The true node, and why it is 119″ when the Moon is 2.75″.** A node is where the orbit meets the ecliptic, and near that meeting the Moon's latitude is level: it climbs at about the tangent of the orbit's 5.145-degree inclination, so an error across the orbit arrives along it divided by that tangent — an amplification of about 11.

It is **not truncation**: `full`, the theories entire, reads 118″ for the same figure. It is **not the bija**: turning it off moves the node 13.3″ of 119″. It is **not the precession model**: deriving the reference pole from the SDK's own model rather than the lunar theory's moved it 0.13 arcseconds. And it is **not the node's own arithmetic** — the table below shows each provider's node sitting on its own Moon's crossing.

| provider | year | crossings | node against its own crossing |
|---|---:|---:|---:|
| built-in | 1850 | 2 | 2.17″ |
| engine | 1850 | 2 | 1.59″ |
| built-in | 2000 | 1 | 0.189″ |
| engine | 2000 | 1 | 0.798″ |
| built-in | 2350 | 1 | 5.19″ |
| engine | 2350 | 1 | 1.98″ |

A month of six-hour steps at each epoch, with the crossing interpolated rather than sampled — at six hours the Moon moves three degrees, and comparing at the nearest sample would measure the step size and call it an error.

**Each node is where its own Moon crosses.** So neither is wrong about its own theory, and what the 119″ measures is the two theories disagreeing about the *plane* rather than about the node. Read backwards, 119″ of node is 10.6″ of orbital tilt — and the Moon's position agrees far better than that, to 1.75″ of latitude. A plane is fixed by a velocity as much as by a position, and the out-of-plane rate is where the disagreement is: 2.36″ a day against a rate whose own amplitude is 4277″, which tilts a 5.145-degree orbit by 10.2″. That is the number the node is amplifying, and it accounts for the tilt the node implies.

The mean node, a polynomial that does not depend on where the Moon is, is unaffected and reads 0.987″ at every tier.

**The two apses.** The osculating apogee reads 565″ and the mean apogee 417″, where the mean *node* beside them is 0.987″. All three are the Moon's, and two of the three are polynomials from one theory, so the gap is not the Moon.

The **osculating** apogee is the one built from a mass rather than from a theory — an osculating element is the two-body orbit matching a position and a velocity, and neither VSOP87 nor ELP2000-82B carries a mass (ADR-0008 refused this body until the constant was sourced). It is also the apse the Sun swings hardest: it wanders by tens of degrees over a month, so 565″ of disagreement is a small fraction of its own motion, and it is bounded by the same lunar velocity the true node is. Unlike the mean apogee it **does** answer to the tier, reading 565″ here against 561″ at `full`, because it is built from where the Moon is rather than from a polynomial.

The **mean** apogee is the one to explain, and this page does not. It is ELP's `W2` polynomial with half a turn added, exactly as the mean node is `W3`, and it is 422 times worse than that node. Its mean of 265″ against its worst of 417″ says a standing offset with an oscillation on top rather than a drift. The candidates are that the engine's mean apogee is not `W2 + 180°` but a series of its own, and that one of the two carries periodic terms the other does not; nothing here distinguishes them, and a page that picked one would be guessing. What can be said is what it is **not**: not the tier, since `full` reads 417″ for the same figure, which is what a polynomial untouched by truncation looks like; and not the Moon, whose own longitude agrees to 2.75″.

Their **latitudes** are a convention and not an error. This SDK answers a mean apogee as a *direction* on the ecliptic, latitude zero by construction; the engine places it on the Moon's own orbital plane, where it reaches the orbit's 5.145-degree inclination. The column reports the difference rather than hiding it, and the two are answering different questions rather than one of them answering it wrongly.

## `full`

**The two sides are not one pipeline, and cannot be made one.** The engine answers the whole frame in a single native call — its step list is `positions:Native` and nothing else — while the built-in ephemeris is completed by the SDK's own steps. Forcing the SDK's everywhere was tried and is refused: the engine's native positions are *apparent*, the SDK can add corrections and never remove them, so a geometric frame comes back `Unsupported { step: "corrections" }`.

So both centres are recorded, and the pair is the point. **Geocentric isolates the ephemeris**: the instants are TT, so nothing in that path needs Delta T. **Topocentric is what a chart receives**, and it carries the two sides' Delta T models as well, through the observer's own rotation.

### From the Earth's centre — the ephemeris

| body | worst | mean | 1800 | 2100 | 2400 | latitude | distance | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| SUN | 0.152 | 0.076 | 0.151 | 0.056 | 0.071 | 0.015 | 7.8e-8 | 1814 |
| MOON | 2.47 | 0.317 | 0.141 | 0.156 | 1.84 | 1.58 | 1.1e-4 | 2396 |
| MERCURY | 0.207 | 0.077 | 0.192 | 0.069 | 0.059 | 0.021 | 2.0e-4 | 1803 |
| VENUS | 0.233 | 0.075 | 0.145 | 0.058 | 0.053 | 0.071 | 1.2e-4 | 2393 |
| MARS | 4.02 | 0.245 | 0.048 | 0.079 | 0.509 | 0.199 | 6.6e-5 | 2399 |
| JUPITER | 1.02 | 0.397 | 0.282 | 0.395 | 0.608 | 0.124 | 1.1e-5 | 2365 |
| SATURN | 0.842 | 0.327 | 0.169 | 0.338 | 0.701 | 0.055 | 5.5e-6 | 2397 |
| URANUS | 4.97 | 1.04 | 0.764 | 0.058 | 3.91 | 0.081 | 1.1e-5 | 2332 |
| NEPTUNE | 6.77 | 2.58 | 2.00 | 2.31 | 6.76 | 0.176 | 4.0e-6 | 2399 |
| PLUTO | 0.006 | 0.001 | 0.003 | 0.000 | 0.002 | 0.001 | 4.5e-6 | 1987 |
| MEAN_NODE | 0.987 | 0.326 | 0.328 | 0.182 | 0.987 | — | — | 2400 |
| TRUE_NODE | 118 | 38.6 | 67.6 | 8.83 | 64.3 | — | — | 2392 |
| MEAN_APOGEE | 417 | 265 | 172 | 260 | 329 | — | — | 1805 |
| OSCULATING_APOGEE | 561 | 107 | 126 | 45.8 | 29.7 | 50.1 | 1.9e-4 | 2016 |

Arcseconds of ecliptic longitude, the shortest way round the circle, except the last two: **latitude** is the worst in arcseconds and **distance** the worst relative difference. A node and an apogee are *directions*, carrying a longitude and nothing else, so the SDK answers them with no latitude and no distance and those two columns would compare a declared zero against whatever the engine supplies — a convention, not an error, and left blank rather than dressed up as a measurement.

### From the place — what a chart receives

| body | worst | mean | 1800 | 2100 | 2400 | latitude | distance | worst at |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| SUN | 0.137 | 0.061 | 0.131 | 0.038 | 0.079 | 0.120 | 1.5e-6 | 1801 |
| MOON | 117 | 23.1 | 1.43 | 18.4 | 94.9 | 53.8 | 5.7e-4 | 2395 |
| MERCURY | 0.251 | 0.071 | 0.233 | 0.049 | 0.025 | 0.124 | 2.0e-4 | 1803 |
| VENUS | 0.544 | 0.103 | 0.325 | 0.090 | 0.071 | 0.383 | 1.2e-4 | 2399 |
| MARS | 4.05 | 0.313 | 0.187 | 0.244 | 0.467 | 0.306 | 6.6e-5 | 2399 |
| JUPITER | 1.12 | 0.426 | 0.337 | 0.619 | 0.473 | 0.357 | 1.1e-5 | 2364 |
| SATURN | 1.04 | 0.358 | 0.031 | 0.555 | 0.632 | 0.152 | 5.5e-6 | 2399 |
| URANUS | 5.09 | 1.06 | 1.02 | 0.176 | 3.95 | 0.184 | 1.1e-5 | 2329 |
| NEPTUNE | 6.91 | 2.59 | 1.80 | 2.57 | 6.91 | 0.276 | 4.0e-6 | 2400 |
| PLUTO | 0.289 | 0.176 | 0.215 | 0.209 | 0.085 | 0.147 | 4.5e-6 | 2004 |
| MEAN_NODE | 0.987 | 0.326 | 0.328 | 0.182 | 0.987 | — | — | 2400 |
| TRUE_NODE | 118 | 38.6 | 67.6 | 8.83 | 64.3 | — | — | 2392 |
| MEAN_APOGEE | 417 | 265 | 172 | 260 | 329 | — | — | 1805 |
| OSCULATING_APOGEE | 3571 | 1953 | 1328 | 207 | 751 | 2766 | 1.6e-2 | 2108 |

Arcseconds of ecliptic longitude, the shortest way round the circle, except the last two: **latitude** is the worst in arcseconds and **distance** the worst relative difference. A node and an apogee are *directions*, carrying a longitude and nothing else, so the SDK answers them with no latitude and no distance and those two columns would compare a declared zero against whatever the engine supplies — a convention, not an error, and left blank rather than dressed up as a measurement.

### The rates, from the centre

| body | longitude ("/day) | latitude ("/day) |
|---|---:|---:|
| SUN | 0.007 | 0.001 |
| MOON | 2.93 | 2.35 |
| MERCURY | 3.22 | 2.00 |
| VENUS | 0.311 | 0.091 |
| MARS | 0.520 | 0.476 |
| JUPITER | 90.0 | 1.07 |
| SATURN | 2.56 | 2.28 |
| URANUS | 5.19 | 2.70 |
| NEPTUNE | 1.57 | 0.945 |
| PLUTO | 0.502 | 0.660 |
| MEAN_NODE | 0.001 | — |
| TRUE_NODE | 26.5 | — |
| MEAN_APOGEE | 2.39 | — |
| OSCULATING_APOGEE | 238 | 1999 |

Worst disagreement in each rate, arcseconds a day. The Moon's latitude rate matters twice over: it is the out-of-plane motion, so it fixes the orbital plane the nodes are cut from.

The Moon is the whole of the difference between the two tables: 2.47″ from the centre, 117″ from the place. Nothing about the ephemeris changed between them. Delta T did — at 2400 the two sides' models differ by enough Earth rotation to move the Moon about a hundred arcseconds, and the Moon is the one body near enough for the observer's position to matter. The trend says the same thing: 1.43″ at 1800, where Delta T is recorded, against 94.9″ at 2400, where it is extrapolated.

The geocentric Moon again as **the time it costs a boundary**, which is what a consumer of an almanac feels and what Phase 3's exit asks be published. A tithi is twelve degrees of the Moon's elongation from the Sun and is crossed at 10.670 degrees a day; a nakshatra is 13°20' of the Moon's own longitude, crossed at its sidereal 13.176 — so the same arcsecond is worth less time at a nakshatra edge.

| boundary | worst | 1800 | 2100 | 2400 |
|---|---:|---:|---:|---:|
| tithi | 5.6 s | 0.3 s | 0.4 s | 4.1 s |
| nakshatra | 4.5 s | 0.3 s | 0.3 s | 3.3 s |

The recorded **topocentric** tithi worst, for comparison, is 263 seconds — the Delta T term again, and the reason the table above is the geocentric one.

**The bija, measured rather than declared.** From the centre, the Moon is 2.47″ with it and 19.3″ without — the correction removes 16.8″ of 19.3″, fitted over 1900 to 2100. The true node moves 118″ against 105″, which is the same shift carried through and not a correction of its own.

ADR-0027 argued the theory to be 19.4″ uncorrected over this span and about 3 corrected, from a floor measurement in ELP's own frame. Reaching 19.3″ and 2.47″ again through the whole completion is an independent path to the same two numbers.

It is a knob rather than a baked-in adjustment, so what it is worth is recorded beside what it was set to — where a `bija: true` field would have asserted only that somebody meant to switch it on.

**The true node, and why it is 118″ when the Moon is 2.47″.** A node is where the orbit meets the ecliptic, and near that meeting the Moon's latitude is level: it climbs at about the tangent of the orbit's 5.145-degree inclination, so an error across the orbit arrives along it divided by that tangent — an amplification of about 11.

It is **not truncation**: `full`, the theories entire, reads 118″ for the same figure. It is **not the bija**: turning it off moves the node 13.3″ of 118″. It is **not the precession model**: deriving the reference pole from the SDK's own model rather than the lunar theory's moved it 0.13 arcseconds. And it is **not the node's own arithmetic** — the table below shows each provider's node sitting on its own Moon's crossing.

| provider | year | crossings | node against its own crossing |
|---|---:|---:|---:|
| built-in | 1850 | 2 | 2.19″ |
| engine | 1850 | 2 | 1.59″ |
| built-in | 2000 | 1 | 0.180″ |
| engine | 2000 | 1 | 0.798″ |
| built-in | 2350 | 1 | 5.16″ |
| engine | 2350 | 1 | 1.98″ |

A month of six-hour steps at each epoch, with the crossing interpolated rather than sampled — at six hours the Moon moves three degrees, and comparing at the nearest sample would measure the step size and call it an error.

**Each node is where its own Moon crosses.** So neither is wrong about its own theory, and what the 118″ measures is the two theories disagreeing about the *plane* rather than about the node. Read backwards, 118″ of node is 10.6″ of orbital tilt — and the Moon's position agrees far better than that, to 1.58″ of latitude. A plane is fixed by a velocity as much as by a position, and the out-of-plane rate is where the disagreement is: 2.35″ a day against a rate whose own amplitude is 4277″, which tilts a 5.145-degree orbit by 10.2″. That is the number the node is amplifying, and it accounts for the tilt the node implies.

The mean node, a polynomial that does not depend on where the Moon is, is unaffected and reads 0.987″ at every tier.

**The two apses.** The osculating apogee reads 561″ and the mean apogee 417″, where the mean *node* beside them is 0.987″. All three are the Moon's, and two of the three are polynomials from one theory, so the gap is not the Moon.

The **osculating** apogee is the one built from a mass rather than from a theory — an osculating element is the two-body orbit matching a position and a velocity, and neither VSOP87 nor ELP2000-82B carries a mass (ADR-0008 refused this body until the constant was sourced). It is also the apse the Sun swings hardest: it wanders by tens of degrees over a month, so 561″ of disagreement is a small fraction of its own motion, and it is bounded by the same lunar velocity the true node is. Unlike the mean apogee it **does** answer to the tier, reading 561″ here against 561″ at `full`, because it is built from where the Moon is rather than from a polynomial.

The **mean** apogee is the one to explain, and this page does not. It is ELP's `W2` polynomial with half a turn added, exactly as the mean node is `W3`, and it is 422 times worse than that node. Its mean of 265″ against its worst of 417″ says a standing offset with an oscillation on top rather than a drift. The candidates are that the engine's mean apogee is not `W2 + 180°` but a series of its own, and that one of the two carries periodic terms the other does not; nothing here distinguishes them, and a page that picked one would be guessing. What can be said is what it is **not**: not the tier, since `full` reads 417″ for the same figure, which is what a polynomial untouched by truncation looks like; and not the Moon, whose own longitude agrees to 2.47″.

Their **latitudes** are a convention and not an error. This SDK answers a mean apogee as a *direction* on the ecliptic, latitude zero by construction; the engine places it on the Moon's own orbital plane, where it reaches the orbit's 5.145-degree inclination. The column reports the difference rather than hiding it, and the two are answering different questions rather than one of them answering it wrongly.

## One correction at a time

A disagreement in the apparent frame is five corrections at once, and a table of it names none of them. This ladder asks for one correction at a time, at `standard`, so a number can be attributed.

The first column is the check that makes the rest mean anything: **how far the engine's own answer moves** when only that correction is asked for. A flag the engine does not honour leaves it at zero, and a comparison against a correction the engine did not apply says nothing at all.

| correction | engine moves the Sun | Sun | Moon | Mars |
|---|---:|---:|---:|---:|
| geometric (of date) | 0.000 | 0.227 | 2.75 | 3.98 |
| light time only | 0.011 | 0.227 | 2.75 | 3.98 |
| deflection only | 0.000 | 0.228 | 2.75 | 3.98 |
| aberration only | 0.000 | 20.9 | 23.3 | 21.3 |
| nutation only | 18.9 | 0.225 | 2.74 | 3.97 |
| apparent, tropical | 39.6 | 0.226 | 2.74 | 3.97 |
| apparent, sidereal | 106035 | 0.228 | 2.75 | 3.98 |

Of the four single corrections the engine distinguishes `light time` and `nutation` — asking for `deflection` and `aberration` moves its own answer not at all. So those rows do not measure agreement: they measure the SDK applying a correction the engine did not, and are here to be read that way rather than mistaken for accuracy. **Only the rows the engine honours, and the two whole-frame rows at the foot, compare like with like.**

The last row runs the whole completion: `positions:Native`, `equinox:Sdk`, `obliquity:Sdk`, `corrections:Sdk`, `ayanamsha:Sdk`, `zodiac-shift:Sdk`. Each names who did the work — `Native` is the provider's own answer, `Sdk` is this crate's.

## The claims this decides

Each bound is one ADR-0027 argued from, not one chosen to be met, and each is decided on the **geocentric** table — the one that isolates the ephemeris. Deciding them on the topocentric table would be charging the lunar theory for a Delta T model six centuries out. A falsified row is the measurement doing its job: it says the tier cannot carry that claim, and the tier's published figure becomes the measured one.

| proposed rule | verdict | measured |
|---|---|---|
| `compact`: the retired single-number claim — one arcsecond for every planet | falsified | worst 21.4″ |
| `compact`: the Sun stays inside one arcsecond over the whole span | falsified | worst 5.45″ |
| `compact`: second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027) | falsified | worst 3.67 s of tithi |
| `compact`: minute-accurate over the whole six centuries (ADR-0027) | **holds** | worst 33 s of tithi |
| `compact`: the Moon meets the almanac's 0.445 arcseconds | falsified | worst 14.6″ |
| `compact`: the mean node is inside one arcsecond | **holds** | worst 0.987″ |
| `standard`: the retired single-number claim — one arcsecond for every planet | falsified | worst 6.76″ |
| `standard`: the Sun stays inside one arcsecond over the whole span | **holds** | worst 0.228″ |
| `standard`: second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027) | **holds** | worst 0.63 s of tithi |
| `standard`: minute-accurate over the whole six centuries (ADR-0027) | **holds** | worst 6.18 s of tithi |
| `standard`: the Moon meets the almanac's 0.445 arcseconds | falsified | worst 2.75″ |
| `standard`: the mean node is inside one arcsecond | **holds** | worst 0.987″ |
| `full`: the retired single-number claim — one arcsecond for every planet | falsified | worst 6.77″ |
| `full`: the Sun stays inside one arcsecond over the whole span | **holds** | worst 0.152″ |
| `full`: second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027) | **holds** | worst 0.35 s of tithi |
| `full`: minute-accurate over the whole six centuries (ADR-0027) | **holds** | worst 5.55 s of tithi |
| `full`: the Moon meets the almanac's 0.445 arcseconds | falsified | worst 2.47″ |
| `full`: the mean node is inside one arcsecond | **holds** | worst 0.987″ |

8 of 18 falsified:

- `compact`: the retired single-number claim — one arcsecond for every planet — worst 21.4″
- `compact`: the Sun stays inside one arcsecond over the whole span — worst 5.45″
- `compact`: second-accurate inside the bija's fitted span, 1900 to 2100 (ADR-0027) — worst 3.67 s of tithi
- `compact`: the Moon meets the almanac's 0.445 arcseconds — worst 14.6″
- `standard`: the retired single-number claim — one arcsecond for every planet — worst 6.76″
- `standard`: the Moon meets the almanac's 0.445 arcseconds — worst 2.75″
- `full`: the retired single-number claim — one arcsecond for every planet — worst 6.77″
- `full`: the Moon meets the almanac's 0.445 arcseconds — worst 2.47″

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

