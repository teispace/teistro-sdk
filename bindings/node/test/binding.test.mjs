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
  altitude,
  at,
  date,
  ianaZone,
  latitude,
  longitude,
  whenUnknown,
  Context,
  Era,
  Resolution,
  Scale,
  TeistroError,
  ZoneKind,
  abiVersion,
  canonicalFrame,
  catalogueVersion,
  defaultProfile,
  fixedOfJulianDay,
  julianDayOfFixed,
  packFrame,
  sdkVersion,
  unpackFrame,
} from '../lib/index.js';
import * as catalogue from '../lib/catalogue.js';

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
  assert.equal(ctx.intl.locale, 'ne-Deva-NP');
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
    () => ctx.keys.id('graha.SUNN'),
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
    () => ctx.calendar.fixedOf(gregorian(2023, 2, 29)),
    (error) => error.detail === 'NONEXISTENT_DATE' && error.status === 'invalid-arg',
  );
});

test('a date converts into Bikram Sambat with its era and its resolution', () => {
  const ctx = context();
  const date = gregorian(2015, 4, 14);
  const bs = ctx.calendar.convert(date, Calendar.BikramSambat);
  assert.equal(bs.year, 2072);
  assert.equal(bs.month, 1);
  assert.equal(bs.day, 1);
  assert.equal(bs.era, Era.Vikrama);
  assert.equal(bs.eraYear, 2072);
  assert.equal(bs.resolution, Resolution.Tabular, 'inside the official table');

  const fixed = ctx.calendar.fixedOf(date);
  assert.equal(fixed, 735702);
  assert.equal(ctx.calendar.weekdayOf(date), 2, 'a Tuesday');
  assert.equal(ctx.calendar.dateOf(Calendar.Gregorian, fixed).era, Era.CommonEra);
  assert.equal(ctx.calendar.monthLength(Calendar.Gregorian, 2024, 2), 29);
  assert.equal(ctx.calendar.isLeap(Calendar.Gregorian, 2024), true);
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
  const resolved = ctx.time.resolve(civil, zone);
  assert.ok(Math.abs(resolved.instantJdUtc - 2446431.2743056) < 1e-6);
  assert.equal(resolved.offsetSeconds, 20700, '+05:45, the offset that began that midnight');
  assert.equal(resolved.era, 'current');
  assert.equal(resolved.source, 'iana');
  assert.equal(resolved.timeKnown, true);
  assert.deepEqual(resolved.warnings, []);
  assert.match(resolved.tzdbVersion, /^20\d\d[a-z]$/u);

  const back = ctx.time.civilOf(resolved.instantJdUtc, zone, Calendar.Gregorian);
  assert.equal(back.civil.date.year, 1986);
  assert.equal(back.civil.time.minute, 20);
  assert.equal(back.civil.time.hasTime, true);
  assert.equal(back.resolution.offsetSeconds, 20700);

  const unknown = { kind: ZoneKind.Iana, offsetSeconds: 0, longitudeDeg: 0, zone: 'Asia/Kathmandou' };
  assert.throws(() => ctx.time.resolve(civil, unknown), (error) => error instanceof TeistroError);
});

test('the time scales convert with what they applied', () => {
  const ctx = context();
  const tt = ctx.time.convert(2451544.5, Scale.Utc, Scale.Tt);
  assert.ok(Math.abs(tt.deltaTSeconds - 64.184) < 1e-9, 'exact through the leap-second table');
  assert.equal(tt.deltaTSource, 'leap-seconds');
  assert.equal(tt.deltaTModel, 'TABLE_THEN_MODEL');
  assert.ok(Math.abs(tt.jd - (2451544.5 + 64.184 / 86400)) < 1e-12);
  const back = ctx.time.convert(tt.jd, Scale.Tt, Scale.Utc);
  assert.ok(Math.abs(back.jd - 2451544.5) < 1e-9);

  const delta = ctx.time.deltaT(2451544.5);
  assert.ok(Math.abs(delta.seconds - 63.83) < 0.02);
  assert.equal(delta.source, 'table');
  assert.throws(() => ctx.time.deltaT(Number.NaN), /expected a finite number/u);
});

