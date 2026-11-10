# The time a sankalpa recites

Status: `implemented`: the structured parts, and the words by a tradition's
template (§5, SR-29). P2-3 of the consumer brief.

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

That is 08:00 on 15 January 2026 in Kathmandu under `nepali-drik` over
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

## 5. The words: a tradition's template (SR-29)

The words are a message in the locale packs, one a tradition, filled with
the parts above and the slots the caller gives. A tradition is its message
key; `sdk.almanac().sankalpa_words(&sankalpa, &asked, tradition, locale)`
answers `SankalpaWords { tradition, locale, params, text }`, and the
answer names its tradition. A consumer registers another tradition by
loading a pack that carries its own key and asking by that key: the
parameters are the ones `sankalpa_params` lists, so a template may use
any of them and leave the rest. A key no loaded locale carries is refused
`UNSUPPORTED` on `tradition`.

**The call.** The `almanac.sankalpaWords` record takes the time's request
(the instant, the place and its clock, the ayana) with `slots`,
`tradition` (Himavatkhande when left out) and `locale`, and answers
`SankalpaWords` sealed with its provenance: `ts_almanac_sankalpa_words`
at the C boundary, the MCP tool of the same name, and
`almanac.sankalpaWords(request)` (`sankalpa_words` in Python) in Node,
Python, Dart and Java. Every refusal is named under `sankalpaWords`
(`sankalpaWords.slots.gotra`, `sankalpaWords.tradition`,
`sankalpaWords.utcOffsetSeconds` …).

**The slots** (`SankalpaFor`): `performer` (the name with its title, in
the nominative, as spoken before *aham*), `gotra` (the stem), `purpose`
(the rite in the accusative), and optionally `river`, `kshetra` and
`pitris` (`father`, `grandfather`, `greatGrandfather`, as stems). A slot
left blank is refused, named under `sankalpa`; a slot left out is not
spoken.

**The shipped tradition, `sdk.sankalpa.himavatkhande`**, the Nepal form.
Its sources:

- *Karmakanda Bhaskara* (Dadhiram Marasini, ed. Venimadhava Dhakal,
  Mahendra Sanskrit University, Beljhundi, Dang, VS 2054; archive.org
  `qmln_karmakanda-bhaskara-…`), p. 500, the tarpana sankalpa: the
  general form the template follows word for word, from *hariḥ oṃ tatsat*
  through *brahmaṇo dvitīyaparārdhe śrīśvetavārāhakalpe … kaliprathamacaraṇe
  bhāratavarṣe bharatakhaṇḍe jambūdvīpe āryāvartāntargate*, the river and
  holy ground (*amukanadyāḥ samīpe amukakṣetre*), *bauddhāvatāre*, the
  samvatsara among the sixty, the Vikrama year, ayana, ritu, month, paksha,
  weekday and tithi, the Sun's, Moon's and Jupiter's signs, and *amukagotrotpanno
  … amukaśarmā'ham*. p. 164, the kanyadana mahasankalpa, adds the place
  *himavatparvataikadeśe* and the nakshatra, yoga and karana. p. 477, the
  Mahalaya shraddha, names the pitris: *amukasagotrāṇām
  asmatpitṛpitāmahaprapitāmahānām amukāmukāmukaśarmaṇāṃ …
  vasurudrādityasvarūpāṇām*.
- *Sankalpa Ratnavali* (Harinath Pyakuryal, Kathmandu 1983), p. 10, a
  worked sankalpa: *… āryāvartāntargate nepāladeśe pāśupate kṣetre …
  kauṇḍinyagotraḥ … harināthaśarmā … vāgmatīsnānam ahaṃ kariṣye*; pp. 11 ff.
  explain each slot.
- *Aryavarta Panchanga* VS 2082, PDF p. 44, a prayaschitta sankalpa citing
  *Karmakanda Bhaskara* (2054): *āryāvartāntargatabrahmāvartaikanepāladeśe*,
  the month *saureṇa* or *cāndreṇa*, and the title by varna (*śarmā*,
  *varmā*, *gupta*; *devī* for a woman).

The template says the place as *āryāvartāntargate himavatparvataikadeśe
nepāladeśe*. "Himavatkhaṇḍe", the name the brief uses, was not found in
any print read; the key keeps it as the tradition's name (C1343). The
lunar month is *Karmakanda Bhaskara*'s, with *adhika* compounded before an
adhika month (C1345). The Gita Press *Nitya Karma Puja Prakash* gives the
Indian form (`notes/research/source-links.md` §6); it is not shipped, and
a consumer who wants it loads it as a tradition.

The template is said in Devanagari in `sa-Deva`, `ne-Deva-NP` and
`hi-Deva-IN` (the words are Sanskrit in each) and in IAST in `en-Latn` and
`sa-Latn`. The locatives of the ayana and ritu are entity forms
(`locative`: *uttarāyaṇe*, *vasantartau* …). The weekday's stem (*ravi*,
*soma*, *bhauma* …) is the template's own sub-message, a `.match` on
`varaKey`: as an entity form, *guru* and *budha* would fold to the
planets' names and make a lexicon search for "guru" ambiguous. The other
members are compounded as their stems, as the print does
(*amukanakṣatre*).

Ashauch, the other half of P2-3, has its default: `DHARMASINDHU`, decided
2026-10-27 under the maintainer's delegation (`ashauch.md`, C713).

[drik]: https://www.drikpanchang.com/panchang/sankalpa/sankalpa.html
[bodh]: https://panchangbodh.com/sankalp-mantra
