# The festival rule language: what it lacks

Status: `design`, 2026-10-21. Built: `MOONRISE` (§3.1), `DAY_MUHURTA`
(§3.2), `GREATER` with the maintainer's rule (§3.3), `dayOfPaksha`
(§3.4), `ON_VARA` with `NONE` (§3.5), the civil dates and the Sun's
entry into a nakshatra (§3.6) and the four-valued `inAdhika` (§3.7); the
rest is not. Cruxes C601 to C604, C606, C608, C614, C616 to C619, C696
(`01-research/feature-universe/19-verification-cruxes.md`).

`festival-rules.md` §4 defines the language a festival rule is written in:
a lunar date, a window, guards over a closed set of predicates, and a
fallback. That language holds every rule *Dharmasindhu* gives a table for,
and the 58 rules fitted to Nepal's national panchanga (§10). This page
designs what it lacks. Each form is sourced and written as the JSON a
request would carry. The test of each is the reviewed table's 57 excluded
rows (§4): which form would express each row, or why none does.

## 1. What the language holds today

| part | members |
|---|---|
| window | `SUNRISE`, `SUNSET`, a fifth of the daylight (`PART`: pratah, sangava, madhyahna, aparahna, sayahna), `PRADOSHA`, `NISHITHA`, a night muhurta (`NIGHT_MUHURTA`, 1–15); since this page, the moonrise (`MOONRISE`, §3.1) and a range of day muhurtas (`DAY_MUHURTA`, §3.2) |
| what is kept | a tithi in a month, or every month; a nakshatra in a paksha |
| cases | `EARLIER_ONLY`, `LATER_ONLY`, `BOTH` (both wholly), `NEITHER`, `EQUAL_PARTS`, `UNEQUAL_PARTS` |
| predicates | `CASE`, `JOINED` (the tithi with a nakshatra), `STANDS` (a nakshatra in a window), `LASTS` (ghatis past an edge), `WHOLLY`; since this page, `GREATER` (§3.3) |
| choice | `EARLIER`, `LATER`, `BY_YUGMA` |
| the adhika month | `inAdhika`: the nija month only, or both; since this page, the adhika alone, or the adhika when the month is doubled (§3.7) |
| counted days | a following rule: 0 to 15 civil days after another rule's day |

## 2. How a form is chosen

A form is added when a source states the rule, or when one form settles
several festivals the language cannot say today. A window that only fits
one festival's verified days better is a candidate. It is not a reason to
add a form: with a dozen windows to choose from, one of them fits any
fourteen days a little better, so §4 marks such a window "a candidate
only".

## 3. The forms

### 3.1 Moonrise: `MOONRISE`

```json
{"window": "MOONRISE"}
```

An instant: the first moonrise after the day's sunrise and before the
next. The cases read it as they read `SUNRISE`. A day the Moon does not
rise on, which happens only far from the equator, leaves the occurrence
unjudged and says so: the text does not say what such a day holds, and
reading it as holding nothing would decide by default.

**Sources.** *Dharmasindhu* p. 9 (OCR ll. 2160–2162): संकष्टचतुर्थी तु
चन्द्रोदयव्यापिनी ग्राह्या · परदिने चन्द्रोदयव्याप्तौ परैव · उभयदिने
चन्द्रोदयव्याप्तौ तृतीयायुतैव ग्राह्या · दिनद्वये चन्द्रोदयव्याप्त्यभावे
परैव. The tithi at moonrise; the later day when the later's moonrise
holds it; the one joined to the 3rd, the earlier, when both do; the later
when neither does. The Gauri and Vinayaka chaturthi on the same page are
madhyahna rites. p. 76 (l. 5070) keeps Ashvina's dark 4th, the Karaka
chaturthi, at moonrise and decides it "as the Sankashta chaturthi", and
p. 44 the Ashunya-shayana vrata at moonrise.

```json
{"key": "SANKASHTA_CHATURTHI", "tithi": "KRISHNA_CHATURTHI", "inAdhika": true,
 "at": {"window": "MOONRISE"},
 "decide": [{"when": [{"is": "CASE", "case": "EARLIER_ONLY"}], "choose": "EARLIER"},
            {"when": [{"is": "CASE", "case": "BOTH"}], "choose": "EARLIER"}],
 "otherwise": "LATER"}
```

The draft of this page guarded `LATER_ONLY` and fell through to the later
day, which sent a 4th held only at the earlier day's moonrise to the
later; the first clause, the tithi at moonrise, gives it the earlier.

**Built.** `DHARMASINDHU` ships `SANKASHTA_CHATURTHI` (every month, the
adhika too) and `KARAKA_CHATURTHI` (Ashvina). `NEPAL` and
`NEPAL_PANCHANGA` leave both out, since the committee prints neither.
A day's moonrise is searched only when a rule reads one, in lockstep
over the range, the search the panchanga makes over a sunrise day, so a
day's moonrise is its panchanga's first to the bit. Over 2025 at Delhi
the rule gives the twelve Sankashti days a published almanac lists for
New Delhi (rank 3), and the Karaka chaturthi on Karva Chauth, 10 October
(`crates/sdk/tests/festival.rs`).

