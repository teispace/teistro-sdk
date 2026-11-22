# The sighted Hijri calendar, measured

Status: `measured`, 2026-11-22, over the built-in sky. The numbers are
printed by the ignored test `sweep` in `crates/sdk/tests/hijri_sighted.rs`
(`cargo test -p teistro --test hijri_sighted -- --ignored --nocapture`);
the Nepal row under the default is held by
`the_default_gives_nepals_printed_eids`, which allows no parting. There
is no `check-hijri-sighted` gate: the sweep reads the printed lists and
reckons seven criteria over 1900 to 2101, so it is run by hand when the
crescent or the sky changes, and this page is rewritten from its output.

It measures `hijri-sighted.md` §2 and §3. A parting is written as the
printed day less the computed one: −1 is a list that keeps the day a day
earlier than the sky at its capital gives.

## 1. Nepal

Twelve Eids, AH 1440 to 1447, as kept in Nepal: Eid al-Fitr every year
and Eid al-Adha in 1443, 1444, 1445 and 1447. Against the tabular
calendar they stand +0, +1, +1, +0, +0, +1, +0, +1 (Fitr) and +0, +0,
+0, +1 (Adha). The sighted months are reckoned at Kathmandu (27.7172 N,
85.324 E, 1400 m, UTC+5:45).

| criterion and zone | agree | parted |
|---|---:|---|
| Odeh, A (default) | 12 of 12 | none |
| Odeh, B | 10 of 12 | Fitr 1445 (2024-04-11) +1; Adha 1447 (2026-05-28) +1 |
| Odeh, C | 7 of 12 | Fitr 1441, 1442, 1443 and 1445, Adha 1447, each +1 |
| Yallop, A | 12 of 12 | none |
| Yallop, B | 10 of 12 | Fitr 1445 +1; Adha 1447 +1 |
| Yallop, C | 10 of 12 | Fitr 1445 +1; Adha 1447 +1 |
| Yallop, D | 9 of 12 | Fitr 1441, Fitr 1445, Adha 1447, each +1 |

A weaker zone sees the crescent an evening sooner, so its months begin a
day earlier and every parting is +1. A local cross-check against three
further Eid al-Adha days from an uncommitted source agreed under the
default, 15 of 15 in all.

## 2. India

Thirteen published lists: the central government's and twelve states'.
Each list's Eid al-Fitr, Eid al-Adha, Muharram (Ashura, 10 Muharram) and
Mawlid (12 Rabi I) over 2019 to 2027, each reckoned at the list's capital.
"Declared" marks a day the list prints as fixed rather than subject to
the sighting.

| criterion and zone | agree |
|---|---:|
| Odeh, A (default) | 339 of 386 |
| Odeh, B | 286 of 386 |
| Odeh, C | 193 of 386 |
| Yallop, A | 340 of 386 |
| Yallop, B | 312 of 386 |
| Yallop, C | 285 of 386 |
| Yallop, D | 269 of 386 |

Odeh A and Yallop A part on one day: the central list's Mawlid of
2027-08-15, which Yallop's A matches and Odeh's puts a day later.

### 2.1 Every parting under the default

