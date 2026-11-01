# Where two charts disagree

Status: `built`, 2026-11-01. Design for the consumer's SR-41
(`docs/consumers/addendum-3.md`), written before the code and corrected
by it.

The almanac answers where two reckonings of one day disagree
(`setup-disagreements.md`). The application asks the same of a chart: one
birth read under two settings (the classical profile and KP, Lahiri and
Raman), or two births read under one (a twin, a corrected birth time),
and which of the facts a reader looks at first differ: the lagna, the
Moon's nakshatra, the running dasha lords, the yogas.

## 1. Shape

The comparison is the almanac's, kept on purpose so a reader of one knows
the other:

- **two contexts**: the call's own and `other`. Pass the same context
  twice to compare two births under one setting;
- the answer carries **`otherProfile` and `otherSettingsHash`** beside the
  facts, so a stored comparison is known stale when either side's
  settings change; it is sealed by the first chart's provenance, whose
  `inputHash` is the hash of the whole request as sent;
- it holds **only the facts that differ**, each with both sides, and
  `compared`, how many facts were read, so an empty answer says "agrees on
  all `compared`" rather than "compared nothing".

The request is `{first, second, at}`: `first` a chart request of one
birth, the record `chart.found` reads (`instant`, the place, the clock,
any `dashas` and `rules`); `second` another, the first's again when left
out; `at` a UTC Julian day. A batch (`instants`) is refused at
`disagreements.first.instants`: a comparison is one chart a side.

## 2. The facts

| fact | compared | differs when |
|---|---|---|
| `lagna` | always | its sign differs |
| `grahas` | each graha both charts place | its sign differs |
| `nakshatra` | when both place the Moon | the Moon's nakshatra or pada differs |
| `dashas` | each level of each system both carry | the period running at `at` names another lord (or sign, for a sign dasha) |
| `yogas` | each rule either reading holds | one holds it and the other does not, or with another net status |

Each placement carries the longitude and the degrees into the sign beside
the sign, because a sign that differs is nearly always one side at 29.9°
and the other at 0.2°, and the reader wants to see that (C1100).

A side whose request names no dasha system is read under Vimshottari, so
the dasha lords are always compared; a side naming its own systems is
compared on the systems both name, matched by key. With no `at`, each
chart's lords are read at its own birth: the dasha a native is born
into, which is what a reader comparing two birth times wants (C1101).

The yogas are compared when either side's request names a `rules`
record. A rule is a key in a rule set, so two sides with different rule
sets disagree on every rule one alone evaluates: that is reported, not
hidden, since it is what the reader asked (C1102).

## 3. At the boundary

`ts_chart_disagreements(context, other, request_json, out_json)` answers
`{value, provenance}`, every catalogue member a full key; a null `other`
is `INVALID_ARG` naming `other`, as the almanac's is. The MCP tool
`chart.disagreements` takes an `other` argument, `{profile, settings,
locale}`, and builds the second context as it builds the first; left
out, both charts are read under the call's context. Every binding has
`chart.disagreements(other, request)`, typed as the almanac's is.
