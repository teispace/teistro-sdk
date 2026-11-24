# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (7 399 513 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `74e3dff5cdb5`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 257 886 | 3 390 163 |
| `chart.found` | 21.5× | 5.99× | 392 675 339 | 20 311 130 |
| `chart.disagreements` | 3.14× | 10.5× | 57 290 501 | 35 750 565 |
| `almanac.days` | 179× | 365× | 3 266 668 052 | 1 237 932 890 |
| `matching.naam` | 0.0180× | 0.0952× | 329 234 | 322 835 |
| `matching.band` | 1.43× | 0.0231× | 26 168 486 | 78 463 |
| `naming.syllables` | 0.0237× | 0.120× | 432 246 | 406 273 |
| `naming.read` | 0.0245× | 0.0890× | 446 528 | 301 730 |
| `naming.check` | 0.0166× | 0.0812× | 302 279 | 275 307 |
| `naming.of` | 0.00394× | 0.0148× | 71 849 | 50 306 |
| `naming.suggest` | 0.0205× | 0.0995× | 374 790 | 337 364 |
| `time.resolve` | 0.00658× | 0.0308× | 120 063 | 104 572 |
| `geo.resolve` | 0.0621× | 0.0192× | 1 134 457 | 65 061 |
| `geo.search` | 0.0611× | 0.0274× | 1 116 448 | 92 748 |
| `geo.reverse` | 0.0709× | 0.0685× | 1 294 485 | 232 350 |
| `geo.names` | 0.0641× | 0.0359× | 1 170 140 | 121 556 |
| `geo.list` | 0.0648× | 0.0371× | 1 182 224 | 125 813 |
| `time.civil` | 0.00617× | 0.0217× | 112 678 | 73 541 |
| `time.convert` | 0.00165× | 0.00840× | 30 075 | 28 474 |
| `calendar.convert` | 0.00475× | 0.0142× | 86 711 | 48 307 |
| `calendar.add` | 0.00542× | 0.0173× | 99 010 | 58 766 |
| `calendar.between` | 0.00532× | 0.0163× | 97 077 | 55 194 |
| `numerology.profile` | 0.0156× | 0.0796× | 284 938 | 269 880 |
| `almanac.navatara` | 0.0150× | 0.0797× | 273 868 | 270 093 |
| `almanac.aayadi` | 0.0114× | 0.0606× | 208 990 | 205 351 |
| `intl.inWords` | 1.52× | 0.0413× | 27 794 863 | 139 848 |
| `intl.time` | 1.43× | 0.0314× | 26 169 221 | 106 452 |
| `intl.date` | 1.36× | 0.0489× | 24 918 320 | 165 797 |
| `intl.lexicon` | 6.51× | 0.516× | 118 802 498 | 1 749 647 |
| `intl.lookup` | 14.9× | 0.198× | 271 641 931 | 672 522 |
| `intl.dates` | 3.81× | 0.0450× | 69 587 568 | 152 642 |
| `verify.claims` | 88.5× | 84.2× | 1 616 000 259 | 285 592 496 |
| `verify.issue` | 0.190× | 1.00× | 3 463 453 | 3 406 776 |
| `verify.receipt` | 0.273× | 1.40× | 4 988 908 | 4 762 585 |
| `almanac.digest` | 25.0× | 31.4× | 455 841 939 | 106 413 033 |
| `almanac.festival_calendar` | 35.8× | 53.1× | 653 380 232 | 179 889 069 |
| `almanac.changes` | 4.84× | 3.08× | 88 380 251 | 10 454 886 |
| `almanac.cells` | 15.7× | 15.8× | 287 300 121 | 53 423 983 |
| `almanac.anniversaries` | 1587× | 7874× | 28 976 287 735 | 26 693 684 989 |
| `almanac.next` | 168× | 372× | 3 061 041 933 | 1 261 777 280 |
| `almanac.sentences` | 12.4× | 5.78× | 226 850 290 | 19 588 007 |
| `almanac.publishedReasons` | 33.0× | 51.3× | 602 354 164 | 173 979 542 |
| `almanac.pakshi` | 10.6× | 9.30× | 194 059 935 | 31 513 550 |
| `almanac.sankalpa` | 74.6× | 86.1× | 1 362 071 674 | 291 879 919 |
| `almanac.sankalpaWords` | 74.9× | 83.7× | 1 367 950 819 | 283 605 723 |
| `almanac.ashauch` | 27.8× | 34.6× | 508 034 550 | 117 358 838 |
| `almanac.household` | 36.1× | 73.1× | 658 739 771 | 247 685 232 |
| `almanac.ashauchKin` | 0.00547× | 0.0269× | 99 790 | 91 325 |
| `almanac.ashauchRite` | 0.00353× | 0.0161× | 64 410 | 54 618 |
| `almanac.deathTime` | 91.7× | 180× | 1 674 791 751 | 608 635 128 |
| `almanac.festivalPack` | 0.122× | 0.648× | 2 223 493 | 2 197 208 |
| `almanac.firstYear` | 156× | 317× | 2 856 229 235 | 1 076 281 063 |
| `chart.rashifal` | 9.77× | 8.85× | 178 390 340 | 29 993 546 |
| `research.study` | 3.03× | 2.92× | 55 251 675 | 9 888 680 |
| `chart.mundane` | 2.70× | 0.902× | 49 273 720 | 3 058 006 |
| `chart.relocated` | 1.05× | 0.857× | 19 208 073 | 2 905 662 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 391 866 | 4 098 045 |
| `almanac.skyView` | 0.644× | 1.23× | 11 751 606 | 4 179 189 |
| `almanac.elect` | 280× | 462× | 5 112 435 750 | 1 565 192 499 |
| `almanac.observance` | 204× | 602× | 3 724 650 654 | 2 041 606 577 |
| `almanac.people` | 12.9× | 13.7× | 234 972 953 | 46 446 994 |
| `almanac.skyPose` | 0.737× | 1.74× | 13 463 486 | 5 896 511 |
| `calendar.in_era` | 0.00501× | 0.0155× | 91 535 | 52 423 |
| `settings.explain` | 0.746× | 0.404× | 13 627 562 | 1 371 014 |
| `settings.diff` | 0.128× | 0.686× | 2 338 038 | 2 326 526 |
| `almanac.limbAngles` | 0.476× | 0.323× | 8 685 231 | 1 096 373 |
| `almanac.blackouts` | 135× | 143× | 2 458 014 906 | 485 645 787 |
| `almanac.occurrences` | 410× | 1562× | 7 484 305 743 | 5 294 948 444 |
| `intl.render` | 0.690× | 0.0530× | 12 597 538 | 179 702 |
