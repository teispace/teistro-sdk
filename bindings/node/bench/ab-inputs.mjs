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
 *   muhurta: Muhurta,
 * }} Input
 *
 * The window and place a marriage muhurta is searched over: `from` and
 * `to`, civil dates (`YYYY-MM-DD`, both included) on `iana`'s clock, whose
 * offset at `from` is `utcOffsetSeconds`.
 *
 * @typedef {{from: string, to: string, iana: string, utcOffsetSeconds: number,
 *   place: {latitude: number, longitude: number, altitude: number}}} Muhurta
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
  'muhurta',
];

/** The days `month` asks for, from the birth's civil date. */
export const MONTH_DAYS = 30;

/** The charts `batch` asks for, an hour apart from the birth. */
export const BATCH_CHARTS = 10;

/** How far on `gochar`'s instant and the end of `sadeSati`'s window are, in Julian years. */
export const TRANSIT_YEARS = 30;

/**
 * The days `muhurta` searches, from the birth's civil date
 * {@link TRANSIT_YEARS} years on: within the baseline's 25 days with
 * windows, so that cap never decides which day is ranked.
 */
export const MUHURTA_DAYS = 25;

/**
 * Where `muhurta` searches: Kathmandu, where the baseline reads a planet's
 * setting (C164), so both sides' asta is read at the place searched.
 */
export const MUHURTA_PLACE = Object.freeze({ latitude: 27.7172, longitude: 85.324, altitude: 1400 });
const MUHURTA_ZONE = 'Asia/Kathmandu';

const JULIAN_YEAR = 365.25;

/** A UTC midnight's civil date, `YYYY-MM-DD`. */
const isoDate = (ms) => new Date(ms).toISOString().slice(0, 10);

/** `zone`'s offset from UTC at `ms`, in seconds, as the zone database holds it. */
function offsetSeconds(zone, ms) {
  const name = new Intl.DateTimeFormat('en-US', { timeZone: zone, timeZoneName: 'longOffset' })
    .formatToParts(new Date(ms))
    .find((part) => part.type === 'timeZoneName').value;
  const [, sign, hours, minutes = '0'] = /^GMT(?:([+-])(\d{2}):?(\d{2})?)?$/.exec(name) ?? [];
  return sign ? (sign === '-' ? -1 : 1) * (Number(hours) * 3600 + Number(minutes) * 60) : 0;
}

/** The marriage muhurta's window for a birth: {@link MUHURTA_DAYS} days, {@link TRANSIT_YEARS} years on. */
function muhurtaWindow({ civilDate: { year, month, day } }) {
  const from = Date.UTC(year + TRANSIT_YEARS, month - 1, day);
  return {
    from: isoDate(from),
    to: isoDate(from + (MUHURTA_DAYS - 1) * 86_400_000),
    iana: MUHURTA_ZONE,
    utcOffsetSeconds: offsetSeconds(MUHURTA_ZONE, from),
    place: MUHURTA_PLACE,
  };
}

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
    muhurta: muhurtaWindow(birth),
  }));
}

/**
 * The corpus's `default` snapshot, the settings every chart of it was
 * recorded under, as the stored fields its manifest's canonical form
 * lists (`ayanamsaId=1|…|useTopocentric=true`): a number, a flag or text.
 */
export function defaultSnapshot(manifest = join(root, 'fixtures/baseline/manifest.json')) {
  const { canonical } = JSON.parse(readFileSync(manifest, 'utf8')).profiles.default.settings_hash;
  const stored = (value) =>
    /^-?\d+$/.test(value) ? Number(value) : value === 'true' ? true : value === 'false' ? false : value;
  return Object.fromEntries(canonical.split('|').map((pair) => pair.split('=')).map(([field, value]) => [field, stored(value)]));
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
