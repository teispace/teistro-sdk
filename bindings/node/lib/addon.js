/**
 * The Node package's native half: finds the prebuilt addon and loads it.
 *
 * Node's alone, and the only file of the layer that is: `index.js`
 * imports what this exports through the package's `#native` import, and
 * the wasm package maps the same name to its own loader, so the layer
 * above is one file for both (`03-design/wasm-binding.md` §3).
 */

import { createRequire } from 'node:module';
import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

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

function addonName() {
  if (process.platform === 'darwin') return 'libteistro_node.dylib';
  if (process.platform === 'win32') return 'teistro_node.dll';
  return 'libteistro_node.so';
}

const [loaded, wasNamed] = loadAddon();

/** The addon's exports: the native object the layer calls. */
export const native = loaded;

/**
 * Whether the addon was named by `TEISTRO_ADDON` rather than found: a
 * deliberate choice, which may load a development build.
 */
export const named = wasNamed;
