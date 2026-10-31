// Nepal's official days, and the modern reckoning beside them.
//
// Nepal's patro is computed by the Nepali committee from the Surya
// Siddhanta: the tithi, nakshatra and yoga by the text with the
// committee's bija, and the sunrise from a modern ephemeris. That is what
// `nepalOfficial` opens. The modern reckoning, `nepali-drik`, reads the
// same day from the modern sky, and the two name a different sunrise
// tithi on a few days a month.
//
// What the shape teaches, and what a reader should copy:
//
//   * The official day is a **preset**, not a recipe:
//     `teistro.nepalOfficial(modern: [engine])`.
//   * The Surya Siddhanta takes the engine **after** it in the chain for
//     its sunrise; an app passes its own, `modern: [teimeris()]`. This
//     file passes none, so the sunrise is the built-in ephemeris's and it
//     runs anywhere.
//   * Where the two disagree, the reason is a tithi's end against the
//     sunrise: the tithi the modern sky still has at sunrise had already
//     ended by the text's, or the other way round. Each row prints both
//     ends.

import 'package:teistro/teistro.dart';

const offsetSeconds = 20700;

/// A Julian day as the local clock reads it.
String clock(double jd) {
  final local = (jd + offsetSeconds / 86400 + 0.5) % 1;
  final minutes = (local * 1440).round() % 1440;
  return '${(minutes ~/ 60).toString().padLeft(2, '0')}:'
      '${(minutes % 60).toString().padLeft(2, '0')}';
}

void main() {
  final teistro = Teistro.open();
  final official = teistro.nepalOfficial(locale: 'en-Latn');
  final modern = teistro.context(
    profile: 'nepali-drik',
    locale: 'en-Latn',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );
  String name(String key) => official.intl.entity(key).name;

  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );

  // The same month under each, one crossing each.
  Almanac month(Context ctx) => ctx.almanac.of(
    from: Calendar.gregorian.date(2025, 11, 1),
    to: Calendar.gregorian.date(2025, 11, 30),
    place: place,
    utcOffsetSeconds: offsetSeconds,
  );
  final days = month(official);
  final others = month(modern);
  final ourDays = days.each.toList();
  final theirDays = others.each.toList();

  print('official ${official.profile}, modern ${modern.profile}');
  var differ = 0;
  for (var i = 0; i < ourDays.length; i++) {
    // The first span of a day's limb is the one standing at its sunrise.
    final ours = ourDays[i].tithi.first;
    final theirs = theirDays[i].tithi.first;
    if (ours.member == theirs.member) continue;
    differ++;
    final d = days.decoded.day;
    print(
      '${d.year[i]}-${d.month[i].toString().padLeft(2, '0')}-'
      '${d.dayOfMonth[i].toString().padLeft(2, '0')}'
      '  sunrise ${clock(ourDays[i].day.sunrise)}'
      '  official ${name(ours.member.fullKey)} until ${clock(ours.whole.to)}'
      '  modern ${name(theirs.member.fullKey)} until ${clock(theirs.whole.to)}',
    );
  }
  print('$differ of ${ourDays.length} days name another sunrise tithi');

  official.dispose();
  modern.dispose();
}
