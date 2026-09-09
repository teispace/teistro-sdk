/**
 * The Teistro SDK for Node: the layer a consumer uses.
 *
 * HAND-WRITTEN, and thin on purpose. Everything beneath it is generated
 * from the API description: the addon (`native/src/generated.rs`), the
 * types (`catalogue.d.ts`, `types.d.ts`, `blob.d.ts`), the catalogue's
 * tables (`catalogue.js`) and the result-blob decoders (`blob.js`). What
 * this file adds is what a generator cannot know: where the addon is,
 * validation at the door, defaults, errors with their field and hint, and
 * results decoded on first use rather than eagerly.
 */

import { createRequire } from 'node:module';
import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  ABI_VERSION,
  AyanaById,
  BodyById,
  CONTEXT_TEST_PROVIDER,
  CalendarById,
  ChartKind,
  DayPartById,
  ChartKindById,
  ChoghadiyaById,
  DirectionById,
  GrahaById,
  HouseSystemById,
  KaalaById,
  KaranaById,
  LunarMonthById,
  MasaById,
  MuhurtaYogaById,
  NakshatraById,
  PakshaById,
  PanchakaById,
  RashiById,
  SDK_VERSION,
  TithiById,
  TimeScaleById,
  VaraById,
  YogaById,
} from './catalogue.js';
import { decodeCharts, decodeIntlRender, decodePanchanga, decodePositions } from './blob.js';
import { entityForms, messages } from './messages.js';

const HERE = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

/**
 * The npm package that carries this host's prebuilt addon.
 *
 * A release publishes one package per platform and this package depends
 * on all of them as optional dependencies, so npm installs the one that
 * matches and skips the rest. The name is built from Node's own words for
 * the host, which are the same words npm matched `os` and `cpu` against.
 */
export function platformPackage() {
  return `@teistro/sdk-${process.platform}-${process.arch}`;
}

/**
 * The addon's path inside its platform package, or `null` when npm did not
 * install one: on a host no release covers, under `--no-optional`, or in
 * a lockfile written on another platform.
 */
function packagedAddon() {
  try {
    return require.resolve(`${platformPackage()}/teistro.node`);
  } catch {
    return null;
  }
}

/**
 * The addon: the one a path names, then the one npm installed for this
 * host, then this repository's own build.
 *
 * A consumer only ever has the second; a contributor only ever has the
 * third, because the platform packages are published rather than checked
 * in. The order matters anyway for the case where someone has both and
 * wants the release they installed.
 */
function loadAddon() {
  const named = process.env.TEISTRO_ADDON;
  const candidates = [
    named,
    packagedAddon(),
    join(HERE, '..', 'native', 'index.node'),
    join(HERE, '..', '..', '..', 'target', 'release', addonName()),
    join(HERE, '..', '..', '..', 'target', 'debug', addonName()),
  ].filter(Boolean);
  const found = candidates.find((path) => existsSync(path));
  if (!found) {
    throw new Error(
      `no Teistro addon for ${process.platform}-${process.arch}. Looked in:\n  ${candidates.join(
        '\n  ',
      )}\nInstall the prebuilt addon with \`npm install ${platformPackage()}\` (npm normally does that for you), build it with \`cargo build --release -p teistro-node\`, or set TEISTRO_ADDON to its path.`,
    );
  }
  return [require(found), found === named];
}

/**
 * Whether a build may be loaded, as a sentence when it may not.
 *
 * The two halves of a binding must be one build: the addon carries the
 * library, and these files were generated from a description of it. A
 * mismatched ABI or version is refused outright. A sanitizer build is
 * refused because it answers differently and slowly and is never chosen
 * by accident; an unoptimised one is refused only when the loader found
 * it itself, because naming a path is a deliberate act and a development
 * build is what a developer means by it.
 *
 * @param {object} info what `buildInfo()` parsed
 * @param {boolean} named whether the path was given rather than searched
 */
export function refuseBuild(info, named) {
  if (info.abi !== ABI_VERSION) {
    return `the addon implements ABI ${info.abi}, these types were generated for ${ABI_VERSION}`;
  }
  if (info.sdk !== SDK_VERSION) {
    return `the addon is Teistro ${info.sdk}, these types were generated from ${SDK_VERSION}`;
  }
  if (info.sanitizer) {
    return `the addon is a ${info.sanitizer} sanitizer build, which is not for use`;
  }
  if (!named && info.optimised === false) {
    return `the addon at ${info.profile ?? 'an unoptimised path'} is an unoptimised build; build it with \`--release\`, or set TEISTRO_ADDON to load this one deliberately`;
  }
  return null;
}

/** What the loaded addon says about its own build. */
function readBuildInfo(addon) {
  try {
    return JSON.parse(addon.buildInfo());
  } catch (cause) {
    throw new Error(`the addon did not describe its build: ${cause.message}`);
  }
}

function addonName() {
  if (process.platform === 'darwin') return 'libteistro_node.dylib';
  if (process.platform === 'win32') return 'teistro_node.dll';
  return 'libteistro_node.so';
}

const [native, wasNamed] = loadAddon();

/** What the loaded addon says about its own build (ADR-0007). */
export const buildInfo = Object.freeze(readBuildInfo(native));

const refusal = refuseBuild(buildInfo, wasNamed);
if (refusal) throw new Error(refusal);

/**
 * A failed call. The status and its code are the same in every binding;
 * `field`, `hint` and `messageKey` are there when the library named them.
 */
export class TeistroError extends Error {
  /** @param {import('./types.js').LastErrorLike} record */
  constructor(record) {
    super(record.message ?? 'the call failed');
    this.name = 'TeistroError';
    this.status = record.status;
    this.code = record.code;
    this.detail = record.detail ?? null;
    this.field = record.field ?? null;
    this.hint = record.hint ?? null;
    this.messageKey = record.messageKey ?? null;
    this.providerCode = record.providerCode ?? 0;
  }

  /** The message, the field and the hint on one line. */
  toString() {
    const where = this.field ? ` (field \`${this.field}\`)` : '';
    const hint = this.hint ? `; ${this.hint}` : '';
    return `${this.name} [${this.status}]: ${this.message}${where}${hint}`;
  }
}

