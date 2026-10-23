# teistro_flutter

The [Teistro SDK](https://pub.dev/packages/teistro) in a Flutter app on
Android and iOS. This package holds a build hook and nothing else. When the
app is built, the hook adds the SDK's native library for each Android ABI
and iOS SDK the build targets. It fetches each one from the release this
package was cut from and checks it against the SHA-256 digest this package
carries before writing it.

```yaml
dependencies:
  teistro_flutter: ^0.1.0
```

```dart title="bindings/flutter/example/lib/main.dart"
import 'package:teistro_flutter/teistro_flutter.dart';
// …
    final teistro = Teistro.open();
```

`Teistro.open()` finds the library the build hook bundled.

`package:teistro_flutter/teistro_flutter.dart` re-exports
`package:teistro/teistro.dart`, so the whole API is available through this
one dependency. You do not need to run `dart run teistro:install`.

## What it bundles

| build | release slice | floor |
|---|---|---|
| Android arm64-v8a | `android-arm64` | API 24, 16 KB pages |
| Android armeabi-v7a | `android-arm` | API 24 |
| Android x86_64 | `android-x64` | API 24, 16 KB pages |
| iOS device | `ios-arm64` | iOS 13.0 |
| iOS simulator (Apple silicon) | `ios-sim-arm64` | iOS 14.0 |
| iOS simulator (Intel) | `ios-sim-x64` | iOS 13.0 |

The build is refused, with a sentence saying how to fix it, when:

- the app's `minSdk` is below 24 or its iOS deployment target is below 13;
- the build targets an ABI the release does not ship (ia32 or RISC-V
  Android);
- a downloaded library does not match its digest.

Desktop Flutter apps and Dart programs get nothing bundled. They find the
library the way `package:teistro` describes.

Each library is linked against the system libraries alone (no
`libc++_shared.so`). It is unsigned; Flutter signs it with the app's
identity.

What a library adds to an app, measured in this repository's own builds:
13.2 MB on arm64-v8a, 9.2 MB on armeabi-v7a and 14.5 MB on x86_64 as
Gradle stores them in the APK (a Play bundle delivers one ABI to each
device), and 12.8 MB in the iOS device app's framework. Every release
archive is about 5.6 MB gzipped.

## How it is tested

Every slice is built as the release builds it, and the example app in
`example/` is built on them: the APK must hold each ABI's library stored
and 16 KB-aligned where Play requires it, and the iOS app's framework must
hold the device slice alone, each still passing the floor check the slice
was built to. The app's integration test then runs on an Android emulator
at API 24 and on an iPhone simulator, asserting the same four facts the
published Dart package's consumer asserts: an optimised build of this
version, 14 April 2015 as 2072-1-1 BS, a Nepali message rendered, and the
Sun's place at J2000. The emulator and simulator also compute the SDK's
whole scenario, which must match Linux x86-64's value for value, as every
other platform's does.

## Without the built-in ephemeris

An app that plugs a real engine in, such as Teimeris through
`teistro_ephemeris_teimeris_flutter`, never opens the built-in ephemeris.
It can bundle the library built without it:

```yaml
hooks:
  user_defines:
    teistro_flutter:
      builtin: false
```

The release ships that slice beside each default one, under the platform's
name with `-no-builtin` after it, with its own digest. The hook checks it
the same way and caches it apart. Everything else in the library is the
same, so a context asking for `Ephemeris.builtin` is refused, and one over
a plugged-in engine answers as before. What it saves on each platform is
measured in `docs/05-testing/mobile-sizes-measured.md`. Leave it out, or
write `true`, for the default. A library named with `library` (below) is
bundled as it is, whatever this says.

## Offline builds and your own library

Set these under `hooks: user_defines: teistro_flutter:` in the app's (or
the workspace's) `pubspec.yaml`:

```yaml
hooks:
  user_defines:
    teistro_flutter:
      # The release's .gz archives, read from a directory rather than
      # fetched. Each is still checked against its digest.
      prebuilt_dir: third_party/teistro
      # Or a library you built yourself, bundled without a digest check.
      # The SDK still checks at load that it is this version's build.
      # library: ../teistro-sdk/target/aarch64-linux-android/release/libteistro_ffi.so
```

One `library` path is one file for every platform the build targets, which
suits `flutter run` on one device or simulator. A build for several
architectures runs the hook once for each and needs one library per
platform: `flutter build ios --simulator` builds `ios-sim-arm64` and
`ios-sim-x64` and joins the two with `lipo`, and an APK for several ABIs
holds one library per ABI. Name them with a map keyed by release platform
(`android-arm64`, `android-arm`, `android-x64`, `ios-arm64`,
`ios-sim-arm64`, `ios-sim-x64`):

```yaml
hooks:
  user_defines:
    teistro_flutter:
      library:
        ios-sim-arm64: ../teistro-sdk/target/aarch64-apple-ios-sim/release/libteistro_ffi.dylib
        ios-sim-x64: ../teistro-sdk/target/x86_64-apple-ios/release/libteistro_ffi.dylib
        android-arm64: ../teistro-sdk/target/aarch64-linux-android/release/libteistro_ffi.so
```

A relative path is read from the directory of the pubspec that declares
it, in either form. A map that does not name the platform being built is
refused with the line to add. The hook reads each library's ELF or Mach-O
header and refuses one built for another processor, which is how a single
path given to a multi-architecture build fails, before `lipo` does. It
cannot tell an iOS device library from an arm64 simulator one, which the
loader reports instead.

Downloads honour `HTTPS_PROXY` and `NO_PROXY`. The hook caches each slice
in Flutter's shared build directory and reuses it while its digest still
matches.
