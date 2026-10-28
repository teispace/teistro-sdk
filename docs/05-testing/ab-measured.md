# The SDK against the baseline engine

Status: `generated` by `cargo xtask ab` from `docs/05-testing/ab/*.json`, gated by `check-ab`. Do not edit. Each record is written by `cargo xtask measure-ab`, which runs `bindings/node/bench/ab.mjs` with the baseline engine's adapter (`03-design/ab-baseline.md`).

Both sides answer every birth of the golden corpus in one Node process, the SDK as `parashari-classical` over Teimeris (`kp-default` for `kp`), each request asking the SDK for at least what the baseline's answer holds. A birth on which the two Suns stand more than an arcminute apart is reported and kept out, and a record that kept any out is no claim. A request a side refuses is kept out of that request alone, and only at the corpus's range-edge births, where the data's years end and a day's spans run past them, or at its polar births for a rule that needs a sunrise on a day without one. So is a request a side answers with nothing it asked for, listed apart, since it was answered. A round times every birth on one side and then the other, the next round the other way round, and a figure is the median of at least 15 rounds, per birth in microseconds.

Then each side, in processes of its own, answers `chart`, `day` and `full` under load on 1, 2, 4 and every thread the machine offers, in worker threads and in processes, at least 3 trials a count; for 30 seconds on one thread, cut into 10 windows for the tail; from a fresh process 9 times for the cold start; and 10000 charts in each of 3 processes for the memory, the heap read after collecting until two reads agree.

**The claim** is that the SDK is not slower: the gate fails a request or a percentile where its median is the larger, and a throughput where it is the smaller, when the two fastest-to-slowest ranges are apart. Where they meet the ratio reads `within spread`. The cold start and the memory are compared and not claimed (§8). Only the baseline may go unmeasured, and only in worker threads, with the reason recorded.

**Agreement and accuracy:** over the same inputs each answer is read into facts and the two sides' compared field by field, under the SDK's timed profiles and under the ones that cast as the baseline casts, and each side is held to a reference neither side computed where one exists (§10, §11). The gate holds every agreement and reference share to the floor the record measured, and each request's named cause to what the record shows.

No other request, and no throughput or percentile, is measurably slower through the SDK.

## `macos-aarch64`

macos aarch64, commit `0b36592cd`, Node v26.12.0, 10 threads offered; the sdk `teistro 0.0.0`, the baseline `baseline 0.6.173`.

### Requests, per request in µs

