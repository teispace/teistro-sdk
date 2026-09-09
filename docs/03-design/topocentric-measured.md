# The topocentric centre, measured

Status: `generated` by `cargo xtask topocentric` over the 6 charts the
conformance corpus records from both centres. Do not edit:
`check-topocentric` regenerates this page and fails on any difference.
The design written from it is
[`astro-timescales-and-frames.md`](astro-timescales-and-frames.md) §4.

## 1. What the corpus records

6 charts are recorded twice — once from the centre of the Earth
and once from the place they were cast for — with every other
setting equal: `c001-kathmandu-1990-04-14`, `c041-la-paz-2010-07-10`, `c049-kathmandu-2010-06-01`, `c050-kathmandu-2015-03-01`, `c052-kathmandu-2020-04-13`, `c055-kathmandu-2024-04-09`. Each pair is an input and its answer, so a
proposed reading of the step reproduces the recorded numbers or it
does not, and nothing here is a comparison inside a tolerance of
this pass's own choosing. The bound below is the corpus's own for a
run against the ephemeris the fixtures came from
(`fixtures/tolerances.json`, `same-ephemeris`): a millionth of a
degree of longitude, which is 0.003600″.

Every other fixture in the corpus is recorded from a place only, so none
of them can decide the step; one of their fields can still falsify part
of it, and §4 uses all 174 rows of it.

## 2. Nine readings of one step

`astro-timescales-and-frames.md` §4 already describes the step: "the
observer's geocentric position (WGS84) and the parallax". A displacement
is what a topocentric position obviously is — the body is seen from a
point some six thousand kilometres off the centre — and the first
reading below is exactly that. Each of the next three adds one term; the
rest take one choice back out of the reading those three settle on, to
check it rather than assume it.

| reading | worst longitude | worst latitude | worst at |
|---|---:|---:|---|
| the observer's displacement alone, which §4 designs | 0.458196″ | 0.302981″ | c052-kathmandu-2020-04-13 MOON |
| and the station's own aberration | 0.289597″ | 0.237426″ | c052-kathmandu-2020-04-13 MOON |
| and on the direction the light came from | 0.360454″ | 0.085770″ | c055-kathmandu-2024-04-09 MOON |
| and carrying the body over the light time the station saves | 0.079939″ | 0.083939″ | c055-kathmandu-2024-04-09 MOON |
| the light time without taking the Earth's aberration off first | 0.264918″ | 0.253683″ | c050-kathmandu-2015-03-01 MOON |
| the settled reading referred to the mean obliquity | 0.059664″ | 0.083180″ | c052-kathmandu-2020-04-13 MOON |
| the settled reading with the station where the light left it | 0.338995″ | 0.183533″ | c055-kathmandu-2024-04-09 MOON |
| the settled reading on a sphere of the equatorial radius | 3.879496″ | 8.964201″ | c055-kathmandu-2024-04-09 MOON |
| the settled reading at sea level rather than the place's height | 0.749155″ | 1.266974″ | c041-la-paz-2010-07-10 MOON |

The designed reading is falsified, and the per-body table in §6 says
why in one column: the displacement alone leaves about a third of an
arcsecond on **every** body — on Saturn, whose whole parallax is under
an arcsecond, as much as on the Moon, whose parallax is forty
arcminutes. A residual that does not shrink with distance is not a
displacement gone wrong; it is a rotation the reading has left out. The
station is moving, four hundred metres a second eastward at these
latitudes, and the light it receives arrives from a direction aberrated
by its own velocity: one and a half parts in a million, a third of an
arcsecond, whatever the body's distance. With it the planets come inside
a thousandth of an arcsecond and stay there for every reading below, so
from here on the Moon is the only witness.

It takes two more terms, and neither is a displacement either. The
recorded position is **apparent**: the Earth's own motion has already
turned it by twenty arcseconds, and displacing a direction that has been
turned displaces it in the wrong direction by that much of itself. And
the station stands nearer the body than the centre does, by up to an
Earth radius, so the light it sees left the body later — by up to a
fiftieth of a second, in which the Moon travels six hundred metres of
its barycentric path. Each is worth about a third of an arcsecond on the
Moon and neither is worth anything on anything else; put in one at a
time they make the answer worse, and put in together they take it from
0.458196″ to 0.079939″.

