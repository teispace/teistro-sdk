// The build handshake: the two halves of the binding must be one build,
// and the loader refuses one that is not.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { ABI_VERSION, SDK_VERSION } from '../lib/catalogue.js';
import { buildInfo, refuseBuild } from '../lib/index.js';

/** A build as the addon reports one, with a field moved for the test. */
const info = (over = {}) => ({
  sdk: SDK_VERSION,
  abi: ABI_VERSION,
  catalogue: 1,
  commit: 'a2a00beb59060011360f7c116d27d4d4fada69a1',
  dirty: false,
  profile: 'release',
  target: 'aarch64-apple-darwin',
  debug_assertions: false,
  optimised: true,
  sanitizer: '',
  rustc: 'rustc 1.98.0',
  ...over,
});

test('the loaded addon describes the build it came from', () => {
  assert.equal(buildInfo.sdk, SDK_VERSION);
  assert.equal(buildInfo.abi, ABI_VERSION);
  assert.equal(buildInfo.catalogue, 1);
  assert.match(buildInfo.target, /-/u);
  assert.match(buildInfo.rustc, /^rustc/u);
  assert.match(buildInfo.commit, /^([0-9a-f]{40}|unknown)$/u);
  assert.equal(Object.isFrozen(buildInfo), true);
});

test('a build that is the one these types came from is taken', () => {
  assert.equal(refuseBuild(info(), false), null);
  assert.equal(refuseBuild(info(), true), null);
});

test('a build of another ABI or another version is refused', () => {
  assert.match(refuseBuild(info({ abi: 99 }), true), /ABI 99/u);
  assert.match(refuseBuild(info({ sdk: '9.9.9' }), true), /Teistro 9\.9\.9/u);
});

test('a sanitizer build is refused however it was found', () => {
  for (const named of [true, false]) {
    assert.match(refuseBuild(info({ sanitizer: 'address' }), named), /address sanitizer build/u);
  }
});

test('an unoptimised build is refused only when it was searched for', () => {
  const debug = info({ optimised: false, profile: 'debug' });
  assert.match(refuseBuild(debug, false), /unoptimised build/u);
  assert.match(refuseBuild(debug, false), /TEISTRO_ADDON/u);
  assert.equal(refuseBuild(debug, true), null, 'a path named is a deliberate act');
});