/**
 * Runs a call on the addon and reports its failure the way this binding
 * promises to: what your own code threw comes back as itself, and what
 * the library refused comes back as a `TeistroError` with a status.
 *
 * Only a code crosses the C boundary, so a provider written in JavaScript
 * would otherwise reach its caller as a number. `thrown` is the object it
 * threw, kept on this side for the length of one call and put back here.
 * The Dart and Python bindings do exactly this.
 *
 * @param {object|null} context the addon handle, for `lastError`
 * @param {Function} call the call to make
 * @param {{error: unknown}} [thrown] where this context's provider leaves
 *   what it threw
 */
function guarded(context, call, thrown) {
  if (thrown) thrown.error = undefined;
  try {
    return call();
  } catch (cause) {
    const record = context?.lastError?.();
    const own = thrown?.error;
    if (own !== undefined) {
      thrown.error = undefined;
      // The boundary's refusal kept as the cause: the caller catches the
      // type it wrote, and a stack trace still shows what the port made
      // of it. `Error.cause` is the standard slot for exactly this.
      if (own instanceof Error && own.cause === undefined && record) {
        own.cause = new TeistroError(record);
      }
      throw own;
    }
    if (!record) throw cause;
    // A provider that failed inside the addon rather than in JavaScript —
    // a column of the wrong length read by the native adapter — leaves
    // its sentence in the caught message rather than in `thrown`.
    const fromProvider = record.status === 'provider' && cause?.message;
    throw new TeistroError(fromProvider ? { ...record, message: cause.message } : record);
  }
}

/**
 * An object as the addon takes it: a field the caller left `null` or
 * `undefined` is dropped rather than passed, because the mechanical layer
 * distinguishes an absent field from a null one and JavaScript does not.
 * Nested plain objects are cleaned too; arrays and typed arrays are not
 * touched.
 */
function clean(value) {
  if (value === null || value === undefined) return undefined;
  if (Array.isArray(value) || ArrayBuffer.isView(value) || typeof value !== 'object') {
    return value;
  }
  const out = {};
  for (const [key, inner] of Object.entries(value)) {
    const kept = clean(inner);
    if (kept !== undefined) out[key] = kept;
  }
  return out;
}

/**
 * A provider as the addon takes it: its description and its callback,
 * apart, because the mechanical layer holds a reference to the function
 * itself rather than a property of an object it does not own.
 */
function describeProvider(provider) {
  // Where a provider written in JavaScript leaves what it threw, for the
  // length of one call: only a code crosses the boundary, so without this
  // the caller would get a summary of its error instead of the error.
  const thrown = { error: undefined };
  if (provider === undefined || provider === null) return [undefined, undefined, thrown];
  if (typeof provider.positions !== 'function') {
    throw new TypeError('provider: expected a `positions(request)` function');
  }
  if (typeof provider.name !== 'string' || provider.name.length === 0) {
    throw new TypeError('provider: expected a `name`, which every result is stamped with');
  }
  if (!Array.isArray(provider.bodies) || provider.bodies.length === 0) {
    throw new TypeError('provider: expected `bodies`, the catalogue keys it answers');
  }
  const info = clean({
    name: provider.name,
    bodies: provider.bodies,
    version: provider.version,
    dataVersion: provider.dataVersion,
    // Named as the Dart and Python bindings name them, so a provider
    // author writing against one of the three reads the same words in
    // all of them (ADR-0004: one API, not three).
    jdMin: provider.jdMin,
    jdMax: provider.jdMax,
    nativeFrameBits:
      provider.nativeFrame === undefined
        ? undefined
        : native.framePack(clean(provider.nativeFrame)),
    speeds: provider.speeds,
    deterministic: provider.deterministic,
  });
  // The callback answers with plain arrays; a column left out is zeroes,
  // which is what a provider that computes no speeds means.
  // The coverage span, and only that: a topocentric frame without an
  // observer, a body the provider never declared and an instant that is
  // not a number are all refused by the port itself, on the SDK's side of
  // the boundary, where the sentence survives into a `TeistroError` that
  // names what is missing. The Dart and Python adapters keep exactly this
  // much and no more.
  const validate = (request) => {
    const low = provider.jdMin ?? 1721057.5;
    const high = provider.jdMax ?? 2816787.5;
    for (const jd of request.jds) {
      if (jd < low || jd > high) {
        throw new RangeError(
          `the instant ${jd} is outside the provider's coverage (${low} to ${high})`,
        );
      }
    }
  };
  const answering = (request) => {
    validate(request);
    const answer = provider.positions(request);
    // Nothing means "I cannot produce that frame"; the SDK then asks for
    // the provider's native frame and completes the rest itself.
    //
    // Answering at all asserts the answer is in the frame that was asked
    // for: `frameBits` left out below means `request.frameBits`, not the
    // provider's own. A provider that computes in one frame must compare
    // `request.frameBits` with its own and return nothing instead —
    // `example/your_own_ephemeris.mjs` does exactly that.
    if (answer === null || answer === undefined) return null;
    const cells = request.jds.length * request.bodies.length;
    // A column left out is zeroes, which is what a provider that computes
    // no speeds means; a column of the wrong length is a mistake, and is
    // said so here rather than read past its end further down.
    const column = (name, fill = Float64Array) => {
      const values = answer[name];
      if (values === undefined || values === null) return Array.from(new fill(cells));
      if (values.length !== cells) {
        throw new RangeError(
          `the provider returned ${values.length} values in \`${name}\` for ${cells} cells`,
        );
      }
      return Array.from(values);
    };
    return {
      frameBits: answer.frameBits ?? request.frameBits,
      lon: column('lon'),
      lat: column('lat'),
      dist: column('dist'),
      lonSpeed: column('lonSpeed'),
      latSpeed: column('latSpeed'),
      distSpeed: column('distSpeed'),
      status: column('status', Int32Array),
      source: column('source', Uint32Array),
    };
  };
  const positions = (request) => {
    try {
      return answering(request);
    } catch (error) {
      // Kept, and rethrown by `guarded` once the call has unwound: the
      // addon can only answer the SDK with a code.
      thrown.error = error;
      throw error;
    }
  };
  return [info, positions, thrown];
}

