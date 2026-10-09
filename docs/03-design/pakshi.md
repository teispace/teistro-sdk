# `pakshi`: Pancha Pakshi, the five birds

Status: `draft`, 2026-10-09. Track B of the completion plan
(`07-roadmap/00-roadmap.md`), the module catalogue's `pakshi` row. The
kernel, `crates/pakshi`, and the façade are built; the boundary and the
bindings are the next step (§7).

## 1. Sources

Pancha Pakshi is a Tamil system: a native's bird, fixed by the birth star,
does one of five things in each fifth of the day and night, and what it is
doing says whether a time is good to begin something.

| short | source | status | use |
|---|---|---|---|
| **AG** | Agastya, *Pañcapaṭci Sāstiram* with its charts and the *Ñānacara Nūl*, Kanchipuram 1880, reviewed by Tiruvalur Tyagaraja Swamigal (Internet Archive `dli.rmrl.007543`, Roja Muthiah Research Library; cited by PDF page) | public domain | the verses: the sequences, the first eaters, the death birds, the sub-period lengths, the relations |
| **AG07** | the same work with Santhalinga Swamigal's commentary, 1907 (Tamil Virtual Academy) | public domain | the birth-star verses with a prose gloss (pp. 60–61) |
| **AY** | P. V. Jagadisa Ayyar, *South Indian Customs*, Madras 1925, ch. "Pancha-Pakshi Sastram", pp. 100–109 | public domain in the US; a table of facts in any case | the only open English source with all four activity tables |
| **PUL** | U. S. Pulippani, *Biorhythms of Natal Moon*, 1993 | in copyright: facts only, never text | the modern standard: the dark half's birth bird, per-half sub-period lengths, the neighbour relations, and the worked examples |

Earlier and later prints of Agastya (1863, 1867, 1879, 1916), Bogar's and
Romarishi's *vināḍi* school and the Sanskrit and Telugu manuscripts are
named in the research notes and not collated. Web pages on the subject
contradict their own rules and are not cited.

## 2. The rules

The birds, in order: the vulture (*vallūṟu*, Ayyar's hawk), the owl, the
crow, the cock, the peacock (AY p. 102; AG p4 v. 5). The activities:
eating, walking, ruling, sleeping, dying. Ruling and eating are good,
walking middling, sleeping and dying bad (AY p. 102; PUL pp. 18–19).

**The native's bird** is the birth star's: Ashvini to Mrigashira the
vulture's, Ardra to Purva Phalguni the owl's, Uttara Phalguni to Vishakha
the crow's, Anuradha to Uttara Ashadha the cock's, Shravana to Revati the
peacock's — groups of 5, 6, 5, 5 and 6 (AY pp. 104–105; AG07 p. 60). PUL reverses the
birds over the same groups for a birth in the dark half; the
public-domain texts give one table (crux P1).

**The day.** A day runs from sunrise to the next: its day half to sunset,
its night half on to sunrise, five yamas each (AG p5 v. 9). Both halves
belong to the weekday of the sunrise that began them (PUL p. 17; P9), and
the day is read in the paksha at that sunrise (P2).

**What a bird does in a yama** is printed as four tables, one per half of
each paksha, each with five weekday columns (AY pp. 106–109; AG pp. 6,
10, 16–18, 22, 26). They are one rule. Each half has a sequence —

| half | sequence |
|---|---|
| bright day | eating, walking, ruling, sleeping, dying |
| bright night | eating, ruling, dying, walking, sleeping |
| dark day | eating, dying, sleeping, ruling, walking |
| dark night | eating, sleeping, walking, dying, ruling |

— and each weekday's half a bird that eats in its first yama (AG p15 v. 1,
p20 v. 1, p25 v. 1). Every bird steps one place along the sequence a
yama, and neighbouring birds stand a fixed number of places apart: one in
the bright day, two in the dark day, four at night. So

```
activity(bird, yama) = sequence[((bird − first_eater) · step + yama) mod 5]
```

which `tables::activity` computes. It was checked against every cell of
Ayyar's four tables, transcribed from the page images, and gives all of
them but one, which is his slip (P7). The test holds the rule to the
transcription cell by cell, so a misread cell or a wrong rule fails it.

**Sub-periods.** Each yama divides into five sub-periods. The native's
bird does its own main activity first and the rest follow its half's
sequence round, each owned by the bird whose main activity it is (PUL
p. 37, with a worked example; P5). Agastya gives each activity one
length for both pakshas — eating 1¼ nazhigai, walking 1½, ruling 2,
sleeping ¾, dying ½ of a yama's six (AG p5 vv. 7–9, p17); PUL prints
other lengths for three of the four halves (P4).

**Death days.** On each weekday one bird is dead for the whole day and
night (AG p15 v. 2, p25 v. 2; PUL p. 57). It is a separate rule from the
tables and is reported beside them, not over them (P10).

**Relations.** Agastya lists each bird's friends and enemies, directed —
the owl counts the vulture a friend, the vulture the owl an enemy (AG
p51, p40 v. 13). PUL makes each bird the friend of its neighbours round a
cycle, V–O–C–K–P in the bright half and V–C–O–K–P in the dark (pp. 48,
121; its bright entry for the cock is a slip, which the symmetry shows).
The two are different relations and are never mixed (P6). The birds'
natural strengths are the crow's whole, the vulture's three quarters,
the owl's half, the cock's quarter and the peacock's eighth (AG p40).

## 3. The printed slips

The rule exposes each by the cell it disagrees with:

- **AY p. 107**, a bright Saturday night's fifth watch, prints the first
  watch again, so the vulture rules twice and never eats. The rule gives
  the vulture eating, as PUL p. 45 does.
