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

  // ── A chart and an almanac, under a geocentric profile ───────────────
  // The scenario above runs under `nepali-default`, whose frame is
  // **topocentric**, and a chart cannot be founded under it at all: the
  // completion's centre step is Phase 3's, so the provider is asked for
  // a frame it does not answer. That refusal is itself worth comparing —
  // the three bindings must fail the same way — and everything after it
  // needs a second context on the SDK's own default profile, which is
  // geocentric.
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  try {
    ctx.found(instant: 2451545, place: place, utcOffsetSeconds: 20700);
    put('chart-under-topocentric', 'founded');
  } on TeistroException catch (error) {
    put('chart-under-topocentric', error.status.key);
  }

  final geo = teistro.context(
    profile: 'parashari-classical',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );
  put('geo-profile', geo.profile);
  put('geo-settings-hash', geo.settingsHash);

  // ── Charts ───────────────────────────────────────────────────────────
  // Two instants, so a per-chart section that ran charts-outermost the
  // wrong way round shows up as the second chart's values in the first's
  // place rather than as nothing at all.
  final charts = geo.foundMany(
    instants: <double>[2460482.5, 2460600.25],
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('chart-count', charts.chartCount);
  put('chart-kind', ChartKind.byId(charts.kind).fullKey);
  put('chart-place-lat', charts.latitudeDeg);
  put('chart-place-lon', charts.longitudeDeg);
  put('chart-model-fnv', fnv(charts.model));
  put('chart-steps', charts.stepsApplied.join(','));
  put('chart-provenance-fnv', fnv(charts.provenance));
  put(
    'chart-provenance-profile',
    (jsonDecode(charts.provenance) as Map<String, Object?>)['profile'],
  );
  put('chart-graha-count', charts.grahaCount);

  for (final chart in charts.each) {
    final i = chart.index;
    put('chart-$i-instant', chart.instant);
    put('chart-$i-lagna', chart.lagnaDeg);
    put('chart-$i-day-lagna', chart.dayLagnaDeg);
    put('chart-$i-ayanamsha', chart.ayanamshaOffsetDeg);
    put('chart-$i-day-part', chart.dayPart.key);
    put('chart-$i-day-elapsed', chart.dayElapsed);
    put('chart-$i-vara', chart.vara.fullKey);
    put('chart-$i-sunrise', charts.day.sunrise[i]);
    put('chart-$i-sunset', charts.day.sunset[i]);
    put(
      'chart-$i-date',
      '${charts.day.year[i]}-${charts.day.month[i]}-${charts.day.dayOfMonth[i]}',
    );
    put('chart-$i-ghati', charts.timing.ghati[i]);
    put('chart-$i-pala', charts.timing.pala[i]);
    put('chart-$i-vipala', charts.timing.vipala[i]);
    put('chart-$i-hora-number', charts.timing.horaNumber[i]);
    put('chart-$i-hora-lord', chart.horaLord.fullKey);
    final grahas = chart.grahas;
    for (var j = 0; j < grahas.length; j += 1) {
      final graha = grahas[j];
      put('chart-$i-graha-$j', graha.graha.fullKey);
      put('chart-$i-graha-$j-lon', graha.longitudeDeg);
      put('chart-$i-graha-$j-lat', graha.latitudeDeg);
      put('chart-$i-graha-$j-speed', graha.speedDegPerDay);
      put('chart-$i-graha-$j-retro', graha.retrograde);
      put('chart-$i-graha-$j-house', graha.house.bhava);
      put('chart-$i-graha-$j-house-method', graha.house.method.fullKey);
      put('chart-$i-graha-$j-placement', graha.placement.bhava);
    }
    final houses = chart.houses;
    for (var k = 0; k < houses.length; k += 1) {
      put('chart-$i-house-$k-madhya', houses[k].madhyaDeg);
      put('chart-$i-house-$k-sandhi', houses[k].sandhiDeg);
    }
    final chalit = chart.chalit;
    for (var k = 0; k < chalit.length; k += 1) {
      put('chart-$i-chalit-$k-madhya', chalit[k].madhyaDeg);
    }
  }
  // `found` is the batch of one unwrapped, and must agree with the batch.
  final single = geo.found(
    instant: 2460482.5,
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('chart-single-lagna', single.lagnaDeg);
  put('chart-single-agrees', single.lagnaDeg == charts.at(0).lagnaDeg);

  // ── An almanac ───────────────────────────────────────────────────────
  // Three days, because a day's lists are ragged and two consecutive days
  // with the same counts would not exercise the offsets.
  final week = geo.almanac(
    from: Calendar.gregorian.date(2024, 6, 17),
    to: Calendar.gregorian.date(2024, 6, 19),
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('almanac-days', week.length);
  put('almanac-calendar', week.calendar.fullKey);
  put('almanac-place-lat', week.decoded.latitudeDeg);
  put('almanac-model-fnv', fnv(week.model));
  put('almanac-provenance-fnv', fnv(week.decoded.provenance));

  for (final day in week.each) {
    final i = day.index;
    final columns = week.decoded;
    put('day-$i-vara', day.vara.fullKey);
    put('day-$i-sunrise', day.sunrise);
    put('day-$i-sunset', day.sunset);
    put('day-$i-next-sunrise', columns.day.nextSunrise[i]);
    put(
      'day-$i-date',
      '${columns.day.year[i]}-${columns.day.month[i]}-${columns.day.dayOfMonth[i]}',
    );
    put('day-$i-window-from', day.window.from);
    put('day-$i-window-to', day.window.to);
    put('day-$i-month', day.month.month.fullKey);
    put('day-$i-amanta', day.month.amanta.fullKey);
    put('day-$i-purnimanta', day.month.purnimanta.fullKey);
    put('day-$i-paksha', day.month.paksha.fullKey);
    put('day-$i-convention', day.month.convention.key);
    put('day-$i-month-kind', day.month.kind.key);
    put('day-$i-ayana', day.ayana.fullKey);
    put('day-$i-disha-shool', day.dishaShool.fullKey);
    // An absent value must be absent in all three, not nought in one.
    final sankranti = day.sankranti;
    put('day-$i-sankranti', sankranti == null ? 'none' : number(sankranti));
    final abhijit = day.abhijit;
    put('day-$i-abhijit', abhijit == null ? 'none' : number(abhijit.at.from));
    put(
      'day-$i-abhijit-effective',
      abhijit == null ? 'none' : abhijit.effective.toString(),
    );
    final brahma = day.brahma;
    put('day-$i-brahma', brahma == null ? 'none' : number(brahma.from));
    // The counts are what the ragged layout turns on: if a binding's
    // prefix sum were off by a day, these would still agree and the spans
    // below would not.
    put('day-$i-tithi-count', day.tithi.length);
    put('day-$i-nakshatra-count', day.nakshatra.length);
    put('day-$i-yoga-count', day.yoga.length);
    put('day-$i-karana-count', day.karana.length);
    put('day-$i-kaala-count', day.kaalas.length);
    put('day-$i-choghadiya-count', day.choghadiya.length);
    put('day-$i-hora-count', day.horas.length);
    put('day-$i-muhurta-count', day.muhurtas.length);
    put('day-$i-moon-event-count', day.moonEvents.length);
    put('day-$i-panchaka-count', day.panchaka.length);
    put('day-$i-moon-sign-count', day.moonSigns.length);
    put('day-$i-sun-sign-count', day.sunSigns.length);
    put('day-$i-muhurta-yoga-count', day.muhurtaYogas.length);
    void limb(String name, List<Span<dynamic>> spans) {
      for (var j = 0; j < spans.length; j += 1) {
        final span = spans[j];
        put('day-$i-$name-$j', (span.member as dynamic).fullKey);
        put('day-$i-$name-$j-whole-from', span.whole.from);
        put('day-$i-$name-$j-inside-to', span.inside.to);
      }
    }

    limb('tithi', day.tithi);
    limb('nakshatra', day.nakshatra);
    limb('yoga', day.yoga);
    limb('karana', day.karana);
    final kaalas = day.kaalas;
    for (var j = 0; j < kaalas.length; j += 1) {
      put('day-$i-kaala-$j', kaalas[j].kaala.fullKey);
      put('day-$i-kaala-$j-from', kaalas[j].at.from);
    }
    final horas = day.horas;
    put('day-$i-hora-0-lord', horas.first.lord.fullKey);
    put('day-$i-hora-0-start', horas.first.start);
    put('day-$i-hora-23-lord', horas.last.lord.fullKey);
    final choghadiya = day.choghadiya;
    put('day-$i-choghadiya-0', choghadiya.first.choghadiya.fullKey);
    put('day-$i-choghadiya-0-daytime', choghadiya.first.daytime);
    final muhurtas = day.muhurtas;
    put('day-$i-muhurta-0-from', muhurtas.first.at.from);
    put('day-$i-muhurta-last-daylight', muhurtas.last.daylight);
    final moon = day.moonEvents;
    for (var j = 0; j < moon.length; j += 1) {
      put('day-$i-moon-$j-kind', moon[j].rise ? 'rise' : 'set');
      put('day-$i-moon-$j-instant', moon[j].instant);
    }
    final held = day.muhurtaYogas;
    for (var j = 0; j < held.length; j += 1) {
      put('day-$i-yoga-held-$j', held[j].yoga.fullKey);
      put(
        'day-$i-yoga-held-$j-cause',
        held[j].tithi == null ? 'vara-nakshatra' : 'vara-tithi-nakshatra',
      );
      put('day-$i-yoga-held-$j-tithi', held[j].tithi?.fullKey ?? 'none');
    }
  }
  // `almanacDay` is the range of one unwrapped, and must agree.
  final oneDay = geo.almanacDay(
    date: Calendar.gregorian.date(2024, 6, 17),
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('almanac-single-agrees', oneDay.sunrise == week.at(0).sunrise);
  geo.dispose();

  final keys = report.keys.toList()..sort();
  for (final key in keys) {
    stdout.write('$key\t${report[key]}\n');
  }
  ctx.dispose();
}
