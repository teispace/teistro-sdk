/**
 * The one JavaScript timing harness of spike 2, shared by the option A and
 * option B benchmarks so their rows are measured the same way.
 *
 * A row is the median of `iterations` timed calls after a warm-up, in
 * microseconds, with the 90th percentile beside it.
 */

import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

/**
 * Times `fn` over `rounds` independent rounds and keeps the round with the
 * lowest median, so a garbage-collection pause or a late optimisation in
 * one round does not stand for the cost of the call.
 */
export function bench(name, fn, { iterations = 2000, warmup = 200, rounds = 3 } = {}) {
  let best = null;
  for (let round = 0; round < rounds; round += 1) {
    for (let i = 0; i < warmup; i += 1) fn();
    const times = new Float64Array(iterations);
    for (let i = 0; i < iterations; i += 1) {
      const start = performance.now();
      fn();
      times[i] = performance.now() - start;
    }
    times.sort();
    const at = (q) => times[Math.min(iterations - 1, Math.floor(q * iterations))] * 1000;
    const row = { name, median_us: at(0.5), p90_us: at(0.9), iterations, rounds };
    if (best === null || row.median_us < best.median_us) best = row;
  }
  return best;
}

/** A row the binding cannot express, with the reason. */
export function unavailable(name, note) {
  return { name, median_us: null, p90_us: null, iterations: 0, note };
}

/** A row derived from two others: `(whole − base) / divisor`. */
export function derived(name, whole, base, divisor) {
  return {
    name,
    median_us: (whole.median_us - base.median_us) / divisor,
    p90_us: (whole.p90_us - base.p90_us) / divisor,
    iterations: whole.iterations,
  };
}

/** Prints the rows as a Markdown table. */
export function report(rows) {
  const value = (v) => (v === null ? 'n/a' : v.toFixed(2));
  console.log('| measurement | median µs | p90 µs |');
  console.log('|---|---:|---:|');
  for (const row of rows) console.log(`| ${row.name} | ${value(row.median_us)} | ${value(row.p90_us)} |`);
}

/** Writes the rows and their context to `<dir>/<file>` as JSON. */
export function writeResults(dir, file, payload) {
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, file), `${JSON.stringify({ node: process.version, arch: process.arch, ...payload }, null, 2)}\n`);
}
