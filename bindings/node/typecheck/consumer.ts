// A consumer of the generated types at maximum strictness (ADR-0023).
//
// Every `@ts-expect-error` below is a proof: TypeScript fails the check
// when the line it marks does *not* error, so a surface that stopped
// refusing a wrong usage fails here rather than in someone's application.
//
// Type-checked by `cargo xtask check-node`.

import type { Body, Graha, Status, TimeScale } from '../lib/catalogue.js';
import type { CalendarDate, DeltaT, ZoneResolution } from '../lib/types.js';
import type { IntlRender, Positions } from '../lib/blob.js';

// A catalogue member is its full key, everywhere.
const sun: Graha = 'graha.SUN';
const ketu: Graha = 'graha.KETU';
// A member from a newer library falls into the one arm every catalogue
// union carries, so a `switch` over it can be exhaustive.
const newer: Graha = 'unknown';

// @ts-expect-error a key that is not a graha
const notAGraha: Graha = 'graha.PLOOTO';
// @ts-expect-error the key of another kind
const wrongKind: Graha = 'rashi.ARIES';
// @ts-expect-error a bare member name is not a key
const bareName: Graha = 'SUN';

// Any other enum is its name in kebab case.
const ok: Status = 'ok';
const refused: Status = 'invalid-arg';
const ut1: TimeScale = 'ut1';
const node: Body = 'mean-node';
// @ts-expect-error not a status
const notAStatus: Status = 'invalid_arg';

/** Exhaustive over a closed union: adding a member breaks this on purpose. */
function scaleName(scale: TimeScale): string {
  switch (scale) {
    case 'ut1':
      return 'Universal Time';
    case 'tt':
      return 'Terrestrial Time';
    default: {
      const unreachable: never = scale;
      return unreachable;
    }
  }
}

/** A boundary struct is readonly: a result is a fact, not a variable. */
function describe(delta: DeltaT, zone: ZoneResolution, date: CalendarDate): string {
  // @ts-expect-error a result's fields cannot be written
  delta.seconds = 0;
  return `${delta.seconds} s by ${delta.model ?? 'the default model'}, ${
    zone.offsetSeconds
  } s in ${zone.tzdbVersion}, ${date.year}-${date.month}-${date.day}`;
}

/** A nullable field is `| null`, so it cannot be used without a check. */
function abbreviation(zone: ZoneResolution): string {
  // @ts-expect-error the abbreviation may be null
  const bad: string = zone.abbreviation;
  return zone.abbreviation ?? '';
}

/** A decoded blob's columns are typed arrays, indexed with a check. */
function firstLongitude(positions: Positions): number {
  const cells = positions.cells;
  // `noUncheckedIndexedAccess` makes an index `number | undefined`.
  const first = cells.lon[0];
  return first ?? Number.NaN;
}

/** A rendered message's parts are what the schema says they are. */
function rendered(message: IntlRender): string {
  // @ts-expect-error the text is a string, not bytes
  const bytes: Uint8Array = message.text;
  return `${message.text} (${message.resolvedFrom}, ${message.warningCount} warnings)`;
}

export { abbreviation, bareName, describe, firstLongitude, ketu, newer, node, notAGraha, notAStatus, ok, refused, rendered, scaleName, sun, ut1, wrongKind };
