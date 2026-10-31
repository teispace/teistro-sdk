"""Reads a feed the binding wrote with a parser that is not ours.

`check-python` runs this with `icalendar` at the pin in `requirements.txt`
(`03-design/icalendar.md` §6). The parser says the text is a calendar and
what it holds; the octet rules it is lenient about are checked by hand.
"""

import datetime
import sys

import icalendar  # type: ignore[import-not-found]

from teistro import Altitude, Calendar, Latitude, Longitude, Observer, Teistro, date, nepal_official

STAMP = 1767225600


def octets(text: str) -> None:
    """RFC 5545 §3.1: CRLF, 75 octets a line, no fold inside a character."""
    assert text.endswith("\r\n")
    assert text.count("\n") == text.count("\r\n"), "a bare newline"
    raw = text.encode()
    for line in raw.split(b"\r\n"):
        assert len(line) <= 75, line
        line.decode()  # A fold inside a character would not decode.
        assert all(byte >= 0x20 and byte != 0x7F or byte == 0x09 for byte in line), line


def main() -> None:
    kathmandu = Observer(latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400))
    with Teistro.open().context(**nepal_official()) as sdk:
        text = sdk.almanac.festival_calendar(
            from_date=date(Calendar.GREGORIAN, 2025, 9, 20),
            to_date=date(Calendar.GREGORIAN, 2025, 11, 10),
            place=kathmandu,
            utc_offset_seconds=20700,
            festivals={"rules": "NEPAL_PANCHANGA"},
            stamp_unix_seconds=STAMP,
            name="चाडपर्व; २०८२, नेपाल",
            refresh_minutes=1440,
            categories=["चाडपर्व", "Nepal, 2082"],
            color="DarkOrange",
            alarm={"triggerMinutes": 540},
            hints=[
                {
                    "select": {"forms": ["EKADASHI", "PARANA"]},
                    "categories": ["व्रत"],
                    "color": "teal",
                    "alarm": {"triggerMinutes": -720, "description": "भोलि व्रत"},
                }
            ],
            events=[
                {
                    "uid": "office@example",
                    "summary": "कार्यालय बन्द, दशैं; " + "क" * 40,
                    "description": "line one\nline two\\three",
                    "day": {"year": 2025, "month": 10, "day": 3},
                    "kind": "HOLIDAY",
                    "tradition": "CIVIL",
                    "categories": ["Work"],
                    "color": "red",
                },
                {"summary": "A call", "fromUnixSeconds": 1759435200, "toUnixSeconds": 1759438800},
            ],
        )
    octets(text)
    parsed = icalendar.Calendar.from_ical(text)
    assert parsed["VERSION"] == "2.0"
    assert str(parsed["NAME"]) == "चाडपर्व; २०८२, नेपाल"
    assert str(parsed["COLOR"]) == "darkorange"
    assert str(parsed["X-APPLE-CALENDAR-COLOR"]) == "#FF8C00"
    # By hand: `icalendar` splits a list at an escaped comma.
    assert "\r\nCATEGORIES:चाडपर्व,Nepal\\, 2082\r\n" in text
    assert str(parsed["CATEGORIES"].cats[0]) == "चाडपर्व"
    events = list(parsed.walk("VEVENT"))
    uids = [str(event["UID"]) for event in events]
    assert len(uids) == len(set(uids)) and len(uids) > 40
    # Every event has what §3.6.1 requires and one alarm.
    for event in events:
        assert "DTSTAMP" in event and "DTSTART" in event and "SUMMARY" in event
        alarms = [part for part in event.subcomponents if part.name == "VALARM"]
        assert len(alarms) == 1, event["SUMMARY"]
        assert str(alarms[0]["ACTION"]) == "DISPLAY"
        assert str(alarms[0]["DESCRIPTION"])
    # Sorted: whole days ascending.
    days = [event["DTSTART"].dt for event in events if not isinstance(event["DTSTART"].dt, datetime.datetime)]
    assert days == sorted(days)
    mine = next(event for event in events if str(event["UID"]) == "office@example")
    assert str(mine["SUMMARY"]) == "कार्यालय बन्द, दशैं; " + "क" * 40
    assert str(mine["DESCRIPTION"]) == "line one\nline two\\three"
    assert [str(c) for c in mine["CATEGORIES"].cats] == ["HOLIDAY", "CIVIL", "Work"]
    assert str(mine["COLOR"]) == "red"
    assert mine["DTSTART"].dt == datetime.date(2025, 10, 3)
    assert mine["DTEND"].dt == datetime.date(2025, 10, 4)
    alarm = next(part for part in mine.subcomponents if part.name == "VALARM")
    assert alarm["TRIGGER"].dt == datetime.timedelta(hours=9)
    call = next(event for event in events if str(event["SUMMARY"]) == "A call")
    assert call["DTSTART"].dt == datetime.datetime(2025, 10, 2, 20, tzinfo=datetime.timezone.utc)
    fasts = [event for event in events if "CATEGORIES" in event and "व्रत" in [str(c) for c in event["CATEGORIES"].cats]]
    assert fasts
    for fast in fasts:
        assert str(fast["COLOR"]) == "teal"
        alarm = next(part for part in fast.subcomponents if part.name == "VALARM")
        assert alarm["TRIGGER"].dt == -datetime.timedelta(hours=12)
        assert str(alarm["DESCRIPTION"]) == "भोलि व्रत"
    # The parser's own output reads back to the same events.
    again = icalendar.Calendar.from_ical(parsed.to_ical())
    assert [str(event["UID"]) for event in again.walk("VEVENT")] == uids
    print(f"{len(events)} events read back", file=sys.stderr)


if __name__ == "__main__":
    main()