**Settles:** Kashi bada Ganesh (Magha's dark 4th, 12 of 14), and
Ramanujacharya's jayanti as a candidate.

**Corrected (C684).** This page first said moonrise settles the monthly
चन्द्रोदय row. Read off the committee's pages, every one of its 25 rows
falls on the day after the new moon: it names the first evening the
crescent is seen, not a fast at moonrise. `NEPAL_PANCHANGA` keeps it as
`CHANDRODAYA` with the forms the language already has: the bright 2nd,
the earlier day when it holds the last muhurta before sunset wholly
(`DAY_MUHURTA` 15 to 15, `WHOLLY`), the later otherwise, 25 of 25 over
both skies.

### 3.2 A range of day muhurtas: `DAY_MUHURTA`

```json
{"window": "DAY_MUHURTA", "from": 8, "to": 9}
```

The daylight in fifteen muhurtas, `from` to `to` inclusive, 1 to 15. A
fifth of the daylight is three muhurtas. This window names the spans the
fifths cannot.

**Sources.** *Dharmasindhu* p. 265 divides the day into fifteen muhurtas
and names the 7th gandharva, the 8th kutapa and the 9th rauhina. Shraddha
belongs to aparahna. p. 36, citing the *Purusharthachintamani*, makes the
7th to the 9th the time for the yugadi shraddha. Kutapa to rauhina is
`{"from": 8, "to": 9}`.

Purvahna is the case that needs this window most (C602). The two readings
need two different windows:

- The consumer reads it as the first two fifths, `{"from": 1, "to": 6}`.
  It gives the printed day on all three rows that pratah misses: Upanga
  lalita in VS 2082, and Pishachamochana in both years.
- *Dharmasindhu* p. 21 reads it as the first half of a day divided in two,
  `{"from": 1, "to": 7}`, with half of the 8th left over. That half shows
  the reading wants a fraction rather than muhurtas, so it stays a fork.

**Settles:** purvahna for the fitted rules (C602), and kutapa–rauhina.
Arjundhara mela, Deopatan Deshoddhar puja, Fagu purnima and Gaura parva
are candidates.

**Built, and measured over the committee's sky** (`festival-measured.md`
§6), where the consumer's three rows were measured over its own:

- Upanga lalita in VS 2082 moves onto the printed day.
- Pishachamochana moves in neither year. Its 14th begins mid-morning in
  VS 2082 and after midday in VS 2083, so six muhurtas give the later day
  in both, and *Dharmasindhu*'s seven fail VS 2083. It is a shraddha, and
  p. 265 keeps shraddha in aparahna. Aparahna gives the printed day in
  both years, so the rule is now kept there.
- Bala chaturdashi, which the first fifth gave in both years, falls a day
  early in VS 2082 under six muhurtas. Its 14th begins 40 minutes after
  that sunrise and ends mid-morning the next day. Its rite is the
  morning's, and the sunrise tithi gives both printed days, so the rule is
  now udaya.

### 3.3 Which day holds more: `GREATER`, and the maintainer's rule

```json
{"is": "GREATER", "day": "LATER"}
```

The predicate holds when this day's window holds more of the tithi, in
time, than the other day's window does. Equal times are a tie, and
`GREATER` holds for neither day.

**The maintainer decided:** the day with greater coverage wins, the
earlier day takes a tie, and when neither day holds the window the
earlier day wins.

The existing guards express that rule exactly in every case but one:

```json
"decide": [{"when": [{"is": "CASE", "case": "LATER_ONLY"}], "choose": "LATER"},
           {"when": [{"is": "WHOLLY", "day": "LATER"}, {"is": "CASE", "case": "UNEQUAL_PARTS"}], "choose": "LATER"}],
"otherwise": "EARLIER"
```

Case by case:

- **Earlier only**: earlier.
- **Later only**: later.
- **Both wholly**: a tie, so earlier.
- **Equal parts**: a tie, so earlier.
- **Neither**: earlier.
- **The later whole and the earlier in part**: the later holds more.
- **The earlier whole and the later in part**: the earlier.
- **Both in part, unequally**: here the guards do not express the rule.
  The tithi begins inside the earlier day's window and ends inside the
  later day's, and only a comparison of the two parts can say which is
  greater.

Over the consumer's verified years that last case occurred 37 times: 18
were kept on the earlier day and 19 on the later. No fixed choice comes
near, and `GREATER` gives the consumer's answer every time.

`GREATER` alone does not give the "both wholly" clause. By time, two
whole windows differ by the minute or two a day's length moves, so it
would give the longer of them. The maintainer's rule calls that a tie, so
a `BOTH` guard stands before the comparison:

```json
"decide": [{"when": [{"is": "CASE", "case": "LATER_ONLY"}], "choose": "LATER"},
           {"when": [{"is": "CASE", "case": "BOTH"}], "choose": "EARLIER"},
           {"when": [{"is": "GREATER", "day": "LATER"}], "choose": "LATER"}],
"otherwise": "EARLIER"
```

At an instant, `GREATER` reads what the cases read: the day that holds it
and not the other. A rule weighing two times stands while the place moves
by half their difference, and its observance's `marginSeconds` says the
lesser of that and the usual margin.

