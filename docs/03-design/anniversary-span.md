# Anniversaries over a span of years

Status: `built`, 2026-10-31. Design for the application's SR-04, SR-23 and
SR-24 (`consumers/addendum-3.md`), written before the code and corrected by
it. It extends `shraddha.md`, whose rules it keeps: this page adds no day
rule for a shraddha or a janma tithi. It adds a name for the rule each day
took, one kind, and a call that reads many people over many years from days
founded once.

## 1. What is asked and answered

`almanac.anniversaries` takes:

| part | what it is |
|---|---|
| `fromYear`, `years` | the span, 1 to 100 years, counted in `calendar`. It runs from month 1, day 1 of the first year to the day before month 1 of the year after the last, so a caller never needs a year's last day. |
| `calendar` | Gregorian by default; the answer's days are written in it |
| place and clock | `latitudeDeg`, `longitudeDeg`, `altitudeM`, `utcOffsetSeconds`, as `almanac.next` takes them |
| `anniversaries` | up to 1,000, each as a festival request writes one (`shraddha.md` §3.1), each key once |

The answer is the festival answer, sealed: its `anniversaries` hold each
day in the order the tithis begin, its `unjudged` what no day could judge.
So a reader written for `festivals` reads this unchanged, and the binding
types are the same. A year holds one day for most people, and none or two
for some: twelve lunations are about eleven days shorter than the year, so
a solar year can hold a tithi twice or miss it, and a samsarpa keeps a
second day (C669).

Each day gains `dayRule`, in this call and in `festivals` alike:

| field | what it says |
|---|---|
| `tithi` | `ORDINARY`, `KSHAYA` or `VRIDDHI`: one sunrise, none or two (C910) |
| `at` | the window read, as the rule spells it: a request's stated one or the kind's |
| `clause` | `ONE_DAY`, `BOTH_WHOLE`, `NEITHER`, `EQUAL_PARTS`, `GREATER_SHARE`, and a janma tithi's `BIRTH_NAKSHATRA` and `TWO_MUHURTA_FLOOR` |
| `source` | where that clause is stated, or the convention's name |

`observance` still holds the case, both days' extents and the margin. The
clause is a name for what they already say, which is why it cannot
disagree with them: it is derived from the case and the guard that decided
(`DayClause::of`).

## 2. The rule, read again at the source

The clause table is `shraddha.md` §2.2 and §2.4 unchanged. Reading the
passages again for this page, in the OCR of the two editions that page
cites, gave three things it did not record.

### 2.1 Two senses of kshaya and vriddhi (C910)

A panchanga calls a tithi kshaya when no sunrise falls in it and vriddhi
when two do. That is the sense `tithi` reports, because it is what decides
a sunrise rite (Kane V.1 p. 89) and what a printed page shows.

The dharmashastra verse uses the words of a tithi's length against its
neighbour. Vyaghra in NS (Parichheda 1, the tithi section, printed p. 36 of
the Dhakal edition, OCR, page image not checked): "खर्वो दर्पस्तथा हिंसा
त्रिविधं तिथिलक्षणम् । खर्वदर्पौ परौ पूज्यौ हिंसा स्यात्पूर्वकालिकी", glossed
"खर्वः साम्यम्, दर्पो वृद्धिः ... हिंसा क्षयः ... एतच्छ्राद्धादिविषयम्": a tithi
is level, growing or shrinking; for a shraddha the level and the growing
take the later day and the shrinking the earlier. This is DS p. 266's
"others" (C663). NS on the parvana (Parichheda 3): "तिथिसाम्यवृद्धिक्षयैः
समव्याप्तौ खर्वादिना निर्णयः । वैषम्येऽधिका": on equal coverage, by that
scheme; on unequal, the greater.

The two senses are different questions, so the answer does not merge
them. `tithi` is the sunrise count. The scheme is not built: it decides
only `EQUAL_PARTS`, which C170 reads as equal to the last digit, and no
such day appears in §5's count, a count a test keeps.

### 2.2 Madhava's variant reads otherwise than `shraddha.md` said (C664)

`shraddha.md` §2.2 gives Madhava's parvana as "the earlier if the following
tithi is kshaya, the later if it is vriddhi". The DS page continues:
"उत्तरतिथेः क्षयवृद्ध्यभावेऽपि परेत्याहुः" (OCR "क्षयदद्वभावेपि"): the later
too when the following tithi is neither. So Madhava's rule is: the earlier
only if the following tithi shrinks, otherwise the later. C664 is corrected
to say so (C912). The default stays DS's own (the earlier). This variant is not
built, and an anniversary cannot ask for it; §7 hands that to the maintainer.

### 2.3 An event has no text (C911)

No text read rules on a wedding's or a founding's yearly day. NS and DS
rule on a death and a birth. So `EVENT` is a **convention**, named
`anniversary.event` in the provenance of every answer that holds one, and
in `source` on each day. It is read as a janma tithi is, because a family's
event is a celebration and not a rite for the dead:

- the window the request states, sunrise by default;
- the day holding the tithi there alone; the earlier when both, neither or
  a tie; else the greater share (`by_share`, which at an instant reduces to
  the sunrise rule of `FestivalRule::udaya`, so there is one path);
- the ordinary month, with a shraddha's adhika and samsarpa policies out
  of reach; the joined lunation of a kshaya year answers to both its names,
  as for a janma tithi (NS "ābdikavad vardhāpane 'pi").

A caller with another reading states the window, or writes a rule.

## 3. The span

