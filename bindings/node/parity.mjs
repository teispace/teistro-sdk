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

/**
 * A local day's every field, under the same keys for a chart's day and an
 * almanac's, because the two layers hand back one record.
 */
const putDay = (prefix, day) => {
  put(`${prefix}-vara`, day.vara);
  put(`${prefix}-sunrise`, day.sunrise);
  put(`${prefix}-sunset`, day.sunset);
  put(`${prefix}-next-sunrise`, day.nextSunrise);
  put(`${prefix}-date`, `${day.date.year}-${day.date.month}-${day.date.day}`);
  put(`${prefix}-calendar`, day.date.calendar);
  put(`${prefix}-era`, day.date.era ?? 'none');
  put(`${prefix}-era-year`, day.date.eraYear);
  put(`${prefix}-resolution`, day.date.resolution);
  put(`${prefix}-polar`, day.polar === null ? 'none' : `${day.polar.kind}/${day.polar.policy}`);
  put(
    `${prefix}-convention`,
    day.air !== null
      ? `${day.convention} ${number(day.air.pressureHpa)} hPa ${number(day.air.temperatureC)} C`
      : (day.convention ?? `custom ${number(day.customAltitudeDeg)}`),
  );
};

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
  resolution: 'DEFINED',
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
const zone = { kind: 'IANA', offsetSeconds: 0, longitudeDeg: 0, zone: 'Asia/Kathmandu' };
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
put('provenance-fnv', fnv(positions.provenanceJson));
put('provenance-profile', positions.provenance.profile);
put('provenance-settings-hash', positions.provenance.settingsHash);
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

// ── A chart founded on a classical astronomy ───────────────────────────
// The Surya Siddhanta by name: the text's zodiac, places, Lagna and day
// (docs/03-design/classical-chart.md), which every binding reaches through
// the selector and must read back alike, deviation and all.
const classical = new Context({
  profile: 'surya-siddhanta',
  ephemeris: 'SURYA_SIDDHANTA',
});
const text = classical.chart.found({ instant: 2447995.4895833335, place, utcOffsetSeconds: 20700 });
put('classical-steps', text.batch.steps.join(','));
put('classical-lagna', text.lagnaDeg);
put('classical-sunrise', text.day.sunrise);
put('classical-deviation', `${text.provenance.deviation.model}: ${text.provenance.deviation.detail}`);
text.grahas.forEach((graha, j) => put(`classical-graha-${j}-lon`, graha.longitudeDeg));
classical.dispose();

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
const parityDasha = JSON.parse('{"kernel":"UDU","key":"ACME_PARITY","sources":["the parity scenario"],"lords":[{"graha":"SUN","years":5},{"graha":"MOON","years":10},{"graha":"MARS","years":7},{"graha":"MERCURY","years":12}],"reference":"MULA","count":"TO_REFERENCE","span":2,"offset":1,"repeats":true,"year_length":"SAVANA_360","depth":2}');
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
  dashas: [
    DashaSystem.Vimshottari,
    DashaSystem.Chara,
    DashaSystem.Kalachakra,
    DashaSystem.ReleasingFortune,
    DashaSystem.Profection,
    DashaSystem.Firdaria,
    DashaSystem.Decennials,
    'dasha_system.ACME_PARITY',
  ],
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
  theme: 'DARK',
  // The text-written rules and the longevity readings, so the four agree on
  // what every chart answers by rule.
  rules: {
    shipped: ['NABHASAS'],
    longevity: true,
    ayurdaya: { enemy_exempt: 'mars', enmity: 'compound', rising: 'every' },
  },
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
  jaimini: true,
  avakahada: true,
  outerPlanets: true,
  gochar: { instants: [2460676.5, 2460736.5], ashtakavarga: true },
  hits: { from: 2460676.5, to: 2460736.5, grahas: ['SUN', 'MERCURY', 'SATURN'], aspects: [0, 90, 180], orbDeg: 2 },
  sadeSati: { from: 2460676.5, to: 2464329, reckoning: 'DEGREE', spells: [4, 7, 8] },
  kp: { number: 74, anyAyanamsha: true },
  fortitudes: {
    dignities: { sectRule: 'DAYLIGHT', rules: { terms: 'EGYPTIAN', triplicities: 'PTOLEMY' }, scores: { peregrine: 0 } },
    rules: { beamsDeg: 15, combustionInSign: false, partile: { WITHIN: { orbDeg: 1 } }, siege: { WITHIN: { spanDeg: 30 } } },
    scores: { regulus: 5 },
    almuten: { fortune: 'REVERSED_BY_NIGHT' },
  },
  lots: { fortune: 'REVERSED_WHILE_MOON_UP' },
  considerations: { moonLateFromDeg: 25 },
  perfection: { house: 7, rules: { horizonDays: 120 } },
  westernAspects: {
    aspects: ['CONJUNCTION', 'SEXTILE', 'SQUARE', 'TRINE', 'QUINCUNX', 'OPPOSITION'],
    orbs: {
      model: 'MOIETIES',
      orbs: [
        ['SUN', 17], ['MOON', 12.5], ['MERCURY', 7], ['VENUS', 8], ['MARS', 7.5],
        ['JUPITER', 12], ['SATURN', 10], ['URANUS', 5], ['NEPTUNE', 5], ['PLUTO', 5],
      ].map(([graha, orbDeg]) => ({ graha, orbDeg })),
    },
  },
  parallels: { orbDeg: 1.5 },
  antiscia: { cusps: {} },
  midpoints: { orbDeg: 1.5 },
  westernHouses: {},
  harmonic: { number: 5 },
  matching: {
    partner: { instant: 2451545.25, place: { latitude: -33.87, longitude: 151.21, altitude: 0 }, utcOffsetSeconds: 36000 },
    partnerRole: 'BRIDE',
    rules: { bhakootLift: 'GARGA' },
    porutham: { lordsFriendship: 'ONE_WAY' },
    kuja: { houses: 'WITH_SECOND', from: 'LAGNA_MOON_VENUS' },
  },
  synastry: {
    partner: { instant: 2451545.25, place: { latitude: -33.87, longitude: 151.21, altitude: 0 }, utcOffsetSeconds: 36000 },
    aspects: ['CONJUNCTION', 'SQUARE', 'TRINE', 'OPPOSITION'],
    zodiac: 'CHARTS',
    parallels: { orbDeg: 1.5 },
    antiscia: { orbs: { model: 'LEO' } },
    midpoints: { orbDeg: 1.5 },
    composite: true,
    davison: true,
  },
  progressions: {
    at: 2470000.5,
    year: 'NOON_SIDEREAL_TIME',
    angles: 'SOLAR_ARC_LONGITUDE',
    direction: 'NAIBOD',
    contacts: {
      from: 2462000.5,
      to: 2465652.5,
      grahas: ['MOON', 'SUN'],
      points: ['LAGNA', 'MARS', 'VENUS'],
      aspects: [0, 45, 90, 135, 180],
    },
  },
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
put('chart-provenance-fnv', fnv(charts.provenanceJson));
put('chart-provenance-profile', charts.provenance.profile);
put('chart-graha-count', charts.decoded.grahaCount);

