# The sighted Hijri calendar

Status: `built`, 2026-11-22. Written before the code and corrected by the
building: §3's first month is looked for from four days before the table,
not three. Measured in `hijri-sighted-measured.md`.

`Calendar::HijriSighted` (`calendar.HIJRI_SIGHTED`), era `era.HIJRI`. The
crescent is `crates/astro/src/crescent.rs`, the chain of months
`crates/panchanga/src/hijri_sighted.rs`, the table of months
`crates/calendar/src/hijri_sighted.rs`. The tabular calendar beside it is
`docs/calendars/hijri.md`.

## 1. Purpose and scope

A Hijri month begins when the new crescent is first seen after the
sunset. Nepal and India date Eid al-Fitr, Eid al-Adha, Ashura and the
Mawlid by that sighting, and so a day or two from the tabular calendar.
Until now the SDK answered with the table, moved as a whole by
`hijriOffset`, or with the declared day laid over it as a published set
(`published-overrides.md` §7).

This page adds a third answer: the months as a sighting at the request's
own place would begin them, computed from the SDK's own topocentric Sun
and Moon and its own sunset. It is a prediction, not a declaration. An
authority that declares a day still has the last word, through a
published set laid over this calendar as over any other.

In scope:
- a month's first day at a place, by one of two published visibility
  criteria and a zone;
- the dated-rule machinery naming the calendar, as it names
  `HIJRI_TABULAR`;
- the settings that choose the criterion and the zone, on every surface.

Out of scope: a sighting reported from another place (a global or
national sighting), Saudi Arabia's Umm al-Qura calendar, the weather,
and any authority's decision.

## 2. The crescent on an evening

### 2.1 The rule a criterion serves

The rule is the hadith's: "fast when you see it and break the fast when
you see it, and if it is hidden from you, complete the count of thirty"
(Sahih al-Bukhari, Kitab al-Sawm, nos. 1906 and 1909; Sahih Muslim,
Kitab al-Siyam, no. 1081). A month is so 29 or 30 days, and the evening
that decides it is the 29th's. A crescent is looked for on the evening
of the 29th; if it is seen the next day is the 1st, and if not the month
completes 30 days whatever the 30th evening shows.

India leaves these days open in its own list: the Department of
Personnel and Training's holiday memorandum marks Id-ul-Fitr, Id-ul-Zuha,
Muharram and Milad-un-Nabi "subject to the sighting of the moon"
(para. 5.1), and a state list follows with a later notification. No
primary source for Nepal's announcing body is in hand; Nepal's dates are
measured as kept (§7).

### 2.2 The best time

