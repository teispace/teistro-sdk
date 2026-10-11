#!/usr/bin/env node
/**
 * `teistro-mcp` for a host that starts servers through Node: npm's
 * (`npx -y @teistro/mcp`), which installs the program in a platform
 * package beside this one, and Claude Desktop's bundle (`.mcpb`), which
 * carries one a platform under `platforms/`. Finds this host's and runs
 * it with this process's arguments, standard streams and exit status,
 * so the host speaks to the program itself.
 */

import { spawn } from 'node:child_process';
import { chmodSync, existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

/**
 * This host as the release names it: `<os>-<cpu>`, with `-musl` on a
 * Linux whose C library is musl. Node's report names the glibc it runs on
 * and has no such field under musl, which is how npm itself tells the
 * two apart when it matches a package's `libc`.
 */
function hostPlatform() {
  const name = `${process.platform}-${process.arch}`;
  if (process.platform !== 'linux') return name;
  const header = process.report?.getReport?.().header;
  return header && !header.glibcVersionRuntime ? `${name}-musl` : name;
}

/**
 * The program: the one `TEISTRO_MCP` names, else the bundle's for this
 * host, else the platform package's npm installed.
 */
function program() {
  const named = process.env.TEISTRO_MCP;
  if (named) return named;
  const file = process.platform === 'win32' ? 'teistro-mcp.exe' : 'teistro-mcp';
  const bundled = join(HERE, '..', 'platforms', hostPlatform(), file);
  if (existsSync(bundled)) return bundled;
  const platform = `@teistro/mcp-${hostPlatform()}`;
  try {
    return join(dirname(require.resolve(`${platform}/package.json`)), file);
  } catch {
    process.stderr.write(
      `teistro-mcp: no prebuilt server for ${hostPlatform()}: npm did not install ${platform}` +
        ' (a host no release covers, `--no-optional`, or a lockfile written elsewhere).' +
        ' Install it with `npm install ' +
        platform +
        '`, take the archive from the release, or set TEISTRO_MCP to the program.\n',
    );
    process.exit(1);
  }
}

const path = program();
if (!existsSync(path)) {
  process.stderr.write(`teistro-mcp: ${path} does not exist\n`);
  process.exit(1);
}
// npm keeps a file's mode in the tarball; a mirror that drops it would
// leave the program unrunnable, so it is set again where the platform has
// modes at all.
if (process.platform !== 'win32') {
  try {
    chmodSync(path, 0o755);
  } catch {
    // A read-only install: the mode is whatever it was, and spawn says.
  }
}

const child = spawn(path, process.argv.slice(2), { stdio: 'inherit', windowsHide: true });
// A signal the host sends this launcher is the server's to act on.
for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP']) {
  process.on(signal, () => {
    if (!child.killed) child.kill(signal);
  });
}
child.on('error', (error) => {
  process.stderr.write(`teistro-mcp: ${path} did not start: ${error.message}\n`);
  process.exit(1);
});
child.on('exit', (code, signal) => {
  if (signal) {
    // Ended by a signal, the launcher ends by it too, so the host sees
    // what the server saw; the handler above would otherwise catch it.
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
