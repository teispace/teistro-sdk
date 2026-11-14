# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (6 027 646 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `0ce9e2476dc4`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 683 750 | 4 287 534 |
| `chart.found` | 21.0× | 5.31× | 391 709 875 | 22 751 016 |
| `chart.disagreements` | 3.13× | 8.76× | 58 440 936 | 37 538 771 |
| `almanac.days` | 175× | 291× | 3 271 188 952 | 1 248 494 261 |
| `matching.naam` | 0.0174× | 0.0740× | 325 201 | 317 297 |
| `matching.band` | 1.07× | 0.0176× | 20 046 294 | 75 582 |
| `naming.syllables` | 0.0230× | 0.0936× | 430 456 | 401 179 |
| `naming.read` | 0.0238× | 0.0694× | 443 889 | 297 437 |
| `naming.check` | 0.0161× | 0.0637× | 300 400 | 273 299 |
| `naming.of` | 0.00374× | 0.0114× | 69 853 | 48 783 |
| `naming.suggest` | 0.0200× | 0.0786× | 373 621 | 336 879 |
| `time.resolve` | 0.00640× | 0.0239× | 119 554 | 102 476 |
| `geo.resolve` | 0.0213× | 0.0146× | 398 817 | 62 637 |
| `time.civil` | 0.00599× | 0.0169× | 111 914 | 72 657 |
| `time.convert` | 0.00167× | 0.00648× | 31 273 | 27 785 |
| `calendar.convert` | 0.00445× | 0.0113× | 83 200 | 48 439 |
| `calendar.add` | 0.00530× | 0.0136× | 98 971 | 58 447 |
| `calendar.between` | 0.00514× | 0.0129× | 96 008 | 55 363 |
| `numerology.profile` | 0.0151× | 0.0627× | 281 947 | 268 910 |
| `almanac.navatara` | 0.0144× | 0.0615× | 269 189 | 263 576 |
| `almanac.aayadi` | 0.0111× | 0.0471× | 207 745 | 201 737 |
| `intl.inWords` | 1.16× | 0.0326× | 21 665 006 | 139 794 |
| `intl.time` | 1.07× | 0.0243× | 20 049 349 | 104 274 |
| `intl.date` | 0.956× | 0.0383× | 17 864 766 | 164 132 |
| `intl.lexicon` | 4.89× | 0.372× | 91 363 395 | 1 594 170 |
| `intl.lookup` | 10.7× | 0.139× | 199 671 459 | 594 229 |
| `intl.dates` | 2.79× | 0.0357× | 52 082 603 | 152 936 |
| `verify.claims` | 82.4× | 65.1× | 1 539 410 791 | 279 021 612 |
| `verify.issue` | 0.168× | 0.721× | 3 144 104 | 3 093 153 |
| `verify.receipt` | 0.344× | 1.42× | 6 428 811 | 6 107 838 |
| `almanac.digest` | 14.7× | 21.7× | 275 341 001 | 93 052 089 |
| `almanac.festival_calendar` | 25.2× | 38.2× | 470 058 919 | 163 795 624 |
| `almanac.changes` | 4.74× | 3.25× | 88 503 402 | 13 944 220 |
| `almanac.cells` | 15.0× | 14.6× | 279 784 533 | 62 586 138 |
| `almanac.anniversaries` | 1547× | 6208× | 28 908 233 160 | 26 619 050 399 |
| `almanac.next` | 150× | 261× | 2 798 197 682 | 1 119 426 349 |
| `almanac.sentences` | 11.4× | 5.87× | 212 449 072 | 25 160 569 |
| `almanac.publishedReasons` | 23.5× | 38.0× | 439 816 855 | 162 894 615 |
| `almanac.pakshi` | 10.3× | 9.46× | 192 932 443 | 40 542 710 |
| `almanac.sankalpa` | 72.7× | 66.7× | 1 357 622 767 | 285 889 531 |
| `almanac.sankalpaWords` | 72.7× | 64.7× | 1 357 873 855 | 277 374 998 |
| `almanac.ashauch` | 26.7× | 28.9× | 499 364 054 | 123 919 835 |
| `almanac.household` | 35.1× | 61.1× | 656 298 482 | 262 157 466 |
| `almanac.ashauchKin` | 0.00528× | 0.0211× | 98 696 | 90 335 |
| `almanac.ashauchRite` | 0.00341× | 0.0124× | 63 624 | 53 102 |
| `almanac.deathTime` | 90.1× | 148× | 1 684 123 730 | 632 442 909 |
| `almanac.festivalPack` | 0.102× | 0.437× | 1 897 234 | 1 875 469 |
| `almanac.firstYear` | 151× | 245× | 2 813 324 158 | 1 049 358 768 |
| `chart.rashifal` | 9.42× | 8.55× | 176 066 061 | 36 665 002 |
| `research.study` | 3.07× | 3.13× | 57 369 145 | 13 402 889 |
| `chart.mundane` | 2.65× | 0.949× | 49 491 853 | 4 068 309 |
| `chart.relocated` | 1.05× | 0.885× | 19 708 936 | 3 793 539 |
| `chart.astrocartography` | 1.12× | 1.15× | 20 850 745 | 4 935 461 |
| `almanac.skyView` | 0.631× | 0.994× | 11 794 787 | 4 259 786 |
| `almanac.elect` | 287× | 425× | 5 360 998 443 | 1 823 614 130 |
| `almanac.observance` | 183× | 431× | 3 410 976 275 | 1 846 851 556 |
| `almanac.people` | 12.5× | 12.6× | 233 367 008 | 53 862 878 |
| `almanac.skyPose` | 0.720× | 1.38× | 13 461 243 | 5 937 852 |
