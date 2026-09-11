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
// 3. Positions in a **sidereal** frame. This is the step everyone gets
//    wrong. The SDK's canonical frame is *tropical*, because that is
//    what an ephemeris computes; a Vedic chart wants the sidereal
//    zodiac, so the request names one and the SDK completes it — and
//    stamps every step it applied, which this example prints.
// 4. Each longitude read as a rashi, a nakshatra and a pada, using the
//    catalogue's own members and the locale's own names.
//
// What it does not need: an ephemeris of your own, a data file, a
// network, or a second library. `ephemeris: 'builtin'` selects the one
// the SDK carries, so every position below is a real sky and this file
// runs anywhere the package installs.

import {
  Ayanamsha,
  Body,
  Calendar,
  Context,
  Graha,
  NakshatraById,
  RashiById,
  at,
  canonicalFrame,
  date,
  ianaZone,
  whenUnknown,
} from '../lib/index.js';

// The grahas of a Vedic chart, each paired with the body an ephemeris
// answers for it. Ketu is not a body: it is Rahu's opposite point, so it
// is computed rather than asked for, as the SDK's own `points` module
// does.
const GRAHAS = [
  [Graha.Sun, Body.Sun],
  [Graha.Moon, Body.Moon],
  [Graha.Mars, Body.Mars],
  [Graha.Mercury, Body.Mercury],
  [Graha.Jupiter, Body.Jupiter],
  [Graha.Venus, Body.Venus],
  [Graha.Saturn, Body.Saturn],
  [Graha.Rahu, Body.MeanNode],
];

// A nakshatra is a twenty-seventh of the circle; a pada a quarter of one.
const NAKSHATRA_DEG = 360 / 27;
const PADA_DEG = NAKSHATRA_DEG / 4;

/** One graha as a chart shows it. */
class Placement {
  constructor(graha, longitude, speed) {
    this.graha = graha;
    this.longitude = longitude;
    this.speed = speed;
  }

  /** The sign it stands in. */
  get rashi() {
    return RashiById.get(Math.floor(this.longitude / 30));
  }

  /** How far into that sign, in degrees. */
  get degreeInRashi() {
    return this.longitude % 30;
  }

  /** The lunar mansion it stands in. */
  get nakshatra() {
    return NakshatraById.get(Math.floor(this.longitude / NAKSHATRA_DEG));
  }

  /** Which quarter of that mansion, 1 to 4. */
  get pada() {
    return Math.floor((this.longitude % NAKSHATRA_DEG) / PADA_DEG) + 1;
  }

  /**
   * Whether it is moving backwards. There is no flag at the boundary: a
   * graha is retrograde when its longitude is decreasing, which is what
   * the speed column says. Rahu always is.
   */
  get retrograde() {
    return this.speed < 0;
  }
}

/**
 * Every graha at one instant, in the sidereal zodiac.
 *
 * One call for the whole grid, never a loop: the boundary takes the
 * instants and the bodies together and answers with columns, so asking
 * for eight grahas costs one crossing rather than eight.
 */
function chart(ctx, instant) {
  // The canonical frame with two fields changed. Everything else — the
  // centre, the corrections, the equinox — is left as the SDK computes
  // it, so this asks for "what you would give me, but sidereal".
  const frame = { ...canonicalFrame(), sidereal: true, ayanamsha: Ayanamsha.Lahiri };
  const sky = ctx.positions({
    instants: [instant],
    bodies: GRAHAS.map(([, body]) => body),
    frame,
  });
  return GRAHAS.map(
    ([graha], index) =>
      new Placement(graha, sky.at(0, index).longitude, sky.at(0, index).longitudeSpeed),
  );
}

const two = (value) => String(value).padStart(2, '0');

const ctx = new Context({
  profile: 'nepali-default',
  locale: 'ne-Deva-NP',
  ephemeris: 'builtin',
});

// ── 1. The record, as it would be written on a form ────────────────────
const birthDay = date(Calendar.BikramSambat, 2042, 9, 17);
const gregorian = ctx.convert(birthDay, Calendar.Gregorian);
console.log(
  `born  BS ${birthDay.year}-${two(birthDay.month)}-${two(birthDay.day)}` +
    `  (${gregorian.year}-${two(gregorian.month)}-${two(gregorian.day)})` +
    `  00:20  Kathmandu`,
);

// ── 2. The instant, with the zone's own history ────────────────────────
const when = ctx.resolve(at(birthDay, { hour: 0, minute: 20 }), ianaZone('Asia/Kathmandu'));
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

// ── 3. The sky, sidereal ───────────────────────────────────────────────
const placements = chart(ctx, when.instantJdUtc);

// ── 4. The chart ───────────────────────────────────────────────────────
console.log('');
console.log('graha            sign               deg  nakshatra      pada');
console.log('─'.repeat(62));
for (const placed of placements) {
  const graha = ctx.entity(placed.graha);
  const rashi = ctx.entity(placed.rashi);
  const nakshatra = ctx.entity(placed.nakshatra);
  console.log(
    `${graha.name.padEnd(12)} ${(graha.glyph ?? '').padEnd(2)} ` +
      `${placed.retrograde ? '℞' : ' '} ` +
      `${rashi.name.padEnd(12)} ` +
      `${placed.degreeInRashi.toFixed(4).padStart(8)}°  ` +
      `${nakshatra.name.padEnd(14)} ${placed.pada}`,
  );
}

// ── What the SDK had to do to answer ───────────────────────────────────
// Every result carries the steps that produced it. Here the provider
// answered tropical positions and the SDK applied the ayanamsha and
// shifted the zodiac; against a provider that answers sidereal natively,
// those steps would say so instead.
console.log('');
const sky = ctx.positions({
  instants: [when.instantJdUtc],
  bodies: [Body.Sun],
  frame: { ...canonicalFrame(), sidereal: true, ayanamsha: Ayanamsha.Lahiri },
});
const steps = sky.steps.map((step) => `${step.name}:${step.implementation}`).join(', ');
console.log(`steps applied  ${steps}`);
console.log(`settings hash  ${ctx.settingsHash.slice(0, 16)}…`);
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
    const resolved = scoped.resolve(noTime, ianaZone('Asia/Kathmandu'));
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
