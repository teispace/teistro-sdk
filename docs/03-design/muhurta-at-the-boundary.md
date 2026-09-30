# Muhurta at the boundary: a search for a time in every binding

Status: `built`, 2026-09-30: §8 steps 1 to 4 built. It is step 7 of [`muhurta.md`](muhurta.md)
§6, and after it the search is reachable outside Rust. Built on
[`panchanga-at-the-boundary-measured.md`](panchanga-at-the-boundary-measured.md)
(the almanac's blob), [`rules-at-the-boundary.md`](rules-at-the-boundary.md)
(a request record and a canonical-JSON section) and
[`kp.md`](kp.md) (the last JSON section to reach every binding).

## 1. Purpose and scope

`teistro_muhurta::search` answers a range of days at a place with the
windows an activity's rules leave, judged and ranked, and the days its
season closed. Only Rust can call it, and only by wiring an almanac, a
founder, a completion and a visibility reckoner together by hand, as
`xtask/src/muhurta.rs` does. This page settles four things: how the
façade asks for a search, how a binding asks for one, how the answer
crosses, and how each binding reads it.

Not covered here: a consumer's own named presets on a context, the
eclipse star (`muhurta.md` §4.1), and rendering a window in words. That
last one waits on a vocabulary the corpus does not give (§2.4).

## 2. What the research found

### 2.1 The request is the almanac's, plus one record

A search takes a place, a clock and a calendar range, which is exactly
what `ts_panchanga_request` already holds. A consumer electing a time
usually shows the almanac of the chosen day beside its windows. If
muhurta had an entry point of its own, the consumer would cross twice
and the days would be founded twice. The rules and the SVG renderer met
the same problem on the chart request, and both solved it by riding it:
a nullable record and a section. The panchanga request carries
`struct_size`, and `handshake-is-checked` holds its entry point to it,
so a field appended at the end is safe for a caller compiled against
the older header. **The search rides on `ts_panchanga_days`:** a
nullable `muhurta_json`, and a `muhurta` section in the panchanga blob.
When the section is not asked for, it is empty.

### 2.2 The days would be founded twice

`ProviderSources::day` asks `Almanac::day` for each day it judges, and
`Almanac::between` founds each day of the range the same way. On a
shared request, the open days of the range would therefore be founded
once for the blob and again for the search. **The sources are served
from the days the request founded** (`ProviderSources::with_days`), and
fall back to the almanac for a date outside them. The fallback is there
so the Rust call that asks for no blob still works unchanged. The
measured page counts the provider calls each path makes, so the saving
is reported and not merely claimed. As measured, it is small: the
almanac of a range is a few per cent of what a search over the range
asks (§6).

### 2.3 Two choices the search made silently

- **The asta criterion.** The season reads Guru and Shukra asta off
  `Heliacal` under one visibility criterion. The xtask pass chose the
  Surya Siddhanta's, and nothing asked a caller. The measured page
  shows the choice matters: the combustion orbs start Shukra's asta 4
  days late against the published BS 2083 dates. It becomes a knob in
  the record: `asta`, defaulting to `SURYA_SIDDHANTA`, which also
  accepts `COMBUSTION_ORB`, `PTOLEMY` or any `Criterion`. The answer
  reports which criterion it applied.
- **The zodiac's instant.** `ProviderSources::new` fixes the chart
  zodiac at one reference instant for the whole range. Over the longest
  range, 366 days, the ayanamsha moves about 50.3″. Taking the reference
  at the **middle** of the range halves the worst case to about 25″ of
  longitude, which is under two seconds of lagna. The answer reports
  the instant used (`zodiacAt`). Fixing the drift outright is noted in
  §7, since two seconds is below the six-minute windows the measured
  page counts.

### 2.4 The corpus does not key by the clauses

`muhurta.md` §4.2 proposed a catalogue kind, `muhurta_clause`, on the
grounds that the interpretation corpus's `muhurta-factor` category keys
by clause and a reader must name them. Q38's rule is to measure that
key by key before spending a kind number. I measured the baseline
engine's `STATE_INTERPRETATIONS["muhurta-factor"]`: **47 records**,
which are **25 factors** paired with a polarity (`TITHI_QUALITY_favorable`,
`RAHU_KAAL_unfavorable`).

- **None of the 25 is spelled as a `ClauseKey`.** Thirteen map to one by
  hand:

  | Factor | Clause |
  | --- | --- |
  | `TITHI_QUALITY` | `TITHI` |
  | `NAKSHATRA_SUITABILITY` | `NAKSHATRA` |
  | `WEEKDAY_SUITABILITY` | `VARA` |
  | `RAHU_KAAL` | `KAALA` |
  | `CHOGHADIYA_QUALITY` | `CHOGHADIYA` |
  | `ABHIJIT` | `ABHIJIT` |
  | `TARA_BALA` | `TARABALA` |
  | `CHANDRA_BALA` | `CHANDRABALA` |
  | `YOGA_SHUDDHI` | `YOGA` |
  | `KARANA_SHUDDHI` | `KARANA` |
  | `MUHURTA_YOGA` | `MUHURTA_YOGA` |
  | `KARTHARI` | `KARTARI` |
  | `UDAYASTA_SHUDDHI` | `SEVENTH_OCCUPIED` |

  Two of these are narrower than the clause they map to. `RAHU_KAAL`
  names one member of the three kaalas, and `PANCHAKA` is the nakshatra
  panchaka, not the lagna's remainder.
- **Twelve name something the SDK does not report as a clause:**
  - `HORA_SUITABILITY`
  - `VARA_EVENT`
  - `TITHI_EVENT`
  - `UTTARAYANA`
  - `DISHA_SHULA`
  - `PAKSHA_BALA`
  - `LAGNA_LORD`
  - `LAGNA_PLACEMENT`
  - `EIGHTH_HOUSE`
  - `DOSHA_BHANGA`
  - `KARAKA_STRENGTH`
  - `KARAKA_COMBUST` / `KARAKA_RETROGRADE`

  Most of these are the baseline ranking's own weights (`Factor`), not
  clauses.
- **The favourable half is an absence.** "The window is clear of the
  Rahu Kaal" is not a clause that held. A clause reports that something
  was present.

So the corpus keys by the baseline's factor vocabulary, not by the one
the SDK computes, and a `muhurta_clause` kind would supply a key space
the corpus still could not land in. **No kind is spent now.** At the
boundary a clause crosses by its serde tag, as `ClauseKey` spells it.
`muhurta-factor` stays on the unmigrated list, and its reason becomes
"keys by the baseline's factors" instead of "wants a module". When a
consumer needs a clause *named* in a locale, this measurement is where
that decision starts.

### 2.5 A request takes what an answer gives, one level deep

Every binding reads a catalogue member back as its full key
(`nakshatra.ROHINI`). A consumer who builds an `ActivityRules` from the
binding's own constants, or edits a preset it was handed, therefore
writes full keys. So far the boundary has accepted full keys in two
separate places:

- `hit_request.rs`'s `GrahaKey`;
- `varsha.rs`'s `Askable for DashaSystem`.

Each strips `<kind>.` by hand, and that works only for a member named at
the top of a record. Inside a nested `ActivityRules` a member is read by
the catalogue enum's own `Deserialize`, which refuses the full key.
**The catalogue's generated reader will accept a member's own full key
beside its bare key and former keys**, through one helper,
`Catalogued::from_either_key`. The schema's key list grows by the same
keys, so it is never stricter than the reader. The two hand copies are
then deleted. The prefix must name the **field's own** kind, so
`yoga.SIDDHI` where a `MuhurtaYoga` is expected is still refused. A
generic strip applied before the deserialiser would have accepted it.

### 2.6 A clause's members depend on its variant

The answer's catalogue members sit inside tagged clauses, and a field
name does not settle its kind:

- `yoga` is a `yoga` in `YOGA` and a `muhurta_yoga` in `MUHURTA_YOGA`;
- `sign` is a `rashi` in four clauses;
- `grahas`, `with`, `by`, `second` and `twelfth` are lists of `graha`.

KP's section keeps serde's bare keys, and each binding respells them
from its own hand-written table. That already happened three times over
a much smaller shape. For thirty variants it would mean three tables of
thirty entries each, drifting separately. **The `muhurta` section writes
full keys**, from one Rust table: an exhaustive match on `ClauseKey`
that lists each catalogue-valued field with its kind. Adding a clause
fails to compile until its fields are listed. A test holds the table to
serde both ways. Every string leaf of a clause of each variant must be
one of:

- a listed field's full key;
- a declared local enum (`Grade`, `Tara`, `Dimension`, `BlackoutKind`,
  the tag).

Every listed field must also exist. Node then hands the section through
typed, while Python and Dart resolve each full key with the lookup they
already have.

## 3. The request

**Rust.** The search is an operation of the almanac area, not an area
of its own. `check-areas` defines an area as a boundary module that a
context member reaches, and the search rides on the panchanga module,
as KP and the rules ride on the chart's:

```rust
let asked = MuhurtaRequest::new(ActivityRules::raman_marriage()).with_native(native);
let found = sdk.almanac().muhurta(&from, &to, &place, clock, &asked)?;
// And the days a consumer shows beside the windows, founded once:
let MuhurtaDays { days, day_hashes, answer } =
    sdk.almanac().muhurta_with_days(&from, &to, &place, clock, &asked)?;
```

`muhurta_with_days` is the shared path of §2.2, and the C boundary calls
it. `muhurta` founds only the days the season leaves open. It stamps the
answer with the first day's provenance, then hands that day on to the
search so it is not founded twice.

**What was applied** goes where the envelope already says it:
`provenance.applied_conventions`. It carries `muhurta.asta` (the
criterion's canonical JSON) and `muhurta.zodiacAt` (the instant). The
input hash covers the range, the place, the clock and the request, so
another criterion is another input.

**The record** a binding writes as `muhurta` (`muhurta_json` in C) is
read by `MuhurtaRequest::from_json` with `deny_unknown_fields`. Its keys
are camel-cased, like every request record:

| Key | Type | Default |
| --- | --- | --- |
| `rules` | `"RAMAN_MARRIAGE"`, `"BASELINE_MARRIAGE"`, or an `ActivityRules` object in its own serde spelling (as `rules_json`'s own rules are) | required |
| `native` | `{star, moonSign, lagna?}` | none |
| `ranking` | `"TEXTS"` or `"BASELINE"` | `"TEXTS"` |
| `daysWithWindows` | integer ≥ 1 | 7 |
| `most` | integer ≥ 1 | 50 |
| `asta` | `"SURYA_SIDDHANTA"`, `"COMBUSTION_ORB"`, `"PTOLEMY"`, or a `Criterion` | `"SURYA_SIDDHANTA"` |

The place, the clock and the range come from the panchanga request
itself. They are not repeated in the record, so the two cannot
disagree.

## 4. The answer

The `muhurta` section holds canonical JSON: the answer's envelope,
`{value, provenance}`. `value` is `Answer`'s serde with catalogue members
written in full (§2.6). The envelope is sealed over exactly that value,
so its content hash is the hash of the bytes a binding holds. The Rust
façade's envelope is sealed over bare keys, so the two hashes differ for
one answer; each belongs to the value beside it. Every binding already
decodes a provenance. An instant is a UTC Julian day, as everywhere else
at the boundary. The muhurta crate's fields are camel-cased, as KP's and
Tajika's are, so the request and the answer spell a field one way.

- **Node** hands it through deep-frozen as `panchanga.muhurta`, typed by
  a discriminated union over `clause` in `index.d.ts`. A typecheck
  fixture reads a clause's member back and feeds it into a request's
  rules (§2.5).
- **Python** builds frozen dataclasses: one per clause variant, a
  `Clause` union, catalogue members resolved.
- **Dart** builds sealed final classes over the private value base KP
  introduced, so an answer compares by value.
- **The parity runners** print every window's interval, clauses, bars
  and score for one search, and all five must agree. The ABI test counts
  which clause kinds that search reaches, and lists the rest by name,
  with the list failing both ways, so a variant no search exercised is
  declared rather than assumed.

## 5. Errors and degenerate states

| Case | Error |
| --- | --- |
| Malformed `muhurta_json` | `INVALID_ARG`, naming the key (`muhurta.rules`, `muhurta.native.star`) |
| `BASELINE` ranking over rules with no baseline event | `INVALID_ARG` on `rules.baseline`, as in Rust |
| `daysWithWindows` or `most` of zero | `INVALID_ARG`: a search for nothing is a mistake, not an answer |
| A range over 366 days, or one that runs backward | the almanac's own refusal, which comes before the search |
| No ephemeris | `CAPABILITY` |

- **A range the season closes entirely** is an answer, not an error:
  no windows, every day in `closed`.
- **A polar day** under `UNDEFINED` is refused by the almanac, as it is
  today.

## 6. Cost

The search's cost is cutting its best days into windows: lagna crossings
and a sky read at each window. Sharing the days (§2.2) was designed as
the saving, and the measured page (§6) shows it is the smaller one. The
almanac is a few per cent of the provider calls the search makes. What
remains is the search's own reading of the sky, window by window, and
that is where the next saving lies: reading a day's windows as one grid,
as the port asks a caller to (`ephemeris-port-and-adapters.md` §3). The
page takes about 80 s locally, and CI's fast-check step duration is read
before the PR merges.

## 7. What this design does not settle

- **Whether KP's section should also write full keys**, dropping the
  bindings' three hand tables. It would be the same rule applied
  retroactively, and it is a change to KP's shape.
- **The zodiac drift** (§2.3), bounded at 25″ and reported. The fix is a
  zodiac taken per day, and it waits on a measurement that shows it
  matters.
- **A native read off a chart.** Each binding could offer
  `Native.of(chart)`. It is a convenience, not a capability, and the
  record already takes the three keys.
- **The MCP server** (Q35), which is last.

## 8. Order of work

1. **Catalogue full keys** (§2.5): the generated reader, the schema,
   `from_either_key`, and the two hand copies deleted. Tested both ways,
   including the wrong kind's prefix being refused.
2. **The façade** (§3): `sdk.almanac().muhurta` and `muhurta_with_days`,
   `MuhurtaRequest` and its JSON record, `ProviderSources::with_days`, and
   the applied criterion and instant. The measured page now searches
   through the façade. It holds the façade to the crate wired by hand,
   and counts what the provider is asked for the search and the almanac
   apart and together (`muhurta-measured.md` §6).
3. **The C ABI**: `muhurta_json` on `ts_panchanga_request`, the
   `muhurta` section with the full-key table (§2.6), the ABI test, and
   the keys test.
4. **The bindings**: Node, wasm, Python and Dart; the parity runners;
   the typecheck fixture; the reached-kinds list.