Both criteria read the sky at one instant. Yallop takes it from Bruin's
curves: the sunset plus four ninths of the lag, the lag being the moonset
less the sunset, `Tb = (5 Ts + 4 Tm) / 9` (Yallop, *A Method for
Predicting the First Sighting of the New Crescent Moon*, NAO Technical
Note 69, 1997, eq. 4.1, p. 4; the printed page, one less than the PDF's).
Odeh adopts the same instant (Odeh, *New
Criterion for Lunar Crescent Visibility*, *Experimental Astronomy* 18,
2004, eq. 1, p. 43). The SDK reads the sunset and the moonset as the
upper limbs' under the standard refraction
(`Horizon::UPPER_LIMB_REFRACTION`), as both authors reckon the lag.

The evening of civil day D is the first sunset after D's local mean
noon, `midnight + 0.5 − longitude / 360` days. The moonset is the first
after half a day before that sunset, so a Moon that sets before the Sun
gives a negative lag. An evening with no positive lag, or with the Moon
not yet past the Sun in ecliptic longitude, has no crescent to look for:
it is not seen under either criterion.

### 2.3 The quantities

At the best time, with no refraction:

| quantity | Yallop | Odeh |
|---|---|---|
| ARCV, the Moon's altitude less the Sun's | geocentric (p. 1) | topocentric (p. 61) |
| ARCL, the angle between the centres | geocentric | topocentric |
| W, the crescent's width, arcminutes | `W′ = SD′ (1 − cos ARCL)`, `SD = 0.27245 π`, `SD′ = SD (1 + sin h sin π)` (eqs. 3.8 to 3.10, p. 3) | `W = SD (1 − cos ARCL)`, topocentric SD and ARCL (p. 61) |

DAZ, the Sun's azimuth less the Moon's, is recorded beside them but read
by neither.

### 2.4 The two criteria

**Odeh** (eq. 2, p. 61), fitted to 737 observations:

`V = ARCV − (−0.1018 W³ + 0.7319 W² − 6.3226 W + 7.1651)`

| zone | V | Odeh's reading (p. 61) |
|---|---|---|
| A | ≥ 5.65 | visible by naked eye |
| B | ≥ 2.00 | visible by optical aid, could be seen by naked eye |
| C | ≥ −0.96 | visible by optical aid only |
| D | < −0.96 | not visible even by optical aid |

**Yallop** (eq. 6.1, p. 11):

`q = (ARCV − (11.8371 − 6.3226 W′ + 0.7319 W′² − 0.1018 W′³)) / 10`

| zone | q | Yallop's reading (Table 5, p. 12) |
|---|---|---|
| A | > +0.216 | easily visible |
| B | > −0.014 | visible under perfect conditions |
| C | > −0.160 | may need optical aid to find the crescent |
| D | > −0.232 | will need optical aid |
| E | > −0.293 | not visible with a telescope |
| F | ≤ −0.293 | not visible, below the Danjon limit |

Odeh's paper also states the Danjon limit, an ARCL below 6.4° (p. 63);
the zones already put such a crescent in D.

### 2.5 The settings

Two knobs in the `calendars` group:

| knob | values | default |
|---|---|---|
| `calendars.hijri_criterion` | `ODEH_2006`, `YALLOP_1997` | `ODEH_2006` |
| `calendars.hijri_zone` | `A`, `B`, `C`, `D` | `A` |

A crescent counts as seen in the zone named and every better one. The
zone is the weakest seen; Odeh's D is a crescent "not visible even by
optical aid", so `ODEH_2006` with `D` is refused as `INVALID_ARG` on
`calendars.hijri_zone`. Yallop's E and F are not offered: neither is a
sighting.

The knob's value names the criterion as its literature does. Odeh's
volume is dated 2004 and was published online in 2006; `ODEH_2006` is the
name the criterion is cited by.

**The default is Odeh to zone A.** It and Yallop to zone A both give every
Nepali Eid measured, and differ on one Indian day: the central list's
Mawlid of 2027, a day not yet kept, which Yallop's zone A matches. Odeh's
is chosen because it reads the sky from the place rather than from the
Earth's centre, which is what a request at a place asks, and because it
is fitted to 737 observations. Every weaker zone loses
Nepal's Fitr of AH 1445 and Adha of AH 1447 (`hijri-sighted-measured.md`
§1).

## 3. The months

### 3.1 The chain

A month's length turns on its own 29th, so a month's first day depends
on every month before it. The reckoning opens a chain [`DEPTH`] = 12
months before the month a span begins in, by the table:

1. The first month's first day is the day after the first evening seen,
   looking from four days before the tabular month's first to one day
   after it (six evenings). If none is seen, it begins on the tabular
   day, unmarked as seen.
2. Each month after: look on the evening of its 29th. Seen, the next
   month begins after 29 days; not seen, after 30.
3. The chain stops at the first month that begins after the span's last
   day.

A first month a day off is drawn back to the sky within a month or two:
the 29th's rule lengthens a month begun early and shortens one begun
late. The measured page holds the depth: no month differs between chains
opened 12 and 24 months back over 1900 to 2101.

### 3.2 The months' names

A sighted month takes the name of the tabular month of the same count
from the chain's opening. The table refuses, as `INTERNAL`, a month that
begins more than three days from the tabular month of its name
(`MOST_FROM_TABLE`), a month other than 29 or 30 days, and months out of
order; none of these can arise from §3.1, and each would mean the
reckoning has lost its place.

