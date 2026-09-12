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
const geo = new Context({ profile: 'parashari-classical', locale: 'ne-Deva-NP', testProvider: true });
put('geo-profile', geo.profile);
put('geo-settings-hash', geo.settingsHash);

// ── Charts ─────────────────────────────────────────────────────────────
// Two instants, so a per-chart section that ran charts-outermost the
// wrong way round shows up as the second chart's values in the first's
// place rather than as nothing at all.
const charts = geo.chart.foundMany({
  instants: [2460482.5, 2460600.25],
  place,
  utcOffsetSeconds: 20700,
});
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

for (const key of [...report.keys()].sort()) {
  process.stdout.write(`${key}\t${report.get(key)}\n`);
}
