# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (6 106 298 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `d88c303f6c62`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 218 397 | 3 348 864 |
| `chart.found` | 21.5× | 6.02× | 392 520 998 | 20 153 489 |
| `chart.disagreements` | 3.14× | 10.7× | 57 212 534 | 35 704 094 |
| `almanac.days` | 177× | 359× | 3 227 334 440 | 1 202 185 088 |
| `matching.naam` | 0.0178× | 0.0947× | 324 030 | 317 005 |
| `matching.band` | 1.13× | 0.0227× | 20 627 616 | 75 996 |
| `naming.syllables` | 0.0236× | 0.120× | 430 734 | 401 057 |
| `naming.read` | 0.0243× | 0.0891× | 442 491 | 298 474 |
| `naming.check` | 0.0164× | 0.0809× | 299 557 | 271 085 |
| `naming.of` | 0.00382× | 0.0147× | 69 574 | 49 062 |
| `naming.suggest` | 0.0204× | 0.101× | 372 350 | 338 652 |
| `time.resolve` | 0.00656× | 0.0307× | 119 426 | 102 878 |
| `geo.resolve` | 0.0223× | 0.0186× | 406 932 | 62 250 |
| `time.civil` | 0.00610× | 0.0217× | 111 043 | 72 811 |
| `time.convert` | 0.00170× | 0.00855× | 30 881 | 28 628 |
| `calendar.convert` | 0.00459× | 0.0142× | 83 634 | 47 616 |
| `calendar.add` | 0.00542× | 0.0177× | 98 700 | 59 338 |
| `calendar.between` | 0.00527× | 0.0166× | 96 091 | 55 491 |
| `numerology.profile` | 0.0155× | 0.0800× | 281 482 | 267 829 |
| `almanac.navatara` | 0.0147× | 0.0790× | 268 590 | 264 456 |
| `almanac.aayadi` | 0.0114× | 0.0604× | 207 993 | 202 238 |
| `intl.inWords` | 1.22× | 0.0417× | 22 246 312 | 139 646 |
| `intl.time` | 1.13× | 0.0311× | 20 628 469 | 104 288 |
| `intl.date` | 1.01× | 0.0489× | 18 439 050 | 163 652 |
| `intl.lexicon` | 5.17× | 0.481× | 94 220 124 | 1 609 543 |
| `intl.lookup` | 11.4× | 0.181× | 207 257 888 | 606 197 |
| `intl.dates` | 2.94× | 0.0456× | 53 525 932 | 152 633 |
| `verify.claims` | 84.6× | 80.0× | 1 540 826 544 | 267 904 106 |
| `verify.issue` | 0.181× | 0.970× | 3 302 215 | 3 248 426 |
| `verify.receipt` | 0.281× | 1.46× | 5 113 342 | 4 894 795 |
| `almanac.digest` | 14.3× | 21.3× | 261 054 558 | 71 208 551 |
| `almanac.festival_calendar` | 25.1× | 42.4× | 456 812 449 | 141 896 661 |
| `almanac.changes` | 4.86× | 3.15× | 88 508 038 | 10 549 852 |
| `almanac.cells` | 15.1× | 15.0× | 274 581 945 | 50 250 913 |
| `almanac.anniversaries` | 1586× | 7944× | 28 888 194 639 | 26 604 146 487 |
| `almanac.next` | 153× | 331× | 2 788 006 116 | 1 106 834 509 |
| `almanac.sentences` | 11.6× | 5.51× | 210 585 085 | 18 468 289 |
| `almanac.publishedReasons` | 23.6× | 44.0× | 430 666 421 | 147 516 461 |
| `almanac.pakshi` | 10.3× | 9.08× | 188 309 573 | 30 411 076 |
| `almanac.sankalpa` | 74.2× | 82.0× | 1 351 364 158 | 274 460 813 |
| `almanac.sankalpaWords` | 74.2× | 79.5× | 1 352 130 581 | 266 220 566 |
| `almanac.ashauch` | 27.2× | 34.4× | 496 251 549 | 115 182 801 |
| `almanac.household` | 35.4× | 72.6× | 644 773 065 | 243 218 265 |
| `almanac.ashauchKin` | 0.00540× | 0.0272× | 98 294 | 90 982 |
| `almanac.ashauchRite` | 0.00346× | 0.0156× | 63 059 | 52 252 |
| `almanac.deathTime` | 90.9× | 176× | 1 655 214 108 | 590 582 570 |
| `almanac.festivalPack` | 0.106× | 0.569× | 1 923 045 | 1 905 736 |
| `almanac.firstYear` | 156× | 320× | 2 836 579 198 | 1 070 724 346 |
| `chart.rashifal` | 9.46× | 8.66× | 172 366 449 | 29 013 495 |
| `research.study` | 3.03× | 2.94× | 55 207 092 | 9 835 756 |
| `chart.mundane` | 2.70× | 0.900× | 49 234 300 | 3 014 386 |
| `chart.relocated` | 1.05× | 0.857× | 19 168 902 | 2 869 431 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 358 519 | 4 057 817 |
| `almanac.skyView` | 0.643× | 1.24× | 11 721 113 | 4 149 705 |
| `almanac.elect` | 281× | 467× | 5 115 836 026 | 1 564 835 674 |
| `almanac.observance` | 192× | 577× | 3 499 813 743 | 1 933 367 598 |
| `almanac.people` | 12.6× | 13.7× | 229 495 606 | 45 933 318 |
| `almanac.skyPose` | 0.737× | 1.75× | 13 420 765 | 5 858 604 |
