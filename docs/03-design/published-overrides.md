# Published overrides: a day an authority decides

Status: `built`, 2026-10-26. The brief's P2-4. Written before any code
and corrected by the building: §4's changes are the caller's, not the
set's, and §5's call is `PublishedSet::from_signed`. The committee gate
(§6) is measured in `festival-measured.md` §7.

Some days are not a rule's to decide: a national panchanga's choice, a
gazette holiday, an Eid by sighting, a correction after printing. An app
must take them as data, update them without a release, and say of every
day whether it was computed or published. This page decides what the SDK
does with them and what it leaves to the app.

## 1. What is the SDK's

A pack's life has two halves.

**Transport** covers:
- fetching;
- metadata;
- expiry;
- rollback;
- key rotation;
- storage.

A consumer that ships packs already has these, and they belong to its
threat model: TUF-style metadata, an offline root, an online signing role.

**Meaning** covers what a published entry does to an answer, how it is
marked, and why it differs from the rule. Only the meaning is astrology,
and only the SDK can know it.

So the SDK takes the meaning whole, and transport only in its smallest
verifiable form:
- **A published set** is a typed value, read from JSON or built in code.
  It rides on the `FestivalRequest`: `with_published(set)` in Rust, and a
  `published` key in the request's JSON. Every path that already takes a
  request then takes the sets without a new parameter: `festivals`,
  `festivals_with_days`, the almanac's `asked`, the iCalendar feed,
  `festivals_json` on the panchanga request, the C boundary and every
  binding.
- **A signed pack** is the set's exact bytes, a detached Ed25519 signature
  and the caller's public key, behind the `signed-packs` feature (§5).
  The SDK embeds no key. A consumer with its own transport passes the set
  it verified and never needs the feature.

## 2. The set

```json
{
  "format": "teistro.published/1",
  "publisher": "a patro printed in Nepal",
  "id": "np-panchanga-2084",
  "version": 3,
  "issued": {"calendar": "GREGORIAN", "year": 2026, "month": 10, "day": 20},
  "entries": [
    {"action": "SET", "rule": "GHATASTHAPANA", "day": {…}, "source": "p. 41"},
    {"action": "ADD", "key": "EID_UL_FITR", "day": {…}, "names": {"en-Latn": "Eid al-Fitr"}, "source": "notice 2084/112"},
    {"action": "WITHDRAW", "rule": "HOLI_HILLS", "day": {…}, "source": "correction 2"}
  ]
}
```

**Actions.**
- **`SET`**: the published day of a rule's occurrence. It is placed on the
  rule's occurrence nearest its day, within seven days. Several
  occurrences that near are told apart by an optional `month` (the amanta
  `Masa`) and `adhika`; without them the entry is refused as ambiguous.
  The occurrence's `day` becomes the published day, and its own day is
  kept as `published.computed`.
- **`ADD`**: a day no rule computes, keyed by its own key, with names per
  locale. A missing locale falls back as every name does; the key is never
  machine-translated.
- **`WITHDRAW`**: the occurrence is kept but marked withdrawn, never
  dropped, so a stored answer still finds it.

**Times.** An entry may carry a published `window` (UTC instants): a
parana, a sait. A `SET` that moves an Ekadashi fast without one leaves
`parana` empty. The SDK does not vouch for a window computed for another
day.

**Checks.** Each refusal is `INVALID_ARG`, naming the entry under
`published`:
- the set is read strictly: an unknown field is refused, and so is a
  second entry for one occurrence;
- the format is named, and its version is one the SDK reads;
- a `SET` or `WITHDRAW` names a rule the request carries;
- an `ADD` names a key no rule of the request uses;
- the day's calendar is one the SDK ships.

**Layering.** Sets apply in the order given, and a later set's entry for
the same occurrence replaces an earlier one's, as a later rule replaces a
pack's (`festival-rules.md` §4.3). The intended use is a state's set
after the nation's.

**Outside the range.** An entry whose day lies outside the asked range
is ignored, since a set is a year and a request may be a month. An entry
inside the range that no occurrence takes is reported in `unplaced`: a
publisher's day the rules cannot find is a finding, not a silence.

## 3. The answer

