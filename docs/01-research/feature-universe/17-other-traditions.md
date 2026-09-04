# Other traditions

Status: `research`, 2026-09-04. Surveyed for the sake of the abstractions;
tier P2 unless noted. Sources: Astro-Vision's Ming Sign (Chinese), Time4J's
calendar list (Chinese, Korean, Vietnamese, Thai), general knowledge of the
systems.

| tradition | what it needs from the SDK | new abstractions | tier |
|---|---|---|---|
| Nadi (Bhrigu Nandi Nadi, Chandra Kala Nadi, Dhruva Nadi) | precise sidereal positions, nakshatra sub-divisions (150 nadiamsas), rule corpora keyed on degree | nadiamsa division (a varga of 150), large rule packs | P1 for nadiamsa, P2 for corpora |
| Regional Indian practice: Tamil (10 porutham, Tamil months and yogas), Kerala (Prashna, Parihara, Papasamya), Bengali (Bangabda), Odia, Malayalam calendar, Telugu and Kannada panchangas | calendars, regional rule packs, name tables in more scripts | calendar plug-ins, locale packs | P1 (calendars), P2 (rules) |
| Sri Lankan (Sinhala) astrology and calendar | Sinhala locale, Buddhist era, Nekath tables | locale pack, calendar | P2 |
| Burmese Mahabote and Thai astrology | weekday-of-birth based charts, Burmese calendar | a non-ecliptic chart kind, calendar | P2 |
| Tibetan (Jungtsi and Kartsi): elements, animals, mewa, parkha; Tibetan calendar | lunisolar calendar with skipped and doubled days; element cycles | calendar, cycle engines | P2 |
| Chinese BaZi (Four Pillars): stems and branches from the solar terms; luck pillars; Zi Wei Dou Shu: palace charts from lunar date and hour | solar terms (Sun longitude crossings every 15°), Chinese lunisolar calendar (needs new Moons and solar terms, ICU4X has the calendar) | sexagenary cycle engine, pillar chart kind, luck-pillar time lords | P2 |
| Hebrew, Islamic and Persian calendrics for horary and electional users | calendars via ICU4X-class algorithms | calendar plug-ins | P1 (calendars) |
| Mayan and Aztec day counts | calendars | | P2 |

## What this changes in the architecture now

- The chart model must allow a chart kind that is not an ecliptic
  placement (pillars, weekday charts). The plan: a `ChartKind` with an
  ecliptic chart as the common case and an extension point for others.
- The calendar port must accept lunisolar calendars that themselves depend
  on the ephemeris (Indian, Chinese, Tibetan). The calendar provider can
  therefore depend on the ephemeris port; the layering allows it.
- Solar terms and the sexagenary cycle are cheap to add once the crossing
  search exists; they are noted so the crossing API is not designed only for
  sign ingresses.
