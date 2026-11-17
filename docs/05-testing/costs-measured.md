# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (6 670 460 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `09f150fb7a34`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 236 227 | 3 367 567 |
| `chart.found` | 21.5× | 5.99× | 392 544 582 | 20 184 920 |
| `chart.disagreements` | 3.14× | 10.6× | 57 286 553 | 35 706 634 |
| `almanac.days` | 178× | 364× | 3 252 670 931 | 1 226 569 177 |
| `matching.naam` | 0.0179× | 0.0940× | 326 448 | 316 451 |
| `matching.band` | 1.20× | 0.0223× | 21 921 246 | 75 058 |
| `naming.syllables` | 0.0236× | 0.119× | 430 321 | 400 876 |
| `naming.read` | 0.0245× | 0.0886× | 446 321 | 298 449 |
| `naming.check` | 0.0166× | 0.0810× | 302 250 | 272 628 |
| `naming.of` | 0.00402× | 0.0149× | 73 356 | 50 133 |
| `naming.suggest` | 0.0204× | 0.0996× | 372 335 | 335 479 |
| `time.resolve` | 0.00662× | 0.0305× | 120 786 | 102 587 |
| `geo.resolve` | 0.0217× | 0.0187× | 395 110 | 62 816 |
| `time.civil` | 0.00607× | 0.0217× | 110 644 | 72 970 |
| `time.convert` | 0.00163× | 0.00839× | 29 724 | 28 240 |
| `calendar.convert` | 0.00462× | 0.0141× | 84 249 | 47 584 |
| `calendar.add` | 0.00546× | 0.0175× | 99 564 | 58 989 |
| `calendar.between` | 0.00526× | 0.0163× | 95 942 | 54 763 |
| `numerology.profile` | 0.0154× | 0.0793× | 281 722 | 267 060 |
| `almanac.navatara` | 0.0147× | 0.0784× | 268 133 | 264 087 |
| `almanac.aayadi` | 0.0114× | 0.0600× | 207 795 | 201 919 |
| `intl.inWords` | 1.29× | 0.0417× | 23 542 090 | 140 489 |
| `intl.time` | 1.20× | 0.0312× | 21 923 130 | 105 157 |
| `intl.date` | 1.21× | 0.0484× | 22 050 382 | 163 105 |
| `intl.lexicon` | 5.32× | 0.482× | 96 985 352 | 1 622 271 |
| `intl.lookup` | 12.6× | 0.182× | 230 433 948 | 613 756 |
| `intl.dates` | 3.35× | 0.0453× | 61 013 132 | 152 565 |
| `verify.claims` | 86.3× | 84.7× | 1 574 633 302 | 285 295 990 |
| `verify.issue` | 0.185× | 0.987× | 3 377 688 | 3 323 969 |
| `verify.receipt` | 0.273× | 1.41× | 4 979 461 | 4 763 425 |
| `almanac.digest` | 23.6× | 28.5× | 429 694 348 | 96 130 810 |
| `almanac.festival_calendar` | 34.4× | 50.2× | 627 153 479 | 169 201 662 |
| `almanac.changes` | 4.85× | 3.14× | 88 517 624 | 10 574 821 |
| `almanac.cells` | 15.7× | 15.8× | 287 066 025 | 53 150 087 |
| `almanac.anniversaries` | 1587× | 7918× | 28 946 168 675 | 26 663 596 681 |
| `almanac.next` | 165× | 358× | 3 011 212 807 | 1 204 635 733 |
| `almanac.sentences` | 12.2× | 5.78× | 222 547 316 | 19 479 163 |
| `almanac.publishedReasons` | 32.1× | 51.6× | 584 492 590 | 173 635 794 |
| `almanac.pakshi` | 10.6× | 9.33× | 193 981 132 | 31 427 200 |
| `almanac.sankalpa` | 74.7× | 86.7× | 1 362 064 810 | 291 804 118 |
| `almanac.sankalpaWords` | 74.8× | 84.2× | 1 364 929 553 | 283 585 080 |
| `almanac.ashauch` | 27.8× | 34.7× | 507 640 113 | 116 911 252 |
| `almanac.household` | 36.1× | 73.3× | 657 936 487 | 246 757 582 |
| `almanac.ashauchKin` | 0.00549× | 0.0270× | 100 139 | 90 840 |
| `almanac.ashauchRite` | 0.00344× | 0.0156× | 62 720 | 52 678 |
| `almanac.deathTime` | 91.8× | 181× | 1 674 757 156 | 608 626 829 |
| `almanac.festivalPack` | 0.108× | 0.582× | 1 977 251 | 1 960 180 |
| `almanac.firstYear` | 157× | 320× | 2 858 224 309 | 1 078 579 830 |
| `chart.rashifal` | 9.76× | 8.85× | 178 074 610 | 29 798 047 |
| `research.study` | 3.03× | 2.93× | 55 277 823 | 9 854 471 |
| `chart.mundane` | 2.70× | 0.902× | 49 254 811 | 3 036 422 |
| `chart.relocated` | 1.05× | 0.857× | 19 189 874 | 2 886 855 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 389 517 | 4 075 855 |
| `almanac.skyView` | 0.644× | 1.24× | 11 744 837 | 4 166 308 |
| `almanac.elect` | 281× | 470× | 5 129 411 498 | 1 584 224 522 |
| `almanac.observance` | 202× | 591× | 3 677 882 600 | 1 989 557 069 |
| `almanac.people` | 12.9× | 13.8× | 235 142 769 | 46 600 518 |
| `almanac.skyPose` | 0.738× | 1.75× | 13 454 035 | 5 880 429 |
| `calendar.in_era` | 0.00498× | 0.0157× | 90 834 | 53 037 |
| `settings.explain` | 0.647× | 0.400× | 11 795 415 | 1 346 674 |
| `settings.diff` | 0.125× | 0.675× | 2 283 294 | 2 274 125 |