**The observances.** Each observance, Ekadashi fast and dated observance
gains `published`, which is absent when no entry touched it. Absent means
computed, so an answer asked without sets is byte-identical to today's,
hash included. When present it holds:
- `status`: `PUBLISHED` or `WITHDRAWN`;
- `publisher`, `set`, `version` and `source`;
- `computed`: the rule's own day, when it differs;
- `window`: when published.

**The answer itself** (`Observances`) gains three lists, each left out
when empty:
- `added`: the `ADD` days, each with its key, names, day, window and
  provenance;
- `unplaced`: the `SET` and `WITHDRAW` entries that matched no occurrence
  in range;
- `applied`: each set's `id`, `version`, `publisher`, and the SHA-256 of
  its canonical JSON. A stored answer then says which data made it.

**The iCalendar feed** writes "Published by" the publisher, set, version
and source in a published day's `DESCRIPTION`, with the rule's own day
when it differs and the provenance after "Answered by"; a computed day
keeps "Computed by". A withdrawn day writes no event, its parana with it,
so a subscriber's calendar drops it. An added day is an event, named from
the set's names in the context's locale (its language's, then the first
given, then its key).

## 4. Why a published day differs

`AlmanacArea::published_reasons(range, place, request, places, skies)`
asks the festival request again, with the same sets, under one change at
a time, in this order:

1. **each of `places`**: where the publisher may have reckoned (a
   `ReckonedPlace`: a name, a place, an offset);
2. **each of `skies`**: another context, such as the committee's Surya
   Siddhanta beside a modern sky, named by its profile and settings hash;
3. **each shipped pack**: asked once, its rules in place of the request's
   for every moved key on which it differs.

The changes are the caller's, not the set's. A set naming its own place
or profile was designed and dropped: a publisher states neither, and a
caller trying a guess learns as much as a set claiming one.

**Reporting.** A change reproduces when the occurrence, found by rule,
set and published day, gives that day with no `computed`. The first that
reproduces is the `reason`. Every change tried is listed with the day it
gave (absent when it gives none within seven days), so a "none" is shown,
not asserted. When none reproduces, the reason is `PUBLISHER`, the
publisher's own choice.

**At the boundary.** `ts_almanac_published_reasons(context, other,
request_json, out_json)`. `other` is the one sky; null, or the context
itself, tries none. The bindings take `places` and an optional `sky`.

## 5. Signed packs (`signed-packs`)

**The call.** `PublishedSet::from_signed(pack, signature, key, at_least)`
returns the set; at the boundary it is `ts_published_verify`, and in the
bindings `publishedFromSigned` (`published_from_signed` in Python). A
build without the feature answers `CAPABILITY`.
- The signature covers the exact bytes, not a re-serialisation.
- The bytes are UTF-8 JSON, read strictly.
- Verification is `verify_strict`, which refuses small-order and
  non-canonical points.

**Refusals.**
- A bad signature is `INVALID_ARG` on `signature`, saying the pack was
  altered or signed by another key.
- A version below `at_least` is refused, so a replayed pack cannot roll a
  correction back.

**Expiry and rotation** stay the transport's (§1).

**The crate** is `ed25519-dalek` 3 (BSD-3-Clause, MSRV 1.85, pure Rust,
builds for wasm32), with its default features off. The precomputed tables
speed up signing and key generation, not one verification. Its size in
each binding is measured before it ships (`SIZES`).

## 6. The gate

- **The committee's printed days** (`festival-measured.md` §7). The
  committee's printed VS 2082 and 2083 days, 589 rows, are written as a set
  over the `NEPAL_PANCHANGA` pack on the modern sky, a Vikram year a range.
  Every printed day an entry is placed on must be the answer's `day`: 578
  of 578 are. An entry whose rule gives no day within `NEAREST_DAYS` of
  the print under that sky is `unplaced`, not laid: 11, a weekday the sky
  does not meet or a month the committee reckons otherwise. An app that
  must show those days publishes them as `ADD`. The reasons are measured
  with the committee's sky tried: it gives 48 of the 49 moved days, and
  the 49th (the Nuwakot mela, a day after the rule's full moon) is the
  publisher's own.
- **Refusals.** A tampered byte, a wrong key, a replayed older version, an
  ambiguous `SET`, a `SET` naming no carried rule and an `ADD` reusing a
  key are each refused by name.
- **No sets.** With no sets, every shipped answer is byte-identical, hash
  included.
- **Every binding** reads one set with each action back, through its
  parity runner.
