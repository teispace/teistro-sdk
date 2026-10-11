# The documentation site's build and deploy

Status: `built`, 2026-09-06.

The site is `site/` (ADR-0012): Fumadocs on Next.js, exported as static
files, with the API reference generated from the same description every
binding is generated from.

## What is generated

`cargo xtask gen ffi` writes `site/content/docs/reference/`: one page per
entry point, grouped by the boundary's own source files, plus a page of
structs and a page of enums, plus the `meta.json` files that order the
sidebar. There were 42 pages when this was written, and the count is
whatever the boundary has.

Each entry point's page carries what the description knows and nothing
invented: the doc comment, a tab per binding with the operation a
consumer calls to reach it and the C declaration beside them, a
parameter table with each parameter's role, unit, range and example,
what the call hands back, the blob schema it fills, its safety contract
and the file it is declared in.

The tabs are the one part the description cannot know, since the
ergonomic layers are written by hand, so `gen ffi` reads them
(`xtask/src/areas.rs`, `reach`): which members of the Node layer call the
entry point, as canonical `area.operation` paths, and how each binding's
parity runner spells each path, which is the spelling that compiles and
runs (Java's `(root).dispose` is `ctx.close`). Rust's runner
lists paths without members, so Rust's tab is the façade's rule,
`sdk.<area>().<operation>`, refused unless the façade defines that
operation. An entry point no operation calls by name says so: a
binding calls it itself or offers it outside a context. Until
2026-10-12 the tabs were derived by a naming rule, and said Node's
addon called `context.calendar_convert`, which napi exposes as
`calendarConvert`.

The reference documents the **C ABI** rather than one binding's
ergonomics, because that is the surface every binding is generated from:
a page describing an entry point describes what every language is really
calling. The hand-written layers over it are documented in the guides.

## The three gates

| gate | what it holds | where it runs |
|---|---|---|
| `cargo xtask check-ffi` | the checked-in reference equals what the generator produces, and nothing else is in the directory | the fast check, on every push |
| `cargo xtask check-docs` | the site's own text obeys the forbidden-terms rule, like every other published word | the fast check |
| `cargo xtask check-site` | the site builds, and every generated page was rendered | `verify` and `docs`, both of which have a Node |

`check-ffi` needs the Rust toolchain and nothing else, which is why the
reference cannot drift even on a machine with no Node. `check-site` is
the other half: a doc comment is prose written for Rust, and it holds
braces and angle brackets that MDX reads as code. The emitter escapes
them and joins each paragraph onto one line — MDX reads a `{` at the
start of a line as an expression, even inside what a Markdown reader
would call code — and this gate is what proves the escaping worked.

## Deploying

[`docs.yml`](../../.github/workflows/docs.yml) builds the site on every
push to `main` and on any pull request that touches it or what it is
generated from, and publishes to GitHub Pages **on a tag only**: the site
people read is the site of the version they are reading about, not of
whatever landed on `main` this morning.

The publish job needs Pages enabled for the repository with GitHub
Actions as its source; nothing else is configured, and no credential is
held anywhere — the deploy takes the runner's OIDC token.

## Executed examples

A guide holds no code of its own (`01-research/platform/10-docs-site.md`,
requirement 2). A program block names the file it was cut from in its
title, ```` ```js title="bindings/node/example/quickstart.mjs" ````,
which the site prints above the block, and `cargo xtask check-guides`
(fast-check) holds the block to that file: each run of its lines between
elisions (`// …`, `# …`) must appear there in order and unbroken, but for
the file's comments and blank lines. The file must be a shared example,
which `check-parity` runs in every binding, or one `xtask/src/guides.rs`
lists with the gate that type-checks or builds it: the adapter's
consumers, which need the engine's data to run, and the Flutter app,
which needs a device. The one rewrite allowed is an example's import of
its package's source (`'../lib/index.js'`), shown as a consumer writes it
(`'@teistro/sdk'`), and the list of those is in the same file.

A block that is not a program on purpose, such as a call's spelling set
beside its other spellings, says `fragment`, and the gate lists each by
page and line. An output block, ```` ```text output="research" ````, is
lines the shared example prints, and `check-parity`, which runs the
examples, holds it to what the example printed. Shell commands, JSON and
POM fragments are not programs and are not checked.

## What is not built yet

- **Versioned docs.** One release is published at a time. Versioning is
  routing, and it waits until there are two releases to route between.
- **Nepali.** The site framework supports it and the product is
  Nepali-first; the guides are English until there are enough of them to
  be worth translating, and the reference is generated from doc comments
  that are English in the source.
- **Interactive examples.** A wasm build of the SDK running in the page
  waits on the wasm binding.
