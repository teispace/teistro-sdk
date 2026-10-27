# What an app pays

Status: `generated` by `cargo xtask app-speed` from `docs/05-testing/app-speed/*.json`, gated by `check-app-speed`. Do not edit. Each record is written by `teistro-ephemeris-teimeris-app-speed` (`adapters/ephemeris-teimeris/rust`), which times the calls an almanac or chart screen makes through the Rust façade, each on a fresh context so that the call memo starts empty.

The **budget** is the performance page's (`02-architecture/09-performance-architecture.md`), written for a native provider: the gate fails a median past 2× its budget under either setup over Teimeris, because wall time is noisy and a tight ceiling would fail on a busy runner rather than on a regression. The **target** is the faster figure a consumer's app needs on a low-end phone, scaled to a laptop core, and is reported, not gated. The built-in ephemeris needs no engine and is held to accuracy, not to the native budget; its ratio to Teimeris is the column on the right, which says `within spread` where the two setups' fastest-to-slowest ranges overlap, since no ratio is measured there. A median taken from fewer than 3 runs, or outside its own range, fails the gate.

```text
cd adapters/ephemeris-teimeris/rust
cargo run --release --features app-speed \
  --bin teistro-ephemeris-teimeris-app-speed \
  > ../../../docs/05-testing/app-speed/desktop-$(uname -s)-$(uname -m).json
cargo xtask app-speed
```

Every gated median is inside 2× its budget, and every thread count answers alike and keeps its floor.

## `android-arm64-android-emulator-sdk-gphone64-arm64`

