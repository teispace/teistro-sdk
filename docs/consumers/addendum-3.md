# Reply to the consumer's addenda 2, 3, 4 and 5

Addendum 3 (2026-10-29) asks for 58 general features, SR-01 to SR-58, and sends
addendum 2 (A2-1 to A2-10) with it, now approved. This page answers each by number:
its **status** (`done`, `partly built`, `planned`, `needs a decision`, `declined`), **where** it
lives or will live, and any **question**. It was checked against `main` at `0b69926d`.
The schedule is in `07-roadmap/00-roadmap.md`, "Addendum 3". Addendum 4 (2026-10-29) answered
this page's questions and added SR-59 to SR-62; its answers are folded into the rows below and its
requests are in §10. Addendum 5 (2026-10-31) added SR-63 to SR-76 and moved SR-58; its requests
are in §11, each checked against `batch/2026-10-31a` (PR #423).

**Order.** The v1.0 release candidate comes first and keeps its scope. After it come the
consumer's P0 requests, then P1, P2, and P3 and later. A request is done when it is
gated, documented, in the IDL and in every binding, and in an MCP tool where it asks for one.

**Names.** The consumer is not named in this repository, so this page's path names
the addendum and not the product. It is a deliberate change from the addendum's suggested path.

## 1. Calendar and time

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-01 | partly built | `panchanga/src/festival/nepal_panchanga.rs`; STATUS P1-5, P1-6 | 267 of 502 festivals ship as rules: Fagu purnima joined from the gazette notices (C693). Still open: 24 rows that need a rule form (C695, C608, C614) and 9 rule festivals with open cruxes (C685–C687, C689). Each crux says why: the gazette notices do not list these days and no rank-1 text in hand sets them, so no rule is forced. |
| SR-02 | done | `panchanga/src/festival/meta.rs`; `almanac.festivalPack` and a festival request's `only` in the IDL, every binding and MCP | Every shipped rule carries `meta`: kind, tradition, communities and regions, each from a named source (`03-design/festival-metadata.md`), and the `festival_kind` and `tradition` catalogue kinds. A rule with no sourced value says so in `unsourced`: 175 of Nepal's 359 keys have no kind (C850). A request filters by them with `only`. |
| SR-03 | partly built | `panchanga/src/festival/india_central.rs`, pack `INDIA_CENTRAL`, `festival-measured.md` §9 | **The central list is built** from seven years of the Department of Personnel and Training's memoranda; the four days set by the moon ship as printed, marked `MOON_SIGHTING`, and are not reckoned (`03-design/india-central-holidays.md`). State sets are next. Plan: central holidays first, from the Government of India's yearly gazette notices of central holidays (rank 1 for the list; each day's rule is cited to a text or a printed panchang). State sets follow, one gated pack per state. The regional festival packs are P2. 58 of the 63 reckoned rows meet the notices; five part and are named with their causes, none fitted: Holi 2023, 2026, 2027 (C854), Dussehra 2022 (C855), Diwali 2024 (C856). |
| SR-04 | done | `almanac.anniversaries`; `03-design/anniversary-span.md` | One call over 1 to 100 years in any shipped calendar. Every day's `dayRule` says whether the tithi was `KSHAYA` or `VRIDDHI`, the window read, the clause that chose the day and where it is stated. The same field is on `festivals`' anniversaries. |
| SR-05 | done | `calendar/src/arithmetic.rs` (`add`, `between`, `Overflow`, `DateSpan`); `calendar.add` and `calendar.between` in the IDL, every binding and MCP | Years, months and days are added in any month-counting calendar, Bikram Sambat included. A difference is counted as an age: the whole months whose anniversary is reached, then the days. A day the target month lacks is resolved by `CONSTRAIN` (the default), `CARRY` or `REJECT`. The research and the rule are in `03-design/calendar-bikram-sambat.md` §11. A birthday on a 32nd that falls in a shorter month is crux C806. |
| SR-06 | done | `panchanga/src/festival/next.rs`, `sdk/src/area/almanac/next.rs`; `almanac.next` in the IDL, every binding and MCP | The next day of a kind (`EKADASHI`, `PURNIMA`, `AMAVASYA`, `SANKRANTI`) or of a rule in a festival request is found after an instant at a place. The look-ahead is 400 days by default, can be set from 1 to 3660, and gives `LIMIT` on `limitDays` when nothing falls within it (`03-design/next-occurrence.md`). Which day a Purnima is read on is crux C807. |
| SR-07 | done | `almanac.changes`, `03-design/change-points.md` | A range's change points: one list sorted by instant, each saying what ended and what began: tithi, nakshatra, yoga, karana, the Moon's sign, the Sun's sign and the vara. It reads the spans a range finds and asks for no rise or set. Its cost is measured in `change-points.md` §5; a budget row in `app-speed-measured.md` waits on device records. |
| SR-08 | built (= A2-10) | `almanac.digest`, `03-design/digest-format.md`, `bindings/c/digest` | A versioned, deterministic N-day digest, 60 days by default, with a reader in C that has no dependencies, so Swift and Kotlin call one reader. The 60-day size is held by `a_60_day_digest_keeps_its_size`, the bytes by a pinned hash and `check-parity`, the reader by `check-c` and the `digest` fuzz target. |
| SR-09 | done | `time.loadZones`, `time/src/zone/pack.rs`, `03-design/zone-packs.md` | The zone database loads from a signed data pack, verified as published sets are, with the caller's key. The embedded database stays as the fallback, per context (C830), and the pack's version is in every resolution and provenance. `cargo xtask zone-pack` builds a pack from a tree `zic` compiled. |
| SR-10 | dates done, charts planned | `time/src/zone/mod.rs`, `provenance.time.clock`, `03-design/time-and-timezone.md` §3.4 | Resolving a zone says which rules gave the offset: the zone database, local mean time from the longitude, or a fixed offset the caller stated. For Kathmandu that is +05:45 today, +05:30 in 1985 and local mean time before the first rule. Every almanac answer now states its clock in `provenance.time.clock`, and a range can be asked by `zone` in place of an offset, each day then read on its own offset. Charts are P1. |
| SR-11 | done | `calendar/src/ical.rs`, `almanac.festival_calendar`, `03-design/icalendar.md` §6 | Merges events the caller supplies, an optional `VALARM`, category and colour hints, and a filter by form and rule, checked with `icalendar` as a black box in `check-python`. `kinds` and `traditions` are in the request and refused as unsupported until `classes_of` reads SR-02's fields (C870). |
| SR-12 | planned, review by the consumer | `intl` packs | The missing Hindi and Sanskrit names are drafted from the cited texts and ship when the consumer's reviewers sign them off (Q42, decided). No name is machine-stubbed. |
| SR-13 | done | `almanac.sentences`, `crates/sdk/src/area/almanac/sentences.rs` | The day's facts as short declarative sentences from templates, every value taken from the answer, in en and ne (`03-design/day-in-sentences.md`). The ne text is a draft, marked `draft` in the answer, and waits on Q42's review. **To confirm:** Q42 says a strict locale refuses a draft; read literally, the Nepali sentences would ship unmarked, so `ne-Deva-NP` drafts `sdk.day` as Hindi and Sanskrit draft `sdk.calendar` (C987). Hindi and Sanskrit wait on Q42's reviewer; until then those locales are refused by name, and a loaded pack carrying `sdk.day` answers. Festivals are not said; `almanac.festivals` holds them. |
| SR-14 | done | `calendar/src/hijri.rs` (`HijriOffset`); a festival request's `hijriOffset` in the IDL, every binding and MCP | −2 to 2 days added to the tabular date for every rule kept on a `HIJRI_TABULAR` date; a published set's day still lays over the moved one, the answer's provenance names `festivals.hijriOffset`, and past two days is `OUT_OF_RANGE` (`03-design/published-overrides.md` §7). |
| SR-15 | partly built | STATUS P2-1; `calendars/regional-solar.md` | Hindi already names a purnimanta month and an adhika month. Planned: the other locales. Kerala's Mesha (C764) and Assam (C765) month starts stay open on their sources. |
| SR-16 | planned (= A2-8) | `siddhanta`, a new profile | Research comes first: the Makaranda tables and the published Mithila panchang, gated over several years. |
| SR-17 | partly built | `panchanga/src/festival/ekadashi.rs` (the Vaishnava Ekadashi) | Planned: a Gaudiya festival pack, and Janmashtami's Vaishnava rule (C175). |

## 2. Panchanga

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-18 | done | `omen::bhadra_vasa`, `Panchanga::bhadra_vasa`, the day's `bhadra_vasa` section | Bhadra's loka by the Moon's sign during Vishti, from *Muhurta Chintamani* Shubhashubha-prakarana v. 45; a sign change inside a Vishti splits it; only the earth's is `harmful` (C820). |
| SR-19 | done | the day's `named_panchaka` | One span from the panchaka's first instant to the Moon leaving Revati, named by the vara it begins on. No text names it so; it ships as a printed convention, `panchanga.panchaka_name` (C821). |
| SR-20 | done | `docs/03-design/printed-conventions.md` | A printed reading with no text is a knob (the print's rule, `NONE`), an entry in the answer's `applied_conventions`, and catalogue members marked `T`. Shiva vasa is the first (`panchanga.shiva_vasa`, C750). |
| SR-21 | done | `panchanga.muhurta_tables = MUHURTA_CHINTAMANI`; `AlmanacRequest::with_markers`, the day's `markers` (`TS_PANCHANGA_MARKERS`) | Muhurta Chintamani's full yoga tables over every star of the day: Amrit Siddhi less its losing tithi, Sarvartha Siddhi, Guru and Ravi Pushya, Dwipushkar and Tripushkar by the verse's generator (C1050). Ganda mula as the almanacs' whole star, gandanta as the text's two ghatis a side, vinchudo as the Moon in Scorpio, each a span, asked (C1051). The text's Friday Sarvartha row ships; its footnote's Anuradha and the abhukta mula extents wait in the cruxes. `03-design/day-markers.md`. |
| SR-22 | done | `AlmanacRequest::with_gowri`, the day's `gowri` (`TS_PANCHANGA_GOWRI`) | The Tamil almanacs' Gowri panchangam: eight Gowri of the daylight and eight of the night, by the rule the printed table follows (the weekday lords from the vara's, Visham in rahu kaala). No text gives it; the table rests on rank-3 reprints (Drik Panchang, ePanchang) checked eighth by eighth, and a Tamil almanac in print is still wanted (C1052). The North Indian Gauri table is not shipped. `03-design/gowri-panchangam.md`. |

## 3. Family, rites and anniversaries

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-23 | done | `AnniversaryKind::Event`; `FestivalRequest::check` | `EVENT` keeps any event by the tithi at the window the request states, sunrise by default. No text rules on it, so it ships as a convention, `anniversary.event`, named in the answer (C911). A caller's rules are held to stated limits, each refused as `LIMIT` by its field: `anniversary-span.md` §6. |
| SR-24 | done | `almanac.anniversaries`; `05-testing/app-speed-measured.md`, the `anniversaries` row | 50 people over 10 years: 706 ms on this Mac against 28.4 s as 50 calls; 900 ms desktop budget, whose Android ratio is an emulator's (C913), so it says nothing of a low-end phone |
| SR-25 | partly built | the saits | Laxmi Puja ships the text's pradosha window; no printed lagna for it was found (C824). The Vijaya Dashami tika is not shipped: no rule fits all four printed years (C823). |
| SR-26 | done | `almanac.deathTime`; `03-design/death-time.md` | At the death and at the burning: the nakshatra, tithi, vara and the Moon's sign, whether it is the panchaka (the Moon in Aquarius or Pisces, which is Dhanishtha's second half to Revati, *Dharmasindhu* p. 334), the nakshatra's feet (tripada, dvipada) and any pushkara yoga, and the rites the text ties to them with their clauses (DS p. 335): the five figures for a burning in the panchaka, the shanti for a death in it or in a tripada nakshatra. The lists are a table a caller can change (C950–C954). |
| SR-27 | done | `almanac.ashauchKin`; `03-design/ashauch.md` §10 | A path of steps (`FATHER`, `MOTHER`, `SON`, `DAUGHTER`, `HUSBAND`, `WIFE`) gives the relation to ask `almanac.ashauch` with, what is kept and the degree counted from the common forebear, as DS pp. 299–302 count it: the observer's own number decides, each living forebear adds a person (NS p. 257), and the named kin are rows of a table (C955, C956, C961). |
| SR-28 | done | `almanac.household`, `almanac.ashauchRite`; `03-design/ashauch.md` §§11–12 | Several events and observers in one request, each observer's overlaps resolved in the order the news reached them (DS p. 305). "May this rite go on?" answers `GOES_ON`, `IN_PART` or `BARRED` with what opens the rite and its clause (DS pp. 309–310). Both are tables a caller can change (C957–C960). Reading DS p. 306 again corrected the shipped overlap rules: a father dying in the mother's period ends with hers under the *Dharmasindhu*'s table (the *Mitākṣarā*'s keeps his own), three-day periods add no last-night days, the added days take in only a pakṣiṇī or three days, and the fire-giver is never taken in. |
| SR-29 | partly built | `panchanga/src/sankalpa.rs`; `AlmanacArea::sankalpa` | The sankalpa's place-and-time words exist. Planned: a reviewed template per tradition, *Himavatkhande* first, with slots for the performer, gotra, place, purpose and three generations of pitris. The answer names its tradition. It needs a rank-1 printed form and Q42's reviewer. |
| SR-30 | done, with one part missing | `ffi/src/naming.rs`, the IDL and every binding (merged in #417) | `names_of`, `name_check` and `syllables` cross the boundary, and a consumer can supply its own name book (`03-design/namakarana.md`). The consumer counts it ready (addendum 4). The MCP tool follows at P1. |
| SR-31 | needs a decision | the `ne` pack | Drafting the ashauch, shraddha and first-year messages is planned. Shipping them waits on the Nepali review already listed in STATUS (Q42). |
| SR-32 | partly built | `matching/src/avakahada.rs` | Planned: the birth paya by both methods on the chart, from the Moon's house counted from the lagna and from the nakshatra. |

## 4. Muhurta

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-33 | planned | `muhurta`; C206 | A ruleset for the committee's vivaha sheet, with a documented divergence for every clause that still differs. |
| SR-34 | planned | `muhurta` | The house and business rites at P1: bhumi puja, shilanyas or griha arambha, vastu shanti, and a shop or business opening. Gem wearing, shiva vasa beside agni vasa for a rudri, and a lagna graded by the native's chart are P2. |
| SR-35 | planned | a new sourced vastu module | Aayadi: aaya, vyaya, the nakshatra and the others, each from its source. |

## 5. Jyotish

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-36 | partly built, waiting on the consumer's source | `render-svg`; `03-design/chart-geometry.md` | The Nepali lotus stays a style and is not the default. The consumer is checking the layout against printed janma patrikas and will send scans and a source (addendum 4). |
| SR-37 | partly built | `crates/rashifal` | Planned: composed readings for each sign and period (day, week, month, year) in ne, en and hi. The hi and ne text waits on Q42's review. |
| SR-38 | needs a decision | `intl` | The Hindi and Sanskrit composed text is drafted from the en templates and ships only after review, with sign-off of ne and hi (Q42). |
| SR-39 | planned, waiting on a source | `matching` | This is already on the list of waits: the bands need a rank-1 source. |
| SR-40 | planned | the dossier | A policy that drops named sections (longevity, the marakas), with the dropped list inside the hash. |
| SR-41 | planned | a chart counterpart of the almanac's `disagreements` | For two settings or two charts: which facts differ (the lagna, the nakshatra, the dasha lords, the yogas). |
| SR-42 | planned, at P3 | a new import module | Readers that map the JHora, Parashara's Light and Kala files to `FoundRequest`, written from the file formats and sample exports only, never from the programs' code. The consumer sends JHora exports when it specifies the feature; samples from the other two programs come from the professionals who own them (addendum 4). |
| SR-43 | planned | the dasha catalogue | The answer lists, per system, the translation that would unblock it, as data on the catalogue page. |
| SR-44 | done | `AlmanacArea::navatara`, `almanac.navatara`, `ts_navatara` | The navatara chakra of a birth star: the 27 stars in the count's order from the birth star, each with its tara and round of nine (Raman, *Muhurtha* ch. III), and the six special stars *Muhurta Chintamani* Nakshatra-prakarana v. 43 names, janma, karma, sanghatika, samudaya, vainashika and manasa (C1053). A king's jati, desha and abhisheka stars are not shipped. Every binding's `almanac.navatara`. `03-design/navatara-chakra.md`. |
| SR-45 | planned | `chart` | Mundane charts as named chart kinds: the Mesha and Makara sankrantis, lunations, eclipses and conjunctions. Event charts are cast from supplied records, each with its source. |
| SR-46 | done | `panchanga/src/rulers.rs`, `LunarYear.rulers` on `almanac.years`, `03-design/year-rulers.md` | The nine offices, each naming the texts that hold it: *Jyotirnibandha* p. 29 vv. 7–10 (six), *Narada Samhita* 2.1–3 (five) and the southern nine (*navanāyaka*), whose commander and lord of prices no verse read yet supports (C1071). Each office's weekday is answered under every rule the texts give, named: the raja by sunrise, by the new moon or by Garga's midday (C1070), an ingress by the daylight or by the sunrise-to-sunrise weekday (C1072). Held against printed rulers for VS 2080, 2082 and 2083. |
| SR-47 | planned | `western`, `astro` | Relocation, and astrocartography lines (feature universe ch. 15). |
| SR-48 | planned | `interpret` | The graha tridosha and tattva weights from BPHS ch. 3, marked as tradition, not health. |

## 6. Tools

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-49 | planned, in its P0 order | `intl` | Amounts in words in ne, hi, en and sa, with the Indian grouping (lakh, crore, arab), a currency's units, and Devanagari or Latin digits. The rules come from CLDR's rule-based number formats under the Unicode licence, which is compatible with Apache-2.0. The consumer confirmed the plan and ships without it if it is late (addendum 4). |

## 7. AI, knowledge and MCP (addendum 2)

| SR / A2 | Status | Where | Note or question |
|---|---|---|---|
| SR-50 | planned | the A2 rows below | Addendum 2 as a whole. Its MCP patterns (A2-6) are P0, and the rest P1. |
| A2-1 | planned | new records in `sdk` and MCP tools | The election search, the observance finder (with both answers and the reason they differ, P1-10) and the people digest, each with 1–3 input examples in its tool. |
| A2-2 | planned | the records | `detail: brief \| full` on results, with token sizes measured and documented. |
| A2-3 | planned | `intl` | `lexicon(locale)` and its reverse, from text to key, including romanised variants. |
| A2-4 | planned | `sdk` | `verify.claims`, built on A2-3. |
| A2-5 | planned | `rectification` | How many minutes the birth time can move before each fact changes. Nothing like it exists today. The search reuses the crate's windows. |
| A2-6 | built | `crates/mcp` (`elicit.rs`, `tasks.rs`, `tools.rs`; `tests/rounds.rs`) | **Built, 2026-10-30.** A record tool refused for a top-level field its request lacks (a birth's `instant`, a place's `latitudeDeg`) answers `resultType: "input_required"` with one form, `inputRequests["teistro.missing"]`, where the client declares form elicitation; the retry's `inputResponses` complete the call, and a declined form gets the refusal. Tasks were built before. The tool list's one order is held by a test. The SDK has no spans, so `traceparent` changes nothing, and a test holds that. `03-design/mcp-server.md`, "A missing input is asked for". |
| A2-7 | planned | the dossier | The dossier's token count per locale, measured, and a language-neutral mode beside the localised one. |
| A2-8 | planned | SR-16 | |
| A2-9 | partly built | `calendars/regional-solar.md` | The Tamil, Malayalam, Bengali and Odia calendars are built and measured against the *Rashtriya Panchang*. Planned: the Tamil school setting (vakya or thirukanitha). |
| A2-10 | built | SR-08 | |
| SR-51 | partly built | `core/src/envelope.rs` | The envelope already carries the SDK version, the settings hash and the digest of the inputs and the output. Missing: a receipt that names the record, and a deterministic `verify(receipt)` that recomputes and compares the result. Planned, as an attester for the OKF format's attested computation. |

## 8. Platform

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-52 | built | `ffi/src/support.rs` (`guarded`, `construct`); `bindings/node/lib/internal.js` | No panic crosses the C boundary: a caught one is the typed status `INTERNAL`, stable code -10, in every binding. On wasm, where a panic aborts, the layer refuses the trapping call and every later one with the same status until the module is reloaded (`03-design/wasm-binding.md` §4, "A panic traps"). `onInternalError(hook)` in the Node and wasm packages gives a host's crash reporter the status, code, kind and SDK version, never the message, and an unsubscribe; Python's `on_internal_error`, Dart's `onInternalError` and Java's `Teistro.onInternalError` give the same report, their only kind `boundary`. |
| SR-53 | built | `05-testing/sizes.json`, `SIZES.md`, `check-wasm`, `bindings/wasm/lib/lazy.js`, `03-design/wasm-lazy-loading.md` | The size of each wasm profile is gated both ways. `@teistro/sdk-wasm/lazy` loads the calendar module, then the jyotish module, when each is asked for, on every host `check-wasm` runs, and the same check times each entry's first answer under Node and in Chrome, and holds that a calendar page fetches the calendar module alone. |
| SR-54 | planned | the benches | A relative cost per entry point, generated from the callgrind bench's instruction counts, so it is measured and not written by hand. |
| SR-55 | needs a decision | `LICENSING` | The SDK and its packs are Apache-2.0 and the corpus is CC0. Whether a hosted API may serve Teimeris's output is the licence owner's call (Q43); the consumer has asked its own maintainer too (addendum 4). |
| SR-56 | planned (v1.x) | roadmap, v1.x | Swift and Kotlin bindings are already on the v1.x list. |

## 9. Research and far future

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-57 | planned | `research` | A dataset format with a reliability rating per record and a settings snapshot per study. |
| SR-58 | partly built, the sky view moved to P2 and the pose to P3 by addendum 5 | `astro` (topocentric altitude and azimuth, visibility) | Planned: every graha, the junction stars, the ecliptic and the horizon at a place and instant first (P2); then a record for a device pose and an instant, giving the grahas, the Moon's junction stars and the rising lagna point. |

## 10. Addendum 4's requests

All four are P1, and each is planned in the P1 wave.

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-59 | planned | `calendar` (the Vira Nirvana Samvat era) and a Jain festival pack in `panchanga` | The era, then the festivals and observances: Paryushana and Das Lakshana, Mahavir Janma Kalyanak, Mahavir Nirvana at Diwali, Ayambil Oli and the monthly fasting tithis. Each rule is cited, and the Shvetambara and Digambara differences are named as a setting. Research comes first: the era's epoch and year boundary, which the sources give differently. |
| SR-60 | partly built | `03-design/festival-rule-language.md` §3.1 (`MOONRISE`); the `DHARMASINDHU` pack | The moonrise window is built from *Dharmasindhu* p. 9: the first moonrise after the day's sunrise and before the next, so a moonrise after midnight belongs to the day it follows, and a day without a moonrise is left unjudged and says so. Sankashta chaturthi and the Karaka chaturthi (Karva Chauth, p. 76) are rules, 12 of 12 against a published 2025 list for Delhi. Planned: the moonrise time and place on the answer, and the other vratas broken at moonrise. |
| SR-61 | planned | `intl`, on A2-3's lexicon | Bikram Sambat and Gregorian dates found in free text in ne, en and hi, with Devanagari or Latin digits and month names in any script or romanised, returned as spans with the parsed date and its calendar. An ambiguous span is returned with every reading, not guessed. |
| SR-62 | partly built | `ParanaRule` on every `EkadashiRule` (Q41) | The Ekadashi parana is built. Planned: the fast's kind (nirjala, phalahar, ekabhukta), its start and its parana window as catalogue fields on every vrata, alongside SR-02's fields. |

## 11. Addendum 5's requests

Checked against the code, not only the request: three rows are already answered, and the
cause of two differs from what the addendum read.

| SR | Wave | Status | Where | Note or question |
|---|---|---|---|---|
| SR-63 | P0 | built | `astro/src/day_sky.rs`, `AlmanacRequest::with_sky`; `03-design/day-sky.md` | The day's sky, as a flag with a stated step, on Rust, C, every binding and `almanac.days`: a track for the Sun and the Moon with the exact rises, transits and sets as points with their altitudes, and on the Moon's the illuminated fraction, phase angle, waxing, bright-limb position angle and parallactic angle, topocentric. Held to JPL Horizons at four places. Polar days say `ALWAYS_ABOVE` or `ALWAYS_BELOW`. |
| SR-64 | P1 | done | `intl/src/lookup.rs`, `03-design/intl-engine-and-packs.md`, "Choosing a locale" | `set_locale` and a context's `locale` take a short tag by RFC 4647 lookup: `ne` chooses `ne-Deva-NP`, and `intl.locale` names the one chosen. `intl.lookupLocale` answers it without changing anything. There is no default: `de` and `hi-Latn` stay refused, and `hi` is `hi-Deva-IN` with that locale's completeness, so no call that refuses it starts answering (C1011). |
| SR-65 | P0 | closed for the corpus | the consumer's side | Answered 2026-10-31: published days stay out of the CC0 corpus, and modern panchangas stay cited and never redistributed. The 730-day gate runs on the consumer's side on every build. |
| SR-66 | P0 | done | `03-design/settings-and-profiles.md` §3.2, `PROFILE_ALIASES` in `core/src/settings/profiles.rs` | The profile is `nepali-drik`, documented in the profile table as a drik reckoning and not the official calendar, which is `nepali-committee` through `nepal_official`. `nepali-default` stays an accepted alias: the same settings, hash and days, and `profile` and `provenance.profile` say `nepali-drik`. The 59 of 730 stays the consumer's figure; the docs do not measure it. |
| SR-67 | P1 | decided, no build | Teimeris's `teimeris-sys/build.rs` and `docs/CONSUMING.md`, "Rust compiled for the browser" | `teimeris-sys` is not built for `wasm32-unknown-unknown` and its build script now refuses it by name instead of stopping at `math.h`: the target has no C library and the core needs stdio, malloc and libm, and supplying them would put a second, unchecked libc under the engine whose libm no gate compares, so the browser would get a third engine's numbers (C1031). The browser runs Teimeris's own Emscripten module, `bindings/wasm`, through the adapter's JavaScript provider (`teimerisWeb`, `adapters/ephemeris-teimeris/node/web.js`), which `check-wasm` holds byte-equal to the native month hash. A Rust provider calling that module through wasm-bindgen is the way a Rust browser app reaches it; it is described there, and not shipped or tested. |
| SR-68 | P1 | done for the build; publishing held | Teimeris `bindings/rust/teimeris-sys`; the adapter's `Cargo.toml` | Any checkout builds with no staging step: `bin2c.rs` and `embed.txt` are committed in the crate (`embed.txt` held to `tiers.json` by `tools/ci/embed_list.py`), and `build.rs` compiles the checkout's own `core/` and `data/` when nothing prebuilt is found, ahead of a staged copy. Teimeris's verify gains `plain_checkout`, which copies the tree with no `build/` and no staged core and runs the Rust tests there (50 sources compiled, every test passed); the same build at `3f169fc` fails. The adapter keeps the sibling path, which is what verify's `teimeris` job clones and now builds plainly; a version dependency waits on publishing, and a consumer pinning by commit uses a `git` dependency with a `rev`. |
| SR-69 | P1 | closed for now | release | Tags stay behind the no-publishing rule, and the consumer pins by commit. |
| SR-70 | P1 | done | `catalogue/tithi.yaml`, `Tithi::attributes` | `key()` withdrawn by the consumer, who now uses it. The tithi's number in the month is `tithi.attributes().month_number`, 1 to 30, beside `number`, 1 to 15 in the paksha. Each binding has `tithiMonthNumber` (`tithi_month_number` in Python, `AlmanacDay.tithiMonthNumber` in Java). |
| SR-71 | P1 | done | `sdk/src/context.rs`, `tests/threads.rs`; `03-design/rust-consumer-surface.md` §4, "A context across threads" | `Context` is `Send` and not `Sync`. `sdk.fork()` is another context over the same settings, ephemeris, cache, layouts and dasha systems, with a copy of the locale state; `sdk.share()` is a `SharedContext`, `Send + Sync` and cheap to clone, that a server keeps and asks for a context per request. Not `Sync` because a lock on the locale state would be paid by every render (C1030). Measured against the parent, interleaved: a build 14.9 µs before and after, a render 2,700 → 2,344 ns and a month 26.8 → 24.5 ms (within the machine's load), a fork 0.4 µs. |
| SR-72 | P1 | done | Teimeris `examples/apps/flutter_panchanga/pubspec.lock` | Reproduced, and not by the update: a fresh `git submodule update` of `3f169fc` was clean (no `.gitattributes`, the lock LF in the index and the tree). The first `flutter pub get`, which an editor runs on opening the folder, rewrote one line: the lock was written by a pre-release Flutter, whose pub recorded the SDK floor as `>=3.18.0-18.0.pre.54`, and pub on Flutter 3.47.9 writes `>=3.41.0` from the same packages. The lock now says what pub writes; the same steps on the new commit, update then `pub get`, leave the submodule clean. A Flutter whose SDK pins other package versions would still rewrite it, as it would any app's lock. |
| SR-73 | P1 | done | `almanac.inZone`, `AlmanacArea::in_zone`, `03-design/digest-format.md` | Accepted, and the offset is added. Every binding's day says `utcOffsetSeconds`, from a new `days` column of the panchanga blob. The digest, which is what `bindings/c/digest` reads, takes `zone`, and format 1.1 adds a `clock` section that the reader gives as `ts_digest_day.utc_offset_seconds`; it still reads format 1.0. The offset is the one at the day's opening (C1010). |
| SR-74 | P2 | teistro's part withdrawn | Teimeris | `proc-macro2`, `md-5` and `serde_bytes` are in use and are ignored with reasons in the manifests, as is the spike's `diplomat-runtime`. Only Teimeris's `swisseph-rs` remains. |
| SR-75 | P0 | built, unreviewed | `i18n/hi-Deva-IN`, `i18n/sa-Deva`; `03-design/intl-engine-and-packs.md` | Both locales carry the whole of `sdk.calendar` (68 messages), drafted by hand from CLDR 48 and standard usage, and ship marked: `_meta.json` `draft`, `isDraft` on every render and on `intl.time` and `intl.date`, held by `check-intl`, a strict locale refused one. By the consumer's answer it does not wait on Q42; the consumer verifies it later. Read first: Sanskrit's Hijri months, `निमेषः` and `क्षणः`, `वङ्गाब्दः`, `होरा`, `शकाब्दः`, and Hindi's Hijri months and `वृद्धि तिथि`. `sa-Latn` carries none of it and falls back, saying so. |
| SR-76 | P0 | built | `intl`, `IntlArea::time`, `IntlArea::date` | `intl.time` and `intl.date` in every binding and MCP: the locale's pattern and parts of the day, `style`, `hour12`, `digits` and `locale`, answering text, typed parts that join to it, the locale that answered and `isDraft`. A calendar with no pattern (the regional solar ones) is refused. |

**The polar note.** `NEAREST_EVENT`'s span is the intended reading: the day runs from the last
sunrise to the next, so it can hold many tithis. `03-design/settings-and-profiles.md` now says so,
and that `CIVIL_MIDNIGHT` is what a calendar shows. The consumer keeps `CIVIL_MIDNIGHT`.

**Order, by the consumer's answer of 2026-10-31.** SR-63, SR-75 and SR-76 unblock its Today screen
and come first.
