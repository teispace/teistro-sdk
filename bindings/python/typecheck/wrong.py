"""The usages that must not type-check, each with the error it must raise.

`cargo xtask check-python` runs a type checker over this file on its own
and fails unless every expectation below is reported; the package
excludes it, because a file whose whole purpose is to be wrong would fail
the clean pass.

This is the Python half of Phase 1's exit criterion: a swapped latitude
and longitude does not compile.
"""

from teistro import Body, Calendar, Observer, Teistro, date, local_mean_zone
from teistro._ffi import Altitude, Latitude, Longitude

teistro = Teistro.open()

# expect: Argument "latitude_deg" to "Observer" has incompatible type "Longitude"; expected "Latitude"
Observer(
    latitude_deg=Longitude(85.324),
    longitude_deg=Longitude(85.324),
    altitude_m=Altitude(1400),
)

# expect: Argument "latitude_deg" to "Observer" has incompatible type "float"; expected "Latitude"
Observer(
    latitude_deg=27.7172,
    longitude_deg=Longitude(85.324),
    altitude_m=Altitude(1400),
)

# expect: Argument 1 to "local_mean_zone" has incompatible type "Altitude"; expected "Longitude"
local_mean_zone(Altitude(1400))

# expect: Argument 1 to "date" has incompatible type "Body"; expected "Calendar"
date(Body.SUN, 2015, 4, 14)

# expect: List item 0 has incompatible type "str"; expected "float"
with teistro.context(test_provider=True) as ctx:
    ctx.positions(instants=["2451545.0"], bodies=[Body.SUN])

# expect: Argument 2 to "convert" of "CalendarArea" has incompatible type "Body"; expected "Calendar"
with teistro.context() as other:
    other.calendar.convert(date(Calendar.GREGORIAN, 2015, 4, 14), Body.SUN)

# An area is reached by its own name, so a misspelling is an error rather
# than a call that fails at run time.
# expect: "Context" has no attribute "calender"
with teistro.context() as misspelt:
    misspelt.calender.is_leap(Calendar.GREGORIAN, 2024)
