// The branded quantities' constructors against the shared validator
// corpus (`docs/03-design/validator-corpus.md`): every value
// `idl/validators.json` lists as valid makes a quantity, and every
// invalid one throws, as the Rust newtypes and every other binding agree.
// The corpus is found by walking up from this file, so the wasm package's
// staged copy of the suite reads the same one.

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { altitude, latitude, longitude } from '../lib/catalogue.js';

const CONSTRUCTORS = { latitude, longitude, altitude };

function corpus() {
  for (let dir = dirname(fileURLToPath(import.meta.url)); ; dir = dirname(dir)) {
    const path = join(dir, 'idl', 'validators.json');
    if (existsSync(path)) {
      return JSON.parse(readFileSync(path, 'utf8'));
    }
    assert.notEqual(dirname(dir), dir, 'no idl/validators.json above this test');
  }
}

test('every branded constructor agrees with the shared corpus', () => {
  const { schema, brands } = corpus();
  assert.equal(schema, 'teistro-validators/1');
  assert.ok(brands.length > 0, 'the corpus names no brand');
  for (const { brand, valid, invalid } of brands) {
    const make = CONSTRUCTORS[brand];
    assert.ok(make, `${brand} is branded in the description and has no constructor here`);
    for (const text of valid) {
      assert.equal(make(Number(text)), Number(text), `${brand}(${text}) should be accepted`);
    }
    for (const text of invalid) {
      assert.throws(() => make(Number(text)), `${brand}(${text}) should be refused`);
    }
  }
});
