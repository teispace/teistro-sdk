// One scenario through the Node binding, printed as the parity report:
// `key<TAB>value` lines, sorted by key. `cargo xtask check-parity` runs
// this and `bindings/dart/bin/parity.dart` and compares what they print,
// so a difference between the two bindings' layers is a failed gate
// rather than something a reader has to notice.
//
// Every value is what this binding's own surface gives: an enum as the
// key it spells, a number formatted to nine decimals, a JSON section as
// its length and its FNV-1a hash, because the point is that the two
// bindings agree, not that they agree with a literal written here.

import {
  Body,
  Calendar,
  Context,
  Scale,
  abiVersion,
  buildInfo,
  canonicalFrame,
  catalogueVersion,
  defaultProfile,
  fixedOfJulianDay,
  julianDayOfFixed,
  packFrame,
  sdkVersion,
  unpackFrame,
  ChartLayout,
  DashaSystem,
  Varga,
} from './lib/index.js';

const report = new Map();

/** A number as every binding spells it: nine decimals, never an exponent. */
const number = (value) => (Number.isInteger(value) ? String(value) : value.toFixed(9));

/** FNV-1a over UTF-8 bytes, so a JSON section can be compared without a parser. */
function fnv(text) {
  const bytes = new TextEncoder().encode(text);
  let hash = 0x811c9dc5;
  for (const byte of bytes) {
    hash = Math.imul(hash ^ byte, 0x01000193) >>> 0;
  }
  return hash.toString(16).padStart(8, '0');
}

const put = (key, value) => report.set(key, typeof value === 'number' ? number(value) : String(value));

// ── The library itself ─────────────────────────────────────────────────
put('abi', abiVersion());
put('sdk', sdkVersion());
put('catalogue-version', catalogueVersion());
put('default-profile', defaultProfile());
put('build-sdk', buildInfo.sdk);
put('build-abi', buildInfo.abi);
put('build-catalogue', buildInfo.catalogue);
put('build-commit', buildInfo.commit);
put('build-dirty', buildInfo.dirty);
put('build-target', buildInfo.target);

// ── A context ──────────────────────────────────────────────────────────
const ctx = new Context({ profile: 'nepali-default', locale: 'ne-Deva-NP', testProvider: true });
put('profile', ctx.profile);
put('locale', ctx.intl.locale);
put('settings-hash', ctx.settingsHash);
put('settings-fnv', fnv(ctx.settingsJson));

// ── The calendars ──────────────────────────────────────────────────────
const gregorian = (year, month, day) => ({
  calendar: Calendar.Gregorian,
  year,
  eraYear: 0,
  month,
  day,
  resolution: 'defined',
  computedMonth: 0,
  computedDay: 0,
});
const date = gregorian(2015, 4, 14);
const bs = ctx.calendar.convert(date, Calendar.BikramSambat);
put('bs-year', bs.year);
put('bs-month', bs.month);
put('bs-day', bs.day);
put('bs-era', bs.era);
put('bs-era-year', bs.eraYear);
put('bs-resolution', bs.resolution);
const fixed = ctx.calendar.fixedOf(date);
put('fixed', fixed);
put('weekday', ctx.calendar.weekdayOf(date));
put('month-length', ctx.calendar.monthLength(Calendar.Gregorian, 2024, 2));
put('is-leap', ctx.calendar.isLeap(Calendar.Gregorian, 2024));
put('jd-of-fixed', julianDayOfFixed(fixed));
const back = fixedOfJulianDay(2457126.75);
put('fixed-of-jd', back.value);
put('fraction-of-jd', back.fraction);

// ── Time ───────────────────────────────────────────────────────────────
const civil = {
  date: gregorian(1986, 1, 1),
  time: { hour: 0, minute: 20, second: 0, hasTime: true, nanos: 0 },
};
const zone = { kind: 'iana', offsetSeconds: 0, longitudeDeg: 0, zone: 'Asia/Kathmandu' };
const resolved = ctx.time.resolve(civil, zone);
put('resolve-jd', resolved.instantJdUtc);
put('resolve-offset', resolved.offsetSeconds);
put('resolve-era', resolved.era);
put('resolve-source', resolved.source);
put('resolve-time-known', resolved.timeKnown);
put('resolve-tzdb', resolved.tzdbVersion);
put('resolve-warnings', resolved.warnings.length);
const civilBack = ctx.time.civilOf(resolved.instantJdUtc, zone, Calendar.Gregorian);
put('civil-year', civilBack.civil.date.year);
put('civil-minute', civilBack.civil.time.minute);
put('civil-offset', civilBack.resolution.offsetSeconds);
const tt = ctx.time.convert(2451544.5, Scale.Utc, Scale.Tt);
put('tt-jd', tt.jd);
put('tt-delta-t', tt.deltaTSeconds);
put('tt-delta-t-source', tt.deltaTSource);
put('tt-delta-t-model', tt.deltaTModel);
const delta = ctx.time.deltaT(2451544.5);
put('delta-t-seconds', delta.seconds);
put('delta-t-source', delta.source);