/** A clause as every runner prints it: 1 held, 0 not. */
const flag = (value) => (value ? 1 : 0);

/** An Ashta Koota as every runner prints it, under `prefix`. */
function putAshta(prefix, matched) {
  const readingText = (r) => {
    switch (r.koota) {
      case 'koota.VASHYA':
        return r.relation;
      case 'koota.TARA':
        return `${r.brideToGroom} ${r.groomToBride}`;
      case 'koota.YONI':
        return `${r.bride} ${r.groom} ${r.relation}`;
      case 'koota.GRAHA_MAITRI':
        return `${r.bride} ${r.groom} ${r.relation} ${flag(r.lifted)}`;
      case 'koota.GANA':
        return `${r.bride} ${r.groom} ${flag(r.dosha)} ${flag(r.lifted)}`;
      case 'koota.BHAKOOT': {
        const e = r.exceptions;
        return [r.apart, r.dosha ?? 'NONE', e.oneLord, e.lordsFriends, e.navamshaLordsFriends, e.taraPure, e.vashya, r.lifted]
          .map((cell) => (typeof cell === 'boolean' ? flag(cell) : cell))
          .join(' ');
      }
      case 'koota.NADI':
        return `${r.bride} ${r.groom} ${flag(r.dosha)} ${flag(r.lifted)}`;
      default:
        return `${r.bride} ${r.groom}`;
    }
  };
  put(`${prefix}-matching`, number(matched.total));
  matched.kootas.forEach((row) =>
    put(
      `${prefix}-matching-${row.reading.koota}`,
      `${number(row.points)} ${number(row.maxPoints)} ${readingText(row.reading)}`,
    ),
  );
}

/** Ten considerations as every runner prints them, under `prefix`. */
function putPorutham(prefix, ten) {
  const e = ten.exception;
  put(
    `${prefix}-porutham`,
    `${ten.agreeing} ${ten.chiefAgreeing} ${flag(e.oneLord)} ${flag(e.lordsFriendly)} ${flag(e.opposite)}`,
  );
  const tenText = (r) => {
    switch (r.koota) {
      case 'koota.TARA':
        return `${r.count} ${r.rule}`;
      case 'koota.GANA':
        return `${r.bride} ${r.groom} ${flag(r.diminished)}`;
      case 'koota.MAHENDRA':
      case 'koota.STREE_DEERGHA':
        return `${r.count}`;
      case 'koota.YONI':
        return `${r.bride} ${r.groom} ${flag(r.hostile)}`;
      case 'koota.BHAKOOT':
        return `${r.apart}`;
      case 'koota.GRAHA_MAITRI':
        return `${r.bride} ${r.groom} ${flag(r.brideCallsFriend)} ${flag(r.groomCallsFriend)}`;
      case 'koota.VASHYA':
        return `${flag(r.brideToGroom)} ${flag(r.groomToBride)}`;
      case 'koota.VEDHA':
        return `${flag(r.pierced)}`;
      default:
        return `${r.bride} ${r.groom}`;
    }
  };
  ten.considerations.forEach((row) =>
    put(`${prefix}-porutham-${row.reading.koota}`, `${flag(row.agrees)} ${flag(row.lifted)} ${tenText(row.reading)}`),
  );
}

// Two pairs of names, as every runner asks them: a Devanagari pair whose
// groom's syllable is Abhijit's, placed in Shravana, and an IAST pair.
for (const [n, [bride, groom, rules]] of [
  ['प्रिया', 'ज़ोया', { name: { abhijit: 'SHRAVANA' }, koota: { nadiDosha: 'MIDDLE_ONLY' } }],
  ['kṛṣṇā', 'śyāma', { name: { latin: 'IAST' }, porutham: { deerghaBeyond: 'SEVENTH' } }],
].entries()) {
  const read = ctx.matching.naam(bride, groom, rules);
  for (const [who, name] of [['bride', read.bride], ['groom', read.groom]]) {
    put(`naam-${n}-${who}`, `${name.cell} ${name.nakshatra ?? 'NONE'} ${name.quarter} ${name.varga}`);
  }
  put(`naam-${n}-varga`, `${read.varga.bride} ${read.varga.groom} ${read.varga.relation}`);
  putAshta(`naam-${n}`, read.ashta);
  putPorutham(`naam-${n}`, read.porutham);
}

