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
  BodyById,
  CONTEXT_TEST_PROVIDER,
  SDK_VERSION,
  TimeScaleById,
} from './catalogue.js';
import { decodeIntlRender, decodePositions } from './blob.js';
import { entityForms, messages } from './messages.js';

const HERE = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

/** The addon: a packaged build first, then the workspace's own. */
function loadAddon() {
  const named = process.env.TEISTRO_ADDON;
  const candidates = [
    named,
    join(HERE, '..', 'native', 'index.node'),
    join(HERE, '..', '..', '..', 'target', 'release', addonName()),
    join(HERE, '..', '..', '..', 'target', 'debug', addonName()),
  ].filter(Boolean);
  const found = candidates.find((path) => existsSync(path));
  if (!found) {
    throw new Error(
      `no Teistro addon found. Looked in:\n  ${candidates.join(
        '\n  ',
      )}\nBuild it with \`cargo build -p teistro-node\`, or set TEISTRO_ADDON to its path.`,
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
 * Runs a call on the addon and rethrows its failure with everything the
 * library said. Only a code crosses the C boundary for a provider's own
 * failure, so when the library reports one the caught message — which
 * came from the provider, in this language — is the one kept.
 */
function guarded(context, call) {
  try {
    return call();
  } catch (cause) {
    const record = context?.lastError?.();
    if (!record) throw cause;
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
  if (provider === undefined || provider === null) return [undefined, undefined];
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
    jdMin: provider.jdRange?.[0],
    jdMax: provider.jdRange?.[1],
    nativeFrameBits: provider.frame === undefined ? undefined : native.framePack(clean(provider.frame)),
    speeds: provider.speeds,
    deterministic: provider.deterministic,
  });
  // The callback answers with plain arrays; a column left out is zeroes,
  // which is what a provider that computes no speeds means.
  const positions = (request) => {
    const answer = provider.positions(request);
    // Nothing means "I cannot produce that frame"; the SDK then asks for
    // the provider's native frame and completes the rest itself.
    if (answer === null || answer === undefined) return null;
    const cells = request.jds.length * request.bodies.length;
    const column = (name) => Array.from(answer[name] ?? new Float64Array(cells));
    return {
      frameBits: answer.frameBits ?? request.frameBits,
      lon: column('lon'),
      lat: column('lat'),
      dist: column('dist'),
      lonSpeed: column('lonSpeed'),
      latSpeed: column('latSpeed'),
      distSpeed: column('distSpeed'),
      status: Array.from(answer.status ?? new Int32Array(cells)),
      source: Array.from(answer.source ?? new Uint32Array(cells)),
    };
  };
  return [info, positions];
}

/** A finite number, or a `TypeError` naming the argument. */
function finite(value, what) {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw new TypeError(`${what}: expected a finite number, got ${String(value)}`);
  }
  return value;
}

/** The instants as the addon takes them: a plain array of doubles. */
function instants(values, what) {
  const list = ArrayBuffer.isView(values) ? Array.from(values) : values;
  if (!Array.isArray(list) || list.length === 0) {
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
    const [info, positions] = describeProvider(provider);
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
    return guarded(this.#inner, () => this.#inner.profile());
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
    return guarded(this.#inner, () => this.#inner.settingsJson());
  }

  /** The SHA-256 of the canonical settings, in hex; every result carries it. */
  get settingsHash() {
    const hash = guarded(this.#inner, () => this.#inner.settingsHash());
    return Buffer.from(hash.bytes).toString('hex');
  }

  /** The locale every render resolves from. */
  get locale() {
    return guarded(this.#inner, () => this.#inner.intlLocale());
  }

  set locale(tag) {
    guarded(this.#inner, () => this.#inner.intlSetLocale(tag));
  }

  /** The SDK's canonical frame: apparent geocentric ecliptic of date, tropical. */
  canonicalFrame() {
    return guarded(this.#inner, () => native.frameCanonical());
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
    const bytes = guarded(this.#inner, () =>
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

  /** Renders a message of the current locale with its parameters. */
  render(key, params) {
    const bytes = guarded(this.#inner, () =>
      this.#inner.intlRender(key, params === undefined ? undefined : JSON.stringify(params)),
    );
    return new Rendered(bytes);
  }

  /** Whether the current locale or its fallbacks have a message. */
  has(key) {
    return guarded(this.#inner, () => this.#inner.intlHas(key)) === 1;
  }

  /**
   * Text from one script into another (`deva`, `iast`), for a Sanskrit
   * or Nepali term written in the other.
   */
  transliterate(text, from = 'deva', to = 'iast') {
    return guarded(this.#inner, () => this.#inner.intlTransliterate(text, from, to));
  }

  /**
   * An entity's forms in the current locale or its fallbacks: its name,
   * its prose form, its transliteration, and the glyph and gender the
   * locale gives it.
   */
  entity(key) {
    return entityForms(guarded(this.#inner, () => this.#inner.intlEntity(key)));
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
    return guarded(this.#inner, () => this.#inner.intlLoadPack(Buffer.from(bytes)));
  }

  /** The date a fixed day falls on in a calendar. */
  dateOf(calendar, fixed) {
    return guarded(this.#inner, () => this.#inner.calendarFromFixed(calendar, fixed));
  }

  /** The fixed day of a date. */
  fixedOf(date) {
    return guarded(this.#inner, () => this.#inner.calendarToFixed(clean(date)));
  }

  /** The same date in another calendar. */
  convert(date, into) {
    return guarded(this.#inner, () => this.#inner.calendarConvert(clean(date), into));
  }

  /** The weekday of a date, Monday `1` to Sunday `7`. */
  weekdayOf(date) {
    return guarded(this.#inner, () => this.#inner.calendarWeekday(clean(date)));
  }

  /** The length of a month. */
  monthLength(calendar, year, month) {
    return guarded(this.#inner, () => this.#inner.calendarMonthLength(calendar, year, month));
  }

  /** Whether a year is a leap year. */
  isLeap(calendar, year) {
    return guarded(this.#inner, () => this.#inner.calendarIsLeap(calendar, year)) === 1;
  }

  /** A civil date and time in a zone, resolved to an instant with its metadata. */
  resolve(civil, zone) {
    return guarded(this.#inner, () => this.#inner.timeResolve(clean(civil), clean(zone)));
  }

  /** The civil date and time of an instant in a zone. */
  civilOf(jdUtc, zone, calendar) {
    return guarded(this.#inner, () =>
      this.#inner.timeCivil(finite(jdUtc, 'jdUtc'), clean(zone), calendar),
    );
  }

  /** Converts an instant between the time scales. */
  convertTime(jd, from, to) {
    return guarded(this.#inner, () => this.#inner.timeConvert(finite(jd, 'jd'), from, to));
  }

  /** Delta T at a UT1 instant, with what produced it. */
  deltaT(jdUt1) {
    return guarded(this.#inner, () => this.#inner.timeDeltaT(finite(jdUt1, 'jdUt1')));
  }

  /** The packed id of a catalogue key. */
  keyId(key) {
    return guarded(this.#inner, () => this.#inner.keyParse(key));
  }

  /** The catalogue key of a packed id. */
  keyName(id) {
    return guarded(this.#inner, () => this.#inner.keyName(id));
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

export { decodeIntlRender, decodePositions } from './blob.js';
export { entityForms, messages } from './messages.js';
export * from './catalogue.js';