// ── Keys ───────────────────────────────────────────────────────────────
const id = ctx.keys.id('graha.SUN');
put('key-id', id);
put('key-name', ctx.keys.name(id));
try {
  ctx.keys.id('graha.SUNN');
  put('refusal', 'none');
} catch (error) {
  put('refusal-status', error.status);
  put('refusal-detail', error.detail);
  put('refusal-hint-names-sun', error.hint.includes('SUN'));
}

// ── The locale engine ──────────────────────────────────────────────────
const rendered = ctx.intl.render('sdk.reason.grahaInBhava', {
  graha: { $entity: 'graha.JUPITER' },
  bhava: 7,
});
put('render-fnv', fnv(rendered.text));
put('render-length', [...rendered.text].length);
put('render-resolved-from', rendered.resolvedFrom);
put('render-fallback', rendered.isFallback);
// A rendered message's parts, which is what a rich renderer walks.
// `sdk.reason.lordship` is one of the two shipped messages carrying
// `{#b}`; the plain one beside it holds every binding to the rule that
// no markup means the one text part, made rather than carried.
const partShape = (parts) =>
  parts
    .map((part) =>
      part.type === 'text'
        ? `text:${part.value}`
        : `${part.kind}:${part.name}(${Object.entries(part.options)
            .sort()
            .map(([name, value]) => `${name}=${value}`)
            .join(',')})`,
    )
    .join('|');
const rich = ctx.intl.render('sdk.reason.lordship', {
  graha: { $entity: 'graha.JUPITER' },
  bhava: 5,
});
put('render-rich-parts', partShape(rich.parts));
put('render-plain-parts', partShape(rendered.parts));
put('has-message', ctx.intl.has('sdk.reason.grahaInBhava'));
put('has-missing-message', ctx.intl.has('sdk.nope.missing'));
put('transliterated', ctx.intl.transliterate('सूर्य बृहस्पति'));
put('entity-sun-name', ctx.intl.entity('graha.SUN').name);
put('entity-sun-iast', ctx.intl.entity('graha.SUN').iast);
put('entity-sun-glyph', ctx.intl.entity('graha.SUN').glyph);
put('entity-sun-gender', ctx.intl.entity('graha.SUN').gender);
put(
  'message-graha-in-bhava',
  ctx.intl.messages.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 }),
);
put(
  'message-bs-date',
  ctx.intl.messages.sdk.calendar.bikramSambat.date.long({ day: 1, monthName: 'बैशाख', year: 2072 }),
);

const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };

// ── Positions ──────────────────────────────────────────────────────────
const frame = canonicalFrame();
put('frame-centre', frame.centre);
put('frame-coordinates', frame.coordinates);
put('frame-bits', packFrame(frame));
put('frame-round-trip', unpackFrame(packFrame(frame)).centre === frame.centre);
const positions = ctx.positions({
  instants: [2451545.0, 2451546.0],
  bodies: [Body.Sun, Body.Moon, Body.Mars],
});
put('cells', positions.cells.length);
put('positions-scale', positions.scale);
put('positions-bodies', positions.bodies.join(','));
for (let i = 0; i < positions.cells.length; i += 1) {
  const instant = Math.floor(i / positions.bodies.length);
  const body = i % positions.bodies.length;
  const cell = positions.at(instant, body);
  put(`cell-${i}-lon`, cell.longitude);
  put(`cell-${i}-lat`, cell.latitude);
  put(`cell-${i}-dist`, cell.distance);
  put(`cell-${i}-lon-speed`, cell.longitudeSpeed);
  put(`cell-${i}-status`, cell.status);
}
put('steps', positions.steps.map((step) => `${step.name}:${step.implementation}`).join(','));
put('provenance-fnv', fnv(positions.decoded.provenance));
put('provenance-profile', positions.provenance.profile);
put('provenance-settings-hash', positions.provenance.settings_hash);
put('provenance-provider-frame', positions.provenance.provider.frame);

