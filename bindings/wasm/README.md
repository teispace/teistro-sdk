# @teistro/sdk-wasm

The Teistro SDK as WebAssembly: **the same API as
[`@teistro/sdk`](../node/README.md)**, over a wasm module instead of a
native addon. Use it in a browser, a Web Worker, or any JavaScript host no
prebuilt addon covers.

```js
import { Body, Context } from '@teistro/sdk-wasm';

const ctx = new Context({ profile: 'parashari-classical', ephemeris: 'BUILTIN' });
try {
  const sky = ctx.positions({ instants: [2451545.0], bodies: [Body.Sun] });
  console.log(sky.at(0, 0).longitude); // ≈ 280.37
} finally {
  ctx.dispose();
}
```

Everything in the Node package's README applies here unchanged: its
documentation, types and examples are the same files. The package is the
Node package's JavaScript layer with a different loader underneath.

## Loading

Importing the package is the whole setup. The module is instantiated
before the layer reads it (with top-level `await` in a browser,
synchronously everywhere else), so nothing you import is ever half-ready.

- **With a bundler** (Vite, webpack 5, Rollup, Parcel, esbuild), install
  and import it. The package finds its `.wasm` file with
  `new URL(…, import.meta.url)`, which bundlers recognise and ship
  beside your bundle. A bundler resolves the package's `#native` import
  to the web loader through its `default` condition.
- **Unbundled in a browser**, serve the package and map `#native` to
  `lib/native.web.js` in an import map. The package's own browser check
  loads it exactly that way.
- **In Node, Deno or Bun**, the `node` condition picks a loader that reads
  the module from the package and compiles it synchronously.
- **In Cloudflare Workers**, install and import it; Wrangler resolves the
  `workerd` condition to a loader that imports the module precompiled, as
  a Worker must. The bundle is about 1.4 MB gzipped, within the free
  plan's 3 MB.

  ```js
  import { Body, Context } from '@teistro/sdk-wasm';

  export default {
    fetch() {
      const ctx = new Context({ profile: 'parashari-classical', ephemeris: 'BUILTIN' });
      try {
        const sky = ctx.positions({ instants: [2451545.0], bodies: [Body.Sun] });
        return Response.json({ sun: sky.at(0, 0).longitude });
      } finally {
        ctx.dispose();
      }
    },
  };
  ```

It needs no cross-origin isolation (no COOP or COEP headers) and no
`SharedArrayBuffer`. That is why it is built with wasm-bindgen and not
napi-rs's own wasm target (`docs/03-design/wasm-binding.md`).

## What differs from the Node package

- **No plugins.** A plugin is a shared library, which a wasm module
  cannot open (ADR-0029). `ephemeris: { plugin }` is refused and names
  what to give instead. In a chain such as
  `[{ plugin }, 'BUILTIN']`, the plugin entry is skipped the way a
  missing file is, so one chain written for both packages falls back
  in a browser.
- **An ephemeris of your own** is `provider`: an object with a
  `positions(request)` function, written in JavaScript. It works exactly
  as in Node, since both bindings share one adapter.
- **Dispose what you create.** A wasm module's memory is not the
  garbage collector's, so call `dispose()` on a context when you are done
  with it. The finaliser is a safety net, not a plan (ADR-0007).
- **One thread.** A context belongs to the thread that made it. For
  parallel work, make one context per Web Worker.

## Size

The module is **4.8 MB, 1.3 MB gzipped**. It carries the `compact` tier of
the built-in ephemeris, which is one arcminute and what ADR-0029 names for
a browser. It is built for size (fat LTO, `opt-level = "s"`, measured to
be as fast as the release build) and ships without its debug names.
`bindings/wasm/size.json` is its budget, and `check-wasm` fails the build
if it grows. For the arcsecond tiers, bring a provider.

## Checked

`cargo xtask check-wasm` stages this package and checks it five ways:

- the Node binding's whole test suite, unchanged, through this package's
  own loader;
- in headless Chrome, unbundled;
- the same probe under Node;
- packed, installed into an empty project and run as a consumer;
- installed, bundled by Wrangler and run in Cloudflare's workerd.

The browser's and the Worker's answers must equal Node's bit for bit, and the module must
stay within its size budget. `cargo xtask check-parity` also compares this
package with the Node, Dart, Python and Rust bindings, value by value.
