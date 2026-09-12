"""A consumer that knows nothing but the published names.

`cargo xtask check-package` installs the staged package into a throwaway
project, runs `teistro-install` against the release's own archive, and
runs this. It asserts the same four facts the C smoke test prints, so a
package that loads but answers differently fails here rather than in the
field.
"""

from teistro import Body, Calendar, Teistro, date


def main() -> None:
    teistro = Teistro.open()
    assert teistro.abi == 1, f"ABI {teistro.abi}"
    print(f"abi {teistro.abi}")
    print(f"sdk {teistro.version}")

    with teistro.context(test_provider=True) as ctx:
        bs = ctx.calendar.convert(date(Calendar.GREGORIAN, 2015, 4, 14), Calendar.BIKRAM_SAMBAT)
        assert (bs.year, bs.month, bs.day) == (2072, 1, 1), bs
        print(f"bs {bs.year}-{bs.month}-{bs.day}")

        sun = ctx.positions(instants=[2451545.0], bodies=[Body.SUN]).at(0, 0)
        assert 0.0 <= sun.longitude < 360.0, sun.longitude
        print(f"sun {sun.longitude:.4f}")


if __name__ == "__main__":
    main()
