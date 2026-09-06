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
put('locale', ctx.locale);
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
const bs = ctx.convert(date, Calendar.BikramSambat);
put('bs-year', bs.year);
put('bs-month', bs.month);
put('bs-day', bs.day);
put('bs-era', bs.era);
put('bs-era-year', bs.eraYear);
put('bs-resolution', bs.resolution);
const fixed = ctx.fixedOf(date);
put('fixed', fixed);
put('weekday', ctx.weekdayOf(date));
put('month-length', ctx.monthLength(Calendar.Gregorian, 2024, 2));
put('is-leap', ctx.isLeap(Calendar.Gregorian, 2024));
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
const resolved = ctx.resolve(civil, zone);
put('resolve-jd', resolved.instantJdUtc);
put('resolve-offset', resolved.offsetSeconds);
put('resolve-era', resolved.era);
put('resolve-source', resolved.source);
put('resolve-time-known', resolved.timeKnown);
put('resolve-tzdb', resolved.tzdbVersion);
put('resolve-warnings', resolved.warnings.length);
const civilBack = ctx.civilOf(resolved.instantJdUtc, zone, Calendar.Gregorian);
put('civil-year', civilBack.civil.date.year);
put('civil-minute', civilBack.civil.time.minute);
put('civil-offset', civilBack.resolution.offsetSeconds);
const tt = ctx.convertTime(2451544.5, Scale.Utc, Scale.Tt);
put('tt-jd', tt.jd);
put('tt-delta-t', tt.deltaTSeconds);
put('tt-delta-t-source', tt.deltaTSource);
put('tt-delta-t-model', tt.deltaTModel);
const delta = ctx.deltaT(2451544.5);
put('delta-t-seconds', delta.seconds);
put('delta-t-source', delta.source);

// ── Keys ───────────────────────────────────────────────────────────────
const id = ctx.keyId('graha.SUN');
put('key-id', id);
put('key-name', ctx.keyName(id));
try {
  ctx.keyId('graha.SUNN');
  put('refusal', 'none');
} catch (error) {
  put('refusal-status', error.status);
  put('refusal-detail', error.detail);
  put('refusal-hint-names-sun', error.hint.includes('SUN'));
}

// ── The locale engine ──────────────────────────────────────────────────
const rendered = ctx.render('sdk.reason.grahaInBhava', {
  graha: { $entity: 'graha.JUPITER' },
  bhava: 7,
});
put('render-fnv', fnv(rendered.text));
put('render-length', [...rendered.text].length);
put('render-resolved-from', rendered.resolvedFrom);
put('render-fallback', rendered.isFallback);
put('has-message', ctx.has('sdk.reason.grahaInBhava'));
put('has-missing-message', ctx.has('sdk.nope.missing'));

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

for (const key of [...report.keys()].sort()) {
  process.stdout.write(`${key}\t${report.get(key)}\n`);
}
