# A rule's days, year by year

Status: `built`, 2026-11-17. Design for the consumer's SR-126
(Tithipatra, addendum 9), written before the code and corrected by it.
It extends [`next-occurrence.md`](next-occurrence.md), whose targets it
reads, and borrows [`anniversary-span.md`](anniversary-span.md)'s span:
a span of years founded once. The cost in §5 was measured through the
built call (`crates/sdk/tests/occurrences.rs`, `price`).

A festival page shows a rule's day from three years before to four years
after the year in view. `almanac.next` steps one day forward from an
instant, and a festival range holds at most 366 days, so an app asked
the pack, narrowed to the rule, once a year: eight calls. This call
answers every year of a span at once.

## 1. What is asked and answered

| part | what it is |
|---|---|
| `fromYear`, `years` | the span: the first year and how many, 1 to 50 (§4), counted in `calendar` |
| `calendar` | the calendar the years are counted in and the days written in, Gregorian by default. Bikram Sambat 2080 to 2087 is `{"fromYear": 2080, "years": 8, "calendar": "BIKRAM_SAMBAT"}` |
| place and clock | `latitudeDeg`, `longitudeDeg`, `altitudeM` and `utcOffsetSeconds` or `zone`, as every almanac request takes them |
| `kind`, `rule` or `group` | what is looked for, as `almanac.next` reads it (`next-occurrence.md` §1, §2), one of the three, refused alike: a kind (`EKADASHI`, `PURNIMA`, `AMAVASYA`, `SANKRANTI`) by the SDK's own rules, a rule, a key of `festivals`, a festival request, or a group of `festivals` (`festival-groups.md`), its members looked for |

The answer is `{keys, years}`. `keys` are the keys looked for: the
rule's, each of a kind's rules, or each of a group's members. `years` holds every year of the span
in order, each `{year, first, last, days, festivals}`:

- `first` and `last`, the year's first day (month 1, day 1) and the day
  before the next year's first;
- `days`, each day of the year the target falls on, in order, as
  `{day, keys}`, the keys looked for that fall on it in the order they
  were looked for;
- `festivals`, the reckoning's entries for those keys in the year, as a
  festival answer holds them; for a group, with each of its spans that
  reaches the year, whole, as a range over the year answers it (a span
  crossing a year's end is in both years).

The answer is sealed with the reckoning's provenance; its input hash is
taken over the request. A convention `festival.years` names the days
founded.

## 2. A year, and a day in it

**A year holds the days whose civil date lies in it (C2120).** A year is
the request calendar's, month 1, day 1, to the day before the next
year's first, as `almanac.anniversaries` counts one. A festival's own
cycle (the lunar year it is kept in) does not decide which year it is
listed under: its day's date does. So a lunar rule asked over solar years
can fall twice in one and not at all in the next. At Kathmandu, Pausha's
bright Ekadashi (`PAUSHA_PUTRADA_EKADASHI`, `NEPAL_PANCHANGA`) falls on
10 January and 30 December 2025, and on no day of 2026. The test holds
that span.

**A year lists every day, never a pick (C2121).** `days` is a list
because one value cannot say what a year holds:

| a year holds | when |
|---|---|
| no day | a lunar rule's day slipped into the year before (above); a rule kept only in an adhika month, in a year with none; a day a publisher withdrew (C2122) |
| one day | most rules, most years |
| two days or more | a lunar rule meeting a solar year twice; a kind (two dozen Ekadashis, twelve Purnimas) |

An empty list is an answer and never a refusal. `PADMINI_EKADASHI`, the
adhika month's bright 11th (Nepal's national panchanga, VS 2083 p. 5),
lists a day in VS 2080 (the adhika Shrawan) and VS 2083 (the adhika
Jestha) and none in VS 2078, 2079, 2081, 2082 or 2084. The test holds
that span. A day that falls in a kshaya month is the rule's to judge, as
it is in every range (`festival-rules.md` §7.1); this call adds no rule
of its own.

**A withdrawn day is not listed, and stays in the year's festivals
(C2122).** A published set may withdraw a rule's day. `next` passes over
it. Here the year's `days` leave it out, so a reader counting days sees
none, and the year's `festivals` keep the entry with its `published`
status, so the reader can say why. A day a published set adds is listed
under its own key.

**An occurrence the rules could not judge is kept in its year (C2123).**
`unjudged` entries whose tithi meets the year's span, from its first
sunrise to the sunrise after its last day, stay in that year's
`festivals`. `days` does not list them, since no day was given. So "no
day" with an unjudged entry means that the day could not be judged, not
that the rule is not kept that year.

**Kinds are in scope (C2124).** A kind is read by the same rules `next`
reads, and the same code path answers it: a kind's year lists each day
any of its rules falls on, with the keys of those that do. When the
Smarta and Vaishnava Ekadashis part, they are two days with one key
each.

## 3. One founding, one reckoning

A festival range is reckoned over its days widened at each edge: two
days, what a rule counts from another's day, and a lunation for a rule
kept only in the adhika month (`festival-rules.md` §4, §7.1). Eight
calls found eight widenings and reckon eight times.

This call finds the span's days **once**, from the widening before the
first year to the widening after the last, as one run
(`Almanac::festival_span`, a range of at most 366 days at a time so the
segment cache does not thrash, `anniversary-span.md` §5). The target is
reckoned over that run **once**. Each year is then cut from that
reckoning (C2126).

