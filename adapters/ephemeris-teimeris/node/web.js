// The Teimeris engine as an ephemeris written in JavaScript, for a host
// that cannot open the adapter's binary: a browser, a worker, an edge
// runtime. The SDK's wasm build takes it as `provider`, or as an entry of
// an `ephemeris` chain, which is how Nepal's official days put it behind
// the Surya Siddhanta:
//
//   import * as engine from 'teimeris-wasm';
//   import { teimerisWeb } from '@teistro/ephemeris-teimeris/web';
//
//   await engine.init();
//   const official = new Context(nepalOfficial(await teimerisWeb(engine)));
//
// HAND-WRITTEN, and the native adapter's `positions` read line for line
// (`../rust/src/lib.rs`): the same flags for a frame, the same engine
// bodies, the same cell statuses and sources, the same coverage. Only
// positions cross: a modern provider's ayanamsha, sunrise and the rest
// are the SDK's own computed from them (`Completion::defines`), so the
// native adapter's overrides are never asked for and an almanac read
// through this provider is the native one bit for bit: `cargo xtask
// check-wasm` holds a month under both setups to the native adapter's
// day hashes (`test/web-day-hash.mjs`).
//
// No Node import: everything here runs where `crypto.subtle` does.
//
// **This module is AGPL-3.0**, because Teimeris is.

import { BLOCKS, FILES } from './ranges.js';

/** The SDK's bodies the engine answers, by catalogue key, and its own. */
const BODIES = Object.freeze([
  ['SUN', 'SUN'],
  ['MOON', 'MOON'],
  ['MERCURY', 'MERCURY'],
  ['VENUS', 'VENUS'],
  ['MARS', 'MARS'],
  ['JUPITER', 'JUPITER'],
  ['SATURN', 'SATURN'],
  ['URANUS', 'URANUS'],
  ['NEPTUNE', 'NEPTUNE'],
  ['PLUTO', 'PLUTO'],
  ['MEAN_NODE', 'MEAN_NODE'],
  ['TRUE_NODE', 'TRUE_NODE'],
  ['MEAN_APOGEE', 'MEAN_APOGEE'],
  ['OSCULATING_APOGEE', 'OSCU_APOGEE'],
]);

/** The port's cell statuses (`CellStatus::code`). */
const CELL = Object.freeze({ OK: 0, UNSUPPORTED: -1, OUT_OF_RANGE: -2, DATA_MISSING: -3, NOT_COMPUTED: -6 });

/** Added to an engine status the port has no name for, as the native adapter adds it. */
const CODE_BASE = -100;

/** The port's sources (`Source::to_bits`): what computed a cell. */
const SOURCE = Object.freeze({ UNKNOWN: 0, FILES: 1, JPL: 2, ANALYTIC: 3 });

/** The engine's two accuracy profiles, as the native adapter spells them. */
const PROFILES = Object.freeze({ compatible: 'COMPATIBLE', COMPATIBLE: 'COMPATIBLE', max: 'MAX', MAX: 'MAX' });

/** The first century of a block's file, or null for a name that is not a block's. */
function century(name) {
  const match = /^(sepl|semo|seas)([_m])(\d{2,3})\.se1$/u.exec(name);
  if (!match) return null;
  const value = Number(match[3]);
  if (value % 6 !== 0 || (match[2] === 'm' && value === 0)) return null;
  return match[2] === 'm' ? -value : value;
}

/** A file's bytes as a `Uint8Array`, from whatever a download gave. */
function bytesOf(name, bytes) {
  if (bytes instanceof Uint8Array) return bytes;
  if (bytes instanceof ArrayBuffer) return new Uint8Array(bytes);
  if (ArrayBuffer.isView(bytes)) return new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  throw new TypeError(`files: \`${name}\` has no bytes; give a Uint8Array or an ArrayBuffer`);
}

