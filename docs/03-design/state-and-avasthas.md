# Planetary state and the avasthas

Status: `built`, written 2026-09-07 from the falsification pass in
[`state-tables-measured.md`](state-tables-measured.md), which measured
every rule below against the corpus's 837 recorded readings before this
page existed. Derives from
[`chart-foundation.md`](chart-foundation.md) (the positions and the
bhavas), [`core-types-and-catalogue.md`](core-types-and-catalogue.md)
(the grahas' exaltations, moolatrikonas, own signs and friendships) and
[`settings-and-profiles.md`](settings-and-profiles.md) (the one knob).
`02-architecture/01-module-catalog.md` gives the module its row. Built as
`crates/state`.

## 1. Purpose and scope

Where a graha *is* the foundation answers. What it **is** — strong or
ruined, at home or in exile, burnt by the Sun, going backwards, at war
with a neighbour, awake or asleep — is this module, and almost every
module above reads it: the strengths weigh it, the rules test it, the
interpretation says it.

It settles: the ladder that gives a dignity; the three friendship
readings and the one thing the catalogue cannot say; combustion as a
table and why it is a knob; the five ages; the planetary war and its
victor; which avasthas are decidable today and which are not; and why a
boundary is a distance rather than a flag.

It is not: the strengths (`strength`, Phase 5), which weigh these; the
aspects (`aspect`), which three avasthas wait on; or the yogas, which
read them.

## 2. Inputs, settings and ports

| input | from |
|---|---|
| the positions | `ChartFoundation`: longitude, latitude, speed, and the bhava each graha falls in |
| the catalogue | each graha's exaltation, debilitation, moolatrikona, own signs and friendships |
| the settings | one knob, `state.combustion_orbs` |

No port, no ephemeris, no instant: everything here is arithmetic over a
founded chart, which is why the whole module is testable against the
corpus without a provider.

**One knob, and it already exists.** `state.combustion_orbs` names a
table, as `aspect.drishti_table` does. The traditions differ over how
near the Sun burns and over whether there is a nearer arc still, and an
implementation that hard-codes one cannot say which it used. Everything
else here is a rule the corpus decides, not a choice.

## 3. The dignity ladder, and two things about it

A body's dignity is the first of a ladder of tests that answers:

1. its **moolatrikona** span,
2. its **exaltation** sign,
3. its **debilitation** sign — deeply so within a degree of the exact
   degree,
4. its **own** signs,
5. and failing all of those, the five-fold friendship with the lord of
   the sign it stands in.

Two of those are not where a reader would put them, and the corpus put
them there.

**Moolatrikona comes before exaltation.** Three grahas have a
moolatrikona span inside their exaltation sign — the Moon's is Taurus
3° to 30° and its exaltation is Taurus — and where the two overlap the
answer is the moolatrikona. Reading the ladder the other way is wrong on
ten of the corpus's readings.

**The shadow grahas take a reduced ladder.** Rahu and Ketu own no sign,
and where a graha falls through to its friendship dignity they are
simply neutral. Their friendships are still computed and still reported;
they just do not reach the dignity. Reading them like a graha is wrong
on 141 readings.

## 4. The friendships, and the one thing the catalogue cannot say

Three readings, each recorded and each exact over the corpus:

- **natural**: the catalogue's own table of friends, neutrals and
  enemies, applied to the lord of the sign the body stands in;
- **temporary**: friend when the dispositor stands in the 2nd, 3rd, 4th,
  10th, 11th or 12th sign from the body, enemy otherwise;
- **the compound**: the classical five-fold table over the two.

And one addition the catalogue structurally cannot make: **a body in its
own sign is its own friend**, in both readings. A graha is not in its own
friends list — it is not in any of its three lists — so the rule lives in
this module rather than in the data. Without it both readings are wrong
on the same 107 readings, which are exactly the ones where a body is its
own dispositor.

The temporary rule is *not* "the dispositor shares the sign": that
happens 87 times to a body that is not the dispositor, and every one of
those is an enemy, because the 1st is not among the six.

## 5. Combustion is a table, and two of them ship

An orb per body, one for direct motion and one for retrograde; and, in
one of the two tables, a deeper orb inside each. The corpus brackets
every row and contradicts none, and the values below are what those
brackets contain.

| body | orb | retrograde | deep | deep retrograde |
|---|---|---|---|---|
| Moon | 12° | 12° | 6° | 6° |
| Mars | 17° | 17° | 8° | 8° |
| Mercury | 14° | 12° | 7° | 6° |
| Jupiter | 11° | 11° | 5° | 5° |
| Venus | 10° | 8° | 5° | 4° |
| Saturn | 15° | 15° | 6° | 6° |

The Sun burns nothing and the shadow grahas do not burn. Where a bracket
is narrow the corpus pins the orb — Mercury direct, between 13.76° and
14.31° — and where it is wide it only fails to contradict it; the
generated page prints every bracket beside its value so a reader can see
which is which.

**The first four columns are one table and the last two are a second.**
The outer orbs are the Surya Siddhanta's degrees of time (IX.6 to 8; the
Moon's, X.1) — the same six numbers `astro`'s heliacal visibility reads,
which a test holds together so the two copies cannot drift. The text
gives nothing inside them. So `SURYA_SIDDHANTA` ships as the six outer
orbs alone and `BPHS` as the same six with the deeper orb the corpus
brackets, and the default profile names the first (ADR-0024: the texts
as read, and nothing else).

That has a consequence worth stating plainly, because it is visible in
the output: **under the default profile no body is ever deeply
combust.** Over the corpus it is exactly one change and it is the
conservative one — the same sixty-six bodies burn, and the thirty-six
the recording engine calls deeply combust come back merely combust. A
caller who wants the deeper reading sets `state.combustion_orbs` to
`BPHS`; a caller who wants to know either way reads `combustion.orbs`,
which reports the orbs the answer was judged against and whether there
was a deeper one at all. Inventing a deep orb for a text that gives none
would have been the SDK making up a rule, which is the one thing this
module refuses to do.

## 6. The five ages, the war, and retrogradation

- **The ages** cut a sign into five parts of six degrees — infant,
  child, youth, old, dead — running forward in an odd sign and backward
  in an even one. The alternation is most of the answer: reading it
  forward everywhere is wrong on 359 of 837 readings.
- **The war** is two of the five planets within a degree of each other,
  and the **northern** body — the one with the greater ecliptic latitude
  — wins. True of all fourteen readings the corpus carries. The
  luminaries and the shadows do not fight.
- **Retrogradation** is the sign of the speed the foundation already
  carries. It is here because combustion reads it and because a caller
  asking "what state is this graha in" expects it in the answer.

## 7. What the corpus cannot settle, and what the module does about it

The **deeptadi**'s nine states and three of the six **lajjitadi**.

What is decided: exaltation is Deepta, an own sign or a moolatrikona is
Swastha, a great friend's sign is Mudita — 246 readings, no exceptions.
Among the lajjitadi, Garvita (exalted or in moolatrikona), Lajjita (in
the fifth house with the Sun, Mars, Saturn, Rahu or Ketu) and Kshobhita
(in the same sign as the Sun) are exact over all 651.

What is not: below that top, 405 readings split six ways, and nothing
the corpus records separates them. Combustion, retrogradation, the war,
the house, the navamsha dignity, and a companion or an aspect of a
benefic or a malefic in the sign were each tried; each is uncorrelated or
anti-correlated. Kshudha, Trishita and Mudita among the lajjitadi are the
same. They are precisely the states whose classical definitions read "or
aspected by", and the SDK has no aspect model yet.

**So the module reports nothing where it cannot decide.** `deeptadi`
returns `Option<AvasthaDeeptadi>` and the lajjitadi list carries only the
three that are decided, with the undecided ones named in a separate
field. A caller can tell an absent answer from a wrong one, which a
plausible guess would take away for good — a wrong rule that reproduces
nothing is discovered; a wrong rule that reproduces a plausible-looking
value is not.

When `aspect` lands, `cargo xtask state` is where the rules are proposed
again, against the same 837 readings.

## 8. A boundary is a distance, not a flag

The recording engine flags a body within a hair of a sign, nakshatra or
pada boundary, on one threshold for all three, which the corpus brackets
to between 21 and 43 arcseconds.

The SDK reports the **distance** to the nearest boundary of each
division. That is always a fact; whether it is *near* depends on how
accurate the provider is, and a constant compiled into the library could
not answer it for two providers of different accuracy. The corpus's own
tolerance file already frames it that way: a classification within the
longitude tolerance of a boundary is an edge case and not a failure. A
caller with a tolerance asks `is_near(tolerance)`; a caller without one
gets the distance and decides.

## 9. The API

```rust
/// What one graha is, in one chart.
pub struct GrahaState {
    pub graha: Graha,
    pub dignity: Dignity,
    pub friendship: Friendship,        // natural, temporary, compound
    pub combustion: Combustion,        // none, combust, deep, with the orb used
    pub motion: Motion,                // direct or retrograde, with the speed
    pub age: AvasthaBaladi,
    pub wakefulness: AvasthaJagradadi,
    pub deeptadi: Option<AvasthaDeeptadi>,
    pub lajjitadi: Vec<AvasthaLajjitadi>,
    pub war: Option<War>,
    pub boundaries: Boundaries,        // the distance to each of the three
}

/// Every graha of a founded chart.
pub fn state(foundation: &ChartFoundation, settings: &Settings)
    -> Result<Vec<GrahaState>, Error>;
```

`GrahaState` is a plain value, `Clone`, serialisable whole, and equal
field for field between two runs of the same inputs. The war is
computed over the whole chart, so the entry point takes the foundation
rather than a graha: a body cannot know it is at war on its own.

## 10. Errors

| condition | outcome |
|---|---|
| a combustion table the SDK does not ship | `UNSUPPORTED`, naming the tables it has |
| a longitude that is not finite | `INVALID_ARG`, before anything is classified |
| a foundation with no Sun | the combustion of every body is `none`, and the value says the Sun was absent rather than claiming nothing is burnt |

Nothing else here can fail: every rule is total over a founded chart.

## 11. Tests

Against the corpus, which decides all of it without a provider: every
recorded dignity, all three friendships, every combustion reading, every
age, every wakefulness, every war and its victor, and the three decided
lajjitadi — 837 readings apiece. The two differences are asserted **as**
differences: the deeply debilitated body the engine records as dreaming
rather than asleep, and the lagna, which the engine gives placeholder
friendships to and the SDK does not treat as a graha at all.

Beside them, the module's own properties: the ladder is total (every
longitude of every sign gives a dignity), the compound table is complete
over its six inputs, a deep combustion is inside its own orb, the ages
partition a sign, and the war is symmetric — if A is at war with B then B
is at war with A, and exactly one of them wins.

## 12. Open questions

- **The deeptadi and the three lajjitadi**, above. They need `aspect`.
- **The deep-debilitation orb.** A degree with the shadow grahas
  excluded, or something under 0.98° for everyone: the corpus cannot
  separate the two. The SDK takes the first because the nodes are
  already excluded from the rest of the ladder.
- **The war's victor.** The northern body wins on all fourteen recorded
  readings, and the corpus rules out none of the other current rules —
  the brighter, the western, the larger — because on these fourteen they
  do not disagree. A fixture where they do would settle it.
- **Whether the lagna has a state at all.** The engine gives it a
  dignity of neutral and placeholder friendships; the SDK does not,
  because the lagna is a point and has no dispositor relationship to
  compute. If a module above turns out to want one, it wants a different
  thing and should say so.
- **Gandanta and marana karaka sthana**, which the module catalogue
  lists here. Neither appears anywhere in the corpus, so there is
  nothing to falsify a rule against and this module builds neither. Both
  are cheap once a source is read: gandanta is a span either side of the
  three water-to-fire junctions and `boundary` already reports the
  distance it needs, and marana karaka sthana is a table of one house
  per graha. They wait on a rank-1 source, not on a computation.
