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
// 3. The chart **founded** at that instant and place. The SDK's
//    canonical frame is *tropical*, because that is what an ephemeris
//    computes; a Vedic chart wants the sidereal zodiac, and the profile
//    says which ayanamsha and which centre, so the founder asks for that
//    frame and the SDK completes it — and stamps every step it applied,
//    which this example prints. Positions asked for directly would not
//    know the profile wants the Moon seen from Kathmandu rather than
//    from the Earth's centre, which moves it most of a degree.
// 4. Each longitude read as a rashi, a nakshatra and a pada, using the
//    catalogue's own members and the locale's own names.
//
// What it does not need: an ephemeris of your own, a data file, a
// network, or a second library. `ephemeris: [NamedEphemeris(Ephemeris.builtin)]` selects
// the one the SDK carries, so every position below is a real sky and
// this file runs anywhere the package installs.

import 'package:teistro/teistro.dart';

/// A nakshatra is a twenty-seventh of the circle; a pada a quarter of one.
const double nakshatraDeg = 360.0 / 27.0;
const double padaDeg = nakshatraDeg / 4.0;

/// The sign a longitude stands in, and how far into it.
(Rashi, double) rashiOf(double longitude) => (
  Rashi.byId(longitude ~/ 30),
  longitude % 30.0,
);

/// The lunar mansion a longitude stands in, and which quarter of it, 1
/// to 4.
(Nakshatra, int) nakshatraOf(double longitude) => (
  Nakshatra.byId(longitude ~/ nakshatraDeg),
  (longitude % nakshatraDeg) ~/ padaDeg + 1,
);

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );

  // ── 1. The record, as it would be written on a form ─────────────────
  final birthDay = Calendar.bikramSambat.date(2042, 9, 17);
  final gregorian = ctx.calendar.convert(birthDay, Calendar.gregorian);
  print(
    'born  BS ${birthDay.year}-${_two(birthDay.month)}-${_two(birthDay.day)}'
    '  (${gregorian.year}-${_two(gregorian.month)}-${_two(gregorian.day)})'
    '  00:20  Kathmandu',
  );

  // ── 2. The instant, with the zone's own history ─────────────────────
  final when = ctx.time.resolve(
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

  // ── 3. The chart ───────────────────────────────────────────────────
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  final chart = ctx.chart.found(
    instant: when.instantJdUtc,
    place: place,
    utcOffsetSeconds: when.offsetSeconds,
  );

  print('');
  print('graha             sign               deg  nakshatra      pada bhava');
  print('─' * 67);
  for (final placed in chart.grahas) {
    final graha = ctx.intl.entity(placed.graha.fullKey);
    final (rashi, degrees) = rashiOf(placed.longitudeDeg);
    final (nakshatra, pada) = nakshatraOf(placed.longitudeDeg);
    print(
      '${graha.name.padRight(12)} ${(graha.glyph ?? '').padRight(2)} '
      // There is no retrograde flag to trust blindly: a graha is
      // retrograde when its longitude is decreasing, which is what the
      // speed says, and `retrograde` is that comparison, named.
      '${placed.retrograde ? '℞' : ' '} '
      '${ctx.intl.entity(rashi.fullKey).name.padRight(12)} '
      '${degrees.toStringAsFixed(4).padLeft(8)}°  '
      '${ctx.intl.entity(nakshatra.fullKey).name.padRight(14)} $pada   '
      // Which bhava it is in, under the chart's placement system -- the
      // question most of the tradition answers with "in the seventh".
      '${placed.house.bhava.toString().padLeft(2)}',
    );
  }

  // ── 4. What the chart is measured in, and against ───────────────────
  print('');
  final (lagna, into) = rashiOf(chart.lagnaDeg);
  print(
    'lagna          ${chart.lagnaDeg.toStringAsFixed(4)}° -- '
    '${ctx.intl.entity(lagna.fullKey).name} at ${into.toStringAsFixed(4)}°, '
    'vara ${chart.day.vara.fullKey}',
  );
  // `null`, and it means what it says: a tropical chart has no
  // ayanamsha, not an ayanamsha of nought.
  final applied =
      chart.ayanamsha?.fullKey ?? (chart.ayanamshaCustom ? 'custom' : null);
  print(
    applied == null
        ? 'ayanamsha      tropical, none applied'
        : 'ayanamsha      ${chart.ayanamshaOffsetDeg.toStringAsFixed(6)}° '
            'applied ($applied)',
  );
  print('steps applied  ${chart.batch.stepsApplied.join(', ')}');
  // The provenance envelope stamps the settings, the provider and the
  // time layer; it is what a stored chart keeps in order to say what
  // computed it.
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
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      settings:
          policy == null
              ? null
              : {
                'time': {'unknown_time': policy},
              },
    );
    final label = (policy ?? 'refuse').padRight(9);
    try {
      final resolved = scoped.time.resolve(noTime, ianaZone('Asia/Kathmandu'));
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
  // resolution carries a `TIME_UNKNOWN_FALLBACK` warning — so a stored
  // chart can never quietly claim a birth time it never had.
}

String _two(int value) => value.toString().padLeft(2, '0');
