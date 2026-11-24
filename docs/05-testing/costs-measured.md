# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (7 284 086 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `e4f43805ad2b`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 261 316 | 3 396 627 |
| `chart.found` | 21.5× | 5.99× | 392 709 529 | 20 341 492 |
| `chart.disagreements` | 3.14× | 10.5× | 57 298 731 | 35 749 273 |
| `almanac.days` | 179× | 365× | 3 266 699 701 | 1 239 441 780 |
| `matching.naam` | 0.0182× | 0.0949× | 332 275 | 322 467 |
| `matching.band` | 1.33× | 0.0232× | 24 317 426 | 78 760 |
| `naming.syllables` | 0.0238× | 0.120× | 434 056 | 406 893 |
| `naming.read` | 0.0244× | 0.0887× | 446 201 | 301 370 |
| `naming.check` | 0.0167× | 0.0814× | 304 579 | 276 407 |
| `naming.of` | 0.00385× | 0.0146× | 70 248 | 49 449 |
| `naming.suggest` | 0.0204× | 0.0989× | 372 991 | 335 954 |
| `time.resolve` | 0.00669× | 0.0311× | 122 133 | 105 544 |
| `geo.resolve` | 0.0615× | 0.0190× | 1 123 291 | 64 571 |
| `geo.search` | 0.0628× | 0.0275× | 1 147 333 | 93 572 |
| `geo.reverse` | 0.0704× | 0.0688× | 1 285 288 | 233 756 |
| `geo.names` | 0.0648× | 0.0358× | 1 183 808 | 121 516 |
| `geo.list` | 0.0651× | 0.0373× | 1 188 434 | 126 754 |
| `time.civil` | 0.00614× | 0.0219× | 112 128 | 74 236 |
| `time.convert` | 0.00168× | 0.00846× | 30 753 | 28 746 |
| `calendar.convert` | 0.00460× | 0.0142× | 84 015 | 48 088 |
| `calendar.add` | 0.00547× | 0.0176× | 99 907 | 59 873 |
| `calendar.between` | 0.00523× | 0.0162× | 95 450 | 54 971 |
| `numerology.profile` | 0.0157× | 0.0804× | 286 419 | 273 234 |
| `almanac.navatara` | 0.0151× | 0.0796× | 276 508 | 270 287 |
| `almanac.aayadi` | 0.0117× | 0.0610× | 213 058 | 207 145 |
| `intl.inWords` | 1.42× | 0.0413× | 25 949 967 | 140 235 |
| `intl.time` | 1.33× | 0.0316× | 24 321 764 | 107 233 |
| `intl.date` | 1.32× | 0.0489× | 24 143 376 | 165 946 |
| `intl.lexicon` | 5.85× | 0.498× | 106 880 746 | 1 693 050 |
| `intl.lookup` | 14.0× | 0.190× | 256 425 636 | 644 562 |
| `intl.dates` | 3.71× | 0.0451× | 67 725 468 | 153 225 |
| `verify.claims` | 87.7× | 84.0× | 1 600 783 256 | 285 455 803 |
| `verify.issue` | 0.190× | 1.01× | 3 468 977 | 3 414 888 |
| `verify.receipt` | 0.274× | 1.40× | 4 995 388 | 4 763 233 |
| `almanac.digest` | 24.9× | 31.3× | 455 121 844 | 106 367 187 |
| `almanac.festival_calendar` | 35.7× | 53.0× | 652 671 956 | 180 082 157 |
| `almanac.changes` | 4.84× | 3.08× | 88 387 170 | 10 464 731 |
| `almanac.cells` | 15.7× | 15.7× | 287 343 181 | 53 412 107 |
| `almanac.anniversaries` | 1587× | 7859× | 28 978 914 371 | 26 694 862 000 |
| `almanac.next` | 168× | 372× | 3 061 378 133 | 1 262 227 738 |
| `almanac.sentences` | 12.3× | 5.77× | 225 017 168 | 19 593 588 |
| `almanac.publishedReasons` | 33.0× | 51.2× | 602 462 734 | 173 916 377 |
| `almanac.pakshi` | 10.6× | 9.28× | 194 091 594 | 31 520 616 |
| `almanac.sankalpa` | 74.6× | 86.0× | 1 362 135 169 | 292 009 633 |
| `almanac.sankalpaWords` | 74.9× | 83.5× | 1 367 175 982 | 283 667 163 |
| `almanac.ashauch` | 27.8× | 34.5× | 507 987 496 | 117 326 330 |
| `almanac.household` | 36.1× | 73.0× | 658 887 471 | 247 792 547 |
| `almanac.ashauchKin` | 0.00549× | 0.0266× | 100 177 | 90 349 |
| `almanac.ashauchRite` | 0.00351× | 0.0156× | 64 027 | 53 037 |
| `almanac.deathTime` | 91.7× | 179× | 1 674 802 815 | 608 889 352 |
| `almanac.festivalPack` | 0.122× | 0.648× | 2 229 295 | 2 201 259 |
| `almanac.firstYear` | 156× | 317× | 2 856 278 979 | 1 076 877 652 |
| `chart.rashifal` | 9.77× | 8.82× | 178 369 503 | 29 959 327 |
| `research.study` | 3.03× | 2.90× | 55 276 451 | 9 861 893 |
| `chart.mundane` | 2.70× | 0.900× | 49 273 663 | 3 057 071 |
| `chart.relocated` | 1.05× | 0.854× | 19 208 127 | 2 901 696 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 395 288 | 4 094 942 |
| `almanac.skyView` | 0.644× | 1.23× | 11 752 863 | 4 186 076 |
| `almanac.elect` | 280× | 461× | 5 112 048 871 | 1 567 374 677 |
| `almanac.observance` | 204× | 601× | 3 724 743 965 | 2 041 352 484 |
| `almanac.people` | 12.9× | 13.7× | 234 992 927 | 46 463 672 |
| `almanac.skyPose` | 0.737× | 1.74× | 13 464 907 | 5 893 386 |
| `calendar.in_era` | 0.00493× | 0.0154× | 89 988 | 52 202 |
| `settings.explain` | 0.704× | 0.404× | 12 860 668 | 1 372 464 |
| `settings.diff` | 0.128× | 0.686× | 2 334 227 | 2 328 551 |
| `almanac.limbAngles` | 0.476× | 0.323× | 8 686 926 | 1 096 061 |
| `almanac.blackouts` | 135× | 143× | 2 458 065 556 | 485 760 263 |
| `almanac.occurrences` | 410× | 1559× | 7 484 603 629 | 5 295 393 460 |
| `intl.render` | 0.648× | 0.0531× | 11 832 361 | 180 241 |
