# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (27 438 262 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `c6a050279cb1`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 19 200 249 | 4 754 220 |
| `chart.found` | 20.5× | 5.32× | 394 311 013 | 25 275 379 |
| `chart.disagreements` | 3.09× | 8.10× | 59 331 470 | 38 501 310 |
| `almanac.days` | 175× | 282× | 3 368 625 585 | 1 342 614 786 |
| `matching.naam` | 0.0167× | 0.0611× | 319 927 | 290 712 |
| `naming.syllables` | 0.0225× | 0.0891× | 431 910 | 423 733 |
| `naming.read` | 0.0228× | 0.0597× | 438 032 | 283 620 |
| `naming.check` | 0.0160× | 0.0542× | 306 394 | 257 813 |
| `naming.of` | 0.00397× | 0.0112× | 76 240 | 53 076 |
| `naming.suggest` | 0.0193× | 0.0718× | 369 940 | 341 390 |
| `time.resolve` | 0.00663× | 0.0230× | 127 244 | 109 206 |
| `time.civil` | 0.00624× | 0.0165× | 119 836 | 78 330 |
| `time.convert` | 0.00184× | 0.00713× | 35 260 | 33 898 |
| `calendar.convert` | 0.00469× | 0.0115× | 90 100 | 54 436 |
| `calendar.add` | 0.00567× | 0.0145× | 108 859 | 69 157 |
| `calendar.between` | 0.00555× | 0.0139× | 106 532 | 66 233 |
| `numerology.profile` | 0.0146× | 0.0533× | 279 376 | 253 324 |
| `almanac.navatara` | 0.0136× | 0.0578× | 261 560 | 274 582 |
| `intl.inWords` | 0.0663× | 0.0324× | 1 273 359 | 154 013 |
| `intl.time` | 0.00716× | 0.0244× | 137 504 | 116 199 |
| `intl.date` | 0.0151× | 0.0371× | 290 090 | 176 285 |
| `intl.lexicon` | 2.45× | 0.271× | 47 028 484 | 1 287 473 |
| `intl.lookup` | 4.17× | 0.106× | 80 006 474 | 503 869 |
| `intl.dates` | 0.317× | 0.0325× | 6 085 074 | 154 747 |
| `verify.claims` | 74.9× | 61.9× | 1 437 193 821 | 294 511 012 |
| `verify.issue` | 0.165× | 0.668× | 3 160 223 | 3 174 799 |
| `verify.receipt` | 0.337× | 1.32× | 6 476 468 | 6 262 371 |
| `almanac.digest` | 15.2× | 24.9× | 291 626 485 | 118 459 931 |
| `almanac.festival_calendar` | 26.4× | 43.9× | 506 048 364 | 208 846 428 |
| `almanac.changes` | 4.81× | 3.75× | 92 404 539 | 17 809 313 |
| `almanac.anniversaries` | 1677× | 6293× | 32 205 815 769 | 29 916 941 981 |
| `almanac.next` | 162× | 300× | 3 106 736 237 | 1 427 151 702 |
| `almanac.sentences` | 10.4× | 6.68× | 199 357 994 | 31 741 789 |
| `almanac.publishedReasons` | 25.0× | 42.8× | 480 691 618 | 203 571 704 |
| `almanac.pakshi` | 10.5× | 10.5× | 202 481 169 | 49 887 700 |
| `almanac.sankalpa` | 71.6× | 63.5× | 1 373 895 959 | 301 775 204 |
| `almanac.ashauch` | 27.8× | 33.3× | 533 840 394 | 158 310 310 |
| `almanac.household` | 38.0× | 70.3× | 728 746 919 | 334 365 434 |
| `almanac.ashauchKin` | 0.00625× | 0.0233× | 119 986 | 110 712 |
| `almanac.ashauchRite` | 0.00347× | 0.0137× | 66 556 | 65 103 |
| `almanac.deathTime` | 89.3× | 139× | 1 714 781 550 | 662 696 030 |
| `almanac.festivalPack` | 0.0838× | 0.348× | 1 608 052 | 1 652 886 |
| `almanac.firstYear` | 161× | 281× | 3 099 417 275 | 1 335 815 509 |
| `chart.rashifal` | 9.58× | 9.27× | 183 984 796 | 44 092 955 |
| `research.study` | 3.08× | 3.17× | 59 208 462 | 15 075 013 |
| `chart.mundane` | 2.61× | 0.995× | 50 204 058 | 4 729 708 |
| `chart.relocated` | 1.05× | 0.890× | 20 226 836 | 4 233 468 |
| `chart.astrocartography` | 1.11× | 1.13× | 21 369 903 | 5 393 313 |
| `almanac.skyView` | 0.619× | 0.917× | 11 882 389 | 4 357 393 |
