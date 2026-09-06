# Pipelines

Status: `built`, 2026-09-06.

Four workflows, and one rule that decides which of them a change waits
for: **the fast check runs on every push and needs the Rust toolchain and
nothing else** (ADR-0014). Everything that needs a C compiler, a Node, a
Dart, or five machines runs on a schedule or before a release, because a
contributor should see red within minutes and a nightly that is always
red teaches people to ignore it.

| workflow | when | what it proves |
|---|---|---|
| [`fast-check`](../../.github/workflows/fast-check.yml) | every push to `main`, every pull request | format, lint, the dependency policy, the workspace's tests, and every gate one toolchain can run; on a pull request, that every commit is signed off |
| [`verify`](../../.github/workflows/verify.yml) | nightly, on demand, on a tag | the bindings on five platforms: the C header against a C compiler, the Node and Dart bindings against the real library, the two against each other, and the packages installed into throwaway projects and run |
| [`hash-matrix`](../../.github/workflows/hash-matrix.yml) | nightly, on demand | the same source computes the same numbers on another architecture, value by value |
| [`release`](../../.github/workflows/release.yml) | a `v*` tag, or a dispatch that publishes nothing | five platforms built, merged, staged and published |

## The fast check

Format, `clippy -D warnings`, `cargo deny check`, `cargo test
--workspace`, and then every gate in `xtask`:

`check-docs`, `check-fixtures`, `check-catalogue`, `check-calendars`,
`check-time`, `check-accuracy`, `check-intl`, `check-ffi`, `check-lints`,
`check-versions`.

Each one exists because of a failure that is easy to make and invisible to
a reader; the comment on the task names that failure. The newest,
`check-versions`, holds the workspace, both package manifests, the
platform packages and the API description to one version, because a
release where they disagree ships a library that refuses its own generated
types at load time.

## Verify

The gates that need another toolchain, on every platform the SDK ships
for: `check-c`, `check-node`, `check-dart`, `check-parity` and
`check-package`. The matrix is the one in
[`02-build-matrix.md`](02-build-matrix.md), which is generated from the
same table the packager builds from — a platform is added by adding a row
in `xtask/src/platform.rs`, and the workflow's matrix is that table
written out.

A missing toolchain skips its own gate and says so. A skip is not a pass:
the line names the tool it wanted, and the release's own run has every
toolchain installed, so nothing reaches a release having only ever been
skipped.

## The hash matrix

Described in the [README](README.md) and measured in
[`05-testing/01-quality-bar.md`](../05-testing/01-quality-bar.md). It is
the determinism criterion, not a build gate: the two Linux architectures
must agree value for value, and macOS is reported against them because
its maths library rounds differently in the last place.

## Release

Four jobs, described in [`03-release-process.md`](03-release-process.md).
Every step is `cargo xtask ...`, so the release that runs on a runner is
the release that runs on a laptop; the workflow is the schedule and the
credentials, and nothing else.

## Cost

CI on a public repository costs nothing in money and something in
patience, which is the budget being spent. The fast check is minutes; the
nightly matrix is five machines and runs while nobody is waiting; the
release matrix runs when a tag is pushed. Nothing runs the full matrix on
a pull request, and nothing publishes without a tag.