/** A finite number, or a `TypeError` naming the argument. */
function finite(value, what) {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw new TypeError(`${what}: expected a finite number, got ${String(value)}`);
  }
  return value;
}

/**
 * The instants as the addon takes them: a plain array of doubles.
 *
 * `allowEmpty` is what separates a batch entry point from a grid one: a
 * batch of none is a legitimate ask — a caller who filtered a list to
 * nothing gets an empty result rather than a refusal — while an empty
 * grid of instants to place bodies at is more likely a mistake. Which
 * of the two `ts_positions` should be is an open question
 * (`03-design/chart-at-the-boundary.md` §8); the boundary itself takes
 * either.
 */
function instants(values, what, { allowEmpty = false } = {}) {
  const list = ArrayBuffer.isView(values) ? Array.from(values) : values;
  if (!Array.isArray(list) || (!allowEmpty && list.length === 0)) {
    throw new TypeError(`${what}: expected a non-empty array of Julian days`);
  }
  return list.map((value, i) => finite(value, `${what}[${i}]`));
}

/** A computed result: the blob, decoded on first use and only once. */
class Decoded {
  #bytes;
  #decode;
  #value = null;

  constructor(bytes, decode) {
    this.#bytes = bytes;
    this.#decode = decode;
  }

  /** The bytes the library returned; the columns are views over them. */
  get bytes() {
    return this.#bytes;
  }

  /** The decoded sections, decoded once. */
  get decoded() {
    this.#value ??= this.#decode(this.#bytes);
    return this.#value;
  }
}

/** Positions over a grid, with the cells readable one at a time. */
/**
 * One row of a decoded column section, as a plain object.
 *
 * The columns are typed-array views over the blob's bytes; this reads
 * one index out of each into the shape an application wants, which is a
 * row. Every column comes across, so a column added to a section needs
 * no change here. The values that name a catalogue member stay ids —
 * `Chart` names the ones a reader reaches for.
 */
function row(columns, index) {
  const out = {};
  for (const [key, value] of Object.entries(columns)) {
    if (key !== 'length') out[key] = value[index];
  }
  return out;
}

/**
 * A batch of founded charts at one place: where every graha stands, in
 * which bhava under both readings, in which zodiac, on which day, at
 * what time of that day.
 *
 * The blob is decoded on first use and only once, so a batch that is
 * fetched and stored costs nothing until something reads it, and the
 * charts in it are views over those bytes rather than copies.
 */
export class Charts extends Decoded {
  constructor(bytes) {
    super(bytes, decodeCharts);
  }

  /** How many charts the batch holds. */
  get length() {
    return this.decoded.chartCount;
  }

  /** What kind of chart these are. */
  get kind() {
    return ChartKindById.get(this.decoded.kind) ?? 'unknown';
  }

  /** The place they were all founded at. */
  get place() {
    const d = this.decoded;
    return { latitude: d.latitudeDeg, longitude: d.longitudeDeg, altitude: d.altitudeM };
  }

  /**
   * One chart of the batch, by index.
   *
   * @param {number} index 0 to `length - 1`
   * @returns {Chart}
   */
  at(index) {
    if (!Number.isInteger(index) || index < 0 || index >= this.length) {
      throw new RangeError(`chart ${index} is outside a batch of ${this.length}`);
    }
    return new Chart(this, index);
  }

  /** Every chart, in the order the instants were asked for. */
  *[Symbol.iterator]() {
    for (let i = 0; i < this.length; i += 1) yield this.at(i);
  }

  /** The steps the SDK applied, each `{ name, implementation }`. */
  get steps() {
    return JSON.parse(this.decoded.steps);
  }

  /** The solar model that reckoned the days, as it describes itself. */
  get model() {
    return this.decoded.model;
  }

  /** The provenance envelope: what computed these, and under what. */
  get provenance() {
    return JSON.parse(this.decoded.provenance);
  }
}

/**
 * One founded chart: a view over its batch, not a copy.
 *
 * A batch of one is the ordinary case, and `Context.found` hands back
 * this rather than the batch around it.
 */
export class Chart {
  #batch;
  #index;

  constructor(batch, index) {
    this.#batch = batch;
    this.#index = index;
  }

  /** The batch this chart belongs to. */
  get batch() {
    return this.#batch;
  }

  /** Where in that batch it sits. */
  get index() {
    return this.#index;
  }

  /** The instant the chart is cast for, as a Julian day (UTC). */
  get instant() {
    return this.#batch.decoded.cast.instant[this.#index];
  }

  /** What kind of chart this is. */
  get kind() {
    return this.#batch.kind;
  }

  /** The place it was founded at. */
  get place() {
    return this.#batch.place;
  }

  /** The lagna at the instant, in the chart's zodiac, degrees. */
  get lagnaDeg() {
    return this.#batch.decoded.cast.lagnaDeg[this.#index];
  }

  /** The lagna at the sunrise that opened the day, degrees. */
  get dayLagnaDeg() {
    return this.#batch.decoded.cast.dayLagnaDeg[this.#index];
  }

  /** The ayanamsha applied at this instant, degrees; zero if tropical. */
  get ayanamshaOffsetDeg() {
    return this.#batch.decoded.cast.ayanamshaOffsetDeg[this.#index];
  }

