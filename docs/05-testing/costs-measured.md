# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (5 824 237 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `f0902f01def0`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 684 886 | 4 287 384 |
| `chart.found` | 21.0× | 5.31× | 391 715 703 | 22 754 287 |
| `chart.disagreements` | 3.13× | 8.75× | 58 398 197 | 37 496 521 |
| `almanac.days` | 175× | 291× | 3 271 151 949 | 1 248 453 401 |
| `matching.naam` | 0.0174× | 0.0740× | 325 517 | 317 075 |
| `matching.band` | 1.00× | 0.0175× | 18 777 971 | 75 035 |
| `naming.syllables` | 0.0230× | 0.0936× | 430 511 | 401 379 |
| `naming.read` | 0.0238× | 0.0694× | 444 006 | 297 449 |
| `naming.check` | 0.0161× | 0.0639× | 300 142 | 273 996 |
| `naming.of` | 0.00375× | 0.0114× | 69 996 | 48 858 |
| `naming.suggest` | 0.0200× | 0.0785× | 373 901 | 336 599 |
| `time.resolve` | 0.00641× | 0.0239× | 119 730 | 102 660 |
| `geo.resolve` | 0.0214× | 0.0146× | 399 200 | 62 721 |
| `time.civil` | 0.00596× | 0.0172× | 111 284 | 73 701 |
| `time.convert` | 0.00168× | 0.00660× | 31 478 | 28 315 |
| `calendar.convert` | 0.00450× | 0.0111× | 84 153 | 47 620 |
| `calendar.add` | 0.00530× | 0.0137× | 99 005 | 58 915 |
| `calendar.between` | 0.00516× | 0.0129× | 96 413 | 55 226 |
| `numerology.profile` | 0.0151× | 0.0625× | 282 453 | 268 019 |
| `almanac.navatara` | 0.0144× | 0.0617× | 269 144 | 264 640 |
| `almanac.aayadi` | 0.0111× | 0.0472× | 207 398 | 202 536 |
| `intl.inWords` | 1.09× | 0.0326× | 20 396 360 | 139 591 |
| `intl.time` | 1.01× | 0.0244× | 18 779 029 | 104 416 |
| `intl.date` | 0.889× | 0.0383× | 16 611 845 | 164 180 |
| `intl.lexicon` | 4.48× | 0.361× | 83 708 349 | 1 546 785 |
| `intl.lookup` | 9.52× | 0.133× | 177 837 336 | 571 180 |
| `intl.dates` | 2.61× | 0.0357× | 48 851 922 | 153 013 |
| `verify.claims` | 81.2× | 65.1× | 1 517 640 796 | 278 950 339 |
| `verify.issue` | 0.168× | 0.721× | 3 144 426 | 3 092 754 |
| `verify.receipt` | 0.344× | 1.42× | 6 429 176 | 6 107 629 |
| `almanac.digest` | 14.7× | 21.7× | 274 760 667 | 93 051 993 |
| `almanac.festival_calendar` | 25.1× | 38.2× | 469 483 417 | 163 797 385 |
| `almanac.changes` | 4.74× | 3.25× | 88 503 482 | 13 944 390 |
| `almanac.cells` | 15.0× | 14.6× | 279 784 980 | 62 586 706 |
| `almanac.anniversaries` | 1547× | 6209× | 28 908 251 692 | 26 619 065 059 |
| `almanac.next` | 150× | 261× | 2 797 501 128 | 1 118 150 372 |
| `almanac.sentences` | 11.3× | 5.87× | 211 174 827 | 25 155 357 |
| `almanac.publishedReasons` | 23.5× | 38.0× | 439 818 446 | 162 895 561 |
| `almanac.pakshi` | 10.3× | 9.46× | 192 929 934 | 40 540 978 |
| `almanac.sankalpa` | 72.7× | 66.7× | 1 357 623 545 | 285 889 943 |
| `almanac.sankalpaWords` | 72.6× | 64.7× | 1 356 616 077 | 277 399 635 |
| `almanac.ashauch` | 26.7× | 28.9× | 499 362 587 | 123 918 387 |
| `almanac.household` | 35.1× | 61.1× | 656 293 471 | 262 152 573 |
| `almanac.ashauchKin` | 0.00530× | 0.0211× | 99 031 | 90 523 |
| `almanac.ashauchRite` | 0.00342× | 0.0124× | 63 971 | 53 136 |
| `almanac.deathTime` | 90.1× | 148× | 1 684 123 993 | 632 443 585 |
| `almanac.festivalPack` | 0.102× | 0.438× | 1 901 271 | 1 877 866 |
| `almanac.firstYear` | 151× | 245× | 2 813 322 593 | 1 049 356 651 |
| `chart.rashifal` | 9.42× | 8.55× | 176 066 232 | 36 664 545 |
| `research.study` | 3.07× | 3.13× | 57 372 577 | 13 405 009 |
| `chart.mundane` | 2.65× | 0.949× | 49 492 961 | 4 068 166 |
| `chart.relocated` | 1.05× | 0.885× | 19 709 406 | 3 793 831 |
| `chart.astrocartography` | 1.12× | 1.15× | 20 851 352 | 4 936 851 |
| `almanac.skyView` | 0.631× | 0.993× | 11 795 395 | 4 258 946 |
| `almanac.elect` | 287× | 425× | 5 361 002 046 | 1 823 617 157 |
| `almanac.observance` | 183× | 431× | 3 410 992 722 | 1 846 867 659 |
| `almanac.people` | 12.5× | 12.6× | 233 324 072 | 53 819 152 |
| `almanac.skyPose` | 0.720× | 1.39× | 13 460 780 | 5 938 812 |