The neither clause did not decide a single verified year: no windowed
row's verified years had the case. It differs from the consumer, which
falls back to sunrise. A tithi that holds neither day's window begins after
the earlier day's window and ends before the later's. It is then running
at the later day's sunrise, so the consumer gives the later day every time
(C603).

`GREATER` compares time, as the consumer does. The evaluator's `held` is a
fraction of each day's window. The two can differ only when both days hold
part of the window and the two windows differ in length, by the minute or
two a day's length changes (C604).

**Settles:** C601. Sita navami in VS 2082 and Mha puja in VS 2083 (with
the Nepal Sambat new year counted from it) move onto the printed day, and
Govatsa dwadashi is a candidate.

**Built.** Every share fit of the `NEPAL_PANCHANGA` pack is these three
guards. Over the committee's sky the pack now parts from the print on 12
of 132 rows, down from 18, and over the modern sky on 29, down from 31.

### 3.4 Either paksha: `dayOfPaksha`

```json
{"dayOfPaksha": 13, "month": null}
```

A rule kept on the same day of both pakshas, as a third form of what is
kept beside `tithi` and `nakshatra`. 15 is the full moon and the new moon
both, so a request names `PURNIMA` or `AMAVASYA` instead.

**Source.** The committee prints a pradosha fast on the 13th of both
pakshas: 24 rows in VS 2082 and 25 in VS 2083, written शनिप्रदोषव्रत and
सोमप्रदोषव्रत on those weekdays and प्रदोषव्रत otherwise. It prints
अष्टमीव्रत 18 times in VS 2082.
Today each needs two rules, one per paksha.

**Settles:** the pradosha and ashtami fasts (§5 of the classification).

**Built, and measured over the committee's sky.** Every pradosha row of
both editions was read from the print: 24 in VS 2082 and 25 in VS 2083.
`PRADOSHA_VRATA` keeps the 13th of each paksha, the adhika months
included, with the maintainer's rule over pradosha (§3.3), and gives all
49. Sunset, sayahna and day muhurtas 13–15 give 49 too. Pradosha is the
rite's own time, so it is the window the rule names.

The plain अष्टमीव्रत is not shipped. No window gives more than 30 of its
37 printed days, and the misses fall a day early in some years and a day
late in others. Where the 8th's sunrise falls on a Tuesday or a Wednesday,
the print names that day भौमाष्टमी or बुधाष्टमी, and it can print the
plain fast on the day before (13 and 14 October 2025, 7 and 8 July
2026). So the two are decided apart, and the plain fast's rule is not in
hand.

### 3.5 The weekday: `ON_VARA`

```json
{"is": "ON_VARA", "day": "LATER", "vara": "SOMAVARA"}
```

The predicate holds when the day is that weekday. A rule kept only on a
weekday puts it in a guard whose fallback refuses: a new choice, `NONE`,
meaning no observance that month.

**Source.** *Dharmasindhu* p. 20 (page image) names Somavati amavasya, the
new moon on a Monday, for the ashvattha puja, and keeps it even when the
new moon touches only a muhurta up to aparahna. It names the 4th on a
Tuesday (Mangala chaturthi) and the 7th on a Sunday (Bhanu saptami), and
calls the new moon on a Tuesday or a Monday great for bathing and gifts.
The committee prints शनिप्रदोष and सोमप्रदोष.

**Settles:** Somavati amavasya, Mangala chaturthi, Bhanu saptami,
Budhashtami, Bhaumashtami and the weekday pradoshas (C618). Every one of
them also needs `NONE`.

**Built, and measured over the committee's sky.** One reading gives every
weekday row of both editions: the day whose sunrise holds the tithi, the
earlier when both do, kept only when that day is the vara.

| rule | printed rows given |
|---|---|
| `BHAUMASHTAMI` | 9 of 9 |
| `BUDHASHTAMI` | 5 of 5 |
| `RAVI_SAPTAMI` | 8 of 8 |
| `MANGALA_CHATURTHI` | 6 of 6 |
| `SOMAVATI_AMAVASYA` | 2 of 2 |

`SOMAVATI_AMAVASYA` also gives Monday 8 March 2027. The print names that
day दर्शश्राद्ध and does not call it Somavati. The weekday pradoshas are
not separate rules: the print names the pradosha fast by its weekday.

### 3.6 Solar and civil dates: `SOLAR_DAY`, `CIVIL_DATE`, `CIVIL_WEEKDAY`

```json
{"solarDay": {"month": "MAKARA", "day": 1}}
{"civilDate": {"calendar": "GREGORIAN", "month": 2, "day": 7}}
{"civilWeekday": {"calendar": "GREGORIAN", "month": 9, "weekday": "SHANIVARA", "nth": 2}}
```

These are three more forms of what is kept, with no window:

- **`SOLAR_DAY`** is a day of the solar month the Sun's sign names. A
  negative day counts back from the month's end: Bisket's beginning is
  the year's fourth-last day.
- **`CIVIL_DATE`** is a date in a named calendar.
- **`CIVIL_WEEKDAY`** is the nth weekday of a month.

Whether the solar month begins on the sankranti's own day follows the
profile's civil convention. In Nepal the Bikram Sambat month begins on
the sankranti's day under punya kala, the convention the SDK's calendar
already applies (C619).

