// The shim's call surface counted (`docs/03-design/compat-shim.md` §6.4,
// item 7): `surface.json` lists every row of §3 with its kind, and this
// file holds the package to it. The functions a shim answers are exactly
// the rows; each refused row throws `CompatRefused` before any
// computation, naming a guide anchor that exists, and is typed `never` in
// the declarations; each forward and compose row is declared, named in
// the design page's §3, and called by `agree.test.mjs`, which holds it to
// the documented SDK call.

import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { test } from 'node:test';

import { CompatRefused, compat } from '@teistro/compat';

const here = (path) => new URL(path, import.meta.url);
const repo = (path) => new URL(`../../../../${path}`, import.meta.url);
const text = (url) => readFileSync(url, 'utf8');

const { rows } = JSON.parse(text(here('../surface.json')));
const KINDS = ['forward', 'compose', 'refuse'];

/** The anchor a Markdown heading gets: lower-cased, punctuation dropped, spaces as hyphens. */
const slug = (heading) =>
  heading
    .toLowerCase()
    .replace(/[^\p{L}\p{N} -]/gu, '')
    .replace(/ /g, '-');

/** Every heading anchor of a Markdown file. */
const anchors = (url) =>
  new Set(
    text(url)
      .split('\n')
      .filter((line) => /^#{1,6} /.test(line))
      .map((line) => slug(line.replace(/^#+ /, ''))),
  );

test('every row is named once, with its section, its entry points, its kind and its SDK call', () => {
  const names = rows.map(({ row }) => row);
  assert.equal(new Set(names).size, names.length, 'a row is named once');
  for (const row of rows) {
    assert.match(row.row, /^[a-z][A-Za-z]*$/, row.row);
    assert.match(row.section, /^3\.[1-6](, 3\.[1-6])*$/, row.row);
    assert.ok(KINDS.includes(row.kind), row.row);
    assert.ok(row.entry.length > 0 && row.sdk.length > 0, row.row);
    assert.equal(row.guide !== undefined, row.kind === 'refuse', `${row.row}: a refusal and only a refusal names a guide`);
    assert.equal(row.instead !== undefined, row.kind === 'refuse', `${row.row}: and says what to do instead`);
  }
});

test('a shim answers exactly the rows', () => {
  const shim = compat({ ephemeris: 'TEST' });
  const own = Object.getOwnPropertyNames(Object.getPrototypeOf(shim)).filter(
    (name) => !['constructor', 'calls', 'dispose'].includes(name),
  );
  assert.deepEqual(own.sort(), rows.map(({ row }) => row).sort());
  shim.dispose();
});

test('each refused row throws CompatRefused at once, naming its row and a guide anchor that exists', () => {
  const shim = compat({ ephemeris: 'TEST' });
  for (const { row, guide, instead } of rows.filter(({ kind }) => kind === 'refuse')) {
    assert.throws(
      () => shim[row]({ not: 'read' }, { not: 'translated' }),
      (error) => error instanceof CompatRefused && error.row === row && error.guide === guide && error.message === instead,
      row,
    );
    const [path, anchor] = guide.split('#');
    assert.ok(existsSync(repo(path)), `${row}: ${path}`);
    assert.ok(anchors(repo(path)).has(anchor), `${row}: #${anchor} in ${path}`);
  }
  // A refused call is a call left to port, so it is counted.
  const refused = rows.filter(({ kind }) => kind === 'refuse').map(({ row }) => row);
  const counts = shim.calls();
  assert.deepEqual(
    refused.map((row) => counts[row]),
    refused.map(() => 1),
  );
  shim.dispose();
});

test('the declarations type every row, a refused one as never', () => {
  const declared = text(here('../lib/index.d.ts'));
  const shim = declared.slice(declared.indexOf('export interface Shim {'), declared.indexOf('\n}\n', declared.indexOf('export interface Shim {')));
  assert.match(declared, /export type Refused = \(\.\.\.args: readonly unknown\[\]\) => never;/);
  for (const { row, kind } of rows) {
    if (kind === 'refuse') assert.match(shim, new RegExp(`\\n  ${row}: Refused;`), row);
    else assert.match(shim, new RegExp(`\\n  ${row}\\(`), row);
  }
});

test('each row is in the design page’s §3, and each answering row is held by agree.test.mjs', () => {
  const page = text(repo('docs/03-design/compat-shim.md'));
  const surface = page.slice(page.indexOf('## 3. The call surface'), page.indexOf('### 3.7'));
  const agree = text(here('agree.test.mjs'));
  for (const { row, kind } of rows) {
    assert.ok(surface.includes(`| \`${row}\` |`), `${row} is a row of §3`);
    if (kind !== 'refuse') assert.match(agree, new RegExp(`\\bs\\.${row}\\(`), `${row} is asked in agree.test.mjs`);
  }
});
