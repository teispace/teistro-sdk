/**
 * The Teistro SDK for JavaScript: the layer a consumer uses, over the
 * Node addon or the wasm module alike.
 *
 * HAND-WRITTEN, and thin on purpose. Everything beneath it is generated
 * from the API description: the glue (`native/src/generated.rs`), the
 * types (`catalogue.d.ts`, `types.d.ts`, `blob.d.ts`), the catalogue's
 * tables (`catalogue.js`) and the result-blob decoders (`blob.js`). What
 * this file adds is what a generator cannot know: where the addon is,
 * validation at the door, defaults, errors with their field and hint, and
 * results decoded on first use rather than eagerly. Where the native half
 * lives is the loader's business (`addon.js` here), so nothing in this
 * file is Node's alone.
 */

// The native surface: the napi addon in the Node package, the wasm
// module in the wasm package. `#native` is this package's own import map
// (`package.json` `imports`), so the layer below is one file for both and
// names no Node built-in itself (03-design/wasm-binding.md §3).
import { native, named as wasNamed, platformPackage } from '#native';

import {
  ABI_VERSION,
  AyanaById,
  AyanamshaById,
  BodyById,
  CONTEXT_TEST_PROVIDER,
  CalendarById,
  DayStateById,
  MoonEventById,
  EraById,
  PolarDayPolicyById,
  PolarKindById,
  ResolutionById,
  SunriseById,
  ChartKind,
  ChartLayoutById,
  DayPartById,
  ChartKindById,
  ChoghadiyaById,
  DirectionById,
  GhatiReckoningById,
  GrahaById,
  HouseSystemById,
  KaalaById,
  KaranaById,
  LunarMonthById,
  MasaById,
  MonthKindById,
  MuhurtaYogaById,
  NakshatraById,
  PakshaById,
  PanchakaById,
  AvasthaBaladiById,
  AvasthaDeeptadiById,
  AvasthaJagradadiById,
  AvasthaLajjitadiById,
  AvasthaSayanadiById,
  AvasthaCheshtaById,
  BurningById,
  DignityById,
  RelationshipById,
  PointById,
  QuadrantById,
  RashiById,
  VarsheshaChosenById,
  TajikaDrishtiById,
  TajikaYogaById,
  YearYogaById,
  SahamById,
  SahamStrongById,
  SahamWeakById,
  HarshaGradeById,
  TajikaRelationById,
  AfflictionById,
  StrengthById,
  BalanceById,
  EkadhipatyaById,
  ShodhanaById,
  VaiseshikamsaById,
  DashaPhaseById,
  NatureById,
  VimshopakaScoringById,
  DashaSystemById,
  VargaById,
  SDK_VERSION,
  TithiById,
  TimeScaleById,
  VaraById,
  YogaById,
} from './catalogue.js';
import { decodeCharts, decodeIntlRender, decodePanchanga, decodePositions } from './blob.js';
import { entityForms, messages } from './messages.js';
import { decodeProvenance, decodeStep } from './records.js';

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

/**
 * The npm package that carries this host's native half: the platform's
 * prebuilt addon, or the wasm module's own package.
 */
export { platformPackage };

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
 * @param {object|null} context the addon handle, for `lastError`; null
 *   for a call that makes a handle, whose refusal carries its own record
 * @param {Function} call the call to make
 * @param {{error: unknown}} [thrown] where this context's provider leaves
 *   what it threw
 */
