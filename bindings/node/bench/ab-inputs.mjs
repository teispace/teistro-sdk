// What both sides of the A/B (`docs/03-design/ab-baseline.md`) are given,
// and how a side is opened in a fresh process or worker thread. Nothing
// here imports the SDK, so a process that opens the baseline alone pays
// for the baseline alone (§6).

import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

/**
 * One golden input, as the runner hands it to both sides: the birth, and
 * what a request needs beyond it (§1), chosen here so that both sides are
 * given the same.
 *
 * @typedef {object} Birth
 * @property {string} id the corpus's chart id
 * @property {number} jdUt the birth's instant, a UT Julian day
 * @property {string} utc the same instant as an ISO-8601 UTC string
 * @property {string} iana the birth's time zone, as the corpus resolved it
 * @property {{latitude: number, longitude: number, altitude: number}} place
 * @property {number} utcOffsetSeconds the civil clock's offset at the birth
 * @property {{year: number, month: number, day: number}} civilDate the birth's civil date
 *
 * @typedef {Birth & {
 *   partner: Birth,
 *   instants: number[],
 *   transitAt: number,
 *   window: {from: number, to: number},
 * }} Input
 */

/**
 * The sky a side casts a request in, as `ab-swisseph.py` reads it (§11).
 *
 * @typedef {{ayanamsha: string, basis: 'true' | 'mean', centre: 'geocentric' | 'topocentric',
 *   node: 'mean' | 'true', houses: 'W' | 'P'}} Sky
 */

/**
 * What a side answers: `version`, `sun`, one function a request of
 * {@link REQUESTS}, each given one {@link Input}, `facts`, an answer read
 * into the vocabulary of `ab-agree.mjs` (§10), and `conventions`, the
 * skies its `chart`, `kp` and `gochar` are cast in. The adapter exports
 * these; the SDK's side is `sdkSide` in `ab.mjs`.
 *
 * @typedef {{version: string, sun: (answer: unknown) => number,
 *   facts: (request: string, answer: any, input: Input) => [string, unknown][],
 *   conventions: {chart: Sky, kp: Sky, gochar: Sky}}
 *   & Record<string, any>} Side
 */

/** The two sides, in the order every list here holds them. */
export const SIDES = ['sdk', 'baseline'];

/** The requests both sides answer, in the order they are timed (§2). */
export const REQUESTS = [
  'chart',
  'day',
  'full',
  'vargas',
  'yogas',
  'strength',
  'dashas',
  'month',
  'milan',
  'varsha',
  'kp',
  'gochar',
  'sadeSati',
  'batch',
];

/** The days `month` asks for, from the birth's civil date. */
export const MONTH_DAYS = 30;

/** The charts `batch` asks for, an hour apart from the birth. */
export const BATCH_CHARTS = 10;

/** How far on `gochar`'s instant and the end of `sadeSati`'s window are, in Julian years. */
export const TRANSIT_YEARS = 30;

const JULIAN_YEAR = 365.25;

const root = join(fileURLToPath(new URL('.', import.meta.url)), '../../..');

/** The golden corpus's inputs, in id order, each with what its requests need. */
export function inputs(dir = join(root, 'fixtures/baseline/charts')) {
  const births = readdirSync(dir)
    .filter((name) => name.endsWith('.json'))
    .sort()
    .map((name) => {
      const { id, input } = JSON.parse(readFileSync(join(dir, name), 'utf8'));
      const [year, month, day] = input.local.date.split('-').map(Number);
      return {
        id,
        jdUt: input.resolved.jd_ut,
        utc: input.resolved.utc,
        iana: input.resolved.iana,
        place: {
          latitude: input.place.latitude,
          longitude: input.place.longitude,
          altitude: input.place.altitude_m,
        },
        utcOffsetSeconds: input.resolved.tz_offset_min * 60,
        civilDate: { year, month, day },
      };
    });
  return births.map((birth, at) => ({
    ...birth,
    partner: births[(at + 1) % births.length],
    instants: Array.from({ length: BATCH_CHARTS }, (_, hour) => birth.jdUt + hour / 24),
    transitAt: birth.jdUt + TRANSIT_YEARS * JULIAN_YEAR,
    window: { from: birth.jdUt, to: birth.jdUt + TRANSIT_YEARS * JULIAN_YEAR },
  }));
}

/** The paths both sides are opened from, refused with a sentence naming the variables. */
export function paths() {
  const adapter = process.env.TEISTRO_AB_BASELINE;
  const plugin = process.env.TEISTRO_TEIMERIS_ADAPTER;
  if (!adapter || !plugin) {
    throw new Error(
      'set TEISTRO_AB_BASELINE to the baseline adapter and TEISTRO_TEIMERIS_ADAPTER to the Teimeris library',
    );
  }
  return { adapter, plugin };
}

/** The SDK's version, as its package names it. */
export function sdkVersion() {
  return `teistro ${JSON.parse(readFileSync(join(root, 'bindings/node/package.json'), 'utf8')).version}`;
}

/**
 * One side, opened in this process or thread: the SDK over `ephemeris`,
 * Teimeris unless told otherwise, or the adapter. Only that side's code
 * is imported.
 *
 * @param {string} name one of {@link SIDES}
 * @returns {Promise<{side: Side, dispose: () => void}>}
 */
export async function open(name, ephemeris) {
  if (name === 'baseline') {
    const side = await import(pathToFileURL(paths().adapter).href);
    return { side, dispose: () => {} };
  }
  const { openSdk } = await import('./ab.mjs');
  return openSdk(ephemeris ?? { plugin: paths().plugin }, sdkVersion());
}
