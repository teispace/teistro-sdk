# Bikram Sambat: the source memo

Status: `draft`, opened 2026-09-05 with what the baseline engine's own
generator established, and revised the same day with the SDK's own
engine and its measurement against the official table. The research
standard of `09-guidelines/04-adding-a-calendar.md` is met for R4 and R5;
R1 to R3 (the committee's own publications and stated method) remain
open, so the calendar ships as `Tabular` inside the published span and
`Computed` outside it. The maintainer's mandate (2026-09-05): the SDK
must compute Bikram Sambat from first principles the way the Nepali
panchanga does, for any year asked (1700 BS included), with the
published spans as the authority inside their range, so that Nepal's
panchanga makers can use the SDK. Done: the shipped table runs from
1700 to 2500 BS, every row outside the official span computed by the
SDK's engine, and a consumer with a model, a clock and a place computes
any year at all.

## What the calendar is

A sidereal solar calendar: a month begins when the Sun enters the next
sidereal sign (a sankranti), reckoned for Nepal, so months are 29 to 32
days and a year 365 or 366. Year Y begins at the Mesha sankranti falling
in AD year Y − 57 (1 Baisakh, mid-April). The month names are Baisakh,
Jestha, Asar, Shrawan, Bhadra, Ashwin, Kartik, Mangsir, Poush, Magh,
Falgun, Chaitra.

## R1: the authority

The government's Panchanga Nirnayak Samiti (the calendar-determining
committee) publishes the official month lengths; the SDK carries them
for BS 1970 to 2095 (`crates/calendar/data/bikram-sambat.json`). The
committee's own publications and its stated method are not yet in hand:
**open**. What the measurement below implies about the method: the
committee's Sun is the Surya Siddhanta's, reckoned in Nepal's civil
clock, and its month begins on the day the sankranti's punya-kala falls
by the Dharmasindhu's rule.

## R2: the authority's publications

Not yet obtained; the rows in the SDK are the baseline engine's copy of
the government table (rank 2), verified there year by year against an
independent almanac and against a dated event (2 Magh 1990 BS is 15
January 1934, the day of the great earthquake). **Open.**

## R3: three independent real-world sources

Not yet cross-validated; the baseline engine's checker against a
published almanac counts as one. **Open.**

## R4: every day in range validated

Done: every day of BS 1700 to 2500 round-trips through the SDK's
calendar, every year has 365 or 366 days, and the anchors 1 Baisakh 1970
(13 April 1913), 2072 (14 April 2015), 2081 (13 April 2024) and 2 Magh
1990 (15 January 1934) hold (`crates/calendar/src/bikram_sambat/mod.rs`,
tests).

## R5: the divergence envelope

### The measurement

`cargo xtask calendars bs-fit` runs the SDK's engine over the official
span (BS 1970 to 2095, 126 years, 1512 month lengths) under every
combination of model, clock and month-start rule, and reports how many
month lengths and years it reproduces, whether the running day count
drifts, and how far each computed 1 Baisakh lies from the official one.
The sankrantis come from the Surya Siddhanta as the text prints it
(`crates/siddhanta`, `docs/03-design/siddhanta.md`): the mean Sun and
its equation from the text's numbers, the epoch at midnight on the
meridian of Lanka at the start of the Kali age, no bija. The place is
Kathmandu (27.7172 N, 85.324 E); the clock is Nepal's history from tzdb
(local mean time +05:41:16 before 1920, +05:30 to 1986, +05:45 since).

| frame | months | years exact | year totals | drift end (max) | 1 Baisakh offset max |
|---|---:|---:|---:|---:|---:|
| the text, the sine table; Nepal's clock; **punya-kala rule** (shipped) | 1490/1512 (98.5 %) | 116/126 | 126/126 | 0 (0) | 0 |
| the text, exact trigonometry; Nepal's clock; punya-kala rule | 1492/1512 (98.7 %) | 116/126 | 126/126 | 0 (0) | 0 |
| the text, the sine table; Kathmandu local mean time; punya-kala rule | 1486/1512 (98.3 %) | 114/126 | 126/126 | 0 (0) | 0 |
| the text, the sine table; Nepal's clock; the civil day of the sankranti | 1362/1512 (90.1 %) | 76/126 | 126/126 | 0 (0) | 0 |
| the text, the sine table; Nepal's clock; every uniform shift of the boundary (200 scanned) | at most 1362 | | | | |
| the text, the sine table; Nepal's clock; sunrise-to-sunrise day for every sankranti | 926/1512 (61.2 %) | 0/126 | 68/126 | 0 (−1) | −1 |
| the text, the sine table; Nepal's clock; before sunset (the Tamil rule) for every sankranti | 911/1512 (60.3 %) | 4/126 | 65/126 | 1 (1) | 1 |
| the text, the sine table; Nepal's clock; before aparahna (the Malabar rule) for every sankranti | 621/1512 (41.1 %) | 0/126 | 61/126 | 1 (1) | 1 |
| the baseline engine's generator (its epoch adjusted by seven hours, a fitted cutoff at 0.705 of the day) | 87.0 % | | 61/126 | within ±1 | |

