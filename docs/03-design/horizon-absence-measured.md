# Proving a rise or a set is absent, measured

Status: `generated` by `cargo xtask absence` over the analytic test
provider. Do not edit: `check-absence` regenerates this page and fails
on any difference.

## 1. Why an absence is expensive

`almanac::events` collects every rise in a window by searching on from
the last one it found, and **the search that ends the loop has no event
to find**. Proving that costs a walk of the whole remaining window at
ten-minute steps — and it happens twice a day, every day, once for the
rises and once for the sets. Of a fifty-day almanac's calls, 543 carry
14 032 cells, and that is what they are.

## 2. The rule

An altitude is `sin h = sin φ sin δ + cos φ cos δ cos H`, and over a
rotation `cos H` sweeps −1 to 1, so for a fixed declination the
altitude is bounded by its two transits:

```text h ≤ 90° − |φ − δ| upper transit, cos H = 1 h ≥ |φ +
δ| − 90° lower transit, cos H = −1 ```

If the upper bound is under the event's target the body never reaches
it; if the lower bound is over it the body never falls to it. Either way
there is no crossing — from two angles rather than a hundred and
forty-four readings.

## 3. What a margin has to cover, measured rather than assumed

The bounds hold for a fixed δ and δ moves, so the range has to be
widened by what happens between the two readings that bound it. **Not by
the day's motion**: that was tried and it is hopeless — six degrees
for the Moon puts every temperate latitude inside the range, proves
nothing at all, and costs two readings to find that out. What matters is
how far δ strays from the straight line between the ends, which the
sweep measures at seven interior points of every window:

| body | greatest stray from the chord |
|---|---:|
| SUN | 0.001° |
| MOON | 0.168° |

The Moon is the reason this pass exists — its declination moves by
degrees in a day where the Sun's crawls — but the stray is what the
margin pays for, and over a day a smooth curve barely leaves its chord.
The table in `rise_set` carries these with a margin, and the claims
below check that it covers what was seen.

## 4. What the sweep found

1768 searches over 13 latitudes from -78° to 78°, 34 days at
11-day steps from J2000, both bodies, rise and set.

| | count |
|---|---:|
| searches | 1768 |
| of them absent | 274 |
| absences the bound proves | 216 |
| searches the solver had to scan | 5 |
| cells an event that exists costs | 8044 |
| cells the absences cost | 40303 |
| cells the bound would save | 31536 |

## 5. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| the margin the solver uses covers the stray the sweep saw | **holds** | SUN 0.0009° seen, 0.0100° in the table; MOON 0.1676° seen, 0.5000° in the table |
| the bound never says absent where an event exists | **holds** | 0 of 1768 searches |
| an absence can be proved without walking the window | **holds** | 216 of 274 absences, 78.2% of the cells they cost |
| every absence can be proved from the two transits alone | falsified | 216 of 274 |

one of the four claims is falsified.

**The one that must hold does.** A bound that said absent where an event
exists would turn a sunrise into silence, and no search in this sweep
did. It is the only claim on this page whose failure would be a defect
rather than a missed saving, and the margin is what makes it hold: with
no margin at all an earlier version of this pass claimed eleven absences
that were not, every one of them the Moon.

The ambitious claim was never likely: the transits bound the altitude
over a **whole rotation**, and a window shorter than one can miss an
event the bounds allow. So the rule is a fast path and never a
replacement — where it cannot prove an absence the walk still runs,
and the answer is the walk's.

## 6. What the solver does with it

Two guards, and both were put there by a measurement rather than by
caution.

**Only when a walk is about to happen.** The check sits at the top of
the scan, not before the iteration. An event that iterates cleanly is
the common case at any temperate latitude, and making it pay two
readings for a proof it does not need took an almanac from 19 632 calls
to 22 020.

**Only over a whole rotation.** The claim above is not a caveat, it is
the guard: a shorter window can hold no event where these bounds allow
one, so the check would spend two readings to say nothing. That is
exactly what an almanac's event loop does every time it ends — it
searches the *remainder* of a day — and at a temperate latitude the
Moon rises every day, so the bound could never have fired there. With
the guard an almanac pays 0.08% and a polar one keeps the 78.2% above.

`Solver::with_absence_check` turns it off, which is what this page's own
sweep does so that the rule is measured against a search that does not
use it.

