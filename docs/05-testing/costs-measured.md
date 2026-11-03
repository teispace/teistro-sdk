# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (26 282 782 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `198304d0ba44`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 19 176 620 | 4 758 796 |
| `chart.found` | 20.6× | 5.32× | 394 143 184 | 25 337 297 |
| `chart.disagreements` | 3.09× | 8.09× | 59 339 394 | 38 509 257 |
| `almanac.days` | 176× | 282× | 3 368 241 207 | 1 343 075 447 |
| `matching.naam` | 0.0168× | 0.0618× | 322 325 | 294 182 |
| `naming.syllables` | 0.0227× | 0.0896× | 434 925 | 426 430 |
| `naming.read` | 0.0230× | 0.0628× | 440 384 | 299 052 |
| `naming.check` | 0.0158× | 0.0549× | 303 537 | 261 235 |
| `naming.of` | 0.00390× | 0.0112× | 74 867 | 53 385 |
| `naming.suggest` | 0.0202× | 0.0716× | 387 449 | 340 803 |
| `time.resolve` | 0.00659× | 0.0232× | 126 398 | 110 262 |
| `time.civil` | 0.00613× | 0.0168× | 117 471 | 79 865 |
| `time.convert` | 0.00172× | 0.00718× | 32 897 | 34 172 |
| `calendar.convert` | 0.00474× | 0.0115× | 90 855 | 54 880 |
| `calendar.add` | 0.00584× | 0.0146× | 111 939 | 69 270 |
| `calendar.between` | 0.00547× | 0.0143× | 104 834 | 68 065 |
| `numerology.profile` | 0.0147× | 0.0535× | 282 068 | 254 699 |
| `almanac.navatara` | 0.0136× | 0.0580× | 261 736 | 275 912 |
| `intl.inWords` | 0.0661× | 0.0330× | 1 267 705 | 156 844 |
| `intl.time` | 0.00734× | 0.0245× | 140 686 | 116 591 |
| `intl.date` | 0.0153× | 0.0371× | 292 560 | 176 379 |
| `intl.lexicon` | 2.42× | 0.272× | 46 415 398 | 1 294 156 |
| `intl.lookup` | 4.10× | 0.105× | 78 633 505 | 499 093 |
| `intl.dates` | 0.317× | 0.0325× | 6 084 962 | 154 787 |
| `verify.claims` | 74.8× | 61.9× | 1 435 365 484 | 294 426 004 |
| `almanac.digest` | 15.2× | 24.9× | 291 671 997 | 118 632 385 |
| `almanac.festival_calendar` | 26.4× | 43.9× | 506 193 635 | 209 064 430 |
| `almanac.changes` | 4.82× | 3.75× | 92 393 334 | 17 839 969 |
| `almanac.anniversaries` | 1680× | 6291× | 32 222 798 532 | 29 935 814 742 |
| `almanac.next` | 162× | 300× | 3 108 433 093 | 1 429 466 016 |
| `almanac.sentences` | 10.4× | 6.67× | 199 318 235 | 31 754 553 |
| `almanac.publishedReasons` | 25.1× | 42.8× | 480 890 872 | 203 851 658 |
| `almanac.pakshi` | 10.6× | 10.5× | 202 505 958 | 49 990 019 |
| `almanac.sankalpa` | 71.6× | 63.4× | 1 373 475 740 | 301 757 721 |
| `almanac.ashauch` | 27.8× | 33.3× | 533 944 113 | 158 539 324 |
| `almanac.household` | 38.0× | 70.4× | 729 143 823 | 334 946 224 |
| `almanac.ashauchKin` | 0.00596× | 0.0228× | 114 352 | 108 542 |
| `almanac.ashauchRite` | 0.00341× | 0.0137× | 65 338 | 65 267 |
| `almanac.deathTime` | 89.4× | 139× | 1 714 336 160 | 662 550 214 |
| `almanac.festivalPack` | 0.0833× | 0.349× | 1 598 336 | 1 661 149 |
| `almanac.firstYear` | 162× | 281× | 3 101 042 879 | 1 337 790 519 |
| `chart.rashifal` | 9.60× | 9.28× | 184 032 860 | 44 180 869 |
| `research.study` | 3.09× | 3.17× | 59 236 991 | 15 084 109 |