  /**
   * Which arc of its day the instant falls in.
   *
   * This and `dayElapsed` belong to the **instant**, not to the day, so
   * they are the chart's rather than `day`'s — which is what the
   * panchanga blob's arrival settled.
   */
  get dayPart() {
    return DayPartById.get(this.#batch.decoded.cast.dayPart[this.#index]) ?? 'unknown';
  }

  /** How far through that arc the instant is, 0 to 1. */
  get dayElapsed() {
    return this.#batch.decoded.cast.dayElapsed[this.#index];
  }

  /**
   * The day the chart belongs to, which is not always its civil date.
   *
   * `vara` is named; the rest are the values the blob carries.
   */
  get day() {
    const day = row(this.#batch.decoded.day, this.#index);
    return { ...day, vara: VaraById.get(day.vara) ?? 'unknown' };
  }

  /** Where in its day the moment falls, and which hora holds it. */
  get timing() {
    const timing = row(this.#batch.decoded.timing, this.#index);
    return { ...timing, horaLord: GrahaById.get(timing.horaLord) ?? 'unknown' };
  }

  /**
   * The grahas, in the catalogue's order, one object each.
   *
   * The columns underneath are views over the blob's bytes, charts
   * outermost; this reads this chart's stride out of them into the shape
   * an application wants, which is a row.
   */
  get grahas() {
    const g = this.#batch.decoded.grahas;
    const count = this.#batch.decoded.grahaCount;
    const base = this.#index * count;
    return Array.from({ length: count }, (_, j) => {
      const i = base + j;
      return {
        graha: GrahaById.get(g.graha[i]) ?? 'unknown',
        longitudeDeg: g.longitudeDeg[i],
        tropicalDeg: g.tropicalDeg[i],
        latitudeDeg: g.latitudeDeg[i],
        distanceAu: g.distanceAu[i],
        speedDegPerDay: g.speedDegPerDay[i],
        retrograde: g.speedDegPerDay[i] < 0,
        house: {
          bhava: g.houseBhava[i],
          method: HouseSystemById.get(g.houseMethod[i]) ?? 'unknown',
          through: g.houseThrough[i],
          fromMadhyaDeg: g.houseFromMadhyaDeg[i],
        },
        placement: {
          bhava: g.placementBhava[i],
          method: HouseSystemById.get(g.placementMethod[i]) ?? 'unknown',
          through: g.placementThrough[i],
          fromMadhyaDeg: g.placementFromMadhyaDeg[i],
        },
      };
    });
  }

  /** The twelve bhavas for "which house is it in", first to twelfth. */
  get houses() {
    return this.#bhavas(this.#batch.decoded.houses);
  }

  /** The twelve bhavas of the chart's chalit. */
  get chalit() {
    return this.#bhavas(this.#batch.decoded.chalit);
  }

  #bhavas(columns) {
    const base = this.#index * 12;
    return Array.from({ length: 12 }, (_, j) => ({
      madhyaDeg: columns.madhyaDeg[base + j],
      sandhiDeg: columns.sandhiDeg[base + j],
    }));
  }

  /** The steps the SDK applied, each `{ name, implementation }`. */
  get steps() {
    return this.#batch.steps;
  }

  /** The provenance envelope of the batch this chart came from. */
  get provenance() {
    return this.#batch.provenance;
  }
}

/**
 * A batch of daily panchangas at one place, decoded on first use and
 * only once.
 *
 * Every per-day list is concatenated across the batch, so a day's rows
 * are found by adding up every earlier day's count. That sum is done
 * once, on first use, rather than per access: the alternative is
 * quadratic over a year of days, which is the shape an almanac is
 * actually asked for.
 */
export class Almanac extends Decoded {
  #starts = null;

  constructor(bytes) {
    super(bytes, decodePanchanga);
  }

  /** How many days the batch holds. */
  get length() {
    return this.decoded.dayCount;
  }

  /** The place they were all founded at. */
  get place() {
    const d = this.decoded;
    return { latitude: d.latitudeDeg, longitude: d.longitudeDeg, altitude: d.altitudeM };
  }

  /** The civil calendar the days' dates are read in. */
  get calendar() {
    return CalendarById.get(this.decoded.calendar) ?? 'unknown';
  }

  /** The solar model that reckoned the days, as it describes itself. */
  get model() {
    return this.decoded.model;
  }

  /** The provenance envelope: what computed these, and under what. */
  get provenance() {
    return JSON.parse(this.decoded.provenance);
  }

  /**
   * One day of the batch, by index.
   *
   * @param {number} index 0 to `length - 1`
   * @returns {AlmanacDay}
   */
  at(index) {
    if (!Number.isInteger(index) || index < 0 || index >= this.length) {
      throw new RangeError(`day ${index} is outside a batch of ${this.length}`);
    }
    return new AlmanacDay(this, index);
  }

  /** Every day, in the order the range runs. */
  *[Symbol.iterator]() {
    for (let i = 0; i < this.length; i += 1) yield this.at(i);
  }

  /**
   * Where day `index`'s rows of a per-day list begin and end.
   *
   * @param {string} list a column of the `counts` section
   * @param {number} index the day
   * @returns {[number, number]}
   */
  range(list, index) {
    this.#starts ??= prefixSums(this.decoded.counts);
    const starts = this.#starts[list];
    return [starts[index], starts[index + 1]];
  }
}

/** The running totals of every count column, computed once. */
function prefixSums(counts) {
  const out = {};
  for (const [name, column] of Object.entries(counts)) {
    if (name === 'length') continue;
    const starts = new Uint32Array(column.length + 1);
    for (let i = 0; i < column.length; i += 1) starts[i + 1] = starts[i] + column[i];
    out[name] = starts;
  }
  return out;
}

/**
 * One day of an almanac: a view over its batch, not a copy.
 *
 * The lists are built on demand from the blob's columns, so a day costs
 * nothing until something is asked of it.
 */
export class AlmanacDay {
  #batch;
  #index;

  constructor(batch, index) {
    this.#batch = batch;
    this.#index = index;
  }

  /** The batch this day belongs to. */
  get batch() {
    return this.#batch;
  }

  /** Where in that batch it sits. */
  get index() {
    return this.#index;
  }

  /**
   * The day itself: its arc, its date and how it was reckoned — the
   * same eighteen fields a chart's day carries, decoded into the same
   * type.
   */
  get day() {
    const day = row(this.#batch.decoded.day, this.#index);
    return { ...day, vara: VaraById.get(day.vara) ?? 'unknown' };
  }

  /** What the spans are clipped to. */
  get window() {
    const d = this.#batch.decoded.days;
    return { from: d.windowFrom[this.#index], to: d.windowTo[this.#index] };
  }

  /** The lunar month, under both conventions, and the fortnight. */
  get month() {
    const d = this.#batch.decoded.days;
    const i = this.#index;
    return {
      month: MasaById.get(d.month[i]) ?? 'unknown',
      amanta: MasaById.get(d.amanta[i]) ?? 'unknown',
      purnimanta: MasaById.get(d.purnimanta[i]) ?? 'unknown',
      paksha: PakshaById.get(d.paksha[i]) ?? 'unknown',
      convention: LunarMonthById.get(this.#batch.decoded.lunarMonth) ?? 'unknown',
    };
  }

  /** Which half of the year the day falls in. */
  get ayana() {
    return AyanaById.get(this.#batch.decoded.days.ayana[this.#index]) ?? 'unknown';
  }

  /** The direction not to travel in, which is the vara's. */
  get dishaShool() {
    return DirectionById.get(this.#batch.decoded.days.dishaShool[this.#index]) ?? 'unknown';
  }

  /** When the Sun entered a new sign inside the day, or `null`. */
  get sankranti() {
    const d = this.#batch.decoded.days;
    return d.hasSankranti[this.#index] ? d.sankranti[this.#index] : null;
  }

  /** Abhijit, and whether it is effective; `null` on a day with no daylight. */
  get abhijit() {
    const d = this.#batch.decoded.days;
    const i = this.#index;
    if (!d.hasAbhijit[i]) return null;
    return {
      from: d.abhijitFrom[i],
      to: d.abhijitTo[i],
      effective: d.abhijitEffective[i] === 1,
    };
  }

  /** Brahma muhurta, or `null` when the night before is not known. */
  get brahma() {
    const d = this.#batch.decoded.days;
    const i = this.#index;
    return d.hasBrahma[i] ? { from: d.brahmaFrom[i], to: d.brahmaTo[i] } : null;
  }

  /** The tithis that touch the day. */
  get tithi() {
    return this.#spans('tithi', TithiById);
  }

  /** The nakshatras the Moon was in. */
  get nakshatra() {
    return this.#spans('nakshatra', NakshatraById);
  }

  /** The nitya yogas. */
  get yoga() {
    return this.#spans('yoga', YogaById);
  }

  /** The karanas: half-tithis, so three or four on an ordinary day. */
  get karana() {
    return this.#spans('karana', KaranaById);
  }

  /** Panchaka, while the Moon is in the last five nakshatras. */
  get panchaka() {
    return this.#spans('panchaka', PanchakaById);
  }

  /** The signs the Moon stood in. */
  get moonSigns() {
    return this.#spans('moonSigns', RashiById);
  }

  /** The signs the Sun stood in; two only on a sankranti day. */
  get sunSigns() {
    return this.#spans('sunSigns', RashiById);
  }

  /** The inauspicious eighths of the daylight. */
  get kaalas() {
    const c = this.#batch.decoded.kaalas;
    return this.#rows('kaalas', (i) => ({
      kaala: KaalaById.get(c.kaala[i]) ?? 'unknown',
      from: c.from[i],
      to: c.to[i],
    }));
  }

  /** Eight choghadiya of the daylight and eight of the night. */
  get choghadiya() {
    const c = this.#batch.decoded.choghadiya;
    return this.#rows('choghadiya', (i) => ({
      choghadiya: ChoghadiyaById.get(c.choghadiya[i]) ?? 'unknown',
      lord: GrahaById.get(c.lord[i]) ?? 'unknown',
      from: c.from[i],
      to: c.to[i],
      daytime: c.daytime[i] === 1,
    }));
  }

  /** The twenty-four horas, from sunrise. */
  get horas() {
    const c = this.#batch.decoded.horas;
    return this.#rows('horas', (i) => ({
      number: c.number[i],
      lord: GrahaById.get(c.lord[i]) ?? 'unknown',
      start: c.start[i],
      end: c.end[i],
    }));
  }

