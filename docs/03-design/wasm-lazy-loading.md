# Lazy wasm modules

Status: `built`, 2026-10-30 (SR-53). `@teistro/sdk-wasm/lazy` loads the
calendar module, then the jyotish module, when each is asked for, and
`check-wasm` times every entry's first answer. Written from measurements of
the staged package, as [`wasm-binding.md`](wasm-binding.md) and
[`wasm-profiles.md`](wasm-profiles.md) were.

## What was measured

The modules of 2026-10-30, batch `22e78afe` with SR-52 on it, bound as they
ship:

| module | raw | gzipped |
|---|---:|---:|
| full | 13,043,708 | 3,054,274 |
| `panchanga` | 5,250,524 | 1,548,839 |

**Time to a first calendar answer**: a fresh process or page, to the end of
`ctx.calendar.convert` on a new context. Node 26.13, the median of seven
processes; Chrome headless with a new profile, the module served gzipped from
localhost. The machine was at a load of 20 to 200 from other work, so these
are one night's readings, good for a ratio and not a budget.

| entry | Node | Chrome, 9 Mbit/s, cold | Chrome, 9 Mbit/s, caches warm |
|---|---:|---:|---:|
| full | 64 ms | 3,074 ms | 67 ms |
| `panchanga` | 58 ms | 1,679 ms | 45 ms |
| lazy, calendar | 73 ms (median of five) | 1,730 ms | 53 ms |

Three things follow.

- **On a disk, the module's size hardly matters.** Node reaches the first
  answer 6 ms sooner with the small module, because V8 compiles a wasm
  function when it is first called, not when the module loads. A server
  gains nothing here worth an entry of its own.
- **On a network, the download is nearly the whole cold start.** The fetch
  of the module took 2,842 ms of the full entry's 3,074 and 1,449 of
  `panchanga`'s 1,679. The cold start halves because the bytes halve.
- **A returning visitor barely pays either way.** Chrome keeps the code it
  compiled for a module fetched by `compileStreaming` or
  `instantiateStreaming`, at 128 kB or more, served as `application/wasm`
  under one URL ([V8, *Code caching for WebAssembly developers*](https://v8.dev/blog/wasm-code-caching)).
  The glue's `init` streams, so the warm runs above are that cache.

## The design

**A third entry over the two the package has.** `lib/lazy.js` holds two
functions, each a dynamic `import()` of an existing entry, remembered:
`loadCalendar()` is `./panchanga.js` and `loadJyotish()` is `./index.js`. It
imports nothing else but `internal.js`, so `onInternalError` is one list of
hooks for both modules and is there before either loads.

- **Dynamic `import()` is the split every bundler already reads.** Vite and
  webpack split on it by default, and esbuild with `splitting: true`, which
  needs `format: 'esm'`; without it esbuild puts the imported code in the
  one output file ([esbuild, *Splitting*](https://esbuild.github.io/api/#splitting)).
- **Every host keeps its loader.** Each entry already resolves `#native` or
  `#native-panchanga` through the condition that host needs, and those
  loaders run on the first import rather than at the page's start.
  `native.web.js` is the only one that fetches, by streaming, so nothing here
  needs `WebAssembly.compileStreaming` of its own.

Rejected:

- **A jyotish module holding only the difference.** It would be the honest
  meaning of "later", and Rust's wasm target has no dynamic linking to build
  it on: two modules sharing `teistro_core` would need one memory and one
  table across a boundary rustc does not stabilise. So `loadJyotish()` fetches
  the full module, calendar and all, and a page that knows it wants a chart
  should ask for that alone.
- **Fetching the module in `lazy.js` before the import**, to make a failed
  load retryable (C890). It needs a URL per host and a `lazy` loader per
  condition, a second copy of what `LOADERS` is.
- **A loading function on `index.js`**, deferring `initSync` behind a call.
  Every call of the layer would have to wait or refuse, and the promise
  that importing is the whole of the setup is the package's first line.

## Cruxes

- **C890: a module that failed to load stays failed, decided.** Answered 503
  once, the calendar module rejected, and a second `import()` of the entry
  gave the same rejection without a request, though the server would have
  answered. A module whose evaluation rejects keeps that rejection, and
  `native.web.js` awaits its fetch at top level. The HTML standard's newer
  rule that a failed *fetch* of a module is not cached
  ([whatwg/html#10327](https://github.com/whatwg/html/pull/10327)) does not
  reach this: the module's own script fetched, and its evaluation failed.
  So the lazy entry keeps the rejection, says "load the page again" with the
  host's error as its `cause`, and the start check holds that sentence.
- **C891: the layer is shipped twice when both load, accepted.** `panchanga.js`
  is a rewritten copy of `index.js`, so a page that loads both fetches that
  file twice and has two `TeistroError` classes. Bundled, one entry is 270 KB
  and the lazy entry's chunks together 480 KB, the rest being shared chunks,
  so the cost is about 210 KB raw. Sharing it would mean turning 8,700 lines
  into a factory over `native`, which is not worth that for pages that load
  both.

## What a host gets

Each condition has its check, from the installed package, its answer held to
Node's to the bit: Chrome through `default`, and workerd, Deno, Bun (as it
stands, bundled, and compiled) and Next.js's edge runtime (Turbopack and
webpack) through their own. The probe (`browser/probe.mjs`, `lazyProbe`) loads
the calendar, finds a chart refused, loads the jyotish and finds it drawn.

A Worker or an edge route carries both modules, since its bundler has no later
fetch to defer to: the deploy pays for both, and the laziness there is only
that the second is not instantiated until asked.

## The gate

`check-wasm`'s start check (`browser/start.mjs`) runs after the profiles:

- each entry's first answer, the median of five Node processes and a cold and
  a warm Chrome page, printed, under ceilings of 3 s and 30 s. A ceiling
  catches a start gone wrong and no finer, since a runner's clock is its own;
- exactly: a page asking the lazy entry for the calendar fetched the calendar
  module and no other;
- exactly: a module refused once is reported with what to do, twice.

The bundler check adds a `lazy` entry for esbuild, Vite and webpack: what loads
first stays under 4 KB, and both modules are shipped or named. The compiler
reads `lib/lazy.d.ts` from the staged package (`bindings/wasm/typecheck`).

**Not claimed:** Safari and Firefox, which `check-wasm` does not run, and Deno
Deploy, Netlify and Vercel's deployed Edge runtime, as on
[`wasm-binding.md`](wasm-binding.md) step 9. Whether Safari keeps compiled code
as Chrome does was not measured.
