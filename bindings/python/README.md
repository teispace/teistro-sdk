# The Python binding

Status: `built`, 2026-09-08.

Everything but four files is rendered from `idl/api.json` by `cargo xtask
gen ffi` and held equal to the boundary crates by `cargo xtask check-ffi`,
exactly as the Node and Dart bindings are. The package has **no runtime
dependencies**: it loads the shared library the release already builds
through `ctypes`, which is in the standard library, so there is no
compiler to run and no wheel per interpreter version.

| file | what it is | written by |
|---|---|---|
| `teistro/catalogue.py` | every enum as an `IntEnum` carrying the id the C boundary uses and the key every pack and fixture spells; a catalogue kind gains `UNKNOWN`, so a member from a newer library is a value and not an exception | the generator |
| `teistro/_ffi.py` | the `ctypes` declarations that match the C header name for name, the branded quantities, a frozen dataclass per boundary struct with its own marshalling, the exception, the struct sizes and the context handle | the generator |
| `teistro/_blob.py` | one decoder per result blob, reading the `TSRB` layout into `memoryview` columns over the blob's own bytes | the generator |
| `teistro/messages.py` | the typed accessors: every message of the SDK's locale as a method of its parameters, every catalogued entity as its forms (`cargo xtask gen intl`) | the generator |
| `teistro/__init__.py` | the layer a consumer uses: finding the shared library and checking its build, the defaults, JSON both ways, and the conveniences a generator cannot know are wanted | by hand |
| `teistro/_host.py` | the port adapter: an ephemeris written in Python bound into the vtable through `ctypes.CFUNCTYPE` trampolines | by hand |
| `teistro/_install.py` | the installer: where a prebuilt library comes from, the digest it must have, and where it is written | by hand |
| `teistro/_prebuilt.py` | the release the installer fetches from and the digest of each platform's library; empty in a checkout, written when a release is staged | the release |
| `tests/` | the surface end to end, the decoders against blobs the library produced, and every struct's size against the library that was built | by hand |
| `example/teistro_example.py` | the code this README shows, run by the gate so the two cannot drift | by hand |
| `parity.py` | this binding's half of the parity report, which `cargo xtask check-parity` compares with the Node and Dart bindings' | by hand |
| `typecheck/wrong.py` | the usages that must not type-check, each with the error it must raise | by hand |

## Installing it

```sh
pip install teistro
teistro-install
```

The package carries no binaries: a wheel that shipped one for every
platform would make every consumer download all of them. `teistro-install`
fetches the shared library for this machine from the release this
package's version was cut from, checks it against a digest recorded when
it was built, and writes it to `.teistro/<version>/`, which is one of the
first places `Teistro.open()` looks.

The download is refused, and nothing is written, when the bytes are not
the ones that were built. On a machine with no network, install from a
file you already have:

```sh
teistro-install --from libteistro_ffi-0.1.0-linux-x64.so.gz
```

`Teistro.open()` looks at `$TEISTRO_LIBRARY` first, then in the package's
own `_lib/` directory, then at what the installer wrote, then in this
repository's build output, and finally asks the platform's loader for the
bare name. Building from source needs none of it: `cargo build --release
-p teistro-ffi`.

## Using it

Six runnable programs live in [`example/`](example/), and
`cargo xtask check-python` runs every one, so none of them can drift from what
the binding does. They are meant to be read in order — a quickstart, a
birth chart, a panchanga, a calendar page, a year of the sky, and an
ephemeris of your own — and [`example/README.md`](example/README.md) says
what each is really teaching.

```sh
cargo build --release -p teistro-ffi
cd bindings/python
TEISTRO_LIBRARY=../../target/release/libteistro_ffi.dylib \
PYTHONPATH=. python3 example/quickstart.py
```

