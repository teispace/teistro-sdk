# The drishti, measured

Status: `generated` by `cargo xtask aspect` over the conformance corpus,
2026-09-07. Do not edit: `check-aspect` regenerates this page and fails
on any difference. The design written from it is
[`aspect-and-drishti.md`](aspect-and-drishti.md).

## 1. What the corpus records, which is nothing

**The corpus records no aspect.** Every key of all 115 fixture files was
searched for a name an aspect could have been recorded under —
`aspect`, `drishti`, `drsti`, `sphuta_drishti`, `graha_drishti`,
`rashi_drishti` — and none were found. The fixture schema carries none
either. This is the first Phase 4 module the corpus cannot check
directly, and saying so plainly is the first result of the pass.

What it can still measure is four things:

1. Each system's **own invariants**, which need no recording: a
table either is mutual or is not.
2. The systems **against each other** over the corpus's 837 real
placements on 93 charts, which is the bhava-chalit pass's
shape — how far apart two things a reader might both call "an
aspect" actually are.
3. The avasthas `crates/state` **refused**, retried with a drishti
in hand. The corpus records 651 of those readings,
so this part has a real answer.
4. What is still missing, written as the specification the missing
thing has to meet.

## 2. The graha drishti, and what holds it together

A graha aspects the sign seven from its own fully, the fourth and eighth
at three quarters, the fifth and ninth at a half, and the third and
tenth at a quarter; Mars raises the fourth and eighth to full, Jupiter
the fifth and ninth, Saturn the third and tenth. The corpus records none
of this, so what can be measured is the table's own consistency — and
the fourth claim below is one this pass proposed and the measurement
**refused**.

| proposed rule | verdict | measured |
|---|---|---|
| every graha aspects seven of the twelve houses | **holds** | 0 of 9 disagree |
| a graha's full aspects are the seventh and its own specials | **holds** | 0 of 9 disagree |
| no graha aspects the sign it stands in | **holds** | 0 of 9 disagree |
| two full aspects that meet do so through the seventh | falsified | 48 of 1020 disagree |
| the ones that do not are Jupiter with itself, or Mars with Saturn | **holds** | 0 of 48 disagree |

A full aspect is very nearly mutual only through the seventh, and the
exceptions are exactly two configurations out of the 1020 ordered pairs
of signs where two grahas reach each other fully:

- **JUPITER and JUPITER** on 24 sign pairs, the first reaching the second across its 5th and its 9th.
- **MARS and SATURN** on 12 sign pairs, the first reaching the second across its 4th.
- **SATURN and MARS** on 12 sign pairs, the first reaching the second across its 10th.

Mars three signs before Saturn is the one that matters: Saturn stands in
Mars's fourth, which Mars aspects fully, and Mars stands in Saturn's
tenth, which Saturn aspects fully. The Jupiter pair is arithmetic rather
than astrology — a graha is in one sign, so Jupiter never faces itself
— which is why a module that reasons about mutual aspects has to take
the pair of **bodies** and not the pair of positions. Over the corpus's
own charts the Mars and Saturn configuration occurs 14 times on 7 of the
93 charts, so it is not a curiosity.

| count | Sun | Moon | Mars | Mercury | Jupiter | Venus | Saturn | Rahu | Ketu |
|---|---|---|---|---|---|---|---|---|---|
| 1 | — | — | — | — | — | — | — | — | — |
| 2 | — | — | — | — | — | — | — | — | — |
| 3 | 1/4 | 1/4 | 1/4 | 1/4 | 1/4 | 1/4 | **full** | 1/4 | 1/4 |
| 4 | 3/4 | 3/4 | **full** | 3/4 | 3/4 | 3/4 | 3/4 | 3/4 | 3/4 |
| 5 | 2/4 | 2/4 | 2/4 | 2/4 | **full** | 2/4 | 2/4 | 2/4 | 2/4 |
| 6 | — | — | — | — | — | — | — | — | — |
| 7 | **full** | **full** | **full** | **full** | **full** | **full** | **full** | **full** | **full** |
| 8 | 3/4 | 3/4 | **full** | 3/4 | 3/4 | 3/4 | 3/4 | 3/4 | 3/4 |
| 9 | 2/4 | 2/4 | 2/4 | 2/4 | **full** | 2/4 | 2/4 | 2/4 | 2/4 |
| 10 | 1/4 | 1/4 | 1/4 | 1/4 | 1/4 | 1/4 | **full** | 1/4 | 1/4 |
| 11 | — | — | — | — | — | — | — | — | — |
| 12 | — | — | — | — | — | — | — | — | — |

