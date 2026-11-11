# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (5 724 751 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `55f0a0cf59b4`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 698 207 | 4 300 240 |
| `chart.found` | 21.0× | 5.32× | 391 819 665 | 22 872 683 |
| `chart.disagreements` | 3.12× | 8.71× | 58 355 626 | 37 475 881 |
| `almanac.days` | 175× | 290× | 3 268 658 265 | 1 246 487 281 |
| `matching.naam` | 0.0179× | 0.0758× | 335 515 | 326 164 |
| `matching.band` | 0.984× | 0.0182× | 18 402 377 | 78 194 |
| `naming.syllables` | 0.0233× | 0.0950× | 435 071 | 408 522 |
| `naming.read` | 0.0241× | 0.0710× | 449 782 | 305 138 |
| `naming.check` | 0.0164× | 0.0648× | 305 734 | 278 797 |
| `naming.of` | 0.00383× | 0.0117× | 71 587 | 50 191 |
| `naming.suggest` | 0.0202× | 0.0787× | 378 365 | 338 412 |
| `time.resolve` | 0.00665× | 0.0251× | 124 277 | 107 786 |
| `geo.resolve` | 0.0218× | 0.0149× | 406 860 | 63 962 |
| `time.civil` | 0.00617× | 0.0180× | 115 303 | 77 428 |
| `time.convert` | 0.00181× | 0.00716× | 33 862 | 30 774 |
| `calendar.convert` | 0.00466× | 0.0116× | 87 203 | 50 092 |
| `calendar.add` | 0.00554× | 0.0159× | 103 571 | 68 309 |
| `calendar.between` | 0.00543× | 0.0146× | 101 602 | 62 814 |
| `numerology.profile` | 0.0153× | 0.0637× | 285 169 | 273 849 |
| `almanac.navatara` | 0.0148× | 0.0628× | 276 206 | 270 059 |
| `almanac.aayadi` | 0.0112× | 0.0478× | 210 108 | 205 736 |
| `intl.inWords` | 1.07× | 0.0330× | 20 027 095 | 141 814 |
| `intl.time` | 0.984× | 0.0248× | 18 406 937 | 106 831 |
| `intl.date` | 0.869× | 0.0389× | 16 250 131 | 167 101 |
| `intl.lexicon` | 4.42× | 0.364× | 82 569 282 | 1 563 713 |
| `intl.lookup` | 9.31× | 0.136× | 174 163 788 | 586 481 |
| `intl.dates` | 2.57× | 0.0357× | 48 017 052 | 153 630 |
| `verify.claims` | 81.0× | 64.9× | 1 513 863 261 | 278 975 195 |
| `verify.issue` | 0.169× | 0.723× | 3 158 994 | 3 107 145 |
| `verify.receipt` | 0.343× | 1.44× | 6 409 158 | 6 171 733 |
| `almanac.digest` | 14.7× | 21.6× | 274 443 762 | 92 974 648 |
| `almanac.festival_calendar` | 25.1× | 38.0× | 469 072 483 | 163 609 165 |
| `almanac.changes` | 4.72× | 3.22× | 88 347 080 | 13 829 587 |
| `almanac.cells` | 15.0× | 14.5× | 279 717 416 | 62 560 672 |
| `almanac.anniversaries` | 1545× | 6187× | 28 890 880 166 | 26 606 220 368 |
| `almanac.next` | 149× | 259× | 2 791 777 308 | 1 113 482 219 |
| `almanac.sentences` | 11.3× | 5.86× | 210 840 934 | 25 208 400 |
| `almanac.publishedReasons` | 23.4× | 37.5× | 438 224 194 | 161 466 547 |
| `almanac.pakshi` | 10.3× | 9.43× | 192 961 786 | 40 569 538 |
| `almanac.sankalpa` | 72.6× | 66.5× | 1 357 516 327 | 285 802 605 |
| `almanac.sankalpaWords` | 72.5× | 64.5× | 1 356 187 281 | 277 349 822 |
| `almanac.ashauch` | 26.7× | 28.8× | 499 181 197 | 123 776 813 |
| `almanac.household` | 35.1× | 60.9× | 655 983 421 | 261 862 363 |
| `almanac.ashauchKin` | 0.00539× | 0.0212× | 100 700 | 90 961 |
| `almanac.ashauchRite` | 0.00336× | 0.0123× | 62 876 | 52 702 |
| `almanac.deathTime` | 90.1× | 147× | 1 683 959 288 | 632 226 970 |
| `almanac.festivalPack` | 0.0995× | 0.433× | 1 859 611 | 1 861 114 |
| `almanac.firstYear` | 150× | 243× | 2 807 212 595 | 1 044 251 815 |
| `chart.rashifal` | 9.41× | 8.49× | 175 893 152 | 36 504 532 |
| `research.study` | 3.07× | 3.11× | 57 368 989 | 13 391 801 |
| `chart.mundane` | 2.65× | 0.945× | 49 486 456 | 4 062 582 |
| `chart.relocated` | 1.05× | 0.882× | 19 707 536 | 3 793 831 |
| `chart.astrocartography` | 1.12× | 1.15× | 20 849 727 | 4 939 645 |
| `almanac.skyView` | 0.631× | 0.994× | 11 804 123 | 4 273 807 |
| `almanac.elect` | 286× | 421× | 5 347 931 201 | 1 810 531 909 |
| `almanac.observance` | 182× | 428× | 3 403 809 419 | 1 840 789 664 |
| `almanac.people` | 12.5× | 12.5× | 233 045 328 | 53 545 267 |
