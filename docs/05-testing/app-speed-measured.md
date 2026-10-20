# What an app pays

Status: `generated` by `cargo xtask app-speed` from `docs/05-testing/app-speed/*.json`, gated by `check-app-speed`. Do not edit. Each record is written by `teistro-ephemeris-teimeris-app-speed` (`adapters/ephemeris-teimeris/rust`), which times the calls an almanac or chart screen makes through the Rust façade, each on a fresh context so that the call memo starts empty.

The **budget** is the performance page's (`02-architecture/09-performance-architecture.md`), written for a native provider: the gate fails a median past 2× its budget under either setup over Teimeris, because wall time is noisy and a tight ceiling would fail on a busy runner rather than on a regression. The **target** is the faster figure a consumer's app needs on a low-end phone, scaled to a laptop core, and is reported, not gated. The built-in ephemeris needs no engine and is held to accuracy, not to the native budget; its ratio to Teimeris is the column on the right.

```text
cd adapters/ephemeris-teimeris/rust
cargo run --release --features app-speed \
  --bin teistro-ephemeris-teimeris-app-speed \
  > ../../../docs/05-testing/app-speed/desktop-$(uname -s)-$(uname -m).json
cargo xtask app-speed
```

Every gated median is inside 2× its budget.

## `android-arm64-android-emulator-sdk-gphone64-arm64`

Android emulator, `sdk_gphone64_arm64` (Android 15, API 35, aarch64), commit `9fc38bc1`, Teimeris 0.1.0, its data built in. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 0.018 | 0.022 | 0.015 | 0.8× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.84 | 0.871 | 40.7 | 22.1× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 56.5 | 30.9 | 1333 | 23.6× |
| year (a year of almanac days) | — | — | 521 | 284 | 12184 | 23.4× |
| festivals (a year of the Nepal pack's festivals) | — | 80.0 ms | 485 | 234 | 12302 | 25.3× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 280 | 115 | 3994 | 14.3× |
| chart (a birth chart with its Vimshottari to depth 3) | 10.0 ms | — | 1.39 | 1.39 | 2.66 | 1.9× |
| matching (Ashta Koota between two cast charts) | — | — | 0.023 | 0.023 | 0.023 | 1.0× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 0.017 | 0.024 |
| drik-teimeris | day | 21 | 1.79 | 2.02 |
| drik-teimeris | month | 21 | 55.4 | 59.5 |
| drik-teimeris | year | 8 | 515 | 531 |
| drik-teimeris | festivals | 9 | 479 | 502 |
| drik-teimeris | muhurta | 15 | 276 | 286 |
| drik-teimeris | chart | 21 | 1.36 | 1.55 |
| drik-teimeris | matching | 21 | 0.023 | 0.065 |
| committee-teimeris | open | 21 | 0.019 | 0.036 |
| committee-teimeris | day | 21 | 0.858 | 0.932 |
| committee-teimeris | month | 21 | 30.6 | 33.6 |
| committee-teimeris | year | 15 | 278 | 290 |
| committee-teimeris | festivals | 17 | 233 | 246 |
| committee-teimeris | muhurta | 21 | 113 | 121 |
| committee-teimeris | chart | 21 | 1.35 | 1.50 |
| committee-teimeris | matching | 21 | 0.023 | 0.031 |
| drik-builtin | open | 21 | 0.013 | 0.017 |
| drik-builtin | day | 21 | 40.4 | 42.3 |
| drik-builtin | month | 4 | 1326 | 1339 |
| drik-builtin | year | 3 | 12173 | 12198 |
| drik-builtin | festivals | 3 | 12283 | 12388 |
| drik-builtin | muhurta | 3 | 3978 | 4021 |
| drik-builtin | chart | 21 | 2.60 | 2.77 |
| drik-builtin | matching | 21 | 0.023 | 0.029 |

## `desktop-Darwin-arm64`

Apple M2 Pro (Darwin 27.0.0, aarch64), commit `f1258970`, Teimeris 0.1.0, its data read from files. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 4.34 | 4.47 | 0.013 | 0.0× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.98 | 1.00 | 45.4 | 23.0× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 60.4 | 34.1 | 1502 | 24.9× |
| year (a year of almanac days) | — | — | 556 | 296 | 15705 | 28.3× |
| festivals (a year of the Nepal pack's festivals) | — | 80.0 ms | 527 | 256 | 14681 | 27.9× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 349 | 126 | 4957 | 14.2× |
| chart (a birth chart with its Vimshottari to depth 3) | 10.0 ms | — | 1.35 | 1.33 | 2.66 | 2.0× |
| matching (Ashta Koota between two cast charts) | — | — | 0.014 | 0.014 | 0.015 | 1.1× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 4.14 | 4.84 |
| drik-teimeris | day | 21 | 1.86 | 2.11 |
| drik-teimeris | month | 21 | 59.6 | 62.8 |
| drik-teimeris | year | 7 | 546 | 745 |
| drik-teimeris | festivals | 8 | 518 | 538 |
| drik-teimeris | muhurta | 11 | 314 | 459 |
| drik-teimeris | chart | 21 | 1.27 | 1.82 |
| drik-teimeris | matching | 21 | 0.013 | 0.025 |
| committee-teimeris | open | 21 | 4.12 | 4.99 |
| committee-teimeris | day | 21 | 0.900 | 1.16 |
| committee-teimeris | month | 21 | 33.0 | 37.6 |
| committee-teimeris | year | 13 | 290 | 331 |
| committee-teimeris | festivals | 15 | 252 | 316 |
| committee-teimeris | muhurta | 21 | 123 | 144 |
| committee-teimeris | chart | 21 | 1.25 | 1.52 |
| committee-teimeris | matching | 21 | 0.012 | 0.020 |
| drik-builtin | open | 21 | 0.013 | 0.016 |
| drik-builtin | day | 21 | 44.6 | 60.5 |
| drik-builtin | month | 3 | 1502 | 1518 |
| drik-builtin | year | 3 | 14393 | 16682 |
| drik-builtin | festivals | 3 | 14167 | 15044 |
| drik-builtin | muhurta | 3 | 4559 | 5545 |
| drik-builtin | chart | 21 | 2.56 | 2.85 |
| drik-builtin | matching | 21 | 0.012 | 0.027 |
