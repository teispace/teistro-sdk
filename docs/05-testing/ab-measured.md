# The SDK against the baseline engine

Status: `generated` by `cargo xtask ab` from `docs/05-testing/ab/*.json`, gated by `check-ab`. Do not edit. Each record is written by `cargo xtask measure-ab`, which runs `bindings/node/bench/ab.mjs` with the baseline engine's adapter (`03-design/ab-baseline.md`).

Both sides answer every birth of the golden corpus in one Node process, the SDK as `parashari-classical` over Teimeris (`kp-default` for `kp`), each request asking the SDK for at least what the baseline's answer holds. A birth on which the two Suns stand more than an arcminute apart is reported and kept out, and a record that kept any out is no claim. A request a side refuses is kept out of that request alone, and only at the corpus's range-edge births, where the data's years end and a day's spans run past them, or at its polar births for a rule that needs a sunrise on a day without one. So is a request a side answers with nothing it asked for, listed apart, since it was answered. A round times every birth on one side and then the other, the next round the other way round, and a figure is the median of at least 15 rounds, per birth in microseconds.

Then each side, in processes of its own, answers `chart`, `day` and `full` under load on 1, 2, 4 and every thread the machine offers, in worker threads and in processes, at least 3 trials a count; for 30 seconds on one thread, cut into 10 windows for the tail; from a fresh process 9 times for the cold start; and 10000 charts in each of 3 processes for the memory, the heap read after collecting until two reads agree.

**The claim** is that the SDK is not slower: the gate fails a request or a percentile where its median is the larger, and a throughput where it is the smaller, when the two fastest-to-slowest ranges are apart. Where they meet the ratio reads `within spread`. The cold start and the memory are compared and not claimed (§8). Only the baseline may go unmeasured, and only in worker threads, with the reason recorded.

**Agreement and accuracy:** over the same inputs each answer is read into facts and the two sides' compared field by field, under the SDK's timed profiles and under the ones that cast as the baseline casts, and each side is held to a reference neither side computed where one exists (§10, §11). The gate holds every agreement and reference share to the floor the record measured, and each request's named cause to what the record shows.

**Measurably slower through the SDK**, each held to a bound of the baseline's time:

