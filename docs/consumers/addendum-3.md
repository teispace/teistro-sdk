# Reply to the consumer's addenda 2, 3 and 4

Addendum 3 (2026-10-29) asks for 58 general features, SR-01 to SR-58, and sends
addendum 2 (A2-1 to A2-10) with it, now approved. This page answers each by number:
its **status** (`done`, `partly built`, `planned`, `needs a decision`, `declined`), **where** it
lives or will live, and any **question**. It was checked against `main` at `0b69926d`.
The schedule is in `07-roadmap/00-roadmap.md`, "Addendum 3". Addendum 4 (2026-10-29) answered
this page's questions and added SR-59 to SR-62; its answers are folded into the rows below and its
requests are in §10.

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
| SR-04 | partly built | `panchanga/src/festival/anniversary.rs` (`AnniversaryKind`, `anniversaries`) | A tithi anniversary resolves now. Planned: a call over a span of years that names the rule it applied to a kshaya or vriddhi tithi, on every day it returns. |
| SR-05 | done | `calendar/src/arithmetic.rs` (`add`, `between`, `Overflow`, `DateSpan`); `calendar.add` and `calendar.between` in the IDL, every binding and MCP | Years, months and days are added in any month-counting calendar, Bikram Sambat included. A difference is counted as an age: the whole months whose anniversary is reached, then the days. A day the target month lacks is resolved by `CONSTRAIN` (the default), `CARRY` or `REJECT`. The research and the rule are in `03-design/calendar-bikram-sambat.md` §11. A birthday on a 32nd that falls in a shorter month is crux C806. |
| SR-06 | done | `panchanga/src/festival/next.rs`, `sdk/src/area/almanac/next.rs`; `almanac.next` in the IDL, every binding and MCP | The next day of a kind (`EKADASHI`, `PURNIMA`, `AMAVASYA`, `SANKRANTI`) or of a rule in a festival request is found after an instant at a place. The look-ahead is 400 days by default, can be set from 1 to 3660, and gives `LIMIT` on `limitDays` when nothing falls within it (`03-design/next-occurrence.md`). Which day a Purnima is read on is crux C807. |
| SR-07 | done | `almanac.changes`, `03-design/change-points.md` | A range's change points: one list sorted by instant, each saying what ended and what began: tithi, nakshatra, yoga, karana, the Moon's sign, the Sun's sign and the vara. It reads the spans a range finds and asks for no rise or set. Its cost is measured in `change-points.md` §5; a budget row in `app-speed-measured.md` waits on device records. |
| SR-08 | planned (= A2-10) | a new export record and its format page | A versioned, deterministic N-day digest, 60 days by default, with a reader in C that has no dependencies, so Swift and Kotlin call one reader. The memory and size budgets are measured on a gate. |
| SR-09 | done | `time.loadZones`, `time/src/zone/pack.rs`, `03-design/zone-packs.md` | The zone database loads from a signed data pack, verified as published sets are, with the caller's key. The embedded database stays as the fallback, per context (C830), and the pack's version is in every resolution and provenance. `cargo xtask zone-pack` builds a pack from a tree `zic` compiled. |
| SR-10 | dates done, charts planned | `time/src/zone/mod.rs`, `provenance.time.clock`, `03-design/time-and-timezone.md` §3.4 | Resolving a zone says which rules gave the offset: the zone database, local mean time from the longitude, or a fixed offset the caller stated. For Kathmandu that is +05:45 today, +05:30 in 1985 and local mean time before the first rule. Every almanac answer now states its clock in `provenance.time.clock`, and a range can be asked by `zone` in place of an offset, each day then read on its own offset. Charts are P1. |
| SR-11 | partly built | `calendar/src/ical.rs` (`Feed`, `Event`, `write`) | The feed writer exists. Planned: merging events the caller supplies, an optional `VALARM`, category and colour hints, and a filter by kind or tradition, which needs SR-02's fields. |
| SR-12 | planned, review by the consumer | `intl` packs | The missing Hindi and Sanskrit names are drafted from the cited texts and ship when the consumer's reviewers sign them off (Q42, decided). No name is machine-stubbed. |
| SR-13 | planned | `interpret`, in ne, en, hi and sa | The day's facts composed as short declarative sentences from templates, every value taken from the answer. Hindi and Sanskrit wait on Q42's reviewer. |
| SR-14 | planned | `calendar/src/hijri.rs` | A request-level day offset (±1 or ±2) beside the published sighting sets (P2-4). |
| SR-15 | partly built | STATUS P2-1; `calendars/regional-solar.md` | Hindi already names a purnimanta month and an adhika month. Planned: the other locales. Kerala's Mesha (C764) and Assam (C765) month starts stay open on their sources. |
| SR-16 | planned (= A2-8) | `siddhanta`, a new profile | Research comes first: the Makaranda tables and the published Mithila panchang, gated over several years. |
| SR-17 | partly built | `panchanga/src/festival/ekadashi.rs` (the Vaishnava Ekadashi) | Planned: a Gaudiya festival pack, and Janmashtami's Vaishnava rule (C175). |

