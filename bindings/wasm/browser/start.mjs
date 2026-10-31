/**
 * The start check: how long each entry takes to its first calendar answer,
 * and which modules it fetched, each process or page a fresh start
 * (`03-design/wasm-lazy-loading.md`).
 *
 * Usage: node start.mjs <staged package> [chrome]
 *
 * Under Node, the median of five fresh processes. In Chrome, a cold page
 * and the same page again with its caches; the page's URL carries the
 * entry in its hash. Then the calendar module refused once: the lazy entry
 * must say so, and say so again without loading it.
 *
 * Prints `{ answer: { node: { entry: ms }, chrome: { entry: { cold, warm,
 * modules } }, unreachable } }`, or `{ error }`.
 */

import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { firstAnswer } from './first-answer.mjs';
import { askChrome } from './serve.mjs';

const ENTRIES = { everything: 'index', panchanga: 'panchanga', lazy: 'lazy' };
const RUNS = 5;
const [packageDir, chrome] = process.argv.slice(2);

if (process.env.TEISTRO_START_ENTRY) {
  // One fresh process, timed from its own start.
  const file = pathToFileURL(resolve(packageDir, 'lib', `${process.env.TEISTRO_START_ENTRY}.js`)).href;
  const entry = await import(file);
  firstAnswer(entry.loadCalendar ? await entry.loadCalendar() : entry);
  process.stdout.write(`${performance.now()}\n`);
  process.exit(0);
}

/** The median of `RUNS` fresh processes. */
function underNode(entry) {
  const script = fileURLToPath(import.meta.url);
  const times = Array.from({ length: RUNS }, () => {
    const run = spawnSync(process.execPath, [script, packageDir], {
      encoding: 'utf8',
      env: { ...process.env, TEISTRO_START_ENTRY: entry },
    });
    if (run.status !== 0) throw new Error(`${entry}: ${run.stderr}`);
    return Number(run.stdout);
  });
  return times.sort((a, b) => a - b)[Math.floor(RUNS / 2)];
}

/** What a page reported, or the failure it reported. */
async function page(hash, options) {
  const body = JSON.parse(await askChrome(packageDir, chrome, `/check/start.html#${hash}`, options));
  if (!body.answer) throw new Error(`${hash}: ${body.error}`);
  return body.answer;
}

try {
  const answer = { node: {}, chrome: {} };
  for (const [name, file] of Object.entries(ENTRIES)) {
    answer.node[name] = underNode(file);
    if (chrome) {
      const profile = mkdtempSync(join(tmpdir(), 'teistro-start-'));
      try {
        const cold = await page(file, { profile });
        const warm = await page(file, { profile });
        answer.chrome[name] = { cold: cold.ms, warm: warm.ms, modules: cold.modules };
      } finally {
        // Chrome's helpers can outlive it and keep writing here; a profile
        // left in the temporary directory fails no measurement.
        try {
          rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
        } catch {}
      }
    }
  }
  if (chrome) {
    const refused = new Set(['/wasm/panchanga/teistro_wasm_bg.wasm']);
    answer.unreachable = (await page('unreachable', { refused })).messages;
  }
  process.stdout.write(`${JSON.stringify({ answer })}\n`);
} catch (error) {
  process.stdout.write(`${JSON.stringify({ error: String(error?.stack ?? error) })}\n`);
  process.exit(1);
}
