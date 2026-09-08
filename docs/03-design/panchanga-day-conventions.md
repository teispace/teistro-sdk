# The daily panchanga's conventions, measured

Status: `generated` by `cargo xtask panchanga` over the conformance
corpus's `panchanga_day` section, 2026-09-07. Do not edit:
`check-panchanga` regenerates this page and fails on any difference. The
design written from it is [`panchanga-day.md`](panchanga-day.md).

A daily panchanga is almost entirely convention. Which arc a period
divides, which instant a limb is read at, which of two nights sizes a
muhurta, whether a yoga is a flag or an interval — none of it shows in
a number and all of it changes the number. The corpus records 55 days of
one implementation's answers and says nowhere how any of them was
reckoned, so this pass proposes a rule for each field and measures it.

Two of the 55 days (c028 and c029) are polar: the Sun did not cross the
horizon and the engine synthesised the bounds (entry 3 of the
deliberate-difference registry). Dividing an arc that is not an arc
measures nothing, so the claims below are measured over the 53 real days
and §11 reports the other two.

## 1. The window is sunrise to the next sunrise

The section records a sunrise and a sunset and never the sunrise that
closes the day, so the first thing to settle is where the day ends.
Every limb list answers it: all four begin at one instant and end at one
instant, on every day of the corpus.

| proposed rule | verdict | measured |
|---|---|---|
| the four limb lists begin at one instant | **holds** | worst 0 s, exactly |
| that instant is the day's sunrise | **holds** | worst 0 s, exactly |
| the four limb lists end at one instant, which is therefore the next sunrise | **holds** | worst 0 s, exactly |

The window is 23.9704 to 24.0197 hours long — a day, plus or minus the
change in the length of the daylight — and every claim below reads the
last limb end as the next sunrise.

This is not the civil day. The section's `local_date` is a civil date,
and the window carrying its limbs opens at that date's sunrise and
closes at the next, so an instant between midnight and sunrise belongs
to the window before. `crates/chart`'s `day` module already inverts that
for a chart; a daily panchanga is the same window asked for by date
rather than by instant.

## 2. A limb is a span, and the corpus's spans are clipped

Each moving limb is recorded as the list of its members that touch the
window, in order, with a start and an end. The first span's start and
the last span's end are the *window's* bounds and not the member's: a
tithi that began yesterday evening is recorded as beginning at sunrise,
and its real start is not recoverable from the fixture.

How many spans a window holds, over the corpus:

| limb | members | 1 | 2 | 3 | 4 |
|---|---|---|---|---|---|
| tithi | 30 | 2 | 52 | 1 | 0 |
| nakshatra | 27 | 1 | 51 | 3 | 0 |
| yoga | 27 | 1 | 48 | 6 | 0 |
| karana | 11 | 0 | 4 | 48 | 3 |

| proposed rule | verdict | measured |
|---|---|---|
| a tithi, nakshatra or yoga span's member is the previous plus one, cycling | **holds** | 0 of 171 disagree |
| a karana span's member is the next of the month's sixty, not of a cycle of eleven | **holds** | 0 of 109 disagree |
| a span ends where the next begins | **holds** | worst 0.009 s |
| a tithi's recorded number counts through the month, one to thirty | **holds** | 0 of 109 disagree |
| a tithi's recorded number counts within its paksha, one to fifteen | falsified | 53 of 109 disagree |

The gap between a span's end and the next span's start is 0.009 s on
every one of the 280 consecutive pairs, which is 1e-7 days: one boundary
instant written twice, at the tolerance the engine's solver stops at.
The SDK's boundary solver stops at the same figure —
`astro::events::TOLERANCE_DAYS` is 1e-7 days — and records the instant
once.

The tithi's number is the other place two conventions meet. The corpus
counts one to thirty through the lunar month; the SDK's catalogue gives
each member a number within its paksha, one to fifteen, which is how a
tithi is named. Both are right and they are not the same field, so a
harness that compares them without saying which is comparing nothing.

The karana is the limb that is not a cycle. Sixty karanas make a lunar
month: Kimstughna opens it, Shakuni, Chatushpada and Naga close it, and
the seven movable ones repeat through everything between, so the seventh
movable karana is followed by the first inside the month and by Shakuni
at the end of it. It is also the only list that reaches four spans, a
karana being half a tithi. Every list can hold one, when a slow member
covers the whole window.

## 3. Every period is a proportional division of an arc

Six of the section's fields divide an arc into equal parts. Not one
divides a clock hour, and the daylight and the night are divided
separately, so a period's length changes with the season and the
latitude.