- **PUL p. 115** prints the dark Sunday's third yama as a copy of the
  fourth; its Table 1 is broken from row 18; its bright friend list for the
  cock contradicts its own enemy list.
- **AG p26**, the dark-day chart, has headers in the dark sequence and
  cells that match Ayyar only read in the bright one; Agastya's own verses
  on pp. 25–26 agree with Ayyar. Which of the two is mis-set needs a Tamil
  reader (§6).

## 4. Worked examples

The kernel's acceptance tests, from PUL p. vii's two dated examples (the
facts only), both reproduced by Ayyar's public-domain tables:

1. 31 October 1984, a bright Wednesday: the cock (Uttara Ashadha) sleeps
   in the day's second yama and dies from the third.
2. 21 May 1991, a bright Tuesday night: the owl (Purva Phalguni) dies in
   the night's second yama, 20:24 to 22:48 by an even day.

And PUL pp. 60–61's sub-periods: a bright Sunday's third yama runs the
vulture's ruling, the owl's sleep, the crow's death, the cock's eating and
the peacock's walking, for 48, 18, 12, 30 and 36 minutes, and a peacock
native walks first, 10:48 to 11:24.

## 5. The baseline engine

Its Pancha Pakshi service is not a reference. Its birth bird groups the
stars by fives and shifts a night birth, which no source does (Purva
Phalguni comes out the crow, the sources' owl). It fixes each yama's
activity by the yama's index in every half, so the night's and the dark
half's sequences are missing; its weekday starts match the sources only
on Sunday and Monday; and its "current activity" does not depend on the
time asked about. Parity with it is declared as a departure, not measured.

## 6. For a Tamil reader

The kernel ships on the readings above. These questions are handed out
rather than guessed:

1. AG p26: are the headers or the cells mis-set? Collate with the 1863,
   1867, 1879, 1907 and 1916 prints.
2. AG p5 vv. 7–9 and p17: are the sub-period lengths per activity for
   every half, and in what order do a yama's sub-periods run?
3. AG p15 v. 2: the word read for Sunday's and Friday's bright death bird.
4. AG p20: what *kaṭainilai* means in the verse listing the night eaters.
5. AG07 p. 60 v. 22: the commentary's count of the cock's stars, and
   where the owl's begin.
6. AG07 p. 61 v. 26: the dark half's sign birds.
7. AG pp. 7–9: is a day's table chosen by weekday or by the tithi group
   its columns are also named for?
8. AG p51 and p40 v. 13: the friend and enemy lists.
9. Any verse that reverses the birth bird in the dark half.

## 7. The model and the order of work

`crates/pakshi` depends on `teistro-core` alone. A `Day` is its sunrise,
sunset and next sunrise, its weekday and its paksha; the façade reads them
from the almanac.

| item | what it is |
|---|---|
| `birth_bird(nakshatra, paksha, BirthBird)` | the native's bird |
| `tables::{activity, first_eater, sequence, subs, death_bird, relation}` | the rules, each cited |
| `read_day(&Day, Bird, &Rules) -> Reading` | the ten yamas as instants, each with the bird's activity, its quality and its timed sub-periods with their owners and how the native regards them; the death bird and the first eaters |
| `now(&Reading, instant)` | the yama and sub-period running then |
| `Rules` | `clock` (P3), `subs` (P4), `relations` (P6) |

1. The kernel with the rule held to every printed cell and the worked
   examples as tests. **Built 2026-10-09.**
2. The façade: the native's bird from a chart's Moon and paksha, and a
   day's reading at a place from the almanac's sunrise, sunset and tithi.
   **Built 2026-10-09** as `AlmanacArea::pakshi(date, place, offset,
   bird, &Rules)` and `ChartArea::pakshi_bird(document, BirthBird)`; a
   day the Sun does not both rise and set is refused as `UNSUPPORTED`,
   and the bird is held to the almanac's own nakshatra and tithi at the
   birth.
3. The boundary (JSON), the bindings and parity, and a guide.
4. Names for the birds and activities in every strict locale, vetted
   first; the catalogue kinds come with them.

## 8. Cruxes

| id | question | decided | why |
|---|---|---|---|
| P1 | the birth bird in the dark half | reversed (PUL), the single table a choice | modern practice and both worked examples; every source agrees for the bright half |
| P2 | the paksha a day is read in | the one at its sunrise | the day runs from sunrise, as the weekday does |
| P3 | how long a yama is | a fifth of the real day or night, six nazhigai from sunrise a choice | AG defines the yama as a fifth of the day; both examples hold either way |
| P4 | the sub-periods' lengths | Agastya's, one per activity; PUL's per-half figures a choice | AG states them "for the waxing and the waning"; PUL's are undocumented |
| P5 | the sub-periods' order | the native's own activity first, then the sequence round | the one fully specified reading, with a worked example |
| P6 | friends and enemies | Agastya's directed lists; PUL's cycles a choice | the public-domain source; the two are never mixed |
| P7 | AY p. 107's fifth watch | the rule | PUL p. 45 confirms it; the printed cell repeats the first watch |
| P8 | the activities' weights | not scored: the activity and its quality are reported | AY and PUL weigh differently; clauses, not a score |
| P9 | the weekday of a night | the weekday of the sunrise that began it | PUL p. 17; the Tamil day runs from sunrise |
| P10 | a death day against the tables | both reported | the texts give them as separate rules |
| P11 | the name, sign and tithi birds | not in v1; `BirthBird` is an enum, so each can be added | named in AG and AG07, not yet read closely enough |
