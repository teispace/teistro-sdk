# The drishti: which bodies reach which

Status: `designed`, written 2026-09-07 from the falsification pass in
[`aspect-drishti-measured.md`](aspect-drishti-measured.md), which
measured every rule below before this page existed. Derives from
[`chart-foundation.md`](chart-foundation.md) (the positions and the
bhavas), [`core-types-and-catalogue.md`](core-types-and-catalogue.md)
(the signs' modality, the grahas' nature) and
[`settings-and-profiles.md`](settings-and-profiles.md) (two knobs, both
of which already exist). `02-architecture/01-module-catalog.md` gives
the module its row. Built as `crates/aspect`.

## 1. Purpose and scope

Which bodies reach which, and how strongly. Three relations, and they
are not variants of one thing:

- the **graha drishti**, a body's gaze counted in houses from the sign
  it stands in, full or in quarters;
- the **rashi drishti**, a relation between *signs* that the Jaimini
  reading works in, and which is always mutual;
- the **conjunction**, which is co-presence in a sign, and beneath it
  the **orb engine** that measures an angular separation against a
  stated tolerance, which `tajika` and a future `western` both need.

It is not: Drik Bala or any other weighing of these (`strength`, Phase
5); the argala, the karakamsa or anything else Jaimini builds *on* the
rashi drishti (`jaimini`); ithasala, ishrafa and the Tajika yogas
(`tajika`, which takes the orb engine from here); or the **sphuta
drishti**, the degree-based value, which does not ship at all and §8
says why.

**This is the first Phase 4 module the corpus cannot check.** Every
other one was designed against a recorded answer; the corpus records no
aspect of any kind, which the pass established by searching every key of
all 115 fixture files. What that changes about how the module is built
is §9.

## 2. Inputs, settings and ports

| input | from |
|---|---|
| the positions | `ChartFoundation`: the sign each graha stands in, and its longitude within it |
| the catalogue | each sign's modality and lord, each graha's nature |
| the settings | two knobs, `aspect.node_aspects` and `aspect.drishti_table` |

No port, no ephemeris, no instant. Like `state`, everything here is
arithmetic over a founded chart, which is what lets the whole module be
tested without a provider.

**Both knobs already exist and neither is read by anything.** That is
the trap `state` fell into: `parashari-classical` had named a combustion
table since ADR-0024 and no crate resolved it, so the SDK's own default
profile failed (registry entry 23). `aspect.drishti_table` names
`PARASHARA` in the root, and this module resolves that key or refuses by
name — with the shipped keys in the message — and a test founds a chart
on **every shipped profile** so the failure cannot recur silently.

## 3. The graha drishti is one table

A graha aspects the sign seven from its own fully; the fourth and eighth
at three quarters; the fifth and ninth at a half; the third and tenth at
a quarter. Mars raises its fourth and eighth to full, Jupiter its fifth
and ninth, Saturn its third and tenth.

| houses ahead | every graha | Mars | Jupiter | Saturn |
|---|---|---|---|---|
| 3 | ¼ | ¼ | ¼ | **full** |
| 4 | ¾ | **full** | ¾ | ¾ |
| 5 | ½ | ½ | **full** | ½ |
| 7 | **full** | **full** | **full** | **full** |
| 8 | ¾ | **full** | ¾ | ¾ |
| 9 | ½ | ½ | **full** | ½ |
| 10 | ¼ | ¼ | ¼ | **full** |

Everything else is nothing, the first included: no graha aspects the
sign it stands in, which is why a conjunction is a separate relation and
not a drishti of zero houses.

The source is the project's own research page
(`01-research/feature-universe/01-vedic-parashari-core.md` §G) and the
table's internal consistency is measured: every graha aspects seven of
the twelve houses, and each has exactly one full aspect plus its own
specials (§2 of the measured page).

**A drishti is counted from the sign, not from the bhava.** Over this
corpus the two are indistinguishable, because all 837 recorded
placements put a body in its whole-sign house whatever chalit the
fixture's settings name — which is worth knowing before anyone compares
against it. They part on any chart whose houses come from cusps, and the
tradition's "the seventh from it" is the sign, so that is what the
module counts and what its provenance says.

## 4. The rashi drishti is a relation between signs

A movable sign aspects the three fixed signs but the one next to it; a
fixed sign the three movable but the one before it; a dual sign the
other three dual signs. Every sign therefore aspects exactly three, and
**a rashi drishti always looks back** — all 36 directed pairs, measured.

That mutuality is the whole difference. A graha drishti is a one-way
gaze that only rarely returns; a rashi drishti is a symmetric relation
on signs with no body in it at all. A module that quietly offered one
where a caller asked for the other would be wrong about half the time:
over the corpus's 6696 ordered pairs of bodies the two agree on 4053 and
each sees relations the other does not.

