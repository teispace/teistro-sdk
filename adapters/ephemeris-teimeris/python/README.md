# The Teimeris ephemeris, as a Teistro adapter

Two things, and a consumer needs both: the **descriptor** an `ephemeris`
chain takes, and a **typed façade** over the engine's own operations.

The SDK computes with whatever ephemeris it is given. In most cases that
should be a real engine — Teimeris here — and the SDK's own built-in is
the fallback rather than the intended path (ADR-0029).

```python title="adapters/ephemeris-teimeris/python/typecheck/consumer.py"
import teistro
from teistro import Body, Ephemeris
from teistro_ephemeris_teimeris import engine, teimeris
# …
    sdk = teistro.Teistro.open()
    # A real engine, and the SDK's own only if it is not there.
    with sdk.context(
        profile="parashari-classical",
        ephemeris=[teimeris(data_dir="./ephe"), Ephemeris.BUILTIN],
    ) as ctx:
# …
        typed = engine(ctx.engine)
        name: str = typed.tm_body_name(body=0)
```

## The descriptor

It names the engine, not a file. The platform binary is this package's to
find, and it fails **at the call** when it cannot — in the line that names
the engine, rather than when a chart is cast.

A chain is ordered and explicit: the entries are tried in order, and a
context asked for an engine and given the built-in without being told is
the silence this refuses. One entry is one entry.

## Other years

The engine carries **1800 to 2399** in its own binary, and that is what it
reads when nothing else is named. Any other years are a **range** the app
adds: a 600-year block is two files, the planets' (`sepl_*.se1`) and the
Moon's (`semo_*.se1`), and a third (`seas_*.se1`) for Chiron, Ceres,
Pallas, Juno and Vesta. Download a block's files into the app's own
storage, then name them:

```python title="adapters/ephemeris-teimeris/python/typecheck/consumer.py"
    # Ranges beyond the built-in years, by path: these two add 1200 to 1799.
    with sdk.context(ephemeris=[teimeris(files=["cache/sepl_12.se1", "cache/semo_12.se1"])]) as ctx:
```

`data_dir` names a directory of data files read after the built-in ones; `embedded=False` leaves the built-in data out.

Each file is checked against its published size and SHA-256 before it is
read, and one that differs is refused by name: that is what a download cut
short looks like. A set with a hole in it, a block missing between two
others or a planet file without its Moon, is refused with the files that
fill it. Every block's files, sizes and digests are in
[the ranges page](../../../docs/05-testing/teimeris-ranges-measured.md),
read from the same manifest the check uses.


## The typed façade

The engine describes 161 functions of its own. What this adapter can
marshal is a measurement rather than a target, taken by
`cargo xtask engine` and published in
`docs/03-design/engine-passthrough-measured.md`; the façade is generated
from that same reading, so it cannot type an argument the call would
refuse by name.

A method hands back the one value its function answers with — 46 of them
do — rather than an object to index. The few that answer with more get a
record.

Every operation is also reachable untyped, through the SDK alone, as
`sdk.engine.call(name, args)`. The façade gives the names back; it does
not gate the route.

## Licence

**AGPL-3.0-only.** The library this package ships links Teimeris, which is
AGPL, and an Apache-2.0 library that links AGPL code is an AGPL work. The
Teistro SDK is Apache-2.0 and never links it: it loads this adapter at run
time, so the licence stays on this side of the boundary. The Rust crate
beside this package keeps `license = "Apache-2.0"` for its own source,
which is a different thing from what is distributed here.