The nodes take the same table as everything else here. What a node
aspects beyond the seventh is a settings question and is measured in
§6.

## 3. The rashi drishti is a different relation, not a variant

The Jaimini reading is a relation between **signs** and not between
bodies: a movable sign aspects the three fixed signs but the one next to
it, a fixed sign the three movable but the one before it, and a dual
sign the other three dual signs.

| proposed rule | verdict | measured |
|---|---|---|
| every sign aspects exactly three signs | **holds** | 0 of 12 disagree |
| no sign aspects itself | **holds** | 0 of 12 disagree |
| a rashi drishti is always mutual | **holds** | 0 of 36 disagree |
| the two systems are the same relation | falsified | 28 pairs shared, 56 graha only, 8 rashi only |

Mutuality is the difference that matters. A rashi drishti always looks
back — all 36 directed pairs — and a graha drishti almost never
does, which is why a module cannot quietly offer one where a caller
asked for the other. Of the 132 ordered pairs of distinct signs the two
agree on 28, and each sees 56 and 8 the other does not.

| sign | aspects |
|---|---|
| ARIES | LEO, SCORPIO, AQUARIUS |
| TAURUS | CANCER, LIBRA, CAPRICORN |
| GEMINI | VIRGO, SAGITTARIUS, PISCES |
| CANCER | TAURUS, SCORPIO, AQUARIUS |
| LEO | ARIES, LIBRA, CAPRICORN |
| VIRGO | GEMINI, SAGITTARIUS, PISCES |
| LIBRA | TAURUS, LEO, AQUARIUS |
| SCORPIO | ARIES, CANCER, CAPRICORN |
| SAGITTARIUS | GEMINI, VIRGO, PISCES |
| CAPRICORN | TAURUS, LEO, SCORPIO |
| AQUARIUS | ARIES, CANCER, LIBRA |
| PISCES | GEMINI, VIRGO, SAGITTARIUS |

## 4. The two systems over the corpus's own placements

6696 ordered pairs of bodies over 93 charts — every graha against
every other, in the positions the corpus actually records.

| proposed rule | verdict | measured |
|---|---|---|
| the two systems pick out the same pairs of bodies | falsified | 4053 of 6696 pairs agree |
| the recorded house is the whole-sign house from the lagna | **holds** | 0 of 837 disagree |
| so a house counted by sign is the house the corpus records | **holds** | 0 of 6696 disagree |

| relation | pairs | share |
|---|---|---|
| a graha drishti of any strength | 3679 | 54.9% |
| of them, full | 893 | 13.3% |
| a rashi drishti | 1718 | 25.7% |
| both | 1377 | 20.6% |
| neither | 2676 | 40.0% |

The two systems agree on 4053 of the 6696 pairs and each sees relations
the other does not, which is the bhava-chalit finding in another place:
they are not variants of one thing, so a value has to say which produced
it.

The middle claim is worth stating because it decides how this corpus may
be compared against at all. All 837 of the recorded placements put a
body in the **whole-sign** house counted from the lagna's own sign,
whatever bhava chalit the fixture's settings name. So over this corpus a
drishti counted from the sign and one counted from the recorded house
are the same relation, and a harness cannot tell them apart here. They
part on any chart whose houses come from cusps, and the module therefore
counts from the **sign** — which is what the tradition's "the seventh
from it" means — and stamps the reading beside the answer.

## 5. A drishti counted by sign is a step function

