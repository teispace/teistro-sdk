# `teistro-ephemeris-builtin`

The Teistro SDK's built-in analytic ephemeris. It implements the
ephemeris port like any other provider, needs no data files, no network
and no licence beyond the SDK's own, and is honest about its accuracy:
every tier publishes its measured worst-case error rather than a claim.

The series are the published scientific results — VSOP87 for the planets
(Bretagnon and Francou 1988), ELP/MPP02 for the Moon — ingested by the
SDK's own generator into tables with their citations embedded. See
[`docs/03-design/builtin-ephemeris-measured.md`](../../docs/03-design/builtin-ephemeris-measured.md)
for what truncation costs, measured rather than claimed.

Apache-2.0.
