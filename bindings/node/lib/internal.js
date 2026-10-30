/**
 * The SDK failing inside itself, as this layer reports it: never expected,
 * and never silent (`03-design/wasm-binding.md` §4, "A panic traps").
 *
 * The C ABI turns a panic into the `INTERNAL` status before it can cross,
 * so the Node addon's calls refuse with it like any refusal. A wasm module
 * cannot: `wasm32-unknown-unknown` aborts on a panic, and the abort reaches
 * JavaScript as a `WebAssembly.RuntimeError` that leaves the module's
 * memory mid-call. That trap is reported here as the same `INTERNAL`
 * status, and every later call is refused with it, since nothing the
 * module holds can be trusted until it is loaded again.
 *
 * A host's crash reporter hears of either through `onInternalError`, with
 * the status, its code, which of the two it was and the SDK's version:
 * never the message, which may quote what the call was given.
 */

/** The stable code of the `INTERNAL` status, the same in every binding. */
export const INTERNAL_CODE = -10;

/** @type {Set<(report: InternalReport) => void>} */
const hooks = new Set();

/**
 * @typedef {object} InternalReport
 * @property {'INTERNAL'} status
 * @property {number} code `INTERNAL_CODE`
 * @property {'boundary' | 'trap'} kind a panic the C ABI caught, or a wasm
 *   module that trapped
 * @property {string} sdkVersion the SDK that failed
 */

/**
 * Calls `hook` with every `InternalReport` from now on; the function it
 * returns stops that.
 *
 * @param {(report: InternalReport) => void} hook
 * @returns {() => void}
 */
export function onInternalError(hook) {
  if (typeof hook !== 'function') {
    throw new TypeError('onInternalError takes a function');
  }
  hooks.add(hook);
  return () => {
    hooks.delete(hook);
  };
}

/**
 * Tells every hook. A hook that throws is not the caller's failure, so
 * what it throws is dropped and the others still hear.
 *
 * @param {'boundary' | 'trap'} kind
 * @param {string} sdkVersion
 */
export function reportInternal(kind, sdkVersion) {
  const report = Object.freeze({ status: 'INTERNAL', code: INTERNAL_CODE, kind, sdkVersion });
  for (const hook of hooks) {
    try {
      hook(report);
    } catch {
      // A crash reporter's own failure is its own.
    }
  }
}

/**
 * Whether `cause` is a wasm module's trap.
 *
 * @param {unknown} cause
 */
export function isTrap(cause) {
  return typeof WebAssembly !== 'undefined' && cause instanceof WebAssembly.RuntimeError;
}

/** The record a trap is refused with, and every call after it. */
export function trapRecord(first) {
  return {
    status: 'INTERNAL',
    code: INTERNAL_CODE,
    detail: null,
    message: first
      ? 'the SDK stopped inside its WebAssembly module (a trap, never expected); load the module again'
      : 'an earlier call stopped inside the WebAssembly module, which cannot be trusted since; load it again',
    field: null,
    hint: 'report it with the call that trapped: support@teispace.com',
  };
}
