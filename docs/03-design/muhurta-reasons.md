# Why a muhurta date is good (SR-154)

Status: `designed`, 2026-11-24. Design for addendum 13 §3 (tithipatra hub,
`docs/04-architecture/sdk-briefs/teistro-teimeris-brief-addendum-13.md`).
It is written before the code, from the clauses the search already reports.

The consumer shows each window a muhurta search offers with a short "why
this date" line and a longer list beside it. The line and the list name the
tithi, nakshatra, yoga and vara that qualify the window, and the doshas it
avoids. The muhurta pages and the apps print both.

## 1. What the search already says, and what it does not

A window is a `Judgement`: its interval, every clause that held over it
(`clause.rs`), the rite's bars that struck it, and the baseline's score under
the `BASELINE` ranking. The search cuts a window at every clause's edge
(`window.rs`), so a clause in a window holds over all of it. Every fact the
words need is therefore in the answer already, except one.

The exception is the favourable half of a dosha, which is an absence
(`muhurta-at-the-boundary.md` §2.4). A clause reports that something was
present. "Clear of Rahu kaala" or "Mars is not in the eighth" is not a
clause that held. It is a clause the search **looks for** and did not find.
So "avoided" may name only a dosha the search judges for every window.
Naming one it never computed would claim a check that was not made.

The doshas judged for every window, read off the producers:

| clause | where it is found | judged when |
|---|---|---|
| `KAALA` (Rahu kaala, Yamaghanda, Gulika) | `day.rs`, the day's kaalas | always |
| `ASHUBHA_YOGA` (dagdha, visha, hutashana, yamaghanta) | `day.rs` | always |
| `VARJYAM` | `day.rs`, the day's omens | always |
| `KARTARI` | `instant.rs` | always |
| `MOON_IN_DUSTHANA` | `instant.rs` | always |
| `MOON_JOINED` | `instant.rs` | always |
| `VENUS_IN_SIXTH` | `instant.rs` | always |
| `MARS_IN_EIGHTH` | `instant.rs` | always |
| `KUNAVAMSA` | `instant.rs` | always |
| `PANCHAKA_REMAINDER` | `instant.rs`, from the window's limbs | always |
| `MALEFIC_IN_LAGNA` | `instant.rs` | always |
| `ASHTAMA_LAGNA` | `instant.rs` | with a native only |
| `GHATA` | `day.rs` | with a native only |

These are left out of "avoided", each for a reason:

- **`LAGNA_TYAJYA`.** It is judged only for a sign that rose whole between
  two cuts (`window.rs`, `only_a_sign_risen_whole_between_cuts_is_judged`).
  Its absence is therefore not always a check that was made.
- **`SEVENTH_OCCUPIED`.** It is a marriage's wish. A rite that does not ask
  for an empty 7th has no dosha in an occupied one.
- **`UNWANTED_PLACEMENT`, `PRADOSHA` and `VRISHA_VASTU`.** These are a
  rite's own. They are said as avoided only when the rite's rules name
  them, as bars or as `unwanted` placements.

A rite's **bars** are always judged, because a bar is read off the clauses.
So each bar the window escaped is "avoided" as well, whatever its key. A
test holds the list to the producers. For each listed key it plants the
clause in a sky or a day and finds it reported.

## 2. The shape

One `WindowReasons` per window, in the answer's order:

| field | what it holds |
|---|---|
| `line` | one item, the short "why this date" |
| `reasons` | the longer list, in three runs: `FOR`, then `DESPITE`, then `AVOIDED` |

An item is the reading voice's shape (`rashifal-reading.md`): an `intent`
(a message key under `sdk.voice.muhurta`), its `params` with the `variant`
slot, and what it rests on:

- `side`: `FOR`, `DESPITE` or `AVOIDED`;
- `clause`: for `FOR` and `DESPITE`, the index of the clause in the
  window's `clauses`; nothing for `AVOIDED`;