What the Moon says, pair by pair and signed, so that a reading which is
right on average and wrong everywhere cannot hide behind a worst case.
The first row is the size of the step itself; every row under it is what
the reading failed to account for:

| reading | c001 | c041 | c049 | c050 | c052 | c055 |
|---|---:|---:|---:|---:|---:|---:|
| *the step itself* | -23.83′ | 20.63′ | 23.71′ | 21.73′ | 39.09′ | -9.58′ |
| displacement | -0.327308″ | -0.294328″ | 0.273615″ | 0.147721″ | 0.458196″ | 0.194594″ |
| + aberration | -0.100487″ | -0.008197″ | 0.056922″ | -0.113660″ | 0.289597″ | -0.068196″ |
| + geometric | -0.098719″ | 0.191578″ | 0.103869″ | 0.127935″ | 0.124306″ | -0.360454″ |
| + light time | 0.000833″ | -0.034795″ | -0.061963″ | -0.023317″ | 0.059707″ | -0.079939″ |
| light time alone | -0.000926″ | -0.234581″ | -0.108916″ | -0.264918″ | 0.224995″ | 0.212325″ |
| mean obliquity | -0.032528″ | -0.034343″ | -0.057208″ | 0.010293″ | 0.059664″ | 0.005773″ |
| retarded station | 0.229084″ | 0.254785″ | -0.275815″ | -0.281976″ | -0.107003″ | -0.338995″ |
| a sphere | 2.727764″ | -0.456465″ | -3.319504″ | 0.474296″ | -2.270745″ | -3.879496″ |
| sea level | 0.316350″ | -0.749155″ | -0.369264″ | -0.306823″ | -0.448772″ | 0.045297″ |

## 3. The Earth the observer stands on

Two more choices are the kind a reader would call harmless, and
the same six pairs falsify both. Standing the observer on a
sphere of the Earth's equatorial radius rather than on the WGS84
ellipsoid moves them by up to twenty-one kilometres and leaves
3.879496″; standing them at sea level rather than at the place's own
height — 1 400 m at Kathmandu, 3 640 m at La Paz — leaves 0.749155″.
Against a bound of 0.003600″ both are decisive, and the second is the
one worth naming: a field a consumer may not bother to fill in
is worth two hundred times the tolerance the corpus is compared
at.

## 4. A direction is not a place

The lunar nodes are recorded in every one of these charts, and in
the pairs they are the same numbers under both centres — not close,
**identical**, to the last bit of a double, in longitude, latitude
and distance alike. Their recorded distance is the same constant in
every fixture in the corpus, 0.002 569 555 astronomical units, which is
384 400 km: a nominal figure and not a measurement, and the first sign
that what is recorded is a direction rather than a place.

The fixtures recorded from a place only cannot compare the two centres,
but they can still falsify this: a node displaced by an observer would
leave the ecliptic by up to 43.67′, and the node's recorded latitude
is zero to the last bits of a double in every one of them, the true node
included. So the rule is about what a point **is**, and not about which
of them the corpus happened to record: a point defined as a direction on
the Moon's orbit is not anywhere, and no observer sees it displaced.

## 5. The speed, which this corpus cannot decide

The step changes a speed far more than it changes a position: the Moon's
longitude speed differs between the two centres by up to 5.485°, a
third of its own value, because the observer is carried eastward at
close to a twentieth of the Moon's own rate. A step that moved positions
and left speeds alone would be wrong by more than the positions it
corrected.

The transform is the position's: subtract the station's velocity from
the body's and read the longitude rate off the difference. It cannot be
**decided** here, because the corpus records a longitude speed and
neither a latitude speed nor a distance speed, and both enter the
answer. Setting the two it does not record to nought reproduces the
recorded topocentric speed to 43.126″, which is the size of what is
missing rather than of the rule. What settles it is a provider that
answers both centres natively — the Teimeris adapter declares the
topocentric override — compared over a grid, which is
`05-testing/ACCURACY.md`'s business and not this page's.

## 6. What the step is worth

