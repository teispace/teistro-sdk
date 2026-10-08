#!/bin/sh
# Runs `cargo xtask <task>` for each argument inside Alpine, the platform
# a musl row ships for (`xtask/src/platform.rs`). The workflows call it
# through `docker run` on the Linux runner of the row's architecture, the
# checkout mounted at the working directory, so the build is native and
# what it writes lands in the runner's `target/`.
#
#   docker run --rm -v "$PWD:/w" -w /w -e TEISTRO_STRICT alpine:3.21 \
#     sh xtask/alpine.sh "package linux-x64-musl" check-package
set -eu

# A C compiler for the C bundle's smoke test and the crates that build C,
# Node 22 and npm for the Node packages, Python and its venv for the
# Python ones. Dart has no musl build, so it is not here.
apk add --no-cache bash build-base curl git nodejs npm python3

# Rust through rustup, whose host on Alpine is the musl target itself.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
  sh -s -- -y --quiet --profile minimal --default-toolchain stable
. "$HOME/.cargo/env"
cargo install --quiet --locked "cargo-auditable@$(cat xtask/cargo-auditable.version)"

# The checkout belongs to the runner's user, not the container's root.
git config --global --add safe.directory "$PWD"

# Each argument is one task with its own arguments, split on spaces.
for task in "$@"; do
  # shellcheck disable=SC2086
  cargo xtask $task
done
