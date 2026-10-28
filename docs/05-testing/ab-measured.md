# The SDK against the baseline engine

Status: `generated` by `cargo xtask ab` from `docs/05-testing/ab/*.json`, gated by `check-ab`. Do not edit. Each record is written by `cargo xtask measure-ab`, which runs `bindings/node/bench/ab.mjs` with the baseline engine's adapter (`03-design/ab-baseline.md`).

Both sides answer every birth of the golden corpus in one Node process, the SDK as `parashari-classical` over Teimeris (`kp-default` for `kp`), each request asking the SDK for at least what the baseline's answer holds. A birth on which the two Suns stand more than an arcminute apart is reported and kept out, and a record that kept any out is no claim. A request a side refuses is kept out of that request alone, and only at the corpus's range-edge births, where the data's years end and a day's spans run past them, or at its polar births for a rule that needs a sunrise on a day without one. So is a request a side answers with nothing it asked for, listed apart, since it was answered. A round times every birth on one side and then the other, the next round the other way round, and a figure is the median of at least 15 rounds, per birth in microseconds.

Then each side, in processes of its own, answers `chart`, `day` and `full` under load on 1, 2, 4 and every thread the machine offers, in worker threads and in processes, at least 3 trials a count; for 30 seconds on one thread, cut into 10 windows for the tail; from a fresh process 9 times for the cold start; and 10000 charts in each of 3 processes for the memory, the heap read after collecting until two reads agree.

**The claim** is that the SDK is not slower: the gate fails a request or a percentile where its median is the larger, and a throughput where it is the smaller, when the two fastest-to-slowest ranges are apart. Where they meet the ratio reads `within spread`. The cold start and the memory are compared and not claimed (§8). Only the baseline may go unmeasured, and only in worker threads, with the reason recorded.

**Agreement and accuracy:** over the same inputs each answer is read into facts and the two sides' compared field by field, under the SDK's timed profiles and under the ones that cast as the baseline casts, and each side is held to a reference neither side computed where one exists (§10, §11). The gate holds every agreement and reference share to the floor the record measured, and each request's named cause to what the record shows.

No other request, and no throughput or percentile, is measurably slower through the SDK.

## `macos-aarch64`

macos aarch64, commit `6970f2332`, Node v26.12.0, 10 threads offered; the sdk `teistro 0.0.0`, the baseline `baseline 0.6.173`.

### Requests, per request in µs