| request | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart (a birth chart with every varga, its drishti, upagrahas, houses, dignities, arudhas and Vimshottari to depth 3) | 1435 | 7728 | 0.19× | 1377 to 1901 | 7492 to 13353 |
| day (the almanac day of the birth's date at its place, its limbs, kaalas, choghadiya and horas) | 1035 | 11864 | 0.09× | 992 to 2827 | 11726 to 51937 |
| full (the whole natal chart: every section and varga, 22 dashas to depth 3, every shipped rule and the longevity) | 22912 | 54517 | 0.42× | 22405 to 24157 | 53391 to 63004 |
| vargas (the chart's foundation, every divisional chart with it) | 907 | 5952 | 0.15× | 860 to 941 | 5877 to 7530 |
| yogas (the yogas and doshas of a chart) | 3322 | 6012 | 0.55× | 3260 to 3414 | 5873 to 6500 |
| strength (the Shadbala, Bhava bala and Ashtakavarga) | 591 | 5976 | 0.10× | 544 to 634 | 5879 to 6475 |
| dashas (eighteen dasha systems to depth 3) | 15352 | 42358 | 0.36× | 15209 to 16404 | 41695 to 43376 |
| month (thirty almanac days from the birth's date) | 12136 | 357718 | 0.03× | 11881 to 13023 | 351574 to 370281 |
| milan (the birth matched with the next one of the corpus) | 694 | 11561 | 0.06× | 661 to 731 | 11407 to 11966 |
| varsha (the first annual chart, its yogas, sahams and Mudda dasha) | 2601 | 6510 | 0.40× | 2547 to 2647 | 6435 to 6661 |
| kp (the KP cusps, sub lords, significators and ruling planets) | 587 | 5773 | 0.10× | 557 to 651 | 5713 to 6354 |
| gochar (the grahas thirty years on over the chart, with the Ashtakavarga) | 574 | 5968 | 0.10× | 542 to 725 | 5871 to 6088 |
| sadeSati (Sade Sati and the dhaiyyas over thirty years) | 2477 | 23232 | 0.11× | 2446 to 2527 | 22948 to 24090 |
| batch (ten whole charts an hour apart, at one place) | 221294 | 567090 | 0.39× | 219446 to 243919 | 565352 to 577404 |

Refused, and kept out of that request's timing:

- c028, `month`, by the sdk: the month-start rule needs a sunrise on RD 725934 at 69.6492°N 18.9553°E 10 m, where the Sun does not rise
- c029, `month`, by the sdk: the month-start rule needs a sunrise on RD 726115 at 69.6492°N 18.9553°E 10 m, where the Sun does not rise
- c047, `day`, by the sdk: MOON at JD 2378496.179673321: OutOfRange
- c047, `full`, by the sdk: MOON at JD 2378496.3978494625: OutOfRange
- c047, `yogas`, by the sdk: MOON at JD 2378496.3978494625: OutOfRange
- c047, `month`, by the sdk: MOON at JD 2378496.179673321: OutOfRange
- c047, `batch`, by the sdk: MOON at JD 2378496.3978494625: OutOfRange
- c048, `day`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange
- c048, `full`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange
- c048, `yogas`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange
- c048, `month`, by the sdk: SUN at JD 2597642.2629888887 UT1: the provider answered OutOfRange
- c048, `gochar`, by the sdk: the provider could not place SUN: OutOfRange
- c048, `batch`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange

Answered with nothing, and kept out of that request's timing:

- c048, `varsha`, by the sdk: answered no pravesha (an empty `praveshas`)

### Throughput, in requests a second

All threads or processes together, each with its side of its own; the ratio is the SDK's throughput over the baseline's, and efficiency the share of linear throughput the count keeps.

| request | mode | at once | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest | SDK efficiency | baseline efficiency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| chart | threads | 1 | 736 | 140 | 5.25× | 720 to 741 | 140 to 141 | 1.00 | 1.00 |
| chart | threads | 2 | 1417 | not measured | — | 1402 to 1425 |  | 0.96 | — |
| chart | threads | 4 | 2717 | not measured | — | 2694 to 2731 |  | 0.92 | — |
| chart | threads | 8 | 4227 | not measured | — | 4082 to 4289 |  | 0.72 | — |
| chart | threads | 10 | 4409 | not measured | — | 4291 to 4539 |  | 0.60 | — |
| chart | processes | 1 | 739 | 141 | 5.24× | 729 to 744 | 141 to 142 | 1.00 | 1.00 |
| chart | processes | 2 | 1466 | 282 | 5.21× | 1453 to 1472 | 280 to 282 | 0.99 | 1.00 |
| chart | processes | 4 | 2883 | 550 | 5.24× | 2824 to 2906 | 547 to 554 | 0.97 | 0.98 |
| chart | processes | 8 | 4435 | 832 | 5.33× | 4302 to 4605 | 822 to 873 | 0.75 | 0.74 |
| chart | processes | 10 | 4619 | 894 | 5.17× | 4453 to 4767 | 859 to 915 | 0.62 | 0.63 |
| day | threads | 1 | 1702 | 83.3 | 20.43× | 1662 to 1705 | 81.3 to 84.0 | 1.00 | 1.00 |
| day | threads | 2 | 3067 | not measured | — | 3053 to 3102 |  | 0.90 | — |
| day | threads | 4 | 5635 | not measured | — | 5589 to 5651 |  | 0.83 | — |
| day | threads | 8 | 8950 | not measured | — | 8726 to 9009 |  | 0.66 | — |
| day | threads | 10 | 9365 | not measured | — | 9155 to 9421 |  | 0.55 | — |
| day | processes | 1 | 1709 | 83.7 | 20.42× | 1694 to 1711 | 82.7 to 84.0 | 1.00 | 1.00 |
| day | processes | 2 | 3391 | 167 | 20.35× | 3331 to 3405 | 165 to 167 | 0.99 | 1.00 |
| day | processes | 4 | 6726 | 329 | 20.44× | 6604 to 6777 | 323 to 331 | 0.98 | 0.98 |
| day | processes | 8 | 10820 | 520 | 20.81× | 10308 to 10860 | 516 to 521 | 0.79 | 0.78 |
| day | processes | 10 | 11476 | 556 | 20.64× | 10462 to 11670 | 544 to 564 | 0.67 | 0.66 |
| full | threads | 1 | 47.3 | 18.3 | 2.58× | 46.3 to 47.3 | 18.0 to 18.3 | 1.00 | 1.00 |
| full | threads | 2 | 87.3 | not measured | — | 76.3 to 91.0 |  | 0.92 | — |
| full | threads | 4 | 183 | not measured | — | 183 to 184 |  | 0.97 | — |
| full | threads | 8 | 182 | not measured | — | 172 to 194 |  | 0.48 | — |
| full | threads | 10 | 274 | not measured | — | 269 to 276 |  | 0.58 | — |
| full | processes | 1 | 47.3 | 19.0 | 2.49× | 47.0 to 47.7 | 18.7 to 19.0 | 1.00 | 1.00 |
| full | processes | 2 | 93.3 | 37.3 | 2.50× | 92.7 to 94.0 | 37.3 to 38.0 | 0.99 | 0.98 |
| full | processes | 4 | 185 | 73.3 | 2.52× | 182 to 187 | 72.0 to 74.0 | 0.98 | 0.96 |
| full | processes | 8 | 269 | 105 | 2.56× | 267 to 277 | 101 to 108 | 0.71 | 0.69 |
| full | processes | 10 | 284 | 103 | 2.75× | 273 to 288 | 100 to 105 | 0.60 | 0.54 |

Not measured, and why:

- the baseline, `chart` on 2 threads: died of SIGSEGV: nothing on stderr
- the baseline, `chart` on 4 threads: died of SIGTRAP: nothing on stderr
- the baseline, `chart` on 8 threads: died of SIGABRT: nothing on stderr
- the baseline, `chart` on 10 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 2 threads: Entity not found: TITHI/undefined
- the baseline, `day` on 4 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 8 threads: died of SIGABRT: nothing on stderr
- the baseline, `day` on 10 threads: died of SIGSEGV: nothing on stderr
- the baseline, `full` on 2 threads: died of SIGABRT: nothing on stderr
- the baseline, `full` on 4 threads: died of SIGABRT: nothing on stderr
- the baseline, `full` on 8 threads: died of SIGSEGV: nothing on stderr
- the baseline, `full` on 10 threads: died of SIGSEGV: nothing on stderr

### The tail of a sustained run, per answer in µs

One thread, each side in a process of its own; a percentile is the median of the windows' with the fastest and slowest. Answers: `chart` 22116 through the SDK and 4296 through the baseline; `day` 51304 through the SDK and 2528 through the baseline; `full` 1413 through the SDK and 568 through the baseline.

| percentile | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart, p50 | 1313 | 6951 | 0.19× | 1281 to 1340 | 6891 to 6976 |
| chart, p95 | 1533 | 8352 | 0.18× | 1450 to 1835 | 8268 to 8390 |
| chart, p99 | 2158 | 8938 | 0.24× | 1923 to 3353 | 8785 to 9062 |
| day, p50 | 571 | 11869 | 0.05× | 568 to 572 | 11813 to 11901 |
| day, p95 | 687 | 12805 | 0.05× | 680 to 696 | 12685 to 12890 |
| day, p99 | 898 | 13309 | 0.07× | 890 to 907 | 13023 to 13633 |
| full, p50 | 20991 | 51901 | 0.40× | 20817 to 21042 | 50089 to 52580 |
| full, p95 | 23300 | 67369 | 0.35× | 22481 to 23797 | 65267 to 68795 |
| full, p99 | 24497 | 73198 | 0.33× | 23388 to 26453 | 70789 to 82110 |

### Cold start, in ms

| phase | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| open (from the process's start to the side being ready, the import and the engine or context included) | 32.3 | 689 | 0.05× | 30.2 to 35.0 | 675 to 714 |
| first (the first chart) | 27.1 | 34.9 | 0.78× | 25.8 to 28.7 | 33.3 to 36.9 |
| process (the whole process, from spawn to exit) | 147 | 817 | 0.18× | 145 to 158 | 804 to 843 |

### Memory over 10000 `chart` requests, after 200 to warm

| measure | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| peak-rss (the peak resident set over the process's life), MB | 188 | 297 | 0.63× | 186 to 203 | 289 to 298 |
| heap (the heap and external bytes, settled, after the workload), MB | 8.94 | 46.0 | 0.19× | 8.93 to 8.94 | 46.0 to 46.0 |
| heap-growth (their growth over the workload), KB | -1041 | 221 | -4.71× | -1044 to -1037 | 220 to 221 |

### Agreement and accuracy

The share of facts on which the two sides agree, exactly or within the field's tolerance, under the SDK's timed profiles (`timed`) and under the profiles that cast as the baseline casts (`same`), so that what still differs under `same` is method (§10); and each side against a reference neither side computed, with its evidence rank, where one exists (§11). Closer is the larger share of the reference's facts matched, the SDK's under its best convention.

| request | agreement, `timed` | agreement, `same` | reference | closer to it | dominant cause |
|---|---:|---:|---|---|---|
| chart | 54.5% of 12485 | 100.0% of 12485 | Swiss Ephemeris (2); PyJHora (3) | the baseline: SDK 99.6% `timed`, baseline 100.0%; the SDK: SDK 72.8% `timed`, baseline 46.5% | convention: the centre: `timed` casts the Moon from the Earth's centre and the baseline from the birthplace, up to an hour of arc apart, on a mean ayanamsha where the baseline's is nutated (12″), which moves the Vimshottari's starts; under `same` only the lagnas of 2350 and 2399 differ, the SDK's IAU 2006 sidereal time against Swiss Ephemeris's that far from J2000 |
| day | 99.2% of 860 | 96.5% of 860 | the committee's print (1) | neither: SDK 34.6% `timed`, baseline 34.6% | method: a day with no sunrise: both now take the refracted upper limb, so only the polar births differ, which the SDK reckons by its polar policy (civil midnight under `timed`, the conformance profile's nearest sunrise and sunset under `same`) and the baseline by its civil-day bounds. The committee's tithi ends are Surya Siddhanta's, hours from both |
| full | 64.1% of 266007 | 99.6% of 266001 | Swiss Ephemeris (2); PyJHora (3) | the baseline: SDK 99.8% `timed`, baseline 100.0%; the SDK: SDK 72.7% `timed`, baseline 46.4% | convention: the centre, and the classical Shadbala readings; under `same` the yogas' readings (as `yogas`), c022's kaala bala, a birth the baseline places in the wrong Hindu day (crux C35), and longevity's: BPHS ch. 43 v. 33 pairs Saturn with the Moon where the baseline pairs the lagna with it, which moves the class the pairs decide; v. 11 takes the Moon for a benefic waxing or waning in the visible half's reduction, the baseline a waning Moon for a malefic; vv. 20 to 22 treble Amsayu's years exalted or in its own sign and double them in its own navamsha or drekkana, where the baseline doubles its own sign, trebles a retrograde graha and halves a debilitated one; Nisargayu's listed years are taken whole, which the baseline reduces; and the cancellations of Balarishta. Under `timed` also the hora lagna, counted in `parashari-classical`'s proportional ghatis, and BPHS's own readings of the three spans (cruxes C104, C303) |
| vargas | 97.1% of 11550 | 100.0% of 11550 | agreement only | — | convention: the centre: a Moon a few arcminutes from a division's edge |
| yogas | 96.9% of 4611 | 97.1% of 4611 | agreement only | — | method: the rules' readings: the SDK applies Kemadruma's cancellations and reads the Nabhasa shapes, Gaja Kesari and Chamara by its texts' clauses, where the baseline holds them present |
| strength | 69.1% of 8635 | 98.6% of 8321 | Raman's Example 56 (2) | the SDK: SDK 100.0% `timed`, baseline 0.0% | convention: the readings: `timed` is the root's, Sripati's, which reproduces Raman's totals, and the baseline its own; under `same` the largest gaps are c022's, a birth the baseline places in the wrong Hindu day (crux C35) |
| dashas | 61.9% of 244860 | 100.0% of 244860 | PyJHora (3) | the SDK: SDK 100.0% `source`, baseline 46.5% | convention: the centre: the Moon's place sets every system's start; PyJHora casts from the Earth's centre as `timed` does |
| month | 99.1% of 24746 | 100.0% of 24752 | agreement only | — | convention: the sunrise, as `day`; under `same` every limb agrees |
| milan | 73.9% of 990 | 81.9% of 990 | *Kalaprakasika* XIII (1) | the SDK: SDK 100.0% `timed`, baseline 92.1% | method: the koota tables: Vashya, Gana, Yoni and Bhakoot score apart under `same`, and the baseline's Stree-deergha agrees beyond the 10th or on the bride's own star (crux C779), where *Kalaprakasika* prints beyond the 13th; the Kuja dosha, under `timed` the verse's (Mars from the lagna in five houses, *Manasagari*, cruxes C285, C286) where the baseline reads three references and six houses, and under `same` the baseline's own cancellations, three of nine, which lift a dosha the SDK keeps |
| varsha | 26.1% of 10908 | 24.4% of 10908 | Charak's worked year (2) | the SDK: SDK 100.0% `timed`, baseline 37.5% | the baseline: the year lord: on Charak's worked year the SDK names all seven offices and the baseline three, its year lord the Moon for the Sun; the sahams are other formulas, and the Mudda dasha follows the year lord |
| kp | 80.8% of 8690 | 97.1% of 8690 | Swiss Ephemeris (2) | the baseline: SDK 97.9% `timed`, baseline 98.2% | convention: the frame: `kp-default` casts from the Earth's centre on its own node, the baseline from the birthplace on the mean node; under `same` c043's cusps, where the baseline's Placidus falls back at 64.8° N and Swiss Ephemeris and the SDK do not, and the lagnas of 2350 and 2399, as `chart`; and an intercepted sign's lord, which the baseline counts a significator of its house and the SDK reports and does not count (crux C154). Under `timed` also the day lord, which `kp-default` takes from sunrise to sunrise as KP Reader VI does and the baseline from the civil day, so a birth before sunrise rules by the day before (c001); `same` asks the civil day (`kp.day_lord_day = CIVIL`, crux C151) |
| gochar | 81.7% of 2779 | 98.4% of 2779 | Swiss Ephemeris (2) | neither: SDK 100.0% `timed`, baseline 100.0% | convention: the ayanamsha's basis: `timed` takes the mean and the baseline the nutated, about 12″ apart; under `same`, cast from the Earth's centre as the baseline casts a transit, the transits agree to a hundredth of an arcsecond and only c051's transit bindus differ; and the vedha of Phaladeepika ch. 26: the nodes take the Sun's pairs (v. 2, crux C136) where the baseline gives them Mars's for the 3rd and the 6th, never obstruct each other (C140), Venus's 11th and 12th pair with the 3rd and the 6th (v. 8), and the Sun and Saturn, the Moon and Mercury do not obstruct each other (vv. 3 to 6), where the baseline exempts the Sun and the Moon |
| sadeSati | 7.6% of 328 | 100.0% of 328 | agreement only | — | convention: the ayanamsha's basis, as `gochar`: 12″ of Saturn moves an edge by hours; under `same` every edge agrees within two seconds |
| batch | 65.2% of 2660070 | 99.6% of 2660010 | Swiss Ephemeris (2) | the baseline: SDK 99.8% `timed`, baseline 100.0% | convention: as `full`, at ten instants |

#### Agreement by field

A continuous field's median and largest difference are in its unit; `same`'s first mismatches, or its worst input, follow.

| request | field | unit, tolerance | `timed` | `same` | unpaired | where `same` differs |
|---|---|---|---:|---:|---:|---|
| chart | longitude | ″, 1.000 | 0.9% of 550; median 13.9, largest 3576.1 | 99.6% of 550; median 0.000, largest 12.0 | 0 | worst c048 `longitude/LAGNA` |
| chart | sign | exact | 99.6% of 550 | 100.0% of 550 | 0 |  |
| chart | nakshatra | exact | 99.1% of 550 | 100.0% of 550 | 0 |  |
| chart | pada | exact | 97.5% of 550 | 100.0% of 550 | 0 |  |
| chart | retrograde | exact | 100.0% of 495 | 100.0% of 495 | 0 |  |
| chart | dashaLord | exact | 92.7% of 4950 | 100.0% of 4950 | 0 |  |
| chart | dashaStart | d, 1.000 | 1.9% of 4840; median 158.2, largest 7240.8 | 100.0% of 4840; median 0.000, largest 0.001 | 0 |  |
| day | event | s, 60.0 | 98.1% of 377; median 3.668, largest 86400.0 | 96.3% of 377; median 2.500, largest 3038077.9 | 1 | worst c028 `event/yoga/0` |
| day | limb | exact | 100.0% of 483 | 96.7% of 483 | 1 | c028 `limb/tithi/0` 11 against 6; c028 `limb/tithi/1` 12 against 7; c028 `limb/nakshatra/0` 17 against 10 |
| full | longitude | ″, 1.000 | 0.9% of 530; median 14.0, largest 3576.1 | 99.8% of 530; median 0.000, largest 7.090 | 0 | worst c046 `longitude/LAGNA` |
| full | sign | exact | 99.6% of 530 | 100.0% of 530 | 0 |  |
| full | nakshatra | exact | 99.1% of 530 | 100.0% of 530 | 0 |  |
| full | pada | exact | 97.4% of 530 | 100.0% of 530 | 0 |  |
| full | retrograde | exact | 100.0% of 477 | 100.0% of 477 | 0 |  |
| full | varga | exact | 97.0% of 11130 | 100.0% of 11130 | 0 |  |
| full | dashaLord | exact | 85.2% of 118932 | 100.0% of 118932 | 19716 |  |
| full | dashaStart | d, 1.000 | 38.1% of 117024; median 83.1, largest 21936.9 | 100.0% of 117024; median 0.000, largest 0.002 | 19292 |  |
| full | shadbala | virupas, 1.000 | 29.1% of 2597; median 12.3, largest 333.9 | 97.6% of 2597; median 0.000, largest 120.0 | 0 | worst c022 `shadbala/MARS/total` |
| full | bhavaBala | virupas, 1.000 | 0.6% of 636; median 73.2, largest 328.1 | 91.7% of 636; median 0.003, largest 120.0 | 0 | worst c022 `bhavaBala/5` |
| full | bindus | exact | 97.9% of 5088 | 100.0% of 5088 | 0 |  |
| full | yoga | exact | 96.9% of 4611 | 97.1% of 4611 | 0 | c001 `yoga/LUNAR_KEMADRUMA` false against true; c002 `yoga/LUNAR_KEMADRUMA` false against true; c003 `yoga/LUNAR_KEMADRUMA` false against true |
| full | drishti | exact | 99.6% of 477 | 100.0% of 477 | 0 |  |
| full | jaiminiPair | exact | 64.2% of 159 | 81.7% of 153 | 110 | c001 `jaiminiPair/decided` "medium" against "long"; c004 `jaiminiPair/decided` "short" against "long"; c005 `jaiminiPair/decided` "medium" against "short" |
| full | ayurdaya | y, 0.010 | 34.1% of 2544; median 0.719, largest 69.7 | 73.4% of 2544; median 0.000, largest 56.3 | 0 | worst c043 `ayurdaya/nisargayu/total` |
| full | maraka | exact | 98.1% of 106 | 100.0% of 106 | 0 |  |
| full | balarishta | exact | 97.2% of 106 | 98.1% of 106 | 0 | c022 `balarishta/standing` true against false; c046 `balarishta/standing` false against true |
| vargas | varga | exact | 97.1% of 11550 | 100.0% of 11550 | 0 |  |
| yogas | yoga | exact | 96.9% of 4611 | 97.1% of 4611 | 0 | c001 `yoga/LUNAR_KEMADRUMA` false against true; c002 `yoga/LUNAR_KEMADRUMA` false against true; c003 `yoga/LUNAR_KEMADRUMA` false against true |
| strength | shadbala | virupas, 1.000 | 29.5% of 2695; median 11.7, largest 333.9 | 97.6% of 2597; median 0.000, largest 120.0 | 0 | worst c022 `shadbala/MARS/total` |
| strength | bhavaBala | virupas, 1.000 | 0.6% of 660; median 72.7, largest 345.1 | 91.7% of 636; median 0.003, largest 120.0 | 0 | worst c022 `bhavaBala/5` |
| strength | bindus | exact | 97.9% of 5280 | 100.0% of 5088 | 0 |  |
| dashas | dashaLord | exact | 85.5% of 123420 | 100.0% of 123420 | 0 |  |
| dashas | dashaStart | d, 1.000 | 38.0% of 121440; median 83.1, largest 21936.9 | 100.0% of 121440; median 0.000, largest 0.004 | 0 |  |
| month | event | s, 60.0 | 98.1% of 10843; median 3.531, largest 83416.0 | 100.0% of 10846; median 2.335, largest 9.772 | 0 |  |
| month | limb | exact | 100.0% of 13903 | 100.0% of 13906 | 0 |  |
| milan | koota | exact | 66.5% of 495 | 68.5% of 495 | 0 | c001 `koota/VASHYA` 0 against 1; c001 `koota/GANA` 5 against 6; c001 `koota/TOTAL` 27.5 against 29.5 |
| milan | porutham | exact | 89.1% of 220 | 94.1% of 220 | 0 | c004 `porutham/STREE_DEERGHA` false against true; c009 `porutham/STREE_DEERGHA` false against true; c016 `porutham/STREE_DEERGHA` false against true |
| milan | kuja | exact | 47.3% of 110 | 90.9% of 110 | 0 | c013 `kuja/BRIDE` true against false; c014 `kuja/GROOM` true against false; c023 `kuja/BRIDE` true against false |
| milan | marriageDosha | exact | 93.9% of 165 | 100.0% of 165 | 0 |  |
| varsha | event | s, 60.0 | 24.1% of 54; median 112.4, largest 334.9 | 16.7% of 54; median 139.1, largest 213.2 | 0 | worst c008 `event/return` |
| varsha | office | exact | 52.4% of 378 | 52.4% of 378 | 0 | c001 `office/yearLord` "VENUS" against "SUN"; c001 `office/triRashi` "VENUS" against "MARS"; c001 `office/dinaRatri` "MARS" against "SUN" |
| varsha | longitude | ″, 1.000 | 0.0% of 864; median 241223.1, largest 647394.3 | 0.0% of 864; median 244083.2, largest 647101.3 | 1350 | worst c037 `longitude/saham/YASHA` |
| varsha | dashaLord | exact | 48.1% of 4860 | 44.4% of 4860 | 540 | c003 `dashaLord/MUDDA/0` "VENUS" against "SUN"; c003 `dashaLord/MUDDA/0.0` "VENUS" against "SUN"; c003 `dashaLord/MUDDA/0.1` "SUN" against "MOON" |
| varsha | dashaStart | d, 1.000 | 6.2% of 4752; median 13.0, largest 56.4 | 6.1% of 4752; median 12.7, largest 53.0 | 540 | worst c038 `dashaStart/MUDDA/8.8` |
| kp | longitude | ″, 1.000 | 1.1% of 1155; median 14.5, largest 335680.9 | 96.2% of 1155; median 0.000, largest 335678.9 | 0 | worst c043 `longitude/cusp/2` |
| kp | kpLord | exact | 93.0% of 4620 | 98.4% of 4620 | 0 | c027 `kpLord/cusp/2/sign` "MARS" against "SATURN"; c027 `kpLord/cusp/2/star` "KETU" against "MOON"; c027 `kpLord/cusp/2/subSub` "SATURN" against "JUPITER" |
| kp | kpSignificator | exact | 93.3% of 2640 | 95.2% of 2640 | 0 | c005 `kpSignificator/6/inLordsStar` "MARS,JUPITER" against "MARS,JUPITER,RAHU"; c005 `kpSignificator/6/lord` "MOON" against "SUN,MOON"; c008 `kpSignificator/6/lord` "MERCURY" against "MERCURY,VENUS" |
| kp | kpRuler | exact | 91.6% of 275 | 100.0% of 275 | 0 |  |
| gochar | longitude | ″, 1.000 | 7.4% of 486; median 12.0, largest 18.3 | 100.0% of 486; median 0.000, largest 0.008 | 0 |  |
| gochar | house | exact | 100.0% of 486 | 100.0% of 486 | 0 |  |
| gochar | gochar | exact | 96.4% of 1051 | 96.7% of 1051 | 0 | c002 `gochar/KETU/vedha` false against true; c004 `gochar/KETU/vedha` true against false; c005 `gochar/MERCURY/vedha` false against true |
| gochar | bindus | exact | 97.2% of 756 | 98.8% of 756 | 0 | c051 `bindus/transit/SUN` 1 against 2; c051 `bindus/transitSarva/SUN` 27 against 28; c051 `bindus/transit/MOON` 4 against 5 |
| sadeSati | event | s, 60.0 | 7.6% of 328; median 2684.4, largest 35385.2 | 100.0% of 328; median 0.154, largest 1.667 | 7 |  |
| batch | longitude | ″, 1.000 | 1.4% of 5300; median 14.7, largest 3623.9 | 99.8% of 5300; median 0.000, largest 7.090 | 0 | worst c046 `longitude/LAGNA@0` |
| batch | sign | exact | 99.8% of 5300 | 100.0% of 5300 | 0 |  |
| batch | nakshatra | exact | 99.5% of 5300 | 100.0% of 5300 | 0 |  |
| batch | pada | exact | 98.4% of 5300 | 100.0% of 5300 | 0 |  |
| batch | retrograde | exact | 100.0% of 4770 | 100.0% of 4770 | 0 |  |
| batch | varga | exact | 97.1% of 111300 | 100.0% of 111300 | 0 |  |
| batch | dashaLord | exact | 87.1% of 1189320 | 100.0% of 1189320 | 197160 |  |
| batch | dashaStart | d, 1.000 | 38.4% of 1170240; median 81.3, largest 22769.3 | 100.0% of 1170240; median 0.000, largest 0.002 | 192920 |  |
| batch | shadbala | virupas, 1.000 | 28.8% of 25970; median 12.4, largest 333.9 | 98.2% of 25970; median 0.000, largest 214.9 | 0 | worst c022 `shadbala/SUN/total@5` |
| batch | bhavaBala | virupas, 1.000 | 0.5% of 6360; median 74.2, largest 432.3 | 93.6% of 6360; median 0.003, largest 214.9 | 0 | worst c022 `bhavaBala/6@5` |
| batch | bindus | exact | 98.7% of 50880 | 100.0% of 50880 | 0 |  |
| batch | yoga | exact | 97.1% of 46110 | 97.2% of 46110 | 0 | c001 `yoga/LUNAR_KEMADRUMA@0` false against true; c001 `yoga/LUNAR_KEMADRUMA@1` false against true; c001 `yoga/NABHASA_SARPA@1` true against false |
| batch | drishti | exact | 99.4% of 4770 | 100.0% of 4770 | 0 |  |
| batch | jaiminiPair | exact | 70.3% of 1590 | 87.2% of 1530 | 1100 | c001 `jaiminiPair/decided@0` "medium" against "long"; c001 `jaiminiPair/decided@1` "long" against "medium"; c001 `jaiminiPair/decided@2` "medium" against "short" |
| batch | ayurdaya | y, 0.010 | 33.8% of 25440; median 0.748, largest 81.0 | 72.1% of 25440; median 0.000, largest 74.8 | 0 | worst c034 `ayurdaya/nisargayu/total@9` |
| batch | maraka | exact | 99.8% of 1060 | 100.0% of 1060 | 0 |  |
| batch | balarishta | exact | 97.0% of 1060 | 97.4% of 1060 | 0 | c007 `balarishta/standing@2` true against false; c008 `balarishta/present@7` false against true; c008 `balarishta/present@8` false against true |

#### Against the references

Each side under each convention it was held to the rung under; the SDK's `source` is `timed` given the reading the source works by.

| reference | request | field | side | convention | matching | where it differs |
|---|---|---|---|---|---:|---|
| Swiss Ephemeris (2) | chart | longitude | the SDK | `timed` | 99.6% of 550; median 0.000, largest 12.0 | worst c048 `longitude/LAGNA` |
| Swiss Ephemeris (2) | full | longitude | the SDK | `timed` | 99.8% of 530; median 0.000, largest 7.098 | worst c046 `longitude/LAGNA` |
| Swiss Ephemeris (2) | batch | longitude | the SDK | `timed` | 99.8% of 5300; median 0.000, largest 7.098 | worst c046 `longitude/LAGNA@0` |
| Swiss Ephemeris (2) | kp | longitude | the SDK | `timed` | 97.9% of 1131; median 0.000, largest 12.0 | worst c048 `longitude/cusp/1` |
| Swiss Ephemeris (2) | gochar | longitude | the SDK | `timed` | 100.0% of 486; median 0.000, largest 0.001 |  |
| Swiss Ephemeris (2) | chart | longitude | the SDK | `same` | 99.6% of 550; median 0.000, largest 12.0 | worst c048 `longitude/LAGNA` |
| Swiss Ephemeris (2) | full | longitude | the SDK | `same` | 99.8% of 530; median 0.000, largest 7.090 | worst c046 `longitude/LAGNA` |
| Swiss Ephemeris (2) | batch | longitude | the SDK | `same` | 99.8% of 5300; median 0.000, largest 7.090 | worst c046 `longitude/LAGNA@0` |
| Swiss Ephemeris (2) | kp | longitude | the SDK | `same` | 97.9% of 1131; median 0.000, largest 12.0 | worst c048 `longitude/cusp/1` |
| Swiss Ephemeris (2) | gochar | longitude | the SDK | `same` | 100.0% of 486; median 0.000, largest 0.008 |  |
| Swiss Ephemeris (2) | chart | longitude | the baseline | `timed` | 100.0% of 550; median 0.000, largest 0.001 |  |
| Swiss Ephemeris (2) | full | longitude | the baseline | `timed` | 100.0% of 530; median 0.000, largest 0.000 |  |
| Swiss Ephemeris (2) | batch | longitude | the baseline | `timed` | 100.0% of 5300; median 0.000, largest 0.000 |  |
| Swiss Ephemeris (2) | kp | longitude | the baseline | `timed` | 98.2% of 1131; median 0.000, largest 335678.9 | worst c043 `longitude/cusp/2` |
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
| PyJHora (3) | dashas | dashaLord | the SDK | `source` | 100.0% of 4293 |  |
| PyJHora (3) | dashas | dashaStart | the SDK | `source` | 100.0% of 4240; median 0.097, largest 0.322 |  |
| the committee's print (1) | day | event | the SDK | `timed` | 34.6% of 52; median 124.1, largest 11334.7 | worst npns 2025-04-17 `event/tithi/0` |
| the committee's print (1) | day | event | the SDK | `same` | 34.6% of 52; median 124.1, largest 11334.7 | worst npns 2025-04-17 `event/tithi/0` |
| the committee's print (1) | day | event | the baseline | `timed` | 34.6% of 52; median 124.6, largest 11340.9 | worst npns 2025-04-17 `event/tithi/0` |
| Raman's Example 56 (2) | strength | shadbala | the SDK | `timed` | 100.0% of 7; median 0.043, largest 0.693 |  |
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
| *Kalaprakasika* XIII (1) | milan | porutham | the SDK | `same` | 100.0% of 165 |  |
| *Kalaprakasika* XIII (1) | milan | porutham | the baseline | `timed` | 92.1% of 165 | c004 `porutham/STREE_DEERGHA` true against false; c009 `porutham/STREE_DEERGHA` true against false; c016 `porutham/STREE_DEERGHA` true against false |

The SDK left unanswered under a convention:

- c047, `strength`, under `same`: the Sun at JD 2378132.0006944444: the provider answered OutOfRange
- c048, `strength`, under `same`: the Sun at JD 2597685.2871633684: the provider answered OutOfRange