| proposed rule | verdict | measured |
|---|---|---|
| rahu kaal, yamaghanda and gulika are each one eighth of the daylight | **holds** | worst 4.55e-9 of an eighth |
| the choghadiya are eight equal parts of the daylight and eight of the night | **holds** | worst 0 s, exactly |
| the horas are twelve over the daylight and twelve over the night | **holds** | worst 0 s, exactly |
| the horas are twenty-four equal parts of the whole window | falsified | worst 9.83 h |

The equal hora — twenty-four sixty-minute hours from sunrise — is out
by up to 9.83 h, which is not a rounding difference but a different
reckoning. The corpus decides it, and it decides for the proportional
one; `day.hora_reckoning` already has both
values and `crates/time`'s hora module already computes either (entry
13 of the registry), so the daily panchanga calls it rather than
dividing an arc of its own.

Which eighth of the daylight each inauspicious period occupies, by the
day of the week — the corpus's table, counted from one:

| vara | rahu kaal | yamaghanda | gulika |
|---|---|---|---|
| Monday (SOMAVARA) | 2 | 4 | 6 |
| Tuesday (MANGALAVARA) | 7 | 3 | 5 |
| Wednesday (BUDHAVARA) | 5 | 2 | 4 |
| Thursday (GURUVARA) | 6 | 1 | 3 |
| Friday (SHUKRAVARA) | 4 | 7 | 2 |
| Saturday (SHANIVARA) | 3 | 6 | 1 |
| Sunday (RAVIVARA) | 8 | 5 | 7 |

The three rows are the classical ones. Yamaghanda and gulika are
arithmetic — the third and the fifth eighth, counted backwards from
the weekday, modulo seven — and rahu kaal is not, so all three ship as
one table rather than as two rules and an exception.

## 4. The choghadiya is the hora's walk, not a grid of names

A choghadiya is usually printed as a grid of seven rows by eight
columns. It is not a grid. The lord of the *k*th eighth of the daylight
is the lord of the *k*th hora — the same walk five weekdays at a time
that `time::hora::lord_of` already computes — and the night's walk
starts four weekdays on from the vara and steps four at a time.

| proposed rule | verdict | measured |
|---|---|---|
| a hora's lord is the vara's, walked five weekdays on per hora | **holds** | 0 of 1320 disagree |
| a day choghadiya's lord is that same walk, one step per eighth | **holds** | 0 of 440 disagree |
| a night choghadiya's walk starts four weekdays on and steps four | **holds** | 0 of 440 disagree |

The name follows from the lord alone, one row per graha:

| lord | choghadiya |
|---|---|
| The Sun | udveg |
| The Moon | amrit |
| Mars | rog |
| Mercury | laabh |
| Jupiter | shubha |
| Venus | char |
| Saturn | kaal |

So the whole of the choghadiya is a seven-row table of names over a walk
the SDK already has. Sixteen intervals a day come out of it and none of
it is a new algorithm.

## 5. Abhijit holds; Brahma muhurta is sized from the wrong night

Both are muhurtas — fifteenths of an arc — and both sit where the
tradition puts them. Abhijit is the eighth muhurta of the daylight,
straddling noon, and is void on Wednesdays: 4 of the 55 days are
Wednesdays and every one is recorded void, while every one of the other
51 is effective.

| proposed rule | verdict | measured |
|---|---|---|
| Abhijit is the eighth of the daylight's fifteen muhurtas | **holds** | worst 0.040 ms |
| Abhijit is void on a Wednesday and effective on every other day | **holds** | 0 of 55 disagree |
| Brahma muhurta is the fourteenth muhurta of the night that ends at this sunrise | falsified | worst 27.563 s |
| Brahma muhurta sits before this sunrise but is sized from the night after the day | **holds** | worst 0 s, exactly |

Brahma muhurta ends before sunrise, so the night it belongs to is the
night that *ends* at that sunrise. The engine puts it there and sizes it
from the night that *follows* the day — the same night its choghadiya
divide. The two nights differ by the change in the length of the
daylight from one day to the next, and that moves the start of Brahma
muhurta by a median of 10.023 s and at worst 27.563 s.

Small, systematic and wrong: the SDK sizes it from the night it is in,
and this becomes a row of the deliberate-difference registry rather than
a defect either implementation has to keep.

## 6. The two lunar months are one month and a rule

The section names the month twice, once under each convention, and the
relation between them is arithmetic: a purnimanta month begins at the
full moon, so through the dark fortnight it is already the next amanta
month.

