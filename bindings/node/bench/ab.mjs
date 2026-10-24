// The interleaved A/B of `docs/03-design/ab-baseline.md`: the SDK through
// Node against the baseline engine, on the golden corpus's inputs, printed
// as one JSON line a side and request. `cargo xtask measure-ab` runs it.
//
//   TEISTRO_AB_BASELINE=/path/to/adapter.mjs \
//   TEISTRO_TEIMERIS_ADAPTER=/path/to/libteistro_ephemeris_teimeris.dylib \
//   node bench/ab.mjs
//
// The adapter is a module kept outside the SDK, beside the baseline
// engine's own exporter. Its contract is `Side` below: the SDK side here
// is written to the same contract, so the runner treats the two alike.

import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { performance } from 'node:perf_hooks';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { Calendar, Context, DashaSystem, date } from '../lib/index.js';

/**
 * One golden input, as the runner hands it to both sides.
 *
 * @typedef {object} Input
 * @property {string} id the corpus's chart id
 * @property {number} jdUt the birth's instant, a UT Julian day
 * @property {{latitude: number, longitude: number, altitude: number}} place
 * @property {number} utcOffsetSeconds the civil clock's offset at the birth
 * @property {{year: number, month: number, day: number}} civilDate the birth's civil date
 */

/**
 * What a side answers. The adapter exports these four; the SDK's side is
 * {@link sdkSide}.
 *
 * @typedef {object} Side
 * @property {string} version what the side is, recorded as it is
 * @property {(input: Input) => unknown} chart a birth chart with its Vimshottari to depth 3
 * @property {(input: Input) => unknown} day the almanac day of the birth's date at its place
 * @property {(answer: unknown) => number} sun the Sun's sidereal longitude, degrees, from the side's own chart
 */

/** The fewest rounds a figure is taken from. */
export const ROUNDS = 15;

/**
 * How far apart the two sides' Suns may be and still be the same work:
 * an arcminute. A different instant or place moves the Sun by more; a
 * different ayanamsha model by less, and that is the golden vectors'
 * concern, not a timing's.
 */
export const SAME_SKY_DEG = 1 / 60;

const root = join(fileURLToPath(new URL('.', import.meta.url)), '../../..');

/** The golden corpus's inputs, in id order. */
export function inputs(dir = join(root, 'fixtures/baseline/charts')) {
  return readdirSync(dir)
    .filter((name) => name.endsWith('.json'))
    .sort()
    .map((name) => {
      const { id, input } = JSON.parse(readFileSync(join(dir, name), 'utf8'));
      const [year, month, day] = input.local.date.split('-').map(Number);
      return {
        id,
        jdUt: input.resolved.jd_ut,
        place: {
          latitude: input.place.latitude,
          longitude: input.place.longitude,
          altitude: input.place.altitude_m,
        },
        utcOffsetSeconds: input.resolved.tz_offset_min * 60,
        civilDate: { year, month, day },
      };
    });
}

/**
 * The SDK's side over a context the caller opened and disposes.
 *
 * @param {Context} ctx
 * @param {string} version
 * @returns {Side}
 */
export function sdkSide(ctx, version) {
  return {
    version,
    chart: (input) =>
      ctx.chart.found({
        instant: input.jdUt,
        place: input.place,
        utcOffsetSeconds: input.utcOffsetSeconds,
        dashas: [DashaSystem.Vimshottari],
      }),
    day: (input) =>
      ctx.almanac.day({
        date: date(Calendar.Gregorian, input.civilDate.year, input.civilDate.month, input.civilDate.day),
        place: input.place,
        utcOffsetSeconds: input.utcOffsetSeconds,
      }),
    sun: (answer) => answer.grahas.find((placed) => placed.graha === 'graha.SUN').longitudeDeg,
  };
}

/**
 * The inputs on which the two sides computed the same sky, and the ones
 * they did not, each with both Suns.
 */
export function agreeing(a, b, all) {
  const kept = [];
  const differ = [];
  for (const input of all) {
    const sa = a.sun(a.chart(input));
    const sb = b.sun(b.chart(input));
    const apart = Math.abs(((sa - sb + 540) % 360) - 180);
    if (apart <= SAME_SKY_DEG) kept.push(input);
    else differ.push({ id: input.id, a: sa, b: sb });
  }
  return { kept, differ };
}

/**
 * Both sides over every input, `rounds` times, alternating which goes
 * first, for each request. A round's figure is its time over the inputs,
 * per input, in microseconds.
 *
 * @returns {{side: string, request: string, rounds: number, inputs: number,
 *   median_us: number, min_us: number, max_us: number}[]}
 */
export function interleaved(sides, all, rounds = ROUNDS) {
  const out = [];
  for (const request of ['chart', 'day']) {
    const runs = sides.map(() => []);
    // Warm each side's paths before anything is timed.
    for (const side of sides) for (const input of all) side[request](input);
    for (let round = 0; round < rounds; round += 1) {
      const order = round % 2 === 0 ? [0, 1] : [1, 0];
      for (const at of order) {
        const side = sides[at];
        const start = performance.now();
        for (const input of all) side[request](input);
        runs[at].push(((performance.now() - start) * 1000) / all.length);
      }
    }
    sides.forEach((side, at) => {
      const sorted = [...runs[at]].sort((x, y) => x - y);
      out.push({
        side: at === 0 ? 'sdk' : 'baseline',
        version: side.version,
        request,
        rounds,
        inputs: all.length,
        median_us: sorted[sorted.length >> 1],
        min_us: sorted[0],
        max_us: sorted[sorted.length - 1],
      });
    });
  }
  return out;
}

async function main() {
  const adapter = process.env.TEISTRO_AB_BASELINE;
  const plugin = process.env.TEISTRO_TEIMERIS_ADAPTER;
  if (!adapter || !plugin) {
    throw new Error(
      'set TEISTRO_AB_BASELINE to the baseline adapter and TEISTRO_TEIMERIS_ADAPTER to the Teimeris library',
    );
  }
  const baseline = await import(pathToFileURL(adapter).href);
  const { version } = JSON.parse(readFileSync(join(root, 'bindings/node/package.json'), 'utf8'));
  const ctx = new Context({ profile: 'parashari-classical', ephemeris: { plugin } });
  try {
    const sdk = sdkSide(ctx, `teistro ${version}`);
    const all = inputs();
    const { kept, differ } = agreeing(sdk, baseline, all);
    for (const d of differ) {
      console.log(JSON.stringify({ differs: d.id, sdk: d.a, baseline: d.b }));
    }
    for (const line of interleaved([sdk, baseline], kept)) {
      console.log(JSON.stringify({ ...line, corpus: all.length }));
    }
  } finally {
    ctx.dispose();
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await main();