  /** The thirty muhurtas: fifteen of the daylight, then fifteen of the night. */
  get muhurtas() {
    const c = this.#batch.decoded.muhurtas;
    return this.#rows('muhurtas', (i) => ({
      from: c.from[i],
      to: c.to[i],
      daylight: c.daylight[i] === 1,
    }));
  }

  /** Every moonrise and moonset inside the day's moon window. */
  get moonEvents() {
    const c = this.#batch.decoded.moonEvents;
    return this.#rows('moonEvents', (i) => ({
      kind: c.kind[i] === 0 ? 'rise' : 'set',
      instant: c.instant[i],
    }));
  }

  /** The muhurta yogas that held, with what made each hold. */
  get muhurtaYogas() {
    const c = this.#batch.decoded.muhurtaYogas;
    return this.#rows('muhurtaYogas', (i) => ({
      yoga: MuhurtaYogaById.get(c.yoga[i]) ?? 'unknown',
      from: c.from[i],
      to: c.to[i],
      because: {
        kind: c.becauseKind[i] === 0 ? 'vara-nakshatra' : 'vara-tithi-nakshatra',
        vara: VaraById.get(c.becauseVara[i]) ?? 'unknown',
        // A `vara-nakshatra` cause has no tithi, and the blob leaves the
        // column at nought rather than at a tithi that did not make it.
        tithi: c.becauseKind[i] === 0 ? null : (TithiById.get(c.becauseTithi[i]) ?? 'unknown'),
        nakshatra: NakshatraById.get(c.becauseNakshatra[i]) ?? 'unknown',
      },
    }));
  }

  /** The completion steps and the settings, as the batch stamped them. */
  get provenance() {
    return this.#batch.provenance;
  }

  #rows(list, build) {
    const [from, to] = this.#batch.range(list, this.#index);
    return Array.from({ length: to - from }, (_, k) => build(from + k));
  }

  #spans(list, names) {
    const c = this.#batch.decoded[list];
    return this.#rows(list, (i) => ({
      member: names.get(c.member[i]) ?? 'unknown',
      whole: { from: c.wholeFrom[i], to: c.wholeTo[i] },
      inside: { from: c.insideFrom[i], to: c.insideTo[i] },
    }));
  }
}

export class Positions extends Decoded {
  constructor(bytes) {
    super(bytes, decodePositions);
  }

  /** The instants of the request, in order. */
  get instants() {
    return this.decoded.instants.jd;
  }

  /** The bodies of the request, in order, by their catalogue key. */
  get bodies() {
    return Array.from(this.decoded.bodies.body, (id) => BodyById.get(id) ?? 'unknown');
  }

  /** The bodies as the ids the blob carries, without a copy. */
  /**
   * How many instants the grid covers, which is the stride a caller
   * needs to read a column: cell `i * bodyCount + j` is instant `i`,
   * body `j`. The Dart binding names these the same way.
   */
  get jdCount() {
    return this.decoded.jdCount;
  }

