/**
 * The edge check's Worker: the installed package, imported by its name as
 * a Worker imports it, and the browser check's probe run once.
 *
 * It answers as a test and not a request, so workerd runs it and exits
 * with no port and no server; the answer is one line of JSON on the
 * runtime's output, which `run.mjs` reads back.
 */

import * as sdk from '@teistro/sdk-wasm';

import { probe } from './probe.mjs';

export default {
  test() {
    let report;
    try {
      report = { answer: probe(sdk) };
    } catch (error) {
      report = { error: `${error.name}: ${error.message}` };
    }
    console.log(JSON.stringify(report));
  },
};
