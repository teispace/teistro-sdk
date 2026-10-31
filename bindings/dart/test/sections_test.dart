// The chart sections the rest of the suite asks for and never reads: the
// drishti between the grahas, the derived points and the divisional
// charts, each held to the rule that makes it rather than to a fixture.

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final Teistro teistro = Teistro.open();

final Observer kathmandu = Observer(
  latitudeDeg: Latitude(27.7172),
  longitudeDeg: Longitude(85.324),
  altitudeM: Altitude(1400),
);

/// The houses a graha casts its full drishti on, counted from its own
/// sign: every graha the seventh, and Mars, Jupiter and Saturn their
/// specials.
const Map<Graha, Set<int>> _casts = {
  Graha.sun: {7},
  Graha.moon: {7},
  Graha.mercury: {7},
  Graha.venus: {7},
  Graha.mars: {4, 7, 8},
  Graha.jupiter: {5, 7, 9},
  Graha.saturn: {3, 7, 10},
};

int _sign(double longitudeDeg) => longitudeDeg ~/ 30;

void main() {
  late Context ctx;
  setUp(() {
    ctx = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
  });
  tearDown(() => ctx.dispose());

  Chart found(double instant, {bool asked = true}) => ctx.chart.found(
    instant: instant,
    place: kathmandu,
    utcOffsetSeconds: 20700,
    aspects: asked,
    points: asked,
    vargas: asked ? const [Varga.d9, Varga.d3] : const <Varga>[],
  );

  const instants = [2460000.25, 2451545.0, 2455197.75];

  test('a chart reads its sections only when asked', () {
    final bare = found(instants.first, asked: false);
    expect(bare.aspects, isEmpty);
    expect(bare.points, isEmpty);
    expect(bare.vargas, isEmpty);
  });

  test("a graha's drishti falls the houses its rule counts", () {
    var read = 0;
    for (final instant in instants) {
      final chart = found(instant);
      final signOf = {
        for (final placed in chart.grahas)
          placed.graha: _sign(placed.longitudeDeg),
      };
      expect(chart.aspects, isNotEmpty);
      for (final drishti in chart.aspects) {
        expect(drishti.from, isNot(drishti.to));
        final counted =
            (signOf[drishti.to]! - signOf[drishti.from]! + 12) % 12 + 1;
        expect(
          drishti.houses,
          counted,
          reason: '${drishti.from} to ${drishti.to} counts the signs between',
        );
        if ((drishti.strength, _casts[drishti.from]) case (
          Strength.full,
          final houses?,
        )) {
          expect(houses, contains(drishti.houses), reason: '${drishti.from}');
          read += 1;
        }
        for (final edge in [drishti.fromEdge, drishti.toEdge]) {
          // To the nearer edge: at most half of each.
          expect(edge.signDeg, inInclusiveRange(0, 15));
          expect(edge.nakshatraDeg, inInclusiveRange(0, 20 / 3));
          expect(edge.padaDeg, inInclusiveRange(0, 5 / 3));
        }
      }
    }
    expect(read, greaterThan(0));
  });

  test('a derived point stands in the sign its longitude names', () {
    for (final instant in instants) {
      final points = found(instant).points;
      expect(points, isNotEmpty);
      expect(
        points.map((at) => at.point).toSet(),
        hasLength(points.length),
        reason: 'each point once',
      );
      for (final at in points) {
        expect(at.longitudeDeg, inInclusiveRange(0, 360));
        expect(
          at.sign,
          Rashi.byId(_sign(at.longitudeDeg)),
          reason: '${at.point}',
        );
        expect(at.boundaries.signDeg, inInclusiveRange(0, 15));
      }
    }
  });

  test('a divisional chart places each graha by its part of the sign', () {
    for (final instant in instants) {
      final chart = found(instant);
      expect(chart.vargas.map((one) => one.varga), [Varga.d9, Varga.d3]);
      final navamsha = chart.vargas.first;
      expect(
        navamsha.grahas.map((one) => one.graha),
        chart.grahas.map((one) => one.graha),
        reason: 'the same grahas, in the same order',
      );
      expect(navamsha.lagna.rashi, Rashi.byId(_sign(chart.lagnaDeg)));
      expect(
        navamsha.lagna.sign,
        Rashi.byId((chart.lagnaDeg ~/ (30 / 9)) % 12),
        reason: 'a navamsha is a ninth of a sign, counted on from Aries',
      );
      for (final (k, placed) in chart.grahas.indexed) {
        final at = navamsha.grahas[k].at;
        expect(at.rashi, Rashi.byId(_sign(placed.longitudeDeg)));
        expect(at.sign, Rashi.byId((placed.longitudeDeg ~/ (30 / 9)) % 12));
      }
      for (final placed in chart.vargas.last.grahas) {
        expect(placed.at.part, inInclusiveRange(0, 3));
      }
    }
  });

  test('a batch reads each chart as the chart alone reads', () {
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      aspects: true,
      points: true,
      vargas: const [Varga.d9],
    );
    for (final (k, instant) in instants.indexed) {
      final one = ctx.chart.found(
        instant: instant,
        place: kathmandu,
        utcOffsetSeconds: 20700,
        aspects: true,
        points: true,
        vargas: const [Varga.d9],
      );
      final many = batch.at(k);
      expect(
        [for (final d in many.aspects) (d.from, d.to, d.houses, d.strength)],
        [for (final d in one.aspects) (d.from, d.to, d.houses, d.strength)],
      );
      expect(
        [for (final p in many.points) (p.point, p.longitudeDeg)],
        [for (final p in one.points) (p.point, p.longitudeDeg)],
      );
      expect(
        [
          for (final g in many.vargas.single.grahas)
            (g.graha, g.at.rashi, g.at.part, g.at.sign),
        ],
        [
          for (final g in one.vargas.single.grahas)
            (g.graha, g.at.rashi, g.at.part, g.at.sign),
        ],
      );
    }
  });
}
