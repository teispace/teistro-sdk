# The time a sankalpa recites

Status: `implemented` (the structured parts); the rendered text waits on a
vetted Sanskrit template (§5). P2-3 of the consumer brief.

A sankalpa is the resolve spoken before a rite. Its first half places the
rite in time: the year of the sixty-year cycle, the half of the year, the
season, the lunar month, the fortnight, the tithi, the weekday, the
nakshatra, the yoga and the karana, often the signs the Sun, the Moon and
Jupiter stand in. Its second half places it on the earth (*Jambudvipe
Bharatavarshe…*, and Nepal's own form). Every part of the first half is a
fact the almanac already reckons. What a sankalpa adds is **which instant
reads each part**, and that is what this page decides.

## 1. Which instant reads each part

Almanacs that print a sankalpa for a moment agree on the split
([Drik Panchang's sankalpa][drik], [PanchangBodh's][bodh]):

| part | read at | why |
|---|---|---|
| samvatsara, with its Vikrama and Shaka numbers | the day's sunrise | the year's name stands for the whole day (*udaya-vyapini*) |
| ayana | the day's sunrise | as the year |
| ritu | the day's sunrise | as the year |
| lunar month, and whether it is adhika | the day's sunrise | as the year |
| vara | the day's sunrise | a weekday runs from sunrise to sunrise |
| paksha, tithi, nakshatra, yoga, karana | the instant of the rite | a limb that changes during the day is recited as it stands when the rite begins: the brief's "which tithi when two stand that day" |
| the Sun's, the Moon's and Jupiter's signs | the instant of the rite | as the limbs |

**The day** is the one whose sunrise the instant follows and whose next
sunrise it precedes. A rite at 04:00 recites the day before's year, month,
season and weekday, as every almanac reckons its days from sunrise. The
civil date that day carries is the one its sunrise falls on.

## 2. The conventions, each named in the answer

The brief asks that every convention be explicit. None is chosen here.
Each is stated beside the part it decided:

- **The ayana.** Most Indian almanacs and Nepal's turn it at Makara and
  Karka sankranti, the sidereal Sun entering Capricorn and Cancer; others,
  Drik Panchang's "Drik" option among them, at the solstices. The two
  differ for about three weeks around each turn. It is the request's
  `ayana` (`SIDEREAL` when left out, the almanac's own `sun.ayana`), and the
  answer's `ayanaReckoning` repeats it. It is a request field rather than a
  setting so that asking a sankalpa never changes the settings hash every
  other answer is stamped with.
- **The ritu** is `panchanga.ritu` (solar, lunar or tropical), named in
  `rituReckoning`.
- **The month** comes under both conventions, `amanta` and `purnimanta`,
  with `month` the profile's (`calendars.lunar_month`) and `convention`
  naming which.
- **The year's name** is `calendars.samvatsara`'s count, named in the
  year's `count` (`samvatsara-measured.md`).
- **The signs** are sidereal, in the profile's ayanamsha, as every other
  sign the almanac gives.

A consumer that wants another month or ritu convention sets the knob in a
settings patch, as for any other almanac answer.

## 3. The answer

`Almanac::sankalpa(at, place, ayana)` in `teistro-panchanga`;
`sdk.almanac().sankalpa(at, place, offset, ayana)` in the façade, and
`sankalpa_request` for the binding record; `ts_almanac_sankalpa` at the C
boundary; `almanac.sankalpa` as an agent tool; and `almanac.sankalpa` in
every binding, each answer typed in its own language.

```json
{
  "at": 2461055.59375,
  "date": {"calendar": "calendar.GREGORIAN", "year": 2026, "month": 1, "day": 15, "…": "…"},
  "vara": "vara.GURUVARA",
  "sunrise": 2461055.55198882, "nextSunrise": 2461056.551913582,
  "year": {"samvatsara": "samvatsara.SIDDHARTHI", "vikrama": 2082, "shaka": 1947, "kali": 5126, "…": "…"},
  "ayana": "ayana.UTTARAYANA", "ayanaReckoning": "SIDEREAL",
  "ritu": "ritu.SHISHIRA", "rituReckoning": "SOLAR",
  "month": {"month": "masa.MAGHA", "convention": "PURNIMANTA", "amanta": "masa.PAUSHA",
            "purnimanta": "masa.MAGHA", "paksha": "paksha.KRISHNA", "kind": "NIJA"},
  "paksha": "paksha.KRISHNA", "tithi": "tithi.KRISHNA_DWADASHI",
  "nakshatra": "nakshatra.JYESHTHA", "yoga": "yoga.VRIDDHI", "karana": "karana.TAITILA",
  "sun": "rashi.CAPRICORN", "moon": "rashi.SCORPIO", "jupiter": "rashi.GEMINI"
}
```

That is 08:00 on 15 January 2026 in Kathmandu under `nepali-default` over
the built-in ephemeris, the morning after Makara sankranti. The boundary writes every catalogue member as its full key
(`Sankalpa::MEMBERS`, held to serde both ways by the SDK's tests), and the
answer is sealed over what it wrote. Its input hash covers the instant,
the place and the ayana reckoning.

The parts are found by the almanac's own day: the day holding the instant
is reckoned once (its civil day, or the day before when the instant
precedes that day's sunrise), its limbs' spans give the members at the
instant, and the three signs are read in the instant's own frame,
topocentric where the profile is. Nothing is reckoned twice and nothing
is reckoned apart from the almanac a consumer already shows, so a
sankalpa never names a tithi the day's panchanga does not hold at that
moment. `crates/sdk/tests/sankalpa.rs` holds this over three days at three
times of day.

## 4. What is refused

An instant, place or offset out of range is `INVALID_ARG`, named under
`sankalpa` (`sankalpa.utcOffsetSeconds`). A context without an ephemeris
is `CAPABILITY`. An instant that falls in no day's window (a polar day
the policy leaves undefined, so the day before has no next sunrise) is
`OUT_OF_RANGE`: a sankalpa that cannot say which day it is says nothing.

## 5. Not yet: the words

The brief asks for the rendered text too, and for which tradition's
template made it. The place formula and the Sanskrit inflections (the
locative *samvatsare*, *ayane*, *ritau*, *māse*, *pakṣe*, *tithau*,
*vāsare*, the nakshatra and yoga names inflected, Nepal's *Himavatkhande*
form) are template data in the `sa` locale pack, and that pack's messages
are not written yet (`locale-coverage-measured.md`: `sa-Deva` names 332
members and translates no message). They wait on a vetted source or a
reviewer who reads Sanskrit, never on a machine stub. When they come, the
template is a message whose arguments are these parts, so nothing here
changes. The tradition default for the template is the maintainer's to
choose (brief §4).

Ashauch, the other half of P2-3, waits on its rule table's tradition
default, which is the maintainer's choice too.

[drik]: https://www.drikpanchang.com/panchang/sankalpa/sankalpa.html
[bodh]: https://panchangbodh.com/sankalp-mantra
