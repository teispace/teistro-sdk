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

/// A local day's every field, under the same keys for a chart's day and
/// an almanac's, because the two layers hand back one record.
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
  put(
    '$prefix-convention',
    day.convention?.key ?? 'custom ${number(day.customAltitudeDeg ?? 0)}',
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
      DashaSystem.registered('ACME_PARITY'),
    ],
    drawings: [
      (ChartLayout.northIndian, Varga.d1),
      (ChartLayout.southIndian, Varga.d9),
      (ChartLayout.westernWheel, Varga.d1),
      (ChartLayout.registered('ACME_KERALA'), Varga.d9),
    ],
    theme: ChartTheme.dark,
    rules: const RuleRequest(shipped: [ShippedRules.nabhasas], longevity: true),
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
                  QuadSegment() => 'quad',
                  ArcSegment() => 'arc',
                  LineSegment() => 'line',
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
  put('almanac-provenance-fnv', fnv(week.decoded.provenance));

  for (final day in week.each) {
    final i = day.index;
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
  final oneDay = geo.almanac.day(
    date: Calendar.gregorian.date(2024, 6, 17),
    place: place,
    utcOffsetSeconds: 20700,
  );
  put('almanac-single-agrees', oneDay.day.sunrise == week.at(0).day.sunrise);
  geo.dispose();

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
