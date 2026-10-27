# @teistro/sdk-wasm

The Teistro SDK as WebAssembly: **the same API as
[`@teistro/sdk`](../node/README.md)**, over a wasm module instead of a
native addon. Use it in a browser, a Web Worker, or any JavaScript host no
prebuilt addon covers.

```js fragment
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
- **In Node**, the `node` condition picks a loader that reads the module
  from the package and compiles it synchronously.
- **In Deno**, install it and import it by name; the `deno` condition
  picks a loader that imports the module in its source phase
  (`import source`), so Deno compiles it with the rest of the program
  and asks no permission to read it: `deno run main.js` needs no
  `--allow-read`. Deno Deploy runs the same runtime and loader, but the
  check runs only the Deno CLI.
- **In Bun**, install and import it; the `bun` condition picks a loader
  whose module is an import Bun's bundler follows, so `bun build` copies
  it beside the bundle and `bun build --compile` embeds it in the
  executable.
- **In Cloudflare Workers**, install and import it; Wrangler resolves the
  `workerd` condition to a loader that imports the module precompiled, as
  a Worker must. The bundle is about 1.4 MB gzipped, within the free
  plan's 3 MB.

  ```js fragment
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

- **In a Next.js edge route** (`export const runtime = 'edge'`), install
  and import it; Next.js resolves the `edge-light` condition to a loader
  that imports the module as `?module`, a module Next.js compiles outside
  the route, since the Edge runtime compiles no WebAssembly from bytes.
  Turbopack and webpack builds both work. Next.js 16 marks the edge
  runtime deprecated. Vercel limits an edge function's size by plan, and
  the build carries the module twice (`docs/03-design/wasm-binding.md`),
  so measure the function before deploying the full module there; the
  `panchanga` profile is about half its size.

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

## The `panchanga` profile

A patro, a calendar widget or an almanac page needs the calendars, the
days and the muhurta search, and not the charts. Import the subpath and
the bundle carries a module about half the size:

```js fragment
import { Calendar, Context } from '@teistro/sdk-wasm/panchanga';
```

It is the same layer and the same types over a module built without the
chart area, so the code is unchanged. What it keeps answers exactly as
the full module does, and a call into what it leaves out (a chart, a
dasha, matching, the Western and KP readings) throws a `TeistroError`
whose `status` is `'CAPABILITY'` and whose message names the family. A
bundler that imports only the subpath ships only its module.

## Size

The module carries the `compact` tier of
the built-in ephemeris, which is one arcminute and what ADR-0029 names for
a browser. It is built for size (fat LTO, `opt-level = "s"`, measured to
be as fast as the release build) and ships without its debug names.
`docs/05-testing/sizes.json` holds its budget and the `panchanga`
module's, `check-wasm` fails the build if either grows, and
`docs/05-testing/SIZES.md` lists what each module and every other
artefact last measured. For the arcsecond tiers, bring a provider.

## Checked

`cargo xtask check-wasm` stages this package and checks it these ways:

- the Node binding's whole test suite, unchanged, through this package's
  own loader;
- in headless Chrome, unbundled;
- the same probe under Node;
- packed, installed into an empty project and run as a consumer;
- installed, bundled by Wrangler and run in Cloudflare's workerd;
- installed and run by Deno with no permission granted;
- installed and run by Bun as it stands, bundled by `bun build`, and
  compiled by `bun build --compile`;
- installed into a Next.js app as an edge route, built by Turbopack and
  by webpack, and served by `next start` with WebAssembly compiled from
  bytes refused, as the deployed Edge runtime refuses it.

Every host's answer must equal Node's bit for bit, and the module must
stay within its size budget. A host none of these runs is not claimed:
Netlify's edge functions and Deno Deploy run Deno and take the `deno`
loader, but only the Deno CLI is run, and Vercel's deployed Edge runtime
is stood in for by Next.js's own. `cargo xtask check-parity` also compares this
package with the Node, Dart, Python and Rust bindings, value by value.
