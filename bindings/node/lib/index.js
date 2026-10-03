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
  RituById,
  AyanamshaById,
  BodyById,
  CHART_ASHTAKAVARGA,
  CHART_ASPECTS,
  CHART_BHAVA_BALA,
  CHART_DASHA_PHALA,
  CHART_HOUSES,
  CHART_JAIMINI,
  CHART_OUTER,
  CHART_POINTS,
  CHART_SHADBALA,
  CHART_STATE,
  CHART_VAISESHIKAMSA,
  CHART_VIMSHOPAKA,
  CONTEXT_TEST_PROVIDER,
  PANCHANGA_ECLIPSES,
  PANCHANGA_NEPAL_SAMBAT,
  PANCHANGA_YEARS,
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
  SunrisesById,
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
  BrahmaRuleById,
  BrahmaOutcomeById,
  AshtakavargaGoodFromById,
  FruitionById,
  GocharFromById,
  HitKindById,
  ReckoningById,
  SectById,
  SectRuleById,
  TermsById,
  TriplicitiesById,
  AccidentById,
  PartileById,
  SiegeById,
  PlaceReadingById,
  FortuneRuleById,
  LotById,
  PtolemaicAspectById,
  RadicalGroundById,
  ApplicationKindById,
  ImpedimentKindById,
  WayById,
  MotionById,
  WesternAspectById,
  AspectPhaseById,
  KakshyaLordById,
  SarvaStandingById,
  GocharVerdictById,
  NodeObstructionById,
  NodeVedhaById,
  VimshopakaScoringById,
  DashaSystemById,
  VargaById,
  SDK_VERSION,
  TithiById,
  TimeScaleById,
  VaraById,
  YogaById,
  Ephemeris,
  KootaById,
  VarnaById,
  YoniById,
  GanaById,
  NadiById,
  VashyaRelationById,
  YoniRelationById,
  MaitriRelationById,
  BhakootDoshaById,
  DhinamRuleById,
  RajjuById,
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
   * Jaimini's significators (`jaimini: true`): the karakamsha, with every
   * graha's house from it in the rasi chart and the navamsha, and the Brahma
   * graha under the settings' `jaimini` group, or why the rule found none;
   * `null` unless asked for.
   */
  get jaimini() {
    return jaiminisOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The transits read against this chart (`gochar: { instants, from }`), one
   * reading an instant in the order asked; empty unless asked for
   * (`03-design/gochar.md`).
   *
   * Each is `{ instant, reference: { from, sign }, rules: { nodeVedha,
   * nodeObstruction }, grahas }`, the houses counted from the natal Moon's
   * sign by Phaladeepika ch. 26 v. 1 unless `from: 'LAGNA'` asked
   * otherwise. Each graha is `{ graha, transit: { sign, degrees }, house,
   * goodHouse, vedhaHouse, obstructedBy, verdict, fruition, fruitfulNow }`:
   * `vedhaHouse` is `null` where nothing can obstruct it, and
   * `obstructedBy` names the grahas standing there, the verses' exemptions
   * left out. With `gochar: { ..., ashtakavarga: true }` each reading's
   * `ashtakavarga` judges the seven by the natal bindus
   * (`03-design/gochar-ashtakavarga.md`); `null` otherwise.
   */
  get gochar() {
    return gocharsOf(this.#batch)[this.#index] ?? [];
  }

  /**
   * The transit hit list (`hits: { from, to, grahas, kinds, points,
   * aspects, orbDeg }`): every ingress, station and aspect to a natal point
   * of the window, sorted by instant, then graha, then kind; empty unless
   * asked for (`03-design/transit-hit-list.md`). The sky is searched once
   * for the whole batch.
   *
   * Each is `{ instant, graha, event }`, the event tagged by `kind`:
   * `{ kind: 'SIGN_INGRESS', into, motion }` with `into` a sign,
   * `{ kind: 'NAKSHATRA_INGRESS', into, motion }` with `into` a nakshatra,
   * `{ kind: 'STATION', turns }`, or `{ kind: 'ASPECT', to, angle, phase,
   * motion }` with `to` either `{ point: 'GRAHA', graha }` or
   * `{ point: 'LAGNA' }` — the spelling `hits.points` takes back.
   */
  get hits() {
    const d = this.#batch.decoded;
    const counts = d.cast.hitCount;
    const starts = startsOf(counts);
    if (starts[counts.length] !== d.hits.instant.length) {
      throw new Error(
        `hits has ${d.hits.instant.length} rows and cast.hit_count sums to ${starts[counts.length]}; ` +
          'it is every chart\'s list, concatenated',
      );
    }
    const from = starts[this.#index];
    return Array.from({ length: counts[this.#index] ?? 0 }, (_, k) => hitOf(d.hits, from + k));
  }

  /**
   * Sade Sati and Saturn's smaller spells (`sadeSati: { from, to,
   * countedFrom, reckoning, spells }`): every period reaching into the
   * window, **whole**, however far its bounds fall outside it; `null`
   * unless asked for (`03-design/sade-sati.md`). Saturn is searched once
   * for the whole batch.
   *
   * It is `{ reference: { from, sign }, reckoning, sadeSati, spells }`:
   * each Sade Sati is `{ phases }`, the rising (12th), peak (1st) and
   * setting (2nd) spells in order, and each spell `{ house, visits }`, a
   * visit `{ from, to }` in UTC Julian days, half-open, a retrograde
   * re-entry a visit of its own (C148). A bound past the ephemeris's
   * coverage is `null`.
   */
  get sadeSati() {
    return sadeSatisOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The chart read as KP (`kp: { number, clock, anyAyanamsha }`): its
   * cusps and planets to the sub-sub lord, its significators in Reader
   * VI's order and the ruling planets of its moment, under the settings'
   * `kp` group; `null` unless asked for (`03-design/kp.md`).
   *
   * It is `{ chart, significators, ruling }`. A longitude and a lord's
   * span are integers in **nanoarcseconds** of the sidereal zodiac, exact
   * (divide by `3.6e12` for degrees). For a horary `number` the cusps are
   * the number's and the ruling planets still the moment's own.
   */
  get kp() {
    return kpsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The seven planets' essential dignities (`dignities: { sectRule, rules,
   * scores }`), with the chart's sect and everything that made them;
   * `null` unless asked for (`03-design/essential-dignities.md`).
   *
   * It is `{ sect, sectRule, rules: { terms, triplicities }, scores,
   * planets }`, the planets in the Chaldean order, each `{ planet,
   * longitudeDeg, dignity, peregrine, score }`. `terms` is `'TABLE'` when
   * the request gave a table of its own.
   */
  get dignities() {
    return dignitiesOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * Both halves of Lilly's table (`fortitudes: { dignities, rules, scores
   * }`): the essential dignities, which `dignities` also reads, and the
   * accidental fortitudes; `null` unless asked for
   * (`03-design/essential-dignities.md` §Accidental fortitudes).
   *
   * It is `{ dignities, sky, rules, scores, planets, almutens }`. `sky` is
   * what the lines were read from, `{ houses, cuspsDeg, ascendantDeg,
   * midheavenDeg, speedsDegPerDay, northNodeDeg, regulusDeg, spicaDeg,
   * algolDeg }`; `rules` and `scores` are what was applied, each a
   * request's own record. The planets are in the Chaldean order, each `{
   * planet, house, accidents, fortitude, debility, net }`, every accident
   * `{ accident, points }`, and `net` Lilly's sum of both halves.
   * `almutens` is `{ rules, fortuneDeg, figure, places, houses }`, each
   * almuten `{ totals, almutens, partakers }` (§The almuten).
   */
  get fortitudes() {
    return fortitudesOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * Valens's fourteen lots (`lots: { sectRule, fortune }`), with the
   * chart's sect and the rules they were read under; `null` unless asked
   * for (`03-design/hellenistic-lots.md`).
   *
   * It is `{ sect, request: { sectRule, fortune }, fortuneReversed, lots
   * }`, the lots in the catalogue's order, each `{ lot, place: {
   * longitudeDeg, sign, lord, house } }`. `fortuneReversed` says whether
   * Fortune was counted from the Moon to the Sun (C221).
   */
  get lots() {
    return lotsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * Lilly's considerations before judgement (`considerations: {
   * moonLateFromDeg, orbsDeg }`), read from the chart's fortitudes and its
   * planetary hour; `null` unless asked for
   * (`03-design/hellenistic-considerations.md`).
   *
   * It is `{ radicality, ascendant, moon, seventh, saturnHouse,
   * saturnRetrograde, ascendantLordCombust, rules }`, each clause with the
   * facts it rests on and none folded into a verdict. The Moon's `course`
   * holds `next`, her first perfection before she leaves her sign, and
   * `withinOrb`, the first already within the moieties; either is `null`
   * when she is void by that reading (C230).
   */
  get considerations() {
    return considerationsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * Whether a horary matter is brought to pass (`perfection: { house }` or
   * `{ querent, quesited }`, Lilly pp. 107–113 and 125–127), weighed on the
   * chart's fortitudes and searched on the ephemeris; `null` unless asked
   * for (`03-design/hellenistic-perfection.md`).
   *
   * It is `{ querent, quesited, application, separation, impediments,
   * translations, collections, ways, horizonDays, rules }`: the relations
   * between the two significators with the facts each rests on, and in
   * `ways.held` which of the seven ways of perfection the figure holds,
   * never a verdict.
   */
  get perfection() {
    return perfectionsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The birth read through its progressions (`progressions: { at, rate,
   * year, angles, direction, contacts }`, Leo's *The Progressed
   * Horoscope*); `null` unless asked for
   * (`03-design/western-progressions.md`).
   *
   * It is `{ progressed, directed, contacts }`. `progressed` is `{ life,
   * sky, armcDeg, angles: { ascendantDeg, midheavenDeg }, grahas }`, the
   * planets of the chart founded at the instant of sky; `directed` is `{
   * life, arcDeg, ascendantDeg, midheavenDeg, planets }`; both are `null`
   * when no `at` was asked. `contacts` lists each `{ life, sky, graha, to,
   * angle, motion }` in the order they fall due, `to` spelled as a hit's,
   * or is `null` when no window was asked.
   */
  get progressions() {
    return progressionsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The Western aspect table (`westernAspects: { aspects, orbs }`, Leo's
   * nine under his orbs by default, C240); `null` unless asked for
   * (`03-design/western-aspects.md`).
   *
   * Each row is `{ first, second, aspect, apartDeg, fromExactDeg, orbDeg,
   * applying }`, the pair in catalogue order over the seven planets and,
   * when `outerPlanets` placed them, the outer three.
   */
  get westernAspects() {
    return westernAspectsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The Western aspects between this chart and the partner's
   * (`synastry: { partner: { instant, place, utcOffsetSeconds }, aspects,
   * orbs, lagna, zodiac }`, Leo's nine under his orbs in the tropical
   * zodiac by default); `null` unless asked for
   * (`03-design/western-synastry.md`).
   *
   * Each row is `{ first, second, aspect, apartDeg, fromExactDeg, orbDeg }`,
   * closest first: `first` a point of this chart and `second` one of the
   * partner's, each `{ point: 'GRAHA', graha }` or `{ point: 'LAGNA' }`.
   */
  get synastry() {
    return synastriesOf(this.#batch).aspects[this.#index] ?? null;
  }

  /**
   * The parallels between this chart and the partner's
   * (`synastry: { partner, parallels: { orbDeg } }`), closest first;
   * `null` unless the synastry record asked for them
   * (`03-design/western-declinations.md`).
   *
   * Each row is `{ first, second, contrary, apartDeg, orbDeg }`, each side
   * `{ point: 'GRAHA', graha }` or `{ point: 'LAGNA' }`, `contrary` true when
   * the two stand on opposite sides of the equator (C243).
   */
  get synastryParallels() {
    return synastriesOf(this.#batch).parallels[this.#index] ?? null;
  }

  /**
   * The antiscia between this chart and the partner's
   * (`synastry: { partner, antiscia: {} }`), closest first: a planet of
   * this chart whose tropical longitude and one of the partner's sum to
   * 180°, or 0° for the contrantiscion, within the orb read at the
   * conjunction, Lilly's moieties by default (C244); `null` unless the
   * synastry record asked for them (`03-design/western-antiscia.md`).
   *
   * Each row is `{ first, second, contrary, apartDeg, orbDeg }`, `first`
   * this chart's planet and `second` the partner's.
   */
  get synastryAntiscia() {
    return synastriesOf(this.#batch).antiscia[this.#index] ?? null;
  }

  /**
   * The equal distances between this chart and the partner's
   * (`synastry: { partner, midpoints: {} }`), closest first: a planet of
   * one chart within the orb of the axis through two of the other's, on
   * the shorter arc's midpoint or opposite it, 0.5° by default (C245,
   * C246); `null` unless the synastry record asked for them
   * (`03-design/western-midpoints.md`).
   *
   * Each row is `{ first, second, middle, partnersPair, far, distanceDeg,
   * fromAxisDeg, orbDeg }`: `partnersPair` is true when the pair is the
   * partner's and `middle` this chart's planet.
   */
  get synastryMidpoints() {
    return synastriesOf(this.#batch).midpoints[this.#index] ?? null;
  }

  /**
   * The composite of this chart and the partner's
   * (`synastry: { partner, composite: true }`): each planet at the near
   * midpoint of its two places, moving at the mean of its two speeds, the
   * midheaven at the near midpoint of the two, and the lagna at theirs,
   * turned by 180° when it stood before the midheaven (C247); in the
   * synastry's zodiac; `null` unless asked
   * (`03-design/western-composites.md`).
   *
   * It is `{ planets, lagnaDeg, midheavenDeg, lagnaTurned }`, each planet
   * `{ graha, longitudeDeg, speedDegPerDay }` in this chart's order.
   */
  get synastryComposite() {
    return synastriesOf(this.#batch).composites[this.#index] ?? null;
  }

  /**
   * The Davison birth of this chart and the partner
   * (`synastry: { partner, davison: true }`): the mean of the two
   * instants, of the two latitudes and altitudes, of the two longitudes
   * the shorter way round, and of the two clocks, this chart's read on
   * the request's (C248); `null` unless asked
   * (`03-design/western-composites.md`).
   *
   * It is `{ instant, place, utcOffsetSeconds }`, the shape a chart
   * request and a synastry's `partner` take, so it founds a chart as a
   * birth does: `ctx.chart.found({ ...chart.synastryDavison })`.
   */
  get synastryDavison() {
    return synastriesOf(this.#batch).davisons[this.#index] ?? null;
  }

  /**
   * The chart's distances from the equator (`parallels: { orbDeg }` asks
   * for them with the parallels); `null` unless asked for
   * (`03-design/western-declinations.md`).
   *
   * `{ obliquityDeg, grahas, lagnaDeg, midheavenDeg }`, each degrees north:
   * the planets as `{ graha, declinationDeg }` in the catalogue's order,
   * and the angles as the Sun's at their degree (Leo, p. 141).
   */
  get declinations() {
    return declinationsOf(this.#batch).declinations[this.#index] ?? null;
  }

  /**
   * The parallels among the chart's planets, closest first: each pair the
   * same distance from the equator within the orb (Leo's 1° by default), on
   * either side of it (C243); `null` unless `parallels` asked.
   *
   * Each row is `{ first, second, contrary, apartDeg, orbDeg }`, `contrary`
   * true when the two stand on opposite sides of the equator.
   */
  get parallels() {
    return declinationsOf(this.#batch).parallels[this.#index] ?? null;
  }

  /**
   * The chart's antiscia (`antiscia: {}` asks for them): each planet's
   * reflection about the solstices and the equinoxes (Lilly, *Christian
   * Astrology*, pp. 90–92), and the pairs standing in one within the
   * orbs, read at the conjunction, Lilly's moieties by default (C244);
   * `null` unless asked (`03-design/western-antiscia.md`).
   *
   * `{ points, pairs, unpaired }`: each point `{ graha, antiscionDeg,
   * contrantiscionDeg }` in tropical degrees, each pair `{ first, second,
   * contrary, apartDeg, orbDeg }` closest first, `contrary` true for the
   * contrantiscion, and `unpaired` the planets the orbs give none.
   */
  get antiscia() {
    return antisciaOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The chart's equal distances (`midpoints: {}` asks for them): each
   * planet within the orb of the axis through two others' midpoint, 0.5°
   * by default (C245), so equally distant from the two, on the shorter
   * arc's midpoint or opposite it (C246; Leo, *How to Judge a Nativity*,
   * pp. 47–48); closest first, `null` unless asked
   * (`03-design/western-midpoints.md`).
   *
   * Each row is `{ first, second, middle, far, distanceDeg, fromAxisDeg,
   * orbDeg }`: the pair in catalogue order, the planet between, `far` true
   * on the point opposite the shorter arc's midpoint.
   */
  get midpoints() {
    return midpointsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The chart's Western houses (`westernHouses: {}` asks for them): the
   * cusps of Placidus, the division Leo's figures are cast in, unless the
   * request or the profile names another (C249), and each planet's house,
   * flagged when Leo reads it with the ascendant (C250); `null` unless
   * asked (`03-design/western-houses.md`).
   *
   * @returns {object|null}
   */
  get westernHouses() {
    return westernHousesOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The chart's harmonic chart (`harmonic: { number: 9 }` asks for it):
   * each planet, the ascendant and the midheaven at its longitude
   * multiplied, in its equal house from the harmonic ascendant (C254),
   * and every pair meeting within the orb, 12° by default (C252), closest
   * first; `null` unless asked (`03-design/western-harmonics.md`).
   *
   * @returns {object|null}
   */
  get harmonic() {
    return harmonicsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The chart matched with a partner's birth by the Ashta Koota of *Muhurta
   * Chintamani* VI.21–34 (`matching: { partner, partnerRole: 'BRIDE' }`
   * asks for it, the chart on the other side): each koota's points and
   * what it read, in the verse's order, and the total out of 36. Never a
   * verdict: the doshas and their exceptions are clauses; `null` unless
   * asked (`03-design/matching.md`).
   *
   * @returns {object|null}
   */
  get matching() {
    return matchingsOf(this.#batch)[this.#index] ?? null;
  }

  /**
   * The chart matched with the same partner by the ten considerations of
   * *Kalaprakasika* XIII (`matching` asks for both systems): whether each
   * agrees and what it read, in the chapter's order, how many agree, how
   * many of the chief five, and the p. 76 exception's clauses. Never a
   * verdict: "at least five" is the reader's to apply; `null` unless asked
   * (`03-design/matching.md`).
   *
   * @returns {object|null}
   */
  get porutham() {
    return poruthamsOf(this.#batch)[this.#index] ?? null;
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
    return this.#placed(this.#batch.decoded.grahas, this.#batch.decoded.grahaCount);
  }

  /**
   * Uranus, Neptune and Pluto, placed as the grahas are, or an empty
   * list unless `outerPlanets: true` asked for them. The section holds
   * the same number a chart, so the batch's rows divided by its charts
   * is this chart's count.
   */
  get outer() {
    const d = this.#batch.decoded;
    const charts = d.cast.instant.length;
    const count = charts === 0 ? 0 : d.outer.graha.length / charts;
    if (!Number.isInteger(count)) {
      throw new Error(`outer has ${d.outer.graha.length} rows for ${charts} charts`);
    }
    return this.#placed(d.outer, count);
  }

  /** One chart's stride of a placed-bodies section, a row a body. */
  #placed(g, count) {
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
  #muhurta = undefined;
  #festivals = undefined;
  #years = undefined;
  #eclipses = undefined;
  #nepalSambat = undefined;

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
   * The muhurta search the request asked for over these days
   * (`03-design/muhurta-at-the-boundary.md`), or `null` when it asked for
   * none: the windows judged clause by clause, the days the season closed,
   * and what computed it. Catalogue members are full keys, as every other
   * accessor gives them, so a clause reads straight back into a request's
   * rules. Parsed once, and frozen to its leaves.
   *
   * @returns {object|null}
   */
  get muhurta() {
    if (this.#muhurta === undefined) this.#muhurta = muhurtaFrom(this.decoded.muhurta);
    return this.#muhurta;
  }

  /**
   * The days the festival rules the request asked for fall on over these
   * days (`03-design/festival-rules.md` §7), or `null` when it asked for
   * none: each observance with the case between its tithi's two days and
   * the guard that decided, the occurrences no day could be given to,
   * and what computed it. Parsed once, and frozen to its leaves.
   *
   * @returns {object|null}
   */
  get festivals() {
    if (this.#festivals === undefined) this.#festivals = festivalsFrom(this.decoded.festivals);
    return this.#festivals;
  }

  /**
   * The lunar years these days fall in
   * (`03-design/calendar-indian-lunisolar.md` §10), or `null` when the
   * request did not ask with `years: true`: `{ value, provenance }`, each
   * year with the samvatsara it carries, its Vikrama and Shaka numbers,
   * its bounds from one Chaitra Shukla Pratipada's sunrise to the next,
   * the Jovian years that ran in it and the one it expunged. Parsed once,
   * and frozen to its leaves.
   *
   * @returns {object|null}
   */
  get years() {
    if (this.#years === undefined) this.#years = envelopeFrom(this.decoded.years);
    return this.#years;
  }

  /**
   * The eclipses of these days (`03-design/eclipses.md`), or `null` when
   * the request did not ask with `eclipses: true`: `{ value, provenance }`,
   * `value` holding `lunar` and `solar`, each eclipse with `here`, how
   * the almanac's place sees it, whose `seen` is `null` where the place
   * does not. Parsed once, and frozen to its leaves.
   *
   * @returns {object|null}
   */
  get eclipses() {
    if (this.#eclipses === undefined) this.#eclipses = envelopeFrom(this.decoded.eclipses);
    return this.#eclipses;
  }

  /**
   * Each day's Nepal Sambat date (`03-design/calendar-indian-lunisolar.md`
   * §11), or `null` when the request did not ask with `nepalSambat: true`:
   * `{ value, provenance }`, `value` one date a day in the days' order,
   * each `{ year, month, kind, paksha }`: the year, which opens at
   * Kachhala's first day; the month, 1 for Kachhala (amanta Kartika) to 12
   * for Kaula; `'ADHIKA'` for Anala; and the half, `'paksha.SHUKLA'`
   * (thwa) or `'paksha.KRISHNA'` (ga). `sdk.calendar.nepalSambatDate`
   * says one. Parsed once, and frozen to its leaves.
   *
   * @returns {object|null}
   */
  get nepalSambat() {
    if (this.#nepalSambat === undefined) this.#nepalSambat = envelopeFrom(this.decoded.nepalSambat);
    return this.#nepalSambat;
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

  /**
   * Which season the day falls in, under `panchanga.ritu`: by default the
   * season of the sidereal solar month the day belongs to, its first day
   * placed by `panchanga.solar_month_start` (`03-design/ritu-measured.md`).
   */
  get ritu() {
    return RituById.get(this.#batch.decoded.days.ritu[this.#index]) ?? 'unknown';
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
      sunrises: SunrisesById.get(c.sunrises[i]) ?? 'unknown',
      ends: Object.freeze({ ghati: c.endsGhati[i], pala: c.endsPala[i], vipala: c.endsVipala[i] }),
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
          (request.aspects === true ? CHART_ASPECTS : 0) |
          (request.points === true ? CHART_POINTS : 0) |
          (request.houses === true ? CHART_HOUSES : 0) |
          (request.ashtakavarga === true ? CHART_ASHTAKAVARGA : 0) |
          (request.vimshopaka === true ? CHART_VIMSHOPAKA : 0) |
          (request.vaiseshikamsa === true ? CHART_VAISESHIKAMSA : 0) |
          (request.shadbala === true ? CHART_SHADBALA : 0) |
          (request.bhavaBala === true ? CHART_BHAVA_BALA : 0) |
          (request.dashaPhala === true ? CHART_DASHA_PHALA : 0) |
          (request.jaimini === true ? CHART_JAIMINI : 0) |
          (request.outerPlanets === true ? CHART_OUTER : 0) |
          (request.state === true ? CHART_STATE : 0),
        vargas: catalogueKeys(request.vargas, 'vargas', 'Varga'),
        dashas: dashaIds(request.dashas, this.#dashas),
        drawings: drawingBits(request.drawings, this.#registered),
        themeJson: themeJson(request.theme),
        rulesJson: rulesJson(request.rules),
        interpretJson: interpretJson(request.interpret),
        varshaJson: varshaJson(request.varsha),
        gocharJson: gocharJson(request.gochar),
        hitsJson: recordJson(
          request.hits,
          'hits',
          'a hit list request record, e.g. { from: 2460676.5, to: 2461041.5, grahas: ["SATURN"] }',
        ),
        sadeSatiJson: recordJson(
          request.sadeSati,
          'sadeSati',
          'a Sade Sati request record, e.g. { from: 2460676.5, to: 2464329, reckoning: "SIGN" }',
        ),
        kpJson: recordJson(request.kp, 'kp', 'a KP request record, e.g. { number: 74 }'),
        dignitiesJson: recordJson(
          request.dignities,
          'dignities',
          'a dignities request record, e.g. { sectRule: "HORIZON", rules: { terms: "EGYPTIAN" } }',
        ),
        fortitudesJson: recordJson(
          request.fortitudes,
          'fortitudes',
          'a fortitudes request record, e.g. { rules: { beamsDeg: 15 }, scores: { regulus: 6 }, almuten: { place: "SIGN" } }',
        ),
        lotsJson: recordJson(
          request.lots,
          'lots',
          'a lots request record, e.g. { fortune: "REVERSED_WHILE_MOON_UP" }',
        ),
        considerationsJson: recordJson(
          request.considerations,
          'considerations',
          'a considerations request record, e.g. { moonLateFromDeg: 25 }',
        ),
        perfectionJson: recordJson(
          request.perfection,
          'perfection',
          'a perfection request record, e.g. { house: 7 } or { querent: "VENUS", quesited: "MARS" }',
        ),
        progressionsJson: recordJson(
          request.progressions,
          'progressions',
          'a progressions request record, e.g. { at: 2460676.5 } or { contacts: { from, to } }',
        ),
        westernAspectsJson: recordJson(
          request.westernAspects,
          'westernAspects',
          'a western aspects request record, e.g. {} or { aspects: ["TRINE", "SQUARE"] }',
        ),
        parallelsJson: recordJson(
          request.parallels,
          'parallels',
          'a parallels request record, e.g. {} or { orbDeg: 1 }',
        ),
        antisciaJson: recordJson(
          request.antiscia,
          'antiscia',
          'an antiscia request record, e.g. {} or { orbs: { model: "LEO" } }',
        ),
        westernHousesJson: recordJson(
          request.westernHouses,
          'westernHouses',
          "a Western houses request record, e.g. {} or { system: 'house_system.KOCH' }",
        ),
        harmonicJson: recordJson(request.harmonic, 'harmonic', 'a harmonic request record, e.g. { number: 9 }'),
        matchingJson: recordJson(
          request.matching,
          'matching',
          "a matching request record, e.g. { partner: { instant: 2447892.5, place: { latitude, longitude, altitude } }, partnerRole: 'BRIDE' }",
        ),
        midpointsJson: recordJson(
          request.midpoints,
          'midpoints',
          'a midpoints request record, e.g. {} or { orbDeg: 1 }',
        ),
        synastryJson: recordJson(
          request.synastry,
          'synastry',
          'a synastry request record, e.g. { partner: { instant: 2460676.5, place: { latitude, longitude, altitude } } }',
        ),
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

/** Each batch's Jaimini significators, decoded once however many charts read them. */
const JAIMINIS = new WeakMap();

/** Every chart's Jaimini significators in a batch; empty when none were asked for. */
function jaiminisOf(batch) {
  let decoded = JAIMINIS.get(batch);
  if (decoded === undefined) {
    const c = batch.decoded.jaimini;
    const h = batch.decoded.jaiminiGrahas;
    const graha = (id) => GrahaById.get(id) ?? 'unknown';
    decoded = Array.from({ length: c.atmakaraka.length }, (_, chart) => {
      const houses = (column) =>
        Object.freeze(Array.from({ length: 9 }, (_, g) => column[chart * 9 + g]));
      const outcome = BrahmaOutcomeById.get(c.brahmaOutcome[chart]) ?? 'unknown';
      return Object.freeze({
        karakamsha: Object.freeze({
          atmakaraka: graha(c.atmakaraka[chart]),
          sign: RashiById.get(c.karakamsha[chart]) ?? 'unknown',
          inRasi: houses(h.inRasi),
          inNavamsha: houses(h.inNavamsha),
        }),
        brahma: Object.freeze({
          rule: BrahmaRuleById.get(c.brahmaRule[chart]) ?? 'unknown',
          countedFrom: RashiById.get(c.countedFrom[chart]) ?? 'unknown',
          qualified: Object.freeze(membersOf(c.qualified[chart], GrahaById)),
          graha: outcome === 'FOUND' ? graha(c.brahma[chart]) : null,
          passedFrom: c.passedFromPresent[chart] !== 0 ? graha(c.passedFrom[chart]) : null,
          none: outcome === 'FOUND' ? null : outcome,
        }),
        grahaArudhas: Object.freeze(
          Array.from({ length: 9 }, (_, g) => {
            const row = chart * 9 + g;
            return h.arudhaPresent[row] !== 0 ? (RashiById.get(h.arudha[row]) ?? 'unknown') : null;
          }),
        ),
      });
    });
    JAIMINIS.set(batch, decoded);
  }
  return decoded;
}

/** Each batch's transits, decoded once however many charts read them. */
const GOCHARS = new WeakMap();

/**
 * Every chart's transits in a batch: `gochar` holds a row a chart an
 * instant, fixed rather than ragged since the request settles how many
 * instants every chart gets, and `gochar_grahas` nine rows under each.
 *
 * @param {Charts} batch
 * @returns {object[][]}
 */
function gocharsOf(batch) {
  let decoded = GOCHARS.get(batch);
  if (decoded === undefined) {
    const c = batch.decoded.gochar;
    const g = batch.decoded.gocharGrahas;
    const a = batch.decoded.gocharAshtakavarga;
    const charts = batch.decoded.chartCount;
    const perChart = charts === 0 ? 0 : c.instant.length / charts;
    if (!Number.isInteger(perChart) || g.graha.length !== c.instant.length * 9) {
      throw new Error(
        `gochar has ${c.instant.length} rows and ${g.graha.length} grahas over ${charts} charts; ` +
          'it is every chart at every instant, nine grahas each',
      );
    }
    const judged = a.graha.length > 0;
    if (judged && a.graha.length !== c.instant.length * 7) {
      throw new Error(
        `gochar_ashtakavarga has ${a.graha.length} rows under ${c.instant.length} transits; ` +
          'it is seven under every one or none',
      );
    }
    const byBindus = (row) =>
      Object.freeze(
        Array.from({ length: 7 }, (_, k) => {
          const at = row * 7 + k;
          return Object.freeze({
            graha: graha(a.graha[at]),
            bindus: a.bindus[at],
            good: a.good[at] !== 0,
            kakshya: Object.freeze({
              index: a.kakshya[at],
              lord: KakshyaLordById.get(a.kakshyaLord[at]) ?? 'unknown',
            }),
            kakshyaBindu: a.kakshyaBindu[at] !== 0,
            sarva: a.sarva[at],
            sarvaStanding: SarvaStandingById.get(a.sarvaStanding[at]) ?? 'unknown',
          });
        }),
      );
    const graha = (id) => GrahaById.get(id) ?? 'unknown';
    const sign = (id) => RashiById.get(id) ?? 'unknown';
    const reading = (row) =>
      Object.freeze({
        instant: c.instant[row],
        reference: Object.freeze({
          from: GocharFromById.get(c.countedFrom[row]) ?? 'unknown',
          sign: sign(c.reference[row]),
        }),
        rules: Object.freeze({
          nodeVedha: NodeVedhaById.get(c.nodeVedha[row]) ?? 'unknown',
          nodeObstruction: NodeObstructionById.get(c.nodeObstruction[row]) ?? 'unknown',
          ashtakavargaGoodFrom: AshtakavargaGoodFromById.get(c.ashtakavargaGoodFrom[row]) ?? 'unknown',
        }),
        grahas: Object.freeze(
          Array.from({ length: 9 }, (_, k) => {
            const at = row * 9 + k;
            return Object.freeze({
              graha: graha(g.graha[at]),
              transit: Object.freeze({ sign: sign(g.sign[at]), degrees: g.degrees[at] }),
              house: g.house[at],
              goodHouse: g.goodHouse[at] !== 0,
              vedhaHouse: g.vedhaHouse[at] === 0 ? null : g.vedhaHouse[at],
              obstructedBy: Object.freeze(membersOf(g.obstructedBy[at], GrahaById)),
              verdict: GocharVerdictById.get(g.verdict[at]) ?? 'unknown',
              fruition: FruitionById.get(g.fruition[at]) ?? 'unknown',
              fruitfulNow: g.fruitfulNow[at] !== 0,
            });
          }),
        ),
        ashtakavarga: judged ? byBindus(row) : null,
      });
    decoded = Array.from({ length: charts }, (_, chart) =>
      Object.freeze(Array.from({ length: perChart }, (_, k) => reading(chart * perChart + k))),
    );
    GOCHARS.set(batch, decoded);
  }
  return decoded;
}

/** Each batch's Sade Sati reports, decoded once however many charts read them. */
const SADE_SATIS = new WeakMap();

/**
 * Every chart's Sade Sati report in a batch: `sade_sati` holds a row a
 * chart, or none when none was asked, and `sade_sati_visits` each chart's
 * visits, ragged by `cast.sade_sati_visit_count` and numbered by `period`
 * — its Sade Satis first (houses 12, 1 and 2), then its smaller spells
 * (`03-design/sade-sati.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function sadeSatisOf(batch) {
  let decoded = SADE_SATIS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const charts = d.cast.sadeSatiVisitCount;
  const c = d.sadeSati;
  const v = d.sadeSatiVisits;
  if (c.reference.length === 0) {
    decoded = Object.freeze(Array.from({ length: charts.length }, () => null));
  } else {
    if (c.reference.length !== charts.length) {
      throw new Error(`sade_sati has ${c.reference.length} rows for ${charts.length} charts; it is one a chart or none`);
    }
    const starts = startsOf(charts);
    if (starts[charts.length] !== v.house.length) {
      throw new Error(
        `sade_sati_visits has ${v.house.length} rows and cast.sade_sati_visit_count sums to ${starts[charts.length]}; ` +
          "it is every chart's visits, concatenated",
      );
    }
    const bound = (jd) => (Number.isNaN(jd) ? null : jd);
    decoded = Object.freeze(
      Array.from({ length: charts.length }, (_, chart) => {
        // A period's rows are adjacent and share `period`; a spell's are
        // the run of one house inside it.
        const periods = [];
        for (let row = starts[chart]; row < starts[chart + 1]; row += 1) {
          const period = v.period[row];
          if (periods.length === period) periods.push([]);
          const spells = periods[period];
          if (spells === undefined) throw new Error(`sade_sati_visits row ${row} skips to period ${period}`);
          const house = v.house[row];
          if (spells.length === 0 || spells[spells.length - 1].house !== house) spells.push({ house, visits: [] });
          spells[spells.length - 1].visits.push(Object.freeze({ from: bound(v.from[row]), to: bound(v.to[row]) }));
        }
        const spell = ({ house, visits }) => Object.freeze({ house, visits: Object.freeze(visits) });
        // The Sade Sati's houses are 12, 1 and 2; a smaller spell is 3 to 11.
        const isSadeSati = (spells) => spells[0].house === 12 || spells[0].house <= 2;
        return Object.freeze({
          reference: Object.freeze({
            from: GocharFromById.get(c.countedFrom[chart]) ?? 'unknown',
            sign: RashiById.get(c.reference[chart]) ?? 'unknown',
          }),
          reckoning: ReckoningById.get(c.reckoning[chart]) ?? 'unknown',
          sadeSati: Object.freeze(
            periods.filter(isSadeSati).map((spells) => Object.freeze({ phases: Object.freeze(spells.map(spell)) })),
          ),
          spells: Object.freeze(periods.filter((spells) => !isSadeSati(spells)).map((spells) => spell(spells[0]))),
        });
      }),
    );
  }
  SADE_SATIS.set(batch, decoded);
  return decoded;
}

/**
 * A natal point as the Rust `NatalPoint` spells it, from a `to_lagna` and a
 * `to_graha` column's cells: `{ point: 'LAGNA' }` or `{ point: 'GRAHA', graha }`.
 *
 * @param {number} toLagna
 * @param {number} toGraha
 * @returns {object}
 */
function pointOf(toLagna, toGraha) {
  return Object.freeze(
    toLagna !== 0 ? { point: 'LAGNA' } : { point: 'GRAHA', graha: GrahaById.get(toGraha) ?? 'unknown' },
  );
}

/**
 * One row of the `hits` section as the Rust `Hit` spells it: the event
 * tagged by `kind`, carrying only the fields its kind has.
 *
 * @param {object} h the decoded section
 * @param {number} row
 * @returns {object}
 */
function hitOf(h, row) {
  const kind = HitKindById.get(h.kind[row]) ?? 'unknown';
  const motion = MotionById.get(h.motion[row]) ?? 'unknown';
  let event;
  switch (kind) {
    case 'SIGN_INGRESS':
      event = { kind, into: RashiById.get(h.into[row]) ?? 'unknown', motion };
      break;
    case 'NAKSHATRA_INGRESS':
      event = { kind, into: NakshatraById.get(h.into[row]) ?? 'unknown', motion };
      break;
    case 'STATION':
      event = { kind, turns: motion };
      break;
    case 'ASPECT':
      event = {
        kind,
        to: pointOf(h.toLagna[row], h.toGraha[row]),
        angle: h.angle[row],
        phase: AspectPhaseById.get(h.phase[row]) ?? 'unknown',
        motion,
      };
      break;
    default:
      event = { kind };
  }
  return Object.freeze({
    instant: h.instant[row],
    graha: GrahaById.get(h.graha[row]) ?? 'unknown',
    event: Object.freeze(event),
  });
}

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

/** Each batch's dignities, decoded once however many charts read them. */
const DIGNITIES = new WeakMap();

/** The flags of `dignity_planets`, in `EssentialDignity`'s order. */
const DIGNITY_FLAGS = ['house', 'exaltation', 'triplicity', 'term', 'face', 'detriment', 'fall'];

/** The five flags a planet that holds none of is peregrine. */
const DIGNITIES_HELD = DIGNITY_FLAGS.slice(0, 5);

/** `house` as it ends a camel-cased column name, `House`. */
const capitalised = (word) => word[0].toUpperCase() + word.slice(1);

/** One row's seven flags, each read from the column `prefix` names it by. */
function dignityAt(columns, prefix, row) {
  return Object.freeze(Object.fromEntries(DIGNITY_FLAGS.map((flag) => [flag, columns[prefix(flag)][row] === 1])));
}

/**
 * Every chart's essential dignities in a batch: `dignities` holds a row a
 * chart, or none when none was asked, `dignity_planets` seven rows a chart
 * in the Chaldean order, and `dignity_receptions` each chart's receptions,
 * ragged by its `receptionCount` (`03-design/essential-dignities.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function dignitiesOf(batch) {
  let decoded = DIGNITIES.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const charts = d.cast.instant.length;
  const c = d.dignities;
  const p = d.dignityPlanets;
  const r = d.dignityReceptions;
  if (c.sect.length === 0) {
    decoded = Object.freeze(Array.from({ length: charts }, () => null));
  } else {
    if (c.sect.length !== charts || p.planet.length !== 7 * charts) {
      throw new Error(
        `dignities has ${c.sect.length} rows and dignity_planets ${p.planet.length} for ${charts} charts; ` +
          'they are one and seven a chart, or none',
      );
    }
    let pair = 0;
    decoded = Object.freeze(
      Array.from({ length: charts }, (_, chart) => {
        const planets = Array.from({ length: 7 }, (_, k) => {
          const row = 7 * chart + k;
          const dignity = dignityAt(p, (flag) => flag, row);
          return Object.freeze({
            planet: GrahaById.get(p.planet[row]) ?? 'unknown',
            longitudeDeg: p.longitude[row],
            dignity,
            peregrine: !DIGNITIES_HELD.some((flag) => dignity[flag]),
            score: p.score[row],
            reception: p.reception[row],
          });
        });
        const receptions = Array.from({ length: c.receptionCount[chart] }, () => {
          const row = pair++;
          const firstIn = dignityAt(r, (flag) => `firstIn${capitalised(flag)}`, row);
          const secondIn = dignityAt(r, (flag) => `secondIn${capitalised(flag)}`, row);
          return Object.freeze({
            planets: Object.freeze([GrahaById.get(r.first[row]) ?? 'unknown', GrahaById.get(r.second[row]) ?? 'unknown']),
            firstIn,
            secondIn,
            mutual: Object.freeze(DIGNITIES_HELD.filter((flag) => firstIn[flag] && secondIn[flag])),
          });
        });
        return Object.freeze({
          sect: SectById.get(c.sect[chart]) ?? 'unknown',
          sectRule: SectRuleById.get(c.sectRule[chart]) ?? 'unknown',
          rules: Object.freeze({
            terms: TermsById.get(c.terms[chart]) ?? 'unknown',
            triplicities: TriplicitiesById.get(c.triplicities[chart]) ?? 'unknown',
          }),
          scores: Object.freeze({
            house: c.scoreHouse[chart],
            exaltation: c.scoreExaltation[chart],
            triplicity: c.scoreTriplicity[chart],
            term: c.scoreTerm[chart],
            face: c.scoreFace[chart],
            detriment: c.scoreDetriment[chart],
            fall: c.scoreFall[chart],
            peregrine: c.scorePeregrine[chart],
          }),
          planets: Object.freeze(planets),
          receptions: Object.freeze(receptions),
        });
      }),
    );
    if (pair !== r.first.length) {
      throw new Error(`dignity_receptions has ${r.first.length} rows and the charts count ${pair}`);
    }
  }
  DIGNITIES.set(batch, decoded);
  return decoded;
}

/** Each batch's fortitudes, decoded once however many charts read them. */
const FORTITUDES = new WeakMap();

/** Lilly's accidental lines, in the order the `fortitudes` section scores them. */
const ACCIDENTAL_LINES = [
  'direct',
  'retrograde',
  'swift',
  'slow',
  'superiorOriental',
  'superiorOccidental',
  'inferiorOriental',
  'inferiorOccidental',
  'increasing',
  'decreasing',
  'freeFromCombustion',
  'cazimi',
  'combust',
  'underBeams',
  'conjunctBenefic',
  'conjunctNorthNode',
  'trineBenefic',
  'sextileBenefic',
  'conjunctMalefic',
  'conjunctSouthNode',
  'opposedMalefic',
  'squareMalefic',
  'besieged',
  'regulus',
  'spica',
  'algol',
];

/** The seven in the Chaldean order, as the `fortitude_houses` almuten columns name them. */
const CHALDEAN = ['Saturn', 'Jupiter', 'Mars', 'Sun', 'Venus', 'Mercury', 'Moon'];

/**
 * An almuten as a ranking: each planet's total, every planet holding the
 * greatest (one unless they tie), and the next total down's, Chapter CV's
 * partakers (`03-design/essential-dignities.md` §The almuten).
 *
 * @param {readonly string[]} planets the seven, in the Chaldean order
 * @param {readonly number[]} totals theirs, in the same order
 */
function almutenOf(planets, totals) {
  const top = Math.max(...totals);
  const below = totals.filter((total) => total < top);
  const next = below.length === 0 ? null : Math.max(...below);
  const holding = (total) => Object.freeze(planets.filter((_, k) => totals[k] === total));
  return Object.freeze({
    totals: Object.freeze(planets.map((planet, k) => Object.freeze({ planet, total: totals[k] }))),
    almutens: holding(top),
    partakers: next === null ? Object.freeze([]) : holding(next),
  });
}

/** A reading with an orb, as the request writes it: the bare name, or `{ WITHIN: { [field]: orb } }`. */
function withOrb(name, field, orb) {
  return name === 'WITHIN' ? Object.freeze({ WITHIN: Object.freeze({ [field]: orb }) }) : name;
}

/**
 * Every chart's accidental fortitudes in a batch: `fortitudes` holds a row
 * a chart, or none when none was asked, `fortitude_houses` twelve rows a
 * chart, `fortitude_planets` seven in the Chaldean order, and
 * `fortitude_accidents` each planet's lines, ragged by its `accidentCount`
 * (`03-design/essential-dignities.md` §Accidental fortitudes). The
 * essential half is the batch's dignities.
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function fortitudesOf(batch) {
  let decoded = FORTITUDES.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const charts = d.cast.instant.length;
  const c = d.fortitudes;
  const h = d.fortitudeHouses;
  const p = d.fortitudePlanets;
  const a = d.fortitudeAccidents;
  if (c.houses.length === 0) {
    decoded = Object.freeze(Array.from({ length: charts }, () => null));
  } else {
    if (c.houses.length !== charts || h.cusp.length !== 12 * charts || p.planet.length !== 7 * charts) {
      throw new Error(
        `fortitudes has ${c.houses.length} rows, fortitude_houses ${h.cusp.length} and fortitude_planets ` +
          `${p.planet.length} for ${charts} charts; they are one, twelve and seven a chart, or none`,
      );
    }
    const essential = dignitiesOf(batch);
    let line = 0;
    decoded = Object.freeze(
      Array.from({ length: charts }, (_, chart) => {
        const dignities = essential[chart];
        const rows = (count) => Array.from({ length: count }, (_, k) => count * chart + k);
        const places = [];
        const planets = rows(7).map((row, k) => {
          const accidents = Array.from({ length: p.accidentCount[row] }, () => {
            const at = line++;
            return Object.freeze({
              accident: AccidentById.get(a.accident[at]) ?? 'unknown',
              points: a.points[at],
            });
          });
          const own = dignities.planets[k];
          places.push(p.places[row]);
          return Object.freeze({
            planet: GrahaById.get(p.planet[row]) ?? 'unknown',
            house: p.house[row],
            accidents: Object.freeze(accidents),
            fortitude: p.fortitude[row],
            debility: p.debility[row],
            net: own.score + own.reception + p.fortitude[row] - p.debility[row],
          });
        });
        return Object.freeze({
          dignities,
          sky: Object.freeze({
            houses: HouseSystemById.get(c.houses[chart]) ?? 'unknown',
            cuspsDeg: Object.freeze(rows(12).map((row) => h.cusp[row])),
            ascendantDeg: c.ascendant[chart],
            midheavenDeg: c.midheaven[chart],
            speedsDegPerDay: Object.freeze(rows(7).map((row) => p.speed[row])),
            northNodeDeg: c.northNode[chart],
            regulusDeg: c.regulus[chart],
            spicaDeg: c.spica[chart],
            algolDeg: c.algol[chart],
          }),
          rules: Object.freeze({
            combustionDeg: c.combustionOrb[chart],
            combustionInSign: c.combustionInSign[chart] === 1,
            beamsDeg: c.beamsOrb[chart],
            cazimiDeg: c.cazimiOrb[chart],
            cuspOrbDeg: c.cuspOrb[chart],
            starOrbDeg: c.starOrb[chart],
            partile: withOrb(PartileById.get(c.partile[chart]) ?? 'unknown', 'orbDeg', c.partileOrb[chart]),
            siege: withOrb(SiegeById.get(c.siege[chart]) ?? 'unknown', 'spanDeg', c.siegeSpan[chart]),
            meanMotionDeg: Object.freeze(rows(7).map((row) => p.meanMotion[row])),
          }),
          scores: Object.freeze({
            houses: Object.freeze(rows(12).map((row) => h.score[row])),
            ...Object.fromEntries(ACCIDENTAL_LINES.map((name) => [name, c[`score${capitalised(name)}`][chart]])),
          }),
          planets: Object.freeze(planets),
          almutens: Object.freeze({
            rules: Object.freeze({
              place: PlaceReadingById.get(c.almutenPlace[chart]) ?? 'unknown',
              fortune: FortuneRuleById.get(c.almutenFortune[chart]) ?? 'unknown',
            }),
            fortuneDeg: c.fortune[chart],
            figure: almutenOf(
              planets.map((at) => at.planet),
              planets.map((at) => at.net),
            ),
            places: almutenOf(
              planets.map((at) => at.planet),
              places,
            ),
            houses: Object.freeze(
              rows(12).map((row) =>
                almutenOf(
                  planets.map((at) => at.planet),
                  CHALDEAN.map((name) => h[`almuten${name}`][row]),
                ),
              ),
            ),
          }),
        });
      }),
    );
    if (line !== a.accident.length) {
      throw new Error(`fortitude_accidents has ${a.accident.length} rows and the planets count ${line}`);
    }
  }
  FORTITUDES.set(batch, decoded);
  return decoded;
}

/** Each batch's considerations, decoded once however many charts read them. */
const CONSIDERATIONS = new WeakMap();

/**
 * The members of a bit set over an enum, bit `n` the member with id `n`, in
 * id order.
 */
function members(set, byId) {
  const found = [];
  for (let id = 0; set >> id !== 0; id += 1) {
    if (((set >> id) & 1) === 1) found.push(byId.get(id) ?? 'unknown');
  }
  return Object.freeze(found);
}

/**
 * Every chart's considerations in a batch: `considerations` holds a row a
 * chart, or none when none was asked, `consideration_perfections` two a
 * chart and `consideration_orbs` seven
 * (`03-design/hellenistic-considerations.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function considerationsOf(batch) {
  let decoded = CONSIDERATIONS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const charts = d.cast.instant.length;
  const c = d.considerations;
  const p = d.considerationPerfections;
  const o = d.considerationOrbs;
  if (c.hourLord.length === 0) {
    decoded = Object.freeze(Array.from({ length: charts }, () => null));
  } else {
    if (c.hourLord.length !== charts || p.present.length !== 2 * charts || o.orbDeg.length !== 7 * charts) {
      throw new Error(
        `considerations has ${c.hourLord.length} rows, consideration_perfections ${p.present.length} ` +
          `and consideration_orbs ${o.orbDeg.length} for ${charts} charts; they are one, two and seven a chart, or none`,
      );
    }
    const perfection = (row) =>
      p.present[row] === 1
        ? Object.freeze({
            planet: GrahaById.get(p.planet[row]) ?? 'unknown',
            aspect: PtolemaicAspectById.get(p.aspect[row]) ?? 'unknown',
            days: p.days[row],
            gapDeg: p.gapDeg[row],
          })
        : null;
    const graha = (id) => GrahaById.get(id) ?? 'unknown';
    const rashi = (id) => RashiById.get(id) ?? 'unknown';
    decoded = Object.freeze(
      Array.from({ length: charts }, (_, k) =>
        Object.freeze({
          radicality: Object.freeze({
            hourLord: graha(c.hourLord[k]),
            ascendantLord: graha(c.ascendantLord[k]),
            grounds: members(c.radicalGrounds[k], RadicalGroundById),
          }),
          ascendant: Object.freeze({
            sign: rashi(c.ascendantSign[k]),
            degree: c.ascendantDegree[k],
            early: c.ascendantEarly[k] === 1,
            late: c.ascendantLate[k] === 1,
            shortAscension: c.shortAscension[k] === 1,
          }),
          moon: Object.freeze({
            sign: rashi(c.moonSign[k]),
            degree: c.moonDegree[k],
            late: c.moonLate[k] === 1,
            lateSign: c.moonLateSign[k] === 1,
            viaCombusta: c.viaCombusta[k] === 1,
            course: Object.freeze({
              next: perfection(2 * k),
              withinOrb: perfection(2 * k + 1),
              daysInSign: c.daysInSign[k],
              eased: c.eased[k] === 1,
            }),
          }),
          seventh: Object.freeze({
            cuspDeg: c.seventhCuspDeg[k],
            lord: graha(c.seventhLord[k]),
            infortunesInHouse: members(c.seventhInfortunes[k], GrahaById),
            lordRetrograde: c.seventhLordRetrograde[k] === 1,
            lordCombust: c.seventhLordCombust[k] === 1,
            lordInFall: c.seventhLordInFall[k] === 1,
            lordInInfortuneTerm: c.seventhLordInInfortuneTerm[k] === 1,
            lordNet: c.seventhLordNet[k],
          }),
          saturnHouse: c.saturnHouse[k],
          saturnRetrograde: c.saturnRetrograde[k] === 1,
          ascendantLordCombust: c.ascendantLordCombust[k] === 1,
          rules: Object.freeze({
            moonLateFromDeg: c.moonLateFromDeg[k],
            orbsDeg: Object.freeze(Array.from(o.orbDeg.subarray(7 * k, 7 * k + 7))),
          }),
        }),
      ),
    );
  }
  CONSIDERATIONS.set(batch, decoded);
  return decoded;
}

/**
 * A per-chart table read from a count section and the rows it is ragged by:
 * each chart's rows, or `null` for every chart when the count section is
 * empty because none was asked.
 *
 * @param {Charts} batch
 * @param {ArrayLike<number>} counts the count section's column
 * @param {number} rows how many rows the ragged section holds
 * @param {string} names the two sections, for the refusal of a torn blob
 * @param {(row: number) => object} read one row, by its index in the ragged section
 * @returns {readonly (readonly object[]|null)[]}
 */
function raggedOf(batch, counts, rows, names, read) {
  const charts = batch.decoded.cast.instant.length;
  if (counts.length === 0) return Object.freeze(Array.from({ length: charts }, () => null));
  const starts = startsOf(counts);
  if (counts.length !== charts || starts[charts] !== rows) {
    throw new Error(`${names}: ${counts.length} counts and ${rows} rows for ${charts} charts`);
  }
  return Object.freeze(
    Array.from({ length: charts }, (_, k) =>
      Object.freeze(Array.from({ length: counts[k] }, (_, n) => read(starts[k] + n))),
    ),
  );
}

/**
 * A section holding a row a chart, or none when its record was not asked:
 * each chart's row read by `read`, or `null` for every chart.
 *
 * @template T
 * @param {Charts} batch
 * @param {number} rows
 * @param {string} name
 * @param {(chart: number) => T} read
 * @returns {readonly (T|null)[]}
 */
function rowAChartOf(batch, rows, name, read) {
  const charts = batch.decoded.cast.instant.length;
  if (rows === 0) return Object.freeze(Array.from({ length: charts }, () => null));
  if (rows !== charts) throw new Error(`${name} has ${rows} rows for ${charts} charts; it is one a chart or none`);
  return Object.freeze(Array.from({ length: charts }, (_, k) => read(k)));
}

/** Each batch's Western aspect tables, decoded once however many charts read them. */
const WESTERN_ASPECTS = new WeakMap();

/**
 * Every chart's Western aspect table in a batch: `western_aspects` holds a
 * row a chart, or none when none was asked, and `western_aspect_rows` is
 * ragged by its count (`03-design/western-aspects.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (readonly object[]|null)[]}
 */
function westernAspectsOf(batch) {
  let decoded = WESTERN_ASPECTS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const r = d.westernAspectRows;
  const graha = (id) => GrahaById.get(id) ?? 'unknown';
  decoded = raggedOf(batch, d.westernAspects.count, r.first.length, 'western_aspects and western_aspect_rows', (row) =>
    Object.freeze({
      first: graha(r.first[row]),
      second: graha(r.second[row]),
      aspect: WesternAspectById.get(r.aspect[row]) ?? 'unknown',
      apartDeg: r.apartDeg[row],
      fromExactDeg: r.fromExactDeg[row],
      orbDeg: r.orbDeg[row],
      applying: r.applying[row] !== 0,
    }),
  );
  WESTERN_ASPECTS.set(batch, decoded);
  return decoded;
}

/** Each batch's synastries, their aspects, parallels, antiscia, composites and Davison births, decoded once however many charts read them. */
const SYNASTRIES = new WeakMap();

/**
 * Every chart's synastry with the partner in a batch: `synastry` holds a
 * row a chart, or none when none was asked, and `synastry_rows` is ragged
 * by its count (`03-design/western-synastry.md`); `synastry_parallels` and
 * `synastry_antiscia` and their rows the same for the parallels and the
 * antiscia across the two, and `synastry_composites` for the composite's
 * planets; `synastry_davisons` holds a Davison birth a chart, or none
 * (`03-design/western-composites.md`).
 *
 * @param {Charts} batch
 * @returns {{ aspects: readonly (readonly object[]|null)[], parallels: readonly (readonly object[]|null)[], antiscia: readonly (readonly object[]|null)[], midpoints: readonly (readonly object[]|null)[], composites: readonly (object|null)[], davisons: readonly (object|null)[] }}
 */
function synastriesOf(batch) {
  let decoded = SYNASTRIES.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const r = d.synastryRows;
  const p = d.synastryParallelRows;
  const aspects = raggedOf(batch, d.synastry.count, r.firstLagna.length, 'synastry and synastry_rows', (row) =>
    Object.freeze({
      first: pointOf(r.firstLagna[row], r.firstGraha[row]),
      second: pointOf(r.secondLagna[row], r.secondGraha[row]),
      aspect: WesternAspectById.get(r.aspect[row]) ?? 'unknown',
      apartDeg: r.apartDeg[row],
      fromExactDeg: r.fromExactDeg[row],
      orbDeg: r.orbDeg[row],
    }),
  );
  const parallels = raggedOf(
    batch,
    d.synastryParallels.count,
    p.contrary.length,
    'synastry and synastry_parallel_rows',
    (row) =>
      Object.freeze({
        first: pointOf(p.firstLagna[row], p.firstGraha[row]),
        second: pointOf(p.secondLagna[row], p.secondGraha[row]),
        contrary: p.contrary[row] !== 0,
        apartDeg: p.apartDeg[row],
        orbDeg: p.orbDeg[row],
      }),
  );
  const a = d.synastryAntiscionRows;
  const antiscia = raggedOf(batch, d.synastryAntiscia.count, a.first.length, 'synastry and synastry_antiscion_rows', (row) =>
    antiscionRowOf(a, row),
  );
  const c = d.synastryComposites;
  const cr = d.synastryCompositeRows;
  const compositePlanets = raggedOf(batch, c.count, cr.graha.length, 'synastry_composites and synastry_composite_rows', (row) =>
    Object.freeze({
      graha: GrahaById.get(cr.graha[row]) ?? 'unknown',
      longitudeDeg: cr.longitudeDeg[row],
      speedDegPerDay: cr.speedDegPerDay[row],
    }),
  );
  const cc = d.synastryCompositeCusps;
  const compositeCusps = raggedOf(batch, c.cuspCount, cc.cuspDeg.length, 'synastry_composites and synastry_composite_cusps', (row) =>
    cc.cuspDeg[row],
  );
  const composites = compositePlanets.map((planets, k) =>
    planets === null
      ? null
      : Object.freeze({
          planets,
          lagnaDeg: c.lagnaDeg[k],
          midheavenDeg: c.midheavenDeg[k],
          lagnaTurned: c.lagnaTurned[k] !== 0,
          cuspsDeg: compositeCusps[k].length === 12 ? compositeCusps[k] : null,
        }),
  );
  const b = d.synastryDavisons;
  const davisons = rowAChartOf(batch, b.instant.length, 'synastry_davisons', (k) =>
    Object.freeze({
      instant: b.instant[k],
      place: Object.freeze({ latitude: b.latitudeDeg[k], longitude: b.longitudeDeg[k], altitude: b.altitudeM[k] }),
      utcOffsetSeconds: b.utcOffsetSeconds[k],
    }),
  );
  const m = d.synastryMidpointRows;
  const midpoints = raggedOf(batch, d.synastryMidpoints.count, m.first.length, 'synastry_midpoints and synastry_midpoint_rows', (row) =>
    Object.freeze({ ...midpointFieldsOf(m, row), partnersPair: m.partnersPair[row] !== 0 }),
  );
  decoded = Object.freeze({ aspects, parallels, antiscia, midpoints, composites: Object.freeze(composites), davisons });
  SYNASTRIES.set(batch, decoded);
  return decoded;
}

/** Each batch's declinations and parallels, decoded once however many charts read them. */
const DECLINATIONS = new WeakMap();

/**
 * Every chart's declinations and parallels in a batch: `declinations`
 * holds a row a chart, or none when none was asked, and
 * `declination_rows` and `parallel_rows` are ragged by its two counts
 * (`03-design/western-declinations.md`).
 *
 * @param {Charts} batch
 * @returns {{ declinations: readonly (object|null)[], parallels: readonly (readonly object[]|null)[] }}
 */
function declinationsOf(batch) {
  let decoded = DECLINATIONS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const row = d.declinations;
  const rows = d.declinationRows;
  const p = d.parallelRows;
  const graha = (id) => GrahaById.get(id) ?? 'unknown';
  const grahas = raggedOf(batch, row.grahaCount, rows.graha.length, 'declinations and declination_rows', (at) =>
    Object.freeze({ graha: graha(rows.graha[at]), declinationDeg: rows.declinationDeg[at] }),
  );
  decoded = Object.freeze({
    declinations: Object.freeze(
      grahas.map((planets, k) =>
        planets === null
          ? null
          : Object.freeze({
              obliquityDeg: row.obliquityDeg[k],
              grahas: planets,
              lagnaDeg: row.lagnaDeg[k],
              midheavenDeg: row.midheavenDeg[k],
            }),
      ),
    ),
    parallels: raggedOf(batch, row.parallelCount, p.first.length, 'declinations and parallel_rows', (at) =>
      Object.freeze({
        first: graha(p.first[at]),
        second: graha(p.second[at]),
        contrary: p.contrary[at] !== 0,
        apartDeg: p.apartDeg[at],
        orbDeg: p.orbDeg[at],
      }),
    ),
  });
  DECLINATIONS.set(batch, decoded);
  return decoded;
}

/**
 * One pair in antiscion, a chart's own (`antiscion_rows`) or across a
 * synastry (`synastry_antiscion_rows`): the two sections share columns.
 *
 * @param {{ first: ArrayLike<number>, second: ArrayLike<number>, contrary: ArrayLike<number>, apartDeg: ArrayLike<number>, orbDeg: ArrayLike<number> }} r
 * @param {number} at
 */
function antiscionRowOf(r, at) {
  return Object.freeze({
    first: GrahaById.get(r.first[at]) ?? 'unknown',
    second: GrahaById.get(r.second[at]) ?? 'unknown',
    contrary: r.contrary[at] !== 0,
    apartDeg: r.apartDeg[at],
    orbDeg: r.orbDeg[at],
  });
}

/** Each batch's antiscia, decoded once however many charts read them. */
const ANTISCIA = new WeakMap();

/**
 * Every chart's antiscia in a batch: `antiscia` holds a row a chart, or
 * none when none was asked, and `antiscion_points` and `antiscion_rows`
 * are ragged by its two counts (`03-design/western-antiscia.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function antisciaOf(batch) {
  let decoded = ANTISCIA.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const p = d.antiscionPoints;
  const r = d.antiscionRows;
  const graha = (id) => GrahaById.get(id) ?? 'unknown';
  const points = raggedOf(batch, d.antiscia.pointCount, p.graha.length, 'antiscia and antiscion_points', (at) => at);
  const pairs = raggedOf(batch, d.antiscia.pairCount, r.first.length, 'antiscia and antiscion_rows', (at) =>
    antiscionRowOf(r, at),
  );
  const c = d.antiscionCuspRows;
  const onCusps = raggedOf(batch, d.antiscia.cuspCount, c.graha.length, 'antiscia and antiscion_cusp_rows', (at) =>
    Object.freeze({ graha: graha(c.graha[at]), house: c.house[at], contrary: c.contrary[at] !== 0 }),
  );
  decoded = Object.freeze(
    points.map((rows, k) =>
      rows === null
        ? null
        : Object.freeze({
            points: Object.freeze(
              rows.map((at) =>
                Object.freeze({
                  graha: graha(p.graha[at]),
                  antiscionDeg: p.antiscionDeg[at],
                  contrantiscionDeg: p.contrantiscionDeg[at],
                }),
              ),
            ),
            pairs: pairs[k],
            unpaired: Object.freeze(rows.filter((at) => p.paired[at] === 0).map((at) => graha(p.graha[at]))),
            onCusps: onCusps[k],
            cuspSystem:
              d.antiscia.cuspSystem[k] === NO_HOUSE_SYSTEM
                ? null
                : (HouseSystemById.get(d.antiscia.cuspSystem[k]) ?? 'unknown'),
          }),
    ),
  );
  ANTISCIA.set(batch, decoded);
  return decoded;
}

/** The `0xFFFF` a house-system column holds where no division was read. */
const NO_HOUSE_SYSTEM = 0xffff;

/** Each batch's Western houses, decoded once however many charts read them. */
const WESTERN_HOUSES = new WeakMap();

/**
 * Every chart's Western houses in a batch: `western_houses` holds a row a
 * chart, or none when none was asked, `western_house_cusps` twelve rows a
 * chart, and `western_house_planets` is ragged by its count
 * (`03-design/western-houses.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function westernHousesOf(batch) {
  let decoded = WESTERN_HOUSES.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const h = d.westernHouses;
  const c = d.westernHouseCusps.cuspDeg;
  const p = d.westernHousePlanets;
  if (c.length !== 12 * h.system.length) {
    throw new Error(`western_house_cusps has ${c.length} rows for ${h.system.length} charts; it is twelve a chart`);
  }
  const planets = raggedOf(batch, h.planetCount, p.graha.length, 'western_houses and western_house_planets', (at) =>
    Object.freeze({
      graha: GrahaById.get(p.graha[at]) ?? 'unknown',
      house: p.house[at],
      withAscendant: p.withAscendant[at] !== 0,
    }),
  );
  decoded = rowAChartOf(batch, h.system.length, 'western_houses', (k) =>
    Object.freeze({
      system: HouseSystemById.get(h.system[k]) ?? 'unknown',
      cuspsDeg: Object.freeze(Array.from(c.slice(12 * k, 12 * k + 12))),
      ascendantDeg: h.ascendantDeg[k],
      reachDeg: h.reachDeg[k],
      planets: planets[k],
    }),
  );
  WESTERN_HOUSES.set(batch, decoded);
  return decoded;
}

/** Each batch's harmonic charts, decoded once however many charts read them. */
const HARMONICS = new WeakMap();

/**
 * A harmonic chart's point from its two cells: `{ point: 'GRAHA', graha }`,
 * `{ point: 'ASCENDANT' }` or `{ point: 'MIDHEAVEN' }`.
 *
 * @param {number} angle
 * @param {number} graha
 * @returns {object}
 */
function harmonicPointOf(angle, graha) {
  if (angle === 1) return HARMONIC_ASCENDANT;
  if (angle === 2) return HARMONIC_MIDHEAVEN;
  return Object.freeze({ point: 'GRAHA', graha: GrahaById.get(graha) ?? 'unknown' });
}
const HARMONIC_ASCENDANT = Object.freeze({ point: 'ASCENDANT' });
const HARMONIC_MIDHEAVEN = Object.freeze({ point: 'MIDHEAVEN' });

/**
 * Every chart's harmonic chart in a batch: `harmonics` holds a row a chart,
 * or none when none was asked, and `harmonic_points` and `harmonic_rows`
 * are ragged by its two counts (`03-design/western-harmonics.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function harmonicsOf(batch) {
  let decoded = HARMONICS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const h = d.harmonics;
  const p = d.harmonicPoints;
  const r = d.harmonicRows;
  const points = raggedOf(batch, h.pointCount, p.angle.length, 'harmonics and harmonic_points', (at) =>
    Object.freeze({
      point: harmonicPointOf(p.angle[at], p.graha[at]),
      longitudeDeg: p.longitudeDeg[at],
      house: p.house[at],
    }),
  );
  const rows = raggedOf(batch, h.rowCount, r.apartDeg.length, 'harmonics and harmonic_rows', (at) =>
    Object.freeze({
      first: harmonicPointOf(r.firstAngle[at], r.firstGraha[at]),
      second: harmonicPointOf(r.secondAngle[at], r.secondGraha[at]),
      apartDeg: r.apartDeg[at],
      multiple: r.multiple[at],
      orbDeg: r.orbDeg[at],
    }),
  );
  decoded = rowAChartOf(batch, h.number.length, 'harmonics', (k) =>
    Object.freeze({ harmonic: h.number[k], points: points[k], rows: rows[k] }),
  );
  HARMONICS.set(batch, decoded);
  return decoded;
}

/** Each batch's matchings, decoded once however many charts read them. */
const MATCHINGS = new WeakMap();

/**
 * Every chart's match with the record's partner in a batch: `matchings`
 * holds a row a chart with what each koota read, or none when none was
 * asked, and `matching_kootas` eight rows a chart, each koota's points in
 * the verse's order (`03-design/matching.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function matchingsOf(batch) {
  let decoded = MATCHINGS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const m = d.matchings;
  const k = d.matchingKootas;
  const of = (byId, id) => byId.get(id) ?? 'unknown';
  const sides = (byId, bride, groom, at) => ({ bride: of(byId, bride[at]), groom: of(byId, groom[at]) });
  const readings = (at) => {
    const dosha = of(BhakootDoshaById, m.bhakootDosha[at]);
    return {
      'koota.VARNA': sides(VarnaById, m.brideVarna, m.groomVarna, at),
      'koota.VASHYA': { relation: of(VashyaRelationById, m.vashya[at]) },
      'koota.TARA': { brideToGroom: m.taraBrideToGroom[at], groomToBride: m.taraGroomToBride[at] },
      'koota.YONI': { ...sides(YoniById, m.brideYoni, m.groomYoni, at), relation: of(YoniRelationById, m.yoni[at]) },
      'koota.GRAHA_MAITRI': {
        ...sides(GrahaById, m.brideLord, m.groomLord, at),
        relation: of(MaitriRelationById, m.maitri[at]),
        lifted: m.maitriLifted[at] === 1,
      },
      'koota.GANA': {
        ...sides(GanaById, m.brideGana, m.groomGana, at),
        dosha: m.ganaDosha[at] === 1,
        lifted: m.ganaLifted[at] === 1,
      },
      'koota.BHAKOOT': {
        apart: m.bhakootApart[at],
        dosha: dosha === 'NONE' ? null : dosha,
        exceptions: Object.freeze({
          oneLord: m.bhakootOneLord[at] === 1,
          lordsFriends: m.bhakootLordsFriends[at] === 1,
          navamshaLordsFriends: m.bhakootNavamshaLordsFriends[at] === 1,
          taraPure: m.bhakootTaraPure[at] === 1,
          vashya: m.bhakootVashya[at] === 1,
        }),
        lifted: m.bhakootLifted[at] === 1,
      },
      'koota.NADI': {
        ...sides(NadiById, m.brideNadi, m.groomNadi, at),
        dosha: m.nadiDosha[at] === 1,
        lifted: m.nadiLifted[at] === 1,
      },
    };
  };
  const eight = Array.from({ length: m.total.length }, () => 8);
  const kootas = raggedOf(batch, eight, k.koota.length, 'matchings and matching_kootas', (row) => ({
    koota: of(KootaById, k.koota[row]),
    points: k.points[row],
    maxPoints: k.maxPoints[row],
  }));
  decoded = rowAChartOf(batch, m.total.length, 'matchings', (at) => {
    const read = readings(at);
    return Object.freeze({
      kootas: Object.freeze(
        kootas[at].map((row) =>
          Object.freeze({
            points: row.points,
            maxPoints: row.maxPoints,
            reading: Object.freeze({ koota: row.koota, ...read[row.koota] }),
          }),
        ),
      ),
      total: m.total[at],
    });
  });
  MATCHINGS.set(batch, decoded);
  return decoded;
}

/** Each batch's ten considerations, decoded once however many charts read them. */
const PORUTHAMS = new WeakMap();

/**
 * Every chart's ten considerations with the record's partner in a batch:
 * `poruthams` holds a row a chart with what each read, or none when none
 * was asked, and `porutham_rows` ten rows a chart, whether each agrees in
 * the chapter's order (`03-design/matching.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function poruthamsOf(batch) {
  let decoded = PORUTHAMS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const p = d.poruthams;
  const r = d.poruthamRows;
  const of = (byId, id) => byId.get(id) ?? 'unknown';
  const sides = (byId, bride, groom, at) => ({ bride: of(byId, bride[at]), groom: of(byId, groom[at]) });
  const readings = (at) => ({
    'koota.TARA': { count: p.count[at], rule: of(DhinamRuleById, p.dhinamRule[at]) },
    'koota.GANA': { ...sides(GanaById, p.brideGana, p.groomGana, at), diminished: p.ganaDiminished[at] === 1 },
    'koota.MAHENDRA': { count: p.count[at] },
    'koota.STREE_DEERGHA': { count: p.count[at] },
    'koota.YONI': { ...sides(YoniById, p.brideYoni, p.groomYoni, at), hostile: p.yoniHostile[at] === 1 },
    'koota.BHAKOOT': { apart: p.apart[at] },
    'koota.GRAHA_MAITRI': {
      ...sides(GrahaById, p.brideLord, p.groomLord, at),
      brideCallsFriend: p.brideCallsFriend[at] === 1,
      groomCallsFriend: p.groomCallsFriend[at] === 1,
    },
    'koota.VASHYA': { brideToGroom: p.brideToGroom[at] === 1, groomToBride: p.groomToBride[at] === 1 },
    'koota.RAJJU': sides(RajjuById, p.brideRajju, p.groomRajju, at),
    'koota.VEDHA': { pierced: p.pierced[at] === 1 },
  });
  const ten = Array.from({ length: p.agreeing.length }, () => 10);
  const rows = raggedOf(batch, ten, r.koota.length, 'poruthams and porutham_rows', (row) => ({
    koota: of(KootaById, r.koota[row]),
    agrees: r.agrees[row] === 1,
    lifted: r.lifted[row] === 1,
  }));
  decoded = rowAChartOf(batch, p.agreeing.length, 'poruthams', (at) => {
    const read = readings(at);
    return Object.freeze({
      considerations: Object.freeze(
        rows[at].map((row) =>
          Object.freeze({
            agrees: row.agrees,
            lifted: row.lifted,
            reading: Object.freeze({ koota: row.koota, ...read[row.koota] }),
          }),
        ),
      ),
      agreeing: p.agreeing[at],
      chiefAgreeing: p.chiefAgreeing[at],
      exception: Object.freeze({
        oneLord: p.oneLord[at] === 1,
        lordsFriendly: p.lordsFriendly[at] === 1,
        opposite: p.opposite[at] === 1,
      }),
    });
  });
  PORUTHAMS.set(batch, decoded);
  return decoded;
}

/** Each batch's equal distances, decoded once however many charts read them. */
const MIDPOINTS = new WeakMap();

/**
 * Every chart's equal distances in a batch: `midpoints` holds a row a
 * chart, or none when none was asked, and `midpoint_rows` is ragged by its
 * count (`03-design/western-midpoints.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (readonly object[]|null)[]}
 */
function midpointsOf(batch) {
  let decoded = MIDPOINTS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const r = d.midpointRows;
  decoded = raggedOf(batch, d.midpoints.count, r.first.length, 'midpoints and midpoint_rows', (at) =>
    Object.freeze(midpointFieldsOf(r, at)),
  );
  MIDPOINTS.set(batch, decoded);
  return decoded;
}

/**
 * One equal distance's fields, a chart's own (`midpoint_rows`) or across a
 * synastry (`synastry_midpoint_rows`, which adds `partnersPair`): the two
 * sections share their other columns.
 *
 * @param {{ first: ArrayLike<number>, second: ArrayLike<number>, middle: ArrayLike<number>, far: ArrayLike<number>, distanceDeg: ArrayLike<number>, fromAxisDeg: ArrayLike<number>, orbDeg: ArrayLike<number> }} r
 * @param {number} at
 */
function midpointFieldsOf(r, at) {
  return {
    first: GrahaById.get(r.first[at]) ?? 'unknown',
    second: GrahaById.get(r.second[at]) ?? 'unknown',
    middle: GrahaById.get(r.middle[at]) ?? 'unknown',
    far: r.far[at] !== 0,
    distanceDeg: r.distanceDeg[at],
    fromAxisDeg: r.fromAxisDeg[at],
    orbDeg: r.orbDeg[at],
  };
}

/** Each batch's progressions, decoded once however many charts read them. */
const PROGRESSIONS = new WeakMap();

/**
 * Every chart's progressions in a batch: `progressions` holds a row a
 * chart, or none when none was asked; the progressed and directed planets
 * are graha-count rows a chart when an instant was asked, and the contacts
 * are ragged by the row's count (`03-design/western-progressions.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function progressionsOf(batch) {
  let decoded = PROGRESSIONS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const charts = d.cast.instant.length;
  const p = d.progressions;
  const g = d.progressedGrahas;
  const m = d.directedGrahas;
  const c = d.progressedContacts;
  if (p.life.length === 0) {
    decoded = Object.freeze(Array.from({ length: charts }, () => null));
  } else {
    const asked = !Number.isNaN(p.life[0]);
    const perChart = asked ? g.graha.length / charts : 0;
    const starts = startsOf(p.contactCount);
    if (
      p.life.length !== charts ||
      m.graha.length !== g.graha.length ||
      (asked && !Number.isInteger(perChart)) ||
      starts[charts] !== c.life.length
    ) {
      throw new Error(
        `progressions has ${p.life.length} rows, progressed_grahas ${g.graha.length}, ` +
          `directed_grahas ${m.graha.length} and progressed_contacts ${c.life.length} for ${charts} charts`,
      );
    }
    const graha = (id) => GrahaById.get(id) ?? 'unknown';
    const rows = (count, from, next) => Object.freeze(Array.from({ length: count }, (_, k) => next(from + k)));
    decoded = Object.freeze(
      Array.from({ length: charts }, (_, k) =>
        Object.freeze({
          progressed: asked
            ? Object.freeze({
                life: p.life[k],
                sky: p.sky[k],
                armcDeg: p.armcDeg[k],
                angles: Object.freeze({ ascendantDeg: p.ascendantDeg[k], midheavenDeg: p.midheavenDeg[k] }),
                grahas: rows(perChart, k * perChart, (row) =>
                  Object.freeze({
                    graha: graha(g.graha[row]),
                    longitudeDeg: g.longitudeDeg[row],
                    tropicalDeg: g.tropicalDeg[row],
                    speedDegPerDay: g.speedDegPerDay[row],
                  }),
                ),
              })
            : null,
          directed: asked
            ? Object.freeze({
                life: p.life[k],
                arcDeg: p.arcDeg[k],
                ascendantDeg: p.directedAscendantDeg[k],
                midheavenDeg: p.directedMidheavenDeg[k],
                planets: rows(perChart, k * perChart, (row) =>
                  Object.freeze({ graha: graha(m.graha[row]), longitudeDeg: m.longitudeDeg[row] }),
                ),
              })
            : null,
          contacts:
            p.contactsAsked[k] === 1
              ? rows(p.contactCount[k], starts[k], (row) =>
                  Object.freeze({
                    life: c.life[row],
                    sky: c.sky[row],
                    graha: graha(c.graha[row]),
                    to: pointOf(c.toLagna[row], c.toGraha[row]),
                    angle: c.angle[row],
                    motion: MotionById.get(c.motion[row]) ?? 'unknown',
                  }),
                )
              : null,
        }),
      ),
    );
  }
  PROGRESSIONS.set(batch, decoded);
  return decoded;
}

/** Each batch's perfections, decoded once however many charts read them. */
const PERFECTIONS = new WeakMap();

/** A dignity bit set as its seven flags, bit `n` the `n`th of `DIGNITY_FLAGS`. */
function dignityOf(bits) {
  return Object.freeze(Object.fromEntries(DIGNITY_FLAGS.map((flag, n) => [flag, ((bits >> n) & 1) === 1])));
}

/**
 * Every chart's perfection in a batch: `perfection` holds a row a chart, or
 * none when none was asked, its impediments, translations and collections
 * ragged by that row's counts, and `perfection_orbs` seven a chart
 * (`03-design/hellenistic-perfection.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function perfectionsOf(batch) {
  let decoded = PERFECTIONS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const charts = d.cast.instant.length;
  const m = d.perfection;
  const i = d.perfectionImpediments;
  const t = d.perfectionTranslations;
  const c = d.perfectionCollections;
  const o = d.perfectionOrbs;
  if (m.querent.length === 0) {
    decoded = Object.freeze(Array.from({ length: charts }, () => null));
  } else {
    if (m.querent.length !== charts || o.orbDeg.length !== 7 * charts) {
      throw new Error(
        `perfection has ${m.querent.length} rows and perfection_orbs ${o.orbDeg.length} ` +
          `for ${charts} charts; they are one and seven a chart, or none`,
      );
    }
    const graha = (id) => GrahaById.get(id) ?? 'unknown';
    const aspect = (id) => PtolemaicAspectById.get(id) ?? 'unknown';
    let impediment = 0;
    let translation = 0;
    let collection = 0;
    const rows = (count, next) => Object.freeze(Array.from({ length: count }, next));
    decoded = Object.freeze(
      Array.from({ length: charts }, (_, k) =>
        Object.freeze({
          querent: graha(m.querent[k]),
          quesited: graha(m.quesited[k]),
          application:
            m.applicationPresent[k] === 1
              ? Object.freeze({
                  aspect: aspect(m.applicationAspect[k]),
                  days: m.applicationDays[k],
                  applying: graha(m.applying[k]),
                  kind: ApplicationKindById.get(m.applicationKind[k]) ?? 'unknown',
                  gapDeg: m.gapDeg[k],
                  withinMoieties: m.withinMoieties[k] === 1,
                })
              : null,
          separation:
            m.separationPresent[k] === 1
              ? Object.freeze({ aspect: aspect(m.separationAspect[k]), pastDeg: m.separationPastDeg[k] })
              : null,
          impediments: rows(m.impedimentCount[k], () => {
            const at = impediment++;
            return Object.freeze({
              kind: ImpedimentKindById.get(i.kind[at]) ?? 'unknown',
              significator: graha(i.significator[at]),
              third: i.thirdPresent[at] === 1 ? graha(i.third[at]) : null,
              aspect: aspect(i.aspect[at]),
              days: i.days[at],
            });
          }),
          translations: rows(m.translationCount[k], () => {
            const at = translation++;
            return Object.freeze({
              translator: graha(t.translator[at]),
              from: graha(t.from[at]),
              to: graha(t.to[at]),
              separating: Object.freeze({
                aspect: aspect(t.separatingAspect[at]),
                pastDeg: t.separatingPastDeg[at],
              }),
              aspect: aspect(t.aspect[at]),
              days: t.days[at],
              received: dignityOf(t.received[at]),
            });
          }),
          collections: rows(m.collectionCount[k], () => {
            const at = collection++;
            return Object.freeze({
              collector: graha(c.collector[at]),
              fromQuerent: Object.freeze({ aspect: aspect(c.fromQuerentAspect[at]), days: c.fromQuerentDays[at] }),
              fromQuesited: Object.freeze({ aspect: aspect(c.fromQuesitedAspect[at]), days: c.fromQuesitedDays[at] }),
              collectorInQuerent: dignityOf(c.collectorInQuerent[at]),
              collectorInQuesited: dignityOf(c.collectorInQuesited[at]),
              querentInCollector: dignityOf(c.querentInCollector[at]),
              quesitedInCollector: dignityOf(c.quesitedInCollector[at]),
            });
          }),
          ways: Object.freeze({
            querent: Object.freeze({
              planet: graha(m.querent[k]),
              house: m.querentHouse[k],
              dignity: dignityOf(m.querentDignity[k]),
            }),
            quesited: Object.freeze({
              planet: graha(m.quesited[k]),
              house: m.quesitedHouse[k],
              dignity: dignityOf(m.quesitedDignity[k]),
            }),
            mutualByHouse: m.mutualByHouse[k] === 1,
            infortunesBetween: members(m.infortunesBetween[k], GrahaById),
            moonRelays: m.moonRelays[k] === 1,
            quesitedInAscendant: m.quesitedInAscendant[k] === 1,
            held: members(m.waysHeld[k], WayById),
          }),
          horizonDays: m.horizonDays[k],
          rules: Object.freeze({
            orbsDeg: Object.freeze(Array.from(o.orbDeg.subarray(7 * k, 7 * k + 7))),
            horizonDays: Number.isNaN(m.horizonRuleDays[k]) ? null : m.horizonRuleDays[k],
            withinSign: m.withinSignRule[k] === 1,
          }),
        }),
      ),
    );
    for (const [name, read, held] of [
      ['perfection_impediments', impediment, i.kind.length],
      ['perfection_translations', translation, t.translator.length],
      ['perfection_collections', collection, c.collector.length],
    ]) {
      if (read !== held) throw new Error(`${name} has ${held} rows and the charts count ${read}`);
    }
  }
  PERFECTIONS.set(batch, decoded);
  return decoded;
}

/** Each batch's lots, decoded once however many charts read them. */
const LOTS = new WeakMap();

/** How many lots the catalogue names, and so how many rows a chart holds. */
const LOT_COUNT = LotById.size;

/**
 * Every chart's lots in a batch: `lots` holds a row a chart, or none when
 * none was asked, and `lot_places` the catalogue's lots for each chart, in
 * its order (`03-design/hellenistic-lots.md`).
 *
 * @param {Charts} batch
 * @returns {readonly (object|null)[]}
 */
function lotsOf(batch) {
  let decoded = LOTS.get(batch);
  if (decoded !== undefined) return decoded;
  const d = batch.decoded;
  const charts = d.cast.instant.length;
  const c = d.lots;
  const p = d.lotPlaces;
  if (c.sect.length === 0) {
    decoded = Object.freeze(Array.from({ length: charts }, () => null));
  } else {
    if (c.sect.length !== charts || p.lot.length !== LOT_COUNT * charts) {
      throw new Error(
        `lots has ${c.sect.length} rows and lot_places ${p.lot.length} for ${charts} charts; ` +
          `they are one and ${LOT_COUNT} a chart, or none`,
      );
    }
    decoded = Object.freeze(
      Array.from({ length: charts }, (_, chart) =>
        Object.freeze({
          sect: SectById.get(c.sect[chart]) ?? 'unknown',
          request: Object.freeze({
            sectRule: SectRuleById.get(c.sectRule[chart]) ?? 'unknown',
            fortune: FortuneRuleById.get(c.fortune[chart]) ?? 'unknown',
          }),
          fortuneReversed: c.fortuneReversed[chart] === 1,
          lots: Object.freeze(
            Array.from({ length: LOT_COUNT }, (_, k) => {
              const row = LOT_COUNT * chart + k;
              return Object.freeze({
                lot: LotById.get(p.lot[row]) ?? 'unknown',
                place: Object.freeze({
                  longitudeDeg: p.longitudeDeg[row],
                  sign: RashiById.get(p.sign[row]) ?? 'unknown',
                  lord: GrahaById.get(p.lord[row]) ?? 'unknown',
                  house: p.house[row],
                }),
              });
            }),
          ),
        }),
      ),
    );
  }
  LOTS.set(batch, decoded);
  return decoded;
}

/** Each batch's KP readings, parsed once however many charts read them. */
const KPS = new WeakMap();

/**
 * Every chart's KP reading in a batch: the `kp` section's JSON, one entry a
 * chart, in this layer's shape — catalogue keys in full (`graha.SUN`), as
 * every other accessor gives them — and frozen to its leaves
 * (`03-design/kp.md`).
 *
 * @param {Charts} batch
 * @returns {object[]}
 */
function kpsOf(batch) {
  let parsed = KPS.get(batch);
  if (parsed === undefined) {
    const json = batch.decoded.kp;
    parsed = json ? JSON.parse(json).map((chart) => deepFreeze(kpFrom(chart))) : [];
    KPS.set(batch, parsed);
  }
  return parsed;
}

/**
 * A chart's KP reading as the boundary's JSON writes it, its bare keys made
 * full.
 */
function kpFrom({ chart, significators, ruling }) {
  const graha = (key) => `graha.${key}`;
  const grahas = (keys) => keys.map(graha);
  const level = ({ lord, span }) => ({ lord: graha(lord), span });
  const lords = ({ sign, star, sub, subSub }) => ({
    sign: graha(sign),
    star: level(star),
    sub: level(sub),
    subSub: level(subSub),
  });
  const rejection = (by) => (by === null ? null : { retrograde: graha(by.retrograde), byStar: by.byStar });
  return {
    chart: {
      system: `house_system.${chart.system}`,
      cusps: chart.cusps.map((cusp) => ({ ...cusp, lords: lords(cusp.lords) })),
      planets: chart.planets.map((planet) => ({ ...planet, graha: graha(planet.graha), lords: lords(planet.lords) })),
    },
    significators: {
      houses: significators.houses.map((house) => ({
        house: house.house,
        inOccupantsStars: grahas(house.inOccupantsStars),
        occupants: grahas(house.occupants),
        inLordsStar: grahas(house.inLordsStar),
        lord: graha(house.lord),
        conjoined: grahas(house.conjoined),
        aspected: grahas(house.aspected),
        intercepted: house.intercepted.map((sign) => `rashi.${sign}`),
      })),
      nodes: significators.nodes.map((node) => ({
        node: graha(node.node),
        conjoined: grahas(node.conjoined),
        starLord: graha(node.starLord),
        aspecting: grahas(node.aspecting),
        signLord: graha(node.signLord),
      })),
    },
    ruling: {
      rulers: ruling.rulers.map((ruler) => ({
        graha: graha(ruler.graha),
        reasons: ruler.reasons.map((reason) => (reason.kind === 'AGENT' ? { ...reason, of: graha(reason.of) } : reason)),
        retrograde: ruler.retrograde,
        rejectedBy: rejection(ruler.rejectedBy),
        rejectedBySub: rejection(ruler.rejectedBySub),
      })),
      rules: ruling.rules,
    },
  };
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
 * The `muhurta` section as this layer hands it out: the envelope's value
 * with its provenance beside it, as every stamped result carries one, and
 * each closed day's date in the shape every other date here has.
 *
 * @param {string} json the section, empty when none was asked for
 * @returns {object|null}
 */
function muhurtaFrom(json) {
  if (!json) return null;
  const { value, provenance } = JSON.parse(json);
  return deepFreeze({
    ...value,
    closed: value.closed.map(({ date, by }) => ({ date: dateFrom(date), by })),
    provenance: decodeProvenance(provenance),
  });
}

/**
 * A JSON section whose value needs no reshaping (the years, the
 * eclipses, the Nepal Sambat dates) as this layer hands it out: the envelope, its provenance
 * decoded as every other one is.
 *
 * @param {string} json the section, empty when none was asked for
 * @returns {object|null}
 */
function envelopeFrom(json) {
  if (!json) return null;
  const { value, provenance } = JSON.parse(json);
  return deepFreeze({ value, provenance: decodeProvenance(provenance) });
}

/**
 * The `festivals` section as this layer hands it out: the envelope's
 * value with its provenance beside it, and every date, the observance's
 * and each extent's, in the shape every other date here has.
 *
 * @param {string} json the section, empty when none was asked for
 * @returns {object|null}
 */
function festivalsFrom(json) {
  if (!json) return null;
  const { value, provenance } = JSON.parse(json);
  return deepFreeze({
    observances: value.observances.map((observance) => ({
      ...observance,
      day: dateFrom(observance.day),
      extents: observance.extents.map((extent) => ({ ...extent, day: dateFrom(extent.day) })),
    })),
    ekadashis: value.ekadashis.map((fast) => ({
      ...fast,
      days: fast.days.map(dateFrom),
      day: dateFrom(fast.day),
    })),
    unjudged: value.unjudged,
    provenance: decodeProvenance(provenance),
  });
}

/**
 * A date as the Rust types serialise it, in the shape `date(...)` builds
 * and `calendar.convert` answers: the era flattened beside its year, and
 * the resolution by name with a divergent one's computed day.
 *
 * @param {object} json
 * @returns {object}
 */
function dateFrom(json) {
  const { resolution } = json;
  const divergent = resolution.kind === 'DIVERGENT';
  return {
    calendar: json.calendar,
    ...(json.era == null ? {} : { era: json.era.era }),
    year: json.year,
    eraYear: json.era == null ? 0 : json.era.year,
    month: json.month,
    day: json.day,
    resolution: resolution.kind,
    computedMonth: divergent ? resolution.computed.month : 0,
    computedDay: divergent ? resolution.computed.day : 0,
  };
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
 * The transits a request asks for, as the JSON the boundary reads
 * (`03-design/gochar.md`): the instants checked here as every other list
 * of Julian days is, and the rest — an empty list, an unknown `from` —
 * refused by the SDK, naming the field from `gochar`.
 *
 * @param {object|undefined} gochar
 * @returns {string|undefined}
 */
function gocharJson(gochar) {
  const example = 'a transit request record, e.g. { instants: [2460676.5], from: "MOON" }';
  if (gochar && typeof gochar === 'object' && !Array.isArray(gochar) && gochar.instants !== undefined) {
    return recordJson(
      { ...gochar, instants: instants(gochar.instants, 'gochar.instants', { allowEmpty: true }) },
      'gochar',
      example,
    );
  }
  return recordJson(gochar, 'gochar', example);
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
   * @param {boolean} [request.eclipses] whether to answer the eclipses of
   *   the days and the place's view of each, as `Almanac.eclipses`
   * @param {boolean} [request.years] whether to answer the lunar years
   *   the days fall in, as `Almanac.years`
   * @param {boolean} [request.nepalSambat] whether to answer each day's
   *   Nepal Sambat date, as `Almanac.nepalSambat`
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
        muhurtaJson: recordJson(request.muhurta, 'muhurta', "a muhurta request record, e.g. { rules: 'RAMAN_MARRIAGE' }"),
        festivalsJson: recordJson(request.festivals, 'festivals', "a festivals request record, e.g. { rules: 'DHARMASINDHU' }"),
        sections:
          (request.years === true ? PANCHANGA_YEARS : 0) |
          (request.eclipses === true ? PANCHANGA_ECLIPSES : 0) |
          (request.nepalSambat === true ? PANCHANGA_NEPAL_SAMBAT : 0),
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
 * @typedef {import('./catalogue.js').Ephemeris|{ plugin: string, config?: object }} EphemerisChoice
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
        `an ephemeris is a name (${Object.values(Ephemeris).map((name) => `'${name}'`).join(', ')}) ` +
          "or an adapter's descriptor, " +
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

/**
 * The hit list that is each graha's returns: its conjunction, at 0°, with
 * its own natal place (`03-design/western-returns.md`). The Moon's is the
 * lunar return, the Sun's the solar. Asked for several grahas, the list
 * also holds each one's crossing of another's natal place, its `to` naming
 * the place.
 *
 * @example ctx.chart.found({ ...birth, hits: returnsRequest(from, to) }).hits
 */
export const returnsRequest = (from, to, grahas = ['MOON']) => ({
  from,
  to,
  grahas,
  kinds: ['ASPECT'],
  points: grahas,
  aspects: [0],
});

export { decodeCharts, decodeIntlRender, decodePanchanga, decodePositions } from './blob.js';
export { entityForms, messages } from './messages.js';
export * from './catalogue.js';
export * from './records.js';
