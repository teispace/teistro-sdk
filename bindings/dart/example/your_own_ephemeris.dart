// An ephemeris of your own, and what happens when it goes wrong.
//
// The SDK computes no positions itself: it asks a **provider**, and a
// provider written in Dart is a first-class one. That is the point of
// the port — an application that already has an ephemeris, a cache, or a
// table of precomputed positions can put it behind the SDK and get the
// whole chart layer for free.
//
// The contract is small and worth reading carefully:
//
// - **One call for the whole grid, never a loop.** The SDK hands over
//   every instant and every body at once and expects every cell back.
// - **Answer `null` for "not in that frame".** Answering at all asserts
//   the answer is in the frame that was asked for, so a provider that
//   computes in one frame must compare `query.frameBits` with its own
//   and return `null` instead; the SDK then asks again in `nativeFrame`
//   and completes the rest itself, stamping each step. This is why an
//   engine that knows nothing about the ayanamsha can serve a Vedic
//   chart.
// - **Say what you cover.** `bodies`, `jdMin` and `jdMax` are checked
//   *before* the provider is called, so a request it cannot serve is
//   refused by name rather than by a wrong answer.
// - **Throwing is allowed.** What it threw is carried across the
//   boundary and rethrown on the caller's side, so the sentence is not
//   lost; only a code crosses the C ABI.

import 'dart:convert';
import 'dart:typed_data';

import 'package:teistro/teistro.dart';

/// An ephemeris backed by whatever you already have.
///
/// This one is a two-body toy — a circular Sun and Moon — standing in
/// for the real thing: a `.se1` reader, a JPL kernel, a database of
/// precomputed rows, or a cache in front of any of them. What matters is
/// the shape, not the arithmetic.
base class TableEphemeris extends EphemerisProvider {
  TableEphemeris({this.wantedFrame});

  /// The one frame this provider computes in, as the port packs it.
  /// `null` means "answer whatever is asked", which a provider may say
  /// only when it really can.
  final int? wantedFrame;

  int calls = 0;
  int cells = 0;
  int refusals = 0;

  @override
  String get name => 'table-ephemeris';

  @override
  String get version => '1.0.0';

  /// What identifies the data, not the code. A result's provenance
  /// carries it, so two runs against different data are distinguishable
  /// even when the code is identical.
  @override
  String get dataVersion => 'demo-rows-2025a';

  @override
  List<Body> get bodies => const [Body.sun, Body.moon];

  /// Cover only what you have. A request outside this is refused before
  /// [positions] is ever called.
  @override
  double get jdMin => 2451545.0;

  @override
  double get jdMax => 2469807.0;

  @override
  PositionAnswer? positions(PositionQuery query) {
    // **Check the frame first.** Answering at all asserts that the
    // answer is in the frame that was asked for; a provider that
    // computes only its own must say so by returning null.
    if (wantedFrame != null && query.frameBits != wantedFrame) {
      refusals += 1;
      return null;
    }

    calls += 1;
    cells += query.cellCount;

    // Cells run instants outermost: cell `i * bodies.length + j` is
    // instant `i`, body `j`. Building the columns in that order is the
    // whole of the contract.
    final lon = Float64List(query.cellCount);
    final lonSpeed = Float64List(query.cellCount);
    var at = 0;
    for (final jd in query.jds) {
      final days = jd - 2451545.0;
      for (final body in query.bodies) {
        final rate = body == Body.sun ? 0.9856 : 13.1764;
        final start = body == Body.sun ? 280.46 : 218.32;
        lon[at] = (start + rate * days) % 360.0;
        lonSpeed[at] = rate;
        at += 1;
      }
    }
    return PositionAnswer(
      lon: lon,
      lat: Float64List(query.cellCount),
      dist: Float64List(query.cellCount)..fillRange(0, query.cellCount, 1.0),
      // Speeds are optional; a column left out is zeroes. Saying
      // `speeds => false` would tell the SDK not to expect them at all.
      lonSpeed: lonSpeed,
    );
  }
}

/// The sentence inside a refusal, whatever kind it is.
///
/// A library refusal and an error raised on this side of the boundary
/// both carry one, but Dart gives them no common supertype that says so,
/// so the shape is matched rather than the type named.
String reason(Object error) => switch (error) {
  TeistroException(:final message) => message,
  StateError(:final message) => message,
  _ => '$error',
};

/// A provider that fails the way a real one does: with a sentence.
base class Broken extends TableEphemeris {
  Broken();

  @override
  String get name => 'broken';

  @override
  PositionAnswer? positions(PositionQuery query) {
    throw StateError('ephemeris file de431.eph is not where the index says');
  }
}