| proposed rule | verdict | measured |
|---|---|---|
| the purnimanta month is the amanta month, plus one through the dark fortnight | **holds** | 0 of 55 disagree |
| the intercalation rule can be checked here | untested | 2 adhika and no kshaya month over 55 days |

`calendars.lunar_month` is the knob and it already exists. What the
corpus does not settle is the intercalation: 2 of the 55 days are marked
adhika and none is marked kshaya, which shows the field is computed and
does not test the rule that computes it. That belongs to the Indian
lunisolar calendar's own page, not here.

## 7. Panchaka follows the Moon's nakshatra, and its kind is a table

Two rules are proposed for when panchaka runs, and the corpus cannot
tell them apart: they differ only while the Moon is in the first half of
Dhanishtha, which is in Capricorn, and no recorded day has it there.

| proposed rule | verdict | measured |
|---|---|---|
| panchaka runs while the Moon is in the last five nakshatras | **holds** | 0 of 55 disagree |
| panchaka runs while the Moon is in Aquarius or Pisces | **holds** | 0 of 55 disagree |
| a day separates the two: the Moon in Dhanishtha's first half | untested | 0 of 55 days |

Its kind is a function of the nakshatra alone — not of the classical
remainder of tithi, vara and nakshatra, which no numbering of the three
reproduces over the 9 days that carry one:

| nakshatra | panchaka |
|---|---|
| Dhanishtha | Mrityu Panchaka |
| Shatabhisha | Agni Panchaka |
| Purva Bhadrapada | Raja Panchaka |
| Uttara Bhadrapada | Chora Panchaka |
| Revati | Roga Panchaka |

The SDK ships the nakshatra rule, which is the one the texts state, and
the design page records that the corpus does not test the half nakshatra
where the two part.

## 8. The muhurta yogas are the one limb the corpus cannot settle

Five are recorded as flags. They fire 13 times, on 12 of the 55 days:

| yoga | days | on |
|---|---|---|
| amrit_siddhi | 0 | — |
| sarvartha_siddhi | 5 | Thursday Magha, Friday Uttara Bhadrapada, Tuesday Dhanishtha, Saturday Purva Bhadrapada, Sunday Pushya |
| siddha | 7 | Monday Pushya, Thursday Anuradha, Sunday Mula, Friday Mula, Monday Mula, Monday Mula, Tuesday Revati |
| dwipushkar | 0 | — |
| tripushkar | 1 | Saturday Purva Bhadrapada |

A vara-and-nakshatra yoga is a table of seven rows by twenty-seven
columns. With 12 positive days there is nothing to derive one from, and
the corpus's positives do not match the published tables: the standard
Sarvartha Siddhi table reproduces one of the engine's five and adds nine
of its own, and no rotation of the weekday or of the nakshatra index
does better than four wrong. Two things the corpus does settle:

1. **The nakshatra is read at sunrise, not across the day.** Reading
   any nakshatra of the day into the Amrit Siddhi pairs fires it on
   four days the engine leaves clear; reading only the nakshatra at
   sunrise agrees with the engine on all fifty-five.
2. **Tripushkar's classical rule holds**: a Bhadra tithi on a Sunday,
   Tuesday or Saturday in a three-footed nakshatra. The corpus has one
   such day and the rule agrees exactly, with no false positives.
   Dwipushkar never fires, so its rule is carried and untested.

So the design ships the yogas as cited tables with the catalogue's own
confidence marks, and reports them as **intervals**, which the corpus's
flags reduce to: a yoga holds while the nakshatra that makes it holds,
and the engine's flag is that interval containing sunrise.

## 9. The Moon's rise and set are the civil day's, not the panchanga day's

Everything else in the section is bounded by sunrise. The Moon's rise
and set are not: 24 of the 108 recorded ones fall outside the window the
limbs occupy, spread from -5.67 to 19.82 hours after sunrise, and not
one of them falls before the local civil midnight. The Moon's change of
sign, by contrast, is inside the window every time.

| proposed rule | verdict | measured |
|---|---|---|
| the Moon's rise and set lie inside the panchanga day | falsified | 24 of 108 disagree |
| the Moon's rise and set are the first at or after the local civil midnight | **holds** | 0 of 108 disagree |
| the Moon's change of sign lies inside the panchanga day | **holds** | 0 of 14 disagree |
| uttarayana runs while the sidereal Sun is in Capricorn to Gemini | **holds** | 0 of 55 disagree |

Two windows in one section is a defect and not a convention: an almanac
reader cannot tell which one they are looking at. The SDK reports the
Moon's events over the window the rest of the day uses, and
`conformance-baseline` carries the engine's choice so that the corpus
still reproduces.