**Built**, with one change: `SOLAR_DAY` is not a third form. A Bikram
Sambat month is the solar month the Sun's sign names under Nepal's
convention, so a solar day is a `civilDate` in that calendar, and its
negative day is the one `civilDate` takes. Two forms say every row:

```json
{"key": "MAKARA_SANKRANTI", "source": "...", "civilDate": {"calendar": "BIKRAM_SAMBAT", "month": 10, "day": 1}}
{"key": "SAFER_INTERNET_DAY", "source": "...", "civilWeekday": {"calendar": "GREGORIAN", "month": 2, "weekday": "MANGALAVARA", "nth": 2}}
```

A rule is a `DatedRule` in a request's `rules`, told apart by its
`civilDate` or `civilWeekday`, and its days come back as the answer's
`dated` list, each `{rule, day, on}`: the day, and the same day in the
rule's own calendar. The list is left out of an answer when empty, so
no answer that asks for none changes its hash. `NEPAL_PANCHANGA` ships
the twelve sankrantis on the first of each Bikram Sambat month, six
national days and Tamu Lhosar, read off the committee's print: every sankranti of VS 2082
on its month's first day (pp. 2–24, printed by the month's name,
"वैशाखसंक्रान्ति, नववर्षारम्भ" to "चैत्रसङ्क्रान्ति"), लोकतन्त्र दिवस on
Baisakh 11, संविधानदिवस on Asoj 3, पृथ्वीजयन्ती on Poush 27 and the
earthquake safety day on Magh 2, तमुल्होछार on Poush 15 (p. 19), सहिद
दिवस on Magh 16 (p. 21) and राष्ट्रिय प्रजातन्त्र दिवस on Phalgun 7
(p. 23), with VS 2083's Baisakh and Poush to Phalgun rows checked too (`crates/sdk/tests/festival.rs`). Each is an event in
the iCalendar feed. Sixteen festivals of a solar date follow, each on the
day both editions print: Bisket's beginning on Chaitra's fourth-last day
and its end on Baisakh 5, the Baisakh jatras of Bhaktapur, Tokha and
Thimi on Baisakh 1 to 4, Kandakarak puja and Kheer khane din on Shrawan 1
and 15, Kirtipur's Bagh Bhairav jatra on Bhadra 1, Vishwakarma puja on
Asoj 1, and Maghi and the Makar snans on Magh 1 (`SOLAR_DAYS`).

A `civilDate` on the Tibetan calendar keeps the day its observance falls
on, the first of a doubled month and of a repeated day, and the day
before a skipped number unless its `"skipped": "AFTER"` says the day
after (`calendars/tibetan.md`, The festival day). `skipped` is refused
on every other calendar, which skips no number.

```json
{"key": "CHOTRUL_DUCHEN", "source": "...", "civilDate": {"calendar": "TIBETAN_PHUGPA", "month": 1, "day": 15, "skipped": "AFTER"}}
```

**The Sun's entry into a nakshatra.** A third form keeps the civil day,
local midnight to the next, in which the sidereal Sun enters a nakshatra
in the almanac's own sky, or the day `days` after it (0 by default, at
most 15, about the Sun's stay in one):

```json
{"key": "BHUMIRAJ", "source": "...", "sunEnters": {"nakshatra": "ARDRA"}}
{"key": "BHUMI_PUJA", "source": "...", "sunEnters": {"nakshatra": "ARDRA", "days": 3}}
```