void main() {
  final teistro = Teistro.open();

  // ── The happy path ─────────────────────────────────────────────────
  {
    final provider = TableEphemeris();
    final ctx = teistro.context(
      profile: 'parashari-classical',
      provider: provider,
    );
    final sky = ctx.positions(
      instants: [for (var day = 0; day < 7; day++) 2451545.0 + day],
      bodies: [Body.sun, Body.moon],
    );
    print('asked    ${provider.calls} time(s) for ${provider.cells} cells');
    print('answered ${sky.cells.length} cells over ${sky.jdCount} days');
    print(
      '  sun  ${sky.at(0, 0).longitude.toStringAsFixed(4).padLeft(8)}°'
      ' -> ${sky.at(6, 0).longitude.toStringAsFixed(4).padLeft(8)}° in a week',
    );
    print(
      '  moon ${sky.at(0, 1).longitude.toStringAsFixed(4).padLeft(8)}°'
      ' -> ${sky.at(6, 1).longitude.toStringAsFixed(4).padLeft(8)}° in a week',
    );
    // The provider's own name and data version are stamped on the
    // answer, which is how a stored chart says what computed it.
    print('  stamped as ${jsonEncode(sky.provenanceOf['provider'])}');
    ctx.dispose();
  }

  // ── A body it never declared ───────────────────────────────────────
  {
    final provider = TableEphemeris();
    final ctx = teistro.context(
      profile: 'parashari-classical',
      provider: provider,
    );
    try {
      ctx.positions(instants: [2451545.0], bodies: [Body.saturn]);
    } catch (error) {
      print('');
      print('refused  ${reason(error)}');
      print('         and the provider was asked ${provider.calls} times');
    }
    ctx.dispose();
  }

  // ── An instant outside its coverage ────────────────────────────────
  {
    final ctx = teistro.context(
      profile: 'parashari-classical',
      provider: TableEphemeris(),
    );
    try {
      ctx.positions(instants: [2200000.0], bodies: [Body.sun]);
    } catch (error) {
      print('refused  ${reason(error)}');
    }
    ctx.dispose();
  }

  // ── A frame it does not compute ────────────────────────────────────
  // This provider computes tropical positions and declares no native
  // frame, so the canonical one is what it answers. Ask for a sidereal
  // zodiac and the SDK does the rest, naming every step it applied.
  {
    final canonical = teistro.canonicalFrame;
    final provider = TableEphemeris(wantedFrame: teistro.packFrame(canonical));
    final ctx = teistro.context(
      profile: 'parashari-classical',
      provider: provider,
    );
    final tropical = ctx.positions(instants: [2451545.0], bodies: [Body.sun]);
    final sidereal = ctx.positions(
      instants: [2451545.0],
      bodies: [Body.sun],
      frame: Frame(
        ayanamsha: Ayanamsha.lahiri,
        centre: canonical.centre,
        equinox: canonical.equinox,
        coordinates: canonical.coordinates,
        sidereal: true,
        lightTime: canonical.lightTime,
        aberration: canonical.aberration,
        deflection: canonical.deflection,
        nutation: canonical.nutation,
      ),
    );
    final steps = sidereal.stepsApplied
        .cast<Map<String, Object?>>()
        .map((step) => '${step['name']}:${step['implementation']}')
        .join(', ');
    print('');
    print(
      'frames   the provider answered'
      ' ${tropical.at(0, 0).longitude.toStringAsFixed(4)}° tropical;'
      ' a sidereal request is'
      ' ${sidereal.at(0, 0).longitude.toStringAsFixed(4)}°',
    );
    print(
      '         it refused the frame ${provider.refusals} time(s), and the',
    );
    print('         SDK completed it: $steps');
    ctx.dispose();
  }

  // ── When the provider itself fails ─────────────────────────────────
  {
    final ctx = teistro.context(
      profile: 'parashari-classical',
      provider: Broken(),
    );
    try {
      ctx.positions(instants: [2451545.0], bodies: [Body.sun]);
    } on StateError catch (error) {
      print('');
      print('thrown   StateError: ${error.message}');
      print('         the error itself crossed back, not just a code');
    }
    ctx.dispose();
  }

  // ── A refusal a user should see ────────────────────────────────────
  // Every refusal from the library carries a status a program can match
  // on and, where the boundary knows one, the detail and a hint to act
  // on. That is what to put in front of a person.
  {
    final ctx = teistro.context(profile: 'parashari-classical');
    try {
      ctx.keys.id('graha.SUNN');
    } on TeistroException catch (error) {
      print('');
      print('status   ${error.status.key}');
      print('message  ${error.message}');
      print('detail   ${error.detail ?? ''}');
      print('hint     ${error.hint ?? ''}');
      print(
        '         a program matches on `status`; a person reads the'
        ' message and the hint',
      );
    }
    ctx.dispose();
  }
}