- **A year read from the span is the year's own reckoning.** The rules
  are local: a day's judgment reads the days beside it, which the span
  holds as each year's widening would. The cut is the same as a range's
  (`AlmanacArea::reckoned_over`, which a festival range now calls too):
  the days reach into an adhika lunation beside either edge (SR-93,
  `festival-malamasa.md` §3), the groups' spans are read before the
  trim (SR-125), and the readings the context's `festivals.observer`
  leaves out are dropped after it (SR-116). The tests read each year back against a festival call over that year
  alone, for a pack's rule over four Bikram Sambat years and for the
  Ekadashi kind over two Gregorian years, and Tihar over two, and require the entries to be
  equal.
- **A rule is reckoned alone.** As `next` does, a rule target cuts the
  request to what its days need (`FestivalRequest::narrowed_to`). A
  pack's other rules cost nothing.
- **The provenance is the first range's,** as a span's is, with its input
  hash taken over the request, and `festival.years` naming the founded
  days, for example `2023-04-12..2031-04-15`.

## 4. The limit

**1 to 50 years (C2125).** The consumer needs eight. A span is founded as
one run of at most 36,600 days (`MOST_SPAN_DAYS`), and a request's
widening adds two days, a rule's count and a lunation at each edge. Fifty
years and any widening fit inside it. A hundred would not, once a rule
counts more than a few days. Fifty years of one rule also cost about
fifty years of founding (§5), a few seconds on the built-in chain. That
is the most one call should block. `years` of 0 or past 50 is
`OUT_OF_RANGE`, naming `occurrences.years`, with a hint to ask the rest
as requests that follow on. `almanac.anniversaries` keeps its own hundred:
it reads origins, not a rule's widening.

## 5. Cost

Measured on this Mac (Apple silicon, release build, 2026-11-17) through
the built call (`cargo test --release -p teistro --test occurrences price
-- --ignored --nocapture`). Each run is on a fresh context, and the
figures are medians in milliseconds, of 11 runs (first run) and of 21 runs (second run). Kathmandu, Bikram Sambat
2080 to 2087. "8 calls" is the consumer's path today: `festivals` over
each year's range, with the request narrowed to the rule, or the kind's
own rules. The built-in chain stands in for the Teimeris adapter, which
this lane does not build.

| request | committee, Surya Siddhanta: first run | second run | drik, built-in: first run | second run |
|---|---:|---:|---:|---:|
| Vijaya Dashami, `NEPAL_PANCHANGA`, 8 years, one call | 792 | 815 | 2,032 | 1,980 |
| Vijaya Dashami, 8 narrowed `festivals` calls | 803 | 1,003 | 1,959 | 1,964 |
| `EKADASHI`, 8 years, one call | 1,504 | 982 | 1,994 | 2,018 |
| `EKADASHI`, 8 `festivals` calls | 1,646 | 1,163 | 2,368 | 1,942 |
| for scale: the whole pack, 8 `festivals` calls | 1,033 | 993 | 3,407 | 2,420 |

The Mac was shared with other lanes' gates while these were taken (load
average 13 to 19), so the runs differ by up to a third. Read them as a
rough comparison, not a record.

**One call costs about what eight do: from the same cost to about a fifth
less.** The reason was measured by timing the call's parts (a scratch
build, not kept). Over 8 years on the committee's sky, founding the span's
days took 1.0 to 1.8 s. Reckoning the narrowed rule over those days took
0.3 to 1.3 ms, and cutting the years took less than 0.1 ms. So founding the
days is nearly all of the cost. Eight calls found the same days plus
eight widenings of a few days each, a few per cent more. Founding once
saves the widenings and the per-call work (checking the request, the
saits, sealing), and nothing more. The consumer gains one boundary
crossing and one answer in place of eight.

**A year founded warm is cheaper, and that does not help a span.** The
built-in chain fits the Moon and the Earth into segments it keeps per
thread, directly mapped, about a year's worth (`chebyshev::Segments`).
The same year asked again on the same thread founds in about 27 ms
against 70 to 100 ms cold. Each year of a span is new, so a span founds
cold, and so do eight calls over eight years. The 1-year figure would
mislead, so the table leaves it out.

**What would make it cheaper.** A rule kept in one lunar month needs only
the days around that month's occurrence. It does not need the whole year.
Founding only the lunations a rule's month can fall in would cut a rule's
year from about 370 days to about 35, so roughly a tenth of the cost. The
same change would cut `next`'s windows. It is a change to the founding,
not to this call, and is left (§7).

## 6. At the boundary and in the bindings

`AlmanacArea::occurrences` and `occurrences_in_full` in the façade, with
`OccurrencesRequest`, `Occurrences`, `OccurrenceYear`, `OccurrenceDay`
and `MOST_OCCURRENCE_YEARS`; `ts_almanac_occurrences` at the boundary,
a JSON request and a canonical `{value, provenance}` answer;
`almanac.occurrences` in every binding, taking the generated
`OccurrencesRequest` record; and the MCP tool `almanac.occurrences`,
whose input schema is the request's.

## 7. Left

- Founding only the lunations a rule's month can hold (§5), for this
  call and `next` alike.
- The `app-speed` record carries no row for this call. As with `next`,
  a budget row waits for the next device recording (`next-occurrence.md`
  §5).
