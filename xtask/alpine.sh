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
# Python ones, GNU tar for the JDK below. Dart has no musl build, so it is
# not here.
apk add --no-cache bash build-base curl git nodejs npm python3 tar

# A JDK for the Java binding and its package (`xtask/temurin-alpine.txt`
# says why Temurin and not Alpine's own), checked against its pinned
# digest before it is unpacked. No Maven: `check-package` excuses its
# Maven arm on a musl row, the native's selection being what differs.
read -r jdk_digest jdk_url <<PIN
$(awk -v arch="$(uname -m)" '$1 == arch { print $2, $3 }' xtask/temurin-alpine.txt)
PIN
[ -n "$jdk_url" ] || { echo "xtask/temurin-alpine.txt pins no JDK for $(uname -m)" >&2; exit 1; }
curl --proto '=https' --tlsv1.2 -sSfL -o /tmp/jdk.tar.gz "$jdk_url"
echo "$jdk_digest  /tmp/jdk.tar.gz" | sha256sum -c -
mkdir -p /opt/jdk
tar -xzf /tmp/jdk.tar.gz -C /opt/jdk --strip-components=1
rm /tmp/jdk.tar.gz
export JAVA_HOME=/opt/jdk
export PATH="$JAVA_HOME/bin:$PATH"

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
