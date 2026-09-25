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
before the layer reads it, with top-level `await`, so nothing you import
is ever half-ready.

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

The module is built at the default ephemeris tier and is not yet
optimised with `wasm-opt`. Per-profile binaries, with the `compact` tier a
browser runs on, and a size gate for each are the design's last step.

## Checked

`cargo xtask check-wasm` stages this package and runs it three ways:

- the Node binding's whole test suite, unchanged, through this package's
  own loader;
- in headless Chrome, unbundled;
- the same probe under Node.

The browser's answer must equal Node's bit for bit.