// ── The chart the topocentric profile founds ───────────────────────────
// The scenario above runs under `nepali-default`, whose frame is
// **topocentric** — inherited from the baseline engine, and what every
// recorded chart in the corpus is. Until the completion's centre step
// this could not found a chart at all, and the refusal was what the three
// bindings compared. Now the chart itself is, which is the stronger
// comparison: the step runs per body, per instant, inside the library, so
// three bindings agreeing on its output is three bindings agreeing on the
// whole of it.
const placed = ctx.chart.found({ instant: 2451545, place, utcOffsetSeconds: 20700 });
put('chart-under-topocentric', 'founded');
put('topocentric-steps', placed.batch.steps.join(','));
put('topocentric-lagna', placed.lagnaDeg);
placed.grahas.forEach((graha, j) => {
  put(`topocentric-graha-${j}`, graha.graha);
  put(`topocentric-graha-${j}-lon`, graha.longitudeDeg);
  put(`topocentric-graha-${j}-lat`, graha.latitudeDeg);
  put(`topocentric-graha-${j}-speed`, graha.speedDegPerDay);
});

// ── A chart and an almanac, under a geocentric profile ─────────────────
// Everything after this runs on the SDK's own default profile, which is
// geocentric, so that the two centres are both exercised.
// A layout of the consumer's own, registered on the context the charts are
// drawn under: the South Indian row renamed, as every runner registers it
// (`03-design/chart-geometry.md` §7f).
const shipped = new Context({ testProvider: true });
const kerala = { ...shipped.chart.layout('SOUTH_INDIAN'), key: 'ACME_KERALA' };
shipped.dispose();
// A dasha system of the consumer's own, the same definition every runner
// registers: a backward count, a two-nakshatra window, an offset, a savana
// year and a depth of two (`03-design/dasha-kernels.md`).
const parityDasha = JSON.parse('{"kernel":"udu","key":"ACME_PARITY","sources":["the parity scenario"],"lords":[{"graha":"SUN","years":5},{"graha":"MOON","years":10},{"graha":"MARS","years":7},{"graha":"MERCURY","years":12}],"reference":"MULA","count":"TO_REFERENCE","span":2,"offset":1,"repeats":true,"year_length":"SAVANA_360","depth":2}');
const geo = new Context({
  profile: 'parashari-classical',
  locale: 'ne-Deva-NP',
  testProvider: true,
  layouts: [kerala],
  dashaSystems: [parityDasha],
});
put('geo-profile', geo.profile);
put('geo-settings-hash', geo.settingsHash);

// ── Charts ─────────────────────────────────────────────────────────────
// Two instants, so a per-chart section that ran charts-outermost the
// wrong way round shows up as the second chart's values in the first's
// place rather than as nothing at all.
// **Two divisional charts asked for**, and two rather than one because
// the layout is charts outermost then charts asked for: one varga over
// two instants and one instant over two vargas are the same number of
// rows, and only asking for two of each can catch a transposed stride.
const charts = geo.chart.foundMany({
  instants: [2460482.5, 2460600.25],
  place,
  utcOffsetSeconds: 20700,
  vargas: [Varga.D9, Varga.D10],
  dashas: [DashaSystem.Vimshottari, DashaSystem.Chara, DashaSystem.Kalachakra, 'dasha_system.ACME_PARITY'],
  // A grid of the founded chart, a grid of a divisional one, and the wheel:
  // straight edges, a divisional chart's own lagna, arcs, marks and the
  // rounded coordinates the wheel's trigonometry leaves.
  drawings: [
    { layout: ChartLayout.NorthIndian, varga: Varga.D1 },
    { layout: ChartLayout.SouthIndian, varga: Varga.D9 },
    { layout: ChartLayout.WesternWheel, varga: Varga.D1 },
    { layout: 'chart_layout.ACME_KERALA', varga: Varga.D9 },
  ],
  // Every drawing written as SVG too, so the four agree on the bytes.
  theme: 'dark',
  // The text-written rules and the longevity readings, so the four agree on
  // what every chart answers by rule.
  rules: { shipped: ['nabhasas'], longevity: true },
  // Every composer, so the four agree on what every chart *says* and not
  // only on what it computes (`03-design/plans-at-the-boundary.md`).
  interpret: {
    placements: true,
    readings: true,
    strength: true,
    houses: true,
    positions: true,
    aspects: true,
    conditions: true,
    karakas: true,
  },
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
});
put('chart-varga-count', charts.vargaCount);
put('chart-drishti-table', charts.drishtiTable);
put('chart-count', charts.length);
put('chart-kind', charts.kind);
put('chart-place-lat', charts.place.latitude);
put('chart-place-lon', charts.place.longitude);
put('chart-model-fnv', fnv(charts.model));
put('chart-steps', charts.steps.join(','));
put('chart-provenance-fnv', fnv(charts.decoded.provenance));
put('chart-provenance-profile', charts.provenance.profile);
put('chart-graha-count', charts.decoded.grahaCount);

