/**
 * The Deno and Bun checks' program: the installed package, imported by
 * its name as a consumer's program imports it, and the browser check's
 * probe run once. It prints one line of JSON, the probe's answer or the
 * error that stopped it, which `run.mjs` reads back.
 */

import * as sdk from '@teistro/sdk-wasm';

import { probe } from './probe.mjs';

let report;
try {
  report = { answer: probe(sdk) };
} catch (error) {
  report = { error: `${error.name}: ${error.message}` };
}
console.log(JSON.stringify(report));