`Almanac::festival_span` founds a span of up to `MOST_SPAN_DAYS` (36,600),
the one reckoning that may pass `MOST_DAYS`, a range of `MOST_DAYS` at a
time, and answers with the first range's provenance. It is not one run,
though the code first was: the built-in chain's segment cache thrashes on a
decade's lockstep rounds, and a single run took 37.9 s where the ranges take
4.0 s. A seam test holds that a day and an occurrence across two ranges read
as they do within one. The widening is what a festival range takes for an
anniversary: 33 days before and 2 after (`festival-rules.md` §4).

`years` stops at 100, a lifetime's janma tithis, and `anniversaries` at
1,000, whether the request was read from text or built by hand. Each refusal
names its field, and the second says to send a second request.

## 4. Why this is not N calls

Three things are shared, and each was a cost per person or per year
before:

1. **The days.** Founded once across the span, where ten festival requests
   found ten years and a lunation around each.
2. **The occurrences.** `Reckoned` holds the distinct tithis and nakshatras
   of the run. `observances` found them again for every rule of every
   person.
3. **The look-ups.** `month_of` and `pair` walked the days from the start
   for every occurrence, which is quadratic over a decade. The days are in
   order, so each is now a binary search. This changes no answer
   (§5's acceptance test) and saves a little on every festival request.

What stays per person: finding the origin (a Lunar one costs nothing, an
Instant or a Date founds a day or two), and judging that tithi's yearly
occurrences.

## 5. Measured

Medians in ms from one fresh context each, `adapters/ephemeris-teimeris`'s
`app_speed` in release on this Mac, 50 people (half births, half deaths, by
their moments) over 10 years at one place:

| request | `drik-teimeris` | `committee-teimeris` |
|---|---|---|
| 50 people, one call (`anniversaries`) | 706 | 267 |
| 1 person, one call (`anniversaries-one`) | 643 | 236 |
| 50 people, 10 `festivals` calls (`anniversaries-yearly`) | 971 | 480 |
| 50 people, 50 calls (`anniversaries-apart`) | 28,443 | 11,580 |

So the founding is nearly all of it: each person past the first adds about
1.3 ms, and the one call is about forty times cheaper than fifty on either
sky. `anniversaries-apart` runs only when named, since fifty calls take
minutes on the built-in chain, so it is a profile taken for this page and is
not in the record. The first three rows are the record's.

**The budget (SR-24).** The `anniversaries` row of `app-speed-measured.md`
holds 900 ms, on the desktop record alone. The application named no figure,
so the phone figure is this page's choice: a second, one that does not
interrupt a screen. Divided by the record's ratio of Android to desktop on
`festivals`, 1.05 for drik and 1.09 for the committee when the row was set,
the larger gives 900 ms. The record re-taken for this page reads about 1.0
for both, which shows how little the ratio holds. **That ratio scales almost nothing (C913).** The Android record is
an emulator on this same Mac, so the ratio says how close the emulator runs,
not how a phone does. The row holds a regression on this Mac; it makes no
claim about a low-end phone, which needs a record of its own.

**The rule, counted.** Every tithi of every month (360 lunar origins) in each
of three kinds, at Kathmandu over 2026 to 2035, Gregorian, the
`nepal_official` profile on the built-in chain:

| kind | days | kshaya | vriddhi | `ONE_DAY` | `GREATER_SHARE` | `BOTH_WHOLE` | `NEITHER` | `EQUAL_PARTS` |
|---|---|---|---|---|---|---|---|---|
| ekoddishta | 3,618 | 138 | 82 | 3,397 | 186 | 10 | 25 | 0 |
| parvana | 3,618 | 138 | 82 | 3,310 | 306 | 0 | 2 | 0 |
| janma tithi | 3,590 | 136 | 81 | 3,373 | 0 | 81 | 136 | 0 |

A rite's window decides, not the tithi's shape: all 220 kshaya and vriddhi
days of a shraddha fell to `ONE_DAY`, since a window of a few ghatis is held
by one day even when no sunrise, or two, fell in the tithi. A janma tithi,
read at sunrise, turns each into `NEITHER` or `BOTH_WHOLE`. That is why
`tithi` and `clause` are two fields. These counts were read once;
`a_decade_of_shraddhas_holds_no_equal_shares` keeps the zero.

The acceptance test, `a_span_answers_what_each_year_answers_alone`, reads a
family of thirty over a decade through this call and through `festivals` a
year at a time and requires the same days. 50 people over 10 years hold 498
days and write 584,715 bytes in full.

## 6. Limits of a rule a caller writes (SR-23)

A rule is the caller's text, and it reaches a loop per occurrence, so its
size is held where it is read (`FestivalRule::check`, `FestivalRequest`). Each
past-limit refusal is `LIMIT` and names its field:

| what | most |
|---|---|
| rules in a request | 4,096 |
| a rule's key | 128 bytes |
| guards on a rule | 32 |
| predicates on a guard | 16 |

These are ceilings against a request that would run away, not a reading of
any text.

## 7. Not built

- Madhava's variant and the kharva-darpa scheme as choices (§2.1, §2.2).
- `published` and `saits` on this call: it is for a family's own days, which
  no authority publishes. `festivals` still takes both, a year at a time.
- A per-person place. One request is one place; a family abroad is a second
  request (the day moves with the place's window, `shraddha.md` §5.2).
- A low-end phone's figure for the budget (C913, §5).

## 8. At the boundary and in the bindings

`AlmanacArea::anniversaries` and `anniversaries_in_full` in the façade;
`ts_almanac_anniversaries` at the boundary, a JSON request and a canonical
`{value, provenance}` answer; `almanac.anniversaries` in every binding; and
the MCP tool `almanac.anniversaries`, whose input schema is the request's.
