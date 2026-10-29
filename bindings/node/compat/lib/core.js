// What every shim call shares: the stored birth record read, the stored
// settings snapshot translated, the contexts opened under it and kept, the
// refusal, and the count of calls (`docs/03-design/compat-shim.md` §2,
// §3.7, §6.1, §6.3).
//
// HAND-WRITTEN. Not exported by the package: `index.js` is the surface, and
// this file is what its functions are built from. The tests import it
// directly, because a cache and a reader are proven through their own
// branches, not only through the calls that use them.

import { Calendar, Context, TeistroError, ianaZone, localMeanZone } from '@teistro/sdk';

/**
 * How many snapshots keep their contexts open. Most stored charts share a
 * handful of snapshots; the one opened longest ago is disposed to make room
 * (§2).
 */
export const CONTEXTS = 16;

/**
 * How many snapshots keep their translation. A translation holds no native
 * resource, so it is kept longer than a context: two snapshots that differ
 * only in a field set aside (the version triple) translate to the same
 * settings and share one context.
 */
export const TRANSLATIONS = 64;

/** Where the migration guide explains each refusal this file raises (§6.1). */
export const GUIDE = Object.freeze({
  /** Step 1 of moving a stored chart: its snapshot mapped to settings. */
  snapshot: 'docs/09-guidelines/baseline-migration.md#83-moving-the-stored-charts',
  /** What the SDK takes as a chart's input: the instant, the place, the clock. */
  birth: 'docs/09-guidelines/baseline-migration.md#82-what-to-store-with-the-sdk',
});

/** The Julian day of the Unix epoch, and a day's milliseconds. */
const UNIX_EPOCH_JD = 2440587.5;
const DAY_MS = 86_400_000;

/**
 * How far the stored Julian day may stand from the stored date (§3.7,
 * §6.1): half the step of a Julian day kept to six decimals, as an
 * application's birth table keeps it (43.2 ms), and the 0.04 ms a double
 * rounds a Julian day of this era by, twice: once where the day was
 * derived and once where its decimal was read. The corpus's records stand
 * at most 0.02 ms apart.
 */
export const INSTANT_TOLERANCE_MS = 43.25;

/**
 * A shim call refused before any computation: an entry point, a stored
 * field or a stored value the SDK does not take, or a birth record that
 * contradicts itself. `row` is the surface or settings row's id; `guide`
 * is the anchor in the migration guide that explains it. An SDK error is
 * never one of these: it passes through as the `TeistroError` it is.
 */
export class CompatRefused extends Error {
  /**
   * @param {string} row
   * @param {string} guide
   * @param {string} message what to do instead, in one sentence
   * @param {{ cause?: unknown }} [options]
   */
  constructor(row, guide, message, options) {
    super(message, options);
    this.name = 'CompatRefused';
    this.row = row;
    this.guide = guide;
  }

  /** The row, the anchor and the message on one line. */
  toString() {
    return `${this.name} [${this.row}]: ${this.message} (${this.guide})`;
  }
}

/** A finite number, or a `TypeError` naming the field. */
function number(record, field) {
  const value = record[field];
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw new TypeError(`birth.${field}: expected a finite number, not ${String(value)}`);
  }
  return value;
}

/** The stored UTC date's milliseconds, from a `Date` or its ISO text. */
function milliseconds(stored) {
  const ms = stored instanceof Date ? stored.getTime() : typeof stored === 'string' ? Date.parse(stored) : Number.NaN;
  if (Number.isNaN(ms)) {
    throw new TypeError(`birth.dateTimeUtc: expected a Date or an ISO-8601 UTC string, not ${String(stored)}`);
  }
  return ms;
}

/**
 * The stored birth record, read as §3.7 maps it: the stored UTC date is
 * the instant, as a UT Julian day, and the stored Julian day, which was
 * derived from it, is refused when it stands further from it than its
 * stored precision allows ({@link INSTANT_TOLERANCE_MS}); the place; the zone the clock is read in, local mean time at
 * the longitude when the record says so. The name, the gender and the
 * time's accuracy are kept for the calls that read them. The birth order
 * and the Bikram Sambat date are the application's, and nothing reads them.
 *
 * @param {object} record the stored birth record
 * @returns {Readonly<{instant: number, place: {latitude: number, longitude: number, altitude: number},
 *   zone: object, name: unknown, gender: unknown, timeAccuracy: unknown}>}
 */
