# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (6 978 640 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `5219cc69c498`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 255 139 | 3 386 261 |
| `chart.found` | 21.5× | 5.98× | 392 610 260 | 20 247 271 |
| `chart.disagreements` | 3.14× | 10.6× | 57 328 105 | 35 775 377 |
| `almanac.days` | 179× | 366× | 3 267 267 112 | 1 237 889 497 |
| `matching.naam` | 0.0179× | 0.0942× | 326 633 | 318 873 |
| `matching.band` | 1.24× | 0.0228× | 22 588 046 | 77 181 |
| `naming.syllables` | 0.0236× | 0.119× | 430 396 | 403 670 |
| `naming.read` | 0.0242× | 0.0882× | 442 564 | 298 715 |
| `naming.check` | 0.0164× | 0.0804× | 299 475 | 272 290 |
| `naming.of` | 0.00389× | 0.0146× | 71 037 | 49 585 |
| `naming.suggest` | 0.0205× | 0.0997× | 374 937 | 337 557 |
| `time.resolve` | 0.00647× | 0.0305× | 118 041 | 103 202 |
| `geo.resolve` | 0.0616× | 0.0187× | 1 124 657 | 63 354 |
| `geo.search` | 0.0625× | 0.0271× | 1 140 804 | 91 899 |
| `geo.reverse` | 0.0713× | 0.0675× | 1 300 939 | 228 555 |
| `geo.names` | 0.0651× | 0.0342× | 1 188 064 | 115 954 |
| `geo.list` | 0.0653× | 0.0369× | 1 191 921 | 124 846 |
| `time.civil` | 0.00611× | 0.0218× | 111 626 | 73 668 |
| `time.convert` | 0.00163× | 0.00814× | 29 826 | 27 561 |
| `calendar.convert` | 0.00460× | 0.0141× | 83 922 | 47 807 |
| `calendar.add` | 0.00536× | 0.0173× | 97 841 | 58 551 |
| `calendar.between` | 0.00518× | 0.0164× | 94 495 | 55 383 |
| `numerology.profile` | 0.0154× | 0.0805× | 280 869 | 272 452 |
| `almanac.navatara` | 0.0148× | 0.0785× | 269 670 | 265 797 |
| `almanac.aayadi` | 0.0114× | 0.0604× | 207 623 | 204 512 |
| `intl.inWords` | 1.33× | 0.0412× | 24 216 738 | 139 480 |
| `intl.time` | 1.24× | 0.0310× | 22 594 941 | 105 137 |
| `intl.date` | 1.25× | 0.0483× | 22 782 563 | 163 497 |
| `intl.lexicon` | 5.47× | 0.487× | 99 884 862 | 1 648 784 |
| `intl.lookup` | 13.0× | 0.184× | 237 142 041 | 623 264 |
| `intl.dates` | 3.49× | 0.0448× | 63 728 706 | 151 692 |
| `verify.claims` | 86.6× | 84.3× | 1 581 375 187 | 285 344 949 |
| `verify.issue` | 0.189× | 1.00× | 3 455 617 | 3 399 194 |
| `verify.receipt` | 0.274× | 1.41× | 4 998 110 | 4 760 819 |
| `almanac.digest` | 24.7× | 31.4× | 451 602 089 | 106 303 342 |
| `almanac.festival_calendar` | 35.6× | 53.1× | 649 105 087 | 179 971 207 |
| `almanac.changes` | 4.84× | 3.09× | 88 377 299 | 10 449 664 |
| `almanac.cells` | 15.7× | 15.8× | 287 292 601 | 53 372 940 |
| `almanac.anniversaries` | 1587× | 7883× | 28 976 511 195 | 26 693 642 545 |
| `almanac.next` | 167× | 372× | 3 057 135 997 | 1 260 687 503 |
| `almanac.sentences` | 12.2× | 5.77× | 223 253 630 | 19 537 406 |
| `almanac.publishedReasons` | 32.8× | 51.3× | 599 620 557 | 173 862 830 |
| `almanac.pakshi` | 10.6× | 9.30× | 194 049 631 | 31 478 003 |
| `almanac.sankalpa` | 74.6× | 86.2× | 1 362 028 606 | 291 853 892 |
| `almanac.sankalpaWords` | 74.8× | 83.8× | 1 365 643 993 | 283 626 862 |
| `almanac.ashauch` | 27.8× | 34.7× | 508 039 831 | 117 389 651 |
| `almanac.household` | 36.1× | 73.1× | 658 709 500 | 247 532 054 |
| `almanac.ashauchKin` | 0.00548× | 0.0268× | 99 969 | 90 815 |
| `almanac.ashauchRite` | 0.00351× | 0.0156× | 64 136 | 52 828 |
| `almanac.deathTime` | 91.7× | 180× | 1 674 862 492 | 608 667 546 |
| `almanac.festivalPack` | 0.111× | 0.588× | 2 027 817 | 1 992 027 |
| `almanac.firstYear` | 156× | 318× | 2 856 229 811 | 1 076 657 114 |
| `chart.rashifal` | 9.77× | 8.85× | 178 272 807 | 29 961 439 |
| `research.study` | 3.03× | 2.92× | 55 247 700 | 9 903 154 |
| `chart.mundane` | 2.70× | 0.901× | 49 271 729 | 3 051 630 |
| `chart.relocated` | 1.05× | 0.856× | 19 206 435 | 2 899 911 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 393 606 | 4 099 583 |
| `almanac.skyView` | 0.643× | 1.23× | 11 747 173 | 4 178 697 |
| `almanac.elect` | 280× | 463× | 5 112 076 808 | 1 567 182 641 |
| `almanac.observance` | 204× | 603× | 3 724 468 505 | 2 041 638 575 |
| `almanac.people` | 12.9× | 13.7× | 234 989 399 | 46 484 324 |
| `almanac.skyPose` | 0.737× | 1.74× | 13 450 050 | 5 884 412 |
| `calendar.in_era` | 0.00499× | 0.0152× | 91 076 | 51 373 |
| `settings.explain` | 0.664× | 0.404× | 12 120 930 | 1 368 248 |
| `settings.diff` | 0.128× | 0.687× | 2 335 029 | 2 327 910 |
| `almanac.limbAngles` | 0.476× | 0.324× | 8 685 321 | 1 097 085 |
| `almanac.blackouts` | 135× | 143× | 2 458 020 479 | 485 673 290 |
| `almanac.occurrences` | 410× | 1563× | 7 479 303 132 | 5 294 045 343 |
| `intl.render` | 0.608× | 0.0525× | 11 095 469 | 177 825 |