A whole-sign relation changes all at once when a body crosses a sign
boundary, so a body near one has an aspect that is decided by the last
arcsecond of the ayanamsha. The corpus flags such bodies itself: 4 of
837 placements carry `near_sign_boundary`, and the closest any body
stands to an edge is 0.0191″.

| proposed rule | verdict | measured |
|---|---|---|
| a whole-sign drishti is safe wherever a body stands | falsified | 52 of 6696 pairs change if a flagged body crosses its edge |

Moving every flagged body across its own edge changes 52 of the 6696
ordered pairs. That is the same argument `state::boundary` makes and the
module answers it the same way: it reports the **distance** to the edge
beside the aspect, and the caller decides whether its provider is
accurate enough for the answer to stand. A drishti is a step function,
and a value that hides which step it is on is hiding the only thing that
could be wrong about it.

## 6. What a node aspects is a choice, and the corpus does not make it

`aspect.node_aspects` offers three readings and the root takes the
first. Nothing in the corpus prefers any of them — there is no
recorded aspect to compare — so this is a measurement of what the
choice **costs**, not of which is right.

| reading | beyond the seventh | aspects cast by a node |
|---|---|---|
| `NONE` | nothing | 289 |
| `FIVE_SEVEN_NINE` | the 5th and the 9th | 464 (1.6× the first) |
| `THREE_SEVEN_ELEVEN` | the 3rd and the 11th | 464 (1.6× the first) |

The two readings that give a node an aspect beyond the seventh each add
two houses, so they reach the same number of bodies and differ in
**which**: 1.6 times what `NONE` reaches, and not the same set. That is
a large enough difference that a value has to carry which reading made
it, which is what the provenance stamp is for; it is not a reason for
the SDK to pick one on the corpus's behalf, because the corpus is
silent. The default stays the root's `NONE` — the reading that claims
least.

## 7. The avasthas the state pass refused, retried

`crates/state` ships three of the six lajjitadi as **undecided** because
their classical definitions read "or aspected by" and no aspect model
existed (`state-tables-measured.md` §7). One now does, so the rules are
proposed again here, against the 651 readings the corpus records the
family for. Each rule is scored both ways: a reading the rule misses and
a reading it invents are different mistakes.

**KSHUDHA**, recorded on 30 of 651 readings.

| proposed rule | verdict | missed | invented |
|---|---|---|---|
| in an enemy's sign | falsified | 0 | 109 |
| in an enemy's sign, or joined by an enemy | falsified | 0 | 198 |
| the same, or **aspected by** an enemy | falsified | 0 | 536 |
| the same, and with Saturn as well | falsified | 0 | 540 |

**TRISHITA**, recorded on 47 of 651 readings.

| proposed rule | verdict | missed | invented |
|---|---|---|---|
| in a watery sign | falsified | 0 | 114 |
| in a watery sign, **aspected by** a malefic | falsified | 0 | 111 |
| the same, and no benefic aspecting it | falsified | 42 | 13 |

**MUDITA**, recorded on 88 of 651 readings.

| proposed rule | verdict | missed | invented |
|---|---|---|---|
| in a friend's sign | falsified | 0 | 264 |
| in a friend's sign, or joined by a friend | falsified | 0 | 367 |
| the same, or **aspected by** Jupiter | falsified | 0 | 471 |

**No rule is exact, and every rule's misses are zero.** That pattern is
the result. Each family's plainest condition — an enemy's sign, a
watery sign, a friend's sign — holds on every one of the readings the
engine records the state for, and on a good many it does not; and
**adding the aspect clause only widens the gap**, because an aspect
reaches more bodies than a sign does. So the tradition's condition is
*necessary* and the engine applies something narrower, and whatever that
is, it is not a drishti: if it were, the aspect rules would have moved
the count towards the engine's rather than away from it.

| proposed rule | verdict | measured |
|---|---|---|
| every recorded KSHUDHA satisfies the tradition's plainest condition | **holds** | 0 of 30 disagree |
| every recorded TRISHITA satisfies the tradition's plainest condition | **holds** | 0 of 47 disagree |
| every recorded MUDITA satisfies the tradition's plainest condition | **holds** | 0 of 88 disagree |

