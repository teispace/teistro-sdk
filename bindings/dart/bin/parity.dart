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
/// A natal point as every runner prints it: `LAGNA`, or the graha's full
/// key.
String natalKey(NatalPoint point) => switch (point) {
  NatalLagna() => 'LAGNA',
  NatalGraha(:final graha) => graha.fullKey,
};

/// A clause as every runner prints it: 1 held, 0 not.
String flag(bool held) => held ? '1' : '0';

/// A consideration's reading as every runner prints it: its fields in
/// serde's order, a member by its full key (a boundary enum by its key)
/// and a flag as 0 or 1.
String poruthamText(PoruthamReading reading) {
  return switch (reading) {
    DhinamPorutham(:final count, :final rule) => '$count ${rule.key}',
    GanamPorutham(:final bride, :final groom, :final diminished) =>
      '${bride.fullKey} ${groom.fullKey} ${flag(diminished)}',
    MahendraPorutham(:final count) => '$count',
    DeerghaPorutham(:final count) => '$count',
    YoniPorutham(:final bride, :final groom, :final hostile) =>
      '${bride.fullKey} ${groom.fullKey} ${flag(hostile)}',
    RasiPorutham(:final apart) => '$apart',
    RasyadhipathiPorutham(
      :final bride,
      :final groom,
      :final brideCallsFriend,
      :final groomCallsFriend,
    ) =>
      '${bride.fullKey} ${groom.fullKey} ${flag(brideCallsFriend)} '
          '${flag(groomCallsFriend)}',
    VasyamPorutham(:final brideToGroom, :final groomToBride) =>
      '${flag(brideToGroom)} ${flag(groomToBride)}',
    RajjuPorutham(:final bride, :final groom) => '${bride.key} ${groom.key}',
    VedhaiPorutham(:final pierced) => flag(pierced),
  };
}

/// A koota's reading as every runner prints it: its fields in serde's
/// order, a member by its full key, a flag as 0 or 1 and no dosha as
/// `NONE`.
String readingText(KootaReading reading) {
  return switch (reading) {
    VarnaKoota(:final bride, :final groom) =>
      '${bride.fullKey} ${groom.fullKey}',
    VashyaKoota(:final relation) => relation.key,
    TaraKoota(:final brideToGroom, :final groomToBride) =>
      '$brideToGroom $groomToBride',
    YoniKoota(:final bride, :final groom, :final relation) =>
      '${bride.fullKey} ${groom.fullKey} ${relation.key}',
    MaitriKoota(:final bride, :final groom, :final relation, :final lifted) =>
      '${bride.fullKey} ${groom.fullKey} ${relation.key} ${flag(lifted)}',
    GanaKoota(:final bride, :final groom, :final dosha, :final lifted) =>
      '${bride.fullKey} ${groom.fullKey} ${flag(dosha)} ${flag(lifted)}',
    BhakootKoota(
      :final apart,
      :final dosha,
      :final exceptions,
      :final lifted,
    ) =>
      [
        '$apart',
        dosha?.key ?? 'NONE',
        flag(exceptions.oneLord),
        flag(exceptions.lordsFriends),
        flag(exceptions.navamshaLordsFriends),
        flag(exceptions.taraPure),
        flag(exceptions.vashya),
        flag(lifted),
      ].join(' '),
    NadiKoota(:final bride, :final groom, :final dosha, :final lifted) =>
      '${bride.fullKey} ${groom.fullKey} ${flag(dosha)} ${flag(lifted)}',
  };
}

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

/// The items joined by spaces, or `none`.
String listed(Iterable<String> items) {
  final joined = items.join(' ');
  return joined.isEmpty ? 'none' : joined;
}

/// Pairs in antiscion as every runner prints them, a chart's own or across
/// a synastry: their count, then each pair's planets, side, gap and orb.
void putAntiscionRows(String key, List<AntiscionRow> rows) {
  put('$key-count', '${rows.length}');
  for (final (n, row) in rows.indexed) {
    put(
      '$key-$n',
      '${row.first.fullKey} ${row.second.fullKey} ${row.contrary ? 1 : 0} '
          '${number(row.apartDeg)} ${number(row.orbDeg)}',
    );
  }
}

/// A festival answer's counts, hash, every observance and every Ekadashi
/// fast, as every runner prints them.
void putFestivals(String prefix, FestivalAnswer answer) {
  put(
    '$prefix-counts',
    '${answer.observances.length} ${answer.unjudged.length}',
  );
  put('$prefix-hash', answer.provenance.contentHash);
  for (final (k, observance) in answer.observances.indexed) {
    final decided = observance.decidedBy;
    final by = switch (decided.by) {
      'GUARD' => 'guard:${decided.index}',
      'AFTER' => 'after:${decided.rule}:${decided.days}',
      _ => 'otherwise',
    };
    final (earlier, later) = observance.extents;
    put(
      '$prefix-$k',
      [
        observance.rule,
        observance.month.fullKey,
        observance.adhika,
        '${observance.day.month}-${observance.day.day}',
        observance.case_,
        by,
        observance.choice,
        number(observance.tithi.from),
        number(earlier.held),
        number(later.held),
      ].join(' '),
    );
  }
  for (final (k, fast) in answer.ekadashis.indexed) {
    put(
      '$prefix-ekadashi-$k',
      [
        fast.rule,
        fast.tithi.fullKey,
        fast.month.fullKey,
        fast.adhika,
        '${fast.day.month}-${fast.day.day}',
        fast.piercedAt ?? '-',
        fast.pierced,
        fast.excess,
        fast.choice,
        number(fast.tithis.$2.from),
      ].join(' '),
    );
  }
}

/// A muhurta answer's counts, hash and every window, as every runner
/// prints them.
void putMuhurta(String prefix, MuhurtaAnswer answer) {
  put(
    '$prefix-counts',
    [
      answer.windows.length,
      answer.closed.length,
      answer.daysJudged,
      answer.daysCut,
      answer.windowsBlackedOut,
      answer.ranking.key,
    ].join(' '),
  );
  put('$prefix-hash', answer.provenance.contentHash);
  for (final (k, window) in answer.windows.indexed) {
    put('$prefix-$k', '${number(window.at.from)} ${number(window.at.to)}');
    put(
      '$prefix-$k-clauses',
      window.clauses.map((c) => c.kind.clause).join(' '),
    );
    put('$prefix-$k-bars', listed(window.barredBy.map((bar) => bar.clause)));
    put(
      '$prefix-$k-placed',
      listed([
        for (final c in window.clauses)
          if (c.kind case UnwantedPlacementClause(:final house, :final by))
            '$house:${by.map((g) => g.fullKey).join(',')}',
      ]),
    );
    final score = window.score;
    put(
      '$prefix-$k-score',
      score == null
          ? 'none'
          : [
            score.value,
            score.cappedAt ?? 'none',
            listed(
              score.factors.map(
                (f) =>
                    '${f.dimension}:${f.weight}:${f.graha?.fullKey ?? 'none'}',
              ),
            ),
          ].join(' '),
    );
  }
  for (final (j, day) in answer.closed.indexed) {
    put(
      '$prefix-closed-$j',
      '${day.date.month}-${day.date.day} ${listed(day.by.map((kind) => kind.fullKey))}',
    );
  }
}

/// A local day's every field, under the same keys for a chart's day and
/// an almanac's, because the two layers hand back one record.
/// An Ashta Koota as every runner prints it, under [prefix].
void putAshta(String prefix, AshtaKoota matched) {
  put('$prefix-matching', number(matched.total));
  for (final row in matched.kootas) {
    put(
      '$prefix-matching-${row.reading.koota.fullKey}',
      '${number(row.points)} ${number(row.maxPoints)} '
          '${readingText(row.reading)}',
    );
  }
}

/// Ten considerations as every runner prints them, under [prefix].
void putPorutham(String prefix, Porutham ten) {
  final e = ten.exception;
  put(
    '$prefix-porutham',
    '${ten.agreeing} ${ten.chiefAgreeing} ${flag(e.oneLord)} '
        '${flag(e.lordsFriendly)} ${flag(e.opposite)}',
  );
  for (final row in ten.considerations) {
    put(
      '$prefix-porutham-${row.reading.koota.fullKey}',
      '${flag(row.agrees)} ${flag(row.lifted)} '
          '${poruthamText(row.reading)}',
    );
  }
}

/// Two pairs of names, as every runner asks them: a Devanagari pair whose
/// groom's syllable is Abhijit's, placed in Shravana, and an IAST pair.
void putNaam(Context ctx) {
  const pairs = [
    (
      'प्रिया',
      'ज़ोया',
      NaamRules(
        name: NameRules(abhijit: AbhijitPada.shravana),
        koota: KootaRules(nadiDosha: NadiDosha.middleOnly),
      ),
    ),
    (
      'kṛṣṇā',
      'śyāma',
      NaamRules(
        name: NameRules(latin: LatinName.iast),
        porutham: PoruthamRules(deerghaBeyond: DeerghaBeyond.seventh),
      ),
    ),
  ];
  for (final (n, (bride, groom, rules)) in pairs.indexed) {
    final read = ctx.matching.naam(bride, groom, rules);
    for (final (who, name) in [('bride', read.bride), ('groom', read.groom)]) {
      put(
        'naam-$n-$who',
        '${name.cell} ${name.nakshatra?.fullKey ?? 'NONE'} ${name.quarter} '
            '${name.varga.key}',
      );
    }
    final varga = read.varga;
    put(
      'naam-$n-varga',
      '${varga.bride.key} ${varga.groom.key} ${varga.relation.key}',
    );
    putAshta('naam-$n', read.ashta);
    putPorutham('naam-$n', read.porutham);
  }
}

