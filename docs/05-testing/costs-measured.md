# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (46 827 249 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `a3479b7315a8`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 18 733 700 | 4 340 696 |
| `chart.found` | 20.9× | 5.30× | 391 951 082 | 23 017 714 |
| `chart.disagreements` | 3.12× | 8.67× | 58 431 379 | 37 619 937 |
| `almanac.days` | 175× | 290× | 3 281 064 766 | 1 259 073 156 |
| `matching.naam` | 0.0176× | 0.0747× | 330 042 | 324 252 |
| `matching.band` | 0.00690× | 0.0176× | 129 315 | 76 479 |
| `naming.syllables` | 0.0230× | 0.0936× | 431 425 | 406 497 |
| `naming.read` | 0.0235× | 0.0699× | 440 005 | 303 412 |
| `naming.check` | 0.0162× | 0.0639× | 304 365 | 277 250 |
| `naming.of` | 0.00391× | 0.0115× | 73 280 | 50 071 |
| `naming.suggest` | 0.0196× | 0.0780× | 367 196 | 338 472 |
| `time.resolve` | 0.00657× | 0.0246× | 123 114 | 106 756 |
| `time.civil` | 0.00604× | 0.0176× | 113 154 | 76 358 |
| `time.convert` | 0.00179× | 0.00698× | 33 542 | 30 295 |
| `calendar.convert` | 0.00475× | 0.0114× | 88 931 | 49 397 |
| `calendar.add` | 0.00556× | 0.0148× | 104 089 | 64 031 |
| `calendar.between` | 0.00561× | 0.0139× | 105 073 | 60 143 |
| `numerology.profile` | 0.0150× | 0.0623× | 280 356 | 270 321 |
| `almanac.navatara` | 0.0145× | 0.0612× | 270 715 | 265 494 |
| `almanac.aayadi` | 0.0111× | 0.0471× | 208 178 | 204 250 |
| `intl.inWords` | 0.0933× | 0.0327× | 1 748 769 | 141 955 |
| `intl.time` | 0.00703× | 0.0248× | 131 645 | 107 639 |
| `intl.date` | 0.0154× | 0.0384× | 287 733 | 166 828 |
| `intl.lexicon` | 3.38× | 0.357× | 63 304 416 | 1 547 523 |
| `intl.lookup` | 6.92× | 0.134× | 129 688 164 | 581 321 |
| `intl.dates` | 0.327× | 0.0354× | 6 126 177 | 153 769 |
| `verify.claims` | 78.5× | 64.4× | 1 469 937 939 | 279 493 432 |
| `verify.issue` | 0.168× | 0.716× | 3 140 450 | 3 108 019 |
| `verify.receipt` | 0.342× | 1.42× | 6 411 665 | 6 157 380 |
| `almanac.digest` | 14.3× | 21.7× | 266 982 175 | 94 230 735 |
| `almanac.festival_calendar` | 24.7× | 38.1× | 462 269 549 | 165 430 988 |
| `almanac.changes` | 4.72× | 3.20× | 88 406 094 | 13 876 122 |
| `almanac.cells` | 15.0× | 14.6× | 280 500 379 | 63 302 955 |
| `almanac.anniversaries` | 1547× | 6151× | 28 989 913 691 | 26 701 561 422 |
| `almanac.next` | 150× | 259× | 2 801 710 043 | 1 122 519 540 |
| `almanac.sentences` | 10.3× | 5.89× | 192 932 720 | 25 553 074 |
| `almanac.publishedReasons` | 23.5× | 37.8× | 440 776 542 | 163 945 391 |
| `almanac.pakshi` | 10.4× | 9.58× | 193 971 774 | 41 581 522 |
| `almanac.sankalpa` | 72.5× | 66.0× | 1 358 063 617 | 286 356 508 |
| `almanac.sankalpaWords` | 71.5× | 64.0× | 1 339 585 269 | 277 876 030 |
| `almanac.ashauch` | 26.7× | 28.8× | 500 285 543 | 124 816 221 |
| `almanac.household` | 35.1× | 60.8× | 658 205 277 | 264 003 661 |
| `almanac.ashauchKin` | 0.00546× | 0.0205× | 102 361 | 89 048 |
| `almanac.ashauchRite` | 0.00347× | 0.0120× | 64 988 | 52 293 |
| `almanac.deathTime` | 89.9× | 146× | 1 684 952 685 | 633 239 643 |
| `almanac.festivalPack` | 0.0935× | 0.422× | 1 750 981 | 1 833 811 |
| `almanac.firstYear` | 150× | 243× | 2 817 391 454 | 1 053 788 550 |
| `chart.rashifal` | 9.41× | 8.46× | 176 194 149 | 36 715 443 |
| `research.study` | 3.07× | 3.13× | 57 496 716 | 13 570 308 |
| `chart.mundane` | 2.64× | 0.946× | 49 528 310 | 4 107 262 |
| `chart.relocated` | 1.05× | 0.887× | 19 749 907 | 3 848 106 |
| `chart.astrocartography` | 1.11× | 1.15× | 20 880 159 | 4 988 028 |
| `almanac.skyView` | 0.629× | 0.984× | 11 782 950 | 4 270 120 |
| `almanac.elect` | 288× | 430× | 5 402 918 262 | 1 864 887 204 |
| `almanac.observance` | 183× | 428× | 3 419 228 513 | 1 856 203 210 |
| `almanac.people` | 12.5× | 12.4× | 233 448 187 | 53 966 915 |
