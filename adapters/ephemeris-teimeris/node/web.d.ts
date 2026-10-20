// The Teimeris engine as an ephemeris written in JavaScript, typed: what
// `web.js` builds over the `teimeris-wasm` module for a host that cannot
// open the adapter's binary.

import type { EphemerisProvider } from '@teistro/sdk';

/** One block file the app holds, under the name the engine publishes it by. */
export interface TeimerisFile {
  /** The published name, such as `sepl_12.se1`. */
  readonly name: string;
  /** The file itself, as a download gives it. */
  readonly bytes: Uint8Array | ArrayBuffer;
}

/** What `teimerisWeb` reads besides the engine's built-in 1800 to 2399. */
export interface TeimerisWebOptions {
  /**
   * Block files the app downloaded, each checked against the size and
   * SHA-256 the engine publishes it with before it is read. A set with a
   * hole (a block missing between two others, or a planet file without
   * its Moon) is refused with the files that fill it.
   */
  readonly files?: readonly TeimerisFile[];
  /**
   * `false` reads a file that is not the published copy, claiming only
   * the years its name gives; `true` by default.
   */
  readonly verify?: boolean;
  /** The engine's accuracy profile; `compatible` by default. */
  readonly profile?: 'compatible' | 'max' | 'COMPATIBLE' | 'MAX';
}

/**
 * The parts of the `teimeris-wasm` module the provider reads, so this
 * package does not depend on the engine's package to type-check.
 */
export interface TeimerisWasm {
  readonly Context: new (options?: { profile?: number }) => unknown;
  readonly Body: Readonly<Record<string, number>>;
  readonly Flags: Readonly<Record<string, number>>;
  readonly Status: Readonly<Record<string, number>>;
  readonly TimeScale: { readonly TT: number; readonly UT1: number };
  readonly Profile: { readonly COMPATIBLE: number; readonly MAX: number };
  ready(): boolean;
  version(): string;
  embeddedFiles(): readonly { readonly name: string }[];
}

/**
 * The provider, as the SDK's `provider` or an `ephemeris` chain takes it:
 * Nepal's official days in a browser are
 * `new Context(nepalOfficial(await teimerisWeb(engine)))`.
 *
 * Rejects when a file is not the published copy, names no block, is
 * given twice, or leaves a hole, and when `engine` has not been
 * initialised.
 */
export function teimerisWeb(
  engine: TeimerisWasm,
  options?: TeimerisWebOptions,
): Promise<EphemerisProvider>;
