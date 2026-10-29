// The package's surface as an application imports it: `compat()`, the
// shim's `calls()` and `dispose()`, and `CompatRefused`
// (`docs/03-design/compat-shim.md` §2, §6.1, §6.3). Its own process, so the
// counts it reads are its own.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import * as pkg from '@teistro/compat';
import { CompatRefused, compat } from '@teistro/compat';

test('the package exports the entry and the refusal, and nothing of its insides', () => {
  assert.deepEqual(Object.keys(pkg).sort(), ['CompatRefused', 'compat']);
  const refused = new CompatRefused('birth.instant', 'docs/09-guidelines/baseline-migration.md#82-what-to-store-with-the-sdk', 'store the instant once', {
    cause: 'why',
  });
  assert.ok(refused instanceof Error);
  assert.deepEqual([refused.name, refused.row, refused.cause], ['CompatRefused', 'birth.instant', 'why']);
});

/** The rows the shim forwards (§7 item 5), each counted from zero. */
const ROWS = ['chart', 'full', 'vargas', 'yogas', 'strength', 'dashas', 'day', 'month', 'milan', 'varsha', 'kp', 'gochar', 'sadeSati', 'batch'];

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
  ]) {
    assert.throws(() => shim[row]({}, {}), (error) => error instanceof TypeError && error.message.startsWith(`options.${field}: required`));
  }
  assert.equal(Object.values(shim.calls()).reduce((a, b) => a + b), 0);
  shim.dispose();
});

test('a shim refuses the options a snapshot decides, and disposes once or by `using`', () => {
  assert.throws(() => compat({ profile: 'parashari-classical' }), TypeError);
  const shim = compat();
  shim.dispose();
  shim.dispose();
  const scoped = compat({ ephemeris: ['TEST'] });
  scoped[Symbol.dispose]();
});
