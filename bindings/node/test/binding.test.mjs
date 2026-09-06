// The Node binding end to end: the same scenario the C binding's smoke
// test walks, through the addon and the ergonomic layer.
//
// `cargo xtask check-node` builds the addon and runs this file. Node's own
// test runner and assertions only, so the binding's tests need no install.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  Body,
  Calendar,
  Context,
  Era,
  Resolution,
  Scale,
  TeistroError,
  ZoneKind,
  abiVersion,
  catalogueVersion,
  defaultProfile,
  fixedOfJulianDay,
  julianDayOfFixed,
  packFrame,
  sdkVersion,
  unpackFrame,
} from '../lib/index.js';

/** A context with the analytic test provider; every test builds its own. */
function context(options = {}) {
  return new Context({ testProvider: true, profile: 'nepali-default', locale: 'ne-Deva-NP', ...options });
}

/** A Gregorian date as the boundary takes one. */
function gregorian(year, month, day) {
  return {
    calendar: Calendar.Gregorian,
    year,
    eraYear: 0,
    month,
    day,
    resolution: Resolution.Defined,
    computedMonth: 0,
    computedDay: 0,
  };
}

test('the addon and the types were generated for the same ABI', () => {
  assert.equal(abiVersion(), 1);
  assert.equal(catalogueVersion(), 1);
  assert.equal(defaultProfile(), 'parashari-classical');
  assert.match(sdkVersion(), /^\d+\.\d+\.\d+$/u);
});

test('a context resolves its settings and reports them', () => {
  const ctx = context();
  assert.equal(ctx.profile, 'nepali-default');
  assert.equal(ctx.locale, 'ne-Deva-NP');
  assert.match(ctx.settingsHash, /^[0-9a-f]{64}$/u);
  assert.equal(ctx.settings.frame.zodiac, 'SIDEREAL');
  assert.equal(ctx.settings.schema, 1);

  // A patch over the profile changes the settings and therefore the hash.
  const patched = context({ settings: { frame: { zodiac: 'TROPICAL' } } });
  assert.equal(patched.settings.frame.zodiac, 'TROPICAL');
  assert.notEqual(patched.settingsHash, ctx.settingsHash);

  // The default profile is the one `defaultProfile()` names.
  assert.equal(new Context({ testProvider: true }).profile, defaultProfile());
});

test('a refusal carries its status, its field and its hint', () => {
  const ctx = context();
  assert.throws(
    () => ctx.keyId('graha.SUNN'),
    (error) => {
      assert.ok(error instanceof TeistroError, 'a TeistroError, not a bare Error');
      assert.equal(error.status, 'unsupported');
      assert.equal(error.code, -6);
      assert.equal(error.detail, 'UNKNOWN_KEY');
      assert.match(error.hint, /did you mean `SUN`/u);
      assert.match(String(error), /TeistroError \[unsupported\]/u);
      return true;
    },
  );
  assert.throws(
    () => new Context({ profile: 'vedic-classic' }),
    /no shipped profile `vedic-classic`/u,
  );
  assert.throws(() => context({ locale: 'xx-Latn' }), /ne-Deva-NP/u);
  assert.throws(
    () => ctx.fixedOf(gregorian(2023, 2, 29)),
    (error) => error.detail === 'NONEXISTENT_DATE' && error.status === 'invalid-arg',
  );
});

test('a date converts into Bikram Sambat with its era and its resolution', () => {
  const ctx = context();
  const date = gregorian(2015, 4, 14);
  const bs = ctx.convert(date, Calendar.BikramSambat);
  assert.equal(bs.year, 2072);
  assert.equal(bs.month, 1);
  assert.equal(bs.day, 1);
  assert.equal(bs.era, Era.Vikrama);
  assert.equal(bs.eraYear, 2072);
  assert.equal(bs.resolution, Resolution.Tabular, 'inside the official table');

  const fixed = ctx.fixedOf(date);
  assert.equal(fixed, 735702);
  assert.equal(ctx.weekdayOf(date), 2, 'a Tuesday');
  assert.equal(ctx.dateOf(Calendar.Gregorian, fixed).era, Era.CommonEra);
  assert.equal(ctx.monthLength(Calendar.Gregorian, 2024, 2), 29);
  assert.equal(ctx.isLeap(Calendar.Gregorian, 2024), true);
  assert.equal(julianDayOfFixed(fixed), 2457126.5);
  assert.deepEqual(fixedOfJulianDay(2457126.75), { value: fixed, fraction: 0.25 });
});

