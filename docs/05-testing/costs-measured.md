# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (8 062 940 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `53eab6372971`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 258 214 | 3 387 816 |
| `chart.found` | 21.5× | 5.98× | 392 608 932 | 20 250 198 |
| `chart.disagreements` | 3.14× | 10.6× | 57 300 769 | 35 742 922 |
| `almanac.days` | 179× | 366× | 3 266 157 823 | 1 238 260 846 |
| `matching.naam` | 0.0180× | 0.0938× | 327 995 | 317 915 |
| `matching.band` | 1.62× | 0.0229× | 29 657 018 | 77 657 |
| `naming.syllables` | 0.0236× | 0.119× | 430 702 | 404 209 |
| `naming.read` | 0.0243× | 0.0882× | 443 678 | 298 970 |
| `naming.check` | 0.0165× | 0.0806× | 301 361 | 272 889 |
| `naming.of` | 0.00390× | 0.0146× | 71 243 | 49 531 |
| `naming.suggest` | 0.0205× | 0.0990× | 374 333 | 335 264 |
| `time.resolve` | 0.00664× | 0.0299× | 121 324 | 101 385 |
| `geo.resolve` | 0.0608× | 0.0191× | 1 109 902 | 64 742 |
| `geo.search` | 0.0629× | 0.0275× | 1 148 888 | 93 044 |
| `geo.reverse` | 0.0712× | 0.0675× | 1 299 242 | 228 700 |
| `geo.names` | 0.0640× | 0.0354× | 1 168 344 | 120 007 |
| `geo.list` | 0.0642× | 0.0363× | 1 173 045 | 123 142 |
| `time.civil` | 0.00619× | 0.0217× | 113 005 | 73 428 |
| `time.convert` | 0.00166× | 0.00829× | 30 263 | 28 072 |
| `calendar.convert` | 0.00477× | 0.0140× | 87 012 | 47 466 |
| `calendar.add` | 0.00545× | 0.0175× | 99 588 | 59 439 |
| `calendar.between` | 0.00528× | 0.0162× | 96 407 | 54 826 |
| `numerology.profile` | 0.0155× | 0.0799× | 282 864 | 270 842 |
| `almanac.navatara` | 0.0148× | 0.0786× | 271 096 | 266 126 |
| `almanac.aayadi` | 0.0115× | 0.0599× | 209 071 | 203 018 |
| `intl.inWords` | 1.71× | 0.0420× | 31 288 240 | 142 175 |
| `intl.time` | 1.62× | 0.0314× | 29 663 687 | 106 316 |
| `intl.date` | 1.59× | 0.0490× | 29 039 482 | 166 058 |
| `intl.lexicon` | 7.64× | 0.548× | 139 516 510 | 1 857 598 |
| `intl.lookup` | 18.9× | 0.215× | 344 282 288 | 727 580 |
| `intl.dates` | 4.39× | 0.0449× | 80 157 703 | 152 240 |
| `verify.claims` | 92.5× | 84.3× | 1 688 751 109 | 285 725 914 |
| `verify.issue` | 0.189× | 1.00× | 3 454 951 | 3 402 598 |
| `verify.receipt` | 0.274× | 1.41× | 4 995 256 | 4 791 089 |
| `almanac.digest` | 25.1× | 31.4× | 457 421 837 | 106 471 208 |
| `almanac.festival_calendar` | 35.9× | 53.1× | 654 966 728 | 179 759 814 |
| `almanac.changes` | 4.84× | 3.09× | 88 382 537 | 10 455 153 |
| `almanac.cells` | 15.7× | 15.8× | 287 337 292 | 53 371 238 |
| `almanac.anniversaries` | 1587× | 7880× | 28 978 057 709 | 26 694 954 403 |
| `almanac.next` | 168× | 372× | 3 061 124 734 | 1 261 825 906 |
| `almanac.sentences` | 12.6× | 5.78× | 230 336 521 | 19 581 741 |
| `almanac.publishedReasons` | 33.0× | 51.4× | 602 461 878 | 174 013 380 |
| `almanac.pakshi` | 10.6× | 9.30× | 194 042 338 | 31 493 158 |
| `almanac.sankalpa` | 74.6× | 86.1× | 1 362 079 313 | 291 828 584 |
| `almanac.sankalpaWords` | 75.1× | 83.7× | 1 372 070 516 | 283 631 774 |
| `almanac.ashauch` | 27.8× | 34.6× | 508 052 275 | 117 316 333 |
| `almanac.household` | 36.1× | 73.1× | 658 784 441 | 247 687 028 |
| `almanac.ashauchKin` | 0.00556× | 0.0268× | 101 521 | 90 630 |
| `almanac.ashauchRite` | 0.00353× | 0.0156× | 64 515 | 52 815 |
| `almanac.deathTime` | 91.8× | 180× | 1 675 442 489 | 608 914 524 |
| `almanac.festivalPack` | 0.122× | 0.647× | 2 219 612 | 2 191 010 |
| `almanac.firstYear` | 156× | 318× | 2 856 232 123 | 1 076 438 256 |
| `chart.rashifal` | 9.77× | 8.83× | 178 355 446 | 29 916 807 |
| `research.study` | 3.03× | 2.92× | 55 294 578 | 9 905 376 |
| `chart.mundane` | 2.70× | 0.903× | 49 277 609 | 3 058 401 |
| `chart.relocated` | 1.05× | 0.857× | 19 208 561 | 2 903 235 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 395 037 | 4 092 739 |
| `almanac.skyView` | 0.643× | 1.23× | 11 745 218 | 4 176 757 |
| `almanac.elect` | 280× | 462× | 5 113 837 688 | 1 565 576 154 |
| `almanac.observance` | 204× | 603× | 3 725 064 688 | 2 042 109 322 |
| `almanac.people` | 12.9× | 13.7× | 235 061 863 | 46 460 270 |
| `almanac.skyPose` | 0.737× | 1.74× | 13 449 417 | 5 893 496 |
| `calendar.in_era` | 0.00505× | 0.0155× | 92 156 | 52 476 |
| `settings.explain` | 0.832× | 0.404× | 15 185 517 | 1 369 022 |
| `settings.diff` | 0.128× | 0.687× | 2 335 537 | 2 326 876 |
| `almanac.limbAngles` | 0.476× | 0.323× | 8 686 996 | 1 094 531 |
| `almanac.blackouts` | 135× | 143× | 2 458 034 377 | 485 674 116 |
| `almanac.occurrences` | 410× | 1563× | 7 483 978 169 | 5 294 445 002 |
| `intl.render` | 0.776× | 0.0528× | 14 159 606 | 178 963 |
