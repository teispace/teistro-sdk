// What every shim call is built from, branch by branch: the stored birth
// record read, the snapshot translated, the contexts kept and disposed, the
// refusal and the count (`docs/03-design/compat-shim.md` §2, §3.7, §6).
//
// `cargo xtask check-node` runs it after the binding's own suite, with
// `@teistro/sdk` resolved to the binding beside this package (`sdk.mjs`).

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { TeistroError, ianaZone, localMeanZone } from '@teistro/sdk';

import {
  CONTEXTS,
  CompatRefused,
  Contexts,
  GUIDE,
  INSTANT_TOLERANCE_MS,
  TRANSLATIONS,
  at,
  calls,
  count,
  merged,
  readBirth,
} from '../lib/core.js';

/** Kathmandu, 1990-04-13 23:45 UTC: the corpus's c001, as an application stores it. */
const RECORD = Object.freeze({
  name: 'Rama',
  dateTimeUtc: new Date('1990-04-13T23:45:00.000Z'),
  julianDay: 2447995.4895833335,
  latitude: 27.7172,
  longitude: 85.324,
  altitude: 1400,
  timezone: 'Asia/Kathmandu',
  isLmt: false,
  gender: 'female',
  birthTimeUncertainty: 'approximate',
  birthOrder: 2,
  bsDate: '2047-01-01',
});

/** The corpus's `default` snapshot, as the recording engine stored it. */
const DEFAULT = Object.freeze({
  ayanamsaId: 1,
  zodiacMode: 'sidereal',
  houseSystem: 'whole-sign',
  siddhanta: 'drik',
  nodeType: 'mean',
  sunriseMode: 'apparent-refraction',
  dashaBalanceMethod: 'spatial',
  dashaMaxLevel: 3,
  ekadhipatyaMethod: 'classical',
  useTopocentric: true,
  engineVersion: '1.8.0',
  swissephVersion: '2.10.03',
});

/** The instant a stored date is: its UT Julian day. */
const INSTANT = 2440587.5 + RECORD.dateTimeUtc.getTime() / 86_400_000;

/** `n` snapshots that each translate to settings of their own. */
function distinct(n) {
  const out = [];
  for (const nodeType of ['mean', 'true']) {
    for (const useTopocentric of [true, false]) {
      for (let dashaMaxLevel = 1; dashaMaxLevel <= 5; dashaMaxLevel += 1) {
        out.push({ ...DEFAULT, nodeType, useTopocentric, dashaMaxLevel });
      }
    }
  }
  assert.ok(n <= out.length, `only ${out.length} distinct snapshots`);
  return out.slice(0, n);
}

/** Whether a context has been disposed: a disposed one says so on use. */
function disposed(ctx) {
  try {
    void ctx.settingsHash;
    return false;
  } catch (error) {
    assert.match(error.message, /disposed/);
    return true;
  }
}

test('a stored birth record is its date, its place and its zone', () => {
  const birth = readBirth(RECORD);
  assert.ok(Object.isFrozen(birth) && Object.isFrozen(birth.place));
  assert.equal(birth.instant, INSTANT);
  // The corpus's Julian day, derived from the same date, within a double's rounding.
  assert.ok(Math.abs(birth.instant - RECORD.julianDay) * 86_400_000 < 0.05);
  assert.deepEqual(birth.place, { latitude: 27.7172, longitude: 85.324, altitude: 1400 });
  assert.deepEqual(birth.zone, ianaZone('Asia/Kathmandu'));
  // Kept for the calls that read them; the birth order and BS date are not read.
  assert.deepEqual([birth.name, birth.gender, birth.timeAccuracy], ['Rama', 'female', 'approximate']);
  assert.equal('birthOrder' in birth || 'bsDate' in birth, false);
  // A row read as JSON holds the date as its ISO text.
  assert.deepEqual(readBirth({ ...RECORD, dateTimeUtc: '1990-04-13T23:45:00Z' }), birth);
  // Local mean time is read at the longitude, whatever zone is stored.
  const lmt = readBirth({ ...RECORD, isLmt: true, timezone: undefined });
  assert.deepEqual(lmt.zone, localMeanZone(85.324));
});

