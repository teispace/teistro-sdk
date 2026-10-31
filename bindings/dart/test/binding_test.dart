// The Dart binding end to end: the same scenario the C binding's smoke
// test and the Node binding's tests walk, through the generated
// declarations and the ergonomic layer.
//
// `cargo xtask check-dart` builds the shared library and runs this file;
// `TEISTRO_LIBRARY` names it.

import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final Teistro teistro = Teistro.open();

/// The fixture directory `cargo xtask check-dart` names, and a reader for
/// the files in it — the convention `blob_test.dart` follows.
final String _fixtures =
    Platform.environment['TEISTRO_FIXTURES'] ?? '../../target/tsrb';
Uint8List _readFixture(String name) =>
    File('$_fixtures/$name').readAsBytesSync();

/// A context with the analytic test provider; every test builds its own.
Context context({
  String? profile = 'nepali-drik',
  Map<String, Object?>? settings,
  String? locale = 'ne-Deva-NP',
  bool testProvider = true,
}) => teistro.context(
  profile: profile,
  settings: settings,
  locale: locale,
  testProvider: testProvider,
);

/// A Gregorian date as the boundary takes one, the long way, which the
/// layer's `Calendar.date` shortens.
CalendarDate gregorian(int year, int month, int day) => CalendarDate(
  calendar: Calendar.gregorian,
  year: year,
  eraYear: 0,
  month: month,
  day: day,
  resolution: Resolution.defined,
  computedMonth: 0,
  computedDay: 0,
);

void main() {
  test('the library and the declarations were generated for the same ABI', () {
    expect(teistro.abi, generatedAbiVersion);
    expect(teistro.catalogue, 1);
    expect(teistro.defaultProfileId, 'parashari-classical');
    expect(
      teistro.version,
      matches(r'^\d+\.\d+\.\d+(-(alpha|beta|rc)(\.\d+)?)?$'),
    );
  });

  test('a short tag finds the locale it means, and says which', () {
    final ctx = context(locale: 'ne');
    expect(ctx.intl.locale, 'ne-Deva-NP');
    expect(ctx.intl.lookupLocale('sa'), 'sa-Deva');
    expect(ctx.intl.locale, 'ne-Deva-NP');
    ctx.intl.locale = 'hi';
    expect(ctx.intl.locale, 'hi-Deva-IN');
    for (final tag in ['de', 'hi-Latn']) {
      expect(
        () => ctx.intl.lookupLocale(tag),
        throwsA(isA<TeistroException>()),
      );
      expect(() => ctx.intl.locale = tag, throwsA(isA<TeistroException>()));
    }
    expect(ctx.intl.locale, 'hi-Deva-IN');
  });

  test('a tithi says its number in the month', () {
    expect(tithiMonthNumber(Tithi.shuklaPratipada), 1);
    expect(tithiMonthNumber(Tithi.purnima), 15);
    expect(tithiMonthNumber(Tithi.krishnaPratipada), 16);
    expect(tithiMonthNumber(Tithi.amavasya), 30);
  });

  test('a day says the clock it was read on, and a digest keeps it', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(40.7128),
      longitudeDeg: Longitude(-74.006),
      altitudeM: Altitude(10),
    );
    final from = gregorian(2026, 10, 31);
    final zone = ianaZone('America/New_York');
    final zoned = ctx.almanac.inZone(
      from: from,
      to: gregorian(2026, 11, 2),
      place: place,
      zone: zone,
    );
    expect(zoned.each.map((day) => day.utcOffsetSeconds).toList(), [
      -14400,
      -18000,
      -18000,
    ]);
    final digest = decodeDigest(
      ctx.almanac.digest(from: from, days: 3, place: place, zone: zone),
    );
    expect(digest.clock.utcOffsetSeconds.toList(), [-14400, -18000, -18000]);
    expect(
      () => ctx.almanac.digest(from: from, days: 3, place: place),
      throwsA(isA<TeistroException>()),
    );
  });

  test('a context resolves its settings and reports them', () {
    final ctx = context();
    expect(ctx.profile, 'nepali-drik');
    expect(ctx.intl.locale, 'ne-Deva-NP');
    expect(ctx.settingsHash, matches(r'^[0-9a-f]{64}$'));
    expect(
      (ctx.settings['frame']! as Map<String, Object?>)['zodiac'],
      'SIDEREAL',
    );
    expect(ctx.settings['schema'], 1);

    // A patch over the profile changes the settings and therefore the hash.
    final patched = context(
      settings: {
        'frame': {'zodiac': 'TROPICAL'},
      },
    );
    expect(
      (patched.settings['frame']! as Map<String, Object?>)['zodiac'],
      'TROPICAL',
    );
    expect(patched.settingsHash, isNot(ctx.settingsHash));

    // An older name opens the profile it names, and the answer says the id.
    final named = context(profile: 'nepali-default');
    expect(named.profile, 'nepali-drik');
    expect(named.settingsHash, ctx.settingsHash);

    // The default profile is the one the library names.
    expect(
      context(profile: null, locale: null).profile,
      teistro.defaultProfileId,
    );
    ctx.dispose();
    expect(
      () => ctx.profile,
      throwsStateError,
      reason: 'a freed context is closed',
    );
  });

  test('a pack loads at runtime and lays its record over the one standing', () {
    // The fixture pack is the base locale's, so this context reads that
    // one: a record is a locale's, and loading into `en-Latn` does not
    // touch what `ne-Deva-NP` says of the same key.
    final ctx = context(locale: 'en-Latn');
    final before = ctx.intl.entity('graha.SUN');
    expect(before.name, isNotEmpty, reason: 'the engine names the Sun');
    expect(before.forms['phala'], isNull);

    final loaded = ctx.intl.loadPack(_readFixture('overlay.tpack'));
    expect(loaded.entries, 1);
    expect(loaded.replaced, 0, reason: 'nothing was thrown away');
    expect(loaded.merged, 1, reason: 'the record kept what the pack lacked');
    expect(loaded.locale, 'en-Latn');
    expect(loaded.sha256, matches(RegExp(r'^[0-9a-f]{64}$')));

    // A record's forms are an open set, so a form no locale of `i18n/`
    // carries is reached through `forms` and not by a named field.
    final after = ctx.intl.entity('graha.SUN');
    expect(after.forms['phala'], 'a reading of the Sun');
    expect(after.name, before.name);
    expect(after.forms['name'], before.name);
  });

  test('the Surya Siddhanta opens by name, and its chart says which parts '
      "are the text's", () {
    // A classical astronomy's chart is the text's throughout
    // (docs/03-design/classical-chart.md), and the envelope says so, where a
    // modern chart's says nothing.
    Chart at(Context ctx) => ctx.chart.found(
      instant: 2447995.4895833335,
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
    );
    final text = teistro.context(
      profile: 'surya-siddhanta',
      ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
    );
    final deviation = at(text).provenance.deviation!;
    expect(deviation.model, 'SURYA_SIDDHANTA');
    expect(
      deviation.detail,
      "the zodiac, the places, the angles and the day are the provider's own",
    );
    text.dispose();
    final modern = context();
    expect(at(modern).provenance.deviation, isNull);
    modern.dispose();
  });

  test('a chart handed out alone carries its own hash, and the batch the '
      "list's", () {
    // STATUS 2h: a batch's provenance hashes the list, and a chart of it the
    // value it holds, which is what a stored chart is checked against.
    final ctx = context();
    final batch = ctx.chart.foundMany(
      instants: [2451545.0, 2451546.0],
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
    );
    final first = batch.at(0).provenance;
    final second = batch.at(1).provenance;
    expect({
      first.contentHash,
      second.contentHash,
      batch.provenance.contentHash,
    }, hasLength(3));
    expect(first.settingsHash, batch.provenance.settingsHash);
    expect(first.sdkVersion.major, batch.provenance.sdkVersion.major);
    expect(() => Confidence.fromJson('MAYBE'), throwsA(isA<FormatException>()));
    // The key a stored answer is reused under crosses on the provenance.
    expect(first.answerKey, matches(RegExp(r'^[0-9a-f]{64}$')));
    final again = ctx.chart.foundMany(
      instants: [2451545.0, 2451546.0],
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
    );
    expect(again.at(0).provenance.answerKey, first.answerKey);
    expect(again.provenance.answerKey, batch.provenance.answerKey);
    ctx.dispose();
  });

  test('a refusal carries its status, its field and its hint', () {
    final ctx = context();
    expect(
      () => ctx.keys.id('graha.SUNN'),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.unsupported)
            .having((e) => e.status.id, 'code', -6)
            .having((e) => e.detail, 'detail', 'UNKNOWN_KEY')
            .having((e) => e.hint, 'hint', contains('did you mean `SUN`'))
            .having((e) => e.toString(), 'toString', contains('UNSUPPORTED')),
      ),
    );
    // No context exists to keep these refusals, so the record crosses
    // whole from the call that failed (ffi-abi-and-api-description.md
    // §6.1).
    expect(
      () => context(profile: 'vedic-classic'),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.unsupported)
            .having(
              (e) => e.message,
              'message',
              contains('no shipped profile `vedic-classic`'),
            )
            .having((e) => e.field, 'field', 'profile')
            .having((e) => e.hint, 'hint', contains('parashari-classical')),
      ),
    );
    expect(
      () => context(locale: 'xx-Latn'),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.field, 'field', 'locale')
            .having((e) => e.hint, 'hint', contains('ne-Deva-NP')),
      ),
    );
    expect(
      () => ctx.calendar.fixedOf(gregorian(2023, 2, 29)),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.detail, 'detail', 'NONEXISTENT_DATE')
            .having((e) => e.status, 'status', Status.invalidArg),
      ),
    );
  });

  test('a date converts into Bikram Sambat with its era and its '
      'resolution', () {
    final ctx = context();
    final date = gregorian(2015, 4, 14);
    final bs = ctx.calendar.convert(date, Calendar.bikramSambat);
    // India's independence in the Indian national calendar.
    final saka = ctx.calendar.convert(
      gregorian(1947, 8, 15),
      Calendar.indianNational,
    );
    expect(
      [saka.era, saka.year, saka.month, saka.day],
      [Era.shaka, 1869, 5, 24],
    );
    // Ekushey in the Bangladeshi calendar: 8 Falgun 1431.
    final ekushey = ctx.calendar.convert(
      gregorian(2025, 2, 21),
      Calendar.bangladeshi,
    );
    expect(
      [ekushey.era, ekushey.year, ekushey.month, ekushey.day],
      [Era.bengali, 1431, 11, 8],
    );
    // 1 Chingam, Kollam 1201, in the Malayalam solar calendar.
    final chingam = ctx.calendar.convert(
      gregorian(2025, 8, 17),
      Calendar.malayalamSolar,
    );
    expect(
      [chingam.era, chingam.year, chingam.month, chingam.day],
      [Era.kollam, 1201, 1, 1],
    );
    // And 1 Ramadan 1446 in the tabular Hijri calendar.
    final ramadan = ctx.calendar.convert(
      gregorian(2025, 3, 1),
      Calendar.hijriTabular,
    );
    expect(
      [ramadan.era, ramadan.year, ramadan.month, ramadan.day],
      [Era.hijri, 1446, 9, 1],
    );
    // The Tibetan year 2000 began with a leap month 1, on 6 February.
    final losar = ctx.calendar.convert(
      gregorian(2000, 2, 6),
      Calendar.tibetanPhugpa,
    );
    expect(
      [losar.year, losar.month, losar.day, losar.leap],
      [2000, 1, 1, Leap.month],
    );
    final back = ctx.calendar.convert(
      Calendar.tibetanPhugpa.date(2000, 1, 1, leap: Leap.month),
      Calendar.gregorian,
    );
    expect([back.year, back.month, back.day, back.leap], [2000, 2, 6, null]);
    expect(bs.year, 2072);
    expect(bs.month, 1);
    expect(bs.day, 1);
    expect(bs.era, Era.vikrama);
    expect(bs.eraYear, 2072);
    expect(
      bs.resolution,
      Resolution.tabular,
      reason: 'inside the official table',
    );

    final fixed = ctx.calendar.fixedOf(date);
    expect(fixed, 735702);
    expect(ctx.calendar.weekdayOf(date), 2, reason: 'a Tuesday');
    expect(ctx.calendar.dateOf(Calendar.gregorian, fixed).era, Era.commonEra);
    expect(ctx.calendar.monthLength(Calendar.gregorian, 2024, 2), 29);
    expect(ctx.calendar.isLeap(Calendar.gregorian, 2024), isTrue);
    expect(teistro.julianDayOfFixed(fixed), 2457126.5);
    expect(teistro.fixedOfJulianDay(2457126.75), (
      value: fixed,
      fraction: 0.25,
    ));
  });

  // Bikram Sambat arithmetic (`03-design/calendar-bikram-sambat.md` §11):
  // a 32nd a month on under each rule, and an age counted from a 32nd.
  test('a Bikram Sambat date adds and an age is counted', () {
    final ctx = context();
    final jestha = Calendar.bikramSambat.date(2081, 2, 32);
    expect(ctx.calendar.monthLength(Calendar.bikramSambat, 2081, 3), 31);
    final constrained = ctx.calendar.add(jestha, months: 1);
    expect([constrained.month, constrained.day], [3, 31]);
    final carried = ctx.calendar.add(
      jestha,
      months: 1,
      overflow: Overflow.carry,
    );
    expect([carried.month, carried.day], [4, 1]);
    expect(
      () => ctx.calendar.add(jestha, months: 1, overflow: Overflow.reject),
      throwsA(isA<TeistroException>().having((e) => e.field, 'field', 'day')),
    );
    final age = ctx.calendar.between(
      Calendar.bikramSambat.date(2050, 2, 32),
      Calendar.bikramSambat.date(2082, 2, 31),
    );
    expect([age.years, age.months, age.days], [32, 0, 0]);
    final back = ctx.calendar.between(
      Calendar.bikramSambat.date(2081, 7, 10),
      Calendar.bikramSambat.date(2050, 1, 15),
    );
    expect([back.years, back.months, back.days], [-31, -5, -26]);
    ctx.dispose();
  });

  // Anniversaries over a span (`03-design/anniversary-span.md`): a barkhi
  // and an event, each day naming the rule that chose it, and a span too
  // long refused on `years`.
  test('anniversaries over a span name the rule of each day', () {
    final ctx = teistro.context(
      profile: 'nepali-committee',
      ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
    );
    const origin = OriginLunar(month: Masa.kartika, tithi: Tithi.shuklaDashami);
    AnniversariesRequest asked({int years = 2}) => AnniversariesRequest(
      fromYear: 2026,
      years: years,
      latitudeDeg: 27.7172,
      longitudeDeg: 85.324,
      altitudeM: 1400,
      utcOffsetSeconds: 20700,
      calendar: Calendar.gregorian,
      anniversaries: const [
        Anniversary(key: 'BARKHI', origin: origin),
        Anniversary(
          key: 'WEDDING',
          origin: origin,
          kind: AnniversaryKind.event,
        ),
      ],
    );
    final span = ctx.almanac.anniversaries(asked());
    expect(span.anniversaries, hasLength(4));
    final barkhi = span.anniversaries.firstWhere((d) => d.key == 'BARKHI');
    expect(barkhi.kind, 'EKODDISHTA');
    expect(barkhi.dayRule.at.window, 'DAY_MUHURTA');
    expect(barkhi.dayRule.source, contains('Dharmasindhu'));
    final wedding = span.anniversaries.firstWhere((d) => d.key == 'WEDDING');
    expect(wedding.dayRule.at.window, 'SUNRISE');
    expect(wedding.dayRule.source, contains('convention'));
    expect(
      () => ctx.almanac.anniversaries(asked(years: 101)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'anniversaries.years',
        ),
      ),
    );
    ctx.dispose();
  });

  // The next occurrence (`03-design/next-occurrence.md`): Maghe sankranti
  // from 1 January 2026 in the first window, Vijaya Dashami within the
  // default limit, and a search too short refused on `limitDays`.
  test('a day is said in sentences and Hindi is refused', () {
    final ctx = teistro.context(
      profile: 'nepali-committee',
      ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final day = Calendar.gregorian.date(2026, 3, 18);
    final en = ctx.almanac.sentences(
      date: day,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      locale: 'en-Latn',
    );
    expect(en.sentences.first.text, 'The day is Wednesday.');
    expect(en.draft, isFalse);
    final ne = ctx.almanac.sentences(
      date: day,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      locale: 'ne-Deva-NP',
    );
    expect(ne.sentences.first.text, 'यो दिन बुधबार हो।');
    expect(ne.draft, isTrue);
    expect(
      () => ctx.almanac.sentences(
        date: day,
        place: kathmandu,
        utcOffsetSeconds: 20700,
        locale: 'hi-Deva-IN',
      ),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'locale'),
      ),
    );
    ctx.dispose();
  });

  test('the next sankranti and a pack\'s festival are found', () {
    final ctx = teistro.context(
      profile: 'nepali-committee',
      ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final sankranti = ctx.almanac.next(
      after: 2461042.0,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      kind: 'SANKRANTI',
    );
    expect(sankranti.keys, ['MAKARA_SANKRANTI']);
    expect([sankranti.day.month, sankranti.day.day], [1, 15]);
    expect([sankranti.searchedDays, sankranti.limitDays], [32, 400]);
    expect(sankranti.festivals.dated, hasLength(1));
    const panchanga = FestivalRequest(rules: FestivalPack.nepalPanchanga);
    final dashami = ctx.almanac.next(
      after: 2461042.0,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      rule: 'VIJAYA_DASHAMI',
      festivals: panchanga,
    );
    expect(dashami.keys, ['VIJAYA_DASHAMI']);
    expect(dashami.festivals.observances.first.rule, 'VIJAYA_DASHAMI');
    expect(
      () => ctx.almanac.next(
        after: 2461042.0,
        place: kathmandu,
        utcOffsetSeconds: 20700,
        rule: 'VIJAYA_DASHAMI',
        festivals: panchanga,
        limitDays: 20,
      ),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'limitDays'),
      ),
    );
    ctx.dispose();
  });

  test('a Nepali birth time resolves with the metadata a stored chart '
      'keeps', () {
    final ctx = context();
    final civil = Calendar.gregorian.date(1986, 1, 1).at(hour: 0, minute: 20);
    expect(
      civil.date,
      isA<CalendarDate>().having((d) => d.year, 'year', 1986),
      reason: 'the layer builds the value the generated class holds',
    );
    final zone = ianaZone('Asia/Kathmandu');
    final resolved = ctx.time.resolve(civil, zone);
    expect(resolved.instantJdUtc, closeTo(2446431.2743056, 1e-6));
    expect(
      resolved.offsetSeconds,
      20700,
      reason: '+05:45, the offset that began that midnight',
    );
    expect(resolved.era, ZoneEra.current);
    expect(resolved.source, ZoneSource.iana);
    expect(resolved.timeKnown, isTrue);
    expect(resolved.warnings, isEmpty, reason: 'nothing had to be guessed');
    expect(resolved.tzdbVersion, matches(r'^20\d\d[a-z]$'));

    final back = ctx.time.civilOf(
      resolved.instantJdUtc,
      zone,
      Calendar.gregorian,
    );
    expect(back.civil.date.year, 1986);
    expect(back.civil.time.minute, 20);
    expect(back.civil.time.hasTime, isTrue);
    expect(back.resolution.offsetSeconds, 20700);

    expect(
      () => ctx.time.resolve(civil, ianaZone('Asia/Kathmandou')),
      throwsA(isA<TeistroException>()),
    );
  });

  test('the time scales convert with what they applied', () {
    final ctx = context();
    final tt = ctx.time.convert(2451544.5, Scale.utc, Scale.tt);
    expect(
      tt.deltaTSeconds,
      closeTo(64.184, 1e-9),
      reason: 'exact through the leap-second table',
    );
    expect(tt.deltaTSource, DeltaTSource.leapSeconds);
    expect(tt.deltaTModel, 'TABLE_THEN_MODEL');
    expect(tt.jd, closeTo(2451544.5 + 64.184 / 86400, 1e-12));
    expect(
      ctx.time.convert(tt.jd, Scale.tt, Scale.utc).jd,
      closeTo(2451544.5, 1e-9),
    );

    final delta = ctx.time.deltaT(2451544.5);
    expect(delta.seconds, closeTo(63.83, 0.02));
    expect(delta.source, DeltaTSource.table);
  });

  test('positions come back in the frame asked for', () {
    final ctx = context();
    final frame = teistro.canonicalFrame;
    expect(frame.centre, Centre.geocentric);
    expect(frame.coordinates, Coordinates.ecliptic);
    expect(frame.sidereal, isFalse);
    expect(frame.ayanamsha, isNull, reason: 'a tropical frame carries none');
    final again = teistro.unpackFrame(teistro.packFrame(frame));
    expect(again.centre, frame.centre);
    expect(again.equinox, frame.equinox);
    expect(again.sidereal, frame.sidereal);
    // A copy changes what it names and keeps the rest.
    final lahiri = frame.copyWith(sidereal: true, ayanamsha: Ayanamsha.lahiri);
    expect((lahiri.sidereal, lahiri.ayanamsha), (true, Ayanamsha.lahiri));
    expect(
      lahiri.copyWith(ayanamsha: null).ayanamsha,
      Ayanamsha.lahiri,
      reason: 'copyWith keeps what it is given null for; it never clears',
    );
    expect(
      (
        lahiri.centre,
        lahiri.equinox,
        lahiri.coordinates,
        lahiri.lightTime,
        lahiri.aberration,
        lahiri.deflection,
        lahiri.nutation,
      ),
      (
        frame.centre,
        frame.equinox,
        frame.coordinates,
        frame.lightTime,
        frame.aberration,
        frame.deflection,
        frame.nutation,
      ),
    );

    final positions = ctx.positions(
      instants: [2451545.0, 2451546.0],
      bodies: [Body.sun, Body.moon, Body.mars],
    );
    expect(positions.bodyKeys, [Body.sun, Body.moon, Body.mars]);
    expect(positions.jds, [2451545.0, 2451546.0]);
    expect(positions.timeScale, TimeScale.ut1);
    expect(positions.cells.length, 6, reason: 'two instants by three bodies');

    final sun = positions.at(0, 0);
    expect(sun.longitude, inInclusiveRange(0, 360));
    expect(sun.status, 0);
    expect(
      positions.at(0, 1).longitudeSpeed.abs(),
      greaterThan(sun.longitudeSpeed.abs()),
      reason: 'the Moon moves faster than the Sun',
    );
    expect(() => positions.at(2, 0), throwsRangeError);

    expect(positions.provenance.profile, 'nepali-drik');
    expect(positions.provenance.calculationVersion, 1);
    expect(positions.provenance.settingsHash, ctx.settingsHash);
    expect(
      positions.provenance.provider.frame,
      'GEOCENTRIC/OF_DATE/ECLIPTIC/TROPICAL/APPARENT',
    );

    // Without an ephemeris the call is a missing capability naming the
    // option a consumer sets, and hinting at what to pass -- the same
    // field and the same hint as Node, Python and C.
    final bare = context(testProvider: false);
    expect(
      () => bare.positions(instants: [2451545.0], bodies: [Body.sun]),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.capability)
            .having((e) => e.field, 'field', 'ephemeris')
            .having((e) => e.hint, 'hint', contains('BUILTIN')),
      ),
    );
    expect(
      () => ctx.positions(instants: [], bodies: [Body.sun]),
      throwsArgumentError,
    );
    expect(
      () => ctx.positions(instants: [2451545.0], bodies: []),
      throwsArgumentError,
    );
  });

  test('the locale engine renders typed parameters, and says where '
      'from', () {
    final ctx = context();
    final rendered = ctx.intl.render('sdk.reason.grahaInBhava', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 7,
    });
    expect(rendered.from, 'ne-Deva-NP');
    expect(rendered.fallback, isFalse);
    expect(rendered.override, isFalse);
    expect(rendered.warningList, isEmpty);
    expect(rendered.text, contains('७'), reason: 'the Nepali numeral seven');
    expect(ctx.intl.has('sdk.reason.grahaInBhava'), isTrue);
    expect(ctx.intl.has('sdk.nope.missing'), isFalse);

    // A missing message renders as its key with a warning, never an error.
    final missing = ctx.intl.render('sdk.nope.missing');
    expect(missing.from, isNull);
    expect(missing.warningList, isNotEmpty);

    ctx.intl.locale = 'en-Latn';
    expect(ctx.intl.locale, 'en-Latn');
    expect(
      ctx.intl.render('sdk.reason.grahaInBhava', {
        'graha': {r'$entity': 'graha.JUPITER'},
        'bhava': 7,
      }).text,
      contains('Jupiter'),
    );
    expect(
      () => ctx.intl.locale = 'fr-Latn',
      throwsA(
        isA<TeistroException>()
            .having((e) => e.field, 'field', 'locale')
            .having((e) => e.hint, 'hint', contains('sa-Deva')),
      ),
    );
  });

  test('a rendered message carries its markup in parts, and plain text '
      'in one', () {
    final ctx = context();
    ctx.intl.locale = 'en-Latn';
    // `sdk.reason.lordship` is one of the two shipped messages that use
    // MF2 markup. Without the parts a renderer can only ever print the
    // text, which is why the message may as well not have had the tag.
    final rich = ctx.intl.render('sdk.reason.lordship', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 5,
    });
    expect(rich.text, 'Jupiter rules house 5');
    final parts = rich.partList;
    expect(parts.map((p) => p.toString()).toList(), [
      '<open b>',
      'Jupiter',
      '<close b>',
      ' rules house 5',
    ]);
    expect(parts.first.options, isEmpty);
    // A renderer that knows no tag joins the text parts and loses nothing.
    expect(parts.where((p) => p.isText).map((p) => p.value).join(), rich.text);

    // A message with no markup is the one text part, made here rather
    // than carried: the boundary sends nothing for it.
    final plain = ctx.intl.render('sdk.reason.grahaInBhava', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 7,
    });
    expect(plain.parts, '[]', reason: 'the boundary sent none');
    expect(plain.partList.single.value, plain.text);
  });

  test('a quantity is its own type, and its constructor checks the '
      'range', () {
    final ctx = context();
    addTearDown(ctx.dispose);
    expect(
      Latitude(27.7172),
      27.7172,
      reason: 'a branded number is a double at run time',
    );
    expect(() => Latitude(91), throwsRangeError);
    expect(() => Longitude(-181), throwsRangeError);
    expect(() => Altitude(20000), throwsRangeError);

    // The place reaches the boundary and comes back through the frame.
    final positions = ctx.positions(
      instants: [2451545.0],
      bodies: [Body.sun],
      observer: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
    );
    expect(positions.cells.length, 1);
  });

  test('a catalogue key packs to an id and back', () {
    final ctx = context();
    final id = ctx.keys.id('graha.SUN');
    expect(
      id,
      1 << 16,
      reason: 'the kind in the high half, the member in the low',
    );
    expect(ctx.keys.name(id), 'graha.SUN');
    expect(
      ctx.keys.name(ctx.keys.id('nakshatra.ASHWINI')),
      'nakshatra.ASHWINI',
    );
    expect(
      () => ctx.keys.name(0xffffffff),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.status,
          'status',
          Status.unsupported,
        ),
      ),
    );
  });

  test('a disposed context says so, and disposing twice is allowed', () {
    final ctx = context();
    expect(ctx.profile, isNotEmpty);
    ctx.dispose();
    // Idempotent: the finaliser and an explicit call both run it.
    ctx.dispose();
    // Named here rather than at the boundary, which would only say
    // `invalid argument` and not which argument. The Node and Python
    // bindings answer the same way.
    expect(
      () => ctx.profile,
      throwsA(
        isA<StateError>().having(
          (e) => e.message,
          'message',
          contains('disposed'),
        ),
      ),
    );
    expect(
      () => ctx.positions(instants: [2451545.0], bodies: [Body.sun]),
      throwsA(isA<StateError>()),
    );
  });

  test('a birth with no time is refused, or reported, but never guessed', () {
    final day = Calendar.bikramSambat.date(2042, 9, 17);
    final zone = ianaZone('Asia/Kathmandu');

    // No policy: refused by name, with the hint naming the three choices.
    final strict = context(locale: null);
    addTearDown(strict.dispose);
    expect(
      () => strict.time.resolve(day.whenUnknown, zone),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.message, 'message', contains('has no time of day'))
            .having(
              (e) => e.hint,
              'hint',
              contains('NOON, MIDNIGHT or SUNRISE'),
            )
            .having((e) => e.field, 'field', 'time'),
      ),
    );

    // NOON: answered, and said twice — the resolution reports the time as
    // unknown *and* warns, so a stored chart cannot claim a time it never
    // had.
    final noon = context(
      locale: null,
      settings: const {
        'time': {'unknown_time': 'NOON'},
      },
    );
    addTearDown(noon.dispose);
    final resolved = noon.time.resolve(day.whenUnknown, zone);
    expect(resolved.timeKnown, isFalse);
    expect(
      resolved.warnings.map((w) => w.key),
      contains('TIME_UNKNOWN_FALLBACK'),
    );
    expect(resolved.instantJdUtc.isFinite, isTrue);

    // A known time on the same date resolves with the time known and no
    // warning: this record sits on the day Nepal moved to +05:45.
    final exact = strict.time.resolve(day.at(hour: 0, minute: 20), zone);
    expect(exact.timeKnown, isTrue);
    expect(exact.offsetSeconds, 5 * 3600 + 45 * 60);
    expect(exact.warnings, isEmpty);
  });

  _engineTests();
}

// ── The engine's own operations ──────────────────────────────────────
// The test provider ships a two-function manifest, so these run with no
// real engine present and still exercise the whole route.

