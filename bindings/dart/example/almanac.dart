// A week's panchangam: the five limbs of each day, and its periods.
//
// A chart is consulted once; a panchanga every morning. This is the page
// a Nepali or Indian almanac prints, and the SDK computes it in **one
// crossing** for the whole week — consecutive days share a boundary, so
// day n's next sunrise is day n+1's sunrise, and asking for seven days
// costs much less than seven days asked for separately.
//
// What the shape teaches, and what a reader should copy:
//
//   * A limb is a **span**, not a name. "Today's tithi" is a question
//     with two answers on most days, and the SDK gives both with the
//     instant each gives way — which is what an almanac row prints.
//   * A span carries its **own** bounds as well as the clipped ones, so
//     "the tithi began yesterday at 21:05" is a fact you can print.
//   * A value a day may not have is **absent**, never a sentinel: no
//     sankranti is `null`, not Julian day zero.
//
// `testProvider: true` selects the analytic ephemeris the SDK carries,
// so this file runs anywhere.

import 'package:teistro/teistro.dart';

const offsetSeconds = 20700;

/// A Julian day as the local clock reads it, which is what an almanac
/// prints; the offset is the one the request was made under.
String clock(double jd) {
  final local = (jd + offsetSeconds / 86400 + 0.5) % 1;
  final minutes = (local * 1440).round() % 1440;
  return '${(minutes ~/ 60).toString().padLeft(2, '0')}:'
      '${(minutes % 60).toString().padLeft(2, '0')}';
}

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'parashari-classical',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );

  // A locale pack names most of the catalogue and not all of it: `masa`
  // and `direction` have no entries in any of the five the SDK ships, so
  // an almanac falls back to the key rather than refusing to print.
  String name(String key) {
    try {
      return ctx.intl.entity(key).name;
    } on TeistroException {
      return key
          .substring(key.indexOf('.') + 1)
          .toLowerCase()
          .replaceAll('_', ' ');
    }
  }

  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );

  // ── One crossing for the whole week ─────────────────────────────────
  final week = ctx.almanac.of(
    from: Calendar.gregorian.date(2024, 6, 17),
    to: Calendar.gregorian.date(2024, 6, 23),
    place: place,
    utcOffsetSeconds: offsetSeconds,
  );

  print(
    '${week.length} days at ${place.latitudeDeg}°N ${place.longitudeDeg}°E, '
    'one crossing',
  );
  print('calendar ${week.calendar.key}   model ${week.model.split(',').first}');
  print('');

  for (final day in week.each) {
    final d = week.decoded.day;
    final i = day.index;
    print(
      '${name(day.vara.fullKey).padRight(12)} '
      '${d.year[i]}-${d.month[i].toString().padLeft(2, '0')}-'
      '${d.dayOfMonth[i].toString().padLeft(2, '0')}'
      '   sunrise ${clock(day.sunrise)}  sunset ${clock(day.sunset)}'
      '   ${name(day.month.amanta.fullKey)} ${name(day.month.paksha.fullKey)}',
    );
    // The five limbs. The vara is one of them and is the day's own; the
    // other four are spans, and a day usually has two of each.
    void limb(String label, List<Span<dynamic>> spans) {
      final printed = spans
          .map((span) {
            // `whole` is the member's own span and `inside` the clipped one,
            // so a member that began yesterday says so rather than looking
            // as though it began at sunrise.
            final began = span.whole.from < span.inside.from ? '‹' : ' ';
            final ends = span.whole.to > span.inside.to ? '›' : ' ';
            final key = (span.member as dynamic).fullKey as String;
            return '$began${name(key)} until ${clock(span.inside.to)}$ends';
          })
          .join('  ');
      print('  ${label.padRight(10)} $printed');
    }

    limb('tithi', day.tithi);
    limb('nakshatra', day.nakshatra);
    limb('yoga', day.yoga);
    limb('karana', day.karana);

    // The periods a day is planned around. Rahu kalam is the one
    // everybody checks; the choghadiya are what a shop opens on.
    final kaalas = day.kaalas
        .map(
          (k) =>
              '${name(k.kaala.fullKey)} ${clock(k.at.from)}–${clock(k.at.to)}',
        )
        .join('  ');
    print('  ${'kaala'.padRight(10)} $kaalas');
    final auspicious = day.choghadiya.where((c) => c.daytime).take(3);
    print(
      '  ${'choghadiya'.padRight(10)} '
      '${auspicious.map((c) => '${name(c.choghadiya.fullKey)} ${clock(c.at.from)}').join('  ')} …',
    );
    final abhijit = day.abhijit;
    if (abhijit != null) {
      print(
        '  ${'abhijit'.padRight(10)} ${clock(abhijit.at.from)}–${clock(abhijit.at.to)}'
        '${abhijit.effective ? '' : '  (not effective on a Wednesday)'}',
      );
    }
    // Absent is absent: no sankranti is null, and a Moon that did not
    // rise inside the window contributes no event at all.
    final sankranti = day.sankranti;
    if (sankranti != null) {
      print(
        '  ${'sankranti'.padRight(10)} the Sun enters a new sign at ${clock(sankranti)}',
      );
    }
    final moon = day.moonEvents
        .map((e) => '${e.rise ? 'rise' : 'set'} ${clock(e.instant)}')
        .join('  ');
    print(
      '  ${'moon'.padRight(10)} '
      '${moon.isEmpty ? '(neither rise nor set inside the window)' : moon}',
    );
    print('');
  }

  // ── What the ragged layout costs a reader, which is nothing ─────────
  // Each day's lists are slices of one concatenated column, found by
  // adding up every earlier day's count. The layer does that sum once
  // when the batch is built, so `day.karana` is a slice and not a search.
  final counted = week.each.fold<int>(
    0,
    (total, day) => total + day.karana.length,
  );
  print('$counted karanas across ${week.length} days, from one blob');
  print('settings hash  ${ctx.settingsHash.substring(0, 16)}…');

  // A day on its own is the range of one unwrapped: same crossing, and
  // the answer is a day rather than a list of one.
  final one = ctx.almanac.day(
    date: Calendar.gregorian.date(2024, 6, 21),
    place: place,
    utcOffsetSeconds: offsetSeconds,
  );
  print(
    'almanacDay     ${name(one.vara.fullKey)}  ${one.horas.length} horas, '
    '${one.muhurtas.length} muhurtas, ${one.choghadiya.length} choghadiya',
  );
  ctx.dispose();
}
