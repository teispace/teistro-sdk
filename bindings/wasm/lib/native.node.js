/**
 * The wasm package's native half under Node, Deno or Bun: the same module
 * a browser loads, read from the package and compiled synchronously, for
 * a host no prebuilt addon covers (a WebContainer, an unusual
 * architecture) or a consumer who wants one artefact everywhere.
 *
 * Chosen by the package's `node` import condition, so a browser bundle
 * never reaches the `node:fs` below.
 */

import { readFileSync } from 'node:fs';

import * as glue from '../wasm/teistro_wasm.js';

glue.initSync({
  module: readFileSync(new URL('../wasm/teistro_wasm_bg.wasm', import.meta.url)),
});

/** The module's exports: the native object the layer calls. */
export const native = glue;

/** Never named by an environment variable: the package carries its module. */
export const named = false;

/** The package that carries this build: the wasm one, on every host. */
export function platformPackage() {
  return '@teistro/sdk-wasm';
}
