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
| `lib/teistro.dart` | the layer a consumer uses: finding the shared library and checking its ABI, the defaults, the JSON both ways, and the conveniences a generator cannot know are wanted | by hand |
| `test/` | the surface end to end, and the decoders against blobs the library produced | by hand |
| `example/` | the code this README shows, run by the gate so the two cannot drift | by hand |

## Using it

This is `example/teistro_example.dart`, which the gate runs:

```dart
import 'package:teistro/teistro.dart';

void main() {
  final teistro = Teistro.open();
  print('Teistro ${teistro.version}, ABI ${teistro.abi}');

  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );

  // 14 April 2015 is 1 Baisakh 2072 BS.
  final bs = ctx.convert(
    Calendar.gregorian.date(2015, 4, 14),
    Calendar.bikramSambat,
  );
  print('${bs.year}-${bs.month}-${bs.day} ${bs.era?.key}');

  // A Kathmandu birth time, with the metadata a stored chart keeps.
  final resolved = ctx.resolve(
    Calendar.gregorian.date(1986, 1, 1).at(hour: 0, minute: 20),
    ianaZone('Asia/Kathmandu'),
  );
  print(
    'JD ${resolved.instantJdUtc.toStringAsFixed(6)} UTC, '
    '${resolved.offsetSeconds} s, tzdb ${resolved.tzdbVersion}',
  );

  // The Sun and the Moon at J2000, in the SDK's canonical frame.
  final sky = ctx.positions(
    instants: [2451545.0],
    bodies: [Body.sun, Body.moon],
  );
  print('the Sun at ${sky.at(0, 0).longitude.toStringAsFixed(4)} degrees');

  // A message in the context's locale.
  print(
    ctx.render('sdk.reason.grahaInBhava', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 7,
    }).text,
  );

  ctx.dispose();
}
```

`Teistro.open()` looks at `$TEISTRO_LIBRARY`, then beside the package,
then in the workspace's `target/release` and `target/debug`, and finally
asks the platform's loader; `Teistro.open(path: ...)` names one outright.
It refuses a library that implements another ABI.

A context frees its native memory when it is collected, so `dispose` is
the explicit form rather than the only one (ADR-0007).

The layer is thin on purpose. Anything it does not wrap is on the
generated types: `teistro.library` is the declarations, `context.inner`
the generated context, and every value class marshals itself.

## Running the tests

```sh
cargo xtask check-dart
```

That builds the C library, resolves the package's dependencies and runs
the tests. It needs the Dart SDK, so it runs by hand and in the nightly
matrix; the fast check needs the Rust toolchain and nothing else
(ADR-0014).

Never edit `lib/src/`: change the Rust source and regenerate.
