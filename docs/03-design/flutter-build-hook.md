# The Flutter package

Status: `designed`, 2026-10-11, before any code. The roadmap item is
`07-roadmap/00-roadmap.md`'s "the Flutter build-hook package, which carries
the library into an Android or iOS build" (`STATUS.md`, How to resume). It
derives from the Dart binding's loader and installer
(`bindings/dart/lib/teistro.dart`, `lib/src/install.dart`), the binding
architecture ([`../02-architecture/07-binding-architecture.md`](../02-architecture/07-binding-architecture.md)), and `QUESTIONS.md` Q16 (the name
`teistro_flutter`), ADR-0014 (a gate skips with a note where its toolchain
is missing) and ADR-0022 (the same bits on every platform). It will be
built as `bindings/flutter`.

Every claim about this repository names the file it was read from; every
claim about Dart, Flutter, Android or Apple names its source, read on
2026-10-11. The open points of §6 are decided there, each with its reason.

## 1. What exists today

- **`bindings/dart`** (`teistro`, `publish_to: none` at 0.0.0, `sdk: ^3.7.0`,
  one dependency `ffi`). It loads through `ffi.DynamicLibrary`, not `@Native`:
  `Teistro.open({path})` takes `path`, then `$TEISTRO_LIBRARY`, then
  `.dart_tool/teistro/<version>/<lib>` (what `dart run teistro:install`
  wrote), then beside the script, then `bindings/dart/`, `target/{release,debug}`,
  `../../target/...`, and last the bare file name through the platform
  loader. Then it does the build handshake (`BuildInfo` plus `refuseBuild`): ABI and
  SDK version must match the generated declarations, a sanitizer build is
  refused, and an unoptimised one is refused unless it was named.
- **The installer** (`lib/src/install.dart`, `bin/install.dart`) fetches
  `libteistro_ffi-<ver>-<os>-<cpu>.<ext>.gz` from the GitHub release and
  gunzips it. It checks the bytes against `prebuiltDigests` in
  `lib/src/prebuilt.dart`, the uncompressed library's SHA-256 by `<os>-<cpu>`
  in Node's words. The table is empty in the repository and is written by
  `cargo xtask package stage` (`xtask/src/package.rs`, `stage_dart` /
  `prebuilt_table`) from the merged release manifests. `--from` installs
  offline. The SHA-256 is the package's own (`lib/src/sha256.dart`). The
  package has no `hook/` and no native-assets code.
- **Platforms** (`xtask/src/platform.rs`, `PLATFORMS: [Platform; 8]`): linux
  x64/arm64 (glibc and musl), darwin arm64/x64, win32 x64/arm64. **No Android
  or iOS row.** Every row is built natively on its runner (`release.yml`
  build matrix: 6 rows plus musl). The `package` step runs `check-package`,
  which runs what it built. A row carries npm names, a glibc floor and a
  wheel tag. `package` emits per row the gzipped library, a C bundle, the
  npm addon package, the agent-server archive and SBOMs.
- **Sizes** (`docs/05-testing/sizes.json`, run 38133623988): the library is
  14.5 MB raw / 4.97 MB gzip on linux-x64, 13.2 / 4.64 MB on linux-arm64,
  and 17.8 / 5.17 MB on darwin-arm64.
- **Docs.** The roadmap (`07-roadmap/00-roadmap.md` l. 112, 132) and STATUS
  "Carried forward" describe this as its own deliverable, "with the mobile
  targets, v1.x", and STATUS `## Next` calls it the one binding deliverable
  left for the release. `QUESTIONS.md` Q16 and `06-cicd/03-release-process.md`
  name **`teistro_flutter`** on pub.dev (watched, not held).
  `02-architecture/07-binding-architecture.md` l. 206 says "Flutter plugin
  builds the native library per platform", which predates build hooks and
  is superseded below. `01-research/platform/11-cicd-release.md` l. 26 names
  cargo-ndk / xcframework, and
  `15-provider-plugins-and-targets.md` l. 80 says "the same binary, bundled
  as an asset".
- **Crate graph**: `teistro-ffi` is `cdylib`+`staticlib`, and `Cargo.lock`
  has no `cc`, `ring` or `openssl`. It is pure Rust, so cross-compiling
  needs only a linker, not a C toolchain per target.