for (const chart of charts) {
  const i = chart.index;
  put(`chart-${i}-content-hash`, chart.provenance.contentHash);
  put(`chart-${i}-instant`, chart.instant);
  put(`chart-${i}-lagna`, chart.lagnaDeg);
  put(`chart-${i}-day-lagna`, chart.dayLagnaDeg);
  put(`chart-${i}-ayanamsha`, chart.ayanamshaOffsetDeg);
  put(`chart-${i}-day-part`, chart.dayPart);
  put(`chart-${i}-day-elapsed`, chart.dayElapsed);
  putDay(`chart-${i}`, chart.day);
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
  put(`chart-${i}-rules-rays`, chart.rules.longevity.rasmi.total);
  put(`chart-${i}-rules-span`, chart.rules.longevity.choice.ayus ?? '');
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
  const { karakamsha: k, brahma: b } = chart.jaimini;
  put(`chart-${i}-jaimini`, `${k.atmakaraka} ${k.sign} ${k.inRasi.join(',')} ${k.inNavamsha.join(',')}`);
  put(`chart-${i}-graha-arudhas`, chart.jaimini.grahaArudhas.map((sign) => sign ?? '-').join(','));
  const birth = chart.avakahada;
  put(
    `chart-${i}-avakahada`,
    `${birth.nakshatra} ${birth.pada} ${birth.rashi} ${birth.nakshatraLord} ${birth.rashiLord} ${birth.varna} ${birth.yoni} ${birth.gana} ${birth.nadi}`,
  );
  const syllable = birth.syllable;
  put(`chart-${i}-avakahada-syllable`, `${syllable.cell} ${syllable.devanagari} ${syllable.iast} ${syllable.varga}`);
  chart.gochar.forEach((reading, at) => {
    const { reference: ref, rules } = reading;
    put(
      `chart-${i}-gochar-${at}`,
      `${number(reading.instant)} ${ref.from} ${ref.sign} ${rules.nodeVedha} ${rules.nodeObstruction} ${rules.ashtakavargaGoodFrom}`,
    );
    reading.ashtakavarga.forEach((a, k) => {
      put(
        `chart-${i}-gochar-${at}-av-${k}`,
        `${a.graha} ${a.bindus} ${a.good} ${a.kakshya.index} ${a.kakshya.lord} ${a.kakshyaBindu} ${a.sarva} ${a.sarvaStanding}`,
      );
    });
    reading.grahas.forEach((g, k) => {
      put(
        `chart-${i}-gochar-${at}-${k}`,
        `${g.graha} ${g.transit.sign} ${number(g.transit.degrees)} ${g.house} ${g.goodHouse} ` +
          `${g.vedhaHouse ?? '-'} ${g.obstructedBy.join(',') || '-'} ${g.verdict} ${g.fruition} ${g.fruitfulNow}`,
      );
    });
  });
  chart.hits.forEach((hit, k) => {
    const e = hit.event;
    const to = e.to === undefined ? '-' : e.to.point === 'LAGNA' ? 'LAGNA' : e.to.graha;
    put(
      `chart-${i}-hit-${k}`,
      `${number(hit.instant)} ${hit.graha} ${e.kind} ${e.into ?? '-'} ${e.motion ?? e.turns} ${to} ` +
        `${e.angle ?? '-'} ${e.phase ?? '-'}`,
    );
  });
  const ss = chart.sadeSati;
  put(`chart-${i}-sade-sati`, `${ss.reference.from} ${ss.reference.sign} ${ss.reckoning}`);
  const bound = (jd) => (jd === null ? '-' : number(jd));
  [...ss.sadeSati.map((one) => one.phases), ...ss.spells.map((spell) => [spell])]
    .flatMap((spells, period) => spells.flatMap((spell) => spell.visits.map((v) => `${period} ${spell.house} ${bound(v.from)} ${bound(v.to)}`)))
    .forEach((line, k) => put(`chart-${i}-sade-sati-${k}`, line));
  const kp = chart.kp;
  const keys = (list) => (list.length === 0 ? '-' : list.join(','));
  const level = ({ lord, span }) => `${lord} ${span.start} ${span.end}`;
  const lords = (l) => `${l.sign} ${level(l.star)} ${level(l.sub)} ${level(l.subSub)}`;
  const rules = kp.ruling.rules;
  put(`chart-${i}-kp`, `${kp.chart.system} ${rules.count} ${rules.nodeRulers} ${rules.retrogradeRejection}`);
  for (const cusp of kp.chart.cusps) put(`chart-${i}-kp-cusp-${cusp.house}`, `${cusp.longitude} ${lords(cusp.lords)}`);
  for (const p of kp.chart.planets) {
    put(`chart-${i}-kp-planet-${p.graha}`, `${p.longitude} ${p.retrograde} ${p.house} ${lords(p.lords)}`);
  }
  for (const h of kp.significators.houses) {
    put(
      `chart-${i}-kp-house-${h.house}`,
      `${keys(h.inOccupantsStars)} ${keys(h.occupants)} ${keys(h.inLordsStar)} ${h.lord} ` +
        `${keys(h.conjoined)} ${keys(h.aspected)} ${keys(h.intercepted)}`,
    );
  }
  for (const n of kp.significators.nodes) {
    put(`chart-${i}-kp-node-${n.node}`, `${keys(n.conjoined)} ${n.starLord} ${keys(n.aspecting)} ${n.signLord}`);
  }
  const rejection = (by) => (by === null ? '-' : `${by.retrograde}:${by.byStar}`);
  kp.ruling.rulers.forEach((r, k) => {
    const reasons = r.reasons.map((why) => (why.kind === 'AGENT' ? `AGENT:${why.of}:${why.by}` : why.kind));
    put(
      `chart-${i}-kp-ruler-${k}`,
      `${r.graha} ${reasons.join(',')} ${r.retrograde} ${rejection(r.rejectedBy)} ${rejection(r.rejectedBySub)}`,
    );
  });
  const dg = chart.dignities;
  const s = dg.scores;
  put(
    `chart-${i}-dignities`,
    `${dg.sect} ${dg.sectRule} ${dg.rules.terms} ${dg.rules.triplicities} ` +
      [s.house, s.exaltation, s.triplicity, s.term, s.face, s.detriment, s.fall, s.peregrine].join(','),
  );
  const DIGNITY_FLAGS = ['house', 'exaltation', 'triplicity', 'term', 'face', 'detriment', 'fall'];
  for (const at of dg.planets) {
    const held = [...DIGNITY_FLAGS.filter((flag) => at.dignity[flag]), ...(at.peregrine ? ['peregrine'] : [])];
    put(
      `chart-${i}-dignity-${at.planet}`,
      `${number(at.longitudeDeg)} ${held.join(',') || '-'} ${at.score} ${at.reception}`,
    );
  }
  dg.receptions.forEach((one, k) => {
    const [firstIn, secondIn] = [one.firstIn, one.secondIn].map((side) =>
      DIGNITY_FLAGS.filter((flag) => side[flag]).join(','),
    );
    put(
      `chart-${i}-reception-${k}`,
      `${one.planets[0]} ${one.planets[1]} ${firstIn} ${secondIn} ${one.mutual.join(',') || '-'}`,
    );
  });
  const ft = chart.fortitudes;
  const withOrb = (reading) => (typeof reading === 'string' ? reading : `WITHIN:${number(Object.values(reading.WITHIN)[0])}`);
  const numbers = (values) => values.map(number).join(',');
  put(
    `chart-${i}-fortitudes`,
    `${ft.sky.houses} ${[ft.sky.northNodeDeg, ft.sky.regulusDeg, ft.sky.spicaDeg, ft.sky.algolDeg].map(number).join(' ')}`,
  );
  const fr = ft.rules;
  put(
    `chart-${i}-fortitude-rules`,
    `${number(fr.combustionDeg)} ${fr.combustionInSign ? 1 : 0} ` +
      `${[fr.beamsDeg, fr.cazimiDeg, fr.cuspOrbDeg, fr.starOrbDeg].map(number).join(' ')} ` +
      `${withOrb(fr.partile)} ${withOrb(fr.siege)} ${numbers(fr.meanMotionDeg)}`,
  );
  const { houses: houseScores, ...lineScores } = ft.scores;
  put(`chart-${i}-fortitude-scores`, `${houseScores.join(',')} ${Object.values(lineScores).join(',')}`);
  put(`chart-${i}-fortitude-houses`, numbers(ft.sky.cuspsDeg));
  ft.planets.forEach((at, k) => {
    const lines = at.accidents.map((line) => `${line.accident}:${line.points}`);
    put(
      `chart-${i}-fortitude-${at.planet}`,
      `${number(ft.sky.speedsDegPerDay[k])} ${at.house} ${lines.join(',') || '-'} ${at.fortitude} ${at.debility} ${at.net}`,
    );
  });
  const al = ft.almutens;
  put(
    `chart-${i}-almuten-rules`,
    `${al.rules.place} ${al.rules.fortune} ${[al.fortuneDeg, ft.sky.ascendantDeg, ft.sky.midheavenDeg].map(number).join(' ')}`,
  );
  const ranked = (almuten) =>
    `${almuten.totals.map((at) => at.total).join(',')} ${almuten.almutens.join(',') || '-'} ${almuten.partakers.join(',') || '-'}`;
  put(`chart-${i}-almuten-figure`, ranked(al.figure));
  put(`chart-${i}-almuten-places`, ranked(al.places));
  al.houses.forEach((almuten, h) => put(`chart-${i}-almuten-house-${h + 1}`, ranked(almuten)));
  const lt = chart.lots;
  put(`chart-${i}-lots`, `${lt.sect} ${lt.request.sectRule} ${lt.request.fortune} ${lt.fortuneReversed ? 1 : 0}`);
  lt.lots.forEach(({ lot, place }) =>
    put(`chart-${i}-lot-${lot}`, `${number(place.longitudeDeg)} ${place.sign} ${place.lord} ${place.house}`),
  );
  const cs = chart.considerations;
  const list = (values) => values.join(',') || '-';
  const perfection = (found) => (found === null ? '-' : `${found.planet} ${found.aspect} ${number(found.days)} ${number(found.gapDeg)}`);
  put(
    `chart-${i}-considerations`,
    `${cs.radicality.hourLord} ${cs.radicality.ascendantLord} ${list(cs.radicality.grounds)} ${cs.ascendant.sign} ${number(cs.ascendant.degree)} ${flag(cs.ascendant.early)} ${flag(cs.ascendant.late)} ${flag(cs.ascendant.shortAscension)}`,
  );
  put(
    `chart-${i}-considerations-moon`,
    `${cs.moon.sign} ${number(cs.moon.degree)} ${flag(cs.moon.late)} ${flag(cs.moon.lateSign)} ${flag(cs.moon.viaCombusta)} ${number(cs.moon.course.daysInSign)} ${flag(cs.moon.course.eased)}`,
  );
  put(`chart-${i}-considerations-next`, perfection(cs.moon.course.next));
  put(`chart-${i}-considerations-within`, perfection(cs.moon.course.withinOrb));
  const sv = cs.seventh;
  put(
    `chart-${i}-considerations-seventh`,
    `${number(sv.cuspDeg)} ${sv.lord} ${list(sv.infortunesInHouse)} ${flag(sv.lordRetrograde)} ${flag(sv.lordCombust)} ${flag(sv.lordInFall)} ${flag(sv.lordInInfortuneTerm)} ${sv.lordNet}`,
  );
  put(`chart-${i}-considerations-saturn`, `${cs.saturnHouse} ${flag(cs.saturnRetrograde)} ${flag(cs.ascendantLordCombust)}`);
  put(`chart-${i}-considerations-rules`, `${number(cs.rules.moonLateFromDeg)} ${cs.rules.orbsDeg.map(number).join(',')}`);
  const pf = chart.perfection;
  const held = (dignity) => list(Object.keys(dignity).filter((name) => dignity[name]));
  put(
    `chart-${i}-perfection`,
    `${pf.querent} ${pf.quesited} ${number(pf.horizonDays)} ${pf.impediments.length} ${pf.translations.length} ${pf.collections.length}`,
  );
  const ap = pf.application;
  put(
    `chart-${i}-perfection-application`,
    ap === null
      ? '-'
      : `${ap.aspect} ${number(ap.days)} ${ap.applying} ${ap.kind} ${number(ap.gapDeg)} ${flag(ap.withinMoieties)}`,
  );
  put(
    `chart-${i}-perfection-separation`,
    pf.separation === null ? '-' : `${pf.separation.aspect} ${number(pf.separation.pastDeg)}`,
  );
  const wy = pf.ways;
  put(
    `chart-${i}-perfection-ways`,
    `${wy.querent.house} ${held(wy.querent.dignity)} ${wy.quesited.house} ${held(wy.quesited.dignity)} ${flag(wy.mutualByHouse)} ${list(wy.infortunesBetween)} ${flag(wy.moonRelays)} ${flag(wy.quesitedInAscendant)} ${list(wy.held)}`,
  );
  pf.impediments.forEach((at, n) =>
    put(`chart-${i}-perfection-impediment-${n}`, `${at.kind} ${at.significator} ${at.third ?? '-'} ${at.aspect} ${number(at.days)}`),
  );
  pf.translations.forEach((at, n) =>
    put(
      `chart-${i}-perfection-translation-${n}`,
      `${at.translator} ${at.from} ${at.to} ${at.separating.aspect} ${number(at.separating.pastDeg)} ${at.aspect} ${number(at.days)} ${held(at.received)}`,
    ),
  );
  pf.collections.forEach((at, n) =>
    put(
      `chart-${i}-perfection-collection-${n}`,
      `${at.collector} ${at.fromQuerent.aspect} ${number(at.fromQuerent.days)} ${at.fromQuesited.aspect} ${number(at.fromQuesited.days)} ${held(at.collectorInQuerent)} ${held(at.collectorInQuesited)} ${held(at.querentInCollector)} ${held(at.quesitedInCollector)}`,
    ),
  );
  put(`chart-${i}-perfection-rules`, `${pf.rules.orbsDeg.map(number).join(',')} ${flag(pf.rules.withinSign)}`);
  const { progressed: pg, directed: dr, contacts } = chart.progressions;
  put(
    `chart-${i}-progressed`,
    `${number(pg.life)} ${number(pg.sky)} ${number(pg.armcDeg)} ${number(pg.angles.ascendantDeg)} ${number(pg.angles.midheavenDeg)}`,
  );
  pg.grahas.forEach((at, n) =>
    put(
      `chart-${i}-progressed-graha-${n}`,
      `${at.graha} ${number(at.longitudeDeg)} ${number(at.tropicalDeg)} ${number(at.speedDegPerDay)}`,
    ),
  );
  put(`chart-${i}-directed`, `${number(dr.arcDeg)} ${number(dr.ascendantDeg)} ${number(dr.midheavenDeg)}`);
  dr.planets.forEach((at, n) => put(`chart-${i}-directed-graha-${n}`, `${at.graha} ${number(at.longitudeDeg)}`));
  put(`chart-${i}-progressed-contact-count`, `${contacts.length}`);
  contacts.forEach((at, n) =>
    put(
      `chart-${i}-progressed-contact-${n}`,
      `${number(at.life)} ${number(at.sky)} ${at.graha} ${at.to.point === 'LAGNA' ? 'LAGNA' : at.to.graha} ${at.angle} ${at.motion}`,
    ),
  );
  const western = chart.westernAspects;
  put(`chart-${i}-western-aspect-count`, `${western.length}`);
  western.forEach((at, n) =>
    put(
      `chart-${i}-western-aspect-${n}`,
      `${at.first} ${at.second} ${at.aspect} ${number(at.apartDeg)} ${number(at.fromExactDeg)} ${number(at.orbDeg)} ${at.applying ? 1 : 0}`,
    ),
  );
  const declined = chart.declinations;
  put(
    `chart-${i}-declinations`,
    `${number(declined.obliquityDeg)} ${number(declined.lagnaDeg)} ${number(declined.midheavenDeg)}`,
  );
  declined.grahas.forEach((at) => put(`chart-${i}-declination-${at.graha}`, number(at.declinationDeg)));
  const parallels = chart.parallels;
  put(`chart-${i}-parallel-count`, `${parallels.length}`);
  parallels.forEach((at, n) =>
    put(
      `chart-${i}-parallel-${n}`,
      `${at.first} ${at.second} ${at.contrary ? 1 : 0} ${number(at.apartDeg)} ${number(at.orbDeg)}`,
    ),
  );
  const reflected = chart.antiscia;
  reflected.points.forEach((at) =>
    put(`chart-${i}-antiscion-${at.graha}`, `${number(at.antiscionDeg)} ${number(at.contrantiscionDeg)}`),
  );
  put(`chart-${i}-antiscia-unpaired`, reflected.unpaired.join(',') || '-');
  const putAntiscionRows = (key, rows) => {
    put(`${key}-count`, `${rows.length}`);
    rows.forEach((at, n) =>
      put(`${key}-${n}`, `${at.first} ${at.second} ${at.contrary ? 1 : 0} ${number(at.apartDeg)} ${number(at.orbDeg)}`),
    );
  };
  putAntiscionRows(`chart-${i}-antiscia`, reflected.pairs);
  put(`chart-${i}-antiscia-cusps`, `${reflected.cuspSystem} ${reflected.onCusps.length}`);
  reflected.onCusps.forEach((at, n) =>
    put(`chart-${i}-antiscia-cusp-${n}`, `${at.graha} ${at.house} ${at.contrary ? 1 : 0}`),
  );
  const houses = chart.westernHouses;
  put(
    `chart-${i}-western-houses`,
    `${houses.system} ${number(houses.ascendantDeg)} ${number(houses.reachDeg)} ${houses.planets.length}`,
  );
  houses.cuspsDeg.forEach((at, n) => put(`chart-${i}-western-cusp-${n + 1}`, number(at)));
  houses.planets.forEach((at) =>
    put(`chart-${i}-western-house-${at.graha}`, `${at.house} ${at.withAscendant ? 1 : 0}`),
  );
  const fifth = chart.harmonic;
  const pointKey = (one) => (one.point === 'GRAHA' ? one.graha : one.point);
  put(`chart-${i}-harmonic`, `${fifth.harmonic} ${fifth.points.length} ${fifth.rows.length}`);
  fifth.points.forEach((at) => put(`chart-${i}-harmonic-${pointKey(at.point)}`, `${number(at.longitudeDeg)} ${at.house}`));
  fifth.rows.forEach((at, n) =>
    put(
      `chart-${i}-harmonic-row-${n}`,
      `${pointKey(at.first)} ${pointKey(at.second)} ${number(at.apartDeg)} ${at.multiple} ${number(at.orbDeg)}`,
    ),
  );
  const between = chart.midpoints;
  put(`chart-${i}-midpoint-count`, `${between.length}`);
  between.forEach((at, n) =>
    put(
      `chart-${i}-midpoint-${n}`,
      `${at.first} ${at.second} ${at.middle} ${at.far ? 1 : 0} ${number(at.distanceDeg)} ${number(at.fromAxisDeg)} ${number(at.orbDeg)}`,
    ),
  );
  putAshta(`chart-${i}`, chart.matching);
  putPorutham(`chart-${i}`, chart.porutham);
  const mars = chart.kuja;
  for (const [who, side] of [['bride', mars.bride], ['groom', mars.groom]]) {
    const readings = side.readings.map((r) => `${r.from} ${r.house} ${flag(r.inHouses)}`);
    put(`chart-${i}-kuja-${who}`, `${readings.join(' ')} ${flag(side.dosha)}`);
  }
  put(`chart-${i}-kuja`, flag(mars.both));
  const doshas = chart.marriageDoshas;
  put(`chart-${i}-doshas`, `${doshas.length}`);
  doshas.forEach((d, n) =>
    put(`chart-${i}-dosha-${n}`, `${d.system} ${d.koota ?? 'NONE'} ${d.side ?? 'NONE'} ${flag(d.lifted)}`),
  );
  const point = (p) => (p.point === 'LAGNA' ? 'LAGNA' : p.graha);
  const synastry = chart.synastry;
  put(`chart-${i}-synastry-count`, `${synastry.length}`);
  synastry.forEach((at, n) =>
    put(
      `chart-${i}-synastry-${n}`,
      `${point(at.first)} ${point(at.second)} ${at.aspect} ${number(at.apartDeg)} ${number(at.fromExactDeg)} ${number(at.orbDeg)}`,
    ),
  );
  const levelled = chart.synastryParallels;
  put(`chart-${i}-synastry-parallel-count`, `${levelled.length}`);
  levelled.forEach((at, n) =>
    put(
      `chart-${i}-synastry-parallel-${n}`,
      `${point(at.first)} ${point(at.second)} ${at.contrary ? 1 : 0} ${number(at.apartDeg)} ${number(at.orbDeg)}`,
    ),
  );
  putAntiscionRows(`chart-${i}-synastry-antiscia`, chart.synastryAntiscia);
  const across = chart.synastryMidpoints;
  put(`chart-${i}-synastry-midpoint-count`, `${across.length}`);
  across.forEach((at, n) =>
    put(
      `chart-${i}-synastry-midpoint-${n}`,
      `${at.first} ${at.second} ${at.middle} ${at.partnersPair ? 1 : 0} ${at.far ? 1 : 0} ${number(at.distanceDeg)} ${number(at.fromAxisDeg)} ${number(at.orbDeg)}`,
    ),
  );
  const composite = chart.synastryComposite;
  put(
    `chart-${i}-composite`,
    `${number(composite.lagnaDeg)} ${number(composite.midheavenDeg)} ${composite.lagnaTurned ? 1 : 0} ${composite.planets.length}`,
  );
  composite.planets.forEach((at, n) =>
    put(`chart-${i}-composite-${n}`, `${at.graha} ${number(at.longitudeDeg)} ${number(at.speedDegPerDay)}`),
  );
  put(`chart-${i}-composite-cusps`, composite.cuspsDeg === null ? '-' : composite.cuspsDeg.map(number).join(' '));
  const davison = chart.synastryDavison;
  put(
    `chart-${i}-davison`,
    `${number(davison.instant)} ${number(davison.place.latitude)} ${number(davison.place.longitude)} ${number(davison.place.altitude)} ${davison.utcOffsetSeconds}`,
  );
  put(
    `chart-${i}-brahma`,
    `${b.rule} ${b.countedFrom} ${b.qualified.join(',') || '-'} ${b.graha ?? '-'} ${b.passedFrom ?? '-'} ${b.none ?? '-'}`,
  );
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
  // Uranus, Neptune and Pluto, which the request asks beside the nine.
  chart.outer.forEach((at, j) =>
    put(
      `chart-${i}-outer-${j}`,
      `${at.graha} ${number(at.longitudeDeg)} ${number(at.latitudeDeg)} ${number(at.speedDegPerDay)} ${at.house.bhava} ${at.placement.bhava}`,
    ),
  );
  chart.houses.forEach((bhava, k) => {
    put(`chart-${i}-house-${k}-madhya`, bhava.madhyaDeg);
    put(`chart-${i}-house-${k}-sandhi`, bhava.sandhiDeg);
  });
  chart.chalit.forEach((bhava, k) => {
    put(`chart-${i}-chalit-${k}-madhya`, bhava.madhyaDeg);
  });
}
// `found` is the batch of one unwrapped, and must agree with the batch.
// **The annual charts, under all three readings.** One crossing each,
// because `varsha_json` names one reading per request — and all three,
// because the point of naming a reading is that a consumer can ask for
// the one they mean, and a reading that crossed as another would be
// invisible in a report that only printed the default.
// Each reading also asks the sixteen yogas a different way, so all three
// ways cross: every matter under the source's readings, every matter under
// Tambira's "some authorities", and no matter at all.
// The sahams likewise: every one under the source's rules, every one
// under each rival rule, and none; and the annual dashas: every one under
// the sources' readings, every one under a rival clock, balance and birth
// period three levels deep, and none.
const MATTERS = {
  SIDEREAL: { matters: 'all', sahams: 'all', dashas: 'all' },
  TROPICAL: {
    matters: 'all',
    yogas: { tambira: 'EITHER_LORD' },
    sahams: 'all',
    sahamRules: { addSign: 'SIGNS', houses: 'EQUAL', roga: 'SATURN' },
    dashas: 'all',
    dashaRules: { clock: 'EVEN', balance: 'ENTRY_MOON', birthPeriod: 'ELAPSED', depth: 3 },
  },
  MEAN: {},
};
// One annual dasha as every runner prints it: its seed, ring and year on
// one line, and its periods on another.
const dashaSaid = (d) =>
  [
    `${d.seed ?? '-'} ${d.ring.first} ${d.ring.remaining === null ? 'null' : d.ring.remaining.toFixed(9)}`,
    `${d.year.from.toFixed(9)} ${d.year.to.toFixed(9)}`,
    '|',
    d.ring.shares.map((s) => `${s.lord}/${s.sign ?? '-'}/${s.weight.toFixed(3)}`).join(' '),
  ].join(' ');
