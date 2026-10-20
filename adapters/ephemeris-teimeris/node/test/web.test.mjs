// The web provider over the engine's own wasm module: what it refuses,
// what it claims, and a range it adds answering where the built-in years
// end. `cargo xtask check-wasm` runs it with the engine's package and its
// published data; each needs a Teimeris checkout, and without one the
// tests say so and skip.
//
//   TEIMERIS_WASM_PACKAGE=<teimeris>/bindings/wasm \
//   TEIMERIS_DATA_DIR=<teimeris>/data node --test test/web.test.mjs

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { test } from 'node:test';

import { BLOCKS, FILES } from '../ranges.js';
import { teimerisWeb } from '../web.js';

const require = createRequire(import.meta.url);
const PACKAGE = process.env.TEIMERIS_WASM_PACKAGE;
const DATA = process.env.TEIMERIS_DATA_DIR;

/** The engine's module, initialised, or null without a checkout. */
async function engine() {
  if (!PACKAGE) return null;
  const module = require(join(PACKAGE, 'lib', 'index.js'));
  await module.init();
  return module;
}

/** A published file as an app would hold it. */
function file(name) {
  return { name, bytes: new Uint8Array(readFileSync(join(DATA, 'ext', name))) };
}

/** A request for the Sun, tropical and geocentric, as the SDK asks it. */
function sun(jds) {
  return { jds, bodies: ['SUN'], scale: 'UT1', frameBits: 0xf0, speeds: true };
}

test('the manifest names every block with both its files', () => {
  assert.equal(BLOCKS.length, 51);
  assert.equal(FILES.length, 153);
  for (const block of BLOCKS) {
    assert.ok(FILES.some((f) => f.file === block.planets), block.planets);
    assert.ok(FILES.some((f) => f.file === block.moon), block.moon);
    assert.ok(block.start < block.end, `${block.century}`);
  }
});

test('the built-in years are claimed, and only they', async (t) => {
  const tm = await engine();
  if (!tm) return t.skip('set TEIMERIS_WASM_PACKAGE to the engine\'s wasm package');
  const provider = await teimerisWeb(tm);
  const built = BLOCKS.find((b) => b.century === 18);
  assert.equal(provider.jdMin, built.start);
  assert.equal(provider.jdMax, built.end);
  assert.equal(provider.name, 'teimeris');
  assert.equal(provider.dataVersion, 'seas_18.se1 semo_18.se1 sepl_18.se1');
  assert.equal(provider.bodies.length, 14);
  const answer = provider.positions(sun([2451545.0]));
  assert.equal(answer.status[0], 0);
  assert.ok(Math.abs(answer.lon[0] - 280.37) < 0.01, `${answer.lon[0]}`);
  assert.equal(answer.source[0], 1, 'computed from the files');
});

test('a file that is not the published copy is refused by name', async (t) => {
  const tm = await engine();
  if (!tm) return t.skip('set TEIMERIS_WASM_PACKAGE to the engine\'s wasm package');
  await assert.rejects(
    teimerisWeb(tm, { files: [{ name: 'sepl_12.se1', bytes: new Uint8Array(10) }] }),
    /`sepl_12\.se1` is 10 bytes .*Download it again/u,
  );
  await assert.rejects(teimerisWeb(tm, { files: [{ name: 'notes.txt', bytes: new Uint8Array(1) }] }), /names no data block/u);
  await assert.rejects(
    teimerisWeb(tm, {
      files: [
        { name: 'sepl_12.se1', bytes: new Uint8Array(1) },
        { name: 'sepl_12.se1', bytes: new Uint8Array(1) },
      ],
      verify: false,
    }),
    /given twice/u,
  );
  await assert.rejects(teimerisWeb(tm, { profile: 'fast' }), /not one of the engine's/u);
  await assert.rejects(teimerisWeb({}), /teimeris-wasm/u);
});

test('a set with a hole is refused with the files that fill it', async (t) => {
  const tm = await engine();
  if (!tm) return t.skip('set TEIMERIS_WASM_PACKAGE to the engine\'s wasm package');
  // 600 to 1199 beside the built-in 1800 to 2399 leaves 1200 to 1799 out,
  // and a planet file without its Moon.
  await assert.rejects(
    teimerisWeb(tm, { files: [{ name: 'sepl_06.se1', bytes: new Uint8Array(1) }], verify: false }),
    (error) =>
      /hole/u.test(error.message) &&
      error.message.includes('semo_06.se1') &&
      error.message.includes('sepl_12.se1') &&
      error.message.includes('semo_12.se1'),
  );
});

test('an added range answers where the built-in years end', async (t) => {
  const tm = await engine();
  if (!tm || !DATA) return t.skip('set TEIMERIS_WASM_PACKAGE and TEIMERIS_DATA_DIR to a Teimeris checkout\'s');
  const jd1500 = 2268923.5;
  const builtIn = await teimerisWeb(tm);
  assert.ok(jd1500 < builtIn.jdMin, 'the built-in years do not reach 1500');
  const ranged = await teimerisWeb(tm, { files: [file('sepl_12.se1'), file('semo_12.se1')] });
  const block = BLOCKS.find((b) => b.century === 12);
  assert.equal(ranged.jdMin, block.start);
  assert.equal(ranged.jdMax, builtIn.jdMax);
  assert.equal(ranged.dataVersion, 'seas_18.se1 semo_12.se1 semo_18.se1 sepl_12.se1 sepl_18.se1');
  const answer = ranged.positions({ ...sun([jd1500]), bodies: ['SUN', 'MOON'] });
  assert.deepEqual(Array.from(answer.status), [0, 0]);
  assert.deepEqual(Array.from(answer.source), [1, 1]);
  // The same instant as the native adapter reads it from the same files
  // (`an_added_range_answers_as_the_directory_does`): the Sun near 289°.
  assert.ok(answer.lon[0] > 280 && answer.lon[0] < 300, `${answer.lon[0]}`);
});
