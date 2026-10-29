// What an application on the baseline engine writes with the shim,
// type-checked at the strictness of the SDK's own consumer. It is not run:
// what is held is that the declarations compose with the SDK's and refuse
// what the shim refuses. Every function the package exports is used once
// (`docs/03-design/compat-shim.md` §6.4).
//
// `cargo xtask check-node` compiles this.

import { compat, CompatRefused, FAMILIES, facts } from '@teistro/compat';
import type { BirthRecord, CompatOptions, Family, Shim, StoredSnapshot } from '@teistro/compat';
import { Body, Calendar, ChartLayout, TeistroError, Varga, at, date, ianaZone } from '@teistro/sdk';
import type {
  Almanac,
  AlmanacDay,
  CalendarDate,
  Chart,
  ChartTiming,
  Charts,
  NaamMilan,
  NameChecked,
  NumerologyProfile,
  PakshiDays,
  Positions,
  RashifalSealed,
  Rendered,
  ResearchCounts,
  ResearchTested,
  ZoneResolution,
} from '@teistro/sdk';

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
const rows: Shim = compat({ ephemeris: 'BUILTIN' });
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
const may = date(Calendar.Gregorian, 2026, 5, 1);
const june = date(Calendar.Gregorian, 2026, 6, 1);
const more: Chart[] = [
  rows.houses(birth, snapshot),
  rows.states(birth, snapshot),
  rows.aspects(birth, snapshot),
  rows.points(birth, snapshot),
  rows.jaimini(birth, snapshot),
  rows.longevity(birth, snapshot),
  rows.timing(birth, snapshot),
  rows.avakahada(birth, snapshot),
  rows.dossier(birth, snapshot, { preset: 'FULL' }),
  rows.drawing(birth, snapshot, { drawings: [{ layout: ChartLayout.SouthIndian, varga: Varga.D9 }] }),
  rows.sarvatobhadra(birth, snapshot, { instants: [2461000.5] }),
  rows.transitCalendar(birth, snapshot, { hits: { from: 2460676.5, to: 2461041.5 } }),
  rows.returns(birth, snapshot, { from: 2461000.5, to: 2461365.5, grahas: ['SUN'] }),
  rows.progressions(birth, snapshot, { at: 2458953.5 }),
  rows.rectify(birth, snapshot, { uncertaintyMinutes: 30 }),
  rows.prashna(birth, snapshot),
  rows.lalkitab(birth, snapshot),
  rows.remedies(birth, snapshot, { at: 2461000.5 }),
  rows.synastry(birth, snapshot, { partner: birth }),
  rows.narrative(birth, snapshot, { interpret: { placements: true } }).chart,
  rows.milanVerdict(birth, snapshot, { partner: birth }).chart,
];
const said: Readonly<Record<string, readonly string[]>> = rows.narrative(birth, snapshot, { interpret: { readings: true } }).text;
const grid: Positions = rows.positions(snapshot, { instants: [2451545], bodies: [Body.Sun] });
const own: unknown = rows.engine(snapshot, 'julday', { year: 2000 });
const offset: number = rows.ayanamsha(birth, snapshot, { instant: 2451545 }).offsetDeg;
const sunrise: number = rows.sunrise(birth, snapshot, { date: may }).sunrise;
const zone: ZoneResolution = rows.resolve(snapshot, at(may, { hour: 12 }), ianaZone('Asia/Kathmandu'));
const ghati: ChartTiming = rows.ghatiPala(birth, snapshot).PROPORTIONAL;
const bs: CalendarDate = rows.bs(snapshot).convert(may, Calendar.BikramSambat);
const eras = rows.eras(birth, snapshot).years;
const sun = rows.entity(snapshot, 'graha.SUN');
const born: CalendarDate = rows.birthCalendar(birth, snapshot);
const changes = rows.changes(birth, snapshot).tithi.map((change) => change.to);
const muhurta: Almanac = rows.muhurta(birth, snapshot, { from: may, to: june, search: { rules: 'BASELINE_MARRIAGE', native: null } });
const eclipses: Almanac = rows.eclipses(birth, snapshot, { from: may, to: june });
const ingresses: number[] = rows.ingresses(birth, snapshot, { from: may, to: june }).map((ingress) => ingress.instant);
const lunations: number[] = rows.lunations(birth, snapshot, { from: may, to: june }).map((lunation) => lunation.instant);
const pakshi: PakshiDays = rows.pakshi(birth, snapshot, {
  from: may,
  to: june,
  native: { nakshatra: 'nakshatra.ASHWINI', paksha: 'paksha.SHUKLA' },
});
const naam: NaamMilan = rows.naam(birth, snapshot, { partner: birth });
const numbers: NumerologyProfile = rows.numerology(birth, snapshot);
const checked: readonly NameChecked[] = rows.nameCheck(birth, snapshot, { names: ['राम'] });
const counted: ResearchCounts = rows.research([birth, birth], snapshot, { rules: { shipped: ['YOGAS'] }, design: { groups: [0, 1] } });
const tested: ResearchTested = rows.research([birth, birth], snapshot, {
  rules: { shipped: ['YOGAS'] },
  design: { groups: [0, 1] },
  test: { seed: 7, permutations: 99, contrast: { kind: 'ANY_DIFFERENCE' } },
});
const rashifal: RashifalSealed = rows.rashifal(birth, snapshot, { first: may, last: june });
const rendered: Rendered = rows.reading(snapshot, 'sdk.reason.grahaInBhava', { bhava: 7 });

// A refused row is typed `never`: the code after it is unreachable, so a
// compiler marks it at the call site.
function located(): string {
  rows.zoneAt(27.7, 85.3);
}
function formatted(): string {
  return rows.format(2447995.5);
}
for (const refused of [rows.sidecar, rows.gemstoneSafety, rows.nameSuggestions]) {
  const nothing: never = refused();
  void nothing;
}

// The A/B's vocabulary: an answer's facts, and how each family compares.
const read: [string, unknown][] = facts('chart', charts[0], { window: { from: 0, to: 1 } });
const family: Family | undefined = FAMILIES['longitude'];
// @ts-expect-error facts reads the answers the A/B compares, not every row's
facts('naam', naam);

// @ts-expect-error a transit asks for its instants
rows.gochar(birth, snapshot);
// @ts-expect-error a muhurta search names its rules
rows.muhurta(birth, snapshot, { from: may, to: june });
// @ts-expect-error a study is of stored birth records
rows.research([2447995.5], snapshot, { rules: {}, design: { groups: [0] } });
// @ts-expect-error a partner is a stored birth record, not an instant
rows.milan(birth, snapshot, { partner: 2447995.5 });
rows.dispose();

void [left, snapshot, text, explain, wrong, missing, charts, today, newYear, month, hours, more, said, grid, own, offset];
void [sunrise, zone, ghati, bs, eras, sun, born, changes, muhurta, eclipses, ingresses, lunations, pakshi, naam, numbers];
void [checked, counted, tested, rashifal, rendered, located, formatted, read, family];
