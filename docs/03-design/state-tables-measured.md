# The planetary state's rules, measured

Status: `generated` by `cargo xtask state` over the conformance corpus's
`positions.bodies` sections, 2026-09-07. Do not edit: `check-state`
regenerates this page and fails on any difference. The design written
from it is [`state-and-avasthas.md`](state-and-avasthas.md).

Beside every position the corpus records what the body *is*: its
dignity, three readings of its friendship with its dispositor, whether
it is combust and how deeply, whether it is retrograde, its age, three
families of avastha, whether it is at war and with whom, and whether it
sits within a hair of a classification boundary.

That is 837 readings of nine grahas over 93 fixtures — the 55 recorded
days and the variants that carry positions. Each field is a rule, and
this pass proposes one and measures it.

**Three of them the corpus cannot settle**, and saying so is the more
valuable half of the pass (§7). Their classical definitions all read
"or aspected by", the SDK has no aspect model yet, and no reading the
corpus carries separates them. A rule guessed there would be a wrong
rule written into code as a fact.

## 1. The friendships, and the one thing the catalogue does not say

A body's friendship with the lord of the sign it stands in comes in
three readings: the natural one, which is a table; the temporary one,
which is where the dispositor stands; and the five-fold compound of the
two.

| proposed rule | verdict | measured |
|---|---|---|
| natural friendship is the catalogue's table, a body in its own sign counting as its own friend | **holds** | 0 of 837 disagree |
| the same without the self exception | falsified | 107 of 837 disagree |
| temporary friendship: the dispositor in the 2nd, 3rd, 4th, 10th, 11th or 12th, and a body in its own sign its own friend | **holds** | 0 of 837 disagree |
| the same without the self exception | falsified | 107 of 837 disagree |
| the five-fold compound is the classical one | **holds** | 0 of 837 disagree |

The catalogue's table lists each graha's friends, neutrals and enemies
and says nothing about itself, because a body is not in its own list.
The engine treats a body in its own sign as its own friend in **both**
readings, and without that the two rules are wrong on the same 107
readings — which are exactly the ones where the dispositor is the
body. The temporary rule is otherwise the classical one, and it is not
the same as "the dispositor shares the sign": that happens 87 times to
another body, and each of those is an enemy.

## 2. The dignity, and two things about the ladder

A body's dignity is the first of a ladder of tests that answers: its
moolatrikona span, its exaltation sign, its debilitation sign, its own
signs, and failing all of those the five-fold friendship with its
dispositor.

| proposed rule | verdict | measured |
|---|---|---|
| moolatrikona, then exaltation, then debilitation, then own sign, then the compound friendship | **holds** | 0 of 837 disagree |
| exaltation ahead of moolatrikona, where a body's two spans overlap | falsified | 10 of 837 disagree |
| the shadow grahas take their friendship dignity like any other body | falsified | 141 of 837 disagree |
| deep debilitation is within one degree of the exact degree | **holds** | 0.7537° at the widest, and the nearest plain one at 1.5090° |

**Moolatrikona comes first.** Three grahas have a moolatrikona span
inside their exaltation sign — the Moon's is Taurus 3° to 30° and its
exaltation is Taurus — and where the two overlap the engine reports
the moolatrikona. Reading the ladder the other way is wrong on
10 readings.

**The shadow grahas take a reduced ladder.** Rahu and Ketu own no
sign, and where a graha would fall through to its friendship dignity
they are simply neutral: reading their friendship instead is wrong on
141. Their friendship values *are* recorded and are
right; they just do not reach the dignity.

The deep debilitation's orb is bracketed rather than stated: the
widest recorded one is 0.7537° from the exact degree and the nearest
plain debilitation of a graha is 1.5090°, so a whole
degree sits inside the bracket. The shadow grahas complicate it —
their nearest plain debilitation is 0.9794°, inside a
degree — so either they are excluded from deep debilitation as they
are from the rest of the ladder, or the orb is smaller than
0.9794° for everyone. The corpus cannot separate the two,
and the SDK takes the first, which is what the rest of the nodes'
treatment already says.

## 3. Combustion, and the table the corpus brackets

A body near the Sun is combust, and nearer still deeply so. The orb
differs by body and by whether the body is retrograde, and the corpus
does not state it — it **brackets** it, between the widest reading
that is combust and the nearest that is not.

