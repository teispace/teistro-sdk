// What an application on the baseline engine writes with the shim,
// type-checked at the strictness of the SDK's own consumer. It is not run:
// what is held is that the declarations compose with the SDK's and refuse
// what the shim refuses. Every function the package exports is used once
// (`docs/03-design/compat-shim.md` §6.4).
//
// `cargo xtask check-node` compiles this.

import { compat, CompatRefused } from '@teistro/compat';
import type { BirthRecord, CompatOptions, Shim, StoredSnapshot } from '@teistro/compat';
import { Calendar, TeistroError, date } from '@teistro/sdk';
import type { Almanac, AlmanacDay, Chart, Charts } from '@teistro/sdk';

const shim = compat({ ephemeris: 'BUILTIN', locale: 'ne-Deva-NP' });

// A day's log of which calls are left to port: zero everywhere is done.
const left: string[] = Object.entries(shim.calls())
  .filter(([, times]) => times > 0)
  .map(([row]) => row);

shim.dispose();

// The stored birth record, as the application keeps it.
const birth: BirthRecord = {
  name: 'Rama',
  dateTimeUtc: '1990-04-13T23:45:00.000Z',
  julianDay: 2447995.4895833335,
  latitude: 27.7172,
  longitude: 85.324,
  altitude: 1400,
  timezone: 'Asia/Kathmandu',
  isLmt: false,
  gender: 'female',
  birthTimeUncertainty: 'exact',
  birthOrder: null,
  bsDate: { year: 2047, month: 1, day: 1 },
};

// The stored snapshot, as its fields or its JSON.
const snapshot: StoredSnapshot = { ayanamsaId: 1, houseSystem: 'whole-sign' };
const text: StoredSnapshot = '{"ayanamsaId": 1}';

/** A refusal is told from the SDK's own error by its class. */
function explain(error: unknown): string {
  if (error instanceof CompatRefused) return `${error.row}: ${error.message} (${error.guide})`;
  if (error instanceof TeistroError) return `${error.status}: ${error.message}`;
  return String(error);
}

// @ts-expect-error each stored snapshot decides the profile
compat({ profile: 'parashari-classical' });
// @ts-expect-error and the settings
compat({ settings: {} });

const options: CompatOptions = { testProvider: true };
const typed: Shim = compat(options);
// @ts-expect-error a count is a fact, not a variable
typed.calls()['chart'] = 1;
typed.dispose();

const wrong: BirthRecord = {
  ...birth,
  // @ts-expect-error a stored gender is one of the three the application keeps
  gender: 'MALE',
};

// @ts-expect-error the Julian day is required: the date is checked against it
const missing: BirthRecord = { ...birth, julianDay: undefined };

// Every row, each answering the SDK's own type.
const rows = compat({ ephemeris: 'BUILTIN' });
const charts: Chart[] = [
  rows.chart(birth, snapshot),
  rows.full(birth, text),
  rows.vargas(birth, snapshot),
  rows.yogas(birth, snapshot),
  rows.strength(birth, snapshot),
  rows.dashas(birth, snapshot),
  rows.milan(birth, snapshot, { partner: birth }),
  rows.varsha(birth, snapshot, { through: 30 }),
  rows.kp(birth, snapshot),
  rows.kp(birth, snapshot, { anyAyanamsha: true }),
  rows.gochar(birth, snapshot, { instants: [2458953.5] }),
  rows.sadeSati(birth, snapshot, { from: 2447995.5, to: 2458953.5 }),
];
const today: AlmanacDay = rows.day(birth, snapshot);
const newYear: AlmanacDay = rows.day(birth, snapshot, { date: date(Calendar.Gregorian, 2026, 4, 14) });
const month: Almanac = rows.month(birth, snapshot, {
  from: date(Calendar.Gregorian, 2026, 4, 14),
  to: date(Calendar.Gregorian, 2026, 5, 13),
});
const hours: Charts = rows.batch(birth, snapshot, { instants: Float64Array.of(2447995.5, 2447995.5417) });
// @ts-expect-error a transit asks for its instants
rows.gochar(birth, snapshot);
// @ts-expect-error a partner is a stored birth record, not an instant
rows.milan(birth, snapshot, { partner: 2447995.5 });
rows.dispose();

void [left, snapshot, text, explain, wrong, missing, charts, today, newYear, month, hours];