The day is the civil one, not the sunrise day, because that is what the
print keeps. The committee prints the entry itself in each page's side
column ("८ आर्द्रा १ मा सूर्य १४:५८" in VS 2082 p. 6, "२१:११" in VS 2083
p. 7), and भूमिरज on the day of that line in both editions. The SDK's
entry under `nepal_official` is the printed minute in both (14:58:04 and
21:10:40). Over 2013 to 2043 the civil day gives 29 of the 31 days a
patro printed in Nepal lists, the sunrise day 26. The two misses, 02:32
in 2023 and 00:32 in 2042, have neighbours on either side of the hour
that keep the civil day (00:51 in 2015, 01:42 in 2019, 03:23 in 2027),
so no cut-off reproduces them (C696). The Sun is never retrograde and
stays over thirteen days in a nakshatra, so the form reads its
nakshatra at each midnight of the range, `n + 1` readings for `n` days,
and only when a rule asks. `on` in the answer names the entry's day.
Nautapa (the Sun's entry into Rohini) and the Tamil Agni nakshatram are
the same form.

**Settles:** the 24 solar festivals and the 165 fixed civil ones of the
classification, Guriya parva (C621), Safer internet day's weekday rule,
and Bhumiraj and the Bhumi puja (C696).

### 3.7 The adhika month: `inAdhika`

```json
{"inAdhika": "FIRST"}
```

`inAdhika` was a yes or no. It is one of four values, and the old two are
still read and written as `false` and `true`, so a rule written before
keeps its bytes and its hash:

| value | meaning |
|---|---|
| `NIJA` | today's `false`, and the default |
| `ALSO` | today's `true` |
| `ONLY` | the adhika month alone, for Padmini (the bright 11th) and Parama (the dark 11th) ekadashi, with `month` empty |
| `FIRST` | the adhika month when the named month is doubled, the nija month otherwise |

**Source.** *Dharmasindhu* p. 38 (page image): "ज्येष्ठे मलमासे सति तत्रैव
दशहरा कार्या न तु शुद्धे", so Dashahara is kept in the adhika Jyeshtha
when Jyeshtha is doubled. The committee printed Dashahara's beginning and
end, and the Gosaikunda and Hatyamochan bathing, in the adhika Jyeshtha of
VS 2083. The text's malamasa rules (p. 4, page image) keep the regular
and occasional rites in the adhika month and bar new optional ones from
it. So `FIRST` is a per-rule choice and not a default.

**Built.** `FIRST` is decided from the lunation beside the occurrence's
own: an adhika month runs new moon to new moon under both conventions, so
it comes just before the amanta nija month of its name, and just after the
dark fortnight a purnimanta almanac names after it. Days that stop before
that lunation leave the occurrence unjudged and say why, rather than
guess; the SDK reads a month further for a request with a `FIRST` rule,
so a range of any length answers as a year does. `NEPAL_PANCHANGA` keeps
`DASHAHARA_SNANA_ARAMBHA` and `GANGA_DASHAHARA` `FIRST`, which carries
the three bathing rites counted from them, and `ANALA_PUNHI` `ONLY`: the
committee's sky now gives the printed day for all six VS 2083 rows, and
VS 2082, with no adhika month, prints Anala punhi nowhere and the pack
keeps none.

**Settles:** C606 (the five partings of VS 2083) and C607, both closed;
Padmini and Parama ekadashi took `ONLY` with `month` empty at sunrise (§3.10, C694).

### 3.8 The Sun's sign: `SUN_IN`

```json
{"is": "SUN_IN", "day": "LATER", "rashi": "VIRGO", "not": true}
```

The predicate holds when the Sun stands in the sign at the day's sunrise.
With `not`, it holds when the Sun does not.

**Source.** *Dharmasindhu* p. 56 (page image), on Durva ashtami: "इदं
दूर्वापूजनव्रतं कन्यार्केऽगस्त्योदये च वर्ज्यम्". The Durva vrata is avoided
with the Sun in Kanya and at Agastya's rising.

In VS 2083 the adhika Jyeshtha pushes Bhadrapada's bright 8th into the
solar Ashwin, with the Sun in Kanya. The committee prints Durva ashtami a
month early, on Shravana's dark 8th. A rule that says this needs a third
thing as well: what replaces a barred day. The print's answer is the same
tithi of the paksha before. That is a choice the language has no word for,
so it waits on more cases (C608).

**Settles, in part:** Durva ashtami in VS 2083. Why the committee moved
Gaura saptami and Amuktabharana with it is not in hand.

### 3.9 Counting back, and two days

A following rule today counts 0 to 15 days forward. Two changes extend
it:

- **A count backwards** (`"days": -1`) for an eve.
- **`BOTH`**, a choice that answers both days of the pair. The answer
  marks the observance as kept on two days, which is what a listing of two
  days says.

**Source.** The consumer's source lists Kojagrat purnima on the eve and the
day in VS 2081–2083 (C617). *Dharmasindhu* p. 74, read off the page image,
keeps Kojagara on the earlier day only when that day alone holds the full
moon at nishitha, and on the later otherwise; pradosha is only some
authorities' reading. The committee prints one day in both editions, the
text's, and `NEPAL_PANCHANGA` ships that rule. So no rule in hand needs
`BOTH`; it would be for a listing's practice, which no text gives.

Mahalakshmi's vrata lasts sixteen days, and Indra jatra's pole falls some
days after it is raised; both were taken for counts. Measured, neither
is: the consumer's days end the vrata 14 to 16 days after its start, on
the dark 8th, and the print lowers the pole 7 days after raising it in
VS 2082 and 6 in VS 2083. Both editions print the vrata's start the day
before Bhadrapada's bright 8th fast and its end on the dark 8th fast, so
it waits on the committee's ashtami fast (C695). No row in hand needs a
count backwards or a count of more than 15 days.

### 3.10 A named ekadashi

A named ekadashi is the Smartas' 11th at sunrise wherever the committee
prints one day, as for Varuthini and Yogini (C688), and the Vaishnavas'
day stays the Ekadashi observers' answer (`festival-rules.md` §8). Kamada
ekadashi needed no form, and Padmini and Parama, the adhika month's
bright and dark 11th, needed only §3.7's `ONLY` with `month` empty, which
the engine already read: a rule with no month is kept in whichever month
is doubled. VS 2083 prints पद्मिनी एकादशीव्रत on 27 May 2026 and परमा
एकादशीव्रत on 11 June, the adhika Jyeshtha's 11ths, and VS 2082, with no
adhika month, prints neither. Over the committee's sky the rule gives
both printed days and all 17 of the consumer's days from 2020 to 2042,
two adhika months in 2028 and 2029 among them; over the modern sky the
adhika month itself moves in 2028 and 2042, and Parama by a day in 2023
and 2039 (C694). `NEPAL_PANCHANGA` ships both. The `month` on
`EkadashiRule` proposed here is not needed for any day in hand.

### 3.11 What is not proposed

- **A crescent sighting.** The Hijri months begin on an announcement. A
  visibility estimate is another feature, and its answer would still not
  be the announced day.
- **A Newar charhe window.** §6 explains why.
- **One window per festival** chosen only because it fits.

## 4. The 57 excluded rows

The reviewed table excludes 57 rows. Each row below gives four things: the
consumer's reason, in short; the best window found over the committee's
sky for the verified years; the form that would express the row; and the
verdict. "None new" means the language can already say the row, and what
is missing is evidence. "None" means no form would decide the row.

| # | key | the consumer's reason | best found here, committee sky | form | verdict |
|---:|---|---|---|---|---|
| 1 | `KAMADA_EKADASHI` | the source gives the 11th or the 12th; 11 of 14 at best | sunrise, 11 of 14 | none new | **shipped** at sunrise: VS 2082 prints the 11th (29 March 2026), the Smartas' day as Varuthini's and Yogini's (C688), and the rule gives every one of the source's 11 verified days |
| 2 | `MATATI_CHARHE` | a charhe; 11 of 14 at aparahna | aparahna, sayahna or pradosha, 11 of 14 | none | C614: no one window fits the charhe |
| 3 | `RAMANUJACHARYA_JAYANTI` | 12 of 14 at sunrise | moonrise, 13 of 14 | §3.1 moonrise | a candidate only: one more year for a new window is an overfit until a source names the time |
| 4 | `NARASIMHA_JAYANTI` | 11 of 14 | day muhurtas 14–15, 12 of 14 | none new | **shipped** by the text: *Dharmasindhu* p. 37, sunset, the later day unless the earlier alone holds it; both printed days, and 12 of 14 verified |
| 5 | `SITHI_CHARHE` | a charhe; 11 of 14 at aparahna | aparahna, sunset, 11 of 14 | none | C614 |
| 6 | `VAT_SAVITRI_VRAT` | 12 of 14 at sunrise, Jyeshtha's new moon | sunrise, 12 of 14 | none | *Dharmasindhu* p. 39 keeps the vrata on the 13th to the full moon, which is another region's form; no window lifts the new moon's; evidence |
| 7 | `MATSYA_DARSHAN` | 11 of 13 | sunrise, 11 of 13 | none | evidence |
| 8 | `GHANTAKARNA_CHATURDASHI` | no part of the day fits two years whose 14th began 46 and 85 minutes before sunset and were kept a day apart | pradosha, 12 of 14 | none | C614: the same days as Gathamuga charhe; the two years contradict any window |
| 9 | `GATHAMUGA_CHARHE` | the same days as Ghantakarna | pradosha, 12 of 14 | none | C614 |
| 10 | `KALKI_JAYANTI` | 12 of 14 at pradosha | pradosha, 12 of 14 | none new | **shipped** on Shravana's bright 6th at sunrise, both printed days; the source's 5th falls the day before both (C693) |
| 11 | `SHIVA_PAVITRAROPAN` | 10 of 14 | pradosha or nishitha, 10 of 14 | none new | **shipped**: both editions print it on the full-moon fast's day, and it takes that rule (C200) |
| 12 | `HAYAGRIVA_JAYANTI` | 10 of 14 | sunrise, 10 of 14 | none new | **shipped**: both editions print it on the full-moon fast's day, and it takes that rule (C200) |
| 13 | `JANMASHTAMI` | the nirnaya rule gives 12 of 14; the source lists two days in 2071 and the sunrise day in 2072 | the shipped rule | none new | §5: the shipped `JANMASHTAMI` is the nirnaya rule exactly; the two misses are the source's, a double listing and a Vaishnava day (C175, C616) |
| 14 | `JUGA_CHARHE` | a charhe; 11 of 13 | sayahna or pradosha, 11 of 13 | none | C614 |
| 15 | `VARAHA_JAYANTI` | the source's rule names Shravana's bright 3rd; 12 verified days are Bhadrapada's | Bhadrapada's bright 3rd at sunrise, 12 of 14 | none new | **shipped** on Shravana's bright 3rd, the source's rule, which both editions print; the Bhadrapada days are the source's list (C693) |
| 16 | `MAHALAXMI_VRAT_BEGINS` | 11 of 14 at nishitha | nishitha, 11 of 14 | none new | evidence; the end is counted from it (§3.9) |
| 17 | `GAURA_PARVA` | 10 of 14 | day muhurtas 2–6, 11 of 14 | §3.2 day muhurtas | a candidate only |
| 18 | `INDRADHWAJA_PATAN` | 10 of 14 in the morning | madhyahna or sayahna, 10 of 14 | none | not a count: the print lowers the pole 7 days after raising it in VS 2082 (4 to 11 September 2025) and 6 in VS 2083 (23 to 29 September 2026); evidence |
| 19 | `NANICHAYA` | 11 of 14 | sunrise, 11 of 14 | none | evidence |
| 20 | `MAHALAXMI_VRAT_ENDS` | 10 of 14 at pradosha | pradosha or nishitha, 10 of 14 | none new | not a count: the consumer's days fall 14 to 16 days after the start, so the end is a tithi. Both editions print the start the day before Bhadrapada's bright 8th fast and the end on its dark 8th fast, which wait on the committee's ashtami fast (C695) |
| 21 | `PHARPING_HARISHANKAR_JATRA` | 9 of 14 | sunrise, 9 of 14 | none | evidence |
| 22 | `NALASWANE_CHARHE` | a charhe; 9 of 14 | aparahna to pradosha, 9 of 14 | none | C614 |
| 23 | `KOJAGRAT_PURNIMA` | 9 of 14; the Kojagara rule (midnight) 7 of 14; the source lists the eve and the day in 2081–2083 | sunrise, 12 of 14 | none new | **shipped** by the text: *Dharmasindhu* p. 74, nishitha, the earlier day only when it alone holds it (C617); both editions print one day, the text's |
| 24 | `GOVATSA_DWADASHI` | 8 of 14 | madhyahna, aparahna or sayahna with `GREATER`, 13 of 14 | none new | **shipped** by the text: *Dharmasindhu* p. 76 keeps it at pradosha, the later day when neither holds it and the earlier when both do, as most authorities say; both editions print it there (2025-10-17, 2026-11-05), on the day whose sunrise holds the 11th. The source's days fit a daylight share better, and the print parts from them |
| 25 | `VISHNU_BAIKUNTHA_CHATURDASHI` | 9 of 14 | sayahna or pradosha, 12 of 14 | none | *Dharmasindhu* p. 82 keeps the vrata on the 14th at arunodaya; evidence |
| 26 | `ARJUNDHARA_MELA` | 12 of 14 in the morning | day muhurtas 1–6, 12 of 14 | none new | **shipped** on Halesi mela's day: both editions print it in the continuation of that row; the source's day after is not printed in VS 2083 (C693) |
| 27 | `GURU_GOBIND_SINGH_JAYANTI` | Poush 21 or 20, or 5 January; 12 of 14 at best | a civil date | §3.6 a civil date | the community's own calendar sets it (C619) |
| 28 | `KASHI_BADA_GANESH_UTSAV` | 8 of 14 | moonrise, sayahna or pradosha, 12 of 14 | none new | **shipped** by the text: Magha's dark 4th (purnimanta; the amanta Pausha's) is a Sankashta chaturthi, kept at moonrise and on the earlier day when both hold it (*Dharmasindhu* p. 9); both editions print it on the day whose sunrise holds the 3rd and whose moonrise the 4th (2026-01-06, 2027-01-25) |
| 29 | `SI_PUNHI` | the source's rule names Magha's full moon; 12 verified days are Magha's bright 5th | Magha's bright 5th at sunrise, 12 of 14 | none new | **shipped** on Magha's full moon, the source's rule: both editions print it on the full-moon fast's day, so it takes that rule (C200); the source's bright 5th days are not printed (C693) |
| 30 | `PASHUPATINATH_CHHAYA_DARSHAN` | 11 of 14 | sunrise, 11 of 14 | none | evidence |
| 31 | `FAGU_PURNIMA` | 11 of 14 | day muhurtas 2–6, 13 of 14 | §3.2 day muhurtas | a candidate only |
| 32 | `DEOPATAN_DESHODDHAR_PUJA` | 11 of 13 | day muhurtas 1–4, 12 of 13 | §3.2 day muhurtas | a candidate only |
| 33 | `RATO_MACHHINDRANATH_SNAN` | 11 of 13 | sunrise, 11 of 13 | none | evidence |
| 34 | `EID_AL_FITR` | the Hijri month begins on a sighting | none | none | published only: a sighting and an announcement; a crescent-visibility estimate is a different feature |
| 35 | `GAI_TIHAR` | 4 of 6 | sayahna, 6 of 6 | none new | the language has sayahna; six years are too few to ship |
| 36 | `EID_AL_ADHA` | the Hijri month begins on a sighting | none | none | published only |
| 37 | `BHOTO_JATRA` | set each year to close the chariot festival | none | none | community: set each year |
| 38 | `MUKTAK_DIWAS` | one verified day | none | none | evidence |
| 39 | `SAFER_INTERNET_DAY` | the organisers' second day of the second week of February; one verified day | none | §3.6 a civil weekday | a Gregorian weekday rule |
| 40 | `PANAUTI_SNAN_MELA` | 2 of 2 by Jyeshtha's full moon or the next day's afternoon | Jyeshtha's full moon at sunrise | none new | **shipped** at sunrise: both editions print it on Jyeshtha's full moon (2025-06-11, 2026-06-29), not on the day before, the full-moon fast's |
| 41 | `PANAUTI_JATRA` | as the mela | Jyeshtha's full moon at sunrise | none new | **shipped**, as the mela; VS 2083 prints it after Ⓑ, the full moon's row |
| 42 | `BARAHAKSHETRA_MELA` | one verified day | none | none | evidence |
| 43 | `JANA_YUDDHA_DIWAS` | the source ended every rule | none | none | no longer kept |
| 44 | `BAHIDYO_BOYEGU` | 2 of 3 | sunrise, 2 of 3 | none new | **shipped** at sunrise: both printed days (C693) |
| 45 | `WORLD_SOIL_CONSERVATION_DAY` | 0 of 3 by 7 or 8 August | none | §3.6 a civil date | the date is not settled; evidence |
| 46 | `LALITPUR_NARSINGH_JATRA` | 1 of 2 | sunrise, 1 of 2 | none new | **shipped** at sunrise: both printed days (C693) |
| 47 | `WALKE_PARVA` | 2 of 3 by Bhadra 1 or Shravana's bright 13th in the morning | none | none new | **shipped** on Bhadra 1, a civil date: both editions print it on भाद्रसङ्क्रान्ति, and all three verified days are Bhadra 1 |
| 48 | `KRISHNA_RATH_JATRA` | 0 of 3 by the source; 1 of 3 at best | madhyahna to moonrise, 2 of 3 | none new | evidence |
| 49 | `RADHA_KRISHNA_RATH_JATRA` | 2 of 2 in the morning | madhyahna to moonrise, 2 of 2 | none new | two years; evidence |
| 50 | `NALA_MACHHINDRANATH_SNAN` | 1 of 3 | sayahna to moonrise, 2 of 3 | none new | evidence |
| 51 | `DISHI_PUJA` | one verified day | none | none | community; evidence |
| 52 | `DISHI_CHARHE` | one verified day | none | none | C614; evidence |
| 53 | `THAKALI_TORANLA_PARVA` | one verified day | none | none | community |
| 54 | `MAJHI_LADI_PUJA` | 1 of 2 | any window, 1 of 2 | none | community; evidence |
| 55 | `MAWLID` | the Hijri month begins on a sighting | none | none | published only |
| 56 | `BABA_GANINATH_JAYANTI` | one verified day | none | none | community |
| 57 | `DURA_MHAIPRU_NAKUMA` | one verified day | none | none | community |

