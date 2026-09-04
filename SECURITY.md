# Security policy

## Reporting a vulnerability

Please do not open a public issue for a security problem. Report it
privately through GitHub's private vulnerability reporting for this
repository (Security tab, "Report a vulnerability"). The maintainers
acknowledge reports within three working days and aim to publish a fix
within ninety days, sooner for anything exploitable.

The SDK performs no I/O of its own; its attack surface is its API inputs,
consumer-supplied providers (callbacks) and data packs. Reports about any
of those are in scope, as are supply-chain concerns about the build and
release pipeline. The threat model is in
`docs/01-research/platform/08-security.md` and the defences in
`docs/02-architecture/10-security-architecture.md`.

## Supported versions

There is no release yet. Once there is, the current minor version and the
previous one receive security fixes.
