# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (46 654 543 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `8b1afe408291`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 19 086 945 | 4 694 215 |
| `chart.found` | 20.6× | 5.34× | 394 000 616 | 25 066 568 |
| `chart.disagreements` | 3.10× | 8.16× | 59 118 484 | 38 310 531 |
| `almanac.days` | 176× | 285× | 3 360 019 788 | 1 338 337 524 |
| `matching.naam` | 0.0171× | 0.0689× | 326 709 | 323 620 |
| `matching.band` | 0.00673× | 0.0163× | 128 519 | 76 316 |
| `naming.syllables` | 0.0224× | 0.0865× | 428 429 | 406 055 |
| `naming.read` | 0.0228× | 0.0647× | 435 578 | 303 695 |
| `naming.check` | 0.0157× | 0.0590× | 299 040 | 276 842 |
| `naming.of` | 0.00374× | 0.0106× | 71 341 | 49 743 |
| `naming.suggest` | 0.0191× | 0.0721× | 364 089 | 338 305 |
| `time.resolve` | 0.00620× | 0.0229× | 118 384 | 107 461 |
| `time.civil` | 0.00582× | 0.0162× | 110 996 | 75 853 |
| `time.convert` | 0.00164× | 0.00641× | 31 216 | 30 080 |
| `calendar.convert` | 0.00448× | 0.0106× | 85 592 | 49 628 |
| `calendar.add` | 0.00520× | 0.0136× | 99 196 | 63 761 |
| `calendar.between` | 0.00517× | 0.0129× | 98 679 | 60 724 |
| `numerology.profile` | 0.0143× | 0.0576× | 273 099 | 270 588 |
| `almanac.navatara` | 0.0140× | 0.0565× | 266 644 | 265 305 |
| `almanac.aayadi` | 0.0109× | 0.0434× | 207 403 | 203 951 |
| `intl.inWords` | 0.0913× | 0.0303× | 1 743 111 | 142 223 |
| `intl.time` | 0.00693× | 0.0228× | 132 203 | 107 220 |
| `intl.date` | 0.0149× | 0.0357× | 284 967 | 167 455 |
| `intl.lexicon` | 3.30× | 0.329× | 63 043 076 | 1 546 251 |
| `intl.lookup` | 6.76× | 0.124× | 129 066 053 | 580 366 |
| `intl.dates` | 0.321× | 0.0328× | 6 125 636 | 153 747 |
| `verify.claims` | 77.7× | 62.6× | 1 483 860 325 | 293 922 278 |
| `verify.issue` | 0.164× | 0.662× | 3 135 700 | 3 107 746 |
| `verify.receipt` | 0.336× | 1.31× | 6 407 040 | 6 158 725 |
| `almanac.digest` | 15.2× | 25.0× | 290 281 725 | 117 510 378 |
| `almanac.festival_calendar` | 26.4× | 44.2× | 504 110 434 | 207 264 067 |
| `almanac.changes` | 4.83× | 3.76× | 92 191 552 | 17 662 667 |
| `almanac.anniversaries` | 1684× | 6360× | 32 144 427 377 | 29 856 430 140 |
| `almanac.next` | 162× | 303× | 3 101 235 750 | 1 422 153 916 |
| `almanac.sentences` | 10.4× | 6.66× | 198 653 179 | 31 279 920 |
| `almanac.publishedReasons` | 25.1× | 43.1× | 479 242 200 | 202 409 252 |
| `almanac.pakshi` | 10.6× | 10.6× | 201 942 372 | 49 557 283 |
| `almanac.sankalpa` | 71.9× | 64.1× | 1 372 771 961 | 301 067 039 |
| `almanac.sankalpaWords` | 70.9× | 62.3× | 1 354 021 247 | 292 313 325 |
| `almanac.ashauch` | 27.9× | 33.6× | 533 108 509 | 157 646 064 |
| `almanac.household` | 38.1× | 70.9× | 727 199 019 | 332 976 854 |
| `almanac.ashauchKin` | 0.00507× | 0.0191× | 96 732 | 89 538 |
| `almanac.ashauchRite` | 0.00319× | 0.0113× | 60 941 | 52 833 |
| `almanac.deathTime` | 89.7× | 141× | 1 713 010 531 | 661 301 530 |
| `almanac.festivalPack` | 0.0917× | 0.391× | 1 749 394 | 1 833 480 |
| `almanac.firstYear` | 162× | 284× | 3 095 168 232 | 1 331 490 059 |
| `chart.rashifal` | 9.59× | 9.29× | 183 100 262 | 43 624 340 |
| `research.study` | 3.08× | 3.18× | 58 846 987 | 14 924 644 |
| `chart.mundane` | 2.62× | 0.994× | 50 083 537 | 4 666 647 |
| `chart.relocated` | 1.05× | 0.894× | 20 103 167 | 4 197 675 |
| `chart.astrocartography` | 1.11× | 1.14× | 21 223 494 | 5 337 170 |
| `almanac.skyView` | 0.620× | 0.920× | 11 829 161 | 4 319 661 |
| `almanac.elect` | 301× | 471× | 5 746 559 837 | 2 208 888 200 |
| `almanac.observance` | 205× | 499× | 3 906 645 466 | 2 342 587 930 |
| `almanac.people` | 12.6× | 13.0× | 240 270 653 | 60 804 546 |
