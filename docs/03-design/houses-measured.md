# The houses, measured

Status: `generated` by `cargo xtask houses` over the conformance
corpus's `houses` sections, 2026-09-07. Do not edit: `check-houses`
regenerates this page and fails on any difference. The design written
from it is [`houses-service.md`](houses-service.md).

## 1. What nothing has read

Most of this section already has a reader. `astro`'s own baseline test
compares all twenty-two systems' cusps, `chart`'s compares the chalit's
madhya, its sandhi and every placement, and `cargo xtask chalit`
measured how far the four chalit methods stand apart. So this pass is
not a second look at those: it is the first look at the four things
nothing has read, which turn out to be the ones a **service** over the
geometry has to get right.

83 fixtures carry a selected system: `PLACIDUS` on 8, `WHOLE_SIGN` on 75.

## 2. The cusps' signs, and the midheaven

Two fields with no reader, and both hold. The sign index beside
each cusp is the sign that cusp's own longitude falls in — 996 of
them, exactly, so it is a rendering and not a second reading. And
the midheaven the engine records for the selected system is the
one the SDK computes from the same instant and place, within
0.0022°, which is the scale `astro`'s own cusp comparison
works at.

| proposed rule | verdict | measured |
|---|---|---|
| a cusp's recorded sign is the sign its own longitude falls in | **holds** | 0 of 996 disagree |
| the recorded midheaven is the one the SDK computes | **holds** | worst 0.0022° over 71 |

The first cusp and the recorded ascendant stand as much as
29.8410° apart, which is not an error: under a
whole-sign chart the first cusp is the **start of the ascendant's
sign** and the ascendant is wherever inside it the ecliptic
actually rises, so the two differ by however far into its sign the
ascendant stands. A service that reports "the first house begins
here" and "the ascendant is here" is reporting two different
numbers, and the design keeps them apart for that reason.

## 3. The degeneracy flag, which nothing had compared

`selected.is_degenerate` says the chosen system had no solution at the
place. It is the field this pass exists for: nothing in the SDK had ever
read it, and it is what a service has to report.

| proposed rule | verdict | measured |
|---|---|---|
| a flagged chart's system is one the catalogue calls polar | **holds** | 0 of 2 disagree |
| the engine's flag is the SDK's own outcome | falsified | 3 of 83 disagree |
| a chart is flagged only inside the polar circle (66.56°) | falsified | the lowest flagged is 64.1466° |

The engine flags 2 of 83 charts, and both are on a system the catalogue
calls `POLAR_UNDEFINED`, so the first claim holds and the two agree that
far.

| fixture | latitude | system |
|---|---|---|
| c027-reykjavik-1975-06-21--placidus | 64.1466° | `PLACIDUS` |
| c043-fairbanks-2015-06-21--placidus | 64.8378° | `PLACIDUS` |

The second and third claims are where they part, and they part in **both
directions**, which is the finding.

The polar circle for an obliquity of 23.4393° is
66.56°. The engine flags charts at 64.1466° — *below* it,
where the SDK computes Placidus without trouble — and leaves
charts clear at up to 69.6492°, *above* it, where the SDK cannot
compute the system asked for at all. So the two are not the same
quantity read to different precision; they disagree about which
charts are the difficult ones.

| the engine flags, the SDK computes | latitude | system |
|---|---|---|
| c027-reykjavik-1975-06-21--placidus | 64.1466° | `PLACIDUS` |
| c043-fairbanks-2015-06-21--placidus | 64.8378° | `PLACIDUS` |

| the engine leaves clear, the SDK cannot | latitude | what the SDK did |
|---|---|---|
| c028-troms-1988-06-21--placidus | 69.6492° | `WHOLE_SIGN` stood in for `PLACIDUS` |

That is a difference to carry rather than reconcile, and it is the
argument for the shape the module takes. A **boolean cannot say what
happened**: the SDK's outcome distinguishes the system computed as
asked, another standing in for it, and one computed at a clamped
latitude, and which of the three occurred is exactly what a caller needs
in order to decide whether to trust the chart. The module reports the
outcome and the policy that produced it; a harness comparing against
this corpus compares the flag with "the outcome was not `DEFINED`" and
allows the charts above, which the deliberate-difference registry names.

## 4. The shift, counted the other way as well

The engine records which bodies the chalit moves out of their whole-sign
house. `chart`'s own test checks every body it lists; what nothing
checked is the other direction — that the SDK finds **no others**. A
rule that shifted one body too many would pass the first check and fail
a chart.

| proposed rule | verdict | measured |
|---|---|---|
| every body the engine lists as shifted does shift | **holds** | 0 of 135 disagree |
| and no other body does | **holds** | 0 of 135 disagree |
| the two counts are the same set | **holds** | 135 listed, 135 found |

Over 75 fixtures the engine lists 135 shifted bodies and reading the
same sandhi the same way finds the same 135. Both directions are counted
because they are different mistakes: a missed shift hides a placement,
and an invented one moves a graha that did not move.

8 further fixtures carry a chalit and no positions — the same
variants §3 flags — so their 38 listed shifts have nothing to be
checked against and are counted here rather than left to swell a
denominator.

## 5. A settings knob nothing reads

`houses.module_overrides` maps a module's name to the house system it
should use — the KP reading takes Placidus where the rest of a chart
is whole-sign — and **nothing in the SDK reads it**. Worse than
unread: the root already *populates* it, so every shipped profile
carries an instruction that nothing has ever asked for.

| proposed rule | verdict | measured |
|---|---|---|
| some shipped profile names a module override | **holds** | 5 of 5 do |

| profile | placement | chalit | overrides |
|---|---|---|---|
| `nepali-default` | `WHOLE_SIGN` | `VEHLOW` | `kp` to `PLACIDUS` |
| `parashari-classical` | `WHOLE_SIGN` | `SRIPATI` | `kp` to `PLACIDUS` |
| `kp-default` | `PLACIDUS` | `PLACIDUS` | `kp` to `PLACIDUS` |
| `western-tropical-default` | `PLACIDUS` | `PLACIDUS` | `kp` to `PLACIDUS` |
| `conformance-baseline` | `WHOLE_SIGN` | `VEHLOW` | `kp` to `PLACIDUS` |

That is the same shape of gap that made a chart founded on the SDK's own
default profile fail when `state` was built: a knob shipped, cited and
resolved by the settings layer, with no module on the other end of it
(registry entry 23). Here it fails more quietly — a KP reading
computed under whole-sign houses is not an error, it is simply the wrong
chart.

It is not a measurement, because there is nothing recorded to measure it
against; it is the reason the module exists as a **service** rather than
a second copy of the geometry. "Which system does this module use here"
is a question about the settings, and the answer has to come from one
place or two modules will disagree about which chart they are reading.

## 6. What this pass decides

- **The cusp signs and the midheaven ship as they are.** Both
fields had no reader and both reproduce, so nothing about the
geometry needs revisiting.
- **The service reports an outcome, not a flag.** The engine's
`is_degenerate` disagrees with the SDK on 3 of 83 charts and in
**both directions**: it flags charts at 64.1466°, below the polar
circle of 66.56°, where the SDK computes the system
asked for, and leaves charts clear at 69.6492°, above it, where
the SDK cannot. Three outcomes carry more than a boolean can,
and which one happened is what decides whether a caller trusts
the chart.
- **The shift is counted both ways**, because a shift the SDK
invents is as wrong as one it misses and only one of the two was
ever checked.
- **`houses.module_overrides` gets a reader**, which is the whole
argument for a service over the geometry rather than beside it.
