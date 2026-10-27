// Contexts on worker threads (`docs/05-testing/01-quality-bar.md`, "no
// global state"): the native module is loaded once per worker, each worker
// founds its own context and asks the same week of days and chart, and
// every worker answers what the main thread answers. A worker that saw
// another's state, or a module whose lazily built tables raced, would
// answer otherwise or crash the process.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Worker } from 'node:worker_threads';

/** The workers asked at once. */
const WORKERS = 4;

/** What a context answers, as plain numbers a worker can post back. */
const asked = `
  const { Calendar, Context, Resolution } = await import(${JSON.stringify(new URL('../lib/index.js', import.meta.url).href)});
  const date = (day) => ({ calendar: Calendar.Gregorian, year: 2024, eraYear: 0, month: 6, day,
    resolution: Resolution.Defined, computedMonth: 0, computedDay: 0 });
  const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
  const ctx = new Context({ testProvider: true, profile: 'nepali-default', locale: 'ne-Deva-NP' });
  const week = ctx.almanac.of({ from: date(17), to: date(23), place, utcOffsetSeconds: 20700 });
  const days = Array.from({ length: week.length }, (_, i) => [week.at(i).day.sunrise, week.at(i).kaalas.length]);
  const chart = ctx.chart.found({ instant: 2451545.25, place, utcOffsetSeconds: 20700 });
  const grahas = chart.grahas.map((g) => [g.graha, g.longitudeDeg]);
  ctx.dispose();
  return JSON.stringify({ days, grahas });
`;

/** The answer computed on a worker of its own. */
function onAWorker() {
  return new Promise((resolve, reject) => {
    const worker = new Worker(
      `const { parentPort } = require('node:worker_threads');
       (async () => { ${asked} })().then((answer) => parentPort.postMessage(answer));`,
      { eval: true },
    );
    worker.once('message', resolve);
    worker.once('error', reject);
    worker.once('exit', (code) => code === 0 || reject(new Error(`a worker exited with ${code}`)));
  });
}

test('contexts on worker threads answer what the main thread answers', async () => {
  const here = await new Function(`return (async () => { ${asked} })()`)();
  const answers = await Promise.all(Array.from({ length: WORKERS }, onAWorker));
  for (const answer of answers) assert.equal(answer, here);
  assert.equal(JSON.parse(here).days.length, 7);
  assert.ok(JSON.parse(here).grahas.length >= 9);
});
