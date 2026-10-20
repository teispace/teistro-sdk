# iCalendar feeds

Status: `built`, 2026-10-20. Design for the consumer brief's P2-2, written
before the code and corrected by it. §5 is measured through the built
writer (`crates/sdk/tests/festival_calendar.rs`).

A calendar app subscribes to a feed of observances, and a user exports a
year of them. Both need one writer, in every binding, that a validator
accepts and that writes the same bytes for the same input. This page
designs that writer and the almanac call that feeds it.

## 1. What RFC 5545 asks, and what the clients read

| part | the rule | source |
|---|---|---|
| lines | CRLF ends every line; a line longer than 75 octets is folded by CRLF and one space, never inside a UTF-8 sequence | RFC 5545 §3.1 |
| text | `\` `;` `,` escaped with `\`, a newline written `\n` | §3.3.11 |
| calendar | `VERSION:2.0` and `PRODID` required; `CALSCALE:GREGORIAN`, `METHOD:PUBLISH` for a published feed | §3.7 |
| event | `UID` and `DTSTAMP` required; `DTSTART` | §3.6.1 |
| all-day | `DTSTART;VALUE=DATE:YYYYMMDD`, and `DTEND;VALUE=DATE` the next day, exclusive | §3.6.1, §3.8.2.2 |
| busy time | `TRANSP:TRANSPARENT`: an observance does not block the day | §3.8.2.7 |
| name, refresh | `NAME` and `REFRESH-INTERVAL;VALUE=DURATION` | RFC 7986 §5.1, §5.7 |
| the same, as clients read them | `X-WR-CALNAME`, `X-PUBLISHED-TTL` | Apple and Google read these; Outlook reads `X-PUBLISHED-TTL` |

No `RRULE`: a lunar date does not recur in Gregorian terms, so each
occurrence is its own event over the range asked.

## 2. The writer: `teistro_calendar::ical`

A pure function from a feed to text. No clock and no I/O:

```rust
pub struct Feed { pub product: String, pub name: String, pub refresh_minutes: Option<u32>,
                  pub stamp_unix_seconds: i64, pub events: Vec<Event> }
pub struct Event { pub uid: String, pub summary: String, pub description: Option<String>,
                   pub day: FixedDay }
pub fn write(feed: &Feed) -> Result<String, Error>
```

- It lives in the calendar crate, beside `FixedDay` and the Gregorian
  calendar it writes dates in. The draft put it in `teistro-serial`,
  which the SDK takes only with the `chart` feature; the almanac is not
  behind it.
- `DTSTAMP` is the feed's `stamp_unix_seconds`, which the caller gives,
  so two runs over one input are byte-identical. A binding fills it with
  the current time when its caller leaves it out; Rust cannot, since
  `wasm32-unknown-unknown` has no clock.
- A day outside the years 1 to 9999 is refused as `OUT_OF_RANGE`: the
  format's year has four digits.
- Events are written in the order given.

Every event is a whole day. The draft also designed a timed event for an
Ekadashi fast's parana window; the SDK computes no parana yet (P1-8), so
that waits for it.

## 3. The stable UID

`UID` is the first 32 hex digits of SHA-256 over
`rule|fixed day|latitude|longitude|altitude`, then `@teistro`, the place's
numbers written in the canonical grammar. A feed published again with
the same rule, day and place updates the event and does not duplicate it.

The UID does not name the profile or the ephemeris. When a setup moves an
observance to another day, the old day's event goes and the new day's
comes.

## 4. The almanac call

`AlmanacArea::festival_calendar(from, to, place, offset, festivals,
options)` answers the feed's text, sealed with the festival answer's
provenance and its content hash taken over the text. `FeedOptions` holds
`name` ("Festivals"), `product` (`-//Teispace//Teistro SDK x.y.z//EN`),
`refresh_minutes` (a day) and `stamp_unix_seconds`.

- Each observance and each Ekadashi fast becomes an all-day event on its
  civil day, ordered by day, then rule key.
- `SUMMARY` is the festival's name in the context's locale
  (`sdk.entity` `festival`), falling back to the rule's key when the
  locale names it not. `check-intl` refuses a shipped pack's key that a
  strict locale does not name.
- `DESCRIPTION` gives the rule's source, then the answer's provenance:
  the SDK's version, the profile, the ephemeris and the festival answer's
  content hash. The provenance line is English in every locale, as the
  sources are.

At the boundary the call is `ts_almanac_festival_calendar`, a request
record (`FestivalCalendarRequest`: the range, the place, the offset,
`festivals`, `stampUnixSeconds`, and optionally `name`, `product` and
`refreshMinutes`, zero for none) and a text answer. Every binding has it
as `almanac.festivalCalendar`.

## 5. Acceptance

- Unit tests on the writer's rules: escaping, folding at 75 octets on a
  character boundary (Devanagari is three octets a character), a line of
  exactly 75 left whole, CRLF, the all-day form across a year's end, the
  refresh interval in its largest unit (zero writing none, since a
  client told to fetch every zero minutes would poll), and a year the format cannot
  write.
- The façade's tests, over Dashain and Tihar 2025 at Kathmandu under
  `nepal_official`: one event for each observance and fast, ordered by
  day, each UID distinct, Vijaya Dashami on the answer's day; in
  `ne-Deva-NP` every line at most 75 octets and the summary the locale's
  name; the same request the same bytes, and a new stamp moving only the
  `DTSTAMP` lines; the binding's record writing what the call writes.
- Read back by a second parser: VS 2082 at Kathmandu (2025-04-14 to
  2026-04-13), in `en-Latn` and `ne-Deva-NP`, parsed by Python's
  `icalendar` 6.1.0: 211 events each, every UID distinct, every event a
  day long, and the parser's own output read back to the same events.
- Every binding reads the same bytes, through its own test.
- Importing into Apple Calendar, Google Calendar and Outlook is a manual
  check, recorded once. Not done here.