## 2. Panchanga

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-18 | done | `omen::bhadra_vasa`, `Panchanga::bhadra_vasa`, the day's `bhadra_vasa` section | Bhadra's loka by the Moon's sign during Vishti, from *Muhurta Chintamani* Shubhashubha-prakarana v. 45; a sign change inside a Vishti splits it; only the earth's is `harmful` (C820). |
| SR-19 | done | the day's `named_panchaka` | One span from the panchaka's first instant to the Moon leaving Revati, named by the vara it begins on. No text names it so; it ships as a printed convention, `panchanga.panchaka_name` (C821). |
| SR-20 | done | `docs/03-design/printed-conventions.md` | A printed reading with no text is a knob (the print's rule, `NONE`), an entry in the answer's `applied_conventions`, and catalogue members marked `T`. Shiva vasa is the first (`panchanga.shiva_vasa`, C750). |
| SR-21 | partly built | `chart` (gandanta) | Planned: ganda mula, vinchudo, Ravi and Guru Pushya and the special yogas as day spans, each with its source. |
| SR-22 | planned | `panchanga` | Gauri panchangam (Gowri nalla neram) day divisions, from a cited table. |

## 3. Family, rites and anniversaries

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-23 | partly built | `FestivalRequest::with_anniversary`; `AnniversaryKind` | Planned: a general anniversary kind for any event kept by tithi, with a stated window (sunrise by default). The festival rules a caller writes are a supported input now. Their limits will be documented and checked on the way in. |
| SR-24 | planned | the same | Batch anniversaries, about 50 people over 10 years in one call. The budget is gated on the desktop bench and scaled to a low-end Android phone by the ratio `app-speed-measured.md` records; the phone itself is not run in CI. |
| SR-25 | partly built | the saits | Laxmi Puja ships the text's pradosha window; no printed lagna for it was found (C824). The Vijaya Dashami tika is not shipped: no rule fits all four printed years (C823). |
| SR-26 | planned | `panchanga` | The death-time check: dhanishtha panchaka, and tripada and dwipada nakshatras at an instant, from a cited text. |
| SR-27 | planned | `panchanga` (ashauch) | A relation resolver: a path in a family graph gives the ashauch relation and the sapinda degree the texts use. |
| SR-28 | partly built | the ashauch rules (the sapinda degree and the default are decided) | Planned: several events and observers in one request, overlapping periods resolved (samapata), and "may this rite go on?" answered with its clause (apavada), all as presets a caller can change. |
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
| SR-44 | planned | `muhurta` or `chart` | The navatara (tara) chakra. |
| SR-45 | planned | `chart` | Mundane charts as named chart kinds: the Mesha and Makara sankrantis, lunations, eclipses and conjunctions. Event charts are cast from supplied records, each with its source. |
| SR-46 | planned | `panchanga` | The year's rulers (the raja, the mantri, the sasyadhipati and the rest), per cited tradition. |
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
| A2-10 | planned | SR-08 | |
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
| SR-58 | partly built | `astro` (topocentric altitude and azimuth, visibility) | Planned: a record for a device pose and an instant, giving the grahas, the Moon's junction stars and the rising lagna point. |

## 10. Addendum 4's requests

All four are P1, and each is planned in the P1 wave.

| SR | Status | Where | Note or question |
|---|---|---|---|
| SR-59 | planned | `calendar` (the Vira Nirvana Samvat era) and a Jain festival pack in `panchanga` | The era, then the festivals and observances: Paryushana and Das Lakshana, Mahavir Janma Kalyanak, Mahavir Nirvana at Diwali, Ayambil Oli and the monthly fasting tithis. Each rule is cited, and the Shvetambara and Digambara differences are named as a setting. Research comes first: the era's epoch and year boundary, which the sources give differently. |
| SR-60 | partly built | `03-design/festival-rule-language.md` §3.1 (`MOONRISE`); the `DHARMASINDHU` pack | The moonrise window is built from *Dharmasindhu* p. 9: the first moonrise after the day's sunrise and before the next, so a moonrise after midnight belongs to the day it follows, and a day without a moonrise is left unjudged and says so. Sankashta chaturthi and the Karaka chaturthi (Karva Chauth, p. 76) are rules, 12 of 12 against a published 2025 list for Delhi. Planned: the moonrise time and place on the answer, and the other vratas broken at moonrise. |
| SR-61 | planned | `intl`, on A2-3's lexicon | Bikram Sambat and Gregorian dates found in free text in ne, en and hi, with Devanagari or Latin digits and month names in any script or romanised, returned as spans with the parsed date and its calendar. An ambiguous span is returned with every reading, not guessed. |
| SR-62 | partly built | `ParanaRule` on every `EkadashiRule` (Q41) | The Ekadashi parana is built. Planned: the fast's kind (nirjala, phalahar, ekabhukta), its start and its parana window as catalogue fields on every vrata, alongside SR-02's fields. |