| body | worst longitude | worst latitude | worst speed | displacement alone leaves | the settled reading leaves | signs | nakshatras | padas |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| JUPITER | 1.222″ | 1.467″ | 10.617″ | 0.279626″ | 0.000040″ | 0 | 0 | 0 |
| KETU | 0.000″ | 0.000″ | 0.001″ | 2545.158010″ | 2544.775750″ | 0 | 0 | 0 |
| MARS | 3.592″ | 4.175″ | 32.401″ | 0.258054″ | 0.000132″ | 0 | 0 | 0 |
| MERCURY | 8.809″ | 8.533″ | 1.25′ | 0.273179″ | 0.000313″ | 0 | 0 | 0 |
| MOON | 39.09′ | 43.67′ | 5.485° | 0.458196″ | 0.079939″ | 0 | 1 | 2 |
| RAHU | 0.000″ | 0.000″ | 0.001″ | 2596.967441″ | 2597.068133″ | 0 | 0 | 0 |
| SATURN | 0.659″ | 0.700″ | 4.950″ | 0.275566″ | 0.000018″ | 0 | 0 | 0 |
| SUN | 8.125″ | 6.752″ | 45.336″ | 0.263311″ | 0.000251″ | 0 | 0 | 0 |
| VENUS | 14.774″ | 8.489″ | 33.388″ | 0.249445″ | 0.000277″ | 0 | 0 | 0 |

The two columns on the right are the argument of §2 in one place. The
displacement alone leaves a quarter to a third of an arcsecond on every
body from the Sun to Saturn, whose parallaxes differ by a factor of
thirteen; the settled reading leaves under a thousandth of an arcsecond
on all of them. `RAHU` and `KETU` are in the table under both readings
because a reading that displaced them would be out by three quarters of
a degree — which is §4's claim, measured rather than asserted.

Over 6 pairs the step moves a body across a pada twice, across a
nakshatra once and across a sign no times. It also changes which lord
the Vimshottari dasha begins with in 1 of the 6:
`c049-kathmandu-2010-06-01` (SUN → MOON). The centre is not a
refinement; it is a setting that changes what the chart says.

## 7. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| the observer's displacement alone, which §4 designs | falsified | worst 0.458196″ in longitude, 0.302981″ in latitude |
| and the station's own aberration | falsified | worst 0.289597″ in longitude, 0.237426″ in latitude |
| and on the direction the light came from | falsified | worst 0.360454″ in longitude, 0.085770″ in latitude |
| and carrying the body over the light time the station saves | falsified | worst 0.079939″ in longitude, 0.083939″ in latitude |
| the light time without taking the Earth's aberration off first | falsified | worst 0.264918″ in longitude, 0.253683″ in latitude |
| the settled reading referred to the mean obliquity | falsified | worst 0.059664″ in longitude, 0.083180″ in latitude |
| the settled reading with the station where the light left it | falsified | worst 0.338995″ in longitude, 0.183533″ in latitude |
| the settled reading on a sphere of the equatorial radius | falsified | worst 3.879496″ in longitude, 8.964201″ in latitude |
| the settled reading at sea level rather than the place's height | falsified | worst 0.749155″ in longitude, 1.266974″ in latitude |
| the settled reading, on every body but the Moon | **holds** | worst 0.000313″ over the other six bodies, 12 times inside the bound |
| the settled reading, on the Moon | falsified | worst 0.079939″, 22 times the bound and 3e-5 of the step it makes |
| `sky::observer` is that reading's station | **holds** | worst 0.000e0 m apart over 6 |
| a direction on the Moon's orbit is where it was | **holds** | 0 of 12 disagree; longitude, latitude and distance, bit for bit |
| and it stays on the ecliptic in every chart cast from a place | **holds** | 0 of 174 disagree; worst latitude 3.6e-15°, where a displaced point at that distance would show 43.67′ |
| the speed is the velocity difference | untested | a change of up to 5.485° left within 43.126″, which is the size of the two rates the corpus does not record |

The step is a displacement, an aberration, a light time and a direction
taken back to the light's own, on the WGS84 ellipsoid at the place's own
height, applied to every body that is somewhere and to no point that is
a direction, with the velocity transformed as the position is.

Two things are left open and are recorded rather than rounded away. The
Moon keeps 0.079939″ — three parts in a hundred thousand of a step
of forty arcminutes, and every other body is a thousand times further
inside the bound — and nothing this pass could construct accounts for
it; the candidates are the recording engine's own bookkeeping for the
Moon's light time and the two rates the corpus does not record, and
neither can be told apart here. And the velocity transform cannot be
decided by this corpus at all. Both belong to a comparison against a
provider that answers both centres natively, which
`05-testing/ACCURACY.md` is for.