void putDay(String prefix, LocalDay day) {
  put('$prefix-vara', day.vara.fullKey);
  put('$prefix-sunrise', day.sunrise);
  put('$prefix-sunset', day.sunset);
  put('$prefix-next-sunrise', day.nextSunrise);
  put('$prefix-date', '${day.date.year}-${day.date.month}-${day.date.day}');
  put('$prefix-calendar', day.date.calendar.fullKey);
  put('$prefix-era', day.date.era?.fullKey ?? 'none');
  put('$prefix-era-year', day.date.eraYear);
  put('$prefix-resolution', day.date.resolution.key);
  final polar = day.polar;
  put(
    '$prefix-polar',
    polar == null ? 'none' : '${polar.kind.key}/${polar.policy.key}',
  );
  final air = day.air;
  put(
    '$prefix-convention',
    air != null
        ? '${day.convention?.key} ${number(air.pressureHpa)} hPa ${number(air.temperatureC)} C'
        : day.convention?.key ?? 'custom ${number(day.customAltitudeDeg ?? 0)}',
  );
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
  put('locale', ctx.intl.locale);
  put('settings-hash', ctx.settingsHash);
  put('settings-fnv', fnv(ctx.settingsJson));
  putNaam(ctx);

  // ── The calendars ────────────────────────────────────────────────────
  final date = Calendar.gregorian.date(2015, 4, 14);
  final bs = ctx.calendar.convert(date, Calendar.bikramSambat);
  put('bs-year', bs.year);
  put('bs-month', bs.month);
  put('bs-day', bs.day);
  put('bs-era', bs.era?.fullKey);
  put('bs-era-year', bs.eraYear);
  put('bs-resolution', bs.resolution.key);
  final fixed = ctx.calendar.fixedOf(date);
  put('fixed', fixed);
  put('weekday', ctx.calendar.weekdayOf(date));
  put('month-length', ctx.calendar.monthLength(Calendar.gregorian, 2024, 2));
  put('is-leap', ctx.calendar.isLeap(Calendar.gregorian, 2024));
  put('jd-of-fixed', teistro.julianDayOfFixed(fixed));
  final back = teistro.fixedOfJulianDay(2457126.75);
  put('fixed-of-jd', back.value);
  put('fraction-of-jd', back.fraction);

  // ── Time ─────────────────────────────────────────────────────────────
  final civil = Calendar.gregorian.date(1986, 1, 1).at(hour: 0, minute: 20);
  final zone = ianaZone('Asia/Kathmandu');
  final resolved = ctx.time.resolve(civil, zone);
  put('resolve-jd', resolved.instantJdUtc);
  put('resolve-offset', resolved.offsetSeconds);
  put('resolve-era', resolved.era.key);
  put('resolve-source', resolved.source.key);
  put('resolve-time-known', resolved.timeKnown);
  put('resolve-tzdb', resolved.tzdbVersion);
  put('resolve-warnings', resolved.warnings.length);
  final civilBack = ctx.time.civilOf(
    resolved.instantJdUtc,
    zone,
    Calendar.gregorian,
  );
  put('civil-year', civilBack.civil.date.year);
  put('civil-minute', civilBack.civil.time.minute);
  put('civil-offset', civilBack.resolution.offsetSeconds);
  final tt = ctx.time.convert(2451544.5, Scale.utc, Scale.tt);
  put('tt-jd', tt.jd);
  put('tt-delta-t', tt.deltaTSeconds);
  put('tt-delta-t-source', tt.deltaTSource.key);
  put('tt-delta-t-model', tt.deltaTModel);
  final delta = ctx.time.deltaT(2451544.5);
  put('delta-t-seconds', delta.seconds);
  put('delta-t-source', delta.source.key);

  // ── Keys ─────────────────────────────────────────────────────────────
  final id = ctx.keys.id('graha.SUN');
  put('key-id', id);
  put('key-name', ctx.keys.name(id));
  try {
    ctx.keys.id('graha.SUNN');
    put('refusal', 'none');
  } on TeistroException catch (error) {
    put('refusal-status', error.status.key);
    put('refusal-detail', error.detail);
    put('refusal-hint-names-sun', error.hint?.contains('SUN'));
  }

  // ── The locale engine ────────────────────────────────────────────────
  final rendered = ctx.intl.render('sdk.reason.grahaInBhava', {
    'graha': {r'$entity': 'graha.JUPITER'},
    'bhava': 7,
  });
  put('render-fnv', fnv(rendered.text));
  put('render-length', rendered.text.runes.length);
  put('render-resolved-from', rendered.resolvedFrom);
  put('render-fallback', rendered.fallback);
  // A rendered message's parts, which is what a rich renderer walks.
  // `sdk.reason.lordship` is one of the two shipped messages carrying
  // `{#b}`; the plain one beside it holds every binding to the rule that
  // no markup means the one text part, made rather than carried.
  String partShape(List<MessagePart> parts) => parts
      .map(
        (part) =>
            part.isText
                ? 'text:${part.value}'
                : '${part.kind}:${part.name}('
                    '${(part.options.entries.toList()..sort((a, b) => a.key.compareTo(b.key))).map((o) => '${o.key}=${o.value}').join(',')})',
      )
      .join('|');
  final rich = ctx.intl.render('sdk.reason.lordship', {
    'graha': {r'$entity': 'graha.JUPITER'},
    'bhava': 5,
  });
  put('render-rich-parts', partShape(rich.partList));
  put('render-plain-parts', partShape(rendered.partList));
  put('has-message', ctx.intl.has('sdk.reason.grahaInBhava'));
  put('has-missing-message', ctx.intl.has('sdk.nope.missing'));
  put('transliterated', ctx.intl.transliterate('सूर्य बृहस्पति'));
  put('entity-sun-name', ctx.intl.entity('graha.SUN').name);
  put('entity-sun-iast', ctx.intl.entity('graha.SUN').iast);
  put('entity-sun-glyph', ctx.intl.entity('graha.SUN').glyph);
  put('entity-sun-gender', ctx.intl.entity('graha.SUN').gender?.key);
  put(
    'message-graha-in-bhava',
    ctx.intl.messages.sdk.reason.grahaInBhava(
      graha: intl.GrahaKey.jupiter,
      bhava: 7,
    ),
  );
  put(
    'message-bs-date',
    ctx.intl.messages.sdk.calendar.bikramSambat.date.long(
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
        .map((step) => '${step.name}:${step.implementation.key}')
        .join(','),
  );
  put('provenance-fnv', fnv(positions.provenanceJson));
  put('provenance-profile', positions.provenance.profile);
  put('provenance-settings-hash', positions.provenance.settingsHash);
  put('provenance-provider-frame', positions.provenance.provider.frame);

  // ── The chart the topocentric profile founds ─────────────────────────
  // The scenario above runs under `nepali-default`, whose frame is
  // **topocentric** — inherited from the baseline engine, and what every
  // recorded chart in the corpus is. Until the completion's centre step
  // this could not found a chart at all, and the refusal was what the
  // three bindings compared. Now the chart itself is, which is the
  // stronger comparison: the step runs per body, per instant, inside the
  // library, so three bindings agreeing on its output is three bindings
  // agreeing on the whole of it.
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  final placed = ctx.chart.found(
    instant: 2451545,
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('chart-under-topocentric', 'founded');
  put('topocentric-steps', placed.batch.stepsApplied.join(','));
  put('topocentric-lagna', placed.lagnaDeg);
  var j = 0;
  for (final graha in placed.grahas) {
    put('topocentric-graha-$j', graha.graha.fullKey);
    put('topocentric-graha-$j-lon', graha.longitudeDeg);
    put('topocentric-graha-$j-lat', graha.latitudeDeg);
    put('topocentric-graha-$j-speed', graha.speedDegPerDay);
    j += 1;
  }

  // ── A chart founded on a classical astronomy ─────────────────────────
  // The Surya Siddhanta by name: the text's zodiac, places, Lagna and day
  // (docs/03-design/classical-chart.md), which every binding reaches
  // through the selector and must read back alike, deviation and all.
  final classical = teistro.context(
    profile: 'surya-siddhanta',
    ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
  );
  final text = classical.chart.found(
    instant: 2447995.4895833335,
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('classical-steps', text.batch.stepsApplied.join(','));
  put('classical-lagna', text.lagnaDeg);
  put('classical-sunrise', text.day.sunrise);
  final deviation = text.provenance.deviation!;
  put('classical-deviation', '${deviation.model}: ${deviation.detail}');
  var k = 0;
  for (final graha in text.grahas) {
    put('classical-graha-$k-lon', graha.longitudeDeg);
    k += 1;
  }
  classical.dispose();

  // ── A chart and an almanac, under a geocentric profile ───────────────
  // Everything after this runs on the SDK's own default profile, which is
  // geocentric, so that the two centres are both exercised.

  // A layout of the consumer's own, registered on the context the charts
  // are drawn under: the South Indian row renamed, as every runner
  // registers it (`03-design/chart-geometry.md` §7f).
  final shipped = teistro.context(testProvider: true);
  final kerala = shipped.chart
      .layout(ChartLayout.southIndian)
      .copyWith(key: 'ACME_KERALA');
  shipped.dispose();
  // A dasha system of the consumer's own, the same definition every runner
  // registers (`03-design/dasha-kernels.md`).
  const parityDasha = UduDashaDefinition(
    key: 'ACME_PARITY',
    sources: ['the parity scenario'],
    lords: [
      DashaLord(Graha.sun, 5),
      DashaLord(Graha.moon, 10),
      DashaLord(Graha.mars, 7),
      DashaLord(Graha.mercury, 12),
    ],
    reference: Nakshatra.mula,
    count: 'TO_REFERENCE',
    span: 2,
    offset: 1,
    repeats: true,
    yearLength: 'SAVANA_360',
    depth: 2,
  );
  final geo = teistro.context(
    profile: 'parashari-classical',
    locale: 'ne-Deva-NP',
    testProvider: true,
    layouts: [kerala],
    dashaSystems: [parityDasha],
  );
  put('geo-profile', geo.profile);
  put('geo-settings-hash', geo.settingsHash);

  // ── Charts ───────────────────────────────────────────────────────────
  // Two instants, so a per-chart section that ran charts-outermost the
  // wrong way round shows up as the second chart's values in the first's
  // place rather than as nothing at all.
  // Two divisional charts asked for, and two rather than one because
  // the layout is charts outermost then charts asked for: only two of
  // each can catch a transposed stride.
  final charts = geo.chart.foundMany(
    instants: <double>[2460482.5, 2460600.25],
    place: place,
    utcOffsetSeconds: 20700,
    vargas: <Varga>[Varga.d9, Varga.d10],
    dashas: [
      DashaSystem.vimshottari,
      DashaSystem.chara,
      DashaSystem.kalachakra,
      DashaSystem.releasingFortune,
      DashaSystem.profection,
      DashaSystem.firdaria,
      DashaSystem.decennials,
      DashaSystem.registered('ACME_PARITY'),
    ],
    drawings: [
      (ChartLayout.northIndian, Varga.d1),
      (ChartLayout.southIndian, Varga.d9),
      (ChartLayout.westernWheel, Varga.d1),
      (ChartLayout.registered('ACME_KERALA'), Varga.d9),
    ],
    theme: ChartTheme.dark,
    rules: const RuleRequest(
      shipped: [ShippedRules.nabhasas],
      longevity: true,
      ayurdaya: AyurdayaRules.parijata,
    ),
    // Every composer, so the four agree on what every chart *says* and not
    // only on what it computes (`03-design/plans-at-the-boundary.md`).
    interpret: const PlanRequest(
      placements: true,
      readings: true,
      strength: true,
      houses: true,
      positions: true,
      aspects: true,
      conditions: true,
      karakas: true,
    ),
    aspects: true,
    points: true,
    houses: true,
    ashtakavarga: true,
    vimshopaka: true,
    vaiseshikamsa: true,
    dashaPhala: true,
    jaimini: true,
    avakahada: true,
    outerPlanets: true,
    gochar: const GocharRequest(
      instants: [2460676.5, 2460736.5],
      ashtakavarga: true,
    ),
    hits: const HitRequest(
      from: 2460676.5,
      to: 2460736.5,
      grahas: [Graha.sun, Graha.mercury, Graha.saturn],
      aspects: [0, 90, 180],
      orbDeg: 2,
    ),
    sadeSati: const SadeSatiRequest(
      from: 2460676.5,
      to: 2464329,
      reckoning: Reckoning.degree,
      spells: [4, 7, 8],
    ),
    kp: const KpRequest(number: 74, anyAyanamsha: true),
    fortitudes: const FortitudeRequest(
      dignities: DignityRequest(
        sectRule: SectRule.daylight,
        terms: Terms.egyptian,
        triplicities: Triplicities.ptolemy,
        scores: DignityScores(peregrine: 0),
      ),
      rules: AccidentalRules(
        beamsDeg: 15,
        combustionInSign: false,
        partile: Partile.within,
        partileOrbDeg: 1,
        siege: Siege.within,
        siegeSpanDeg: 30,
      ),
      scores: AccidentalScores(regulus: 5),
      almuten: AlmutenRules(fortune: FortuneRule.reversedByNight),
    ),
    lots: const LotRequest(fortune: FortuneRule.reversedWhileMoonUp),
    considerations: const ConsiderationRules(moonLateFromDeg: 25),
    perfection: const PerfectionRequest.ofHouse(
      7,
      rules: PerfectionRules(horizonDays: 120),
    ),
    westernAspects: const WesternAspectRequest(
      aspects: [
        WesternAspect.conjunction,
        WesternAspect.sextile,
        WesternAspect.square,
        WesternAspect.trine,
        WesternAspect.quincunx,
        WesternAspect.opposition,
      ],
      orbs: OrbModel.moieties({
        Graha.sun: 17,
        Graha.moon: 12.5,
        Graha.mercury: 7,
        Graha.venus: 8,
        Graha.mars: 7.5,
        Graha.jupiter: 12,
        Graha.saturn: 10,
        Graha.uranus: 5,
        Graha.neptune: 5,
        Graha.pluto: 5,
      }),
    ),
    parallels: const ParallelRequest(orbDeg: 1.5),
    antiscia: const AntisciaRequest(cusps: WesternHouseRequest()),
    midpoints: const MidpointRequest(orbDeg: 1.5),
    westernHouses: const WesternHouseRequest(),
    harmonic: const HarmonicRequest(5),
    matching: MatchingRequest(
      Partner(
        instant: 2451545.25,
        place: Observer(
          latitudeDeg: Latitude(-33.87),
          longitudeDeg: Longitude(151.21),
          altitudeM: Altitude(0),
        ),
        utcOffsetSeconds: 36000,
      ),
      partnerRole: MatchRole.bride,
      rules: const KootaRules(bhakootLift: BhakootLift.garga),
      porutham: const PoruthamRules(lordsFriendship: LordsFriendship.oneWay),
      kuja: const KujaRules(
        houses: KujaHouses.withSecond,
        from: KujaFrom.lagnaMoonVenus,
      ),
    ),
    synastry: SynastryRequest(
      Partner(
        instant: 2451545.25,
        place: Observer(
          latitudeDeg: Latitude(-33.87),
          longitudeDeg: Longitude(151.21),
          altitudeM: Altitude(0),
        ),
        utcOffsetSeconds: 36000,
      ),
      table: const WesternAspectRequest(
        aspects: [
          WesternAspect.conjunction,
          WesternAspect.square,
          WesternAspect.trine,
          WesternAspect.opposition,
        ],
      ),
      zodiac: SynastryZodiac.charts,
      parallels: const ParallelRequest(orbDeg: 1.5),
      antiscia: const AntisciaRequest(orbs: OrbModel.leo),
      midpoints: const MidpointRequest(orbDeg: 1.5),
      composite: true,
      davison: true,
    ),
    progressions: const ProgressionsRequest(
      at: 2470000.5,
      year: YearMeasure.noonSiderealTime,
      angles: AngleMethod.solarArcLongitude,
      direction: DirectionArc.naibod,
      contacts: ProgressionContacts(
        from: 2462000.5,
        to: 2465652.5,
        grahas: [Graha.moon, Graha.sun],
        points: [NatalLagna(), NatalGraha(Graha.mars), NatalGraha(Graha.venus)],
        aspects: [0, 45, 90, 135, 180],
      ),
    ),
    shadbala: true,
    bhavaBala: true,
    state: true,
  );
  put('chart-varga-count', charts.vargaCount);
  put('chart-drishti-table', charts.drishtiTable);
  put('chart-count', charts.chartCount);
  put('chart-kind', ChartKind.byId(charts.kind).fullKey);
  put('chart-place-lat', charts.latitudeDeg);
  put('chart-place-lon', charts.longitudeDeg);
  put('chart-model-fnv', fnv(charts.model));
  put('chart-steps', charts.stepsApplied.join(','));
  put('chart-provenance-fnv', fnv(charts.provenanceJson));
  put('chart-provenance-profile', charts.provenance.profile);
  put('chart-graha-count', charts.grahaCount);

  for (final chart in charts.each) {
    final i = chart.index;
    put('chart-$i-content-hash', chart.provenance.contentHash);
    put('chart-$i-instant', chart.instant);
    put('chart-$i-lagna', chart.lagnaDeg);
    put('chart-$i-day-lagna', chart.dayLagnaDeg);
    put('chart-$i-ayanamsha', chart.ayanamshaOffsetDeg);
    put('chart-$i-day-part', chart.dayPart.key);
    put('chart-$i-day-elapsed', chart.dayElapsed);
    putDay('chart-$i', chart.day);
    final timing = chart.timing;
    put('chart-$i-ghati', timing.ghati);
    put('chart-$i-pala', timing.pala);
    put('chart-$i-vipala', timing.vipala);
    put('chart-$i-hora-number', timing.horaNumber);
    put('chart-$i-hora-lord', timing.horaLord.fullKey);
    final states = chart.states;
    for (var j = 0; j < states.length; j += 1) {
      final state = states[j];
      final key = 'chart-$i-state-$j';
      put(key, state.graha.fullKey);
      put('$key-sign', state.sign.fullKey);
      put('$key-house', state.house);
      put('$key-dignity', state.dignity.fullKey);
      put('$key-natural', state.friendship.natural.fullKey);
      put('$key-compound', state.friendship.compound.fullKey);
      put('$key-dispositor', state.friendship.dispositor?.fullKey ?? 'none');
      put('$key-burning', state.combustion.burning.key);
      put('$key-from-sun', state.combustion.fromSunDeg ?? 'none');
      put('$key-orb', state.combustion.orbDeg ?? 'none');
      put('$key-age', state.age.fullKey);
      put('$key-wakefulness', state.wakefulness.fullKey);
      put('$key-deeptadi', state.deeptadi?.fullKey ?? 'none');
      final sayanadi = state.sayanadi;
      put(
        '$key-sayanadi',
        sayanadi == null
            ? 'none'
            : '${sayanadi.avastha.fullKey} '
                '${sayanadi.cheshtas.map((c) => c.fullKey).join(',')}',
      );
      put(
        '$key-holding',
        state.lajjitadi.holding.isEmpty
            ? 'none'
            : state.lajjitadi.holding.map((m) => m.fullKey).join(','),
      );
      put(
        '$key-undecided',
        state.lajjitadi.undecided.isEmpty
            ? 'none'
            : state.lajjitadi.undecided.map((m) => m.fullKey).join(','),
      );
      put(
        '$key-war',
        state.war == null
            ? 'none'
            : '${state.war!.opponent.fullKey}:${state.war!.isWinner}',
      );
      put('$key-sign-edge', state.boundaries.signDeg);
    }
    for (final bhava in chart.bhavas) {
      put('chart-$i-bhava-${bhava.number}-sign', bhava.sign.fullKey);
      put('chart-$i-bhava-${bhava.number}-lord', bhava.lord.fullKey);
      put('chart-$i-bhava-${bhava.number}-quadrant', bhava.quadrant.key);
    }
    final found = chart.points;
    put('chart-$i-point-count', found.length);
    for (var k = 0; k < found.length; k += 1) {
      final one = found[k];
      put('chart-$i-point-$k', one.point.fullKey);
      put('chart-$i-point-$k-lon', one.longitudeDeg);
      put('chart-$i-point-$k-sign', one.sign.fullKey);
      put('chart-$i-point-$k-sign-edge', one.boundaries.signDeg);
    }
    // Every drishti, because the count differs from chart to chart --
    // which is why the section is ragged.
    final drishti = chart.aspects;
    put('chart-$i-aspect-count', drishti.length);
    for (var k = 0; k < drishti.length; k += 1) {
      final one = drishti[k];
      put('chart-$i-aspect-$k', '${one.from.fullKey}>${one.to.fullKey}');
      put('chart-$i-aspect-$k-houses', one.houses);
      put('chart-$i-aspect-$k-strength', one.strength.key);
      put('chart-$i-aspect-$k-from-sign', one.fromEdge.signDeg);
      put('chart-$i-aspect-$k-to-sign', one.toEdge.signDeg);
    }
    final answered = chart.rules!;
    put(
      'chart-$i-rules-present',
      (answered['present']! as List<Object?>)
          .map((held) => (held! as Map<String, Object?>)['rule'])
          .join(','),
    );
    final ayurdaya =
        (answered['longevity']! as Map<String, Object?>)['ayurdaya']!
            as Map<String, Object?>;
    put(
      'chart-$i-rules-pindayu',
      (ayurdaya['pindayu']! as Map<String, Object?>)['years'],
    );
    // **Every item said**, not merely counted: the only place the four
    // bindings are compared on text, which exercises the composers, the
    // params shape and the locale engine at once.
    for (final composed in chart.plans!.entries) {
      final items =
          (composed.value! as List<Object?>).cast<Map<String, Object?>>();
      put('chart-$i-plan-${composed.key}-count', items.length);
      for (var n = 0; n < items.length; n += 1) {
        final key = items[n]['key']! as String;
        final said =
            ctx.intl
                .render(key, items[n]['params']! as Map<String, Object?>)
                .text;
        put('chart-$i-plan-${composed.key}-$n', '$key: $said');
      }
    }
    final drawings = chart.drawings;
    for (var d = 0; d < drawings.length; d += 1) {
      final drawing = drawings[d];
      final key = 'chart-$i-drawing-$d';
      put(key, drawing.layout.fullKey);
      put('$key-varga', drawing.varga.fullKey);
      put('$key-cells', drawing.cells.length);
      put('$key-frames', drawing.frame.length);
      put('$key-marks', drawing.marks.length);
      put('$key-svg', drawing.svg);
      for (var c = 0; c < drawing.cells.length; c += 1) {
        final cell = drawing.cells[c];
        final at = '$key-cell-$c';
        put('$at-sign', cell.sign.fullKey);
        put('$at-house', cell.house);
        put('$at-lagna', cell.lagna);
        put('$at-ring', cell.ring);
        put('$at-bodies', cell.bodies.isEmpty ? 'none' : cell.bodies.join(','));
        put('$at-label', '${number(cell.label.x)},${number(cell.label.y)}');
        put('$at-anchor', '${number(cell.anchor.x)},${number(cell.anchor.y)}');
        put(
          '$at-start',
          '${number(cell.outline.start.x)},${number(cell.outline.start.y)}',
        );
        put(
          '$at-steps',
          cell.outline.segments
              .map(
                (step) => switch (step) {
                  QuadSegment() => 'QUAD',
                  ArcSegment() => 'ARC',
                  LineSegment() => 'LINE',
                },
              )
              .join(','),
        );
      }
      for (var m = 0; m < drawing.marks.length; m += 1) {
        final mark = drawing.marks[m];
        final at = '$key-mark-$m';
        put(at, mark.body);
        put('$at-at', '${number(mark.at.x)},${number(mark.at.y)}');
        put('$at-lon', mark.longitudeDeg);
      }
    }
    put('chart-$i-dasha-count', chart.dashas.length);
    final av = chart.ashtakavarga!;
    put('chart-$i-ashtakavarga', '${av.shodhana.key} ${av.ekadhipatya.key}');
    for (final g in av.grahas) {
      final key = 'chart-$i-ashtakavarga-${g.graha.fullKey}';
      put(key, g.bindus.join(','));
      put('$key-reduced', g.reduced?.join(','));
      put('$key-pindas', '${g.rashiPinda},${g.grahaPinda},${g.yogaPinda}');
    }
    put(
      'chart-$i-sarvashtakavarga',
      [av.sarva, av.trikona, av.reduced].map((row) => row.join(',')).join(';'),
    );
    for (final g in chart.shadbala!.grahas) {
      final key = 'chart-$i-shadbala-${g.graha.fullKey}';
      final (st, ka) = (g.sthana, g.kaala);
      put(
        key,
        [
          st.uchcha,
          st.saptavargaja,
          st.ojayugma,
          st.kendradi,
          st.drekkana,
          g.dig,
          ka.nathonnatha,
          ka.paksha,
          ka.tribhaga,
          ka.abda,
          ka.masa,
          ka.vara,
          ka.hora,
          ka.ayana,
          ka.yuddha,
          g.cheshta,
          g.naisargika,
          g.drik,
        ].map(number).join(','),
      );
      put(
        '$key-total',
        '${number(g.virupas)},${number(g.rupas)},${number(g.requiredRupas)},${g.strong},${number(g.ishta)},${number(g.kashta)},${number(g.subhaRashmi)},${number(g.ashubhaRashmi)}',
      );
    }
    for (final b in chart.bhavaBala!.bhavas) {
      put(
        'chart-$i-bhava-bala-${b.bhava}',
        '${b.lord.fullKey} ${[b.adhipati, b.dig, b.drishti, b.special, b.virupas].map(number).join(',')}',
      );
    }
    for (final g in chart.vaiseshikamsa!.grahas) {
      final standings = [
        g.shadvarga,
        g.saptavarga,
        g.dashavarga,
        g.shodashavarga,
      ];
      put(
        'chart-$i-vaiseshikamsa-${g.graha.fullKey}',
        '${standings.map((s) => '${s.goodVargas}:${s.name?.fullKey}').join(',')} ${g.impaired}',
      );
    }
    for (final g in chart.dashaPhala!.grahas) {
      put(
        'chart-$i-dasha-phala-${g.graha.fullKey}',
        '${g.subhankas.map(number).join(',')} ${g.nature.fullKey} ${g.phase.key} ${g.favourable} ${g.unfavourable}',
      );
    }
    final jr = chart.jaimini!;
    final k = jr.karakamsha;
    final b = jr.brahma;
    put(
      'chart-$i-jaimini',
      '${k.atmakaraka.fullKey} ${k.sign.fullKey} ${k.inRasi.join(',')} ${k.inNavamsha.join(',')}',
    );
    put(
      'chart-$i-graha-arudhas',
      jr.grahaArudhas.map((sign) => sign?.fullKey ?? '-').join(','),
    );
    final birth = chart.avakahada!;
    put(
      'chart-$i-avakahada',
      '${birth.nakshatra.fullKey} ${birth.pada} ${birth.rashi.fullKey} '
          '${birth.nakshatraLord.fullKey} ${birth.rashiLord.fullKey} ${birth.varna.fullKey} '
          '${birth.yoni.fullKey} ${birth.gana.fullKey} ${birth.nadi.fullKey}',
    );
    final syllable = birth.syllable;
    put(
      'chart-$i-avakahada-syllable',
      '${syllable.cell} ${syllable.devanagari} ${syllable.iast} ${syllable.varga.key}',
    );
    final qualified = b.qualified.map((g) => g.fullKey).join(',');
    put(
      'chart-$i-brahma',
      '${b.rule.key} ${b.countedFrom.fullKey} ${qualified.isEmpty ? '-' : qualified} '
          '${b.graha?.fullKey ?? '-'} ${b.passedFrom?.fullKey ?? '-'} ${b.none?.key ?? '-'}',
    );
    for (final (at, reading) in chart.gochar.indexed) {
      final ref = reading.reference;
      put(
        'chart-$i-gochar-$at',
        '${number(reading.instant)} ${ref.from.key} ${ref.sign.fullKey} '
            '${reading.rules.nodeVedha.key} ${reading.rules.nodeObstruction.key} '
            '${reading.rules.ashtakavargaGoodFrom.key}',
      );
      for (final (k, a) in (reading.ashtakavarga ?? const []).indexed) {
        put(
          'chart-$i-gochar-$at-av-$k',
          '${a.graha.fullKey} ${a.bindus} ${a.good} ${a.kakshya.index} '
              '${a.kakshya.lord.key} ${a.kakshyaBindu} ${a.sarva} ${a.sarvaStanding.key}',
        );
      }
      for (final (k, g) in reading.grahas.indexed) {
        final by = g.obstructedBy.map((o) => o.fullKey).join(',');
        put(
          'chart-$i-gochar-$at-$k',
          '${g.graha.fullKey} ${g.transit.sign.fullKey} ${number(g.transit.degrees)} '
              '${g.house} ${g.goodHouse} ${g.vedhaHouse ?? '-'} ${by.isEmpty ? '-' : by} '
              '${g.verdict.key} ${g.fruition.key} ${g.fruitfulNow}',
        );
      }
    }
    for (final (k, hit) in chart.hits.indexed) {
      final e = hit.event;
      final (into, motion, to, angle, phase) = switch (e) {
        SignIngress(:final into, :final motion) => (
          into.fullKey,
          motion.key,
          '-',
          '-',
          '-',
        ),
        NakshatraIngress(:final into, :final motion) => (
          into.fullKey,
          motion.key,
          '-',
          '-',
          '-',
        ),
        Station(:final turns) => ('-', turns.key, '-', '-', '-'),
        AspectHit(:final to, :final angle, :final phase, :final motion) => (
          '-',
          motion.key,
          natalKey(to),
          '$angle',
          phase.key,
        ),
      };
      put(
        'chart-$i-hit-$k',
        '${number(hit.instant)} ${hit.graha.fullKey} ${e.kind.key} $into $motion $to $angle $phase',
      );
    }
    final ss = chart.sadeSati!;
    put(
      'chart-$i-sade-sati',
      '${ss.reference.from.key} ${ss.reference.sign.fullKey} ${ss.reckoning.key}',
    );
    String bound(double? jd) => jd == null ? '-' : number(jd);
    final periods = [
      for (final one in ss.sadeSati) one.phases,
      for (final spell in ss.spells) [spell],
    ];
    final lines = [
      for (final (period, spells) in periods.indexed)
        for (final spell in spells)
          for (final v in spell.visits)
            '$period ${spell.house} ${bound(v.from)} ${bound(v.to)}',
    ];
    for (final (k, line) in lines.indexed) {
      put('chart-$i-sade-sati-$k', line);
    }
    final kp = chart.kp!;
    String keys(List<KeyOf<Object>> members) =>
        members.isEmpty ? '-' : members.map((m) => m.fullKey).join(',');
    String level(KpLevel at) =>
        '${at.lord.fullKey} ${at.span.start} ${at.span.end}';
    String lords(KpLords of) =>
        '${of.sign.fullKey} ${level(of.star)} ${level(of.sub)} ${level(of.subSub)}';
    String rejection(KpRejection? by) =>
        by == null ? '-' : '${by.retrograde.fullKey}:${by.byStar}';
    final rules = kp.ruling.rules;
    put(
      'chart-$i-kp',
      '${kp.chart.system.fullKey} ${rules.count} ${rules.nodeRulers} ${rules.retrogradeRejection}',
    );
    for (final cusp in kp.chart.cusps) {
      put(
        'chart-$i-kp-cusp-${cusp.house}',
        '${cusp.longitude} ${lords(cusp.lords)}',
      );
    }
    for (final p in kp.chart.planets) {
      put(
        'chart-$i-kp-planet-${p.graha.fullKey}',
        '${p.longitude} ${p.retrograde} ${p.house} ${lords(p.lords)}',
      );
    }
    for (final h in kp.significators.houses) {
      put(
        'chart-$i-kp-house-${h.house}',
        '${keys(h.inOccupantsStars)} ${keys(h.occupants)} ${keys(h.inLordsStar)} ${h.lord.fullKey} '
            '${keys(h.conjoined)} ${keys(h.aspected)} ${keys(h.intercepted)}',
      );
    }
    for (final n in kp.significators.nodes) {
      put(
        'chart-$i-kp-node-${n.node.fullKey}',
        '${keys(n.conjoined)} ${n.starLord.fullKey} ${keys(n.aspecting)} ${n.signLord.fullKey}',
      );
    }
    for (final (k, r) in kp.ruling.rulers.indexed) {
      final reasons = [
        for (final why in r.reasons)
          why.of == null ? why.kind : 'AGENT:${why.of!.fullKey}:${why.by}',
      ].join(',');
      put(
        'chart-$i-kp-ruler-$k',
        '${r.graha.fullKey} $reasons ${r.retrograde} ${rejection(r.rejectedBy)} ${rejection(r.rejectedBySub)}',
      );
    }
    final dg = chart.dignities!;
    final sc = dg.scores;
    put(
      'chart-$i-dignities',
      '${dg.sect.key} ${dg.sectRule.key} ${dg.rules.terms.key} '
          '${dg.rules.triplicities.key} '
          '${[sc.house, sc.exaltation, sc.triplicity, sc.term, sc.face, sc.detriment, sc.fall, sc.peregrine].join(',')}',
    );
    List<String> flagsHeld(EssentialDignity d) => [
      if (d.house) 'house',
      if (d.exaltation) 'exaltation',
      if (d.triplicity) 'triplicity',
      if (d.term) 'term',
      if (d.face) 'face',
      if (d.detriment) 'detriment',
      if (d.fall) 'fall',
    ];
    String listed(List<String> names) => names.isEmpty ? '-' : names.join(',');
    for (final at in dg.planets) {
      final held = [...flagsHeld(at.dignity), if (at.peregrine) 'peregrine'];
      put(
        'chart-$i-dignity-${at.planet.fullKey}',
        '${number(at.longitudeDeg)} ${listed(held)} ${at.score} ${at.reception}',
      );
    }
    for (final (k, one) in dg.receptions.indexed) {
      put(
        'chart-$i-reception-$k',
        '${one.planets.$1.fullKey} ${one.planets.$2.fullKey} '
            '${flagsHeld(one.firstIn).join(',')} '
            '${flagsHeld(one.secondIn).join(',')} '
            '${listed([for (final kind in one.mutual) kind.name])}',
      );
    }
    final ft = chart.fortitudes!;
    String numbers(List<double> values) => values.map(number).join(',');
    put(
      'chart-$i-fortitudes',
      '${ft.sky.houses.fullKey} '
          '${[ft.sky.northNodeDeg, ft.sky.regulusDeg, ft.sky.spicaDeg, ft.sky.algolDeg].map(number).join(' ')}',
    );
    final fr = ft.rules;
    final partile =
        fr.partile == Partile.within
            ? 'WITHIN:${number(fr.partileOrbDeg)}'
            : fr.partile.key;
    final siege =
        fr.siege == Siege.within
            ? 'WITHIN:${number(fr.siegeSpanDeg)}'
            : fr.siege.key;
    put(
      'chart-$i-fortitude-rules',
      '${number(fr.combustionDeg)} ${fr.combustionInSign ? 1 : 0} '
          '${[fr.beamsDeg, fr.cazimiDeg, fr.cuspOrbDeg, fr.starOrbDeg].map(number).join(' ')} '
          '$partile $siege ${numbers(fr.meanMotionDeg)}',
    );
    put(
      'chart-$i-fortitude-scores',
      '${ft.scores.houses.join(',')} ${ft.scores.lines.values.join(',')}',
    );
    put('chart-$i-fortitude-houses', numbers(ft.sky.cuspsDeg));
    for (final (k, at) in ft.planets.indexed) {
      final lines = [
        for (final line in at.accidents) '${line.accident.key}:${line.points}',
      ];
      put(
        'chart-$i-fortitude-${at.planet.fullKey}',
        '${number(ft.sky.speedsDegPerDay[k])} ${at.house} ${listed(lines)} '
            '${at.fortitude} ${at.debility} ${at.net}',
      );
    }
    final al = ft.almutens;
    put(
      'chart-$i-almuten-rules',
      '${al.rules.place.key} ${al.rules.fortune.key} '
          '${[al.fortuneDeg, ft.sky.ascendantDeg, ft.sky.midheavenDeg].map(number).join(' ')}',
    );
    String ranked(Almuten almuten) =>
        '${almuten.totals.map((at) => at.total).join(',')} '
        '${listed([for (final planet in almuten.almutens) planet.fullKey])} '
        '${listed([for (final planet in almuten.partakers) planet.fullKey])}';
    put('chart-$i-almuten-figure', ranked(al.figure));
    put('chart-$i-almuten-places', ranked(al.places));
    for (final (k, almuten) in al.houses.indexed) {
      put('chart-$i-almuten-house-${k + 1}', ranked(almuten));
    }
    final lt = chart.lots!;
    put(
      'chart-$i-lots',
      '${lt.sect.key} ${lt.request.sectRule.key} ${lt.request.fortune.key} '
          '${lt.fortuneReversed ? 1 : 0}',
    );
    for (final placed in lt.lots) {
      final at = placed.place;
      put(
        'chart-$i-lot-${placed.lot.key}',
        '${number(at.longitudeDeg)} ${at.sign.fullKey} ${at.lord.fullKey} '
            '${at.house}',
      );
    }
    final cs = chart.considerations!;
    int flag(bool value) => value ? 1 : 0;
    String perfection(Perfection? found) =>
        found == null
            ? '-'
            : '${found.planet.fullKey} ${found.aspect.key} ${number(found.days)} '
                '${number(found.gapDeg)}';
    final rd = cs.radicality;
    final asc = cs.ascendant;
    put(
      'chart-$i-considerations',
      '${rd.hourLord.fullKey} ${rd.ascendantLord.fullKey} '
          '${listed([for (final g in rd.grounds) g.key])} ${asc.sign.fullKey} '
          '${number(asc.degree)} ${flag(asc.early)} ${flag(asc.late)} '
          '${flag(asc.shortAscension)}',
    );
    final mn = cs.moon;
    put(
      'chart-$i-considerations-moon',
      '${mn.sign.fullKey} ${number(mn.degree)} ${flag(mn.late)} '
          '${flag(mn.lateSign)} ${flag(mn.viaCombusta)} '
          '${number(mn.course.daysInSign)} ${flag(mn.course.eased)}',
    );
    put('chart-$i-considerations-next', perfection(mn.course.next));
    put('chart-$i-considerations-within', perfection(mn.course.withinOrb));
    final sv = cs.seventh;
    put(
      'chart-$i-considerations-seventh',
      '${number(sv.cuspDeg)} ${sv.lord.fullKey} '
          '${listed([for (final g in sv.infortunesInHouse) g.fullKey])} '
          '${flag(sv.lordRetrograde)} ${flag(sv.lordCombust)} '
          '${flag(sv.lordInFall)} ${flag(sv.lordInInfortuneTerm)} ${sv.lordNet}',
    );
    put(
      'chart-$i-considerations-saturn',
      '${cs.saturnHouse} ${flag(cs.saturnRetrograde)} '
          '${flag(cs.ascendantLordCombust)}',
    );
    put(
      'chart-$i-considerations-rules',
      '${number(cs.rules.moonLateFromDeg)} '
          '${cs.rules.orbsDeg.map(number).join(',')}',
    );
    final pf = chart.perfection!;
    String commas(Iterable<String> values) {
      final joined = values.join(',');
      return joined.isEmpty ? '-' : joined;
    }

    String held(EssentialDignity d) => commas([
      if (d.house) 'house',
      if (d.exaltation) 'exaltation',
      if (d.triplicity) 'triplicity',
      if (d.term) 'term',
      if (d.face) 'face',
      if (d.detriment) 'detriment',
      if (d.fall) 'fall',
    ]);
    put(
      'chart-$i-perfection',
      '${pf.querent.fullKey} ${pf.quesited.fullKey} ${number(pf.horizonDays)} '
          '${pf.impediments.length} ${pf.translations.length} '
          '${pf.collections.length}',
    );
    final ap = pf.application;
    put(
      'chart-$i-perfection-application',
      ap == null
          ? '-'
          : '${ap.aspect.key} ${number(ap.days)} ${ap.applying.fullKey} '
              '${ap.kind.key} ${number(ap.gapDeg)} ${flag(ap.withinMoieties)}',
    );
    final sp = pf.separation;
    put(
      'chart-$i-perfection-separation',
      sp == null ? '-' : '${sp.aspect.key} ${number(sp.pastDeg)}',
    );
    final wy = pf.ways;
    put(
      'chart-$i-perfection-ways',
      '${wy.querent.house} ${held(wy.querent.dignity)} ${wy.quesited.house} '
          '${held(wy.quesited.dignity)} ${flag(wy.mutualByHouse)} '
          '${commas([for (final g in wy.infortunesBetween) g.fullKey])} '
          '${flag(wy.moonRelays)} ${flag(wy.quesitedInAscendant)} '
          '${commas([for (final w in wy.held) w.key])}',
    );
    for (final (n, at) in pf.impediments.indexed) {
      put(
        'chart-$i-perfection-impediment-$n',
        '${at.kind.key} ${at.significator.fullKey} ${at.third?.fullKey ?? '-'} '
            '${at.aspect.key} ${number(at.days)}',
      );
    }
    for (final (n, at) in pf.translations.indexed) {
      put(
        'chart-$i-perfection-translation-$n',
        '${at.translator.fullKey} ${at.from.fullKey} ${at.to.fullKey} '
            '${at.separating.aspect.key} ${number(at.separating.pastDeg)} '
            '${at.aspect.key} ${number(at.days)} ${held(at.received)}',
      );
    }
    for (final (n, at) in pf.collections.indexed) {
      put(
        'chart-$i-perfection-collection-$n',
        '${at.collector.fullKey} ${at.fromQuerent.aspect.key} '
            '${number(at.fromQuerent.days)} ${at.fromQuesited.aspect.key} '
            '${number(at.fromQuesited.days)} ${held(at.collectorInQuerent)} '
            '${held(at.collectorInQuesited)} ${held(at.querentInCollector)} '
            '${held(at.quesitedInCollector)}',
      );
    }
    put(
      'chart-$i-perfection-rules',
      '${pf.rules.orbsDeg.map(number).join(',')} ${flag(pf.rules.withinSign)}',
    );
    final pr = chart.progressions!;
    final pg = pr.progressed!;
    final dr = pr.directed!;
    final contacts = pr.contacts!;
    put(
      'chart-$i-progressed',
      '${number(pg.life)} ${number(pg.sky)} ${number(pg.armcDeg)} '
          '${number(pg.angles.ascendantDeg)} ${number(pg.angles.midheavenDeg)}',
    );
    for (final (n, g) in pg.grahas.indexed) {
      put(
        'chart-$i-progressed-graha-$n',
        '${g.graha.fullKey} ${number(g.longitudeDeg)} ${number(g.tropicalDeg)} '
            '${number(g.speedDegPerDay)}',
      );
    }
    put(
      'chart-$i-directed',
      '${number(dr.arcDeg)} ${number(dr.ascendantDeg)} ${number(dr.midheavenDeg)}',
    );
    for (final (n, p) in dr.planets.indexed) {
      put(
        'chart-$i-directed-graha-$n',
        '${p.graha.fullKey} ${number(p.longitudeDeg)}',
      );
    }
    put('chart-$i-progressed-contact-count', '${contacts.length}');
    for (final (n, c) in contacts.indexed) {
      put(
        'chart-$i-progressed-contact-$n',
        '${number(c.life)} ${number(c.sky)} ${c.graha.fullKey} ${natalKey(c.to)} '
            '${c.angle} ${c.motion.key}',
      );
    }
    final western = chart.westernAspects!;
    put('chart-$i-western-aspect-count', '${western.length}');
    for (final (n, row) in western.indexed) {
      put(
        'chart-$i-western-aspect-$n',
        '${row.first.fullKey} ${row.second.fullKey} ${row.aspect.key} '
            '${number(row.apartDeg)} ${number(row.fromExactDeg)} '
            '${number(row.orbDeg)} ${row.applying ? 1 : 0}',
      );
    }
    final declined = chart.declinations!;
    put(
      'chart-$i-declinations',
      '${number(declined.obliquityDeg)} ${number(declined.lagnaDeg)} '
          '${number(declined.midheavenDeg)}',
    );
    for (final at in declined.grahas) {
      put(
        'chart-$i-declination-${at.graha.fullKey}',
        number(at.declinationDeg),
      );
    }
    final parallels = chart.parallels!;
    put('chart-$i-parallel-count', '${parallels.length}');
    for (final (n, row) in parallels.indexed) {
      put(
        'chart-$i-parallel-$n',
        '${row.first.fullKey} ${row.second.fullKey} ${row.contrary ? 1 : 0} '
            '${number(row.apartDeg)} ${number(row.orbDeg)}',
      );
    }
    final reflected = chart.antiscia!;
    for (final reflection in reflected.points) {
      put(
        'chart-$i-antiscion-${reflection.graha.fullKey}',
        '${number(reflection.antiscionDeg)} '
            '${number(reflection.contrantiscionDeg)}',
      );
    }
    put(
      'chart-$i-antiscia-unpaired',
      reflected.unpaired.isEmpty
          ? '-'
          : reflected.unpaired.map((one) => one.fullKey).join(','),
    );
    putAntiscionRows('chart-$i-antiscia', reflected.pairs);
    put(
      'chart-$i-antiscia-cusps',
      '${reflected.cuspSystem!.fullKey} ${reflected.onCusps.length}',
    );
    for (final (n, at) in reflected.onCusps.indexed) {
      put(
        'chart-$i-antiscia-cusp-$n',
        '${at.graha.fullKey} ${at.house} ${at.contrary ? 1 : 0}',
      );
    }
    final counted = chart.westernHouses!;
    put(
      'chart-$i-western-houses',
      '${counted.system.fullKey} ${number(counted.ascendantDeg)} '
          '${number(counted.reachDeg)} ${counted.planets.length}',
    );
    for (final (n, at) in counted.cuspsDeg.indexed) {
      put('chart-$i-western-cusp-${n + 1}', number(at));
    }
    for (final at in counted.planets) {
      put(
        'chart-$i-western-house-${at.graha.fullKey}',
        '${at.house} ${at.withAscendant ? 1 : 0}',
      );
    }
    putAshta('chart-$i', chart.matching!);
    putPorutham('chart-$i', chart.porutham!);
    final mars = chart.kuja!;
    for (final (who, side) in [('bride', mars.bride), ('groom', mars.groom)]) {
      final readings = [
        for (final r in side.readings)
          '${r.from.key} ${r.house} ${flag(r.inHouses)}',
      ];
      put('chart-$i-kuja-$who', '${readings.join(' ')} ${flag(side.dosha)}');
    }
    put('chart-$i-kuja', '${flag(mars.both)}');
    final doshas = chart.marriageDoshas!;
    put('chart-$i-doshas', '${doshas.length}');
    for (final (n, d) in doshas.indexed) {
      put(
        'chart-$i-dosha-$n',
        '${d.system.key} ${d.koota?.fullKey ?? 'NONE'} '
            '${d.side?.key ?? 'NONE'} ${flag(d.lifted)}',
      );
    }
    final fifth = chart.harmonic!;
    String pointKey(HarmonicPoint point) => switch (point) {
      HarmonicGraha(:final graha) => graha.fullKey,
      HarmonicAngle(:final point) => point,
    };
    put(
      'chart-$i-harmonic',
      '${fifth.harmonic} ${fifth.points.length} ${fifth.rows.length}',
    );
    for (final at in fifth.points) {
      put(
        'chart-$i-harmonic-${pointKey(at.point)}',
        '${number(at.longitudeDeg)} ${at.house}',
      );
    }
    for (final (n, at) in fifth.rows.indexed) {
      put(
        'chart-$i-harmonic-row-$n',
        '${pointKey(at.first)} ${pointKey(at.second)} ${number(at.apartDeg)} '
            '${at.multiple} ${number(at.orbDeg)}',
      );
    }
    final between = chart.midpoints!;
    put('chart-$i-midpoint-count', '${between.length}');
    for (final (n, row) in between.indexed) {
      put(
        'chart-$i-midpoint-$n',
        '${row.first.fullKey} ${row.second.fullKey} ${row.middle.fullKey} '
            '${row.far ? 1 : 0} ${number(row.distanceDeg)} '
            '${number(row.fromAxisDeg)} ${number(row.orbDeg)}',
      );
    }
    final synastry = chart.synastry!;
    put('chart-$i-synastry-count', '${synastry.length}');
    for (final (n, row) in synastry.indexed) {
      put(
        'chart-$i-synastry-$n',
        '${natalKey(row.first)} ${natalKey(row.second)} ${row.aspect.key} '
            '${number(row.apartDeg)} ${number(row.fromExactDeg)} '
            '${number(row.orbDeg)}',
      );
    }
    final levelled = chart.synastryParallels!;
    put('chart-$i-synastry-parallel-count', '${levelled.length}');
    for (final (n, row) in levelled.indexed) {
      put(
        'chart-$i-synastry-parallel-$n',
        '${natalKey(row.first)} ${natalKey(row.second)} '
            '${row.contrary ? 1 : 0} ${number(row.apartDeg)} '
            '${number(row.orbDeg)}',
      );
    }
    putAntiscionRows('chart-$i-synastry-antiscia', chart.synastryAntiscia!);
    final across = chart.synastryMidpoints!;
    put('chart-$i-synastry-midpoint-count', '${across.length}');
    for (final (n, row) in across.indexed) {
      put(
        'chart-$i-synastry-midpoint-$n',
        '${row.first.fullKey} ${row.second.fullKey} ${row.middle.fullKey} '
            '${row.partnersPair ? 1 : 0} ${row.far ? 1 : 0} '
            '${number(row.distanceDeg)} ${number(row.fromAxisDeg)} '
            '${number(row.orbDeg)}',
      );
    }
    final composite = chart.synastryComposite!;
    put(
      'chart-$i-composite',
      '${number(composite.lagnaDeg)} ${number(composite.midheavenDeg)} '
          '${composite.lagnaTurned ? 1 : 0} ${composite.planets.length}',
    );
    for (final (n, at) in composite.planets.indexed) {
      put(
        'chart-$i-composite-$n',
        '${at.graha.fullKey} ${number(at.longitudeDeg)} ${number(at.speedDegPerDay)}',
      );
    }
    put(
      'chart-$i-composite-cusps',
      composite.cuspsDeg?.map(number).join(' ') ?? '-',
    );
    final davison = chart.synastryDavison!;
    put(
      'chart-$i-davison',
      '${number(davison.instant)} ${number(davison.place.latitudeDeg)} '
          '${number(davison.place.longitudeDeg)} ${number(davison.place.altitudeM)} '
          '${davison.utcOffsetSeconds}',
    );
    final vs = chart.vimshopaka!;
    put('chart-$i-vimshopaka', vs.scoring.key);
    for (final g in vs.grahas) {
      put(
        'chart-$i-vimshopaka-${g.graha.fullKey}',
        [
          g.shadvarga,
          g.saptavarga,
          g.dashavarga,
          g.shodashavarga,
        ].map(number).join(','),
      );
    }
    for (final (j, dasha) in chart.dashas.indexed) {
      final key = 'chart-$i-dasha-$j';
      final balance = dasha.balance;
      put(key, dasha.system.fullKey);
      put('$key-seed', dasha.seed?.fullKey);
      put('$key-first-lord', dasha.firstLord.fullKey);
      put('$key-overflow', dasha.overflow);
      put('$key-balance', balance?.method.key);
      put('$key-remaining', balance?.remaining);
      put('$key-balance-days', balance?.days);
      final w = balance?.written;
      put(
        '$key-balance-written',
        w == null
            ? null
            : '${w.years},${w.months},${w.days},${w.hours},${w.minutes}',
      );
      put('$key-moon-span-from', dasha.moonSpan?.from);
      put('$key-moon-span-to', dasha.moonSpan?.to);
      put('$key-depth', dasha.depth);
      put('$key-periods', dasha.periods.length);
      for (final (k, period) in dasha.periods.indexed) {
        if (period.level > 2) continue;
        final sign = period.sign == null ? '' : ' ${period.sign!.fullKey}';
        put('$key-period-$k', '${period.path}$sign ${period.lord.fullKey}');
        put('$key-period-$k-from', period.from);
        put('$key-period-$k-to', period.to);
      }
      put(
        '$key-at',
        [
          for (final period in dasha.at(chart.instant + 5000)) period.path,
        ].join(','),
      );
    }
    final vargas = chart.vargas;
    for (var v = 0; v < vargas.length; v += 1) {
      final varga = vargas[v];
      put('chart-$i-varga-$v', varga.varga.fullKey);
      put('chart-$i-varga-$v-lagna-rashi', varga.lagna.rashi.fullKey);
      put('chart-$i-varga-$v-lagna-part', varga.lagna.part);
      put('chart-$i-varga-$v-lagna-sign', varga.lagna.sign.fullKey);
      for (var j = 0; j < varga.grahas.length; j += 1) {
        final placed = varga.grahas[j];
        put('chart-$i-varga-$v-graha-$j', placed.graha.fullKey);
        put('chart-$i-varga-$v-graha-$j-rashi', placed.at.rashi.fullKey);
        put('chart-$i-varga-$v-graha-$j-part', placed.at.part);
        put('chart-$i-varga-$v-graha-$j-sign', placed.at.sign.fullKey);
      }
    }
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
    // Uranus, Neptune and Pluto, which the request asks beside the nine.
    final outer = chart.outer;
    for (var j = 0; j < outer.length; j += 1) {
      final at = outer[j];
      put(
        'chart-$i-outer-$j',
        '${at.graha.fullKey} ${number(at.longitudeDeg)} '
            '${number(at.latitudeDeg)} ${number(at.speedDegPerDay)} '
            '${at.house.bhava} ${at.placement.bhava}',
      );
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
  // **The annual charts, under all three readings.** One crossing each,
  // because `varshaJson` names one reading per request — and all three,
  // because a reading that crossed as another would be invisible in a
  // report that only printed the default.
  // Each reading also asks the sixteen yogas a different way, so all three
  // ways cross: every matter under the source's readings, every matter
  // under Tambira's "some authorities", and no matter at all.
  String pairSaid(TajikaBetween p) =>
      '${p.faster.fullKey}>${p.slower.fullKey}:${p.drishti.key}:'
      '${p.yoga?.key ?? '-'}:${p.apartDeg.toStringAsFixed(6)}';
  String clausesSaid(List<Affliction> clauses) =>
      clauses.isEmpty ? 'none' : clauses.map((c) => c.key).join('+');
  String heldSaid(HeldYearYoga h) => [
    h.yoga.key,
    h.through?.fullKey ?? '-',
    h.entering?.fullKey ?? '-',
    h.between == null ? '-' : 'pair',
    h.legs?.map(pairSaid).join('/') ?? '-',
    if (h.afflictions case final a?)
      '${clausesSaid(a.lagnesha)}/${clausesSaid(a.karyesha)}'
    else
      '-',
  ].join(':');
  // One saham as every runner prints it: its place, its clauses, its
  // lord's strengths and how the seven stand to it.
  String sahamSaid(TajikaSaham p) => [
    '${p.longitudeDeg.toStringAsFixed(6)} ${p.sign.fullKey} '
        '${p.lord.fullKey} ${p.house} ${p.addedSign}',
    'S:${p.strong.map((c) => c.key).join(',')} '
        'W:${p.weak.map((c) => c.key).join(',')}',
    '${p.lordVishwa} ${p.lordHarsha.key} ${p.inNodeAxis}',
    p.seven
        .map((s) => '${s.drishti.key}/${s.relation.key}/${s.company ? 1 : 0}')
        .join(' '),
  ].join(' | ');
  for (final reading in VarshaReading.values) {
    final years = geo.chart.foundMany(
      instants: <double>[2460482.5, 2460600.25],
      place: place,
      utcOffsetSeconds: 20700,
      varsha: VarshaRequest(
        through: 12,
        reading: reading,
        place: AnnualPlace.birth,
        matters: reading == VarshaReading.mean ? null : Matters.all,
        yogas:
            reading == VarshaReading.tropical
                ? const YogaRules(tambira: TambiraMover.eitherLord)
                : const YogaRules(),
        // The sahams likewise: every one under the source's rules, every
        // one under each rival rule, and none.
        sahams: reading == VarshaReading.mean ? null : Sahams.all,
        sahamRules:
            reading == VarshaReading.tropical
                ? const SahamRules(
                  addSign: AddSign.signs,
                  houses: HousePoints.equal,
                  roga: RogaReading.saturn,
                )
                : const SahamRules(),
        // And the annual dashas: every one under the sources' readings,
        // every one under a rival clock, balance and birth period three
        // levels deep, and none.
        dashas: reading == VarshaReading.mean ? null : AnnualDashas.all,
        dashaRules:
            reading == VarshaReading.tropical
                ? const AnnualDashaRules(
                  clock: YearClock.even,
                  balance: MuddaBalance.entryMoon,
                  birthPeriod: BirthPeriod.elapsed,
                  depth: 3,
                )
                : const AnnualDashaRules(),
      ),
    );
    var i = 0;
    for (final chart in years.each) {
      final found = chart.praveshas;
      put('chart-$i-varsha-${reading.key}-count', found.length);
      for (final p in chart.sahams) {
        put(
          'chart-$i-varsha-${reading.key}-natal-saham-${p.saham.key}',
          sahamSaid(p),
        );
      }
      for (final one in found) {
        final stem = 'chart-$i-varsha-${reading.key}-${one.year}';
        put(stem, one.instant);
        put('$stem-muntha', one.muntha.sign.fullKey);
        put('$stem-muntha-lord', one.muntha.lord.fullKey);
        put('$stem-muntha-deg', one.muntha.longitudeDeg);
        final annual = one.annual!;
        put('$stem-annual-lagna', annual.lagnaDeg);
        put('$stem-annual-by-day', annual.byDay);
        final b = annual.officeBearers;
        put(
          '$stem-annual-bearers',
          [
            b.muntha,
            b.janmaLagna,
            b.varshaLagna,
            b.triRashi,
            b.dinaRatri,
          ].map((lord) => lord.fullKey).join(' '),
        );
        final yearLord = annual.yearLord;
        put('$stem-year-lord', yearLord.graha.fullKey);
        put('$stem-year-lord-chosen', yearLord.chosen.key);
        put('$stem-year-lord-bala', yearLord.vishwa.toString());
        put(
          '$stem-yogas',
          annual.yogas
              .map(
                (p) =>
                    '${p.faster.fullKey}>${p.slower.fullKey}:'
                    '${p.drishti.key}:${p.yoga.key}:${p.apartDeg.toStringAsFixed(6)}',
              )
              .join(' '),
        );
        put(
          '$stem-states',
          'R:${annual.retrograde.map((g) => g.fullKey).join(',')} '
              'C:${annual.combust.map((g) => g.fullKey).join(',')}',
        );
        for (final m in annual.matters) {
          final at = '$stem-matter-${m.house}';
          put(
            at,
            '${m.sign.fullKey} ${m.lagnesha.fullKey}>${m.karyesha.fullKey} ${m.sameLord}',
          );
          put('$at-pair', m.between == null ? '-' : pairSaid(m.between!));
          put('$at-unanswered', m.unanswered.map((y) => y.key).join(','));
          put('$at-held', m.held.map(heldSaid).join(' '));
        }
        for (final p in annual.sahams) {
          put('$stem-saham-${p.saham.key}', sahamSaid(p));
        }
        for (final d in annual.dashas) {
          final said = '$stem-dasha-${d.system.fullKey}';
          final ring = d.ring;
          put(
            said,
            '${d.seed?.fullKey ?? '-'} ${ring.first} '
            '${ring.remaining?.toStringAsFixed(9) ?? 'null'} '
            '${d.year.from.toStringAsFixed(9)} ${d.year.to.toStringAsFixed(9)} | '
            '${ring.shares.map((s) => '${s.lord.fullKey}/${s.sign?.fullKey ?? '-'}/${s.weight.toStringAsFixed(3)}').join(' ')}',
          );
          put(
            '$said-periods',
            d.periods
                .map(
                  (p) =>
                      '${p.path}:${p.lord.fullKey}:${p.sign?.fullKey ?? '-'}:'
                      '${p.from.toStringAsFixed(9)}:${p.to.toStringAsFixed(9)}',
                )
                .join(' '),
          );
        }
        put(
          '$stem-harsha',
          annual.harsha
              .map(
                (h) =>
                    '${h.graha.fullKey}:${h.house}:'
                    '${h.sthana ? 1 : 0}${h.uchchaSwakshetra ? 1 : 0}'
                    '${h.striPurusha ? 1 : 0}${h.dinaRatri ? 1 : 0}:'
                    '${h.total}:${h.grade.key}',
              )
              .join(' '),
        );
        put(
          '$stem-year-claims',
          yearLord.claims
              .map(
                (c) =>
                    '${c.graha.fullKey}:${c.vishwa}:${c.portfolios}:${c.aspectsLagna}',
              )
              .join(' '),
        );
      }
      i += 1;
    }
  }

  final single = geo.chart.found(
    instant: 2460482.5,
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('chart-single-lagna', single.lagnaDeg);
  put('chart-single-agrees', single.lagnaDeg == charts.at(0).lagnaDeg);

  // ── An almanac ───────────────────────────────────────────────────────
  // Three days, because a day's lists are ragged and two consecutive days
  // with the same counts would not exercise the offsets.
  final week = geo.almanac.of(
    from: Calendar.gregorian.date(2024, 6, 17),
    to: Calendar.gregorian.date(2024, 6, 19),
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('almanac-days', week.length);
  put('almanac-calendar', week.calendar.fullKey);
  put('almanac-place-lat', week.decoded.latitudeDeg);
  put('almanac-model-fnv', fnv(week.model));
  put('almanac-provenance-fnv', fnv(week.provenanceJson));

  for (final day in week.each) {
    final i = day.index;
    put('day-$i-content-hash', day.provenance.contentHash);
    putDay('day-$i', day.day);
    put('day-$i-window-from', day.window.from);
    put('day-$i-window-to', day.window.to);
    put('day-$i-month', day.month.month.fullKey);
    put('day-$i-amanta', day.month.amanta.fullKey);
    put('day-$i-purnimanta', day.month.purnimanta.fullKey);
    put('day-$i-paksha', day.month.paksha.fullKey);
    put('day-$i-convention', day.month.convention.key);
    put('day-$i-month-kind', day.month.kind.key);
    put('day-$i-ayana', day.ayana.fullKey);
    put('day-$i-ritu', day.ritu.fullKey);
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
        put('day-$i-$name-$j-sunrises', span.sunrises.key);
        put(
          'day-$i-$name-$j-ends',
          '${span.ends.ghati}-${span.ends.pala}-${span.ends.vipala}',
        );
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
      put('day-$i-moon-$j-kind', moon[j].rise ? 'RISE' : 'SET');
      put('day-$i-moon-$j-instant', moon[j].instant);
    }
    final held = day.muhurtaYogas;
    for (var j = 0; j < held.length; j += 1) {
      put('day-$i-yoga-held-$j', held[j].yoga.fullKey);
      put(
        'day-$i-yoga-held-$j-cause',
        held[j].tithi == null ? 'VARA_NAKSHATRA' : 'VARA_TITHI_NAKSHATRA',
      );
      put('day-$i-yoga-held-$j-tithi', held[j].tithi?.fullKey ?? 'none');
    }
  }
  // `almanacDay` is the range of one unwrapped, and must agree.
  final oneDay = geo.almanac.day(
    date: Calendar.gregorian.date(2024, 6, 17),
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('almanac-single-agrees', oneDay.day.sunrise == week.at(0).day.sunrise);

  // ── A muhurta search ─────────────────────────────────────────────────
  // Both rankings over 2024-11-25..27: the texts bar the windows for
  // different reasons and the baseline scores them; and a thread ceremony,
  // whose rules want grahas out of houses.
  for (final (name, rules, ranking) in [
    ('raman', MuhurtaActivity.ramanMarriage, MuhurtaRanking.texts),
    ('baseline', MuhurtaActivity.baselineMarriage, MuhurtaRanking.baseline),
    ('upanayana', MuhurtaActivity.ramanUpanayana, MuhurtaRanking.texts),
  ]) {
    final muhurta =
        geo.almanac
            .of(
              from: Calendar.gregorian.date(2024, 11, 25),
              to: Calendar.gregorian.date(2024, 11, 27),
              place: place,
              utcOffsetSeconds: 20700,
              muhurta: MuhurtaRequest(
                rules: rules,
                ranking: ranking,
                native: const MuhurtaNative(
                  star: Nakshatra.rohini,
                  moonSign: Rashi.taurus,
                  lagna: Rashi.leo,
                ),
                daysWithWindows: 3,
                most: 12,
              ),
            )
            .muhurta!;
    putMuhurta('muhurta-$name', muhurta);
  }

  // ── Festivals ────────────────────────────────────────────────────────
  // The shipped pack, the pack amended by a rule of the consumer's own
  // (Lakshmi puja on whichever day holds the new moon at sunrise), and the
  // Nepal pack with a following rule of the consumer's own (two days after
  // Lakshmi puja), over 2024-10-10..11-03.
  const ownRule = <String, Object?>{
    'key': 'LAKSHMI_PUJA',
    'source': 'the tithi at sunrise',
    'month': 'masa.ASHWINA',
    'tithi': 'tithi.AMAVASYA',
    'at': {'window': 'SUNRISE'},
    'decide': <Object?>[],
    'otherwise': 'LATER',
  };
  const ownFollowing = <String, Object?>{
    'key': 'TWO_AFTER',
    'source': 'two days after Lakshmi puja',
    'after': 'LAKSHMI_PUJA',
    'days': 2,
  };
  for (final (name, rules) in <(String, Object)>[
    ('shipped', FestivalPack.dharmasindhu),
    ('amended', [FestivalPack.dharmasindhu, ownRule]),
    ('nepal', [FestivalPack.nepal, ownFollowing]),
  ]) {
    final festivals =
        geo.almanac
            .of(
              from: Calendar.gregorian.date(2024, 10, 10),
              to: Calendar.gregorian.date(2024, 11, 3),
              place: place,
              utcOffsetSeconds: 20700,
              festivals: FestivalRequest(rules: rules),
            )
            .festivals!;
    putFestivals('festivals-$name', festivals);
  }

  // ── The lunar years ─────────────────────────────────────────────────
  // 2024-03-20..04-20 holds a Chaitra Shukla Pratipada: two years, their
  // bounds and their Jovian years.
  final years =
      geo.almanac
          .of(
            from: Calendar.gregorian.date(2024, 3, 20),
            to: Calendar.gregorian.date(2024, 4, 20),
            place: place,
            utcOffsetSeconds: 20700,
            years: true,
          )
          .years!;
  put('years-count', years.value.length);
  put('years-hash', years.provenance.contentHash);
  for (final (k, year) in years.value.indexed) {
    put(
      'years-$k',
      [
        year.samvatsara.fullKey,
        year.count,
        year.vikrama,
        year.shaka,
        number(year.opened),
        number(year.began),
        number(year.ended),
        year.lupta?.fullKey ?? '-',
      ].join(' '),
    );
    put(
      'years-$k-jovian',
      listed([
        for (final jovian in year.jovian)
          '${jovian.member.fullKey}:${jovian.count}:${number(jovian.from)}',
      ]),
    );
  }

  // ── The Nepal Sambat dates ─────────────────────────────────────────
  // 2024-10-30..11-03 holds Kartika's new moon, where the year turns.
  final nepalSambat =
      geo.almanac
          .of(
            from: Calendar.gregorian.date(2024, 10, 30),
            to: Calendar.gregorian.date(2024, 11, 3),
            place: place,
            utcOffsetSeconds: 20700,
            nepalSambat: true,
          )
          .nepalSambat!;
  put('nepal-sambat-hash', nepalSambat.provenance.contentHash);
  put(
    'nepal-sambat',
    listed([
      for (final d in nepalSambat.value)
        '${d.year}:${d.month}:${d.kind.key}:${d.paksha.fullKey}',
    ]),
  );
  geo.dispose();

  // ── The eclipses ─────────────────────────────────────────────────────
  // September 2025 at Kathmandu over the built-in sky, which the test
  // provider cannot complete: a total lunar eclipse seen whole and a
  // partial solar one the place does not see (`03-design/eclipses.md`).
  final builtin = teistro.context(
    profile: 'nepali-default',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );
  final eclipses =
      builtin.almanac
          .of(
            from: Calendar.gregorian.date(2025, 9, 1),
            to: Calendar.gregorian.date(2025, 9, 30),
            place: place,
            utcOffsetSeconds: 20700,
            eclipses: true,
          )
          .eclipses!;
  builtin.dispose();
  String maybe(double? value) => value == null ? '-' : number(value);
  String moment(EclipseMoment? m) =>
      m == null ? '-' : '${number(m.at)}@${number(m.altitudeDeg)}';
  String seen(EclipseSeen? s) =>
      s == null ? '-' : '${number(s.from)}..${number(s.to)}';
  put('eclipses-hash', eclipses.provenance.contentHash);
  put(
    'eclipses-count',
    '${eclipses.value.lunar.length} ${eclipses.value.solar.length}',
  );
  for (final (k, LunarEclipseHere(:eclipse, :here))
      in eclipses.value.lunar.indexed) {
    final c = eclipse.contacts;
    put(
      'eclipses-lunar-$k',
      [
        eclipse.kind.fullKey,
        eclipse.shadow.key,
        number(eclipse.greatest),
        number(eclipse.gamma),
        number(eclipse.umbralMagnitude),
        number(eclipse.penumbralMagnitude),
      ].join(' '),
    );
    put(
      'eclipses-lunar-$k-contacts',
      [c.p1, c.u1, c.u2, c.u3, c.u4, c.p4].map(maybe).join(' '),
    );
    put(
      'eclipses-lunar-$k-here',
      [
        for (final m in [
          here.p1,
          here.u1,
          here.u2,
          here.greatest,
          here.u3,
          here.u4,
          here.p4,
        ])
          moment(m),
        seen(here.seen),
        seen(here.umbralSeen),
      ].join(' '),
    );
  }
  for (final (k, SolarEclipseHere(:eclipse, :here))
      in eclipses.value.solar.indexed) {
    put(
      'eclipses-solar-$k',
      [
        eclipse.kind.fullKey,
        number(eclipse.greatest),
        number(eclipse.gamma),
        number(eclipse.magnitude),
        number(eclipse.latitude),
        number(eclipse.longitude),
      ].join(' '),
    );
    put(
      'eclipses-solar-$k-here',
      here == null
          ? '-'
          : [
            here.kind.fullKey,
            number(here.magnitude),
            number(here.obscuration),
            for (final m in [
              here.first,
              here.second,
              here.third,
              here.fourth,
              here.maximum,
            ])
              moment(m),
            seen(here.seen),
          ].join(' '),
    );
  }

  // ── The surface's shape ───────────────────────────────────────────
  //
  // The lines above compare what the bindings ANSWER. These compare
  // where an operation LIVES: every key is the canonical
  // `area.operation` path, and the member each binding references beside
  // it is its own spelling of it. A binding that moved an operation to
  // another area, or renamed one, prints a key the others do not and the
  // gate fails — which is what `03-design/surface-areas.md` asks of this
  // runner, and what `check-parity` could not see before.
  //
  // In Dart a tear-off is resolved at compile time, so a member that
  // moved does not merely print `missing` here: the runner stops
  // building.
  for (final entry in <(String, Object?)>[
    ('calendar.date_of', ctx.calendar.dateOf),
    ('calendar.fixed_of', ctx.calendar.fixedOf),
    ('calendar.convert', ctx.calendar.convert),
    ('calendar.weekday_of', ctx.calendar.weekdayOf),
    ('calendar.month_length', ctx.calendar.monthLength),
    ('calendar.is_leap', ctx.calendar.isLeap),
    ('time.resolve', ctx.time.resolve),
    ('time.civil_of', ctx.time.civilOf),
    ('time.convert', ctx.time.convert),
    ('time.delta_t', ctx.time.deltaT),
    ('intl.locale', ctx.intl.locale),
    ('intl.render', ctx.intl.render),
    ('intl.has', ctx.intl.has),
    ('intl.transliterate', ctx.intl.transliterate),
    ('intl.entity', ctx.intl.entity),
    ('intl.messages', ctx.intl.messages),
    ('intl.load_pack', ctx.intl.loadPack),
    ('keys.id', ctx.keys.id),
    ('keys.name', ctx.keys.name),
    ('matching.naam', ctx.matching.naam),
    ('frame.canonical', ctx.frame.canonical),
    ('frame.pack', ctx.frame.pack),
    ('frame.unpack', ctx.frame.unpack),
    ('chart.layout', ctx.chart.layout),
    ('chart.found', ctx.chart.found),
    ('chart.found_many', ctx.chart.foundMany),
    ('almanac.of', ctx.almanac.of),
    ('almanac.day', ctx.almanac.day),
    ('engine.names', ctx.engine.names),
    ('engine.signature', ctx.engine.signature),
    ('engine.call', ctx.engine.call),
    ('engine.call_json', ctx.engine.callJson),
    ('engine.manifest', ctx.engine.manifest),
    ('engine.manifest_json', ctx.engine.manifestJson),
    ('(root).engine', ctx.engine),
    ('(root).positions', ctx.positions),
    ('(root).profile', ctx.profile),
    ('(root).settings', ctx.settings),
    ('(root).settings_json', ctx.settingsJson),
    ('(root).settings_hash', ctx.settingsHash),
    ('(root).dispose', ctx.dispose),
  ]) {
    put('surface.${entry.$1}', entry.$2 == null ? 'missing' : 'present');
  }

  final keys = report.keys.toList()..sort();
  for (final key in keys) {
    stdout.write('$key\t${report[key]}\n');
  }
  ctx.dispose();
}
