# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (8 181 080 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `505e7390d561`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 248 341 | 3 378 413 |
| `chart.found` | 21.5× | 5.99× | 392 588 296 | 20 230 298 |
| `chart.disagreements` | 3.14× | 10.6× | 57 253 394 | 35 695 697 |
| `almanac.days` | 179× | 367× | 3 265 840 873 | 1 238 243 724 |
| `matching.naam` | 0.0180× | 0.0948× | 328 058 | 320 180 |
| `matching.band` | 1.65× | 0.0226× | 30 067 617 | 76 200 |
| `naming.syllables` | 0.0235× | 0.119× | 429 011 | 401 836 |
| `naming.read` | 0.0241× | 0.0885× | 439 795 | 299 153 |
| `naming.check` | 0.0165× | 0.0814× | 300 619 | 274 926 |
| `naming.of` | 0.00387× | 0.0142× | 70 600 | 48 089 |
| `naming.suggest` | 0.0201× | 0.0980× | 366 904 | 331 180 |
| `time.resolve` | 0.00646× | 0.0294× | 117 822 | 99 260 |
| `geo.resolve` | 0.0589× | 0.0186× | 1 074 879 | 62 722 |
| `geo.search` | 0.0604× | 0.0283× | 1 102 229 | 95 777 |
| `geo.reverse` | 0.0696× | 0.0719× | 1 269 467 | 243 027 |
| `geo.names` | 0.0619× | 0.0352× | 1 128 787 | 118 757 |
| `geo.place` | 0.0594× | 0.0217× | 1 083 237 | 73 205 |
| `geo.list` | 0.0624× | 0.0389× | 1 138 850 | 131 539 |
| `time.civil` | 0.00610× | 0.0210× | 111 265 | 71 064 |
| `time.convert` | 0.00166× | 0.00770× | 30 242 | 26 013 |
| `calendar.convert` | 0.00460× | 0.0132× | 83 911 | 44 760 |
| `calendar.add` | 0.00526× | 0.0163× | 96 045 | 54 990 |
| `calendar.between` | 0.00501× | 0.0147× | 91 385 | 49 602 |
| `numerology.profile` | 0.0153× | 0.0791× | 279 494 | 267 195 |
| `almanac.navatara` | 0.0148× | 0.0781× | 270 416 | 264 017 |
| `almanac.aayadi` | 0.0113× | 0.0596× | 206 169 | 201 246 |
| `intl.inWords` | 1.74× | 0.0413× | 31 697 284 | 139 425 |
| `intl.time` | 1.65× | 0.0309× | 30 069 967 | 104 401 |
| `intl.date` | 1.63× | 0.0483× | 29 700 350 | 163 087 |
| `intl.lexicon` | 7.72× | 0.552× | 140 792 461 | 1 863 349 |
| `intl.lookup` | 19.2× | 0.216× | 350 979 968 | 728 858 |
| `intl.dates` | 4.49× | 0.0453× | 81 914 930 | 153 143 |
| `verify.claims` | 92.9× | 84.6× | 1 695 423 882 | 285 709 366 |
| `verify.issue` | 0.189× | 1.00× | 3 440 773 | 3 387 705 |
| `verify.receipt` | 0.268× | 1.38× | 4 888 947 | 4 671 189 |
| `almanac.digest` | 25.1× | 31.5× | 457 960 811 | 106 307 844 |
| `almanac.festival_calendar` | 35.9× | 53.2× | 655 504 137 | 179 724 149 |
| `almanac.changes` | 4.84× | 3.09× | 88 372 010 | 10 448 657 |
| `almanac.cells` | 15.7× | 15.8× | 287 257 816 | 53 353 215 |
| `almanac.anniversaries` | 1588× | 7902× | 28 979 120 224 | 26 696 041 602 |
| `almanac.next` | 168× | 374× | 3 061 568 316 | 1 261 937 801 |
| `almanac.sentences` | 12.6× | 5.80× | 230 738 180 | 19 581 773 |
| `almanac.publishedReasons` | 33.0× | 51.5× | 602 682 198 | 173 934 065 |
| `almanac.pakshi` | 10.6× | 9.31× | 194 028 324 | 31 469 560 |
| `almanac.sankalpa` | 74.6× | 86.4× | 1 362 045 898 | 291 968 163 |
| `almanac.sankalpaWords` | 75.2× | 84.0× | 1 372 745 023 | 283 686 443 |
| `almanac.ashauch` | 27.8× | 34.7× | 507 999 429 | 117 344 387 |
| `almanac.household` | 36.1× | 73.3× | 658 827 871 | 247 706 348 |
| `almanac.ashauchKin` | 0.00532× | 0.0257× | 97 009 | 86 718 |
| `almanac.ashauchRite` | 0.00343× | 0.0149× | 62 519 | 50 370 |
| `almanac.deathTime` | 91.8× | 180× | 1 674 744 163 | 608 767 837 |
| `almanac.festivalPack` | 0.121× | 0.648× | 2 211 986 | 2 190 095 |
| `almanac.firstYear` | 157× | 319× | 2 856 405 839 | 1 076 533 958 |
| `chart.rashifal` | 9.77× | 8.84× | 178 304 167 | 29 875 634 |
| `research.study` | 3.03× | 2.91× | 55 242 573 | 9 830 685 |
| `chart.mundane` | 2.70× | 0.901× | 49 263 798 | 3 043 919 |
| `chart.relocated` | 1.05× | 0.858× | 19 198 557 | 2 897 461 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 385 805 | 4 084 912 |
| `almanac.skyView` | 0.643× | 1.23× | 11 731 627 | 4 158 269 |
| `almanac.elect` | 280× | 464× | 5 115 139 563 | 1 567 010 152 |
| `almanac.observance` | 204× | 604× | 3 725 450 409 | 2 041 456 691 |
| `almanac.people` | 12.9× | 13.7× | 234 922 207 | 46 381 301 |
| `almanac.skyPose` | 0.735× | 1.73× | 13 411 406 | 5 848 687 |
| `calendar.in_era` | 0.00492× | 0.0146× | 89 760 | 49 285 |
| `settings.explain` | 0.843× | 0.402× | 15 378 602 | 1 358 923 |
| `settings.diff` | 0.127× | 0.685× | 2 323 576 | 2 314 483 |
| `almanac.limbAngles` | 0.476× | 0.323× | 8 682 797 | 1 089 944 |
| `almanac.blackouts` | 135× | 144× | 2 458 024 050 | 485 659 438 |
| `almanac.occurrences` | 410× | 1568× | 7 483 817 721 | 5 295 765 163 |
| `intl.render` | 0.786× | 0.0512× | 14 352 121 | 173 073 |
