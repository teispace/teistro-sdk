# Contributing to Teistro SDK

Thank you for considering a contribution. This page is the short version;
the long version is `docs/09-guidelines/`.

## Before anything

- Read [`docs/STATUS.md`](docs/STATUS.md) for where the project stands and
  [`docs/QUESTIONS.md`](docs/QUESTIONS.md) for what is decided.
- Significant changes (a new module, an API shape, a default, a binding, a
  pack licence, a dependency policy) start as an RFC in `rfcs/`; see
  [`rfcs/README.md`](rfcs/README.md). Small changes go straight to a pull
  request.
- Accuracy reports need the data that reproduces them: birth data or
  instant, place, settings, provider and tier, the expected value and its
  source (a classical text, another program's output with its version).
  The issue template asks for each.

## The loop

You need the stable Rust toolchain (`rustup` installs it; `rust-toolchain.toml`
selects it) and nothing else. Every repository task is Rust and runs as
`cargo xtask <task>` (ADR-0014).

1. Fork and branch: `feat/<area>-<short>`, `fix/<area>-<short>`,
   `docs/<short>`, `spike/<short>`.
2. Make the change. Documents state their status and date; numbers come
   from a source or a gate; open questions go to `docs/QUESTIONS.md`.
3. Run the fast check locally:

   ```
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo xtask check-docs
   ```

4. Commit with a sign-off (below) and a Conventional Commits subject.
5. Open a pull request using the template: the numbers that moved, the
   change, how it was verified.

The gate list grows with the code (`docs/05-testing/`); every gate is a
`cargo xtask` task.

## Sign-off (Developer Certificate of Origin)

Every commit certifies the [DCO](DCO) with a `Signed-off-by` line carrying
your real name and email:

```
git commit -s
```

Pull requests whose commits lack the sign-off fail the `dco` check.

## Commit messages

Conventional Commits subjects: `feat`, `fix`, `perf`, `docs`, `test`,
`ci`, `build`, `refactor`, `chore`, with an optional scope in parentheses
(`feat(dasha): ...`). The body says what was wrong and how it was found, or
what was added and why it is shaped that way, with the measurement when
there is one. One commit per unit of work. No tool or assistant attribution
trailers.

## Conventions

- British spelling in prose and comments (`behaviour`, `optimise`,
  `licence` the noun).
- Comments explain why and name the measurement, the alternative rejected
  and the defect that motivated the code.
- Generated files are never hand-edited; their header names the generator.
- A claim in a document is derived from its source or checked by a gate.
- A new gate is proven red once before it is trusted.

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).

## Licence

By contributing you agree that your contributions are licensed under the
Apache License 2.0, as stated in the DCO.