Two things follow, and they are worth more than a fitted rule would have
been. **`crates/state` can say `no` where it now says nothing**: a body
outside the necessary condition certainly does not hold the state, and
that costs no aspect model at all — a sign and a dignity decide it,
which the module already has. And what stays undecided stays undecided
for a measured reason: 652 readings satisfy the condition and only 165
of them are recorded, so the engine's own extra restriction is not in
anything it records beside them.

The deeptadi's lower states are the other half of what the state pass
refused, and they are tried the same way. 405 readings have a dignity
that does not decide one, 332 of them are SHANTA, and the question is
whether a benefic's aspect is what makes the difference.

| proposed rule | verdict | missed | invented |
|---|---|---|---|
| aspected by a benefic | falsified | 40 | 62 |
| aspected by a benefic and no malefic | falsified | 323 | 5 |
| joined or aspected by a benefic | falsified | 21 | 68 |
| not aspected by a malefic | falsified | 320 | 5 |

The best of them is wrong on 89 of the 405. A drishti does not separate
the deeptadi either, and the same conclusion follows: what the engine
records under these names does not follow from what it records beside
them, and the SDK goes on reporting nothing where it cannot decide.

## 8. The sphuta drishti, and why it does not ship

The degree-based drishti — a value in virupas rather than a relation
between signs — is what Drik Bala weighs (Phase 5,
`strength-schemes.md`) and what `aspect.drishti_table` is named for.
**No source in this project gives its construction.** The research page
records that one exists ("the BPHS formula with the Mars, Jupiter and
Saturn special cases",
`01-research/feature-universe/01-vedic-parashari-core.md` §G) and
nowhere writes it down; the corpus records no drishti value to fit one
to; and a piecewise formula recalled rather than read is exactly the
kind of thing this pass exists to keep out of the code.

So the module ships no sphuta drishti and refuses the table by name, and
the question is registered as a crux. What the pass can do is publish
the **specification** any construction has to meet, so that reading a
source later is a matter of checking it against this table rather than
trusting it:

| at the house | the value must be | because |
|---|---|---|
| 3 | 15 virupas | 1/4 of a full aspect |
| 4 | 45 virupas | 3/4 of a full aspect |
| 5 | 30 virupas | 2/4 of a full aspect |
| 7 | 60 virupas | all of a full aspect |
| 8 | 45 virupas | 3/4 of a full aspect |
| 9 | 30 virupas | 2/4 of a full aspect |
| 10 | 15 virupas | 1/4 of a full aspect |

and, for each of the three special grahas, 60 virupas at its own two
houses instead of the value above: Mars at the fourth and eighth,
Jupiter at the fifth and ninth, Saturn at the third and tenth. A
construction that does not reproduce all 13 of those values is not the
one the whole-sign table is a summary of, and that is a test a future
source has to pass before it ships.

## 9. What this pass decides

- **The graha drishti and the rashi drishti ship**, as tables whose
invariants hold and whose sources are the project's own research
page. They are different relations and the module keeps them
apart: over the corpus's 6696 pairs they agree on 4053 and disagree
on the rest.
- **A drishti is counted from the sign.** Over this corpus that is
indistinguishable from counting by house, because all 837 recorded
placements are whole-sign; on a chart whose houses come from
cusps it is not, so the reading is part of the answer and is
stamped with it.
- **A drishti carries the distance to the sign edge that decides
it**, for the reason `state::boundary` carries one: the module
cannot know how accurate the caller's provider is.
- **The node's aspect stays the root's `NONE`.** Nothing in the
corpus prefers a reading, and the knob already exists to say so.
- **The sphuta drishti does not ship.** Its construction has no
source here; §8 is the specification it will have to meet.
- **The three lajjitadi gain a necessary condition and stay
undecided beyond it.** §7 tried every rule the tradition states,
with a real drishti, and none is exact — but none misses a
single recorded reading either, so `crates/state` can answer
`no` with certainty where the condition fails and needs no
aspect model to do it. Adding the aspect clause widens the gap
rather than closing it, which is evidence the recording engine
does not compute these from a drishti at all.
