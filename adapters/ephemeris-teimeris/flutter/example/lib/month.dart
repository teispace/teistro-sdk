// The answer the app gives, apart from the widget, so the integration test
// asks the same question the screen does.

import 'package:teistro_ephemeris_teimeris_flutter/teistro_ephemeris_teimeris_flutter.dart';
import 'package:teistro_flutter/teistro_flutter.dart';

/// One month's almanac at Kathmandu under each setup over Teimeris, as the
/// lines `teistro-ephemeris-teimeris-day-hash` prints on a desktop: the two
/// are equal when the phone's answer is the desktop's.
List<String> monthHashes(Teistro teistro) {
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  final setups = <String, Context>{
    'drik-teimeris': teistro.context(
      profile: 'parashari-classical',
      ephemeris: [teimeris()],
    ),
    'committee-teimeris': teistro.nepalOfficial(modern: [teimeris()]),
  };
  return [
    for (final MapEntry(key: key, value: context) in setups.entries)
      () {
        final month = context.almanac.of(
          from: Calendar.gregorian.date(2025, 11, 1),
          to: Calendar.gregorian.date(2025, 11, 30),
          place: place,
          utcOffsetSeconds: 20700,
        );
        final provenance = month.provenance;
        context.dispose();
        return '$key settings ${provenance.settingsHash} '
            'content ${provenance.contentHash}';
      }(),
  ];
}

/// Whether the SDK library the app bundled carries the built-in ephemeris:
/// an app on Teimeris bundles the one built without it.
bool hasBuiltin(Teistro teistro) {
  try {
    teistro
        .context(ephemeris: const [NamedEphemeris(Ephemeris.builtin)])
        .dispose();
    return true;
  } on TeistroException {
    return false;
  }
}
