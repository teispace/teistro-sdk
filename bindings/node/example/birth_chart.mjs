// A birth chart: from a Nepali birth record to the nine grahas placed.
//
// This is the scenario the SDK exists for, and it is not one call. What
// it takes, in order:
//
// 1. A birth record as people actually write one — a Bikram Sambat date,
//    a local clock time, and a place.
// 2. That civil time resolved to an **instant**, which needs the zone's
//    history: Nepal was +05:30 until 1986 and +05:45 after, and the
//    resolution says which rule it used and from which tzdb.
// 3. The chart **founded** at that instant and place. The SDK's canonical
//    frame is *tropical*, because that is what an ephemeris computes; a
//    Vedic chart wants the sidereal zodiac, and the profile says which
//    ayanamsha and which centre, so the founder asks for that frame and
//    the SDK completes it — and stamps every step it applied, which this
//    example prints. Positions asked for directly would not know the
//    profile wants the Moon seen from Kathmandu rather than from the
//    Earth's centre, which moves it most of a degree.
// 4. Each longitude read as a rashi, a nakshatra and a pada, using the
//    catalogue's own members and the locale's own names.
//
// What it does not need: an ephemeris of your own, a data file, a
// network, or a second library. `ephemeris: 'builtin'` selects the one
// the SDK carries, so every position below is a real sky and this file
// runs anywhere the package installs.

import {
  Calendar,
  ChartKind,
  Context,
  NakshatraById,
  RashiById,
  at,
  date,
  ianaZone,
  whenUnknown,
} from '../lib/index.js';

// A nakshatra is a twenty-seventh of the circle; a pada a quarter of one.
const NAKSHATRA_DEG = 360 / 27;
const PADA_DEG = NAKSHATRA_DEG / 4;

/** The sign a longitude stands in, and how far into it. */
const rashiOf = (longitude) => [RashiById.get(Math.floor(longitude / 30)), longitude % 30];

/** The lunar mansion a longitude stands in, and which quarter of it, 1 to 4. */
const nakshatraOf = (longitude) => [
  NakshatraById.get(Math.floor(longitude / NAKSHATRA_DEG)),
  Math.floor((longitude % NAKSHATRA_DEG) / PADA_DEG) + 1,
];

const two = (value) => String(value).padStart(2, '0');

const ctx = new Context({
  profile: 'nepali-default',
  locale: 'ne-Deva-NP',
  ephemeris: 'builtin',
});

// ── 1. The record, as it would be written on a form ────────────────────
const birthDay = date(Calendar.BikramSambat, 2042, 9, 17);
const gregorian = ctx.calendar.convert(birthDay, Calendar.Gregorian);
console.log(
  `born  BS ${birthDay.year}-${two(birthDay.month)}-${two(birthDay.day)}` +
    `  (${gregorian.year}-${two(gregorian.month)}-${two(gregorian.day)})` +
    `  00:20  Kathmandu`,
);

// ── 2. The instant, with the zone's own history ────────────────────────
const when = ctx.time.resolve(at(birthDay, { hour: 0, minute: 20 }), ianaZone('Asia/Kathmandu'));
const offset = when.offsetSeconds;
console.log(
  `      JD ${when.instantJdUtc.toFixed(6)} UTC` +
    `   offset ${offset >= 0 ? '+' : '-'}${two(Math.floor(Math.abs(offset) / 3600))}` +
    `:${two(Math.floor((Math.abs(offset) % 3600) / 60))}` +
    `   ${when.source} (tzdb ${when.tzdbVersion})`,
);
// This record sits on the day Nepal moved from +05:30 to +05:45, which is
// why the zone's history matters and a fixed offset would be wrong:
// `iana` above says the answer came from the embedded database rather
// than from a guess.

// ── 3. The chart ───────────────────────────────────────────────────────
const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
const chart = ctx.chart.found({
  instant: when.instantJdUtc,
  place,
  utcOffsetSeconds: when.offsetSeconds,
  kind: ChartKind.Natal,
});

