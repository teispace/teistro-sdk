# The rashifal's reading voice (SR-152)

Status: `building`, 2026-11-24; S1 (the plan, in every binding) built,
S2 (the en and ne packs) next.

A second register for the rashifal, `reading`, beside the technical one the
[`rashifal`](rashifal.md) composer says ("Saturn in the 12th: Sade Sati
rising"). It is what the national dailies and calendar apps print: a headline,
a short reading, a plain "why", four areas with a band word and a note, and
the lucky items. It reads the same facts and judges nothing new.

The consumer's brief is addendum 13 §1 (tithipatra hub,
`docs/04-architecture/sdk-briefs/teistro-teimeris-brief-addendum-13.md`). Its
decision D-324: every domain word lives in teistro, and the wording loads as
data, so a wording fix needs no code release.

## What the texts decide, and what they do not

No text writes a rashifal (C361). The texts decide each **fact**: a graha's
house from the sign and its verdict under vedha (gochar, Phaladeepika ch.
26), Saturn's phase (C149), and an ingress or station and whether its house
is good (v. 2). The **score** and its band are the baseline's formula, which
is unsourced (C361), and the band's lines are the consumer's design
(`rashifal.md`, "The score band"). The **wording** is the consumer's voice:
its survey of the dailies (hub `docs/02-research/notes/rashifal-voice.md`),
reviewed by native readers (Q42).

So the reading voice keeps them apart. A plan says which facts each sentence
rests on, the facts are the SDK's, and the sentences are a pack's.

## The shape

One `ReadingPlan` per sign and period, in the order a page shows it:

| field | what it holds | rests on |
|---|---|---|
| `headline` | one item | the overall band and the strongest influence |
| `parts` | the reading: a day 2 to 3 items, a week 4 to 5, a month and a year one short paragraph per part | each item its own facts |
| `why` | one item: the strongest influence in plain words | the first key influence |
| `areas` | four `AreaReading`s: WORK, MONEY, LOVE_FAMILY and HEALTH, each with its band and a one-item note | the area's band, then the strongest transit feeding it |
| `lucky` | colour, number, day and direction, as entity slots | the sign's lord (below) |
| `syllables` | the sign's nine name syllables (चु, चे, चो, ला…), as entity slots | the nine nakshatra padas the sign holds |

An `Item` is the interpret crate's: a message key and its parameters. A
reading item adds two fields:

- **`variant`**, the wording chosen among the pack's variants for that key
  (below);
- **`rests_on`**, the facts it says, each a `Fact`:
  - `TRANSIT { graha }`: the graha's house and verdict from the sign;
  - `SATURN`: Saturn's standing, its phase or spell;
  - `EVENT { index }`: the n-th event of the reading;
  - `INFLUENCE { index }`: the n-th key influence of the score;
  - `BAND { area }`: an area's band, the overall's under `OVERALL`;
  - `LUCKY` and `SYLLABLES`: the sign's own items.

Every index names a value in the `RashiReading` and `BaselineScore` that the
same answer carries, so a checker (and the consumer's later AI writer, which
is checked against these facts) can trace each sentence. A test holds the
invariant: every item names at least one fact, and every fact it names
exists in the answer.

### The four areas

The baseline scores eight life areas (`LifeArea`), fed by houses
(`baseline.rs`, `areas_of`). The reading's four are read from them:

| reading area | baseline areas | houses that feed it |
|---|---|---|
| WORK | CAREER | 3, 6, 10, 11 |
| MONEY | FINANCE | 2, 11 |
| LOVE_FAMILY | RELATIONSHIPS and FAMILY, the mean of the two | 2, 4, 5, 7 |
| HEALTH | HEALTH | 1, 6, 8, 12 |

Each area's band is its score read under the same `rashifal.score_bands` as
the overall, so the area's word and the overall's word come from one rule.
LOVE_FAMILY's mean is rounded as the overall is (C2359). The other four
baseline areas (EDUCATION, SPIRITUALITY and the overall's own houses) are
not shown; the overall band already reads them.

### Intents: what the SDK chooses, and what a pack writes

The voice guide (hub `docs/02-research/notes/rashifal-voice.md`) writes each
language natively from its own pattern bank, so one sentence key cannot serve
every language. The plan therefore names an **intent**: what is said and on
which facts. Each language's pack maps the intent to its own patterns. The
SDK decides everything a checker must trust:

- **which facts are said**: guide §8.4, in code. The opening is the band or
  the area split (the two best areas and the worst). The reading takes each of
  those areas from its strongest fact. The why is the strongest good influence
  in the guide's order (Jupiter, Saturn, Rahu, Ketu, the Sun, Mars, Venus,
  Mercury, the Moon), or the Moon when it changes sign that day, or Saturn when
  its phase changes in the period; on a CARE day, the strongest not-good fact
  worded as care. The closing is the band or the lucky items, never a caution;
- **how firmly**: the hedge is part of the intent, from the facts (§5.2):
  FIRM when the band is EXCELLENT and two facts agree, LIKELY on one good fact,
  POSSIBLE when obstructed, and MAY for every caution. Nothing is certain;
- **how many cautions**: at most one a day, two a week, one per half of a
  month and one per part of a year (§7.2). Each is paired with what still
  works and is never last;
- **who is named**: grahas only in the why and in the events, and the Moon
  only on a day.

A pack's message for an intent (`reading.rashifal.area.WORK.GOOD`,
`reading.rashifal.why.TRANSIT`, `reading.rashifal.caution.HOUSE_8` and so on)
holds that language's patterns as variants. The pack chooses the words; it
never chooses a fact.

### The parts of each period