test('positions come back in the frame asked for, decoded on first use', () => {
  const ctx = context();
  const frame = ctx.frame.canonical();
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
  const rendered = ctx.intl.render('sdk.reason.grahaInBhava', {
    graha: { $entity: 'graha.JUPITER' },
    bhava: 7,
  });
  assert.equal(rendered.resolvedFrom, 'ne-Deva-NP');
  assert.equal(rendered.isFallback, false);
  assert.equal(rendered.isOverride, false);
  assert.deepEqual(rendered.warnings, []);
  assert.match(rendered.text, /७/u, 'the Nepali numeral seven');
  assert.equal(String(rendered), rendered.text);
  assert.equal(ctx.intl.has('sdk.reason.grahaInBhava'), true);
  assert.equal(ctx.intl.has('sdk.nope.missing'), false);

  // A missing message renders as its key with a warning, never an error.
  const missing = ctx.intl.render('sdk.nope.missing');
  assert.equal(missing.resolvedFrom, null);
  assert.ok(missing.warnings.length > 0);

  ctx.intl.locale = 'en-Latn';
  assert.equal(ctx.intl.locale, 'en-Latn');
  const english = ctx.intl.render('sdk.reason.grahaInBhava', {
    graha: { $entity: 'graha.JUPITER' },
    bhava: 7,
  });
  assert.match(english.text, /Jupiter/u);
  assert.throws(() => {
    ctx.intl.locale = 'fr-Latn';
  }, /sa-Deva/u);
  assert.throws(
    () => ctx.intl.render('sdk.reason.grahaInBhava', 'not an object'),
    (error) => error.status === 'invalid-arg' && error.field === 'params_json',
  );
});

test('a quantity is its own type, and its constructor checks the range', () => {
  const ctx = context();
  assert.equal(latitude(27.7172), 27.7172, 'a branded number is a number at run time');
  assert.throws(() => latitude(91), RangeError);
  assert.throws(() => longitude(-181), RangeError);
  assert.throws(() => altitude(20000), RangeError);
  assert.throws(() => latitude(Number.NaN), TypeError);
  assert.throws(() => longitude('85.3'), TypeError);

  // The place reaches the boundary and comes back through the frame.
  const positions = ctx.positions({
    instants: [2451545.0],
    bodies: [Body.Sun],
    observer: {
      latitudeDeg: latitude(27.7172),
      longitudeDeg: longitude(85.324),
      altitudeM: altitude(1400),
    },
  });
  assert.equal(positions.cells.length, 1);
});

test('a catalogue key packs to an id and back', () => {
  const ctx = context();
  const id = ctx.keys.id('graha.SUN');
  assert.equal(id, (1 << 16) | 0, 'the kind in the high half, the member in the low');
  assert.equal(ctx.keys.name(id), 'graha.SUN');
  assert.equal(ctx.keys.name(ctx.keys.id('nakshatra.ASHWINI')), 'nakshatra.ASHWINI');
  assert.throws(() => ctx.keys.name(0xffffffff), (error) => error.status === 'unsupported');
});

/** An ephemeris written in JavaScript: a straight line per body. */
function jsProvider(over = {}) {
  const calls = [];
  return {
    calls,
    provider: {
      name: 'a-provider-in-javascript',
      bodies: [Body.Sun, Body.Moon],
      version: '1.2.3',
      positions(request) {
        calls.push(request);
        const cells = request.jds.length * request.bodies.length;
        const lon = new Float64Array(cells);
        const lonSpeed = new Float64Array(cells);
        for (let i = 0; i < request.jds.length; i += 1) {
          for (let j = 0; j < request.bodies.length; j += 1) {
            const k = i * request.bodies.length + j;
            lon[k] = (request.jds[i] + j * 100) % 360;
            lonSpeed[k] = j === 0 ? 0.9856 : 13.176;
          }
        }
        return { frameBits: request.frameBits, lon, lonSpeed, status: new Int32Array(cells) };
      },
      ...over,
    },
  };
}