The disha shool is the vara's, one direction each:

| vara | disha shool |
|---|---|
| Monday | east |
| Tuesday | north |
| Wednesday | north |
| Thursday | south |
| Friday | west |
| Saturday | east |
| Sunday | west |

## 10. The daily limbs are geocentric and the natal ones are not

The corpus records the limbs twice: once for the birth instant in
`panchanga`, and once as the day's spans here. Where the birth falls
inside the window the two can be compared, and they ought to agree —
the same limb at the same instant.

| proposed rule | verdict | measured |
|---|---|---|
| the day's spans and the natal block classify the birth instant alike | falsified | 5 of 136 disagree |

The 5 that differ are all within minutes of a boundary:

c015 nakshatra, c019 tithi, c019 karana, c026 karana, c049 nakshatra

That is the lunar parallax, which reaches about a degree and so moves a
tithi boundary by up to two hours; it is entry 1 of the
deliberate-difference registry — the natal panchanga uses the
topocentric Moon and the daily one is geocentric, as every almanac is.

So the frame is a decision the daily panchanga has to make and cannot
inherit: a profile that computes a chart topocentrically still wants a
geocentric almanac. The design adds one knob for it, defaulting to the
geocentric reading, so the choice is in the settings hash rather than
buried in the module.

## 11. A period that cannot exist is recorded as a period of no length

| day | daylight | rahu kaal | first hora | thirteenth hora | first night choghadiya |
|---|---|---|---|---|---|
| c028 (1988-06-21) | 24.000 h | 3.000 h | 2.000 h | 0.000 h | 0.000 h |
| c029 (1988-12-21) | 0.000 h | 0.000 h | 0.000 h | 2.000 h | 3.000 h |

Under polar day the twelve horas of the night are twelve intervals of no
length; under polar night the twelve of the daylight are. An empty
interval is not the absence of a period — it is a period claiming to
begin and end at one instant, and code that asks which hora it is gets
an answer out of it.

The SDK reports absence as absence. `time::local_day` already carries
`DayState::Polar` with the policy that synthesised the bounds, and the
daily panchanga's periods are absent where the arc they divide does not
exist — which is why the value is a structure of options and not a
structure of intervals.

## 12. The catalogue already carries what the limbs print

Beside each span the corpus prints what the member *is* — the tithi's
paksha and class, the nakshatra's muhurta nature, the yoga's
auspiciousness, whether the karana is Vishti. Every one of those is an
attribute the SDK's catalogue already declares, so the comparison is
whether two independently sourced tables agree.

| proposed rule | verdict | measured |
|---|---|---|
| `karana.is_vishti` is the SDK catalogue's attribute of that member | **holds** | 0 of 164 disagree |
| `nakshatra.nature` is the SDK catalogue's attribute of that member | **holds** | 0 of 112 disagree |
| `tithi.group` is the SDK catalogue's attribute of that member | **holds** | 0 of 109 disagree |
| `tithi.paksha` is the SDK catalogue's attribute of that member | **holds** | 0 of 109 disagree |
| `yoga.nature` is the SDK catalogue's attribute of that member | **holds** | 0 of 115 disagree |

They do, member for member, over every span of every day. So the daily
panchanga adds no attribute table of its own: it names members, and what
a member is comes from the catalogue, in whatever language the caller
asked for. The corpus prints tithi.number that the catalogue does not
carry.

## 13. What this decides

1. **The day is a window, and the window is a `LocalDay`.** Sunrise to
   the next sunrise, on all 55 days, for every field but two. Nothing
   in the daily panchanga needs a day model of its own.
2. **Every period is a proportional division of an arc**, and there are
   only two arcs: the daylight and the night. One divider serves the
   eighths, the choghadiya, the horas and the muhurtas.
3. **The choghadiya is the hora's walk.** It ships as a seven-row table
   of names over a function `crates/time` already has, not as a grid of
   fifty-six.
4. **A limb is a span, and the corpus's spans are clipped.** The SDK
   carries the member's own bounds as well, because "the tithi ends at
   14:32" and "the tithi began yesterday at 21:05" are both things an
   almanac prints and a clipped view answers only one of them.
5. **The frame is a knob.** The daily panchanga is geocentric where the
   chart may not be, and the difference reaches the limb a birth falls
   in.
6. **Two rows join the deliberate-difference registry**: Brahma muhurta
   sized from the following night, and the Moon's rise and set taken
   over the civil day inside a sunrise-bounded section.
7. **The muhurta yogas need a rank-1 source.** The corpus settles the
   convention they are read at and one of the five rules, and nothing
   else about them.
