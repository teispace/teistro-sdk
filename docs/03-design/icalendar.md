# iCalendar feeds

Status: `built`, 2026-10-20; §6, SR-11, 2026-10-30. Design for the consumer
brief's P2-2, written before the code and corrected by it. §5 is measured
through the built writer (`crates/sdk/tests/festival_calendar.rs`).

A calendar app subscribes to a feed of observances, and a user exports a
year of them. Both need one writer, in every binding, that a validator
accepts and that writes the same bytes for the same input. This page
designs that writer and the almanac call that feeds it.

## 1. What RFC 5545 asks, and what the clients read

| part | the rule | source |
|---|---|---|
| lines | CRLF ends every line; a line longer than 75 octets is folded by CRLF and one space, never inside a UTF-8 sequence | RFC 5545 §3.1 |
| text | `\` `;` `,` escaped with `\`, a newline written `\n`; no control character but a tab, the rest dropped | §3.3.11 |
| calendar | `VERSION:2.0` and `PRODID` required; `CALSCALE:GREGORIAN`, `METHOD:PUBLISH` for a published feed | §3.7 |
| event | `UID` and `DTSTAMP` required; `DTSTART` | §3.6.1 |
| all-day | `DTSTART;VALUE=DATE:YYYYMMDD`, and `DTEND;VALUE=DATE` the next day, exclusive | §3.6.1, §3.8.2.2 |
| timed | `DTSTART:YYYYMMDDTHHMMSSZ` and `DTEND` in UTC (form 2), the end exclusive | §3.3.5, §3.8.2.2 |
| busy time | `TRANSP:TRANSPARENT`: an observance does not block the day | §3.8.2.7 |
| name, refresh | `NAME` and `REFRESH-INTERVAL;VALUE=DURATION` | RFC 7986 §5.1, §5.7 |
| uid | unique among a feed's events, so a second is refused | §3.8.4.7 |
| categories | `CATEGORIES`, values escaped, joined by unescaped commas; on an event and on the calendar | §3.8.1.2; RFC 7986 §5.6 |
| colour | `COLOR`, a CSS3 colour name and nothing else; on an event and on the calendar | RFC 7986 §5.9; CSS Color Level 3 §4.3 |
| the same, as clients read it | `X-APPLE-CALENDAR-COLOR:#RRGGBB` on the calendar | Apple's clients read a hexadecimal value |
| alarm | `VALARM` inside the event: `ACTION:DISPLAY`, `TRIGGER`, a duration from the start, and `DESCRIPTION`, which a display alarm requires | §3.6.6, §3.8.6.3 |
| the same, as clients read them | `X-WR-CALNAME`, `X-PUBLISHED-TTL` | Apple and Google read these; Outlook reads `X-PUBLISHED-TTL` |

No `RRULE`: a lunar date does not recur in Gregorian terms, so each
occurrence is its own event over the range asked.

## 2. The writer: `teistro_calendar::ical`

A pure function from a feed to text. No clock and no I/O:

```rust
pub struct Feed { pub product: String, pub name: String, pub refresh_minutes: Option<u32>,
                  pub stamp_unix_seconds: i64, pub events: Vec<Event> }
pub struct Event { pub uid: String, pub summary: String, pub description: Option<String>,
                   pub when: When }
pub enum When { Day(FixedDay), Span { from_unix_seconds: i64, to_unix_seconds: i64 } }
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
- A span is written in UTC, so no `VTIMEZONE` is needed: a client shows
  it in the viewer's zone. A span that does not end after it starts is
  refused as `INVALID_ARG`.

An event is a whole day, or a span for a fast's parana (§4).

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

- Each observance, each Ekadashi fast and each dated day (a sankranti,
  a national day) becomes an all-day event on its
  civil day, ordered by day, then rule key.
- Each Ekadashi fast whose rule keeps a parana (`festival-rules.md`
  §8.5) adds a timed event on the parana day: the window kept, its start
  rounded **up** to the minute and its end **down**. A calendar app
  shows minutes and drops the seconds, so a window from 06:12:40 would
  read 06:12, before the meal may be taken. Rounded inward, every shown
  minute lies inside the computed window. A window under a minute, or a
  fast with no window kept, writes no event, and a rule without
  `parana` writes none. Its `SUMMARY` is `sdk.calendar.parana` over the
  fast's name ("Parana: Smarta Ekadashi", "स्मार्त एकादशीको पारणा"),
  and its `UID` hashes `<rule>|PARANA` with the parana day and the
  place. Timed events sort among a day's events by their start, after
  the day's whole-day events.
- A vrata whose rule lists parana readings (`festival-rules.md` §8.7)
  adds the same timed event from its **first** reading, when that
  reading ends the window: Shivaratri's `WITHIN { 3 }`, sunrise to the
  14th's end or three yamas. A first reading that only opens the meal
  (Janmashtami's and Rama Navami's `TITHI_END`) has no end to write, and
  the feed does not invent one.
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
  Read again on 2026-10-22, with the pack grown and the parana added:
  511 events each, 75 of them timed (one for each of the three
  observers' 25 fasts). Every timed event is zoned, ends after it starts
  and falls on whole minutes, and in `ne-Deva-NP` it is named "स्मार्त
  एकादशीको पारणा". Read again with the vratas' parana: 512 events, 76
  timed, the new one Shivaratri's on 2026-02-16, 06:42 to 15:06 at
  Kathmandu (sunrise to three yamas). Shila Charhe, counted from
  Shivaratri's day, writes none: the fast is broken once.
- A fast's parana (`a_fast_s_parana_is_a_timed_event_inside_its_window`):
  over Dashain and Tihar 2025, one timed event for each fast that keeps
  a window, on its parana day, each end on a whole minute, inside the
  computed window and less than a minute from it. A rule without a
  parana writes no timed event, and in `ne-Deva-NP` the summary is the
  locale's.
- A vrata's parana
  (`a_vrata_s_parana_is_timed_from_its_first_reading_when_that_ends`):
  over February to September 2026, Shivaratri's is a timed event on its
  parana day, and Janmashtami's and Rama Navami's open readings write
  none. Proved red by dropping the vratas from the feed.
- Every binding reads the same bytes, through its own test, and counts
  one whole-day event per occurrence and one timed event per parana.
- Importing into Apple Calendar, Google Calendar and Outlook is a manual
  check, recorded once. Not done here.

## 6. Merging, filtering and hints (SR-11)

A feed publisher's holidays are data: the consumer asks to merge its own
dated events, to set an alarm, to colour and categorise, and to filter by
kind or tradition. All of it is optional on `FeedOptions` and on the
request record, and a request naming none of it writes the bytes it wrote.

| field | what it does |
|---|---|
| `events` | The caller's own, each a `day` (in `calendar`, the request's when left out) or `fromUnixSeconds` with `toUnixSeconds`. `uid` is optional: left out, it hashes the summary, the time and the place as §3 hashes a rule's. A supplied event whose `UID` is a computed event's stands in its place, the one way to correct one day of a feed; two supplied events of one `UID` are refused. |
| `only` | Selectors; the feed keeps the events any selects, every event when left out. |
| `hints` | Selector, `categories`, `color`, `alarm`: the first that selects an event speaks for it. |
| `alarm` | The alarm on every event no hint or event of its own gives another. |
| `categories`, `color` | The calendar's own. |

A **selector** is `forms`, `kinds`, `traditions` and `rules`. A list left
out asks nothing, a list given must hold the event's value, and every list
given must. `forms` is how the event came to be: `OBSERVANCE`, `EKADASHI`,
`DATED`, `ANNIVERSARY`, `ADDED`, `PARANA`, `SUPPLIED`. `rules` holds rule
keys; a parana answers to its fast's key and a supplied event to its `UID`.
A filter applies to every event alike, a supplied one included, so one rule
states what a filter keeps; `{"forms": ["SUPPLIED"]}` keeps a caller's own.

An event's `CATEGORIES` are its kind, its tradition, its own and the
hint's, in that order, a repeat written once. Its colour and alarm are its
own, then the hint's, and for the alarm then the feed's. An alarm's
`triggerMinutes` is signed from the event's start, and a whole day starts
at its first moment, so `540` rings at 09:00 on the day and `-720` at noon
before it. `TRIGGER` is relative, so a client rings it in the viewer's
zone. Its `DESCRIPTION` is the event's summary unless the alarm says more.

**Kind and tradition come from SR-02.** A rule's catalogue fields are read
in one function, `classes_of` in `sdk/src/area/almanac/feed.rs`, and until a
rule carries them it returns none; a supplied event states its own `kind`
and `tradition` today. A selector naming `kinds` or `traditions` when no
event of the feed carries that field is refused, `UNSUPPORTED` on that
field, and not answered with an empty feed that would read as "nothing is
of that tradition" (C870). The refusal is about the data, so it needs no
change when SR-02 lands: a feed of rules that carry a tradition stops
being refused, and a caller's custom rules that carry none keep being.

### Acceptance

- `festival_calendar.rs`: a supplied day and span merged in the day's order
  with a stable `UID`; a supplied `UID` replacing a computed event and two
  of one `UID` refused; `only` by form and by rule, a parana answering to
  its fast; the first hint speaking, the feed's alarm filling in; a
  selector on a field nothing carries refused and, with one event carrying
  it, read; the record's refusals named by field.
- Read back by a parser used as a black box: `check-python` writes a feed
  through the binding with every field set, in `ne-Deva-NP`, and reads it
  with Python's `icalendar` at the pin in
  `bindings/python/blackbox/requirements.txt`, then its own octet rules
  (`bindings/python/blackbox/ical_feed.py`): CRLF, 75 octets, no bare
  control character, no fold inside a character.