test('an ephemeris written in JavaScript answers the SDK', () => {
  const { calls, provider } = jsProvider();
  const ctx = new Context({ provider, profile: 'nepali-default' });
  const positions = ctx.positions({
    instants: [2451545.0, 2451546.0],
    bodies: [Body.Sun, Body.Moon],
  });

  // One call for the whole grid: the port is a batch call, not a loop.
  assert.equal(calls.length, 1);
  const asked = calls[0];
  assert.deepEqual(Array.from(asked.jds), [2451545, 2451546]);
  assert.deepEqual(asked.bodies, ['sun', 'moon']);
  assert.equal(asked.scale, 'ut1');
  assert.equal(asked.speeds, true);
  assert.equal(asked.observer, undefined, 'a geocentric frame needs none');

  assert.equal(positions.cells.length, 4);
  assert.ok(Math.abs(positions.at(0, 0).longitude - ((2451545 + 0) % 360)) < 1e-9);
  assert.equal(positions.at(0, 1).longitudeSpeed, 13.176);
  assert.equal(positions.provenance.provider.name, 'a-provider-in-javascript');
  assert.equal(positions.provenance.provider.version, '1.2.3');
});

test('a provider that refuses a frame is completed by the SDK', () => {
  const equatorial = { ...canonicalFrame(), coordinates: 'equatorial' };
  const asked = [];
  const provider = {
    name: 'equatorial-provider',
    bodies: [Body.Sun],
    nativeFrame: equatorial,
    positions(request) {
      asked.push(request.frameBits);
      // Nothing means "not in that frame"; the SDK asks again in ours.
      if (request.frameBits !== packFrame(equatorial)) return null;
      const cells = request.jds.length * request.bodies.length;
      return {
        frameBits: request.frameBits,
        lon: new Float64Array(cells).fill(280),
        lat: new Float64Array(cells),
        status: new Int32Array(cells),
      };
    },
  };
  const positions = new Context({ provider }).positions({
    instants: [2451545.0],
    bodies: [Body.Sun],
    frame: canonicalFrame(),
  });
  assert.equal(asked.length, 2, "the canonical frame, then the provider's own");
  const steps = positions.steps.map((step) => `${step.name}:${step.implementation}`);
  assert.deepEqual(steps, [
    'positions:NATIVE',
    'delta-t:SDK',
    'obliquity:SDK',
    'rotate-equatorial-to-ecliptic:SDK',
  ]);
  assert.ok(Math.abs(positions.at(0, 0).longitude - 280.8787) < 1e-3, 'rotated onto the ecliptic');
});