for (const chart of charts) {
  const i = chart.index;
  put(`chart-${i}-instant`, chart.instant);
  put(`chart-${i}-lagna`, chart.lagnaDeg);
  put(`chart-${i}-day-lagna`, chart.dayLagnaDeg);
  put(`chart-${i}-ayanamsha`, chart.ayanamshaOffsetDeg);
  put(`chart-${i}-day-part`, chart.dayPart);
  put(`chart-${i}-day-elapsed`, chart.dayElapsed);
  put(`chart-${i}-vara`, chart.day.vara);
  put(`chart-${i}-sunrise`, chart.day.sunrise);
  put(`chart-${i}-sunset`, chart.day.sunset);
  put(`chart-${i}-date`, `${chart.day.year}-${chart.day.month}-${chart.day.dayOfMonth}`);
  put(`chart-${i}-ghati`, chart.timing.ghati);
  put(`chart-${i}-pala`, chart.timing.pala);
  put(`chart-${i}-vipala`, chart.timing.vipala);
  put(`chart-${i}-hora-number`, chart.timing.horaNumber);
  put(`chart-${i}-hora-lord`, chart.timing.horaLord);
  chart.states.forEach((state, j) => {
    const key = `chart-${i}-state-${j}`;
    put(key, state.graha);
    put(`${key}-sign`, state.sign);
    put(`${key}-house`, state.house);
    put(`${key}-dignity`, state.dignity);
    put(`${key}-natural`, state.friendship.natural);
    put(`${key}-compound`, state.friendship.compound);
    put(`${key}-dispositor`, state.friendship.dispositor ?? 'none');
    put(`${key}-burning`, state.combustion.burning);
    put(`${key}-from-sun`, state.combustion.fromSunDeg ?? 'none');
    put(`${key}-orb`, state.combustion.orbDeg ?? 'none');
    put(`${key}-age`, state.age);
    put(`${key}-wakefulness`, state.wakefulness);
    put(`${key}-deeptadi`, state.deeptadi ?? 'none');
    put(`${key}-holding`, state.lajjitadi.holding.join(',') || 'none');
    put(`${key}-undecided`, state.lajjitadi.undecided.join(',') || 'none');
    put(`${key}-war`, state.war ? `${state.war.opponent}:${state.war.isWinner}` : 'none');
    put(
      `${key}-sayanadi`,
      state.sayanadi ? `${state.sayanadi.avastha} ${state.sayanadi.cheshtas.join(',')}` : 'none',
    );
    put(`${key}-sign-edge`, state.boundaries.signDeg);
  });
  chart.bhavas.forEach((bhava) => {
    put(`chart-${i}-bhava-${bhava.number}-sign`, bhava.sign);
    put(`chart-${i}-bhava-${bhava.number}-lord`, bhava.lord);
    put(`chart-${i}-bhava-${bhava.number}-quadrant`, bhava.quadrant);
  });
  chart.points.forEach((found, k) => {
    put(`chart-${i}-point-${k}`, found.point);
    put(`chart-${i}-point-${k}-lon`, found.longitudeDeg);
    put(`chart-${i}-point-${k}-sign`, found.sign);
    put(`chart-${i}-point-${k}-sign-edge`, found.boundaries.signDeg);
  });
  put(`chart-${i}-point-count`, chart.points.length);
  // **Every drishti**, because the count differs from chart to chart —
  // which is why the section is ragged — so a runner that printed only
  // the count would agree while the rows disagreed.
  put(`chart-${i}-aspect-count`, chart.aspects.length);
  chart.aspects.forEach((drishti, k) => {
    put(`chart-${i}-aspect-${k}`, `${drishti.from}>${drishti.to}`);
    put(`chart-${i}-aspect-${k}-houses`, drishti.houses);
    put(`chart-${i}-aspect-${k}-strength`, drishti.strength);
    put(`chart-${i}-aspect-${k}-from-sign`, drishti.fromEdge.signDeg);
    put(`chart-${i}-aspect-${k}-to-sign`, drishti.toEdge.signDeg);
  });
  put(`chart-${i}-rules-present`, chart.rules.present.map((held) => held.rule).join(','));
  put(`chart-${i}-rules-pindayu`, chart.rules.longevity.ayurdaya.pindayu.years);
  // **Every item said**, not merely counted: this is the only place the
  // four bindings are compared on text, and it exercises the composers,
  // the params shape and the locale engine in one comparison.
  for (const [composer, items] of Object.entries(chart.plans)) {
    put(`chart-${i}-plan-${composer}-count`, items.length);
    items.forEach((item, n) => {
      put(`chart-${i}-plan-${composer}-${n}`, `${item.key}: ${ctx.intl.render(item.key, item.params).text}`);
    });
  }
  chart.drawings.forEach((drawing, d) => {
    const key = `chart-${i}-drawing-${d}`;
    put(key, drawing.layout);
    put(`${key}-varga`, drawing.varga);
    put(`${key}-cells`, drawing.cells.length);
    put(`${key}-frames`, drawing.frame.length);
    put(`${key}-marks`, drawing.marks.length);
    put(`${key}-svg`, drawing.svg);
    drawing.cells.forEach((cell, c) => {
      const at = `${key}-cell-${c}`;
      put(`${at}-sign`, cell.sign);
      put(`${at}-house`, cell.house);
      put(`${at}-lagna`, cell.lagna);
      put(`${at}-ring`, cell.ring);
      put(`${at}-bodies`, cell.bodies.join(',') || 'none');
      put(`${at}-label`, `${number(cell.label.x)},${number(cell.label.y)}`);
      put(`${at}-anchor`, `${number(cell.anchor.x)},${number(cell.anchor.y)}`);
      put(`${at}-start`, `${number(cell.outline.start.x)},${number(cell.outline.start.y)}`);
      put(`${at}-steps`, cell.outline.segments.map((step) => step.kind).join(','));
    });
    drawing.marks.forEach((mark, m) => {
      const at = `${key}-mark-${m}`;
      put(at, mark.body);
      put(`${at}-at`, `${number(mark.at.x)},${number(mark.at.y)}`);
      put(`${at}-lon`, mark.longitudeDeg);
    });
  });
  put(`chart-${i}-dasha-count`, chart.dashas.length);
  const av = chart.ashtakavarga;
  put(`chart-${i}-ashtakavarga`, `${av.shodhana} ${av.ekadhipatya}`);
  av.grahas.forEach((g) => {
    const key = `chart-${i}-ashtakavarga-${g.graha}`;
    put(key, g.bindus.join(','));
    put(`${key}-reduced`, g.reduced ? g.reduced.join(',') : null);
    put(`${key}-pindas`, `${g.rashiPinda},${g.grahaPinda},${g.yogaPinda}`);
  });
  put(`chart-${i}-sarvashtakavarga`, `${av.sarva.join(',')};${av.trikona.join(',')};${av.reduced.join(',')}`);
  chart.shadbala.grahas.forEach((g) => {
    const key = `chart-${i}-shadbala-${g.graha}`;
    const { sthana: st, kaala: ka } = g;
    const parts = [st.uchcha, st.saptavargaja, st.ojayugma, st.kendradi, st.drekkana, g.dig];
    parts.push(ka.nathonnatha, ka.paksha, ka.tribhaga, ka.abda, ka.masa, ka.vara, ka.hora, ka.ayana, ka.yuddha);
    parts.push(g.cheshta, g.naisargika, g.drik);
    put(key, parts.map(number).join(','));
    put(`${key}-total`, `${number(g.virupas)},${number(g.rupas)},${number(g.requiredRupas)},${g.strong},${number(g.ishta)},${number(g.kashta)},${number(g.subhaRashmi)},${number(g.ashubhaRashmi)}`);
  });
  chart.bhavaBala.bhavas.forEach((b) => {
    put(`chart-${i}-bhava-bala-${b.bhava}`, `${b.lord} ${[b.adhipati, b.dig, b.drishti, b.special, b.virupas].map(number).join(',')}`);
  });
  chart.vaiseshikamsa.grahas.forEach((g) => {
    const standings = [g.shadvarga, g.saptavarga, g.dashavarga, g.shodashavarga];
    put(`chart-${i}-vaiseshikamsa-${g.graha}`, `${standings.map((s) => `${s.goodVargas}:${s.name}`).join(',')} ${g.impaired}`);
  });
  chart.dashaPhala.grahas.forEach((g) => {
    put(
      `chart-${i}-dasha-phala-${g.graha}`,
      `${g.subhankas.map(number).join(',')} ${g.nature} ${g.phase} ${g.favourable} ${g.unfavourable}`,
    );
  });
  const vs = chart.vimshopaka;
  put(`chart-${i}-vimshopaka`, vs.scoring);
  vs.grahas.forEach((g) => {
    put(`chart-${i}-vimshopaka-${g.graha}`, [g.shadvarga, g.saptavarga, g.dashavarga, g.shodashavarga].map(number).join(','));
  });
  chart.dashas.forEach((dasha, j) => {
    const key = `chart-${i}-dasha-${j}`;
    const balance = dasha.balance;
    put(key, dasha.system);
    put(`${key}-seed`, dasha.seed);
    put(`${key}-first-lord`, dasha.firstLord);
    put(`${key}-overflow`, dasha.overflow);
    put(`${key}-balance`, balance?.method ?? null);
    put(`${key}-remaining`, balance?.remaining ?? null);
    put(`${key}-balance-days`, balance?.days ?? null);
    const w = balance?.written;
    put(`${key}-balance-written`, w ? [w.years, w.months, w.days, w.hours, w.minutes].join(',') : null);
    put(`${key}-moon-span-from`, dasha.moonSpan?.from ?? null);
    put(`${key}-moon-span-to`, dasha.moonSpan?.to ?? null);
    put(`${key}-depth`, dasha.depth);
    put(`${key}-periods`, dasha.periods.length);
    dasha.periods.forEach((period, k) => {
      if (period.level > 2) return;
      put(`${key}-period-${k}`, `${period.path}${period.sign ? ` ${period.sign}` : ''} ${period.lord}`);
      put(`${key}-period-${k}-from`, period.from);
      put(`${key}-period-${k}-to`, period.to);
    });
    put(`${key}-at`, dasha.at(chart.instant + 5000).map((period) => period.path).join(','));
  });
  chart.vargas.forEach((varga, v) => {
    put(`chart-${i}-varga-${v}`, varga.varga);
    put(`chart-${i}-varga-${v}-lagna-rashi`, varga.lagna.rashi);
    put(`chart-${i}-varga-${v}-lagna-part`, varga.lagna.part);
    put(`chart-${i}-varga-${v}-lagna-sign`, varga.lagna.sign);
    varga.grahas.forEach((placed, j) => {
      put(`chart-${i}-varga-${v}-graha-${j}`, placed.graha);
      put(`chart-${i}-varga-${v}-graha-${j}-rashi`, placed.at.rashi);
      put(`chart-${i}-varga-${v}-graha-${j}-part`, placed.at.part);
      put(`chart-${i}-varga-${v}-graha-${j}-sign`, placed.at.sign);
    });
  });
  chart.grahas.forEach((graha, j) => {
    put(`chart-${i}-graha-${j}`, graha.graha);
    put(`chart-${i}-graha-${j}-lon`, graha.longitudeDeg);
    put(`chart-${i}-graha-${j}-lat`, graha.latitudeDeg);
    put(`chart-${i}-graha-${j}-speed`, graha.speedDegPerDay);
    put(`chart-${i}-graha-${j}-retro`, graha.retrograde);
    put(`chart-${i}-graha-${j}-house`, graha.house.bhava);
    put(`chart-${i}-graha-${j}-house-method`, graha.house.method);
    put(`chart-${i}-graha-${j}-placement`, graha.placement.bhava);
  });
  chart.houses.forEach((bhava, k) => {
    put(`chart-${i}-house-${k}-madhya`, bhava.madhyaDeg);
    put(`chart-${i}-house-${k}-sandhi`, bhava.sandhiDeg);
  });
  chart.chalit.forEach((bhava, k) => {
    put(`chart-${i}-chalit-${k}-madhya`, bhava.madhyaDeg);
  });
}
// `found` is the batch of one unwrapped, and must agree with the batch.
const single = geo.chart.found({ instant: 2460482.5, place, utcOffsetSeconds: 20700 });
put('chart-single-lagna', single.lagnaDeg);
put('chart-single-agrees', single.lagnaDeg === charts.at(0).lagnaDeg);

