// A chart reading: one call for the whole document a reader interprets.
//
// `birth_chart.dart` placed the grahas. A reading is what comes after: the
// divisional charts, the houses, what each graha **is** rather than where
// it is, which grahas look at which, and the derived points. Each is a
// section a request asks for by name, and each is computed from the same
// founded chart in the same crossing — so asking for all of them costs one
// call, and asking for none of them costs nothing
// (`docs/03-design/chart-reading.md`).
//
// What it teaches:
//
// 1. **Sections are asked for.** `vargas`, `aspects`, `points`, `houses`
//    and `state` are off by default, so a birth chart does not pay for
//    twenty-one divisional charts it will not show.
// 2. **Vargottama is a comparison, not a flag**: a graha whose navamsha
//    sign is the sign it stands in. The layer gives both signs.
// 3. **A dignity and a house are different questions**, answered by
//    different sections: `states` says how a graha fares in its sign,
//    `bhavas` who rules a house.
// 4. **The drishti are ragged**: how many there are depends on where the
//    grahas stand, not on how many grahas there are.
//
// The record is `birth_chart.dart`'s own, so the two can be read side by
// side.

import 'package:teistro/teistro.dart';

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );
  String name(String fullKey) => ctx.intl.entity(fullKey).name;

  final born = Calendar.bikramSambat.date(2042, 9, 17);
  final when = ctx.time.resolve(
    born.at(hour: 0, minute: 20),
    ianaZone('Asia/Kathmandu'),
  );

  // ── One call for every section ──────────────────────────────────────
  final chart = ctx.chart.found(
    instant: when.instantJdUtc,
    place: Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    ),
    utcOffsetSeconds: when.offsetSeconds,
    vargas: const [Varga.d9, Varga.d10],
    aspects: true,
    points: true,
    houses: true,
    state: true,
  );
  final navamsha = chart.vargas[0];
  final dasamsha = chart.vargas[1];

  print('reading  BS 2042-09-17  00:20  Kathmandu');
  final lagna = Rashi.byId(chart.lagnaDeg ~/ 30);
  print(
    'lagna    ${name(lagna.fullKey)} ${(chart.lagnaDeg % 30).toStringAsFixed(4)}°'
    '   D9 ${name(navamsha.lagna.sign.fullKey)}'
    '   D10 ${name(dasamsha.lagna.sign.fullKey)}',
  );

  // ── What each graha is ──────────────────────────────────────────────
  print('');
  print(
    'graha        house  dignity          age            D9 sign      vargottama  combust',
  );
  print('─' * 88);
  final states = chart.states;
  for (var j = 0; j < states.length; j += 1) {
    final state = states[j];
    final inNavamsha = navamsha.grahas[j].at;
    final vargottama = inNavamsha.sign == inNavamsha.rashi ? 'yes' : 'no';
    print(
      '${name(state.graha.fullKey).padRight(12)} ${'${state.house}'.padLeft(5)}  '
      '${name(state.dignity.fullKey).padRight(16)} '
      '${name(state.age.fullKey).padRight(14)} '
      '${name(inNavamsha.sign.fullKey).padRight(12)} ${vargottama.padRight(11)} '
      '${state.combustion.burning.key}',
    );
  }

  // ── The houses ──────────────────────────────────────────────────────
  // The tenth house, by the houses service: the sign its middle falls in,
  // that sign's lord, and which kind of house it is.
  final tenth = chart.bhavas[9];
  print('');
  print(
    'bhava 10 ${name(tenth.sign.fullKey)}, ruled by ${name(tenth.lord.fullKey)}'
    ' (${tenth.quadrant.key})',
  );

  // ── Which grahas look at the Moon ───────────────────────────────────
  // `houses` counts inclusively from the looking graha's sign, so the
  // seventh is the house opposite it.
  final aspects = chart.aspects;
  print(
    'drishti  ${aspects.length} under ${chart.batch.drishtiTable}; on the Moon:',
  );
  for (final drishti in aspects.where((one) => one.to == Graha.moon)) {
    print(
      '         ${name(drishti.from.fullKey).padRight(12)}'
      ' house ${'${drishti.houses}'.padLeft(2)} from it  ${drishti.strength.key}',
    );
  }

  // ── The derived points ──────────────────────────────────────────────
  // Gulika is Saturn's portion of the day's arc, which is why a chart with
  // no day to divide has none — the section is ragged for that reason.
  final points = chart.points;
  final gulika = points.where((one) => one.point == Point.gulika).firstOrNull;
  final where =
      gulika == null
          ? 'none'
          : '${name(gulika.sign.fullKey)} ${(gulika.longitudeDeg % 30).toStringAsFixed(4)}°';
  print('gulika   $where   ${points.length} points');
  print('settings hash  ${ctx.settingsHash.substring(0, 16)}…');
  ctx.dispose();
}
