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
| `lib/index.js`, `lib/index.d.ts` | the layer a consumer uses: where the addon is, validation at the door, defaults, errors with their field and hint, results decoded on first use | by hand |
| `native/src/provider.rs` | the port adapter: an ephemeris written in JavaScript bound into the port's vtable | by hand |
| `test/` | the decoders against blobs the library produced, and the whole surface through the layer | by hand |
| `typecheck/` | a consumer and the layer's own declarations at maximum strictness, where every wrong usage is a compile error the file asserts | by hand |
| `parity.mjs` | this binding's half of the parity report, which `cargo xtask check-parity` compares with the Dart binding's | by hand |

A catalogue member is its full key everywhere a string names it
(`'graha.SUN'`), which is what packs, fixtures and serialised results
carry; any other enum member is its name in kebab case (`'invalid-arg'`).
A field the library may not fill is an optional property, which is what
the addon carries on both sides.

## Using it

```js
import { Body, Calendar, Context } from '@teistro/sdk';

const ctx = new Context({ profile: 'nepali-default', locale: 'ne-Deva-NP' });
const positions = ctx.positions({
  instants: [2451545.0],
  bodies: [Body.Sun, Body.Moon],
});
console.log(positions.at(0, 0).longitude, positions.provenance.settings_hash);
```

A context with no provider computes calendars, times and messages;
positions need one, and `{ testProvider: true }` selects the SDK's
analytic provider for examples and tests.

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