/** Lower-case hex of a digest. */
function hex(buffer) {
  return Array.from(new Uint8Array(buffer), (byte) => byte.toString(16).padStart(2, '0')).join('');
}

/** A refusal naming what a download should be, as the native adapter's does. */
function differs(name, size, digest, published) {
  return new Error(
    `\`${name}\` is ${size} bytes with SHA-256 ${digest}; the published file is ` +
      `${published.bytes} bytes with SHA-256 ${published.sha256}. Download it again: ` +
      'a file cut short or altered is refused rather than read.',
  );
}

/**
 * The files a caller added, checked: each a block's, named once, and,
 * unless `verify` is false, the published copy byte for byte.
 */
async function checked(files, verify) {
  const seen = new Set();
  const added = [];
  for (const entry of files) {
    if (entry === null || typeof entry !== 'object' || typeof entry.name !== 'string') {
      throw new TypeError('files: each entry is `{ name, bytes }`, the name the file is published under');
    }
    const { name } = entry;
    if (century(name) === null) {
      throw new TypeError(`files: \`${name}\` names no data block; a block's files are named like sepl_12.se1`);
    }
    if (seen.has(name)) throw new TypeError(`files: \`${name}\` is given twice`);
    seen.add(name);
    const bytes = bytesOf(name, entry.bytes);
    const published = FILES.find((file) => file.file === name) ?? null;
    const digest = hex(await crypto.subtle.digest('SHA-256', bytes));
    const same = published !== null && published.bytes === bytes.byteLength && published.sha256 === digest;
    if (verify && !same) {
      if (published === null) throw new Error(`\`${name}\` is not a file the engine publishes`);
      throw differs(name, bytes.byteLength, digest, published);
    }
    added.push({ name, bytes, published: same ? published : null });
  }
  return added;
}

/**
 * The span a set of files serves, by the native adapter's rule: the
 * outermost planet blocks' nominal years, narrowed to what their files
 * record where the set holds the published copies; or a refusal naming
 * the files that fill a hole.
 */
function coverage(names, publishedOf) {
  const planets = [...new Set(names.filter((name) => name.startsWith('sepl')).map(century))].sort((a, b) => a - b);
  if (planets.length === 0) {
    throw new Error('the data covers no years: add a block of planet and Moon files, or keep the built-in data');
  }
  const missing = [];
  for (let at = planets[0]; at <= planets[planets.length - 1]; at += 6) {
    const block = BLOCKS.find((b) => b.century === at);
    if (!block) continue;
    if (!planets.includes(at)) missing.push(block.planets);
    if (!names.includes(block.moon)) missing.push(block.moon);
  }
  if (missing.length > 0) {
    const sizes = missing.map((name) => {
      const file = FILES.find((f) => f.file === name);
      return file ? `${name} (${file.bytes} bytes)` : name;
    });
    throw new Error(`the data has a hole: it holds ${names.join(', ')} and lacks ${sizes.join(', ')}`);
  }
  const first = BLOCKS.find((b) => b.century === planets[0]);
  const last = BLOCKS.find((b) => b.century === planets[planets.length - 1]);
  let low = first.start;
  let high = last.end;
  for (const name of [first.planets, first.moon]) {
    const file = publishedOf(name);
    if (file) low = Math.max(low, file.jdStart);
  }
  for (const name of [last.planets, last.moon]) {
    const file = publishedOf(name);
    if (file) high = Math.min(high, file.jdEnd);
  }
  return [low, high];
}

