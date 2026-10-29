// The package's surface as an application imports it: `compat()`, the
// shim's `calls()` and `dispose()`, `CompatRefused`, and the A/B's
// vocabulary, `facts` and `FAMILIES` (`docs/03-design/compat-shim.md` §2,
// §5, §6.1, §6.3). Its own process, so the counts it reads are its own.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

import * as pkg from '@teistro/compat';
import { CompatRefused, FAMILIES, compat, facts } from '@teistro/compat';

/** The rows of the surface (`surface.json`), each counted from zero. */
const ROWS = JSON.parse(readFileSync(new URL('../surface.json', import.meta.url), 'utf8')).rows.map(({ row }) => row);

test('the package exports the entry, the refusal and the vocabulary, and nothing of its insides', () => {
  assert.deepEqual(Object.keys(pkg).sort(), ['CompatRefused', 'FAMILIES', 'compat', 'facts']);
  const refused = new CompatRefused('birth.instant', 'docs/09-guidelines/baseline-migration.md#82-what-to-store-with-the-sdk', 'store the instant once', {
    cause: 'why',
  });
  assert.ok(refused instanceof Error);
  assert.deepEqual([refused.name, refused.row, refused.cause], ['CompatRefused', 'birth.instant', 'why']);
  assert.ok(Object.isFrozen(FAMILIES));
  assert.equal(FAMILIES.longitude.kind, 'angle');
});

test('a shim counts the calls of its rows, and none are made yet', () => {
  const shim = compat({ ephemeris: 'TEST', locale: 'ne-Deva-NP' });
  const counts = shim.calls();
  assert.ok(Object.isFrozen(counts));
  assert.deepEqual(counts, Object.fromEntries(ROWS.map((row) => [row, 0])));
  shim.dispose();
});

test('a row missing what it needs is a TypeError naming the option, before any call is counted', () => {
  const shim = compat({ ephemeris: 'TEST' });
  for (const [row, field] of [
    ['month', 'from'],
    ['milan', 'partner'],
    ['varsha', 'through'],
    ['gochar', 'instants'],
    ['sadeSati', 'from'],
    ['batch', 'instants'],
    ['drawing', 'drawings'],
    ['narrative', 'interpret'],
    ['muhurta', 'from'],
    ['sarvatobhadra', 'instants'],
    ['eclipses', 'from'],
    ['ingresses', 'from'],
    ['lunations', 'from'],
    ['pakshi', 'from'],
    ['transitCalendar', 'hits'],
    ['returns', 'from'],
    ['naam', 'partner'],
    ['rectify', 'uncertaintyMinutes'],
    ['nameCheck', 'names'],
    ['synastry', 'partner'],
    ['rashifal', 'first'],
    ['milanVerdict', 'partner'],
  ]) {
    assert.throws(
      () => shim[row]({}, {}),
      (error) => error instanceof TypeError && error.message.startsWith(`options.${field}: required`),
      row,
    );
  }
  assert.throws(() => shim.muhurta({}, {}, { from: 1, to: 2 }), /options\.search: required/);
  assert.throws(() => shim.pakshi({}, {}, { from: 1, to: 2 }), /options\.native: required/);
  assert.throws(() => shim.research({}, {}, {}), /births: expected the stored birth records/);
  assert.throws(() => shim.research([], {}), /study: required/);
  assert.equal(Object.values(shim.calls()).reduce((a, b) => a + b), 0);
  shim.dispose();
});

test('a call that reads the stored name refuses a record without one', () => {
  const shim = compat({ ephemeris: 'TEST' });
  const birth = {
    dateTimeUtc: '1990-04-13T23:45:00.000Z',
    julianDay: 2447995.4895833335,
    latitude: 27.7172,
    longitude: 85.324,
    altitude: 1400,
    timezone: 'Asia/Kathmandu',
    isLmt: false,
  };
  assert.throws(() => shim.numerology(birth, {}), /birth\.name: numerology reads the stored name/);
  assert.throws(() => shim.naam({ ...birth, name: 'राम' }, {}, { partner: birth }), /partner\.name: naam reads the stored name/);
  shim.dispose();
});

test('facts reads an answer into the vocabulary, and refuses a request it has none for', () => {
  const chart = {
    grahas: [{ graha: 'graha.SUN', longitudeDeg: 31, retrograde: false }],
    lagnaDeg: 365,
    dashas: [],
    rules: { present: [{ rule: 'GAJA_KESARI' }] },
  };
  const read = new Map(facts('chart', chart));
  assert.equal(read.get('sign/SUN'), 1);
  assert.equal(read.get('longitude/LAGNA'), 5);
  assert.deepEqual(facts('yogas', chart), [['yoga/GAJA_KESARI', true]]);
  assert.deepEqual(facts('yogas', chart, { figures: [['GAJA_KESARI'], ['RUCHAKA']] }), [
    ['yoga/GAJA_KESARI', true],
    ['yoga/RUCHAKA', false],
  ]);
  for (const [key] of read) assert.ok(key.slice(0, key.indexOf('/')) in FAMILIES, key);
  assert.throws(() => facts('sadeSati', { sadeSati: { sadeSati: [] } }), /options\.window: required/);
  assert.deepEqual(facts('sadeSati', { sadeSati: { sadeSati: [] } }, { window: { from: 0, to: 1 } }), []);
  assert.throws(() => facts('naam', {}), /request: no facts for naam/);
});

test('a shim refuses the options a snapshot decides, and disposes once or by `using`', () => {
  assert.throws(() => compat({ profile: 'parashari-classical' }), TypeError);
  const shim = compat();
  shim.dispose();
  shim.dispose();
  const scoped = compat({ ephemeris: ['TEST'] });
  scoped[Symbol.dispose]();
});