// ── An almanac ─────────────────────────────────────────────────────────
// Three days, because a day's lists are ragged and two consecutive days
// with the same counts would not exercise the offsets.
const week = geo.almanac.of({
  from: gregorian(2024, 6, 17),
  to: gregorian(2024, 6, 19),
  place,
  utcOffsetSeconds: 20700,
});
put('almanac-days', week.length);
put('almanac-calendar', week.calendar);
put('almanac-place-lat', week.place.latitude);
put('almanac-model-fnv', fnv(week.model));
put('almanac-provenance-fnv', fnv(week.decoded.provenance));

for (const day of week) {
  const i = day.index;
  put(`day-${i}-vara`, day.day.vara);
  put(`day-${i}-sunrise`, day.day.sunrise);
  put(`day-${i}-sunset`, day.day.sunset);
  put(`day-${i}-next-sunrise`, day.day.nextSunrise);
  put(`day-${i}-date`, `${day.day.year}-${day.day.month}-${day.day.dayOfMonth}`);
  put(`day-${i}-window-from`, day.window.from);
  put(`day-${i}-window-to`, day.window.to);
  put(`day-${i}-month`, day.month.month);
  put(`day-${i}-amanta`, day.month.amanta);
  put(`day-${i}-purnimanta`, day.month.purnimanta);
  put(`day-${i}-paksha`, day.month.paksha);
  put(`day-${i}-convention`, day.month.convention);
  put(`day-${i}-month-kind`, day.month.kind);
  put(`day-${i}-ayana`, day.ayana);
  put(`day-${i}-disha-shool`, day.dishaShool);
  // An absent value must be absent in all three, not nought in one.
  put(`day-${i}-sankranti`, day.sankranti === null ? 'none' : number(day.sankranti));
  put(`day-${i}-abhijit`, day.abhijit === null ? 'none' : number(day.abhijit.from));
  put(`day-${i}-abhijit-effective`, day.abhijit === null ? 'none' : day.abhijit.effective);
  put(`day-${i}-brahma`, day.brahma === null ? 'none' : number(day.brahma.from));
  // The counts are what the ragged layout turns on: if a binding's
  // prefix sum were off by a day, these would still agree and the spans
  // below would not.
  put(`day-${i}-tithi-count`, day.tithi.length);
  put(`day-${i}-nakshatra-count`, day.nakshatra.length);
  put(`day-${i}-yoga-count`, day.yoga.length);
  put(`day-${i}-karana-count`, day.karana.length);
  put(`day-${i}-kaala-count`, day.kaalas.length);
  put(`day-${i}-choghadiya-count`, day.choghadiya.length);
  put(`day-${i}-hora-count`, day.horas.length);
  put(`day-${i}-muhurta-count`, day.muhurtas.length);
  put(`day-${i}-moon-event-count`, day.moonEvents.length);
  put(`day-${i}-panchaka-count`, day.panchaka.length);
  put(`day-${i}-moon-sign-count`, day.moonSigns.length);
  put(`day-${i}-sun-sign-count`, day.sunSigns.length);
  put(`day-${i}-muhurta-yoga-count`, day.muhurtaYogas.length);
  for (const [limb, spans] of [
    ['tithi', day.tithi],
    ['nakshatra', day.nakshatra],
    ['yoga', day.yoga],
    ['karana', day.karana],
  ]) {
    spans.forEach((span, j) => {
      put(`day-${i}-${limb}-${j}`, span.member);
      put(`day-${i}-${limb}-${j}-whole-from`, span.whole.from);
      put(`day-${i}-${limb}-${j}-inside-to`, span.inside.to);
    });
  }
  day.kaalas.forEach((kaala, j) => {
    put(`day-${i}-kaala-${j}`, kaala.kaala);
    put(`day-${i}-kaala-${j}-from`, kaala.from);
  });
  put(`day-${i}-hora-0-lord`, day.horas[0].lord);
  put(`day-${i}-hora-0-start`, day.horas[0].start);
  put(`day-${i}-hora-23-lord`, day.horas[day.horas.length - 1].lord);
  put(`day-${i}-choghadiya-0`, day.choghadiya[0].choghadiya);
  put(`day-${i}-choghadiya-0-daytime`, day.choghadiya[0].daytime);
  put(`day-${i}-muhurta-0-from`, day.muhurtas[0].from);
  put(`day-${i}-muhurta-last-daylight`, day.muhurtas[day.muhurtas.length - 1].daylight);
  day.moonEvents.forEach((event, j) => {
    put(`day-${i}-moon-${j}-kind`, event.kind);
    put(`day-${i}-moon-${j}-instant`, event.instant);
  });
  day.muhurtaYogas.forEach((held, j) => {
    put(`day-${i}-yoga-held-${j}`, held.yoga);
    put(`day-${i}-yoga-held-${j}-cause`, held.because.kind);
    put(`day-${i}-yoga-held-${j}-tithi`, held.because.tithi ?? 'none');
  });
}
// `almanacDay` is the range of one unwrapped, and must agree.
const oneDay = geo.almanac.day({ date: gregorian(2024, 6, 17), place, utcOffsetSeconds: 20700 });
put('almanac-single-agrees', oneDay.day.sunrise === week.at(0).day.sunrise);
geo.dispose();