/** The engine's flags for a packed frame (`Frame::to_bits`), as `map_flags` sets them. */
function flagsOf(Flags, bits, speeds) {
  let flags = Flags.EPH_SWISS;
  const centre = bits & 0b11;
  const switches = [
    [!speeds, Flags.NO_SPEED],
    [(bits & (1 << 3)) !== 0, Flags.EQUATORIAL],
    [(bits & (1 << 2)) !== 0, Flags.J2000],
    [(bits & (1 << 7)) === 0, Flags.NO_NUTATION],
    [(bits & (1 << 4)) === 0, Flags.NO_LIGHT_TIME],
    [(bits & (1 << 5)) === 0, Flags.NO_ABERRATION],
    [(bits & (1 << 6)) === 0, Flags.NO_DEFLECTION],
    [centre === 1, Flags.TOPOCENTRIC],
    [centre === 2, Flags.HELIOCENTRIC],
    [centre === 3, Flags.BARYCENTRIC],
    [(bits & (1 << 8)) !== 0, Flags.SIDEREAL],
  ];
  for (const [on, flag] of switches) {
    if (on) flags |= flag;
  }
  return flags;
}

/** The port's status for an engine status, as `map_status` reads it. */
function statusOf(Status, code) {
  switch (code) {
    case Status.OK:
      return CELL.OK;
    case Status.ERR_OUT_OF_RANGE:
      return CELL.OUT_OF_RANGE;
    case Status.ERR_NO_EPHEMERIS:
    case Status.ERR_DATA_CORRUPT:
    case Status.ERR_IO:
      return CELL.DATA_MISSING;
    case Status.ERR_UNSUPPORTED:
      return CELL.UNSUPPORTED;
    default:
      return CODE_BASE + code;
  }
}

/** The port's source for the flags the engine used, as `map_source` reads them. */
function sourceOf(Flags, used) {
  if (used & Flags.EPH_JPL) return SOURCE.JPL;
  if (used & Flags.EPH_MOSHIER) return SOURCE.ANALYTIC;
  if (used & Flags.EPH_SWISS) return SOURCE.FILES;
  return SOURCE.UNKNOWN;
}

/**
 * The provider: the engine's built-in 1800 to 2399, and any blocks the
 * app downloaded, each checked against the size and SHA-256 the engine
 * publishes them with (`./ranges.js`) before it is read.
 *
 * ```js
 * // The built-in years, and 1200 to 1799 the app downloaded.
 * const provider = await teimerisWeb(engine, {
 *   files: [
 *     { name: 'sepl_12.se1', bytes: await (await fetch('/ephe/sepl_12.se1')).arrayBuffer() },
 *     { name: 'semo_12.se1', bytes: await (await fetch('/ephe/semo_12.se1')).arrayBuffer() },
 *   ],
 * });
 * ```
 *
 * @param {object} engine the `teimeris-wasm` module, after `await engine.init()`
 * @param {object} [options]
 * @param {readonly {name: string, bytes: Uint8Array | ArrayBuffer}[]} [options.files]
 *   block files the app holds, under the names the engine publishes them by
 * @param {boolean} [options.verify] `false` reads a file that is not the
 *   published copy, claiming only the years its name gives
 * @param {string} [options.profile] the engine's accuracy profile,
 *   `compatible` (the default) or `max`
 * @returns {Promise<object>} a provider, as the SDK's `provider` takes one
 */