## 5. A mutual full aspect is nearly always the seventh, and the
exception is Mars and Saturn

The pass proposed that two grahas aspecting each other fully must be in
the seventh from each other, and the measurement **refused it**: 48 of
the 1020 sign pairs where two grahas reach each other fully are not the
seventh, and they are exactly two configurations.

- **Mars and Saturn three signs apart.** Saturn stands in Mars's fourth,
  which Mars aspects fully; Mars stands in Saturn's tenth, which Saturn
  aspects fully. It occurs 14 times on 7 of the corpus's 93 charts, so
  it is not a curiosity, and a rule pack looking for a mutual full
  aspect between two malefics will find it.
- **Jupiter with itself across a trine**, which is arithmetic and not
  astrology: a graha stands in one sign, so Jupiter never faces itself.

The second is the design consequence. `mutual` takes a pair of
**bodies** and not a pair of positions, so the impossible case cannot be
constructed; a caller asking "is anything in mutual full aspect here"
gets the seven charts and not a spurious ninety-three.

## 6. The conjunction, and the orb engine under it

A conjunction in this tradition is co-presence in a **sign**, with no
orb at all, and that is what `conjunct` answers. Underneath it sits a
general engine that `tajika` and a future `western` need and that has
nothing to do with signs: an angular separation measured against a
stated tolerance, with whether the faster body is closing on the angle
or leaving it.

Keeping the two apart matters because they answer different questions. A
Tajika ithasala needs the applying separation in degrees and the
deeptamsha orb; a Parashari yoga needs "in the same sign" and nothing
else. One engine that tried to be both would need an orb parameter that
half its callers must pass as "not applicable".

## 7. A drishti is a step function, so it carries its distance

A whole-sign relation changes all at once when a body crosses a sign
edge, so a body near one has an aspect decided by the last arcsecond of
the ayanamsha. The corpus flags four such placements itself, the closest
standing 0.019″ from an edge; moving every flagged body across its own
edge changes 52 of the 6696 ordered pairs.

The module answers this the way `state::boundary` does: it reports the
**distance** from each end of a relation to the sign edge that decides
it, and ships no threshold. A constant compiled into the library could
not answer "is this aspect safe" for two providers of different
accuracy. The tolerance is the caller's to state, and `Drishti::is_firm`
takes it as an argument.

`Boundaries` moves down into `teistro-core` for this, because it is a
fact about the canonical angle and two modules now want it —
`core::interval::Interval` moved for the same reason when `panchanga`
needed it. `state::boundary` re-exports it, so nothing above changes.

## 8. The sphuta drishti does not ship

The degree-based drishti — a value in virupas rather than a relation
between signs — is what Drik Bala weighs and what `aspect.drishti_table`
is named for. **No source in this project gives its construction.** The
research page records that one exists and never writes it down; the
corpus records no drishti value to fit one to; and a piecewise formula
recalled rather than read is exactly what this project's whole working
pattern exists to keep out of the code. It is registered as crux C45.

What the pass publishes instead is the **specification** any
construction must meet: 15 virupas at the third and tenth, 30 at the
fifth and ninth, 45 at the fourth and eighth, 60 at the seventh, and 60
at each special graha's own two houses — thirteen values that any
candidate formula has to reproduce before it ships.

The module does give `Strength::virupas`, which is the whole-sign value
in the same unit (a quarter is fifteen). That is the table above read in
virupas rather than a sphuta drishti, and the name and the documentation
say so: what is missing is the interpolation *between* the houses, not
the values at them.

## 9. Building a module the corpus cannot check

The other four Phase 4 modules were held to a recorded answer. This one
cannot be, so the tests carry the weight instead, and they are of three
kinds:

1. **Invariants over the whole space**, not samples. Every graha against
   every house, every sign against every sign, every ordered pair — the
   space is 12 × 12 × 9 and is exhausted rather than sampled, the way
   `vargas`'s exhaustive test does it.
2. **Agreement with the falsification pass**, which computes the same
   tables independently and without this crate. `cargo xtask aspect`
   never imports `teistro-aspect`; the crate's tests assert the pass's
   published numbers — 1020 mutual pairs, 48 beyond the seventh, 3679
   graha drishti relations over the corpus — so the two derivations
   have to keep agreeing.
3. **The corpus's own placements**, for the counts §4 and §5 publish.
   The corpus cannot say whether an aspect is right, but it can say how
   many there are, and a change that moved that number would be caught.

## 10. The API

