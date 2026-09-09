# The Indian lunisolar month, measured

Status: `generated` by `cargo xtask lunisolar`. Do not edit:
`check-lunisolar` regenerates this page and fails on any difference. The
design it is written for is the Indian lunisolar calendar, which
[`panchanga-day.md`](panchanga-day.md) §8 defers to.

A lunar month runs from one new moon to the next and takes its name from
the solar month it belongs to. Two cases break that correspondence and
the calendar has to say which: a month holding **no** sankranti is
**adhika**, intercalary; one holding **two** is **kshaya**, omitted.
`panchanga` names the month and cannot mark it, and no other module can
either.

**The corpus cannot settle this alone.** It records `is_adhika`
on every day — the *answer* — and none of the inputs: not the new
moon that opened the month, nor the sankranti that named it. So
this pass computes the sky, from the **Surya Siddhanta**, as the
Bikram Sambat engine does; the calendar then needs no ephemeris.
The sample is **12 368 lunar months over 1000 years**, 1500 CE to
2500 CE.

## 1. Three cases, and how often each happens

| sankrantis in the month | months | what it is |
|---|---|---|
| 0 | 388 | **adhika**: intercalary, and the name repeats |
| 1 | 11 961 | an ordinary month |
| 2 | 19 | **kshaya**: the second name is skipped |

So "every lunar month holds one sankranti" is **false**, which is the
whole reason the calendar needs a rule. An adhika month comes round
every **2.58 years** — the classical figure is seven in nineteen, one
every 2.71 — and a kshaya month every **53**, which is why a corpus of
fifty-five days holds two of the first and none of the second.

## 2. The rule against the corpus

Of the fifty-five recorded days, **55** fall inside the measured span
and carry a month to compare.

| claim | agree |
|---|---|
| a month with no sankranti is the one the corpus marks adhika | 55 of 55 |
| the month's name is the sign the Sun stands in at its opening new moon | 54 of 55 |

The **marking** is what this pass exists to settle, and it reproduces
every recorded day, including the two the corpus marks adhika — Delhi
in August 1947 and Fairbanks in June 2015. Both fall in a month the text
finds no sankranti in.

The **naming** needs no change at all, which was not obvious. An
adhika month has no sankranti to be named by, so a reader expects
a special rule — "it takes the following month's name" is the
usual formulation. It does not need one: the Sun stands in the
same sign at the adhika month's new moon and at the nija month's,
so the existing rule gives both the same name by itself. August
1947 is Shravana twice over, once adhika and once not.

Where the naming parts from the corpus:

| day | corpus | computed |
|---|---|---|
| `c055-kathmandu-2024-04-09` | CHAITRA | PHALGUNA |

## 3. Where the text and a drik recording part

The classification asks whether a sankranti falls inside a window of
about 29.5 days, so a boundary that moves by half an hour almost never
changes the answer. **Which month an instant belongs to** is not so
forgiving: an instant within minutes of a new moon belongs to one month
under the text and the other under a modern reckoning.

The closest the corpus comes is `c055-kathmandu-2024-04-09`, **19
minutes** from a month boundary. That is the recorded day the two
disagree about, and it is the eclipse new moon of 8 April 2024: the
text's conjunction and the drik one are about half an hour apart, and
the recorded instant falls between them.

So the two answers have different robustness, and a consumer should be
told which is which. **Adhika and kshaya are the text's to give**;
*which month a given moment falls in*, within an hour of a new moon, is
not — that wants drik values, and comparing them wants the conformance
harness over an adapter that Phase 1 deferred.

## 4. Kshaya has a season, and the pass did not propose it

Every one of the 19 kshaya months in the sample takes its two sankrantis
from the same short arc of the year: Vrishchika (4), Dhanu (16), Makara
(15), Kumbha (3).

Nothing here looks for that. It falls out of the counts, and the reason
is the Earth's orbit: perihelion is in early January, the Sun's apparent
motion is fastest there, and only there can it cross two sign boundaries
inside one lunar month. A rule that produced a kshaya month in, say,
Karka would be wrong on astronomy the calendar never states, and this is
the check that would catch it.

## 5. What this decides

| proposed rule | verdict | measured |
|---|---|---|
| every lunar month holds exactly one sankranti | falsified | 407 of 12368 disagree; which is why the calendar needs a rule at all |
| a month with no sankranti is the one the corpus marks adhika | **holds** | 0 of 55 disagree |
| the month's name is the sign the Sun stands in at its opening new moon | falsified | 1 of 55 disagree; the one is an instant nineteen minutes from a new moon, where the text's conjunction and the recording's are on either side of it — a boundary and not a rule. An adhika month needs no naming rule of its own |
| a month with two sankrantis is kshaya | untested | 19 in the sample and none in the corpus, so the rule is measured and not tested |
| no month holds more than two sankrantis | **holds** | 0 of 12368 disagree |
| kshaya falls only where the Sun moves fastest | **holds** | every one of them between Vrishchika and Kumbha |

**The rule is the count of sankrantis in the lunar month**: none is
adhika, one is ordinary, two is kshaya. It reproduces every recorded day
and needs no special naming.

**The text is enough to decide it.** The Surya Siddhanta's own Sun and
Moon settle the classification, so the Indian lunisolar calendar
computes from the text as the Bikram Sambat engine does and needs no
ephemeris — which is also what the tradition itself did.

**Kshaya is measured and not tested.** The corpus records none, so
nothing here holds the rule to an authority; it is the classical
definition, its frequency is what the astronomy predicts, and its season
is a check it passes. A rank-1 panchangam naming a kshaya year would
turn a measurement into a test, and until one does the page says so.