console.log('');
console.log('graha             sign               deg  nakshatra      pada bhava');
console.log('─'.repeat(67));
for (const placed of chart.grahas) {
  const graha = ctx.intl.entity(placed.graha);
  const [rashi, degrees] = rashiOf(placed.longitudeDeg);
  const [nakshatra, pada] = nakshatraOf(placed.longitudeDeg);
  console.log(
    `${graha.name.padEnd(12)} ${(graha.glyph ?? '').padEnd(2)} ` +
      // There is no retrograde flag to trust blindly: a graha is
      // retrograde when its longitude is decreasing, which is what the
      // speed says, and `retrograde` is that comparison, named.
      `${placed.retrograde ? '℞' : ' '} ` +
      `${ctx.intl.entity(rashi).name.padEnd(12)} ` +
      `${degrees.toFixed(4).padStart(8)}°  ` +
      `${ctx.intl.entity(nakshatra).name.padEnd(14)} ${pada}   ` +
      // Which bhava it is in, under the chart's placement system -- the
      // question most of the tradition answers with "in the seventh".
      `${String(placed.house.bhava).padStart(2)}`,
  );
}

// ── 4. What the chart is measured in, and against ──────────────────────
console.log('');
const [lagna, into] = rashiOf(chart.lagnaDeg);
console.log(
  `lagna          ${chart.lagnaDeg.toFixed(4)}° -- ${ctx.intl.entity(lagna).name}` +
    ` at ${into.toFixed(4)}°, vara ${chart.day.vara}`,
);
// `null`, and it means what it says: a tropical chart has no ayanamsha,
// not an ayanamsha of nought.
const ayanamsha =
  chart.ayanamsha ?? (chart.ayanamshaCustom ? 'custom' : null);
console.log(
  ayanamsha === null
    ? 'ayanamsha      tropical, none applied'
    : `ayanamsha      ${chart.ayanamshaOffsetDeg.toFixed(6)}° applied (${ayanamsha})`,
);
console.log(`steps applied  ${chart.steps.join(', ')}`);
// The provenance envelope stamps the settings, the provider and the
// time layer; it is what a stored chart keeps in order to say what
// computed it.
console.log(`settings hash  ${chart.provenance.settings_hash.slice(0, 16)}…`);
ctx.dispose();

// ── A birth with no recorded time ──────────────────────────────────────
// The commonest data problem in the field, and the SDK does **not** pick
// a time for you. `whenUnknown` says the time is unknown; what happens
// next is the profile's `time.unknown_time` policy, and by default there
// is none, so the call is refused with a hint naming the choices.
console.log('');
const noTime = whenUnknown(birthDay);
for (const policy of [undefined, 'NOON', 'MIDNIGHT']) {
  const scoped = new Context({
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    ephemeris: 'builtin',
    settings: policy ? { time: { unknown_time: policy } } : undefined,
  });
  try {
    const resolved = scoped.time.resolve(noTime, ianaZone('Asia/Kathmandu'));
    console.log(
      `${(policy ?? 'refuse').padEnd(9)} JD ${resolved.instantJdUtc.toFixed(6)}` +
        `  time known ${resolved.timeKnown}` +
        `  ${resolved.warnings.join(', ') || '(no warning)'}`,
    );
  } catch (error) {
    console.log(`${(policy ?? 'refuse').padEnd(9)} ${error.message}`);
    console.log(`${' '.repeat(10)}hint: ${error.hint}`);
  }
  scoped.dispose();
}
// MIDNIGHT is refused for a different reason, and it is this record's own:
// the clocks jumped at midnight on this very date, so 00:00 never
// happened in Kathmandu. A chart cast on a guessed midnight would have
// been cast on a time that does not exist.
// NOON answers, and says so twice — `timeKnown` is false and the
// resolution carries a `time-unknown-fallback` warning — so a stored
// chart can never quietly claim a birth time it never had.
