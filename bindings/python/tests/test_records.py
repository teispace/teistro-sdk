"""The records the binding hands across the C boundary, field by field.

`test_sizes.py` holds each `ctypes` struct to the size the description
computed; this holds the generated Python record to its struct. A record is
written into its struct and read back, and the trip must give the same
record: a field written but not read (or read from its neighbour) is caught
here rather than as a wrong number in a provider's answer.

Every record makes the trip from its zero value, and the ones the port
carries to a provider written in Python (`03-design/ephemeris-port-and-
adapters.md` §12) make it again with every field set, since a zero cannot
tell two fields apart.
"""

from __future__ import annotations

import ctypes
import dataclasses
import unittest
from typing import Any

from teistro import _ffi
from teistro.catalogue import Body, TimeScale


def _records() -> list[type[Any]]:
    """Every record the generator wrote with a C struct behind it."""
    return [
        record
        for record in vars(_ffi).values()
        if isinstance(record, type)
        and record.__module__ == _ffi.__name__
        and dataclasses.is_dataclass(record)
        and hasattr(record, "_to_c")
        and hasattr(record, "_of")
    ]


def _round_trip(value: Any) -> Any:
    """`value` written into a fresh struct and read back."""
    owned: list[Any] = []
    return type(value)._of(value._to_c(owned))


#: The records whose zero struct is not a value: a date's absent leap is
#: `0xFF`, not zero, so they start from the struct with it marked absent.
_ABSENT_LEAP = 0xFF


def _zero(record: type[Any]) -> Any:
    """The record's zero value, with an absent leap marked absent."""
    if record is _ffi.CalendarDate:
        date = _ffi._CalendarDateStruct()
        date.leap = _ABSENT_LEAP
        return record._of(date)
    if record is _ffi.CivilDateTime:
        instant = _ffi._CivilDateTimeStruct()
        instant.date.leap = _ABSENT_LEAP
        return record._of(instant)
    return record._empty()


_KATHMANDU = _ffi.Observer(
    longitude_deg=_ffi.Longitude(85.324),
    latitude_deg=_ffi.Latitude(27.7172),
    altitude_m=_ffi.Altitude(1400.0),
)

#: Every field off its zero and off its neighbours: a Delta T model by id,
#: the packed models, and a stated Delta T.
_CONVENTIONS = _ffi.Conventions(delta_t_model=3, models=0x0102, delta_t_seconds=69.25)


class EveryRecord(unittest.TestCase):
    def test_every_record_survives_a_trip_through_its_struct(self) -> None:
        records = _records()
        # The records the generator wrote, so a new one is counted here.
        self.assertGreaterEqual(len(records), 25)
        for record in records:
            with self.subTest(record=record.__name__):
                zero = _zero(record)
                self.assertEqual(_round_trip(zero), zero)

    def test_a_trip_writes_the_struct_size_the_library_reads(self) -> None:
        for record in _records():
            with self.subTest(record=record.__name__):
                owned: list[Any] = []
                raw = _zero(record)._to_c(owned)
                if hasattr(raw, "struct_size"):
                    self.assertEqual(raw.struct_size, ctypes.sizeof(raw))


class WhatAProviderIsAsked(unittest.TestCase):
    """The questions the port puts to a provider, with every field set."""

    def test_the_conventions_travel_whole(self) -> None:
        self.assertEqual(_round_trip(_CONVENTIONS), _CONVENTIONS)

    def test_a_position_request_carries_its_grid_its_place_and_its_conventions(
        self,
    ) -> None:
        request = _ffi.PositionRequest(
            scale=TimeScale.TT,
            frame_bits=0x0201,
            speeds=True,
            jds=[2_451_545.0, 2_451_546.5],
            bodies=[Body.SUN, Body.MOON],
            conventions=_CONVENTIONS,
            observer=_KATHMANDU,
        )
        back = _round_trip(request)
        self.assertEqual(back, request)
        self.assertEqual(back.conventions.delta_t_seconds, 69.25)
        # Without a place, none is read back.
        geocentric = dataclasses.replace(request, observer=None, speeds=False)
        self.assertEqual(_round_trip(geocentric), geocentric)

    def test_a_horizon_request_carries_every_convention_of_the_horizon(self) -> None:
        request = _ffi.HorizonRequest(
            body=1,
            kind=2,
            disc=1,
            refraction=1,
            refraction_formula=2,
            disc_size=2,
            radii=1,
            latitude=1,
            dip=2,
            observer=_KATHMANDU,
            from_jd_ut1=2_451_545.0,
            window_days=1.5,
            altitude_deg=-0.8333,
            pressure_hpa=1013.25,
            temperature_c=15.0,
            semidiameter_arcmin=16.0,
            sun_radius_km=695_700.0,
            moon_radius_km=1737.4,
            horizon_altitude_deg=0.25,
            conventions=_CONVENTIONS,
        )
        self.assertEqual(_round_trip(request), request)

    def test_an_obliquity_is_read_back_as_written(self) -> None:
        zero = _ffi.Obliquity._empty()
        fields = {
            field.name: float(index + 1)
            for index, field in enumerate(dataclasses.fields(zero))
            if isinstance(getattr(zero, field.name), float)
        }
        stated = dataclasses.replace(zero, **fields)
        self.assertEqual(_round_trip(stated), stated)


if __name__ == "__main__":
    unittest.main()