| proposed rule | verdict | measured |
|---|---|---|
| the `BPHS` orb table reproduces every recorded reading | **holds** | 0 of 558 disagree |

Every bracket below contains the classical value, and the `BPHS` table
the SDK ships is those values. Where a bracket is wide the corpus is not
pinning the orb, only failing to contradict it; where it is narrow —
Mercury direct, between 13.76° and 14.31° — it is.

| body | direction | deep out to | combust range | clear from | orb | deep orb |
|---|---|---|---|---|---|---|
| JUPITER | direct | 2.9113° | — | 11.8010° | 11 | 5 |
| JUPITER | retrograde | — | — | 119.7002° | 11 | 5 |
| MARS | direct | 7.5242° | 9.8829° to 16.6909° | 20.5318° | 17 | 8 |
| MARS | retrograde | — | — | 159.1163° | 17 | 8 |
| MERCURY | direct | 6.9610° | 7.8658° to 13.7645° | 14.3144° | 14 | 7 |
| MERCURY | retrograde | 5.3790° | 6.3421° to 11.6281° | 13.9363° | 12 | 6 |
| MOON | direct | 2.3795° | 7.7447° to 9.4851° | 16.4617° | 12 | 6 |
| SATURN | direct | 5.9325° | 7.5162° to 12.4305° | 20.3056° | 15 | 6 |
| SATURN | retrograde | — | — | 111.8465° | 15 | 6 |
| VENUS | direct | 4.5155° | 5.4278° to 9.3177° | 11.3620° | 10 | 5 |
| VENUS | retrograde | — | — | 12.9270° | 8 | 4 |

The orb is a settings knob — `state.combustion_orbs` names a table —
because the tables differ between traditions and an implementation that
hard-codes one cannot say which it used. The deep orb is where they
part: the `SURYA_SIDDHANTA` table the default profile names gives the
same outer orbs and nothing inside them, so under it the deep column
above does not apply and the readings it brackets come back merely
combust.

## 4. The five ages

A sign is cut into five parts of six degrees, and a body's age is which
part it stands in — infant, child, youth, old, dead — running
forward in an odd sign and backward in an even one.

| proposed rule | verdict | measured |
|---|---|---|
| the five ages run forward in an odd sign and backward in an even one, six degrees each | **holds** | 0 of 837 disagree |
| they always run forward | falsified | 359 of 837 disagree |

That the direction alternates is not a detail: reading it forward
everywhere is wrong on 359 of 837 readings, which is most of the even
signs.

## 5. The planetary war

Two planets close enough together are said to be at war, and one of them
wins. Neither the orb nor the victor is stated in the corpus and both
are decided by it.

| proposed rule | verdict | measured |
|---|---|---|
| a war is two of the five planets within a degree of each other | **holds** | 0 of 14 disagree |
| the northern body — the one with the greater ecliptic latitude — wins | **holds** | 0 of 14 disagree |

The orb is bracketed between 0.9597°, the widest war recorded, and
1.2989°, the nearest pair of planets that is not one — so a whole
degree sits inside it. The luminaries and the shadow grahas do not
fight: no recorded war involves one, and the classical rule excludes
them.

The victor is the **northern** body, on every one of the 14 readings the
corpus carries. Several other rules are current — the brighter body,
the one further west, the one with the greater diameter — and the
corpus rules out none of them, only agrees with this one; latitude is
what it is recorded with.

The wars the corpus records, the winner named first:

| fixture | war | separation |
|---|---|---|
| c014-dhaka-2009-06-20 | MARS beats VENUS | 0.5439° |
| c024-london-1947-05-18 | MARS beats VENUS | 0.1163° |
| c033-cape-town-1990-02-11 | VENUS beats SATURN | 0.7371° |
| c044-ushuaia-1995-12-22 | MARS beats MERCURY | 0.9597° |
| c048-kathmandu-2399-12-30 | SATURN beats VENUS | 0.4171° |
| c033-cape-town-1990-02-11--tropical | VENUS beats SATURN | 0.7371° |
| c033-cape-town-1990-02-11--true-node | VENUS beats SATURN | 0.7371° |

## 6. The avasthas the corpus settles

Four of them, and each is a rule over facts the corpus already records.