  /** How many bodies the grid covers. */
  get bodyCount() {
    return this.decoded.bodyCount;
  }

  get bodyIds() {
    return this.decoded.bodies.body;
  }

  /** The time scale the instants are on. */
  get scale() {
    return TimeScaleById.get(this.decoded.scale) ?? 'unknown';
  }

  /** The cells, instants outermost, as typed arrays over the blob. */
  get cells() {
    return this.decoded.cells;
  }

  /** The completion steps the SDK applied, in order. */
  get steps() {
    return JSON.parse(this.decoded.steps);
  }

  /** Everything that reproduces this result (ADR-0020). */
  get provenance() {
    return JSON.parse(this.decoded.provenance);
  }

  /**
   * One cell as a plain object, built on demand; the columns stay where
   * they are.
   *
   * @param {number} instant the instant's index
   * @param {number} body the body's index
   */
  at(instant, body) {
    const { cells } = this.decoded;
    const width = this.decoded.bodyCount;
    if (instant < 0 || instant >= this.decoded.jdCount || body < 0 || body >= width) {
      throw new RangeError(
        `at(${instant}, ${body}): the grid is ${this.decoded.jdCount} by ${width}`,
      );
    }
    const i = instant * width + body;
    return {
      longitude: cells.lon[i],
      latitude: cells.lat[i],
      distance: cells.dist[i],
      longitudeSpeed: cells.lonSpeed[i],
      latitudeSpeed: cells.latSpeed[i],
      distanceSpeed: cells.distSpeed[i],
      status: cells.status[i],
      source: cells.source[i],
    };
  }
}

/** A rendered message. */
export class Rendered extends Decoded {
  constructor(bytes) {
    super(bytes, decodeIntlRender);
  }

  /** The plain text, markup stripped. */
  get text() {
    return this.decoded.text;
  }

  /** The locale whose message answered, `null` when none had it. */
  get resolvedFrom() {
    return this.decoded.resolvedFrom || null;
  }

  /** Whether a fallback locale answered. */
  get isFallback() {
    return this.decoded.isFallback === 1;
  }

  /** Whether a runtime override answered. */
  get isOverride() {
    return this.decoded.isOverride === 1;
  }

  /** Every problem met; rendering continues past each. */
  get warnings() {
    return JSON.parse(this.decoded.warnings);
  }

  /** The text, so a rendered message can be used where a string is. */
  toString() {
    return this.text;
  }
}

/**
 * A context: settings resolved from a profile and a patch, a locale, and
 * an ephemeris. One context serves one thread; a worker builds its own.
 */
export class Context {
  #inner;
  #messages = null;
  /** Where this context's own provider leaves what it threw. */
  #thrown;
  /** Whether `dispose` has already freed the handle. */
  #disposed = false;

  /**
   * Runs a call on the addon, putting back whatever this context's
   * provider threw. Every call can reach the provider — a chart asks for
   * positions — so every call goes through here.
   */
  #call(run) {
    // Said here rather than at the boundary: a call on a freed handle is
    // refused there as `invalid argument`, which does not tell a reader
    // that the context they disposed is the argument. The Dart and Python
    // bindings name it the same way.
    if (this.#disposed) {
      throw new Error('this context was disposed; open another one');
    }
    return guarded(this.#inner, run, this.#thrown);
  }

