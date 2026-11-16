# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (6 130 173 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `7395cdf24859`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 230 099 | 3 357 316 |
| `chart.found` | 21.5× | 6.02× | 392 569 976 | 20 201 770 |
| `chart.disagreements` | 3.14× | 10.6× | 57 212 144 | 35 703 227 |
| `almanac.days` | 177× | 359× | 3 229 919 098 | 1 205 514 419 |
| `matching.naam` | 0.0179× | 0.0950× | 325 673 | 318 899 |
| `matching.band` | 1.13× | 0.0227× | 20 649 017 | 76 090 |
| `naming.syllables` | 0.0237× | 0.121× | 432 626 | 405 593 |
| `naming.read` | 0.0244× | 0.0889× | 444 294 | 298 479 |
| `naming.check` | 0.0166× | 0.0816× | 302 464 | 274 059 |
| `naming.of` | 0.00395× | 0.0150× | 72 008 | 50 294 |
| `naming.suggest` | 0.0204× | 0.100× | 371 086 | 336 754 |
| `time.resolve` | 0.00652× | 0.0309× | 118 929 | 103 809 |
| `geo.resolve` | 0.0216× | 0.0191× | 394 340 | 64 045 |
| `time.civil` | 0.00623× | 0.0218× | 113 542 | 73 102 |
| `time.convert` | 0.00165× | 0.00850× | 30 095 | 28 537 |
| `calendar.convert` | 0.00454× | 0.0141× | 82 762 | 47 245 |
| `calendar.add` | 0.00552× | 0.0179× | 100 584 | 60 192 |
| `calendar.between` | 0.00518× | 0.0163× | 94 509 | 54 572 |
| `numerology.profile` | 0.0154× | 0.0812× | 281 196 | 272 567 |
| `almanac.navatara` | 0.0149× | 0.0793× | 270 863 | 266 273 |
| `almanac.aayadi` | 0.0115× | 0.0606× | 208 928 | 203 366 |
| `intl.inWords` | 1.22× | 0.0419× | 22 266 873 | 140 552 |
| `intl.time` | 1.13× | 0.0309× | 20 650 765 | 103 836 |
| `intl.date` | 1.01× | 0.0488× | 18 453 604 | 163 933 |
| `intl.lexicon` | 5.18× | 0.481× | 94 377 530 | 1 615 459 |
| `intl.lookup` | 11.4× | 0.181× | 207 432 865 | 606 851 |
| `intl.dates` | 2.94× | 0.0455× | 53 537 458 | 152 659 |
| `verify.claims` | 85.1× | 85.0× | 1 551 574 571 | 285 250 053 |
| `verify.issue` | 0.182× | 0.976× | 3 321 306 | 3 275 194 |
| `verify.receipt` | 0.272× | 1.41× | 4 961 161 | 4 741 617 |
| `almanac.digest` | 14.8× | 22.2× | 269 430 993 | 74 482 172 |
| `almanac.festival_calendar` | 25.6× | 43.5× | 465 879 502 | 146 179 353 |
| `almanac.changes` | 4.86× | 3.14× | 88 513 679 | 10 558 421 |
| `almanac.cells` | 15.8× | 15.8× | 287 147 793 | 53 136 984 |
| `almanac.anniversaries` | 1588× | 7943× | 28 954 172 278 | 26 667 793 516 |
| `almanac.next` | 154× | 334× | 2 811 079 133 | 1 119 799 552 |
| `almanac.sentences` | 12.1× | 5.80× | 221 281 209 | 19 479 918 |
| `almanac.publishedReasons` | 24.0× | 44.7× | 438 193 193 | 150 172 132 |
| `almanac.pakshi` | 10.6× | 9.36× | 194 020 349 | 31 417 825 |
| `almanac.sankalpa` | 74.7× | 86.9× | 1 362 080 601 | 291 813 604 |
| `almanac.sankalpaWords` | 74.8× | 84.5× | 1 362 740 540 | 283 543 634 |
| `almanac.ashauch` | 27.9× | 34.8× | 507 733 039 | 116 967 131 |
| `almanac.household` | 36.1× | 73.5× | 658 069 908 | 246 766 741 |
| `almanac.ashauchKin` | 0.00551× | 0.0268× | 100 454 | 90 025 |
| `almanac.ashauchRite` | 0.00347× | 0.0158× | 63 207 | 53 073 |
| `almanac.deathTime` | 91.9× | 181× | 1 674 728 062 | 608 451 870 |
| `almanac.festivalPack` | 0.106× | 0.566× | 1 924 121 | 1 899 047 |
| `almanac.firstYear` | 157× | 321× | 2 858 983 613 | 1 077 682 555 |
| `chart.rashifal` | 9.76× | 8.82× | 178 009 642 | 29 620 108 |
| `research.study` | 3.03× | 2.93× | 55 219 985 | 9 822 235 |
| `chart.mundane` | 2.70× | 0.899× | 49 244 586 | 3 018 316 |
| `chart.relocated` | 1.05× | 0.856× | 19 183 840 | 2 872 414 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 370 726 | 4 063 429 |
| `almanac.skyView` | 0.643× | 1.24× | 11 729 556 | 4 161 353 |
| `almanac.elect` | 281× | 471× | 5 129 504 585 | 1 582 059 434 |
| `almanac.observance` | 193× | 580× | 3 523 245 941 | 1 947 237 058 |
| `almanac.people` | 12.9× | 13.9× | 235 039 275 | 46 501 288 |
| `almanac.skyPose` | 0.737× | 1.75× | 13 430 340 | 5 862 144 |
| `calendar.in_era` | 0.00493× | 0.0154× | 89 806 | 51 621 |
