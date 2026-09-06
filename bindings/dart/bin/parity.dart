// One scenario through the Dart binding, printed as the parity report:
// `key<TAB>value` lines, sorted by key. `cargo xtask check-parity` runs
// this and `bindings/node/parity.mjs` and compares what they print, so a
// difference between the two bindings' layers is a failed gate rather
// than something a reader has to notice.
//
// Every value is what this binding's own surface gives: an enum as the
// key it spells, a number formatted to nine decimals, a JSON section as
// its length and its FNV-1a hash, because the point is that the two
// bindings agree, not that they agree with a literal written here.

import 'dart:convert';
import 'dart:io';

import 'package:teistro/messages.dart' as intl;
import 'package:teistro/teistro.dart';

final Map<String, String> report = {};

/// A number as every binding spells it: nine decimals, never an exponent,
/// and an integer value written plainly.
String number(num value) {
  if (value is int) return value.toString();
  final double d = value.toDouble();
  return d == d.truncateToDouble() && d.abs() < 1e15
      ? d.toInt().toString()
      : d.toStringAsFixed(9);
}

/// FNV-1a over UTF-8 bytes, so a JSON section can be compared without a
/// parser.
String fnv(String text) {
  var hash = 0x811c9dc5;
  for (final byte in utf8.encode(text)) {
    hash = ((hash ^ byte) * 0x01000193) & 0xffffffff;
  }
  return hash.toRadixString(16).padLeft(8, '0');
}

void put(String key, Object? value) {
  report[key] = switch (value) {
    num n => number(n),
    bool b => b.toString(),
    _ => '$value',
  };
}

