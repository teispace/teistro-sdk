/**
 * The wasm package's native half in Cloudflare Workers (workerd): the
 * module imported, already compiled, and instantiated synchronously.
 *
 * Workers compile no WebAssembly at run time: `WebAssembly.compile` and
 * `instantiate` on bytes are refused, and there is no `import.meta.url`
 * to fetch the module beside. What a Worker has instead is the import
 * below, which Wrangler (and every bundler that targets workerd) turns into
 * a `WebAssembly.Module` compiled when the Worker is deployed. The web
 * loader, taken here, bundles without a word and fails when the Worker
 * starts (`Invalid URL string`), which is why this one exists and why the
 * package's `workerd` condition comes before every other.
 *
 * `initSync` needs no top-level `await`, and instantiating a compiled
 * module does no I/O, so this is also within what a Worker may do while
 * its script is being evaluated (`03-design/wasm-binding.md`).
 */

import * as glue from '../wasm/teistro_wasm.js';
import module from '../wasm/teistro_wasm_bg.wasm';

glue.initSync({ module });

/** The module's exports: the native object the layer calls. */
export const native = glue;

/** Never named by an environment variable: a Worker's are its bindings. */
export const named = false;

/** The package that carries this build: the wasm one, on every host. */
export function platformPackage() {
  return '@teistro/sdk-wasm';
}