| proposed rule | verdict | measured |
|---|---|---|
| the wakefulness follows the dignity: exalted, moolatrikona or own sign awake; enemy or debilitated asleep; the rest dreaming | **holds** | 0 of 651 disagree |
| garvita, the proud: exalted or in moolatrikona | **holds** | 0 of 651 disagree |
| lajjita, the ashamed: in the fifth house with the Sun, Mars, Saturn, Rahu or Ketu | **holds** | 0 of 651 disagree |
| kshobhita, the agitated: standing in the same sign as the Sun | **holds** | 0 of 651 disagree |

One thing the wakefulness does that it should not: a **deeply**
debilitated body is recorded dreaming rather than asleep, on all
3 readings that have one. Plain debilitation is
asleep, so the deeper state is the milder one — which is what a
switch that lists the plain values and defaults for the rest does,
and it joins the deliberate-difference registry.

## 7. The avasthas the corpus cannot settle

The deeptadi's nine states and three of the six lajjitadi. What the
corpus **does** settle about the deeptadi is its top: exaltation is
Deepta, an own sign or a moolatrikona is Swastha, a great friend's is
Mudita — 246 readings, no exceptions. Below that, 405 readings split
six ways, and nothing the corpus records separates them:

| dignity | the deeptadi recorded with it |
|---|---|
| DEBILITATED | KHALA 28, SHANTA 30 |
| DEEP_DEBILITATED | SHANTA 3 |
| ENEMY | DINA 13, DUKHI 2, SHANTA 43, VIKALA 3 |
| EXALTED | DEEPTA 36 |
| FRIEND | DUKHI 7, SHANTA 92 |
| GREAT_ENEMY | DINA 9, DUKHI 2, KOPA 4, SHANTA 26 |
| GREAT_FRIEND | MUDITA 103 |
| MOOLTRIKONA | SWASTHA 14 |
| NEUTRAL | DUKHI 5, SHANTA 138 |
| OWN_SIGN | SWASTHA 93 |

Debilitation splits between Khala and Shanta; an enemy's sign splits
four ways. Neither combustion, retrogradation, the war, the house, the
navamsha dignity, nor a companion or an aspect of a benefic or a malefic
in the sign separates them — each was tried and each is uncorrelated
or anti-correlated. The same is true of kshudha, trishita and mudita
among the lajjitadi.

Those are the states whose classical definitions read "or aspected by",
and the SDK has no aspect model yet. So the design reports the states it
can decide and **nothing** where it cannot, rather than a plausible
guess: a caller can tell an absent answer from a wrong one. When
`aspect` lands, this pass is where the rules are proposed again.

## 8. The boundary flags are one threshold

A body within a hair of a sign, nakshatra or pada boundary is flagged,
because its classification would flip under a slightly different
ayanamsha. The threshold is not recorded, and the corpus brackets it to
between 0.00582° and 0.01199° — about 21 and 43 arcseconds — for
all three divisions at once.

| division | flagged out to | unflagged from |
|---|---|---|
| nakshatra | 0.00582° | 0.01199° |
| pada | 0.00582° | 0.01199° |
| sign | 0.00209° | 0.04810° |

One threshold fits all three, which is what a single constant in the
engine looks like. The SDK does not ship a constant: it reports the
**distance** to the nearest boundary of each division, which is always a
fact, and a caller asks whether that is inside whatever tolerance its
provider claims. The corpus's own tolerance file already frames the
question that way — a classification within the longitude tolerance of
a boundary is an edge case and not a failure — and a fixed number in
the library could not answer it for two providers of different accuracy.

## 9. What this decides

1. **The friendship table needs one addition.** A body in its own
   sign is its own friend, naturally and temporarily; the catalogue
   cannot say it because a body is not in its own list.
2. **The dignity ladder checks moolatrikona before exaltation**, and
   the shadow grahas take a reduced one that ends at neutral.
3. **Combustion is a table, and the table is a knob.** The corpus
   brackets every row of it and contradicts none.
4. **The five ages alternate direction**, which is most of the
   readings.
5. **A war is a degree wide and the northern body wins**, on every
   reading the corpus has.
6. **Report absence rather than a guess.** The deeptadi below its top
   three, and three of the six lajjitadi, are not decidable from what
   the corpus records and what the SDK can compute. They wait for
   `aspect`, and until then the answer is nothing rather than
   something plausible.
7. **A boundary is a distance, not a flag.** How near a body is to a
   classification boundary is a fact; whether that is *near* depends
   on the provider, and belongs to the caller.

Measured over 837 readings.