void main() {
  final teistro = Teistro.open();

  // ── The library itself ───────────────────────────────────────────────
  put('abi', teistro.abi);
  put('sdk', teistro.version);
  put('catalogue-version', teistro.catalogue);
  put('default-profile', teistro.defaultProfileId);
  put('build-sdk', teistro.build.sdk);
  put('build-abi', teistro.build.abi);
  put('build-catalogue', teistro.build.catalogue);
  put('build-commit', teistro.build.commit);
  put('build-dirty', teistro.build.dirty);
  put('build-target', teistro.build.target);

  // ── A context ────────────────────────────────────────────────────────
  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );
  put('profile', ctx.profile);
  put('locale', ctx.locale);
  put('settings-hash', ctx.settingsHash);
  put('settings-fnv', fnv(ctx.settingsJson));

  // ── The calendars ────────────────────────────────────────────────────
  final date = Calendar.gregorian.date(2015, 4, 14);
  final bs = ctx.convert(date, Calendar.bikramSambat);
  put('bs-year', bs.year);
  put('bs-month', bs.month);
  put('bs-day', bs.day);
  put('bs-era', bs.era?.fullKey);
  put('bs-era-year', bs.eraYear);
  put('bs-resolution', bs.resolution.key);
  final fixed = ctx.fixedOf(date);
  put('fixed', fixed);
  put('weekday', ctx.weekdayOf(date));
  put('month-length', ctx.monthLength(Calendar.gregorian, 2024, 2));
  put('is-leap', ctx.isLeap(Calendar.gregorian, 2024));
  put('jd-of-fixed', teistro.julianDayOfFixed(fixed));
  final back = teistro.fixedOfJulianDay(2457126.75);
  put('fixed-of-jd', back.value);
  put('fraction-of-jd', back.fraction);

  // ── Time ─────────────────────────────────────────────────────────────
  final civil = Calendar.gregorian.date(1986, 1, 1).at(hour: 0, minute: 20);
  final zone = ianaZone('Asia/Kathmandu');
  final resolved = ctx.resolve(civil, zone);
  put('resolve-jd', resolved.instantJdUtc);
  put('resolve-offset', resolved.offsetSeconds);
  put('resolve-era', resolved.era.key);
  put('resolve-source', resolved.source.key);
  put('resolve-time-known', resolved.timeKnown);
  put('resolve-tzdb', resolved.tzdbVersion);
  put('resolve-warnings', resolved.warnings.length);
  final civilBack = ctx.civilOf(
    resolved.instantJdUtc,
    zone,
    Calendar.gregorian,
  );
  put('civil-year', civilBack.civil.date.year);
  put('civil-minute', civilBack.civil.time.minute);
  put('civil-offset', civilBack.resolution.offsetSeconds);
  final tt = ctx.convertTime(2451544.5, Scale.utc, Scale.tt);
  put('tt-jd', tt.jd);
  put('tt-delta-t', tt.deltaTSeconds);
  put('tt-delta-t-source', tt.deltaTSource.key);
  put('tt-delta-t-model', tt.deltaTModel);
  final delta = ctx.deltaT(2451544.5);
  put('delta-t-seconds', delta.seconds);
  put('delta-t-source', delta.source.key);

  // ── Keys ─────────────────────────────────────────────────────────────
  final id = ctx.keyId('graha.SUN');
  put('key-id', id);
  put('key-name', ctx.keyName(id));
  try {
    ctx.keyId('graha.SUNN');
    put('refusal', 'none');
  } on TeistroException catch (error) {
    put('refusal-status', error.status.key);
    put('refusal-detail', error.detail);
    put('refusal-hint-names-sun', error.hint?.contains('SUN'));
  }

  // ── The locale engine ────────────────────────────────────────────────
  final rendered = ctx.render('sdk.reason.grahaInBhava', {
    'graha': {r'$entity': 'graha.JUPITER'},
    'bhava': 7,
  });
  put('render-fnv', fnv(rendered.text));
  put('render-length', rendered.text.runes.length);
  put('render-resolved-from', rendered.resolvedFrom);
  put('render-fallback', rendered.fallback);
  put('has-message', ctx.has('sdk.reason.grahaInBhava'));
  put('has-missing-message', ctx.has('sdk.nope.missing'));
  put('transliterated', ctx.transliterate('सूर्य बृहस्पति'));
  put('entity-sun-name', ctx.entity('graha.SUN').name);
  put('entity-sun-iast', ctx.entity('graha.SUN').iast);
  put('entity-sun-glyph', ctx.entity('graha.SUN').glyph);
  put('entity-sun-gender', ctx.entity('graha.SUN').gender?.key);
  put(
    'message-graha-in-bhava',
    ctx.messages.sdk.reason.grahaInBhava(
      graha: intl.GrahaKey.jupiter,
      bhava: 7,
    ),
  );
  put(
    'message-bs-date',
    ctx.messages.sdk.calendar.bikramSambat.date.long(
      day: 1,
      monthName: 'बैशाख',
      year: 2072,
    ),
  );

  // ── Positions ────────────────────────────────────────────────────────
  final frame = teistro.canonicalFrame;
  put('frame-centre', frame.centre.key);
  put('frame-coordinates', frame.coordinates.key);
  put('frame-bits', teistro.packFrame(frame));
  put(
    'frame-round-trip',
    teistro.unpackFrame(teistro.packFrame(frame)).centre == frame.centre,
  );
  final positions = ctx.positions(
    instants: [2451545.0, 2451546.0],
    bodies: [Body.sun, Body.moon, Body.mars],
  );
  put('cells', positions.cells.length);
  put('positions-scale', positions.timeScale.key);
  put('positions-bodies', positions.bodyKeys.map((b) => b.key).join(','));
  for (var i = 0; i < positions.cells.length; i++) {
    final instant = i ~/ positions.bodyCount;
    final body = i % positions.bodyCount;
    final cell = positions.at(instant, body);
    put('cell-$i-lon', cell.longitude);
    put('cell-$i-lat', cell.latitude);
    put('cell-$i-dist', cell.distance);
    put('cell-$i-lon-speed', cell.longitudeSpeed);
    put('cell-$i-status', cell.status);
  }
  put(
    'steps',
    positions.stepsApplied
        .cast<Map<String, Object?>>()
        .map((step) => '${step['name']}:${step['implementation']}')
        .join(','),
  );
  put('provenance-fnv', fnv(positions.provenance));
  put('provenance-profile', positions.provenanceOf['profile']);
  put('provenance-settings-hash', positions.provenanceOf['settings_hash']);
  put(
    'provenance-provider-frame',
    (positions.provenanceOf['provider']! as Map<String, Object?>)['frame'],
  );

  final keys = report.keys.toList()..sort();
  for (final key in keys) {
    stdout.write('$key\t${report[key]}\n');
  }
  ctx.dispose();
}