| request | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart (a birth chart with every varga, its drishti, upagrahas, houses, dignities, arudhas and Vimshottari to depth 3) | 3373 | 34227 | within spread | 1799 to 14455 | 8792 to 62926 |
| day (the almanac day of the birth's date at its place, its limbs, kaalas, choghadiya and horas) | 2076 | 38491 | 0.05× | 1101 to 10023 | 14493 to 114050 |
| full (the whole natal chart: every section and varga, 22 dashas to depth 3, every shipped rule and the longevity) | 42150 | 149560 | within spread | 22707 to 148293 | 54809 to 442983 |
| vargas (the chart's foundation, every divisional chart with it) | 4492 | 20063 | within spread | 896 to 17486 | 6259 to 39631 |
| yogas (the yogas and doshas of a chart) | 11921 | 19713 | within spread | 4329 to 31055 | 6597 to 34883 |
| strength (the Shadbala, Bhava bala and Ashtakavarga) | 1174 | 15219 | 0.08× | 806 to 3191 | 7549 to 35449 |
| dashas (eighteen dasha systems to depth 3) | 42315 | 110475 | within spread | 16192 to 109953 | 42403 to 357355 |
| month (thirty almanac days from the birth's date) | 12737 | 378209 | 0.03× | 12135 to 16985 | 364740 to 476273 |
| milan (the birth matched with the next one of the corpus) | 718 | 11790 | 0.06× | 682 to 889 | 11667 to 14700 |
| varsha (the first annual chart, its yogas, sahams and Mudda dasha) | 2666 | 6688 | 0.40× | 2629 to 2794 | 6619 to 9207 |
| kp (the KP cusps, sub lords, significators and ruling planets) | 623 | 5994 | 0.10× | 580 to 1543 | 5896 to 7669 |
| gochar (the grahas thirty years on over the chart, with the Ashtakavarga) | 589 | 6139 | 0.10× | 553 to 632 | 6054 to 7054 |
| sadeSati (Sade Sati and the dhaiyyas over thirty years) | 2604 | 25099 | 0.10× | 2556 to 3187 | 24258 to 28200 |
| batch (ten whole charts an hour apart, at one place) | 249344 | 710128 | within spread | 225499 to 1072284 | 577403 to 2793559 |

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
| chart | threads | 1 | 700 | 93.3 | 7.50× | 689 to 716 | 84.3 to 106 | 1.00 | 1.00 |
| chart | threads | 2 | 1061 | not measured | — | 1057 to 1241 |  | 0.76 | — |
| chart | threads | 4 | 2368 | not measured | — | 1663 to 2389 |  | 0.85 | — |
| chart | threads | 8 | 490 | not measured | — | 473 to 599 |  | 0.09 | — |
| chart | threads | 10 | 699 | not measured | — | 585 to 767 |  | 0.10 | — |
| chart | processes | 1 | 721 | 106 | 6.80× | 699 to 729 | 104 to 107 | 1.00 | 1.00 |
| chart | processes | 2 | 1063 | 240 | 4.42× | 595 to 1118 | 197 to 244 | 0.74 | 1.13 |
| chart | processes | 4 | 2443 | 189 | 12.93× | 2394 to 2610 | 180 to 350 | 0.85 | 0.45 |
| chart | processes | 8 | 376 | 107 | 3.51× | 362 to 562 | 79.7 to 129 | 0.07 | 0.13 |
| chart | processes | 10 | 659 | 209 | 3.16× | 607 to 831 | 181 to 214 | 0.09 | 0.20 |
| day | threads | 1 | 1631 | 36.0 | 45.30× | 1601 to 1661 | 23.7 to 39.0 | 1.00 | 1.00 |
| day | threads | 2 | 2560 | not measured | — | 2446 to 2688 |  | 0.78 | — |
| day | threads | 4 | 4929 | not measured | — | 1912 to 5393 |  | 0.76 | — |
| day | threads | 8 | 8198 | not measured | — | 8138 to 8364 |  | 0.63 | — |
| day | threads | 10 | 2528 | not measured | — | 2064 to 4371 |  | 0.16 | — |
| day | processes | 1 | 1632 | 81.7 | 19.99× | 1585 to 1654 | 44.0 to 81.7 | 1.00 | 1.00 |
| day | processes | 2 | 3169 | 139 | 22.80× | 3094 to 3334 | 125 to 158 | 0.97 | 0.85 |
| day | processes | 4 | 6268 | 111 | 56.30× | 5974 to 6384 | 103 to 183 | 0.96 | 0.34 |
| day | processes | 8 | 5553 | 451 | 12.31× | 4535 to 6243 | 351 to 468 | 0.43 | 0.69 |
| day | processes | 10 | 1657 | 166 | 9.98× | 1465 to 1729 | 100 to 270 | 0.10 | 0.20 |
| full | threads | 1 | 22.3 | 6.33 | 3.53× | 17.0 to 28.0 | 6.00 to 8.33 | 1.00 | 1.00 |
| full | threads | 2 | 16.0 | not measured | — | 15.7 to 16.7 |  | 0.36 | — |
| full | threads | 4 | 16.3 | not measured | — | 13.7 to 21.3 |  | 0.18 | — |
| full | threads | 8 | 63.7 | not measured | — | 17.3 to 86.3 |  | 0.36 | — |
| full | threads | 10 | 275 | not measured | — | 179 to 283 |  | 1.23 | — |
| full | processes | 1 | 22.0 | 2.33 | 9.43× | 16.0 to 29.7 | 1.67 to 4.67 | 1.00 | 1.00 |
| full | processes | 2 | 35.0 | 2.67 | 13.12× | 14.7 to 40.7 | 2.67 to 4.67 | 0.80 | 0.57 |
| full | processes | 4 | 11.7 | 7.33 | 1.59× | 10.7 to 12.0 | 7.00 to 8.33 | 0.13 | 0.79 |
| full | processes | 8 | 123 | 15.0 | 8.22× | 115 to 134 | 13.3 to 18.7 | 0.70 | 0.80 |
| full | processes | 10 | 134 | 14.7 | 9.16× | 86.7 to 244 | 7.67 to 24.7 | 0.61 | 0.63 |

Not measured, and why:

- the baseline, `chart` on 2 threads: died of SIGSEGV: nothing on stderr
- the baseline, `chart` on 4 threads: died of SIGABRT: nothing on stderr
- the baseline, `chart` on 8 threads: died of SIGABRT: nothing on stderr
- the baseline, `chart` on 10 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 2 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 4 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 8 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 10 threads: died of SIGSEGV: nothing on stderr
- the baseline, `full` on 2 threads: Entity not found: RASHI/undefined
- the baseline, `full` on 4 threads: died of SIGABRT: nothing on stderr
- the baseline, `full` on 8 threads: died of SIGSEGV: nothing on stderr
- the baseline, `full` on 10 threads: died of SIGABRT: nothing on stderr

### The tail of a sustained run, per answer in µs

One thread, each side in a process of its own; a percentile is the median of the windows' with the fastest and slowest. Answers: `chart` 11269 through the SDK and 547 through the baseline; `day` 26767 through the SDK and 1497 through the baseline; `full` 854 through the SDK and 436 through the baseline.

| percentile | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart, p50 | 3055 | 25735 | 0.12× | 1358 to 4162 | 16090 to 95807 |
| chart, p95 | 14836 | 232534 | 0.06× | 1544 to 43040 | 95826 to 318381 |
| chart, p99 | 52711 | 325854 | 0.16× | 2152 to 108605 | 160471 to 582283 |
| day, p50 | 735 | 14496 | 0.05× | 583 to 1102 | 12019 to 104478 |
| day, p95 | 3304 | 34752 | 0.10× | 767 to 7401 | 13173 to 306162 |
| day, p99 | 7584 | 55726 | within spread | 1059 to 30092 | 16593 to 416731 |
| full, p50 | 28885 | 54925 | 0.53× | 23862 to 36244 | 52420 to 113187 |
| full, p95 | 63063 | 142321 | within spread | 43973 to 129095 | 67568 to 247014 |
| full, p99 | 183391 | 256797 | within spread | 72401 to 260535 | 74457 to 470058 |

### Cold start, in ms

| phase | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| open (from the process's start to the side being ready, the import and the engine or context included) | 34.0 | 764 | 0.04× | 31.1 to 44.8 | 699 to 1980 |
| first (the first chart) | 28.1 | 36.6 | within spread | 26.8 to 55.4 | 34.8 to 182 |
| process (the whole process, from spawn to exit) | 169 | 901 | 0.19× | 148 to 209 | 831 to 2151 |

### Memory over 10000 `chart` requests, after 200 to warm

| measure | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| peak-rss (the peak resident set over the process's life), MB | 185 | 299 | 0.62× | 185 to 194 | 298 to 299 |
| heap (the heap and external bytes, settled, after the workload), MB | 8.92 | 46.2 | 0.19× | 8.92 to 8.93 | 46.0 to 46.2 |
| heap-growth (their growth over the workload), KB | -1037 | 388 | -2.67× | -1038 to -1033 | 217 to 388 |

### Agreement and accuracy

The share of facts on which the two sides agree, exactly or within the field's tolerance, under the SDK's timed profiles (`timed`) and under the profiles that cast as the baseline casts (`same`), so that what still differs under `same` is method (§10); and each side against a reference neither side computed, with its evidence rank, where one exists (§11). Closer is the larger share of the reference's facts matched, the SDK's under its best convention.

| request | agreement, `timed` | agreement, `same` | reference | closer to it | dominant cause |
|---|---:|---:|---|---|---|
| chart | 54.5% of 12485 | 100.0% of 12485 | Swiss Ephemeris (2); PyJHora (3) | the baseline: SDK 99.6% `timed`, baseline 100.0%; the SDK: SDK 72.8% `timed`, baseline 46.5% | convention: the centre: `timed` casts the Moon from the Earth's centre and the baseline from the birthplace, up to an hour of arc apart, on a mean ayanamsha where the baseline's is nutated (12″), which moves the Vimshottari's starts; under `same` only the lagnas of 2350 and 2399 differ, the SDK's IAU 2006 sidereal time against Swiss Ephemeris's that far from J2000 |
| day | 99.2% of 860 | 96.5% of 860 | the committee's print (1) | neither: SDK 34.6% `timed`, baseline 34.6% | method: a day with no sunrise: both now take the refracted upper limb, so only the polar births differ, which the SDK reckons by its polar policy (civil midnight under `timed`, the conformance profile's nearest sunrise and sunset under `same`) and the baseline by its civil-day bounds. The committee's tithi ends are Surya Siddhanta's, hours from both |
| full | 64.3% of 262615 | 99.9% of 262615 | Swiss Ephemeris (2); PyJHora (3) | the baseline: SDK 99.8% `timed`, baseline 100.0%; the SDK: SDK 72.7% `timed`, baseline 46.4% | convention: the centre, and the classical Shadbala readings; under `same` the yogas' readings (as `yogas`), and c022's kaala bala, a birth the baseline places in the wrong Hindu day (crux C35) |
| vargas | 97.1% of 11550 | 100.0% of 11550 | agreement only | — | convention: the centre: a Moon a few arcminutes from a division's edge |
| yogas | 96.9% of 4611 | 97.1% of 4611 | agreement only | — | method: the rules' readings: the SDK applies Kemadruma's cancellations and reads the Nabhasa shapes, Gaja Kesari and Chamara by its texts' clauses, where the baseline holds them present |
| strength | 69.1% of 8635 | 98.6% of 8321 | Raman's Example 56 (2) | the SDK: SDK 100.0% `timed`, baseline 0.0% | convention: the readings: `timed` is the root's, Sripati's, which reproduces Raman's totals, and the baseline its own; under `same` the largest gaps are c022's, a birth the baseline places in the wrong Hindu day (crux C35) |
| dashas | 61.9% of 244860 | 100.0% of 244860 | PyJHora (3) | the SDK: SDK 100.0% `source`, baseline 46.5% | convention: the centre: the Moon's place sets every system's start; PyJHora casts from the Earth's centre as `timed` does |
| month | 99.1% of 24746 | 100.0% of 24752 | agreement only | — | convention: the sunrise, as `day`; under `same` every limb agrees |
| milan | 73.4% of 715 | 76.4% of 715 | *Kalaprakasika* XIII (1) | the SDK: SDK 100.0% `timed`, baseline 92.1% | method: the koota tables: Vashya, Gana, Yoni and Bhakoot score apart under `same`, and the baseline's Stree-deergha is Brihaspati's count beyond 9 where *Kalaprakasika* prints beyond 13 |
| varsha | 26.1% of 10908 | 24.4% of 10908 | Charak's worked year (2) | the SDK: SDK 100.0% `timed`, baseline 37.5% | the baseline: the year lord: on Charak's worked year the SDK names all seven offices and the baseline three, its year lord the Moon for the Sun; the sahams are other formulas, and the Mudda dasha follows the year lord |
| kp | 80.8% of 8690 | 96.9% of 8690 | Swiss Ephemeris (2) | the baseline: SDK 97.9% `timed`, baseline 98.2% | convention: the frame: `kp-default` casts from the Earth's centre on its own node, the baseline from the birthplace on the mean node; under `same` c043's cusps, where the baseline's Placidus falls back at 64.8° N and Swiss Ephemeris and the SDK do not, and the lagnas of 2350 and 2399, as `chart`; the day lord, which the SDK takes from sunrise to sunrise as KP Reader VI does and the baseline from midnight, so a birth before sunrise rules by the day before; and an intercepted sign's lord, which the baseline counts a significator of its house and the SDK reports and does not count (crux C154) |
| gochar | 65.9% of 1350 | 99.7% of 1350 | Swiss Ephemeris (2) | neither: SDK 100.0% `timed`, baseline 100.0% | convention: the ayanamsha's basis: `timed` takes the mean and the baseline the nutated, about 12″ apart; under `same`, cast from the Earth's centre as the baseline casts a transit, the transits agree to a hundredth of an arcsecond and only c051's transit bindus differ |
| sadeSati | 7.6% of 328 | 100.0% of 328 | agreement only | — | convention: the ayanamsha's basis, as `gochar`: 12″ of Saturn moves an edge by hours; under `same` every edge agrees within two seconds |
| batch | 65.4% of 2626150 | 99.9% of 2626150 | Swiss Ephemeris (2) | the baseline: SDK 99.8% `timed`, baseline 100.0% | convention: as `full`, at ten instants |

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
| varsha | event | s, 60.0 | 24.1% of 54; median 112.4, largest 334.9 | 16.7% of 54; median 139.1, largest 213.2 | 0 | worst c008 `event/return` |
| varsha | office | exact | 52.4% of 378 | 52.4% of 378 | 0 | c001 `office/yearLord` "VENUS" against "SUN"; c001 `office/triRashi` "VENUS" against "MARS"; c001 `office/dinaRatri` "MARS" against "SUN" |
| varsha | longitude | ″, 1.000 | 0.0% of 864; median 241223.1, largest 647394.3 | 0.0% of 864; median 244083.2, largest 647101.3 | 1350 | worst c037 `longitude/saham/YASHA` |
| varsha | dashaLord | exact | 48.1% of 4860 | 44.4% of 4860 | 540 | c003 `dashaLord/MUDDA/0` "VENUS" against "SUN"; c003 `dashaLord/MUDDA/0.0` "VENUS" against "SUN"; c003 `dashaLord/MUDDA/0.1` "SUN" against "MOON" |
| varsha | dashaStart | d, 1.000 | 6.2% of 4752; median 13.0, largest 56.4 | 6.1% of 4752; median 12.7, largest 53.0 | 540 | worst c038 `dashaStart/MUDDA/8.8` |
| kp | longitude | ″, 1.000 | 1.1% of 1155; median 14.5, largest 335680.9 | 96.2% of 1155; median 0.000, largest 335678.9 | 0 | worst c043 `longitude/cusp/2` |
| kp | kpLord | exact | 93.0% of 4620 | 98.4% of 4620 | 0 | c027 `kpLord/cusp/2/sign` "MARS" against "SATURN"; c027 `kpLord/cusp/2/star` "KETU" against "MOON"; c027 `kpLord/cusp/2/subSub` "SATURN" against "JUPITER" |
| kp | kpSignificator | exact | 93.3% of 2640 | 95.2% of 2640 | 0 | c005 `kpSignificator/6/inLordsStar` "MARS,JUPITER" against "MARS,JUPITER,RAHU"; c005 `kpSignificator/6/lord` "MOON" against "SUN,MOON"; c008 `kpSignificator/6/lord` "MERCURY" against "MERCURY,VENUS" |
| kp | kpRuler | exact | 91.6% of 275 | 92.0% of 275 | 0 | c001 `kpRuler/DAY_LORD` "VENUS" against "SATURN"; c003 `kpRuler/DAY_LORD` "MARS" against "MERCURY"; c005 `kpRuler/DAY_LORD` "SUN" against "MOON" |
| gochar | longitude | ″, 1.000 | 7.4% of 486; median 12.0, largest 18.3 | 100.0% of 486; median 0.000, largest 0.008 | 0 |  |
| gochar | house | exact | 100.0% of 486 | 100.0% of 486 | 0 |  |
| gochar | bindus | exact | 97.4% of 378 | 98.9% of 378 | 0 | c051 `bindus/transit/SUN` 1 against 2; c051 `bindus/transit/MOON` 4 against 5; c051 `bindus/transit/MARS` 3 against 4 |
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