test('a provider that fails says so in its own words', () => {
  const throwing = {
    name: 'throws',
    bodies: [Body.Sun],
    positions() {
      throw new Error('no data for that instant');
    },
  };
  // The provider's **own** error reaches the caller, not a summary of it:
  // only a code crosses the C boundary, so the layer keeps the object it
  // threw and puts it back. The boundary's refusal is kept as its cause,
  // so nothing the library said is lost. The Dart and Python bindings
  // surface a provider's failure the same way.
  assert.throws(
    () => new Context({ provider: throwing }).positions({ instants: [2451545.0], bodies: [Body.Sun] }),
    (error) => {
      assert.equal(error.message, 'no data for that instant');
      assert.equal(error.status, undefined, 'the provider threw an Error, not a TeistroError');
      assert.equal(error.cause?.status, 'provider', "the library's own refusal, kept as the cause");
      return true;
    },
  );

  const short = { name: 'short', bodies: [Body.Sun], positions: () => ({ lon: [1], status: [0] }) };
  assert.throws(
    () => new Context({ provider: short }).positions({ instants: [2451545.0, 2451546.0], bodies: [Body.Sun] }),
    /returned 1 values in `lon` for 2 cells/u,
  );

  // A body the provider never declared is refused by the port, on the
  // SDK's side of the boundary and before the provider is asked, so the
  // sentence survives: it names the body and what the provider answers.
  const onlySun = {
    name: 'only-sun',
    bodies: [Body.Sun],
    positions: () => assert.fail('the provider must not be asked'),
  };
  assert.throws(
    () => new Context({ provider: onlySun }).positions({ instants: [2451545.0], bodies: [Body.Mars] }),
    (error) => {
      assert.equal(error.status, 'unsupported');
      assert.match(error.message, /MARS; it answers SUN/u);
      return true;
    },
  );

  // A provider is checked at the door, before a call can reach it.
  assert.throws(() => new Context({ provider: { name: 'x', bodies: [Body.Sun] } }), TypeError);
  assert.throws(() => new Context({ provider: { bodies: [Body.Sun], positions() {} } }), TypeError);
  assert.throws(() => new Context({ provider: { name: 'x', bodies: [], positions() {} } }), TypeError);
  assert.throws(
    () => new Context({ provider: { name: 'x', bodies: ['graha.PLOOTO'], positions() {} } }),
    /not a body the port knows/u,
  );
});

test('a provider answers only the bodies it declared', () => {
  const { provider } = jsProvider({ bodies: [Body.Sun] });
  const ctx = new Context({ provider });
  assert.throws(
    () => ctx.positions({ instants: [2451545.0], bodies: [Body.Sun, Body.Mars] }),
    (error) => error.status === 'unsupported' || error.status === 'capability',
  );
});

test('a disposed context says so, and disposing twice is allowed', () => {
  const ctx = new Context({ testProvider: true });
  assert.equal(typeof ctx.profile, 'string');
  ctx.dispose();
  // Idempotent, because a `using` scope and an explicit call both run it.
  ctx.dispose();
  // Named here rather than at the boundary, which would only say
  // `invalid argument` and not which argument. The Dart and Python
  // bindings answer the same way.
  assert.throws(() => ctx.profile, /this context was disposed/u);
  assert.throws(
    () => ctx.positions({ instants: [2451545.0], bodies: [Body.Sun] }),
    /this context was disposed/u,
  );
});

test('every catalogue enum has a complete id table', () => {
  // The tables let a caller turn an id the boundary gave back — a cell's
  // source, a decoded column, a key's low half — into the enum value it
  // stands for. They are generated, so what is worth holding is that
  // each one is *complete* and agrees with its own enum in both
  // directions; a table missing a member fails silently at the one
  // lookup that needs it.
  const tables = Object.keys(catalogue).filter(
    (name) => name.endsWith('ById') && catalogue[name] instanceof Map,
  );
  assert.ok(tables.length > 50, `expected the whole catalogue, got ${tables.length} tables`);
  let entries = 0;
  for (const name of tables) {
    const base = name.slice(0, -'ById'.length);
    const values = catalogue[base];
    assert.ok(values, `${name} has no companion \`${base}\``);
    const known = new Set(Object.values(values));
    const mapped = new Set();
    for (const [id, key] of catalogue[name]) {
      entries += 1;
      assert.equal(typeof id, 'number', `${name} is keyed by ${typeof id}, not an id`);
      assert.ok(known.has(key), `${name}[${id}] is \`${key}\`, not a value of ${base}`);
      mapped.add(key);
    }
    for (const key of known) {
      assert.ok(mapped.has(key), `${base}.\`${key}\` is missing from ${name}`);
    }
  }
  // The total is a canary for a whole enum vanishing, which the
  // per-enum check above cannot see. It moves whenever the catalogue or
  // the boundary gains a member, which is a deliberate change: the
  // description's own page reports the same figure.
  assert.equal(entries, 949, 'every member of every enum is in a table');
});

