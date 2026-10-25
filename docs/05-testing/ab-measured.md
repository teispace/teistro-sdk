# The SDK against the baseline engine

Status: `generated` by `cargo xtask ab` from `docs/05-testing/ab/*.json`, gated by `check-ab`. Do not edit. Each record is written by `cargo xtask measure-ab`, which runs `bindings/node/bench/ab.mjs` with the baseline engine's adapter (`03-design/ab-baseline.md`).

Both sides answer every birth of the golden corpus in one Node process, the SDK as `parashari-classical` over Teimeris (`kp-default` for `kp`), each request asking the SDK for at least what the baseline's answer holds. A birth on which the two Suns stand more than an arcminute apart is reported and kept out, and a record that kept any out is no claim. A request a side refuses is kept out of that request alone, and only at the corpus's range-edge births, where the data's years end and a day's spans run past them, or at its polar births for a rule that needs a sunrise on a day without one. A round times every birth on one side and then the other, the next round the other way round, and a figure is the median of at least 15 rounds, per birth in microseconds.

Then each side, in processes of its own, answers `chart`, `day` and `full` under load on 1, 2, 4 and every thread the machine offers, in worker threads and in processes, at least 3 trials a count; for 30 seconds on one thread, cut into 10 windows for the tail; from a fresh process 9 times for the cold start; and 10000 charts in each of 3 processes for the memory, the heap read after collecting until two reads agree.

**The claim** is that the SDK is not slower: the gate fails a request or a percentile where its median is the larger, and a throughput where it is the smaller, when the two fastest-to-slowest ranges are apart. Where they meet the ratio reads `within spread`. The cold start and the memory are compared and not claimed (§8). Only the baseline may go unmeasured, and only in worker threads, with the reason recorded.

**Agreement and accuracy:** over the same inputs each answer is read into facts and the two sides' compared field by field, under the SDK's timed profiles and under the ones that cast as the baseline casts, and each side is held to a reference neither side computed where one exists (§10, §11). The gate holds every agreement and reference share to the floor the record measured, and each request's named cause to what the record shows.

**Measurably slower through the SDK**, each held to a bound of the baseline's time:

- `yogas`, at most 2.50×: the shipped Nabhasa set bundles about 2 500 of the SDK's 3 130 classical rules and cannot be asked for in part, so the SDK evaluates about five times the baseline's 650 or so; its cost a rule is the lower of the two.

No other request, and no throughput or percentile, is measurably slower through the SDK.

## `macos-aarch64`

macos aarch64, commit `749d9f824`, Node v26.12.0, 10 threads offered; the sdk `teistro 0.0.0`, the baseline `baseline 0.6.173`.

### Requests, per request in µs

