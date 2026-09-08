// A birth chart: from a Nepali birth record to the nine grahas placed.
//
// This is the scenario the SDK exists for, and it is not one call. What
// it takes, in order:
//
// 1. A birth record as people actually write one — a Bikram Sambat date,
//    a local clock time, and a place.
// 2. That civil time resolved to an **instant**, which needs the zone's
//    history: Nepal was +05:30 until 1986 and +05:45 after, and the
//    resolution says which rule it used and from which tzdb.
// 3. Positions in a **sidereal** frame. This is the step everyone gets
//    wrong. The SDK's canonical frame is *tropical*, because that is
//    what an ephemeris computes; a Vedic chart wants the sidereal
//    zodiac, so the request names one and the SDK completes it — and
//    stamps every step it applied, which this example prints.
// 4. Each longitude read as a rashi, a nakshatra and a pada, using the
//    catalogue's own members and the locale's own names.
//
// What it does not need: an ephemeris of your own. `testProvider: true`
// selects the analytic one the SDK carries so this file runs anywhere.

import 'package:teistro/teistro.dart';

/// The grahas of a Vedic chart, each paired with the body an ephemeris
/// answers for it. Ketu is not a body: it is Rahu's opposite point, so
/// it is computed rather than asked for, as the SDK's own `points`
/// module does.
const List<(Graha, Body)> grahas = [
  (Graha.sun, Body.sun),
  (Graha.moon, Body.moon),
  (Graha.mars, Body.mars),
  (Graha.mercury, Body.mercury),
  (Graha.jupiter, Body.jupiter),
  (Graha.venus, Body.venus),
  (Graha.saturn, Body.saturn),
  (Graha.rahu, Body.meanNode),
];

/// A nakshatra is a twenty-seventh of the circle; a pada a quarter of one.
const double nakshatraDeg = 360.0 / 27.0;
const double padaDeg = nakshatraDeg / 4.0;

/// One graha as a chart shows it.
final class Placement {
  const Placement(this.graha, this.longitude, this.speed);

  final Graha graha;
  final double longitude;
  final double speed;

  /// The sign it stands in.
  Rashi get rashi => Rashi.byId(longitude ~/ 30);

  /// How far into that sign, in degrees.
  double get degreeInRashi => longitude % 30.0;

  /// The lunar mansion it stands in.
  Nakshatra get nakshatra => Nakshatra.byId(longitude ~/ nakshatraDeg);

  /// Which quarter of that mansion, 1 to 4.
  int get pada => (longitude % nakshatraDeg) ~/ padaDeg + 1;

  /// Whether it is moving backwards. There is no flag at the boundary: a
  /// graha is retrograde when its longitude is decreasing, which is what
  /// the speed column says. Rahu always is.
  bool get retrograde => speed < 0;
}

