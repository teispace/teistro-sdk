# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (6 105 832 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `614a4aca398b`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 144 852 | 3 285 166 |
| `chart.found` | 21.6× | 6.09× | 392 370 515 | 20 008 650 |
| `chart.disagreements` | 3.15× | 10.8× | 57 074 450 | 35 488 707 |
| `almanac.days` | 177× | 364× | 3 218 814 375 | 1 194 499 380 |
| `matching.naam` | 0.0180× | 0.0968× | 326 760 | 318 098 |
| `matching.band` | 1.14× | 0.0229× | 20 599 142 | 75 313 |
| `naming.syllables` | 0.0238× | 0.123× | 431 491 | 402 507 |
| `naming.read` | 0.0245× | 0.0906× | 444 851 | 297 555 |
| `naming.check` | 0.0166× | 0.0836× | 300 372 | 274 542 |
| `naming.of` | 0.00386× | 0.0150× | 69 952 | 49 193 |
| `naming.suggest` | 0.0206× | 0.102× | 373 275 | 336 365 |
| `time.resolve` | 0.00656× | 0.0311× | 119 114 | 102 285 |
| `geo.resolve` | 0.0219× | 0.0190× | 396 532 | 62 525 |
| `time.civil` | 0.00615× | 0.0222× | 111 555 | 72 856 |
| `time.convert` | 0.00172× | 0.00853× | 31 299 | 28 008 |
| `calendar.convert` | 0.00462× | 0.0145× | 83 886 | 47 656 |
| `calendar.add` | 0.00543× | 0.0178× | 98 551 | 58 596 |
| `calendar.between` | 0.00529× | 0.0166× | 95 992 | 54 448 |
| `numerology.profile` | 0.0156× | 0.0820× | 282 594 | 269 233 |
| `almanac.navatara` | 0.0149× | 0.0807× | 270 195 | 265 152 |
| `almanac.aayadi` | 0.0115× | 0.0617× | 208 033 | 202 581 |
| `intl.inWords` | 1.22× | 0.0424× | 22 216 572 | 139 258 |
| `intl.time` | 1.14× | 0.0317× | 20 599 368 | 104 137 |
| `intl.date` | 1.01× | 0.0496× | 18 406 924 | 163 065 |
| `intl.lexicon` | 5.19× | 0.491× | 94 225 280 | 1 612 945 |
| `intl.lookup` | 11.4× | 0.184× | 207 191 219 | 604 263 |
| `intl.dates` | 2.95× | 0.0464× | 53 447 348 | 152 292 |
| `verify.claims` | 84.8× | 81.2× | 1 539 501 087 | 266 706 313 |
| `verify.issue` | 0.174× | 0.945× | 3 158 229 | 3 105 068 |
| `verify.receipt` | 0.353× | 1.85× | 6 413 729 | 6 093 714 |
| `almanac.digest` | 14.3× | 21.2× | 259 365 703 | 69 504 375 |
| `almanac.festival_calendar` | 25.0× | 42.3× | 453 830 027 | 139 104 707 |
| `almanac.changes` | 4.86× | 3.13× | 88 236 222 | 10 282 288 |
| `almanac.cells` | 15.1× | 15.1× | 273 777 242 | 49 471 951 |
| `almanac.anniversaries` | 1583× | 8048× | 28 725 027 550 | 26 438 709 567 |
| `almanac.next` | 153× | 332× | 2 771 590 521 | 1 089 401 490 |
| `almanac.sentences` | 11.6× | 5.48× | 209 995 766 | 17 992 493 |
| `almanac.publishedReasons` | 23.6× | 44.1× | 428 059 935 | 144 917 728 |
| `almanac.pakshi` | 10.3× | 9.09× | 187 750 533 | 29 852 235 |
| `almanac.sankalpa` | 74.4× | 83.2× | 1 350 101 425 | 273 192 311 |
| `almanac.sankalpaWords` | 74.5× | 80.7× | 1 350 884 794 | 264 978 588 |
| `almanac.ashauch` | 27.2× | 34.5× | 494 432 610 | 113 288 275 |
| `almanac.household` | 35.3× | 72.9× | 640 978 386 | 239 458 377 |
| `almanac.ashauchKin` | 0.00543× | 0.0275× | 98 497 | 90 351 |
| `almanac.ashauchRite` | 0.00350× | 0.0162× | 63 498 | 53 303 |
| `almanac.deathTime` | 91.1× | 179× | 1 652 875 407 | 588 162 931 |
| `almanac.festivalPack` | 0.106× | 0.580× | 1 921 470 | 1 905 675 |
| `almanac.firstYear` | 156× | 321× | 2 821 799 427 | 1 055 401 843 |
| `chart.rashifal` | 9.46× | 8.62× | 171 590 556 | 28 322 829 |
| `research.study` | 3.03× | 2.93× | 55 009 381 | 9 609 201 |
| `chart.mundane` | 2.71× | 0.893× | 49 143 854 | 2 934 247 |
| `chart.relocated` | 1.05× | 0.853× | 19 098 463 | 2 801 695 |
| `chart.astrocartography` | 1.12× | 1.22× | 20 289 417 | 3 993 537 |
| `almanac.skyView` | 0.645× | 1.25× | 11 697 269 | 4 120 561 |
| `almanac.elect` | 279× | 462× | 5 061 156 290 | 1 516 279 910 |
| `almanac.observance` | 191× | 581× | 3 474 047 273 | 1 907 376 621 |
| `almanac.people` | 12.6× | 13.7× | 228 682 873 | 44 962 005 |
| `almanac.skyPose` | 0.739× | 1.78× | 13 410 720 | 5 848 369 |