| list | agree | parted |
|---|---:|---|
| Andhra Pradesh | 23 of 28 | Fitr 2026-03-20 −1; Adha 2026-05-27 −1; Muharram 2026-06-25 −1; Mawlid 2021-10-20 +1; Mawlid 2026-08-25 −1 |
| Assam | 20 of 22 | Adha 2026-05-27 −1; Muharram 2021-08-19 −1 |
| Bihar | 31 of 32 | Mawlid 2025-09-05 −1 |
| Central | 25 of 28 | Muharram 2021-08-20 +1 (declared); Mawlid 2025-09-05 −1; Mawlid 2027-08-15 −1 |
| Gujarat | 29 of 32 | Fitr 2019-06-06 +1; Adha 2026-05-27 −1; Muharram 2019-09-11 +1 |
| Karnataka | 30 of 32 | Muharram 2021-08-20 +1; Mawlid 2021-10-20 +1 |
| Kerala | 7 of 24 | Fitr 2021-05-13, 2022-05-02, 2023-04-21, 2024-04-10, 2026-03-20; Adha 2021-07-20, 2022-07-09, 2023-06-28, 2025-06-06, 2026-05-27; Muharram 2022-08-08, 2023-07-28, 2024-07-16, 2026-06-25; Mawlid 2022-10-08, 2023-09-27, 2026-08-25: all −1 |
| Maharashtra | 26 of 28 | Fitr 2021-05-13 −1; Adha 2023-06-28 −1 |
| Odisha | 26 of 32 | Adha 2023-06-30 +1; Adha 2026-05-27 −1; Muharram 2019-09-09 −1; Muharram 2021-08-20 +1; Muharram 2023-07-28 −1; Mawlid 2023-09-29 +1: all declared |
| Tamil Nadu | 31 of 32 | Muharram 2021-08-20 +1 |
| Telangana | 31 of 32 | Adha 2026-05-27 −1 |
| Uttar Pradesh | 30 of 32 | Adha 2026-05-27 −1; Mawlid 2025-09-05 −1 |
| West Bengal | 30 of 32 | Adha 2026-05-27 −1; Muharram 2021-08-20 +1 (declared) |

What the partings say:

- **Kerala** keeps 17 of its 24 days a day early, every one −1. That is
  a sighting accepted from west of India, not the sky at
  Thiruvananthapuram; no criterion or zone reaches it (Odeh C, the
  nearest, still parts on 6).
- **Eid al-Adha 2026** is printed on 27 May by eight lists, Odisha's as
  declared; the central list's declared day is 28 May, which the default
  gives, as it gives Nepal's. The evening that decides it is 17 May, Dhu
  al-Qa'da's 29th, whose crescent is in zone B at Kathmandu (§3).
- **The rest** are single days in one or two lists, both signs, most
  of them days printed ahead of a sighting.

The weaker zones' partings, every one, are in the sweep's output. Almost
every parting a weaker zone adds over the default is +1: a month begun a
day early, on a 29th evening the default does not see.

## 3. Margins

Odeh's V at Kathmandu on the evening before each Nepali month's first
day (the evening its crescent was seen, or the 30th's) and the evening
before that (the 29th's, when the month before ran 30 days). Zone A
opens at 5.65.

| month | first day | V, evening before | V, the evening before that |
|---|---|---:|---:|
| Shawwal 1440 | 2019-06-05 | 7.24 | no crescent |
| Shawwal 1441 | 2020-05-25 | 16.69 | 1.26 |
| Shawwal 1442 | 2021-05-14 | 14.65 | 0.36 |
| Shawwal 1443 | 2022-05-03 | 15.25 | 0.25 |
| Shawwal 1444 | 2023-04-22 | 13.02 | −3.36 |
| Shawwal 1445 | 2024-04-11 | 24.60 | 4.29 |
| Shawwal 1446 | 2025-03-31 | 10.60 | −7.12 |
| Shawwal 1447 | 2026-03-21 | 16.73 | −1.38 |
| Dhu al-Hijja 1443 | 2022-07-01 | 10.54 | −1.42 |
| Dhu al-Hijja 1444 | 2023-06-20 | 10.94 | −1.86 |
| Dhu al-Hijja 1445 | 2024-06-08 | 8.43 | −5.65 |
| Dhu al-Hijja 1447 | 2026-05-19 | 24.86 | 4.93 |

The closest calls are the 29th evenings of Ramadan 1445 (4.29) and Dhu
al-Qa'da 1447 (4.93), both zone B and so not seen under the default:
1.36 and 0.72 below the edge. The closest seen evening is Ramadan 1440's
last, 7.24, 1.59 above it. An arcminute of error in the arc of vision
moves V by 0.017.

## 4. Depth

Under the default at Kathmandu, months reckoned by chains opened 12 and
24 months before each of a hundred two-year spans, 1900 to 2101: 0 of
2570 months differ. 1357 of them began after a 29th whose crescent was
not seen, a 30-day month before; the mean month of 29.53 days needs 53%.
