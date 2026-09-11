# ADR-0028: Selecting the built-in ephemeris at the boundary

Status: accepted (maintainer, 2026-09-11)
Date: 2026-09-11
Extends: ADR-0008 (which named the built-in ephemeris and required it be
removable) and ADR-0002 (the ephemeris-agnostic port). ADR-0013's
override policy and ADR-0023's type-safety rules are unchanged.

## Context

ADR-0008 says the SDK ships a built-in ephemeris so that "a full chart
computes with nothing but the SDK installed", that it ships in three
tiers as features, and that it is **removable** when a consumer brings
their own provider. It does not say how a caller *asks* for it.

Nothing did. The built-in crate is depended on by the reference adapter
and by `xtask` and by nothing else: it never reaches `crates/ffi`, so
the promise held for a Rust consumer and for no one else. The three
bindings' own examples cast their charts with `TestProvider`, whose
documentation says in as many words that its positions are not
astronomy.

The boundary offers a caller exactly three things today:

| what is passed | what the context gets |
|---|---|
| a `ProviderVtable` | that provider, over the C ABI |
| null, with `TS_CONTEXT_TEST_PROVIDER` | the analytic test provider |
| null, no flag | no ephemeris; positions are `CAPABILITY` |

So the question is narrow: how does a caller who passes no vtable say
**which** of the SDK's own providers it wants, in a way that stays
typed in four languages, survives a second built-in arriving, and can
still refuse when the build does not carry one.

## Decision

**A typed selector on the options, not a flag bit.**

`TsContextOptions` gains an `ephemeris` field carrying a `TsEphemeris`
enum — `NONE`, `BUILTIN`, `TEST` — with `NONE` the zero value and so the
default. A caller who passes a vtable leaves it at `NONE`; the vtable
wins and the selector is ignored, because a caller who supplied a
provider has already answered the question.

`TS_CONTEXT_TEST_PROVIDER` keeps working and is **superseded**: it means
`ephemeris = TEST`, and the rule between them is total rather than a
conflict to report — the selector decides whenever it is not `NONE`, and
the flag decides when it is. One code path computes the answer; the two
spellings are one `resolve` function, so they cannot drift.

Three reasons the selector and not a second flag bit:

1. **They are exclusive and flags are not.** `TEST` and `BUILTIN` cannot
   both be true, and a bitmask invites asking. An enum makes the illegal
   state unrepresentable, which is ADR-0023's whole subject.
2. **A second built-in is already in the plan.** ADR-0021 places
   `ephemeris-de`, the JPL DE reader, in v1.x. A flag bit per provider
   would spend the flag word on a list that is really a choice.
3. **It is what a binding can type.** The IDL emits an `api: enum=` as a
   real enum in every target — a TypeScript union, a Python `IntEnum`, a
   Dart enum — where `flags: u32` is an untyped integer with a comment.

**The built-in is a cargo feature of `teistro-ffi`, on by default.**
`builtin-ephemeris` pulls the crate in at the `standard` tier;
`builtin-compact` and `builtin-full` select another, additively, the
richest winning as the crate's own `cfg` already does. A consumer
building their own shared library for a platform that counts kilobytes
turns the feature off and keeps the vtable path. That is what ADR-0008's
"removable" means for an artefact that cannot be tree-shaken.

**A build without it refuses by name.** Asking for `BUILTIN` where the
feature is off is `UNSUPPORTED`, naming the feature and what to do. It is
not a silent fall back to no ephemeris, and not a silent fall back to the
test provider — either would answer a chart the caller did not ask for.

**The examples move to it.** All three bindings' `birth_chart` examples
cast with the built-in rather than the test provider. An example is
documentation that runs, and one that computed a chart from positions
that are not astronomy taught the wrong thing about what the SDK is.
Their *tests* keep the test provider wherever what is being tested is the
boundary rather than the sky.

## Consequences

- The SDK's central promise becomes true outside Rust. A consumer who
  installs `@teistro/sdk` and nothing else computes a real chart.
- **This is an ABI change.** `TsContextOptions` grows, and `check_size`
  compares sizes exactly, so a caller compiled against the old header
  fails loudly with `SCHEMA_VERSION` rather than reading a short struct.
  That is the mechanism working. Every binding is generated and gated by
  `check-ffi`, `check-surface` and `check-parity`, so the change is
  mechanical and the gates prove it landed everywhere.
- Phase 4's exit becomes reachable: its golden vectors are required "in
  three bindings on both providers", and the second provider now exists
  outside Rust.
- The shipped artefacts carry the `standard` tier unless a package says
  otherwise, which is the first time a tier choice has had a consumer
  visible consequence. `SIZES` will say what each binding's artefact
  costs, per ADR-0008's "features and packages".
- A context can now be built with no vtable and still compute, so the
  no-ephemeris path stops being the common case in tests and starts
  being what it is: a deliberate configuration a caller can still ask
  for, and which still refuses with `CAPABILITY`.

## Alternatives considered

**A second flag bit, `TS_CONTEXT_BUILTIN_EPHEMERIS`.** Cheapest: no ABI
change at all, since `flags` already exists. Rejected because the states
are exclusive and a bitmask cannot say so — a caller setting both would
have to be given a precedence rule or an error, and both are worse than
a field that cannot express the question. It also does not scale to
`ephemeris-de`.

**A provider *name* as a string.** Maximally extensible, and it reads
well (`ephemeris: "builtin"`). Rejected on ADR-0023 grounds: a string is
a typo a caller finds at runtime, and the binding surfaces would have to
carry a hand-written union to type it. The enum is generated from one
declaration.

**Registering the built-in through the same `ProviderVtable` the adapter
uses**, so the boundary has exactly one path. Rejected because the vtable
exists to carry a provider across a C ABI from *another library*, and
paying its indirection and its error-code round trip to reach a provider
that is already linked into this one would be cost with no buyer. The
port stays agnostic; what changes is that the SDK now admits to owning
one of the implementations.

**Shipping a package per tier from the start** (`@teistro/sdk-compact`
and so on). Deferred rather than rejected: the feature makes it possible,
and what decides it is what the artefacts actually measure, which `SIZES`
does not yet publish. One package at `standard` until then.
