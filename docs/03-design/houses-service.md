# The houses service: which house, under which reading

Status: `designed`, written 2026-09-07 from the falsification pass in
[`houses-measured.md`](houses-measured.md). Derives from
[`chart-foundation.md`](chart-foundation.md) (the bhavas and the
placements), [`astro-house-systems.md`](astro-house-systems.md) (the
twenty-two systems and the polar policies),
[`chart-bhava-chalit.md`](chart-bhava-chalit.md) (how far the four
chalit methods stand apart) and
[`settings-and-profiles.md`](settings-and-profiles.md) (four knobs, one
of which nothing reads). `02-architecture/01-module-catalog.md` gives
the module its row. Built as `crates/houses`.

## 1. Purpose and scope

The geometry already exists. `astro::houses` computes twenty-two systems
with their polar outcomes, `chart::bhava` makes bhavas of any system's
cusps with the madhya beside the sandhi, and both are measured against
the corpus. **This module is not a second copy of either.** It is the
layer that answers the questions a caller actually asks, and that
nothing currently answers:

- *Which system should this module use here?* — which is a settings
  question, and `houses.module_overrides` is a knob **nothing reads**.
- *Which house is this body in?* — under **both** readings, with the
  bodies that differ between them named.
- *Can I trust this chart?* — which is the degeneracy outcome, and the
  pass found the engine's own boolean disagrees with the SDK in both
  directions.
- *What kind of house is it?* — kendra, trikona, dusthana, upachaya, and
  the lord, which `strength` and `rules` both need and neither has.

It is not: the cusps themselves (`astro`), the bhava arithmetic
(`chart::bhava`), or anything about what a house *means*
(`interpret`).

## 2. Inputs, settings and ports

| input | from |
|---|---|
| the two sets of bhavas | `ChartFoundation`: `houses` under the placement system, `chalit` under the chalit |
| the cusps of any other system | `astro::houses::houses_at`, which needs no ephemeris |
| the settings | all four `houses` knobs, including the one with no reader |
| the catalogue | each system's `degeneracy`, and the signs' lords |

No port and no ephemeris: an ascendant is a sidereal time and a
latitude. **No new knob either** — the point of this module is to read
the four that exist.

## 3. Which system, and for whom

`houses.placement_system` answers "which house is it in" for most of a
chart; `houses.chalit_system` answers it for the chalit; and
`houses.module_overrides` overrides either for a named module, which is
how the KP reading takes Placidus while the rest of a chart is
whole-sign.

```rust
pub enum Purpose { Placement, Chalit }
pub fn system_for(settings: &Settings, purpose: Purpose, module: Option<&str>) -> HouseSystem;
```

One function, because the alternative is every module reading the
settings itself and two of them disagreeing about which system a chart
is in. The pass calls this out as the same shape of gap that made a
chart founded on the SDK's own default profile fail when `state` was
built: a knob shipped and cited with nothing on the other end of it
(registry entry 23).

## 4. Two readings, and the bodies that differ

A chart has two answers to "which house", and the tradition uses both:
the whole-sign or placement reading, and the chalit. The corpus records
the second as a list of the bodies the chalit **moves**, and that is the
right shape — a caller wants the disagreement, not two lists to diff.

The pass checked it in the direction nothing had: `chart`'s test
verifies every body the engine lists, and the pass verifies that the SDK
lists **no others**. Both hold, and the two sets are the same 135 bodies
over 75 fixtures. So the module reports:

```rust
pub struct Placed {
    pub graha: Graha,
    pub placement: u8,     // under the placement system
    pub chalit: u8,        // under the chalit
    pub shifted: bool,     // the two disagree
}
```

with `shifted` derived rather than stored twice, and a `moved()`
iterator over just those — which is what a chart display draws in a
different colour.

## 5. A boolean cannot say what happened

The engine records `is_degenerate` per chart: one bit for "the chosen
system had no solution here". The SDK's `astro::houses` returns an
`Outcome` with three cases — the system computed as asked, another
standing in for it, or one computed at a clamped latitude — and until
this pass nothing had ever compared the two.

**They disagree in both directions**, which is the pass's finding and
the argument for this module's shape:

| | latitude | |
|---|---|---|
| the engine flags, the SDK computes | 64.15°, 64.84° | Reykjavik and Fairbanks under Placidus |
| the engine leaves clear, the SDK cannot | 69.65° | Tromsø under Placidus, where whole sign stood in |

