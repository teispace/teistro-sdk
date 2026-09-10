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

## 3. What the declination does, measured rather than assumed

The bounds hold for a fixed δ and δ moves, so the sweep records what
it moved by over each day:

| body | greatest change in a day |
|---|---:|
| SUN | 0.405° |
| MOON | 5.705° |

The Moon is the reason this pass exists: its declination moves by
degrees where the Sun's crawls, so a margin sized for the Sun would be
wrong for the Moon in the direction that matters.

## 4. What the sweep found

1768 searches over 13 latitudes from -78° to 78°, 34 days at
11-day steps from J2000, both bodies, rise and set.

| | count |
|---|---:|
| searches | 1768 |
| of them absent | 274 |
| absences the bound proves | 164 |
| searches the solver had to scan | 5 |
| cells an event that exists costs | 8044 |
| cells the absences cost | 40303 |
| cells the bound would save | 23944 |

## 5. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| the bound never says absent where an event exists | **holds** | 0 of 1768 searches |
| an absence can be proved without walking the window | **holds** | 164 of 274 absences, 59.4% of the cells they cost |
| every absence can be proved from the two transits alone | falsified | 164 of 274 |

one of the three claims is falsified.

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