| period | parts | from |
|---|---|---|
| day | 2 to 3 items; split at the Moon's ingress when it changes sign that day, with its time | the day's reading and the Moon's events, which the reading voice asks for on a day even though a score never counts them (C2331) |
| week | start, middle and end, from the daily scores of its days; the week's best day when one day scores highest for the sign | seven daily readings of the same sign |
| month | two halves, split at the month's largest ingress or station (Saturn's, Jupiter's and the nodes' first), dated | the month's events |
| year | the slow grahas' standing (Jupiter, Saturn with its sade sati or dhaiya and their dates, Rahu and Ketu), then each quarter's events | the year's events |

A longer period never contradicts a shorter one (the guide's D-313 guard). A
week names a best day only when that day's own score is the week's highest,
and ties name none.

### Choosing a wording

A pack carries several wordings of each intent. The plan names one by
`variant`, chosen by a seed:

- the seed hashes the period's first civil date, the sign and the period
  kind, so the same day always reads the same, everywhere and in every
  binding;
- within a reading, one key's variants are taken without repeating until
  every one is used, so a week of five items never says one line twice;
- a day's reading avoids the variant the previous day used for the same key
  where the pack has more than one, so consecutive days differ.

The variant is a number the plan carries, not a string: the wording lives in
the pack, and a pack with fewer variants reads the number modulo its count.
So a pack may add wordings without changing any answer but the words.

### The lucky items

The baseline's table (`baseline.rs`, `lucky`) is keyed by the sign's lord.
Each item is traced to that graha's own attribute in the texts:

- **direction**: the graha's direction, BPHS ch. 3 (Sun east, Venus
  south-east, Mars south, Rahu south-west, Saturn west, Moon north-west,
  Mercury north, Jupiter north-east). The table matches.
- **day**: the graha's own weekday, the vara it lords. The table matches.
- **colour**: the text's colour of the graha, as the consumer asked (it
  replaces the baseline's table). Brihat Jataka ch. 2 (v. 4 or 5, by
  edition) gives the seven: the Sun copper, the Moon white, Mars deep red,
  Mercury green, Jupiter yellow, Venus variegated and Saturn black. The Narada
  Purana's list agrees. Against the baseline's table, the Sun (orange), Venus
  (white) and Saturn (blue) change. The `colour` kind gains COPPER, VARIEGATED
  and BLACK, appended. The verse number is read from N. Chidambaram Iyer's
  1885 translation (public domain) before S4 ships (C2361).
- **number**: the graha's number is numerology's, not jyotisha's: Cheiro's
  single numbers of the planets (p. 22), which `teistro-numerology` already
  cites (Sun 1, Moon 2, Jupiter 3, Mercury 5, Venus 6, Saturn 8, Mars 9). The
  table matches; the item names Cheiro as its source, apart from the texts'.

The consumer asked whether the items may vary by the day's lord or the Moon's
nakshatra. They vary only if a text says so; until then they are the sign's,
the same every day, as the baseline's are.

### The syllables

A sign holds nine nakshatra padas, and each pada has its name syllable
(`teistro_core::catalogue::Nakshatra`, spelled as Muhurta Chintamani p. 173
prints it, C299). The plan lists
the sign's nine in the padas' order, as entity slots a pack writes in its own
script and convention. Nepali prints the short-vowel forms and Hindi the long
ones, both from the same IAST syllable, so the spelling rule is the locale's
and the syllable is the SDK's. en-Latn-NP romanizes the Nepali set and names
the sign Nepal's way, "Mesh (Aries)" (C2332).

## Where it lives

- **`teistro-interpret::rashifal_reading`**, behind a `rashifal-reading`
  feature, off by default, so a build without it carries no code and no
  wording for it. It reads a `RashiReading` and its `BaselineScore`, and
  nothing else.
- **The messages** are keyed under `sdk.reading.rashifal.*` in a pack of their
  own (`.tbundle`, `intl.loadPack`), en and ne first, then hi and sa. Every
  variant ships marked unreviewed until the consumer's native reviewers sign
  it off (Q42, SR-75).
- **The façade and every binding** answer `rashifal.reading(...)` with the
  plan as data: the items, their variants and their facts. A binding says
  the plan through its intl engine as it says any plan.

## Slices

1. **S1, the plan:** the shape above, the facts, the areas, the seed, the
   syllables and the lucky items as the baseline has them, with keys and no
   wording; in every binding, with the tracing test. The consumer wires its
   layout against it.
2. **S2:** en and ne wording packs, from the consumer's voice guide.
3. **S3:** hi and sa.
4. **S4:** the lucky items traced item by item, as above.

## Cruxes

| # | question | decided | why |
|---|---|---|---|
| C2359 | LOVE_FAMILY's band | the mean of RELATIONSHIPS and FAMILY, rounded as the overall is | the consumer asked for one area; the mean keeps both houses' weight. Rejected: the lower of the two (it makes the area read worse than either half would on its own page) |
| C2360 | the wording's choice | a seeded variant number in the plan, the wording in the pack | the same day reads the same in every binding, and a pack can add wordings without a code release |
| C2362 | sentence keys across languages | intents chosen and traced in Rust; each language's pack maps an intent to its own patterns | the guide writes each language natively with different pattern counts (82 ne, 46 hi, 46 en, 16 sa), so a shared sentence key would force translation. Rejected: patterns with their fact conditions in the pack (the choice of fact would then live in data no test traces) |
| C2363 | the week's best day | named only when one day's own score is the week's strict highest | the consumer asked for it only when computed; a tie names none, so a week never contradicts its days |
| C2361 | the lucky items | the sign lord's, traced to the graha's own attributes; an item no text gives is left out | the consumer asked for traceable items and for none made up |
