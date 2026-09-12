# ADR-0029: The provider plugin model

Status: accepted (maintainer, 2026-09-11)
Date: 2026-09-11
Extends: ADR-0002 (the ephemeris-agnostic port), ADR-0008 and ADR-0028
(the built-in and how a context selects it), ADR-0019 (licensing).
Research: [`01-research/platform/15-provider-plugins-and-targets.md`](../01-research/platform/15-provider-plugins-and-targets.md)

## Context

ADR-0002 made the port ephemeris-agnostic and ADR-0028 let a context ask
for the SDK's own built-in by name. Neither says how a consumer supplies
**somebody else's** engine, and the survey found the answer is: outside
Rust, they cannot. Both adapters are `rlib`s with no C entry point and no
package for any binding, so a Node, Python or Dart consumer can reach an
engine only by writing a provider in their own language.

The maintainer's brief puts the weight the other way round: **in some 98%
of cases a consumer should be on Teimeris or Swiss Ephemeris**, and the
built-in is the fallback. v1 must cover Rust, Node, wasm, Flutter/Dart and
Python.

The licences make the shape non-negotiable. The SDK is Apache-2.0,
Teimeris and Swiss Ephemeris are AGPL-3.0, and the workspace already
excludes the adapters so AGPL never enters its dependency graph. An
engine can therefore never be linked into the SDK's own artefact.

## Decision

**An ephemeris is a plugin the consumer installs beside the SDK**, in the
sense a transport is a plugin beside nodemailer: its own package, its own
licence, its own release cadence, named in configuration rather than
compiled in.

### Two kinds of adapter, because there are two seams

| kind | ships | loaded by | for |
|---|---|---|---|
| **native** | a shared library exporting a vtable factory | the SDK's own loader | Rust, C, Node, Python, Dart on mobile and desktop |
| **host** | code implementing `positions(request)` in the host language | passed as `provider` | wasm, Flutter web, a remote service, an engine of one's own |

These are not a fast path and a slow path. The port's one required
operation takes a **grid**, so a host adapter pays one callback per batch
— measured at 0.08 µs into Dart, against a whole-chart difference of 11.4
µs versus 10.8. A host adapter is a first-class way to supply an engine,
and it is the *only* way in a browser.

### The loader belongs to the SDK

`ts_provider_load(path, config_json, …)` and `ts_provider_free` at the
boundary. Node, Python and Dart can each load a library themselves, and
three copies of that is how the three bindings once grew three copies of
the provider error policy before `validate` moved to the SDK's side.
One implementation, one place the `unsafe` lives, one error vocabulary,
and **no raw pointer crossing a host language**.

An adapter package therefore ships a platform binary and tells the SDK
where it is. It does not ship binding code.

### A declared fallback chain, and never a silent one

An entry is an **adapter's own descriptor** or the built-in's name:

```js
import teimeris from '@teistro/ephemeris-teimeris';

const sdk = teistro.open({
  ephemeris: [teimeris({ dataDir: './ephe' }), 'builtin'],
});
```

A descriptor rather than a bare string, for four reasons: it fails at
*import* when the package is not installed rather than at runtime when a
chart is cast; it is typed, so the adapter's own configuration is checked
by the consumer's own tooling; it carries that configuration, which
differs per engine and which a string cannot; and it is the shape the
analogy already has, where a transport is a value a module exports rather
than a name the library looks up. `'builtin'` stays a name because it is
not a package — it is a feature of the SDK itself (ADR-0028).

Ordered, explicit, and tried in order. It encodes the brief directly — the
engine when it is there, the built-in when it is not — and the SDK already
records which provider answered in the chart's provenance, so a consumer
can always tell which they got.

Never automatic. A context asked for one ephemeris and given another
without being told is the kind of silence this project keeps removing;
a chain is a caller *saying* they will accept the fallback.

### Refusals name what is missing

A chain that ends unmatched is `UNSUPPORTED` naming each entry and why it
failed — not found, wrong ABI version, data directory missing, refused to
open. A consumer who wanted an engine and got a fallback because their
data path was wrong has been misled by their own tooling; a consumer who
wanted an engine and is told "teimeris: no data at /x; builtin: not
compiled in" can act.