| form | rows |
|---|---:|
| moonrise (§3.1) | 2 |
| day muhurtas (§3.2) | 3 |
| `GREATER` (§3.3) | 1 |
| civil and solar dates (§3.6) | 3 |
| counts and two days (§3.9) | 0: both rows proposed for a count are refuted by the print (rows 18 and 20) |
| none new: the language can say it; 12 of them are shipped, the committee's print of both years settling each, and the evidence is short for the other 9 | 21 |
| none: a charhe, a sighting, a day set each year, one verified day, or a count the print refutes | 27 |

## 5. Janmashtami

The consumer excluded Janmashtami at 12 of 14. It names the nirnaya rule:
the 8th at midnight; on both nights, the one with Rohini, else the second;
on neither night, sunrise.

The shipped `JANMASHTAMI` (*Dharmasindhu* pp. 49–50) is that rule. Its
window is nishitha, and it has three guards:

1. Rohini joined at the later nishitha gives the later day.
2. Rohini joined at the earlier nishitha gives the earlier day.
3. The earlier day alone holding the 8th gives the earlier day.

Otherwise the later day: when only the later holds it, when both do, and
when neither does. When neither night holds the 8th, it began after the
earlier midnight and ended before the later one. It then holds the later
day's sunrise, so "sunrise" and "the later" are one answer.