test('a Julian day kept to six decimals agrees with its date, and one further apart is refused', () => {
  // As an application's birth table keeps it: the instant is still the date's.
  const kept = Number(RECORD.julianDay.toFixed(6));
  assert.equal(readBirth({ ...RECORD, julianDay: kept }).instant, INSTANT);
  // Half the step of the sixth decimal is the farthest a kept day stands.
  const halfStep = 0.5e-6 * 86_400_000;
  assert.ok(halfStep < INSTANT_TOLERANCE_MS && INSTANT_TOLERANCE_MS - halfStep < 0.1);
  const ms = RECORD.dateTimeUtc.getTime();
  assert.equal(readBirth({ ...RECORD, dateTimeUtc: new Date(ms + 43) }).instant, 2440587.5 + (ms + 43) / 86_400_000);
  assert.throws(
    () => readBirth({ ...RECORD, dateTimeUtc: new Date(ms + 44) }),
    (error) =>
      error instanceof CompatRefused &&
      error.row === 'birth.instant' &&
      error.guide === GUIDE.birth &&
      /ms apart/.test(error.message) &&
      String(error).startsWith('CompatRefused [birth.instant]: '),
  );
});

test('a birth record of the wrong shape is a TypeError naming its field', () => {
  for (const record of [null, [RECORD], 'c001']) {
    assert.throws(() => readBirth(record), /^TypeError: birth: expected the stored birth record/);
  }
  const wrong = [
    [{ julianDay: '2447995.49' }, /birth\.julianDay/],
    [{ dateTimeUtc: 638_063_100_000 }, /birth\.dateTimeUtc/],
    [{ dateTimeUtc: 'yesterday' }, /birth\.dateTimeUtc/],
    [{ latitude: Number.POSITIVE_INFINITY }, /birth\.latitude/],
    [{ altitude: undefined }, /birth\.altitude/],
    [{ isLmt: 'no' }, /birth\.isLmt/],
    [{ timezone: null }, /birth\.timezone/],
  ];
  for (const [change, field] of wrong) {
    assert.throws(() => readBirth({ ...RECORD, ...change }), (error) => error instanceof TypeError && field.test(error.message));
  }
});

test('a birth founds at its instant, its place and the clock its zone kept then', () => {
  const contexts = new Contexts({ ephemeris: 'TEST' });
  const ctx = contexts.natal(DEFAULT);
  assert.deepEqual(at(ctx, readBirth(RECORD)), {
    instant: INSTANT,
    place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
    utcOffsetSeconds: 20700,
  });
  // A zone the database does not have is the SDK's refusal, untouched.
  assert.throws(() => at(ctx, readBirth({ ...RECORD, timezone: 'Asia/Atlantis' })), TeistroError);
  contexts.dispose();
});

test('every row is counted from zero, and a row called is counted however it was listed', () => {
  count('chart');
  count('chart');
  count('day');
  const counts = calls(['chart', 'kp']);
  assert.ok(Object.isFrozen(counts));
  assert.deepEqual(counts, { chart: 2, kp: 0, day: 1 });
});

test('the options are a context’s, less what a snapshot decides', () => {
  for (const options of [null, ['TEST'], 'TEST']) {
    assert.throws(() => new Contexts(options), /^TypeError: options: expected the context options/);
  }
  assert.throws(() => new Contexts({ profile: 'parashari-classical' }), /^TypeError: profile: each stored snapshot decides it/);
  assert.throws(() => new Contexts({ settings: {} }), /^TypeError: settings: each stored snapshot decides it/);
  // With no options the SDK's default ephemeris is the chain.
  const contexts = new Contexts();
  assert.equal(contexts.natal(DEFAULT).profile, 'conformance-baseline');
  contexts.dispose();
});