Android emulator, `sdk_gphone64_arm64` (Android 15, API 35, aarch64), commit `cfa50116`, Teimeris 0.1.0, its data built in. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 0.020 | 0.024 | 0.014 | 0.7× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.11 | 0.567 | 2.09 | 1.9× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 17.1 | 10.9 | 32.3 | 1.9× |
| year (a year of almanac days) | 500 ms | — | 159 | 106 | 314 | 2.0× |
| festivals (a year of the Nepal pack's festivals) | 160 ms | 80.0 ms | 58.0 | 24.4 | 119 | 2.1× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 138 | 110 | 474 | 3.4× |
| chart (a birth chart with its Vimshottari to depth 3) | 2.00 ms | — | 0.571 | 0.564 | 0.957 | 1.7× |
| parity-chart (the full parity chart: every section and varga, the baseline's parity-tier dashas to depth 3, every shipped rule) | 10.0 ms | — | 11.3 | 11.0 | 12.8 | 1.1× |
| widest-chart (the parity chart with every dasha a chart gives) | 25.0 ms | — | 21.7 | 20.8 | 23.1 | within spread |
| matching (Ashta Koota between two cast charts) | 1.00 ms | — | 0.024 | 0.024 | 0.024 | within spread |
| rectify (a day-long window rectified at the default one-minute seed) | 1000 ms | — | 34.2 | 23.0 | 46.3 | 1.4× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 0.019 | 0.046 |
| drik-teimeris | day | 21 | 1.07 | 1.47 |
| drik-teimeris | month | 21 | 16.7 | 17.6 |
| drik-teimeris | year | 21 | 155 | 197 |
| drik-teimeris | festivals | 21 | 56.9 | 65.4 |
| drik-teimeris | muhurta | 21 | 136 | 165 |
| drik-teimeris | chart | 21 | 0.550 | 0.663 |
| drik-teimeris | parity-chart | 21 | 10.9 | 11.8 |
| drik-teimeris | widest-chart | 21 | 20.4 | 22.8 |
| drik-teimeris | matching | 21 | 0.023 | 0.044 |
| drik-teimeris | rectify | 21 | 33.3 | 36.3 |
| committee-teimeris | open | 21 | 0.021 | 0.028 |
| committee-teimeris | day | 21 | 0.540 | 0.832 |
| committee-teimeris | month | 21 | 10.5 | 11.6 |
| committee-teimeris | year | 21 | 104 | 116 |
| committee-teimeris | festivals | 21 | 23.4 | 25.0 |
| committee-teimeris | muhurta | 21 | 108 | 127 |
| committee-teimeris | chart | 21 | 0.539 | 0.739 |
| committee-teimeris | parity-chart | 21 | 10.3 | 11.6 |
| committee-teimeris | widest-chart | 21 | 19.7 | 69.3 |
| committee-teimeris | matching | 21 | 0.023 | 0.047 |
| committee-teimeris | rectify | 21 | 21.7 | 24.0 |
| drik-builtin | open | 21 | 0.013 | 0.017 |
| drik-builtin | day | 21 | 1.99 | 2.38 |
| drik-builtin | month | 21 | 31.6 | 33.5 |
| drik-builtin | year | 13 | 311 | 318 |
| drik-builtin | festivals | 21 | 117 | 164 |
| drik-builtin | muhurta | 9 | 469 | 477 |
| drik-builtin | chart | 21 | 0.915 | 1.19 |
| drik-builtin | parity-chart | 21 | 12.2 | 14.2 |
| drik-builtin | widest-chart | 21 | 22.2 | 24.6 |
| drik-builtin | matching | 21 | 0.023 | 0.063 |
| drik-builtin | rectify | 21 | 44.6 | 47.2 |

Contexts on threads, the `month` request's grid on each, a context a thread, of the 4 threads this machine offers. **Efficiency** is the throughput over the one thread's times the count (1 is linear); up to 4 threads, and fewer than the machine offers, it is held to 0.75. **Alike** is whether every thread's month hashed as one thread's did.

| setup | threads | median trial, ms | months a second | efficiency | alike |
|---|---:|---:|---:|---:|---|
| drik-teimeris | 1 | 33.6 (5 trials of 3 months) | 89.21 | 1.000 | yes |
| drik-teimeris | 2 | 36.7 (5 trials of 3 months) | 163.57 | 0.917 | yes |
| drik-teimeris | 4 | 46.8 (5 trials of 3 months) | 256.47 | 0.719 **under** | yes |
| committee-teimeris | 1 | 33.6 (5 trials of 3 months) | 89.15 | 1.000 | yes |
| committee-teimeris | 2 | 35.1 (5 trials of 3 months) | 170.98 | 0.959 | yes |
| committee-teimeris | 4 | 49.1 (5 trials of 3 months) | 244.17 | 0.685 **under** | yes |
| drik-builtin | 1 | 99.6 (5 trials of 3 months) | 30.11 | 1.000 | yes |
| drik-builtin | 2 | 102 (5 trials of 3 months) | 58.70 | 0.975 | yes |
| drik-builtin | 4 | 107 (5 trials of 3 months) | 112.49 | 0.934 | yes |

## `desktop-Darwin-arm64`

Apple M2 Pro (Darwin 27.0.0, aarch64), commit `cfa50116`, Teimeris 0.1.0, its data built in. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 0.020 | 0.021 | 0.013 | 0.6× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.08 | 0.540 | 2.27 | 2.1× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 17.2 | 11.1 | 36.5 | 2.1× |
| year (a year of almanac days) | 500 ms | — | 152 | 101 | 328 | 2.2× |
| festivals (a year of the Nepal pack's festivals) | 160 ms | 80.0 ms | 55.1 | 22.3 | 126 | 2.3× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 138 | 106 | 500 | 3.6× |
| chart (a birth chart with its Vimshottari to depth 3) | 2.00 ms | — | 0.512 | 0.508 | 0.990 | 1.9× |
| parity-chart (the full parity chart: every section and varga, the baseline's parity-tier dashas to depth 3, every shipped rule) | 10.0 ms | — | 9.71 | 9.09 | 11.3 | 1.2× |
| widest-chart (the parity chart with every dasha a chart gives) | 25.0 ms | — | 18.1 | 17.5 | 19.7 | 1.1× |
| matching (Ashta Koota between two cast charts) | 1.00 ms | — | 0.012 | 0.012 | 0.012 | within spread |
| rectify (a day-long window rectified at the default one-minute seed) | 1000 ms | — | 31.2 | 21.2 | 46.7 | 1.5× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 0.019 | 0.044 |
| drik-teimeris | day | 21 | 1.06 | 1.17 |
| drik-teimeris | month | 21 | 16.9 | 17.8 |
| drik-teimeris | year | 21 | 150 | 155 |
| drik-teimeris | festivals | 21 | 54.5 | 56.6 |
| drik-teimeris | muhurta | 21 | 136 | 141 |
| drik-teimeris | chart | 21 | 0.507 | 0.575 |
| drik-teimeris | parity-chart | 21 | 9.57 | 10.6 |
| drik-teimeris | widest-chart | 21 | 17.9 | 18.8 |
| drik-teimeris | matching | 21 | 0.012 | 0.022 |
| drik-teimeris | rectify | 21 | 30.8 | 33.0 |
| committee-teimeris | open | 21 | 0.021 | 0.026 |
| committee-teimeris | day | 21 | 0.531 | 0.583 |
| committee-teimeris | month | 21 | 11.0 | 11.4 |
| committee-teimeris | year | 21 | 99.8 | 104 |
| committee-teimeris | festivals | 21 | 22.2 | 23.5 |
| committee-teimeris | muhurta | 21 | 104 | 109 |
| committee-teimeris | chart | 21 | 0.501 | 0.547 |
| committee-teimeris | parity-chart | 21 | 9.05 | 9.42 |
| committee-teimeris | widest-chart | 21 | 17.3 | 18.3 |
| committee-teimeris | matching | 21 | 0.012 | 0.013 |
| committee-teimeris | rectify | 21 | 21.1 | 21.7 |
| drik-builtin | open | 21 | 0.013 | 0.015 |
| drik-builtin | day | 21 | 2.26 | 2.31 |
| drik-builtin | month | 21 | 36.3 | 37.1 |
| drik-builtin | year | 13 | 326 | 331 |
| drik-builtin | festivals | 21 | 125 | 128 |
| drik-builtin | muhurta | 9 | 497 | 501 |
| drik-builtin | chart | 21 | 0.975 | 1.05 |
| drik-builtin | parity-chart | 21 | 11.2 | 11.9 |
| drik-builtin | widest-chart | 21 | 19.6 | 20.6 |
| drik-builtin | matching | 21 | 0.012 | 0.016 |
| drik-builtin | rectify | 21 | 46.4 | 47.8 |

Contexts on threads, the `month` request's grid on each, a context a thread, of the 10 threads this machine offers. **Efficiency** is the throughput over the one thread's times the count (1 is linear); up to 4 threads, and fewer than the machine offers, it is held to 0.75. **Alike** is whether every thread's month hashed as one thread's did.

| setup | threads | median trial, ms | months a second | efficiency | alike |
|---|---:|---:|---:|---:|---|
| drik-teimeris | 1 | 35.7 (5 trials of 3 months) | 84.15 | 1.000 | yes |
| drik-teimeris | 2 | 37.2 (5 trials of 3 months) | 161.12 | 0.957 | yes |
| drik-teimeris | 4 | 38.2 (5 trials of 3 months) | 314.48 | 0.934 | yes |
| drik-teimeris | 8 | 51.2 (5 trials of 3 months) | 468.97 | 0.697 | yes |
| drik-teimeris | 10 | 59.9 (5 trials of 3 months) | 500.44 | 0.595 | yes |
| committee-teimeris | 1 | 34.3 (5 trials of 3 months) | 87.40 | 1.000 | yes |
| committee-teimeris | 2 | 36.5 (5 trials of 3 months) | 164.53 | 0.941 | yes |
| committee-teimeris | 4 | 39.4 (5 trials of 3 months) | 304.61 | 0.871 | yes |
| committee-teimeris | 8 | 51.8 (5 trials of 3 months) | 462.94 | 0.662 | yes |
| committee-teimeris | 10 | 58.6 (5 trials of 3 months) | 512.33 | 0.586 | yes |
| drik-builtin | 1 | 111 (5 trials of 3 months) | 27.05 | 1.000 | yes |
| drik-builtin | 2 | 113 (5 trials of 3 months) | 52.87 | 0.977 | yes |
| drik-builtin | 4 | 116 (5 trials of 3 months) | 103.87 | 0.960 | yes |
| drik-builtin | 8 | 155 (5 trials of 3 months) | 155.23 | 0.717 | yes |
| drik-builtin | 10 | 182 (5 trials of 3 months) | 165.26 | 0.611 | yes |

## `ios-sim-arm64-ios-simulator-iphone-18-pro`

iOS simulator, iPhone 18 Pro (iOS 27.0, aarch64), commit `cfa50116`, Teimeris 0.1.0, its data built in. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 0.025 | 0.025 | 0.013 | 0.5× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.10 | 0.572 | 2.12 | 1.9× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 17.0 | 10.8 | 32.2 | 1.9× |
| year (a year of almanac days) | 500 ms | — | 152 | 101 | 307 | 2.0× |
| festivals (a year of the Nepal pack's festivals) | 160 ms | 80.0 ms | 56.3 | 23.4 | 118 | 2.1× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 134 | 108 | 476 | 3.6× |
| chart (a birth chart with its Vimshottari to depth 3) | 2.00 ms | — | 0.570 | 0.551 | 0.930 | 1.6× |
| parity-chart (the full parity chart: every section and varga, the baseline's parity-tier dashas to depth 3, every shipped rule) | 10.0 ms | — | 10.7 | 9.83 | 11.7 | within spread |
| widest-chart (the parity chart with every dasha a chart gives) | 25.0 ms | — | 19.2 | 18.5 | 20.4 | 1.1× |
| matching (Ashta Koota between two cast charts) | 1.00 ms | — | 0.013 | 0.014 | 0.014 | within spread |
| rectify (a day-long window rectified at the default one-minute seed) | 1000 ms | — | 30.7 | 21.8 | 43.6 | 1.4× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 0.022 | 0.069 |
| drik-teimeris | day | 21 | 1.07 | 1.33 |
| drik-teimeris | month | 21 | 16.5 | 22.9 |
| drik-teimeris | year | 21 | 150 | 155 |
| drik-teimeris | festivals | 21 | 55.2 | 57.2 |
| drik-teimeris | muhurta | 21 | 131 | 136 |
| drik-teimeris | chart | 21 | 0.549 | 0.598 |
| drik-teimeris | parity-chart | 21 | 10.3 | 14.4 |
| drik-teimeris | widest-chart | 21 | 18.8 | 19.7 |
| drik-teimeris | matching | 21 | 0.013 | 0.019 |
| drik-teimeris | rectify | 21 | 30.0 | 32.5 |
| committee-teimeris | open | 21 | 0.025 | 0.037 |
| committee-teimeris | day | 21 | 0.557 | 0.604 |
| committee-teimeris | month | 21 | 10.5 | 11.6 |
| committee-teimeris | year | 21 | 99.4 | 104 |
| committee-teimeris | festivals | 21 | 22.6 | 24.6 |
| committee-teimeris | muhurta | 21 | 105 | 110 |
| committee-teimeris | chart | 21 | 0.535 | 0.676 |
| committee-teimeris | parity-chart | 21 | 9.45 | 10.2 |
| committee-teimeris | widest-chart | 21 | 18.3 | 19.3 |
| committee-teimeris | matching | 21 | 0.013 | 0.055 |
| committee-teimeris | rectify | 21 | 21.8 | 23.2 |
| drik-builtin | open | 21 | 0.013 | 0.016 |
| drik-builtin | day | 21 | 2.09 | 2.18 |
| drik-builtin | month | 21 | 32.0 | 33.9 |
| drik-builtin | year | 13 | 304 | 316 |
| drik-builtin | festivals | 21 | 116 | 123 |
| drik-builtin | muhurta | 9 | 471 | 481 |
| drik-builtin | chart | 21 | 0.915 | 0.997 |
| drik-builtin | parity-chart | 21 | 11.4 | 12.2 |
| drik-builtin | widest-chart | 21 | 20.0 | 21.2 |
| drik-builtin | matching | 21 | 0.013 | 0.018 |
| drik-builtin | rectify | 21 | 42.9 | 86.3 |

Contexts on threads, the `month` request's grid on each, a context a thread, of the 10 threads this machine offers. **Efficiency** is the throughput over the one thread's times the count (1 is linear); up to 4 threads, and fewer than the machine offers, it is held to 0.75. **Alike** is whether every thread's month hashed as one thread's did.

| setup | threads | median trial, ms | months a second | efficiency | alike |
|---|---:|---:|---:|---:|---|
| drik-teimeris | 1 | 33.3 (5 trials of 3 months) | 90.20 | 1.000 | yes |
| drik-teimeris | 2 | 34.9 (5 trials of 3 months) | 172.06 | 0.954 | yes |
| drik-teimeris | 4 | 39.6 (5 trials of 3 months) | 302.73 | 0.839 | yes |
| drik-teimeris | 8 | 52.8 (5 trials of 3 months) | 454.96 | 0.630 | yes |
| drik-teimeris | 10 | 59.6 (5 trials of 3 months) | 503.21 | 0.558 | yes |
| committee-teimeris | 1 | 32.7 (5 trials of 3 months) | 91.83 | 1.000 | yes |
| committee-teimeris | 2 | 34.9 (5 trials of 3 months) | 172.05 | 0.937 | yes |
| committee-teimeris | 4 | 38.0 (5 trials of 3 months) | 315.84 | 0.860 | yes |
| committee-teimeris | 8 | 48.7 (5 trials of 3 months) | 493.09 | 0.671 | yes |
| committee-teimeris | 10 | 61.2 (5 trials of 3 months) | 490.19 | 0.534 | yes |
| drik-builtin | 1 | 97.8 (5 trials of 3 months) | 30.68 | 1.000 | yes |
| drik-builtin | 2 | 101 (5 trials of 3 months) | 59.59 | 0.971 | yes |
| drik-builtin | 4 | 104 (5 trials of 3 months) | 114.94 | 0.937 | yes |
| drik-builtin | 8 | 143 (5 trials of 3 months) | 167.66 | 0.683 | yes |
| drik-builtin | 10 | 171 (5 trials of 3 months) | 175.58 | 0.572 | yes |
