# The Dart binding

Status: `built`, 2026-09-06.

Everything but two things is rendered from `idl/api.json` by `cargo xtask
gen ffi` and held equal to the boundary crates by `cargo xtask check-ffi`:
`lib/teistro.dart` and the tests are written by hand. The generated files
carry `// dart format off`, so the generator's layout is what ships and
`dart format .` still passes over the package.

| file | what it is | written by |
|---|---|---|
| `lib/src/catalogue.dart` | every enum as a Dart enum carrying the id the C boundary uses and the key every pack and fixture spells; a catalogue kind gains an `unknown` member so a `switch` stays exhaustive against a newer library | the generator |
| `lib/src/ffi.dart` | the `dart:ffi` declarations that match the C header name for name, a typed value class per boundary struct, the exception, and the context with a native finaliser | the generator |
| `lib/src/blob.dart` | one decoder per result blob, reading the `TSRB` layout into typed-data views over the blob's bytes | the generator |
| `lib/src/messages.dart`, `lib/messages.dart` | the typed accessors: every message of the SDK's locale as a function of its parameters, every catalogued entity as its forms (`cargo xtask gen intl`) | the generator, and a one-line entry point by hand |
| `lib/teistro.dart` | the layer a consumer uses: finding the shared library and checking its ABI, the defaults, the JSON both ways, and the conveniences a generator cannot know are wanted | by hand |
| `lib/src/host.dart` | the port adapter: an ephemeris written in Dart bound into the vtable through `NativeCallable.isolateLocal` | by hand |
| `test/` | the surface end to end, and the decoders against blobs the library produced | by hand |
| `example/` | the code this README shows, run by the gate so the two cannot drift | by hand |
| `bin/parity.dart` | this binding's half of the parity report, which `cargo xtask check-parity` compares with the Node binding's | by hand |
| `typecheck/wrong.dart` | the usages that must not compile, each with the error it must raise | by hand |
| `lib/src/sha256.dart` | SHA-256, so the installer can check a download without the package taking a dependency for it | by hand |
| `lib/src/install.dart`, `bin/install.dart` | the installer: where a prebuilt library comes from, the digest it must have, and where it is written | by hand |
| `lib/src/prebuilt.dart` | the release the installer fetches from and the digest of each platform's library; empty in a checkout, written when a release is staged | the release |
| `packaging/consumer.dart` | a consumer that uses the published package and the library its installer fetched, run by `cargo xtask check-package` | by hand |

## Installing it

```sh
dart pub add teistro
dart run teistro:install
```

The package carries no binaries: a pub package that shipped one for every
platform would make every consumer download all of them. `install` fetches
the shared library for this machine from the release this package's
version was cut from, checks it against a digest recorded when it was
built, and writes it to `.dart_tool/teistro/<version>/`, which is the
first place `Teistro.open()` looks.

The download is refused, and nothing is written, when the bytes are not
the ones that were built. On a machine with no network, install from a
file you already have:

```sh
dart run teistro:install --from libteistro_ffi-0.1.0-linux-x64.so.gz
```

`Teistro.open()` looks at `$TEISTRO_LIBRARY` first, then at what the
installer wrote, then beside the script, then in this repository's build
output, and finally asks the platform's loader for the bare name. Building
from source needs none of it: `cargo build --release -p teistro-ffi`.

## Using it

Six runnable programs live in [`example/`](example/), and
`cargo xtask check-dart` runs every one, so none of them can drift from what
the binding does. They are meant to be read in order — a quickstart, a
birth chart, a panchanga, a calendar page, a year of the sky, and an
ephemeris of your own — and [`example/README.md`](example/README.md) says
what each is really teaching.

```sh
cargo build --release -p teistro-ffi
cd bindings/dart
TEISTRO_LIBRARY=../../target/release/libteistro_ffi.dylib \
dart run example/quickstart.dart
```

The one thing to know before writing anything real is in
[`example/birth_chart.dart`](example/birth_chart.dart): **the canonical
frame is tropical**, because that is what an ephemeris computes. A Vedic
chart asks for a sidereal one and the SDK completes it, naming every step
it applied.

## An ephemeris of your own

Extend `EphemerisProvider` and give the context one. It is asked once for
a whole grid, never in a loop, and everything but the name, the bodies
and the positions has a default.

```dart
final class MyEphemeris extends EphemerisProvider {
  @override
  String get name => 'my-ephemeris';

  @override
  List<Body> get bodies => const [Body.sun, Body.moon];

  @override
  PositionAnswer? positions(PositionQuery query) {
    final cells = query.cellCount;
    // Return null for "not in that frame": the SDK then asks in your
    // native frame and completes the rest itself, stamping every step.
    return PositionAnswer(
      lon: Float64List(cells),
      lat: Float64List(cells),
      dist: Float64List(cells),
    );
  }
}

final ctx = teistro.context(provider: MyEphemeris());
```

A body, an observer or an instant the provider never declared is refused
before the call reaches it, by name. What the provider throws is what the
caller sees, because only a code crosses the C boundary and the adapter
keeps the sentence.

## Running the tests

```sh
cargo xtask check-dart
cargo xtask check-parity
```

The first builds the C library, resolves the package's dependencies and
runs the tests; the second walks one scenario through this binding and
the Node binding and compares the ninety values they report, so a
difference between the two layers is a failed gate rather than something
a reader has to notice. It needs the Dart SDK, so it runs by hand and in the nightly
matrix; the fast check needs the Rust toolchain and nothing else
(ADR-0014).

Never edit `lib/src/`: change the Rust source and regenerate.
