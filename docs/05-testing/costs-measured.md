# What each entry point costs, measured

Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count to it within 10%. Do not edit.

Each entry point is a record, the name the C boundary, every binding and the agent server's tools share, answered on the example request of `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the entry point's first answer on a fresh context, what a one-off call pays, the ephemeris's segments decoded for it included; **again** is the same request a second time on that context, what a consumer that keeps its context pays per call. Both are instructions above the context alone (24 356 523 to build it), so they are counts, not times: a relative cost is the same on a phone and a server where a time is not, and an entry point's figure says how its work compares, not how long it takes on any machine. The engine's records (`engine.*`) answer from the consumer's own engine and are not priced.

Counted at `335943617b96`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, aarch64-linux.

| entry point | first call | again | instructions, first call | instructions, again |
|---|---:|---:|---:|---:|
| `chart.found` (the unit) | 1.00× | 1.00× | 19 596 231 | 3 883 077 |
| `chart.found` | 21.4× | 5.91× | 420 199 265 | 22 936 218 |
| `almanac.days` | 179× | 355× | 3 498 247 768 | 1 378 895 505 |
| `matching.naam` | 0.0159× | 0.0725× | 311 850 | 281 613 |
| `naming.syllables` | 0.0216× | 0.106× | 422 985 | 412 862 |
| `naming.read` | 0.0221× | 0.0709× | 432 106 | 275 266 |
| `naming.check` | 0.0149× | 0.0644× | 292 794 | 249 881 |
| `naming.of` | 0.00362× | 0.0132× | 70 895 | 51 337 |
| `naming.suggest` | 0.0175× | 0.0810× | 343 313 | 314 345 |
| `time.resolve` | 0.00609× | 0.0265× | 119 248 | 103 011 |
| `time.civil` | 0.00523× | 0.0190× | 102 551 | 73 959 |
| `time.convert` | 0.00164× | 0.00827× | 32 210 | 32 131 |
| `calendar.convert` | 0.00390× | 0.0130× | 76 383 | 50 638 |
| `calendar.add` | 0.00482× | 0.0166× | 94 433 | 64 620 |
| `calendar.between` | 0.00453× | 0.0157× | 88 814 | 60 964 |
| `numerology.profile` | 0.0140× | 0.0630× | 273 576 | 244 535 |
| `almanac.navatara` | 0.0129× | 0.0688× | 253 504 | 267 072 |
| `intl.inWords` | 0.0633× | 0.0382× | 1 239 788 | 148 427 |
| `intl.time` | 0.00678× | 0.0284× | 132 907 | 110 247 |
| `intl.date` | 0.0143× | 0.0427× | 280 009 | 165 971 |
| `almanac.digest` | 15.4× | 30.9× | 302 298 736 | 119 880 163 |
| `almanac.festival_calendar` | 27.0× | 55.9× | 528 976 831 | 216 870 612 |
| `almanac.changes` | 4.98× | 4.80× | 97 535 309 | 18 652 613 |
| `almanac.anniversaries` | 1734× | 8155× | 33 973 585 041 | 31 665 921 191 |
| `almanac.next` | 167× | 391× | 3 271 097 412 | 1 518 159 218 |
| `almanac.sentences` | 10.7× | 8.23× | 208 845 772 | 31 975 760 |
| `almanac.publishedReasons` | 25.7× | 54.7× | 503 444 750 | 212 559 053 |
| `almanac.pakshi` | 10.7× | 12.5× | 210 348 615 | 48 687 654 |
| `almanac.sankalpa` | 74.0× | 83.2× | 1 451 082 242 | 322 894 892 |
| `almanac.ashauch` | 28.7× | 43.2× | 562 041 653 | 167 879 092 |
| `almanac.household` | 39.2× | 91.2× | 767 697 284 | 354 073 748 |
| `almanac.ashauchKin` | 0.00587× | 0.0278× | 114 944 | 108 022 |
| `almanac.ashauchRite` | 0.00329× | 0.0160× | 64 398 | 62 095 |
| `almanac.deathTime` | 92.7× | 183× | 1 816 344 041 | 709 981 400 |
| `almanac.festivalPack` | 0.0793× | 0.418× | 1 554 031 | 1 622 744 |
| `almanac.firstYear` | 166× | 366× | 3 261 697 983 | 1 420 927 685 |
| `chart.rashifal` | 9.61× | 10.6× | 188 370 206 | 41 038 361 |
| `research.study` | 3.03× | 3.29× | 59 448 563 | 12 782 438 |
