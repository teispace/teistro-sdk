/**
 * Runs the edge check: bundles `worker.mjs` against the installed wasm
 * package with the pinned Wrangler, exactly as a consumer's `wrangler
 * deploy` bundles it, runs the bundle in the pinned workerd, and prints
 * what the Worker answered — the probe's answer, or the error that
 * stopped it.
 *
 * Usage: node run.mjs <project>
 *
 * `<project>` is a directory whose `node_modules` holds the installed
 * package (the check's throwaway consumer). The tools are this
 * directory's own, installed from its lock file.
 *
 * `wrangler deploy --dry-run` is the bundle a deploy would upload, with
 * no account and no network; `workerd test` runs it with no port. Both
 * are run through their packages rather than a `.bin` shim, so the same
 * lines work on every platform.
 */

import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const [projectArg] = process.argv.slice(2);
if (!projectArg) {
  console.error('usage: node run.mjs <project>');
  process.exit(2);
}
const HERE = fileURLToPath(new URL('.', import.meta.url));
const DEADLINE_MS = 180_000;
const require = createRequire(join(HERE, 'package.json'));

/** Prints a report and ends with its verdict. */
function report(body) {
  process.stdout.write(`${JSON.stringify(body)}\n`);
  process.exit(body.answer ? 0 : 1);
}

// Whatever stops this script is a report too, so the gate always reads
// one and never an empty line.
process.on('uncaughtException', (error) => report({ error: `run.mjs: ${error.stack ?? error}` }));

/** A pinned tool's executable, by its manifest's `bin`: packages need
 * not export their bin files, so they are not resolved as modules. */
function bin(pkg, name) {
  const dir = join(HERE, 'node_modules', pkg);
  const manifest = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
  return join(dir, typeof manifest.bin === 'string' ? manifest.bin : manifest.bin[name]);
}

/** Runs a program to completion, or reports why it did not finish. */
function run(program, args, cwd, env = {}) {
  const done = spawnSync(program, args, {
    cwd,
    encoding: 'utf8',
    timeout: DEADLINE_MS,
    env: { ...process.env, ...env },
  });
  if (done.error) report({ error: `${program} did not run: ${done.error.message}` });
  return done;
}

// The Worker's own directory inside the project, so `@teistro/sdk-wasm`
// resolves to the installed package as it does in a consumer's Worker.
const worker = join(resolve(projectArg), 'workerd');
const dist = join(worker, 'dist');
rmSync(worker, { recursive: true, force: true });
mkdirSync(worker, { recursive: true });
copyFileSync(join(HERE, 'worker.mjs'), join(worker, 'worker.mjs'));
copyFileSync(join(HERE, '..', 'browser', 'probe.mjs'), join(worker, 'probe.mjs'));

// The newest date this workerd supports: a date is required, and one the
// runtime is older than is refused, so it is read from the runtime.
// `workerd` is CommonJS: its exports are the binary's path (`default`)
// and that date.
const workerd = require('workerd');
const date = workerd.compatibilityDate;

const bundled = run(
  process.execPath,
  [
    bin('wrangler', 'wrangler'),
    'deploy',
    '--dry-run',
    '--outdir',
    dist,
    '--name',
    'teistro-edge-check',
    '--compatibility-date',
    date,
    'worker.mjs',
  ],
  worker,
  // No telemetry, and the logs beside the bundle rather than in the
  // user's preferences.
  { WRANGLER_SEND_METRICS: 'false', WRANGLER_LOG_PATH: join(worker, 'logs') },
);
if (bundled.status !== 0) {
  report({ error: `wrangler did not bundle the Worker:\n${bundled.stderr}${bundled.stdout}` });
}

// Every module Wrangler wrote, named as the bundle imports it.
const files = readdirSync(dist);
const modules = [
  '(name = "worker.js", esModule = embed "worker.js")',
  ...files
    .filter((file) => file.endsWith('.wasm'))
    .map((file) => `(name = "${file}", wasm = embed "${file}")`),
];
const config = join(dist, 'check.capnp');
writeFileSync(
  config,
  `using Workerd = import "/workerd/workerd.capnp";
const config :Workerd.Config = (services = [(name = "check", worker = .check)]);
const check :Workerd.Worker = (
  modules = [${modules.join(', ')}],
  compatibilityDate = "${date}",
);
`,
);

const ran = run(workerd.default, ['test', config], dist);
const line = `${ran.stdout}\n${ran.stderr}`
  .split('\n')
  .find((text) => text.startsWith('{"answer"') || text.startsWith('{"error"'));
if (!line) {
  report({ error: `the Worker reported nothing (workerd exited ${ran.status}):\n${ran.stderr}${ran.stdout}` });
}
report(JSON.parse(line));