const periodsSaid = (d) =>
  d.periods.map((p) => `${p.path}:${p.lord}:${p.sign ?? '-'}:${p.from.toFixed(9)}:${p.to.toFixed(9)}`).join(' ');
// One saham as every runner prints it: its place, its clauses, its lord's
// strengths and how the seven stand to it.
const sahamSaid = (p) =>
  [
    `${p.longitudeDeg.toFixed(6)} ${p.sign} ${p.lord} ${p.house} ${p.addedSign}`,
    `S:${p.strong.join(',')} W:${p.weak.join(',')}`,
    `${p.lordVishwa} ${p.lordHarsha} ${p.inNodeAxis}`,
    p.seven.map((s) => `${s.drishti}/${s.relation}/${+s.company}`).join(' '),
  ].join(' | ');
const pairSaid = (p) =>
  `${p.faster}>${p.slower}:${p.drishti}:${p.yoga ?? '-'}:${p.apartDeg.toFixed(6)}`;
const heldSaid = (h) =>
  [
    h.yoga,
    h.through ?? '-',
    h.entering ?? '-',
    h.between === null ? '-' : 'pair',
    h.legs === null ? '-' : h.legs.map(pairSaid).join('/'),
    h.afflictions === null
      ? '-'
      : [h.afflictions.lagnesha, h.afflictions.karyesha]
          .map((clauses) => (clauses.length === 0 ? 'none' : clauses.join('+')))
          .join('/'),
  ].join(':');
