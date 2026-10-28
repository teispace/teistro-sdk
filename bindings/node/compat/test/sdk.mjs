// `@teistro/sdk` resolved to this checkout's own binding, for the tests.
//
// The package takes the SDK as a peer, which an application installs
// beside it; a checkout has no `node_modules` to find it in. The two are
// released in lockstep (`docs/03-design/compat-shim.md` §6.2), so the
// binding beside this package is the version it is written against.
// Loaded with `node --import`, before any test imports the package.

import { registerHooks } from 'node:module';

const sdk = new URL('../../lib/index.js', import.meta.url).href;

registerHooks({
  resolve: (specifier, context, next) =>
    specifier === '@teistro/sdk' ? { url: sdk, shortCircuit: true } : next(specifier, context),
});