  /**
   * @param {object} [options]
   * @param {string} [options.profile] a shipped profile's id; the default
   *   is what `defaultProfile()` names
   * @param {object} [options.settings] a settings patch over the profile
   * @param {string} [options.locale] the locale every render resolves from
   * @param {boolean} [options.testProvider] use the SDK's analytic test
   *   provider, for examples and tests only
   * @param {object} [options.provider] an ephemeris of your own: `name`,
   *   `bodies` (their catalogue keys) and `positions(request)`, which
   *   answers with the columns; everything else has a default
   */
  constructor(options = {}) {
    const { profile, settings, locale, testProvider = false, provider } = options;
    const [info, positions, thrown] = describeProvider(provider);
    this.#thrown = thrown;
    this.#inner = guarded(null, () =>
      new native.Context(
        clean({
          flags: testProvider ? CONTEXT_TEST_PROVIDER : 0,
          profile,
          settingsJson: settings === undefined ? undefined : JSON.stringify(settings),
          locale,
        }),
        info,
        positions,
      ),
    );
  }

  /** The id of the profile the settings came from. */
  get profile() {
    return this.#call(() => this.#inner.profile());
  }

  /** The resolved settings, as their canonical document. */
  get settings() {
    return JSON.parse(this.settingsJson);
  }

  /**
   * The same document as the text the library wrote, which is what the
   * settings hash is taken over and what a stored chart keeps.
   */
  get settingsJson() {
    return this.#call(() => this.#inner.settingsJson());
  }

  /** The SHA-256 of the canonical settings, in hex; every result carries it. */
  get settingsHash() {
    const hash = this.#call(() => this.#inner.settingsHash());
    return Buffer.from(hash.bytes).toString('hex');
  }

  /** The locale every render resolves from. */
  get locale() {
    return this.#call(() => this.#inner.intlLocale());
  }

  set locale(tag) {
    this.#call(() => this.#inner.intlSetLocale(tag));
  }

  /** The SDK's canonical frame: apparent geocentric ecliptic of date, tropical. */
  canonicalFrame() {
    return this.#call(() => native.frameCanonical());
  }

  /**
   * Positions over a grid of instants and bodies, completed into the
   * frame asked for.
   *
   * @param {object} request
   * @param {readonly number[]|Float64Array} request.instants Julian days
   * @param {readonly string[]} request.bodies the bodies, by key
   * @param {string} [request.scale] `ut1` or `tt`; `ut1` by default
   * @param {object} [request.frame] a frame; the canonical one by default
   * @param {boolean} [request.speeds] whether speeds are wanted
   * @param {object} [request.observer] the place a topocentric frame needs
   */
  positions(request) {
    const frame = request.frame ?? this.canonicalFrame();
    const bytes = this.#call(() =>
      this.#inner.positions(
        clean({
          scale: request.scale ?? 'ut1',
          frameBits: native.framePack(clean(frame)),
          speeds: request.speeds ?? true,
          observer: request.observer,
          jds: instants(request.instants, 'instants'),
          bodies: request.bodies,
        }),
      ),
    );
    return new Positions(bytes);
  }

  /**
   * Founds a chart at an instant and a place.
   *
   * Everything but this is the context's settings, so two charts founded
   * under one context are comparable and the settings hash says why. The
   * clock is here because nothing else knows it: a chart's day runs from
   * a local sunrise and its date is a civil date, and a longitude gives
   * local *mean* time rather than a civil offset.
   *
   * @param {object} request
   * @param {number} request.instant the instant, as a Julian day (UTC)
   * @param {object} request.place `{ latitude, longitude, altitude }` in
   *   degrees and metres
   * @param {number} request.utcOffsetSeconds the local clock's offset
   *   from UTC, east positive
   * @param {string} [request.kind] a chart kind; `ChartKind.Natal` by default
   * @returns {Chart}
   */
  found(request) {
    return this.foundMany({
      ...request,
      instants: [finite(request.instant, 'instant')],
    }).at(0);
  }

  /**
   * Founds a chart at each of many instants, at one place, in one
   * crossing.
   *
   * The founder shares the settings and the solar model across the
   * batch, so a hundred instants cost one setup rather than a hundred —
   * which is what a rectification pass wants. A batch of none is an
   * empty result rather than an error.
   *
   * @param {object} request
   * @param {ArrayLike<number>} request.instants the instants, as Julian
   *   days (UTC): one chart each
   * @param {object} request.place `{ latitude, longitude, altitude }` in
   *   degrees and metres
   * @param {number} request.utcOffsetSeconds the local clock's offset
   *   from UTC, east positive
   * @param {string} [request.kind] a chart kind; `ChartKind.Natal` by default
   * @returns {Charts}
   */
  foundMany(request) {
    const place = request.place ?? {};
    const bytes = this.#call(() =>
      this.#inner.chartFound({
        kind: request.kind ?? ChartKind.Natal,
        instants: instants(request.instants, 'instants', { allowEmpty: true }),
        latitudeDeg: finite(place.latitude, 'place.latitude'),
        longitudeDeg: finite(place.longitude, 'place.longitude'),
        altitudeM: finite(place.altitude ?? 0, 'place.altitude'),
        utcOffsetSeconds: finite(request.utcOffsetSeconds, 'utcOffsetSeconds'),
      }),
    );
    return new Charts(bytes);
  }

  /**
   * The almanac of every day in a range, at one place.
   *
   * A **range** rather than a list of dates, because consecutive days
   * share a boundary — day *n*'s next sunrise is day *n+1*'s sunrise —
   * so a month of days costs much less than thirty days computed
   * separately. A range holding more than a year and a day is refused
   * by name.
   *
   * @param {object} request
   * @param {object} request.from the first day, as `date(...)` builds one
   * @param {object} request.to the last day, both ends included
   * @param {object} request.place `{ latitude, longitude, altitude }`
   * @param {number} request.utcOffsetSeconds the local clock's offset
   *   from UTC, east positive
   * @returns {Almanac}
   */
  almanac(request) {
    const place = request.place ?? {};
    const from = request.from ?? {};
    const to = request.to ?? from;
    const bytes = this.#call(() =>
      this.#inner.panchangaDays({
        calendar: from.calendar,
        fromYear: finite(from.year, 'from.year'),
        fromMonth: finite(from.month, 'from.month'),
        fromDay: finite(from.day, 'from.day'),
        toYear: finite(to.year, 'to.year'),
        toMonth: finite(to.month, 'to.month'),
        toDay: finite(to.day, 'to.day'),
        latitudeDeg: finite(place.latitude, 'place.latitude'),
        longitudeDeg: finite(place.longitude, 'place.longitude'),
        altitudeM: finite(place.altitude ?? 0, 'place.altitude'),
        utcOffsetSeconds: finite(request.utcOffsetSeconds, 'utcOffsetSeconds'),
      }),
    );
    return new Almanac(bytes);
  }

  /**
   * The almanac of one day, which is the range of one unwrapped.
   *
   * @param {object} request
   * @param {object} request.date the day, as `date(...)` builds one
   * @param {object} request.place `{ latitude, longitude, altitude }`
   * @param {number} request.utcOffsetSeconds the local clock's offset
   *   from UTC, east positive
   * @returns {AlmanacDay}
   */
  almanacDay(request) {
    return this.almanac({ ...request, from: request.date, to: request.date }).at(0);
  }

  /** Renders a message of the current locale with its parameters. */
  render(key, params) {
    const bytes = this.#call(() =>
      this.#inner.intlRender(key, params === undefined ? undefined : JSON.stringify(params)),
    );
    return new Rendered(bytes);
  }

  /** Whether the current locale or its fallbacks have a message. */
  has(key) {
    return this.#call(() => this.#inner.intlHas(key)) === 1;
  }

  /**
   * Text from one script into another (`deva`, `iast`), for a Sanskrit
   * or Nepali term written in the other.
   */
  transliterate(text, from = 'deva', to = 'iast') {
    return this.#call(() => this.#inner.intlTransliterate(text, from, to));
  }

  /**
   * An entity's forms in the current locale or its fallbacks: its name,
   * its prose form, its transliteration, and the glyph and gender the
   * locale gives it.
   */
  entity(key) {
    return entityForms(this.#call(() => this.#inner.intlEntity(key)));
  }

  /**
   * The typed accessors: every message of the SDK's own locale as a
   * function of its parameters, and every catalogued entity as its forms.
   * A key is spelled once, by the generator, and never by an application.
   *
   * ```js
   * ctx.messages.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 });
   * ctx.messages.entity.graha.SUN().name;
   * ```
   */
  get messages() {
    this.#messages ??= messages({
      render: (key, params) => this.render(key, params).text,
      entity: (key) => this.entity(key),
    });
    return this.#messages;
  }

  /** Loads a `.tpack` or `.tbundle` file into the locale engine. */
  loadPack(bytes) {
    return this.#call(() => this.#inner.intlLoadPack(Buffer.from(bytes)));
  }

  /** The date a fixed day falls on in a calendar. */
  dateOf(calendar, fixed) {
    return this.#call(() => this.#inner.calendarFromFixed(calendar, fixed));
  }

  /** The fixed day of a date. */
  fixedOf(date) {
    return this.#call(() => this.#inner.calendarToFixed(clean(date)));
  }

  /** The same date in another calendar. */
  convert(date, into) {
    return this.#call(() => this.#inner.calendarConvert(clean(date), into));
  }

  /** The weekday of a date, Monday `1` to Sunday `7`. */
  weekdayOf(date) {
    return this.#call(() => this.#inner.calendarWeekday(clean(date)));
  }

  /** The length of a month. */
  monthLength(calendar, year, month) {
    return this.#call(() => this.#inner.calendarMonthLength(calendar, year, month));
  }

  /** Whether a year is a leap year. */
  isLeap(calendar, year) {
    return this.#call(() => this.#inner.calendarIsLeap(calendar, year)) === 1;
  }

  /** A civil date and time in a zone, resolved to an instant with its metadata. */
  resolve(civil, zone) {
    return this.#call(() => this.#inner.timeResolve(clean(civil), clean(zone)));
  }

  /** The civil date and time of an instant in a zone. */
  civilOf(jdUtc, zone, calendar) {
    return this.#call(() =>
      this.#inner.timeCivil(finite(jdUtc, 'jdUtc'), clean(zone), calendar),
    );
  }

  /** Converts an instant between the time scales. */
  convertTime(jd, from, to) {
    return this.#call(() => this.#inner.timeConvert(finite(jd, 'jd'), from, to));
  }

  /** Delta T at a UT1 instant, with what produced it. */
  deltaT(jdUt1) {
    return this.#call(() => this.#inner.timeDeltaT(finite(jdUt1, 'jdUt1')));
  }

  /** The packed id of a catalogue key. */
  keyId(key) {
    return this.#call(() => this.#inner.keyParse(key));
  }

  /** The catalogue key of a packed id. */
  keyName(id) {
    return this.#call(() => this.#inner.keyName(id));
  }

  /**
   * Frees the context's native memory now, rather than when the collector
   * gets to it.
   *
   * A context is finaliser-backed, so forgetting this leaks nothing in the
   * end; but a service that builds one per request and waits for the
   * collector can hold thousands at once (ADR-0007, finding 4). Calling it
   * twice is allowed, and a call on a disposed context is refused with
   * `INVALID_ARG` rather than crashing.
   */
  dispose() {
    this.#disposed = true;
    this.#inner.dispose();
  }

  /**
   * The same, for `using` (explicit resource management), so a context can
   * be scoped:
   *
   * ```js
   * using ctx = new Context({ testProvider: true });
   * ```
   */
  [Symbol.dispose]() {
    this.dispose();
  }
}

