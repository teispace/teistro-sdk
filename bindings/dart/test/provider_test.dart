// An ephemeris written in Dart, answering the SDK through the port's
// vtable: the same scenarios the Node binding's provider tests walk.

import 'dart:typed_data';

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final Teistro teistro = Teistro.open();

/// A provider whose longitudes are a straight line per body, so a cell's
/// value says which instant and which body it came from.
final class StraightLine extends EphemerisProvider {
  StraightLine({
    this.answers = const [Body.sun, Body.moon],
    this.nativeFrame,
    this.onCall,
  });

  final List<Body> answers;

  @override
  final Frame? nativeFrame;

  /// Called with every query, so a test can count them.
  final void Function(PositionQuery query)? onCall;

  final List<PositionQuery> calls = [];

  @override
  String get name => 'a-provider-in-dart';

  @override
  String get version => '1.2.3';

  @override
  List<Body> get bodies => answers;

  @override
  PositionAnswer? positions(PositionQuery query) {
    calls.add(query);
    onCall?.call(query);
    if (nativeFrame != null &&
        query.frameBits != teistro.packFrame(nativeFrame!)) {
      // Nothing means "not in that frame"; the SDK asks again in ours.
      return null;
    }
    final cells = query.cellCount;
    final lon = Float64List(cells);
    final lonSpeed = Float64List(cells);
    for (var i = 0; i < query.jds.length; i++) {
      for (var b = 0; b < query.bodies.length; b++) {
        final k = i * query.bodies.length + b;
        lon[k] = (query.jds[i] + b * 100) % 360;
        lonSpeed[k] = b == 0 ? 0.9856 : 13.176;
      }
    }
    return PositionAnswer(
      lon: lon,
      lat: Float64List(cells),
      dist: Float64List(cells)..fillRange(0, cells, 1),
      lonSpeed: lonSpeed,
    );
  }
}

void main() {
  test('an ephemeris written in Dart answers the SDK', () {
    final provider = StraightLine();
    final ctx = teistro.context(profile: 'nepali-default', provider: provider);
    addTearDown(ctx.dispose);

    final positions = ctx.positions(
      instants: [2451545.0, 2451546.0],
      bodies: [Body.sun, Body.moon],
    );

    // One call for the whole grid: the port is a batch call, not a loop.
    expect(provider.calls, hasLength(1));
    final asked = provider.calls.single;
    expect(asked.jds, [2451545.0, 2451546.0]);
    expect(asked.bodies, [Body.sun, Body.moon]);
    expect(asked.scale, TimeScale.ut1);
    expect(asked.speeds, isTrue);
    expect(
      asked.observer,
      isNull,
      reason: 'a geocentric frame needs no observer',
    );

    expect(positions.cells.length, 4);
    expect(positions.at(0, 0).longitude, closeTo(2451545 % 360, 1e-9));
    expect(positions.at(0, 1).longitudeSpeed, 13.176);
    final provenance =
        positions.provenanceOf['provider']! as Map<String, Object?>;
    expect(provenance['name'], 'a-provider-in-dart');
    expect(provenance['version'], '1.2.3');
    expect(ctx.provider, same(provider));
  });

  test('a provider that refuses a frame is completed by the SDK', () {
    final equatorial = Frame(
      centre: teistro.canonicalFrame.centre,
      equinox: teistro.canonicalFrame.equinox,
      coordinates: Coordinates.equatorial,
      sidereal: false,
      lightTime: teistro.canonicalFrame.lightTime,
      aberration: teistro.canonicalFrame.aberration,
      deflection: teistro.canonicalFrame.deflection,
      nutation: teistro.canonicalFrame.nutation,
    );
    final provider = StraightLine(
      answers: const [Body.sun],
      nativeFrame: equatorial,
    );
    final ctx = teistro.context(provider: provider);
    addTearDown(ctx.dispose);

    final positions = ctx.positions(
      instants: [2451545.0],
      bodies: [Body.sun],
      frame: teistro.canonicalFrame,
    );
    expect(
      provider.calls,
      hasLength(2),
      reason: "the canonical frame, then the provider's own",
    );
    expect(
      positions.stepsApplied.cast<Map<String, Object?>>().map(
        (step) => '${step['name']}:${step['implementation']}',
      ),
      [
        'positions:NATIVE',
        'delta-t:SDK',
        'obliquity:SDK',
        'rotate-equatorial-to-ecliptic:SDK',
      ],
    );
  });

  test('a provider that fails says so in its own words', () {
    final ctx = teistro.context(
      provider: _Throwing('no data for that instant'),
    );
    addTearDown(ctx.dispose);
    expect(
      () => ctx.positions(instants: [2451545.0], bodies: [Body.sun]),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.provider)
            .having(
              (e) => e.message,
              'message',
              contains('no data for that instant'),
            ),
      ),
    );
  });

  test('a provider answers only the bodies and the instants it '
      'declared', () {
    final ctx = teistro.context(provider: StraightLine(answers: [Body.sun]));
    addTearDown(ctx.dispose);
    expect(
      () => ctx.positions(instants: [2451545.0], bodies: [Body.sun, Body.mars]),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.status,
          'status',
          anyOf(Status.unsupported, Status.capability),
        ),
      ),
    );
    expect(
      () => ctx.positions(instants: [1e9], bodies: [Body.sun]),
      throwsA(isA<TeistroException>()),
    );
  });

  test('a provider is checked at the door', () {
    expect(
      () => teistro.context(provider: _Nameless()),
      throwsA(isA<ArgumentError>()),
    );
    expect(
      () => teistro.context(provider: StraightLine(answers: const [])),
      throwsA(isA<ArgumentError>()),
    );
  });

  test('a short column is refused rather than read past its end', () {
    final ctx = teistro.context(provider: _Short());
    addTearDown(ctx.dispose);
    expect(
      () => ctx.positions(instants: [2451545.0, 2451546.0], bodies: [Body.sun]),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.message,
          'message',
          contains('1 values in `lon` for 2 cells'),
        ),
      ),
    );
  });
}

final class _Throwing extends EphemerisProvider {
  _Throwing(this.why);

  final String why;

  @override
  String get name => 'throws';

  @override
  List<Body> get bodies => const [Body.sun];

  @override
  PositionAnswer? positions(PositionQuery query) => throw StateError(why);
}

final class _Nameless extends EphemerisProvider {
  @override
  String get name => '';

  @override
  List<Body> get bodies => const [Body.sun];

  @override
  PositionAnswer? positions(PositionQuery query) => null;
}

final class _Short extends EphemerisProvider {
  @override
  String get name => 'short';

  @override
  List<Body> get bodies => const [Body.sun];

  @override
  PositionAnswer positions(PositionQuery query) => PositionAnswer(
    lon: Float64List(1),
    lat: Float64List(query.cellCount),
    dist: Float64List(query.cellCount),
  );
}