- **This machine**: Flutter 3.47.2 stable / Dart 3.13.2, NDK 27.0.12077973
  and 28.2.13676358 installed. No Android or iOS Rust targets are installed.

## 2. The state of build hooks (October 2026)

- **Stable.** "Support for build hooks was introduced in Dart 3.10", and
  link hooks arrived in 3.13 [dart.dev/tools/hooks]. Flutter: "Since Flutter
  3.38, the recommended way to bind to native code is ... build hooks ...
  no longer requires OS-specific build files" [docs.flutter.dev
  bind-native-code]. The current stable is Flutter 3.47 (release notes,
  updated 2026-08-12).
- **Packages** (publisher dart.dev): `hooks` 2.2.0 provides `build()`,
  `BuildInput`, `output.assets`, `output.dependencies` and `userDefines`.
  `code_assets` 2.1.0 provides `CodeAsset`, `OS`, `Architecture`,
  `input.config.code.{targetOS, targetArchitecture, iOS.targetSdk,
  iOS.targetVersion, android.targetNdkApi}`, and the link modes
  `DynamicLoadingBundled`, `DynamicLoadingSystem` and `LookupInProcess`
  [pub.dev/packages/hooks, /code_assets].
- **Prebuilt binaries**: "The recommended approach is to download the
  pre-compiled binaries at build time and verify their integrity with a
  file hash" [bind-native-code]. The official `download_asset` example
  reads a `userDefines['local_build']` switch, downloads with
  `HttpClient()..findProxy = HttpClient.findProxyFromEnvironment`, and
  checks against a generated `assetHashes` table keyed by
  `<name>_<os>_<arch>[_<sdk>]` covering android arm/arm64/ia32/riscv64/x64
  and ios arm64-iphoneos, arm64-iphonesimulator and x64-iphonesimulator. It
  uses **MD5**, with a TODO admitting it does not guard against a
  compromised host [github.com/dart-lang/native, pkgs/hooks/example/build/
  download_asset]. We use SHA-256, which we already have.
- **Environment, cache, network**: hooks run "semi-hermetic". Only PATH,
  HOME, temp, the proxy variables and the Android NDK/SDK variables pass
  through, and changing any of them invalidates the cache. "The hook should
  place downloaded and generated assets in `BuildInput.sharedOutputDirectory`"
  [dart.dev/tools/hooks]. Network access is allowed (the official example
  downloads), but nothing caches it for you.
- **User defines**: `hooks: user_defines: <package>: {...}` in the **root**
  pubspec (or the workspace pubspec). Dependencies cannot set defaults.
  `input.userDefines.path(key)` resolves a path relative to that pubspec
  [dart.dev/tools/hooks].
- **Android**: flutter_tools copies each asset to
  `jniLibs/lib/<abi>/` (`isolated/native_assets/android/native_assets.dart`)
  and runs the hook once per ABI it builds: arm, arm64 and x64 by default.
  The defaults are `minSdkVersion = 24`, `ndkVersion = '28.2.13676358'` and
  compileSdk 36 (`flutter_tools/lib/src/android/gradle_utils.dart`,
  `FlutterExtension.kt`). The hook sees the app's minSdk as
  `android.targetNdkApi`. Google Play has required 16 KB page alignment of
  64-bit native code for apps targeting API 35+ since 1 November 2025
  [developer.android.com/16kb-page-size].
- **iOS**: the hook is invoked per (SDK, arch): iphoneos-arm64,
  iphonesimulator-arm64 and iphonesimulator-x64. It "must produce the same
  filename regardless of the target architecture" and "the same framework
  name" for device and simulator, and the same asset-id set across SDKs,
  because Flutter `lipo`s the simulator slices and runs
  `xcodebuild -create-xcframework` [bind-native-code]. Flutter wraps each
  dylib in its own framework, named by stripping `lib` and `.dylib`
  (`frameworkUri`, `macos/native_assets_host.dart`), so `libteistro_ffi.dylib`
  becomes `teistro_ffi.framework/teistro_ffi`. It then writes the dSYM,
  strips, sets the install name to `@rpath/teistro_ffi.framework/teistro_ffi`,
  and **code-signs with the app's identity** (`codesignDylib`,
  `ios/native_assets.dart`). The fix that made the manifest and the stamped
  install name agree landed on 2026-08-18 (flutter commit 295e9f1e). Flutter
  states iOS `targetIOSVersion = 15` (same file, with a TODO). We ship the
  dylibs unsigned.
