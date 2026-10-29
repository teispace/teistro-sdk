// A consumer of the published `@teistro/compat`, and nothing else.
//
// The package's own tests import `@teistro/compat` through a resolve hook
// that points it at this checkout, so they prove the code and not the
// package: the name, the export map, the `files` list (its `surface.json`
// above all, which the package reads for its rows) and the exact peer
// dependency on `@teistro/sdk` are only exercised by importing it from a
// project that installed it. That is what this file does, and
// `cargo xtask check-package` runs it beside the SDK's own consumer in the
// throwaway project built from the staged packages.

import assert from 'node:assert/strict';

import { Context } from '@teistro/sdk';
import { CompatRefused, compat } from '@teistro/compat';

/** Kathmandu, 1990-04-13 23:45 UTC, as an application on the baseline stores a birth. */
const birth = {
  dateTimeUtc: new Date('1990-04-13T23:45:00.000Z'),
  julianDay: 2447995.4895833335,
  latitude: 27.7172,
  longitude: 85.324,
  altitude: 1400,
  timezone: 'Asia/Kathmandu',
  isLmt: false,
};

/** The settings snapshot the baseline stored beside the chart. */
const snapshot = { ayanamsaId: 1, houseSystem: 'whole-sign', nodeType: 'mean', useTopocentric: true, engineVersion: '1.8.0' };

const shim = compat({ ephemeris: 'TEST' });
try {
  const chart = shim.chart(birth, snapshot);
  console.log(`the shim's chart has ${chart.grahas.length} grahas, lagna ${chart.lagnaDeg.toFixed(4)}°`);
  assert.ok(chart.grahas.some((placed) => placed.graha === 'graha.SUN'), 'the Sun is placed');

  // Its settings are the translation's: a context opened from it seals the same hash.
  const translator = new Context({ ephemeris: 'NONE' });
  const { natal } = translator.recordedSettings(snapshot);
  translator.dispose();
  const ctx = new Context({ ephemeris: 'TEST', profile: natal.profile, settings: natal.settings });
  assert.equal(chart.provenance.settingsHash, ctx.settingsHash);
  ctx.dispose();

  // A refused row throws at once, naming a guide anchor shipped in its rows.
  assert.throws(
    () => shim.zoneAt(27.7, 85.3),
    (error) => error instanceof CompatRefused && error.row === 'zoneAt' && error.guide.includes('#'),
  );
  const counts = shim.calls();
  assert.deepEqual([counts.chart, counts.zoneAt], [1, 1]);
  console.log(`${Object.keys(counts).length} rows counted, chart and zoneAt called once each`);
} finally {
  shim.dispose();
}
