/**
 * Runs the Next.js edge check: an app with one edge route that imports
 * the installed wasm package, built by each of Next.js's bundlers as a
 * consumer's `next build` builds it, served by `next start`, and asked
 * once; prints what the route answered — the probe's answer, or the
 * error that stopped it.
 *
 * Usage: node run.mjs <project>
 *
 * `<project>` is a directory whose `node_modules` holds the installed
 * package (the check's throwaway consumer). The app is written into
 * `<project>/next`, with this directory's manifest and lock file, and
 * the pinned Next.js is installed there with `npm ci`, so the app is a
 * project of its own, as a consumer's is, and not one reaching into this
 * directory through links. The installed package is copied in beside
 * Next.js.
 *
 * The manifest is not `"type": "module"`: with it, Next.js 16.4.3's
 * webpack build of an edge route fails before any of the package is run
 * (`route_client-reference-manifest.js` is never written), with a route
 * that imports nothing.
 *
 * Both bundlers are run: Turbopack is `next build`'s default, and
 * webpack is what `--webpack` and older apps build with. Each build and
 * each `next start` runs under `strict.cjs`, which refuses WebAssembly
 * compiled from bytes as the deployed Edge runtime does. Their answers
 * must be alike, and that one is printed.
 */

import { spawn, spawnSync } from 'node:child_process';
import { copyFileSync, cpSync, mkdirSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const [projectArg] = process.argv.slice(2);
if (!projectArg) {
  console.error('usage: node run.mjs <project>');
  process.exit(2);
}
const HERE = fileURLToPath(new URL('.', import.meta.url));
const DEADLINE_MS = 300_000;
const PACKAGE = '@teistro/sdk-wasm';

/** Prints a report and ends with its verdict. */
function report(body) {
  process.stdout.write(`${JSON.stringify(body)}\n`);
  process.exit(body.answer ? 0 : 1);
}

// Whatever stops this script is a report too, so the gate always reads
// one and never an empty line.
process.on('uncaughtException', (error) => report({ error: `run.mjs: ${error.stack ?? error}` }));

const project = resolve(projectArg);
const app = join(project, 'next');
// No telemetry, and the edge sandbox as strict as the deployed runtime.
const env = {
  ...process.env,
  NEXT_TELEMETRY_DISABLED: '1',
  NODE_OPTIONS: `--require ${JSON.stringify(join(HERE, 'strict.cjs'))}`,
};

/** Runs a program to completion, or reports why it did not finish. */
function run(program, args, what) {
  const done = spawnSync(program, args, {
    cwd: app,
    encoding: 'utf8',
    timeout: DEADLINE_MS,
    env,
    // npm is a script on Windows, which only a shell starts.
    shell: process.platform === 'win32' && program === 'npm',
  });
  if (done.error) report({ error: `${what} did not run: ${done.error.message}` });
  if (done.status !== 0) report({ error: `${what} failed:\n${done.stderr}${done.stdout}` });
}

rmSync(app, { recursive: true, force: true });
mkdirSync(join(app, 'app', 'probe'), { recursive: true });
for (const file of ['package.json', 'package-lock.json']) {
  copyFileSync(join(HERE, file), join(app, file));
}
copyFileSync(join(HERE, 'route.js'), join(app, 'app', 'probe', 'route.js'));
copyFileSync(join(HERE, '..', 'browser', 'probe.mjs'), join(app, 'app', 'probe', 'probe.mjs'));
// The project is its own root: the consumer around it has a lock file
// too, which Next.js would otherwise take for the workspace's. Named by
// its real path, since Turbopack refuses a root its output is not
// inside, and compares the two after resolving links (macOS's `/var`).
writeFileSync(
  join(app, 'next.config.mjs'),
  `const root = ${JSON.stringify(realpathSync(app))};\nexport default { turbopack: { root }, outputFileTracingRoot: root };\n`,
);
run('npm', ['ci', '--silent', '--no-audit', '--no-fund'], 'npm ci');
cpSync(join(project, 'node_modules', PACKAGE), join(app, 'node_modules', PACKAGE), {
  recursive: true,
});

const next = join(app, 'node_modules', 'next', 'dist', 'bin', 'next');

/** What the route answers when the app is built by `bundler` and served. */
async function answered(bundler, flags) {
  rmSync(join(app, '.next'), { recursive: true, force: true });
  run(process.execPath, [next, 'build', ...flags], `next build (${bundler})`);
  const server = spawn(
    process.execPath,
    [next, 'start', '--port', '0', '--hostname', '127.0.0.1'],
    { cwd: app, env },
  );
  try {
    const port = await new Promise((found, failed) => {
      let printed = '';
      const timer = setTimeout(
        () => failed(new Error(`next start printed no address:\n${printed}`)),
        DEADLINE_MS,
      );
      const read = (chunk) => {
        printed += chunk;
        const address = printed.match(/http:\/\/127\.0\.0\.1:(\d+)/);
        if (address) {
          clearTimeout(timer);
          found(address[1]);
        }
      };
      server.stdout.on('data', read);
      server.stderr.on('data', read);
      server.on('exit', (code) => failed(new Error(`next start exited ${code}:\n${printed}`)));
    });
    const response = await fetch(`http://127.0.0.1:${port}/probe`);
    const text = await response.text();
    let body;
    try {
      body = JSON.parse(text);
    } catch {
      return { error: `the route (${bundler}) answered ${response.status}, not JSON:\n${text}` };
    }
    return body.error ? { error: `the route (${bundler}): ${body.error}` } : body;
  } finally {
    server.kill();
  }
}

let runs;
try {
  runs = [await answered('Turbopack', []), await answered('webpack', ['--webpack'])];
} catch (error) {
  report({ error: `run.mjs: ${error.stack ?? error}` });
}
const failed = runs.find((body) => body.error);
if (failed) report(failed);
const [first, second] = runs;
if (JSON.stringify(first.answer) !== JSON.stringify(second.answer)) {
  report({ error: `Turbopack's build and webpack's answered differently: ${JSON.stringify(runs)}` });
}
report(first);
