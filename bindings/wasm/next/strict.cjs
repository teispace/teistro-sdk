/**
 * Makes Next.js's local edge runtime refuse what the deployed one
 * refuses: compiling WebAssembly from bytes.
 *
 * Next.js documents `WebAssembly.compile`, and `instantiate` given bytes,
 * as unsupported in the Edge runtime, but the sandbox `next start` runs
 * an edge route in leaves code generation as `vm` defaults it, allowed.
 * So a route that compiles its module from bytes builds and answers
 * locally and fails once deployed: the package's web loader, picked
 * without an `edge-light` condition, passed `next build --webpack` and
 * `next start` that way. Loaded with `--require` before Next.js, this
 * turns WebAssembly code generation off in every context `vm` creates,
 * which is what the edge sandbox is made with, and leaves the rest of
 * each context's settings as they were; a module compiled ahead of time
 * still instantiates.
 */

'use strict';

const vm = require('node:vm');

const createContext = vm.createContext;
vm.createContext = (sandbox, options = {}) =>
  createContext(sandbox, {
    ...options,
    codeGeneration: { strings: options.codeGeneration?.strings ?? true, wasm: false },
  });
