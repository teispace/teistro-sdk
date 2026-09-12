# The Node binding

Status: `generated`, 2026-09-06.

Everything but two files is rendered from `idl/api.json` by `cargo xtask
gen ffi` and held equal to the boundary crates by `cargo xtask check-ffi`.
The two are `lib/index.js` and its declarations, the ergonomic layer,
which is thin on purpose.

| file | what it is | written by |
|---|---|---|
| `native/src/generated.rs` | the napi glue: a class over the context handle, an object per boundary struct, the enums as the strings the tables name, the calls with their `unsafe` blocks | the generator |
| `lib/catalogue.d.ts`, `lib/catalogue.js` | every enum as a string union with a frozen table beside it, and an id table for the enums a result blob's columns carry | the generator |
| `lib/types.d.ts` | every boundary struct as a readonly interface, with each member's documentation, unit, range and example | the generator |
| `lib/blob.d.ts`, `lib/blob.js` | one decoder per result blob, reading the `TSRB` layout into typed-array views over the blob's own bytes | the generator |
| `lib/messages.js`, `lib/messages.d.ts` | the typed accessors: every message of the SDK's locale as a function of its parameters, every catalogued entity as its forms (`cargo xtask gen intl`) | the generator |
| `lib/index.js`, `lib/index.d.ts` | the layer a consumer uses: where the addon is, validation at the door, defaults, errors with their field and hint, results decoded on first use | by hand |
| `native/src/provider.rs` | the port adapter: an ephemeris written in JavaScript bound into the port's vtable | by hand |
| `test/` | the decoders against blobs the library produced, and the whole surface through the layer | by hand |
| `typecheck/` | a consumer, the layer's declarations and the typed accessors at maximum strictness, where every wrong usage is a compile error the file asserts, a swapped latitude and longitude among them | by hand |
| `parity.mjs` | this binding's half of the parity report, which `cargo xtask check-parity` compares with the Dart binding's | by hand |
| `packaging/consumer.mjs` | a consumer that imports the published package by name, run by `cargo xtask check-package` inside a project that installed it | by hand |

A catalogue member is its full key everywhere a string names it
(`'graha.SUN'`), which is what packs, fixtures and serialised results
carry; any other enum member is its name in kebab case (`'invalid-arg'`).
A field the library may not fill is an optional property, which is what
the addon carries on both sides.

## Installing it

```sh
npm install @teistro/sdk
```

The addon is a prebuilt binary, and it is not in this package. A release
publishes one package per platform — `@teistro/sdk-linux-x64`,
`@teistro/sdk-darwin-arm64`, and so on — and this package depends on all
of them as optional dependencies, so npm installs exactly the one that
matches the host and skips the rest. Nothing is compiled at install time
and there is no install script.

The loader looks for the addon in three places, in order: `$TEISTRO_ADDON`
when it names one, the platform package npm installed (`platformPackage()`
says which that is), and this repository's own `cargo build` output. When
it finds none, the error names the package that was wanted and the command
that builds one.

A host no release covers — musl today, anything else tomorrow — installs
no platform package. Build the addon from source (`cargo build --release
-p teistro-node`) and point `$TEISTRO_ADDON` at it.

## Using it

Six runnable programs live in [`example/`](example/), and
`cargo xtask check-node` runs every one, so none of them can drift from
what the binding does. Start with
[`example/quickstart.mjs`](example/quickstart.mjs):

```js
import { Body, Calendar, Context, at, date, ianaZone } from '@teistro/sdk';

const ctx = new Context({
  profile: 'nepali-default',
  locale: 'ne-Deva-NP',
  ephemeris: 'builtin',
});
const bs = ctx.calendar.convert(date(Calendar.Gregorian, 2015, 4, 14), Calendar.BikramSambat);
const sky = ctx.positions({ instants: [2451545.0], bodies: [Body.Sun, Body.Moon] });
console.log(bs.year, sky.at(0, 0).longitude, sky.provenance.settings_hash);
ctx.dispose();
```

Then read them in order — a birth chart, a panchanga, a calendar page, a
year of the sky, and an ephemeris of your own. The one thing to know
before writing anything real is in
[`example/birth_chart.mjs`](example/birth_chart.mjs): **the canonical
frame is tropical**, so a Vedic chart asks for a sidereal one and the SDK
completes it.

A context frees its native memory when it is collected, so `dispose()` is
the explicit form rather than the only one (ADR-0007). `using ctx = new
Context(...)` calls it for you where the runtime has explicit resource
management — that is **Node 24 and above**, so this package's own tests
use `try`/`finally` instead and so should anything that has to run on the
Node 20 this package supports.

## Which ephemeris

A context with no ephemeris computes calendars, times and messages;
positions need one. `ephemeris` names it, or names an **ordered chain**
tried in order (ADR-0029):

```js
import teimeris from '@teistro/ephemeris-teimeris';

const ctx = new Context({
  // A real engine, and the SDK's own only if it is not there.
  ephemeris: [teimeris({ dataDir: './ephe' }), 'builtin'],
});
```

**That is the intended path.** In most cases a consumer should be on a
real engine — Teimeris, Swiss Ephemeris — installed as its own package
under its own licence, and the SDK's `'builtin'` is the fallback that
makes a chart compute with nothing else installed. `'test'` (or the older
`{ testProvider: true }`) selects the analytic test provider, whose
positions are **not astronomy**.

A chain is a caller *saying* they will accept the fallback: one entry is
one entry, and a context asked for an engine and given the built-in
without being told is the silence this refuses. Nothing in the chain
opening is one refusal naming each entry that failed.

An engine brings its own operations with it, beyond the eight the SDK
names, at `ctx.engine` — and the adapter's package carries a typed façade
over them.

## An ephemeris of your own

```js
const ctx = new Context({
  provider: {
    name: 'my-engine',
    bodies: [Body.Sun, Body.Moon],
    positions(request) {
      // One call for the whole grid, never a loop. One value per cell,
      // instants outermost: cell `i * bodies.length + j` is instant `i`,
      // body `j`.
      const cells = request.jds.length * request.bodies.length;
      return { lon: new Float64Array(cells), status: new Int32Array(cells) };
    },
  },
});
```

Returning nothing means "not in that frame": the SDK asks again in the
provider's own frame and completes the rest itself, so an engine that
computes equatorial positions gets the ecliptic ones for free, each step
stamped in the result's provenance. A provider that throws reaches the
caller as a `TeistroError` with its own sentence, because only a code
crosses the C boundary and the binding knows the message.

The adapter is `native/src/provider.rs`, hand-written like the layer: the
architecture puts a port adapter in the ergonomic layer because every
binding wraps its own callback mechanism. What it does is small, because
the port already carries the machinery.

## Running the tests

```sh
cargo xtask check-node
cargo xtask check-parity
```

The first builds the addon, copies it where the loader looks, writes blob
fixtures through the C ABI, runs the tests with Node, and type-checks the
consumer when a TypeScript compiler is on the machine (`npm install
typescript` in `typecheck/`, or set `TSC`). It needs Node, so it runs by
hand and in the nightly matrix; the fast check needs the Rust toolchain
and nothing else (ADR-0014). The second walks one scenario through this
binding and the Dart binding and compares the ninety values they report,
so a difference between the two layers is a failed gate rather than
something a reader has to notice.

Never edit what the generator writes: change the Rust source and
regenerate.
