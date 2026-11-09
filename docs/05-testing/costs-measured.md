# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (27 099 375 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `dbd46a886b56`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 19 071 237 | 4 684 515 |
| `chart.found` | 20.7× | 5.33× | 393 900 658 | 24 949 515 |
| `chart.disagreements` | 3.10× | 8.16× | 59 076 135 | 38 245 023 |
| `almanac.days` | 176× | 286× | 3 359 751 712 | 1 337 458 944 |
| `matching.naam` | 0.0169× | 0.0680× | 321 373 | 318 684 |
| `naming.syllables` | 0.0220× | 0.0855× | 420 002 | 400 512 |
| `naming.read` | 0.0224× | 0.0639× | 427 228 | 299 240 |
| `naming.check` | 0.0156× | 0.0584× | 296 873 | 273 803 |
| `naming.of` | 0.00359× | 0.0106× | 68 547 | 49 862 |
| `naming.suggest` | 0.0190× | 0.0712× | 362 672 | 333 717 |
| `time.resolve` | 0.00635× | 0.0225× | 121 043 | 105 623 |
| `time.civil` | 0.00571× | 0.0161× | 108 899 | 75 583 |
| `time.convert` | 0.00175× | 0.00640× | 33 331 | 29 959 |
| `calendar.convert` | 0.00442× | 0.0103× | 84 255 | 48 188 |
| `calendar.add` | 0.00538× | 0.0135× | 102 575 | 63 239 |
| `calendar.between` | 0.00523× | 0.0128× | 99 743 | 59 756 |
| `numerology.profile` | 0.0142× | 0.0568× | 270 892 | 265 923 |
| `almanac.navatara` | 0.0137× | 0.0555× | 261 970 | 260 148 |
| `almanac.aayadi` | 0.0106× | 0.0430× | 201 507 | 201 254 |
| `intl.inWords` | 0.0643× | 0.0298× | 1 226 584 | 139 771 |
| `intl.time` | 0.00678× | 0.0224× | 129 371 | 105 122 |
| `intl.date` | 0.0146× | 0.0342× | 277 648 | 160 043 |
| `intl.lexicon` | 2.38× | 0.273× | 45 363 236 | 1 277 631 |
| `intl.lookup` | 3.93× | 0.107× | 75 036 622 | 501 945 |
| `intl.dates` | 0.301× | 0.0328× | 5 736 844 | 153 571 |
| `verify.claims` | 75.0× | 62.7× | 1 429 597 461 | 293 695 862 |
| `verify.issue` | 0.164× | 0.660× | 3 119 575 | 3 093 793 |
| `verify.receipt` | 0.336× | 1.32× | 6 405 174 | 6 167 708 |
| `almanac.digest` | 15.2× | 25.1× | 290 253 002 | 117 487 575 |
| `almanac.festival_calendar` | 26.4× | 44.3× | 504 104 433 | 207 303 636 |
| `almanac.changes` | 4.83× | 3.77× | 92 187 520 | 17 667 720 |
| `almanac.anniversaries` | 1685× | 6373× | 32 143 601 833 | 29 856 558 314 |
| `almanac.next` | 163× | 304× | 3 101 245 542 | 1 422 390 645 |
| `almanac.sentences` | 10.4× | 6.66× | 198 596 776 | 31 204 251 |
| `almanac.publishedReasons` | 25.1× | 43.2× | 479 210 822 | 202 377 206 |
| `almanac.pakshi` | 10.6× | 10.6× | 201 873 105 | 49 511 895 |
| `almanac.sankalpa` | 72.0× | 64.3× | 1 372 768 468 | 301 083 574 |
| `almanac.ashauch` | 28.0× | 33.7× | 533 109 176 | 157 669 084 |
| `almanac.household` | 38.1× | 71.1× | 727 241 083 | 333 032 985 |
| `almanac.ashauchKin` | 0.00520× | 0.0195× | 99 263 | 91 354 |
| `almanac.ashauchRite` | 0.00318× | 0.0112× | 60 730 | 52 500 |
| `almanac.deathTime` | 89.8× | 141× | 1 713 028 081 | 661 303 018 |
| `almanac.festivalPack` | 0.0917× | 0.392× | 1 748 094 | 1 834 249 |
| `almanac.firstYear` | 162× | 284× | 3 095 213 432 | 1 331 507 226 |
| `chart.rashifal` | 9.60× | 9.31× | 183 070 655 | 43 623 821 |
| `research.study` | 3.09× | 3.18× | 58 840 212 | 14 912 756 |
| `chart.mundane` | 2.63× | 0.997× | 50 087 791 | 4 669 266 |
| `chart.relocated` | 1.05× | 0.895× | 20 096 563 | 4 192 199 |
| `chart.astrocartography` | 1.11× | 1.14× | 21 220 186 | 5 330 479 |
| `almanac.skyView` | 0.620× | 0.920× | 11 821 160 | 4 311 879 |
| `almanac.elect` | 301× | 472× | 5 747 178 524 | 2 210 094 371 |
| `almanac.observance` | 205× | 500× | 3 905 870 450 | 2 342 392 278 |
| `almanac.people` | 12.6× | 13.0× | 240 283 472 | 60 777 118 |