The tradition's day count (one day beyond the text's midnight count,
as the worked hand computation uses; `siddhanta.md` §4) changes no month
length under any rule: it shifts every boundary by a whole day, which
the punya-kala rule's day boundaries absorb. Exact trigonometry changes
one boundary in 1512; the sine table is the tradition's own arithmetic
and is bit-identical on every platform, so it ships.

### Which sankranti decides how

The measurement's decisive table: for each of the twelve sankrantis, how
many of its 126 boundaries each rule reproduces, and the plateau of
uniform shifts (days added to the sankranti before taking the civil day)
that reproduces the most.

| sign | civil day | following day | sunrise to sunrise | before sunset | before aparahna | best shift (days) |
|---|---:|---:|---:|---:|---:|---|
| Mesha (0) | **126** | 0 | 97 | 96 | 70 | 0.000 |
| Vrishabha (1) | **125** | 0 | 100 | 98 | 70 | −0.020 to 0.000 |
| Mithuna (2) | **126** | 0 | 99 | 98 | 68 | −0.005 to +0.005 |
| Karka (3) | 97 | 0 | **124** | 70 | 42 | −0.235 to −0.230 (126) |
| Simha (4) | **126** | 0 | 98 | 97 | 70 | −0.005 to +0.005 |
| Kanya (5) | **125** | 1 | 94 | 96 | 70 | −0.005 to +0.020 |
| Tula (6) | **125** | 1 | 92 | 94 | 71 | +0.005 (126) |
| Vrischika (7) | **125** | 1 | 90 | 93 | 69 | +0.005 to +0.015 (126) |
| Dhanu (8) | **126** | 0 | 91 | 89 | 67 | −0.005 to +0.005 |
| Makara (9) | 88 | 38 | 52 | **125** | 106 | +0.290 to +0.305 (125) |
| Kumbha (10) | **124** | 0 | 94 | 90 | 66 | −0.015 (126) |
| Meena (11) | **124** | 0 | 96 | 92 | 67 | −0.025 to −0.010 (125) |

Ten sankrantis follow the civil day, midnight to midnight, in Nepal's
clock. The two ayana sankrantis do not: Karka's month begins a day
earlier whenever the sankranti falls between midnight and about half
past five (a sankranti at night belongs to the day that ended), and
Makara's a day later whenever it falls after about a quarter to five
(a sankranti after sunset belongs to the day beginning). That is the
Dharmasindhu's rule for observing a sankranti's punya-kala (first
pariccheda, the sankranti section, cruxes register C29): a sankranti in
the first half of the night belongs to the day that ended and in the
second half to the day beginning, except a Karka sankranti at night,
observed on the preceding day, and a Makara sankranti at night, observed
on the following day. The SDK names it `MonthStartRule::Punyakala`:
Karka by the sunrise-to-sunrise day, Makara by the before-sunset rule,
the ten others by the civil day. It reproduces every one of the 126
official New Year days, every year total, 116 years exactly and 1490 of
1512 month lengths, with no drift.

### The residual