- **pub.dev size**: the validator warns above 100 MB gzipped / 256 MB raw
  [dart.dev/tools/pub/publishing; pub `validator/size.dart`].

## 3. Decision

**D1: a separate package, `teistro_flutter`, holding only a build hook.** It
lives at `bindings/flutter/` and depends on `teistro` at **the exact same
version**. Reasons:
1. A hook in `teistro` itself would run for every consumer: `dart test`,
   servers, and our own `check-dart`/`check-parity`. Network access would
   become a build requirement for all of them, and the installer and the
   search path that the Dart gates are built on would change. The VM story
   works and is gated, so it stays as it is.
2. The name is the one Q16 already settled. The roadmap and STATUS describe
   this as a separate deliverable.
3. A Flutter app adds a single dependency (`teistro_flutter`, which
   re-exports `package:teistro/teistro.dart`) and never runs
   `dart run teistro:install`.
4. The Teimeris adapter's Dart package (`teistro_ephemeris_teimeris`,
   `PluginEphemeris`) will need the same thing for mobile. The hook's body
   is written as one function over (stem, version, digest table, base URL)
   that the adapter's own hook can call (see open point O5).

The trade-off: a consumer must know to add the second package. The `teistro`
README, the `Teistro.open` refusal on Android/iOS and the docs-site guide all
say so.

