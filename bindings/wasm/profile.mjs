/**
 * The profile check: a profile's module answers what it keeps as the
 * full module does, to the bit, and refuses what it leaves out as a
 * capability, naming the family (`03-design/wasm-profiles.md`).
 *
 * Usage: node profile.mjs <staged package> <profile>
 *
 * Both entries are imported from the staged package, each through its own
 * `node` loader, so the two modules are the ones a consumer gets. Prints
 * `{ answer: { full, profile, refusal } }`: the gate holds `full` and
 * `profile` equal and reads the refusal, or `{ error }`.
 */

import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [packageDir, profile] = process.argv.slice(2);
if (!packageDir || !profile) {
  console.error('usage: node profile.mjs <staged package> <profile>');
  process.exit(2);
}
const entry = (file) => import(pathToFileURL(resolve(packageDir, 'lib', file)).href);

/** What every profile keeps: the calendars, the almanac and its muhurta search. */
function kept(sdk) {
  const { Calendar, Context, Resolution } = sdk;
  const date = (year, month, day) => ({
    calendar: Calendar.Gregorian,
    year,
    eraYear: 0,
    month,
    day,
    resolution: Resolution.Defined,
    computedMonth: 0,
    computedDay: 0,
  });
  const ctx = new Context({ profile: 'nepali-default', ephemeris: 'BUILTIN' });
  try {
    const almanac = ctx.almanac.of({
      from: date(2026, 11, 25),
      to: date(2026, 12, 3),
      place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
      utcOffsetSeconds: 20700,
      muhurta: {
        rules: 'RAMAN_MARRIAGE',
        native: { star: 'ROHINI', moonSign: 'rashi.TAURUS', lagna: 'LEO' },
        daysWithWindows: 9,
        most: 1000,
      },
    });
    return {
      settingsHash: ctx.settingsHash,
      bikramSambat: ctx.calendar.convert(date(2015, 4, 14), Calendar.BikramSambat),
      // Each day's hash is the SHA-256 of its canonical value, so equal
      // hashes are equal days to the bit.
      days: Array.from({ length: almanac.length }, (_, k) => almanac.at(k).provenance.contentHash),
      // JSON writes each number in the shortest text that reads back to
      // the same bits.
      muhurta: JSON.stringify(almanac.muhurta.windows),
    };
  } finally {
    ctx.dispose();
  }
}

/** One call into the chart area, which every profile leaves out: its refusal. */
function refused(sdk) {
  const ctx = new sdk.Context({ profile: 'nepali-default', ephemeris: 'BUILTIN' });
  try {
    ctx.chart.found({ instant: 2451545, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0 });
    return null;
  } catch (error) {
    return { status: error.status ?? null, message: String(error.message) };
  } finally {
    ctx.dispose();
  }
}

try {
  const full = await entry('index.js');
  const part = await entry(`${profile}.js`);
  const answer = { full: kept(full), profile: kept(part), refusal: refused(part) };
  if (refused(full) !== null) throw new Error('the full module refused the chart it carries');
  process.stdout.write(`${JSON.stringify({ answer })}\n`);
} catch (error) {
  process.stdout.write(`${JSON.stringify({ error: String(error?.stack ?? error) })}\n`);
  process.exit(1);
}
