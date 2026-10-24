// The Node row of `docs/03-design/ffi-cost.md`: the scenario every binding
// times, printed as one JSON line a key. `cargo xtask ffi-cost --measure`
// runs it after building the addon in release.
//
// `call-*` asks for 1 instant and 3 bodies, `grid-*` for 366 instants and
// 10 bodies. `*-native` is a context over the library's test provider and
// `*-hosted` one over a provider written here, answering fixed columns.
// Each call reads the answer's grid size, so the lazy decode is paid as a
// consumer pays it. `decode` is the decoder alone, on the grid's bytes.

import { performance } from 'node:perf_hooks';

import { Body, Context, decodePositions } from '../lib/index.js';

const RUN_MS = 50;
const RUNS = 5;

const BODIES = [
  Body.Sun,
  Body.Moon,
  Body.Mercury,
  Body.Venus,
  Body.Mars,
  Body.Jupiter,
  Body.Saturn,
  Body.MeanNode,
  Body.Uranus,
  Body.Neptune,
];

/** A provider written in JavaScript: every cell a fixed longitude. */
const provider = {
  name: 'ffi-cost-in-javascript',
  bodies: BODIES,
  version: '1',
  positions(request) {
    const cells = request.jds.length * request.bodies.length;
    const lon = new Float64Array(cells).fill(123.25);
    const lonSpeed = new Float64Array(cells).fill(1);
    return { frameBits: request.frameBits, lon, lonSpeed, status: new Int32Array(cells) };
  },
};

function time(key, call) {
  for (let i = 0; i < 100; i += 1) call();
  let calls = 1;
  for (;;) {
    const start = performance.now();
    for (let i = 0; i < calls; i += 1) call();
    if (performance.now() - start >= RUN_MS || calls >= 1 << 24) break;
    calls *= 2;
  }
  const runs = [];
  for (let r = 0; r < RUNS; r += 1) {
    const start = performance.now();
    for (let i = 0; i < calls; i += 1) call();
    runs.push(((performance.now() - start) * 1000) / calls);
  }
  runs.sort((a, b) => a - b);
  console.log(
    JSON.stringify({
      binding: 'node',
      key,
      runs: RUNS,
      calls,
      median_us: runs[RUNS >> 1],
      min_us: runs[0],
      max_us: runs[RUNS - 1],
    }),
  );
}

const native = new Context({ profile: 'nepali-default', testProvider: true });
const hosted = new Context({ profile: 'nepali-default', provider });
try {
  const call = { instants: [2451545], bodies: [Body.Sun, Body.Moon, Body.Mars] };
  const grid = {
    instants: Array.from({ length: 366 }, (_, day) => 2451545 + day),
    bodies: BODIES,
  };
  // The hosted provider's answer is what it sent, so a figure is never
  // taken over a call that failed or answered something else.
  const answered = hosted.positions(grid);
  if (answered.jdCount * answered.bodyCount !== 3660 || answered.decoded.cells.lon[3659] !== 123.25) {
    throw new Error('the hosted provider did not answer the grid it was asked');
  }
  let sink = 0;
  time('call-native', () => {
    sink += native.positions(call).jdCount;
  });
  time('call-hosted', () => {
    sink += hosted.positions(call).jdCount;
  });
  time('grid-native', () => {
    sink += native.positions(grid).jdCount;
  });
  time('grid-hosted', () => {
    sink += hosted.positions(grid).jdCount;
  });
  const bytes = native.positions(grid).bytes;
  time('decode', () => {
    sink += decodePositions(bytes).jdCount;
  });
  if (sink === 0) throw new Error('nothing was answered');
} finally {
  native.dispose();
  hosted.dispose();
}