```rust
// the kernel: signs and houses, no chart
pub enum Strength { None, Quarter, Half, ThreeQuarters, Full }
impl Strength {
    pub const fn quarters(self) -> u8;      // 0 to 4
    pub const fn virupas(self) -> u16;      // the whole-sign value, §8
    pub const fn is_full(self) -> bool;
}

pub fn quarters(graha: Graha, houses: u8) -> Strength;
pub fn between(graha: Graha, from: Rashi, to: Rashi) -> Strength;
pub fn looking_back(houses: u8) -> u8;      // the same relation from the other end

pub mod rashi {
    pub fn aspects(from: Rashi, to: Rashi) -> bool;
    pub fn aspected(sign: Rashi) -> [Rashi; 3];
}

// the orb engine, which knows nothing of signs
pub struct Angle { pub key: &'static str, pub degrees: f64 }
pub struct Hit { pub angle: Angle, pub apart_deg: f64, pub from_exact_deg: f64, pub applying: bool }
pub fn hits(from: Moving, to: Moving, angles: &[Angle], orb_deg: f64) -> Vec<Hit>;

// the assembly
pub struct Drishti {
    pub from: Graha, pub to: Graha,
    pub houses: u8, pub strength: Strength,
    pub from_edge: Boundaries, pub to_edge: Boundaries,
}
impl Drishti { pub fn is_firm(&self, tolerance_deg: f64) -> bool; }

pub struct Mutual { pub first: Graha, pub second: Graha, pub houses: u8, pub both_full: bool }

pub struct Aspects { /* … */ }
impl Aspects {
    pub fn of(foundation: &ChartFoundation, settings: &Settings) -> Result<Aspects, Error>;
    pub fn cast_by(&self, graha: Graha) -> impl Iterator<Item = &Drishti>;
    pub fn on(&self, graha: Graha) -> impl Iterator<Item = &Drishti>;
    pub fn between(&self, from: Graha, to: Graha) -> Option<&Drishti>;
    pub fn mutual(&self) -> impl Iterator<Item = Mutual>;
    pub fn conjunct(&self, graha: Graha) -> impl Iterator<Item = Graha>;
    pub fn strongest_on(&self, graha: Graha) -> Option<&Drishti>;
    pub fn is_aspected_by(&self, target: Graha, source: Graha) -> bool;
}
```

Two shapes are deliberate. `Aspects::of` takes the whole foundation,
because a relation is between two bodies and no body knows on its own
who is looking at it. And every accessor is an iterator over borrowed
relations rather than a fresh `Vec`, because `rules` will ask "is X
aspected by a malefic" once per predicate per chart and should not
allocate to find out.

## 11. Errors

| when | what |
|---|---|
| a drishti table the SDK does not ship | `UNSUPPORTED`, naming the ones it has, with the field `aspect.drishti_table` |
| a house count outside 1 to 12 | `Strength::None`, not an error: it is arithmetic that cannot arise from two signs |
| a foundation with fewer than two bodies | an empty set of relations, which is the true answer |
| an orb that is negative or not finite | `INVALID_ARG` naming the field and the range |

## 12. Tests

- **Exhaustive**: every graha against every house (108 cells); every
  ordered pair of signs under both systems (144 each); every ordered
  pair of grahas and signs for mutuality (11 664).
- **Against the pass**: the published counts of
  `aspect-drishti-measured.md`, recomputed through this crate.
- **Over the corpus**: the relation counts of §4 and the boundary counts
  of §5, over all 93 fixtures that carry positions.
- **Over every shipped profile**: a chart founds and its aspects compute
  under each of the five, so a profile naming a table nothing resolves
  fails here rather than in a consumer's application.

## 13. Open questions

- **The sphuta drishti's construction** (C45), above. Drik Bala waits on
  it, and the specification in §8 is what a source will be checked
  against.
- **What a node aspects.** `aspect.node_aspects` offers three readings,
  the corpus prefers none, and the root's `NONE` stays the default
  because it claims least. The two that give a node more each add two
  houses, so they reach the same *number* of bodies and a different set.
- **Whether a drishti should be offered counted from the bhava.** The
  corpus cannot tell the two apart, so nothing here settles it; if a
  module above wants it, it wants a different relation and should say
  so, as the chalit finding says of houses.
- **The three lajjitadi, still.** §7 of the pass gives them a *necessary*
  condition that holds on every recorded reading and needs no aspect at
  all, so `crates/state` can now answer "certainly not" where it
  answered nothing. What it still cannot answer is "certainly yes": the
  engine applies something narrower than the tradition's condition and
  it is not a drishti, because adding the aspect clause widens the gap
  rather than closing it.
