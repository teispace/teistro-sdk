/**
 * Spike 2, option A: the ergonomic layer of the Node binding.
 *
 * HAND-WRITTEN. The generated layer beneath it (`native/src/generated.rs`,
 * `generated.d.ts`, `blob.js`) is the mechanical contract; this file is
 * the part that is allowed to be JavaScript with opinions: defaults,
 * validation at the door, a `Chart` that decodes its blob lazily, and the
 * ABI handshake. Everything else comes from the description.
 */

import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { dashaRow, dashaTree, decodeChart } from './blob.js';

const HERE = dirname(fileURLToPath(import.meta.url));
/** The ABI this layer was written against; the addon must agree. */
const ABI_VERSION = 1;

/** Loads the addon: a packaged `index.node`, else the workspace build. */
function loadNative() {
  const packaged = join(HERE, '..', 'native', 'index.node');
  const built = join(HERE, '..', '..', '..', '..', 'target', 'release', 'libteistro_spike_a_node.dylib');
  const candidate = existsSync(packaged) ? packaged : built;
  const module = { exports: {} };
  process.dlopen(module, candidate);
  return module.exports;
}

const native = loadNative();
if (native.abiVersion() !== ABI_VERSION) {
  throw new Error(`addon ABI ${native.abiVersion()}, this layer expects ${ABI_VERSION}`);
}

export const { Ayanamsha, NodeKind, Body, Status } = native;
export const { abiVersion, chartNodeCount, settingsDefault, statusMessage } = native;

/** A Julian Day in UT: finite, or a `TypeError` at the door. */
export function julianDay(value) {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw new TypeError(`julianDay: expected a finite number, got ${String(value)}`);
  }
  return value;
}

/** A dasha depth: an integer from 1 to 5, or a `RangeError` at the door. */
export function dashaDepth(value) {
  if (!Number.isInteger(value) || value < 1 || value > 5) {
    throw new RangeError(`dashaDepth: expected an integer from 1 to 5, got ${String(value)}`);
  }
  return value;
}

const ENUMS = { ayanamsha: Ayanamsha, node: NodeKind };

/** Fills in the defaults and rejects a value outside its enum before the native call sees it. */
function resolveSettings(options = {}) {
  const settings = { ...settingsDefault(), ...options };
  for (const [key, table] of Object.entries(ENUMS)) {
    if (!Object.values(table).includes(settings[key])) {
      throw new RangeError(`${key}: expected one of ${Object.values(table).join(', ')}, got ${String(settings[key])}`);
    }
  }
  settings.dashaDepth = dashaDepth(settings.dashaDepth);
  return settings;
}

/** A computed chart: the blob, decoded on first use, and the views over it. */
export class Chart {
  #decoded = null;

  constructor(bytes) {
    this.bytes = bytes;
  }

  /** The columns, decoded once without copying. */
  get decoded() {
    this.#decoded ??= decodeChart(this.bytes);
    return this.#decoded;
  }

  /** The instant the chart was computed for. */
  get jdUt() {
    return this.decoded.jdUt;
  }

  /** The number of tree nodes. */
  get nodeCount() {
    return this.decoded.nodeCount;
  }

  /** The nine positions as plain objects, built on demand. */
  positions() {
    const p = this.decoded.positions;
    const out = new Array(p.length);
    for (let i = 0; i < p.length; i += 1) {
      out[i] = {
        body: p.body[i],
        longitudeDeg: p.longitudeDeg[i],
        longitudeNas: p.longitudeNas[i],
        latitudeDeg: p.latitudeDeg[i],
        speedDegPerDay: p.speedDegPerDay[i],
        sign: p.sign[i],
        nakshatra: p.nakshatra[i],
        pada: p.pada[i],
        retrograde: p.retrograde[i] === 1,
      };
    }
    return out;
  }

  /** The tree as nested objects, all of it. */
  dashaTree() {
    return dashaTree(this.decoded);
  }

  /** One row of the tree, without touching the rest. */
  dashaRow(index) {
    return dashaRow(this.decoded, index);
  }
}

/**
 * Creates a context. `provider` is a function `(jdUt, body) => Position`
 * implemented in JavaScript; omit it for the built-in test provider.
 */
export function createContext(options = {}, provider = null) {
  const settings = resolveSettings(options);
  const context = new native.Context(settings, provider ?? null);
  return {
    /** The one batch call: a `Chart` over the blob the native side returned. */
    computeChart(jdUt) {
      return new Chart(context.chartCompute(julianDay(jdUt)));
    },
    /** The settings the context was built with. */
    settings() {
      return context.settings();
    },
    /** The provider's code from the last failure, `0` when there was none. */
    lastProviderCode() {
      return context.lastProviderCode();
    },
  };
}
