# Observance finder: one observance under each tradition

Status: `built`, 2026-11-01. Design for the consumer addendum's A2-1 (its
second finder), written before the code. The record is
`almanac.observance` (`ts_almanac_observance`, every binding's
`almanac.observance`, the MCP tool `almanac.observance`).

The question: when does this observance fall in this year, under each
tradition the app shows, and where they part, why? The festival packs
answer one tradition at a time (`almanac.festivals`), and the disagreements
record (P1-10, `03-design/setup-disagreements.md`) compares two reckonings
of every day. The observance finder asks one rule of several traditions
and pairs the occurrences.

## 1. What a tradition is

A tradition is **a named rule reckoned once**: a festival request (a pack
or rules spelt out), one rule key of it, and the reckoning it is reckoned
under. `THIS` is the context the call is made on; `OTHER` is a second
context handed beside it. So:

- the Smarta and the Vaishnava Ekadashi are two traditions of one pack,
  two rules, one sky;
- the official and the modern Haribodhini Ekadashi (P1-10) are two traditions of one
  rule, two skies: `{"rule": "HARIBODHINI_EKADASHI"}` reckoned `THIS` on the modern
  context and `OTHER` on the official one;
- a pack against another pack's rule of the same name is two rules.

Between 1 and 8 traditions; each has a unique name the answer calls it by.

## 2. The request

```json
{"year": 2026,
 "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
 "traditions": [
   {"name": "smarta", "festivals": {"rules": "DHARMASINDHU"}, "rule": "EKADASHI_SMARTA"},
   {"name": "vaishnava", "festivals": {"rules": "DHARMASINDHU"}, "rule": "EKADASHI_VAISHNAVA"}]}
```

The range is a `year` of `calendar` (its first day to its last, so a
Bikram Sambat year is Baishakh 1 to Chaitra's end) or `first` and `last`.
Each `festivals` is read by the festival reader and narrowed to `rule`
and what it follows, so the other rules of a pack are not reckoned.

## 3. Pairing

Each tradition's rule gives its occurrences in the range: a karmakala
rule one per tithi it falls on, an Ekadashi rule one per fast. Two
traditions' occurrences are **one occurrence** when their tithi intervals
overlap: the same lunar day, whatever civil day each keeps it on (C1161).
It is the disagreement record's own pairing (`disagreement.rs`'s
`overlap`), extended to N columns: a row is joined by the first occurrence
of a tradition that overlaps an occurrence already in it, and rows come
out in time order. A tradition with nothing in a row keeps no day for that
occurrence in the range.

## 4. The answer

`{traditions, occurrences, parted, unjudged}`:

- `traditions`: each `{name, rule, reckoning, profile, settingsHash,
  provider}`, so the answer says which sky each was reckoned under: the
  settings hash leaves the ephemeris out, and the provider stamp names it.
- `occurrences`: each `{month, adhika, kept, agree, differs, reason}`.
  `kept` is one entry per tradition, in the request's order: its `day`
  and the festival answer's own `observance` (the tithi's extent on the two
  candidate days and what decided) or `fast` (the vedha, the excess and the
  table's choice), or no `day` where it keeps none.
- `differs`: `ABSENT` (a tradition keeps none), `DAY` (different civil
  days), `MONTH` (a different month, or one under an adhika month).
- `reason`, where days or months part (C1162):

| reason | when |
|---|---|
| `PUBLISHED` | a tradition's day was set by a published override |
| `RULE` | one sky, different rules: the texts decide differently |
| `SKY` | one rule, different skies: the reckonings differ (P1-10's reasons apply) |
| `RULE_AND_SKY` | both differ |

- `parted`: how many occurrences do not agree.
- `unjudged`: each tradition's tithis the festival engine could not judge,
  with why.

The answer is sealed by the first tradition's reckoning, with the request
as its input hash.

## 5. Refusals

Named under `observance`: neither `year` nor `first` (`observance.year`),
both (`observance.first`), `year` with `last`; no tradition
(`observance.traditions`), more than 8 (`LIMIT`); a name given twice
(`observance.traditions[i].name`); a festival request its reader refuses
(`observance.traditions[i].festivals.*`); a rule it does not carry
(`INVALID_ARG`) or a dated rule, which has no tithi to pair on
(`UNSUPPORTED`, `observance.traditions[i].rule`); an `OTHER` tradition with
no other context (`observance.traditions[i].reckoning`). At the boundary a
null `other`, or `other` the context itself, is no other context.

## 6. MCP

The tool compares, as `chart.disagreements` does: its `other` argument
names a second context by `profile`, `settings` and `locale`, and a
tradition reckoned `OTHER` is reckoned under it. Without `other` an `OTHER`
tradition is refused by its field, as the boundary refuses a null other.
The server's one ephemeris reckons both, so two traditions part on the sky
only where their settings do.

## 7. Cost

One festival reckoning per tradition, narrowed to the rule; pairing is
quadratic in a year's occurrences per tradition (24 for an Ekadashi rule).
Measured by criterion in `crates/sdk/benches/finders.rs`.

## 8. Cruxes

C1161 (one occurrence is overlapping tithis), C1162 (why traditions part).
