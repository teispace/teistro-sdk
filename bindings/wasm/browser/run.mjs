/**
 * Runs the browser check: serves the staged wasm package and this page,
 * opens the page in a headless Chrome, and prints what the page posts
 * back — the probe's answer, or the error that stopped it.
 *
 * Usage: node run.mjs <staged package> <chrome>
 *
 * The page reports by posting to this server rather than being read out
 * of the DOM, so nothing depends on how long a browser takes to compile
 * the module: the run ends when the page says so, or at the deadline.
 */

import { spawn } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const [packageDir, chrome] = process.argv.slice(2);
if (!packageDir || !chrome) {
  console.error('usage: node run.mjs <staged package> <chrome>');
  process.exit(2);
}
const HERE = fileURLToPath(new URL('.', import.meta.url));
const DEADLINE_MS = 120_000;
const TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  // `instantiateStreaming` compiles only what is served as wasm.
  '.wasm': 'application/wasm',
};

/** The file a path names: the page's own files under `/check/`, the package's elsewhere. */
function fileFor(path) {
  const [root, rest] = path.startsWith('/check/')
    ? [HERE, path.slice('/check/'.length)]
    : [resolve(packageDir), path.slice(1)];
  const file = normalize(join(root, rest || 'index.html'));
  return file.startsWith(root) ? file : null;
}

let settle;
const reported = new Promise((resolve_) => {
  settle = resolve_;
});

const server = createServer((request, response) => {
  if (request.method === 'POST' && request.url === '/result') {
    let body = '';
    request.on('data', (chunk) => {
      body += chunk;
    });
    request.on('end', () => {
      response.end();
      settle(body);
    });
    return;
  }
  const path = request.url === '/' ? '/check/index.html' : request.url.split('?')[0];
  const file = fileFor(path);
  try {
    const bytes = readFileSync(file);
    response.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
    response.end(bytes);
  } catch {
    response.writeHead(404);
    response.end();
  }
});

server.listen(0, '127.0.0.1', () => {
  const { port } = server.address();
  const profile = mkdtempSync(join(tmpdir(), 'teistro-chrome-'));
  const browser = spawn(
    chrome,
    [
      '--headless=new',
      '--no-sandbox',
      '--disable-gpu',
      '--no-first-run',
      '--no-default-browser-check',
      `--user-data-dir=${profile}`,
      `http://127.0.0.1:${port}/`,
    ],
    { stdio: 'ignore' },
  );
  const deadline = setTimeout(
    () => settle(JSON.stringify({ error: `no report within ${DEADLINE_MS / 1000} s` })),
    DEADLINE_MS,
  );
  reported.then((body) => {
    clearTimeout(deadline);
    server.close();
    // The profile is removed once Chrome has let go of it: removing it
    // while the browser still writes there fails, and a leftover
    // temporary directory is not worth failing the check over.
    const finish = () => {
      try {
        rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
      } catch {
        // Left for the system's temporary-file cleaning.
      }
      process.stdout.write(`${body}\n`);
      process.exit(JSON.parse(body).answer ? 0 : 1);
    };
    if (browser.exitCode !== null || browser.signalCode !== null) {
      finish();
    } else {
      browser.once('exit', finish);
      browser.kill();
    }
  });
});
