/**
 * Runs the Deno or the Bun check: the installed wasm package run by the
 * pinned runtime, as a consumer's program runs it, and prints what it
 * answered — the probe's answer, or the error that stopped it.
 *
 * Usage: node run.mjs <project> <deno|bun>
 *
 * `<project>` is a directory whose `node_modules` holds the installed
 * package (the check's throwaway consumer). The runtimes are this
 * directory's own, installed from its lock file.
 *
 * - **Deno** runs the program with no permission granted, which is the
 *   point: the Node loader, which Deno would take without a `deno`
 *   condition, reads the module with `node:fs` and is refused there.
 * - **Bun** runs it three ways, each of which must answer alike: as it
 *   stands, bundled by `bun build`, and compiled by `bun build --compile`
 *   into one executable. The bundle and the executable run from a
 *   directory of their own, so a module they did not carry with them is
 *   not found beside them by chance.
 */

import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync, readFileSync, rmSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const [projectArg, runtime] = process.argv.slice(2);
if (!projectArg || !['deno', 'bun'].includes(runtime)) {
  console.error('usage: node run.mjs <project> <deno|bun>');
  process.exit(2);
}
const HERE = fileURLToPath(new URL('.', import.meta.url));
const DEADLINE_MS = 180_000;

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

/** The report a run of the program printed, or one saying it printed none. */
function reported(how, done) {
  const line = `${done.stdout}`
    .split('\n')
    .find((text) => text.startsWith('{"answer"') || text.startsWith('{"error"'));
  if (!line) {
    return { error: `${how} reported nothing (exited ${done.status}):\n${done.stderr}${done.stdout}` };
  }
  const body = JSON.parse(line);
  return body.error ? { error: `${how}: ${body.error}` } : body;
}

// The program's own directory inside the project, so `@teistro/sdk-wasm`
// resolves to the installed package as it does in a consumer's program.
const work = join(resolve(projectArg), runtime);
rmSync(work, { recursive: true, force: true });
mkdirSync(work, { recursive: true });
copyFileSync(join(HERE, 'main.mjs'), join(work, 'main.mjs'));
copyFileSync(join(HERE, '..', 'browser', 'probe.mjs'), join(work, 'probe.mjs'));

if (runtime === 'deno') {
  // Deno's bin is a Node script that starts the platform's binary, so it
  // needs no install script. Its cache goes beside the program rather
  // than in the user's.
  report(
    reported(
      'Deno, with no permission granted,',
      run(process.execPath, [bin('deno', 'deno'), 'run', '--no-prompt', 'main.mjs'], work, {
        DENO_DIR: join(work, 'cache'),
        DENO_NO_UPDATE_CHECK: '1',
        NO_COLOR: '1',
      }),
    ),
  );
}

// Bun's bin is the binary itself, put there by its install script.
const bun = bin('bun', 'bun');
const out = join(work, 'out');
const executable = join(out, process.platform === 'win32' ? 'main.exe' : 'main');
const built = (args) => {
  const done = run(bun, ['build', ...args, 'main.mjs'], work);
  if (done.status !== 0) report({ error: `bun build ${args.join(' ')} failed:\n${done.stderr}${done.stdout}` });
};
built(['--target=bun', '--outdir', out]);
built(['--compile', '--outfile', executable]);

const runs = [
  reported('Bun', run(bun, ['main.mjs'], work)),
  reported("Bun's bundle", run(bun, [join(out, 'main.js')], out)),
  reported("Bun's executable", run(executable, [], out)),
];
const failed = runs.find((body) => body.error);
if (failed) report(failed);
const [first] = runs;
const differs = runs.find((body) => JSON.stringify(body.answer) !== JSON.stringify(first.answer));
if (differs) report({ error: `Bun's three runs answered differently: ${JSON.stringify(runs)}` });
report(first);
