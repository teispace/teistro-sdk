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

  // A struct crosses as a class, both ways; one the engine takes a null
  // for may be left out.
  final TmDatetime utc = ctx.engine.tmLocalToUtc(
    local: const TmDatetime(
      year: 2026,
      month: 9,
      day: 13,
      hour: 6,
      minute: 30,
      second: 0,
    ),
    utcOffsetHours: 5.75,
    cal: 1,
  );
  final orbit = ctx.engine.tmNodesApsidesCalc(
    jd: 2461296.5,
    scale: 1,
    body: 4,
    flags: 0,
    method: 0,
    apsis: 0,
  );
  print('${utc.hour}:${utc.minute} UTC, perihelion ${orbit.perihelion.lon}');

  // An array crosses as a list: as long as the inputs, as many as asked,
  // or as many as there are.
  final List<double> deltas = ctx.engine.tmDeltaTMany(
    jdsUt1: [2451545, 2461296.5],
  );
  final List<TmPosition> grid = ctx.engine.tmPositionCalcGrid(
    bodies: [0, 1],
    jds: [2451545],
    scale: 1,
    flags: 0,
  );
  final List<int> defaults = ctx.engine.tmChartDefaultBodies();
  print(
    '${deltas.first} s, the Sun at ${grid.first.lon}, ${defaults.length} bodies',
  );

  // A struct that points at another takes it as a nested object, or null.
  final query = ctx.engine.tmStarQueryInitSized();
  final List<TmStar> stars = ctx.engine.tmStarSearch(
    query: TmStarQuery(
      useMagnitude: query.useMagnitude,
      magnitudeMin: query.magnitudeMin,
      magnitudeMax: query.magnitudeMax,
      raDeg: query.raDeg,
      decDeg: query.decDeg,
      radiusDeg: query.radiusDeg,
      nameContains: 'Aldeb',
    ),
  );
  final TmPosition moon = ctx.engine.tmPositionCalc(
    req: const TmPositionRequest(
      jd: 2451545,
      scale: 1,
      body: 1,
      flags: 0,
      observer: null,
      center: 0,
      ayanamsha: 0,
      ayanamshaSet: 0,
    ),
  );
  print('${stars.length} stars, the Moon at ${moon.lon}');

  // Cusps are as long as the house system says, asked before the call.
  final houses = ctx.engine.tmHousesCalc(
    req: const TmHousesRequest(
      jdUt1: 2451545,
      geoLatDeg: 27.7172,
      geoLonDeg: 85.324,
      system: 0,
      flags: 0,
    ),
  );
  final List<double> cusps = houses.cusps;
  print('${cusps.length} cusps, ascendant ${houses.outAngles.ascendant}');

  ctx.dispose();
}