test('a birth with no time is refused, or reported, but never guessed', () => {
  const day = date(Calendar.BikramSambat, 2042, 9, 17);
  const zone = ianaZone('Asia/Kathmandu');

  // No policy: refused by name, with the hint naming the three choices.
  const strict = new Context({ profile: 'nepali-default', testProvider: true });
  assert.throws(
    () => strict.time.resolve(whenUnknown(day), zone),
    (error) => {
      assert.match(error.message, /has no time of day/u);
      assert.match(error.hint, /NOON, MIDNIGHT or SUNRISE/u);
      assert.equal(error.field, 'time');
      return true;
    },
  );
  strict.dispose();

  // NOON: answered, and said twice — the resolution reports the time as
  // unknown *and* warns, so a stored chart cannot claim a time it never
  // had.
  const noon = new Context({
    profile: 'nepali-default',
    testProvider: true,
    settings: { time: { unknown_time: 'NOON' } },
  });
  const resolved = noon.time.resolve(whenUnknown(day), zone);
  assert.equal(resolved.timeKnown, false);
  assert.ok(resolved.warnings.includes('time-unknown-fallback'), 'the fallback is warned about');
  assert.ok(Number.isFinite(resolved.instantJdUtc));
  noon.dispose();

  // A known time on the same date resolves with the time known and no
  // warning: this record sits on the day Nepal moved to +05:45.
  const known = new Context({ profile: 'nepali-default', testProvider: true });
  const exact = known.time.resolve(at(day, { hour: 0, minute: 20 }), zone);
  assert.equal(exact.timeKnown, true);
  assert.equal(exact.offsetSeconds, 5 * 3600 + 45 * 60);
  assert.deepEqual(exact.warnings, []);
  known.dispose();
});


// ── The engine's own operations ──────────────────────────────────────
// The test provider ships a two-function manifest, so these run with no
// real engine present and still exercise the whole route.

test('the engine names its own operations', () => {
  const engine = context().engine;
  assert.ok(engine.names.includes('tp_echo'));
  assert.equal(engine.manifest.engine, 'test-provider');
});

test('an operation is called by the name the engine gives it', () => {
  const engine = context().engine;
  assert.deepEqual(engine.call('tp_echo', { value: 6 }), { value: 6 });
  assert.equal(engine.call('tp_sum', { values: [1, 2, 3.5] }).total, 6.5);
  // The JSON form, for an answer being handed on rather than read.
  assert.equal(engine.callJson('tp_echo', '{"value":6}'), '{"value":6.0}');
});

test('the names come from the engine and not from the package', () => {
  const engine = context().engine;
  assert.equal(engine.signature('tm_eclipse_when'), undefined);
  // And the manifest carries the role of every parameter.
  assert.deepEqual(
    engine.signature('tp_sum').params.map((p) => p.role),
    ['array_in', 'array_len', 'scalar_out'],
  );
});

/**
 * There is no index signature and no proxy: a name is reached by `call`
 * and not by spelling it on the object.
 *
 * ADR-0030 considered the proxy and rejected it under ADR-0023 — it
 * type-checks the misspelling, and Dart and Rust cannot express it — so
 * this asserts the absence, because the absence is the decision.
 */
test('an operation is not a property of the engine', () => {
  const engine = context().engine;
  assert.equal(engine.tp_echo, undefined, 'reached by `call`, not by name');
  assert.ok(Object.isFrozen(engine), 'and nothing can be added to it');
});

test("the engine's own refusal comes back", () => {
  const engine = context().engine;
  assert.throws(() => engine.call('tm_eclipse_when'), /tm_eclipse_when/);
});

/**
 * An area is a **value**: built once with the context, destructurable,
 * and passable to something that needs only that much of the SDK. That
 * is what makes the grouping worth having rather than merely tidy
 * (`03-design/surface-areas.md`).
 */