export function readBirth(record) {
  if (typeof record !== 'object' || record === null || Array.isArray(record)) {
    throw new TypeError(
      "birth: expected the stored birth record, e.g. { dateTimeUtc, julianDay, latitude, longitude, altitude, timezone: 'Asia/Kathmandu', isLmt: false }",
    );
  }
  const julianDay = number(record, 'julianDay');
  const stored = milliseconds(record.dateTimeUtc);
  const apart = Math.abs(stored - (julianDay - UNIX_EPOCH_JD) * DAY_MS);
  if (apart > INSTANT_TOLERANCE_MS) {
    throw new CompatRefused(
      'birth.instant',
      GUIDE.birth,
      `the stored date ${new Date(stored).toISOString()} and Julian day ${julianDay} are ${apart} ms apart; ` +
        'derive the Julian day from the stored date again, then pass the record again',
    );
  }
  const instant = UNIX_EPOCH_JD + stored / DAY_MS;
  const latitude = number(record, 'latitude');
  const longitude = number(record, 'longitude');
  const altitude = number(record, 'altitude');
  if (typeof record.isLmt !== 'boolean') {
    throw new TypeError(`birth.isLmt: expected true or false, not ${String(record.isLmt)}`);
  }
  let zone;
  if (record.isLmt) {
    zone = localMeanZone(longitude);
  } else if (typeof record.timezone === 'string') {
    zone = ianaZone(record.timezone);
  } else {
    throw new TypeError(`birth.timezone: expected an IANA zone's name, not ${String(record.timezone)}`);
  }
  return Object.freeze({
    instant,
    place: Object.freeze({ latitude, longitude, altitude }),
    zone,
    name: record.name,
    gender: record.gender,
    timeAccuracy: record.birthTimeUncertainty,
  });
}

/**
 * What a chart request takes from a birth: the instant, the place and the
 * civil clock's offset there, read on `ctx` from the birth's zone.
 *
 * @param {Context} ctx
 * @param {ReturnType<typeof readBirth>} birth
 */
export function at(ctx, birth) {
  const { resolution } = ctx.time.civilOf(birth.instant, birth.zone, Calendar.Gregorian);
  return { instant: birth.instant, place: birth.place, utcOffsetSeconds: resolution.offsetSeconds };
}

/** How many times this thread called each row (§6.3). */
const counted = new Map();

/** One call of `row`, counted. */
export function count(row) {
  counted.set(row, (counted.get(row) ?? 0) + 1);
}

/**
 * Every row in `rows` with how many times this thread called it, zero
 * included, and any other row called.
 *
 * @param {readonly string[]} rows
 * @returns {Readonly<Record<string, number>>}
 */
export function calls(rows) {
  const out = {};
  for (const row of rows) out[row] = 0;
  for (const [row, times] of counted) out[row] = times;
  return Object.freeze(out);
}

/** Two settings patches, the second's groups laid over the first's. */
export function merged(a, b) {
  const out = { ...a };
  for (const [group, fields] of Object.entries(b)) out[group] = { ...a?.[group], ...fields };
  return out;
}

/** Disposes a snapshot's kept contexts: its variants, its KP and its natal one. */
function release({ natal, kp, variants }) {
  for (const context of variants.values()) context.dispose();
  kp?.dispose();
  natal.dispose();
}

/** The options a context takes that a snapshot decides, so `compat` refuses. */
const DECIDED = ['profile', 'settings'];

/**
 * The SDK contexts the shim computes on, opened under what each stored
 * snapshot means and kept (§2): a natal context per distinct translation,
 * and its KP context opened on first use. A context opened longest ago is
 * disposed when a seventeenth is needed.
 */
export class Contexts {
  /** The context options every opening shares, the ephemeris aside. */
  #shared;
  /** The chain the caller named, as a list, or `undefined` for the SDK's default. */
  #chain;
  /** The locale when the snapshot stores none. */
  #locale;
  /** The context the translations are asked of, opened on first use. */
  #translator = null;
  /** A snapshot's text to its translation, oldest first. */
  #translations = new Map();
  /** A translation's key to its contexts, least recently used first. */
  #open = new Map();
  #disposed = false;

  /** @param {object} [options] the context options, without `profile` and `settings` */
  constructor(options = {}) {
    if (typeof options !== 'object' || options === null || Array.isArray(options)) {
      throw new TypeError("options: expected the context options, e.g. { ephemeris: [teimeris()], locale: 'ne-Deva-NP' }");
    }
    for (const field of DECIDED) {
      if (options[field] !== undefined) {
        throw new TypeError(`${field}: each stored snapshot decides it; pass the snapshot to the call instead`);
      }
    }
    const { ephemeris, locale, ...shared } = options;
    this.#shared = shared;
    this.#chain = ephemeris === undefined ? undefined : Array.isArray(ephemeris) ? [...ephemeris] : [ephemeris];
    this.#locale = locale;
  }

