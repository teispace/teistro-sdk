// The binding gives back the native memory it takes
// (`docs/05-testing/01-quality-bar.md`, "leak checks per binding"), read as
// the Python, Java and Dart tests read it: the C allocator's own count of
// the bytes in use, `malloc_zone_statistics` on macOS and glibc's
// `mallinfo2` on Linux. The resident set will not do, since macOS
// compresses idle pages and a process keeping its contexts shrinks.
//
// Node has no foreign-function interface of its own, so the count is read
// through koffi, pinned in `../leaks/` and installed there by
// `cargo xtask check-node`; the package itself does not depend on it. Where
// it is not installed, or the allocator cannot be read faithfully (Windows,
// musl, Intel macOS: the Python test says why), the tests skip.
//
// Each family runs its cycles once to settle, then `RUNS` times measured,
// and at least one measured run must leave the bytes in use within `BUDGET`
// of where it began: a leak grows them in every run, while what V8 allocates
// for itself moves them a little, and now and then once.
//
// `node --test` runs each file in a process of its own, so no other file's
// allocations reach these counts.

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { test } from 'node:test';
import { setFlagsFromString } from 'node:v8';
import { runInNewContext } from 'node:vm';

import { Body, buildInfo, Context } from '../lib/index.js';

/** How far the bytes in use may move over a measured run. */
const BUDGET = 64 * 1024;
/** The cycles of one run. */
const CYCLES = 500;
/** The measured runs of each family. */
const RUNS = 3;

/** koffi, from where the gate installed it, or `null`. */
function koffi() {
  try {
    return createRequire(new URL('../leaks/package.json', import.meta.url))('koffi');
  } catch {
    return null;
  }
}

/** How to read the bytes in use, where the allocator says, or `null`. */
function reader() {
  // A wasm build allocates in its own memory, and a sanitizer's runtime
  // owns `malloc` and keeps freed blocks back on purpose.
  if (buildInfo.target.startsWith('wasm32') || buildInfo.sanitizer) return null;
  const ffi = koffi();
  if (!ffi) return null;
  if (process.platform === 'darwin' && process.arch === 'arm64') {
    const stats = ffi.struct('malloc_statistics_t', {
      blocks_in_use: 'uint',
      size_in_use: 'size_t',
      max_size_in_use: 'size_t',
      size_allocated: 'size_t',
    });
    const statistics = ffi
      .load('/usr/lib/libSystem.B.dylib')
      .func('malloc_zone_statistics', 'void', ['void *', ffi.out(ffi.pointer(stats))]);
    return () => {
      const out = {};
      statistics(null, out);
      return out.size_in_use;
    };
  }
  const glibc = process.platform === 'linux' && process.report?.getReport?.().header.glibcVersionRuntime;
  if (glibc) {
    const fields = ['arena', 'ordblks', 'smblks', 'hblks', 'hblkhd', 'usmblks', 'fsmblks', 'uordblks', 'fordblks', 'keepcost'];
    const info = ffi.struct('mallinfo2', Object.fromEntries(fields.map((field) => [field, 'size_t'])));
    let mallinfo2;
    try {
      mallinfo2 = ffi.load('libc.so.6').func('mallinfo2', info, []);
    } catch {
      // A glibc before 2.33 has no `mallinfo2`.
      return null;
    }
    return () => {
      const read = mallinfo2();
      // The small blocks in use and the large ones mapped on their own.
      return Number(read.uordblks) + Number(read.hblkhd);
    };
  }
  return null;
}

const inUse = reader();
const skip = inUse ? false : "the allocator's bytes in use cannot be read here (needs ../leaks installed)";

/** A full collection: what a context dropped unclosed is freed by. */
const collect = (() => {
  setFlagsFromString('--expose-gc');
  return runInNewContext('gc');
})();

/** How little the bytes in use may move between two reads to count as settled. */
const QUIET = 4 * 1024;
/** The most collections one settling waits through. */
const SETTLE_PASSES = 100;

/**
 * Collects until the finalisers have freed what they held: until two reads
 * a collection apart agree to `QUIET`. Not a fixed number of passes, which
 * Java's test found too few on a loaded runner.
 */
async function settle() {
  let last = inUse();
  let quiet = 0;
  for (let pass = 0; pass < SETTLE_PASSES && quiet < 2; pass += 1) {
    collect();
    await new Promise((resolve) => setTimeout(resolve, 5));
    const now = inUse();
    quiet = Math.abs(now - last) <= QUIET ? quiet + 1 : 0;
    last = now;
  }
}

/**
 * At least one of `RUNS` runs of `cycle` grows the bytes in use within
 * `BUDGET`. A failure names every run's growth and one run's over four
 * times the cycles: a leak grows with the cycles, and what the allocator
 * keeps for itself does not.
 */
async function givesBack(cycle) {
  const run = async (count) => {
    await settle();
    const before = inUse();
    for (let at = 0; at < count; at += 1) cycle();
    await settle();
    return inUse() - before;
  };
  await run(CYCLES);
  const grown = [];
  for (let at = 0; at < RUNS; at += 1) grown.push(await run(CYCLES));
  const least = Math.min(...grown);
  if (least <= BUDGET) return;
  const longer = await run(4 * CYCLES);
  assert.fail(
    `the bytes in use grew ${grown} over ${RUNS} runs of ${CYCLES} cycles ` +
      `(${Math.floor(least / CYCLES)} a cycle at least), and ${longer} over ${4 * CYCLES}; ` +
      `the budget is ${BUDGET}`,
  );
}

test('a context disposed gives back its native memory', { skip }, async () => {
  await givesBack(() => {
    const ctx = new Context({ testProvider: true, profile: 'nepali-default' });
    assert.ok(ctx.profile);
    ctx.dispose();
  });
});

test('a context dropped undisposed is given back by its finaliser', { skip }, async () => {
  await givesBack(() => {
    const ctx = new Context({ testProvider: true, profile: 'nepali-default' });
    assert.ok(ctx.profile);
  });
});

test('charts and positions give back their answers', { skip }, async () => {
  const ctx = new Context({ testProvider: true, profile: 'nepali-default', locale: 'en-Latn' });
  try {
    await givesBack(() => {
      ctx.chart.found({
        instant: 2451545,
        place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
        utcOffsetSeconds: 20700,
      });
      ctx.positions({ instants: [2451545, 2451546], bodies: [Body.Sun, Body.Moon] });
    });
  } finally {
    ctx.dispose();
  }
});