for (const reading of ['SIDEREAL', 'TROPICAL', 'MEAN']) {
  const years = geo.chart.foundMany({
    instants: [2460482.5, 2460600.25],
    place,
    utcOffsetSeconds: 20700,
    varsha: { reading, through: 12, place: 'birth', ...MATTERS[reading] },
  });
  let i = 0;
  for (const chart of years) {
    put(`chart-${i}-varsha-${reading}-count`, chart.praveshas.length);
    for (const p of chart.sahams) put(`chart-${i}-varsha-${reading}-natal-saham-${p.saham}`, sahamSaid(p));
    for (const one of chart.praveshas) {
      put(`chart-${i}-varsha-${reading}-${one.year}`, one.instant);
      put(`chart-${i}-varsha-${reading}-${one.year}-muntha`, one.muntha.sign);
      put(`chart-${i}-varsha-${reading}-${one.year}-muntha-lord`, one.muntha.lord);
      put(`chart-${i}-varsha-${reading}-${one.year}-muntha-deg`, one.muntha.longitudeDeg);
      put(`chart-${i}-varsha-${reading}-${one.year}-annual-lagna`, one.annual.lagnaDeg);
      put(`chart-${i}-varsha-${reading}-${one.year}-annual-by-day`, one.annual.byDay);
      const b = one.annual.officeBearers;
      put(
        `chart-${i}-varsha-${reading}-${one.year}-annual-bearers`,
        [b.muntha, b.janmaLagna, b.varshaLagna, b.triRashi, b.dinaRatri].join(' '),
      );
      const lord = one.annual.yearLord;
      put(`chart-${i}-varsha-${reading}-${one.year}-year-lord`, lord.graha);
      put(`chart-${i}-varsha-${reading}-${one.year}-year-lord-chosen`, lord.chosen);
      put(`chart-${i}-varsha-${reading}-${one.year}-year-lord-bala`, lord.vishwa.toString());
      put(
        `chart-${i}-varsha-${reading}-${one.year}-yogas`,
        one.annual.yogas
          .map((p) => `${p.faster}>${p.slower}:${p.drishti}:${p.yoga}:${p.apartDeg.toFixed(6)}`)
          .join(' '),
      );
      const at = `chart-${i}-varsha-${reading}-${one.year}`;
      put(`${at}-states`, `R:${one.annual.retrograde.join(',')} C:${one.annual.combust.join(',')}`);
      for (const m of one.annual.matters) {
        put(`${at}-matter-${m.house}`, `${m.sign} ${m.lagnesha}>${m.karyesha} ${m.sameLord}`);
        put(`${at}-matter-${m.house}-pair`, m.between === null ? '-' : pairSaid(m.between));
        put(`${at}-matter-${m.house}-unanswered`, m.unanswered.join(','));
        put(`${at}-matter-${m.house}-held`, m.held.map(heldSaid).join(' '));
      }
      for (const p of one.annual.sahams) put(`${at}-saham-${p.saham}`, sahamSaid(p));
      for (const d of one.annual.dashas) {
        put(`${at}-dasha-${d.system}`, dashaSaid(d));
        put(`${at}-dasha-${d.system}-periods`, periodsSaid(d));
      }
      put(
        `${at}-harsha`,
        one.annual.harsha
          .map((h) => `${h.graha}:${h.house}:${+h.sthana}${+h.uchchaSwakshetra}${+h.striPurusha}${+h.dinaRatri}:${h.total}:${h.grade}`)
          .join(' '),
      );
      put(
        `chart-${i}-varsha-${reading}-${one.year}-year-claims`,
        lord.claims
          .map((c) => `${c.graha}:${c.vishwa}:${c.portfolios}:${c.aspectsLagna}`)
          .join(' '),
      );
    }
    i += 1;
  }
}

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
put('almanac-provenance-fnv', fnv(week.provenanceJson));