### 3.3 The table

`SightedMonths` is a `CalendarSystem` over the span it covers: `date_of`,
`fixed_of`, `month_length` and `is_leap` (Dhu al-Hijja of 30 days), with
`needs_place` and `needs_ephemeris` set. A day outside the span is
refused as `OUT_OF_RANGE`. Each month records whether it began on a
crescent seen (`seen`) or after 30 days.

## 4. The surface

**The almanac.** `Almanac::sighted_hijri(first, last, place)` reckons the
months over a span at a place under the context's settings.

**The rules.** A dated rule names the calendar as it names the table:

```json
{ "key": "EID_AL_FITR", "on": { "civilDate": { "calendar": "HIJRI_SIGHTED", "month": 10, "day": 1 } } }
```

`civilDate` and `civilWeekday` read it. A festival request with such a
rule reckons the months over its viewed days at its own place, once, and
only then; a request with none pays nothing. `nakshatraInMonth` and
`tithiInMonth` refuse it, as they refuse every non-Hindu calendar.
`hijriOffset` moves the tabular calendar only.

**Provenance.** The answer names the test the months were reckoned by:
`calendars.hijri_criterion` = `ODEH_2006 to zone A`, with the rule in
words.

**Bindings.** The calendar is a catalogue member and the knobs are
settings, so every binding, the IDL and the schema carry them through
the generators; no binding has code of its own.

## 5. Errors and limits

- A place where the Sun does not set on an evening is refused as the
  rise and set solver refuses it (`LIMIT`): the sighted calendar has no
  answer in a polar summer.
- A rule on `HIJRI_SIGHTED` outside a festival request at a place is
  refused as `INVALID_ARG` on `civilDate.calendar`.
- The answer is a sighting by a good observer under a clear sky at the
  place. Cloud, haze, an authority accepting a sighting from elsewhere,
  or a decision in advance all part from it; the measured page names
  every such parting found.
- The months are as good as the sky. An arcminute of error in the arc of
  vision moves V by 0.017; the nearest Nepali margin stands 0.72 from
  its zone's edge (`hijri-sighted-measured.md` §3).

## 6. Cost

A request reads one evening a month, the 29th's, plus at most six for
the chain's opening, each once (the evenings are held in a map). Twelve
months of depth add twelve evenings. An evening is a sunset, a moonset
and one sky at the best time. A year's request so reads about 30
evenings at its place. The measured page's whole sweep — seven
criteria-and-zone pairs over fourteen places and nine years, the margins,
and 200 years of the depth check twice over — runs in about 25 s in a
debug build.

## 7. Tests

- **The criteria.** Odeh's zone edges (eq. 2) and Yallop's (Table 5),
  both zero points of the polynomials, and the zone D refusal are pinned
  in `crescent.rs`.
- **The table.** Every day converts and converts back; a month of 31
  days, months out of order and a month four days from the table are
  refused (`calendar/src/hijri_sighted.rs`).
- **Nepal.** `crates/sdk/tests/hijri_sighted.rs` holds the twelve Nepali
  Eids of AH 1440 to 1447 at Kathmandu (27.7172 N, 85.324 E, 1400 m)
  under the default, with no parting allowed.
- **Through the façade.** A festival request with a rule on
  `HIJRI_SIGHTED` keeps it on the sighted day, names the criterion in its
  provenance, and refuses Odeh's zone D on `calendars.hijri_zone`.
- **The sweep.** An ignored test prints every criterion and zone against
  Nepal and India's thirteen lists, the margins and the depth: the
  measured page.

## 8. Localisation

`sdk.calendar.HIJRI_SIGHTED` takes the tabular calendar's month names
and date forms by reference, so the two never drift. The calendar is
named in `sdk.entity.json`: "Sighted Hijri calendar", "हिजरी पात्रो
(चन्द्रदर्शन)", "हिजरी कैलेंडर (चंद्रदर्शन)", "हिजरी दिनदर्शिका
(चन्द्रदर्शन)"; sa-Latn is derived by `gen intl`.

