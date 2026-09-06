// The generated decoders against blobs the library really produced.
//
// `cargo xtask check-node` writes the fixtures with
// `cargo run -p teistro-ffi --example blob_fixtures` and runs this file;
// it takes the fixture directory as its one argument. Node's own test
// runner and assertions only, so the binding's tests need no install.

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { decodePositions, decodeIntlRender } from '../lib/blob.js';
import { Body, Graha, Kind, Status, TimeScale } from '../lib/catalogue.js';

const dir = process.argv[2] ?? 'target/tsrb';
const read = (name) => new Uint8Array(readFileSync(join(dir, name)));

test('a positions blob decodes into views over its own bytes', () => {
  const bytes = read('positions.tsrb');
  const positions = decodePositions(bytes);

  // The summary says what grid the cells cover.
  assert.equal(positions.jdCount, 2);
  assert.equal(positions.bodyCount, 3);
  assert.equal(positions.scale, 0, 'UT1, the scale the request named');
  assert.ok(Number.isInteger(positions.frameBits));

  // The instants and bodies come back in the order they were asked for.
  assert.deepEqual(Array.from(positions.instants.jd), [2451545, 2451546]);
  assert.deepEqual(Array.from(positions.bodies.body), [0, 1, 4], 'the Sun, the Moon, Mars');
  assert.equal(positions.instants.length, 2);

  // One row per cell, instants outermost.
  const cells = positions.cells;
  assert.equal(cells.length, 6);
  assert.ok(cells.lon instanceof Float64Array);
  assert.ok(cells.status instanceof Int32Array);
  assert.ok(cells.source instanceof Uint32Array);
  for (let i = 0; i < cells.length; i += 1) {
    assert.equal(cells.status[i], 0, `cell ${i} has a value`);
    assert.ok(cells.lon[i] >= 0 && cells.lon[i] < 360, `cell ${i} longitude in range`);
  }
  // The Moon moves faster than the Sun, and everything moves.
  assert.ok(Math.abs(cells.lonSpeed[1]) > Math.abs(cells.lonSpeed[0]));

  // The columns are views: writing through one writes into the blob.
  const before = cells.lon[0];
  cells.lon[0] = -1;
  assert.equal(decodePositions(bytes).cells.lon[0], -1, 'the column shares the buffer');
  cells.lon[0] = before;

  // The steps and the provenance are the JSON the library wrote.
  const steps = JSON.parse(positions.steps);
  assert.ok(Array.isArray(steps) && steps.length > 0);
  assert.ok(steps.every((s) => typeof s.name === 'string' && typeof s.implementation === 'string'));
  const provenance = JSON.parse(positions.provenance);
  assert.equal(provenance.profile, 'nepali-default');
  assert.equal(provenance.calculation_version, 1);
  assert.equal(provenance.settings_hash.length, 64);
  assert.equal(provenance.provider.frame, 'GEOCENTRIC/OF_DATE/ECLIPTIC/TROPICAL/APPARENT');
  assert.equal(provenance.time.delta_t_model, 'TABLE_THEN_MODEL');
});

test('a render blob decodes its text, its locale and its warnings', () => {
  const rendered = decodeIntlRender(read('intl_render.tsrb'));
  assert.equal(rendered.resolvedFrom, 'ne-Deva-NP');
  assert.equal(rendered.isFallback, 0);
  assert.equal(rendered.isOverride, 0);
  assert.equal(rendered.warningCount, 0);
  assert.deepEqual(JSON.parse(rendered.warnings), []);
  assert.match(rendered.text, /७/u, 'the Nepali numeral seven');
  assert.ok(rendered.text.includes('गुरु'), 'Jupiter by its Nepali name');
});

test('a blob of the wrong shape is refused, never misread', () => {
  const bytes = read('positions.tsrb');
  assert.throws(() => decodePositions(new Uint8Array(4)), /not a Teistro result blob/);
  assert.throws(() => decodePositions(Uint8Array.from(bytes).fill(0, 4, 8)), /layout version 0/);
  assert.throws(() => decodeIntlRender(bytes), /blob schema 1, expected 2/);
  assert.throws(() => decodePositions([1, 2, 3]), /expected a Uint8Array/);
  const truncated = bytes.slice(0, bytes.length - 8);
  assert.throws(() => decodePositions(truncated), /the header says/);
});

test('a decoder reads a blob that does not start on an eight-byte boundary', () => {
  const bytes = read('positions.tsrb');
  const shifted = new Uint8Array(bytes.length + 1);
  shifted.set(bytes, 1);
  const view = shifted.subarray(1);
  assert.equal(view.byteOffset % 8, 1);
  assert.equal(decodePositions(view).jdCount, 2, 'the odd offset is copied once, not misread');
});

test('the catalogue tables are the keys every pack and fixture carries', () => {
  assert.equal(Graha.Sun, 'graha.SUN');
  assert.equal(Graha.Ketu, 'graha.KETU');
  assert.equal(Kind.Nakshatra, 'nakshatra', "a kind names itself as a key's first segment does");
  assert.equal(Kind.AvasthaBaladi, 'avastha_baladi');
  assert.equal(Status.InvalidArg, 'invalid-arg');
  assert.equal(Body.MeanNode, 'mean-node');
  assert.equal(TimeScale.Ut1, 'ut1');
  assert.equal(Object.isFrozen(Graha), true, 'the tables cannot be edited');
  const values = new Set(Object.values(Graha));
  assert.equal(values.size, Object.keys(Graha).length, 'no two members share a value');
});