/** The ABI the addon implements. */
export const abiVersion = () => native.abiVersion();
/** The SDK's version. */
export const sdkVersion = () => native.sdkVersion();
/** The catalogue's schema version, stamped in every result's provenance. */
export const catalogueVersion = () => native.catalogueVersion();
/** The profile a context uses when none is named. */
export const defaultProfile = () => native.defaultProfile();
/** The SDK's canonical frame. */
export const canonicalFrame = () => native.frameCanonical();
/** The Julian day at the UTC midnight that begins a fixed day. */
export const julianDayOfFixed = (fixed) => native.calendarJdOfFixed(fixed);
/** The fixed day a Julian day falls in, and the fraction elapsed. */
export const fixedOfJulianDay = (jd) => native.calendarFixedOfJd(finite(jd, 'jd'));
/** Packs a frame's fields into the bits a position request carries. */
export const packFrame = (frame) => native.framePack(clean(frame));
/** Reads packed frame bits back into their fields. */
export const unpackFrame = (bits) => native.frameUnpack(bits);

/**
 * A date in a calendar, without naming the fields a call fills in.
 *
 * The era and the era year are what the call resolves them to, and the
 * resolution is `defined`, which is what a date a caller states means.
 * The Dart and Python bindings have the same helper, so the three read
 * alike.
 *
 * @example date(Calendar.Gregorian, 2015, 4, 14)
 */
export const date = (calendar, year, month, day) => ({
  calendar,
  year,
  eraYear: 0,
  month,
  day,
  resolution: 'defined',
  computedMonth: 0,
  computedDay: 0,
});

/**
 * A date at a time of day.
 *
 * @example at(date(Calendar.Gregorian, 1986, 1, 1), { hour: 0, minute: 20 })
 */
export const at = (day, { hour = 0, minute = 0, second = 0, nanos = 0 } = {}) => ({
  date: day,
  time: { hour, minute, second, hasTime: true, nanos },
});

/**
 * A date whose time of day is unknown.
 *
 * Nothing guesses one. Unless the profile sets `time.unknown_time`, a
 * resolution refuses it by name and the hint says what to choose; under
 * `NOON` it resolves with `timeKnown` false and a
 * `time-unknown-fallback` warning, and under `SUNRISE` it needs the
 * place and a solar model.
 *
 * @example whenUnknown(date(Calendar.Gregorian, 1986, 1, 1))
 */
export const whenUnknown = (day) => ({
  date: day,
  time: { hour: 0, minute: 0, second: 0, hasTime: false, nanos: 0 },
});

/** A zone of the embedded database, by its IANA name. */
export const ianaZone = (name) => ({
  kind: 'iana',
  offsetSeconds: 0,
  longitudeDeg: 0,
  zone: name,
});

/** A fixed offset from UTC, in seconds east. */
export const fixedZone = (offsetSeconds) => ({
  kind: 'fixed',
  offsetSeconds,
  longitudeDeg: 0,
});

/**
 * Local mean time at a longitude east of Greenwich, which is what a chart
 * from before the zone existed is cast in.
 */
export const localMeanZone = (longitudeDeg) => ({
  kind: 'local-mean',
  offsetSeconds: 0,
  longitudeDeg,
});

export { decodeIntlRender, decodePositions } from './blob.js';
export { entityForms, messages } from './messages.js';
export * from './catalogue.js';
