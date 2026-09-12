# The Teimeris ephemeris, as a Teistro adapter

Two things, and a consumer needs both: the **descriptor** an `ephemeris`
chain takes, and a **typed façade** over the engine's own operations.

The SDK computes with whatever ephemeris it is given. In most cases that
should be a real engine — Teimeris here — and the SDK's own built-in is
the fallback rather than the intended path (ADR-0029).

```dart
import 'package:teistro/teistro.dart';
import 'package:teistro_ephemeris_teimeris/teistro_ephemeris_teimeris.dart';

final sdk = Teistro.open();
final ctx = sdk.context(ephemeris: [teimeris(dataDir: './ephe')]);
ctx.engine.tmBodyName(body: 0); // 'Sun'
```

The façade is an **extension**, so importing this package is what makes
the names exist; there is nothing to wrap.

## The descriptor

It names the engine, not a file. The platform binary is this package's to
find, and it fails **at the call** when it cannot — in the line that names
the engine, rather than when a chart is cast.

A chain is ordered and explicit: the entries are tried in order, and a
context asked for an engine and given the built-in without being told is
the silence this refuses. One entry is one entry.

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