- `key`: the clause's key (`MARS_IN_EIGHTH`), so an `AVOIDED` item names
  what was looked for.

The list's three runs:

- **`FOR`**: every favourable clause (`Clause::favourable`), graded limbs
  first in the order a panchanga prints them, then the rest in the
  window's order:
  1. nakshatra;
  2. tithi;
  3. vara;
  4. yoga;
  5. karana;
  6. month (lunar or solar);
  7. lagna;
  8. muhurta yoga;
  9. abhijit;
  10. tarabala and chandrabala;
  11. choghadiya;
  12. the neutralisers.
- **`DESPITE`**: every clause against the time that held without barring
  it, such as a rejected karana or a kaala in a rite that does not bar
  kaalas. The texts rank such windows lower, and saying so keeps the list
  honest. Each is a caution.
- **`AVOIDED`**: every dosha of §1 and every bar of the rite that is not in
  the window, in the table's order and then the bars' order.

The **line** says the qualifying limbs. Its intent is
`sdk.voice.muhurta.line`, with these slots:

- `limbs`: the `FOR` limbs among nakshatra, tithi, vara and yoga, as a list
  of entities;
- `count`: how many limbs that is;
- `avoided`: how many doshas were avoided;
- `despite`: how many cautions there are, for a consumer that shows them
  beside the line;
- `variant`: the wording chosen, as in every item.

The pack's `.match` on `count` chooses the wording. With no qualifying limb,
the line says how many doshas the window is clear of.

## 3. Why the composer lives in the muhurta crate

The composer must name every clause kind, and the crate already matches
every kind exhaustively in `spelling.rs`. Putting the composer beside it
means a clause added later does not compile until it has a side and an
intent. The crate gains `teistro-intl` with no default features, for
`Params` and `Value`, as `teistro-interpret` does.

It needs no new façade feature. The `muhurta` family already gates the
crate, and a sub-switch would fail `families-are-forwarded`.

## 4. At the boundary

`MuhurtaRequest` gains `reasons: bool`, `false` by default. When it is
`true`, the SDK composes the reasons after the search and sets
`Answer::reasons`, one per window. The field is skipped when it is empty,
so every answer that does not ask is byte for byte what it was, and its
content hash and the conformance pins stay put.

Each binding reads `reasons` as typed records, as it reads the rashifal's
`ReadingPlan`:

- Python: `WindowReasons` and `ReasonItem`;
- Node: types in `index.d.ts`;
- Dart: classes in `teistro.dart`;
- Java: records.

Each binding's test renders every item in en and ne with no fallback, no
warning and no `{` left in the text.

## 5. The wording

The wording lives in `sdk.voice.muhurta`, in the same pack file as the
rashifal's voice (`i18n/*/sdk.voice.json`):

- `line`;
- `favours.<KEY>`, `despite.<KEY>` and `avoided.<KEY>`, one per clause key
  a side can hold. The first run's segment is `favours`, not `for`, because
  each segment becomes a module of the generated accessors, and `for` is a
  keyword there.

Entity slots carry the members (`nakshatra.ROHINI`), so a locale's entity
names do the naming. The en and ne wordings ship final, with no `draft`
mark, and the maintainer reviews them after shipping (Q42), as the
rashifal's. The hi and sa
wordings come with the voice's S3.

A test holds the pack to the composer. Every intent the composer can say,
listed by `muhurta::reasons::intents()`, renders in en and ne without a
warning.

## 6. Decisions

| crux | question | decision | why |
|---|---|---|---|
| C2400 | what "avoided" may name | only a dosha judged for every window (§1), and the rite's bars | an absence is a claim only where the search looked |
| C2401 | where the composer lives | the muhurta crate, beside `spelling.rs` | an exhaustive match makes a new clause name its side |
| C2402 | the cost to callers who do not ask | none: `reasons` is skipped when empty | hashes and conformance pins stay put |
| C2403 | a clause against the time that did not bar | said, as `DESPITE`, a caution | a list of only good news would mislead |