  /** How many snapshots have their contexts open. */
  get size() {
    return this.#open.size;
  }

  #live() {
    if (this.#disposed) {
      throw new Error('this shim was disposed; open another one with compat()');
    }
  }

  /**
   * A snapshot translated by the SDK (`recordedSettings`), with the key
   * its contexts are kept under. A stored field or value the translation
   * refuses is a `CompatRefused` carrying the SDK's record as its cause.
   */
  translate(snapshot) {
    this.#live();
    const text = typeof snapshot === 'string' ? snapshot : JSON.stringify(snapshot);
    const kept = this.#translations.get(text);
    if (kept !== undefined) return kept;
    this.#translator ??= new Context({ ephemeris: 'NONE' });
    let recorded;
    try {
      recorded = this.#translator.recordedSettings(snapshot);
    } catch (error) {
      if (error instanceof TeistroError && error.field?.startsWith('snapshot.')) {
        throw new CompatRefused(
          error.field,
          GUIDE.snapshot,
          `${error.message}; open a Context with settings of your own choosing for this chart instead`,
          { cause: error },
        );
      }
      throw error;
    }
    const locale = recorded.locale ?? this.#locale;
    const translation = Object.freeze({
      key: JSON.stringify([recorded.natal, recorded.kp, recorded.ephemeris, locale ?? null]),
      recorded,
      locale,
    });
    if (this.#translations.size === TRANSLATIONS) {
      this.#translations.delete(this.#translations.keys().next().value);
    }
    this.#translations.set(text, translation);
    return translation;
  }

  /** The kept contexts of a translation, opened with its natal context if new. */
  #entry(snapshot) {
    const { key, recorded, locale } = this.translate(snapshot);
    const kept = this.#open.get(key);
    if (kept !== undefined) {
      this.#open.delete(key);
      this.#open.set(key, kept);
      return kept;
    }
    const natalChain = recorded.ephemeris === null ? this.#chain : [recorded.ephemeris, ...(this.#chain ?? [])];
    const entry = {
      natal: this.#context(recorded.natal, natalChain, locale),
      kp: null,
      opening: () => this.#context(recorded.kp, this.#chain, locale),
      variants: new Map(),
      variant: (patch) => this.#context({ ...recorded.natal, settings: merged(recorded.natal.settings, patch) }, natalChain, locale),
    };
    if (this.#open.size === CONTEXTS) {
      const [oldest, evicted] = this.#open.entries().next().value;
      this.#open.delete(oldest);
      release(evicted);
    }
    this.#open.set(key, entry);
    return entry;
  }

  #context(opening, ephemeris, locale) {
    return new Context({
      ...this.#shared,
      profile: opening.profile,
      settings: opening.settings,
      ...(locale === undefined ? {} : { locale }),
      ...(ephemeris === undefined ? {} : { ephemeris }),
    });
  }

  /** The natal context a snapshot means (§4.1). */
  natal(snapshot) {
    return this.#entry(snapshot).natal;
  }

  /** The KP context a snapshot means, opened on first use (§4.2). */
  kp(snapshot) {
    const entry = this.#entry(snapshot);
    entry.kp ??= entry.opening();
    return entry.kp;
  }

  /**
   * The natal context a snapshot means with `patch` laid over its
   * settings, opened on first use and kept beside it under `name`: a
   * convention the baseline casts one computation in, such as the frame
   * its muhurta search reads, or a second reckoning to read beside the
   * first. The patch is the same whenever `name` is.
   *
   * @param {string} name
   * @param {object} patch a settings patch, its groups laid over the natal ones
   */
  variant(snapshot, name, patch) {
    const entry = this.#entry(snapshot);
    let context = entry.variants.get(name);
    if (context === undefined) {
      context = entry.variant(patch);
      entry.variants.set(name, context);
    }
    return context;
  }

  /** Disposes every context; a second call does nothing. */
  dispose() {
    if (this.#disposed) return;
    this.#disposed = true;
    for (const entry of this.#open.values()) release(entry);
    this.#open.clear();
    this.#translations.clear();
    this.#translator?.dispose();
  }
}