- `muhurta`, at most 1.50×: the SDK judges every window of every day, cut wherever a clause changes (a limb, a kaala, a lagna), and keeps them all; the baseline bars a day at sunrise and scores the two windows it keeps (1.38× on 29 October 2026).
- `prashna`, at most 3.50×: a prashna is read off a founded chart, which finds the sunrises of its day (the chart's day, its birth timing and the lagna at sunrise) and asks the Shadbala, where the baseline casts the nine grahas and the lagna and reads nothing else: a bare founded chart took 430 µs of the request's 517 on 29 October 2026, the Shadbala 40.

No other request, and no throughput or percentile, is measurably slower through the SDK.

## `macos-aarch64`

macos aarch64, commit `0ea86c909`, Node v26.12.0, 10 threads offered; the sdk `teistro 0.0.0`, the baseline `baseline 0.6.173`.

The baseline's entry points: not counted, since the adapter exports no `surface` (`03-design/compat-shim.md` §6.4).

### Requests, per request in µs

| request | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart (a birth chart with every varga, its drishti, upagrahas, houses, dignities, arudhas and Vimshottari to depth 3) | 1444 | 7398 | 0.20× | 1389 to 1575 | 7283 to 7681 |
| day (the almanac day of the birth's date at its place, its limbs, kaalas, choghadiya and horas) | 1109 | 11733 | 0.09× | 1092 to 1151 | 11616 to 11933 |
| full (the whole natal chart: every section and varga, 22 dashas to depth 3, every shipped rule and the longevity) | 21636 | 50486 | 0.43× | 21372 to 21782 | 50097 to 51866 |
| vargas (the chart's foundation, every divisional chart with it) | 861 | 5575 | 0.15× | 843 to 910 | 5533 to 5707 |
| yogas (the yogas and doshas of a chart) | 3323 | 5727 | 0.58× | 3282 to 3462 | 5657 to 7006 |
| strength (the Shadbala, Bhava bala and Ashtakavarga) | 564 | 5707 | 0.10× | 545 to 612 | 5640 to 6043 |
| dashas (eighteen dasha systems to depth 3) | 15115 | 39410 | 0.38× | 14930 to 15580 | 39177 to 40903 |
| month (thirty almanac days from the birth's date) | 12360 | 348096 | 0.04× | 12245 to 12887 | 347116 to 351053 |
| milan (the birth matched with the next one of the corpus) | 688 | 11120 | 0.06× | 666 to 721 | 11030 to 11340 |
| varsha (the first annual chart, its yogas, sahams and Mudda dasha) | 2901 | 6298 | 0.46× | 2880 to 3097 | 6164 to 6867 |
| kp (the KP cusps, sub lords, significators and ruling planets) | 565 | 5630 | 0.10× | 543 to 598 | 5584 to 6079 |
| gochar (the grahas thirty years on over the chart, with the Ashtakavarga) | 567 | 5763 | 0.10× | 544 to 612 | 5693 to 6006 |
| sadeSati (Sade Sati and the dhaiyyas over thirty years) | 4282 | 22422 | 0.19× | 4250 to 4419 | 22213 to 23238 |
| batch (ten whole charts an hour apart, at one place) | 211792 | 522870 | 0.41× | 209905 to 213597 | 517937 to 527157 |
| muhurta (a marriage muhurta over 25 days at Kathmandu thirty years on, then against the native) | 456863 | 329861 | 1.39× | 453149 to 460642 | 327204 to 333228 |
| prashna (a prashna at the birth's instant and place: its answer and points, timing, topic, the querent's number's sign and the Moon's course) | 552 | 186 | 2.97× | 540 to 598 | 176 to 270 |
| rashifal (every sign's rashifal for a day, a week, a month and a year in 2000 to 2049, with the baseline's scores) | 32600 | 67076 | 0.49× | 32440 to 33477 | 66763 to 69057 |

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
- c048, `muhurta`, by the sdk: SUN at JD 2608597.2629888887 UT1: the provider answered OutOfRange

Answered with nothing, and kept out of that request's timing:

- c048, `varsha`, by the sdk: answered no pravesha (an empty `praveshas`)

### Throughput, in requests a second

All threads or processes together, each with its side of its own; the ratio is the SDK's throughput over the baseline's, and efficiency the share of linear throughput the count keeps.

| request | mode | at once | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest | SDK efficiency | baseline efficiency |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| chart | threads | 1 | 741 | 141 | 5.27× | 710 to 743 | 140 to 141 | 1.00 | 1.00 |
| chart | threads | 2 | 1429 | not measured | — | 1406 to 1436 |  | 0.96 | — |
| chart | threads | 4 | 2691 | not measured | — | 2652 to 2714 |  | 0.91 | — |
| chart | threads | 8 | 4358 | not measured | — | 4309 to 4362 |  | 0.73 | — |
| chart | threads | 10 | 4758 | not measured | — | 4558 to 4794 |  | 0.64 | — |
| chart | processes | 1 | 743 | 146 | 5.10× | 735 to 744 | 146 to 146 | 1.00 | 1.00 |
| chart | processes | 2 | 1445 | 287 | 5.03× | 1445 to 1467 | 287 to 287 | 0.97 | 0.99 |
| chart | processes | 4 | 2878 | 562 | 5.12× | 2823 to 2889 | 559 to 563 | 0.97 | 0.96 |
| chart | processes | 8 | 4756 | 916 | 5.19× | 4646 to 4791 | 912 to 916 | 0.80 | 0.79 |
| chart | processes | 10 | 5100 | 985 | 5.18× | 4937 to 5151 | 964 to 987 | 0.69 | 0.68 |
| day | threads | 1 | 1756 | 85.7 | 20.50× | 1752 to 1762 | 84.7 to 85.7 | 1.00 | 1.00 |
| day | threads | 2 | 3162 | not measured | — | 3103 to 3169 |  | 0.90 | — |
| day | threads | 4 | 5929 | not measured | — | 5919 to 5939 |  | 0.84 | — |
| day | threads | 8 | 9478 | not measured | — | 8782 to 9516 |  | 0.67 | — |
| day | threads | 10 | 10173 | not measured | — | 9956 to 10320 |  | 0.58 | — |
| day | processes | 1 | 1766 | 85.3 | 20.69× | 1755 to 1766 | 85.3 to 85.7 | 1.00 | 1.00 |
| day | processes | 2 | 3467 | 169 | 20.56× | 3451 to 3475 | 169 to 169 | 0.98 | 0.99 |
| day | processes | 4 | 6817 | 333 | 20.45× | 6817 to 6827 | 333 to 333 | 0.97 | 0.98 |
| day | processes | 8 | 11203 | 540 | 20.73× | 10951 to 11322 | 540 to 541 | 0.79 | 0.79 |
| day | processes | 10 | 12031 | 579 | 20.77× | 11913 to 12156 | 577 to 585 | 0.68 | 0.68 |
| full | threads | 1 | 48.0 | 18.7 | 2.57× | 47.7 to 48.3 | 18.7 to 19.0 | 1.00 | 1.00 |
| full | threads | 2 | 94.0 | not measured | — | 94.0 to 94.0 |  | 0.98 | — |
| full | threads | 4 | 183 | not measured | — | 183 to 183 |  | 0.95 | — |
| full | threads | 8 | 298 | not measured | — | 296 to 300 |  | 0.78 | — |
| full | threads | 10 | 324 | not measured | — | 321 to 327 |  | 0.68 | — |
| full | processes | 1 | 48.3 | 19.7 | 2.46× | 48.0 to 48.3 | 19.7 to 19.7 | 1.00 | 1.00 |
| full | processes | 2 | 95.3 | 39.0 | 2.44× | 94.0 to 95.3 | 38.7 to 39.0 | 0.99 | 0.99 |
| full | processes | 4 | 188 | 75.3 | 2.49× | 186 to 188 | 74.7 to 75.3 | 0.97 | 0.96 |
| full | processes | 8 | 298 | 115 | 2.60× | 297 to 301 | 115 to 116 | 0.77 | 0.73 |
| full | processes | 10 | 329 | 114 | 2.87× | 324 to 330 | 113 to 115 | 0.68 | 0.58 |

Not measured, and why:

- the baseline, `chart` on 2 threads: died of SIGSEGV: nothing on stderr
- the baseline, `chart` on 4 threads: died of SIGSEGV: nothing on stderr
- the baseline, `chart` on 8 threads: died of SIGSEGV: nothing on stderr
- the baseline, `chart` on 10 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 2 threads: died of SIGABRT: nothing on stderr
- the baseline, `day` on 4 threads: died of SIGSEGV: nothing on stderr
- the baseline, `day` on 8 threads: died of SIGABRT: nothing on stderr
- the baseline, `day` on 10 threads: died of SIGSEGV: nothing on stderr
- the baseline, `full` on 2 threads: Entity not found: RASHI/undefined
- the baseline, `full` on 4 threads: died of SIGTRAP: nothing on stderr
- the baseline, `full` on 8 threads: died of SIGSEGV: nothing on stderr
- the baseline, `full` on 10 threads: died of SIGSEGV: nothing on stderr

### The tail of a sustained run, per answer in µs

One thread, each side in a process of its own; a percentile is the median of the windows' with the fastest and slowest. Answers: `chart` 22434 through the SDK and 4338 through the baseline; `day` 52947 through the SDK and 2580 through the baseline; `full` 1458 through the SDK and 593 through the baseline.

| percentile | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart, p50 | 1321 | 6849 | 0.19× | 1287 to 1356 | 6788 to 6945 |
| chart, p95 | 1444 | 8266 | 0.17× | 1412 to 1494 | 8196 to 8644 |
| chart, p99 | 2041 | 8676 | 0.24× | 1914 to 2099 | 8592 to 12479 |
| day, p50 | 556 | 11654 | 0.05× | 555 to 558 | 11635 to 11717 |
| day, p95 | 655 | 12565 | 0.05× | 645 to 663 | 12520 to 12617 |
| day, p99 | 870 | 13002 | 0.07× | 868 to 873 | 12903 to 13187 |
| full, p50 | 20317 | 49837 | 0.41× | 20200 to 20375 | 47703 to 51000 |
| full, p95 | 22390 | 63894 | 0.35× | 22224 to 22556 | 61330 to 66380 |
| full, p99 | 23372 | 70181 | 0.33× | 22717 to 23827 | 68073 to 74479 |

### Cold start, in ms

| phase | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| open (from the process's start to the side being ready, the import and the engine or context included) | 30.7 | 659 | 0.05× | 29.9 to 31.7 | 655 to 671 |
| first (the first chart) | 40.3 | 36.2 | 1.11× | 39.1 to 41.2 | 35.6 to 37.0 |
| process (the whole process, from spawn to exit) | 157 | 789 | 0.20× | 156 to 159 | 782 to 798 |

### Memory over 10000 `chart` requests, after 200 to warm

| measure | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| peak-rss (the peak resident set over the process's life), MB | 203 | 301 | 0.67× | 193 to 206 | 300 to 319 |
| heap (the heap and external bytes, settled, after the workload), MB | 9.12 | 46.3 | 0.20× | 9.12 to 9.12 | 46.2 to 46.3 |
| heap-growth (their growth over the workload), KB | -1104 | 330 | -3.35× | -1109 to -1102 | 162 to 330 |

### Agreement and accuracy

The share of facts on which the two sides agree, exactly or within the field's tolerance, under the SDK's timed profiles (`timed`) and under the profiles that cast as the baseline casts (`same`), so that what still differs under `same` is method (§10); and each side against a reference neither side computed, with its evidence rank, where one exists (§11). Closer is the larger share of the reference's facts matched, the SDK's under its best convention.

| request | agreement, `timed` | agreement, `same` | reference | closer to it | dominant cause |
|---|---:|---:|---|---|---|
| chart | 54.5% of 12485 | 100.0% of 12485 | Swiss Ephemeris (2); PyJHora (3) | the baseline: SDK 99.6% `timed`, baseline 100.0%; the SDK: SDK 72.8% `timed`, baseline 46.5% | convention: the centre: `timed` casts the Moon from the Earth's centre and the baseline from the birthplace, up to an hour of arc apart, on a mean ayanamsha where the baseline's is nutated (12″), which moves the Vimshottari's starts; under `same` only the lagnas of 2350 and 2399 differ, the SDK's IAU 2006 sidereal time against Swiss Ephemeris's that far from J2000 |
| day | 99.2% of 860 | 96.5% of 860 | the committee's print (1) | neither: SDK 34.6% `timed`, baseline 34.6% | method: a day with no sunrise: both now take the refracted upper limb, so only the polar births differ, which the SDK reckons by its polar policy (civil midnight under `timed`, the conformance profile's nearest sunrise and sunset under `same`) and the baseline by its civil-day bounds. The committee's tithi ends are Surya Siddhanta's, hours from both |
| full | 64.7% of 270300 | 99.6% of 270294 | Swiss Ephemeris (2); PyJHora (3) | the baseline: SDK 99.8% `timed`, baseline 100.0%; the SDK: SDK 72.7% `timed`, baseline 46.4% | convention: the centre, and the classical Shadbala readings; under `same` the yogas' readings (as `yogas`), c022's kaala bala, a birth the baseline places in the wrong Hindu day (crux C35), and longevity's: BPHS ch. 43 v. 33 pairs Saturn with the Moon where the baseline pairs the lagna with it, which moves the class the pairs decide; v. 11 takes the Moon for a benefic waxing or waning in the visible half's reduction, the baseline a waning Moon for a malefic; vv. 20 to 22 treble Amsayu's years exalted or in its own sign and double them in its own navamsha or drekkana, where the baseline doubles its own sign, trebles a retrograde graha and halves a debilitated one; Nisargayu's listed years are taken whole, which the baseline reduces; and the cancellations of Balarishta. Under `timed` also the hora lagna, counted in `parashari-classical`'s proportional ghatis, and BPHS's own readings of the three spans (cruxes C104, C303) |
| vargas | 97.1% of 11550 | 100.0% of 11550 | agreement only | — | convention: the centre: a Moon a few arcminutes from a division's edge |
| yogas | 96.9% of 4611 | 97.1% of 4611 | agreement only | — | method: the rules' readings: the SDK applies Kemadruma's cancellations and reads the Nabhasa shapes, Gaja Kesari and Chamara by its texts' clauses, where the baseline holds them present |
| strength | 69.1% of 8635 | 98.6% of 8321 | Raman's Example 56 (2) | the SDK: SDK 100.0% `timed`, baseline 0.0% | convention: the readings: `timed` is the root's, Sripati's, which reproduces Raman's totals, and the baseline its own; under `same` the largest gaps are c022's, a birth the baseline places in the wrong Hindu day (crux C35) |
| dashas | 61.9% of 244860 | 100.0% of 244860 | PyJHora (3) | the SDK: SDK 100.0% `source`, baseline 46.5% | convention: the centre: the Moon's place sets every system's start; PyJHora casts from the Earth's centre as `timed` does |
| month | 99.1% of 24746 | 100.0% of 24752 | agreement only | — | convention: the sunrise, as `day`; under `same` every limb agrees |
| milan | 73.9% of 990 | 83.2% of 990 | *Kalaprakasika* XIII (1) | the SDK: SDK 100.0% `timed`, baseline 92.1% | method: the koota tables: under `same` Vashya (C260), Gana (C262), Yoni (C261) and Bhakoot (C263) score apart; the baseline gives a lifted nadi its 8 points where the SDK reports the lift beside the points, which it never moves (C283, C284: c009, c019, c048); and it scores the Tara 0 for two stars nine apart, each the other's 10th and 19th, where the verse rejects only the 3rd, 5th and 7th of each nine (C780: c003, c014, c043). `same` asks the baseline's Stree-deergha (C779), so every porutham agrees. The Kuja dosha, under `timed` the verse's (Mars from the lagna in five houses, *Manasagari*, cruxes C285, C286) where the baseline reads three references and six houses, and under `same` the baseline's own cancellations, three of nine, which lift a dosha the SDK keeps (ten charts) |
| varsha | 26.1% of 10908 | 24.4% of 10908 | Charak's worked year (2) | the SDK: SDK 100.0% `timed`, baseline 37.5% | the baseline: the year lord: on Charak's worked year the SDK names all seven offices and the baseline three, its year lord the Moon for the Sun; the sahams are other formulas, and the Mudda dasha follows the year lord |
| kp | 80.8% of 8690 | 97.1% of 8690 | Swiss Ephemeris (2) | the baseline: SDK 97.9% `timed`, baseline 98.2% | convention: the frame: `kp-default` casts from the Earth's centre on its own node, the baseline from the birthplace on the mean node; under `same` c043's cusps, where the baseline's Placidus falls back at 64.8° N and Swiss Ephemeris and the SDK do not, and the lagnas of 2350 and 2399, as `chart`; and an intercepted sign's lord, which the baseline counts a significator of its house and the SDK reports and does not count (crux C154). Under `timed` also the day lord, which `kp-default` takes from sunrise to sunrise as KP Reader VI does and the baseline from the civil day, so a birth before sunrise rules by the day before (c001); `same` asks the civil day (`kp.day_lord_day = CIVIL`, crux C151) |
| gochar | 81.7% of 2779 | 98.8% of 2779 | Swiss Ephemeris (2) | neither: SDK 100.0% `timed`, baseline 100.0% | convention: the ayanamsha's basis: `timed` takes the mean and the baseline the nutated, about 12″ apart; under `same`, a birth cast from its place and its transits from the Earth's centre on one context (`gochar.transit_centre = GEOCENTRIC`, crux C778), the transits agree to a hundredth of an arcsecond and every bindu agrees; what differs is the vedha of Phaladeepika ch. 26, all 33 misses: the nodes take the Sun's pairs (v. 2, crux C136) where the baseline gives them Mars's for the 3rd and the 6th, and never obstruct each other (C140), 20 of them; Venus's 11th and 12th pair with the 3rd and the 6th (v. 8), 9; and the Sun and Saturn, the Moon and Mercury do not obstruct each other (vv. 3 to 6), where the baseline exempts the Sun and the Moon, 4 |
| sadeSati | 7.6% of 328 | 100.0% of 328 | agreement only | — | convention: the ayanamsha's basis, as `gochar`: 12″ of Saturn moves an edge by hours; under `same` every edge agrees within two seconds |
| batch | 65.7% of 2703000 | 99.6% of 2702940 | Swiss Ephemeris (2) | the baseline: SDK 99.8% `timed`, baseline 100.0% | convention: as `full`, at ten instants |
| muhurta | 44.5% of 1155 | 46.7% of 1157 | agreement only | — | method: 20 of 54 rankings agree rank for rank under `same`, 19 of them with no window on either side; no miss is the SDK's. The day: the baseline admits or bars a whole day by the star and the solar month at sunrise and samples its seasons once a day at 0h UT, barring the day whose local midnight they hold; the SDK judges each window by its own (crux C783), so over 47 searches 224 days the baseline bars by the star at sunrise keep the SDK's windows after the star changes, 14 it keeps lose every window to a star or a kaala, 32 differ at a season's edge (Kharmas 28) and 2 by the solar month. Chaturmas by the Sun's sign (C166), 20 days; Guru's and Shukra's asta seen heliacally at Kathmandu (C44, C164), 8. The day's score: the baseline's weekday is sunrise's UT date, the day before when the Sun rises before 05:45 at Kathmandu (C782), moving the weekday's 15 or 10 and Rahu kaala's 30; its five special-yoga tables are its own (C784). A window's score agrees on every day whose score does, its sky geocentric with the true node (C781) |
| prashna | 96.6% of 495 | 100.0% of 477 | agreement only | — | convention: the frame: the baseline casts a prashna from the Earth's centre on the true node, where its birth chart is topocentric on the mean node (crux C800), so under `timed` the Moon's applying aspect may meet the other node and a factor its score counts moves; under `same`, asked on that frame, every reading agrees |
| rashifal | 84.4% of 69732 | 88.4% of 69740 | agreement only | — | convention: the instant and the events: `timed` reads a period at sunrise (C358) and reports every graha's ingresses and stations but the Moon's (C360), the baseline at 06:00 and fewer; under `same`, asked the baseline's clock and event grahas, the transits agree to the arcsecond and what differs is the vedha, as `gochar` (C136, C140, C801), the baseline's events (it finds no node's ingress, nor an event in a scan's last partial step, C360), and the scores those move: where a sign's readings agree its scores agree within a point on 99.5% (C802) |

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
| full | drishtiGrade | exact | 99.7% of 3816 | 100.0% of 3816 | 0 |  |
| full | rashiDrishti | exact | 99.8% of 477 | 100.0% of 477 | 0 |  |
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
| milan | porutham | exact | 89.1% of 220 | 100.0% of 220 | 0 |  |
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
| gochar | gochar | exact | 96.4% of 1051 | 96.9% of 1051 | 0 | c002 `gochar/KETU/vedha` false against true; c004 `gochar/KETU/vedha` true against false; c005 `gochar/MERCURY/vedha` false against true |
| gochar | bindus | exact | 97.2% of 756 | 100.0% of 756 | 0 |  |
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
| batch | drishtiGrade | exact | 99.7% of 38160 | 100.0% of 38160 | 0 |  |
| batch | rashiDrishti | exact | 99.8% of 4770 | 100.0% of 4770 | 0 |  |
| batch | jaiminiPair | exact | 70.3% of 1590 | 87.2% of 1530 | 1100 | c001 `jaiminiPair/decided@0` "medium" against "long"; c001 `jaiminiPair/decided@1` "long" against "medium"; c001 `jaiminiPair/decided@2` "medium" against "short" |
| batch | ayurdaya | y, 0.010 | 33.8% of 25440; median 0.748, largest 81.0 | 72.1% of 25440; median 0.000, largest 74.8 | 0 | worst c034 `ayurdaya/nisargayu/total@9` |
| batch | maraka | exact | 99.8% of 1060 | 100.0% of 1060 | 0 |  |
| batch | balarishta | exact | 97.0% of 1060 | 97.4% of 1060 | 0 | c007 `balarishta/standing@2` true against false; c008 `balarishta/present@7` false against true; c008 `balarishta/present@8` false against true |
| muhurta | muhurta | exact | 44.5% of 1155 | 46.7% of 1157 | 1530 | c001 `muhurta/general/ranking` "2020-04-15#1=63 2020-05-08#13=55 2020-05-08#2b=46 2020-05-01#13=46 2020-05-03#13=34 2020-05-05#4=33 2020-05-04#0=30 2020-05-04#7=30 2020-05-07#0=30 2020-04-26#9=30 2020-04-27#0=30 2020-04-20#14=30 2020-05-05#3=27 2020-04-25#10=24 2020-04-22#1=24 2020-04-27#7=21 2020-05-02#6=17 2020-04-14#11=16 2020-04-15#3=13 2020-04-26#3=11 2020-05-06#10=10 2020-04-21#4=10 2020-05-06#9=6 2020-05-02#11=0 2020-04-25#11=0 2020-04-14#12=0 2020-04-20#7=0 2020-04-21#12=0 2020-04-22#3=0 2020-05-03#15=0" against "2020-05-05#12=98 2020-04-15#1=78 2020-04-15#10=55 2020-05-05#4=48 2020-05-04#0=30 2020-05-04#7=30 2020-04-27#0=30 2020-04-26#9=29 2020-04-22#1=24 2020-04-22#10=18 2020-05-02#6=12 2020-04-21#4=10 2020-04-27#7=7 2020-05-07#8=5 2020-04-26#3=1 2020-05-02#11=0 2020-04-21#12=0 2020-05-07#15=0"; c001 `muhurta/general/count` 30 against 18; c001 `muhurta/general/window/2020-04-15#1` 63 against 78 |
| prashna | prashna | exact | 96.6% of 495 | 100.0% of 477 | 0 |  |
| rashifal | longitude | ″, 1.000 | 1.8% of 1980; median 37.0, largest 2031.4 | 100.0% of 1980; median 0.000, largest 0.000 | 0 |  |
| rashifal | rashifal | exact | 94.3% of 43992 | 95.6% of 44000 | 0 | c001 `rashifal/DAILY/3/RAHU/vedha` false against true; c001 `rashifal/DAILY/4/RAHU/vedha` false against true; c001 `rashifal/DAILY/5/KETU/vedha` false against true |
| rashifal | rashifalScore | points, 1.000 | 72.9% of 23760; median 0.000, largest 66.0 | 74.1% of 23760; median 0.000, largest 44.0 | 0 | worst c028 `rashifalScore/MONTHLY/2/area/education` |

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
| *Kalaprakasika* XIII (1) | milan | porutham | the SDK | `same` | 92.1% of 165 | c004 `porutham/STREE_DEERGHA` true against false; c009 `porutham/STREE_DEERGHA` true against false; c016 `porutham/STREE_DEERGHA` true against false |
| *Kalaprakasika* XIII (1) | milan | porutham | the baseline | `timed` | 92.1% of 165 | c004 `porutham/STREE_DEERGHA` true against false; c009 `porutham/STREE_DEERGHA` true against false; c016 `porutham/STREE_DEERGHA` true against false |

The SDK left unanswered under a convention:

- c047, `strength`, under `same`: the Sun at JD 2378132.0006944444: the provider answered OutOfRange
- c048, `strength`, under `same`: the Sun at JD 2597685.2871633684: the provider answered OutOfRange
- c047, `prashna`, under `same`: the Sun at JD 2378132.0006944444: the provider answered OutOfRange
- c048, `prashna`, under `same`: the Sun at JD 2597685.2871633684: the provider answered OutOfRange
