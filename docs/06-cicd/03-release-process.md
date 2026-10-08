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
| `verify` | the whole verify matrix (`verify.yml`, called), every binding's gates on every platform, the wasm package and the ephemeris tiers; `publish` waits for it |
| `build` | a runner a platform, each `cargo xtask package <platform>` then `cargo xtask check-package`, each uploading its own artefacts and manifest |
| `wasm` | one runner, `cargo xtask package wasm` then `cargo xtask check-wasm` (the Node suite through the package's own loader, headless Chrome and Cloudflare's workerd each held to Node bit for bit, the package installed and run), uploading `@teistro/sdk-wasm` |
| `stage` | downloads all six, `cargo xtask package stage` (which refuses a release missing the wasm package as it refuses one missing a platform), and publishes the checksum list into the run's summary |
| `publish` | after `verify` and `stage`: the GitHub release with every archive and `checksums.txt`; then every platform package and `@teistro/sdk-wasm` to npm, then the one that depends on the platform packages; then the Dart package |

Every job runs under `TEISTRO_STRICT`, so a gate that skips part of
itself for a missing tool fails rather than passing on what it did not
check (`01-pipelines.md`).

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
source (2026-09-07), so `docs` can publish on a tag.

**Revised 2026-09-10: the names are reserved now; the credentials still
are not.** This section previously deferred both together, on the ground
that "a credential that exists before it is needed is a credential nobody
is watching". That argument is about credentials and remains right. It was
doing double duty as an argument about *names*, and names are a different
risk: every one of them was still free on 2026-09-10 — checked against
each registry's own API, `teistro` and all twenty-one `teistro-*` crates,
the `@teistro` npm scope, `teistro` on PyPI, `teistro` and
`teistro_flutter` on pub.dev — and a name that is free is a name anyone
may take. A competing Rust astrology SDK took 866 stars in five weeks with
a `cargo add` line
(`01-research/competitive-analysis/02-developer-market.md`); a naming
collision would cost the SDK its whole identity across four registries at
once, and unlike a credential the loss is not recoverable by rotating
anything.

So the two are separated:

| | when | how |
|---|---|---|
| the npm `@teistro` scope | **done 2026-09-10** | created as a free public organisation in the web console; creating it reserved the whole scope, and `@teistro/sdk@0.0.0` was published into it as a pointer so `npm i` leads somewhere rather than 404ing |
| `teistro` and `teistro-*` on crates.io | **done 2026-09-10** | twenty-two `0.0.0` stubs — the umbrella and every crate that exists in the workspace — each saying it is a placeholder and pointing at `STATUS.md` and the roadmap. crates.io rate-limits a new crate to one per ten minutes after a burst of five, so the set takes about three hours to publish and the script is re-runnable, skipping what is already held |
| module names not yet written (`teistro-dasha`, `teistro-western`, `teistro-render-svg`, …) | at the release | **deliberately not held.** crates.io's placeholder policy covers a crate you are actively working on; a name for code that does not exist is the squatting it objects to. They are claimed as they are built |
| `teistro` on PyPI | not yet | no credential on this machine; the name was free on 2026-09-10 |
| pub.dev | at the release | pub.dev discourages placeholder packages and may remove one; the name is watched instead |
| `com.teispace` on Maven Central | at the release | a namespace is verified by a DNS TXT record on `teispace.com`, which nobody else can add, so it needs no holding |
| any standing publishing credential | at the release | unchanged, and preferably never — see trusted publishing below |

The rule the original sentence was protecting survives intact: **no
long-lived publishing credential exists before the release.** A token
minted, used once and revoked within the hour is not a credential nobody
is watching.

Every registry publishes by the runner's OIDC token where it can, since a
workflow that authenticates by who it is cannot leak a secret it does not
hold.

