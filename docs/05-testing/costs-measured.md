# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%, or within 0.0100× the unit where that is more. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (26 759 225 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `a404b7a268fa`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, x86_64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 19 077 222 | 4 685 080 |
| `chart.found` | 20.6× | 5.33× | 393 897 212 | 24 955 781 |
| `chart.disagreements` | 3.10× | 8.16× | 59 064 480 | 38 229 230 |
| `almanac.days` | 176× | 286× | 3 360 066 474 | 1 337 800 317 |
| `matching.naam` | 0.0170× | 0.0682× | 324 929 | 319 711 |
| `naming.syllables` | 0.0221× | 0.0858× | 422 153 | 401 918 |
| `naming.read` | 0.0225× | 0.0640× | 428 550 | 299 744 |
| `naming.check` | 0.0157× | 0.0586× | 300 272 | 274 320 |
| `naming.of` | 0.00365× | 0.0107× | 69 584 | 49 921 |
| `naming.suggest` | 0.0192× | 0.0711× | 367 148 | 333 202 |
| `time.resolve` | 0.00647× | 0.0227× | 123 372 | 106 518 |
| `time.civil` | 0.00592× | 0.0161× | 112 939 | 75 444 |
| `time.convert` | 0.00181× | 0.00651× | 34 556 | 30 506 |
| `calendar.convert` | 0.00447× | 0.0103× | 85 360 | 48 200 |
| `calendar.add` | 0.00546× | 0.0135× | 104 209 | 63 316 |
| `calendar.between` | 0.00531× | 0.0128× | 101 263 | 59 984 |
| `numerology.profile` | 0.0144× | 0.0567× | 274 074 | 265 832 |
| `almanac.navatara` | 0.0138× | 0.0559× | 263 488 | 261 844 |
| `almanac.aayadi` | 0.0106× | 0.0431× | 202 902 | 201 927 |
| `intl.inWords` | 0.0642× | 0.0304× | 1 225 655 | 142 513 |
| `intl.time` | 0.00686× | 0.0223× | 130 870 | 104 372 |
| `intl.date` | 0.0146× | 0.0341× | 278 502 | 159 849 |
| `intl.lexicon` | 2.34× | 0.271× | 44 619 446 | 1 271 042 |
| `intl.lookup` | 3.86× | 0.106× | 73 697 266 | 498 377 |
| `intl.dates` | 0.301× | 0.0325× | 5 738 508 | 152 347 |
| `verify.claims` | 74.9× | 62.7× | 1 428 273 059 | 293 659 836 |
| `verify.issue` | 0.164× | 0.661× | 3 122 391 | 3 097 128 |
| `verify.receipt` | 0.335× | 1.32× | 6 399 069 | 6 174 274 |
| `almanac.digest` | 15.2× | 25.1× | 290 253 065 | 117 484 981 |
| `almanac.festival_calendar` | 26.4× | 44.2× | 504 167 763 | 207 308 500 |
| `almanac.changes` | 4.83× | 3.77× | 92 187 553 | 17 665 176 |
| `almanac.anniversaries` | 1685× | 6373× | 32 144 937 703 | 29 856 021 555 |
| `almanac.next` | 163× | 304× | 3 100 924 343 | 1 422 079 349 |
| `almanac.sentences` | 10.4× | 6.66× | 198 599 389 | 31 193 864 |
| `almanac.publishedReasons` | 25.1× | 43.2× | 479 222 619 | 202 385 756 |
| `almanac.pakshi` | 10.6× | 10.6× | 201 893 851 | 49 493 445 |
| `almanac.sankalpa` | 72.0× | 64.3× | 1 372 767 621 | 301 079 986 |
| `almanac.ashauch` | 27.9× | 33.7× | 533 115 294 | 157 668 282 |
| `almanac.household` | 38.1× | 71.1× | 727 230 075 | 333 044 009 |
| `almanac.ashauchKin` | 0.00530× | 0.0192× | 101 172 | 89 966 |
| `almanac.ashauchRite` | 0.00325× | 0.0111× | 61 990 | 52 171 |
| `almanac.deathTime` | 89.8× | 141× | 1 713 048 708 | 661 401 586 |
| `almanac.festivalPack` | 0.0769× | 0.328× | 1 466 194 | 1 535 870 |
| `almanac.firstYear` | 162× | 284× | 3 095 249 566 | 1 331 739 872 |
| `chart.rashifal` | 9.60× | 9.30× | 183 126 736 | 43 575 847 |
| `research.study` | 3.08× | 3.18× | 58 839 344 | 14 921 696 |
| `chart.mundane` | 2.63× | 0.995× | 50 085 648 | 4 662 551 |
| `chart.relocated` | 1.05× | 0.896× | 20 098 793 | 4 196 980 |
| `chart.astrocartography` | 1.11× | 1.14× | 21 221 795 | 5 329 990 |
| `almanac.skyView` | 0.620× | 0.921× | 11 823 360 | 4 312 833 |
| `almanac.elect` | 301× | 471× | 5 747 424 752 | 2 208 854 646 |
| `almanac.observance` | 205× | 500× | 3 905 141 922 | 2 342 071 751 |
| `almanac.people` | 12.6× | 13.0× | 240 308 841 | 60 777 855 |
