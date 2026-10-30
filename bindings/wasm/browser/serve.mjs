/**
 * The server the browser checks share: the staged package at `/`, this
 * directory under `/check/`, and a page's report at `POST /result`.
 */

import { spawn } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = fileURLToPath(new URL('.', import.meta.url));
const DEADLINE_MS = 120_000;
const TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  // `instantiateStreaming` compiles only what is served as wasm.
  '.wasm': 'application/wasm',
};

/**
 * Serves the package, opens `page` in a headless Chrome and resolves with
 * what the page posts back. The page reports by posting rather than being
 * read out of the DOM, so nothing depends on how long a browser takes to
 * compile a module: the run ends when the page says so, or at the deadline.
 *
 * @param {string} packageDir the staged package
 * @param {string} chrome the browser
 * @param {string} page the path to open
 * @param {{ profile?: string, refused?: Set<string> }} [options] a profile
 *   to keep, so a later run finds its caches, and the paths to fail once
 * @returns {Promise<string>} the body the page posted
 */
export function askChrome(packageDir, chrome, page, { profile, refused } = {}) {
  let settle;
  const reported = new Promise((resolve_) => {
    settle = resolve_;
  });
  const server = serving(packageDir, settle, refused);
  return new Promise((answered) => {
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      const directory = profile ?? mkdtempSync(join(tmpdir(), 'teistro-chrome-'));
      const browser = spawn(
        chrome,
        [
          '--headless=new',
          '--no-sandbox',
          '--disable-gpu',
          '--no-first-run',
          '--no-default-browser-check',
          `--user-data-dir=${directory}`,
          `http://127.0.0.1:${port}${page}`,
        ],
        { stdio: 'ignore' },
      );
      const deadline = setTimeout(
        () => settle(JSON.stringify({ error: `no report within ${DEADLINE_MS / 1000} s` })),
        DEADLINE_MS,
      );
      reported.then((body) => {
        clearTimeout(deadline);
        server.closeAllConnections();
        server.close();
        // The profile is removed once Chrome has let go of it: removing it
        // while the browser still writes there fails, and a leftover
        // temporary directory is not worth failing the check over.
        const finish = () => {
          if (!profile) {
            try {
              rmSync(directory, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
            } catch {
              // Left for the system's temporary-file cleaning.
            }
          }
          answered(body);
        };
        if (browser.exitCode !== null || browser.signalCode !== null) {
          finish();
        } else {
          browser.once('exit', finish);
          browser.kill();
        }
      });
    });
  });
}

/**
 * @param {string} packageDir the staged package
 * @param {(body: string) => void} settle told what the page posted
 * @param {Set<string>} [refused] paths answered 503 once, as a dropped
 *   connection would fail them
 * @returns {import('node:http').Server} not yet listening
 */
function serving(packageDir, settle, refused = new Set()) {
  const fileFor = (path) => {
    const [root, rest] = path.startsWith('/check/')
      ? [HERE, path.slice('/check/'.length)]
      : [resolve(packageDir), path.slice(1)];
    const file = normalize(join(root, rest || 'index.html'));
    return file.startsWith(root) ? file : null;
  };
  return createServer((request, response) => {
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
    const path = request.url.split('?')[0];
    if (refused.delete(path)) {
      response.writeHead(503);
      response.end();
      return;
    }
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
}
