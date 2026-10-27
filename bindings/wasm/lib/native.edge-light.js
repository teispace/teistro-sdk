/**
 * The wasm package's native half in Vercel's Edge runtime and Next.js's
 * edge routes: the module imported, already compiled, and instantiated.
 *
 * The Edge runtime compiles no WebAssembly from bytes, as a Worker
 * compiles none, and Next.js bundles a route for it with a loader of its
 * own for a `.wasm` import: `?module` asks it for a `WebAssembly.Module`,
 * compiled outside the route. Both of Next.js's bundlers, Turbopack and
 * webpack, read that spelling, and resolve the package's `edge-light`
 * condition for an edge route.
 *
 * Where Next.js runs a route itself (`next build` and `next start`), the
 * route runs in a realm of its own and the module comes from Next.js's,
 * so the glue's `instanceof WebAssembly.Module` is false, and `initSync`
 * would take the module for bytes and fail (`WebAssembly.Module():
 * Argument 0 must be a buffer source`). The module is a module in every
 * realm, as `WebAssembly.Instance` checks it, only its prototype is
 * another realm's; so it is given this realm's, and instantiated
 * synchronously. The other way, `init`, needs a top-level `await`, and
 * webpack warns of one in every edge build (`your target environment
 * does not appear to support 'async/await'`) although the runtime has
 * it (`03-design/wasm-binding.md`).
 */

import * as glue from '../wasm/teistro_wasm.js';
import module from '../wasm/teistro_wasm_bg.wasm?module';

if (!(module instanceof WebAssembly.Module)) {
  Object.setPrototypeOf(module, WebAssembly.Module.prototype);
}
glue.initSync({ module });

/** The module's exports: the native object the layer calls. */
export const native = glue;

/** Never named by an environment variable: an edge route's are its own. */
export const named = false;

/** No sanitizer build is loaded as wasm. */
export const sanitizer = '';

/** The package that carries this build: the wasm one, on every host. */
export function platformPackage() {
  return '@teistro/sdk-wasm';
}
