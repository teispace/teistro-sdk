# The release process

Status: `built`, 2026-09-06.

A release is a tag. Everything else is `cargo xtask`, so the release that
runs on a runner is the release that runs on a laptop, and the workflow
supplies the schedule and the credentials and nothing else.

## One version

The SDK has one version, declared in `[workspace.package]` in the root
`Cargo.toml`. Every crate takes it from there. Three files outside Cargo's
reach repeat it, and `cargo xtask check-versions` — a gate on every push —
holds them to it:

| file | what must agree |
|---|---|
| `bindings/node/package.json` | `version`, and an `optionalDependencies` entry for every platform package at exactly that version |
| `bindings/dart/pubspec.yaml` | `version` |
| `idl/api.json` | `sdk_version`, so a bump without `cargo xtask gen ffi` is caught |
| `bindings/dart/lib/src/prebuilt.dart` | `prebuiltVersion`, the release its installer fetches from |

A release where they disagree ships a library that refuses its own
generated types at load time (`refuseBuild`), after the user has installed
it. The gate catches it on the pull request instead.

Two rules ride along, because they are the same fact in another form:

- The Node package is `private` and the Dart package says `publish_to:
  none` **exactly while** the version is `0.0.0`. Publishing takes a
  deliberate bump and not a slip of the hand.
- A version that is not `0.0.0` has a `## <version>` entry in the
  changelog, because that entry is where "does this move any number" is
  answered.

## Cutting a release

```sh
cargo xtask version 0.1.0        # every manifest, and the platform packages
cargo xtask gen ffi              # the API description and the catalogues
cargo check --workspace          # Cargo.lock
$EDITOR CHANGELOG.md             # rename `## Unreleased` and answer Numbers
cargo xtask check-versions       # says what is still wrong
```

Then a pull request, the fast check, a merge, and:

```sh
git tag -s v0.1.0 -m 'Teistro 0.1.0'
git push origin v0.1.0
```

`cargo xtask version` writes only the files that change and prints them.
It is the one command that has to know the list; nobody else does.

## What the tag starts

| job | what it does |
|---|---|
| `gate` | `check-tag` (the tag is the version the repository carries), `check-versions`, the documentation, FFI and determinism gates, and the changelog's entry for this version |
| `build` | five runners, each `cargo xtask package <platform>` then `cargo xtask check-package`, each uploading its own artefacts and manifest |
| `stage` | downloads all five, `cargo xtask package stage`, and publishes the checksum list into the run's summary |
| `publish` | the GitHub release with every archive and `checksums.txt`; then the five platform packages to npm, then the one that depends on them; then the Dart package |

The platform packages are published **before** the package that depends on
them, because npm resolves an optional dependency at install time and a
consumer who installs in the seconds between would get a package whose
addon does not exist yet.

Publishing runs in a GitHub environment called `release`, so it can be
held for a review, and only on a tag: a `workflow_dispatch` builds,
stages and checks everything and publishes nothing, which is how the whole
chain is rehearsed before a tag exists.

## What is set up, and what is not

GitHub Pages is enabled for the repository with GitHub Actions as its
source (2026-09-07), so `docs` can publish on a tag. Nothing else is:
the npm organisation, the pub.dev account and the publishing credential
are deferred to the release itself, because nothing needs them before it
and a credential that exists before it is needed is a credential nobody
is watching. The names are held by nobody: the `@teistro` scope and
`@teistro/sdk` on npm, and `teistro` on pub.dev, were all free when this
was written.

When the time comes, prefer npm's trusted publishing over a token: the
`publish` job already asks for the `id-token: write` permission it needs,
and a workflow that authenticates by who it is cannot leak a secret it
does not hold. The `NODE_AUTH_TOKEN` in the workflow is the fallback for
a registry that does not support it.

## Provenance

npm packages are published with `--provenance`, which records in a public
transparency log which workflow, at which commit, built the tarball. The
Dart package uses pub.dev's automated publishing, which takes the runner's
OIDC token rather than a credential anyone holds. Both need
`id-token: write`, which is why the publish job asks for it and no other
job does.

The manifest and `checksums.txt` are attached to the release, and the Dart
package carries the digests of the libraries it will fetch, so a download
is checked against a number recorded when the library was built rather
than against the download itself.

## What a consumer installs

| binding | how | where the library comes from |
|---|---|---|
| C | `teistro-c-<version>-<platform>.tar.gz` | the bundle: header, shared and static library |
| Node | `npm install @teistro/sdk` | the platform package npm chose for the host, loaded by name |
| Dart | `dart pub add teistro` then `dart run teistro:install` | the release, checked against the package's own digest table, written to `.dart_tool/teistro/<version>/` |

Every one of them is installed into a throwaway project and run before it
is published: that is `cargo xtask check-package`, and it runs on all five
platforms in the `build` job.

An air-gapped machine has two ways in: `dart run teistro:install --from
<archive>` installs from a file it already has, and `$TEISTRO_LIBRARY`
(Dart) or `$TEISTRO_ADDON` (Node) names a library outright. A machine that
builds from source needs neither: `cargo build --release -p teistro-ffi`,
and both loaders find `target/release`.

## Withdrawing

There is no un-publish. A release that is wrong is followed by another
release; the changelog entry for it says what moved and why, under the
same **Numbers** rule as any other entry.