test('an area is a value that can be destructured and kept', () => {
  const ctx = context();
  const { calendar, time, keys } = ctx;
  assert.equal(calendar, ctx.calendar, 'the same object every read');
  assert.equal(calendar.isLeap(Calendar.Gregorian, 2024), true);
  assert.equal(time.deltaT(2451545.0).seconds > 60, true);
  assert.equal(keys.name(keys.id('graha.SUN')), 'graha.SUN');
  assert.ok(Object.isFrozen(calendar), 'and nothing can be added to it');
});

/**
 * **An engine, plugged in** (ADR-0029): the 98% path, where a consumer
 * names an adapter's platform binary and never sees a vtable.
 *
 * It runs only where the adapter has been built and its data is present,
 * because a checkout has neither and a test that failed for that would
 * fail for everyone. `TEISTRO_TEIMERIS_ADAPTER` names the library — the
 * same variable `crates/ffi/tests/abi.rs` reads for the same reason.
 */
test('an ephemeris is plugged in by naming its platform binary', () => {
  const plugin = process.env.TEISTRO_TEIMERIS_ADAPTER;
  if (!plugin) {
    console.log('skipped: set TEISTRO_TEIMERIS_ADAPTER to the adapter\'s library');
    return;
  }
  // `dispose()` in a `finally`, not `using`: the explicit resource
  // management syntax needs Node 24 and this suite runs on 20.
  const ctx = new Context({
    profile: 'parashari-classical',
    ephemeris: { plugin },
  });
  try {
    const sky = ctx.positions({ instants: [2451545.0], bodies: [Body.Sun] });
    // The Sun at J2000 is near 280.4°, which is astronomy rather than
    // this package: what is being tested is that a real engine answered.
    assert.ok(
      Math.abs(sky.at(0, 0).longitude - 280.37) < 0.5,
      `the Sun at J2000 came back as ${sky.at(0, 0).longitude}`,
    );
    // And its own functions came with it, which no SDK operation offers.
    assert.equal(ctx.engine.manifest.engine, 'teimeris');
    assert.equal(ctx.engine.call('tm_body_name', { body: 0 }).buf, 'Sun');
  } finally {
    ctx.dispose();
  }
});

/**
 * A chain is **ordered and explicit** (ADR-0029): tried in order, and a
 * refusal names every entry that failed rather than only the last, which
 * would hide the one the caller actually wanted. Needs no adapter.
 */
test('an ephemeris chain is tried in order and refuses naming each', () => {
  // An adapter that is not there, then the built-in: the fallback the
  // caller wrote down.
  const fellBack = new Context({
    profile: 'parashari-classical',
    ephemeris: [{ plugin: '/nowhere/adapter.so' }, 'builtin'],
  });
  try {
    const sky = fellBack.positions({ instants: [2451545.0], bodies: [Body.Sun] });
    assert.ok(Math.abs(sky.at(0, 0).longitude - 280.37) < 0.5);
  } finally {
    fellBack.dispose();
  }

  // Nothing in the chain opening is one refusal that names each.
  assert.throws(
    () => new Context({ ephemeris: [{ plugin: '/a.so' }, { plugin: '/b.so' }] }),
    (error) => /a\.so/u.test(error.message) && /b\.so/u.test(error.message),
  );

  // A chain of none names nothing, which is a mistake rather than a
  // default; and a descriptor without a `plugin` is not a descriptor.
  assert.throws(() => new Context({ ephemeris: [] }), /names nothing/u);
  assert.throws(() => new Context({ ephemeris: [{ config: {} }] }), /descriptor/u);
});

/**
 * `provider` and `ephemeris` each answer one question, so both together
 * is a refusal rather than one silently winning.
 */
test('a provider and a named ephemeris together are refused', () => {
  assert.throws(
    () =>
      new Context({
        ephemeris: 'builtin',
        provider: { name: 'x', bodies: [], positions: () => null },
      }),
    /give one of them/u,
  );
});

test('a context without an ephemeris says so', () => {
  const bare = new Context({ profile: 'nepali-default' });
  assert.throws(() => bare.engine);
});