The consumer's two misses are its source's:

- one year lists two days;
- another keeps the sunrise day, though the 8th and Rohini held the
  midnight before.

The second is the Vaishnava day, which C175 records against the Smarta
rule. So no new form is needed. The rule is in every pack, and `NEPAL`
keeps it as the text gives it, since it is a rite of the night (C616).

## 6. The Newar charhe

The monthly dark 14th's Newar observances fit no single time of day:

- **On the committee's print** (25 monthly charhe in VS 2082 and 2083),
  the share of sayahna gives 24 and aparahna 23. The 25th is Shila charhe,
  which is Shivaratri's day.
- **Over the consumer's 14 years** (137 occurrences), pradosha gives 105,
  sayahna 102 and aparahna 100.
- **Ghantakarna and Gathamuga**: two years contradict every window. Their
  14ths began 46 and 85 minutes before sunset and ended before the next
  sunset, and they were kept a day apart.

So the charhe the consumer computes ship as separate rules with their own
windows: Dila at aparahna, Lai at sayahna, Pahan at pradosha, and Shila on
Shivaratri's day. No language form is proposed. The likely source is a
Nepal Sambat almanac's own reckoning, which is not in hand, and the form
should come from that text and not from a fit (C614).

## 7. Forks

| # | fork | default | the other reading |
|---|---|---|---|
| C601 | two partial holdings | **`GREATER`, by time** | the share table (shipped meanwhile) |
| C602 | purvahna | **day muhurtas 1–6** | the first half of the day (p. 21) |
| C603 | neither day | **the earlier, as the maintainer decided**, for a rule with no source of its own | the later, the consumer's sunrise fallback; no verified year has the case |
| C604 | weighing by | **time** | the fraction of each day's window |
| C606 | a doubled month | **`NIJA`**, `FIRST` per rule where a source says so (built) | `FIRST` for every rule |
| C608 | a barred day | **not encoded**; `SUN_IN` with no replacement | the paksha before, as VS 2083 prints Durva ashtami |
| C617 | a two-day listing | **one day**, the rule's | `BOTH` |
| C618 | a weekday | **`ON_VARA` with `NONE`** | one rule per weekday-tithi pair, outside the language |
| C619 | a solar day | **`SOLAR_DAY`** on the profile's civil convention | the sankranti's instant, with punya kala as a window |