Eleven boundaries in 126 years, each a pair of divergent month lengths,
and each within 25 minutes of the boundary the rule uses (the sankranti
in the shipped frame, Nepal's clock of the time):

| BS year | boundary | the SDK's sankranti | sunrise, sunset | official month starts |
|---|---|---|---|---|
| 1977 | Tula (Kartik 1) | 23:59 | | a day later |
| 1977 | Makara (Magh 1) | 16:51 | sunset 16:59 | a day later |
| 1981 | Meena (Chaitra 1) | 00:07 | | a day earlier |
| 1989 | Vrischika (Mangsir 1) | 23:57 | | a day later |
| 1998 | Kanya (Ashwin 1) | 23:47 | | a day later |
| 2024 | Karka (Shrawan 1) | 05:24 | sunrise 04:59 | a day earlier |
| 2035 | Vrishabha (Jestha 1) | 00:09 | | a day earlier |
| 2046 | Kumbha (Falgun 1) | 00:20 | | a day earlier |
| 2051 | Karka (Shrawan 1) | 05:19 | sunrise 05:14 | a day earlier |
| 2066 | Meena (Chaitra 1) | 00:14 | | a day earlier |
| 2073 | Kumbha (Falgun 1) | 00:00 | | a day earlier |

The committee's sankranti fell on the other side of the boundary in each
case, so its instant differs from the SDK's by between one and twenty-five
minutes there, earlier in spring and summer and later in autumn. Some
1512 boundaries fall within 25 minutes of a boundary about fifty times
in the span, and eleven of those flip: the size of the residual is what
hand computation in ghatis and palas with the text's tables gives, and
no uniform shift, clock or trigonometry reduces it (the shift scan and
the clock rows above). What would close it is the committee's own tables
of sankranti instants (R2). Each of the eleven is a `Divergent` date in
the SDK: inside the official span the table wins and the date reports
both labels (`CalendarResolution::Divergent { tabular, computed, model }`).

### Continuity and the computed span

Because every official 1 Baisakh is reproduced, the computed years on
either side of the official span join it without a gap or an overlap,
and the year totals agree throughout; the running day count never
leaves zero. The shipped table (`crates/calendar/src/bikram_sambat/generated.rs`)
is generated by `cargo xtask gen calendars` for BS 1700 to 2500 (801
years, 9.6 KB of month lengths): the official rows verbatim, every other
row from the engine under the frame stamped in the file, and the eleven
divergences with the engine's rows for those years; `cargo xtask
check-calendars` regenerates it in CI and fails on any difference. A
year outside the table is computed on request by `bikram_sambat::Engine`
from a model, a clock, a place and a rule, and stamped `Computed`.

### What the baseline engine's generator had established

Kept for the record: Surya Siddhanta longitudes reproduce the official
table far better than modern (drik) ones (87.0 % of month splits against
72.0 % with the best cutoff), so the official calendar is computed from
the classical text; the clock history matters; a fitted cutoff at 0.705
of the day, with the generator's epoch seven hours before the text's,
was the best single rule, which the SDK's measurement explains: seven
hours of epoch and 0.295 days of cutoff nearly cancel to the civil day,
and the rest was the two ayana sankrantis.

## The SDK's engine

1. Sankranti instants from a `SolarModel` (`crates/calendar/src/solar`):
   the Surya Siddhanta (`crates/siddhanta`) today, modern positions
   through the ephemeris port with the profile's ayanamsha when the port
   is promoted; found by the shared boundary solver (`crates/astro`).
2. The month-start rule as named rows (`MonthStartRule`): `SANKRANTI_DAY`
   (Orissa), `FOLLOWING_DAY` (Bengal), `BEFORE_SUNSET` (Tamil),
   `BEFORE_APARAHNA` (Malabar), `SUNRISE_TO_SUNRISE`, `SHIFTED { days }`
   (the family every uniform convention belongs to), `PUNYAKALA` (the
   Dharmasindhu; Nepal's official calendar). The place and the clock
   come from the caller; Nepal's clock history is `teistro_time::zones::nepal()`.
3. The measurement (`cargo xtask calendars bs-fit`, `--detail` for the
   per-sankranti tables and the residual) and its report
   (`crates/calendar/data/bikram-sambat-fit.json`), both from
   `bikram_sambat::fit`.
4. The table generated for any span (`cargo xtask gen calendars`, the
   span in `xtask/src/calendars.rs`) and held by `check-calendars`.
5. Inside the official span the table wins and a disagreement is
   reported as `Divergent`, never hidden.

## Open items

- R1 to R3: the committee's publications and stated method; its tables
  of sankranti instants would settle the eleven residual boundaries.
- C29: the Dharmasindhu's verse for the ayana sankrantis' punya-kala,
  cited by number.
- C28: a cited bija set for the Surya Siddhanta, should the committee use
  one (the measurement suggests it does not for the Sun).
- Modern positions (drik) through the ephemeris port as a second
  `SolarModel`, to publish its numbers beside these from the SDK's own
  code rather than the baseline's.
- The Buddha era and Nepal Sambat new-year rules for the era numbers.