function guarded(context, call, thrown) {
  if (thrown) thrown.error = undefined;
  try {
    return call();
  } catch (cause) {
    // A context keeps its last refusal; a call that makes a handle has no
    // context to keep it on, so the addon throws it with the whole record
    // attached instead (`ffi-abi-and-api-description.md` §6.1).
    // The record comes on the error the addon threw for the call that
    // failed, and from nowhere else: reading the context's last error for
    // any exception would report an argument this layer refused as
    // whatever the library refused last. A provider's own throw is the one
    // case with no such error, and there the call did reach the library.
    const own = thrown?.error;
    const record = cause?.lastError ?? (own !== undefined ? context?.lastError?.() : undefined);
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
    const fromProvider = record.status === 'PROVIDER' && cause?.message;
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
  // Nothing is checked here. The coverage span, a topocentric frame
  // without an observer, a body the provider never declared and an instant
  // that is not a number are all the port's, on the SDK's side of the
  // boundary: an instant outside the span comes back as an `OUT_OF_RANGE`
  // cell and never reaches `positions`, exactly as it would from a native
  // provider, and the rest are refused as a `TeistroError` that names what
  // is missing. The Dart and Python adapters check nothing either.
  const answering = (request) => {
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

/**
 * A decoded result the library stamped with a provenance envelope: the
 * positions, a batch of charts, an almanac. A rendered message is decoded
 * too and carries none, so the envelope's getters live here and not on
 * `Decoded`.
 */
class Stamped extends Decoded {
  #provenance = null;

  /**
   * The provenance envelope: what computed these, and under what. Its
   * `contentHash` is the whole batch's.
   */
  get provenance() {
    this.#provenance ??= Object.freeze(decodeProvenance(JSON.parse(this.provenanceJson)));
    return this.#provenance;
  }

  /**
   * The provenance envelope as the canonical JSON the library stamped:
   * the bytes to store beside the result, byte-identical in every binding.
   */
  get provenanceJson() {
    return this.decoded.provenanceJson;
  }
}

/**
 * The provenance of one member of a batch handed out alone: the batch's,
 * with that member's own `contentHash` from the blob's `content_hashes`
 * section — so a chart founded alone carries the hash of its own value and
 * not of a list of one.
 */
function memberProvenance(batch, index) {
  const own = batch.decoded.contentHashes.slice(64 * index, 64 * index + 64);
  return Object.freeze({ ...batch.provenance, contentHash: own });
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

/** The convention column's value for a custom sunrise altitude. */
const CUSTOM_SUNRISE = 0xff;

/**
 * One row of a `day` section -- a chart's or an almanac's, which share it --
 * read into the record both layers hand back: the civil date as
 * `calendar.convert` spells one, so it can be handed straight back to it,
 * every id named, and the day's state and convention as the shapes they
 * are rather than as the columns they cross in.
 */
function localDay(section, index) {
  const r = row(section, index);
  const era = EraById.get(r.era);
  const custom = r.conventionKind === CUSTOM_SUNRISE;
  return {
    date: {
      calendar: CalendarById.get(r.calendar),
      ...(era === undefined ? {} : { era }),
      year: r.year,
      eraYear: r.eraYear,
      month: r.month,
      day: r.dayOfMonth,
      resolution: ResolutionById.get(r.resolution),
      computedMonth: r.computedMonth,
      computedDay: r.computedDay,
    },
    vara: VaraById.get(r.vara) ?? 'unknown',
    sunrise: r.sunrise,
    sunset: r.sunset,
    nextSunrise: r.nextSunrise,
    polar:
      DayStateById.get(r.stateKind) === 'POLAR'
        ? {
            kind: PolarKindById.get(r.statePolarKind) ?? 'unknown',
            policy: PolarDayPolicyById.get(r.statePolarPolicy) ?? 'unknown',
          }
        : null,
    convention: custom ? null : (SunriseById.get(r.conventionKind) ?? 'unknown'),
    customAltitudeDeg: custom ? r.conventionValue : null,
    // No air has a pressure of zero, so a zero says the convention named
    // none.
    air:
      r.airPressureHpa > 0 ? { pressureHpa: r.airPressureHpa, temperatureC: r.airTemperatureC } : null,
  };
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
export class Charts extends Stamped {
  /** The full key of each dasha system a context registered, by its id. */
  #dashaNames;

  /**
   * @param {Uint8Array} bytes the blob the library returned
   * @param {Map<number, string>} [dashaNames] the full key of each dasha
   *   system the founding context registered, by its id; a system missing
   *   from it reads as `'unknown'`
   */
  constructor(bytes, dashaNames = new Map()) {
    super(bytes, decodeCharts);
    this.#dashaNames = dashaNames;
  }

  /** The full key of a registered dasha system's id, when this batch knows it. */
  dashaName(id) {
    return this.#dashaNames.get(id);
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

  /** How many divisional charts each chart of the batch holds. */
  get vargaCount() {
    return this.decoded.vargaCount;
  }

  /** The drishti table every aspect was read under; empty if none were asked for. */
  get drishtiTable() {
    return this.decoded.drishtiTable;
  }

  /** The steps the SDK applied, in order, each `name:Implementation`. */
  get steps() {
    return JSON.parse(this.decoded.steps);
  }

  /** The solar model that reckoned the days, as it describes itself. */
  get model() {
    return this.decoded.model;
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
   * The catalogued ayanamsha this chart was read under, or `null` when
   * none was applied -- a tropical chart -- or the settings defined their
   * own, which `ayanamshaCustom` says. One for the batch, since the frame
   * is the request's.
   */
  get ayanamsha() {
    const { ayanamshaKind, ayanamsha } = this.#batch.decoded;
    return ayanamshaKind === 1 ? (AyanamshaById.get(ayanamsha) ?? 'unknown') : null;
  }

  /** Whether the ayanamsha is one the settings define rather than a catalogued one. */
  get ayanamshaCustom() {
    return this.#batch.decoded.ayanamshaKind === 2;
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
    return localDay(this.#batch.decoded.day, this.#index);
  }

  /** Where in its day the moment falls, and which hora holds it. */
  get timing() {
    const timing = row(this.#batch.decoded.timing, this.#index);
    return {
      ...timing,
      ghatiReckoning: GhatiReckoningById.get(timing.ghatiReckoning) ?? 'unknown',
      horaLord: GrahaById.get(timing.horaLord) ?? 'unknown',
    };
  }

  /**
   * The divisional charts asked for, in the order they were asked.
   *
   * Empty unless `vargas` named some: a caller who wants a birth chart
   * does not pay for twenty-one of them
   * (`03-design/chart-reading.md` §4).
   *
   * Each is `{ varga, lagna, grahas }`, where the lagna is a placement
   * `{ rashi, part, sign }` — the sign it stands in, which part of it, and
   * the sign the divisional chart puts it in — and each graha is
   * `{ graha, at }` with `at` the same placement: the shape the Rust, Dart
   * and Python surfaces give it. `at.sign === at.rashi` is the body keeping
   * its sign, which in the navamsha is **vargottama**.
   */
  get vargas() {
    const d = this.#batch.decoded;
    const count = d.vargaCount;
    const grahaCount = d.grahaCount;
    const base = this.#index * count;
    return Array.from({ length: count }, (_, v) => {
      const row = base + v;
      const from = row * grahaCount;
      return {
        varga: VargaById.get(d.vargas.varga[row]) ?? 'unknown',
        lagna: {
          rashi: RashiById.get(d.vargas.lagnaRashi[row]) ?? 'unknown',
          part: d.vargas.lagnaPart[row],
          sign: RashiById.get(d.vargas.lagnaSign[row]) ?? 'unknown',
        },
        grahas: Array.from({ length: grahaCount }, (_, j) => ({
          graha: GrahaById.get(d.grahas.graha[this.#index * grahaCount + j]) ?? 'unknown',
          at: {
            rashi: RashiById.get(d.vargaGrahas.rashi[from + j]) ?? 'unknown',
            part: d.vargaGrahas.part[from + j],
            sign: RashiById.get(d.vargaGrahas.sign[from + j]) ?? 'unknown',
          },
        })),
      };
    });
  }

  /**
   * The charts drawn in the layouts asked for, in the order asked; empty
   * unless `drawings` named some (`03-design/chart-geometry.md`).
   *
   * Each is `{ layout, varga, cells, frame, marks }`. A cell is
   * `{ outline, sign, house, lagna, ring, label, anchor, bodies }`: an
   * outline in the unit square (y downwards), the sign **and** the house it
   * shows, and the bodies standing in it as catalogue keys. `marks` places
   * each body at its own degree on a wheel and is empty for a grid. The
   * shape is the Rust, Dart and Python surfaces' own.
   */
  get drawings() {
    return drawingsOf(this.#batch)[this.#index] ?? [];
  }

  /**
   * What this chart answers by rule, as the SDK writes it: `present`, each
   * `{ rule, result }` with the rule by key, and `houses` and `longevity` when
   * the request asked; `null` unless the request named rules
   * (`03-design/rules-at-the-boundary.md`).
   */
  get rules() {
    return rulesOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * What this chart has to say, as the composers wrote it: a key per
   * composer the request asked for, each an array of `{ key, params }`
   * holding no words at all — and only those the request named.
   * An item's `params` are the record `intl.render` takes, so
   * `sdk.intl.render(item.key, item.params)` says it in the context's
   * locale — and the same plan says it in any other
   * (`03-design/plans-at-the-boundary.md`). `null` unless the request named
   * a composer.
   */
  get plans() {
    return plansOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The drishti this chart casts, strongest first among those a body
   * casts; empty unless `aspects: true` asked for them.
   *
   * Each is `{ from, to, houses, strength, fromEdge, toEdge }`, where an
   * edge is `{ signDeg, nakshatraDeg, padaDeg }` — how near that end
   * stands to a boundary, which is what an ayanamsha that moved would
   * change.
   *
   * The section is **ragged**: a chart's relations depend on where the
   * bodies stand rather than on how many there are, so two charts of the
   * same nine grahas hold different numbers of them.
   */
  /**
   * The annual charts' instants: the Sun's returns to where it stood at
   * birth, `1` opening the first year of life
   * (`03-design/annual-chart.md`). Empty unless the request asked with
   * `varsha`.
   *
   * The section is **ragged** for a reason of its own: the request
   * settles how many returns are *wanted* and the ephemeris settles how
   * many there *are*, so a chart late enough to run past it answers
   * fewer rather than refusing.
   *
   * The place is yours: a return is an instant, and casting it for a
   * birthplace or for a residence is a choice the schools differ on, so
   * the boundary answers the instant and `found` is how you make the
   * chart.
   */
  get praveshas() {
    const d = this.#batch.decoded;
    const counts = d.cast.praveshaCount;
    let from = 0;
    for (let i = 0; i < this.#index; i += 1) from += counts[i];
    const count = counts[this.#index] ?? 0;
    return Array.from({ length: count }, (_, k) => ({
      year: d.praveshas.year[from + k],
      instant: d.praveshas.jd[from + k],
      muntha: {
        sign: RashiById.get(d.praveshas.munthaSign[from + k]) ?? 'unknown',
        lord: GrahaById.get(d.praveshas.munthaLord[from + k]) ?? 'unknown',
        longitudeDeg: d.praveshas.munthaDeg[from + k],
      },
      annual: annualOf(d, from + k),
    }));
  }

  /**
   * The birth chart's own sahams, each with its strength clause by clause
   * — which has no year lord — in the order `varsha.sahams` named them;
   * empty unless it asked (`03-design/tajika-saham-strength.md`). The
   * source reads a year's sahams beside these.
   */
  get sahams() {
    const d = this.#batch.decoded;
    const from = startsOf(d.cast.natalSahamCount)[this.#index];
    const count = d.cast.natalSahamCount[this.#index] ?? 0;
    return Array.from({ length: count }, (_, k) =>
      sahamAt(d.natalSahams, d.natalSahamSeven, from + k),
    );
  }

  get aspects() {
    const d = this.#batch.decoded;
    const counts = d.cast.aspectCount;
    let from = 0;
    for (let i = 0; i < this.#index; i += 1) from += counts[i];
    const count = counts[this.#index] ?? 0;
    return Array.from({ length: count }, (_, k) => {
      const i = from + k;
      return {
        from: GrahaById.get(d.aspects.from[i]) ?? 'unknown',
        to: GrahaById.get(d.aspects.to[i]) ?? 'unknown',
        houses: d.aspects.houses[i],
        strength: StrengthById.get(d.aspects.strength[i]) ?? 'unknown',
        fromEdge: {
          signDeg: d.aspects.fromSignDeg[i],
          nakshatraDeg: d.aspects.fromNakshatraDeg[i],
          padaDeg: d.aspects.fromPadaDeg[i],
        },
        toEdge: {
          signDeg: d.aspects.toSignDeg[i],
          nakshatraDeg: d.aspects.toNakshatraDeg[i],
          padaDeg: d.aspects.toPadaDeg[i],
        },
      };
    });
  }

  /**
   * The dashas asked for (`dashas: [DashaSystem.Vimshottari]`), each with its
   * balance at birth and its periods to the settings' depth, in the order
   * asked; empty unless some were.
   */
  get dashas() {
    return dashasOf(this.#batch)[this.#index] ?? [];
  }

  /**
   * The Ashtakavarga (`ashtakavarga: true`): each graha's bindus by sign from
   * Aries, the sarvashtakavarga, and their reductions and pindas under the
   * settings' reading; `null` unless asked for.
   */
  get ashtakavarga() {
    return ashtakavargasOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The Vaiseshikamsa (`vaiseshikamsa: true`): each graha's count of good
   * vargas and the name it earns in each scheme, and whether it is impaired;
   * `null` unless asked for.
   */
  get vaiseshikamsa() {
    return vaiseshikamsasOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The dasha phala (`dashaPhala: true`): each graha's Subhanka in the seven
   * vargas, whether its rasi place is auspicious, where in its dasha its
   * effects come and whether its placement makes the dasha favourable or
   * unfavourable; `null` unless asked for.
   */
  get dashaPhala() {
    return dashaPhalasOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The Vimshopaka (`vimshopaka: true`): each graha's strength out of 20
   * across the divisional charts under the four schemes, each varga scored
   * under the settings' reading; `null` unless asked for.
   */
  get vimshopaka() {
    return vimshopakasOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The Shadbala (`shadbala: true`): each graha's six strengths in virupas,
   * the Sthana and Kaala by component, their sum in rupas and whether it
   * reaches the requirement, under the context's `strength.*` settings;
   * `null` unless asked for.
   */
  get shadbala() {
    return shadbalasOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The Bhava bala (`bhavaBala: true`): each bhava's lord's Shadbala, its Dig
   * and drishti balas, its special rules and their sum in virupas, under the
   * context's `strength.bhava_*` settings; `null` unless asked for.
   */
  get bhavaBala() {
    return bhavaBalasOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The derived points — the upagrahas and the special lagnas — or an
   * empty list unless `points: true` asked for them.
   *
   * Each is `{ point, longitudeDeg, sign, boundaries }`. Gulika and
   * Mandi are Saturn's eighth of the day's arc, so they are the two a
   * chart with no arc to divide cannot have — which is why the section
   * is ragged.
   */
  get points() {
    const d = this.#batch.decoded;
    const counts = d.cast.pointCount;
    let from = 0;
    for (let i = 0; i < this.#index; i += 1) from += counts[i];
    const count = counts[this.#index] ?? 0;
    return Array.from({ length: count }, (_, k) => {
      const i = from + k;
      return {
        point: PointById.get(d.points.point[i]) ?? 'unknown',
        longitudeDeg: d.points.longitudeDeg[i],
        sign: RashiById.get(d.points.sign[i]) ?? 'unknown',
        boundaries: {
          signDeg: d.points.signDeg[i],
          nakshatraDeg: d.points.nakshatraDeg[i],
          padaDeg: d.points.padaDeg[i],
        },
      };
    });
  }

  /**
   * The twelve bhavas as the houses service reads them, or an empty
   * list unless `houses: true` asked for them.
   *
   * Each is `{ number, sign, lord, quadrant }`, where `sign` is the sign
   * the bhava's **middle** falls in — which under an unequal division is
   * not the sign it begins in. The madhya and the sandhi are on
   * `bhavaBounds`; this is what only the houses service computes.
   */
  get bhavas() {
    const d = this.#batch.decoded;
    const base = this.#index * 12;
    if (d.bhavas.length === 0) return [];
    return Array.from({ length: 12 }, (_, j) => ({
      number: j + 1,
      sign: RashiById.get(d.bhavas.sign[base + j]) ?? 'unknown',
      lord: GrahaById.get(d.bhavas.lord[base + j]) ?? 'unknown',
      quadrant: QuadrantById.get(d.bhavas.quadrant[base + j]) ?? 'unknown',
    }));
  }

  /**
   * What each graha **is**, as opposed to where it is — or an empty list
   * unless `state: true` asked for it.
   *
   * One per graha, in the same order as `grahas`. The three lajjitadi
   * lists cross as bit sets and are handed back as arrays of keys,
   * because a set over six members is a set and not three ragged lists.
   *
   * The motion is not here: `grahas[j].retrograde` already says it.
   */
  get states() {
    const d = this.#batch.decoded;
    if (d.states.length === 0) return [];
    const count = d.grahaCount;
    const base = this.#index * count;
    // `id >= 0` skips the generated `UNKNOWN = -1` sentinel every
    // catalogue map carries, and `Array.from` because `Map.entries()`
    // is an iterator: an iterator's `map` answers an iterator, and a
    // caller wants a list.
    const set = (bits) =>
      Array.from(AvasthaLajjitadiById.entries())
        .filter(([id]) => id >= 0 && (bits & (1 << id)) !== 0)
        .map(([, key]) => key);
    return Array.from({ length: count }, (_, j) => {
      const i = base + j;
      const s = d.states;
      return {
        graha: GrahaById.get(s.graha[i]) ?? 'unknown',
        sign: RashiById.get(s.sign[i]) ?? 'unknown',
        house: s.house[i],
        dignity: DignityById.get(s.dignity[i]) ?? 'unknown',
        friendship: {
          natural: RelationshipById.get(s.natural[i]) ?? 'unknown',
          temporary: RelationshipById.get(s.temporary[i]) ?? 'unknown',
          compound: RelationshipById.get(s.compound[i]) ?? 'unknown',
          dispositor: s.hasDispositor[i] ? (GrahaById.get(s.dispositor[i]) ?? 'unknown') : null,
        },
        combustion: {
          burning: BurningById.get(s.burning[i]) ?? 'unknown',
          fromSunDeg: s.hasFromSun[i] ? s.fromSunDeg[i] : null,
          orbDeg: s.hasOrbs[i] ? s.orbDeg[i] : null,
          deepOrbDeg: s.hasDeepOrb[i] ? s.deepOrbDeg[i] : null,
        },
        age: AvasthaBaladiById.get(s.age[i]) ?? 'unknown',
        wakefulness: AvasthaJagradadiById.get(s.wakefulness[i]) ?? 'unknown',
        deeptadi: s.hasDeeptadi[i] ? (AvasthaDeeptadiById.get(s.deeptadi[i]) ?? 'unknown') : null,
        lajjitadi: {
          holding: set(s.lajjitadiHolding[i]),
          ruledOut: set(s.lajjitadiRuledOut[i]),
          undecided: set(s.lajjitadiUndecided[i]),
        },
        war: s.hasWar[i]
          ? {
              opponent: GrahaById.get(s.warOpponent[i]) ?? 'unknown',
              isWinner: s.warWon[i] !== 0,
              apartDeg: s.warApartDeg[i],
            }
          : null,
        sayanadi: s.hasSayanadi[i]
          ? {
              avastha: AvasthaSayanadiById.get(s.sayanadi[i]) ?? 'unknown',
              cheshtas: [s.cheshta1, s.cheshta2, s.cheshta3, s.cheshta4, s.cheshta5].map(
                (column) => AvasthaCheshtaById.get(column[i]) ?? 'unknown',
              ),
            }
          : null,
        boundaries: {
          signDeg: s.signDeg[i],
          nakshatraDeg: s.nakshatraDeg[i],
          padaDeg: s.padaDeg[i],
        },
      };
    });
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

  /** The steps the SDK applied, in order, each `name:IMPLEMENTATION`. */
  get steps() {
    return this.#batch.steps;
  }

  /**
   * What computed this chart, and under what: the batch's provenance
   * stamped with this chart's own `contentHash`.
   */
  get provenance() {
    return memberProvenance(this.#batch, this.#index);
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
export class Almanac extends Stamped {
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
    return localDay(this.#batch.decoded.day, this.#index);
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
      // Whether this month is the intercalary one. The *name* above
      // needs no case for it — an adhika month and the nija month after
      // it take the same name — so this is the mark beside the name.
      kind: MonthKindById.get(d.monthKind[i]) ?? 'unknown',
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
      kind: MoonEventById.get(c.kind[i]) ?? 'unknown',
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
        kind: c.becauseKind[i] === 0 ? 'VARA_NAKSHATRA' : 'VARA_TITHI_NAKSHATRA',
        vara: VaraById.get(c.becauseVara[i]) ?? 'unknown',
        // A `VARA_NAKSHATRA` cause has no tithi, and the blob leaves the
        // column at nought rather than at a tithi that did not make it.
        tithi: c.becauseKind[i] === 0 ? null : (TithiById.get(c.becauseTithi[i]) ?? 'unknown'),
        nakshatra: NakshatraById.get(c.becauseNakshatra[i]) ?? 'unknown',
      },
    }));
  }

  /**
   * What computed this day, and under what: the batch's provenance
   * stamped with this day's own `contentHash`.
   */
  get provenance() {
    return memberProvenance(this.#batch, this.#index);
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

export class Positions extends Stamped {
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

  /** The bodies as the ids the blob carries, without a copy. */
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
    return JSON.parse(this.decoded.steps).map(decodeStep);
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

  /**
   * The message in parts, its markup kept: what a rich renderer walks.
   *
   * The boundary sends nothing when the message has no markup, because
   * the parts would then be the text written twice; the one part is
   * made here rather than carried.
   */
  get parts() {
    const written = JSON.parse(this.decoded.parts || '[]');
    return written.length ? written : [{ type: 'text', value: this.text }];
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
/**
 * The engine's own operations, reached by the names it gives them.
 *
 * The SDK names eight operations. An engine names far more, and what it
 * names beyond them is reached through here rather than around the SDK.
 * **Nothing in this class is a list of an engine's operations**: it asks
 * the engine what it offers and calls what the answer names, so a
 * function the engine gains after this package ships is callable without
 * a new release of it.
 *
 * Use the engine's own spelling, because that is what its manifest says
 * and what its documentation calls it:
 *
 * ```js
 * const engine = context.ephemeris;
 * const answer = engine.tp_echo({ value: 6 });
 * ```
 *
 * A member is looked up in the manifest, so a name the engine does not
 * have is `undefined` rather than a function that fails when called, and
 * `Object.keys` lists what the engine offers.
 */
export class Engine {
  #reach;
  #manifest = null;

  constructor(reach) {
    this.#reach = reach;
    Object.freeze(this);
  }

  /** The manifest as the engine wrote it. */
  get manifestJson() {
    return this.#reach((inner) => inner.ephemerisManifest());
  }

  /**
   * The manifest, parsed and remembered. Read once per engine: it
   * changes when the engine does, and an engine does not change under a
   * live context.
   */
  get manifest() {
    this.#manifest ??= JSON.parse(this.manifestJson);
    return this.#manifest;
  }

  /** Every operation the engine offers, in its own order. */
  get names() {
    return (this.manifest.functions ?? []).map((f) => f.name);
  }

  /**
   * What the manifest says about one operation, or `undefined`. Its
   * parameters carry the role of each, which says which a caller
   * supplies and which the engine fills.
   */
  signature(name) {
    return (this.manifest.functions ?? []).find((f) => f.name === name);
  }

  /**
   * Calls an operation by name, with its parameters as an object keyed
   * by the names the manifest gives. Answers with the engine's own
   * answer, parsed.
   */
  call(name, argumentsObject = {}) {
    return JSON.parse(this.callJson(name, JSON.stringify(argumentsObject)));
  }

  /**
   * Calls an operation with arguments already written as JSON, and
   * answers with the engine's own JSON: the form to use when the answer
   * is being handed on rather than read.
   */
  callJson(name, argumentsJson) {
    return this.#reach((inner) => inner.ephemerisCall(name, argumentsJson));
  }
}

/**
 * What every area is.
 *
 * A **value**, which is what makes the areas worth having rather than
 * merely tidy: `const { calendar } = sdk` keeps working, and an area can
 * be passed to something that only needs that much of the SDK
 * (`03-design/surface-areas.md`).
 *
 * A class with prototype methods rather than a frozen object literal of
 * arrow functions, which is what the design page first said: a literal
 * of six closures is seven allocations per context and this is one, and
 * the methods are shared by every context rather than rebuilt for each.
 * The instance is frozen, so an area cannot be added to from outside.
 *
 * `#reach` is the only thing an area holds. It runs a call on the
 * addon's context with the disposed check and the provider's own thrown
 * value already applied, so no area touches the handle.
 */
/**
 * Each area's way to the boundary, kept beside the area rather than on it:
 * an area's members are exactly the operations `index.d.ts` declares, and
 * a helper on the instance would be a public member nothing declares.
 */
const reaches = new WeakMap();

/** Calls the boundary through the area's context. */
const run = (area, work) => reaches.get(area)(work);

class Area {
  constructor(reach) {
    reaches.set(this, reach);
    // Frozen, so a consumer cannot add to an area; a subclass keeps its own
    // state (`IntlArea`'s memo) in private fields, which are not properties.
    Object.freeze(this);
  }
}

/** `sdk.calendar` — the calendars, and the fixed day they share. */
export class CalendarArea extends Area {
  /** The date a fixed day falls on in a calendar. */
  dateOf(calendar, fixed) {
    return run(this, (inner) => inner.calendarFromFixed(calendar, fixed));
  }

  /** The fixed day of a date. */
  fixedOf(date) {
    return run(this, (inner) => inner.calendarToFixed(clean(date)));
  }

  /** The same date in another calendar. */
  convert(date, into) {
    return run(this, (inner) => inner.calendarConvert(clean(date), into));
  }

  /** The weekday of a date, Monday `1` to Sunday `7`. */
  weekdayOf(date) {
    return run(this, (inner) => inner.calendarWeekday(clean(date)));
  }

  /** The length of a month. */
  monthLength(calendar, year, month) {
    return run(this, (inner) => inner.calendarMonthLength(calendar, year, month));
  }

  /** Whether a year is a leap year. */
  isLeap(calendar, year) {
    return run(this, (inner) => inner.calendarIsLeap(calendar, year)) === 1;
  }
}

/** `sdk.time` — the scales, the zones and what separates them. */
export class TimeArea extends Area {
  /** A civil date and time in a zone, resolved to an instant with its metadata. */
  resolve(civil, zone) {
    return run(this, (inner) => inner.timeResolve(clean(civil), clean(zone)));
  }

  /** The civil date and time of an instant in a zone. */
  civilOf(jdUtc, zone, calendar) {
    return run(this, (inner) => inner.timeCivil(finite(jdUtc, 'jdUtc'), clean(zone), calendar));
  }

  /**
   * Converts an instant between the time scales.
   *
   * `convertTime` on the flat surface, because the calendar had taken
   * `convert`. The area carries the word now.
   */
  convert(jd, from, to) {
    return run(this, (inner) => inner.timeConvert(finite(jd, 'jd'), from, to));
  }

  /** Delta T at a UT1 instant, with what produced it. */
  deltaT(jdUt1) {
    return run(this, (inner) => inner.timeDeltaT(finite(jdUt1, 'jdUt1')));
  }
}

/** `sdk.intl` — the locale, its messages and the scripts they are in. */
export class IntlArea extends Area {
  #messages = null;

  /** The locale every render resolves from. */
  get locale() {
    return run(this, (inner) => inner.intlLocale());
  }

  set locale(tag) {
    run(this, (inner) => inner.intlSetLocale(tag));
  }

  /** Renders a message of the current locale with its parameters. */
  render(key, params) {
    const bytes = run(this, (inner) =>
      inner.intlRender(key, params === undefined ? undefined : JSON.stringify(params)),
    );
    return new Rendered(bytes);
  }

  /** Whether the current locale or its fallbacks have a message. */
  has(key) {
    return run(this, (inner) => inner.intlHas(key)) === 1;
  }

  /**
   * Text from one script into another (`deva`, `iast`), for a Sanskrit
   * or Nepali term written in the other.
   */
  transliterate(text, from = 'deva', to = 'iast') {
    return run(this, (inner) => inner.intlTransliterate(text, from, to));
  }

  /**
   * An entity's forms in the current locale or its fallbacks: its name,
   * its prose form, its transliteration, and the glyph and gender the
   * locale gives it.
   */
  entity(key) {
    return entityForms(run(this, (inner) => inner.intlEntity(key)));
  }

  /**
   * The typed accessors: every message of the SDK's own locale as a
   * function of its parameters, and every catalogued entity as its forms.
   * A key is spelled once, by the generator, and never by an application.
   *
   * ```js
   * sdk.intl.messages.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 });
   * sdk.intl.messages.entity.graha.SUN().name;
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
    const pack = bytesOf(bytes, 'bytes');
    return run(this, (inner) => inner.intlLoadPack(pack));
  }
}

/** `sdk.keys` — the catalogue's keys and their packed ids. */
export class KeysArea extends Area {
  /** The packed id of a catalogue key. */
  id(key) {
    return run(this, (inner) => inner.keyParse(key));
  }

  /** The catalogue key of a packed id. */
  name(id) {
    return run(this, (inner) => inner.keyName(id));
  }
}

/** `sdk.frame` — the coordinate conventions a request is expressed in. */
export class FrameArea extends Area {
  /** The SDK's canonical frame: apparent geocentric ecliptic of date, tropical. */
  canonical() {
    return run(this, () => native.frameCanonical());
  }

  /** Packs a frame's fields into the bits a position request carries. */
  pack(frame) {
    return run(this, () => native.framePack(clean(frame)));
  }

  /** The frame a packed set of bits describes. */
  unpack(bits) {
    return run(this, () => native.frameUnpack(bits));
  }
}

/** `sdk.chart` — a chart founded at an instant and a place. */
export class ChartArea extends Area {
  /** The member id of each layout the context registered, by its full key. */
  #registered;
  /** The member id of each dasha system the context registered, by its full key. */
  #dashas;
  /** The same turned round, for a batch to name them by. */
  #dashaNames;

  constructor(reach, registered, dashas) {
    super(reach);
    this.#registered = registered;
    this.#dashas = dashas;
    this.#dashaNames = new Map(Array.from(dashas, ([key, id]) => [id, key]));
  }

  /**
   * A layout this context can draw in, shipped or registered, as its row:
   * the record a context's `layouts` option takes. Copy a shipped row, give
   * it a key of its own, change what differs and register it
   * (`03-design/chart-geometry.md` §7f).
   *
   * @param {string} key the layout's key, bare (`NORTH_INDIAN`) or full
   * @returns {object} the row, a fresh object to change
   */
  layout(key) {
    return JSON.parse(run(this, (inner) => inner.chartLayoutRow(key)));
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
   * @param {ReadonlyArray<string>} [request.vargas] the divisional
   *   charts to compute, as `Varga` keys, in the order to answer them;
   *   none by default, because a caller who wants a birth chart should
   *   not pay for twenty-one of them
   * @param {boolean} [request.aspects] whether to compute the drishti —
   *   which body looks at which, and how strongly; false by default
   * @param {boolean} [request.points] whether to compute the derived
   *   points — the upagrahas and the special lagnas; false by default
   * @param {boolean} [request.houses] whether to compute the houses
   *   service — each bhava's sign, its lord and its quadrant; false by
   *   default
   * @param {ReadonlyArray<{layout: string, varga: string}>} [request.drawings]
   *   the charts to draw, each a `ChartLayout` and a `Varga` (`Varga.D1` for
   *   the founded chart), in the order to answer them; none by default
   * @param {boolean} [request.state] whether to compute what each graha
   *   *is* — its dignity, its friendships, what the Sun does to it, its
   *   avasthas and any war it is in; false by default
   * @returns {Charts}
   */
  foundMany(request) {
    const place = request.place ?? {};
    const bytes = run(this, (inner) =>
      inner.chartFound({
        kind: request.kind ?? ChartKind.Natal,
        instants: instants(request.instants, 'instants', { allowEmpty: true }),
        latitudeDeg: finite(place.latitude, 'place.latitude'),
        longitudeDeg: finite(place.longitude, 'place.longitude'),
        altitudeM: finite(place.altitude ?? 0, 'place.altitude'),
        utcOffsetSeconds: finite(request.utcOffsetSeconds, 'utcOffsetSeconds'),
        // The sections beside the foundation, which the SDK takes as a
        // bit set and nothing here writes as one
        // (`03-design/chart-reading.md` §5): a named option each, and
        // one more as each crosses.
        sections:
          (request.aspects === true ? SECTION_ASPECTS : 0) |
          (request.points === true ? SECTION_POINTS : 0) |
          (request.houses === true ? SECTION_HOUSES : 0) |
          (request.ashtakavarga === true ? SECTION_ASHTAKAVARGA : 0) |
          (request.vimshopaka === true ? SECTION_VIMSHOPAKA : 0) |
          (request.vaiseshikamsa === true ? SECTION_VAISESHIKAMSA : 0) |
          (request.shadbala === true ? SECTION_SHADBALA : 0) |
          (request.bhavaBala === true ? SECTION_BHAVA_BALA : 0) |
          (request.dashaPhala === true ? SECTION_DASHA_PHALA : 0) |
          (request.state === true ? SECTION_STATE : 0),
        vargas: catalogueKeys(request.vargas, 'vargas', 'Varga'),
        dashas: dashaIds(request.dashas, this.#dashas),
        drawings: drawingBits(request.drawings, this.#registered),
        themeJson: themeJson(request.theme),
        rulesJson: rulesJson(request.rules),
        interpretJson: interpretJson(request.interpret),
        varshaJson: varshaJson(request.varsha),
      }),
    );
    return new Charts(bytes, this.#dashaNames);
  }
}

/** Each batch's dashas, decoded once however many charts read them. */
const DASHAS = new WeakMap();

/**
 * Every chart's dashas in a batch: `dashas` holds a row a chart a system and
 * `dasha_periods` each row's periods, ragged by `period_count`
 * (`03-design/dasha-kernels.md`).
 *
 * @param {Charts} batch
 * @returns {object[][]}
 */
function dashasOf(batch) {
  let decoded = DASHAS.get(batch);
  if (decoded === undefined) {
    const d = batch.decoded;
    const per = d.dashaCount;
    const charts = per === 0 ? 0 : d.dashas.length / per;
    let start = 0;
    decoded = Array.from({ length: charts }, (_, chart) =>
      Array.from({ length: per }, (_, j) => {
        const row = chart * per + j;
        const count = d.dashas.periodCount[row];
        const dasha = dashaFrom(batch, row, start, count);
        start += count;
        return dasha;
      }),
    );
    DASHAS.set(batch, decoded);
  }
  return decoded;
}

/**
 * A dasha's periods from a period section — the births' `dasha_periods` or
 * the years' `year_dasha_periods`, which share a layout — so a period is
 * decoded in one place. A period's path is its index below the nearest
 * earlier period one level up, so it is rebuilt by truncating the path to
 * the level before it.
 *
 * @param {object} cols the decoded period section
 * @param {number} start its first row
 * @param {number} count how many rows
 * @param {(i: number) => boolean} signed whether row `i` is a sign's period
 * @returns {readonly object[]}
 */
function periodsFrom(cols, start, count, signed) {
  const periods = [];
  const path = [];
  for (let i = start; i < start + count; i += 1) {
    const level = cols.level[i];
    path.length = level - 1;
    path.push(cols.index[i]);
    periods.push(
      Object.freeze({
        path: path.join('/'),
        level,
        sign: signed(i) ? (RashiById.get(cols.sign[i]) ?? 'unknown') : null,
        lord: GrahaById.get(cols.lord[i]) ?? 'unknown',
        from: cols.fromJd[i],
        to: cols.toJd[i],
      }),
    );
  }
  return Object.freeze(periods);
}

/**
 * The periods running at a Julian day (UTC), from the mahadasha down.
 * Depth first order means a period's children follow it, so one walk that
 * takes the next level's running period finds the chain.
 *
 * @param {readonly object[]} periods
 * @param {number} jd
 * @returns {object[]}
 */
function chainAt(periods, jd) {
  const chain = [];
  for (const period of periods) {
    if (period.level === chain.length + 1 && period.from <= jd && jd < period.to) chain.push(period);
  }
  return chain;
}

/** One dasha row and its periods, in this layer's shape. */
function dashaFrom(batch, row, start, count) {
  const d = batch.decoded;
  const rows = d.dashas;
  const seeded = rows.seeded[row] !== 0;
  const signed = rows.signed[row] !== 0;
  const periods = periodsFrom(d.dashaPeriods, start, count, () => signed);
  const spanFrom = rows.moonSpanFrom[row];
  return Object.freeze({
    system: DashaSystemById.get(rows.system[row]) ?? batch.dashaName(rows.system[row]) ?? 'unknown',
    seed: seeded ? (NakshatraById.get(rows.seed[row]) ?? 'unknown') : null,
    firstLord: GrahaById.get(rows.firstLord[row]) ?? 'unknown',
    overflow: rows.overflow[row] !== 0,
    balance: seeded
      ? Object.freeze({
          method: BalanceById.get(rows.balance[row]) ?? 'unknown',
          remaining: rows.remaining[row],
          days: rows.balanceDays[row],
          written: Object.freeze({
            years: rows.balanceYears[row],
            months: rows.balanceMonths[row],
            days: rows.balanceDayCount[row],
            hours: rows.balanceHours[row],
            minutes: rows.balanceMinutes[row],
          }),
        })
      : null,
    moonSpan: Number.isNaN(spanFrom) ? null : Object.freeze({ from: spanFrom, to: rows.moonSpanTo[row] }),
    depth: rows.depth[row],
    periods,
    /**
     * The periods running at a Julian day (UTC), from the mahadasha down to
     * the depth the periods go; empty before birth and past the cycle.
     */
    at(jd) {
      return chainAt(periods, jd);
    },
  });
}

/** Each batch's Ashtakavargas, decoded once however many charts read them. */
const ASHTAKAVARGAS = new WeakMap();

/** Every chart's Ashtakavarga in a batch; empty when none was asked for. */
function ashtakavargasOf(batch) {
  let decoded = ASHTAKAVARGAS.get(batch);
  if (decoded === undefined) {
    const d = batch.decoded;
    const rows = d.ashtakavarga;
    const bins = d.ashtakavargaBindus;
    const sums = d.sarvashtakavarga;
    const twelve = (column, from) => Object.freeze(Array.from(column.subarray(from, from + 12)));
    decoded = Array.from({ length: rows.length / 7 }, (_, chart) => {
      const grahas = Array.from({ length: 7 }, (_, g) => {
        const row = chart * 7 + g;
        const eachGraha = ShodhanaById.get(rows.shodhana[row]) === 'EACH_GRAHA';
        return Object.freeze({
          graha: GrahaById.get(rows.graha[row]) ?? 'unknown',
          bindus: twelve(bins.bindus, row * 12),
          reduced: eachGraha ? twelve(bins.reduced, row * 12) : null,
          rashiPinda: rows.rashiPinda[row],
          grahaPinda: rows.grahaPinda[row],
          yogaPinda: rows.yogaPinda[row],
        });
      });
      return Object.freeze({
        shodhana: ShodhanaById.get(rows.shodhana[chart * 7]) ?? 'unknown',
        ekadhipatya: EkadhipatyaById.get(rows.ekadhipatya[chart * 7]) ?? 'unknown',
        grahas: Object.freeze(grahas),
        sarva: twelve(sums.sarva, chart * 12),
        trikona: twelve(sums.trikona, chart * 12),
        reduced: twelve(sums.reduced, chart * 12),
      });
    });
    ASHTAKAVARGAS.set(batch, decoded);
  }
  return decoded;
}

/** Each batch's Vaiseshikamsas, decoded once however many charts read them. */
const VAISESHIKAMSAS = new WeakMap();

/** Every chart's Vaiseshikamsa in a batch; empty when none was asked for. */
function vaiseshikamsasOf(batch) {
  let decoded = VAISESHIKAMSAS.get(batch);
  if (decoded === undefined) {
    const c = batch.decoded.vaiseshikamsa;
    const standing = (row, scheme) => {
      const good = c[`${scheme}Good`][row];
      return Object.freeze({
        goodVargas: good,
        name: good >= 2 ? (VaiseshikamsaById.get(c[`${scheme}Name`][row]) ?? 'unknown') : null,
      });
    };
    decoded = Array.from({ length: c.length / 7 }, (_, chart) =>
      Object.freeze({
        grahas: Object.freeze(
          Array.from({ length: 7 }, (_, g) => {
            const row = chart * 7 + g;
            return Object.freeze({
              graha: GrahaById.get(c.graha[row]) ?? 'unknown',
              shadvarga: standing(row, 'shadvarga'),
              saptavarga: standing(row, 'saptavarga'),
              dashavarga: standing(row, 'dashavarga'),
              shodashavarga: standing(row, 'shodashavarga'),
              impaired: c.impaired[row] === 1,
            });
          }),
        ),
      }),
    );
    VAISESHIKAMSAS.set(batch, decoded);
  }
  return decoded;
}

/** The seven vargas whose Subhanka columns the dasha phala carries, in order. */
const SUBHANKA_VARGAS = ['D1', 'D2', 'D3', 'D7', 'D9', 'D12', 'D30'];

/** Each batch's dasha phalas, decoded once however many charts read them. */
const DASHA_PHALAS = new WeakMap();

/** Every chart's dasha phala in a batch; empty when none was asked for. */
function dashaPhalasOf(batch) {
  let decoded = DASHA_PHALAS.get(batch);
  if (decoded === undefined) {
    const c = batch.decoded.dashaPhala;
    decoded = Array.from({ length: c.graha.length / 9 }, (_, chart) =>
      Object.freeze({
        grahas: Object.freeze(
          Array.from({ length: 9 }, (_, g) => {
            const row = chart * 9 + g;
            return Object.freeze({
              graha: GrahaById.get(c.graha[row]) ?? 'unknown',
              subhankas: Object.freeze(SUBHANKA_VARGAS.map((varga) => c[`subhanka${varga}`][row])),
              subhanka: c.subhanka[row],
              asubhanka: c.asubhanka[row],
              nature: NatureById.get(c.nature[row]) ?? 'unknown',
              phase: DashaPhaseById.get(c.phase[row]) ?? 'unknown',
              favourable: c.favourable[row] !== 0,
              unfavourable: c.unfavourable[row] !== 0,
            });
          }),
        ),
      }),
    );
    DASHA_PHALAS.set(batch, decoded);
  }
  return decoded;
}

/** Each batch's Vimshopakas, decoded once however many charts read them. */
const VIMSHOPAKAS = new WeakMap();

/** Every chart's Vimshopaka in a batch; empty when none was asked for. */
function vimshopakasOf(batch) {
  let decoded = VIMSHOPAKAS.get(batch);
  if (decoded === undefined) {
    const rows = batch.decoded.vimshopaka;
    decoded = Array.from({ length: rows.length / 7 }, (_, chart) =>
      Object.freeze({
        scoring: VimshopakaScoringById.get(rows.scoring[chart * 7]) ?? 'unknown',
        grahas: Object.freeze(
          Array.from({ length: 7 }, (_, g) => {
            const row = chart * 7 + g;
            return Object.freeze({
              graha: GrahaById.get(rows.graha[row]) ?? 'unknown',
              shadvarga: rows.shadvarga[row],
              saptavarga: rows.saptavarga[row],
              dashavarga: rows.dashavarga[row],
              shodashavarga: rows.shodashavarga[row],
            });
          }),
        ),
      }),
    );
    VIMSHOPAKAS.set(batch, decoded);
  }
  return decoded;
}

/** Each batch's Bhava balas, decoded once however many charts read them. */
const BHAVA_BALAS = new WeakMap();

/** Every chart's Bhava bala in a batch; empty when none was asked for. */
function bhavaBalasOf(batch) {
  let decoded = BHAVA_BALAS.get(batch);
  if (decoded === undefined) {
    const c = batch.decoded.bhavaBala;
    decoded = Array.from({ length: c.length / 12 }, (_, chart) =>
      Object.freeze({
        bhavas: Object.freeze(
          Array.from({ length: 12 }, (_, h) => {
            const row = chart * 12 + h;
            return Object.freeze({
              bhava: h + 1,
              lord: GrahaById.get(c.lord[row]) ?? 'unknown',
              adhipati: c.adhipati[row],
              dig: c.dig[row],
              drishti: c.drishti[row],
              special: c.special[row],
              virupas: c.virupas[row],
            });
          }),
        ),
      }),
    );
    BHAVA_BALAS.set(batch, decoded);
  }
  return decoded;
}

/** Each batch's Shadbalas, decoded once however many charts read them. */
const SHADBALAS = new WeakMap();

/** Every chart's Shadbala in a batch; empty when none was asked for. */
function shadbalasOf(batch) {
  let decoded = SHADBALAS.get(batch);
  if (decoded === undefined) {
    const c = batch.decoded.shadbala;
    const graha = (row) => {
      const sthana = {
        uchcha: c.uchcha[row],
        saptavargaja: c.saptavargaja[row],
        ojayugma: c.ojayugma[row],
        kendradi: c.kendradi[row],
        drekkana: c.drekkana[row],
      };
      const kaala = {
        nathonnatha: c.nathonnatha[row],
        paksha: c.paksha[row],
        tribhaga: c.tribhaga[row],
        abda: c.abda[row],
        masa: c.masa[row],
        vara: c.vara[row],
        hora: c.hora[row],
        ayana: c.ayana[row],
        yuddha: c.yuddha[row],
      };
      return Object.freeze({
        graha: GrahaById.get(c.graha[row]) ?? 'unknown',
        sthana: Object.freeze(sthana),
        dig: c.dig[row],
        kaala: Object.freeze(kaala),
        cheshta: c.cheshta[row],
        naisargika: c.naisargika[row],
        drik: c.drik[row],
        virupas: c.virupas[row],
        rupas: c.rupas[row],
        requiredRupas: c.requiredRupas[row],
        ishta: c.ishta[row],
        kashta: c.kashta[row],
        subhaRashmi: c.subhaRashmi[row],
        ashubhaRashmi: c.ashubhaRashmi[row],
        strong: c.strong[row] === 1,
      });
    };
    decoded = Array.from({ length: c.length / 7 }, (_, chart) =>
      Object.freeze({ grahas: Object.freeze(Array.from({ length: 7 }, (_, g) => graha(chart * 7 + g))) }),
    );
    SHADBALAS.set(batch, decoded);
  }
  return decoded;
}

/** Each batch's rule answers, parsed once however many charts read them. */
const RULES = new WeakMap();

/**
 * Every chart's answers by rule in a batch: the `rules` section's JSON, one
 * entry a chart, frozen as it was written.
 *
 * @param {Charts} batch
 * @returns {object[]}
 */
function rulesOf(batch) {
  return sectionOf(RULES, batch, 'rules');
}

/** Each batch's plans, parsed once however many charts read them. */
const PLANS = new WeakMap();

/**
 * Every chart's narrative plans in a batch: the `plans` section's JSON, one
 * entry a chart, frozen as it was written
 * (`03-design/plans-at-the-boundary.md`).
 *
 * @param {Charts} batch
 * @returns {object[]}
 */
function plansOf(batch) {
  return sectionOf(PLANS, batch, 'plans');
}

/**
 * A batch's JSON section, parsed once however many charts read it and frozen
 * to its leaves, so a reading handed out is a reading kept. Empty when the
 * request did not ask for the section.
 *
 * @param {WeakMap<Charts, object[]>} cache
 * @param {Charts} batch
 * @param {string} name
 * @returns {object[]}
 */
function sectionOf(cache, batch, name) {
  let parsed = cache.get(batch);
  if (parsed === undefined) {
    const json = batch.decoded[name];
    parsed = json ? JSON.parse(json).map((chart) => deepFreeze(chart)) : [];
    cache.set(batch, parsed);
  }
  return parsed;
}

/**
 * A value frozen to its leaves, so a reading handed out is a reading kept.
 *
 * @template T
 * @param {T} value
 * @returns {T}
 */
function deepFreeze(value) {
  if (value !== null && typeof value === 'object') {
    for (const inner of Object.values(value)) deepFreeze(inner);
    Object.freeze(value);
  }
  return value;
}

/**
 * The rules a request asks a chart to answer, as the JSON the boundary reads
 * (`03-design/rules-at-the-boundary.md`). The SDK refuses what it cannot read,
 * naming the field from `rules_json`.
 *
 * @param {object|undefined} rules
 * @returns {string|undefined}
 */
function rulesJson(rules) {
  return recordJson(rules, 'rules', 'a rule request record, e.g. { shipped: ["NABHASAS"] }');
}

/**
 * The plans a request asks a chart for, as the JSON the boundary reads
 * (`03-design/plans-at-the-boundary.md`). The SDK refuses what it cannot
 * read, naming the field from `interpret_json`.
 *
 * @param {object|undefined} interpret
 * @returns {string|undefined}
 */
function interpretJson(interpret) {
  return recordJson(interpret, 'interpret', 'a plan request record, e.g. { placements: true }');
}

/**
 * The annual charts a request asks for, as the JSON the boundary reads
 * (`03-design/annual-chart.md`). The SDK refuses what it cannot read,
 * naming the field from `varsha_json`.
 *
 * @param {object|undefined} varsha
 * @returns {string|undefined}
 */
function varshaJson(varsha) {
  if (varsha && typeof varsha === 'object' && !Array.isArray(varsha) && varsha.place !== undefined) {
    return recordJson(
      { ...varsha, place: annualPlace(varsha.place) },
      'varsha',
      'an annual-chart request record',
    );
  }
  return recordJson(
    varsha,
    'varsha',
    'an annual-chart request record, e.g. { reading: "SIDEREAL", through: 40 }',
  );
}

/**
 * Where each year's chart is cast, in the place shape `found` itself takes
 * — `'birth'`, or `{ latitude, longitude, altitude, utcOffsetSeconds }` —
 * written in the boundary's words. A key this does not know is passed
 * through rather than dropped, so the SDK refuses it by name.
 *
 * @param {'birth'|object} place
 * @returns {'birth'|object}
 */
function annualPlace(place) {
  // A word crosses as written, so a wrong one is refused by the SDK, by
  // `varsha_json.place`, in the same words every binding gets.
  if (typeof place === 'string') return place;
  if (!place || typeof place !== 'object' || Array.isArray(place)) {
    throw new TypeError(
      "varsha.place: expected 'birth' or { latitude, longitude, altitude, utcOffsetSeconds }",
    );
  }
  const { latitude, longitude, altitude, utcOffsetSeconds, ...rest } = place;
  return {
    ...rest,
    latitudeDeg: finite(latitude, 'varsha.place.latitude'),
    longitudeDeg: finite(longitude, 'varsha.place.longitude'),
    altitudeM: finite(altitude ?? 0, 'varsha.place.altitude'),
    utcOffsetSeconds: finite(utcOffsetSeconds, 'varsha.place.utcOffsetSeconds'),
  };
}

/**
 * A request option that crosses as a JSON record: the record written down,
 * or nothing where it was not given. Anything else is this layer's own
 * `TypeError`, named and shown, rather than a refusal from across the
 * boundary.
 *
 * @param {object|undefined} value
 * @param {string} field
 * @param {string} example
 * @returns {string|undefined}
 */
function recordJson(value, field, example) {
  if (value === undefined || value === null) return undefined;
  if (typeof value === 'object' && !Array.isArray(value)) return JSON.stringify(value);
  throw new TypeError(`${field}: expected ${example}`);
}

/**
 * A return's own chart, when `varsha.place` asked for the charts: row
 * `row` of `annual_charts`, which runs beside `praveshas` row for row or
 * is empty. Anything between is a layout this layer does not know how to
 * read, and it says so rather than pairing a year with another's chart.
 *
 * @param {object} d the decoded batch
 * @param {number} row
 * @returns {object|null}
 */
function annualOf(d, row) {
  const charts = d.annualCharts;
  if (charts.lagnaDeg.length === 0) return null;
  if (charts.lagnaDeg.length !== d.praveshas.year.length) {
    throw new Error(
      `annual_charts has ${charts.lagnaDeg.length} rows beside ${d.praveshas.year.length} returns; ` +
        'it is all of them or none',
    );
  }
  const lord = (column) => GrahaById.get(column[row]) ?? 'unknown';
  // The claims are ragged by `claimCount`, as the returns are by
  // `praveshaCount`: this year's block starts where the ones before end.
  const from = startsOf(charts.claimCount)[row];
  const count = charts.claimCount[row] ?? 0;
  const claims = d.yearClaims;
  return {
    lagnaDeg: charts.lagnaDeg[row],
    byDay: charts.daylight[row] === 1,
    officeBearers: {
      muntha: lord(d.praveshas.munthaLord),
      janmaLagna: lord(charts.janmaLagnaLord),
      varshaLagna: lord(charts.varshaLagnaLord),
      triRashi: lord(charts.triRashiLord),
      dinaRatri: lord(charts.dinaRatriLord),
    },
    yogas: yogasOf(d, row),
    retrograde: grahasIn(charts.retrograde[row]),
    combust: grahasIn(charts.combust[row]),
    matters: mattersOf(d, row),
    sahams: sahamsOf(d, row),
    harsha: harshaOf(d, row),
    dashas: annualDashasOf(d, row),
    yearLord: {
      graha: lord(charts.yearLord),
      chosen: VarsheshaChosenById.get(charts.yearLordChosen[row]) ?? 'unknown',
      vishwa: bala(charts.yearLordVishwa[row]),
      moonPassedOver: charts.moonPassedOver[row] === 1,
      claims: Array.from({ length: count }, (_, k) => ({
        graha: GrahaById.get(claims.graha[from + k]) ?? 'unknown',
        vishwa: bala(claims.vishwa[from + k]),
        portfolios: claims.portfolios[from + k],
        aspectsLagna: claims.aspectsLagna[from + k] === 1,
      })),
    },
  };
}

/**
 * The pairs of a year's chart that make a Tajika yoga, ragged by
 * `yogaCount` as the claims are by `claimCount`.
 *
 * @param {object} d the decoded batch
 * @param {number} row
 * @returns {object[]}
 */
function yogasOf(d, row) {
  const charts = d.annualCharts;
  const from = startsOf(charts.yogaCount)[row];
  const count = charts.yogaCount[row] ?? 0;
  return Array.from({ length: count }, (_, k) => pairAt(d.yearYogas, from + k));
}

/**
 * A year's sahams, ragged by `sahamCount` (`03-design/tajika-sahams.md`).
 *
 * @param {object} d the decoded batch
 * @param {number} row
 * @returns {object[]}
 */
function sahamsOf(d, row) {
  const from = startsOf(d.annualCharts.sahamCount)[row];
  const count = d.annualCharts.sahamCount[row] ?? 0;
  return Array.from({ length: count }, (_, k) => sahamAt(d.yearSahams, d.yearSahamSeven, from + k));
}

/**
 * A year's annual dashas, ragged by `dashaCount`, each with its ring and
 * its periods ragged under it by `shareCount` and `periodCount`
 * (`03-design/annual-dashas.md`).
 *
 * @param {object} d the decoded batch
 * @param {number} row
 * @returns {object[]}
 */
function annualDashasOf(d, row) {
  const rows = d.yearDashas;
  const shares = d.yearDashaShares;
  const cols = d.yearDashaPeriods;
  const from = startsOf(d.annualCharts.dashaCount)[row];
  const count = d.annualCharts.dashaCount[row] ?? 0;
  const shareStarts = startsOf(rows.shareCount);
  const periodStarts = startsOf(rows.periodCount);
  return Array.from({ length: count }, (_, k) => {
    const at = from + k;
    const ring = Array.from({ length: rows.shareCount[at] }, (_, j) => {
      const i = shareStarts[at] + j;
      return Object.freeze({
        lord: GrahaById.get(shares.lord[i]) ?? 'unknown',
        sign: shares.hasSign[i] === 1 ? (RashiById.get(shares.sign[i]) ?? 'unknown') : null,
        weight: shares.weight[i],
      });
    });
    const first = rows.first[at];
    const remaining = rows.remaining[at];
    const periods = periodsFrom(cols, periodStarts[at], rows.periodCount[at], (i) => cols.hasSign[i] === 1);
    return Object.freeze({
      system: DashaSystemById.get(rows.system[at]) ?? 'unknown',
      seed: rows.seeded[at] === 1 ? (NakshatraById.get(rows.seed[at]) ?? 'unknown') : null,
      firstLord: ring[first]?.lord ?? 'unknown',
      ring: Object.freeze({
        shares: Object.freeze(ring),
        first,
        remaining: Number.isNaN(remaining) ? null : remaining,
      }),
      year: Object.freeze({ from: rows.fromJd[at], to: rows.toJd[at] }),
      periods,
      /**
       * The periods running at a Julian day (UTC), from the mahadasha down;
       * empty outside the year.
       */
      at(jd) {
        return chainAt(periods, jd);
      },
    });
  });
}

/**
 * One saham row, from the years' sections or the births': where it fell,
 * its strength clause by clause, and the seven rows under it
 * (`03-design/tajika-saham-strength.md`). One decoder for both, so they
 * cannot drift.
 *
 * @param {object} cols the saham section
 * @param {object} seven the seven-row section under it
 * @param {number} k the row
 * @returns {object}
 */
function sahamAt(cols, seven, k) {
  const axis = cols.nodeAxis[k];
  return {
    saham: SahamById.get(cols.saham[k]) ?? 'unknown',
    longitudeDeg: cols.longitudeDeg[k],
    sign: RashiById.get(cols.sign[k]) ?? 'unknown',
    lord: GrahaById.get(cols.lord[k]) ?? 'unknown',
    house: cols.house[k],
    addedSign: cols.addedSign[k] === 1,
    strong: membersOf(cols.strong[k], SahamStrongById),
    weak: membersOf(cols.weak[k], SahamWeakById),
    lordVishwa: bala(cols.lordVishwa[k]),
    lordHarsha: HarshaGradeById.get(cols.lordHarsha[k]) ?? 'unknown',
    // 2 is the boundary's "the chart placed no nodes to read".
    inNodeAxis: axis === 2 ? null : axis === 1,
    // A handicap the source names: the 6th, 8th or 12th.
    handicapped: [6, 8, 12].includes(cols.house[k]),
    seven: Array.from({ length: 7 }, (_, g) => {
      const at = 7 * k + g;
      return {
        graha: GrahaById.get(seven.graha[at]) ?? 'unknown',
        drishti: TajikaDrishtiById.get(seven.drishti[at]) ?? 'unknown',
        relation: TajikaRelationById.get(seven.relation[at]) ?? 'unknown',
        company: seven.company[at] === 1,
      };
    }),
  };
}

/**
 * A founded year's Harsha bala: seven rows a year, fixed, in the
 * catalogue's order (`03-design/tajika-harsha.md`).
 *
 * @param {object} d the decoded batch
 * @param {number} row
 * @returns {object[]}
 */
function harshaOf(d, row) {
  const h = d.yearHarsha;
  return Array.from({ length: 7 }, (_, g) => {
    const at = 7 * row + g;
    return {
      graha: GrahaById.get(h.graha[at]) ?? 'unknown',
      house: h.house[at],
      sthana: h.sthana[at] === 1,
      uchchaSwakshetra: h.uchchaSwakshetra[at] === 1,
      striPurusha: h.striPurusha[at] === 1,
      dinaRatri: h.dinaRatri[at] === 1,
      total: h.total[at],
      grade: HarshaGradeById.get(h.grade[at]) ?? 'unknown',
    };
  });
}

/** Each ragged count column's prefix sums, computed once per batch. */
const STARTS = new WeakMap();

/**
 * Where each row's block starts in the section a count column is ragged
 * by: `starts[row]` to `starts[row + 1]`. Computed once for the column,
 * so reading every year of a batch is linear rather than quadratic.
 *
 * @param {ArrayLike<number>} counts
 * @returns {Uint32Array}
 */
function startsOf(counts) {
  let starts = STARTS.get(counts);
  if (starts === undefined) {
    starts = new Uint32Array(counts.length + 1);
    for (let i = 0; i < counts.length; i += 1) starts[i + 1] = starts[i] + counts[i];
    STARTS.set(counts, starts);
  }
  return starts;
}

/**
 * How two planets stand, from a section carrying the pair columns — the
 * year's own pairs, a matter's lords (under `pair`) or a yoga's legs. The
 * yoga is `null` where they make none, which only the matter sections
 * carry.
 *
 * @param {object} cols the decoded section
 * @param {number} k the row
 * @param {string} [prefix] the columns' prefix, `'pair'` for a matter's
 * @returns {object}
 */
function pairAt(cols, k, prefix = '') {
  const at = (name) => cols[prefix ? prefix + name[0].toUpperCase() + name.slice(1) : name];
  const present = at('yogaPresent');
  return {
    faster: GrahaById.get(at('faster')[k]) ?? 'unknown',
    slower: GrahaById.get(at('slower')[k]) ?? 'unknown',
    drishti: TajikaDrishtiById.get(at('drishti')[k]) ?? 'unknown',
    yoga:
      present === undefined || present[k] === 1
        ? (TajikaYogaById.get(at('yoga')[k]) ?? 'unknown')
        : null,
    orbDeg: at('orbDeg')[k],
    apartDeg: at('apartDeg')[k],
  };
}

/**
 * The members of a bit set over a small closed enum: bit `n` is the member
 * with id `n`, in id order.
 *
 * @param {number} bits
 * @param {Map<number, string>} byId
 * @returns {string[]}
 */
function membersOf(bits, byId) {
  const found = [];
  for (const [id, key] of byId) if (bits & (1 << id)) found.push(key);
  return found;
}

/**
 * The seven a year's bit set names, in graha id order.
 *
 * @param {number} bits
 * @returns {string[]}
 */
function grahasIn(bits) {
  return membersOf(bits ?? 0, GrahaById);
}

/**
 * A year's matters: each the question it asked, the lords' own pair, the
 * yogas it could not answer, and every yoga that held with what made it —
 * ragged three deep, each block located once by `startsOf`
 * (`03-design/tajika-yogas.md`, "Crossing the boundary").
 *
 * @param {object} d the decoded batch
 * @param {number} row
 * @returns {object[]}
 */
function mattersOf(d, row) {
  const matters = d.yearMatters;
  const held = d.matterYogas;
  const heldStarts = startsOf(matters.heldCount);
  const legStarts = startsOf(held.legCount);
  const counted = startsOf(d.annualCharts.matterCount);
  const found = [];
  for (let m = counted[row]; m < counted[row + 1]; m += 1) {
    const between = matters.sameLord[m] === 1 ? null : pairAt(matters, m, 'pair');
    const unanswered = membersOf(matters.unanswered[m], YearYogaById);
    const yogas = [];
    for (let h = heldStarts[m]; h < heldStarts[m + 1]; h += 1) {
      const legs = [];
      for (let l = legStarts[h]; l < legStarts[h + 1]; l += 1) legs.push(pairAt(d.matterLegs, l));
      yogas.push({
        yoga: YearYogaById.get(held.yoga[h]) ?? 'unknown',
        between: held.byPair[h] === 1 ? between : null,
        through: held.throughPresent[h] === 1 ? (GrahaById.get(held.through[h]) ?? 'unknown') : null,
        entering:
          held.enteringPresent[h] === 1 ? (GrahaById.get(held.entering[h]) ?? 'unknown') : null,
        legs: legs.length === 0 ? null : legs,
        afflictions:
          held.afflictionsPresent[h] === 1
            ? {
                lagnesha: membersOf(held.lagneshaAfflictions[h], AfflictionById),
                karyesha: membersOf(held.karyeshaAfflictions[h], AfflictionById),
              }
            : null,
      });
    }
    found.push({
      house: matters.house[m],
      sign: RashiById.get(matters.sign[m]) ?? 'unknown',
      lagnesha: GrahaById.get(matters.lagnesha[m]) ?? 'unknown',
      karyesha: GrahaById.get(matters.karyesha[m]) ?? 'unknown',
      sameLord: matters.sameLord[m] === 1,
      between,
      held: yogas,
      unanswered,
      // `null` where this call could not answer for the yoga, which is not
      // the same answer as `false`.
      holds: (yoga) =>
        unanswered.includes(yoga) ? null : yogas.some((one) => one.yoga === yoga),
    });
  }
  return found;
}

/**
 * A Tajika strength, which the boundary carries exactly as an integer
 * count of sub-sub units — 3600 to a unit — because two office-bearers a
 * sub-sub unit apart decide a year between them. `units` is the figure a
 * reader compares; `toString()` is how the sources write one.
 *
 * @param {number} subSub
 * @returns {object}
 */
function bala(subSub) {
  const units = Math.trunc(subSub / 3600);
  const rest = subSub - units * 3600;
  const parts = {
    units,
    subUnits: Math.trunc(rest / 60),
    subSub: rest % 60,
    total: subSub,
  };
  const pad = (n) => String(n).padStart(2, '0');
  return {
    ...parts,
    toString: () => `${pad(parts.units)}:${pad(parts.subUnits)}:${pad(parts.subSub)}`,
  };
}

/** Each batch's drawings, parsed once however many charts read them. */
const DRAWINGS = new WeakMap();

/**
 * Every chart's drawings in a batch, in this layer's shape: catalogue keys in
 * full (`rashi.LEO`, `varga.D9`) and camel-cased fields, as every other
 * accessor gives them.
 *
 * @param {Charts} batch
 * @returns {object[][]}
 */
function drawingsOf(batch) {
  let parsed = DRAWINGS.get(batch);
  if (parsed === undefined) {
    const { drawings, svgs } = batch.decoded;
    const written = svgs ? JSON.parse(svgs) : [];
    parsed = drawings
      ? JSON.parse(drawings).map((charted, chart) =>
          charted.map((drawing, index) => drawingFrom(drawing, written[chart]?.[index])),
        )
      : [];
    DRAWINGS.set(batch, parsed);
  }
  return parsed;
}

/**
 * A drawing as the boundary's JSON writes it, in this layer's shape, with
 * its SVG when the request gave a theme.
 */
function drawingFrom({ varga, placed }, svg) {
  return Object.freeze({
    svg,
    layout: `chart_layout.${placed.layout}`,
    varga: `varga.${varga}`,
    cells: placed.cells.map((cell) =>
      Object.freeze({
        outline: cell.outline,
        sign: `rashi.${cell.sign}`,
        house: cell.house,
        lagna: cell.lagna,
        ring: cell.ring,
        label: cell.label,
        anchor: cell.anchor,
        bodies: cell.bodies,
      }),
    ),
    frame: placed.frame,
    marks: placed.marks.map((mark) =>
      Object.freeze({ body: mark.body, ring: mark.ring, at: mark.at, longitudeDeg: mark.longitude_deg }),
    ),
  });
}

/**
 * The theme a request draws its SVGs in, as the JSON the boundary reads: a
 * shipped theme's key, or a record naming only what it changes over the
 * light theme or the one its `extends` names (`03-design/render-svg.md`).
 *
 * @param {string|object|undefined} theme
 * @returns {string|undefined}
 */
function themeJson(theme) {
  if (theme === undefined || theme === null) return undefined;
  if (typeof theme === 'string') return JSON.stringify({ extends: theme });
  if (typeof theme === 'object' && !Array.isArray(theme)) return JSON.stringify(theme);
  throw new TypeError("theme: expected 'LIGHT', 'DARK' or a theme record");
}

/**
 * The member id of each layout a context registered, by its full key: asked
 * of the context once, when it is made, so a request resolves a consumer's
 * own layout without crossing the boundary again (§7f).
 *
 * @param {object} inner the addon's context
 * @param {string} kind the kind the rows are of (`chart_layout`, `dasha_system`)
 * @param {ReadonlyArray<{key: string}>|undefined} rows the rows it registered
 * @returns {Map<string, number>}
 */
function registeredIds(inner, kind, rows) {
  return new Map(
    (rows ?? []).map(({ key }) => {
      const full = `${kind}.${key}`;
      return [full, guarded(inner, () => inner.keyParse(full)) & 0xffff];
    }),
  );
}

/** Every catalogue key by its id, turned round, for a request to write ids. */
const idOf = (byId) => new Map(Array.from(byId, ([id, key]) => [key, id]));
const LAYOUT_IDS = idOf(ChartLayoutById);
const DASHA_IDS = idOf(DashaSystemById);

/**
 * The dashas a request asked for, as the ids the boundary takes: a
 * `DashaSystem`, or the `dasha_system.*` key of a system this context
 * registered (`03-design/dasha-kernels.md`).
 *
 * @param {ReadonlyArray<string>|undefined} asked
 * @param {Map<string, number>} registered the context's own systems' ids
 * @returns {number[]}
 */
function dashaIds(asked, registered) {
  return catalogueKeys(asked, 'dashas', 'DashaSystem').map((key, at) => {
    const id = DASHA_IDS.get(key) ?? registered.get(key);
    if (id === undefined) {
      throw new TypeError(
        `dashas[${at}]: expected a DashaSystem, or the dasha_system.* key of a system this context registered`,
      );
    }
    return id;
  });
}
const VARGA_IDS = idOf(VargaById);

/**
 * The drawings a request asked for, as the packed ids the boundary takes:
 * `layout_id << 16 | varga_id` each, so a caller names pairs and nothing
 * here writes bits by hand (`03-design/chart-geometry.md`).
 *
 * @param {ReadonlyArray<{layout: string, varga: string}>|undefined} asked
 * @param {Map<string, number>} registered the context's own layouts' ids
 * @returns {number[]}
 */
function drawingBits(asked, registered) {
  if (asked === undefined || asked === null) return [];
  if (!Array.isArray(asked)) {
    throw new TypeError('drawings: expected an array of { layout, varga }');
  }
  return asked.map((drawing, at) => {
    const named = drawing?.layout;
    // A shipped layout is in the catalogue's table; a consumer's own is in
    // the ids its context resolved once, when it was made
    // (`03-design/chart-geometry.md` §7f).
    const layout = LAYOUT_IDS.get(named) ?? registered.get(named);
    const varga = VARGA_IDS.get(drawing?.varga);
    if (layout === undefined) {
      throw new TypeError(
        `drawings[${at}].layout: expected a ChartLayout, or the chart_layout.* key of a layout this context registered`,
      );
    }
    if (varga === undefined) {
      throw new TypeError(`drawings[${at}].varga: expected a Varga key`);
    }
    return ((layout << 16) | varga) >>> 0;
  });
}

/**
 * `TS_CHART_ASPECTS`, the one section bit this layer offers so far.
 *
 * The bits are the C ABI's vocabulary; a consumer of this binding writes
 * `aspects: true` (`03-design/chart-reading.md` §5).
 */
const SECTION_ASPECTS = 4;

/** `TS_CHART_POINTS`, the derived points. */
const SECTION_POINTS = 8;

/** `TS_CHART_HOUSES`, the houses service. */
const SECTION_HOUSES = 16;

/** `TS_CHART_ASHTAKAVARGA`, the Ashtakavarga. */
const SECTION_ASHTAKAVARGA = 32;

/** `TS_CHART_VIMSHOPAKA`, the Vimshopaka. */
const SECTION_VIMSHOPAKA = 64;

/** `TS_CHART_VAISESHIKAMSA`, the Vaiseshikamsa. */
const SECTION_VAISESHIKAMSA = 512;
/** `TS_CHART_DASHA_PHALA`, the dasha phala. */
const SECTION_DASHA_PHALA = 1024;

/** `TS_CHART_SHADBALA`, the Shadbala. */
const SECTION_SHADBALA = 128;

/** `TS_CHART_BHAVA_BALA`, the Bhava bala. */
const SECTION_BHAVA_BALA = 256;

/** `TS_CHART_STATE`, the planetary states. */
const SECTION_STATE = 2;

/**
 * The divisional charts a request asked for, checked.
 *
 * An absent list is none, which is the default: the sections are
 * "pay for what you ask for" (`03-design/chart-reading.md` §4).
 *
 * @param {ReadonlyArray<string>|undefined} asked
 * @returns {string[]}
 */
function catalogueKeys(asked, field, kind) {
  if (asked === undefined || asked === null) return [];
  const list = ArrayBuffer.isView(asked) ? Array.from(asked) : asked;
  if (!Array.isArray(list)) {
    throw new TypeError(`${field}: expected an array of ${kind} keys`);
  }
  return list.map((key, at) => {
    if (typeof key !== 'string') {
      throw new TypeError(`${field}[${at}]: expected a ${kind} key`);
    }
    return key;
  });
}

/**
 * `sdk.almanac` — a day, or a run of days, with its limbs.
 *
 * The boundary calls this `panchanga` and the area takes the consumer's
 * word: an almanac is what the operation answers, and a panchanga is one
 * tradition's name for five of its limbs
 * (`03-design/surface-areas.md`).
 */
export class AlmanacArea extends Area {
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
  of(request) {
    const place = request.place ?? {};
    const from = request.from ?? {};
    const to = request.to ?? from;
    const bytes = run(this, (inner) =>
      inner.panchangaDays({
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
  day(request) {
    return this.of({ ...request, from: request.date, to: request.date }).at(0);
  }
}

export class Context {
  #inner;
  /** The engine's own operations, built with the areas. */
  #engine;
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
   * @param {EphemerisChoice|readonly EphemerisChoice[]} [options.ephemeris]
   *   which ephemeris to compute with, or an **ordered chain** of them,
   *   tried in order (ADR-0029).
   *
   *   An entry is an adapter's own descriptor — what
   *   `@teistro/ephemeris-teimeris` and its like export, carrying the
   *   platform binary they ship and their own configuration — or one of
   *   the SDK's own by name: `BUILTIN` is the analytic ephemeris the SDK
   *   carries, which needs no files, no network and no licence beyond
   *   the SDK's own; `TEST` is the test provider, whose positions are
   *   **not astronomy**.
   *
   *   A chain is a caller **saying** they will accept the fallback. One
   *   entry is one entry: a context asked for an engine and given the
   *   built-in without being told is the silence this refuses.
   * @param {boolean} [options.testProvider] the older spelling of
   *   `ephemeris: 'TEST'`; `ephemeris` wins when both are given
   * @param {object} [options.provider] an ephemeris of your own: `name`,
   *   `bodies` (their catalogue keys) and `positions(request)`, which
   *   answers with the columns; everything else has a default
   */
  constructor(options = {}) {
    const { profile, settings, locale, layouts, dashaSystems, ephemeris, testProvider = false, provider } = options;
    if (layouts !== undefined && !Array.isArray(layouts)) {
      throw new TypeError('layouts: expected an array of layout rows');
    }
    if (dashaSystems !== undefined && !Array.isArray(dashaSystems)) {
      throw new TypeError('dashaSystems: expected an array of dasha system definitions');
    }
    // Two ways to answer one question, so both together is a refusal
    // rather than one silently winning — the rule the settings patch
    // has.
    if (provider !== undefined && ephemeris !== undefined) {
      throw new TypeError(
        'provider and ephemeris each name the ephemeris to compute with; give one of them',
      );
    }
    const chain = ephemerisChain(ephemeris, testProvider);
    const [info, positions, thrown] = describeProvider(provider);
    this.#thrown = thrown;
    const settled = clean({
      flags: 0,
      profile,
      settingsJson: settings === undefined ? undefined : JSON.stringify(settings),
      locale,
      layoutsJson: layouts === undefined ? undefined : JSON.stringify(layouts),
      dashasJson: dashaSystems === undefined ? undefined : JSON.stringify(dashaSystems),
    });
    this.#inner = guarded(null, () => open(chain, settled, info, positions));

    // The areas, built once and never rebuilt: each holds the one way in
    // and nothing else, so a consumer may destructure one and keep it
    // (ADR-0030, `03-design/surface-areas.md`).
    const reach = (run) => this.#call(() => run(this.#inner));
    /** The calendars, and the fixed day they share. */
    this.calendar = new CalendarArea(reach);
    /** The scales, the zones and what separates them. */
    this.time = new TimeArea(reach);
    /** The locale, its messages and the scripts they are in. */
    this.intl = new IntlArea(reach);
    /** The catalogue's keys and their packed ids. */
    this.keys = new KeysArea(reach);
    /** The coordinate conventions a request is expressed in. */
    this.frame = new FrameArea(reach);
    /** A chart founded at an instant and a place. */
    /** Charts, and the layouts they are drawn in. */
    this.chart = new ChartArea(
      reach,
      registeredIds(this.#inner, 'chart_layout', layouts),
      registeredIds(this.#inner, 'dasha_system', dashaSystems),
    );
    /** A day, or a run of days, with its limbs. */
    this.almanac = new AlmanacArea(reach);
    this.#engine = new Engine(reach);
  }

  /**
   * The engine's own operations, beyond the eight the SDK names.
   *
   * **Not `ephemeris`**: `engine` says *this particular engine, not the
   * portable contract*, so a consumer reading their own code sees the
   * difference between a call that survives changing provider and one
   * that does not (ADR-0030).
   *
   * Throws when the context has no ephemeris, or when the one it has
   * describes nothing of its own — asked now rather than at the first
   * call, so a caller learns it where they can act on it.
   */
  get engine() {
    // Reading the manifest is what asks the question.
    this.#engine.manifestJson;
    return this.#engine;
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
    return hex(hash.bytes);
  }

  /**
   * Positions over a grid of instants and bodies, completed into the
   * frame asked for.
   *
   * **On the context and not in an area**, because an operation whose
   * name is its own area's name is a root operation: it is the SDK's one
   * primitive over the port, and every area above is built on it
   * (`03-design/surface-areas.md`).
   *
   * @param {object} request
   * @param {readonly number[]|Float64Array} request.instants Julian days
   * @param {readonly string[]} request.bodies the bodies, by key
   * @param {string} [request.scale] `UT1` or `TT`; `UT1` by default
   * @param {object} [request.frame] a frame; the canonical one by default
   * @param {boolean} [request.speeds] whether speeds are wanted
   * @param {object} [request.observer] the place a topocentric frame needs
   */
  positions(request) {
    const frame = request.frame ?? this.frame.canonical();
    const bytes = this.#call(() =>
      this.#inner.positions(
        clean({
          scale: request.scale ?? 'UT1',
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

/**
 * One entry of an ephemeris chain, normalised: either a name of the
 * SDK's own or a loaded adapter.
 *
 * @typedef {'NONE'|'BUILTIN'|'TEST'|{ plugin: string, config?: object }} EphemerisChoice
 */

/**
 * The chain as a list, however it was written.
 *
 * One entry is a list of one. A caller who wants a fallback writes the
 * fallback down, which is what ADR-0029 means by never automatic.
 *
 * @param {EphemerisChoice|readonly EphemerisChoice[]|undefined} ephemeris
 * @param {boolean} testProvider the older spelling of `'TEST'`
 * @returns {readonly EphemerisChoice[]}
 */
/**
 * Bytes as the native half takes them: a `Uint8Array` as it is (a Node
 * `Buffer` is one), any other view or an `ArrayBuffer` over the same
 * memory, and an array of byte values copied. Written without `Buffer`,
 * which a browser does not have.
 *
 * @param {Uint8Array|ArrayBuffer|ArrayBufferView|number[]} value
 * @param {string} field what the value was passed as, for the refusal
 * @returns {Uint8Array}
 */
function bytesOf(value, field) {
  if (value instanceof Uint8Array) return value;
  if (ArrayBuffer.isView(value)) {
    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  }
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  if (Array.isArray(value)) return Uint8Array.from(value);
  throw new TypeError(
    `${field}: expected bytes (a Uint8Array, an ArrayBuffer or an array of byte values); got ${
      value === null ? 'null' : typeof value
    }`,
  );
}

/** Bytes as lower-case hex, two digits each. */
function hex(bytes) {
  let out = '';
  for (const byte of bytes) out += byte.toString(16).padStart(2, '0');
  return out;
}

function ephemerisChain(ephemeris, testProvider) {
  // One rule, written once: a named ephemeris wins, and the older flag
  // decides only when none was named (ADR-0028).
  if (ephemeris === undefined) {
    return [testProvider ? 'TEST' : 'NONE'];
  }
  const entries = Array.isArray(ephemeris) ? ephemeris : [ephemeris];
  if (entries.length === 0) {
    throw new TypeError('an ephemeris chain of none names nothing; give an entry or omit it');
  }
  for (const entry of entries) {
    const named = typeof entry === 'string';
    if (!named && (entry === null || typeof entry.plugin !== 'string')) {
      throw new TypeError(
        "an ephemeris is a name ('NONE', 'BUILTIN', 'TEST') or an adapter's descriptor, " +
          "which carries a `plugin` path; got " +
          JSON.stringify(entry),
      );
    }
  }
  return entries;
}

/**
 * Opens the context on the first entry of the chain that answers.
 *
 * **A chain of one is not a chain, so nothing is caught for it.** Found
 * by an existing test: a bad *profile* is not an ephemeris failure, and
 * catching it to try the next entry replaced a refusal carrying its
 * status, its field and its hint with a bare "nothing could be opened".
 * With one entry there is no next entry, so the refusal is the refusal.
 *
 * With more than one, **every refusal is kept and reported together**,
 * because a chain that said only why its last entry failed would hide
 * the one the caller actually wanted.
 */
function open(chain, settled, info, positions) {
  if (chain.length === 1) {
    return attempt(chain[0], settled, info, positions);
  }
  const refusals = [];
  for (const entry of chain) {
    try {
      return attempt(entry, settled, info, positions);
    } catch (refusal) {
      refusals.push(`${typeof entry === 'string' ? entry : entry.plugin}: ${refusal.message}`);
    }
  }
  throw new Error(
    `no ephemeris in the chain could be opened:\n  ${refusals.join('\n  ')}`,
  );
}

/** One entry of the chain, opened. */
function attempt(entry, settled, info, positions) {
  if (typeof entry === 'string') {
    return new native.Context({ ...settled, ephemeris: entry }, info, positions);
  }
  // A plugin is a shared library, which a wasm module cannot open
  // (ADR-0029). Asked of the loaded native half rather than of a flag, so
  // it is true of whatever build this is; and refused here, as an entry
  // that cannot open, so a chain written for both — a plugin, then
  // 'BUILTIN' — falls back in a browser as it does where the file is
  // missing.
  if (typeof native.Provider !== 'function') {
    throw new TypeError(
      `\`ephemeris.plugin\` opens a shared library, which this build (${buildInfo.target}) ` +
        "cannot; give `provider`, an ephemeris written in JavaScript, or 'BUILTIN'",
    );
  }
  // The context takes its own reference to the adapter, so the handle
  // this loads is freed at once: what keeps the library loaded is the
  // context, and a consumer never holds either.
  const loaded = new native.Provider(entry.plugin, JSON.stringify(entry.config ?? {}));
  try {
    return native.Context.newWithProvider({ ...settled, ephemeris: 'NONE' }, loaded);
  } finally {
    loaded.dispose();
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
 * resolution is `DEFINED`, which is what a date a caller states means.
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
  resolution: 'DEFINED',
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
 * `TIME_UNKNOWN_FALLBACK` warning, and under `SUNRISE` it needs the
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
  kind: 'IANA',
  offsetSeconds: 0,
  longitudeDeg: 0,
  zone: name,
});

/** A fixed offset from UTC, in seconds east. */
export const fixedZone = (offsetSeconds) => ({
  kind: 'FIXED',
  offsetSeconds,
  longitudeDeg: 0,
});

/**
 * Local mean time at a longitude east of Greenwich, which is what a chart
 * from before the zone existed is cast in.
 */
export const localMeanZone = (longitudeDeg) => ({
  kind: 'LOCAL_MEAN',
  offsetSeconds: 0,
  longitudeDeg,
});

export { decodeCharts, decodeIntlRender, decodePanchanga, decodePositions } from './blob.js';
export { entityForms, messages } from './messages.js';
export * from './catalogue.js';
export * from './records.js';