void _engineTests() {
  test('the engine names its own operations', () {
    final engine = context().engine;
    expect(engine.names, contains('tp_echo'));
    expect(engine.has('tp_sum'), isTrue);
    expect(engine.manifest['engine'], 'test-provider');
  });

  test('an operation is called by the name the engine gives it', () {
    final engine = context().engine;
    expect(engine('tp_echo', {'value': 6.0}), {'value': 6.0});
    final summed =
        engine('tp_sum', {
              'values': [1.0, 2.0, 3.5],
            })
            as Map<String, Object?>;
    expect(summed['total'], 6.5);
  });

  test('the manifest carries the role of every parameter', () {
    final engine = context().engine;
    final signature = engine.signature('tp_sum')!;
    final roles = [
      for (final param in signature['params'] as List<Object?>)
        (param as Map<String, Object?>)['role'],
    ];
    expect(roles, ['array_in', 'array_len', 'scalar_out']);
    expect(engine.signature('tm_no_such_thing'), isNull);
  });

  test("the engine's own refusal comes back", () {
    final engine = context().engine;
    expect(
      () => engine('tm_eclipse_when'),
      throwsA(predicate((e) => '$e'.contains('tm_eclipse_when'))),
    );
  });

  // An area is a **value**: built once with the context, held, and
  // passable to something that needs only that much of the SDK. That is
  // what makes the grouping worth having rather than merely tidy
  // (`03-design/surface-areas.md`).
  /// **An engine, plugged in** (ADR-0029): the 98% path, where a
  /// consumer names an adapter's platform binary and never sees a
  /// vtable.
  ///
  /// It runs only where the adapter has been built and its data is
  /// present, because a checkout has neither and a test that failed for
  /// that would fail for everyone. `TEISTRO_TEIMERIS_ADAPTER` names the
  /// library — the same variable `crates/ffi/tests/abi.rs` reads for the
  /// same reason.
  test('an ephemeris is plugged in by naming its platform binary', () {
    final plugin = Platform.environment['TEISTRO_TEIMERIS_ADAPTER'];
    if (plugin == null || plugin.isEmpty) {
      printOnFailure(
        'the adapter is built separately; set TEISTRO_TEIMERIS_ADAPTER to its library',
      );
      markTestSkipped('the adapter is not built in this checkout');
      return;
    }
    final ctx = teistro.context(
      profile: 'parashari-classical',
      ephemeris: [PluginEphemeris(plugin: plugin)],
    );
    addTearDown(ctx.dispose);
    final sky = ctx.positions(instants: [2451545.0], bodies: [Body.sun]);
    // The Sun at J2000 is near 280.4°, which is astronomy rather than
    // this package: what is tested is that a real engine answered.
    expect(sky.at(0, 0).longitude, closeTo(280.37, 0.5));
    // And its own functions came with it, which no SDK operation offers.
    expect(ctx.engine.manifest['engine'], 'teimeris');
    expect(
      (ctx.engine('tm_body_name', {'body': 0}) as Map<String, Object?>)['buf'],
      'Sun',
    );
  });

  /// **Nepal's official days**: the Surya Siddhanta takes the adapters
  /// after it, as the façade's chain does, so the committee's limbs are
  /// the text's and its sunrise the engine's. Before this, each entry
  /// opened alone and the adapter after the text was never asked.
  test('the Surya Siddhanta takes its sunrise from the adapter after it', () {
    expect(nepalOfficialChain(), hasLength(1));
    String stamp(Context ctx) =>
        ctx
            .positions(instants: [2451545.0], bodies: [Body.sun])
            .provenance
            .provider
            .dataVersion;
    // An adapter that is not there is passed over, as the façade passes
    // over it, and the text keeps the built-in's sunrise.
    final fellBack = teistro.nepalOfficial(
      modern: const [PluginEphemeris(plugin: '/nowhere/adapter.so')],
    );
    addTearDown(fellBack.dispose);
    expect(stamp(fellBack), contains('sunrise from teistro-builtin'));
    final plugin = Platform.environment['TEISTRO_TEIMERIS_ADAPTER'];
    if (plugin == null || plugin.isEmpty) {
      markTestSkipped('the adapter is not built in this checkout');
      return;
    }
    final official = teistro.nepalOfficial(
      modern: [PluginEphemeris(plugin: plugin)],
      locale: 'en-Latn',
    );
    addTearDown(official.dispose);
    expect(stamp(official), contains('sunrise from teimeris'));
  });

  /// Two reckonings of November 2025 at Kathmandu: the committee's over
  /// the Surya Siddhanta and the modern one. Each tithi difference names
  /// both members and the instant they meet under each sky, on opposite
  /// sides of its sunrise; Haribodhini Ekadashi is fasted on the 1st by the
  /// text and the 2nd by the modern sky (`03-design/setup-disagreements.md`).
  // Dashain 2025 at Kathmandu as an iCalendar feed: one event for each
  // observance and fast the festival answer gives, Vijaya Dashami on 2
  // October, every line CRLF and at most 75 octets, and the same request
  // the same bytes (`03-design/icalendar.md`).
  test('an almanac writes its festivals as an iCalendar feed', () {
    final sky = teistro.nepalOfficial();
    addTearDown(sky.dispose);
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final from = Calendar.gregorian.date(2025, 9, 20);
    final to = Calendar.gregorian.date(2025, 11, 10);
    const panchanga = FestivalRequest(rules: FestivalPack.nepalPanchanga);
    String feed({int? refreshMinutes = 1440}) => sky.almanac.festivalCalendar(
      from: from,
      to: to,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      festivals: panchanga,
      stampUnixSeconds: 1767225600,
      name: 'Dashain',
      refreshMinutes: refreshMinutes,
    );
    final ics = feed();
    expect(ics, startsWith('BEGIN:VCALENDAR\r\nVERSION:2.0\r\n'));
    final hinted = sky.almanac.festivalCalendar(
      from: from,
      to: to,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      festivals: panchanga,
      stampUnixSeconds: 1767225600,
      color: 'gold',
      categories: const ['Nepal'],
      alarm: const FeedAlarm(triggerMinutes: -60),
      events: [
        SuppliedEvent(
          uid: 'mine@app',
          summary: 'Mine',
          day: const DayAsked(year: 2025, month: 10, day: 5),
        ),
      ],
    );
    expect(hinted, contains('COLOR:gold\r\n'));
    expect(hinted, contains('UID:mine@app\r\n'));
    expect(hinted, contains('BEGIN:VALARM\r\n'));
    final digest = sky.almanac.digest(
      from: from,
      days: 3,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      festivals: panchanga,
    );
    expect(decodeDigest(digest).dayCount, 3);
    expect(
      () => sky.almanac.digest(
        from: from,
        days: 0,
        place: kathmandu,
        utcOffsetSeconds: 20700,
      ),
      throwsA(isA<TeistroException>()),
    );
    expect(ics, contains('X-WR-CALNAME:Dashain\r\n'));
    final answer = sky.almanac.festivals(
      from: from,
      to: to,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      festivals: panchanga,
    );
    expect(
      'DTSTART;VALUE=DATE:'.allMatches(ics).length,
      answer.observances.length + answer.ekadashis.length + answer.dated.length,
    );
    // And a timed event for each fast's parana.
    final parana =
        answer.ekadashis.where((fast) => fast.parana?.kept != null).length;
    expect(parana, greaterThan(0));
    expect('\r\nDTSTART:'.allMatches(ics).length, parana);
    // Kartik's sankranti on the first of the Bikram Sambat month.
    final tula = answer.dated.firstWhere((day) => day.rule == 'TULA_SANKRANTI');
    expect(
      (tula.rule, tula.day.month, tula.day.day),
      ('TULA_SANKRANTI', 10, 18),
    );
    expect(
      (tula.on.calendar, tula.on.month, tula.on.day),
      (Calendar.bikramSambat, 7, 1),
    );
    expect(
      ics.replaceAll('\r\n ', ''),
      contains(
        'DTSTART;VALUE=DATE:20251002\r\nDTEND;VALUE=DATE:20251003\r\n'
        'SUMMARY:Vijaya Dashami\r\n',
      ),
    );
    for (final line in ics.split('\r\n')) {
      expect(utf8.encode(line).length, lessThanOrEqualTo(75), reason: line);
    }
    expect(feed(), ics);
    expect(feed(refreshMinutes: null), isNot(contains('REFRESH-INTERVAL')));
  });

  // 2026 is the 40th year of the 17th rabjung, Parabhava, Fire–male–Horse
  // (Janson, appendix B); a year before the first rabjung is refused by
  // name (`calendars/tibetan.md`).
  test('a Tibetan year says its names', () {
    final sky = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    addTearDown(sky.dispose);
    expect(
      sky.calendar.tibetanYear(2026),
      const TibetanYear(
        year: 2026,
        rabjung: 17,
        yearInRabjung: 40,
        samvatsara: Samvatsara.parabhava,
        element: TibetanElement.fire,
        gender: Gender.male,
        animal: TibetanAnimal.horse,
      ),
    );
    expect(
      () => sky.calendar.tibetanYear(1026),
      throwsA(isA<TeistroException>().having((e) => e.field, 'field', 'year')),
    );
  });

  // 08:00 on 15 January 2026 in Kathmandu, the morning after Makara
  // sankranti: Siddharthi's year, Magha (purnimanta) and Shishira at
  // sunrise, Krishna Dwadashi in Jyeshtha at the rite, the Sun in Capricorn
  // and Jupiter in Gemini; at 04:00 it is still the 14th's, and the
  // tropical ayana turned at the solstice (`03-design/sankalpa.md`).
  test('an almanac says the time a sankalpa recites', () {
    final sky = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    addTearDown(sky.dispose);
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    SankalpaAnswer ask(
      double at, [
      AyanaReckoning ayana = AyanaReckoning.sidereal,
      int offset = 20700,
    ]) => sky.almanac.sankalpa(
      at: at,
      place: kathmandu,
      utcOffsetSeconds: offset,
      ayana: ayana,
    );
    final found = ask(2461055.59375);
    final value = found.value;
    expect(found.provenance.inputHash, isNotEmpty);
    expect([value.date.year, value.date.month, value.date.day], [2026, 1, 15]);
    expect(value.vara, Vara.guruvara);
    expect(value.year.samvatsara, Samvatsara.siddharthi);
    expect(value.year.vikrama, 2082);
    expect(value.year.kali, 5126);
    expect(value.month.month, Masa.magha);
    expect(value.month.amanta, Masa.pausha);
    expect(value.month.kind, MonthKind.nija);
    expect(value.ritu, Ritu.shishira);
    expect(value.ayana, Ayana.uttarayana);
    expect(value.ayanaReckoning, AyanaReckoning.sidereal);
    expect(value.tithi, Tithi.krishnaDwadashi);
    expect(value.nakshatra, Nakshatra.jyeshtha);
    expect(
      [value.sun, value.moon, value.jupiter],
      [Rashi.capricorn, Rashi.scorpio, Rashi.gemini],
    );
    expect(value.sunrise < value.at && value.at < value.nextSunrise, isTrue);

    expect(ask(2461055.59375 - 4 / 24).value.date.day, 14);
    const between = 2461045.6;
    expect(ask(between).value.ayana, Ayana.dakshinayana);
    expect(ask(between, AyanaReckoning.tropical).value.ayana, Ayana.uttarayana);
    expect(
      () => ask(between, AyanaReckoning.sidereal, 999999),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'sankalpa.utcOffsetSeconds',
        ),
      ),
    );
  });

  // The central holidays' pack says what each rule is, and marks a day the
  // moon decides (`03-design/india-central-holidays.md`).
  test('a festival pack lists its rules with their values', () {
    final sky = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    addTearDown(sky.dispose);
    final listing = sky.almanac.festivalPack('INDIA_CENTRAL');
    expect(listing['pack'], 'INDIA_CENTRAL');
    final rows =
        (listing['rules']! as List<Object?>).cast<Map<String, Object?>>();
    Map<String, Object?> row(String key) =>
        rows.firstWhere((rule) => rule['key'] == key);
    expect(row('EID_AL_FITR')['subjectTo'], 'MOON_SIGHTING');
    expect((row('HOLI')['meta']! as Map<String, Object?>)['kind'], 'GAZETTED');
    expect(() => sky.almanac.festivalPack('NO_SUCH_PACK'), throwsA(anything));
    expect(FestivalPack.indiaCentral.key, listing['pack']);
    expect(
      const FestivalRequest(
        rules: FestivalPack.indiaCentral,
        only: MetaFilter(kinds: [FestivalKind.gazetted]),
      ),
      isNot(const FestivalRequest(rules: FestivalPack.indiaCentral)),
      reason: 'a filter is part of the request',
    );
  });

  // A sapinda's death at 10:00 on 11 January 2026 in Kathmandu: ten days
  // by the relation's row, from the 11th; heard fifty days on, three days
  // by the late table's row within ninety; a second death three days later,
  // during the first, ends with it (`03-design/ashauch.md`).
  test('an almanac says the period of impurity after a death', () {
    final sky = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    addTearDown(sky.dispose);
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1337),
    );
    const at = 2461051.677083;
    AshauchAnswer ask(AshauchRequest request) => sky.almanac.ashauch(
      request: request,
      place: kathmandu,
      utcOffsetSeconds: 20700,
    );
    final found = ask(
      const AshauchRequest(
        key: 'grandfather',
        event: AshauchDeath(at: at),
        relation: AshauchRelation.sapinda,
      ),
    );
    final value = found.value;
    expect(found.provenance.inputHash, isNotEmpty);
    expect(value.key, 'grandfather');
    expect(value.kind, AshauchKind.death);
    expect(value.relation, AshauchRelation.sapinda);
    expect(value.clause.clause, 'RELATION');
    expect(value.clause.said, isNotEmpty);
    expect(value.days, 10);
    expect(value.length, AshauchLength.full);
    final dayOne = value.dayOne;
    expect([dayOne.year, dayOne.month, dayOne.day], [2026, 1, 11]);
    expect(value.dayOneBy.by, 'DAYLIGHT');
    expect(value.end > value.start, isTrue);
    expect(value.overlap, isNull);
    expect(value.table, 'DHARMASINDHU');

    final heard =
        ask(
          const AshauchRequest(
            key: 'heard',
            event: AshauchDeath(at: at),
            relation: AshauchRelation.sapinda,
            heardAt: at + 50,
          ),
        ).value;
    expect(heard.clause.clause, 'LATE');
    expect(heard.clause.withinDays, 90);
    expect(heard.clause.abroad, isFalse);
    expect(heard.length, const AshauchLength.count(3));

    final second =
        ask(
          AshauchRequest(
            key: 'second',
            event: const AshauchDeath(at: at + 3),
            relation: AshauchRelation.sapinda,
            during: [value.running],
          ),
        ).value;
    expect(second.overlap?.overlap, 'ENDS_WITH_EARLIER');
    expect(second.overlap?.earlier, 'grandfather');
    expect(second.end, value.end);

    final named = sky.almanac.ashauch(
      request: const AshauchRequest(
        key: 'mitakshara',
        event: AshauchDeath(at: at),
        relation: AshauchRelation.sapinda,
      ),
      place: kathmandu,
      utcOffsetSeconds: 20700,
      tradition: AshauchTradition.mitakshara,
    );
    expect(named.value.table, 'MITAKSHARA');
  });

  // A household at Kathmandu (`03-design/ashauch.md` §11): a father dies at
  // 10:00 on 11 January 2026 and the mother at 14:00 on 15 January. The son
  // keeps the father's end and a pakshini; a sapinda's second period ends
  // with the first. The relation resolver and the rite answer without a
  // sky (§§10, 12); the death-time check reads the Moon (`death-time.md`).
  test(
    'an almanac says a household, a relation, a rite and the death time',
    () {
      final sky = teistro.context(
        profile: 'nepali-drik',
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      addTearDown(sky.dispose);
      final kathmandu = Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1337),
      );
      const father = 2461051.677083;
      const mother = 2461055.84375;
      final found = sky.almanac.household(
        household: const Household(
          events: [
            HouseholdEvent(
              key: 'father',
              event: AshauchDeath(
                at: father,
                deceased: Deceased(sex: AshauchSex.male),
              ),
            ),
            HouseholdEvent(
              key: 'mother',
              event: AshauchDeath(
                at: mother,
                deceased: Deceased(sex: AshauchSex.female),
              ),
            ),
          ],
          observers: [
            HouseholdObserver(
              key: 'son',
              ties: [
                HouseholdTie(event: 'father', relation: AshauchRelation.child),
                HouseholdTie(event: 'mother', relation: AshauchRelation.child),
              ],
            ),
            HouseholdObserver(
              key: 'nephew',
              ties: [
                HouseholdTie(
                  event: 'father',
                  relation: AshauchRelation.sapinda,
                ),
                HouseholdTie(
                  event: 'mother',
                  relation: AshauchRelation.sapinda,
                ),
              ],
            ),
          ],
        ),
        place: kathmandu,
        utcOffsetSeconds: 20700,
      );
      expect(found.provenance.inputHash, isNotEmpty);
      final [son, nephew] = found.value.observers;
      expect(son.periods[1].overlap?.overlap, 'MOTHER_AFTER_FATHER');
      expect(son.pureAt, son.periods[1].end);
      expect(nephew.periods[1].overlap?.overlap, 'ENDS_WITH_EARLIER');
      expect(nephew.pureAt, nephew.periods[0].end);
      expect(
        () => sky.almanac.household(
          household: const Household(
            events: [],
            observers: [
              HouseholdObserver(
                key: 'son',
                ties: [
                  HouseholdTie(event: 'uncle', relation: AshauchRelation.child),
                ],
              ),
            ],
          ),
          place: kathmandu,
          utcOffsetSeconds: 20700,
        ),
        throwsA(
          isA<TeistroException>().having(
            (error) => error.field,
            'field',
            'household.household.observers[0].ties[0].event',
          ),
        ),
      );

      AshauchAnswer ask(AshauchRequest request) => sky.almanac.ashauch(
        request: request,
        place: kathmandu,
        utcOffsetSeconds: 20700,
      );
      final first =
          ask(
            const AshauchRequest(
              key: 'first',
              event: AshauchDeath(at: father),
              relation: AshauchRelation.sapinda,
            ),
          ).value;
      final second =
          ask(
            AshauchRequest(
              key: 'second',
              event: const AshauchDeath(at: father + 9.55),
              relation: AshauchRelation.sapinda,
              during: [first.running],
            ),
          ).value;
      expect(second.overlap?.overlap, 'LAST_NIGHT');
      expect(second.overlap?.from, first.end);
      expect(second.running.extendedFrom, first.end);
      expect(second.running.extendedDays, second.overlap?.days);

      final kin = sky.almanac.ashauchKin(
        const KinQuery(
          kin: KinAsked(
            path: [KinStep.father, KinStep.father, KinStep.son, KinStep.son],
            observer: KinObserver(sex: AshauchSex.male),
          ),
        ),
      );
      expect(kin.relation, AshauchRelation.sapinda);
      expect(kin.keeps, KinKeeps.period);
      expect(kin.degree?.observer, 3);
      expect(
        () => sky.almanac.ashauchKin(
          const KinQuery(
            kin: KinAsked(
              path: [KinStep.son, KinStep.father],
              observer: KinObserver(sex: AshauchSex.male),
            ),
          ),
        ),
        throwsA(
          isA<TeistroException>().having(
            (error) => error.field,
            'field',
            'kin.kin.path[1]',
          ),
        ),
      );
      final rite = sky.almanac.ashauchRite(
        const RiteQuery(
          rite: AshauchRiteAsked(
            rite: AshauchRite.marriage,
            begun: true,
            noOtherWay: true,
          ),
        ),
      );
      expect(rite.verdict, RiteVerdict.goesOn);

      final death = sky.almanac.deathTime(
        at: father,
        place: kathmandu,
        utcOffsetSeconds: 20700,
      );
      expect(death.value.table, 'DHARMASINDHU');
      expect(
        death.value.death.panchaka,
        [Rashi.aquarius, Rashi.pisces].contains(death.value.death.moon),
      );
      expect(death.value.cremation, isNull);
      expect(
        () => sky.almanac.deathTime(
          at: father,
          crematedAt: father - 1,
          place: kathmandu,
          utcOffsetSeconds: 20700,
        ),
        throwsA(
          isA<TeistroException>().having(
            (error) => error.field,
            'field',
            'deathTime.crematedAt',
          ),
        ),
      );
    },
  );

  // A death at 10:00 on 20 October 2026 in Kathmandu: the sixteen, fifteen
  // of them done again after a twelfth-day sapindana (DS p. 331); with
  // Dhakal's readings for Nepal the first abdika is the eleventh day's and
  // the anniversary the second (`03-design/shraddha.md` §3.3).
  test('an almanac says the rites of the first year after a death', () {
    final sky = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    addTearDown(sky.dispose);
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1337),
    );
    const at = 2461333.677083;
    FirstYearAnswer ask(FirstYearRequest request) => sky.almanac.firstYear(
      request: request,
      place: kathmandu,
      utcOffsetSeconds: 20700,
    );
    List<int> ymd(CalendarDate day) => [day.year, day.month, day.day];

    final found = ask(const FirstYearRequest(key: 'grandfather', at: at));
    final year = found.value;
    expect(found.provenance.inputHash, isNotEmpty);
    expect(year.key, 'grandfather');
    expect(year.rites.where((rite) => rite.place != null), hasLength(16));
    expect(year.rites.where((rite) => rite.drawnTo != null), hasLength(15));
    expect(year.rites.first.rite, Rite.adyaMasika);
    final sapindana = year.rites.firstWhere(
      (rite) => rite.rite == Rite.sapindana,
    );
    expect(ymd(sapindana.day), ymd(year.sapindana));

    final nepal =
        ask(
          const FirstYearRequest(
            key: 'grandfather',
            at: at,
            firstAbdika: FirstAbdika.eleventhDay,
            firstYearAdhika: FirstYearAdhika.nija,
          ),
        ).value;
    final abdikas = [
      for (final rite in nepal.rites)
        if (rite.rite == Rite.abdika) rite,
    ];
    final first = abdikas.firstWhere((rite) => rite.year == 1);
    expect(ymd(first.day), ymd(nepal.eleventhDay));
    expect(abdikas.where((rite) => rite.year == 2), isNotEmpty);
  });

  test('two reckonings say where they disagree and why', () {
    final official = teistro.nepalOfficial();
    addTearDown(official.dispose);
    final modern = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    addTearDown(modern.dispose);
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final from = Calendar.gregorian.date(2025, 11, 1);
    final to = Calendar.gregorian.date(2025, 11, 30);
    final found = official.almanac.disagreements(
      modern,
      from: from,
      to: to,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      festivals: FestivalRequest(rules: FestivalPack.nepal),
    );
    final days = found.days!;
    expect(days.compared, 30);
    expect(days.otherProfile, 'nepali-drik');
    expect(days.otherSettingsHash, modern.settingsHash);
    expect(
      [
        for (final day in days.days)
          if (day.tithi != null) day.date.day,
      ],
      [1, 2, 9, 10, 11],
    );
    final first = days.days.first.tithi!;
    expect(first.official.member, Tithi.shuklaEkadashi);
    expect(first.other.member, Tithi.shuklaDashami);
    expect(first.official.change!.minutesFromSunrise, lessThan(0));
    expect(first.other.change!.minutesFromSunrise, greaterThan(0));
    final haribodhini = found.festivals!.ekadashis.firstWhere(
      (fast) => fast.tithi == Tithi.shuklaEkadashi,
    );
    expect((haribodhini.official!.day.day, haribodhini.other!.day.day), (1, 2));

    final lean = official.almanac.disagreements(
      modern,
      from: from,
      to: to,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      days: false,
      festivals: FestivalRequest(rules: FestivalPack.nepal),
    );
    expect(lean.days, isNull);
    expect(lean.festivals!.ekadashis, found.festivals!.ekadashis);
    expect(
      () => official.almanac.disagreements(
        modern,
        from: from,
        place: kathmandu,
        utcOffsetSeconds: 20700,
        days: false,
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'disagreements.days',
        ),
      ),
    );
  });

  /// A chain is **ordered and explicit** (ADR-0029): tried in order, and
  /// a refusal names every entry that failed rather than only the last,
  /// which would hide the one the caller actually wanted. Needs no
  /// adapter.
  test('an ephemeris chain is tried in order and refuses naming each', () {
    // An adapter that is not there, then the built-in: the fallback the
    // caller wrote down.
    final fellBack = teistro.context(
      profile: 'parashari-classical',
      ephemeris: const [
        PluginEphemeris(plugin: '/nowhere/adapter.so'),
        NamedEphemeris(Ephemeris.builtin),
      ],
    );
    addTearDown(fellBack.dispose);
    expect(
      fellBack
          .positions(instants: [2451545.0], bodies: [Body.sun])
          .at(0, 0)
          .longitude,
      closeTo(280.37, 0.5),
    );

    // Nothing in the chain opening is one refusal that names each.
    expect(
      () => teistro.context(
        ephemeris: const [
          PluginEphemeris(plugin: '/a.so'),
          PluginEphemeris(plugin: '/b.so'),
        ],
      ),
      throwsA(
        predicate((e) => '$e'.contains('/a.so') && '$e'.contains('/b.so')),
      ),
    );

    // A chain of none names nothing, which is a mistake rather than a
    // default.
    expect(
      () => teistro.context(ephemeris: const []),
      throwsA(predicate((e) => '$e'.contains('names nothing'))),
    );
  });

  test('an area is a value that can be held and passed', () {
    final ctx = context();
    final calendar = ctx.calendar;
    expect(calendar, same(ctx.calendar), reason: 'the same object every read');
    expect(calendar.isLeap(Calendar.gregorian, 2024), isTrue);
    expect(ctx.time.deltaT(2451545.0).seconds, greaterThan(60));
    expect(ctx.keys.name(ctx.keys.id('graha.SUN')), 'graha.SUN');
  });

  test('a theme writes each drawing as SVG, and a wrong one is refused by its '
      'field', () {
    final ctx = context();
    List<Drawing> found(ChartTheme? theme) =>
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              drawings: const [
                (ChartLayout.northIndian, Varga.d1),
                (ChartLayout.westernWheel, Varga.d1),
              ],
              theme: theme,
            )
            .drawings;

    expect(found(null).first.svg, isNull, reason: 'no theme, no SVG');
    final [north, wheel] = found(ChartTheme.dark);
    expect(north.svg, startsWith('<svg xmlns="http://www.w3.org/2000/svg"'));
    expect(north.svg, contains('data-body="graha.SUN">सू'));
    expect(north.svg, contains('fill="#121212"'));
    expect(wheel.svg, contains('<line '));

    final glyphs =
        found(
          ChartTheme.light.copyWith(
            style: const ThemeStyle(size: 600),
            content: const ThemeContent(
              bodyForm: BodyForm.glyph,
              cellLabel: CellLabel.house,
              retrogradeMark: '',
            ),
          ),
        ).first.svg;
    expect(glyphs, contains('viewBox="0 0 600 600"'));
    expect(glyphs, contains('data-body="graha.SUN">☉'));

    expect(
      () => found(
        ChartTheme.light.copyWith(style: const ThemeStyle(ink: 'black')),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'theme.style.ink',
        ),
      ),
    );
    ctx.dispose();
  });

  test('rules are answered in the same crossing, and a wrong one is refused '
      'by its field', () {
    final ctx = context();
    Map<String, Object?>? found(RuleRequest? rules) =>
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              rules: rules,
            )
            .rules;

    expect(found(null), isNull, reason: 'no rules, no answers');
    final answered =
        found(
          const RuleRequest(shipped: [ShippedRules.nabhasas], longevity: true),
        )!;
    final present = answered['present']! as List<Object?>;
    expect(present, isNotEmpty);
    for (final held in present.cast<Map<String, Object?>>()) {
      expect(held['rule'], isA<String>());
      expect((held['result']! as Map<String, Object?>)['present'], isTrue);
    }
    final longevity = answered['longevity']! as Map<String, Object?>;
    final ayurdaya = longevity['ayurdaya']! as Map<String, Object?>;
    expect((ayurdaya['pindayu']! as Map<String, Object?>)['years'], isA<num>());
    // Jeevasarman's span, worked as Pindayu (Jataka Parijata ch. 5 v. 17).
    expect(
      (ayurdaya['jeevasarman']! as Map<String, Object?>)['method'],
      'jeevasarman',
    );
    expect((ayurdaya['rules']! as Map<String, Object?>)['enmity'], 'natural');
    // The spans read as *Jataka Parijata* reads them, and a choice nothing
    // reads refused.
    final chosen =
        found(
          const RuleRequest(
            longevity: true,
            ayurdaya: AyurdayaRules.parijata,
            threePairs: ThreePairsRules(saturn: SaturnAmongPairs.raises),
          ),
        )!;
    final rules =
        ((chosen['longevity']! as Map<String, Object?>)['ayurdaya']!
                as Map<String, Object?>)['rules']!
            as Map<String, Object?>;
    expect(
      [
        rules['enemy_exempt'],
        rules['enmity'],
        rules['rising'],
        rules['combine'],
      ],
      ['mars', 'compound', 'every', 'largest'],
    );
    expect(
      () => found(const RuleRequest(ayurdaya: AyurdayaRules.parijata)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'rules.ayurdaya',
        ),
      ),
    );
    // The rays, the note's by default, and as the verses read them.
    final rays = longevity['rasmi']! as Map<String, Object?>;
    expect(rays['grahas'], hasLength(7));
    expect(rays['rules'], {'place': 'dwadasamsa'});
    expect(['short', 'medium', 'long'], contains(rays['class']));
    final sun =
        (rays['grahas']! as List<Object?>).first! as Map<String, Object?>;
    expect(
      sun['basic_years']! as num,
      greaterThanOrEqualTo(sun['basic']! as num),
    );
    final verse =
        (found(
                  const RuleRequest(longevity: true, rasmi: RasmiRules.verse),
                )!['longevity']!
                as Map<String, Object?>)['rasmi']!
            as Map<String, Object?>;
    expect(verse['rules'], {'place': 'sign'});
    // v. 33: eight candidates, the lagna weighed by its Bhava bala.
    final choice = longevity['choice']! as Map<String, Object?>;
    expect(choice['candidates'], hasLength(8));
    expect(choice['all_weighed'], isTrue);
    // v. 34: each span in solar years too, 360 days a year to 365.
    final pinda = (choice['candidates']! as List<Object?>)
        .cast<Map<String, Object?>>()
        .firstWhere((c) => c['ayus'] == 'pinda');
    expect(
      (pinda['solar_years']! as num) * 365,
      closeTo((pinda['years']! as num) * 360, 1e-9),
    );
    // v. 27: the dashas' span, a cycle of 120 years less what had run.
    final dasayus = longevity['dasayus']! as Map<String, Object?>;
    expect(dasayus['years']! as num, inInclusiveRange(100, 120));
    // v. 26: the wheel of time's span, the note's figure's portion by
    // default.
    expect((longevity['chakrayus']! as Map<String, Object?>)['rules'], {
      'portion': 'star',
    });
    final byPada =
        (found(
                  const RuleRequest(
                    longevity: true,
                    chakrayus: ChakrayusRules(portion: ChakraPortion.pada),
                  ),
                )!['longevity']!
                as Map<String, Object?>)['chakrayus']!
            as Map<String, Object?>;
    expect(byPada['rules'], {'portion': 'pada'});
    // ch. 10: the ashtakavarga spans, the seven's years summed, and the
    // candidate Mars names carries their sum.
    final spans = longevity['ashtakavarga']! as Map<String, Object?>;
    expect(spans['rules'], {
      'measures': 'parijata',
      'divisor': 'thirty',
      'reductions': 'verse',
      'occupied': 'reduced',
    });
    // v. 48: the lagna's own ashtakavarga holds 49 bindus.
    expect(
      ((spans['lagna']! as Map<String, Object?>)['bindus']! as List<Object?>)
          .fold<num>(0, (sum, b) => sum + (b! as num)),
      49,
    );
    final givers =
        (spans['grahas']! as List<Object?>).cast<Map<String, Object?>>();
    expect(
      givers.fold<num>(0, (sum, giver) => sum + (giver['years']! as num)),
      closeTo(spans['bhinna']! as num, 1e-9),
    );
    final mars = (choice['candidates']! as List<Object?>)
        .cast<Map<String, Object?>>()
        .firstWhere((c) => c['ayus'] == 'bhinnashtakavarga');
    expect(mars['years'], spans['bhinna']);
    final byBalabhadra =
        (found(
                  const RuleRequest(
                    longevity: true,
                    ashtakavarga: AshtakavargaAyusRules(
                      measures: AyusSignMeasures.bphs,
                      divisor: AyusDivisor.sevenOverTwentySeven,
                      reductions: AshtakaReductions.balabhadra,
                      occupied: OccupiedBindus.raw,
                    ),
                  ),
                )!['longevity']!
                as Map<String, Object?>)['ashtakavarga']!
            as Map<String, Object?>;
    expect(byBalabhadra['rules'], {
      'measures': 'bphs',
      'divisor': 'seven-over-twenty-seven',
      'reductions': 'balabhadra',
      'occupied': 'raw',
    });
    expect(
      () => found(const RuleRequest(rasmi: RasmiRules.verse)),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'rules.rasmi'),
      ),
    );

    final first = (present.first! as Map<String, Object?>)['rule']! as String;
    final withMine =
        found(
          RuleRequest(
            shipped: const [ShippedRules.nabhasas],
            rules: [
              <String, Object?>{
                'key': 'MINE',
                'category': 'raja',
                'source': <String, Object?>{'text': 'BPHS'},
                'conditions': [
                  <String, Object?>{'type': 'rule', 'key': first},
                ],
              },
            ],
          ),
        )!;
    expect(
      (withMine['present']! as List<Object?>).cast<Map<String, Object?>>().any(
        (held) => held['rule'] == 'MINE',
      ),
      isTrue,
    );

    expect(
      () => found(
        const RuleRequest(
          rules: [
            <String, Object?>{'key': 'X', 'category': 'raja'},
          ],
        ),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'rules.rules[0]',
        ),
      ),
    );
    ctx.dispose();
  });

  test('plans compose in the same crossing, and render with nothing in '
      'between', () {
    final ctx = context();
    Map<String, Object?>? found(PlanRequest? interpret, {RuleRequest? rules}) =>
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              rules: rules,
              interpret: interpret,
            )
            .plans;

    expect(found(null), isNull, reason: 'no composer, no plans');
    final plans =
        found(
          const PlanRequest(
            placements: true,
            readings: true,
            strength: true,
            houses: true,
            positions: true,
            aspects: true,
            conditions: true,
            karakas: true,
            chalit: true,
            states: true,
            bhavaBala: true,
            vimshopaka: true,
            panchanga: true,
            dashaPhala: true,
            ashtakavarga: true,
          ),
          rules: const RuleRequest(shipped: [ShippedRules.nabhasas]),
        )!;
    final placements = plans['placements']! as List<Object?>;
    expect(placements, isNotEmpty, reason: 'every chart places its grahas');
    expect(plans['readings'], isA<List<Object?>>());
    // The strengths read the Shadbala, which this request never asked for:
    // a composer's own section is computed for it.
    final weighed = plans['strength']! as List<Object?>;
    expect(
      weighed,
      hasLength(7 * 2),
      reason: 'the seven grahas, a score and a sufficiency each',
    );
    // And the houses read the bhavas, which it never asked for either.
    final ruled = plans['houses']! as List<Object?>;
    expect(ruled, hasLength(12 * 2), reason: 'a sign and a lord each');
    // And the positions read the same states the placements do.
    final degrees = plans['positions']! as List<Object?>;
    expect(degrees, hasLength(10), reason: 'the lagna, then the nine grahas');
    // The chalit needs no section: both readings are on the grahas.
    final shifts = plans['chalit']! as List<Object?>;
    expect(shifts.length, lessThanOrEqualTo(9), reason: 'at most one a graha');
    // The drishtis read the aspects section, never asked for either.
    final looks = plans['aspects']! as List<Object?>;
    expect(looks, isNotEmpty, reason: 'every chart holds a drishti');
    // The conditions and the karakas read the same states the placements
    // do, so one section serves four composers.
    final states = plans['conditions']! as List<Object?>;
    expect(states.length, greaterThanOrEqualTo(18), reason: 'nine of each');
    final karakas = plans['karakas']! as List<Object?>;
    expect(karakas, hasLength(15), reason: 'seven of seven, eight of eight');
    // The dasha phala reads its own section, computed for it like the
    // rest: three items a graha always, and a fourth only where the
    // placement tilts the dasha one way or the other.
    final dashas = plans['dashaPhala']! as List<Object?>;
    expect(dashas.length, greaterThanOrEqualTo(9 * 3), reason: 'three each');
    expect(dashas.length, lessThanOrEqualTo(9 * 4), reason: 'and four at most');
    // The states say the half of that section a `Placement` never carried.
    final carried = plans['states']! as List<Object?>;
    expect(
      carried.length,
      greaterThanOrEqualTo(9 * 3),
      reason: 'three a graha at least',
    );
    // Each bhava is weighed and never judged, and the Vimshopaka says all
    // four schemes.
    final houses = plans['bhavaBala']! as List<Object?>;
    expect(houses, hasLength(12), reason: 'one a bhava');
    final scored = plans['vimshopaka']! as List<Object?>;
    expect(scored, hasLength(7 * 4), reason: 'seven grahas, four schemes');
    // The almanac says the five limbs and the Moon's pada.
    final almanac = plans['panchanga']! as List<Object?>;
    expect(almanac.length, greaterThanOrEqualTo(6), reason: 'limbs and pada');
    expect(almanac.length, lessThanOrEqualTo(7), reason: 'and the day');
    // The Ashtakavarga reads its own section *and* the placements: a
    // graha's bindus are the ones of the sign it stands in.
    final bindus = plans['ashtakavarga']! as List<Object?>;
    expect(bindus, hasLength(7 + 12), reason: 'a graha each, then a sign each');

    // Each item said by handing its params straight to the renderer, which
    // is the property the crossing exists for.
    var said = 0;
    for (final item
        in [
          ...placements,
          ...plans['readings']! as List<Object?>,
          ...weighed,
          ...ruled,
          ...degrees,
          ...looks,
          ...states,
          ...karakas,
          ...dashas,
          ...carried,
          ...almanac,
          ...houses,
          ...scored,
        ].cast<Map<String, Object?>>()) {
      final key = item['key']! as String;
      expect(key, startsWith('sdk.'));
      final rendered = ctx.intl.render(
        key,
        item['params']! as Map<String, Object?>,
      );
      expect(rendered.text, isNotEmpty, reason: '$key said nothing');
      expect(rendered.isFallback, 0, reason: '$key fell back');
      expect(rendered.warningCount, 0, reason: '$key warned');
      said += 1;
    }
    expect(said, greaterThan(20), reason: 'only $said items said');

    // A reading says what the rules answered, so it needs rules beside it.
    expect(
      () => found(const PlanRequest(readings: true)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'interpret.readings',
        ),
      ),
    );
    // The Sade Sati plan says what a window found, so it needs one beside
    // it; with one and no pack loaded it is present and empty, which is an
    // answer.
    expect(
      () => found(const PlanRequest(sadeSati: true)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'interpret.sadeSati',
        ),
      ),
    );
    final periods = ctx.chart.found(
      instant: 2451545.0,
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
      sadeSati: const SadeSatiRequest(from: 2451545, to: 2462502.5),
      interpret: const PlanRequest(sadeSati: true),
    );
    expect(periods.plans!['sadeSati'], isEmpty, reason: 'no pack, no words');
    expect(
      periods.sadeSati!.sadeSati,
      isNotEmpty,
      reason: 'the report is on the chart beside the plan',
    );
    // The transits are said one plan an instant, from the readings beside
    // them, and need no corpus; asked without a gochar request, refused.
    expect(
      () => found(const PlanRequest(gochar: true)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'interpret.gochar',
        ),
      ),
    );
    final transits = ctx.chart.found(
      instant: 2451545.0,
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
      gochar: const GocharRequest(
        instants: [2461000.5, 2461365.5],
        ashtakavarga: true,
      ),
      interpret: const PlanRequest(gochar: true),
    );
    final transitPlans = transits.plans!['gochar']! as List<Object?>;
    expect(transitPlans, hasLength(2), reason: 'one plan an instant');
    for (final plan in transitPlans.cast<List<Object?>>()) {
      final items = plan.cast<Map<String, Object?>>();
      expect(items.first['key'], 'sdk.reason.gochar.from');
      expect(
        items.where((item) => item['key'] == 'sdk.reason.gochar.transit'),
        hasLength(9),
      );
    }
    ctx.dispose();
  });

  test('a chart carries the sphuta drishti by degree and the rashi drishti '
      'by sign', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      aspects: true,
    );
    final gazes = chart.sphutaDrishti;
    expect(gazes, hasLength(7 * 8), reason: 'the seven at the eight others');
    expect(
      gazes.every(
        (g) => g.from != Graha.rahu && g.from != Graha.ketu && g.from != g.to,
      ),
      isTrue,
    );
    expect(gazes.every((g) => g.virupas >= 0 && g.virupas < 90), isTrue);
    expect(chart.rashiDrishti, hasLength(9));
    expect(chart.rashiDrishti.every((r) => r.signs.length == 3), isTrue);
    final bare = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(bare.sphutaDrishti, isEmpty, reason: 'nothing unless asked');
    expect(bare.rashiDrishti, isEmpty);
    ctx.dispose();
  });

  test('a layout of your own is registered, drawn by its key, and refused by '
      'its field', () {
    final base = context();
    final row = base.chart.layout(ChartLayout.southIndian);
    expect(row.key, 'SOUTH_INDIAN');
    expect(row.shape, isA<GridShape>());
    expect(row.shape.direction, LayoutDirection.clockwise);
    expect(
      (row.toJson()['shape']! as Map<String, Object?>)['direction'],
      'CLOCKWISE',
      reason: 'the words a row answers are keys',
    );
    expect(
      row.toJson(),
      LayoutRow.fromJson(row.toJson()).toJson(),
      reason: 'a row reads back as it was written',
    );
    expect(
      () => base.chart.layout(ChartLayout.registered('ACME_KERALA')),
      throwsA(isA<TeistroException>().having((e) => e.field, 'field', 'key')),
    );
    base.dispose();

    final kerala = row.copyWith(key: 'ACME_KERALA');
    final own = ChartLayout.registered('ACME_KERALA');
    final ctx = teistro.context(
      profile: 'nepali-drik',
      testProvider: true,
      layouts: [kerala],
    );
    expect(ctx.chart.layout(own).toJson(), kerala.toJson());
    expect(ctx.keys.name(ctx.keys.id(own.fullKey)), own.fullKey);

    final [south, drawn] =
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              drawings: [(ChartLayout.southIndian, Varga.d1), (own, Varga.d1)],
            )
            .drawings;
    expect(drawn.layout, own);
    expect(south.layout, ChartLayout.southIndian);
    expect(
      [for (final cell in drawn.cells) cell.sign],
      [for (final cell in south.cells) cell.sign],
    );
    expect(
      () => ctx.chart.found(
        instant: 2451545.0,
        place: Observer(
          latitudeDeg: Latitude(0),
          longitudeDeg: Longitude(0),
          altitudeM: Altitude(0),
        ),
        utcOffsetSeconds: 0,
        drawings: [(ChartLayout.registered('ACME_ODIA'), Varga.d1)],
      ),
      throwsArgumentError,
    );
    ctx.dispose();

    Matcher field(String name) =>
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', name));
    expect(
      () => teistro.context(testProvider: true, layouts: [kerala, row]),
      field('options.layouts_json[1].key'),
    );
    expect(
      () => teistro.context(
        testProvider: true,
        layouts: [kerala.copyWith(sources: const [])],
      ),
      field('options.layouts_json[0].sources'),
    );
  });

  /// A chart's Ashtakavarga crosses whole: each graha's bindus holding the
  /// classical totals, the sum, and each graha's reductions under the default
  /// reading; null unless asked.
  test('a chart carries its Ashtakavarga and each graha\'s reductions', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      ashtakavarga: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .ashtakavarga,
      isNull,
    );
    final av = chart.ashtakavarga!;
    expect(
      (av.shodhana, av.ekadhipatya),
      (Shodhana.eachGraha, Ekadhipatya.bphs),
    );
    expect(
      [for (final g in av.grahas) g.bindus.reduce((a, b) => a + b)],
      [48, 49, 39, 54, 56, 52, 39],
    );
    expect(av.sarva.reduce((a, b) => a + b), 337);
    expect(av.reduced, [
      for (var sign = 0; sign < 12; sign += 1)
        av.grahas.fold<int>(0, (sum, g) => sum + g.reduced![sign]),
    ]);
    expect(
      av.grahas.every((g) => g.yogaPinda == g.rashiPinda + g.grahaPinda),
      isTrue,
    );
    ctx.dispose();
  });

  /// A chart's Bhava bala crosses whole: every bhava's components under the
  /// default reading, the verses', whose totals are their parts'; null unless
  /// asked.
  test('a chart carries its Bhava bala, each bhava\'s strength', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      bhavaBala: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .bhavaBala,
      isNull,
    );
    final bhavas = chart.bhavaBala!.bhavas;
    expect([for (final b in bhavas) b.bhava], List.generate(12, (i) => i + 1));
    for (final b in bhavas) {
      expect(
        b.adhipati + b.dig + b.drishti + b.special,
        closeTo(b.virupas, 1e-9),
      );
      expect(b.dig, inInclusiveRange(0.0, 60.0));
    }
    ctx.dispose();
  });

  /// A chart's Shadbala crosses whole: every graha's six strengths under the
  /// default reading, the chapter's, whose natural strengths are 28 sevenths
  /// of a rupa and whose totals are their components'; null unless asked.
  test('a chart carries its Shadbala, each graha\'s six strengths', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      shadbala: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .shadbala,
      isNull,
    );
    final grahas = chart.shadbala!.grahas;
    expect(grahas, hasLength(7));
    expect(
      grahas.fold<double>(0, (sum, g) => sum + g.naisargika),
      closeTo(240, 1e-9),
    );
    for (final g in grahas) {
      final six =
          g.sthana.total +
          g.dig +
          g.kaala.total +
          g.cheshta +
          g.naisargika +
          g.drik;
      expect(six, closeTo(g.virupas, 1e-9), reason: '${g.graha}');
      expect(g.strong, g.rupas >= g.requiredRupas);
    }
    ctx.dispose();
  });

  /// A chart's dasha phala crosses whole: the nine grahas' Subhankas within
  /// each varga's share and complementary in total; null unless asked, and
  /// the Shadbala's rays beside the phalas.
  test('a chart carries its dasha phala, and the Shadbala its rays', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      dashaPhala: true,
      shadbala: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .dashaPhala,
      isNull,
    );
    final grahas = chart.dashaPhala!.grahas;
    expect(grahas, hasLength(9));
    expect(grahas.last.graha, Graha.ketu);
    for (final g in grahas) {
      expect(g.subhankas, hasLength(7));
      for (var k = 0; k < 7; k++) {
        expect(g.subhankas[k], inInclusiveRange(0, k == 0 ? 60 : 30));
      }
      expect(g.subhanka + g.asubhanka, closeTo(240, 1e-9));
    }
    for (final s in chart.shadbala!.grahas) {
      expect(s.subhaRashmi, inInclusiveRange(1, 7));
      expect(s.subhaRashmi + s.ashubhaRashmi, closeTo(8, 1e-9));
    }
    ctx.dispose();
  });

  /// A chart's Jaimini significators cross whole: the karakamsha with nine
  /// houses in each chart, the Atmakaraka's own navamsha the first from it,
  /// and the Brahma graha found or its absence named -- never both, never
  /// neither; null unless asked.
  test('a chart carries its hit list, the sky once for the batch', () {
    // The built-in ephemeris, whose Mercury turns retrograde in the window:
    // the test provider's planets never stand still.
    final ctx = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    expect(
      ctx.chart
          .found(instant: 2451545, place: place, utcOffsetSeconds: 20700)
          .hits,
      isEmpty,
    );
    const asked = HitRequest(
      from: 2460676.5,
      to: 2460866.5,
      grahas: [Graha.sun, Graha.mercury, Graha.saturn],
      aspects: [0, 90, 180],
      orbDeg: 2,
    );
    final batch = ctx.chart.foundMany(
      instants: [2447995.4895833335, 2451545],
      place: place,
      utcOffsetSeconds: 20700,
      hits: asked,
    );
    String line(Hit h) => switch (h.event) {
      SignIngress(:final into, :final motion) =>
        '${h.instant} ${h.graha.key} ${into.key} ${motion.key}',
      NakshatraIngress(:final into, :final motion) =>
        '${h.instant} ${h.graha.key} ${into.key} ${motion.key}',
      Station(:final turns) => '${h.instant} ${h.graha.key} ${turns.key}',
      AspectHit(:final to, :final angle, :final phase) =>
        '${h.instant} ${h.graha.key} ${to.point} $angle ${phase.key}',
      final other => '${h.instant} ${h.graha.key} ${other.kind.key}',
    };
    final first = batch.at(0).hits;
    final second = batch.at(1).hits;
    List<String> sky(List<Hit> hits) => [
      for (final h in hits)
        if (h.event is! AspectHit) line(h),
    ];
    expect(sky(first), sky(second));
    expect(first.map(line), isNot(second.map(line)));
    expect(
      {for (final h in first) h.event.kind},
      {
        HitKind.signIngress,
        HitKind.nakshatraIngress,
        HitKind.station,
        HitKind.aspect,
      },
    );
    for (final (k, h) in first.indexed) {
      if (k > 0) expect(first[k - 1].instant, lessThanOrEqualTo(h.instant));
      expect([Graha.sun, Graha.mercury, Graha.saturn], contains(h.graha));
      if (h.event case AspectHit(:final angle)) {
        expect([0, 90, 180], contains(angle));
      }
    }
    // What an aspect names as `to` is what a request takes back.
    final aspect = first.map((h) => h.event).whereType<AspectHit>().first;
    final again =
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              hits: HitRequest(
                from: asked.from,
                to: asked.to,
                grahas: asked.grahas,
                kinds: const [HitKind.aspect],
                points: [aspect.to],
                aspects: asked.aspects,
                orbDeg: asked.orbDeg,
              ),
            )
            .hits;
    expect(again, isNotEmpty);
    for (final h in again) {
      expect((h.event as AspectHit).to, aspect.to);
    }
    expect(
      () => ctx.chart.found(
        instant: 2451545,
        place: place,
        utcOffsetSeconds: 20700,
        hits: const HitRequest(from: 2460676.5, to: 2460600.5),
      ),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'hits.to'),
      ),
    );
    ctx.dispose();
  });

  /// The five kinds past the default four cross with their own fields: a
  /// varga ingress's chart and sign, a lunar phase, a phase return, and an
  /// eclipse's kind and natal point. The birth is an hour after the solar
  /// eclipse of 29 March 2025, so 2025 holds an eclipse on its luminaries.
  test('a hit list carries varga ingresses, lunar phases and eclipses', () {
    final ctx = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const kinds = [
      HitKind.vargaIngress,
      HitKind.lunarPhase,
      HitKind.phaseReturn,
      HitKind.solarEclipse,
      HitKind.lunarEclipse,
    ];
    HitRequest asked({
      List<Varga> vargas = const [Varga.d9, Varga.d30],
      double eclipseOrbDeg = 3,
    }) => HitRequest(
      from: 2460676.5,
      to: 2461041.5,
      grahas: const [Graha.sun, Graha.mars],
      kinds: kinds,
      vargas: vargas,
      eclipseOrbDeg: eclipseOrbDeg,
    );
    final hits =
        ctx.chart
            .found(
              instant: 2460764,
              place: place,
              utcOffsetSeconds: 0,
              hits: asked(),
            )
            .hits;
    expect({for (final h in hits) h.event.kind}, kinds.toSet());
    for (final h in hits) {
      switch (h.event) {
        case VargaIngress(:final varga):
          expect([Varga.d9, Varga.d30], contains(varga));
        case SolarEclipseHit(:final to):
          expect(h.graha, Graha.sun);
          expect(['GRAHA', 'LAGNA', 'MIDHEAVEN'], contains(to.point));
        case LunarEclipseHit(:final to):
          expect(h.graha, Graha.moon);
          expect(['GRAHA', 'LAGNA', 'MIDHEAVEN'], contains(to.point));
        case LunarPhase() || PhaseReturn():
          expect(h.graha, Graha.moon);
        default:
          fail('a kind not asked for: ${h.event.kind}');
      }
    }
    expect(
      hits.where((h) => h.event is LunarPhase).length,
      inInclusiveRange(48, 50),
    );
    for (final (bad, field) in [
      (asked(vargas: const [Varga.d1]), 'hits.vargas'),
      (asked(eclipseOrbDeg: 20), 'hits.eclipseOrbDeg'),
    ]) {
      expect(
        () => ctx.chart.found(
          instant: 2460764,
          place: place,
          utcOffsetSeconds: 0,
          hits: bad,
        ),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a lunar return is the Moon back on her own natal place', () {
    final ctx = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final paris = Observer(
      latitudeDeg: Latitude(48.8534),
      longitudeDeg: Longitude(2.3488),
      altitudeM: Altitude(0),
    );
    final asked = HitRequest.returns(from: 2451546, to: 2451911.25);
    expect(asked.grahas, [Graha.moon]);
    expect(asked.kinds, [HitKind.aspect]);
    expect(asked.points, [const NatalGraha(Graha.moon)]);
    expect(asked.aspects, [0]);
    Chart at(double instant, {HitRequest? hits}) => ctx.chart.found(
      instant: instant,
      place: paris,
      utcOffsetSeconds: 0,
      hits: hits,
    );
    final lunar = at(2451545, hits: asked).hits;
    expect(lunar, hasLength(13));
    for (final (k, hit) in lunar.indexed) {
      expect(hit.graha, Graha.moon);
      final event = hit.event as AspectHit;
      expect(event.to, const NatalGraha(Graha.moon));
      expect(event.angle, 0);
      if (k > 0) {
        expect(hit.instant - lunar[k - 1].instant, inInclusiveRange(27, 27.7));
      }
    }
    double moon(Chart chart) =>
        chart.grahas.firstWhere((g) => g.graha == Graha.moon).longitudeDeg;
    expect(moon(at(lunar.first.instant)), closeTo(moon(at(2451545)), 1 / 3600));
    ctx.dispose();
  });

  test('a chart carries its Sade Sati, each period whole', () {
    final ctx = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const births = [2447995.4895833335, 2451545.2];
    SadeSatiReport? found(double instant, SadeSatiRequest? asked) =>
        ctx.chart
            .found(
              instant: instant,
              place: place,
              utcOffsetSeconds: 20700,
              sadeSati: asked,
            )
            .sadeSati;
    expect(found(births[0], null), isNull);
    for (final reckoning in Reckoning.values) {
      final asked = SadeSatiRequest(
        from: 2460676.5,
        to: 2464329,
        reckoning: reckoning,
        spells: const [4, 7, 8],
      );
      final batch = ctx.chart.foundMany(
        instants: births,
        place: place,
        utcOffsetSeconds: 20700,
        sadeSati: asked,
      );
      for (final (k, instant) in births.indexed) {
        final report = batch.at(k).sadeSati!;
        expect(report, found(instant, asked), reason: 'a batch is each alone');
        expect(report.reckoning, reckoning);
        expect(report.reference.from, GocharFrom.moon);
        expect(report.sadeSati.length + report.spells.length, greaterThan(0));
        for (final one in report.sadeSati) {
          expect([for (final spell in one.phases) spell.house], [12, 1, 2]);
          final visits = [for (final spell in one.phases) ...spell.visits]
            ..sort((a, b) => a.from!.compareTo(b.from!));
          for (var i = 1; i < visits.length; i += 1) {
            expect(visits[i - 1].to!, lessThanOrEqualTo(visits[i].from!));
          }
          // Asked at one instant inside its peak, from what the report
          // named: the same Sade Sati, whole.
          final peak = one.phases[1].visits.first;
          final now =
              found(
                instant,
                SadeSatiRequest(
                  from: (peak.from! + peak.to!) / 2,
                  countedFrom: report.reference.from,
                  reckoning: report.reckoning,
                ),
              )!;
          expect(now.sadeSati, [one], reason: '$reckoning');
        }
        for (final spell in report.spells) {
          expect([4, 7, 8], contains(spell.house));
        }
      }
    }
    for (final (bad, field) in [
      (const SadeSatiRequest(from: 2460676.5, to: 2460600.5), 'sadeSati.to'),
      (const SadeSatiRequest(from: 2460676.5, spells: [2]), 'sadeSati.spells'),
    ]) {
      expect(
        () => found(births[0], bad),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('an almanac carries the muhurta search it was asked for', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final from = gregorian(2026, 11, 25);
    Almanac days({
      CalendarDate? since,
      CalendarDate? to,
      MuhurtaRequest? muhurta,
    }) => ctx.almanac.of(
      from: since ?? from,
      to: to ?? gregorian(2026, 12, 3),
      place: place,
      utcOffsetSeconds: 20700,
      muhurta: muhurta,
    );
    final plain = days();
    expect(plain.muhurta, isNull);

    final almanac = days(
      muhurta: const MuhurtaRequest(
        rules: MuhurtaActivity.ramanMarriage,
        native: MuhurtaNative(
          star: Nakshatra.rohini,
          moonSign: Rashi.taurus,
          lagna: Rashi.leo,
        ),
        daysWithWindows: 9,
        most: 1000,
      ),
    );
    final answer = almanac.muhurta!;
    expect(answer.windows, isNotEmpty);
    expect(answer.ranking, MuhurtaRanking.texts);
    // The days are the ones asked without a search.
    for (var k = 0; k < plain.length; k += 1) {
      expect(
        almanac.at(k).provenance.contentHash,
        plain.at(k).provenance.contentHash,
      );
    }
    final kinds = [
      for (final window in answer.windows)
        for (final clause in window.clauses) clause.kind,
    ];
    expect(
      kinds.whereType<TarabalaClause>(),
      isNotEmpty,
      reason: 'the native is read',
    );
    expect(
      kinds.whereType<NakshatraClause>().every(
        (clause) => clause.nakshatra != Nakshatra.unknown,
      ),
      isTrue,
    );
    final knobs = {
      for (final convention in answer.provenance.appliedConventions)
        convention.knob,
    };
    expect(knobs, containsAll(['muhurta.asta', 'muhurta.zodiacAt']));

    // A clause answered is a bar a request may name, as it was read.
    final amrit = kinds.whereType<ChoghadiyaClause>().firstWhere(
      (clause) => clause.choghadiya == Choghadiya.amrit,
    );
    const graded = {
      'best': <Object?>[],
      'middling': <Object?>[],
      'rejected': <Object?>[],
      'otherwise': 'MIDDLING',
    };
    final rules = <String, Object?>{
      'day': {
        'tithis': graded,
        'nakshatras': graded,
        'yogas': graded,
        'karanas': graded,
        'varas': graded,
        'chandrabala': {'avoid': <Object?>[]},
      },
      'months': {'reckoning': 'ANY'},
      'lagnas': graded,
      'padas': <Object?>[],
      'heeds': <Object?>[],
      'bars': [amrit],
      'unjudged': <Object?>[],
      'baseline': null,
    };
    final barred =
        days(
          to: from,
          muhurta: MuhurtaRequest(
            rules: rules,
            daysWithWindows: 1,
            most: 100000,
          ),
        ).muhurta!;
    final struck = barred.windows.where((window) => window.barredBy.isNotEmpty);
    expect(
      struck,
      isNotEmpty,
      reason: 'the bar read back strikes the windows it names',
    );
    expect(struck.every((window) => window.barredBy.single == amrit), isTrue);

    // A closed day names its blackouts as members, and a request takes one
    // in either spelling: 1 June 2026 is in Jyeshtha's adhika month.
    final june = gregorian(2026, 6, 1);
    List<BlackoutKind> closedBy(Object rules) =>
        days(
          since: june,
          to: june,
          muhurta: MuhurtaRequest(rules: rules),
        ).muhurta!.closed.single.by;
    expect(
      closedBy(MuhurtaActivity.ramanMarriage),
      contains(BlackoutKind.adhikaMasa),
    );
    for (final heed in <Object>[BlackoutKind.adhikaMasa, 'ADHIKA_MASA']) {
      expect(
        closedBy({
          ...rules,
          'bars': <Object?>[],
          'heeds': [heed],
        }),
        [BlackoutKind.adhikaMasa],
      );
    }

    for (final (asked, field) in [
      (
        const MuhurtaRequest(rules: MuhurtaActivity.ramanMarriage, most: 0),
        'muhurta.most',
      ),
      (const MuhurtaRequest(rules: 'RAMAN_MARRIAGE'), 'muhurta.rules'),
      (
        const MuhurtaRequest(rules: <String, Object?>{'day': 1}),
        'muhurta.rules.day',
      ),
    ]) {
      expect(
        () => days(muhurta: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
        reason: field,
      );
    }

    // A rite beyond marriage, by name: its unwanted placements come back
    // typed, and the 8th the thread ceremony says must be empty bars as
    // itself.
    final thread =
        days(
          muhurta: const MuhurtaRequest(
            rules: MuhurtaActivity.ramanUpanayana,
            daysWithWindows: 9,
            most: 1000,
          ),
        ).muhurta!;
    final placed = [
      for (final window in thread.windows)
        for (final clause in window.clauses)
          if (clause.kind case final UnwantedPlacementClause placement)
            placement,
    ];
    expect(placed, isNotEmpty);
    expect(
      placed.every((p) => p.house >= 1 && p.house <= 12 && p.by.isNotEmpty),
      isTrue,
    );
    final barring = [
      for (final window in thread.windows)
        for (final bar in window.barredBy)
          if (bar case final UnwantedPlacementClause placement) placement,
    ];
    expect(barring.every((b) => b.house == 8), isTrue);
    expect(
      () => days(
        muhurta: MuhurtaRequest(
          rules: {
            ...rules,
            'unwanted': [
              {
                'grahas': ['MARS'],
                'houses': [13],
              },
            ],
          },
        ),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'muhurta.rules.unwanted[0].houses',
        ),
      ),
    );
    ctx.dispose();
  });

  test('an almanac carries the lunar years it was asked for', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    // Across Chaitra Shukla Pratipada of VS 2083, 19 March 2026.
    Almanac days({bool years = false}) => ctx.almanac.of(
      from: gregorian(2026, 3, 10),
      to: gregorian(2026, 4, 10),
      place: place,
      utcOffsetSeconds: 20700,
      years: years,
    );
    final plain = days();
    expect(plain.years, isNull);

    final almanac = days(years: true);
    final answer = almanac.years!;
    final years = answer.value;
    expect(
      [for (final year in years) year.samvatsara],
      [Samvatsara.siddharthi, Samvatsara.raudra],
    );
    expect([for (final year in years) year.vikrama], [2082, 2083]);
    expect(
      [for (final year in years) year.count],
      ['BARHASPATYA', 'BARHASPATYA'],
    );
    expect(years.first.ended, years.last.began);
    expect(years.last.opened, lessThan(years.last.began));
    expect(years.last.jovian.first.member, isNot(Samvatsara.unknown));
    expect(() => years.add(years.first), throwsUnsupportedError);
    expect(answer.provenance.contentHash, isNotEmpty);
    for (var k = 0; k < plain.length; k++) {
      expect(
        almanac.at(k).provenance.contentHash,
        plain.at(k).provenance.contentHash,
      );
    }
    ctx.dispose();
  });

  test('a day carries each star\'s varjyam unless the settings name none', () {
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<(List<Span<Nakshatra>>, List<Span<Nakshatra>>)> week([
      Map<String, Object?>? settings,
    ]) {
      final ctx = teistro.context(
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
        settings: settings,
      );
      final days = ctx.almanac.of(
        from: gregorian(2024, 6, 15),
        to: gregorian(2024, 6, 21),
        place: place,
        utcOffsetSeconds: 20700,
      );
      final read = [
        for (var i = 0; i < days.length; i += 1)
          (days.at(i).varjyam, days.at(i).nakshatra),
      ];
      ctx.dispose();
      return read;
    }

    var seen = 0;
    for (final (varjyam, stars) in week()) {
      for (final window in varjyam) {
        final star = stars.firstWhere((span) => span.member == window.member);
        double length(Span<Nakshatra> span) => span.whole.to - span.whole.from;
        expect(length(window) * 15, closeTo(length(star), 1e-6));
        expect(window.whole.from, greaterThanOrEqualTo(star.whole.from));
        seen += 1;
      }
    }
    expect(seen, inInclusiveRange(5, 9));
    expect([
      for (final (varjyam, _) in week({
        'panchanga': {'varjyam': 'NONE'},
      }))
        varjyam.length,
    ], List.filled(7, 0));
  });

  test('a day carries each star\'s amrita kalam unless the settings name '
      'none', () {
    // The varjyam's arithmetic on the table the published panchangas
    // print (C677).
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<(List<Span<Nakshatra>>, List<Span<Nakshatra>>)> week([
      Map<String, Object?>? settings,
    ]) {
      final ctx = teistro.context(
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
        settings: settings,
      );
      final days = ctx.almanac.of(
        from: gregorian(2024, 6, 15),
        to: gregorian(2024, 6, 21),
        place: place,
        utcOffsetSeconds: 20700,
      );
      final read = [
        for (var i = 0; i < days.length; i += 1)
          (days.at(i).amritKalam, days.at(i).nakshatra),
      ];
      ctx.dispose();
      return read;
    }

    var seen = 0;
    for (final (amrita, stars) in week()) {
      for (final window in amrita) {
        final star = stars.firstWhere((span) => span.member == window.member);
        double length(Span<Nakshatra> span) => span.whole.to - span.whole.from;
        expect(length(window) * 15, closeTo(length(star), 1e-6));
        seen += 1;
      }
    }
    expect(seen, inInclusiveRange(5, 9));
    expect([
      for (final (amrita, _) in week({
        'panchanga': {'amrit_kalam': 'NONE'},
      }))
        amrita.length,
    ], List.filled(7, 0));
  });

  test('a day asked for its lagnas carries the signs rising from sunrise '
      'to the next, and one not asked none', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final day = ctx.almanac.day(
      date: gregorian(2024, 6, 15),
      place: place,
      utcOffsetSeconds: 20700,
      lagnas: true,
    );
    final plain = ctx.almanac.day(
      date: gregorian(2024, 6, 15),
      place: place,
      utcOffsetSeconds: 20700,
    );
    ctx.dispose();
    final table = day.lagnas;
    expect(table.length, inInclusiveRange(13, 14));
    expect(table.first.inside.from, day.day.sunrise);
    expect(table.last.inside.to, day.day.nextSunrise);
    for (var j = 1; j < table.length; j += 1) {
      expect(table[j - 1].whole.to, table[j].whole.from);
      expect(table[j].member.id, (table[j - 1].member.id + 1) % 12);
    }
    expect(plain.lagnas, isEmpty);
  });

  test('a day asked for its homa ahuti gives a graha to each stretch of '
      'its stars, and one not asked none', () {
    // Muhurta Chintamani, Nakshatra-prakarana v. 35 (crux C751): the
    // Moon's star counted from the Sun's, in threes, from the Sun to Ketu.
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final day = ctx.almanac.day(
      date: gregorian(2024, 6, 15),
      place: place,
      utcOffsetSeconds: 20700,
      homaAhuti: true,
    );
    final plain = ctx.almanac.day(
      date: gregorian(2024, 6, 15),
      place: place,
      utcOffsetSeconds: 20700,
    );
    ctx.dispose();
    final shares = day.homaAhuti;
    expect(shares.length, greaterThanOrEqualTo(day.nakshatra.length));
    expect(shares.first.inside.from, day.window.from);
    expect(shares.last.inside.to, day.window.to);
    for (var j = 1; j < shares.length; j += 1) {
      expect(shares[j - 1].whole.to, shares[j].whole.from);
    }
    expect(plain.homaAhuti, isEmpty);
  });

  test('fire follows each tithi with the vara, and Rahu walks the daylight '
      'from the east', () {
    // Muhurta Chintamani, Nakshatra-prakarana v. 36: the tithi from Shukla
    // Pratipada as 1, one more, the vara from Sunday as 1, by four (C748);
    // Jyotirnibandha p. 337's eight directions from the east (C749).
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final days = ctx.almanac.of(
      from: gregorian(2024, 6, 15),
      to: gregorian(2024, 6, 21),
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
    );
    final week = [for (var i = 0; i < days.length; i += 1) days.at(i)];
    ctx.dispose();
    const walk = [
      Direction.east,
      Direction.northwest,
      Direction.south,
      Direction.northeast,
      Direction.west,
      Direction.southeast,
      Direction.north,
      Direction.southwest,
    ];
    for (final day in week) {
      expect(day.agniVasa.length, day.tithi.length);
      for (var k = 0; k < day.tithi.length; k += 1) {
        expect(day.agniVasa[k].whole.from, day.tithi[k].whole.from);
        final remainder =
            (day.tithi[k].member.id + 1 + 1 + day.day.vara.id + 1) % 4;
        expect(day.agniVasa[k].member, switch (remainder) {
          1 => AgniVasa.svarga,
          2 => AgniVasa.patala,
          _ => AgniVasa.prithvi,
        });
      }
      expect([for (final span in day.rahuVasa) span.member], walk);
      expect(day.rahuVasa.first.whole.from, day.day.sunrise);
      expect(day.rahuVasa.last.whole.to, closeTo(day.day.sunset, 1e-9));
    }
    // Muhurta Chintamani, Shubhashubha-prakarana vv. 8-9 (C760): the
    // dagdha, visha and hutashana tithis by number in either paksha, Sunday
    // first, and the Yamaghanta nakshatra; each yoga runs as its limb does.
    const rows = [
      [12, 4, 12],
      [11, 6, 6],
      [5, 7, 7],
      [3, 2, 8],
      [6, 8, 9],
      [8, 9, 10],
      [9, 7, 11],
    ];
    const yogas = [
      AshubhaYoga.dagdha,
      AshubhaYoga.visha,
      AshubhaYoga.hutashana,
    ];
    const stars = [
      Nakshatra.magha,
      Nakshatra.vishakha,
      Nakshatra.ardra,
      Nakshatra.mula,
      Nakshatra.krittika,
      Nakshatra.rohini,
      Nakshatra.hasta,
    ];
    var shunned = 0;
    for (final day in week) {
      final vara = day.day.vara.id;
      final expected = [
        for (final tithi in day.tithi)
          for (var k = 0; k < 3; k += 1)
            if (rows[vara][k] == tithi.member.id % 15 + 1)
              (yogas[k], tithi.whole.from),
        for (final star in day.nakshatra)
          if (star.member == stars[vara])
            (AshubhaYoga.yamaghanta, star.whole.from),
      ];
      // Dart's sort is not stable, so a tithi's two yogas keep the verse's
      // order by their place in the list.
      final order = [for (var k = 0; k < expected.length; k += 1) k]
        ..sort((a, b) {
          final by = expected[a].$2.compareTo(expected[b].$2);
          return by != 0 ? by : a.compareTo(b);
        });
      expect(
        [for (final span in day.ashubhaYogas) (span.member, span.whole.from)],
        [for (final k in order) expected[k]],
      );
      shunned += expected.length;
    }
    expect(shunned, greaterThan(0));
    // Muhurta Chintamani, Shubhashubha-prakarana v. 45 (C820): Bhadra dwells
    // by the Moon's sign, Aries first, inside a Vishti; and the Surya
    // Panchanga's Shiva vasa, (2t + 5) mod 7 over the tithi t from 1 (C750).
    const loka = [
      BhadraVasa.svarga, BhadraVasa.svarga, BhadraVasa.svarga, //
      BhadraVasa.martya, BhadraVasa.martya, BhadraVasa.patala, //
      BhadraVasa.patala, BhadraVasa.svarga, BhadraVasa.patala, //
      BhadraVasa.patala, BhadraVasa.martya, BhadraVasa.martya,
    ];
    const shiva = [
      ShivaVasa.shmashana,
      ShivaVasa.kailasa,
      ShivaVasa.gauri,
      ShivaVasa.vrishabha,
      ShivaVasa.sabha,
      ShivaVasa.bhojana,
      ShivaVasa.krida,
    ];
    var dwelt = 0;
    for (final day in week) {
      for (final span in day.bhadraVasa) {
        expect(
          day.karana.where(
            (k) =>
                k.member == Karana.vishti &&
                k.whole.from <= span.whole.from &&
                span.whole.to <= k.whole.to,
          ),
          hasLength(1),
        );
        final sign = day.moonSigns.singleWhere(
          (s) =>
              s.whole.from <= span.whole.from && span.whole.from < s.whole.to,
        );
        expect(span.member, loka[sign.member.id]);
        dwelt += 1;
      }
      expect(
        [for (final span in day.shivaVasa) (span.member, span.whole.from)],
        [
          for (final tithi in day.tithi)
            (shiva[(2 * (tithi.member.id + 1) + 5) % 7], tithi.whole.from),
        ],
      );
      expect(
        day.namedPanchaka.length,
        lessThanOrEqualTo(day.panchaka.isEmpty ? 0 : 1),
      );
    }
    expect(dwelt, greaterThan(0));
    // The Sun entered Mithuna on 15 June 2024: one day of the week holds its
    // punya kala, inside that day's daylight (Dharmasindhu p. 2, C752).
    final held = [
      for (final day in week)
        for (final kala in day.punyaKala) (day, kala),
    ];
    expect(held, hasLength(1));
    final (day, kala) = held.single;
    expect(kala.member, Rashi.gemini);
    expect(kala.inside.from, greaterThanOrEqualTo(day.day.sunrise));
    expect(kala.inside.to, lessThanOrEqualTo(day.day.sunset));
    expect(kala.inside.from, lessThan(kala.inside.to));
  });

  test('a day marks its durmuhurtas and carries the shool, the chandra vasa '
      'and the chandrashtama', () {
    // 15 to 21 June 2024 at Kathmandu, Saturday first (crux C398): Muhurta
    // Chintamani's Tuesday names Agni, who presides over no muhurta of the
    // daylight, so its second is the night's seventh; Raman's is the 11th.
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<AlmanacDay> week([Map<String, Object?>? settings]) {
      final ctx = teistro.context(
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
        settings: settings,
      );
      final days = ctx.almanac.of(
        from: gregorian(2024, 6, 15),
        to: gregorian(2024, 6, 21),
        place: place,
        utcOffsetSeconds: 20700,
      );
      final read = [for (var i = 0; i < days.length; i += 1) days.at(i)];
      // A batch owns its decoded columns, so its days outlive the context.
      ctx.dispose();
      return read;
    }

    List<String> censured(AlmanacDay day) {
      final muhurtas = day.muhurtas;
      final daylight = muhurtas.where((m) => m.daylight).length;
      return [
        for (var k = 0; k < muhurtas.length; k += 1)
          if (muhurtas[k].durmuhurta)
            muhurtas[k].daylight ? 'D${k + 1}' : 'N${k + 1 - daylight}',
      ];
    }

    final texts = week();
    expect(texts.map(censured).toList(), [
      ['D1', 'D2'],
      ['D14'],
      ['D9', 'D12'],
      ['D4', 'N7'],
      ['D8'],
      ['D6', 'D12'],
      ['D4', 'D9'],
    ]);
    expect(
      censured(
        week({
          'panchanga': {'durmuhurta': 'RAMAN'},
        })[3],
      ),
      ['D4', 'D11'],
    );

    var shools = 0;
    for (final day in texts) {
      final signs = day.moonSigns;
      expect(day.chandraVasa.length, signs.length);
      expect(day.chandrashtama.length, signs.length);
      for (var k = 0; k < signs.length; k += 1) {
        expect(day.chandraVasa[k].whole.from, signs[k].whole.from);
        expect(day.chandrashtama[k].whole.to, signs[k].whole.to);
        // The Moon stands in the eighth sign from the natal one.
        expect((day.chandrashtama[k].member.id + 7) % 12, signs[k].member.id);
      }
      for (final shool in day.nakshatraShool) {
        expect(
          day.nakshatra.any(
            (star) =>
                star.whole.from == shool.whole.from &&
                star.whole.to == shool.whole.to,
          ),
          isTrue,
        );
        shools += 1;
      }
    }
    expect(shools, inInclusiveRange(1, 3));
  });

  test('an almanac carries the eclipses it was asked for', () {
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    // September 2025 at Kathmandu: the total lunar eclipse of the 7th, seen
    // whole near midnight, and the partial solar eclipse of the 21st over
    // the South Pacific, which Nepal does not see.
    (Eclipses, Eclipses?) asked([Map<String, Object?>? settings]) {
      final ctx = teistro.context(
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
        settings: settings,
      );
      Almanac days({bool eclipses = false}) => ctx.almanac.of(
        from: gregorian(2025, 9, 1),
        to: gregorian(2025, 9, 30),
        place: place,
        utcOffsetSeconds: 20700,
        eclipses: eclipses,
      );
      final found = (days(eclipses: true).eclipses!, days().eclipses);
      ctx.dispose();
      return found;
    }

    final (answer, plain) = asked();
    expect(plain, isNull);
    expect(
      [for (final e in answer.value.lunar) e.eclipse.kind],
      [LunarEclipseKind.total],
    );
    expect(
      [for (final e in answer.value.solar) e.eclipse.kind],
      [SolarEclipseKind.partial],
    );
    final lunar = answer.value.lunar.single;
    expect(lunar.eclipse.shadow, EclipseShadowRule.danjon);
    expect(lunar.eclipse.contacts.u2!, lessThan(lunar.eclipse.greatest));
    expect(lunar.here.greatest.altitudeDeg, greaterThan(40));
    expect(
      lunar.here.seen,
      EclipseSeen(from: lunar.here.p1.at, to: lunar.here.p4.at),
    );
    expect(
      lunar.here.umbralSeen,
      EclipseSeen(from: lunar.here.u1!.at, to: lunar.here.u4!.at),
    );
    expect(() => answer.value.lunar.add(lunar), throwsUnsupportedError);
    // The sutak opens praharas before the umbra is seen and closes as it
    // ends (Dharmasindhu p. 28); the unseen solar eclipse has none.
    expect(lunar.vedha!.from, lessThan(lunar.here.umbralSeen!.from));
    expect(lunar.vedha!.to, lunar.here.umbralSeen!.to);
    final solar = answer.value.solar.single;
    expect(solar.here == null || solar.here!.seen == null, isTrue);
    expect(solar.vedha, isNull);
    expect(
      answer.provenance.appliedConventions.map((c) => c.knob),
      contains('eclipse.window'),
    );

    final (chauvenet, _) = asked({
      'panchanga': {'eclipse_shadow': 'CHAUVENET'},
    });
    final shadowed = chauvenet.value.lunar.single.eclipse;
    expect(shadowed.shadow, EclipseShadowRule.chauvenet);
    expect(
      shadowed.umbralMagnitude,
      greaterThan(lunar.eclipse.umbralMagnitude),
    );
    expect(
      chauvenet.provenance.settingsHash,
      isNot(answer.provenance.settingsHash),
    );
  });

  // Each day's Nepal Sambat date beside the days
  // (`03-design/calendar-indian-lunisolar.md` §11): one a day, the year
  // turning at Kachhala's first day, said as the committee's page header
  // prints it.
  test('an almanac carries each day\'s Nepal Sambat date', () {
    final ctx = teistro.context(
      profile: 'nepali-drik',
      locale: 'ne-Deva-NP',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    Almanac days({bool nepalSambat = false}) => ctx.almanac.of(
      from: gregorian(2025, 10, 20),
      to: gregorian(2025, 10, 23),
      place: place,
      utcOffsetSeconds: 20700,
      nepalSambat: nepalSambat,
    );
    expect(days().nepalSambat, isNull);
    expect(days().sky, isNull);
    final sky =
        ctx.almanac
            .of(
              from: gregorian(2025, 10, 20),
              to: gregorian(2025, 10, 23),
              place: place,
              utcOffsetSeconds: 20700,
              sky: true,
              skyStepMinutes: 60,
            )
            .sky!;
    expect(sky.value, hasLength(4));
    final today = sky.value.first;
    expect(today.stepMinutes, 60);
    expect(today.sun.state, 'CROSSES');
    final noon = today.sun.track.where((point) => point.kind == 'TRANSIT');
    expect(noon, hasLength(1));
    expect(noon.first.altitudeDeg, inInclusiveRange(40, 70));
    expect(today.moon.track.every((point) => point.moon != null), isTrue);
    expect(today.sun.track.every((point) => point.moon == null), isTrue);
    final almanac = days(nepalSambat: true);
    final answer = almanac.nepalSambat!;
    expect(answer.value, hasLength(almanac.length));
    expect(answer.value, const [
      NepalSambatDate(
        year: 1145,
        month: 12,
        kind: MonthKind.nija,
        paksha: Paksha.krishna,
      ),
      NepalSambatDate(
        year: 1145,
        month: 12,
        kind: MonthKind.nija,
        paksha: Paksha.krishna,
      ),
      NepalSambatDate(
        year: 1146,
        month: 1,
        kind: MonthKind.nija,
        paksha: Paksha.shukla,
      ),
      NepalSambatDate(
        year: 1146,
        month: 1,
        kind: MonthKind.nija,
        paksha: Paksha.shukla,
      ),
    ]);
    expect(() => answer.value.add(answer.value.first), throwsUnsupportedError);
    expect(answer.provenance.inputHash, almanac.provenance.inputHash);
    final first = answer.value[2];
    expect(
      ctx.intl.messages.sdk.calendar.nepalSambatDate(
        year: first.year,
        month: first.month,
        kind: first.kind.key,
        paksha: first.paksha.key,
      ),
      'ने.सं. ११४६ कछलाथ्व',
    );
    ctx.dispose();
  });

  // A limb's member naming two days or none, and its end in ghatis
  // (`03-design/nepal-day-measured.md`): Nepal's print, under the
  // committee's Surya Siddhanta.
  test('a span says which sunrises it held and when it ended in ghatis', () {
    final ctx = teistro.context(
      profile: 'nepali-committee',
      ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
    );
    AlmanacDay day(int y, int m, int d) => ctx.almanac
        .of(
          from: gregorian(y, m, d),
          to: gregorian(y, m, d),
          place: Observer(
            latitudeDeg: Latitude(27.7172),
            longitudeDeg: Longitude(85.324),
            altitudeM: Altitude(1400),
          ),
          utcOffsetSeconds: 20700,
        )
        .at(0);
    // 13 April 2025: Krishna Pratipada day and night, a vriddhi.
    expect(
      [
        for (final span in day(2025, 4, 13).tithi)
          if (span.sunrises == Sunrises.both) span.member,
      ],
      [Tithi.krishnaPratipada],
    );
    // 26 April 2025: Krishna Chaturdashi between the sunrises, a kshaya.
    final kshaya = day(2025, 4, 26).tithi;
    expect(
      [for (final span in kshaya) span.sunrises],
      [Sunrises.opening, Sunrises.neither, Sunrises.next],
    );
    expect(kshaya[1].member, Tithi.krishnaChaturdashi);
    // 25 September 2026: Chaturdashi ended at 22:16, sunrise 05:54.
    final ends = day(2026, 9, 25).tithi[0].ends;
    expect(ends.ghati, inInclusiveRange(40, 41));
    expect(ends.pala, lessThan(60));
    expect(ends.vipala, lessThan(60));
    ctx.dispose();
  });

  // A day's season is its solar month's, and a month begins on the day
  // Nepal's calendar begins it (`03-design/ritu-measured.md`).
  test('a day\'s season turns on the first of its solar month', () {
    List<Ritu> seasons([Map<String, Object?>? settings]) {
      final ctx = teistro.context(
        profile: 'nepali-drik',
        settings: settings,
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      final days = ctx.almanac.of(
        from: gregorian(2026, 3, 13),
        to: gregorian(2026, 3, 16),
        place: Observer(
          latitudeDeg: Latitude(27.7172),
          longitudeDeg: Longitude(85.324),
          altitudeM: Altitude(1400),
        ),
        utcOffsetSeconds: 20700,
      );
      final named = [for (var k = 0; k < days.length; k++) days.at(k).ritu];
      ctx.dispose();
      return named;
    }

    // 15 March 2026 is 1 Chaitra 2082: Vasanta from that day.
    expect(seasons(), [
      Ritu.shishira,
      Ritu.shishira,
      Ritu.vasanta,
      Ritu.vasanta,
    ]);
    // Amanta Phalguna runs to the new moon of 19 March: Shishira.
    expect(
      seasons({
        'panchanga': {'ritu': 'LUNAR'},
      }),
      List.filled(4, Ritu.shishira),
    );
  });

  test('an almanac carries the festivals it was asked for', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    Almanac days({FestivalRequest? festivals}) => ctx.almanac.of(
      from: gregorian(2026, 10, 15),
      to: gregorian(2026, 11, 10),
      place: place,
      utcOffsetSeconds: 20700,
      festivals: festivals,
    );
    final plain = days();
    expect(plain.festivals, isNull);

    final almanac = days(
      festivals: const FestivalRequest(rules: FestivalPack.dharmasindhu),
    );
    final answer = almanac.festivals!;

    // Without the days: founded as only what the rules read, and the same
    // answer to the bit, stamp and all.
    final lean = ctx.almanac.festivals(
      from: gregorian(2026, 10, 15),
      to: gregorian(2026, 11, 10),
      place: place,
      utcOffsetSeconds: 20700,
      festivals: const FestivalRequest(rules: FestivalPack.dharmasindhu),
    );
    expect(lean.observances, answer.observances);
    expect(lean.ekadashis, answer.ekadashis);
    expect(lean.provenance.contentHash, answer.provenance.contentHash);
    expect(lean.provenance.inputHash, answer.provenance.inputHash);
    expect(answer.observances.every((o) => o.marginSeconds >= 0), isTrue);
    expect(answer.ekadashis, isNotEmpty);
    expect(answer.ekadashis.every((f) => f.marginSeconds > 0), isTrue);
    // Each fast's parana: a window kept inside the daylight's fifths.
    for (final fast in answer.ekadashis) {
      final kept = fast.parana!.kept!;
      expect(kept.at.window, 'PART');
      expect(kept.span.from < kept.span.to, isTrue);
    }
    // A vrata's parana (§8.7): Shivaratri 2026, inside the 14th within
    // three yamas, then at its end.
    final night = ctx.almanac.festivals(
      from: gregorian(2026, 2, 10),
      to: gregorian(2026, 2, 20),
      place: place,
      utcOffsetSeconds: 20700,
      festivals: const FestivalRequest(rules: FestivalPack.dharmasindhu),
    );
    final shivaratri = night.observances.firstWhere(
      (o) => o.rule == 'SHIVARATRI',
    );
    final [within, atEnd] = shivaratri.parana;
    expect((within.reading.kind, within.reading.yamas), ('WITHIN', 3));
    expect(within.day.day, shivaratri.day.day + 1);
    expect(within.from < within.to!, isTrue);
    expect(
      (within.fromBy, atEnd.reading.kind, atEnd.to),
      ('SUNRISE', 'TITHI_END', null),
    );
    // The Navaratri's close (§8.7): October 2025, the meal after the Devi's
    // visarjana by the text, at the 10th's first daylight by the
    // Nirnayasindhu.
    final october = ctx.almanac.festivals(
      from: gregorian(2025, 9, 26),
      to: gregorian(2025, 10, 6),
      place: place,
      utcOffsetSeconds: 20700,
      festivals: const FestivalRequest(rules: FestivalPack.dharmasindhu),
    );
    final navami = october.observances.firstWhere(
      (o) => o.rule == 'MAHA_NAVAMI',
    );
    final visarjana = october.observances.firstWhere(
      (o) => o.rule == 'DEVI_VISARJAN',
    );
    final [afterRite, onTheTenth] = navami.parana;
    expect(
      (
        afterRite.reading.kind,
        afterRite.reading.rite,
        afterRite.reading.remnant,
        onTheTenth.reading.kind,
      ),
      ('AFTER_RITE', 'DEVI_VISARJAN', 0, 'NEXT_TITHI'),
    );
    expect(
      (afterRite.day.month, afterRite.day.day, afterRite.fromBy),
      (visarjana.day.month, visarjana.day.day, 'RITE'),
    );
    // The parana's nakshatra bar (C700): Kathmandu, 7 July 2025, the meal
    // held past Anuradha's first quarter, which ends at 05:36.
    final smarta = <String, Object?>{
      'key': 'EKADASHI_SMARTA',
      'source': '',
      'vedha': 'SUNRISE',
      'table': {
        'pure': {
          'eleventh': 'EARLIER',
          'twelfth': 'EARLIER',
          'both': 'LATER',
          'neither': 'EARLIER',
        },
        'pierced': {
          'eleventh': 'EARLIER',
          'twelfth': 'LATER',
          'both': 'LATER',
          'neither': 'EARLIER',
        },
      },
      'parana': {
        'at': [
          {'window': 'PART', 'part': 'PRATAH'},
        ],
        'hariVasara': true,
        'bars': [
          {
            'tithi': 'SHUKLA_EKADASHI',
            'month': 'ASHADHA',
            'nakshatra': 'ANURADHA',
            'portion': {'parts': 4, 'which': 1},
          },
        ],
      },
    };
    final july = ctx.almanac.festivals(
      from: gregorian(2025, 7, 1),
      to: gregorian(2025, 7, 10),
      place: place,
      utcOffsetSeconds: 20700,
      festivals: FestivalRequest(rules: [smarta]),
    );
    final shayani = july.ekadashis.first.parana!;
    final barred = shayani.barred!;
    expect(
      barred.bar,
      const NakshatraBar(
        tithi: Tithi.shuklaEkadashi,
        month: Masa.ashadha,
        nakshatra: Nakshatra.anuradha,
        parts: 4,
        which: 1,
      ),
    );
    expect(barred.setAside, isFalse);
    expect(
      (shayani.kept!.fromBy, shayani.kept!.span.from),
      ('NAKSHATRA', barred.span.to),
    );

    // An anniversary kept by the tithi (`03-design/shraddha.md`): a parvana
    // of the dark 1st falls on 2 September 2020, the day the committee
    // announced for its Sora Shraddha.
    final sora = ctx.almanac.festivals(
      from: gregorian(2020, 8, 25),
      to: gregorian(2020, 9, 20),
      place: place,
      utcOffsetSeconds: 20700,
      festivals: const FestivalRequest(
        rules: [
          {
            'key': 'PRATIPADA',
            'origin': {
              'by': 'LUNAR',
              'month': 'BHADRAPADA',
              'tithi': 'KRISHNA_PRATIPADA',
            },
            'kind': 'PARVANA',
          },
        ],
      ),
    );
    final pratipada = sora.anniversaries.single;
    expect(
      (pratipada.key, pratipada.kind, pratipada.monthBy),
      ('PRATIPADA', 'PARVANA', 'NIJA'),
    );
    expect(pratipada.origin.tithi, Tithi.krishnaPratipada);
    expect(pratipada.count, isNull);
    expect(pratipada.observance.month, Masa.bhadrapada);
    expect(
      (pratipada.observance.day.month, pratipada.observance.day.day),
      (9, 2),
    );
    expect(july.anniversaries, isEmpty);
    expect(
      [for (final observance in answer.observances) observance.rule],
      [
        'MAHA_NAVAMI',
        'VIJAYA_DASHAMI',
        'DEVI_VISARJAN',
        'SANKASHTA_CHATURTHI',
        'KARAKA_CHATURTHI',
        'LAKSHMI_PUJA',
        'BALI_PRATIPADA',
      ],
    );
    final dashami = answer.observances[1];
    expect(dashami.day.calendar, Calendar.gregorian);
    expect(dashami.day.month, 10);
    expect(dashami.extents.$1.day.calendar, Calendar.gregorian);
    expect(['GUARD', 'OTHERWISE'], contains(dashami.decidedBy.by));
    expect(answer.unjudged, isEmpty);
    // Two Ekadashis in the range, each under the pack's three observers.
    const observers = [
      'EKADASHI_VAISHNAVA',
      'EKADASHI_SMARTA',
      'EKADASHI_SMARTA_RENUNCIANT',
    ];
    expect(
      [for (final fast in answer.ekadashis) fast.rule],
      [...observers, ...observers],
    );
    final fast = answer.ekadashis.first;
    expect((fast.tithi, fast.month), (Tithi.shuklaEkadashi, Masa.ashwina));
    expect(fast.days.$1.calendar, Calendar.gregorian);
    expect(fast.day.calendar, Calendar.gregorian);
    expect(['EARLIER', 'LATER'], contains(fast.choice));
    for (var k = 0; k < plain.length; k += 1) {
      expect(
        almanac.at(k).provenance.contentHash,
        plain.at(k).provenance.contentHash,
      );
    }
    expect({
      for (final c in answer.provenance.appliedConventions) c.knob,
    }, contains('festival.days'));
    // Parsed twice, an observance is the same value.
    expect(
      days(
        festivals: const FestivalRequest(rules: [FestivalPack.dharmasindhu]),
      ).festivals!.observances,
      answer.observances,
    );

    // Lakshmi puja on whichever day holds its tithi at sunrise: a rule of
    // the consumer's own, its members as members, replacing the shipped one.
    final sunrise = <String, Object?>{
      'key': 'LAKSHMI_PUJA',
      'source': 'the tithi at sunrise',
      'month': Masa.ashwina,
      'tithi': Tithi.amavasya,
      'at': {'window': 'SUNRISE'},
      'decide': <Object?>[],
      'otherwise': 'LATER',
    };
    final moved =
        days(
          festivals: FestivalRequest(
            rules: [FestivalPack.dharmasindhu, sunrise],
          ),
        ).festivals!;
    expect(
      moved.observances.singleWhere((o) => o.rule == 'LAKSHMI_PUJA').decidedBy,
      const FestivalDecided(by: 'OTHERWISE'),
    );
    expect(moved.provenance.inputHash, isNot(answer.provenance.inputHash));

    // Nepal's pack, and a day of the consumer's own counted two days from
    // Lakshmi puja's: it names the rule it counts from and the count.
    const following = <String, Object?>{
      'key': 'TWO_AFTER',
      'source': 'mine',
      'after': 'LAKSHMI_PUJA',
      'days': 2,
    };
    final counted =
        days(
          festivals: const FestivalRequest(
            rules: [FestivalPack.nepal, following],
          ),
        ).festivals!;
    final byRule = {for (final o in counted.observances) o.rule: o};
    final (two, lakshmi) = (byRule['TWO_AFTER']!, byRule['LAKSHMI_PUJA']!);
    expect(
      two.decidedBy,
      const FestivalDecided(by: 'AFTER', rule: 'LAKSHMI_PUJA', days: 2),
    );
    expect(two.day.day, lakshmi.day.day + 2);
    expect(
      (two.tithi.from, two.tithi.to),
      (lakshmi.tithi.from, lakshmi.tithi.to),
    );

    // Every observance says its month; Nepal's monthly full-moon fast is
    // judged at the instant of sunset, and a rule of one's own kept every
    // month leaves its month out.
    expect((dashami.month, dashami.adhika), (Masa.ashwina, false));
    final vrata = byRule['PURNIMA_VRATA']!;
    expect(vrata.month, Masa.ashwina);
    expect(vrata.extents.$1.window.from, vrata.extents.$1.window.to);
    const everyMonth = <String, Object?>{
      'key': 'EVERY_FULL_MOON',
      'source': 'mine',
      'tithi': 'tithi.PURNIMA',
      'inAdhika': true,
      'at': {'window': 'SUNSET'},
      'decide': <Object?>[],
      'otherwise': 'LATER',
    };
    final mine =
        days(festivals: const FestivalRequest(rules: [everyMonth])).festivals!;
    expect(
      [for (final o in mine.observances) (o.rule, o.month)],
      [('EVERY_FULL_MOON', Masa.ashwina)],
    );
    final hasta = <String, Object?>{
      for (final entry in everyMonth.entries)
        if (entry.key != 'tithi') entry.key: entry.value,
      'key': 'DARK_HASTA',
      'nakshatra': 'nakshatra.HASTA',
      'paksha': 'paksha.KRISHNA',
    };
    final kept = days(festivals: FestivalRequest(rules: [hasta])).festivals!;
    expect(
      [for (final o in kept.observances) (o.rule, o.month)],
      [('DARK_HASTA', Masa.ashwina)],
    );

    for (final (asked, field) in [
      (const FestivalRequest(rules: 'DHARMASINDHU'), 'festivals.rules'),
      (
        FestivalRequest(
          rules: [
            FestivalPack.dharmasindhu,
            {...sunrise, 'key': ''},
          ],
        ),
        'festivals.rules[1].key',
      ),
      (
        FestivalRequest(
          rules: [
            {
              ...sunrise,
              'at': {'window': 'DUSK'},
            },
          ],
        ),
        'festivals.rules[0].at.window',
      ),
      (
        FestivalRequest(
          rules: [
            {...sunrise, 'nakshatra': 'HASTA', 'paksha': 'SHUKLA'},
          ],
        ),
        'festivals.rules[0]',
      ),
      (
        FestivalRequest(
          rules: [
            {
              ...sunrise,
              'at': {'window': 'NIGHT_MUHURTA', 'muhurta': 16},
            },
          ],
        ),
        'festivals.rules[0].at.muhurta',
      ),
      (
        FestivalRequest(
          rules: [
            {
              ...sunrise,
              'at': {'window': 'DAY_MUHURTA', 'from': 7, 'to': 6},
            },
          ],
        ),
        'festivals.rules[0].at.to',
      ),
      (
        const FestivalRequest(
          rules: [
            FestivalPack.dharmasindhu,
            {
              'key': 'MINE',
              'source': '',
              'vedha': 'DUSK',
              'table': <String, Object?>{},
            },
          ],
        ),
        'festivals.rules[1].vedha',
      ),
      (
        const FestivalRequest(
          rules: [
            FestivalPack.dharmasindhu,
            {'key': 'MINE', 'source': '', 'after': 'HOLIKA', 'days': 16},
          ],
        ),
        'festivals.rules[1].days',
      ),
      (
        const FestivalRequest(
          rules: [
            FestivalPack.dharmasindhu,
            {'key': 'MINE', 'source': '', 'after': 'NOBODY', 'days': 1},
          ],
        ),
        'festivals.following[0].after',
      ),
      (const FestivalRequest(rules: [1]), 'festivals.rules[0]'),
    ]) {
      expect(
        () => days(festivals: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
        reason: field,
      );
    }
    ctx.dispose();
  });

  // `almanac.publishedReasons` (`03-design/published-overrides.md` §4):
  // the committee's day published over the modern sky is the committee's
  // sky, and a day no change gives is the publisher's own.
  test('a published day is explained by the change that gives it', () {
    final modern = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final committee = teistro.context(
      profile: 'nepali-committee',
      ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    FestivalAnswer of(Context ctx) => ctx.almanac.festivals(
      from: gregorian(2027, 9, 1),
      to: gregorian(2027, 11, 30),
      place: kathmandu,
      utcOffsetSeconds: 20700,
      festivals: const FestivalRequest(rules: FestivalPack.nepal),
    );
    int fixed(CalendarDate on) => modern.calendar.fixedOf(on);
    final theirs = of(committee).observances;
    final (observance, committees) =
        [
          for (final o in of(modern).observances)
            for (final t in theirs)
              if (o.rule == t.rule &&
                  fixed(o.day) != fixed(t.day) &&
                  (fixed(o.day) - fixed(t.day)).abs() <= 3)
                (o, t),
        ].first;
    FestivalRequest published(CalendarDate on) => FestivalRequest(
      rules: FestivalPack.nepal,
      published: [
        {
          'format': 'teistro.published/1',
          'publisher': 'a patro printed in Nepal',
          'id': 'np',
          'version': 1,
          'entries': [
            {
              'action': 'SET',
              'rule': observance.rule,
              'source': 'p. 1',
              'day': {
                'calendar': 'GREGORIAN',
                'year': on.year,
                'month': on.month,
                'day': on.day,
              },
            },
          ],
        },
      ],
    );
    PublishedReasons reasons(
      CalendarDate on, {
      Context? sky,
      List<ReckonedPlace> places = const [],
    }) => modern.almanac.publishedReasons(
      from: gregorian(2027, 9, 1),
      to: gregorian(2027, 11, 30),
      place: kathmandu,
      utcOffsetSeconds: 20700,
      festivals: published(on),
      sky: sky,
      places: places,
    );
    final [reason] = reasons(committees.day, sky: committee).reasons;
    expect(
      [reason.rule, fixed(reason.day), fixed(reason.computed)],
      [observance.rule, fixed(committees.day), fixed(observance.day)],
    );
    expect(
      [reason.reason.by, reason.reason.profile],
      ['SKY', 'nepali-committee'],
    );
    final far = modern.calendar.dateOf(
      Calendar.gregorian,
      fixed(observance.day) + 5,
    );
    final delhi = ReckonedPlace(
      name: 'Delhi',
      place: Observer(
        latitudeDeg: Latitude(28.6139),
        longitudeDeg: Longitude(77.209),
        altitudeM: Altitude(216),
      ),
      utcOffsetSeconds: 19800,
    );
    final [own] = reasons(far, places: [delhi]).reasons;
    expect(own.reason.by, 'PUBLISHER');
    expect(
      own.tried.first.change,
      const PublishedCause(by: 'PLACE', name: 'Delhi'),
    );
    expect(own.tried.every((t) => !t.reproduces), isTrue);
    modern.dispose();
    committee.dispose();
  });

  test('a signed pack answers its set or is refused by name', () {
    // The vector the Rust suite pins (crates/sdk/tests/signed_pack.rs).
    final pack = Uint8List.fromList(
      utf8.encode(
        '{"format":"teistro.published/1","publisher":"a patro printed in Nepal","id":"np-2083","version":3,'
        '"entries":[{"action":"SET","rule":"GHATASTHAPANA","day":{"calendar":"GREGORIAN","year":2026,'
        '"month":10,"day":11},"source":"p. 41"}]}',
      ),
    );
    Uint8List hex(String text) => Uint8List.fromList([
      for (var at = 0; at < text.length; at += 2)
        int.parse(text.substring(at, at + 2), radix: 16),
    ]);
    final key = hex(
      'ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c',
    );
    final signature = hex(
      'cae87f1a99d4869d5ba41086cd6ceba8cdb035fd11225f53e09882e98423e213'
      '32fb40c9ad564380fb9abcdf4c21cb2726f80fb692511b9593f0f9e60fbe320e',
    );
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final set = ctx.almanac.publishedFromSigned(
      pack,
      signature,
      key,
      atLeast: 3,
    );
    expect(set['id'], 'np-2083');
    expect(set['version'], 3);
    final tampered = Uint8List.fromList(pack)..[pack.length - 20] ^= 1;
    for (final (call, field) in <(Map<String, Object?> Function(), String)>[
      (
        () => ctx.almanac.publishedFromSigned(tampered, signature, key),
        'signature',
      ),
      (
        () => ctx.almanac.publishedFromSigned(pack, signature, key, atLeast: 4),
        'version',
      ),
      (
        () => ctx.almanac.publishedFromSigned(
          pack,
          signature,
          Uint8List.sublistView(key, 0, 31),
        ),
        'key',
      ),
    ]) {
      expect(
        call,
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
        reason: field,
      );
    }
    ctx.dispose();
  });

  test('a festival day carries its elected sait', () {
    // The NEPAL pack elects Ghatasthapana's in 2025 at the committee's
    // printed 09:13, a Pisces navamsa in a Libra lagna
    // (`03-design/festival-sait.md`).
    final ctx = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    addTearDown(ctx.dispose);
    final answer = ctx.almanac.festivals(
      from: gregorian(2025, 9, 22),
      to: gregorian(2025, 9, 22),
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
      festivals: FestivalRequest(rules: FestivalPack.nepal),
    );
    final sait =
        answer.observances
            .firstWhere((o) => o.rule == 'NAVARATRA_ARAMBHA')
            .sait!;
    final chosen = sait.candidates[sait.chosen!];
    expect(
      [chosen.lagna, chosen.navamsa, chosen.whole, sait.window],
      [Rashi.libra, Rashi.pisces, true, 0],
    );
    final minutes = ((sait.at! + 0.5 + 5.75 / 24) % 1) * 1440;
    expect((minutes - (9 * 60 + 13)).abs(), lessThan(2));
    expect(
      answer.observances
          .where((o) => o.rule != 'NAVARATRA_ARAMBHA')
          .every((o) => o.sait == null),
      isTrue,
    );
  });

  test('a published set lays its days over the rules', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    FestivalAnswer asked([List<Map<String, Object?>> published = const []]) =>
        ctx.almanac.festivals(
          from: gregorian(2025, 9, 26),
          to: gregorian(2025, 10, 6),
          place: place,
          utcOffsetSeconds: 20700,
          festivals: FestivalRequest(
            rules: FestivalPack.dharmasindhu,
            published: published,
          ),
        );
    final plain = asked();
    expect(plain.added, isEmpty);
    expect(plain.unplaced, isEmpty);
    expect(plain.applied, isEmpty);
    expect(plain.observances.every((o) => o.published == null), isTrue);
    final navami = plain.observances.firstWhere((o) => o.rule == 'MAHA_NAVAMI');
    final visarjana = plain.observances.firstWhere(
      (o) => o.rule == 'DEVI_VISARJAN',
    );
    Map<String, Object?> on(CalendarDate of, [int by = 0]) => {
      'calendar': 'GREGORIAN',
      'year': of.year,
      'month': of.month,
      'day': of.day + by,
    };
    final set = <String, Object?>{
      'format': 'teistro.published/1',
      'publisher': 'a patro printed in Nepal',
      'id': 'np-2082',
      'version': 2,
      'entries': [
        {
          'action': 'SET',
          'rule': 'MAHA_NAVAMI',
          'day': on(navami.day, 1),
          'source': 'p. 1',
        },
        {
          'action': 'WITHDRAW',
          'rule': 'DEVI_VISARJAN',
          'day': on(visarjana.day),
          'source': 'p. 2',
        },
        {
          'action': 'ADD',
          'key': 'MINE',
          'day': on(navami.day),
          'names': {'en-Latn': 'Mine'},
          'source': 'p. 3',
        },
        {
          'action': 'SET',
          'rule': 'HOLIKA',
          'day': on(navami.day, 2),
          'source': 'p. 4',
        },
      ],
    };
    final answer = asked([set]);
    final moved = answer.observances.firstWhere((o) => o.rule == 'MAHA_NAVAMI');
    expect(
      moved.published,
      Published(
        status: 'PUBLISHED',
        publisher: 'a patro printed in Nepal',
        set: 'np-2082',
        version: 2,
        source: 'p. 1',
        computed: navami.day,
      ),
    );
    expect(moved.day.day, navami.day.day + 1);
    expect(moved.parana, isEmpty);
    final withdrawn = answer.observances.firstWhere(
      (o) => o.rule == 'DEVI_VISARJAN',
    );
    expect(withdrawn.published!.status, 'WITHDRAWN');
    expect(_dateKey(withdrawn.day), _dateKey(visarjana.day));
    expect(answer.added, hasLength(1));
    expect(answer.added.single.key, 'MINE');
    expect(_dateKey(answer.added.single.day), _dateKey(navami.day));
    expect(answer.added.single.names, {'en-Latn': 'Mine'});
    expect(
      [
        for (final u in answer.unplaced) [u.set, u.entry, u.rule],
      ],
      [
        ['np-2082', 3, 'HOLIKA'],
      ],
    );
    expect(
      [
        for (final a in answer.applied) [a.id, a.version, a.hash.length],
      ],
      [
        ['np-2082', 2, 64],
      ],
    );
    expect(answer.provenance.inputHash, isNot(plain.provenance.inputHash));
    final stray = {
      ...set,
      'entries': [
        {
          ...(set['entries']! as List<Object?>).first! as Map<String, Object?>,
          'rule': 'NO_SUCH_RULE',
        },
      ],
    };
    expect(
      () => asked([stray]),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'festivals.published[0].entries[0].rule',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its dispositors', () {
    // Each chain the sign lords followed from the planet's own sign until one
    // repeats, its end one of the chart's ends, and the final dispositor
    // exactly when every chain stops at one planet in its own sign
    // (`03-design/essential-dignities.md` §Dispositors).
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    // The traditional lords, Aries to Pisces.
    const lords = [
      Graha.mars,
      Graha.venus,
      Graha.mercury,
      Graha.moon,
      Graha.sun,
      Graha.mercury,
      Graha.venus,
      Graha.mars,
      Graha.jupiter,
      Graha.saturn,
      Graha.saturn,
      Graha.jupiter,
    ];
    final finals = <bool>{};
    for (var step = 0; step < 30; step += 1) {
      final read =
          ctx.chart
              .found(
                instant: 2460676.5 + step * 23.7,
                place: kathmandu,
                utcOffsetSeconds: 20700,
                dignities: const DignityRequest(),
              )
              .dignities!;
      final lordOf = {
        for (final p in read.planets) p.planet: lords[p.longitudeDeg ~/ 30],
      };
      final disposed = read.dispositions;
      expect(disposed.rulers, 'TRADITIONAL');
      expect(
        [for (final c in disposed.chains) c.planet],
        [for (final p in read.planets) p.planet],
      );
      for (final chain in disposed.chains) {
        final walked = <Graha>[];
        for (
          var at = lordOf[chain.planet]!;
          at != chain.planet && !walked.contains(at);
          at = lordOf[at]!
        ) {
          walked.add(at);
        }
        expect(chain.dispositors, walked, reason: '${chain.planet}');
        expect(disposed.ends, contains(chain.end));
        final last = walked.isEmpty ? chain.planet : walked.last;
        if (chain.end.kind == ChainEnd.ownSign) {
          expect(chain.end.planets, [last]);
          expect(lordOf[last], last);
        } else {
          expect(chain.end.kind, ChainEnd.cycle);
          expect(chain.end.planets.length, greaterThanOrEqualTo(2));
          expect(chain.end.planets, contains(last));
        }
      }
      final one =
          disposed.ends.length == 1 &&
          disposed.ends.first.kind == ChainEnd.ownSign;
      expect(
        disposed.finalDispositor,
        one ? disposed.ends.first.planets.first : null,
      );
      finals.add(disposed.finalDispositor == null);
    }
    expect(finals, {true, false}, reason: 'both a final dispositor and none');
    ctx.dispose();
  });

  test('a chart carries its essential dignities', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    const instants = [2460676.5, 2460676.75];
    Dignities? found(
      double instant,
      DignityRequest? asked, {
      Observer? place,
      int utcOffsetSeconds = 20700,
    }) =>
        ctx.chart
            .found(
              instant: instant,
              place: place ?? kathmandu,
              utcOffsetSeconds: utcOffsetSeconds,
              dignities: asked,
            )
            .dignities;
    expect(found(instants[0], null), isNull);

    final read = found(instants[0], const DignityRequest())!;
    expect(read.sectRule, SectRule.horizon);
    expect(
      read.rules,
      const AppliedDignityRules(
        terms: Terms.ptolemaicLilly,
        triplicities: Triplicities.lilly,
      ),
    );
    expect(read.scores, DignityScores.lilly);
    expect(read.planets.map((at) => at.planet), [
      Graha.saturn,
      Graha.jupiter,
      Graha.mars,
      Graha.sun,
      Graha.venus,
      Graha.mercury,
      Graha.moon,
    ]);
    const lilly = DignityScores.lilly;
    for (final at in read.planets) {
      final d = at.dignity;
      final held = [d.house, d.exaltation, d.triplicity, d.term, d.face];
      expect(at.peregrine, !held.contains(true), reason: '${at.planet}');
      final worth = [
        (d.house, lilly.house),
        (d.exaltation, lilly.exaltation),
        (d.triplicity, lilly.triplicity),
        (d.term, lilly.term),
        (d.face, lilly.face),
        (d.detriment, lilly.detriment),
        (d.fall, lilly.fall),
        (at.peregrine, lilly.peregrine),
      ];
      final score = worth
          .where((one) => one.$1)
          .fold<int>(0, (sum, one) => sum + one.$2);
      expect(at.score, score, reason: '${at.planet}');
    }

    // Each reception whole both ways, in the Chaldean order, and scored
    // only when mutual by house or by exaltation.
    final order = [for (final at in read.planets) at.planet];
    expect(read.receptions, isNotEmpty);
    for (final one in read.receptions) {
      final (first, second) = one.planets;
      expect(order.indexOf(first), lessThan(order.indexOf(second)));
      for (final side in [one.firstIn, one.secondIn]) {
        expect(DignityKind.values.any(side.holds), isTrue, reason: '$one');
      }
      expect(one.mutual, [
        for (final kind in DignityKind.values)
          if (one.firstIn.holds(kind) && one.secondIn.holds(kind)) kind,
      ]);
    }
    for (final at in read.planets) {
      bool by(DignityKind kind) => read.receptions.any(
        (one) =>
            (one.planets.$1 == at.planet || one.planets.$2 == at.planet) &&
            one.mutual.contains(kind),
      );
      expect(
        at.reception,
        (by(DignityKind.house) ? lilly.house : 0) +
            (by(DignityKind.exaltation) ? lilly.exaltation : 0),
        reason: '${at.planet}',
      );
    }

    // Every rule reported as asked, and a score left out stays Lilly's.
    final night =
        found(
          instants[0],
          const DignityRequest(
            sectRule: SectRule.night,
            triplicities: Triplicities.ptolemy,
            scores: DignityScores(peregrine: 0),
          ),
        )!;
    expect(night.sect, Sect.night);
    expect(night.rules.triplicities, Triplicities.ptolemy);
    expect(night.scores, const DignityScores(peregrine: 0));

    // 21 December 1988 at Tromsø: the Sun culminates under the horizon.
    final tromso = Observer(
      latitudeDeg: Latitude(69.6492),
      longitudeDeg: Longitude(18.9553),
      altitudeM: Altitude(0),
    );
    expect(
      found(
        2447516.9583333335,
        const DignityRequest(),
        place: tromso,
        utcOffsetSeconds: 3600,
      )!.sect,
      Sect.night,
    );

    // A table of the caller's own: Aries' Egyptian terms in every sign.
    const row = [
      Term(Graha.jupiter, 6),
      Term(Graha.venus, 12),
      Term(Graha.mercury, 20),
      Term(Graha.mars, 25),
      Term(Graha.saturn, 30),
    ];
    final own =
        found(instants[0], DignityRequest(table: List.filled(12, row)))!;
    expect(own.rules.terms, Terms.table);
    for (final at in own.planets) {
      final degree = at.longitudeDeg % 30;
      final lord = row.firstWhere((term) => degree < term.end).lord;
      expect(at.dignity.term, lord == at.planet, reason: '${at.planet}');
    }

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      dignities: const DignityRequest(),
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).dignities,
        found(instant, const DignityRequest()),
        reason: 'each alone',
      );
    }

    for (final (bad, field) in [
      (
        DignityRequest(table: List.filled(11, row)),
        'dignities.rules.terms.TABLE',
      ),
      // `TABLE` names a table in an answer and carries none in a request.
      (const DignityRequest(terms: Terms.table), 'dignities.rules.terms.TABLE'),
    ]) {
      expect(
        () => found(instants[0], bad),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its lots', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    const instants = [2460676.5, 2460676.75];
    Chart found(
      double instant, {
      LotRequest? lots,
      FortitudeRequest? fortitudes,
    }) => ctx.chart.found(
      instant: instant,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      lots: lots,
      fortitudes: fortitudes,
    );
    double apart(double one, double other) =>
        math.min((one - other) % 360, (other - one) % 360);
    expect(found(instants[0]).lots, isNull);

    for (final instant in instants) {
      final chart = found(
        instant,
        lots: LotRequest.valens,
        fortitudes: const FortitudeRequest(),
      );
      final read = chart.lots!;
      final fortitudes = chart.fortitudes!;
      expect(read.request, LotRequest.valens);
      expect(read.fortuneReversed, read.sect == Sect.night);
      expect([for (final placed in read.lots) placed.lot], Lot.values);
      final at = {
        for (final placed in read.lots) placed.lot: placed.place.longitudeDeg,
      };
      final longitude = {
        for (final own in fortitudes.dignities.planets)
          own.planet: own.longitudeDeg,
      };
      final ascendant = fortitudes.sky.ascendantDeg;
      final (start, end) =
          read.fortuneReversed
              ? (Graha.moon, Graha.sun)
              : (Graha.sun, Graha.moon);
      final fortune = (ascendant + longitude[end]! - longitude[start]!) % 360;
      expect(apart(at[Lot.fortune]!, fortune), lessThan(1e-9));
      expect(
        apart(at[Lot.daimon]!, 2 * ascendant - fortune),
        lessThan(1e-9),
        reason: 'Daimon mirrors Fortune',
      );
      for (final placed in read.lots) {
        expect(placed.place.longitudeDeg, inInclusiveRange(0, 360));
        expect(placed.place.house, inInclusiveRange(1, 12));
      }
      // The answer's rules are a request as they stand.
      expect(found(instant, lots: read.request).lots, read);
    }

    const asked = LotRequest(
      sectRule: SectRule.daylight,
      fortune: FortuneRule.reversedWhileMoonUp,
    );
    expect(found(instants[0], lots: asked).lots!.request, asked);

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      lots: LotRequest.valens,
    );
    for (final (k, instant) in instants.indexed) {
      expect(batch.at(k).lots, found(instant, lots: LotRequest.valens).lots);
    }
    ctx.dispose();
  });

  test('a chart carries its considerations', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(-0.12),
      altitudeM: Altitude(0),
    );
    final instants = [for (var k = 0; k < 16; k += 1) 2451545 + k / 8];
    Chart found(
      double instant, {
      ConsiderationRules? considerations,
      FortitudeRequest? fortitudes,
    }) => ctx.chart.found(
      instant: instant,
      place: london,
      utcOffsetSeconds: 0,
      considerations: considerations,
      fortitudes: fortitudes,
    );
    expect(found(instants[0]).considerations, isNull);

    var voids = 0;
    for (final instant in instants) {
      final chart = found(
        instant,
        considerations: ConsiderationRules.lilly,
        fortitudes: const FortitudeRequest(),
      );
      final read = chart.considerations!;
      final sky = chart.fortitudes!.sky;
      expect(read.rules, ConsiderationRules.lilly);
      expect(read.ascendant.sign.id, sky.ascendantDeg ~/ 30);
      expect(read.seventh.cuspDeg, sky.cuspsDeg[6]);
      expect(read.ascendant.early, read.ascendant.degree < 3);
      final course = read.moon.course;
      if (course.next == null) expect(course.withinOrb, isNull);
      if (course.next == null || course.withinOrb == null) voids += 1;
      expect(
        read.radicality.grounds.contains(RadicalGround.oneLord),
        read.radicality.hourLord == read.radicality.ascendantLord,
      );
      // The answer's rules are a request as they stand.
      expect(found(instant, considerations: read.rules).considerations, read);
    }
    expect(voids, greaterThan(0), reason: 'a void Moon in the batch');

    const asked = ConsiderationRules(
      moonLateFromDeg: 25,
      orbsDeg: [9, 9, 7, 15, 7, 7, 12],
    );
    expect(
      found(instants[0], considerations: asked).considerations!.rules,
      asked,
    );

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: london,
      utcOffsetSeconds: 0,
      considerations: ConsiderationRules.lilly,
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).considerations,
        found(instant, considerations: ConsiderationRules.lilly).considerations,
      );
    }
    for (final (rules, field) in [
      (
        const ConsiderationRules(moonLateFromDeg: 31),
        'considerations.moonLateFromDeg',
      ),
      (
        const ConsiderationRules(orbsDeg: [10, 12, 7.5, 17, 8, 7, -1]),
        'considerations.orbsDeg',
      ),
    ]) {
      expect(
        () => found(instants[0], considerations: rules),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its perfection', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(-0.12),
      altitudeM: Altitude(0),
    );
    final instants = [for (var k = 0; k < 12; k += 1) 2451545 + 23 * k + k / 7];
    const seventh = PerfectionRequest.ofHouse(7);
    Chart found(
      double instant, {
      PerfectionRequest? perfection,
      FortitudeRequest? fortitudes,
    }) => ctx.chart.found(
      instant: instant,
      place: london,
      utcOffsetSeconds: 0,
      perfection: perfection,
      fortitudes: fortitudes,
    );
    expect(found(instants[0]).perfection, isNull);

    var applying = 0;
    var hindered = 0;
    for (final instant in instants) {
      final chart = found(
        instant,
        perfection: seventh,
        fortitudes: const FortitudeRequest(),
      );
      final read = chart.perfection!;
      expect(read.rules, PerfectionRules.lilly);
      expect(read.querent, isNot(read.quesited));
      final houses = {
        for (final at in chart.fortitudes!.planets) at.planet: at.house,
      };
      expect(read.ways.querent.house, houses[read.querent]);
      expect(read.ways.quesited.house, houses[read.quesited]);
      if (read.application case final application?) {
        applying += 1;
        expect(application.days, inInclusiveRange(0, read.horizonDays));
        expect([read.querent, read.quesited], contains(application.applying));
      }
      for (final way in [
        Way.conjunction,
        Way.sextileOrTrine,
        Way.square,
        Way.opposition,
      ]) {
        if (read.ways.held.contains(way)) {
          expect(read.application, isNotNull, reason: '$way');
        }
      }
      for (final impediment in read.impediments) {
        hindered += 1;
        expect(
          impediment.third == null,
          impediment.kind == ImpedimentKind.refranation,
        );
      }
      for (final translation in read.translations) {
        expect(
          {translation.from, translation.to},
          {read.querent, read.quesited},
        );
      }
      // The answer's rules are a request as they stand.
      expect(
        found(
          instant,
          perfection: PerfectionRequest.ofHouse(7, rules: read.rules),
        ).perfection,
        read,
      );
    }
    expect(applying, greaterThan(0), reason: 'the sweep applies');
    expect(hindered, greaterThan(0), reason: 'and is hindered');

    final named =
        found(
          instants[0],
          perfection: const PerfectionRequest.between(
            Graha.venus,
            Graha.mars,
            rules: PerfectionRules(horizonDays: 30),
          ),
        ).perfection!;
    expect(
      (
        named.querent,
        named.quesited,
        named.rules.horizonDays,
        named.horizonDays,
      ),
      (Graha.venus, Graha.mars, 30.0, 30.0),
    );
    final every =
        found(
          instants[0],
          perfection: const PerfectionRequest.between(
            Graha.venus,
            Graha.mars,
            rules: PerfectionRules(withinSign: false),
          ),
        ).perfection!;
    expect(every.rules.withinSign, isFalse);

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: london,
      utcOffsetSeconds: 0,
      perfection: seventh,
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).perfection,
        found(instant, perfection: seventh).perfection,
      );
    }
    for (final (asked, field) in [
      (const PerfectionRequest(), 'perfection.quesited'),
      (
        const PerfectionRequest(house: 7, quesited: Graha.mars),
        'perfection.house',
      ),
      (
        const PerfectionRequest.ofHouse(
          7,
          rules: PerfectionRules(horizonDays: -1),
        ),
        'perfection.rules.horizonDays',
      ),
    ]) {
      expect(
        () => found(instants[0], perfection: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its progressions', () {
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(0),
      altitudeM: Altitude(0),
    );
    const birth = 2400629.742361111;
    Chart found(double instant, {ProgressionsRequest? progressions}) =>
        ctx.chart.found(
          instant: instant,
          place: london,
          utcOffsetSeconds: 0,
          progressions: progressions,
        );
    expect(found(birth).progressions, isNull);

    // His forty-seventh year: the map at sidereal time 5h 54m 16s (p. 35).
    const at = birth + 46 * 365.242189;
    final read =
        found(
          birth,
          progressions: const ProgressionsRequest(at: at),
        ).progressions!;
    final progressed = read.progressed!;
    final directed = read.directed!;
    expect(progressed.sky, closeTo(birth + 46, 1e-9));
    expect(progressed.armcDeg / 15, closeTo(5 + 54 / 60 + 16 / 3600, 2 / 3600));
    expect(progressed.grahas.length, directed.planets.length);
    expect(read.contacts, isNull);
    final sun = progressed.grahas.firstWhere((g) => g.graha == Graha.sun);
    final directedSun = directed.planets.firstWhere(
      (g) => g.graha == Graha.sun,
    );
    expect(sun.longitudeDeg, closeTo(directedSun.longitudeDeg, 1e-9));

    // The Moon sesquiquadrate Mercury (p. 305): the 21st by a year, the
    // 22nd by his rule.
    const october = ProgressionContacts(
      from: 2417484.5,
      to: 2417515.5,
      grahas: [Graha.moon],
      points: [NatalGraha(Graha.mercury)],
      aspects: [135],
    );
    for (final (year, day) in [
      (YearMeasure.tropical, 21),
      (YearMeasure.noonSiderealTime, 22),
    ]) {
      final contacts =
          found(
            birth,
            progressions: ProgressionsRequest(year: year, contacts: october),
          ).progressions!;
      expect(contacts.progressed, isNull);
      final [contact] = contacts.contacts!;
      expect(
        (contact.graha, contact.to, contact.angle, contact.motion),
        (Graha.moon, const NatalGraha(Graha.mercury), 135, Motion.direct),
      );
      expect((contact.life - 2417484.5).floor() + 1, day, reason: '$year');
    }
    final none =
        found(
          birth,
          progressions: const ProgressionsRequest(
            contacts: ProgressionContacts(
              from: 2417484.5,
              to: 2417515.5,
              grahas: [Graha.moon],
              aspects: [90],
            ),
          ),
        ).progressions!;
    expect(none.contacts, isEmpty, reason: 'a window asked holding none');
    expect(none.parallels, isNull, reason: 'none unless the window asks');

    // Jupiter "has moved to the parallel of the radical Sun" (p. 33):
    // within Leo's degree as his year opens, and separating.
    final leo =
        found(
          birth,
          progressions: const ProgressionsRequest(
            contacts: ProgressionContacts(
              from: 2417523.5,
              to: 2417888.5,
              grahas: [Graha.jupiter],
              points: [NatalGraha(Graha.sun)],
              parallels: ParallelRequest(),
            ),
          ),
        ).progressions!;
    final [parallel] = leo.parallels!;
    expect(
      (
        parallel.graha,
        parallel.to,
        parallel.enters,
        parallel.closest,
        parallel.contrary,
      ),
      (Graha.jupiter, const NatalGraha(Graha.sun), null, 2417523.5, false),
    );
    expect(parallel.closestDeg, lessThan(1));
    expect(leo.mutual, isNull, reason: 'no mutual contacts unless asked');

    // The progressed Moon quincunx the progressed Venus (p. 41, "P. to
    // P."), July 1907 by Leo's month for each degree.
    final pp =
        found(
          birth,
          progressions: const ProgressionsRequest(
            contacts: ProgressionContacts(
              from: 2417484.5,
              to: 2417941.5,
              grahas: [Graha.moon, Graha.venus],
              aspects: [150],
              mutual: true,
            ),
          ),
        ).progressions!;
    final [venus] = pp.mutual!;
    expect(
      (venus.first, venus.second, venus.angle),
      (Graha.moon, Graha.venus, 150),
    );
    expect(venus.life, inInclusiveRange(2417712.5, 2417773.5));

    final instants = [birth, birth + 3000.25, birth + 9000.5];
    const many = ProgressionsRequest(
      at: 2430000.5,
      angles: AngleMethod.solarArcLongitude,
      direction: DirectionArc.naibod,
    );
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: london,
      utcOffsetSeconds: 0,
      progressions: many,
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).progressions,
        found(instant, progressions: many).progressions,
      );
    }
    for (final (asked, field) in [
      (const ProgressionsRequest(), 'progressions.at'),
      (
        const ProgressionsRequest(at: at, direction: DirectionArc.perYear(0)),
        'progressions.direction',
      ),
      (
        const ProgressionsRequest(
          contacts: ProgressionContacts(from: 2, to: 1),
        ),
        'progressions.contacts.to',
      ),
      (
        const ProgressionsRequest(
          contacts: ProgressionContacts(
            from: 1,
            to: 2,
            parallels: ParallelRequest(orbDeg: 11),
          ),
        ),
        'progressions.contacts.parallels.orbDeg',
      ),
    ]) {
      expect(
        () => found(birth, progressions: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart places the Black Moon Lilith its request names', () {
    // The request's `lilith` places the apogee it names beside the nine and
    // leaves the nine as they were (`03-design/lilith.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(0),
      altitudeM: Altitude(0),
    );
    Chart found([Lilith? lilith]) => ctx.chart.found(
      instant: 2451545.0,
      place: london,
      utcOffsetSeconds: 0,
      lilith: lilith,
    );
    final plain = found();
    expect(plain.lilith, isNull);

    final founded = found(Lilith.mean);
    final mean = founded.lilith!;
    // ELP2000-82B's mean perigee at J2000 is 83.3532°, so the apogee stands
    // half a circle on, in Sagittarius (Meeus, ch. 47).
    expect(mean.apogee, Lilith.mean);
    expect(mean.longitudeDeg, inExclusiveRange(263, 264));
    // A direction on the ecliptic of date, with no distance: the latitude is
    // the frame's rounding, a ten-billionth of a degree.
    expect(mean.latitudeDeg.abs(), lessThan(1e-6));
    expect(mean.distanceAu, 0);
    expect(mean.retrograde, isFalse);
    expect(mean.house.bhava, inInclusiveRange(1, 12));
    // Asking for it leaves the nine as they were, to the bit.
    expect(
      [for (final g in founded.grahas) g.longitudeDeg],
      [for (final g in plain.grahas) g.longitudeDeg],
    );

    final osculating = found(Lilith.osculating).lilith!;
    expect(osculating.apogee, Lilith.osculating);
    expect(osculating.distanceAu, greaterThan(0.0026));
    // The two apogees part by no more than about thirty degrees.
    final apart =
        ((osculating.longitudeDeg - mean.longitudeDeg + 540) % 360 - 180).abs();
    expect(apart, lessThan(31));

    expect(
      () => found(Lilith.none),
      throwsA(isA<ArgumentError>().having((e) => e.name, 'name', 'lilith')),
    );
    ctx.dispose();
  });

  test('a chart carries its Western aspects', () {
    // King Edward VII's nativity: Leo's four (*How to Judge a Nativity*,
    // pp. 295–296) under his orbs, Lilly's moieties refusing the outer
    // three, a batch the charts one at a time, and refusals named in the
    // record (`03-design/western-aspects.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final palace = Observer(
      latitudeDeg: Latitude(51.501),
      longitudeDeg: Longitude(-0.142),
      altitudeM: Altitude(0),
    );
    const birth = 2393783.95;
    Chart found(
      double instant, {
      WesternAspectRequest? asked,
      bool outerPlanets = false,
    }) => ctx.chart.found(
      instant: instant,
      place: palace,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      westernAspects: asked,
    );
    expect(found(birth).westernAspects, isNull);

    final rows =
        found(
          birth,
          asked: const WesternAspectRequest(),
          outerPlanets: true,
        ).westernAspects!;
    final held = {
      for (final row in rows) ({row.first, row.second}, row.aspect),
    };
    bool holds(Graha a, WesternAspect aspect, Graha b) =>
        held.any((h) => h.$2 == aspect && h.$1.containsAll({a, b}));
    for (final (a, aspect, b) in [
      (Graha.sun, WesternAspect.trine, Graha.uranus),
      (Graha.sun, WesternAspect.sextile, Graha.mars),
      (Graha.sun, WesternAspect.square, Graha.neptune),
      (Graha.moon, WesternAspect.square, Graha.saturn),
    ]) {
      expect(holds(a, aspect, b), isTrue, reason: '$a $aspect $b');
    }
    expect(rows.every((row) => row.fromExactDeg <= row.orbDeg), isTrue);

    // Lilly's moieties over the seven: the Moon (12½) and Saturn (10)
    // square within 11¼.
    final square = found(
      birth,
      asked: WesternAspectRequest.lilly,
    ).westernAspects!.firstWhere(
      (row) => row.first == Graha.moon && row.second == Graha.saturn,
    );
    expect((square.aspect, square.orbDeg), (WesternAspect.square, 11.25));

    final instants = [birth, birth + 3000.25, birth + 9000.5];
    const two = WesternAspectRequest(
      aspects: [WesternAspect.trine, WesternAspect.square],
    );
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: palace,
      utcOffsetSeconds: 0,
      westernAspects: two,
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).westernAspects,
        found(instant, asked: two).westernAspects,
      );
    }
    for (final (asked, field, outer) in [
      (
        const WesternAspectRequest(aspects: []),
        'westernAspects.aspects',
        false,
      ),
      (
        const WesternAspectRequest(
          aspects: [WesternAspect.trine, WesternAspect.trine],
        ),
        'westernAspects.aspects',
        false,
      ),
      (
        const WesternAspectRequest(
          aspects: [WesternAspect.trine],
          orbs: OrbModel.byAspect({WesternAspect.trine: 91}),
        ),
        'westernAspects.orbs.orbs',
        false,
      ),
      (WesternAspectRequest.lilly, 'westernAspects.orbs.orbs', true),
    ]) {
      expect(
        () => found(birth, asked: asked, outerPlanets: outer),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its aspect configurations', () {
    // King Edward VII's nativity: his two T-squares as the SDK pins them,
    // each read back from the chart's own aspect table, and refusals
    // named in the record (`03-design/western-configurations.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final palace = Observer(
      latitudeDeg: Latitude(51.501),
      longitudeDeg: Longitude(-0.142),
      altitudeM: Altitude(0),
    );
    const birth = 2393783.95;
    Chart found({WesternConfigurationRequest? asked}) => ctx.chart.found(
      instant: birth,
      place: palace,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      westernAspects: const WesternAspectRequest(),
      westernConfigurations: asked,
    );
    expect(found().westernConfigurations, isNull);

    final chart = found(asked: const WesternConfigurationRequest());
    final shapes = chart.westernConfigurations!;
    expect(
      [
        for (final shape in shapes)
          '${shape.kind.key} ${shape.bodies.map((b) => b.key).join(',')} '
              '${shape.apex?.key} ${shape.sign} ${shape.inside}',
      ],
      [
        'T_SQUARE MOON,JUPITER,URANUS JUPITER null []',
        'T_SQUARE MARS,VENUS,PLUTO MARS null []',
      ],
    );
    bool holds(Graha a, WesternAspect aspect, Graha b) =>
        chart.westernAspects!.any(
          (row) =>
              row.aspect == aspect &&
              {row.first, row.second}.containsAll({a, b}),
        );
    for (final shape in shapes) {
      final apex = shape.apex!;
      final [x, y] = [
        for (final body in shape.bodies)
          if (body != apex) body,
      ];
      expect(holds(x, WesternAspect.opposition, y), isTrue);
      expect(holds(apex, WesternAspect.square, x), isTrue);
      expect(holds(apex, WesternAspect.square, y), isTrue);
      expect(shape.widestFromExactDeg, inInclusiveRange(0, 10));
    }
    for (final (asked, field) in [
      (
        const WesternConfigurationRequest(stelliumLeast: 2),
        'westernConfigurations.stelliumLeast',
      ),
      (
        const WesternConfigurationRequest(
          aspects: WesternAspectRequest(aspects: []),
        ),
        'westernConfigurations.aspects',
      ),
    ]) {
      expect(
        () => found(asked: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its declinations and parallels', () {
    // King George V (Leo, *How to Judge a Nativity*, p. 130): the recast's
    // declinations, his four parallels, a batch the charts one at a time,
    // and refusals named in the record
    // (`03-design/western-declinations.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final george = Observer(
      latitudeDeg: Latitude(51.5045),
      longitudeDeg: Longitude(-0.1366),
      altitudeM: Altitude(0),
    );
    const birth = 2402390.554166667;
    Chart found(
      double instant, {
      ParallelRequest? asked,
      bool outerPlanets = false,
    }) => ctx.chart.found(
      instant: instant,
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      parallels: asked,
    );
    expect(found(birth).declinations, isNull);
    expect(found(birth).parallels, isNull);

    final chart = found(
      birth,
      asked: const ParallelRequest(),
      outerPlanets: true,
    );
    final read = chart.declinations!;
    expect(read.grahas, hasLength(10));
    expect(read.graha(Graha.sun), closeTo(22.2997, 0.01));
    expect(read.lagnaDeg, closeTo(0.8366, 0.01));
    expect(read.midheavenDeg, closeTo(-23.452, 0.01));
    expect(
      [
        for (final row in chart.parallels!)
          (row.first, row.second, row.contrary),
      ],
      [
        (Graha.moon, Graha.neptune, true),
        (Graha.sun, Graha.jupiter, true),
        (Graha.jupiter, Graha.uranus, true),
        (Graha.mercury, Graha.venus, false),
      ],
    );

    final instants = [birth, birth - 3000.25];
    const wider = ParallelRequest(orbDeg: 1.5);
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: george,
      utcOffsetSeconds: 0,
      parallels: wider,
    );
    for (final (k, instant) in instants.indexed) {
      final one = found(instant, asked: wider);
      expect(batch.at(k).parallels, one.parallels);
      expect(batch.at(k).declinations, one.declinations);
    }
    for (final orb in [0.0, 11.0]) {
      expect(
        () => found(birth, asked: ParallelRequest(orbDeg: orb)),
        throwsA(
          isA<TeistroException>().having(
            (e) => e.field,
            'field',
            'parallels.orbDeg',
          ),
        ),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its equal distances', () {
    // King George V (Leo, *How to Judge a Nativity*, p. 130): his recast's
    // one under the default, Pluto on the far point of the Moon and
    // Jupiter, eight at 1.5°, and refusals named in the record
    // (`03-design/western-midpoints.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final george = Observer(
      latitudeDeg: Latitude(51.5045),
      longitudeDeg: Longitude(-0.1366),
      altitudeM: Altitude(0),
    );
    const birth = 2402390.554166667;
    Chart found(double instant, {MidpointRequest? asked}) => ctx.chart.found(
      instant: instant,
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      midpoints: asked,
    );
    expect(found(birth).midpoints, isNull);

    final rows = found(birth, asked: const MidpointRequest()).midpoints!;
    expect(rows, hasLength(1));
    final row = rows.single;
    expect(
      (row.first, row.second, row.middle, row.far),
      (Graha.moon, Graha.jupiter, Graha.pluto, true),
    );
    expect(row.fromAxisDeg, closeTo(0.052, 0.01));
    expect(row.distanceDeg, closeTo(137.69, 0.01));
    expect(row.orbDeg, 0.5);

    const wide = MidpointRequest(orbDeg: 1.5);
    final eight = found(birth, asked: wide).midpoints!;
    expect(eight, hasLength(8));
    for (final (n, at) in eight.indexed.skip(1)) {
      expect(eight[n - 1].fromAxisDeg, lessThanOrEqualTo(at.fromAxisDeg));
    }
    final batch = ctx.chart.foundMany(
      instants: [birth, birth - 3000.25],
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      midpoints: wide,
    );
    expect(batch.at(0).midpoints, found(birth, asked: wide).midpoints);
    expect(
      () => found(birth, asked: const MidpointRequest(orbDeg: 11)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'midpoints.orbDeg',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its antiscia', () {
    // King George V (Leo, *How to Judge a Nativity*, p. 130): his recast's
    // one pair under Lilly's moieties, the outer three unpaired, Leo's orbs
    // pairing them, and refusals named in the record
    // (`03-design/western-antiscia.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final george = Observer(
      latitudeDeg: Latitude(51.5045),
      longitudeDeg: Longitude(-0.1366),
      altitudeM: Altitude(0),
    );
    const birth = 2402390.554166667;
    Chart found(
      double instant, {
      AntisciaRequest? asked,
      bool outerPlanets = false,
    }) => ctx.chart.found(
      instant: instant,
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      antiscia: asked,
    );
    expect(found(birth).antiscia, isNull);

    final read =
        found(
          birth,
          asked: const AntisciaRequest(),
          outerPlanets: true,
        ).antiscia!;
    final sun = read.points.firstWhere((at) => at.graha == Graha.sun);
    expect(sun.antiscionDeg, closeTo(107.5685, 0.01));
    expect(read.pairs, hasLength(1));
    final pair = read.pairs.single;
    expect(
      (pair.first, pair.second, pair.contrary),
      (Graha.mars, Graha.mercury, false),
    );
    expect(pair.apartDeg, closeTo(5.935, 0.02));
    expect(read.unpaired, [Graha.uranus, Graha.neptune, Graha.pluto]);
    expect(read.onCusps, isEmpty, reason: 'no cusps unless asked');
    expect(read.cuspSystem, isNull);

    // On the cusps, Lilly's Regiomontanus unless named: his Uranus
    // reflects 0.63° past the fourth cusp, into the next degree.
    final on =
        found(
          birth,
          asked: const AntisciaRequest(cusps: WesternHouseRequest()),
          outerPlanets: true,
        ).antiscia!;
    expect(on.onCusps, isEmpty);
    expect(on.cuspSystem, HouseSystem.regiomontanus);
    final named =
        found(
          birth,
          asked: const AntisciaRequest(
            cusps: WesternHouseRequest(system: HouseSystem.placidus),
          ),
        ).antiscia!;
    expect(named.cuspSystem, HouseSystem.placidus);

    const leo = AntisciaRequest(orbs: OrbModel.leo);
    expect(
      found(birth, asked: leo, outerPlanets: true).antiscia!.unpaired,
      isEmpty,
    );
    final instants = [birth, birth - 3000.25];
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: george,
      utcOffsetSeconds: 0,
      antiscia: leo,
    );
    for (final (k, instant) in instants.indexed) {
      expect(batch.at(k).antiscia, found(instant, asked: leo).antiscia);
    }
    expect(
      () => found(
        birth,
        asked: const AntisciaRequest(
          orbs: OrbModel.byAspect({WesternAspect.trine: 3}),
        ),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'antiscia.orbs.orbs',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its harmonic', () {
    // Churchill's 9th harmonic as Addey reads it (*Harmonics in
    // Astrology*, pp. 97–98): the Moon on Saturn in the third, Venus
    // rising, Pluto in the tenth (`03-design/western-harmonics.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final blenheim = Observer(
      latitudeDeg: Latitude(51.8414),
      longitudeDeg: Longitude(-1.3611),
      altitudeM: Altitude(0),
    );
    const birth = 2405857.564892;
    Chart found(double instant, {HarmonicRequest? asked}) => ctx.chart.found(
      instant: instant,
      place: blenheim,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      harmonic: asked,
    );
    expect(found(birth).harmonic, isNull);

    final ninth = found(birth, asked: const HarmonicRequest(9)).harmonic!;
    expect((ninth.harmonic, ninth.points.length), (9, 12));
    int house(HarmonicPoint point) =>
        ninth.points.firstWhere((one) => one.point == point).house;
    expect(
      [
        for (final graha in [
          Graha.moon,
          Graha.saturn,
          Graha.venus,
          Graha.pluto,
        ])
          house(HarmonicGraha(graha)),
      ],
      [3, 3, 1, 10],
    );
    expect(house(HarmonicPoint.ascendant), 1);
    expect(ninth.points.last.point, HarmonicPoint.midheaven);
    final row = ninth.rows.firstWhere(
      (one) =>
          one.first == const HarmonicGraha(Graha.moon) &&
          one.second == const HarmonicGraha(Graha.saturn),
    );
    expect(row.apartDeg, lessThan(0.6));
    expect((row.multiple, row.orbDeg), (4, 12.0));

    const fifth = HarmonicRequest(5, orbDeg: 3);
    final instants = [birth, birth + 100.5];
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: blenheim,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      harmonic: fifth,
    );
    for (final (k, instant) in instants.indexed) {
      expect(batch.at(k).harmonic, found(instant, asked: fifth).harmonic);
    }
    for (final (asked, field) in [
      (const HarmonicRequest(0), 'harmonic.number'),
      (const HarmonicRequest(9, orbDeg: 31), 'harmonic.orbDeg'),
    ]) {
      expect(
        () => found(birth, asked: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its match with a partner', () {
    // A birth matched with itself: one sign and one nakshatra, so every
    // koota but Nadi takes its whole points and the shared nadi none, 28,
    // whatever the Moon (*Muhurta Chintamani* VI.21–34,
    // `03-design/matching.md`).
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const birth = 2451545.0;
    Chart found(double instant, {MatchingRequest? asked}) => ctx.chart.found(
      instant: instant,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      matching: asked,
    );
    expect(found(birth).matching, isNull);

    final itself =
        found(
          birth,
          asked: MatchingRequest(
            Partner(instant: birth, place: kathmandu, utcOffsetSeconds: 20700),
            partnerRole: MatchRole.bride,
          ),
        ).matching!;
    expect(itself.total, 28);
    expect(
      [for (final row in itself.kootas) row.reading.koota],
      [
        Koota.varna,
        Koota.vashya,
        Koota.tara,
        Koota.yoni,
        Koota.grahaMaitri,
        Koota.gana,
        Koota.bhakoot,
        Koota.nadi,
      ],
    );
    expect(
      [for (final row in itself.kootas) row.maxPoints],
      [1, 2, 3, 4, 5, 6, 7, 8],
    );
    expect(itself.kootas[1].reading, const VashyaKoota(VashyaRelation.mutual));
    expect(
      itself.kootas[2].reading,
      const TaraKoota(brideToGroom: 1, groomToBride: 1),
    );
    final bhakoot = itself.kootas[6].reading as BhakootKoota;
    expect(
      (
        bhakoot.apart,
        bhakoot.dosha,
        bhakoot.lifted,
        bhakoot.exceptions.oneLord,
      ),
      (1, null, false, true),
    );
    final nadi = itself.kootas[7].reading as NadiKoota;
    expect((nadi.dosha, nadi.bride == nadi.groom), (true, true));
    // One star in one pada is the nadi dosha VI.36 does not lift; one gana
    // and one lord leave nothing to lift.
    final gana = itself.kootas[5].reading as GanaKoota;
    final maitri = itself.kootas[4].reading as MaitriKoota;
    expect(
      (nadi.lifted, gana.dosha, gana.lifted, maitri.lifted),
      (false, false, false, false),
    );

    // The ten considerations ride on the same request: one star in one
    // sign shares its Rajju, which the one lord lifts.
    final ten =
        found(
          birth,
          asked: MatchingRequest(
            Partner(instant: birth, place: kathmandu, utcOffsetSeconds: 20700),
            partnerRole: MatchRole.bride,
          ),
        ).porutham!;
    expect(
      [for (final row in ten.considerations) row.reading.koota],
      [
        Koota.tara,
        Koota.gana,
        Koota.mahendra,
        Koota.streeDeergha,
        Koota.yoni,
        Koota.bhakoot,
        Koota.grahaMaitri,
        Koota.vashya,
        Koota.rajju,
        Koota.vedha,
      ],
    );
    final dhinam = ten.considerations.first.reading as DhinamPorutham;
    expect(dhinam.count, 1);
    expect(dhinam.rule.key, startsWith('COMMON_'));
    final rajju = ten.considerations[8];
    final divisions = rajju.reading as RajjuPorutham;
    expect(
      (divisions.bride == divisions.groom, rajju.agrees, rajju.lifted),
      (true, true, true),
    );
    expect((ten.exception.oneLord, ten.exception.opposite), (true, false));
    expect(ten.agreeing, ten.considerations.where((row) => row.agrees).length);
    expect(found(birth).porutham, isNull);

    // The Kuja dosha rides on it too (Manasagari): one birth on both sides
    // reads Mars alike, so both carry it or neither does.
    final mars =
        found(
          birth,
          asked: MatchingRequest(
            Partner(instant: birth, place: kathmandu, utcOffsetSeconds: 20700),
            partnerRole: MatchRole.bride,
            kuja: const KujaRules(from: KujaFrom.lagnaMoonVenus),
          ),
        ).kuja!;
    expect(mars.bride, mars.groom);
    expect([for (final r in mars.bride.readings) r.from], KujaReference.values);
    for (final reading in mars.bride.readings) {
      expect(reading.inHouses, [1, 4, 7, 8, 12].contains(reading.house));
    }
    expect(mars.bride.dosha, mars.bride.readings.any((r) => r.inHouses));
    expect(mars.both, mars.bride.dosha);
    expect(found(birth).kuja, isNull);

    // The marriage doshas gather the three (C289): the shared nadi in one
    // pada first, then each consideration that disagrees or was lifted,
    // then the Kuja sides.
    final self = found(
      birth,
      asked: MatchingRequest(
        Partner(instant: birth, place: kathmandu, utcOffsetSeconds: 20700),
        partnerRole: MatchRole.bride,
      ),
    );
    final doshas = self.marriageDoshas!;
    expect(
      doshas.first,
      const MarriageDosha(
        system: DoshaSystem.ashtaKoota,
        koota: Koota.nadi,
        side: null,
        lifted: false,
      ),
    );
    expect(
      [
        for (final d in doshas)
          if (d.system == DoshaSystem.porutham) (d.koota, d.lifted),
      ],
      [
        for (final row in self.porutham!.considerations)
          if (!row.agrees || row.lifted) (row.reading.koota, row.lifted),
      ],
    );
    expect(
      [
        for (final d in doshas)
          if (d.system == DoshaSystem.kuja) d.side,
      ],
      self.kuja!.bride.dosha
          ? [MatchRole.bride, MatchRole.groom]
          : <MatchRole>[],
    );
    expect(found(birth).marriageDoshas, isNull);

    final asked = MatchingRequest(
      Partner(instant: 2447892.5, place: kathmandu, utcOffsetSeconds: 20700),
      partnerRole: MatchRole.groom,
      rules: const KootaRules(nadiDosha: NadiDosha.middleOnly),
    );
    final instants = [birth, birth + 9.5, birth + 17.25];
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      matching: asked,
    );
    for (final (k, instant) in instants.indexed) {
      final one = found(instant, asked: asked);
      final alone = one.matching!;
      expect(batch.at(k).matching, alone);
      expect(batch.at(k).porutham, one.porutham);
      expect(batch.at(k).kuja, one.kuja);
      expect(batch.at(k).marriageDoshas, one.marriageDoshas);
      final swapped =
          found(
            instant,
            asked: MatchingRequest(asked.partner, partnerRole: MatchRole.bride),
          ).matching!;
      final ours = alone.kootas.first.reading as VarnaKoota;
      final theirs = swapped.kootas.first.reading as VarnaKoota;
      expect((theirs.bride, theirs.groom), (ours.groom, ours.bride));
    }
    // The Nepal knobs reach the koota: Muhurta Martanda's gana (C761),
    // bride row by groom column.
    const martanda = [
      [6.0, 5.0, 1.0],
      [6.0, 6.0, 0.0],
      [0.0, 0.0, 6.0],
    ];
    final nepal = MatchingRequest(
      asked.partner,
      partnerRole: MatchRole.groom,
      rules: const KootaRules(
        gana: GanaPoints.muhurtaMartanda,
        vashya: VashyaPoints.printedInNepal,
      ),
    );
    for (final instant in instants) {
      final row = found(instant, asked: nepal).matching!.kootas[5];
      final gana = row.reading as GanaKoota;
      expect(row.points, martanda[gana.bride.index][gana.groom.index]);
    }
    ctx.dispose();

    final western = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    expect(
      () => western.chart.found(
        instant: birth,
        place: kathmandu,
        utcOffsetSeconds: 20700,
        matching: asked,
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'matching.partner',
        ),
      ),
    );
    western.dispose();
  });

  test('a chart carries its Western houses', () {
    // Leo's own illustration (*How to Judge a Nativity*, p. 150), "a female
    // born at 2.42 A.M. 13th December, 1835, London", against the SDK
    // test's Moshier recast: Placidus, Saturn rising
    // (`03-design/western-houses.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(-0.1),
      altitudeM: Altitude(0),
    );
    const birth = 2391625.6125;
    bool near(double a, double b) => ((a - b + 540) % 360 - 180).abs() < 0.01;
    Chart found(double instant, {WesternHouseRequest? asked}) =>
        ctx.chart.found(
          instant: instant,
          place: london,
          utcOffsetSeconds: 0,
          outerPlanets: true,
          westernHouses: asked,
        );
    expect(found(birth).westernHouses, isNull);

    final houses =
        found(birth, asked: const WesternHouseRequest()).westernHouses!;
    expect(houses.system, HouseSystem.placidus);
    expect(houses.cuspsDeg, hasLength(12));
    expect(
      near(houses.cuspsDeg[0], 202.1436),
      isTrue,
      reason: '${houses.cuspsDeg}',
    );
    expect(
      near(houses.cuspsDeg[9], 119.3147),
      isTrue,
      reason: '${houses.cuspsDeg}',
    );
    expect(
      near(houses.reachDeg, 191.6089),
      isTrue,
      reason: '${houses.reachDeg}',
    );
    WesternHousePlacement placed(Graha graha) =>
        houses.planets.firstWhere((one) => one.graha == graha);
    expect(
      (placed(Graha.saturn).house, placed(Graha.saturn).withAscendant),
      (1, true),
    );
    expect(
      (placed(Graha.sun).house, placed(Graha.sun).withAscendant),
      (2, false),
    );
    expect(placed(Graha.mars).house, 3);

    const koch = WesternHouseRequest(system: HouseSystem.koch);
    expect(found(birth, asked: koch).westernHouses!.system, HouseSystem.koch);
    final instants = [birth, birth + 100.5];
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: london,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      westernHouses: const WesternHouseRequest(),
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).westernHouses,
        found(instant, asked: const WesternHouseRequest()).westernHouses,
      );
    }
    ctx.dispose();
  });

  test('two names match star to star without a chart', () {
    // प्रि is Uttara Phalguni's 4th syllable and कृ Mrigashira's 4th
    // (Svarodaya vv. 3-8, the first consonant taking the cluster's vowel);
    // pa and ka stand 4 vargas apart, eater and eaten (VI.35).
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final read = ctx.matching.naam('प्रिया', 'कृष्ण');
    expect(
      (read.bride.nakshatra, read.bride.quarter, read.bride.varga),
      (Nakshatra.uttaraPhalguni, 4, NameVarga.rat),
    );
    expect(
      (read.groom.nakshatra, read.groom.quarter, read.groom.varga),
      (Nakshatra.mrigashira, 4, NameVarga.cat),
    );
    expect(
      read.varga,
      const VargaKoota(
        bride: NameVarga.rat,
        groom: NameVarga.cat,
        relation: VargaRelation.enemy,
      ),
    );
    expect(read.ashta.kootas, hasLength(8));
    expect(
      read.ashta.total,
      read.ashta.kootas.fold<double>(0, (sum, row) => sum + row.points),
    );
    expect(read.porutham.considerations, hasLength(10));
    // The ten considerations count the groom's star from the bride's.
    expect(
      (read.porutham.considerations.first.reading as DhinamPorutham).count,
      21,
    );

    // IAST, when declared, is the same name.
    expect(
      ctx.matching.naam(
        'priyā',
        'kṛṣṇa',
        const NaamRules(name: NameRules(latin: LatinName.iast)),
      ),
      read,
    );

    // Abhijit's row is none of the 27: refused, unless the rules place it.
    final placed = ctx.matching.naam(
      'सीता',
      'ज़ोया',
      const NaamRules(name: NameRules(abhijit: AbhijitPada.shravana)),
    );
    expect((placed.groom.nakshatra, placed.groom.quarter), (null, 3));
    for (final (run, field) in [
      (() => ctx.matching.naam('सीता', 'ज़ोया'), 'naam.groom.abhijit'),
      (() => ctx.matching.naam('Sita', 'राम'), 'naam.bride.name'),
    ]) {
      expect(
        run,
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a child named by the cakra, and a name book suggested from', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final makara = ctx.naming.syllables(
      const SyllablesOfRashi(rashi: Rashi.capricorn),
    );
    expect(makara, hasLength(13));
    expect(makara.where((one) => one.pada == null), hasLength(4));
    const ardra = Birth(nakshatra: Nakshatra.ardra, pada: 3);
    final read = ctx.naming.read(['राम', 'ऐश्वर्य']);
    expect(read.first.reading?.readings, hasLength(1));
    expect(read.last.refused?.field, 'naming.names[1]');
    expect(
      ctx.naming.check(['चूडामणि'], ardra).single.check?.samePada,
      isFalse,
    );
    expect(
      ctx.naming.of(const Birth(nakshatra: Nakshatra.krittika, pada: 1)).deity,
      Deity.agni,
    );
    const book = Book(
      names: [
        BookName(devanagari: 'गणेश', sex: Sex.male),
        BookName(devanagari: 'गीता', sex: Sex.female),
      ],
    );
    final loaded = ctx.naming.load(book);
    expect(loaded.book, matches(RegExp(r'^[0-9a-f]{64}$')));
    expect(loaded.names, 2);
    expect(loaded.refused, isEmpty);
    const girls = SuggestFilter(sex: Sex.female);
    final byId = ctx.naming.suggest(loaded.book, ardra, girls);
    expect(
      [for (final one in byId.names) (one.index, one.by)],
      [(1, InitialBy.substitute)],
    );
    expect(ctx.naming.suggest(book, ardra, girls), byId);
    String? field(void Function() call) {
      try {
        call();
      } on TeistroException catch (error) {
        return error.field;
      }
      return null;
    }

    expect(field(() => ctx.naming.suggest('0' * 64, ardra)), 'naming.bookId');
    expect(
      field(
        () =>
            ctx.naming.of(const Birth(nakshatra: Nakshatra.krittika, pada: 5)),
      ),
      'naming.birth.pada',
    );
    expect(
      field(
        () => ctx.naming.load(const Book(names: [BookName(devanagari: 'Ram')])),
      ),
      'naming.book.names[0].devanagari',
    );
    expect(() => ctx.naming.suggest(3, ardra), throwsArgumentError);
    ctx.dispose();
  });

  test(
    'a stored baseline settings snapshot translates into the contexts it opens',
    () {
      final ctx = context();
      final translated = ctx.recordedSettings({
        'ayanamsaId': 3,
        'houseSystem': 'placidus',
        'engineVersion': '1.8.0',
      });
      final natal = translated['natal']! as Map<String, Object?>;
      final kp = translated['kp']! as Map<String, Object?>;
      expect(
        (natal['profile'], kp['profile']),
        ('conformance-baseline', 'kp-default'),
      );
      final settings = natal['settings']! as Map<String, Object?>;
      expect(
        (settings['houses']! as Map<String, Object?>)['placement_system'],
        'PLACIDUS',
      );
      expect((translated['ephemeris'], translated['locale']), (null, null));
      expect(
        (translated['notes']! as List<Object?>).map(
          (note) => (note! as Map<String, Object?>)['field'],
        ),
        ['engineVersion'],
      );
      expect(
        ctx.recordedSettings({'siddhanta': 'surya'})['ephemeris'],
        'SURYA_SIDDHANTA',
      );
      // The translation opens a context as it is.
      final opened = context(
        profile: natal['profile']! as String,
        settings: settings,
      );
      expect(opened.profile, 'conformance-baseline');
      opened.dispose();
      expect(
        () => ctx.recordedSettings({'suryaBija': true}),
        throwsA(
          isA<TeistroException>()
              .having((e) => e.status, 'status', Status.unsupported)
              .having((e) => e.field, 'field', 'snapshot.suryaBija')
              .having((e) => e.hint, 'hint', 'C777'),
        ),
      );
      expect(
        () => ctx.recordedSettings({'nodeType': 'osculating'}),
        throwsA(
          isA<TeistroException>().having(
            (e) => e.field,
            'field',
            'snapshot.nodeType',
          ),
        ),
      );
      ctx.dispose();
    },
  );

  test('a birth star reads into its navatara chakra without a sky', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final chakra = ctx.almanac.navatara(Nakshatra.ashwini);
    expect(chakra, hasLength(27));
    // Raman, Muhurtha ch. III: from Ashwini, Shravana is the 22nd, Kshema.
    expect(
      chakra.firstWhere((row) => row.nakshatra == Nakshatra.shravana),
      const NavataraRow(
        nakshatra: Nakshatra.shravana,
        count: 22,
        tara: Tara.kshema,
        cycle: 3,
        special: null,
      ),
    );
    // Muhurta Chintamani, Nakshatra-prakarana v. 43: the special stars.
    expect(
      [
        for (final row in chakra)
          if (row.special case final star?)
            '${row.count} ${row.nakshatra.key} ${star.key}',
      ],
      [
        '1 ASHWINI JANMA',
        '10 MAGHA KARMA',
        '16 VISHAKHA SANGHATIKA',
        '18 JYESHTHA SAMUDAYA',
        '23 DHANISHTHA VAINASHIKA',
        '25 PURVA_BHADRAPADA MANASA',
      ],
    );
    expect(
      () => ctx.almanac.navatara(Nakshatra.unknown),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'navatara.birth',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a name and a birth date read under numerology without a chart', () {
    // Balliett pp. 18-19: Henry 34/7, Elder 26/8, the name 15/6; born
    // 17 January 1872, 1 + 8 + 9 = 18 = 9. Cheiro p. 70: Henry 18/9 and
    // Elder 19/1, their singles 10.
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final read = ctx.numerology.profile(
      'Henry Elder',
      const BirthDate(1872, 1, 17),
    );
    final name = read.pythagoreanName;
    expect(
      [
        for (final word in name.words)
          (word.text, word.total, word.reduction.number),
      ],
      [('Henry', 34, 7), ('Elder', 26, 8)],
    );
    expect(name.words.first.letters.first, const NumerologyLetter('H', 8));
    expect(
      (name.system, name.compound, name.reduction),
      (
        NumerologySystem.pythagorean,
        null,
        const Reduction(steps: [15, 6], number: 6),
      ),
    );
    expect(
      read.pythagoreanBirth.sum,
      const Reduction(steps: [18, 9], number: 9),
    );
    expect(read.pythagoreanBirth.apart, isEmpty);
    expect(
      (read.chaldeanName.system, read.chaldeanName.compound),
      (NumerologySystem.chaldean, 10),
    );
    expect(read.chaldeanBirth.birth.number, 8);
    expect(read.baseline, isNull);
    expect(
      ctx.numerology
          .profile(
            'Henry Elder',
            const BirthDate(1872, 1, 17),
            const NumerologyRules.baseline(),
          )
          .baseline,
      isNotNull,
    );
    // The SDK's own baseline, by name, is the layer's spelt out.
    expect(
      ctx.numerology.profile(
        'Henry Elder',
        const BirthDate(1872, 1, 17),
        const NumerologyRequestRulesBaseline(),
      ),
      ctx.numerology.profile(
        'Henry Elder',
        const BirthDate(1872, 1, 17),
        const NumerologyRules.baseline(),
      ),
    );
    expect(
      () => ctx.numerology.profile(
        'Henry Elder',
        const BirthDate(1872, 1, 17),
        const NumerologyRequestRulesValue(
          NumerologyRules(masters: Masters.none),
        ),
      ),
      returnsNormally,
    );

    // Balliett p. 90: born 11 July 1838 is "9, 11", the day apart.
    final wanamaker =
        ctx.numerology
            .profile('John Wanamaker', const BirthDate(1838, 7, 11))
            .pythagoreanBirth;
    expect(wanamaker.sum?.number, 9);
    expect(wanamaker.apart, [11]);

    for (final (run, field) in [
      (
        () => ctx.numerology.profile('Kṛṣṇa', const BirthDate(1990, 1, 1)),
        'numerology.name',
      ),
      (
        () => ctx.numerology.profile('Rama', const BirthDate(1900, 2, 29)),
        'numerology.date',
      ),
    ]) {
      expect(
        run,
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test(
    'a chart carries its avakahada, whose syllable names the birth pada',
    () {
      // Read as a name, in either script, the birth syllable is the birth
      // pada (C297); null unless asked; a tropical chart refuses it by name.
      final ctx = teistro.context(
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      final kathmandu = Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      );
      expect(
        ctx.chart
            .found(
              instant: 2451545.0,
              place: kathmandu,
              utcOffsetSeconds: 20700,
            )
            .avakahada,
        isNull,
      );
      final padas = <(Nakshatra, int)>{};
      for (var step = 0; step < 28; step++) {
        final read =
            ctx.chart
                .found(
                  instant: 2451545.0 + step * 0.83,
                  place: kathmandu,
                  utcOffsetSeconds: 20700,
                  avakahada: true,
                )
                .avakahada!;
        final named =
            ctx.matching
                .naam(read.syllable.devanagari, read.syllable.devanagari)
                .bride;
        expect(
          (named.cell, named.nakshatra, named.quarter, named.varga),
          (read.syllable.cell, read.nakshatra, read.pada, read.syllable.varga),
          reason: read.syllable.devanagari,
        );
        expect(
          ctx.matching
              .naam(
                read.syllable.iast,
                read.syllable.iast,
                const NaamRules(name: NameRules(latin: LatinName.iast)),
              )
              .bride,
          named,
          reason: read.syllable.iast,
        );
        padas.add((read.nakshatra, read.pada));
        // The ghata Moon is Muhurta Chintamani's count from the Moon's own
        // sign (C758).
        const counts = [1, 5, 9, 2, 6, 10, 3, 7, 4, 8, 11, 12];
        final at = read.rashi.id;
        expect(read.ghata.chandra, Rashi.byId((at + counts[at] - 1) % 12));
      }
      expect(padas.length, greaterThan(20));
      ctx.dispose();

      final western = teistro.context(
        profile: 'western-tropical-default',
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      expect(
        () => western.chart.found(
          instant: 2451545.0,
          place: kathmandu,
          utcOffsetSeconds: 20700,
          avakahada: true,
        ),
        throwsA(
          isA<TeistroException>().having((e) => e.field, 'field', 'avakahada'),
        ),
      );
      western.dispose();
    },
  );

  test('a chart carries its moment, the eclipse its place sees', () {
    // The rising sign's edges bound the instant, and the lunar eclipse of
    // 2025 March 14, seen from New York, is the instant's and not a day
    // later's (C378); null unless asked.
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final newYork = Observer(
      latitudeDeg: Latitude(40.71),
      longitudeDeg: Longitude(-74.0),
      altitudeM: Altitude(0),
    );
    const eclipse = 2460748.7908;
    expect(
      ctx.chart
          .found(instant: eclipse, place: newYork, utcOffsetSeconds: 0)
          .moment,
      isNull,
    );
    final batch = ctx.chart.foundMany(
      instants: [eclipse, eclipse + 1],
      place: newYork,
      utcOffsetSeconds: 0,
      moment: true,
    );
    final during = batch.at(0).moment!;
    final after = batch.at(1).moment!;
    for (final (moment, instant) in [(during, eclipse), (after, eclipse + 1)]) {
      expect(moment.rising.rose!, lessThanOrEqualTo(instant));
      expect(moment.rising.sets!, greaterThan(instant));
    }
    expect(during.eclipse.kind, BirthEclipse.lunar);
    expect(during.eclipse.greatest!, closeTo(eclipse, 0.1));
    expect(
      after.eclipse,
      const MomentEclipse(kind: BirthEclipse.none, greatest: null),
    );
    ctx.dispose();
  });

  test('a chart carries its synastry with a partner', () {
    // King George V and Queen Mary (Leo, *How to Judge a Nativity*,
    // p. 130): the recast's closest contacts, the lagna left out on
    // request, a batch the charts one at a time, and refusals named in the
    // record (`03-design/western-synastry.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final george = Observer(
      latitudeDeg: Latitude(51.5045),
      longitudeDeg: Longitude(-0.1366),
      altitudeM: Altitude(0),
    );
    const birth = 2402390.554166667;
    final mary = Partner(
      instant: 2403113.499305556,
      place: Observer(
        latitudeDeg: Latitude(51.5058),
        longitudeDeg: Longitude(-0.1878),
        altitudeM: Altitude(0),
      ),
    );
    Chart found(
      double instant, {
      SynastryRequest? asked,
      bool outerPlanets = false,
    }) => ctx.chart.found(
      instant: instant,
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      synastry: asked,
    );
    expect(found(birth).synastry, isNull);

    final rows =
        found(
          birth,
          asked: SynastryRequest(mary),
          outerPlanets: true,
        ).synastry!;
    const mars = NatalGraha(Graha.mars);
    for (final (first, aspect, second, fromExactDeg) in [
      (mars, WesternAspect.opposition, const NatalLagna(), 0.32),
      (mars, WesternAspect.sextile, const NatalGraha(Graha.sun), 0.39),
      (
        const NatalGraha(Graha.pluto),
        WesternAspect.conjunction,
        const NatalGraha(Graha.pluto),
        1.69,
      ),
    ]) {
      final row = rows.firstWhere(
        (row) =>
            row.first == first && row.aspect == aspect && row.second == second,
      );
      expect(row.fromExactDeg, closeTo(fromExactDeg, 0.01));
    }
    expect(rows.every((row) => row.fromExactDeg <= row.orbDeg), isTrue);

    final without =
        found(birth, asked: SynastryRequest(mary, lagna: false)).synastry!;
    expect(
      without.every(
        (row) => row.first is NatalGraha && row.second is NatalGraha,
      ),
      isTrue,
    );

    final instants = [birth, birth - 3000.25];
    final two = SynastryRequest(
      mary,
      table: const WesternAspectRequest(
        aspects: [WesternAspect.sextile, WesternAspect.opposition],
      ),
    );
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: george,
      utcOffsetSeconds: 0,
      synastry: two,
    );
    for (final (k, instant) in instants.indexed) {
      expect(batch.at(k).synastry, found(instant, asked: two).synastry);
    }

    // The parallels across: none unless asked, then the recast's closest
    // (Uranus with Uranus, 0.05°) and a contrary pair within 0.95°.
    expect(
      found(birth, asked: SynastryRequest(mary)).synastryParallels,
      isNull,
    );
    final parallels =
        found(
          birth,
          asked: SynastryRequest(mary, parallels: const ParallelRequest()),
          outerPlanets: true,
        ).synastryParallels!;
    const uranus = NatalGraha(Graha.uranus);
    expect((parallels.first.first, parallels.first.second), (uranus, uranus));
    expect(parallels.first.apartDeg, closeTo(0.049, 0.005));
    expect(parallels.any((row) => row.contrary && row.apartDeg < 0.95), isTrue);
    expect(
      parallels.every((row) => row.apartDeg <= row.orbDeg && row.orbDeg == 1),
      isTrue,
    );
    expect(
      found(
        birth,
        asked: SynastryRequest(
          mary,
          parallels: const ParallelRequest(orbDeg: 0.000001),
        ),
      ).synastryParallels,
      isEmpty,
    );

    // The antiscia across: none unless asked, then the recast's seven under
    // Lilly's moieties, closest first (Saturn's antiscion on Jupiter, 0.09°).
    expect(found(birth, asked: SynastryRequest(mary)).synastryAntiscia, isNull);
    final reflected =
        found(
          birth,
          asked: SynastryRequest(mary, antiscia: const AntisciaRequest()),
        ).synastryAntiscia!;
    expect(
      [for (final row in reflected) (row.first, row.second, row.contrary)],
      [
        (Graha.saturn, Graha.jupiter, false),
        (Graha.saturn, Graha.moon, false),
        (Graha.mercury, Graha.mars, false),
        (Graha.mars, Graha.saturn, true),
        (Graha.venus, Graha.mars, false),
        (Graha.mars, Graha.mercury, false),
        (Graha.mars, Graha.sun, false),
      ],
    );
    expect(reflected.first.apartDeg, closeTo(0.089, 0.005));
    final far = Partner(instant: 9000000, place: mary.place);
    for (final (asked, field) in [
      (
        SynastryRequest(mary, table: WesternAspectRequest.lilly),
        'synastry.lagna',
      ),
      (SynastryRequest(far), 'synastry.partner'),
      (
        SynastryRequest(mary, parallels: const ParallelRequest(orbDeg: 11)),
        'synastry.parallels.orbDeg',
      ),
      (
        SynastryRequest(
          mary,
          antiscia: const AntisciaRequest(
            orbs: OrbModel.byAspect({WesternAspect.trine: 3}),
          ),
        ),
        'synastry.antiscia.orbs.orbs',
      ),
      (
        SynastryRequest(
          mary,
          antiscia: const AntisciaRequest(cusps: WesternHouseRequest()),
        ),
        'synastry.antiscia.cusps',
      ),
    ]) {
      expect(
        () => found(birth, asked: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    // Lilly's own reading leaves the lagna out, and the outer three need
    // leaving out too.
    expect(
      found(birth, asked: SynastryRequest.lilly(mary)).synastry,
      isNotEmpty,
    );
    ctx.dispose();
  });

  test(
    'a synastry reads the equal distances and makes the composite and Davison birth',
    () {
      // King George V and Queen Mary, against the SDK test's Moshier recast
      // (`03-design/western-composites.md`).
      final ctx = teistro.context(
        profile: 'western-tropical-default',
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      final george = Observer(
        latitudeDeg: Latitude(51.5045),
        longitudeDeg: Longitude(-0.1366),
        altitudeM: Altitude(0),
      );
      const birth = 2402390.554166667;
      final mary = Partner(
        instant: 2403113.499305556,
        place: Observer(
          latitudeDeg: Latitude(51.5058),
          longitudeDeg: Longitude(-0.1878),
          altitudeM: Altitude(0),
        ),
      );
      Chart found(SynastryRequest asked) => ctx.chart.found(
        instant: birth,
        place: george,
        utcOffsetSeconds: 0,
        outerPlanets: true,
        synastry: asked,
      );
      bool near(double a, double b) => ((a - b + 540) % 360 - 180).abs() < 0.01;

      final plain = found(SynastryRequest(mary));
      expect(plain.synastryComposite, isNull);
      expect(plain.synastryDavison, isNull);

      // The equal distances across: none unless asked, then the SDK test's
      // recast within 1°, her Venus on his Sun and Neptune the closest.
      expect(plain.synastryMidpoints, isNull);
      final equal =
          found(
            SynastryRequest(mary, midpoints: const MidpointRequest(orbDeg: 1)),
          ).synastryMidpoints!;
      expect(equal, hasLength(10));
      final closest = equal.first;
      expect(
        (
          closest.first,
          closest.second,
          closest.middle,
          closest.partnersPair,
          closest.far,
        ),
        (Graha.sun, Graha.neptune, Graha.venus, true, false),
      );
      expect(closest.fromAxisDeg, closeTo(0.14, 0.01));

      final chart = found(
        SynastryRequest(mary, composite: true, davison: true),
      );
      final composite = chart.synastryComposite!;
      expect(composite.planets, hasLength(10));
      for (final (graha, longitude) in [
        (Graha.sun, 68.8193),
        (Graha.moon, 259.7289),
        (Graha.mars, 130.5171),
        (Graha.pluto, 44.2555),
      ]) {
        final at = composite.planets.firstWhere((one) => one.graha == graha);
        expect(
          near(at.longitudeDeg, longitude),
          isTrue,
          reason: '$graha ${at.longitudeDeg}',
        );
      }
      expect(
        near(composite.midheavenDeg, 258.1797),
        isTrue,
        reason: '${composite.midheavenDeg}',
      );
      expect(
        near(composite.lagnaDeg, 334.007),
        isTrue,
        reason: '${composite.lagnaDeg}',
      );
      expect(composite.lagnaTurned, isFalse);
      // Its Placidus cusps, the near midpoints of the two charts', the first
      // and tenth its lagna and midheaven.
      final cusps = composite.cuspsDeg!;
      expect(cusps, hasLength(12));
      expect(near(cusps[2], 57.8601), isTrue, reason: '${cusps[2]}');
      expect(
        (cusps[0], cusps[9]),
        (composite.lagnaDeg, composite.midheavenDeg),
      );

      // The Davison birth is a Partner, so it founds a chart as a birth does.
      final davison = chart.synastryDavison!;
      expect(davison.instant, closeTo(2402752.026736111, 1e-8));
      expect(davison.place.longitudeDeg, closeTo(-0.1622, 1e-9));
      expect(davison.utcOffsetSeconds, 0);
      final between = ctx.chart.found(
        instant: davison.instant,
        place: davison.place,
        utcOffsetSeconds: davison.utcOffsetSeconds,
      );
      final mars = between.grahas.firstWhere((one) => one.graha == Graha.mars);
      expect(
        near(mars.tropicalDeg, 20.1267),
        isTrue,
        reason: '${mars.tropicalDeg}',
      );
      ctx.dispose();
    },
  );

  test('a chart carries the outer planets when asked', () {
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(0),
      altitudeM: Altitude(0),
    );
    const birth = 2400629.742361111;
    Chart found({
      bool outerPlanets = false,
      ProgressionsRequest? progressions,
    }) => ctx.chart.found(
      instant: birth,
      place: london,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      progressions: progressions,
    );
    final bare = found();
    expect(bare.outer, isEmpty);
    final asked = found(outerPlanets: true);
    expect(asked.outer.map((at) => at.graha), [
      Graha.uranus,
      Graha.neptune,
      Graha.pluto,
    ]);
    expect(
      asked.grahas.map((at) => at.longitudeDeg),
      bare.grahas.map((at) => at.longitudeDeg),
      reason: 'the nine are unmoved',
    );
    for (final at in asked.outer) {
      expect(at.distanceAu, greaterThan(15));
      expect(at.house.bhava, inInclusiveRange(1, 12));
    }

    const at = birth + 46 * 365.242189;
    final later =
        found(
          outerPlanets: true,
          progressions: const ProgressionsRequest(at: at),
        ).progressions!;
    expect(later.progressed!.grahas.length, 12);
    expect(later.directed!.planets.length, 12);
    expect(later.progressed!.grahas[11].graha, Graha.pluto);

    // Leo's progressed Moon quincunx Uranus, April 1907 (p. 41).
    const contacts = ProgressionContacts(
      from: 2417484.5,
      to: 2417941.5,
      grahas: [Graha.moon],
      points: [NatalGraha(Graha.uranus)],
    );
    final reached =
        found(
          outerPlanets: true,
          progressions: const ProgressionsRequest(contacts: contacts),
        ).progressions!.contacts!;
    expect(reached, hasLength(1));
    expect(reached.single.to, const NatalGraha(Graha.uranus));
    expect(reached.single.angle, 150);
    expect(reached.single.life, inInclusiveRange(2417635.5, 2417696.4));
    expect(
      () => found(progressions: const ProgressionsRequest(contacts: contacts)),
      throwsA(isA<TeistroException>()),
    );
    ctx.dispose();
  });

  test('a chart carries its accidental fortitudes', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    const instants = [2460676.5, 2460676.75];
    Chart found(
      double instant, {
      FortitudeRequest? fortitudes,
      DignityRequest? dignities,
    }) => ctx.chart.found(
      instant: instant,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      fortitudes: fortitudes,
      dignities: dignities,
    );
    expect(found(instants[0]).fortitudes, isNull);

    final chart = found(instants[0], fortitudes: const FortitudeRequest());
    final read = chart.fortitudes!;
    expect(read.dignities, chart.dignities);
    expect(read.sky.houses, HouseSystem.regiomontanus);
    expect(read.sky.cuspsDeg, hasLength(12));
    expect(read.sky.speedsDegPerDay, hasLength(7));
    expect(read.rules, AccidentalRules.lilly);
    expect(read.scores, AccidentalScores.lilly);
    expect(
      read.planets.map((at) => at.planet),
      read.dignities.planets.map((at) => at.planet),
    );
    const solar = {
      Accident.cazimi,
      Accident.combust,
      Accident.underBeams,
      Accident.freeFromCombustion,
    };
    for (final (k, at) in read.planets.indexed) {
      final own = read.dignities.planets[k];
      final points = [
        read.scores.houses[at.house - 1],
        for (final line in at.accidents) line.points,
      ];
      expect(
        at.fortitude,
        points.where((n) => n > 0).fold<int>(0, (a, b) => a + b),
        reason: '${at.planet}',
      );
      expect(
        at.debility,
        points.where((n) => n < 0).fold<int>(0, (a, b) => a - b),
        reason: '${at.planet}',
      );
      expect(
        at.net,
        own.score + own.reception + at.fortitude - at.debility,
        reason: '${at.planet}',
      );
      expect(
        at.accidents.where((line) => solar.contains(line.accident)),
        hasLength(at.planet == Graha.sun ? 0 : 1),
        reason: '${at.planet}',
      );
    }

    // The almutens: Lilly's of the figure is the greatest net, Fortune the
    // ascendant plus the Moon less the Sun, and every house has one.
    final almutens = read.almutens;
    expect(almutens.rules, AlmutenRules.lilly);
    expect(
      [for (final at in almutens.figure.totals) at.total],
      [for (final at in read.planets) at.net],
    );
    final greatest = read.planets
        .map((at) => at.net)
        .reduce((a, b) => a > b ? a : b);
    expect(almutens.figure.almutens, [
      for (final at in read.planets)
        if (at.net == greatest) at.planet,
    ]);
    double longitude(Graha planet) =>
        read.dignities.planets
            .firstWhere((at) => at.planet == planet)
            .longitudeDeg;
    final fortune =
        (read.sky.ascendantDeg + longitude(Graha.moon) - longitude(Graha.sun)) %
        360;
    expect(almutens.fortuneDeg, closeTo(fortune, 1e-9));
    expect(almutens.houses, hasLength(12));
    for (final almuten in [
      almutens.figure,
      almutens.places,
      ...almutens.houses,
    ]) {
      expect(almuten.almutens, isNotEmpty);
      expect(almuten.partakers.where(almuten.almutens.contains), isEmpty);
    }

    // The answer's rules and scores are a request as they stand, and one
    // changed is obeyed.
    expect(
      found(
        instants[0],
        fortitudes: FortitudeRequest(
          rules: read.rules,
          scores: read.scores,
          almuten: almutens.rules,
        ),
      ).fortitudes,
      read,
    );
    const asked = FortitudeRequest(
      rules: AccidentalRules(
        beamsDeg: 15,
        partile: Partile.within,
        partileOrbDeg: 1,
        siege: Siege.within,
        siegeSpanDeg: 30,
      ),
      scores: AccidentalScores(regulus: 5),
      almuten: AlmutenRules(
        place: PlaceReading.sign,
        fortune: FortuneRule.reversedByNight,
      ),
    );
    final other = found(instants[0], fortitudes: asked).fortitudes!;
    expect(other.rules, asked.rules);
    expect(other.scores, asked.scores);
    expect(other.almutens.rules, asked.almuten);

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      fortitudes: const FortitudeRequest(),
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).fortitudes,
        found(instant, fortitudes: const FortitudeRequest()).fortitudes,
        reason: 'each alone',
      );
    }

    for (final (fortitudes, dignities, field) in [
      (
        const FortitudeRequest(rules: AccidentalRules(beamsDeg: -1)),
        null,
        'fortitudes.rules.beamsDeg',
      ),
      (const FortitudeRequest(), const DignityRequest(), 'dignities'),
    ]) {
      expect(
        () => found(instants[0], fortitudes: fortitudes, dignities: dignities),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its prashna', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    Prashna? found(PrashnaRequest? asked) =>
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              prashna: asked,
            )
            .prashna;
    expect(found(null), isNull);

    final plain = found(const PrashnaRequest())!;
    expect(plain.links, isNull, reason: 'no house, no links');
    expect(plain.score, isNull, reason: 'no points unless asked');
    expect(plain.rules, const PrashnaRules(), reason: 'the texts\' own');
    expect(plain.mook.person, isNotNull);

    // Every reading the baseline engine takes, by name, answers as the
    // rules it echoes asked back spelt out.
    final byName =
        found(
          const PrashnaRequest(house: 7, rules: PrashnaRequestRulesBaseline()),
        )!;
    expect(byName.rules.timing, TimingRule.baseline);
    final spelled = found(PrashnaRequest(house: 7, rules: byName.rules))!;
    expect(spelled.rules, byName.rules);
    expect(spelled.verdict.outcome, byName.verdict.outcome);
    expect(spelled.timing.graha, byName.timing.graha);
    expect(spelled.score?.points, isNotNull);
    expect(spelled.score?.points, byName.score?.points);

    final read =
        found(
          const PrashnaRequest(
            house: 7,
            number: 14,
            rules: PrashnaRules(
              score: ScoreRule.baseline,
              mook: MookRule.baseline,
              graded: GradedRule.parashari,
            ),
          ),
        )!;
    expect(read.numberSign, Rashi.taurus);
    expect(read.mook.person, isNull, reason: 'a person only the verse names');
    final links = read.links!;
    expect(links.house, 7);
    expect(links.sameLord, links.between == null);
    expect(links.states, isNotNull);
    expect(links.holds(YearYoga.ithasala), isNotNull);
    final score = read.score!;
    expect(
      score.factors.fold<int>(0, (sum, factor) => sum + factor.points),
      score.points,
      reason: 'the points are the factors added',
    );

    expect(plain.link, isNull, reason: 'no house, no link');
    expect(
      plain.rules.offices,
      const OfficeRules(),
      reason: 'the gloss\'s own',
    );
    final link = read.link!;
    expect(PrashnaMeaning.values, contains(link.meaning));
    expect(
      link.kamboola == null,
      isNot(read.links!.holds(YearYoga.kamboola) ?? false),
    );
    expect(read.triKala.lagnesha, isNot(Graha.unknown));
    expect(plain.graded, isNull, reason: 'off by default (C373)');
    final graded = read.graded!;
    expect(graded.rule, GradedRule.parashari);
    expect([
      ...graded.onLagna,
      ...graded.onLagnesha,
      ...graded.ill,
    ], everyElement(isNot(Graha.unknown)));
    final past =
        found(
          const PrashnaRequest(
            house: 7,
            tense: PrashnaTense.past,
            rules: PrashnaRules(offices: OfficeRules(hudda: HuddaTable.charak)),
          ),
        )!;
    expect(past.rules.offices.hudda, HuddaTable.charak);
    if (past.link!.kind == PrashnaLinkKind.separated) {
      expect(past.link!.meaning, PrashnaMeaning.past);
    }
    expect(past.link!.meaning, isNot(PrashnaMeaning.failsOrPast));

    for (final (bad, field) in [
      (const PrashnaRequest(house: 13), 'prashna.question.house'),
      (const PrashnaRequest(number: 109), 'prashna.question.number'),
    ]) {
      expect(
        () => found(bad),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('an almanac reads a native\'s bird over its days', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final madras = Observer(
      latitudeDeg: Latitude(13.0827),
      longitudeDeg: Longitude(80.2707),
      altitudeM: Altitude(6),
    );
    final days = ctx.almanac.pakshi(
      from: gregorian(1984, 10, 30),
      to: gregorian(1984, 10, 31),
      place: madras,
      utcOffsetSeconds: 19800,
      native: const PakshiNative.star(Nakshatra.uttaraAshadha, Paksha.shukla),
    );
    expect(
      days.provenance.inputHash,
      isNotEmpty,
      reason: 'the request is sealed',
    );
    expect(days.value, hasLength(2));
    final tuesday = days.value[0].reading!;
    final wednesday = days.value[1].reading!;
    expect(tuesday.day.nextSunrise, wednesday.day.sunrise);
    expect(
      (wednesday.day.vara, wednesday.day.paksha),
      (Vara.budhavara, Paksha.shukla),
    );
    expect(wednesday.bird, PakshiBird.cock);
    // PUL p. vii: the cock sleeps in the day's second yama and dies from
    // the third.
    expect(
      [for (final y in wednesday.yamas.sublist(1, 3)) y.activity],
      [PakshiActivity.sleeping, PakshiActivity.dying],
    );
    expect(wednesday.yamas.first.subs.first.ownerIs, PakshiRelation.own);
    expect(days.value[1].date.day, 31);

    final polar = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      settings: {
        'day': {'polar_day_policy': 'NEAREST_EVENT'},
      },
    );
    final midsummer = polar.almanac.pakshi(
      from: gregorian(2024, 6, 21),
      place: Observer(
        latitudeDeg: Latitude(69.6492),
        longitudeDeg: Longitude(18.9553),
        altitudeM: Altitude(0),
      ),
      utcOffsetSeconds: 7200,
      native: const PakshiNative.bird(PakshiBird.owl),
    );
    expect(
      midsummer.value.single.reading,
      isNull,
      reason: 'no sunset, no yamas',
    );
    polar.dispose();
    ctx.dispose();
  });

  test('a study counts and tests its rules over a batch of births', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    final births = [
      for (var i = 0; i < 6; i++)
        ResearchBirth(
          instant: 2447000.25 + 977.3 * i,
          place: kathmandu,
          utcOffsetSeconds: 20700,
        ),
    ];
    const yogas = RuleRequest(shipped: [ShippedRules.yogas]);
    const design = ResearchDesign(groups: [0, 1, 0, 1, 0, 1]);
    final table = ctx.research.counts(
      births: births,
      rules: yogas,
      design: design,
    );
    expect(table.rows, isNotEmpty);
    for (final row in table.rows) {
      for (final group in row.counts) {
        expect(
          group.present + group.absent + group.unreadable + group.unstable,
          3,
          reason: row.predicate,
        );
      }
    }
    // A seed past what a signed 64-bit int holds is the same seed here.
    final widest = (BigInt.one << 64) - BigInt.one;
    final tested = ctx.research.compare(
      births: births,
      rules: yogas,
      design: design,
      test: ResearchGroupTest(
        seed: widest,
        permutations: 199,
        contrast: const ResearchContrast.caseVsRest(1),
        alpha: 0.05,
      ),
    );
    // Six births, three cases: 20 labellings, fewer than the 199 asked,
    // so each is read once and the p-values are exact.
    expect(tested.enumerated, isTrue);
    expect(tested.permutations, 20);
    expect(tested.resolution, 1 / 20);
    for (final row in tested.rows) {
      expect(row.p.value, greaterThanOrEqualTo(tested.resolution));
      expect(row.adjusted.maxT, greaterThanOrEqualTo(row.p.value));
      expect(row.underAlpha, isNotNull);
    }
    expect(tested.provenance.inputHash, isNot(table.provenance.inputHash));
    expect(
      () => ctx.research.timed(
        subjects: const [],
        rules: yogas,
        dasha: DashaSystem.vimshottari,
        shuffle: ResearchEventShuffle.agesAtEvent,
        test: ResearchEventTest(seed: BigInt.one, permutations: 19),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'research.subjects',
        ),
      ),
    );
    // Self-matched, each person is read against their own year.
    final lives = [
      for (final birth in births)
        ResearchSubject(birth: birth, event: birth.instant + 9000.5),
    ];
    const year = ResearchReferents(windowDays: 365.25, stepDays: 30);
    final matched = ctx.research.timed(
      subjects: lives,
      rules: yogas,
      dasha: DashaSystem.vimshottari,
      shuffle: ResearchEventShuffle.selfMatched,
      referents: year,
      test: ResearchEventTest(seed: BigInt.one, permutations: 19),
    );
    expect(matched.rows, hasLength(tested.rows.length));
    expect(
      () => ctx.research.timed(
        subjects: lives,
        rules: yogas,
        dasha: DashaSystem.vimshottari,
        shuffle: ResearchEventShuffle.selfMatched,
        test: ResearchEventTest(seed: BigInt.one, permutations: 19),
      ),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'referents'),
      ),
    );
    // A transit rule is read at each pairing's instant, the sky's cost
    // chosen with maxPairs: six lives under the age shuffle need the sky at
    // up to 36 instants.
    const saturnReturn = RuleRequest(
      rules: [
        {
          'key': 'SATURN_RETURN',
          'category': 'transit',
          'source': {'text': 'a test'},
          'conditions': [
            {
              'type': 'transit-in-house',
              'planet': 'SATURN',
              'houses': [1],
              'from': 'SATURN',
            },
          ],
        },
      ],
    );
    final transit = ctx.research.timed(
      subjects: lives,
      rules: saturnReturn,
      dasha: DashaSystem.vimshottari,
      shuffle: ResearchEventShuffle.agesAtEvent,
      test: ResearchEventTest(seed: BigInt.one, permutations: 19),
    );
    expect(transit.rows.map((row) => row.predicate), ['SATURN_RETURN']);
    expect(
      () => ctx.research.timed(
        subjects: lives,
        rules: saturnReturn,
        dasha: DashaSystem.vimshottari,
        shuffle: ResearchEventShuffle.agesAtEvent,
        test: ResearchEventTest(
          seed: BigInt.one,
          permutations: 19,
          maxPairs: 2,
        ),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'test.maxPairs',
        ),
      ),
    );
    // An uncertain birth's window is read every step, and the step is
    // part of the study it pre-registers.
    final uncertain = [
      for (final birth in births)
        ResearchBirth(
          instant: birth.instant,
          place: birth.place,
          utcOffsetSeconds: birth.utcOffsetSeconds,
          uncertaintyMinutes: 30,
        ),
    ];
    ResearchCounts stepped(double? step) => ctx.research.counts(
      births: uncertain,
      rules: yogas,
      design: design,
      uncertaintyStepMinutes: step,
    );
    expect(
      stepped(10).provenance.inputHash,
      isNot(stepped(null).provenance.inputHash),
    );
    expect(
      () => stepped(0),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'uncertaintyStepMinutes',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart reads the Sarvatobhadra chakra', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const instant = 2447995.4895833335;
    const instants = [2460676.5, 2460776.5, 2460876.5];
    Chart found(SarvatobhadraRequest? asked) => ctx.chart.found(
      instant: instant,
      place: place,
      utcOffsetSeconds: 20700,
      sarvatobhadra: asked,
    );
    expect(found(null).sarvatobhadra, isNull);

    final natal = found(const SarvatobhadraRequest(instants: instants));
    final read = natal.sarvatobhadra!;
    expect(read, hasLength(instants.length));
    Nakshatra star(double longitude) =>
        Nakshatra.values
            .where((n) => n != Nakshatra.unknown)
            .toList()[(longitude * 27 ~/ 360)];
    final moon =
        natal.grahas.firstWhere((g) => g.graha == Graha.moon).longitudeDeg;
    const five = [
      Graha.mars,
      Graha.mercury,
      Graha.jupiter,
      Graha.venus,
      Graha.saturn,
    ];
    for (final (k, one) in read.indexed) {
      expect(one.native.nakshatra, star(moon));
      final sky =
          ctx.chart
              .found(instant: instants[k], place: place, utcOffsetSeconds: 0)
              .grahas;
      expect(
        [for (final g in one.grahas) g.graha],
        [for (final g in sky) g.graha],
      );
      for (final (n, vedha) in one.grahas.indexed) {
        expect(vedha.place.nakshatra, star(sky[n].longitudeDeg));
        if (five.contains(vedha.graha)) {
          expect(
            vedha.motion == VedhaMotion.retrograde,
            sky[n].speedDegPerDay < 0,
            reason: '${vedha.graha} by the sky',
          );
        }
        for (final reach in vedha.cells) {
          expect(reach.cell == null, reach.by == VedhaBy.stands);
        }
      }
      expect(one.grahas.first.motion, VedhaMotion.fast);
      expect(one.grahas.last.motion, VedhaMotion.retrograde);
      const five28 = [
        VedhaPoint.nakshatra,
        VedhaPoint.letter,
        VedhaPoint.vowel,
        VedhaPoint.tithi,
        VedhaPoint.rashi,
      ];
      expect(
        one.maleficPoints,
        one.points
            .where((p) => five28.contains(p.point) && p.malefics.isNotEmpty)
            .length,
      );
    }

    const given = [
      VedhaMotion.retrograde,
      VedhaMotion.mean,
      VedhaMotion.fast,
      VedhaMotion.mean,
      VedhaMotion.retrograde,
    ];
    final asked =
        found(
          SarvatobhadraRequest(
            instants: instants,
            motion: SarvatobhadraMotion.given(given),
            name: const NameSounds(letter: 'र', vowel: ChakraVowel.a),
            rules: const SarvatobhadraRules(sight: VedhaSight.allThree),
          ),
        ).sarvatobhadra!.first;
    expect([
      for (final graha in five)
        asked.grahas.firstWhere((g) => g.graha == graha).motion,
    ], given);
    expect(asked.grahas.every((g) => g.lines.length == 3), isTrue);
    expect(
      asked.native.name,
      const NameSounds(letter: 'र', vowel: ChakraVowel.a),
    );
    expect(asked.rules, const SarvatobhadraRules(sight: VedhaSight.allThree));
    expect([for (final p in asked.points) p.point], VedhaPoint.values);
    expect(
      () => found(const SarvatobhadraRequest(instants: [])),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'sarvatobhadra.instants',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart reads its tithi praveshas', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const instant = 2447995.4895833335;
    Chart found(double at, [TithiPraveshaRequest? asked]) => ctx.chart.found(
      instant: at,
      place: place,
      utcOffsetSeconds: 20700,
      tithiPravesha: asked,
    );
    double elongation(Chart chart) {
      double of(Graha graha) =>
          chart.grahas.firstWhere((g) => g.graha == graha).longitudeDeg;
      return (of(Graha.moon) - of(Graha.sun)) % 360;
    }

    expect(found(instant).tithiPravesha, isNull);
    final natal = found(instant, const TithiPraveshaRequest(through: 3));
    final years = natal.tithiPravesha!;
    expect([for (final y in years) y.year], [1, 2, 3]);
    final born = elongation(natal);
    for (final year in years) {
      expect(year.rule, TithiRule.natalSign);
      expect(year.fellBack, year.rule != year.picked);
      expect(year.elongationDeg, closeTo(born, 1e-9));
      expect((year.at - year.solarReturn).abs(), lessThan(33));
      expect(year.inSign, inInclusiveRange(0, 2));
      expect(elongation(found(year.at)), closeTo(born, 1e-3));
    }
    final nearest =
        found(
          instant,
          const TithiPraveshaRequest(through: 3, rule: TithiRule.nearestReturn),
        ).tithiPravesha!;
    for (final year in nearest) {
      expect(year.picked, TithiRule.nearestReturn);
      expect(year.fellBack, isFalse);
      expect((year.at - year.solarReturn).abs(), lessThan(16));
    }
    expect(
      () => found(instant, const TithiPraveshaRequest(through: 0)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'tithiPravesha.through',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart reads its month and day charts', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const instant = 2447995.4895833335;
    Chart found(double at, [MasaPraveshaRequest? asked]) => ctx.chart.found(
      instant: at,
      place: place,
      utcOffsetSeconds: 20700,
      masaPravesha: asked,
    );
    double sun(double at) =>
        found(at).grahas.firstWhere((g) => g.graha == Graha.sun).longitudeDeg;
    double apart(double a, double b) {
      final d = (a - b) % 360;
      return d < 360 - d ? d : 360 - d;
    }

    expect(found(instant).masaPravesha, isNull);
    final born = sun(instant);
    final months =
        found(instant, const MasaPraveshaRequest(year: 3)).masaPravesha!;
    expect(
      [for (final m in months) '${m.year} ${m.month} ${m.day}'],
      [for (var m = 1; m <= 12; m++) '3 $m 1'],
    );
    for (final month in months) {
      expect(
        apart(month.sunDeg, born + 30 * (month.month - 1)),
        lessThan(1e-9),
      );
    }
    expect(apart(sun(months[4].at), months[4].sunDeg), lessThan(1e-4));
    final days =
        found(
          instant,
          const MasaPraveshaRequest(year: 3, month: 5),
        ).masaPravesha!;
    expect(days, hasLength(30));
    expect(days.first.at, months[4].at);
    expect(apart(sun(days[9].at), months[4].sunDeg + 9), lessThan(1e-4));
    expect(
      () => found(
        instant,
        const MasaPraveshaRequest(year: 3, reading: ReturnLongitude.mean),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'masaPravesha.reading',
        ),
      ),
    );

    // A place founds each month and reads its lord; the first month's is
    // the year's.
    expect(months.first.masesha, isNull);
    final third =
        ctx.chart
            .found(
              instant: instant,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: const VarshaRequest(through: 3, place: AnnualPlace.birth),
            )
            .praveshas[2]
            .annual!;
    final lords =
        found(
          instant,
          const MasaPraveshaRequest(year: 3, place: AnnualPlace.birth),
        ).masaPravesha!;
    expect(lords, hasLength(12));
    expect(lords.first.masesha!.graha, third.yearLord.graha);
    for (final row in lords) {
      final lord = row.masesha!;
      expect(lord.reading, MaseshaReading.chain);
      expect(lord.graha, lord.chain.graha);
      expect(lord.chain.claims.every((c) => c.vishwa.total > 0), isTrue);
    }
    final others =
        found(
          instant,
          const MasaPraveshaRequest(
            year: 3,
            place: AnnualPlace.birth,
            masesha: MaseshaRules(reading: MaseshaReading.lagnaLord),
          ),
        ).masaPravesha!;
    for (final row in others) {
      expect(row.masesha!.graha, row.masesha!.bearers.masaLagna);
    }

    // Each month's results beside its lord: twelve houses, five lords.
    expect(months.first.results, isNull);
    for (final row in lords) {
      final results = row.results!;
      expect(results.houses, hasLength(12));
      expect(results.lords.map((one) => one.office), MonthLord.values);
      expect(results.lords[1].graha, row.masesha!.graha);
      expect(results.lords[2].graha, third.yearLord.graha);
      expect(results.lords[3].graha, row.masesha!.bearers.masaLagna);
      expect(results.houses.first.house, 1);
    }
    // Each month's dasha: the five Huddas of the stronger lord's sign, end
    // to end from the month's opening to the next's.
    expect(months.first.dasha, isNull);
    for (final (index, row) in lords.indexed) {
      final dasha = row.dasha!;
      final bearers = row.masesha!.bearers;
      expect(
        dasha.graha,
        dasha.office == MonthOffice.muntha ? bearers.muntha : bearers.masaLagna,
      );
      expect(dasha.periods.fold<int>(0, (sum, one) => sum + one.degrees), 30);
      expect(dasha.periods.first.starts, row.at);
      if (index < 11) {
        expect(dasha.periods.last.ends, closeTo(lords[index + 1].at, 1e-6));
      }
    }
    final civil =
        found(
          instant,
          const MasaPraveshaRequest(
            year: 3,
            place: AnnualPlace.birth,
            dasha: MasaDashaRules(clock: MonthClock.days),
          ),
        ).masaPravesha!.first.dasha!;
    expect(civil.rules, const MasaDashaRules(clock: MonthClock.days));
    expect(
      civil.periods.last.ends - civil.periods.first.starts,
      closeTo(30, 1e-9),
    );
    expect(
      () => found(
        instant,
        const MasaPraveshaRequest(
          year: 3,
          place: AnnualPlace.birth,
          results: MasaPhalaRules(weakBelow: 9 * 3600, strongFrom: 3600),
        ),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'masaPravesha.results.strongFrom',
        ),
      ),
    );
    // Beside a month, a place founds each day and reads its lord; the first
    // day's is the month's.
    final daily =
        found(
          instant,
          const MasaPraveshaRequest(
            year: 3,
            month: 2,
            place: AnnualPlace.birth,
          ),
        ).masaPravesha!;
    expect(daily, hasLength(30));
    expect(daily.first.dinesha!.graha, lords[1].masesha!.graha);
    for (final row in daily) {
      final lord = row.dinesha!;
      expect(row.masesha, isNull);
      expect(lord.graha, lord.chain.graha);
      expect(lord.bearers.masaLagna, lords[1].masesha!.bearers.masaLagna);
      // Each day's results beside its lord: the four lords the day's, the
      // year's, the month's and the Muntha's.
      final results = row.dayResults!;
      expect(results.houses, hasLength(12));
      expect(results.lords.map((one) => one.graha), [
        lord.graha,
        third.yearLord.graha,
        lords[1].masesha!.graha,
        lord.bearers.muntha,
      ]);
    }
    expect(
      daily.map((row) => row.dayResults!.moonState).toSet().length,
      greaterThan(1),
    );
    final waning =
        found(
          instant,
          const MasaPraveshaRequest(
            year: 3,
            month: 2,
            place: AnnualPlace.birth,
            results: MasaPhalaRules(kshina: KshinaRule.darkEleventhToNewMoon),
          ),
        ).masaPravesha!;
    expect(waning, hasLength(30));
    ctx.dispose();
  });

  test('a chart writes its dossier', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    Chart found([DossierRequest? asked]) => ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      dossier: asked,
    );
    const asked = DossierRequest(
      sections: [
        DossierSection.chart,
        DossierSection.grahas,
        DossierSection.shadbala,
      ],
    );
    expect(found().dossier, isNull);
    final text = found(asked).dossier!;
    expect(text, matches(RegExp(r'^# teistro dossier \d+\n== LEGEND ==\n')));
    expect(
      [
        for (final line in text.split('\n'))
          if (line.startsWith('== ')) line.substring(3, line.length - 3),
      ],
      ['LEGEND', 'CHART', 'GRAHAS', 'SHADBALA'],
      reason: 'the record widened the request',
    );
    expect(text, contains('+05:45'));
    expect(found(asked).dossier, text, reason: 'the same bytes');
    expect(
      () => found(const DossierRequest(sections: [DossierSection.dashas])),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'dossier.sections[0]',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its Lal Kitab', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const instant = 2447995.4895833335;
    LalKitab? found(LalKitabRequest? asked) =>
        ctx.chart
            .found(
              instant: instant,
              place: place,
              utcOffsetSeconds: 20700,
              lalkitab: asked,
            )
            .lalkitab;
    expect(found(null), isNull);

    final plain = found(const LalKitabRequest())!;
    expect(
      plain.cycle,
      const LalKitabCycle(planet: Graha.saturn, year: 1),
      reason: 'the book\'s general table',
    );
    expect(plain.year, isNull);
    expect(plain.reading.planets, hasLength(9));
    expect(
      plain.reading.planets.every(
        (p) => p.graha != Graha.unknown && p.house >= 1 && p.house <= 12,
      ),
      isTrue,
    );
    expect((plain.periods.first.from, plain.periods.last.to), (1, 120));

    final rows = [
      for (var y = 0; y < 120; y++)
        [for (var h = 0; h < 12; h++) (h + y) % 12 + 1],
    ];
    final read =
        found(
          LalKitabRequest(
            cycle: const LalKitabCycle(planet: Graha.venus, year: 17),
            year: 43,
            varshphal: rows,
          ),
        )!;
    final year = read.year!;
    expect(year.ruler, Graha.jupiter);
    expect(year.thirds, [Graha.ketu, Graha.jupiter, Graha.sun]);
    final annual = year.annual!;
    for (final (n, natal) in read.reading.planets.indexed) {
      expect(annual.planets[n].house, (natal.house + 41) % 12 + 1);
    }
    expect(
      () => found(const LalKitabRequest(year: 121)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'lalkitab.year',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its remedies', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const instant = 2447995.4895833335;
    Remedies? found(RemedyRequest? asked) =>
        ctx.chart
            .found(
              instant: instant,
              place: place,
              utcOffsetSeconds: 20700,
              remedies: asked,
            )
            .remedies;
    expect(found(null), isNull);

    final plain = found(const RemedyRequest())!;
    expect(plain.subjects.antardasha, isNull, reason: 'no instant, no periods');
    expect(plain.rules, const RemedyRules(), reason: 'the texts\' own');
    expect(plain.functional.rows, hasLength(7), reason: 'the seven lords');

    final read =
        found(
          const RemedyRequest(
            at: instant + 20 * 365.25,
            rules: RemedyRules(shanti: ShantiRules(rik: RikSource.yajnavalkya)),
          ),
        )!;
    expect(read.rules.shanti.rik, RikSource.yajnavalkya);
    expect(read.functional.lagna, isNot(Rashi.unknown));
    expect(
      read.subjects.subjects.every((one) => one.reasons.isNotEmpty),
      isTrue,
    );
    expect(
      [for (final one in read.shantis) one.graha],
      [for (final one in read.subjects.subjects) one.graha],
      reason: 'a śānti for each subject, in their order',
    );
    final running = read.subjects.antardasha!;
    expect(running.holds, hasLength(running.shanti.conditions.length));
    final devata = read.ishtaDevata;
    expect(devata.atmakaraka, isNot(Graha.unknown));
    for (final one in [devata.inRasi, devata.inNavamsha]) {
      expect(one.sign, isNot(Rashi.unknown));
      expect(one.devotions.every((d) => d.deities.isNotEmpty), isTrue);
    }
    final amatya = devata.amatya;
    expect(amatya.graha, isNot(anyOf(Graha.unknown, devata.atmakaraka)));
    for (final one in [amatya.inRasi, amatya.inNavamsha]) {
      expect(one.twelfth.sign, isNot(Rashi.unknown));
      expect(one.house, inInclusiveRange(1, 12));
      expect(one.joined.every((d) => d.graha != amatya.graha), isTrue);
    }
    expect(amatya.inNavamsha.sign, amatya.amsha);
    expect(
      () => found(const RemedyRequest(at: double.nan)),
      throwsA(
        isA<ArgumentError>().having((e) => e.name, 'name', 'remedies.at'),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its rectification', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const instant = 2447995.4895833335;
    Rectification? found(RectificationRequest? asked) =>
        ctx.chart
            .found(
              instant: instant,
              place: place,
              utcOffsetSeconds: 20700,
              rectification: asked,
            )
            .rectification;
    expect(found(null), isNull);

    const purifyOnly = RectificationRequest(purify: PurifyRequest(minutes: 20));
    final purified = found(purifyOnly)!;
    expect(purified.conception, isNull, reason: 'only what was asked');
    expect(purified.circumstance, isNull);
    expect(purified.baseline, isNull);
    expect(purified.svarodaya, isNull);
    final standing = purified.purified!;
    final runs = [...standing.intervals, ...standing.removed];
    expect(runs, isNotEmpty);
    expect(standing.grid.cells, greaterThan(0));
    for (final clause in runs.expand((run) => run.verdict.clauses)) {
      expect(clause.sign, isNot(Rashi.unknown));
      expect(clause.lagna, isNot(Rashi.unknown));
    }

    const asked = RectificationRequest(
      purify: PurifyRequest(minutes: 20),
      conception: ConceptionRules(),
      circumstance: CircumstanceRequest(
        facts: BirthFacts(fatherPresent: false),
      ),
      baseline: BaselineRectificationRequest(
        uncertaintyMinutes: 30,
        sex: Sex.male,
        events: [
          LifeEvent(kind: LifeEventKind.marriage, on: instant + 25 * 365.25),
          LifeEvent(
            kind: LifeEventKind.accident,
            on: instant + 30 * 365.25,
            heldOut: true,
          ),
        ],
      ),
    );
    final read = found(asked)!;
    final moon = read.conception!.moon;
    expect(moon.moonSign, isNot(Rashi.unknown));
    expect(moon.rising, isNot(Rashi.unknown));
    expect(moon.predicted.sign, isNot(Rashi.unknown));
    expect(moon.moonNakshatra, isNot(Nakshatra.unknown));
    expect(read.conception!.pranapadaHouse.house, inInclusiveRange(1, 12));
    final circumstance = read.circumstance!;
    expect(circumstance.sky.grahasDeg, hasLength(7), reason: 'Sun to Saturn');
    expect(circumstance.presentation.lord, isNot(Graha.unknown));
    expect(circumstance.presentation.rising, isNot(Rising.unknown));
    expect(circumstance.attending.between, isNot(contains(Graha.unknown)));
    final indications = [for (final w in circumstance.weights) w.indication];
    expect(indications, [CircumstanceIndication.father]);
    final baseline = read.baseline!;
    expect(baseline.candidates, isNotEmpty);
    expect(baseline.candidates.first.lagna, isNot(Rashi.unknown));
    expect(baseline.candidates.first.lagnaNakshatra, isNot(Nakshatra.unknown));
    expect(baseline.window.from, lessThan(baseline.window.to));
    expect(baseline.eventsUsed, 1);
    expect(baseline.eventsHeldOut, 1);
    expect(baseline.holdOut, hasLength(1));
    final notes = [for (final stage in baseline.stages) ...stage.notes];
    final sexes = notes.whereType<TattvaSexNote>();
    expect([for (final note in sexes) note.sex], [Sex.male]);
    final reported = notes.whereType<ReportedTimeNote>();
    expect([for (final note in reported) note.uncertaintyMinutes], [30]);
    final fits = notes.whereType<EventFitNote>().toList();
    expect(fits, hasLength(1), reason: 'the event held out is not fitted');
    expect(fits.first.event, 0);
    expect(fits.first.id, isNull);
    expect(fits.first.eventKind, LifeEventKind.marriage);
    expect(fits.first.lords, isNot(contains(Graha.unknown)));
    const none = RectificationRequest(purify: PurifyRequest(minutes: 0));
    expect(
      () => found(none),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'rectification.purify.minutes',
        ),
      ),
    );

    const told = RectificationRequest(svarodaya: SvarodayaRequest(minutes: 20));
    final sv = found(told)!.svarodaya!;
    expect(sv.at.tithi, isNot(Tithi.unknown));
    expect(sv.runs, isNotEmpty);
    expect(sv.runs.first.from, closeTo(instant - 20 / 1440, 1e-6));
    expect(sv.runs.last.to, closeTo(instant + 20 / 1440, 1e-6));
    for (var k = 1; k < sv.runs.length; k++) {
      expect(sv.runs[k].from, sv.runs[k - 1].to, reason: 'the runs tile');
    }
    expect(sv.at.run.from, lessThanOrEqualTo(instant));
    expect(sv.at.run.to, greaterThan(instant));
    final female = sv.at.run.nadi == SvarodayaNadi.moon;
    expect(sv.at.run.sex, female ? Sex.female : Sex.male);
    const idle = RectificationRequest(svarodaya: SvarodayaRequest(minutes: 0));
    expect(
      () => found(idle),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'rectification.svarodaya.minutes',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its KP reading', () {
    final ctx = teistro.context(
      profile: 'kp-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(13.08),
      longitudeDeg: Longitude(80.27),
      altitudeM: Altitude(6),
    );
    const births = [2447995.4895833335, 2451545.2];
    KpReading? found(double instant, KpRequest? asked, [Context? under]) =>
        (under ?? ctx).chart
            .found(
              instant: instant,
              place: place,
              utcOffsetSeconds: 19800,
              kp: asked,
            )
            .kp;
    expect(found(births[0], null), isNull);

    final reading = found(births[0], const KpRequest())!;
    expect(reading.chart.system, HouseSystem.placidus);
    expect(reading.chart.cusps, hasLength(12));
    expect(reading.significators.houses, hasLength(12));
    for (final planet in reading.chart.planets) {
      final span = planet.lords.subSub.span;
      expect(
        span.start <= planet.longitude && planet.longitude < span.end,
        isTrue,
        reason: '${planet.graha} inside its sub-sub',
      );
    }
    expect(reading.ruling.rules.count, 'FIVE');
    expect(reading.ruling.rulers.every((r) => r.reasons.isNotEmpty), isTrue);
    expect(reading, found(births[0], const KpRequest()), reason: 'a value');

    // A horary number casts the cusps; the ruling planets stay the moment's.
    const asked = KpRequest(number: 74);
    final horary = found(births[0], asked)!;
    final lagna = horary.chart.cusps.first;
    expect(lagna.longitude, lagna.lords.sub.span.start);
    expect(horary.ruling, reading.ruling);

    final batch = ctx.chart.foundMany(
      instants: births,
      place: place,
      utcOffsetSeconds: 19800,
      kp: asked,
    );
    for (final (k, instant) in births.indexed) {
      expect(batch.at(k).kp, found(instant, asked), reason: 'each alone');
    }

    for (final (bad, field) in [
      (const KpRequest(number: 250), 'kp.number'),
      (const KpRequest(clock: 90000), 'kp.clock'),
    ]) {
      expect(
        () => found(births[0], bad),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();

    final lahiri = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    expect(
      () => found(births[0], const KpRequest(), lahiri),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'frame.ayanamsha',
        ),
      ),
    );
    expect(
      found(
        births[0],
        const KpRequest(anyAyanamsha: true),
        lahiri,
      )!.chart.cusps,
      hasLength(12),
    );
    lahiri.dispose();
  });

  test(
    'a chart carries its transits, each verdict its own house and vedha',
    () {
      final ctx = context();
      final place = Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      );
      const birth = 2447995.4895833335;
      expect(
        ctx.chart
            .found(instant: birth, place: place, utcOffsetSeconds: 20700)
            .gochar,
        isEmpty,
      );
      final instants = [for (var k = 0; k < 24; k++) 2460676.5 + 30 * k];
      final verdicts = <GocharVerdict>{};
      for (final from in GocharFrom.values) {
        final readings =
            ctx.chart
                .found(
                  instant: birth,
                  place: place,
                  utcOffsetSeconds: 20700,
                  gochar: GocharRequest(instants: instants, from: from),
                )
                .gochar;
        expect(readings, hasLength(instants.length));
        for (final (k, reading) in readings.indexed) {
          expect(reading.instant, instants[k]);
          expect(reading.reference.from, from);
          expect(reading.rules.nodeVedha, NodeVedha.likeTheSun);
          expect(reading.rules.nodeObstruction, NodeObstruction.notEachOther);
          expect(reading.grahas, hasLength(9));
          for (final g in reading.grahas) {
            expect(g.house, inInclusiveRange(1, 12));
            expect(
              g.transit.degrees,
              allOf(greaterThanOrEqualTo(0), lessThan(30)),
            );
            final expected =
                !g.goodHouse
                    ? GocharVerdict.notGood
                    : g.obstructedBy.isNotEmpty
                    ? GocharVerdict.obstructed
                    : GocharVerdict.good;
            expect(g.verdict, expected, reason: '${g.graha} in ${g.house}');
            if (!g.goodHouse) expect(g.vedhaHouse, isNull);
            verdicts.add(g.verdict);
          }
          final rahu = reading.grahas[7];
          final ketu = reading.grahas[8];
          expect(
            (ketu.house - rahu.house) % 12,
            6,
            reason: 'the nodes stand opposite',
          );
          expect(rahu.obstructedBy, isNot(contains(Graha.ketu)));
        }
      }
      expect(verdicts, GocharVerdict.values.toSet());
      final judged =
          ctx.chart
              .found(
                instant: birth,
                place: place,
                utcOffsetSeconds: 20700,
                gochar: GocharRequest(instants: instants, ashtakavarga: true),
              )
              .gochar;
      for (final reading in judged) {
        expect(reading.ashtakavarga, hasLength(7));
        for (final (k, one) in reading.ashtakavarga!.indexed) {
          final moving = reading.grahas[k];
          expect(one.graha, moving.graha);
          expect(one.good, one.bindus >= 5);
          expect(
            one.kakshya.index,
            (moving.transit.degrees / 3.75).floor() + 1,
          );
        }
      }
      expect(
        () => ctx.chart.found(
          instant: birth,
          place: place,
          utcOffsetSeconds: 20700,
          gochar: const GocharRequest(instants: []),
        ),
        throwsA(
          isA<TeistroException>().having(
            (e) => e.field,
            'field',
            'gochar.instants',
          ),
        ),
      );
      ctx.dispose();
    },
  );

  test('a chart carries its sthira karakas and karaka bhavas', () {
    // BPHS ch. 32 vv. 18 to 24 as `03-design/jaimini-significators.md` §7
    // tabulates them: the table's order, the verse's candidates, a pair's
    // stronger chosen and saying how, and each house counted from its graha.
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const sthira = [
      (Signification.father, [Graha.sun, Graha.venus]),
      (Signification.mother, [Graha.moon, Graha.mars]),
      (Signification.sister, [Graha.mars]),
      (Signification.wifesBrother, [Graha.mars]),
      (Signification.youngerBrother, [Graha.mars]),
      (Signification.maternalRelatives, [Graha.mercury]),
      (Signification.maternalUncle, [Graha.mercury]),
      (Signification.paternalGrandfather, [Graha.jupiter]),
      (Signification.husband, [Graha.venus]),
      (Signification.son, [Graha.saturn]),
    ];
    const bhavas = [
      (Graha.sun, 9, Signification.father),
      (Graha.moon, 4, Signification.mother),
      (Graha.mars, 3, Signification.brother),
      (Graha.mercury, 6, Signification.maternalUncle),
      (Graha.jupiter, 5, Signification.son),
      (Graha.venus, 7, Signification.wife),
      (Graha.saturn, 8, Signification.death),
    ];
    final chosen = <ChosenBy>{};
    for (var step = 0; step < 40; step++) {
      final chart = ctx.chart.found(
        instant: 2451545.0 + step * 2.9,
        place: place,
        utcOffsetSeconds: 20700,
        jaimini: true,
      );
      final reading = chart.jaimini!;
      // A record's list compares by identity, so the two fields apart.
      expect(
        [for (final k in reading.sthiraKarakas) k.signification],
        [for (final (of, _) in sthira) of],
      );
      expect(
        [for (final k in reading.sthiraKarakas) k.candidates],
        [for (final (_, from) in sthira) from],
      );
      for (final k in reading.sthiraKarakas) {
        expect(k.candidates, contains(k.graha));
        expect(k.chosenBy == ChosenBy.alone, k.candidates.length == 1);
        if (k.candidates.length == 2) chosen.add(k.chosenBy);
      }
      expect([
        for (final b in reading.karakaBhavas)
          (b.graha, b.house, b.signification),
      ], bhavas);
      for (final b in reading.karakaBhavas) {
        final own = chart.grahas[b.graha.id].longitudeDeg ~/ 30;
        expect(b.sign.id, (own + b.house - 1) % 12);
      }
    }
    expect(chosen, contains(ChosenBy.sign));
    expect(chosen.length, greaterThan(1));
    ctx.dispose();
  });

  test('a chart carries its karakamsha and its Brahma graha, or why not', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .jaimini,
      isNull,
    );
    final outcomes = <BrahmaOutcome>{};
    for (var step = 0; step < 40; step++) {
      final reading =
          ctx.chart
              .found(
                instant: 2451545.0 + step * 0.37,
                place: place,
                utcOffsetSeconds: 20700,
                jaimini: true,
              )
              .jaimini!;
      final k = reading.karakamsha;
      final b = reading.brahma;
      for (final houses in [k.inRasi, k.inNavamsha]) {
        expect(houses, hasLength(9));
        for (final house in houses) {
          expect(house, inInclusiveRange(1, 12));
        }
      }
      expect(k.inNavamsha[k.atmakaraka.id], 1);
      expect(b.rule, BrahmaRule.verses);
      expect(b.graha == null, b.none != null);
      expect(b.none, isNot(BrahmaOutcome.found));
      if (b.graha != null) {
        expect(b.qualified, contains(b.passedFrom ?? b.graha));
      }
      outcomes.add(b.none ?? BrahmaOutcome.found);
      expect(reading.grahaArudhas, hasLength(9));
      expect(reading.grahaArudhas.take(7), everyElement(isNotNull));
      // The default co-lordship gives the nodes no own sign, so no arudha.
      expect(reading.grahaArudhas.skip(7), everyElement(isNull));
    }
    expect(outcomes, contains(BrahmaOutcome.found));
    expect(outcomes.length, greaterThan(1));
    ctx.dispose();
  });

  /// Every graha's state carries its Sayanadi: the nine grahas a state and a
  /// sub-state under each of the five ankas, the outer planets none.
  test(
    'a graha\'s state carries its Sayanadi and a sub-state for every anka',
    () {
      final ctx = context();
      final place = Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      );
      final states =
          ctx.chart
              .found(
                instant: 2451545.0,
                place: place,
                utcOffsetSeconds: 20700,
                state: true,
              )
              .states;
      const nine = {
        Graha.sun,
        Graha.moon,
        Graha.mars,
        Graha.mercury,
        Graha.jupiter,
        Graha.venus,
        Graha.saturn,
        Graha.rahu,
        Graha.ketu,
      };
      for (final state in states) {
        final sayanadi = state.sayanadi;
        if (!nine.contains(state.graha)) {
          expect(sayanadi, isNull, reason: state.graha.fullKey);
          continue;
        }
        expect(sayanadi, isNotNull, reason: state.graha.fullKey);
        expect(sayanadi!.cheshtas, hasLength(5));
        expect(sayanadi.cheshta(3), sayanadi.cheshtas[2]);
        expect(() => sayanadi.cheshta(6), throwsRangeError);
      }
      ctx.dispose();
    },
  );

  /// The three states the verses define cross as booleans, and as null
  /// where the verses give the body no answer: Ketu and the outer planets
  /// have no marana karaka sthana, the outer planets no pushkara.
  test(
    'a graha\'s state carries its pushkara and marana karaka sthana answers',
    () {
      final ctx = context();
      final place = Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      );
      final states =
          ctx.chart
              .found(
                instant: 2451545.0,
                place: place,
                utcOffsetSeconds: 20700,
                state: true,
              )
              .states;
      const named = {
        Graha.sun,
        Graha.moon,
        Graha.mars,
        Graha.mercury,
        Graha.jupiter,
        Graha.venus,
        Graha.saturn,
        Graha.rahu,
      };
      for (final state in states) {
        final classical =
            named.contains(state.graha) || state.graha == Graha.ketu;
        final reason = state.graha.fullKey;
        expect(state.pushkaraNavamsha == null, !classical, reason: reason);
        expect(state.pushkaraBhaga == null, !classical, reason: reason);
        expect(
          state.maranaKarakaSthana == null,
          !named.contains(state.graha),
          reason: reason,
        );
      }
      expect(states.where((s) => s.maranaKarakaSthana != null), hasLength(8));
      ctx.dispose();
    },
  );

  /// A chart's Vaiseshikamsa crosses whole: each scheme's count within its
  /// vargas, a name for every count from two; null unless asked.
  test('a chart carries its Vaiseshikamsa, each scheme\'s count and name', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      vaiseshikamsa: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .vaiseshikamsa,
      isNull,
    );
    final grahas = chart.vaiseshikamsa!.grahas;
    expect(grahas, hasLength(7));
    for (final g in grahas) {
      for (final (standing, vargas) in [
        (g.shadvarga, 6),
        (g.saptavarga, 7),
        (g.dashavarga, 10),
        (g.shodashavarga, 16),
      ]) {
        expect(standing.goodVargas, lessThanOrEqualTo(vargas));
        expect(standing.name == null, standing.goodVargas < 2);
      }
    }
    ctx.dispose();
  });

  /// A chart's Vimshopaka crosses whole: every graha's four scores out of 20
  /// under the default reading, the text's, whose least in any varga is 5;
  /// null unless asked.
  // A chart names the ayanamsha it was read under. The blob carried it and
  // no layer read it, so a reader could see an offset and not whose; and a
  // tropical chart has none, which is not an ayanamsha of nought.
  test('a chart names its ayanamsha, and a tropical one has none', () {
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final sidereal = context();
    final chart = sidereal.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(chart.ayanamsha, Ayanamsha.lahiri);
    expect(chart.ayanamshaCustom, isFalse);
    expect(chart.ayanamshaOffsetDeg, isNot(0.0));
    sidereal.dispose();
    final tropical = context(
      settings: {
        'frame': {'zodiac': 'TROPICAL'},
      },
    );
    final western = tropical.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(western.ayanamsha, isNull);
    expect(western.ayanamshaCustom, isFalse);
    tropical.dispose();
  });

  // A chart's day and an almanac's are one record, and its date is the one
  // `calendar.convert` takes. Before, Dart flattened the weekday and the
  // sunrise onto the chart and left the rest out.
  test('a chart\'s day is the almanac\'s, and its date converts', () {
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final ctx = context();
    final day =
        ctx.chart
            .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
            .day;
    final same =
        ctx.almanac
            .day(date: day.date, place: place, utcOffsetSeconds: 20700)
            .day;
    expect(
      [same.vara, same.sunrise, same.sunset, same.nextSunrise, same.date.day],
      [day.vara, day.sunrise, day.sunset, day.nextSunrise, day.date.day],
    );
    expect(
      [
        day.date.calendar,
        day.date.era,
        day.date.year,
        day.date.month,
        day.date.day,
      ],
      [Calendar.bikramSambat, Era.vikrama, 2056, 9, 17],
    );
    final gregorian = ctx.calendar.convert(day.date, Calendar.gregorian);
    expect([gregorian.year, gregorian.month, gregorian.day], [2000, 1, 1]);
    expect(day.vara, Vara.shanivara);
    expect(day.sunrise < day.sunset && day.sunset < 2451545.0, isTrue);
    expect(2451545.0 < day.nextSunrise, isTrue);
    expect(day.polar, isNull);
    expect(day.convention, Sunrise.centreRefraction);
    expect(day.customAltitudeDeg, isNull);
    ctx.dispose();

    // A custom altitude is a number and no named convention.
    final custom = context(
      settings: {
        'day': {
          'sunrise': {'kind': 'CUSTOM', 'altitude_deg': -0.5},
        },
      },
    );
    final own =
        custom.chart
            .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
            .day;
    expect(own.convention, isNull);
    expect(own.customAltitudeDeg, -0.5);
    expect(own.air, isNull);
    custom.dispose();

    // An air named for a refracted convention: what was left out comes
    // back resolved at the place, 856 hPa at 1400 m, and the thinner air
    // lifts the Sun less, so it clears the horizon later than under the
    // fixed 34′.
    LocalDay refracted(Map<String, Object?> sunrise) {
      final each = context(
        settings: {
          'day': {'sunrise': sunrise},
        },
      );
      try {
        return each.chart
            .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
            .day;
      } finally {
        each.dispose();
      }
    }

    final almanac = refracted({
      'kind': 'NAMED',
      'which': 'UPPER_LIMB_REFRACTION',
    });
    final standard = refracted({
      'kind': 'ATMOSPHERIC',
      'which': 'UPPER_LIMB_REFRACTION',
      'air': <String, Object?>{},
    });
    expect(almanac.air, isNull);
    expect(standard.convention, Sunrise.upperLimbRefraction);
    expect(standard.air!.pressureHpa, closeTo(855.99, 0.01));
    expect(standard.air!.temperatureC, 15.0);
    final later = (standard.sunrise - almanac.sunrise) * 86400.0;
    expect(later > 20.0 && later < 35.0, isTrue, reason: '$later s');
    final weather = refracted({
      'kind': 'ATMOSPHERIC',
      'which': 'LOWER_LIMB_REFRACTION',
      'air': {'pressure_hpa': 870.0, 'temperature_c': -4.5},
    });
    expect(
      [weather.air!.pressureHpa, weather.air!.temperatureC],
      [870.0, -4.5],
    );
    expect(
      () => refracted({
        'kind': 'ATMOSPHERIC',
        'which': 'CENTRE_NO_REFRACTION',
        'air': <String, Object?>{},
      }),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.message,
          'message',
          contains('does not refract'),
        ),
      ),
    );

    // Tromsø at midsummer: civil midnight holds the instant and says so;
    // the nearest real sunrise is weeks away, and the refusal names the
    // policy rather than the instant.
    final tromso = Observer(
      latitudeDeg: Latitude(69.65),
      longitudeDeg: Longitude(18.96),
      altitudeM: Altitude(0),
    );
    Context under(String policy) => context(
      settings: {
        'day': {'polar_day_policy': policy},
      },
    );
    final civil = under('CIVIL_MIDNIGHT');
    final polar =
        civil.chart
            .found(instant: 2451716.5, place: tromso, utcOffsetSeconds: 7200)
            .day
            .polar!;
    expect(
      [polar.kind, polar.policy],
      [PolarKind.day, PolarDayPolicy.civilMidnight],
    );
    civil.dispose();
    final nearest = under('NEAREST_EVENT');
    expect(
      () => nearest.chart.found(
        instant: 2451716.5,
        place: tromso,
        utcOffsetSeconds: 7200,
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'day.polar_day_policy',
        ),
      ),
    );
    nearest.dispose();
  });

  test('a chart carries its Vimshopaka, each graha\'s four scores', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      vimshopaka: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .vimshopaka,
      isNull,
    );
    final vs = chart.vimshopaka!;
    expect(vs.scoring, VimshopakaScoring.bphs);
    expect(
      [for (final g in vs.grahas) g.graha],
      [
        Graha.sun,
        Graha.moon,
        Graha.mars,
        Graha.mercury,
        Graha.jupiter,
        Graha.venus,
        Graha.saturn,
      ],
    );
    for (final g in vs.grahas) {
      for (final score in [
        g.shadvarga,
        g.saptavarga,
        g.dashavarga,
        g.shodashavarga,
      ]) {
        expect(score, inInclusiveRange(5.0, 20.0), reason: '${g.graha}');
      }
    }
    ctx.dispose();
  });

  /// A chart's dashas cross whole: the balance, the periods to the
  /// settings' depth with their paths, and the chain at an instant read off
  /// them.
  /// A consumer's own dasha system crosses: registered on the context, asked
  /// for by its key, named by it in the answer, and every period its
  /// catalogued twin's; a definition the checks refuse is named by its place
  /// and field.
  /// A consumer's system may count over the twenty-eight nakshatras with
  /// Abhijit in groups of its own: Shashtihayani stated as BPHS states it
  /// reads as the catalogued one (crux C1).
  test('a consumer system counted with Abhijit reads as the text row', () {
    const shashti = UduDashaDefinition(
      key: 'ACME_SHASHTI',
      lords: [
        DashaLord(Graha.jupiter, 10),
        DashaLord(Graha.sun, 10),
        DashaLord(Graha.mars, 10),
        DashaLord(Graha.moon, 6),
        DashaLord(Graha.mercury, 6),
        DashaLord(Graha.venus, 6),
        DashaLord(Graha.saturn, 6),
        DashaLord(Graha.rahu, 6),
      ],
      reference: Nakshatra.ashwini,
      groups: [3, 4, 3, 4, 3, 4, 3, 4],
      wheel: DashaWheel.withAbhijit,
      repeats: false,
    );
    final ctx = teistro.context(testProvider: true, dashaSystems: [shashti]);
    addTearDown(ctx.dispose);
    final [consumer, shipped] =
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              dashas: [
                DashaSystem.registered('ACME_SHASHTI'),
                DashaSystem.shashtihayani,
              ],
            )
            .dashas;
    expect(consumer.periods.length, shipped.periods.length);
    for (var i = 0; i < shipped.periods.length; i++) {
      final (a, b) = (consumer.periods[i], shipped.periods[i]);
      expect((a.path, a.lord, a.from, a.to), (b.path, b.lord, b.from, b.to));
    }
  });

  test('an event study reads a dasha system the context registered', () {
    // Vimshottari's table under a consumer's key: its periods deliver as
    // the catalogue's do, and only the registration the hash seals differs.
    final twin = UduDashaDefinition(
      key: 'ACME_VIMSHOTTARI',
      lords: [
        const DashaLord(Graha.ketu, 7),
        const DashaLord(Graha.venus, 20),
        const DashaLord(Graha.sun, 6),
        const DashaLord(Graha.moon, 10),
        const DashaLord(Graha.mars, 7),
        const DashaLord(Graha.rahu, 18),
        const DashaLord(Graha.jupiter, 16),
        const DashaLord(Graha.saturn, 19),
        const DashaLord(Graha.mercury, 17),
      ],
      reference: Nakshatra.ashwini,
    );
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      dashaSystems: [twin],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    final lives = [
      for (var i = 0; i < 6; i++)
        ResearchSubject(
          birth: ResearchBirth(
            instant: 2447000.25 + 977.3 * i,
            place: kathmandu,
            utcOffsetSeconds: 20700,
          ),
          event: 2447000.25 + 977.3 * i + 9000.5,
        ),
    ];
    ResearchTested timed(KeyOf<DashaSystem> dasha) => ctx.research.timed(
      subjects: lives,
      rules: const RuleRequest(shipped: [ShippedRules.yogas]),
      dasha: dasha,
      shuffle: ResearchEventShuffle.agesAtEvent,
      test: ResearchEventTest(seed: BigInt.one, permutations: 19),
    );
    final own = timed(DashaSystem.registered('ACME_VIMSHOTTARI'));
    final shipped = timed(DashaSystem.vimshottari);
    expect(own.rows, hasLength(shipped.rows.length));
    for (var i = 0; i < shipped.rows.length; i++) {
      final (a, b) = (own.rows[i], shipped.rows[i]);
      expect((a.predicate, a.p.value), (b.predicate, b.p.value));
    }
    expect(own.provenance.inputHash, isNot(shipped.provenance.inputHash));
    expect(
      () => timed(DashaSystem.registered('ACME_OTHER')),
      throwsA(isA<TeistroException>().having((e) => e.field, 'field', 'dasha')),
    );
    ctx.dispose();
  });

  test('a consumer dasha system registers and reads as its twin', () {
    final twin = UduDashaDefinition(
      key: 'ACME_VIMSHOTTARI',
      lords: [
        const DashaLord(Graha.ketu, 7),
        const DashaLord(Graha.venus, 20),
        const DashaLord(Graha.sun, 6),
        const DashaLord(Graha.moon, 10),
        const DashaLord(Graha.mars, 7),
        const DashaLord(Graha.rahu, 18),
        const DashaLord(Graha.jupiter, 16),
        const DashaLord(Graha.saturn, 19),
        const DashaLord(Graha.mercury, 17),
      ],
      reference: Nakshatra.ashwini,
    );
    const signTwin = RashiDashaDefinition(key: 'ACME_CHARA');
    final ctx = teistro.context(
      testProvider: true,
      dashaSystems: [twin, signTwin],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      dashas: [
        DashaSystem.registered('ACME_VIMSHOTTARI'),
        DashaSystem.vimshottari,
      ],
    );
    final [consumer, shipped] = chart.dashas;
    expect(consumer.system, DashaSystem.registered('ACME_VIMSHOTTARI'));
    expect(shipped.system, DashaSystem.vimshottari);
    expect(consumer.periods.length, shipped.periods.length);
    for (var i = 0; i < shipped.periods.length; i++) {
      final (a, b) = (consumer.periods[i], shipped.periods[i]);
      expect((a.path, a.lord, a.from, a.to), (b.path, b.lord, b.from, b.to));
    }
    // The other kernel, the same way: a sign-based system of one's own
    // answers as the catalogued row it copies.
    final signs = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      dashas: [DashaSystem.registered('ACME_CHARA'), DashaSystem.chara],
    );
    final [own, chara] = signs.dashas;
    expect(own.system, DashaSystem.registered('ACME_CHARA'));
    expect(chara.system, DashaSystem.chara);
    expect(own.periods.length, chara.periods.length);
    for (var i = 0; i < chara.periods.length; i++) {
      final (a, b) = (own.periods[i], chara.periods[i]);
      expect((a.path, a.lord, a.from, a.to), (b.path, b.lord, b.from, b.to));
    }
    expect(
      () => ctx.chart.found(
        instant: 2451545.0,
        place: place,
        utcOffsetSeconds: 20700,
        dashas: [DashaSystem.registered('ACME_OTHER')],
      ),
      throwsArgumentError,
    );
    ctx.dispose();
    expect(
      () => teistro.context(
        testProvider: true,
        dashaSystems: [
          const RashiDashaDefinition(key: 'ACME_THIRTEEN', strongerOf: [1, 13]),
        ],
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'options.dashas_json[0].stronger_of[1]',
        ),
      ),
    );
    expect(
      () => teistro.context(
        testProvider: true,
        dashaSystems: [
          UduDashaDefinition(
            key: twin.key,
            lords: twin.lords,
            reference: twin.reference,
            span: 0,
          ),
        ],
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'options.dashas_json[0].span',
        ),
      ),
    );
    // Every word a definition writes is a key, and its fields are spelt
    // as the document spells them; the SDK reads it back as written.
    const sthira = RashiDashaDefinition(
      key: 'ACME_STHIRA',
      start: RashiStart.arudhaLagna,
      order: RashiOrder.trineGroups,
      length: RashiLength.byModality(movable: 7, fixed: 8, dual: 9),
      namedLord: RashiNamedLord.first,
      strongerOf: [1, 7],
    );
    expect(sthira.toJson(), {
      'kernel': 'RASHI',
      'key': 'ACME_STHIRA',
      'start': 'ARUDHA_LAGNA',
      'order': 'TRINE_GROUPS',
      'length': {
        'BY_MODALITY': {'movable': 7, 'fixed': 8, 'dual': 9},
      },
      'named_lord': 'FIRST',
      'stronger_of': [1, 7],
    });
    teistro
        .context(
          testProvider: true,
          dashaSystems: [
            sthira,
            const RashiDashaDefinition(
              key: 'ACME_NAVA',
              length: RashiLength.fixed(9),
            ),
          ],
        )
        .dispose();
  });

  /// The annual charts cross: a request's `varsha` answers each chart's
  /// returns in year order, ragged per chart, and the instant founds as a
  /// chart of its own.
  test('a chart carries the annual charts its birth opens', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      varsha: const VarshaRequest(through: 12),
    );
    final years = chart.praveshas;
    expect(years.length, 12);
    expect(
      [for (final one in years) one.year],
      [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    );
    expect(years.first.instant, greaterThan(2447995.4895833335));
    // Eleven sidereal years between the first and the twelfth, to a day.
    final span = years.last.instant - years.first.instant;
    expect((span - 11 * 365.2564).abs(), lessThan(1));

    // The Muntha advances one sign a year from the birth lagna, which is
    // the whole of its rule; its lord is that sign's.
    final natal = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
    );
    final lagna = natal.lagnaDeg ~/ 30;
    for (final one in years) {
      expect(one.muntha.sign.id, (lagna + one.year) % 12);
      expect(one.muntha.lord, isNot(Graha.unknown));
    }
    // Twelve years is a whole circle back to the lagna's own sign.
    expect(years.last.muntha.sign, Rashi.byId(lagna % 12));

    // The two readings of the degree agree on the sign and part inside it.
    final carried = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      varsha: const VarshaRequest(
        through: 12,
        muntha: MunthaDegree.natalDegree,
      ),
    );
    for (var i = 0; i < years.length; i += 1) {
      expect(carried.praveshas[i].muntha.sign, years[i].muntha.sign);
      expect(
        carried.praveshas[i].muntha.longitudeDeg,
        greaterThanOrEqualTo(years[i].muntha.longitudeDeg),
      );
    }

    // No place, no chart founded: the instants alone, as before.
    expect(years.every((one) => one.annual == null), isTrue);

    // At the birthplace each year's chart comes back with its five
    // office-bearers; the birth lagna's lord is shared by every year and the
    // Muntha's lord is the one already on the return.
    final cast =
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: const VarshaRequest(
                through: 12,
                place: AnnualPlace.birth,
              ),
            )
            .praveshas;
    for (final one in cast) {
      expect(one.annual, isNotNull);
      expect(
        one.annual!.officeBearers.janmaLagna,
        cast.first.annual!.officeBearers.janmaLagna,
      );
      expect(one.annual!.officeBearers.muntha, one.muntha.lord);
    }
    final again = ctx.chart.found(
      instant: cast[3].instant,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(again.lagnaDeg, cast[3].annual!.lagnaDeg);

    // Each year names a lord, chosen among its own claimants.
    for (final one in cast) {
      final lord = one.annual!.yearLord;
      expect(lord.claims.length, inInclusiveRange(1, 5));
      // Among its own claimants, unless it succeeds the Moon, whose Ithasala
      // may be with any planet.
      final succeedsTheMoon = const [
        VarsheshaChosen.moonsIthasala,
        VarsheshaChosen.moonsSignLord,
      ].contains(lord.chosen);
      if (succeedsTheMoon) {
        expect(lord.moonPassedOver, isTrue);
      } else {
        expect(lord.claims.map((claim) => claim.graha), contains(lord.graha));
      }
      final ranked = [for (final claim in lord.claims) claim.vishwa.total];
      final sorted = [...ranked]..sort((int a, int b) => b - a);
      expect(ranked, orderedEquals(sorted));
      expect(lord.vishwa.toString(), matches(r'^\d\d:\d\d:\d\d$'));
      expect(lord.vishwa.units, lord.vishwa.total ~/ 3600);
    }

    // The pairs that make a yoga: never a neutral aspect, and each of
    // Table X-3's four kinds standing where its own degrees put it.
    final kinds = <TajikaYoga>{};
    for (final one in cast) {
      for (final pair in one.annual!.yogas) {
        expect(pair.drishti, isNot(TajikaDrishti.none));
        expect(pair.orbDeg, greaterThan(0));
        kinds.add(pair.yoga);
        if (pair.yoga == TajikaYoga.ithasalaVartamana) {
          expect(pair.apartDeg, greaterThanOrEqualTo(1));
        }
        if (pair.yoga == TajikaYoga.ithasalaPoorna) {
          expect(pair.apartDeg.abs(), lessThan(1));
        }
        if (pair.yoga == TajikaYoga.ishrafa) {
          expect(pair.apartDeg, lessThanOrEqualTo(-1));
        }
      }
    }
    // The corpus's own years reach every kind the boundary can say, so a
    // variant that stopped crossing would be caught here and not only in
    // the enum's member count.
    expect(kinds, hasLength(TajikaYoga.values.length));

    // At a residence, in the parts `found` takes, the lagnas move.
    final delhi =
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: VarshaRequest(
                through: 12,
                place: AnnualPlace.at(
                  Observer(
                    latitudeDeg: Latitude(28.6139),
                    longitudeDeg: Longitude(77.209),
                    altitudeM: Altitude(216),
                  ),
                  utcOffsetSeconds: 19800,
                ),
              ),
            )
            .praveshas;
    for (var i = 0; i < delhi.length; i += 1) {
      expect(
        (delhi[i].annual!.lagnaDeg - cast[i].annual!.lagnaDeg).abs(),
        greaterThan(0.1),
      );
    }

    // The instant founds as a chart of its own; the place is the caller's.
    final annual = ctx.chart.found(
      instant: years.last.instant,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(annual.instant, years.last.instant);

    // Not asked for is empty, not zeroes.
    expect(
      ctx.chart
          .found(
            instant: 2447995.4895833335,
            place: place,
            utcOffsetSeconds: 20700,
          )
          .praveshas,
      isEmpty,
    );

    // The rivals are asked for by name and are not the same instant.
    final tropical =
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: const VarshaRequest(
                through: 12,
                reading: ReturnLongitude.tropical,
              ),
            )
            .praveshas;
    expect(tropical.last.instant, isNot(years.last.instant));

    expect(
      () => ctx.chart.found(
        instant: 2447995.4895833335,
        place: place,
        utcOffsetSeconds: 20700,
        varsha: const VarshaRequest(through: 0),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'varsha.through',
        ),
      ),
    );
    ctx.dispose();
  });

  test("a year's chart answers the sahams asked for", () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<Pravesha> years(VarshaRequest varsha) =>
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: varsha,
            )
            .praveshas;

    for (final one in years(
      const VarshaRequest(
        through: 4,
        place: AnnualPlace.birth,
        sahams: Sahams.these([Saham.karyaSiddhi, Saham.punya]),
      ),
    )) {
      final annual = one.annual!;
      expect(annual.sahams.map((p) => p.saham), [
        Saham.karyaSiddhi,
        Saham.punya,
      ]);
      final lagna = annual.lagnaDeg ~/ 30;
      for (final point in annual.sahams) {
        expect(point.longitudeDeg, inInclusiveRange(0, 360));
        final sign = point.longitudeDeg ~/ 30;
        expect(point.sign, Rashi.byId(sign));
        expect(point.house, (sign - lagna) % 12 + 1);
        expect(point.lord.id, lessThan(7));
      }
    }

    // `all` is the forty-one in the source's order; unasked is none.
    final every =
        years(
          const VarshaRequest(
            through: 1,
            place: AnnualPlace.birth,
            sahams: Sahams.all,
          ),
        ).first.annual!.sahams;
    expect(every.map((p) => p.saham), Saham.values);
    expect(
      years(
        const VarshaRequest(through: 1, place: AnnualPlace.birth),
      ).first.annual!.sahams,
      isEmpty,
    );

    // The rules cross in the boundary's casing.
    final never = years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        sahams: Sahams.all,
        sahamRules: SahamRules(addSign: AddSign.never),
      ),
    );
    expect(
      never.every((one) => one.annual!.sahams.every((p) => !p.addedSign)),
      isTrue,
    );
    years(
      const VarshaRequest(
        through: 1,
        place: AnnualPlace.birth,
        sahams: Sahams.these([Saham.mrityu]),
        sahamRules: SahamRules(
          houses: HousePoints.equal,
          roga: RogaReading.saturn,
        ),
      ),
    );

    Matcher refusedBy(String field) =>
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field));
    // Each saham carries its strength, every founded year its Harsha
    // bala, and every birth its own sahams, which need no place.
    final chart = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      varsha: const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        sahams: Sahams.all,
        sahamStrength: SahamStrengthReadings(
          natures: SahamNatures.chapter,
          friendship: SahamFriendship.positional,
          weakBelow: 5 * 3600,
        ),
        harshaRules: HarshaRules(venus: VenusPlace.twelfth),
      ),
    );
    for (final one in chart.praveshas) {
      final annual = one.annual!;
      for (final saham in annual.sahams) {
        final near =
            saham.strong.contains(SahamStrong.lordConjoins) ||
            saham.strong.contains(SahamStrong.lordAspectsSaham);
        // The two (c) clauses negate each other: exactly one holds.
        expect(near, isNot(saham.weak.contains(SahamWeak.lordApart)));
        expect(saham.seven, hasLength(7));
        expect(
          saham.seven.firstWhere((s) => s.graha == saham.lord).company,
          saham.strong.contains(SahamStrong.lordConjoins),
        );
        expect(saham.inNodeAxis, isNotNull);
      }
      expect(annual.harsha, hasLength(7));
      for (final h in annual.harsha) {
        final parts =
            [
              h.sthana,
              h.uchchaSwakshetra,
              h.striPurusha,
              h.dinaRatri,
            ].where((held) => held).length;
        expect(h.total, 5 * parts);
      }
    }
    expect(chart.sahams, hasLength(41));
    expect(
      chart.sahams.every((s) => !s.strong.contains(SahamStrong.withYearLord)),
      isTrue,
    );
    final natal = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      varsha: const VarshaRequest(
        through: 1,
        sahams: Sahams.these([Saham.punya]),
      ),
    );
    expect(natal.sahams.map((s) => s.saham), [Saham.punya]);
    expect(natal.praveshas.first.annual, isNull);
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          sahams: Sahams.these([Saham.punya, Saham.punya]),
        ),
      ),
      refusedBy('varsha.sahams'),
    );
    ctx.dispose();
  });

  test("a year's chart answers the annual dashas asked for", () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<Pravesha> years(VarshaRequest varsha) =>
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: varsha,
            )
            .praveshas;
    Matcher refusedBy(String field) =>
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field));

    final asked = years(
      const VarshaRequest(
        through: 3,
        place: AnnualPlace.birth,
        dashas: AnnualDashas.these([DashaSystem.mudda, DashaSystem.patyayini]),
      ),
    );
    for (final (k, one) in asked.indexed) {
      final annual = one.annual!;
      expect(annual.dashas.map((d) => d.system), [
        DashaSystem.mudda,
        DashaSystem.patyayini,
      ]);
      for (final dasha in annual.dashas) {
        expect(dasha.year.from, one.instant);
        if (k + 1 < asked.length) {
          expect(dasha.year.to, closeTo(asked[k + 1].instant, 2e-7));
        }
        expect(dasha.firstLord, dasha.ring.shares[dasha.ring.first].lord);
        // The mahadashas run end to end across the year.
        final mahas = dasha.periods.where((p) => p.level == 1).toList();
        expect(mahas.first.from, dasha.year.from);
        expect(mahas.last.to, dasha.year.to);
        for (var i = 1; i < mahas.length; i += 1) {
          expect(mahas[i].from, mahas[i - 1].to);
        }
        // Mid-year a mahadasha runs, and one of its own under it.
        final chain = dasha.at((dasha.year.from + dasha.year.to) / 2);
        expect(chain, hasLength(2));
        expect(chain[1].path, startsWith('${chain[0].path}/'));
        expect(dasha.at(dasha.year.to), isEmpty);
      }
      final [mudda, patyayini] = annual.dashas;
      expect(mudda.seed, isNotNull);
      expect(mudda.ring.shares, hasLength(9));
      expect(mudda.ring.remaining, inInclusiveRange(0, 1));
      expect(patyayini.seed, isNull);
      expect(patyayini.ring.remaining, isNull);
      expect(patyayini.ring.shares.where((s) => s.sign != null), hasLength(1));
      expect(patyayini.periods.any((p) => p.sign != null), isTrue);
    }
    final firsts = [
      for (final one in asked) one.annual!.dashas.first.ring.first,
    ];
    for (var i = 1; i < firsts.length; i += 1) {
      expect(firsts[i], (firsts[i - 1] + 1) % 9);
    }

    // `all` is the three in the catalogue's order; unasked is none.
    expect(
      years(
        const VarshaRequest(
          through: 1,
          place: AnnualPlace.birth,
          dashas: AnnualDashas.all,
        ),
      ).first.annual!.dashas.map((d) => d.system),
      [DashaSystem.patyayini, DashaSystem.mudda, DashaSystem.varshaYogini],
    );
    expect(
      years(
        const VarshaRequest(through: 1, place: AnnualPlace.birth),
      ).first.annual!.dashas,
      isEmpty,
    );

    // The rules cross in the boundary's casing.
    final days =
        years(
          const VarshaRequest(
            through: 1,
            place: AnnualPlace.birth,
            dashas: AnnualDashas.these([DashaSystem.mudda]),
            dashaRules: AnnualDashaRules(
              clock: YearClock.days(360),
              depth: 1,
              birthPeriod: BirthPeriod.elapsed,
              measure: Balance.temporal,
              balance: MuddaBalance.entryMoon,
            ),
          ),
        ).first.annual!.dashas.single;
    expect(days.year.to - days.year.from, 360);
    expect(days.periods.every((p) => p.level == 1), isTrue);
    years(
      const VarshaRequest(
        through: 1,
        place: AnnualPlace.birth,
        dashas: AnnualDashas.all,
        dashaRules: AnnualDashaRules(
          clock: YearClock.even,
          measure: Balance.spatial,
          birthPeriod: BirthPeriod.compressed,
          balance: MuddaBalance.whole,
        ),
      ),
    );

    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          dashas: AnnualDashas.these([DashaSystem.mudda]),
        ),
      ),
      refusedBy('varsha.dashas'),
    );
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          dashas: AnnualDashas.these([DashaSystem.vimshottari]),
        ),
      ),
      refusedBy('varsha.dashas'),
    );
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          dashas: AnnualDashas.all,
          dashaRules: AnnualDashaRules(clock: YearClock.days(0)),
        ),
      ),
      refusedBy('varsha.dashaRules.clock'),
    );
    ctx.dispose();
  });

  test("a year's chart answers the Tajika yogas for the matters asked", () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<Pravesha> years(VarshaRequest varsha) =>
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: varsha,
            )
            .praveshas;

    for (final one in years(
      const VarshaRequest(
        through: 6,
        place: AnnualPlace.birth,
        matters: Matters.houses([7, 1]),
      ),
    )) {
      final annual = one.annual!;
      expect(annual.matters.map((matter) => matter.house), [7, 1]);
      expect(
        [...annual.retrograde, ...annual.combust].every((g) => g.id < 7),
        isTrue,
      );
      for (final matter in annual.matters) {
        // All sixteen are built and the façade reads the states, so every
        // one answers true or false.
        expect(matter.unanswered, isEmpty);
        expect(matter.holds(YearYoga.kuttha), isA<bool>());
        expect(matter.holds(YearYoga.ithasala), isA<bool>());
        expect(matter.karyesha != matter.lagnesha, !matter.sameLord);
        for (final held in matter.held) {
          if (held.between != null) expect(held.between, same(matter.between));
          if (held.legs != null) expect(held.legs!.length, 2);
          expect(
            held.afflictions != null,
            held.yoga == YearYoga.rudda || held.yoga == YearYoga.durapha,
          );
        }
        final pairYoga = matter.between?.yoga;
        expect(
          matter.holds(YearYoga.ithasala),
          pairYoga != null && pairYoga != TajikaYoga.ishrafa,
        );
      }
      final first = annual.matters[1];
      expect(first.sameLord, isTrue);
      expect(first.between, isNull);
      expect(
        first.held.every(
          (held) =>
              held.yoga == YearYoga.ikabala || held.yoga == YearYoga.induvara,
        ),
        isTrue,
      );
    }

    final every = years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        matters: Matters.all,
      ),
    );
    expect(every.first.annual!.matters.map((m) => m.house), [
      for (var house = 1; house <= 12; house += 1) house,
    ]);
    expect(
      years(
        const VarshaRequest(through: 2, place: AnnualPlace.birth),
      ).first.annual!.matters,
      isEmpty,
    );
    // The commentary's full Moon can only take a Kuttha away.
    int kutthas(Iterable<Pravesha> found) =>
        found
            .expand((one) => one.annual!.matters)
            .where((matter) => matter.holds(YearYoga.kuttha) ?? false)
            .length;
    final waxing = years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        matters: Matters.all,
        yogas: YogaRules(moonBenefic: MoonBenefic.waxing),
      ),
    );
    expect(kutthas(waxing), lessThanOrEqualTo(kutthas(every)));

    // The rule records cross in the boundary's casing.
    years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        varshesha: VarsheshaRules(
          noneAspects: NoneAspects.annualLagnaLord,
          tied: VarsheshaTied.dinaRatriPati,
          moon: MoonMayRule.ithasala,
          moonPartner: MoonPartner.officeBearer,
          subDegree: SubDegree.ishrafa,
        ),
      ),
    );
    years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        matters: Matters.houses([10]),
        yogas: YogaRules(
          subDegree: SubDegree.ishrafa,
          weakBelow: 4 * 3600,
          strongFrom: 12 * 3600,
          tambira: TambiraMover.eitherLord,
          moonBenefic: MoonBenefic.waxing,
        ),
      ),
    );

    Matcher refusedBy(String field) =>
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field));
    expect(
      () =>
          years(const VarshaRequest(through: 2, matters: Matters.houses([7]))),
      refusedBy('varsha.matters'),
    );
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          matters: Matters.houses([7, 7]),
        ),
      ),
      refusedBy('varsha.matters'),
    );
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          matters: Matters.houses([7]),
          yogas: YogaRules(weakBelow: 12 * 3600, strongFrom: 4 * 3600),
        ),
      ),
      refusedBy('varsha.yogas.strongFrom'),
    );
    ctx.dispose();
  });

  test('a rashifal period is read for each of the twelve signs', () {
    final ctx = teistro.context(
      profile: 'nepali-drik',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final week = RashifalRequest(
      first: Calendar.gregorian.date(2026, 10, 4),
      last: Calendar.gregorian.date(2026, 10, 10),
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
    );
    RashifalRequest like({
      CalendarDate? last,
      RashifalSnapshot? snapshot,
      List<Graha>? events,
      List<int>? spells,
    }) => RashifalRequest(
      first: week.first,
      last: last ?? week.last,
      place: week.place,
      utcOffsetSeconds: week.utcOffsetSeconds,
      snapshot: snapshot,
      events: events,
      spells: spells,
    );

    final read = ctx.chart.rashifal(week, baseline: BaselinePeriod.weekly);
    expect(
      read.provenance.inputHash,
      isNotEmpty,
      reason: 'the request is sealed',
    );
    final period = read.period;
    expect(
      [period.reference.calendar, period.reference.day],
      [Calendar.gregorian, 7],
    );
    expect([period.transits.length, period.retrograde.length], [9, 9]);
    final signs = [
      for (final r in Rashi.values)
        if (r.id >= 0) r,
    ];
    expect([for (final r in period.readings) r.rashi], signs);
    // Each sign's houses are counted from itself: the Sun's sign is the
    // same in every reading, its house one less from each next sign.
    final sun = period.transits[0].sign;
    for (final (index, reading) in period.readings.indexed) {
      expect(reading.gochar.grahas[0].graha, Graha.sun);
      expect(
        reading.gochar.grahas[0].house,
        (signs.indexOf(sun) - index) % 12 + 1,
        reason: reading.rashi.key,
      );
      expect(reading.gochar.instant, period.instant);
    }
    expect(read.baseline, hasLength(12));
    for (final score in read.baseline!) {
      expect(score.overall, inInclusiveRange(0, 100));
      expect(score.areas, hasLength(8));
    }
    expect(ctx.chart.rashifal(week).baseline, isNull);

    // 06:00 at +05:45 is 00:15 UTC on the reference day.
    final six = ctx.chart.rashifal(like(snapshot: const AtClock(hour: 6)));
    expect(six.period.instant, closeTo(2461320.5 + 15 / 1440, 1e-9));
    final mars = ctx.chart.rashifal(like(events: const [Graha.mars]));
    expect([
      for (final r in mars.period.readings)
        for (final e in r.events) e.hit.graha,
    ], everyElement(Graha.mars));
    final many = ctx.chart.rashifalMany([week]);
    final [alone] = many.value;
    expect(alone.period.instant, period.instant);
    expect(
      many.provenance.inputHash,
      ctx.chart.rashifal(week).provenance.inputHash,
    );

    for (final (request, field) in [
      (
        like(last: Calendar.gregorian.date(2026, 10, 3)),
        'rashifal.periods[0].last',
      ),
      (like(spells: const [13]), 'rashifal.periods[0].spells'),
    ]) {
      expect(
        () => ctx.chart.rashifal(request),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its dashas, their periods and the chain', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      dashas: [DashaSystem.vimshottari, DashaSystem.chara],
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .dashas,
      isEmpty,
    );
    final [dasha, chara] = chart.dashas;
    expect(dasha.system, DashaSystem.vimshottari);
    expect(dasha.balance!.method, Balance.spatial);
    expect(
      dasha.balance!.remaining,
      allOf(greaterThan(0), lessThanOrEqualTo(1)),
    );
    expect(dasha.moonSpan, isNull);
    expect(dasha.depth, 3);
    expect(dasha.periods, hasLength(9 + 81 + 729));
    final [first, second, ...] = dasha.periods;
    expect((first.path, first.level, first.from), ('0', 1, 2451545.0));
    expect(first.lord, dasha.firstLord);
    expect(
      (second.path, second.level, second.lord),
      ('0/0', 2, dasha.firstLord),
    );
    expect(dasha.periods.last.path, '8/8/8');
    expect(
      first.sign,
      isNull,
      reason: "a nakshatra-seeded period is its lord's",
    );

    // A sign-based dasha: no seed, no balance, twelve signs each divided in
    // twelve from its own sign.
    expect(chara.system, DashaSystem.chara);
    expect((chara.seed, chara.balance), (null, null));
    expect(chara.periods, hasLength(12 + 144 + 1728));
    final [maha, own, ...] = chara.periods;
    expect(
      (maha.path, own.path, own.sign, maha.from),
      ('0', '0/0', maha.sign, 2451545.0),
    );
    expect(maha.sign, isNotNull);
    expect(chara.firstLord, maha.lord);
    expect(
      {for (final p in chara.periods.where((p) => p.level == 1)) p.sign},
      hasLength(12),
      reason: 'every sign once',
    );
    expect(chara.at(2451545.0 + 5000), hasLength(3));

    const instant = 2451545.0 + 5000;
    final chain = dasha.at(instant);
    expect([for (final p in chain) p.level], [1, 2, 3]);
    expect(chain.every((p) => p.from <= instant && instant < p.to), isTrue);
    expect(dasha.at(2451544), isEmpty, reason: 'before birth');

    expect(
      () => ctx.chart.found(
        instant: 2451545.0,
        place: place,
        utcOffsetSeconds: 0,
        dashas: [DashaSystem.vimshottari, DashaSystem.panchaswara],
      ),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'dashas[1]'),
      ),
    );
    ctx.dispose();
  });
}

/// A date's calendar, year, month and day, which [CalendarDate] does not
/// compare by.
List<Object?> _dateKey(CalendarDate on) => [
  on.calendar,
  on.year,
  on.month,
  on.day,
];
