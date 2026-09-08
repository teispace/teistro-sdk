"""The README's example, kept honest.

`cargo xtask check-python` runs this file, so the README cannot drift
from what the binding does.
"""

from teistro import Body, Calendar, Teistro, at, date, iana_zone, intl


def main() -> None:
    teistro = Teistro.open()
    print(f"Teistro {teistro.version}, ABI {teistro.abi}")

    with teistro.context(
        profile="nepali-default", locale="ne-Deva-NP", test_provider=True
    ) as ctx:
        # 14 April 2015 is 1 Baisakh 2072 BS.
        bs = ctx.convert(date(Calendar.GREGORIAN, 2015, 4, 14), Calendar.BIKRAM_SAMBAT)
        era = bs.era.key if bs.era is not None else ""
        print(f"{bs.year}-{bs.month}-{bs.day} {era}")

        # A Kathmandu birth time, with the metadata a stored chart keeps.
        resolved = ctx.resolve(
            at(date(Calendar.GREGORIAN, 1986, 1, 1), hour=0, minute=20),
            iana_zone("Asia/Kathmandu"),
        )
        print(
            f"JD {resolved.instant_jd_utc:.6f} UTC, "
            f"{resolved.offset_seconds} s, tzdb {resolved.tzdb_version}"
        )

        # The Sun and the Moon at J2000, in the SDK's canonical frame.
        sky = ctx.positions(instants=[2451545.0], bodies=[Body.SUN, Body.MOON])
        print(f"the Sun at {sky.at(0, 0).longitude:.4f} degrees")

        # A message in the context's locale, by its typed accessor, and an
        # entity's name in that locale.
        print(
            ctx.messages.sdk.reason.graha_in_bhava(
                graha=intl.GrahaKey.JUPITER, bhava=7
            )
        )
        sun = ctx.entity("graha.SUN")
        print(f"{sun.name} {sun.glyph}")


if __name__ == "__main__":
    main()