The settings' sources are in `ROOT_SOURCES` (SR-110) and their effect
notes in every locale's `sdk.settings.json`; the measured text they rest on:

| knob | source | effect (en-Latn) |
|---|---|---|
| `calendars.hijri_criterion` | Odeh, *Experimental Astronomy* 18, 2004, eq. 2, p. 61; Yallop, NAO TN 69, 1997, eq. 6.1, p. 11 | Which test judges the crescent on the evening of a month's 29th. Odeh's reads the sky from the place; Yallop's from the Earth's centre. Under zone A they parted on one of 398 days measured. |
| `calendars.hijri_zone` | Odeh, eq. 2 and its zones, p. 61; Yallop, Table 5, p. 12 | The weakest zone a crescent counts as seen in. A weaker zone sees a crescent sooner and begins months a day earlier; zone A matches every Nepali Eid measured, zone B loses two. |

ne-Deva-NP: "महिनाको २९ गतेको साँझ चन्द्रमा देखिन्छ कि देखिँदैन भनी कुन
मापदण्डले जाँच्ने। ओदेहको मापदण्डले ठाउँबाट आकाश हेर्छ, यालपको पृथ्वीको
केन्द्रबाट।" / "चन्द्रमा देखिएको मानिने सबैभन्दा कमजोर क्षेत्र। कमजोर
क्षेत्रले चन्द्रमा छिटो देख्छ र महिना एक दिन अघि सुरु हुन्छ।"

hi-Deva-IN: "महीने की २९ तारीख की शाम चंद्रमा दिखता है या नहीं, यह किस
मानदंड से परखा जाए। ओदेह का मानदंड आकाश को स्थान से देखता है, यालप का
पृथ्वी के केंद्र से।" / "चंद्रमा को दिखा हुआ मानने का सबसे कमज़ोर क्षेत्र।
कमज़ोर क्षेत्र चंद्रमा को जल्दी देखता है और महीना एक दिन पहले शुरू होता है।"

sa-Deva: "मासस्य एकोनत्रिंशे दिने सायं चन्द्रः दृश्यते न वेति केन
मानदण्डेन परीक्ष्यते। ओदेहस्य मानदण्डः स्थानात् आकाशं पश्यति, यालपस्य
भूकेन्द्रात्।" / "यस्मिन् क्षेत्रे चन्द्रः दृष्टः इति मन्यते तेषु
दुर्बलतमम्। दुर्बलतरं क्षेत्रं चन्द्रं शीघ्रं पश्यति, मासः च एकेन दिनेन
पूर्वं आरभते।"

## 9. The Eids in the Nepal pack

`NEPAL_PANCHANGA` keeps the gazette's two Eids (clauses 2.1 त and थ,
which print no day) as two dated rules,

```json
{ "key": "EID_AL_FITR", "on": { "civilDate": { "calendar": "HIJRI_SIGHTED", "month": 10, "day": 1 } } }
{ "key": "EID_AL_ADHA", "on": { "civilDate": { "calendar": "HIJRI_SIGHTED", "month": 12, "day": 10 } } }
```

gazetted, of the Islamic tradition, and of Nepal. The request's place is
Kathmandu's for the national list; the default settings need no change.
A request on the pack therefore reckons the sighted months over its days.
A declared day still goes in as a published `SET` entry, which wins over
the computed one and is marked as published. The test
`the_nepal_pack_keeps_its_printed_eids` holds the pack to every printed
Eid from AH 1445 on.

## 10. Open questions

- Nepal's announcing body and its practice (a local sighting, or one
  accepted from India) have no primary source in hand. The measurement
  shows a local sighting under zone A at Kathmandu matches every Eid
  measured.
- Kerala's list runs a day earlier than any criterion at
  Thiruvananthapuram on 17 of its 24 days. That is a sighting accepted
  from elsewhere, not a sky; a profile for it would be a published set,
  not a zone.