export async function teimerisWeb(engine, options = {}) {
  const { files = [], verify = true, profile = 'compatible' } = options;
  if (engine === null || typeof engine !== 'object' || typeof engine.Context !== 'function') {
    throw new TypeError("engine: expected the `teimeris-wasm` module, as `import * as engine from 'teimeris-wasm'` gives it");
  }
  if (typeof engine.ready === 'function' && !engine.ready()) {
    throw new Error('engine: call `await engine.init()` first; the module is not instantiated');
  }
  const spelled = PROFILES[profile];
  if (spelled === undefined) {
    throw new TypeError(`profile: \`${profile}\` is not one of the engine's; give compatible or max`);
  }
  if (!Array.isArray(files)) throw new TypeError('files: expected an array of `{ name, bytes }`');
  const added = await checked(files, verify);
  const { Body, Flags, Status, TimeScale } = engine;
  const embedded = engine
    .embeddedFiles()
    .map((file) => file.name)
    .filter((name) => century(name) !== null);
  const names = [...new Set([...added.map((file) => file.name), ...embedded])].sort();
  // A file the app added under an embedded name is read first, as the
  // engine resolves registered sources before its own table.
  const publishedOf = (name) => {
    const own = added.find((file) => file.name === name);
    if (own) return own.published;
    return FILES.find((file) => file.file === name) ?? null;
  };
  const [jdMin, jdMax] = coverage(names, publishedOf);
  const context = new engine.Context({ profile: engine.Profile[spelled] });
  for (const file of added) context.addSourceFromBytes(file.name, file.bytes);
  const scales = { UT1: TimeScale.UT1, TT: TimeScale.TT };
  const bodyOf = new Map(BODIES.map(([key, own]) => [key, Body[own]]));

  return Object.freeze({
    name: 'teimeris',
    version: engine.version(),
    dataVersion: names.join(' '),
    bodies: BODIES.map(([key]) => key),
    jdMin,
    jdMax,
    speeds: true,
    deterministic: true,
    positions(request) {
      const { jds, bodies, frameBits, speeds, observer } = request;
      const engineBodies = Int32Array.from(bodies, (key) => {
        const body = bodyOf.get(key);
        if (body === undefined) throw new RangeError(`teimeris answers no \`${key}\``);
        return body;
      });
      if ((frameBits & (1 << 8)) !== 0) {
        // The catalogue's ayanamsha id is the engine's mode for every
        // member (held by the adapter's `every_ayanamsha_id_is_the_engine_s_mode`).
        context.setAyanamsha(frameBits >>> 16, 0, 0);
      }
      const options = {
        flags: flagsOf(Flags, frameBits, speeds),
        scale: scales[request.scale],
        observer: observer
          ? { longitude: observer.longitudeDeg, latitude: observer.latitudeDeg, altitude: observer.altitudeM }
          : null,
      };
      let raw;
      try {
        raw = context.positionsColumns(jds, engineBodies, options);
      } catch (error) {
        // A batch that partly failed still carries every cell's own status.
        if (error?.results === undefined) throw error;
        raw = error.results;
      }
      // The engine's grid is body-major (`body × instants + instant`); the
      // port's is instants-outermost (`instant × bodies + body`).
      const count = jds.length;
      const cells = count * bodies.length;
      const out = {
        lon: new Float64Array(cells),
        lat: new Float64Array(cells),
        dist: new Float64Array(cells),
        lonSpeed: new Float64Array(cells),
        latSpeed: new Float64Array(cells),
        distSpeed: new Float64Array(cells),
        status: new Int32Array(cells),
        source: new Uint32Array(cells),
      };
      for (let i = 0; i < count; i += 1) {
        for (let b = 0; b < bodies.length; b += 1) {
          const index = i * bodies.length + b;
          const at = b * count + i;
          const used = raw.flags_used[at] ?? 0;
          const source = sourceOf(Flags, used);
          const engineStatus = statusOf(Status, raw.status[at] ?? CELL.NOT_COMPUTED);
          let status = engineStatus;
          if (jds[i] < jdMin || jds[i] > jdMax) {
            status = CELL.OUT_OF_RANGE;
          } else if (engineStatus === CELL.OK && source === SOURCE.ANALYTIC) {
            // The engine fell back to its analytic model for this cell;
            // the port reports missing data rather than a position from a
            // model the caller did not ask for.
            status = CELL.DATA_MISSING;
          }
          out.status[index] = status;
          if (status !== CELL.OK) continue;
          out.lon[index] = raw.lon[at];
          out.lat[index] = raw.lat[at];
          out.dist[index] = raw.dist[at];
          out.lonSpeed[index] = raw.lon_speed[at];
          out.latSpeed[index] = raw.lat_speed[at];
          out.distSpeed[index] = raw.dist_speed[at];
          out.source[index] = source;
        }
      }
      return out;
    },
  });
}