/// Every graha at one instant, in the sidereal zodiac.
///
/// One call for the whole grid, never a loop: the boundary takes the
/// instants and the bodies together and answers with columns, so asking
/// for eight grahas costs one crossing rather than eight.
List<Placement> chart(Teistro teistro, Context ctx, double instant) {
  // The canonical frame with two fields changed. Everything else — the
  // centre, the corrections, the equinox — is left as the SDK computes
  // it, so this asks for "what you would give me, but sidereal".
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
    instants: [instant],
    bodies: [for (final (_, body) in grahas) body],
    frame: frame,
  );
  return [
    for (var i = 0; i < grahas.length; i++)
      Placement(
        grahas[i].$1,
        sky.at(0, i).longitude,
        sky.at(0, i).longitudeSpeed,
      ),
  ];
}

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );

  // ── 1. The record, as it would be written on a form ─────────────────
  final birthDay = Calendar.bikramSambat.date(2042, 9, 17);
  final gregorian = ctx.convert(birthDay, Calendar.gregorian);
  print(
    'born  BS ${birthDay.year}-${_two(birthDay.month)}-${_two(birthDay.day)}'
    '  (${gregorian.year}-${_two(gregorian.month)}-${_two(gregorian.day)})'
    '  00:20  Kathmandu',
  );

  // ── 2. The instant, with the zone's own history ─────────────────────
  final when = ctx.resolve(
    birthDay.at(hour: 0, minute: 20),
    ianaZone('Asia/Kathmandu'),
  );
  final offset = when.offsetSeconds;
  print(
    '      JD ${when.instantJdUtc.toStringAsFixed(6)} UTC'
    '   offset ${offset >= 0 ? '+' : '-'}${_two(offset.abs() ~/ 3600)}'
    ':${_two(offset.abs() % 3600 ~/ 60)}'
    '   ${when.source.key} (tzdb ${when.tzdbVersion})',
  );
  // This record sits on the day Nepal moved from +05:30 to +05:45, which
  // is why the zone's history matters and a fixed offset would be wrong:
  // `iana` above says the answer came from the embedded database rather
  // than from a guess.

  // ── 3. The sky, sidereal ────────────────────────────────────────────
  final placements = chart(teistro, ctx, when.instantJdUtc);

  // ── 4. The chart ────────────────────────────────────────────────────
  print('');
  print('graha            sign               deg  nakshatra      pada');
  print('─' * 62);
  for (final placed in placements) {
    final graha = ctx.entity(placed.graha.fullKey);
    final rashi = ctx.entity(placed.rashi.fullKey);
    final nakshatra = ctx.entity(placed.nakshatra.fullKey);
    print(
      '${graha.name.padRight(12)} ${(graha.glyph ?? '').padRight(2)} '
      '${placed.retrograde ? '℞' : ' '} '
      '${rashi.name.padRight(12)} '
      '${placed.degreeInRashi.toStringAsFixed(4).padLeft(8)}°  '
      '${nakshatra.name.padRight(14)} ${placed.pada}',
    );
  }

  // ── What the SDK had to do to answer ────────────────────────────────
  // Every result carries the steps that produced it. Here the provider
  // answered tropical positions and the SDK applied the ayanamsha and
  // shifted the zodiac; against a provider that answers sidereal
  // natively, those steps would say so instead.
  print('');
  final sky = ctx.positions(
    instants: [when.instantJdUtc],
    bodies: [Body.sun],
    frame: Frame(
      ayanamsha: Ayanamsha.lahiri,
      centre: teistro.canonicalFrame.centre,
      equinox: teistro.canonicalFrame.equinox,
      coordinates: teistro.canonicalFrame.coordinates,
      sidereal: true,
      lightTime: teistro.canonicalFrame.lightTime,
      aberration: teistro.canonicalFrame.aberration,
      deflection: teistro.canonicalFrame.deflection,
      nutation: teistro.canonicalFrame.nutation,
    ),
  );
  final steps = sky.stepsApplied
      .cast<Map<String, Object?>>()
      .map((step) => '${step['name']}:${step['implementation']}')
      .join(', ');
  print('steps applied  $steps');
  print('settings hash  ${ctx.settingsHash.substring(0, 16)}…');
  ctx.dispose();

  // ── A birth with no recorded time ──────────────────────────────────
  // The commonest data problem in the field, and the SDK does **not**
  // pick a time for you. `whenUnknown` says the time is unknown; what
  // happens next is the profile's `time.unknown_time` policy, and by
  // default there is none, so the call is refused with a hint naming
  // the choices.
  print('');
  final noTime = birthDay.whenUnknown;
  for (final policy in [null, 'NOON', 'MIDNIGHT']) {
    final scoped = teistro.context(
      profile: 'nepali-default',
      locale: 'ne-Deva-NP',
      testProvider: true,
      settings:
          policy == null
              ? null
              : {
                'time': {'unknown_time': policy},
              },
    );
    final label = (policy ?? 'refuse').padRight(9);
    try {
      final resolved = scoped.resolve(noTime, ianaZone('Asia/Kathmandu'));
      // The warnings are catalogue members here rather than strings, so
      // the key is what to print; it is the same word in every binding.
      final warnings =
          resolved.warnings.isEmpty
              ? '(no warning)'
              : resolved.warnings.map((w) => w.key).join(', ');
      print(
        '$label JD ${resolved.instantJdUtc.toStringAsFixed(6)}'
        '  time known ${resolved.timeKnown}  $warnings',
      );
    } on TeistroException catch (error) {
      print('$label ${error.message}');
      print('${' ' * 10}hint: ${error.hint}');
    }
    scoped.dispose();
  }
  // MIDNIGHT is refused for a different reason, and it is this record's
  // own: the clocks jumped at midnight on this very date, so 00:00 never
  // happened in Kathmandu. A chart cast on a guessed midnight would have
  // been cast on a time that does not exist.
  // NOON answers, and says so twice — `timeKnown` is false and the
  // resolution carries a `time-unknown-fallback` warning — so a stored
  // chart can never quietly claim a birth time it never had.
}

String _two(int value) => value.toString().padLeft(2, '0');