**On Windows, run Python in UTF-8 mode** — `PYTHONUTF8=1`, or
`python -X utf8` — whenever you print what this SDK returns. The console's
default encoding there is cp1252, and `print` of a Devanagari string
raises `UnicodeEncodeError` before a character reaches the screen; the
string itself was never the problem. PEP 540's UTF-8 mode is the
documented answer and becomes Python's default in 3.15. `cargo xtask
check-python` sets it, so the examples run there as they do anywhere.

The one thing to know before writing anything real is in
[`example/birth_chart.py`](example/birth_chart.py): **the canonical
frame is tropical**, because that is what an ephemeris computes. A Vedic
chart asks for a sidereal one and the SDK completes it, naming every step
it applied.

## Types

The package ships `py.typed` and carries its annotations inline, so a
type checker sees the same facts the runtime does and there is no stub
beside the module to drift from it.

A quantity that would otherwise be swappable is its own type:

```python
from teistro import Observer
from teistro._ffi import Altitude, Latitude, Longitude

Observer(
    latitude_deg=Latitude(27.7172),
    longitude_deg=Longitude(85.3240),
    altitude_m=Altitude(1400),
)
```

Passing a `Longitude` where a `Latitude` is wanted, or a bare `float`
where either is, is an error a checker reports —
`typecheck/wrong.py` is the file that proves it — and a value outside the
range the description states is a `ValueError` at run time.

## Positions grids

A decoded column is a read-only `memoryview` over the blob's own bytes,
not a copy, so a large grid costs one allocation. `numpy.asarray` wraps
such a view without copying:

```python
import numpy as np

sky = ctx.positions(instants=jds, bodies=[Body.SUN])
longitudes = np.asarray(sky.decoded.cells.lon)   # no copy
```

numpy is not a dependency; the buffer protocol is.

## Which ephemeris

A context with no ephemeris computes calendars, times and messages;
positions need one. `ephemeris` names it, or names an **ordered chain**
tried in order (ADR-0029):

```python
from teistro_ephemeris_teimeris import teimeris

# A real engine, and the SDK's own only if it is not there.
ctx = sdk.context(ephemeris=[teimeris(data_dir="./ephe"), Ephemeris.BUILTIN])
```

**That is the intended path.** In most cases a consumer should be on a
real engine -- Teimeris, Swiss Ephemeris -- installed as its own package
under its own licence, and `Ephemeris.BUILTIN` is the fallback that makes
a chart compute with nothing else installed. `Ephemeris.TEST` (or the
older `test_provider=True`) selects the analytic test provider, whose
positions are **not astronomy**.

A chain is a caller *saying* they will accept the fallback: one entry is
one entry, and a context asked for an engine and given the built-in
without being told is the silence this refuses. Nothing in the chain
opening is one refusal naming each entry that failed.

An engine brings its own operations with it, beyond the eight the SDK
names, at `ctx.engine` -- and the adapter's package carries a typed façade
over them.

## An ephemeris of your own

Subclass `EphemerisProvider` and give the context one. It is asked once
for a whole grid, never in a loop, and everything but the name, the
bodies and the positions has a default.

```python
from teistro import Body, EphemerisProvider, PositionAnswer, PositionQuery, Teistro


class MyEphemeris(EphemerisProvider):
    name = "my-ephemeris"
    bodies = [Body.SUN, Body.MOON]

    def positions(self, query: PositionQuery) -> PositionAnswer | None:
        cells = query.cell_count
        # Return None for "not in that frame": the SDK then asks in your
        # native frame and completes the rest itself.
        return PositionAnswer(
            lon=[0.0] * cells, lat=[0.0] * cells, dist=[1.0] * cells
        )


with Teistro.open().context(provider=MyEphemeris()) as ctx:
    ...
```

What the provider raises reaches the caller: only a code crosses the C
boundary, so the adapter keeps the exception and re-raises it on the
other side. That matters more here than in a compiled binding — an
exception that escapes a `ctypes` callback prints a traceback and returns
zero, which the port would read as success.

## Running the tests

`cargo xtask check-python` does all of it. By hand:

```sh
cargo build --release -p teistro-ffi
cargo run -p teistro-ffi --example blob_fixtures -- target/tsrb
cd bindings/python
TEISTRO_LIBRARY=../../target/release/libteistro_ffi.dylib \
TEISTRO_FIXTURES=../../target/tsrb \
PYTHONPATH=. python3 -m unittest discover -s tests -t .
```

The type-check step needs `mypy`, and the gate installs it: the version
is pinned in [`typecheck/requirements.txt`](typecheck/requirements.txt)
and goes into a `.venv` beside this file the first time the gate finds
none, so every machine and every runner checks with the same one.
`$MYPY` names one outright where you would rather use your own.