test('a Nepali birth time resolves with the metadata a stored chart keeps', () => {
  const ctx = context();
  const civil = {
    date: gregorian(1986, 1, 1),
    time: { hour: 0, minute: 20, second: 0, hasTime: true, nanos: 0 },
  };
  const zone = { kind: ZoneKind.Iana, offsetSeconds: 0, longitudeDeg: 0, zone: 'Asia/Kathmandu' };
  const resolved = ctx.resolve(civil, zone);
  assert.ok(Math.abs(resolved.instantJdUtc - 2446431.2743056) < 1e-6);
  assert.equal(resolved.offsetSeconds, 20700, '+05:45, the offset that began that midnight');
  assert.equal(resolved.era, 'current');
  assert.equal(resolved.source, 'iana');
  assert.equal(resolved.timeKnown, true);
  assert.deepEqual(resolved.warnings, []);
  assert.match(resolved.tzdbVersion, /^20\d\d[a-z]$/u);

  const back = ctx.civilOf(resolved.instantJdUtc, zone, Calendar.Gregorian);
  assert.equal(back.civil.date.year, 1986);
  assert.equal(back.civil.time.minute, 20);
  assert.equal(back.civil.time.hasTime, true);
  assert.equal(back.resolution.offsetSeconds, 20700);

  const unknown = { kind: ZoneKind.Iana, offsetSeconds: 0, longitudeDeg: 0, zone: 'Asia/Kathmandou' };
  assert.throws(() => ctx.resolve(civil, unknown), (error) => error instanceof TeistroError);
});

test('the time scales convert with what they applied', () => {
  const ctx = context();
  const tt = ctx.convertTime(2451544.5, Scale.Utc, Scale.Tt);
  assert.ok(Math.abs(tt.deltaTSeconds - 64.184) < 1e-9, 'exact through the leap-second table');
  assert.equal(tt.deltaTSource, 'leap-seconds');
  assert.equal(tt.deltaTModel, 'TABLE_THEN_MODEL');
  assert.ok(Math.abs(tt.jd - (2451544.5 + 64.184 / 86400)) < 1e-12);
  const back = ctx.convertTime(tt.jd, Scale.Tt, Scale.Utc);
  assert.ok(Math.abs(back.jd - 2451544.5) < 1e-9);

  const delta = ctx.deltaT(2451544.5);
  assert.ok(Math.abs(delta.seconds - 63.83) < 0.02);
  assert.equal(delta.source, 'table');
  assert.throws(() => ctx.deltaT(Number.NaN), /expected a finite number/u);
});