for (const day of week) {
  const i = day.index;
  put(`day-${i}-content-hash`, day.provenance.contentHash);
  putDay(`day-${i}`, day.day);
  put(`day-${i}-window-from`, day.window.from);
  put(`day-${i}-window-to`, day.window.to);
  put(`day-${i}-month`, day.month.month);
  put(`day-${i}-amanta`, day.month.amanta);
  put(`day-${i}-purnimanta`, day.month.purnimanta);
  put(`day-${i}-paksha`, day.month.paksha);
  put(`day-${i}-convention`, day.month.convention);
  put(`day-${i}-month-kind`, day.month.kind);
  put(`day-${i}-ayana`, day.ayana);
  put(`day-${i}-ritu`, day.ritu);
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
      put(`day-${i}-${limb}-${j}-sunrises`, span.sunrises);
      put(`day-${i}-${limb}-${j}-ends`, `${span.ends.ghati}-${span.ends.pala}-${span.ends.vipala}`);
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

// ── A muhurta search ───────────────────────────────────────────────────
// Both rankings over 2024-11-25..27: the texts bar the windows for
// different reasons and the baseline scores them; and a thread ceremony,
// whose rules want grahas out of houses.
const listed = (items) => (items.length === 0 ? 'none' : items.join(' '));
for (const [name, rules, ranking] of [
  ['raman', 'RAMAN_MARRIAGE', 'TEXTS'],
  ['baseline', 'BASELINE_MARRIAGE', 'BASELINE'],
  ['upanayana', 'RAMAN_UPANAYANA', 'TEXTS'],
]) {
  const { muhurta } = geo.almanac.of({
    from: gregorian(2024, 11, 25),
    to: gregorian(2024, 11, 27),
    place,
    utcOffsetSeconds: 20700,
    muhurta: {
      rules,
      ranking,
      native: { star: 'ROHINI', moonSign: 'TAURUS', lagna: 'LEO' },
      daysWithWindows: 3,
      most: 12,
    },
  });
  const key = (what) => `muhurta-${name}${what}`;
  put(
    key('-counts'),
    [
      muhurta.windows.length,
      muhurta.closed.length,
      muhurta.daysJudged,
      muhurta.daysCut,
      muhurta.windowsBlackedOut,
      muhurta.ranking,
    ].join(' '),
  );
  put(key('-hash'), muhurta.provenance.contentHash);
  muhurta.windows.forEach((window, k) => {
    put(key(`-${k}`), `${number(window.at.from)} ${number(window.at.to)}`);
    put(key(`-${k}-clauses`), window.clauses.map((clause) => clause.clause).join(' '));
    put(
      key(`-${k}-bars`),
      listed(window.barredBy.map((bar) => (typeof bar === 'string' ? bar : bar.clause))),
    );
    put(
      key(`-${k}-placed`),
      listed(
        window.clauses
          .filter((clause) => clause.clause === 'UNWANTED_PLACEMENT')
          .map((clause) => `${clause.house}:${clause.by.join(',')}`),
      ),
    );
    put(
      key(`-${k}-score`),
      window.score === null
        ? 'none'
        : [
            window.score.value,
            window.score.cappedAt ?? 'none',
            listed(
              window.score.factors.map((f) => `${f.dimension}:${f.weight}:${f.graha ?? 'none'}`),
            ),
          ].join(' '),
    );
  });
  muhurta.closed.forEach((day, j) => {
    put(key(`-closed-${j}`), `${day.date.month}-${day.date.day} ${listed(day.by)}`);
  });
}

// ── Festivals ──────────────────────────────────────────────────────────
// The shipped pack, the pack amended by a rule of the consumer's own
// (Lakshmi puja on whichever day holds the new moon at sunrise), and the
// Nepal pack with a following rule of the consumer's own (two days after
// Lakshmi puja), over 2024-10-10..11-03.
const ownRule = {
  key: 'LAKSHMI_PUJA',
  source: 'the tithi at sunrise',
  month: 'masa.ASHWINA',
  tithi: 'tithi.AMAVASYA',
  at: { window: 'SUNRISE' },
  decide: [],
  otherwise: 'LATER',
};
const ownFollowing = {
  key: 'TWO_AFTER',
  source: 'two days after Lakshmi puja',
  after: 'LAKSHMI_PUJA',
  days: 2,
};
for (const [name, rules] of [
  ['shipped', 'DHARMASINDHU'],
  ['amended', ['DHARMASINDHU', ownRule]],
  ['nepal', ['NEPAL', ownFollowing]],
]) {
  const { festivals } = geo.almanac.of({
    from: gregorian(2024, 10, 10),
    to: gregorian(2024, 11, 3),
    place,
    utcOffsetSeconds: 20700,
    festivals: { rules },
  });
  const key = (what) => `festivals-${name}${what}`;
  put(key('-counts'), `${festivals.observances.length} ${festivals.unjudged.length}`);
  put(key('-hash'), festivals.provenance.contentHash);
  festivals.observances.forEach((observance, k) => {
    const decided = observance.decidedBy;
    const by =
      decided.by === 'GUARD'
        ? `guard:${decided.index}`
        : decided.by === 'AFTER'
          ? `after:${decided.rule}:${decided.days}`
          : 'otherwise';
    const [earlier, later] = observance.extents;
    put(
      key(`-${k}`),
      [
        observance.rule,
        observance.month,
        observance.adhika,
        `${observance.day.month}-${observance.day.day}`,
        observance.case,
        by,
        observance.choice,
        number(observance.tithi.from),
        number(earlier.held),
        number(later.held),
      ].join(' '),
    );
  });
  festivals.ekadashis.forEach((fast, k) => {
    put(
      key(`-ekadashi-${k}`),
      [
        fast.rule,
        fast.tithi,
        fast.month,
        fast.adhika,
        `${fast.day.month}-${fast.day.day}`,
        fast.piercedAt ?? '-',
        fast.pierced,
        fast.excess,
        fast.choice,
        number(fast.tithis[1].from),
      ].join(' '),
    );
  });
}

// ── The lunar years ────────────────────────────────────────────────────
// 2024-03-20..04-20 holds a Chaitra Shukla Pratipada: two years, their
// bounds and their Jovian years.
{
  const { years } = geo.almanac.of({
    from: gregorian(2024, 3, 20),
    to: gregorian(2024, 4, 20),
    place,
    utcOffsetSeconds: 20700,
    years: true,
  });
  put('years-count', years.value.length);
  put('years-hash', years.provenance.contentHash);
  years.value.forEach((year, k) => {
    put(
      `years-${k}`,
      [
        year.samvatsara,
        year.count,
        year.vikrama,
        year.shaka,
        number(year.opened),
        number(year.began),
        number(year.ended),
        year.lupta ?? '-',
      ].join(' '),
    );
    put(
      `years-${k}-jovian`,
      listed(year.jovian.map((jovian) => `${jovian.member}:${jovian.count}:${number(jovian.from)}`)),
    );
  });
}

// ── The Nepal Sambat dates ─────────────────────────────────────────────
// 2024-10-30..11-03 holds Kartika's new moon, where the year turns.
{
  const { nepalSambat } = geo.almanac.of({
    from: gregorian(2024, 10, 30),
    to: gregorian(2024, 11, 3),
    place,
    utcOffsetSeconds: 20700,
    nepalSambat: true,
  });
  put('nepal-sambat-hash', nepalSambat.provenance.contentHash);
  put('nepal-sambat', listed(nepalSambat.value.map((d) => `${d.year}:${d.month}:${d.kind}:${d.paksha}`)));
}
geo.dispose();

// ── The eclipses ───────────────────────────────────────────────────────
// September 2025 at Kathmandu over the built-in sky, which the test
// provider cannot complete: a total lunar eclipse seen whole and a
// partial solar one the place does not see (`03-design/eclipses.md`).
{
  const sky = new Context({ profile: 'nepali-default', ephemeris: 'BUILTIN' });
  const { eclipses } = sky.almanac.of({
    from: gregorian(2025, 9, 1),
    to: gregorian(2025, 9, 30),
    place,
    utcOffsetSeconds: 20700,
    eclipses: true,
  });
  sky.dispose();
  const maybe = (value) => (value === null ? '-' : number(value));
  const moment = (m) => (m === null ? '-' : `${number(m.at)}@${number(m.altitudeDeg)}`);
  const seen = (s) => (s === null ? '-' : `${number(s.from)}..${number(s.to)}`);
  put('eclipses-hash', eclipses.provenance.contentHash);
  put('eclipses-count', `${eclipses.value.lunar.length} ${eclipses.value.solar.length}`);
  eclipses.value.lunar.forEach(({ eclipse, here }, k) => {
    const c = eclipse.contacts;
    put(
      `eclipses-lunar-${k}`,
      [eclipse.kind, eclipse.shadow, number(eclipse.greatest), number(eclipse.gamma), number(eclipse.umbralMagnitude), number(eclipse.penumbralMagnitude)].join(' '),
    );
    put(`eclipses-lunar-${k}-contacts`, [c.p1, c.u1, c.u2, c.u3, c.u4, c.p4].map(maybe).join(' '));
    put(
      `eclipses-lunar-${k}-here`,
      [here.p1, here.u1, here.u2, here.greatest, here.u3, here.u4, here.p4].map(moment).concat(seen(here.seen), seen(here.umbralSeen)).join(' '),
    );
  });
  eclipses.value.solar.forEach(({ eclipse, here }, k) => {
    put(
      `eclipses-solar-${k}`,
      [eclipse.kind, number(eclipse.greatest), number(eclipse.gamma), number(eclipse.magnitude), number(eclipse.point.latitude), number(eclipse.point.longitude)].join(' '),
    );
    put(
      `eclipses-solar-${k}-here`,
      here === null
        ? '-'
        : [here.kind, number(here.magnitude), number(here.obscuration)]
            .concat([here.first, here.second, here.third, here.fourth, here.maximum].map(moment), seen(here.seen))
            .join(' '),
    );
  });
}

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
  ['matching.naam', shape.matching.naam],
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
