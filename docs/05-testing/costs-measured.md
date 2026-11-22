# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (6 881 192 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `48be9b1fe079`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 252 491 | 3 382 005 |
| `chart.found` | 21.5× | 5.99× | 392 601 812 | 20 253 039 |
| `chart.disagreements` | 3.14× | 10.6× | 57 299 569 | 35 766 147 |
| `almanac.days` | 179× | 367× | 3 270 288 742 | 1 241 559 171 |
| `matching.naam` | 0.0180× | 0.0943× | 328 818 | 318 951 |
| `matching.band` | 1.22× | 0.0229× | 22 221 644 | 77 297 |
| `naming.syllables` | 0.0237× | 0.120× | 432 714 | 404 695 |
| `naming.read` | 0.0243× | 0.0883× | 443 270 | 298 749 |
| `naming.check` | 0.0165× | 0.0809× | 301 643 | 273 513 |
| `naming.of` | 0.00399× | 0.0147× | 72 820 | 49 781 |
| `naming.suggest` | 0.0205× | 0.100× | 373 352 | 339 422 |
| `time.resolve` | 0.00669× | 0.0305× | 122 082 | 103 239 |
| `geo.resolve` | 0.0618× | 0.0186× | 1 127 145 | 62 964 |
| `geo.search` | 0.0636× | 0.0273× | 1 161 721 | 92 452 |
| `geo.reverse` | 0.0704× | 0.0684× | 1 284 711 | 231 307 |
| `geo.names` | 0.0637× | 0.0345× | 1 162 709 | 116 626 |
| `geo.list` | 0.0640× | 0.0371× | 1 168 321 | 125 426 |
| `time.civil` | 0.00621× | 0.0218× | 113 415 | 73 566 |
| `time.convert` | 0.00176× | 0.00838× | 32 093 | 28 326 |
| `calendar.convert` | 0.00473× | 0.0140× | 86 302 | 47 370 |
| `calendar.add` | 0.00551× | 0.0173× | 100 520 | 58 363 |
| `calendar.between` | 0.00523× | 0.0162× | 95 507 | 54 676 |
| `numerology.profile` | 0.0155× | 0.0804× | 283 084 | 271 760 |
| `almanac.navatara` | 0.0149× | 0.0785× | 271 797 | 265 329 |
| `almanac.aayadi` | 0.0114× | 0.0600× | 207 275 | 202 813 |
| `intl.inWords` | 1.31× | 0.0413× | 23 857 531 | 139 751 |
| `intl.time` | 1.22× | 0.0311× | 22 231 161 | 105 215 |
| `intl.date` | 1.22× | 0.0484× | 22 353 619 | 163 646 |
| `intl.lexicon` | 5.37× | 0.484× | 98 100 740 | 1 637 701 |
| `intl.lookup` | 12.8× | 0.183× | 233 298 791 | 620 065 |
| `intl.dates` | 3.44× | 0.0452× | 62 740 333 | 152 989 |
| `verify.claims` | 86.4× | 84.4× | 1 577 502 905 | 285 444 655 |
| `verify.issue` | 0.188× | 0.998× | 3 422 968 | 3 375 671 |
| `verify.receipt` | 0.273× | 1.40× | 4 983 631 | 4 748 057 |
| `almanac.digest` | 24.7× | 31.4× | 451 392 364 | 106 353 246 |
| `almanac.festival_calendar` | 35.5× | 53.1× | 648 830 360 | 179 748 964 |
| `almanac.changes` | 4.85× | 3.13× | 88 535 281 | 10 601 513 |
| `almanac.cells` | 15.7× | 15.8× | 287 274 088 | 53 353 519 |
| `almanac.anniversaries` | 1587× | 7891× | 28 971 384 042 | 26 688 260 106 |
| `almanac.next` | 168× | 374× | 3 061 911 017 | 1 265 345 705 |
| `almanac.sentences` | 12.2× | 5.78× | 222 897 112 | 19 532 632 |
| `almanac.publishedReasons` | 33.0× | 52.0× | 601 570 819 | 175 814 851 |
| `almanac.pakshi` | 10.6× | 9.31× | 194 042 518 | 31 492 938 |
| `almanac.sankalpa` | 74.6× | 86.3× | 1 362 093 597 | 291 902 469 |
| `almanac.sankalpaWords` | 74.8× | 83.8× | 1 365 253 073 | 283 568 204 |
| `almanac.ashauch` | 27.8× | 34.7× | 507 961 852 | 117 312 105 |
| `almanac.household` | 36.1× | 73.2× | 658 691 998 | 247 637 540 |
| `almanac.ashauchKin` | 0.00552× | 0.0268× | 100 707 | 90 728 |
| `almanac.ashauchRite` | 0.00358× | 0.0160× | 65 287 | 53 953 |
| `almanac.deathTime` | 91.7× | 180× | 1 674 639 320 | 608 562 998 |
| `almanac.festivalPack` | 0.111× | 0.591× | 2 030 443 | 1 998 599 |
| `almanac.firstYear` | 157× | 320× | 2 861 542 076 | 1 081 664 769 |
| `chart.rashifal` | 9.76× | 8.89× | 178 127 026 | 30 074 737 |
| `research.study` | 3.03× | 2.93× | 55 251 494 | 9 903 408 |
| `chart.mundane` | 2.70× | 0.905× | 49 277 479 | 3 060 979 |
| `chart.relocated` | 1.05× | 0.857× | 19 203 007 | 2 898 520 |
| `chart.astrocartography` | 1.12× | 1.21× | 20 386 446 | 4 089 470 |
| `almanac.skyView` | 0.644× | 1.24× | 11 755 960 | 4 187 798 |
| `almanac.elect` | 281× | 469× | 5 130 805 241 | 1 584 631 849 |
| `almanac.observance` | 204× | 606× | 3 732 320 931 | 2 048 692 828 |
| `almanac.people` | 12.9× | 13.8× | 235 297 559 | 46 763 590 |
| `almanac.skyPose` | 0.737× | 1.75× | 13 460 567 | 5 901 816 |
| `calendar.in_era` | 0.00504× | 0.0155× | 92 055 | 52 466 |
| `settings.explain` | 0.655× | 0.403× | 11 956 586 | 1 361 325 |
| `settings.diff` | 0.127× | 0.684× | 2 321 087 | 2 312 486 |
| `almanac.limbAngles` | 0.476× | 0.325× | 8 690 499 | 1 098 325 |
| `almanac.blackouts` | 135× | 144× | 2 458 411 700 | 486 081 576 |
| `almanac.occurrences` | 410× | 1565× | 7 478 487 471 | 5 292 756 325 |
