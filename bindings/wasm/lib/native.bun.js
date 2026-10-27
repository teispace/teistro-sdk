/**
 * The wasm package's native half in Bun: the module read from where Bun
 * says it is, and compiled synchronously.
 *
 * Bun runs the Node loader unbundled, but a `bun build` bundle or a
 * `bun build --compile` executable leaves the module behind: the Node
 * loader names it only by a URL built at run time, which no bundler
 * follows, so the executable looks for it in its own embedded file system
 * and finds nothing (`ENOENT … /$bunfs/wasm/teistro_wasm_bg.wasm`). An
 * import with `type: 'file'` is a dependency Bun's bundler sees: it
 * copies the module beside a bundle, embeds it in an executable, and
 * gives back a path to read it from. The attribute is Bun's and no other
 * host's, which is why only the `bun` condition picks this file
 * (`03-design/wasm-binding.md`).
 *
 * The path is absolute unbundled and in an executable, and relative to
 * the bundle in a bundle (`./teistro_wasm_bg-<hash>.wasm`), where reading
 * it as given would look in the working directory instead.
 */

import { readFileSync } from 'node:fs';
import { isAbsolute } from 'node:path';

import * as glue from '../wasm/teistro_wasm.js';
import path from '../wasm/teistro_wasm_bg.wasm' with { type: 'file' };

glue.initSync({
  module: readFileSync(isAbsolute(path) ? path : new URL(path, import.meta.url)),
});

/** The module's exports: the native object the layer calls. */
export const native = glue;

/** Never named by an environment variable: the package carries its module. */
export const named = false;

/** No sanitizer build is loaded as wasm. */
export const sanitizer = '';

/** The package that carries this build: the wasm one, on every host. */
export function platformPackage() {
  return '@teistro/sdk-wasm';
}