| registry | how the `publish` job authenticates | what the maintainer sets up once |
|---|---|---|
| npm | trusted publishing, through npm 11.5.1 or later, which the job installs | a package's first publish takes `NPM_TOKEN`, minted for it and revoked after, because a trusted publisher is configured on a package that exists. Then each of the seven packages names `release.yml` and the `release` environment as its trusted publisher, and the secret is deleted |
| PyPI | trusted publishing through `pypa/gh-action-pypi-publish`, which also attaches a PEP 740 attestation to each file | a pending publisher for `teistro` naming `release.yml` and the `release` environment, which PyPI allows before the project exists |
| pub.dev | automated publishing by the OIDC token | automated publishing enabled for `teistro`, on tags `v{{version}}` |
| Maven Central | a Portal user token, since Central has no trusted publishing (none found 2026-10-08); `cargo xtask publish maven` signs every file with PGP, bundles the layout and uploads it to the Portal's publisher API, the token and passphrase on `curl`'s and `gpg`'s standard input | the `com.teispace` namespace verified by its TXT record; a PGP key whose public half is on `keys.openpgp.org` and whose fingerprint is in `SECURITY.md`, imported from `MAVEN_GPG_PRIVATE_KEY`; `MAVEN_CENTRAL_USERNAME`/`MAVEN_CENTRAL_PASSWORD` minted for the release and revoked after. The first release uploads `USER_MANAGED`, validated by Central and published by hand in the Portal; later ones pass `--automatic` |

`twine upload` exchanges no token, so the step that used it would have
failed for want of a credential. It was replaced before the first release
could find out.

## Provenance

**What each file is made of** is said twice (`xtask/src/sbom.rs`):

- **Inside the file.** Every shared library and Node addon is built
  through `cargo auditable`, pinned in `xtask/cargo-auditable.version`,
  which links the crates it carries into a `.dep-v0` section. A scanner
  such as `cargo audit bin` or Trivy can then read a file on its own.
  `package` refuses a file without that section. The static library
  carries none, because `cargo auditable` writes only into what is
  linked, and the bundle's bill covers it.
- **Beside it.** A CycloneDX 1.5 bill of materials per artefact, written
  from `cargo tree` for the artefact's own package and target, with each
  registry crate's checksum from `Cargo.lock`. Each platform has two: the
  library's (`teistro-<version>-<platform>-library.cdx.json`, also inside
  the C bundle) and the addon's (also inside the platform's npm package).
  The wasm package carries its module's. A bill has no timestamp and
  sorts every list, so a rebuild of the same commit writes the same
  bytes. It is attached to the release and listed in `checksums.txt`.
  `cargo tree` is used rather than `cargo metadata`, because metadata
  resolves features for the whole workspace and would list crates that
  another member's features switch on. The library's bill would have
  named `clap`.

**Who built each file** is attested: `actions/attest-build-provenance`
signs every archive, every bill, the manifest and `checksums.txt`, and
records the signature in the public transparency log. Run
`gh attestation verify <file> --repo teispace/teistro-sdk` to check one.

**Advisories** are checked on every push by `cargo deny check` in the fast
check. It reads the RustSec database `cargo audit` reads, and refuses a
yanked crate as well.

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
| Python | `pip install teistro` | the platform wheel pip chose for the host, which carries the library in `teistro/_lib/`; on a host no wheel fits, the source distribution and `teistro-install`, checked against the package's own digest table |
| Dart | `dart pub add teistro` then `dart run teistro:install` | the release, checked against the package's own digest table, written to `.dart_tool/teistro/<version>/` |

Every one of them is installed into a throwaway project and run before it
is published: that is `cargo xtask check-package`, and it runs for every
platform in the `build` job.

An air-gapped machine has two ways in: `dart run teistro:install --from
<archive>` installs from a file it already has, and `$TEISTRO_LIBRARY`
(Dart) or `$TEISTRO_ADDON` (Node) names a library outright. A machine that
builds from source needs neither: `cargo build --release -p teistro-ffi`,
and both loaders find `target/release`.

## Withdrawing

There is no un-publish. A release that is wrong is followed by another
release; the changelog entry for it says what moved and why, under the
same **Numbers** rule as any other entry.
