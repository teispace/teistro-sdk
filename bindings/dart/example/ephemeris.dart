// A year of the sky in one call, and what to do with the columns.
//
// The boundary takes a **grid** — instants by bodies — and answers with a
// result blob whose sections are columns. That shape is the whole reason
// a year of positions costs one crossing rather than 365, and it is what
// a service computing tables, transits or ingresses should be using.
//
// Three things this example is really about:
//
// 1. **One call, not a loop.** 366 instants by 3 bodies is 1098 cells in
//    a single request. Asking day by day would cross the boundary 366
//    times and recompute the provider's own setup each time.
// 2. **The columns are views, not copies.** `positions.cells.lon` is a
//    `Float64List` over the blob's own bytes, so a table of a million
//    cells costs one allocation rather than a million boxed doubles.
// 3. **What the answer says about itself.** Every result carries the
//    steps applied and a provenance envelope with the settings hash —
//    the two things a cache key and an audit trail are made of.
//
// `Ephemeris.builtin` computes with the analytic ephemeris the SDK
// carries, so this file runs anywhere with nothing installed. It is the
// fallback rather than the intended path — most consumers should be on a
// real engine — but it is astronomy: the scans below find sign ingresses
// and retrograde stations because the sky has them.

import 'dart:convert';
import 'dart:typed_data';

import 'package:teistro/teistro.dart';

/// A year from the start of 2025, one sample a day at noon UTC.
const double startJd = 2460676.5;
const int days = 366;

/// A body is what an ephemeris answers; a graha is what a chart names.
/// They are different catalogues and the lunar node is where they part —
/// `meanNode` is the body, `rahu` the graha — so the two are paired
/// explicitly rather than derived from each other's spelling.
const List<(Body, Graha)> bodies = [
  (Body.sun, Graha.sun),
  (Body.mars, Graha.mars),
  (Body.meanNode, Graha.rahu),
];

/// Every day on which a body changed sign.
///
/// Reads one body's column out of the grid. Cells run instants
/// outermost, so body `column` at day `i` is cell `i * stride + column`.
List<(int, Rashi)> ingresses(
  Float64List longitudes,
  int dayCount,
  int stride,
  int column,
) {
  final found = <(int, Rashi)>[];
  Rashi? previous;
  for (var day = 0; day < dayCount; day++) {
    final sign = Rashi.byId(longitudes[day * stride + column] ~/ 30);
    if (previous != null && sign != previous) found.add((day, sign));
    previous = sign;
  }
  return found;
}

/// Every day on which a body turned, direct to retrograde or back.
///
/// The same shape as [ingresses] over a different column: a station is a
/// sign change in `lonSpeed` rather than in `lon`. One grid answers
/// both, which is the reason to ask for a grid.
List<(int, String)> stations(
  Float64List speeds,
  int dayCount,
  int stride,
  int column,
) {
  final found = <(int, String)>[];
  for (var day = 1; day < dayCount; day++) {
    final before = speeds[(day - 1) * stride + column];
    final after = speeds[day * stride + column];
    if ((before < 0) != (after < 0)) {
      found.add((day, after < 0 ? 'retrograde' : 'direct'));
    }
  }
  return found;
}

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );

  // ── Which build am I talking to? ───────────────────────────────────
  // A service checks this once at start-up. The binding already refuses
  // a library that is not the build it was generated from; this is how
  // to log what it did load.
  final build = teistro.build;
  print(
    'library  Teistro ${build.sdk}  ABI ${build.abi}'
    '  catalogue ${build.catalogue}  ${build.target}'
    '  ${build.commit.substring(0, 8)}${build.dirty ? '-dirty' : ''}',
  );

  // ── One call for the whole year ────────────────────────────────────
  final canonical = teistro.canonicalFrame;
  final frame = Frame(
    ayanamsha: Ayanamsha.lahiri,
    centre: canonical.centre,
    equinox: canonical.equinox,
    coordinates: canonical.coordinates,
    sidereal: true,
    lightTime: canonical.lightTime,
    aberration: canonical.aberration,
    deflection: canonical.deflection,
    nutation: canonical.nutation,
  );
  final sky = ctx.positions(
    instants: [for (var day = 0; day < days; day++) startJd + day],
    bodies: [for (final (body, _) in bodies) body],
    frame: frame,
  );
  print(
    'grid     ${sky.jdCount} instants x ${sky.bodyCount} bodies'
    ' = ${sky.cells.length} cells in one call',
  );

  // ── The columns ────────────────────────────────────────────────────
  final cells = sky.cells;
  print(
    'columns  lon is a ${cells.lon.runtimeType}'
    ' of ${cells.lon.length} values, ${cells.lon.lengthInBytes} bytes',
  );

  // ── What the columns are for ───────────────────────────────────────
  print('');
  for (var column = 0; column < bodies.length; column++) {
    final (body, graha) = bodies[column];
    final name = ctx.intl.entity(graha.fullKey).name;
    final crossings = ingresses(cells.lon, days, sky.bodyCount, column);
    final turns = stations(cells.lonSpeed, days, sky.bodyCount, column);
    final speed = cells.lonSpeed[column];
    final direction = speed < 0 ? 'retrograde' : 'direct';
    print(
      '  ${body.key.padRight(10)} ${name.padRight(8)}'
      ' ${direction.padRight(10)} at'
      ' ${speed >= 0 ? '+' : ''}${speed.toStringAsFixed(4).padLeft(7)}°/day,'
      ' ${crossings.length} sign change(s), ${turns.length} station(s)',
    );
    for (final (day, sign) in crossings.take(3)) {
      final signName = ctx.intl.entity(sign.fullKey).name;
      print(
        '      day ${day.toString().padLeft(3)}'
        '  enters ${sign.key.padRight(12)} $signName',
      );
    }
    if (crossings.length > 3) {
      print('      … and ${crossings.length - 3} more');
    }
    for (final (day, into) in turns) {
      print('      day ${day.toString().padLeft(3)}  turns  $into');
    }
  }

  // ── What the answer says about itself ──────────────────────────────
  print('');
  final steps = sky.stepsApplied
      .cast<Map<String, Object?>>()
      .map((step) => '${step['name']}:${step['implementation']}')
      .join(', ');
  print('steps    $steps');
  final provenance = sky.provenanceOf;
  print('profile  ${provenance['profile']}');
  print('hash     ${provenance['settings_hash']}');
  print(
    '         two contexts with the same settings hash compute the same'
    ' numbers, so it is the cache key',
  );
  // The whole envelope is canonical JSON: byte-identical across every
  // binding, which is what makes it safe to hash and store.
  print('envelope ${jsonEncode(provenance).length} bytes of canonical JSON');

  ctx.dispose();
}