test('a snapshot translates once, under its text', () => {
  const contexts = new Contexts({ ephemeris: ['TEST'] });
  const translated = contexts.translate(DEFAULT);
  assert.equal(contexts.translate(DEFAULT), translated);
  assert.equal(translated.recorded.natal.profile, 'conformance-baseline');
  // Its JSON is another text, translated to the same key.
  assert.equal(contexts.translate(JSON.stringify(DEFAULT)).key, translated.key);
  // The oldest translation is let go to keep the newest.
  const first = { ...DEFAULT, engineVersion: 'v0' };
  const kept = contexts.translate(first);
  for (let at = 1; at <= TRANSLATIONS; at += 1) contexts.translate({ ...DEFAULT, engineVersion: `v${at}` });
  const again = contexts.translate(first);
  assert.notEqual(again, kept);
  assert.equal(again.key, kept.key);
  contexts.dispose();
});

test('the locale is the snapshot’s, else the options’, else the SDK’s', () => {
  const contexts = new Contexts({ ephemeris: 'TEST', locale: 'hi-Deva-IN' });
  assert.equal(contexts.translate({ ...DEFAULT, locale: { language: 'en', script: 'Latn' } }).locale, 'en-Latn');
  assert.equal(contexts.translate(DEFAULT).locale, 'hi-Deva-IN');
  assert.equal(contexts.natal({ ...DEFAULT, locale: { language: 'en' } }).intl.locale, 'en-Latn');
  assert.equal(contexts.natal(DEFAULT).intl.locale, 'hi-Deva-IN');
  contexts.dispose();
  const bare = new Contexts({ ephemeris: 'TEST' });
  assert.equal(bare.translate(DEFAULT).locale, undefined);
  assert.equal(typeof bare.natal(DEFAULT).intl.locale, 'string');
  bare.dispose();
});

test('a stored field or value the SDK refuses is CompatRefused, its record the cause', () => {
  const contexts = new Contexts({ ephemeris: 'TEST' });
  assert.throws(
    () => contexts.natal({ ...DEFAULT, suryaBija: true }),
    (error) =>
      error instanceof CompatRefused &&
      error.row === 'snapshot.suryaBija' &&
      error.guide === GUIDE.snapshot &&
      / instead$/.test(error.message) &&
      error.cause instanceof TeistroError &&
      error.cause.status === 'UNSUPPORTED' &&
      error.cause.hint === 'C777',
  );
  assert.throws(
    () => contexts.natal({ ...DEFAULT, nodeType: 'osculating' }),
    (error) => error instanceof CompatRefused && error.row === 'snapshot.nodeType' && error.cause.status === 'INVALID_ARG',
  );
  // Anything else the SDK says passes through as itself.
  assert.throws(() => contexts.natal([1]), (error) => error instanceof TypeError && !(error instanceof CompatRefused));
  assert.throws(
    () => contexts.natal('{"ayanamsaId": '),
    (error) => error instanceof TeistroError && !(error instanceof CompatRefused),
  );
  // A refused snapshot opened nothing.
  assert.equal(contexts.size, 0);
  contexts.dispose();
});

test('a natal context is kept per translation, and KP is opened on first use beside it', () => {
  const contexts = new Contexts({ ephemeris: 'TEST' });
  const natal = contexts.natal(DEFAULT);
  assert.equal(natal.profile, 'conformance-baseline');
  assert.equal(natal.settings.dasha.depth.VIMSHOTTARI, 3);
  // A snapshot differing only in a field set aside shares the context.
  assert.equal(contexts.natal({ ...DEFAULT, engineVersion: '1.9.0' }), natal);
  assert.notEqual(contexts.natal({ ...DEFAULT, nodeType: 'true' }), natal);
  const kp = contexts.kp(DEFAULT);
  assert.equal(kp.profile, 'kp-default');
  assert.equal(contexts.kp(DEFAULT), kp);
  assert.equal(contexts.size, 2);
  contexts.dispose();
});

