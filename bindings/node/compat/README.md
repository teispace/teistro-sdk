# @teistro/compat

A migration aid for a Node application built on the baseline engine. It
lets the application keep its stored rows (a birth record per person and a
settings snapshot per chart) while it moves its calls to the Teistro SDK
one call at a time. A shim call takes the stored record and snapshot as
they are, opens the SDK under the settings the snapshot means, and answers
the SDK's own result. The design is
[`docs/03-design/compat-shim.md`](../../../docs/03-design/compat-shim.md);
the migration guide is
[`docs/09-guidelines/baseline-migration.md`](../../../docs/09-guidelines/baseline-migration.md),
and every row with its SDK call is in the generated map,
[`docs/09-guidelines/baseline-map.md`](../../../docs/09-guidelines/baseline-map.md).

**Status:** the whole call surface of the design page's §3, 62 rows
listed in [`surface.json`](surface.json): 46 forward, each one SDK call
with its inputs mapped; 11 compose, a few SDK calls or one call and a
read across its answer; and 5 refuse, typed `never`. Fifteen of them
(`chart`, `full`, `vargas`, `yogas`, `strength`, `dashas`, `day`,
`month`, `milan`, `varsha`, `kp`, `gochar`, `sadeSati`, `batch` and
`muhurta`) are the calls the A/B against the baseline engine times, so
each is held to "not slower than the baseline". Every answering row is
held to the bit against the SDK call it stands for, over the baseline
corpus. Nothing is published: the package is `private` until releases
are cut, like every Teistro package.

## Using it

The package's only dependency is `@teistro/sdk`, as a peer at exactly the
same version. Open one shim per process or worker thread, over the
ephemeris chain and the locale every context it opens shares:

```ts title="bindings/node/compat/typecheck/consumer.ts"
import { compat, CompatRefused, FAMILIES, facts } from '@teistro/compat';
// …
const shim = compat({ ephemeris: 'BUILTIN', locale: 'ne-Deva-NP' });

const left: string[] = Object.entries(shim.calls())
  .filter(([, times]) => times > 0)
  .map(([row]) => row);

shim.dispose();
```

Each row takes the stored birth record and the stored snapshot the chart
was computed under, and answers the SDK's own result. A row that needs
more takes it as an option: the partner's stored record for `milan`, the
instants for `gochar` and `batch`, the range for `month` and `sadeSati`:

```ts title="bindings/node/compat/typecheck/consumer.ts"
const rows: Shim = compat({ ephemeris: 'BUILTIN' });
const charts: Chart[] = [
  rows.chart(birth, snapshot),
// …
  rows.milan(birth, snapshot, { partner: birth }),
// …
];
const today: AlmanacDay = rows.day(birth, snapshot);
```

A few rows read no birth and take what their SDK call takes, on the
snapshot's context: `positions`, `engine`, `resolve`, `bs` (the context's
calendars), `entity` and `reading`; `research` takes a list of stored
records. A composed row answers a small record of the SDK's values, such
as `sunrise`'s sunrise, sunset and next sunrise, or `narrative`'s chart
beside its plans said in the snapshot's locale:

```ts title="bindings/node/compat/typecheck/consumer.ts"
const said: Readonly<Record<string, readonly string[]>> = rows.narrative(birth, snapshot, { interpret: { readings: true } }).text;
const grid: Positions = rows.positions(snapshot, { instants: [2451545], bodies: [Body.Sun] });
// …
const sunrise: number = rows.sunrise(birth, snapshot, { date: may }).sunrise;
```

A row the SDK cannot answer (`zoneAt`, `format`, `sidecar`,
`gemstoneSafety`, `nameSuggestions`) throws `CompatRefused` at once,
naming what to do instead. It is typed `never`, so the compiler marks the
code after the call unreachable and the call is found where it is made:

```ts title="bindings/node/compat/typecheck/consumer.ts"
function located(): string {
  rows.zoneAt(27.7, 85.3);
}
```

`kp` answers on the KP context the snapshot means, Krishnamurti's
ayanamsha on the chart's frame with the baseline's ruling planets;
`{ anyAyanamsha: true }` answers on the chart's own context instead.
`muhurta` searches on the snapshot's muhurta context
(`recordedSettings`'s `muhurta`), the frame the baseline searches in,
from the Earth's centre with the true node, and `ghatiPala` reads its
second reckoning on a context with the other one; each is opened beside
the snapshot's own on first use.

## Comparing the two engines

While both engines run side by side, `facts(row, answer)` reads an
answer into the A/B's vocabulary: flat `[family/what, value]` pairs for
the fields both engines answer. `FAMILIES` says how each family compares,
by a difference within a stated tolerance or by equality:

```ts title="bindings/node/compat/typecheck/consumer.ts"
const read: [string, unknown][] = facts('chart', charts[0], { window: { from: 0, to: 1 } });
const family: Family | undefined = FAMILIES['longitude'];
```

In an application the chain is a real engine, such as
`[teimeris({ dataDir: './ephe' })]` from `@teistro/ephemeris-teimeris`.
`profile` and `settings` are not options: each stored snapshot decides
them. The locale given here is used for a snapshot that stores none.

The shim keeps a natal context for each distinct translated snapshot,
and a KP context opened on first use beside it. It keeps at most sixteen
snapshots' contexts and disposes the one used longest ago to make room.
`shim.dispose()` frees them all. The cache belongs to the shim, so each
worker thread opens its own, as the SDK requires.

## The stored rows

A snapshot is translated by the SDK's `recordedSettings`, which every
binding has, and the table it follows is the design page's §4.2.

A birth record is read as follows:

| the record holds | the shim |
|---|---|
| `dateTimeUtc` and `julianDay` | founds at the date, the instant the application computed from, and refuses the record when the Julian day derived from it stands more than 43.25 ms away: half the step of a Julian day kept to six decimals, and a double's rounding |
| `latitude`, `longitude`, `altitude` | the place |
| `timezone`, or `isLmt` | the clock's offset, read from the IANA zone, or local mean time at the longitude |
| `birthTimeUncertainty`, `gender` | read only by rectification, as its `accuracy` and `sex` (`other` as none) |
| `name` | read only by numerology and naam, which refuse a record without one |
| `birthOrder`, `bsDate` | ignored: they are the application's data, and the instant already fixes the date |

## Errors

- An SDK error passes through as the `TeistroError` it is, never wrapped.
- A refused entry point, stored field or stored value throws
  `CompatRefused` before any computation. Its `row` names the row
  (`snapshot.suryaBija`, `birth.instant`), its `guide` is the anchor in
  the migration guide that explains it, and its message says what to do
  instead. When the SDK's translation refused a stored value, its
  `TeistroError` is the `cause`, with its crux as the hint.
- A record of the wrong shape (a missing Julian day, a latitude that is
  not a number) is a `TypeError` naming the field.

## When it ends

The shim exists so that an application on the baseline engine can delete
the baseline's packages. Its end is set here and in the changelog from the
first release:

- it is marked `@deprecated` in every export's documentation once the last
  baseline application reports no shim calls left;
- it stops being released at the first SDK minor version after that, or at
  the SDK's 2.0, whichever comes first;
- `recordedSettings` stays in the SDK. Stored snapshots outlive the
  migration, and reading one is a permanent need.

"No shim calls left" is observable without telemetry: `shim.calls()`
returns, for each row, how many times this thread called it. Log it once
a day. When every count is zero, the application has finished porting.
