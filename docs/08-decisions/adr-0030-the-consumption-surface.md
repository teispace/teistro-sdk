# ADR-0030: The consumption surface, and the engine namespace

Status: accepted (maintainer, 2026-09-11)
Date: 2026-09-11
Extends: ADR-0002 (the agnostic port), ADR-0023 (type safety in every
binding), ADR-0029 (the provider plugin model).
Research: [`01-research/platform/15-provider-plugins-and-targets.md`](../01-research/platform/15-provider-plugins-and-targets.md)

## Context

A context today carries about forty methods on one object: `positions`,
`almanac`, `almanacDay`, `render`, `has`, `transliterate`, `entity`,
`loadPack`, `dateOf`, `fixedOf`, `convert`, `weekdayOf`, `monthLength`,
`isLeap`, `resolve`, `civilOf`, and so on. Every area of the SDK is flat
against every other, and the list grows with each phase — Phase 5 adds
dashas and strengths, Phase 6 rules and interpretation, Phase 7 five more
traditions.

The maintainer's brief asks for `sdk.<area>.<operation>`, and for the
engine's own functions to be reachable where the SDK has not ported one:

> some core level functions that sdk didn't port/have can be accessed
> whenever necessary as they may need so, for flexibility we include this

The mechanism for that exists and reaches nothing: `ts_ephemeris_manifest`
and `ts_ephemeris_call` are built and wrapped, and the Teimeris adapter
does not claim the route.

## Decision

### 1. The surface is namespaced by area, and the areas are derived

`sdk.<area>.<operation>`, with the areas taken from what the boundary
already records rather than invented. `idl/api.json` carries each
function's `source` module, and the documentation site already groups its
reference by exactly that:

`calendar`, `time`, `intl`, `frame`, `keys`, `positions`, `chart`,
`panchanga`, `engine` — with the context's own lifetime (`close`,
`profile`, `settings`) staying at the root, because it is the object
rather than an area of it.

The bindings' hand-written layer is richer than the 43 boundary functions
and does not map one to one, so a binding may group a helper with the area
it serves rather than with the entry point it calls. What it may not do is
invent an area the boundary does not have: a new namespace means a new
module at the boundary, which keeps four languages and the reference site
agreeing without anyone maintaining a list.

**Namespaces are values, not calls.** Each is built once when the context
is, holds the context, and is frozen. Nothing allocates per call, and a
consumer may destructure one and keep it — `const { panchanga } = sdk` —
which is the ergonomics half of why areas are worth having at all.

### 2. `sdk.engine.*` is the escape hatch, and is named for what it is

Not `sdk.ephemeris.*`. **`engine`** says *this particular engine, not the
portable contract*, and a consumer reading their own code should see the
difference between a call that survives changing provider and one that
does not.

Two routes, both under that name:

```
sdk.engine.names                     // what this engine offers
sdk.engine.signature(name)           // its parameters and their roles
sdk.engine.call(name, args)          // dynamic, any engine that describes itself
sdk.engine.tmCalendarWeekday(…)      // typed, when the adapter ships a façade
```

The dynamic route works for any engine that answers `native_manifest`.
The typed route is generated, and the next decision says by whom.

### 3. The typed façade ships in the **adapter**, not the SDK

This is the point that keeps ADR-0002 intact. The SDK must not know what
functions Teimeris has; if it generated a typed `sdk.engine.tm*` it would
know, and the agnostic port would be agnostic in name only.

So: the **adapter package** carries the generated façade and augments the
SDK's own types with it — a TypeScript declaration merge, a Python
protocol, a Dart extension. Installing `@teistro/ephemeris-teimeris` is
what makes `sdk.engine.tmCalendarWeekday` exist and be typed; without it,
`sdk.engine.call('tm_calendar_weekday', …)` still works and is not typed.

It is generated rather than written. `tools/idl/teimeris.idl` describes
161 functions with each parameter's role, and the SDK's own generators are
already role-driven — "nothing in them names a function of the slice".
The same shape of generator emits the façade.

### 4. Engine functions are **not** hoisted to the root

The brief allows both `sdk.function` and `sdk.engine.function`. This ADR
reads that as: **the SDK's own ported operations at `sdk.<area>.<op>`, the
engine's own at `sdk.engine.<op>`** — and declines to also surface engine
functions at the root.

Hoisting would give two spellings for one call, which is the defect this
project keeps finding under other names, and it would hide the only thing
a consumer needs to know about such a call: that it does not survive
changing the engine. A name that says `engine` is the warning, and moving
it to the root removes the warning while keeping the risk.

### 5. What proves universal is promoted into the port

The escape hatch is an escape hatch. When an engine function turns out to
be wanted by everyone, it becomes an SDK operation with a portable
contract, implemented over the port and available on every provider —
and the engine route keeps working for whoever already wrote against it.

The example that prompted this is already on the right side of the line:
a weekday is `sdk.calendar.weekdayOf(date)`, portable, and no consumer
needs to reach into an engine for it.

## Consequences

- **A surface change before v1, and a large one.** Three hand-written
  binding layers are restructured, four reference surfaces regenerate, and
  every example and doc line moves. Cheap now, breaking later, and it gets
  more expensive with every phase that adds an area.
- **`check-parity` and `check-surface` gain the grouping**, so a binding
  that puts an operation in the wrong area fails a gate rather than a
  review.
- **The engine route becomes real**, which is work in three places: the
  adapter answers `native_manifest` and `native_call` from the engine's
  IDL, a generator emits the façade, and the adapter package ships it.
- **An adapter can now change what type-checks in a consumer's project.**
  That is the intended behaviour — the same as a nodemailer transport
  bringing its own options type — and it means an adapter's version has to
  be as carefully managed as the SDK's.
- A consumer who never plugs an engine sees `sdk.engine.names` come back
  empty and `call` refuse with `CAPABILITY`, which is the same contract as
  every other operation that needs an ephemeris.

## Alternatives considered

**Keep the flat surface.** No migration, no gate changes, and it is what
every binding ships today. Rejected on where it ends: forty methods now,
and the phases that remain add dashas, strengths, rules, interpretation,
five traditions and eleven application modules. A flat object with two
hundred methods is not a surface anyone can hold, and the cost of fixing
it rises every phase.

**Namespace by hand, per binding.** Each ecosystem groups what its users
expect. Rejected because three bindings would drift into three shapes and
the parity gate could not tell a deliberate difference from a mistake.

**Generate the typed engine façade into the SDK.** One package to install,
and the types are there without thinking. Rejected on ADR-0002: the SDK
would name the functions of one engine, and every other engine would be a
second-class citizen of a port that is supposed not to have any.

**A proxy object with a dynamic index signature**, so
`sdk.engine.anything(…)` resolves at runtime. Ergonomic and available for
free in JavaScript and Python. Rejected under ADR-0023: it type-checks
everything, including the misspelling, and Dart and Rust cannot express it
anyway — so it would be a surface that exists in two of five targets.