// ── The surface's shape ─────────────────────────────────────────────
//
// The lines above compare what the bindings ANSWER. These compare where
// an operation LIVES: every key is the canonical `area.operation` path,
// and the member each binding references beside it is its own spelling
// of it. A binding that moved an operation to another area, or renamed
// one, prints a key the others do not and the gate fails — which is what
// `03-design/surface-areas.md` asks of this runner, and what
// `check-parity` could not see before.
//
// Referenced rather than called: a call needs arguments and some of them
// need an ephemeris, and what is being compared is the shape.
const shape = new Context({ testProvider: true, profile: 'nepali-default' });
for (const [path, member] of [
  ['calendar.date_of', shape.calendar.dateOf],
  ['calendar.fixed_of', shape.calendar.fixedOf],
  ['calendar.convert', shape.calendar.convert],
  ['calendar.weekday_of', shape.calendar.weekdayOf],
  ['calendar.month_length', shape.calendar.monthLength],
  ['calendar.is_leap', shape.calendar.isLeap],
  ['time.resolve', shape.time.resolve],
  ['time.civil_of', shape.time.civilOf],
  ['time.convert', shape.time.convert],
  ['time.delta_t', shape.time.deltaT],
  ['intl.locale', shape.intl.locale],
  ['intl.render', shape.intl.render],
  ['intl.has', shape.intl.has],
  ['intl.transliterate', shape.intl.transliterate],
  ['intl.entity', shape.intl.entity],
  ['intl.messages', shape.intl.messages],
  ['intl.load_pack', shape.intl.loadPack],
  ['keys.id', shape.keys.id],
  ['keys.name', shape.keys.name],
  ['frame.canonical', shape.frame.canonical],
  ['frame.pack', shape.frame.pack],
  ['frame.unpack', shape.frame.unpack],
  ['chart.layout', shape.chart.layout],
  ['chart.found', shape.chart.found],
  ['chart.found_many', shape.chart.foundMany],
  ['almanac.of', shape.almanac.of],
  ['almanac.day', shape.almanac.day],
  ['engine.names', shape.engine.names],
  ['engine.signature', shape.engine.signature],
  ['engine.call', shape.engine.call],
  ['engine.call_json', shape.engine.callJson],
  ['engine.manifest', shape.engine.manifest],
  ['engine.manifest_json', shape.engine.manifestJson],
  ['(root).engine', shape.engine],
  ['(root).positions', shape.positions],
  ['(root).profile', shape.profile],
  ['(root).settings', shape.settings],
  ['(root).settings_json', shape.settingsJson],
  ['(root).settings_hash', shape.settingsHash],
  ['(root).dispose', shape.dispose],
]) {
  put(`surface.${path}`, member === undefined || member === null ? 'missing' : 'present');
}
shape.dispose();

for (const key of [...report.keys()].sort()) {
  process.stdout.write(`${key}\t${report.get(key)}\n`);
}