test('the Surya Siddhanta goes in front of the chain, or stands alone', () => {
  const surya = { ...DEFAULT, siddhanta: 'surya' };
  for (const options of [{ ephemeris: 'TEST' }, {}]) {
    const contexts = new Contexts(options);
    const natal = contexts.natal(surya);
    assert.equal(natal.settings.frame.siddhanta.kind, 'SURYA');
    // KP is on the chain the caller named, without the text.
    assert.equal(contexts.kp(surya).profile, 'kp-default');
    contexts.dispose();
  }
});

test('a seventeenth snapshot disposes the one used longest ago, KP and all', () => {
  const contexts = new Contexts({ ephemeris: 'TEST' });
  const [first, second, ...rest] = distinct(CONTEXTS + 1);
  const firstNatal = contexts.natal(first);
  const secondNatal = contexts.natal(second);
  const secondKp = contexts.kp(second);
  for (const snapshot of rest.slice(0, CONTEXTS - 2)) contexts.natal(snapshot);
  assert.equal(contexts.size, CONTEXTS);
  // Using the first makes the second the oldest.
  assert.equal(contexts.natal(first), firstNatal);
  contexts.natal(rest[CONTEXTS - 2]);
  assert.equal(contexts.size, CONTEXTS);
  assert.ok(disposed(secondNatal) && disposed(secondKp));
  assert.ok(!disposed(firstNatal));
  // Asked again, the second opens afresh.
  assert.notEqual(contexts.natal(second), secondNatal);
  contexts.dispose();
});

test('disposing frees every context once, and a disposed shim says so', () => {
  const contexts = new Contexts({ ephemeris: 'TEST' });
  const natal = contexts.natal(DEFAULT);
  const kp = contexts.kp(DEFAULT);
  contexts.dispose();
  contexts.dispose();
  assert.ok(disposed(natal) && disposed(kp));
  assert.equal(contexts.size, 0);
  assert.throws(() => contexts.natal(DEFAULT), /this shim was disposed; open another one with compat\(\)/);
  // One that never translated has nothing to free.
  new Contexts().dispose();
});

test('a variant is the natal context with a patch laid over it, kept by name and freed with it', () => {
  const contexts = new Contexts({ ephemeris: 'TEST' });
  const natal = contexts.natal(DEFAULT);
  const muhurta = contexts.variant(DEFAULT, 'muhurta', { frame: { node: 'TRUE', centre: 'GEOCENTRIC' } });
  assert.notEqual(muhurta, natal);
  assert.deepEqual([muhurta.settings.frame.node, muhurta.settings.frame.centre], ['TRUE', 'GEOCENTRIC']);
  // The rest is the snapshot's: its dasha depth stands.
  assert.equal(muhurta.settings.dasha.depth.VIMSHOTTARI, natal.settings.dasha.depth.VIMSHOTTARI);
  assert.equal(contexts.variant(DEFAULT, 'muhurta', {}), muhurta, 'kept by name');
  const ghati = contexts.variant(DEFAULT, 'ghati', { day: { ghati_reckoning: 'PROPORTIONAL' } });
  assert.equal(ghati.settings.day.ghati_reckoning, 'PROPORTIONAL');
  assert.equal(contexts.size, 1, 'variants are the snapshot’s, not snapshots of their own');
  contexts.dispose();
  assert.ok(disposed(muhurta) && disposed(ghati) && disposed(natal));
});

test('settings patches merge group by group', () => {
  assert.deepEqual(merged({ frame: { node: 'MEAN', centre: 'TOPOCENTRIC' }, day: { x: 1 } }, { frame: { node: 'TRUE' } }), {
    frame: { node: 'TRUE', centre: 'TOPOCENTRIC' },
    day: { x: 1 },
  });
  assert.deepEqual(merged(undefined, { day: { y: 2 } }), { day: { y: 2 } });
});
