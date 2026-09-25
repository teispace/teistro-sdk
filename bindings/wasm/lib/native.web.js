/**
 * The wasm package's native half for a browser, a worker or a bundler:
 * the module instantiated before the layer above reads it.
 *
 * `new URL(…, import.meta.url)` is how every mainstream bundler (Vite,
 * webpack 5, Rollup, esbuild with its asset loader, Parcel) finds a file a
 * module needs and ships it beside the bundle, and how a browser fetches
 * it unbundled; `init` streams and compiles it. Top-level `await` means
 * the layer never sees a module that is not ready: importing the package
 * is the whole of the setup. Nothing here is Node's, and `index.js` names
 * no Node built-in either (`03-design/wasm-binding.md`).
 */

import init, * as glue from '../wasm/teistro_wasm.js';

await init({ module_or_path: new URL('../wasm/teistro_wasm_bg.wasm', import.meta.url) });

/** The module's exports: the native object the layer calls. */
export const native = glue;

/** Never named by an environment variable: a browser has none. */
export const named = false;

/** The package that carries this build: the wasm one, on every host. */
export function platformPackage() {
  return '@teistro/sdk-wasm';
}
