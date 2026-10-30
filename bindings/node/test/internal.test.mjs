// The SDK failing inside itself (SR-52): a wasm module's trap reported as
// the `INTERNAL` status a panic at the C ABI is, and a host's crash
// reporter hearing of either without the message.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { INTERNAL_CODE, isTrap, onInternalError, reportInternal, trapRecord } from '../lib/internal.js';
import * as sdk from '../lib/index.js';

/** `(module (func (export "trap") unreachable))`: a real trap. */
const trapping = new WebAssembly.Instance(
  new WebAssembly.Module(
    Uint8Array.from([
      0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02,
      0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x74, 0x72, 0x61, 0x70, 0x00, 0x00, 0x0a, 0x05, 0x01, 0x03,
      0x00, 0x00, 0x0b,
    ]),
  ),
);

test('a wasm trap is told from every other failure', () => {
  let caught;
  try {
    trapping.exports.trap();
  } catch (cause) {
    caught = cause;
  }
  assert.ok(isTrap(caught), String(caught));
  assert.equal(isTrap(new Error('unreachable')), false);
  assert.equal(isTrap(undefined), false);
});

test('a trap is refused as INTERNAL, then and on every call after it', () => {
  for (const first of [true, false]) {
    const record = trapRecord(first);
    assert.equal(record.status, 'INTERNAL');
    assert.equal(record.code, INTERNAL_CODE);
    assert.equal(record.code, -10);
    assert.match(record.message, /load (the module|it) again/u);
    const error = new sdk.TeistroError(record);
    assert.equal(error.status, 'INTERNAL');
  }
  assert.notEqual(trapRecord(true).message, trapRecord(false).message);
});

test('a hook hears the status, code, kind and version, never the message', () => {
  const heard = [];
  const stop = onInternalError((report) => heard.push(report));
  const thrower = onInternalError(() => {
    throw new Error('a broken reporter');
  });
  reportInternal('trap', '1.2.3');
  reportInternal('boundary', '1.2.3');
  stop();
  thrower();
  reportInternal('trap', '1.2.3');
  assert.deepEqual(heard, [
    { status: 'INTERNAL', code: -10, kind: 'trap', sdkVersion: '1.2.3' },
    { status: 'INTERNAL', code: -10, kind: 'boundary', sdkVersion: '1.2.3' },
  ]);
  assert.ok(Object.isFrozen(heard[0]));
  assert.equal('message' in heard[0], false);
  assert.throws(() => onInternalError('not a function'), TypeError);
});

test('the package exports the hook and the code', () => {
  assert.equal(sdk.onInternalError, onInternalError);
  assert.equal(sdk.INTERNAL_CODE, -10);
});
