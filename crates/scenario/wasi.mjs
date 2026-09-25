// Runs the scenario binary built for `wasm32-wasip1` under Node's own
// WASI, so the hash matrix has a wasm32 column with nothing to install
// beyond Node (ADR-0022; `.github/workflows/hash-matrix.yml`).
//
//   node crates/scenario/wasi.mjs <teistro-scenario.wasm> values > values-wasm32.tsv
//
// Every argument after the module is handed to the program, which prints
// to standard output exactly what it prints natively.
import { readFile } from 'node:fs/promises';
import { WASI } from 'node:wasi';

const [module, ...args] = process.argv.slice(2);
if (!module) {
  console.error('usage: node wasi.mjs <teistro-scenario.wasm> <section | values>');
  process.exit(2);
}
const wasi = new WASI({ version: 'preview1', args: ['teistro-scenario', ...args], env: {} });
const compiled = await WebAssembly.compile(await readFile(module));
const instance = await WebAssembly.instantiate(compiled, wasi.getImportObject());
process.exitCode = wasi.start(instance);