The polar circle for the SDK's obliquity is 66.56°. The engine flags
*below* it, where Placidus is defined, and not *above* it, where it is
not — so the two are not the same quantity read to different precision.
They disagree about which charts are the difficult ones.

A caller deciding whether to trust a chart needs to know which of the
three happened, not merely that something did. So the module reports the
outcome and the policy that produced it, and a harness comparing against
this corpus compares the flag with "the outcome was not `DEFINED`" and
allows the three charts, which registry entry 26 names.

## 6. What kind of house it is

Four classifications, each a set of house numbers, all universally
attested and none in the catalogue yet:

| kind | houses |
|---|---|
| kendra (angular) | 1, 4, 7, 10 |
| panapara (succedent) | 2, 5, 8, 11 |
| apoklima (cadent) | 3, 6, 9, 12 |
| trikona (trine) | 1, 5, 9 |
| dusthana | 6, 8, 12 |
| upachaya | 3, 6, 10, 11 |

The first three **partition** the twelve and the last three overlap
them, which is a property worth asserting rather than assuming: a
`Quadrant` enum for the partition, and predicates for the rest. The
house's **lord** is the lord of the sign its madhya falls in, and the
madhya rather than the sandhi because under an unequal division a house
can begin in one sign and be centred in another.

These ship here because `strength` needs them for the Dig Bala and the
Bhava Bala, `rules` needs them for every yoga stated in terms of a
kendra or a dusthana, and both would otherwise write their own copy.

## 7. The API

```rust
pub enum Purpose { Placement, Chalit }
pub enum Quadrant { Kendra, Panapara, Apoklima }

pub struct Bhava {
    pub number: u8,              // 1 to 12
    pub sign: Rashi,             // the sign its madhya falls in
    pub lord: Graha,
    pub madhya_deg: f64,
    pub sandhi_deg: f64,
    pub quadrant: Quadrant,
}
impl Bhava {
    pub fn is_trikona(&self) -> bool;
    pub fn is_dusthana(&self) -> bool;
    pub fn is_upachaya(&self) -> bool;
}

pub struct Houses { /* … */ }
impl Houses {
    pub fn of(foundation: &ChartFoundation, settings: &Settings) -> Result<Houses, Error>;
    pub fn bhava(&self, number: u8) -> Option<&Bhava>;
    pub fn all(&self) -> &[Bhava; 12];
    pub fn placed(&self, graha: Graha) -> Option<&Placed>;
    pub fn moved(&self) -> impl Iterator<Item = &Placed>;
    pub fn outcome(&self) -> Outcome;
    pub fn quadrant(&self, number: u8) -> Option<Quadrant>;
}

pub fn system_for(settings: &Settings, purpose: Purpose, module: Option<&str>) -> HouseSystem;
pub fn classify(number: u8) -> Option<Quadrant>;
```

`Houses::of` takes the foundation because both readings are already on
it — the module does not recompute cusps that `chart` has already
placed. `system_for` is free-standing because a module asking "which
system do I use" has settings and no chart yet.

## 8. Errors

| when | what |
|---|---|
| a house number outside 1 to 12 | `None`, not an error: it is arithmetic that cannot arise from a chart |
| a foundation whose bhavas are not twelve | `INVALID_ARG` naming the field |
| a longitude that is not finite | `INVALID_ARG` naming the field |
| a degenerate chart | **not** an error: the outcome is reported and the caller decides |

The last is the design decision §5 argues for. A module that refused a
polar chart would make the policy knob meaningless.

## 9. Tests

- **Exhaustive over the classifications**: all twelve houses, that the
  three quadrants partition them, that the overlapping sets are the
  attested ones, and that every house has a lord.
- **Against the corpus**: the shift counted both ways over all 75
  fixtures that carry positions and a chalit, and the degeneracy outcome
  against the engine's flag with the three registered disagreements
  asserted **as** disagreements.
- **Over every shipped profile**, as `aspect` and `points` do, plus the
  module override actually taking effect.

## 10. Open questions

- **The engine's degeneracy criterion** (§5). It is not a latitude
  threshold and this pass could not identify it from three charts. Worth
  another look if the corpus ever grows more polar fixtures.
- **Whether a house's lord follows the madhya or the sandhi.** The
  corpus records no house lords, so nothing here settles it; the module
  takes the madhya and says so, and a module above that wants the other
  is asking a different question.
- **A catalogue kind for the classifications.** They are localisable
  terms with keys, which is what the catalogue is for, and they are
  currently an enum in this crate. Worth moving when a second module
  wants to *name* them rather than test them.
