// What this package's README tells a consumer to write, held to the
// analyser at the package's own strictness.
//
// It is analysed and never run: `cargo xtask check-dart` analyses this
// package, so a README snippet that stopped compiling — a renamed
// descriptor, a moved façade, a changed chain — fails a gate rather than
// misleading a reader. Running it would need the engine's data, which a
// checkout does not have.

import 'package:teistro/teistro.dart';
import 'package:teistro_ephemeris_teimeris/teistro_ephemeris_teimeris.dart';

void main() {
  final teistro = Teistro.open();

  // A real engine, and the SDK's own only if it is not there.
  final ctx = teistro.context(
    profile: 'parashari-classical',
    ephemeris: [
      teimeris(dataDir: './ephe'),
      const NamedEphemeris(Ephemeris.builtin),
    ],
  );

  // The eight operations the SDK names, at the areas it groups them by.
  final sky = ctx.positions(instants: [2451545.0], bodies: [Body.sun]);
  print('the Sun is at ${sky.at(0, 0).longitude}');

  // And the engine's own, typed by the extension this package adds: one
  // value comes back as itself, and more than one as a record.
  final String name = ctx.engine.tmBodyName(body: 0);
  final double seconds = ctx.engine.tmDeltaT(jdUt1: 2451545.0);
  final ({int major, int minor, int patch}) version = ctx.engine.tmVersion();
  print('$name, $seconds seconds, engine $version');

  ctx.dispose();
}