test('positions come back in the frame asked for, decoded on first use', () => {
  const ctx = context();
  const frame = ctx.canonicalFrame();
  assert.equal(frame.centre, 'geocentric');
  assert.equal(frame.coordinates, 'ecliptic');
  assert.equal(frame.sidereal, false);
  assert.equal(frame.ayanamsha, undefined, 'a tropical frame carries none');
  assert.deepEqual(unpackFrame(packFrame(frame)), frame, 'the packing round-trips');

  const positions = ctx.positions({
    instants: [2451545.0, 2451546.0],
    bodies: [Body.Sun, Body.Moon, Body.Mars],
  });
  assert.deepEqual(positions.bodies, ['sun', 'moon', 'mars']);
  assert.deepEqual(Array.from(positions.instants), [2451545, 2451546]);
  assert.equal(positions.scale, 'ut1');
  assert.equal(positions.cells.length, 6, 'two instants by three bodies');

  const sun = positions.at(0, 0);
  assert.ok(sun.longitude >= 0 && sun.longitude < 360);
  assert.equal(sun.status, 0);
  assert.ok(
    Math.abs(positions.at(0, 1).longitudeSpeed) > Math.abs(sun.longitudeSpeed),
    'the Moon moves faster than the Sun',
  );
  assert.throws(() => positions.at(2, 0), RangeError);

  assert.ok(positions.steps.every((step) => typeof step.name === 'string'));
  assert.equal(positions.provenance.profile, 'nepali-default');
  assert.equal(positions.provenance.calculation_version, 1);
  assert.equal(positions.provenance.settings_hash, ctx.settingsHash);
  assert.equal(
    positions.provenance.provider.frame,
    'GEOCENTRIC/OF_DATE/ECLIPTIC/TROPICAL/APPARENT',
  );

  // The same request twice gives the same bytes: the result is a fact.
  const again = ctx.positions({
    instants: [2451545.0, 2451546.0],
    bodies: [Body.Sun, Body.Moon, Body.Mars],
  });
  assert.deepEqual(Buffer.from(again.bytes), Buffer.from(positions.bytes));

  // Without an ephemeris the call is a missing capability, named.
  const bare = new Context({});
  assert.throws(
    () => bare.positions({ instants: [2451545.0], bodies: [Body.Sun] }),
    (error) => error.status === 'capability' && error.field === 'provider',
  );
  assert.throws(() => ctx.positions({ instants: [], bodies: [Body.Sun] }), TypeError);
  assert.throws(() => ctx.positions({ instants: [Number.NaN], bodies: [Body.Sun] }), TypeError);
  assert.throws(() => ctx.positions({ instants: [2451545.0], bodies: ['graha.PLOOTO'] }), Error);
});

test('the locale engine renders typed parameters, and says where from', () => {
  const ctx = context();
  const rendered = ctx.render('sdk.reason.grahaInBhava', {
    graha: { $entity: 'graha.JUPITER' },
    bhava: 7,
  });
  assert.equal(rendered.resolvedFrom, 'ne-Deva-NP');
  assert.equal(rendered.isFallback, false);
  assert.equal(rendered.isOverride, false);
  assert.deepEqual(rendered.warnings, []);
  assert.match(rendered.text, /७/u, 'the Nepali numeral seven');
  assert.equal(String(rendered), rendered.text);
  assert.equal(ctx.has('sdk.reason.grahaInBhava'), true);
  assert.equal(ctx.has('sdk.nope.missing'), false);

  // A missing message renders as its key with a warning, never an error.
  const missing = ctx.render('sdk.nope.missing');
  assert.equal(missing.resolvedFrom, null);
  assert.ok(missing.warnings.length > 0);

  ctx.locale = 'en-Latn';
  assert.equal(ctx.locale, 'en-Latn');
  const english = ctx.render('sdk.reason.grahaInBhava', {
    graha: { $entity: 'graha.JUPITER' },
    bhava: 7,
  });
  assert.match(english.text, /Jupiter/u);
  assert.throws(() => {
    ctx.locale = 'fr-Latn';
  }, /sa-Deva/u);
  assert.throws(
    () => ctx.render('sdk.reason.grahaInBhava', 'not an object'),
    (error) => error.status === 'invalid-arg' && error.field === 'params_json',
  );
});

test('a catalogue key packs to an id and back', () => {
  const ctx = context();
  const id = ctx.keyId('graha.SUN');
  assert.equal(id, (1 << 16) | 0, 'the kind in the high half, the member in the low');
  assert.equal(ctx.keyName(id), 'graha.SUN');
  assert.equal(ctx.keyName(ctx.keyId('nakshatra.ASHWINI')), 'nakshatra.ASHWINI');
  assert.throws(() => ctx.keyName(0xffffffff), (error) => error.status === 'unsupported');
});
