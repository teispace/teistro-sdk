/**
 * Runs the tree-shaking check: bundles two entries against the installed
 * wasm package with each pinned bundler, as a consumer's production build
 * bundles them, and prints what each bundle weighs and whether it carries
 * the module.
 *
 * Usage: node run.mjs <project>
 *
 * `<project>` is a directory whose `node_modules` holds the installed
 * package (the check's throwaway consumer). The bundlers are this
 * directory's own, installed from its lock file.
 *
 * - `catalogue` imports one member from `@teistro/sdk-wasm/catalogue`. It
 *   must ship no `.wasm` and stay under a few kilobytes, which holds only
 *   when the generated tables are marked pure and the package declares no
 *   side effects.
 * - `everything` imports the package's entry. It must ship the module:
 *   the control that shows the first measurement can see one at all.
 * - `panchanga` imports the profile's subpath. It must ship the profile's
 *   module and not the full one. A bundler that emits the module is told
 *   apart by `wasmBytes`, what the emitted `.wasm` files weigh; one that
 *   leaves `new URL(…)` for the consumer to serve (esbuild) by
 *   `references`, the module paths its scripts name.
 *
 * Prints one JSON object:
 * `{ answer: [{ bundler, entry, bytes, wasm, wasmBytes, references }] }`,
 * or `{ error }` when a bundler failed.
 */

import { mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const [projectArg] = process.argv.slice(2);
if (!projectArg) {
  console.error('usage: node run.mjs <project>');
  process.exit(2);
}
const project = resolve(projectArg);
const HERE = fileURLToPath(new URL('.', import.meta.url));
const require = createRequire(join(HERE, 'package.json'));

const ENTRIES = {
  catalogue: "import { Graha } from '@teistro/sdk-wasm/catalogue';\nconsole.log(Graha.Sun);\n",
  everything: "import { Context } from '@teistro/sdk-wasm';\nconsole.log(typeof Context);\n",
  panchanga: "import { Context } from '@teistro/sdk-wasm/panchanga';\nconsole.log(typeof Context);\n",
};

/** Every file under `directory`, as paths. */
function files(directory) {
  return readdirSync(directory, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => join(entry.parentPath, entry.name));
}

/**
 * What a bundle's output directory holds: the bytes of its JavaScript,
 * the bytes of the modules it emitted, and whether a module came with it, as an emitted `.wasm` file or a
 * reference the bundle resolves at run time.
 */
function measured(out) {
  const written = files(out);
  const scripts = written.filter((path) => /\.m?js$/.test(path));
  const bytes = scripts.reduce((total, path) => total + statSync(path).size, 0);
  const modules = written.filter((path) => path.endsWith('.wasm'));
  const wasmBytes = modules.reduce((total, path) => total + statSync(path).size, 0);
  // An inlined module is a data URI, whose base64 opens with `AGFzbQ`
  // (the module's magic, `\0asm`), and names no file.
  const wasm =
    written.some((path) => path.endsWith('.wasm')) ||
    scripts.some((path) => /\.wasm|AGFzbQ/.test(readFileSync(path, 'utf8')));
  // The glue's own fallback names the module bare, beside itself; a
  // loader's path has a directory in it.
  const named = scripts.flatMap((path) => readFileSync(path, 'utf8').match(/[\w./-]+\.wasm/g) ?? []);
  const references = [...new Set(named.filter((path) => path.includes('/')))].sort();
  return { bytes, wasm, wasmBytes, references };
}

const bundlers = {
  async esbuild(entry, out) {
    const { build } = require('esbuild');
    await build({
      entryPoints: [entry],
      bundle: true,
      minify: true,
      format: 'esm',
      platform: 'browser',
      target: 'es2022',
      outdir: out,
      absWorkingDir: project,
      loader: { '.wasm': 'file' },
      logLevel: 'silent',
    });
  },
  async vite(entry, out) {
    const { build } = await import(require.resolve('vite'));
    await build({
      root: project,
      logLevel: 'silent',
      configFile: false,
      // An application's build, as a consumer runs it, not library mode,
      // which inlines every asset whatever its size.
      build: {
        outDir: out,
        emptyOutDir: true,
        target: 'es2022',
        modulePreload: false,
        rollupOptions: { input: entry },
      },
    });
  },
  async webpack(entry, out) {
    const webpack = require('webpack');
    const stats = await new Promise((done, fail) =>
      webpack(
        {
          mode: 'production',
          context: project,
          entry,
          target: 'web',
          output: { path: out, filename: 'bundle.js', module: true },
          experiments: { outputModule: true, topLevelAwait: true },
          resolve: { modules: [join(project, 'node_modules')] },
        },
        (error, result) => (error ? fail(error) : done(result)),
      ),
    );
    if (stats.hasErrors()) throw new Error(stats.toString({ all: false, errors: true }));
  },
};

const work = join(project, 'bundles');
rmSync(work, { recursive: true, force: true });
const answer = [];
try {
  for (const [entryName, source] of Object.entries(ENTRIES)) {
    const entry = join(work, `${entryName}.mjs`);
    mkdirSync(work, { recursive: true });
    writeFileSync(entry, source);
    for (const [bundler, bundle] of Object.entries(bundlers)) {
      const out = join(work, `${bundler}-${entryName}`);
      await bundle(entry, out);
      answer.push({ bundler, entry: entryName, ...measured(out) });
    }
  }
} catch (error) {
  process.stdout.write(`${JSON.stringify({ error: String(error?.stack ?? error) })}\n`);
  process.exit(1);
}
process.stdout.write(`${JSON.stringify({ answer })}\n`);
