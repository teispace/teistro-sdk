# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (27 794 596 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `a9d9bfe30441`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 19 072 022 | 4 686 361 |
| `chart.found` | 20.7× | 5.33× | 393 903 302 | 24 961 914 |
| `chart.disagreements` | 3.10× | 8.18× | 59 120 204 | 38 315 274 |
| `almanac.days` | 176× | 286× | 3 359 463 860 | 1 338 202 456 |
| `matching.naam` | 0.0171× | 0.0682× | 325 889 | 319 616 |
| `naming.syllables` | 0.0223× | 0.0857× | 425 222 | 401 423 |
| `naming.read` | 0.0229× | 0.0640× | 436 205 | 299 788 |
| `naming.check` | 0.0157× | 0.0585× | 299 483 | 273 937 |
| `naming.of` | 0.00377× | 0.0107× | 71 816 | 49 982 |
| `naming.suggest` | 0.0190× | 0.0716× | 361 898 | 335 742 |
| `time.resolve` | 0.00643× | 0.0226× | 122 590 | 106 024 |
| `time.civil` | 0.00599× | 0.0160× | 114 329 | 75 018 |
| `time.convert` | 0.00180× | 0.00639× | 34 273 | 29 938 |
| `calendar.convert` | 0.00451× | 0.0104× | 86 084 | 48 538 |
| `calendar.add` | 0.00536× | 0.0135× | 102 191 | 63 385 |
| `calendar.between` | 0.00520× | 0.0126× | 99 218 | 59 271 |
| `numerology.profile` | 0.0144× | 0.0571× | 274 776 | 267 464 |
| `almanac.navatara` | 0.0141× | 0.0558× | 267 984 | 261 330 |
| `almanac.aayadi` | 0.0108× | 0.0431× | 205 511 | 201 766 |
| `intl.inWords` | 0.0917× | 0.0301× | 1 748 874 | 140 913 |
| `intl.time` | 0.00699× | 0.0228× | 133 388 | 106 797 |
| `intl.date` | 0.0148× | 0.0344× | 282 260 | 160 986 |
| `intl.lexicon` | 2.42× | 0.274× | 46 201 651 | 1 284 488 |
| `intl.lookup` | 4.02× | 0.108× | 76 663 697 | 506 539 |
| `intl.dates` | 0.315× | 0.0327× | 6 008 155 | 153 260 |
| `verify.claims` | 75.0× | 62.7× | 1 431 239 201 | 293 719 442 |
| `verify.issue` | 0.164× | 0.661× | 3 126 343 | 3 099 195 |
| `verify.receipt` | 0.334× | 1.32× | 6 361 229 | 6 170 281 |
| `almanac.digest` | 15.2× | 25.1× | 290 263 451 | 117 459 386 |
| `almanac.festival_calendar` | 26.4× | 44.2× | 504 105 780 | 207 307 765 |
| `almanac.changes` | 4.83× | 3.77× | 92 192 224 | 17 667 582 |
| `almanac.anniversaries` | 1685× | 6371× | 32 143 620 898 | 29 856 013 094 |
| `almanac.next` | 163× | 303× | 3 100 831 117 | 1 422 232 044 |
| `almanac.sentences` | 10.4× | 6.67× | 198 636 925 | 31 250 245 |
| `almanac.publishedReasons` | 25.1× | 43.2× | 479 243 632 | 202 392 427 |
| `almanac.pakshi` | 10.6× | 10.6× | 201 875 423 | 49 491 222 |
| `almanac.sankalpa` | 72.0× | 64.2× | 1 372 788 967 | 301 080 723 |
| `almanac.ashauch` | 28.0× | 33.7× | 533 079 356 | 157 703 434 |
| `almanac.household` | 38.1× | 71.1× | 727 220 196 | 333 148 678 |
| `almanac.ashauchKin` | 0.00529× | 0.0194× | 100 804 | 91 148 |
| `almanac.ashauchRite` | 0.00340× | 0.0113× | 64 823 | 53 040 |
| `almanac.deathTime` | 89.8× | 141× | 1 713 027 500 | 661 360 256 |
| `almanac.festivalPack` | 0.0918× | 0.390× | 1 750 230 | 1 826 664 |
| `almanac.firstYear` | 162× | 284× | 3 095 508 941 | 1 331 499 350 |
| `chart.rashifal` | 9.60× | 9.29× | 183 071 475 | 43 555 936 |
| `research.study` | 3.08× | 3.19× | 58 835 758 | 14 948 537 |
| `chart.mundane` | 2.63× | 0.997× | 50 092 670 | 4 670 663 |
| `chart.relocated` | 1.05× | 0.895× | 20 098 171 | 4 192 545 |
| `chart.astrocartography` | 1.11× | 1.14× | 21 221 499 | 5 332 703 |
| `almanac.skyView` | 0.620× | 0.920× | 11 823 626 | 4 309 772 |
| `almanac.elect` | 301× | 472× | 5 746 664 487 | 2 209 776 071 |
| `almanac.observance` | 205× | 500× | 3 905 336 002 | 2 342 762 181 |
| `almanac.people` | 12.6× | 13.0× | 240 324 446 | 60 761 282 |