### The licence boundary is documented, loudly

Every adapter package states its licence in its own README, its package
metadata and the SDK's provider documentation, and says what loading it
into a process means for distributing the result. The SDK does not advise
on the consequence — it is not the SDK's to advise on — but it will not
let a consumer discover the licence after shipping.

## Consequences

- **Both adapters gain `crate-type = ["cdylib", "rlib"]`** and a stable
  exported factory. The `rlib` stays: a Rust consumer links directly and
  should not pay a loader.
- **A new boundary entry point and a new failure mode.** Loading is
  `ts_provider_load`, and the ABI version of the loaded adapter is checked
  against the library's own, because a stale adapter against a newer SDK
  is otherwise undefined behaviour rather than an error.
- **The packaging matrix grows** by one artefact per adapter per target
  triple. `check-package` covers the SDK's own packages today and will
  have to cover adapters, which is where the real recurring cost of this
  decision sits.
- **wasm gets the built-in and a host adapter, and not the loader.** There
  is nothing to load. A browser consumer runs on the `compact` tier —
  which exists for exactly this, at about 80 KB — unless they bring a host
  adapter over an engine compiled to wasm or a remote service.
- The built-in stays a cargo feature of the SDK (ADR-0028) rather than
  becoming a plugin. It is the fallback of last resort, it must work when
  nothing else was installed, and it carries no licence a consumer has to
  think about.

### Built so far, and where it stops (2026-09-12)

The **route** is complete: an adapter exports the plugin entry points,
`ts_provider_load` opens one, `ts_context_new_with_provider` builds a
context over it, and all three ergonomic layers reach both. A real
Teimeris computes the Sun at J2000 identically from Node, Dart and
Python, and its own 62 operations come with it.

The **surface** is not yet the one decided above, and that is worth
naming rather than leaving to be noticed. What ships is the primitive
underneath a descriptor:

```js
new Context({ plugin: '…/libteistro_ephemeris_teimeris.dylib',
              pluginConfig: { dataDir: './ephe' } })
```

A **path**, which "A declared fallback chain" rejected for four stated
reasons, and **no chain**. Both gaps have the same cause: a descriptor is
a value *the adapter's package exports*, and there is no package yet —
`import teimeris from '@teistro/ephemeris-teimeris'` has nothing to
import. So the primitive had to come first, and it is what the descriptor
will be built on: `teimeris({ dataDir })` returns the binary its own
package ships plus the configuration, which is exactly a `plugin` and a
`pluginConfig`.

**`plugin` must not survive as a second spelling.** When the packages
arrive, the descriptor and the chain become the surface and this option
is folded into them — `ephemeris` taking an ordered list of descriptors
and `'builtin'` — because two ways to name one ephemeris is the defect
this project keeps finding under other names. It is recorded here so the
fold is a planned step and not a discovery.

## Alternatives considered

**Link the engine into the SDK and select it by name**, the way ADR-0028
selects the built-in. Simplest for a consumer, and refused on licence
alone: an Apache-2.0 library that links AGPL code is an AGPL work, which
would make every consumer's application one too. The exclusion of the
adapters from the workspace exists to prevent exactly this.

**Each binding loads adapters itself**, natively, with no SDK loader.
Fewer moving parts at the boundary. Rejected because it multiplies the
policy by the number of bindings and hands a raw function-pointer table
across a host language, and because wasm still cannot do it — so the
portable route has to exist anyway.

**Only the host seam, and no native loading at all.** Genuinely tempting:
it is uniform across every target, needs no packaging matrix, and the
measurements say the cost is small. Rejected because it puts the engine's
lifetime and thread-safety in the host's hands and requires each ecosystem
to grow its own binding to the same C engine — three copies of the work
the SDK's own C ABI was built to avoid. It remains the answer where
loading is impossible.

**Adapter discovery by convention**, scanning `node_modules` or site
packages for anything named `teistro-ephemeris-*`. Convenient, and
rejected: an ephemeris chosen by what happens to be installed is the
opposite of a chart saying where its numbers came from. The chain is
explicit.