| request | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart (a birth chart with every varga, its drishti, upagrahas, houses, dignities, arudhas and Vimshottari to depth 3) | 1455 | 7617 | 0.19× | 1384 to 1647 | 7528 to 8424 |
| day (the almanac day of the birth's date at its place, its limbs, kaalas, choghadiya and horas) | 1125 | 11947 | 0.09× | 1111 to 1165 | 11829 to 12460 |
| full (the whole natal chart: every section and varga, 22 dashas to depth 3, every shipped rule and the longevity) | 34073 | 52842 | 0.64× | 33817 to 34840 | 52292 to 54677 |
| vargas (the chart's foundation, every divisional chart with it) | 853 | 5785 | 0.15× | 823 to 874 | 5714 to 6841 |
| yogas (the yogas and doshas of a chart) | 12654 | 5934 | 2.13× | 12446 to 13227 | 5889 to 6317 |
| strength (the Shadbala, Bhava bala and Ashtakavarga) | 562 | 5907 | 0.10× | 537 to 593 | 5793 to 6381 |
| dashas (eighteen dasha systems to depth 3) | 15658 | 41210 | 0.38× | 15487 to 16242 | 40765 to 42289 |
| month (thirty almanac days from the birth's date) | 12093 | 363311 | 0.03× | 11956 to 12697 | 356268 to 365106 |
| milan (the birth matched with the next one of the corpus) | 670 | 11481 | 0.06× | 637 to 717 | 11406 to 12552 |
| varsha (the first annual chart, its yogas, sahams and Mudda dasha) | 2619 | 6734 | 0.39× | 2552 to 3104 | 6418 to 7283 |
| kp (the KP cusps, sub lords, significators and ruling planets) | 580 | 5768 | 0.10× | 550 to 612 | 5729 to 6408 |
| gochar (the grahas thirty years on over the chart, with the Ashtakavarga) | 567 | 6218 | 0.09× | 543 to 706 | 5896 to 7262 |
| sadeSati (Sade Sati and the dhaiyyas over thirty years) | 2506 | 23073 | 0.11× | 2455 to 3003 | 22787 to 24092 |
| batch (ten whole charts an hour apart, at one place) | 228274 | 547668 | 0.42× | 224829 to 236004 | 536045 to 563807 |

Refused, and kept out of that request's timing:

- c028, `month`, by the sdk: the month-start rule needs a sunrise on RD 725934 at 69.6492°N 18.9553°E 10 m, where the Sun does not rise
- c029, `month`, by the sdk: the month-start rule needs a sunrise on RD 726115 at 69.6492°N 18.9553°E 10 m, where the Sun does not rise
- c047, `day`, by the sdk: MOON at JD 2378496.179673321: OutOfRange
- c047, `full`, by the sdk: MOON at JD 2378496.179673321: OutOfRange
- c047, `yogas`, by the sdk: MOON at JD 2378496.179673321: OutOfRange
- c047, `month`, by the sdk: MOON at JD 2378496.179673321: OutOfRange
- c047, `batch`, by the sdk: MOON at JD 2378496.179673321: OutOfRange
- c048, `day`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange
- c048, `full`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange
- c048, `yogas`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange
- c048, `month`, by the sdk: SUN at JD 2597642.2629888887 UT1: the provider answered OutOfRange
- c048, `varsha`, by the sdk: Cannot read properties of undefined (reading 'annual')
- c048, `gochar`, by the sdk: the provider could not place SUN: OutOfRange
- c048, `batch`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange

### Throughput, in requests a second

All threads or processes together, each with its side of its own; the ratio is the SDK's throughput over the baseline's, and efficiency the share of linear throughput the count keeps.

| request | mode | at once | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest | SDK efficiency | baseline efficiency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| chart | threads | 1 | 756 | 140 | 5.40× | 744 to 758 | 137 to 141 | 1.00 | 1.00 |
| chart | threads | 2 | 1455 | not measured | — | 1445 to 1460 |  | 0.96 | — |
| chart | threads | 4 | 2770 | not measured | — | 2745 to 2799 |  | 0.92 | — |
| chart | threads | 8 | 4452 | not measured | — | 4334 to 4473 |  | 0.74 | — |
| chart | threads | 10 | 4165 | not measured | — | 3889 to 4341 |  | 0.55 | — |
| chart | processes | 1 | 758 | 144 | 5.26× | 747 to 761 | 142 to 144 | 1.00 | 1.00 |
| chart | processes | 2 | 1486 | 285 | 5.22× | 1467 to 1495 | 279 to 286 | 0.98 | 0.99 |
| chart | processes | 4 | 2936 | 563 | 5.22× | 2929 to 2992 | 562 to 563 | 0.97 | 0.98 |
| chart | processes | 8 | 4806 | 916 | 5.25× | 4552 to 4910 | 906 to 925 | 0.79 | 0.80 |
| chart | processes | 10 | 5031 | 975 | 5.16× | 4519 to 5077 | 950 to 986 | 0.66 | 0.68 |
| day | threads | 1 | 1633 | 84.0 | 19.44× | 1599 to 1641 | 83.0 to 84.7 | 1.00 | 1.00 |
| day | threads | 2 | 2994 | not measured | — | 2902 to 3032 |  | 0.92 | — |
| day | threads | 4 | 5799 | not measured | — | 5767 to 5813 |  | 0.89 | — |
| day | threads | 8 | 8898 | not measured | — | 8831 to 9023 |  | 0.68 | — |
| day | threads | 10 | 9593 | not measured | — | 9235 to 9681 |  | 0.59 | — |
| day | processes | 1 | 1617 | 82.3 | 19.64× | 1614 to 1642 | 82.3 to 83.7 | 1.00 | 1.00 |
| day | processes | 2 | 3311 | 168 | 19.71× | 3253 to 3320 | 168 to 169 | 1.02 | 1.02 |
| day | processes | 4 | 6649 | 333 | 19.99× | 6587 to 6671 | 329 to 334 | 1.03 | 1.01 |
| day | processes | 8 | 10744 | 487 | 22.08× | 10615 to 10855 | 441 to 491 | 0.83 | 0.74 |
| day | processes | 10 | 11126 | 552 | 20.17× | 10876 to 11501 | 550 to 570 | 0.69 | 0.67 |
| full | threads | 1 | 30.0 | 18.0 | 1.67× | 29.7 to 30.7 | 17.7 to 18.3 | 1.00 | 1.00 |
| full | threads | 2 | 54.0 | not measured | — | 46.0 to 54.7 |  | 0.90 | — |
| full | threads | 4 | 96.7 | not measured | — | 91.7 to 102 |  | 0.81 | — |
| full | threads | 8 | 168 | not measured | — | 167 to 173 |  | 0.70 | — |
| full | threads | 10 | 190 | not measured | — | 188 to 195 |  | 0.63 | — |
| full | processes | 1 | 31.0 | 18.7 | 1.66× | 29.7 to 31.0 | 18.0 to 18.7 | 1.00 | 1.00 |
| full | processes | 2 | 46.3 | 37.0 | within spread | 22.7 to 58.0 | 35.7 to 38.3 | 0.75 | 0.99 |
| full | processes | 4 | 111 | 45.7 | 2.42× | 102 to 113 | 42.7 to 63.3 | 0.89 | 0.61 |
| full | processes | 8 | 158 | 96.0 | 1.65× | 147 to 161 | 91.3 to 98.7 | 0.64 | 0.64 |
| full | processes | 10 | 175 | 79.0 | 2.22× | 130 to 182 | 78.7 to 96.3 | 0.56 | 0.42 |

Not measured, and why:

- the baseline, `chart` on 2 threads: died of SIGSEGV: nothing on stderr
- the baseline, `chart` on 4 threads: died of SIGSEGV: nothing on stderr
- the baseline, `chart` on 8 threads: died of SIGSEGV: nothing on stderr
- the baseline, `chart` on 10 threads: died of SIGABRT: nothing on stderr
- the baseline, `day` on 2 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 4 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 8 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 10 threads: died of SIGABRT: nothing on stderr
- the baseline, `full` on 2 threads: died of SIGSEGV: nothing on stderr
- the baseline, `full` on 4 threads: died of SIGSEGV: nothing on stderr
- the baseline, `full` on 8 threads: died of SIGABRT: 9: warmed (file:///Users/iamkrishnaa/Projects/Teispace/teistro/teistro-sdk/.claude/worktrees/agent-a253d46103ea6b3f0/bindings/node/bench/ab-load.mjs:92:20)
- the baseline, `full` on 10 threads: died of SIGSEGV: nothing on stderr

### The tail of a sustained run, per answer in µs

One thread, each side in a process of its own; a percentile is the median of the windows' with the fastest and slowest. Answers: `chart` 21706 through the SDK and 4159 through the baseline; `day` 49111 through the SDK and 2532 through the baseline; `full` 899 through the SDK and 576 through the baseline.

| percentile | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart, p50 | 1330 | 7136 | 0.19× | 1290 to 1363 | 7047 to 7247 |
| chart, p95 | 1575 | 8699 | 0.18× | 1497 to 1815 | 8463 to 8824 |
| chart, p99 | 2310 | 9437 | 0.24× | 2121 to 3018 | 8911 to 12581 |
| day, p50 | 594 | 11840 | 0.05× | 581 to 597 | 11756 to 11871 |
| day, p95 | 723 | 12792 | 0.06× | 684 to 765 | 12631 to 12842 |
| day, p99 | 917 | 13410 | 0.07× | 872 to 1074 | 13142 to 14931 |
| full, p50 | 33377 | 51040 | 0.65× | 32043 to 33995 | 49145 to 52723 |
| full, p95 | 35887 | 65419 | 0.55× | 34156 to 36960 | 63749 to 71809 |
| full, p99 | 42765 | 72104 | within spread | 35725 to 175197 | 70548 to 85711 |

### Cold start, in ms

| phase | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| open (from the process's start to the side being ready, the import and the engine or context included) | 32.4 | 729 | 0.04× | 30.4 to 40.9 | 687 to 804 |
| first (the first chart) | 28.5 | 35.3 | within spread | 26.5 to 63.7 | 33.8 to 106 |
| process (the whole process, from spawn to exit) | 152 | 911 | 0.17× | 146 to 201 | 817 to 941 |

### Memory over 10000 `chart` requests, after 200 to warm

| measure | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| peak-rss (the peak resident set over the process's life), MB | 466 | 297 | 1.57× | 400 to 610 | 297 to 298 |
| heap (the heap and external bytes, settled, after the workload), MB | 8.86 | 46.0 | 0.19× | 8.86 to 8.86 | 46.0 to 46.2 |
| heap-growth (their growth over the workload), KB | -1037 | 220 | -4.71× | -1037 to -1037 | 220 to 388 |

### Agreement and accuracy

The share of facts on which the two sides agree, exactly or within the field's tolerance, under the SDK's timed profiles (`timed`) and under the profiles that cast as the baseline casts (`same`), so that what still differs under `same` is method (§10); and each side against a reference neither side computed, with its evidence rank, where one exists (§11). Closer is the larger share of the reference's facts matched, the SDK's under its best convention.

| request | agreement, `timed` | agreement, `same` | reference | closer to it | dominant cause |
|---|---:|---:|---|---|---|
| chart | 54.5% of 12485 | 100.0% of 12031 | Swiss Ephemeris (2); PyJHora (3) | the baseline: SDK 99.6% `timed`, baseline 100.0%; the SDK: SDK 72.8% `timed`, baseline 46.5% | convention: the centre: `timed` casts the Moon from the Earth's centre and the baseline from the birthplace, up to an hour of arc apart, on a mean ayanamsha where the baseline's is nutated (12″), which moves the Vimshottari's starts; under `same` only the lagnas of 2350 and 2399 differ, the SDK's IAU 2006 sidereal time against Swiss Ephemeris's that far from J2000 |
| day | 86.9% of 858 | 96.5% of 860 | the committee's print (1) | neither: SDK 34.6% `same`, baseline 34.6% | convention: the sunrise: `timed` takes the disc's centre unrefracted and the baseline the refracted upper limb; under `same` only the polar births differ, the conformance profile's nearest sunrise against the baseline's civil-day bounds. The committee's tithi ends are Surya Siddhanta's, hours from both |
| full | 64.4% of 262615 | 99.9% of 252705 | Swiss Ephemeris (2); PyJHora (3) | the baseline: SDK 99.8% `timed`, baseline 100.0%; the SDK: SDK 72.7% `timed`, baseline 46.4% | convention: the centre, and the classical Shadbala readings; under `same` the yogas' readings (as `yogas`), and c022's kaala bala, a birth the baseline places in the wrong Hindu day (crux C35) |
| vargas | 97.1% of 11550 | 100.0% of 11130 | agreement only | — | convention: the centre: a Moon a few arcminutes from a division's edge |
| yogas | 96.9% of 4611 | 97.1% of 4611 | agreement only | — | method: the rules' readings: the SDK applies Kemadruma's cancellations and reads the Nabhasa shapes, Gaja Kesari and Chamara by its texts' clauses, where the baseline holds them present |
| strength | 69.6% of 8635 | 98.6% of 8321 | Raman's Example 56 (2) | the SDK: SDK 100.0% `source`, baseline 0.0% | convention: the readings: `timed` is the classical drik and luminaries' cheshta, the baseline its own; the SDK given Sripati's readings reproduces Raman's totals, and under `same` the largest gaps are c022's, a birth the baseline places in the wrong Hindu day (crux C35) |
| dashas | 61.9% of 244860 | 100.0% of 244860 | PyJHora (3) | the SDK: SDK 72.8% `timed`, baseline 46.5% | convention: the centre: the Moon's place sets every system's start; PyJHora casts from the Earth's centre as `timed` does |
| month | 86.3% of 24680 | 100.0% of 24752 | agreement only | — | convention: the sunrise, as `day`; under `same` every limb agrees |
| milan | 73.4% of 715 | 76.4% of 715 | *Kalaprakasika* XIII (1) | the SDK: SDK 100.0% `timed`, baseline 92.1% | method: the koota tables: Vashya, Gana, Yoni and Bhakoot score apart under `same`, and the baseline's Stree-deergha is Brihaspati's count beyond 9 where *Kalaprakasika* prints beyond 13 |
| varsha | 26.1% of 10908 | 24.4% of 10908 | Charak's worked year (2) | the SDK: SDK 100.0% `timed`, baseline 37.5% | the baseline: the year lord: on Charak's worked year the SDK names all seven offices and the baseline three, its year lord the Moon for the Sun; the sahams are other formulas, and the Mudda dasha follows the year lord |
| kp | 74.6% of 5775 | 97.9% of 5775 | Swiss Ephemeris (2) | the baseline: SDK 97.9% `timed`, baseline 98.2% | convention: the frame: `kp-default` casts from the Earth's centre on its own node, the baseline from the birthplace on the mean node; under `same` c043's cusps, where the baseline's Placidus falls back at 64.8° N and Swiss Ephemeris and the SDK do not, and the lagnas of 2350 and 2399, as `chart` |
| gochar | 65.9% of 1350 | 99.7% of 1350 | Swiss Ephemeris (2) | neither: SDK 100.0% `timed`, baseline 100.0% | convention: the ayanamsha's basis: `timed` takes the mean and the baseline the nutated, about 12″ apart; under `same`, cast from the Earth's centre as the baseline casts a transit, the transits agree to a hundredth of an arcsecond and only c051's transit bindus differ |
| sadeSati | 7.6% of 328 | 100.0% of 328 | agreement only | — | convention: the ayanamsha's basis, as `gochar`: 12″ of Saturn moves an edge by hours; under `same` every edge agrees within two seconds |
| batch | 65.4% of 2626150 | 99.9% of 2527050 | Swiss Ephemeris (2) | the baseline: SDK 99.8% `timed`, baseline 100.0% | convention: as `full`, at ten instants |

#### Agreement by field

A continuous field's median and largest difference are in its unit; `same`'s first mismatches, or its worst input, follow.

| request | field | unit, tolerance | `timed` | `same` | unpaired | where `same` differs |
|---|---|---|---:|---:|---:|---|
| chart | longitude | ″, 1.000 | 0.9% of 550; median 13.9, largest 3576.1 | 99.6% of 530; median 0.000, largest 12.0 | 0 | worst c048 `longitude/LAGNA` |
| chart | sign | exact | 99.6% of 550 | 100.0% of 530 | 0 |  |
| chart | nakshatra | exact | 99.1% of 550 | 100.0% of 530 | 0 |  |
| chart | pada | exact | 97.5% of 550 | 100.0% of 530 | 0 |  |
| chart | retrograde | exact | 100.0% of 495 | 100.0% of 477 | 0 |  |
| chart | dashaLord | exact | 92.7% of 4950 | 100.0% of 4770 | 0 |  |
| chart | dashaStart | d, 1.000 | 1.9% of 4840; median 158.2, largest 7240.8 | 100.0% of 4664; median 0.000, largest 0.001 | 0 |  |
| day | event | s, 60.0 | 70.7% of 376; median 17.3, largest 86400.0 | 96.3% of 377; median 2.500, largest 3038077.9 | 1 | worst c028 `event/yoga/0` |
| day | limb | exact | 99.6% of 482 | 96.7% of 483 | 1 | c028 `limb/tithi/0` 11 against 6; c028 `limb/tithi/1` 12 against 7; c028 `limb/nakshatra/0` 17 against 10 |
| full | longitude | ″, 1.000 | 0.9% of 530; median 14.0, largest 3576.1 | 99.8% of 510; median 0.000, largest 7.090 | 0 | worst c046 `longitude/LAGNA` |
| full | sign | exact | 99.6% of 530 | 100.0% of 510 | 0 |  |
| full | nakshatra | exact | 99.1% of 530 | 100.0% of 510 | 0 |  |
| full | pada | exact | 97.4% of 530 | 100.0% of 510 | 0 |  |
| full | retrograde | exact | 100.0% of 477 | 100.0% of 459 | 0 |  |
| full | varga | exact | 97.0% of 11130 | 100.0% of 10710 | 0 |  |
| full | dashaLord | exact | 85.2% of 118932 | 100.0% of 114444 | 18972 |  |
| full | dashaStart | d, 1.000 | 38.1% of 117024; median 83.1, largest 21936.9 | 100.0% of 112608; median 0.000, largest 0.002 | 18564 |  |
| full | shadbala | virupas, 1.000 | 30.6% of 2597; median 11.9, largest 393.2 | 98.4% of 2499; median 0.000, largest 120.0 | 0 | worst c022 `shadbala/MARS/total` |
| full | bhavaBala | virupas, 1.000 | 1.1% of 636; median 78.6, largest 453.2 | 94.4% of 612; median 0.003, largest 120.0 | 0 | worst c022 `bhavaBala/5` |
| full | bindus | exact | 97.9% of 5088 | 100.0% of 4896 | 0 |  |
| full | yoga | exact | 96.9% of 4611 | 97.1% of 4437 | 0 | c001 `yoga/LUNAR_KEMADRUMA` false against true; c002 `yoga/LUNAR_KEMADRUMA` false against true; c003 `yoga/LUNAR_KEMADRUMA` false against true |
| vargas | varga | exact | 97.1% of 11550 | 100.0% of 11130 | 0 |  |
| yogas | yoga | exact | 96.9% of 4611 | 97.1% of 4611 | 0 | c001 `yoga/LUNAR_KEMADRUMA` false against true; c002 `yoga/LUNAR_KEMADRUMA` false against true; c003 `yoga/LUNAR_KEMADRUMA` false against true |
| strength | shadbala | virupas, 1.000 | 30.9% of 2695; median 11.5, largest 393.2 | 97.6% of 2597; median 0.000, largest 120.0 | 0 | worst c022 `shadbala/MARS/total` |
| strength | bhavaBala | virupas, 1.000 | 1.1% of 660; median 77.3, largest 453.2 | 91.7% of 636; median 0.003, largest 120.0 | 0 | worst c022 `bhavaBala/5` |
| strength | bindus | exact | 97.9% of 5280 | 100.0% of 5088 | 0 |  |
| dashas | dashaLord | exact | 85.5% of 123420 | 100.0% of 123420 | 0 |  |
| dashas | dashaStart | d, 1.000 | 38.0% of 121440; median 83.1, largest 21936.9 | 100.0% of 121440; median 0.000, largest 0.004 | 0 |  |
| month | event | s, 60.0 | 69.5% of 10810; median 18.6, largest 85775.1 | 100.0% of 10846; median 2.335, largest 9.772 | 0 |  |
| month | limb | exact | 99.5% of 13870 | 100.0% of 13906 | 0 |  |
| milan | koota | exact | 66.5% of 495 | 68.5% of 495 | 0 | c001 `koota/VASHYA` 0 against 1; c001 `koota/GANA` 5 against 6; c001 `koota/TOTAL` 27.5 against 29.5 |
| milan | porutham | exact | 89.1% of 220 | 94.1% of 220 | 0 | c004 `porutham/STREE_DEERGHA` false against true; c009 `porutham/STREE_DEERGHA` false against true; c016 `porutham/STREE_DEERGHA` false against true |
| varsha | event | s, 60.0 | 24.1% of 54; median 112.4, largest 334.9 | 16.7% of 54; median 139.1, largest 213.2 | 0 | worst c008 `event/return` |
| varsha | office | exact | 52.1% of 378 | 52.4% of 378 | 0 | c001 `office/yearLord` "VENUS" against "SUN"; c001 `office/triRashi` "VENUS" against "MARS"; c001 `office/dinaRatri` "MARS" against "SUN" |
| varsha | longitude | ″, 1.000 | 0.0% of 864; median 243444.0, largest 647871.0 | 0.0% of 864; median 244083.2, largest 647101.3 | 1350 | worst c037 `longitude/saham/YASHA` |
| varsha | dashaLord | exact | 48.1% of 4860 | 44.4% of 4860 | 540 | c003 `dashaLord/MUDDA/0` "VENUS" against "SUN"; c003 `dashaLord/MUDDA/0.0` "VENUS" against "SUN"; c003 `dashaLord/MUDDA/0.1` "SUN" against "MOON" |
| varsha | dashaStart | d, 1.000 | 6.2% of 4752; median 13.0, largest 56.4 | 6.1% of 4752; median 12.7, largest 53.0 | 540 | worst c038 `dashaStart/MUDDA/8.8` |
| kp | longitude | ″, 1.000 | 1.1% of 1155; median 14.5, largest 335680.9 | 96.2% of 1155; median 0.000, largest 335678.9 | 0 | worst c043 `longitude/cusp/2` |
| kp | kpLord | exact | 93.0% of 4620 | 98.4% of 4620 | 0 | c027 `kpLord/cusp/2/sign` "MARS" against "SATURN"; c027 `kpLord/cusp/2/star` "KETU" against "MOON"; c027 `kpLord/cusp/2/subSub` "SATURN" against "JUPITER" |
| gochar | longitude | ″, 1.000 | 7.4% of 486; median 12.0, largest 18.3 | 100.0% of 486; median 0.000, largest 0.008 | 0 |  |
| gochar | house | exact | 100.0% of 486 | 100.0% of 486 | 0 |  |
| gochar | bindus | exact | 97.4% of 378 | 98.9% of 378 | 0 | c051 `bindus/transit/SUN` 1 against 2; c051 `bindus/transit/MOON` 4 against 5; c051 `bindus/transit/MARS` 3 against 4 |
| sadeSati | event | s, 60.0 | 7.6% of 328; median 2684.4, largest 35385.2 | 100.0% of 328; median 0.154, largest 1.667 | 7 |  |
| batch | longitude | ″, 1.000 | 1.4% of 5300; median 14.7, largest 3623.9 | 99.8% of 5100; median 0.000, largest 7.090 | 0 | worst c046 `longitude/LAGNA@0` |
| batch | sign | exact | 99.8% of 5300 | 100.0% of 5100 | 0 |  |
| batch | nakshatra | exact | 99.5% of 5300 | 100.0% of 5100 | 0 |  |
| batch | pada | exact | 98.4% of 5300 | 100.0% of 5100 | 0 |  |
| batch | retrograde | exact | 100.0% of 4770 | 100.0% of 4590 | 0 |  |
| batch | varga | exact | 97.1% of 111300 | 100.0% of 107100 | 0 |  |
| batch | dashaLord | exact | 87.1% of 1189320 | 100.0% of 1144440 | 189720 |  |
| batch | dashaStart | d, 1.000 | 38.4% of 1170240; median 81.3, largest 22769.3 | 100.0% of 1126080; median 0.000, largest 0.002 | 185640 |  |
| batch | shadbala | virupas, 1.000 | 30.2% of 25970; median 12.3, largest 393.2 | 99.0% of 24990; median 0.000, largest 214.9 | 0 | worst c022 `shadbala/SUN/total@5` |
| batch | bhavaBala | virupas, 1.000 | 0.5% of 6360; median 78.6, largest 453.2 | 96.2% of 6120; median 0.003, largest 214.9 | 0 | worst c022 `bhavaBala/6@5` |
| batch | bindus | exact | 98.7% of 50880 | 100.0% of 48960 | 0 |  |
| batch | yoga | exact | 97.1% of 46110 | 97.2% of 44370 | 0 | c001 `yoga/LUNAR_KEMADRUMA@0` false against true; c001 `yoga/LUNAR_KEMADRUMA@1` false against true; c001 `yoga/NABHASA_SARPA@1` true against false |

#### Against the references

Each side under each convention it was held to the rung under; the SDK's `source` is `timed` given the reading the source works by.

| reference | request | field | side | convention | matching | where it differs |
|---|---|---|---|---|---:|---|
| Swiss Ephemeris (2) | chart | longitude | the SDK | `timed` | 99.6% of 550; median 0.000, largest 12.0 | worst c048 `longitude/LAGNA` |
| Swiss Ephemeris (2) | full | longitude | the SDK | `timed` | 99.8% of 530; median 0.000, largest 7.098 | worst c046 `longitude/LAGNA` |
| Swiss Ephemeris (2) | batch | longitude | the SDK | `timed` | 99.8% of 5300; median 0.000, largest 7.098 | worst c046 `longitude/LAGNA@0` |
| Swiss Ephemeris (2) | kp | longitude | the SDK | `timed` | 97.9% of 1131; median 0.000, largest 12.0 | worst c048 `longitude/cusp/1` |
| Swiss Ephemeris (2) | gochar | longitude | the SDK | `timed` | 100.0% of 486; median 0.000, largest 0.001 |  |
| Swiss Ephemeris (2) | chart | longitude | the SDK | `same` | 99.6% of 530; median 0.000, largest 12.0 | worst c048 `longitude/LAGNA` |
| Swiss Ephemeris (2) | full | longitude | the SDK | `same` | 99.8% of 510; median 0.000, largest 7.090 | worst c046 `longitude/LAGNA` |
| Swiss Ephemeris (2) | batch | longitude | the SDK | `same` | 99.8% of 5100; median 0.000, largest 7.090 | worst c046 `longitude/LAGNA@0` |
| Swiss Ephemeris (2) | kp | longitude | the SDK | `same` | 97.9% of 1131; median 0.000, largest 12.0 | worst c048 `longitude/cusp/1` |
| Swiss Ephemeris (2) | gochar | longitude | the SDK | `same` | 100.0% of 486; median 0.000, largest 0.008 |  |
| Swiss Ephemeris (2) | chart | longitude | the baseline | `timed` | 100.0% of 550; median 0.000, largest 0.001 |  |
| Swiss Ephemeris (2) | full | longitude | the baseline | `timed` | 100.0% of 530; median 0.000, largest 0.000 |  |
| Swiss Ephemeris (2) | batch | longitude | the baseline | `timed` | 100.0% of 5300; median 0.000, largest 0.000 |  |
| Swiss Ephemeris (2) | kp | longitude | the baseline | `timed` | 98.2% of 1131; median 0.000, largest 335678.9 | worst c043 `longitude/cusp/8` |
| Swiss Ephemeris (2) | gochar | longitude | the baseline | `timed` | 100.0% of 486; median 0.000, largest 0.000 |  |
| PyJHora (3) | chart | dashaLord | the SDK | `timed` | 100.0% of 4293 |  |
| PyJHora (3) | chart | dashaStart | the SDK | `timed` | 45.3% of 4240; median 1.242, largest 2.949 | worst c051 `dashaStart/VIMSHOTTARI/1.1` |
| PyJHora (3) | full | dashaLord | the SDK | `timed` | 100.0% of 4131 |  |
| PyJHora (3) | full | dashaStart | the SDK | `timed` | 45.1% of 4080; median 1.446, largest 2.949 | worst c051 `dashaStart/VIMSHOTTARI/1.1` |
| PyJHora (3) | dashas | dashaLord | the SDK | `timed` | 100.0% of 4293 |  |
| PyJHora (3) | dashas | dashaStart | the SDK | `timed` | 45.3% of 4240; median 1.242, largest 2.949 | worst c051 `dashaStart/VIMSHOTTARI/1.1` |
| PyJHora (3) | chart | dashaLord | the SDK | `same` | 92.5% of 4293 | c007 `dashaLord/VIMSHOTTARI/0` "MARS" against "MOON"; c007 `dashaLord/VIMSHOTTARI/1` "RAHU" against "MARS"; c007 `dashaLord/VIMSHOTTARI/1.0` "RAHU" against "MARS" |
| PyJHora (3) | chart | dashaStart | the SDK | `same` | 0.0% of 4240; median 164.6, largest 7240.1 | worst c049 `dashaStart/VIMSHOTTARI/8` |
| PyJHora (3) | full | dashaLord | the SDK | `same` | 92.2% of 4131 | c007 `dashaLord/VIMSHOTTARI/0` "MARS" against "MOON"; c007 `dashaLord/VIMSHOTTARI/1` "RAHU" against "MARS"; c007 `dashaLord/VIMSHOTTARI/1.0` "RAHU" against "MARS" |
| PyJHora (3) | full | dashaStart | the SDK | `same` | 0.0% of 4080; median 169.4, largest 7240.1 | worst c049 `dashaStart/VIMSHOTTARI/8` |
| PyJHora (3) | dashas | dashaLord | the SDK | `same` | 92.5% of 4293 | c007 `dashaLord/VIMSHOTTARI/0` "MARS" against "MOON"; c007 `dashaLord/VIMSHOTTARI/1` "RAHU" against "MARS"; c007 `dashaLord/VIMSHOTTARI/1.0` "RAHU" against "MARS" |
| PyJHora (3) | dashas | dashaStart | the SDK | `same` | 0.0% of 4240; median 164.6, largest 7240.1 | worst c049 `dashaStart/VIMSHOTTARI/8` |
| PyJHora (3) | chart | dashaLord | the baseline | `timed` | 92.5% of 4293 | c007 `dashaLord/VIMSHOTTARI/0` "MARS" against "MOON"; c007 `dashaLord/VIMSHOTTARI/1` "RAHU" against "MARS"; c007 `dashaLord/VIMSHOTTARI/1.0` "RAHU" against "MARS" |
| PyJHora (3) | chart | dashaStart | the baseline | `timed` | 0.0% of 4240; median 164.6, largest 7240.1 | worst c049 `dashaStart/VIMSHOTTARI/8` |
| PyJHora (3) | full | dashaLord | the baseline | `timed` | 92.2% of 4131 | c007 `dashaLord/VIMSHOTTARI/0` "MARS" against "MOON"; c007 `dashaLord/VIMSHOTTARI/1` "RAHU" against "MARS"; c007 `dashaLord/VIMSHOTTARI/1.0` "RAHU" against "MARS" |
| PyJHora (3) | full | dashaStart | the baseline | `timed` | 0.0% of 4080; median 169.4, largest 7240.1 | worst c049 `dashaStart/VIMSHOTTARI/8` |
| PyJHora (3) | dashas | dashaLord | the baseline | `timed` | 92.5% of 4293 | c007 `dashaLord/VIMSHOTTARI/0` "MARS" against "MOON"; c007 `dashaLord/VIMSHOTTARI/1` "RAHU" against "MARS"; c007 `dashaLord/VIMSHOTTARI/1.0` "RAHU" against "MARS" |
| PyJHora (3) | dashas | dashaStart | the baseline | `timed` | 0.0% of 4240; median 164.6, largest 7240.1 | worst c049 `dashaStart/VIMSHOTTARI/8` |
| the committee's print (1) | day | event | the SDK | `timed` | 0.0% of 52; median 170.4, largest 11334.7 | worst npns 2025-04-17 `event/tithi/0` |
| the committee's print (1) | day | event | the SDK | `same` | 34.6% of 52; median 124.1, largest 11334.7 | worst npns 2025-04-17 `event/tithi/0` |
| the committee's print (1) | day | event | the baseline | `timed` | 34.6% of 52; median 124.6, largest 11340.9 | worst npns 2025-04-17 `event/tithi/0` |
| Raman's Example 56 (2) | strength | shadbala | the SDK | `timed` | 0.0% of 7; median 40.3, largest 152.6 | worst raman `shadbala/MOON/total` |
| Raman's Example 56 (2) | strength | shadbala | the SDK | `same` | 0.0% of 7; median 24.5, largest 38.6 | worst raman `shadbala/MOON/total` |
| Raman's Example 56 (2) | strength | shadbala | the baseline | `timed` | 0.0% of 7; median 24.5, largest 38.6 | worst raman `shadbala/MOON/total` |
| Raman's Example 56 (2) | strength | shadbala | the SDK | `source` | 100.0% of 7; median 0.043, largest 0.693 |  |
| Charak's worked year (2) | varsha | event | the SDK | `timed` | 100.0% of 1; median 51.0, largest 51.0 |  |
| Charak's worked year (2) | varsha | office | the SDK | `timed` | 100.0% of 7 |  |
| Charak's worked year (2) | varsha | event | the SDK | `same` | 0.0% of 1; median 286.0, largest 286.0 | worst charak `event/return` |
| Charak's worked year (2) | varsha | office | the SDK | `same` | 100.0% of 7 |  |
| Charak's worked year (2) | varsha | event | the baseline | `timed` | 0.0% of 1; median 267.9, largest 267.9 | worst charak `event/return` |
| Charak's worked year (2) | varsha | office | the baseline | `timed` | 42.9% of 7 | charak `office/yearLord` "MOON" against "SUN"; charak `office/dinaRatri` "MOON" against "SUN"; charak `office/triRashi` "JUPITER" against "MARS" |
| *Kalaprakasika* XIII (1) | milan | porutham | the SDK | `timed` | 100.0% of 165 |  |
| *Kalaprakasika* XIII (1) | milan | porutham | the SDK | `same` | 100.0% of 156 |  |
| *Kalaprakasika* XIII (1) | milan | porutham | the baseline | `timed` | 92.1% of 165 | c004 `porutham/STREE_DEERGHA` true against false; c009 `porutham/STREE_DEERGHA` true against false; c016 `porutham/STREE_DEERGHA` true against false |

The SDK left unanswered under a convention:

- c028, `chart`, under `same`: BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise, not 102.11188888888888
- c029, `chart`, under `same`: BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise, not 102.18277777777777
- c028, `full`, under `same`: BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise, not 102.11188888888888
- c029, `full`, under `same`: BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise, not 102.18277777777777
- c028, `vargas`, under `same`: BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise, not 102.11188888888888
- c029, `vargas`, under `same`: BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise, not 102.18277777777777
- c047, `strength`, under `same`: the Sun at JD 2378132.0006944444: the provider answered OutOfRange
- c048, `strength`, under `same`: the Sun at JD 2597685.2871633684: the provider answered OutOfRange
- c028, `batch`, under `same`: BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise, not 102.11188888888888
- c029, `batch`, under `same`: BHAVA_LAGNA needs an elapsed time of 0 to 36 hours since sunrise, not 102.18277777777777