**D2: the hook downloads, and the digest is pinned (SHA-256); nothing is bundled.**
- A download fetches only the slice being built, about 4.6 to 5 MB gzipped
  per ABI. Bundling six mobile slices would add roughly 30 MB to every
  consumer's `pub get`, plus 30 MB more if desktop Flutter were added. That
  stays under pub.dev's 100 MB, but it contradicts the installer's stated
  rule ("A pub package cannot carry a binary for every platform without
  making every consumer download all of them"), and Dart's own guidance is
  to download and hash.
- Digests come from **one table**. Mobile rows go into the existing
  `prebuiltDigests` (`android-arm64`, `android-arm`, `android-x64`,
  `ios-arm64`, `ios-sim-arm64`, `ios-sim-x64`), which `package stage`
  already writes from the build manifests. `teistro` gains a small public
  library, `package:teistro/prebuilt.dart`, that exports `prebuiltVersion`,
  `prebuiltBase`, `prebuiltDigests`, `prebuiltUri(platform)` (taking an
  explicit file name) and `verified()`. The hook imports it, so the check
  stays in one place (it is `verified`, with its tested refusals) and no
  second table exists.
- The hook works in four steps:
  1. Map the build's target to a release platform name, or refuse it by
     name (ia32 or riscv64 Android; an iOS `targetVersion` below the
     library's floor; an Android `targetNdkApi` below 24).
  2. Look for `outputDirectoryShared/<version>/<platform>/<lib>`. If it is
     there and hashes right, use it. If not, fetch the archive with proxy
     support, then gunzip and `verified()` it **before** writing. The write
     goes to a temp name followed by a rename, so concurrent per-arch
     invocations cannot see a half-written file.
  3. Add `CodeAsset(package: 'teistro_flutter', name: 'teistro_ffi.dart',
     linkMode: DynamicLoadingBundled(), file: ...)`. The file name is
     `libteistro_ffi.so` on Android and `libteistro_ffi.dylib` on iOS, the
     same for every arch and SDK, as Flutter requires.
  4. Refuse loudly. Every failure is an `InstallException` sentence with a
     hint, such as "no prebuilt for android-riscv64 in 0.x; build
     arm64/arm/x64" or "offline: set `prebuilt_dir`".
- **User defines** (root pubspec, `hooks: user_defines: teistro_flutter:`):
  - `prebuilt_dir: <dir>` reads the same `.gz` names from a directory,
    still verified. This covers offline and air-gapped builds and is what
    CI uses.
  - `library: <path>` names a library outright with no digest check, like
    `Teistro.open(path:)`. The handshake still applies at load, and the
    hook prints a warning that it is unverified.
  - There is no "build from source in the hook". It would make cargo and
    the NDK a build requirement for Flutter apps, and `library:` covers
    contributors.

**D3: loader rules in `teistro`.** `Teistro.searchPath` gains a first
**bundled** step. On Android it opens the bare `libteistro_ffi.so` (jniLibs
is on the app's linker namespace path). On iOS it opens
`teistro_ffi.framework/teistro_ffi` (dyld resolves it through the app's
`@rpath`), the name derived by Flutter's `frameworkUri` above. The step is
tried only when `Platform.isAndroid || Platform.isIOS`, before the
filesystem probes, which mean nothing in an app sandbox. `$TEISTRO_LIBRARY`
and `path:` still win. A bundled library counts as **not named** for
`refuseBuild` (so a debug build is refused, which is right because the hook
only ever ships release builds). The not-found `StateError` on mobile says:
"add `teistro_flutter` to your app's dependencies". `install.dart`'s
`hostPlatform` gains `android-*`/`ios-*` so its names match the table.
`@Native`/asset-id resolution is not adopted now (open point O1): the
generated class takes a `DynamicLibrary`, and switching ffigen to
native-function mode is a binding-wide change that this deliverable does not
need.

**D4: the Rust targets and how they are built.** There are six new rows:

| platform | triple | ABI / SDK | runner | floor |
|---|---|---|---|---|
| android-arm64 | aarch64-linux-android | arm64-v8a | ubuntu-latest | API 24, 16 KB pages |
| android-arm | armv7-linux-androideabi | armeabi-v7a | ubuntu-latest | API 24 |
| android-x64 | x86_64-linux-android | x86_64 | ubuntu-latest | API 24, 16 KB pages |
| ios-arm64 | aarch64-apple-ios | iphoneos | macos-latest | iOS 13.0 |
| ios-sim-arm64 | aarch64-apple-ios-sim | iphonesimulator | macos-latest | iOS 13.0 |
| ios-sim-x64 | x86_64-apple-ios | iphonesimulator | macos-latest | iOS 13.0 |

- **No cargo-ndk.** The crate graph has no C, so `xtask package` sets
  `CARGO_TARGET_<TRIPLE>_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/
  linux-x86_64/bin/<clang-triple>24-clang` and `AR=llvm-ar` itself. This is
  one rule in Rust with no extra tool to pin, and cargo-auditable keeps
  working. The **NDK is pinned to Flutter's default, 28.2.13676358**, in
  `xtask/ndk.version`, read the way `cargo-auditable.version` is, and
  installed by `sdkmanager` on the runner. API 24 is Flutter's default
  minSdk and is above Rust's own minimum. 64-bit Android rows link with
  `-C link-arg=-Wl,-z,max-page-size=16384`. NDK r28 already defaults to it,
  but the flag makes the rule explicit, and a check reads it back (below).
- **iOS**: `IPHONEOS_DEPLOYMENT_TARGET=13.0` (below Flutter's 15, so every
  Flutter app accepts it) and `-C link-arg=-Wl,-headerpad_max_install_names`,
  so Flutter's `install_name_tool` rewrite always fits. The library is
  unsigned, because Flutter signs it with the app's identity.
- **The table.** These rows join `PLATFORMS` in `platform.rs`, which stays
  one table, with a new field `ships: Ships::{Host, Mobile}`. The Mobile
  rows have no npm/wheel/jar/C bundle/agent server, and `os`/`cpu` give the
  release name. Every consumer of `PLATFORMS` (npm staging, wheels, jar,
  `check-package`, `runner-matches-the-platform-table`) filters by `ships`,
  so "adding a platform is adding a row" still holds. `package <mobile>`
  cross-compiles: it is the first set of rows that is not native, and that
  is unavoidable. It emits only the gzipped library, its SBOM and the
  manifest, and then runs the **mobile floor check** in place of `floor.rs`:
  - Android: `NEEDED` must be a subset of {libc.so, libm.so, libdl.so}, so
    there is no `libc++_shared.so` and no `android_libcpp_shared`. Every
    `PT_LOAD` `p_align` must be at least 0x4000 on the 64-bit rows.
  - iOS: `LC_BUILD_VERSION` must have platform IOS or IOSSIMULATOR and
    `minos` 13.0, and the arch must be the row's.
  These are read with the `object` crate. Building twice must also give the
  same bits (ADR-0022), as for every other row.

**D5: how it is proved.** The checks are ordered from cheap to the device:

1. `check-flutter-hook` (fast-check, pure Dart, no toolchain beyond
   `dart`): unit tests of the hook body on a fabricated `BuildInput` with
   the hooks package's test helpers. They cover every Flutter target mapping
   to a platform or a refusal. A `prebuilt_dir` with a good archive is
   accepted, and one with a flipped byte is refused with expected/found. A
   cached file is reused without a fetch, and a cached file with a wrong
   hash is refused. iOS gives one file name for all three (SDK, arch). These
   tests are proved red first.
2. `check-flutter` (verify.yml, ubuntu): build the three Android slices.
   Stage the packages with a merged manifest so the digests are real (as
   `check-package` does), and create a throwaway app from
   `bindings/flutter/example` with `prebuilt_dir` pointing at `target/dist`.
   Run `flutter build apk --release`. Then `unzip -l` must show
   `lib/{arm64-v8a,armeabi-v7a,x86_64}/libteistro_ffi.so` and nothing else
   of ours. Each `.so` must report its own triple through the `ts_build_info`
   JSON string read from the file (the `BuildInfo.target` field), and
   `zipalign -c -P 16 -v 4` must pass.
3. `check-flutter-android` (verify.yml, ubuntu with KVM, an x86_64 emulator
   at API 24 and API 35): `flutter test integration_test` in the example app
   asserts **consumer.dart's four facts**. `packaging/consumer.dart` is moved
   into a function that both runners call, so it stays one suite: optimised,
   no sanitizer, version equal to `prebuiltVersion`, and 14 April 2015 =
   2072-1-1 BS, and so on.
4. `check-flutter-ios` (verify.yml, macos-latest): `flutter build ios
   --release --no-codesign` must produce
   `Runner.app/Frameworks/teistro_ffi.framework/teistro_ffi` with arm64 and
   `minos` 13. The integration test runs on a booted iOS simulator
   (`xcrun simctl`), with the same suite.
5. **The hash matrix** (`hash-matrix.yml`, nightly) gains rows for
   `aarch64-linux-android`/`x86_64-linux-android` and `aarch64-apple-ios-sim`.
   The `teistro-scenario` binary is pushed with `adb` and run in the
   emulator, or run with `xcrun simctl spawn booted`, and compared by
   `compare-hashes` like every row. This holds ADR-0022's byte-identity on
   the mobile ABIs, not just "it loads". Stretch goal: run
   `bin/parity.dart`'s runner inside the integration test and diff it
   against the VM's output (reusing a suite as the acceptance test).

## 4. Files

New:
- `docs/03-design/flutter-build-hook.md`: this page, gated by `check-docs`.
- `bindings/flutter/pubspec.yaml`: `name: teistro_flutter`, `environment:
  sdk: ^3.10.0, flutter: '>=3.38.0'`. Dependencies: `teistro: <exact>`,
  `hooks: ^2.2.0`, `code_assets: ^2.1.0`. `publish_to: none` until release,
  as `teistro` has.
- `bindings/flutter/hook/build.dart`: about 15 lines, calling `bundlePrebuilt`.
- `bindings/flutter/lib/teistro_flutter.dart`: `export
  'package:teistro/teistro.dart';`.
- `bindings/flutter/lib/src/bundle.dart`: `bundlePrebuilt(input, output,
  {stem, digests, version, base})`, the target-to-platform mapping, the cache
  and the refusals.
- `bindings/flutter/test/bundle_test.dart`, `bindings/flutter/README.md`,
  `bindings/flutter/LICENSE` and `NOTICE`, copied at stage time like the
  Dart package's.
- `bindings/flutter/example/`: a minimal app plus `integration_test/teistro_test.dart`.
- `bindings/dart/lib/prebuilt.dart`: the public face of the digest table and
  `verified`.
- `bindings/dart/lib/src/acceptance.dart` (or similar): the four facts, used
  by `packaging/consumer.dart` and the integration test.
- `xtask/src/flutter_package.rs`: `check-flutter-hook`, `check-flutter`,
  `check-flutter-android` and `check-flutter-ios`, skipping with a note
  where the toolchain is missing (ADR-0014).
- `xtask/src/mobile_floor.rs`: the Android/iOS read-backs from D4.
- `xtask/ndk.version`.

Changed:
- `xtask/src/platform.rs`: six rows, the `ships` field and the names.
- `xtask/src/package.rs`: cross-compiling `compile` for the Mobile rows and
  the reduced `stage_platform`. `stage` adds the mobile digests to
  `prebuilt.dart` and stages `dist/pub/teistro_flutter` with the exact pin.
  Refuse to stage a release missing any Mobile manifest, as it refuses
  missing npm platforms.
- `xtask/src/release.rs` (`check-versions`/`set_dart`): `teistro_flutter`'s
  version and pin, and `publish_to` both ways.
- `xtask/src/lints.rs`: `runner-matches-the-platform-table` covers the new
  rows in `release.yml`/`hash-matrix.yml`.
- `bindings/dart/lib/teistro.dart`: the bundled step and the mobile refusal.
  `lib/src/install.dart`: the `hostPlatform` names.
- `.github/workflows/release.yml`: build rows for the six, each running
  `cargo xtask package <name>` but not `check-package`. Publish
  `dist/pub/teistro_flutter` after `dist/pub/teistro`. The `*.gz` upload
  already covers the archives.
- `.github/workflows/verify.yml` gets the three `check-flutter*` jobs;
  `fast-check.yml` gets `check-flutter-hook`; `hash-matrix.yml` gets the
  mobile rows.
- `docs/05-testing/sizes.json`: six library rows via `cargo xtask sizes
  --from`. `02-architecture/07-binding-architecture.md` l. 206: replace
  "Flutter plugin builds the native library" with the hook. STATUS and the
  roadmap: move the item from v1.x to built. The `teistro` README gets the
  Flutter section.

## 5. Order, each step with its gate

1. **The design page lands.** Gate: `check-docs`.
2. **Platform rows and the cross build** (Android first, then iOS). Gate:
   `cargo xtask package android-arm64` (on CI, not on this machine) produces
   an archive that passes the mobile floor check. Prove the check red first
   with a 4 KB-aligned build and an iOS 17 `minos`. Measure sizes.
3. **The hash matrix rows** on the emulator and simulator. Gate:
   `compare-hashes` reports 0 differing values. This is done before any
   packaging work, because a mobile row that computes differently is a
   defect in the core and not in the package.
4. **`prebuilt.dart` public plus the loader's bundled step.** Gates:
   `check-dart` (the existing tests are unchanged) and a test that
   `searchPath` on a faked Android/iOS lists the bundled name first.
5. **The `teistro_flutter` package and its unit tests.** Gate:
   `check-flutter-hook` in fast-check, proved red first.
6. **APK inspection.** Gate: `check-flutter` on ubuntu (verify, dispatched
   on the branch).
7. **Device runs.** Gates: `check-flutter-android` and `check-flutter-ios`.
8. **Release wiring.** Gate: a release rehearsal (dispatch, nothing
   published) whose `stage` carries nine-plus-six digests and stages both
   pub packages. `dart pub publish --dry-run` must pass on both, with sizes
   recorded.
9. **Docs**: README, the guide with an executed example, STATUS and the
   roadmap.

## 6. Risks, and the open points decided

- **O1 Loader names.** The bundled names rely on Flutter's conventions
   (`jniLibs`, `frameworkUri` stripping `lib`/`.dylib`). The device runs
   (D5.3/4) are the gate. If Flutter changes them, the fix is moving the
   generated layer to `@Native` with an asset id (`package:teistro_flutter/
   teistro_ffi.dart`), which is a separate decision.
- **O2 armeabi-v7a cannot be run in CI.** x86_64 emulators have no ARM
   translation we can rely on, and GitHub's Arm Linux runners do not offer
   KVM for an arm64 emulator. If the row ships untested, it breaks "a host
   is checked or unclaimed". The options: (a) ship it with the build check,
   the APK check and a hash-matrix row under `qemu-arm` for a static
   `armv7-unknown-linux-musleabihf` scenario built with the same target
   features; (b) refuse `arm` in the hook with a hint (`--target-platform
   android-arm64,android-x64`). A default `flutter build apk` asks for
   arm, so (b) breaks the default command. **Decided: (a)**, and the
   release notes say which proof each ABI has.
- **O3 Apple's required-reason APIs.** App Store Connect scans embedded
   binaries for required-reason APIs (file timestamps, `mach_absolute_time`
   and others). Run `nm -u` on the iOS dylib in the floor check, list any
   such symbols, and document what the app's `PrivacyInfo.xcprivacy` must
   declare. Flutter's generated framework has no place for a manifest of
   our own. **Decided:** the floor check lists the symbols and fails on one
   the README's privacy section does not name; Apple's current list is read
   when step 2 lands.
- **O4 App size.** At about 13 to 14 MB raw per ABI in the APK (AAB splits
   per ABI), a Flutter app grows a lot. A mobile release profile
   (`opt-level = "s"`, `lto = "fat"`; `panic = "abort"` is **not** an
   option because the boundary catches panics) is a measured decision.
   **Decided:** the mobile rows ship the release profile every row ships
   until step 2 measures the alternative by gzipped and installed bytes; a
   profile is changed only by that measurement.
- **O5 The Teimeris adapter on mobile.** `PluginEphemeris` names a path,
   and in an app that path is the bundled name of the adapter's own asset.
   That requires the adapter's package to call `bundlePrebuilt` and the
   adapter's release to build the six mobile rows. The engine's build has
   C in it, which does need an NDK `cc`. **Decided:** v1.x; the README says
   the built-in ephemeris is what a mobile app has until then.
- **O6 Desktop Flutter.** macOS, Windows and Linux Flutter apps have the
   same problem today: `.dart_tool` is not inside an app bundle. The hook
   can map those to the existing release rows at almost no cost, and the
   loader would add the bundle names. **Decided:** a step after D5.4, gated
   by `flutter build macos`/`linux`/`windows` inspection, before the
   release candidate if the mobile steps leave time and in v1.x otherwise.
- **O7 Network in builds.** CI for consumers must reach GitHub releases
   or set `prebuilt_dir`. Corporate proxies are honoured through the
   pass-through variables, and that has to be documented. A compromised
   release host is caught by the digest, because the table travels in the
   pub package, a separate trust path, and the attestations remain
   (`gh attestation verify`).
- **O8 pub.dev's first publish.** Automated publishing is configured on
   a package that exists, so `teistro_flutter`'s first upload is a manual
   step for the maintainer, as npm's was. The name is not held (pub.dev
   removes placeholders).
- **O9 Version skew.** Because of the exact pin, a `teistro` patch release
   forces a `teistro_flutter` release. `check-versions` holds them together,
   and `refuseBuild` catches any skew at load.

## Sources

- https://dart.dev/tools/hooks (stable in 3.10, env pass-through,
  `outputDirectoryShared`, user defines, asset ids)
- https://docs.flutter.dev/platform-integration/bind-native-code (3.38,
  download-and-hash, iOS naming rules, `android_libcpp_shared`)
- https://docs.flutter.dev/release/release-notes (3.47 current)
- https://pub.dev/packages/hooks (2.2.0), https://pub.dev/packages/code_assets (2.1.0)
- https://github.com/dart-lang/native/tree/main/pkgs/hooks/example/build/download_asset
  (`hook/build.dart`, `hook_helpers/download.dart`, `hashes.dart` (MD5),
  `target_versions.dart`)
- flutter/flutter master: `packages/flutter_tools/lib/src/isolated/native_assets/{android,ios,macos}/`
  (jniLibs, `frameworkUri`, strip, install names, `codesignDylib`,
  `targetIOSVersion = 15`); `lib/src/android/gradle_utils.dart` (minSdk 24,
  NDK 28.2.13676358)
- https://flutter.googlesource.com/mirrors/flutter/+/295e9f1e553f2e946d6e24f3bf28ca33fc78a83d
  (install name and manifest agree, 2026-08-18)
- https://dart.dev/tools/pub/publishing (100 MB gzipped / 256 MB)
- https://developer.android.com/16kb-page-size (Play requirement since 2025-11-01)
