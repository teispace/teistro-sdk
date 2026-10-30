/**
 * Runs the browser check: serves the staged wasm package and this page,
 * opens the page in a headless Chrome, and prints what the page posts
 * back — the probe's answer, or the error that stopped it.
 *
 * Usage: node run.mjs <staged package> <chrome>
 */

import { askChrome } from './serve.mjs';

const [packageDir, chrome] = process.argv.slice(2);
if (!packageDir || !chrome) {
  console.error('usage: node run.mjs <staged package> <chrome>');
  process.exit(2);
}
const body = await askChrome(packageDir, chrome, '/check/index.html');
process.stdout.write(`${body}\n`);
process.exit(JSON.parse(body).answer ? 0 : 1);
