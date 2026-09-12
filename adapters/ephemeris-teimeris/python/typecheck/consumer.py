"""What this package's README tells a consumer to write, held to mypy at
the package's own strictness.

It is type-checked and never run: `cargo xtask check-python` checks this
package, so a README snippet that stopped type-checking -- a renamed
descriptor, a moved façade, a changed chain -- fails a gate rather than
misleading a reader. Running it would need the engine's data, which a
checkout does not have.
"""

from __future__ import annotations

import teistro
from teistro import Body, Ephemeris

from teistro_ephemeris_teimeris import engine, teimeris


def main() -> None:
    """The README's own shape."""
    sdk = teistro.Teistro.open()

    # A real engine, and the SDK's own only if it is not there.
    with sdk.context(
        profile="parashari-classical",
        ephemeris=[teimeris(data_dir="./ephe"), Ephemeris.BUILTIN],
    ) as ctx:
        # The operations the SDK names, at the areas it groups them by.
        sky = ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
        print(f"the Sun is at {sky.at(0, 0).longitude}")

        # And the engine's own, typed by the façade this package carries:
        # one value comes back as itself, and more than one as a record.
        typed = engine(ctx.engine)
        name: str = typed.tm_body_name(body=0)
        seconds: float = typed.tm_delta_t(jd_ut1=2451545.0)
        version = typed.tm_version()
        print(f"{name}, {seconds} seconds, engine {version['major']}")
