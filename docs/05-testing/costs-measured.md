# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (5 686 397 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `9d95c9d69d1e`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 747 546 | 4 343 341 |
| `chart.found` | 20.9× | 5.30× | 391 964 171 | 23 010 993 |
| `chart.disagreements` | 3.12× | 8.65× | 58 453 925 | 37 570 034 |
| `almanac.days` | 175× | 290× | 3 281 332 387 | 1 259 451 444 |
| `matching.naam` | 0.0177× | 0.0749× | 332 699 | 325 346 |
| `matching.band` | 0.971× | 0.0178× | 18 204 154 | 77 216 |
| `naming.syllables` | 0.0231× | 0.0938× | 433 955 | 407 544 |
| `naming.read` | 0.0239× | 0.0706× | 447 479 | 306 747 |
| `naming.check` | 0.0164× | 0.0641× | 306 567 | 278 486 |
| `naming.of` | 0.00377× | 0.0115× | 70 707 | 50 149 |
| `naming.suggest` | 0.0202× | 0.0782× | 377 970 | 339 562 |
| `time.resolve` | 0.00653× | 0.0246× | 122 481 | 106 848 |
| `time.civil` | 0.00612× | 0.0177× | 114 733 | 76 846 |
| `time.convert` | 0.00167× | 0.00719× | 31 250 | 31 225 |
| `calendar.convert` | 0.00461× | 0.0114× | 86 490 | 49 646 |
| `calendar.add` | 0.00543× | 0.0148× | 101 816 | 64 421 |
| `calendar.between` | 0.00522× | 0.0139× | 97 931 | 60 178 |
| `numerology.profile` | 0.0151× | 0.0625× | 283 792 | 271 588 |
| `almanac.navatara` | 0.0146× | 0.0621× | 274 315 | 269 852 |
| `almanac.aayadi` | 0.0111× | 0.0478× | 208 981 | 207 704 |
| `intl.inWords` | 1.06× | 0.0330× | 19 829 005 | 143 235 |
| `intl.time` | 0.971× | 0.0245× | 18 207 065 | 106 535 |
| `intl.date` | 0.854× | 0.0383× | 16 016 795 | 166 423 |
| `intl.lexicon` | 4.36× | 0.358× | 81 652 679 | 1 556 848 |
| `intl.lookup` | 9.15× | 0.134× | 171 447 011 | 582 181 |
| `intl.dates` | 2.53× | 0.0353× | 47 463 977 | 153 482 |
| `verify.claims` | 80.6× | 64.4× | 1 511 744 244 | 279 505 362 |
| `verify.issue` | 0.168× | 0.717× | 3 156 281 | 3 112 292 |
| `verify.receipt` | 0.341× | 1.42× | 6 385 419 | 6 157 541 |
| `almanac.digest` | 14.7× | 21.7× | 275 609 875 | 94 216 331 |
| `almanac.festival_calendar` | 25.1× | 38.1× | 470 902 860 | 165 409 735 |
| `almanac.changes` | 4.72× | 3.20× | 88 427 634 | 13 877 911 |
| `almanac.cells` | 15.0× | 14.6× | 280 517 338 | 63 285 132 |
| `almanac.anniversaries` | 1546× | 6148× | 28 992 428 056 | 26 703 063 598 |
| `almanac.next` | 149× | 259× | 2 801 932 391 | 1 123 013 390 |
| `almanac.sentences` | 11.3× | 5.88× | 211 029 784 | 25 545 309 |
| `almanac.publishedReasons` | 23.5× | 37.7× | 440 785 930 | 163 923 207 |
| `almanac.pakshi` | 10.3× | 9.57× | 193 982 149 | 41 580 855 |
| `almanac.sankalpa` | 72.4× | 65.9× | 1 358 089 407 | 286 403 727 |
| `almanac.sankalpaWords` | 72.4× | 64.0× | 1 356 482 999 | 277 858 270 |
| `almanac.ashauch` | 26.7× | 28.8× | 500 315 459 | 124 889 266 |
| `almanac.household` | 35.1× | 60.8× | 658 356 292 | 264 008 274 |
| `almanac.ashauchKin` | 0.00536× | 0.0209× | 100 557 | 90 959 |
| `almanac.ashauchRite` | 0.00336× | 0.0122× | 62 925 | 53 079 |
| `almanac.deathTime` | 89.9× | 146× | 1 685 128 575 | 633 336 562 |
| `almanac.festivalPack` | 0.0988× | 0.421× | 1 851 567 | 1 830 227 |
| `almanac.firstYear` | 150× | 243× | 2 817 643 144 | 1 053 987 536 |
| `chart.rashifal` | 9.40× | 8.43× | 176 215 026 | 36 627 136 |
| `research.study` | 3.07× | 3.12× | 57 546 799 | 13 572 294 |
| `chart.mundane` | 2.64× | 0.946× | 49 531 913 | 4 110 619 |
| `chart.relocated` | 1.05× | 0.886× | 19 763 990 | 3 848 356 |
| `chart.astrocartography` | 1.11× | 1.15× | 20 899 019 | 4 988 249 |
| `almanac.skyView` | 0.630× | 0.984× | 11 807 303 | 4 273 104 |
| `almanac.elect` | 288× | 430× | 5 404 594 486 | 1 865 995 323 |
| `almanac.observance` | 182× | 428× | 3 420 079 924 | 1 856 794 156 |
| `almanac.people` | 12.5× | 12.4× | 233 538 743 | 54 063 313 |
