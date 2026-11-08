# Brief and full answers, measured

Status: `generated` by `cargo xtask detail-tokens` and held by
`check-detail-tokens`; do not edit. Each record's example request
(`crates/sdk/tests/requests.json`) answered over the built-in ephemeris
at both details (`result-detail.md`), each written as the envelope an
agent reads (`Answered::envelope`, canonical JSON: what the agent server
sends as a tool's text), and counted by the dependency-free
approximation of `xtask/src/tokens.rs` (C1165), the ruler
`dossier-tokens-measured.md` describes: read the columns against each
other, not as a model's bill. An agent server's result carries the same
JSON twice, as text and as structured content, so a host that reads both
pays each figure twice.

## The claims

| proposed rule | verdict | measured |
|---|---|---|
| every path a brief keeps is a property of its record's answer schema, or runs into a value the schema leaves open | **holds** | 0 of 242 disagree |
| no brief leaves out the tag a `oneOf` branch is told apart by | **holds** | 0 of 242 disagree |
| every brief that keeps fields answers its example's value in fewer tokens than the full answer | **holds** | 0 of 18 disagree |
| every brief that keeps fields belongs to a record whose answer states a schema | **holds** | 0 of 18 disagree |
| every record set aside here is a record, and refuses its example over the built-in ephemeris | **holds** | 0 of 2 disagree |

## Over 51 records

| | full | brief | brief against full |
|---|---|---|---|
| the envelope | 342 514 | 99 766 | 29.1% |
| the value | 324 270 | 96 577 | 29.8% |
| the provenance | 17 705 | 2390 | 13.5% |

18 of the 51 records answered keep fields in brief; the rest answer their whole value, and every one writes the brief provenance (`Brief::PROVENANCE`).

13 of the 242 paths run into a value its schema leaves open, a section the answer carries as any JSON; what lies below is the section's own, unchecked here: `almanac.days` `muhurta.value.windows.at`, `almanac.days` `muhurta.value.windows.barredBy`, `almanac.days` `muhurta.value.windows.score`, `almanac.days` `muhurta.value.closed`, `almanac.days` `muhurta.value.daysJudged`, `almanac.days` `muhurta.value.daysCut`, `almanac.days` `muhurta.value.windowsBlackedOut`, `almanac.days` `muhurta.value.ranking`, `almanac.days` `muhurta.value.unjudged`, `almanac.days` `sky.value.from`, `almanac.days` `sky.value.to`, `almanac.days` `sky.value.sun.state`, `almanac.days` `sky.value.moon.state`.

Set aside:

- `engine.call`: asks an engine's own operation, which the built-in ephemeris has none of; its value is the engine's JSON, unread by the SDK, so its brief is the whole (refused here: UNSUPPORTED).
- `engine.manifest`: describes an engine's own operations, which the built-in ephemeris has none of; its brief is the whole (refused here: UNSUPPORTED).

## Each record

Tokens of the envelope at each detail, and what the brief keeps of the
value.

| record | full | brief | brief against full | the brief keeps |
|---|---|---|---|---|
| `almanac.anniversaries` | 17 241 | 6370 | 36.9% | `anniversaries.key`, `anniversaries.kind`, `anniversaries.origin`, `anniversaries.observance.rule`, `anniversaries.observance.day`, `anniversaries.observance.tithi`, `anniversaries.observance.month`, `anniversaries.observance.adhika`, `anniversaries.observance.case`, `anniversaries.observance.choice`, `ekadashis`, `observances`, `unjudged` |
| `almanac.ashauch` | 952 | 193 | 20.3% | `key`, `kind`, `relation`, `start`, `end`, `length`, `days` |
| `almanac.ashauchKin` | 144 | 144 | 100.0% | the whole value |
| `almanac.ashauchRite` | 106 | 106 | 100.0% | the whole value |
| `almanac.changes` | 763 | 250 | 32.8% | the whole value |
| `almanac.days` | 151 706 | 11 734 | 7.7% | `days.day`, `days.limbs`, `days.kaalas`, `days.durmuhurtas`, `days.month`, `days.moon`, `days.sun`, `days.omens`, `festivals.value`, `years.value`, `eclipses.value`, `nepalSambat.value`, `muhurta.value.windows.at`, `muhurta.value.windows.barredBy`, `muhurta.value.windows.score`, `muhurta.value.closed`, `muhurta.value.daysJudged`, `muhurta.value.daysCut`, `muhurta.value.windowsBlackedOut`, `muhurta.value.ranking`, `muhurta.value.unjudged`, `sky.value.from`, `sky.value.to`, `sky.value.sun.state`, `sky.value.moon.state` |
| `almanac.deathTime` | 808 | 310 | 38.4% | the whole value |
| `almanac.digest` | 3453 | 180 | 5.2% | `format`, `days`, `size`, `seal` |
| `almanac.elect` | 2572 | 877 | 34.1% | `windows.rank`, `windows.at`, `windows.open`, `windows.holds.clause`, `windows.holds.grade`, `windows.fails.clause`, `windows.barredBy`, `closed`, `daysJudged`, `daysCut`, `windowsBlackedOut`, `ranking`, `unjudged` |
| `almanac.festivalPack` | 3311 | 1644 | 49.7% | `pack`, `rules.key`, `rules.form`, `rules.meta`, `rules.subjectTo`, `rules.lastYear` |
| `almanac.festival_calendar` | 6495 | 5861 | 90.2% | the whole value |
| `almanac.firstYear` | 12 373 | 4196 | 33.9% | `key`, `origin`, `dayOne`, `eleventhDay`, `sapindana`, `rites.rite`, `rites.place`, `rites.month`, `rites.adhika`, `rites.day`, `rites.form`, `rites.year`, `rites.observance.day` |
| `almanac.household` | 1379 | 310 | 22.5% | `observers.key`, `observers.pureAt`, `observers.periods.key`, `observers.periods.kind`, `observers.periods.relation`, `observers.periods.start`, `observers.periods.end`, `observers.periods.length`, `observers.periods.days` |
| `almanac.navatara` | 1221 | 1221 | 100.0% | the whole value |
| `almanac.next` | 1534 | 498 | 32.5% | `day`, `keys`, `from`, `searchedDays`, `festivals.observances.rule`, `festivals.observances.day`, `festivals.observances.tithi`, `festivals.observances.month`, `festivals.observances.choice`, `festivals.ekadashis`, `festivals.unjudged` |
| `almanac.observance` | 35 908 | 18 129 | 50.5% | `traditions.name`, `traditions.rule`, `traditions.reckoning`, `traditions.profile`, `occurrences.month`, `occurrences.adhika`, `occurrences.kept.tradition`, `occurrences.kept.day`, `occurrences.kept.fast.rule`, `occurrences.kept.fast.tithi`, `occurrences.kept.fast.month`, `occurrences.kept.fast.adhika`, `occurrences.kept.fast.choice`, `occurrences.kept.fast.day`, `occurrences.kept.fast.parana.day`, `occurrences.kept.fast.parana.kept.span`, `occurrences.agree`, `occurrences.differs`, `occurrences.reason`, `unjudged`, `parted` |
| `almanac.pakshi` | 13 958 | 2734 | 19.6% | `date`, `reading.day`, `reading.bird`, `reading.deathBird`, `reading.deadToday`, `reading.eaters`, `reading.yamas.half`, `reading.yamas.yama`, `reading.yamas.span`, `reading.yamas.activity`, `reading.yamas.quality` |
| `almanac.people` | 3742 | 1744 | 46.6% | `day`, `sunrise`, `nextSunrise`, `nakshatra`, `moonSign`, `dasha`, `people.name`, `people.birthStar`, `people.moonSign`, `people.lagna`, `people.dasha`, `people.tara`, `people.chandraHouse`, `people.chandrashtama`, `people.sadeSati`, `people.transits.graha`, `people.transits.house`, `people.transits.verdict`, `people.hits` |
| `almanac.publishedReasons` | 747 | 118 | 15.8% | the whole value |
| `almanac.sankalpa` | 1214 | 709 | 58.4% | the whole value |
| `almanac.sentences` | 3628 | 1010 | 27.8% | `sentences.text`, `locale`, `withheld`, `draft` |
| `almanac.skyView` | 3724 | 622 | 16.7% | `at`, `grahas`, `stars` |
| `calendar.add` | 149 | 149 | 100.0% | the whole value |
| `calendar.between` | 28 | 28 | 100.0% | the whole value |
| `calendar.convert` | 149 | 149 | 100.0% | the whole value |
| `chart.astrocartography` | 4219 | 3693 | 87.5% | the whole value |
| `chart.disagreements` | 1449 | 886 | 61.1% | `otherProfile`, `at`, `compared`, `lagna`, `grahas`, `nakshatra`, `dashas`, `yogas` |
| `chart.found` | 33 149 | 17 360 | 52.4% | `charts.foundation.instant`, `charts.foundation.place`, `charts.foundation.kind`, `charts.foundation.zodiac`, `charts.foundation.lagna_deg`, `charts.foundation.houses`, `charts.foundation.grahas.graha`, `charts.foundation.grahas.longitude_deg`, `charts.foundation.grahas.speed_deg_per_day`, `charts.foundation.grahas.placement`, `charts.foundation.grahas.house`, `charts.panchanga`, `charts.vargas`, `charts.state`, `charts.aspects`, `charts.points`, `charts.houses`, `charts.ashtakavarga`, `charts.vimshopaka`, `charts.vaiseshikamsa`, `charts.shadbala.grahas.graha`, `charts.shadbala.grahas.rupas`, `charts.shadbala.grahas.required_rupas`, `charts.shadbala.grahas.strong`, `charts.bhava_bala`, `charts.dasha_phala`, `charts.jaimini`, `charts.avakahada`, `charts.moment`, `charts.drawings`, `charts.dashas`, `readings`, `plans`, `sade_sati`, `gochar`, `hits`, `sarvatobhadra`, `dossier`, `dignities`, `fortitudes.scores`, `fortitudes.planets`, `fortitudes.almutens`, `lots.sect`, `lots.lots`, `considerations`, `perfections`, `matchings`, `svgs`, `kp.chart.planets`, `kp.significators`, `kp.ruling`, `prashna`, `remedies`, `lalkitab`, `rectification`, `varsha`, `tithi_pravesha`, `masa_pravesha`, `western` |
| `chart.mundane` | 706 | 182 | 25.8% | the whole value |
| `chart.rashifal` | 17 454 | 4641 | 26.6% | `period.first`, `period.last`, `period.instant`, `period.transits`, `period.retrograde`, `period.panchanga`, `period.readings.rashi`, `period.readings.saturn`, `period.readings.events`, `baseline` |
| `chart.relocated` | 1008 | 494 | 49.0% | the whole value |
| `intl.date` | 237 | 237 | 100.0% | the whole value |
| `intl.dates` | 252 | 252 | 100.0% | the whole value |
| `intl.inWords` | 210 | 210 | 100.0% | the whole value |
| `intl.lexicon` | 3193 | 3193 | 100.0% | the whole value |
| `intl.lookup` | 329 | 329 | 100.0% | the whole value |
| `intl.time` | 183 | 183 | 100.0% | the whole value |
| `matching.naam` | 1293 | 1293 | 100.0% | the whole value |
| `naming.check` | 1013 | 1013 | 100.0% | the whole value |
| `naming.of` | 120 | 120 | 100.0% | the whole value |
| `naming.read` | 1125 | 1125 | 100.0% | the whole value |
| `naming.suggest` | 584 | 584 | 100.0% | the whole value |
| `naming.syllables` | 1323 | 1323 | 100.0% | the whole value |
| `numerology.profile` | 889 | 889 | 100.0% | the whole value |
| `research.study` | 1364 | 851 | 62.4% | the whole value |
| `time.civil` | 264 | 264 | 100.0% | the whole value |
| `time.convert` | 104 | 104 | 100.0% | the whole value |
| `time.resolve` | 210 | 210 | 100.0% | the whole value |
| `verify.claims` | 905 | 412 | 45.5% | the whole value |
| `verify.issue` | 3314 | 320 | 9.7% | `answer`, `receipt.format`, `receipt.record`, `receipt.requestDigest`, `receipt.outputDigest`, `receipt.digest` |
| `verify.receipt` | 312 | 312 | 100.0% | the whole value |

## What each brief leaves out

Every property the record's answer schema states beside a path its brief
keeps and the brief does not keep, read from the schema rather than from
the example, so a field the example leaves empty is listed too. A field
added to an answer is left out of its brief until its record's list
names it.

- `almanac.anniversaries`: `added`, `anniversaries.count`, `anniversaries.dayRule`, `anniversaries.monthBy`, `anniversaries.observance.decidedBy`, `anniversaries.observance.extents`, `anniversaries.observance.marginSeconds`, `anniversaries.observance.moonrise`, `anniversaries.observance.parana`, `anniversaries.observance.published`, `anniversaries.observance.sait`, `applied`, `dated`, `unplaced`.
- `almanac.ashauch`: `clause`, `dayOne`, `dayOneBy`, `lastDay`, `overlap`, `ritesFrom`, `sex`, `table`.
- `almanac.days`: `dayHashes`, `days.choghadiya`, `days.clock`, `days.ghati_reckoning`, `days.gowri`, `days.homa_ahuti`, `days.horas`, `days.lagnas`, `days.markers`, `days.muhurtas`, `days.named_panchaka`, `days.punya_kala`, `days.shiva_vasa`, `days.window`, `eclipses.provenance`, `festivals.provenance`, `muhurta.provenance`, `nepalSambat.provenance`, `sky.provenance`, `years.provenance`.
- `almanac.digest`: `base64`.
- `almanac.elect`: `windows.fails.at`, `windows.fails.by`, `windows.fails.choghadiya`, `windows.fails.count`, `windows.fails.grade`, `windows.fails.grahas`, `windows.fails.holds`, `windows.fails.house`, `windows.fails.kaala`, `windows.fails.karana`, `windows.fails.lord`, `windows.fails.masa`, `windows.fails.nakshatra`, `windows.fails.navamsa`, `windows.fails.pada`, `windows.fails.panchaka`, `windows.fails.reading`, `windows.fails.row`, `windows.fails.second`, `windows.fails.sign`, `windows.fails.tithi`, `windows.fails.twelfth`, `windows.fails.vara`, `windows.fails.with`, `windows.fails.yoga`, `windows.holds.at`, `windows.holds.by`, `windows.holds.choghadiya`, `windows.holds.count`, `windows.holds.grahas`, `windows.holds.holds`, `windows.holds.house`, `windows.holds.kaala`, `windows.holds.karana`, `windows.holds.lord`, `windows.holds.masa`, `windows.holds.nakshatra`, `windows.holds.navamsa`, `windows.holds.pada`, `windows.holds.panchaka`, `windows.holds.reading`, `windows.holds.row`, `windows.holds.second`, `windows.holds.sign`, `windows.holds.tithi`, `windows.holds.twelfth`, `windows.holds.vara`, `windows.holds.with`, `windows.holds.yoga`, `windows.score`.
- `almanac.festivalPack`: `rules.source`, `rules.unsourced`.
- `almanac.firstYear`: `dayOneBy`, `rites.bars`, `rites.clause`, `rites.drawnTo`, `rites.observance.adhika`, `rites.observance.case`, `rites.observance.choice`, `rites.observance.decidedBy`, `rites.observance.extents`, `rites.observance.marginSeconds`, `rites.observance.month`, `rites.observance.moonrise`, `rites.observance.parana`, `rites.observance.published`, `rites.observance.rule`, `rites.observance.sait`, `rites.observance.tithi`.
- `almanac.household`: `observers.periods.clause`, `observers.periods.dayOne`, `observers.periods.dayOneBy`, `observers.periods.lastDay`, `observers.periods.overlap`, `observers.periods.ritesFrom`, `observers.periods.sex`, `observers.periods.table`.
- `almanac.next`: `festivals.added`, `festivals.anniversaries`, `festivals.applied`, `festivals.dated`, `festivals.observances.adhika`, `festivals.observances.case`, `festivals.observances.decidedBy`, `festivals.observances.extents`, `festivals.observances.marginSeconds`, `festivals.observances.moonrise`, `festivals.observances.parana`, `festivals.observances.published`, `festivals.observances.sait`, `festivals.unplaced`, `limitDays`.
- `almanac.observance`: `occurrences.kept.fast.days`, `occurrences.kept.fast.excess`, `occurrences.kept.fast.mahadvadashi`, `occurrences.kept.fast.marginSeconds`, `occurrences.kept.fast.parana.barred`, `occurrences.kept.fast.parana.hariVasaraEnds`, `occurrences.kept.fast.parana.kept.at`, `occurrences.kept.fast.parana.kept.fromBy`, `occurrences.kept.fast.parana.kept.toBy`, `occurrences.kept.fast.parana.tithi`, `occurrences.kept.fast.pierced`, `occurrences.kept.fast.piercedAt`, `occurrences.kept.fast.published`, `occurrences.kept.fast.tithis`, `occurrences.kept.observance`, `traditions.provider`, `traditions.settingsHash`.
- `almanac.pakshi`: `reading.yamas.subs`.
- `almanac.people`: `people.dashaOpens`, `people.transits.fruitfulNow`, `people.transits.fruition`, `people.transits.goodHouse`, `people.transits.obstructedBy`, `people.transits.transit`, `people.transits.vedhaHouse`.
- `almanac.sentences`: `sentences.key`, `sentences.params`.
- `almanac.skyView`: `atmosphere`, `ecliptic`, `eclipticStepDeg`, `horizon`, `horizonStepDeg`, `obliquityDeg`, `siderealTimeDeg`.
- `chart.disagreements`: `otherSettingsHash`.
- `chart.found`: `charts.foundation.chalit`, `charts.foundation.day`, `charts.foundation.day_lagna_deg`, `charts.foundation.grahas.distance_au`, `charts.foundation.grahas.latitude_deg`, `charts.foundation.grahas.tropical_deg`, `charts.foundation.lilith`, `charts.foundation.outer`, `charts.foundation.steps`, `charts.foundation.timing`, `charts.shadbala.grahas.ashubha_rashmi`, `charts.shadbala.grahas.cheshta`, `charts.shadbala.grahas.dig`, `charts.shadbala.grahas.drik`, `charts.shadbala.grahas.ishta`, `charts.shadbala.grahas.kaala`, `charts.shadbala.grahas.kashta`, `charts.shadbala.grahas.naisargika`, `charts.shadbala.grahas.sthana`, `charts.shadbala.grahas.subha_rashmi`, `charts.shadbala.grahas.virupas`, `charts.shadbala.rules`, `fortitudes.dignities`, `fortitudes.rules`, `fortitudes.sky`, `hashes`, `kp.chart.cusps`, `kp.chart.system`, `lots.fortuneReversed`, `lots.request`.
- `chart.rashifal`: `period.readings.gochar`, `period.reference`.
- `verify.issue`: `receipt.calculationVersion`, `receipt.catalogueVersion`, `receipt.locale`, `receipt.packs`, `receipt.provider`, `receipt.request`, `receipt.sdkVersion`, `receipt.settings`, `receipt.tzdb`.

## The fields of the larger answers

Each record whose full answer is 400 tokens or more, its value's
top-level fields (an answer that is a list, its items' fields summed)
with their tokens, largest first; a field the brief keeps any of is
marked.

- `almanac.anniversaries`: `anniversaries` 16 404 (kept), `ekadashis` 8 (kept), `observances` 8 (kept), `unjudged` 7 (kept).
- `almanac.ashauch`: `dayOne` 83, `lastDay` 83, `clause` 64, `dayOneBy` 15, `length` 15 (kept), `end` 11 (kept), `start` 11 (kept), `table` 10, `key` 9 (kept), `relation` 9 (kept), `kind` 8 (kept), `days` 5 (kept).
- `almanac.changes`: `points` 101 (kept), `window` 32 (kept).
- `almanac.days`: `sky` 96 695 (kept), `muhurta` 33 782 (kept), `days` 15 108 (kept), `years` 2437 (kept), `festivals` 1239 (kept), `nepalSambat` 880 (kept), `eclipses` 677 (kept), `dayHashes` 115.
- `almanac.deathTime`: `death` 88 (kept), `cremation` 86 (kept), `table` 10 (kept), `rites` 7 (kept).
- `almanac.digest`: `base64` 2622, `seal` 38 (kept), `format` 6 (kept), `size` 6 (kept), `days` 5 (kept).
- `almanac.elect`: `windows` 1478 (kept), `unjudged` 122 (kept), `ranking` 9 (kept), `windowsBlackedOut` 9 (kept), `closed` 7 (kept), `daysJudged` 7 (kept), `daysCut` 6 (kept).
- `almanac.festivalPack`: `rules` 3290 (kept), `pack` 11 (kept).
- `almanac.festival_calendar`: .
- `almanac.firstYear`: `rites` 11 398 (kept), `origin` 87 (kept), `eleventhDay` 84 (kept), `sapindana` 84 (kept), `dayOne` 83 (kept), `dayOneBy` 15, `key` 9 (kept).
- `almanac.household`: `observers` 767 (kept).
- `almanac.navatara`: `nakshatra` 293 (kept), `tara` 225 (kept), `special` 182 (kept), `count` 162 (kept), `cycle` 162 (kept).
- `almanac.next`: `festivals` 493 (kept), `day` 82 (kept), `from` 82 (kept), `keys` 13 (kept), `limitDays` 7, `searchedDays` 7 (kept).
- `almanac.observance`: `occurrences` 34 698 (kept), `traditions` 449 (kept), `unjudged` 7 (kept), `parted` 6 (kept).
- `almanac.pakshi`: `reading` 12 945 (kept), `date` 231 (kept).
- `almanac.people`: `people` 2853 (kept), `day` 59 (kept), `nextSunrise` 13 (kept), `sunrise` 13 (kept), `dasha` 10 (kept), `nakshatra` 10 (kept), `moonSign` 9 (kept).
- `almanac.publishedReasons`: `reasons` 7 (kept).
- `almanac.sankalpa`: `year` 226 (kept), `date` 82 (kept), `month` 75 (kept), `tithi` 15 (kept), `nakshatra` 14 (kept), `ayana` 13 (kept), `nextSunrise` 13 (kept), `sunrise` 13 (kept), `jupiter` 12 (kept), `karana` 12 (kept), `paksha` 12 (kept), `sun` 12 (kept), `ayanaReckoning` 11 (kept), `moon` 11 (kept), `rituReckoning` 11 (kept), `at` 10 (kept), `ritu` 10 (kept), `vara` 10 (kept), `yoga` 10 (kept).
- `almanac.sentences`: `sentences` 2833 (kept), `locale` 12 (kept), `withheld` 7 (kept), `draft` 6 (kept).
- `almanac.skyView`: `ecliptic` 1687, `horizon` 796, `grahas` 262 (kept), `stars` 227 (kept), `atmosphere` 26, `siderealTimeDeg` 14, `obliquityDeg` 13, `at` 9 (kept), `eclipticStepDeg` 8, `horizonStepDeg` 8.
- `chart.astrocartography`: `lines` 3496 (kept), `greenwichRamcDeg` 14 (kept), `obliquityDeg` 13 (kept), `projection` 12 (kept), `instant` 10 (kept), `toleranceDeg` 10 (kept), `latitudeLimitDeg` 8 (kept), `stepDeg` 6 (kept).
- `chart.disagreements`: `dashas` 388 (kept), `yogas` 157 (kept), `lagna` 96 (kept), `nakshatra` 83 (kept), `otherSettingsHash` 47, `otherProfile` 15 (kept), `at` 9 (kept), `grahas` 7 (kept), `compared` 6 (kept).
- `chart.found`: `charts` 11 917 (kept), `kp` 10 476 (kept), `fortitudes` 6978 (kept), `lots` 1720 (kept), `western` 931 (kept), `varsha` 252 (kept), `hashes` 83, `tithi_pravesha` 10 (kept), `considerations` 9 (kept), `masa_pravesha` 9 (kept), `rectification` 9 (kept), `sarvatobhadra` 9 (kept), `dignities` 8 (kept), `matchings` 8 (kept), `perfections` 8 (kept), `sade_sati` 8 (kept), `dossier` 7 (kept), `gochar` 7 (kept), `lalkitab` 7 (kept), `plans` 7 (kept), `prashna` 7 (kept), `readings` 7 (kept), `remedies` 7 (kept), `hits` 6 (kept), `svgs` 6 (kept).
- `chart.mundane`: `instant` 12 (kept), `kind` 12 (kept), `latitudeDeg` 10 (kept), `longitudeDeg` 9 (kept), `utcOffsetSeconds` 9 (kept), `altitudeM` 8 (kept).
- `chart.rashifal`: `period` 13 293 (kept), `baseline` 3488 (kept).
- `chart.relocated`: `places` 350 (kept), `obliquityDeg` 13 (kept), `instant` 10 (kept).
- `intl.lexicon`: `entries` 3171 (kept), `locale` 12 (kept).
- `matching.naam`: `porutham` 602 (kept), `ashta` 563 (kept), `bride` 41 (kept), `groom` 39 (kept), `varga` 35 (kept).
- `naming.check`: `check` 985 (kept), `refused` 12 (kept).
- `naming.read`: `reading` 1097 (kept), `refused` 12 (kept).
- `naming.suggest`: `names` 390 (kept), `prescribed` 134 (kept), `book` 41 (kept), `refused` 7 (kept).
- `naming.syllables`: `initials` 463 (kept), `devanagari` 130 (kept), `rashi` 130 (kept), `nakshatra` 129 (kept), `varga` 104 (kept), `iast` 99 (kept), `cell` 65 (kept), `pada` 65 (kept).
- `numerology.profile`: `chaldeanName` 348 (kept), `pythagoreanName` 348 (kept), `pythagoreanBirth` 114 (kept), `chaldeanBirth` 60 (kept), `baseline` 6 (kept).
- `research.study`: `rows` 733 (kept).
- `verify.claims`: `claims` 297 (kept).
- `verify.issue`: `receipt` 3157 (kept), `answer` 147 (kept).
