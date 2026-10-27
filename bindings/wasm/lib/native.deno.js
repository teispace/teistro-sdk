/**
 * The wasm package's native half in Deno, and so in Deno Deploy and
 * Netlify's edge functions: the module imported in its source phase,
 * already compiled, and instantiated synchronously.
 *
 * Deno resolves a package's `node` condition too, and the Node loader
 * runs there, but it reads the module with `node:fs`, which Deno refuses
 * without `--allow-read` (`NotCapable: Requires read access`). An
 * `import source` is part of the module graph instead: Deno compiles the
 * module while it loads the graph, asks no permission for it, and
 * carries it into whatever it deploys or compiles, as it carries every
 * other import. Source-phase imports are new, and not every host the
 * package serves reads them, which is why only the `deno` condition
 * picks this file (`03-design/wasm-binding.md`).
 */

import * as glue from '../wasm/teistro_wasm.js';
import source module from '../wasm/teistro_wasm_bg.wasm';

glue.initSync({ module });

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
