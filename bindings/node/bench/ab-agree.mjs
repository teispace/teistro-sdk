// What the two sides of the A/B answer, compared field by field
// (`docs/03-design/ab-baseline.md` §10). Each side reads its own answers
// into facts in one vocabulary, `[key, value]` with the key
// `family/what`: the SDK's reader is `@teistro/compat`'s `facts`
// (`compat/lib/facts.js`, `03-design/compat-shim.md` §5), the baseline's
// is the adapter's `facts`. The families say how two values compare:
// continuous ones by their difference, in a stated unit and against a
// stated tolerance, categorical ones by equality.

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

// The package's `@teistro/sdk` is this checkout's binding, as in its own
// tests; the hook resolves the imports made after it, so the vocabulary
// is imported once it is in place.
import '../compat/test/sdk.mjs';

const { FAMILIES, facts } = await import('../compat/lib/facts.js');

export { FAMILIES };

const root = join(fileURLToPath(new URL('.', import.meta.url)), '../../..');

/**
 * The yogas and doshas both catalogues name, each with the baseline's key
 * or keys for it: the SDK's rule for the baseline's figure where the rules
 * crate's own comparison names one (`crates/rules/tests/same-figures.json`),
 * and the baseline's own rules the SDK ships under their keys.
 *
 * @returns {[string, string[]][]}
 */
export function figures() {
  const read = (path) => JSON.parse(readFileSync(join(root, path), 'utf8'));
  const own = ['computed-doshas.json', 'computed-yogas.json'].flatMap((file) =>
    read(`crates/rules/rules/${file}`).rules.map((rule) => [rule.key, [rule.key]]),
  );
  return [...read('crates/rules/tests/same-figures.json'), ...own];
}

/**
 * The SDK's facts for one request's answer, in the vocabulary of
 * {@link FAMILIES}: the package's `facts`, given the input's Sade Sati
 * window and the joined figures.
 *
 * @param {string} request
 * @param {any} answer
 * @param {import('./ab-inputs.mjs').Input} input the input it answered
 * @param {[string, string[]][]} joined {@link figures}
 * @returns {[string, unknown][]}
 */
export function sdkFacts(request, answer, input, joined) {
  return facts(request, answer, { figures: joined, window: input.window });
}

/**
 * The baseline's yoga facts in the joined vocabulary: a figure is present
 * where any of the baseline's keys for it is.
 *
 * @param {[string, unknown][]} facts the adapter's, `yoga/<its key>` among them
 */
export function joinYogas(facts, joined) {
  if (!facts.some(([key]) => key.startsWith('yoga/'))) return facts;
  const present = new Set(facts.filter(([key, value]) => key.startsWith('yoga/') && value).map(([key]) => key.slice(5)));
  const suffix = (key) => key.slice(key.indexOf('@'));
  const rest = facts.filter(([key]) => !key.startsWith('yoga/'));
  // A batch's yogas carry their chart's `@i`; each chart is joined alone.
  const charts = [...new Set(facts.filter(([key]) => key.startsWith('yoga/') && key.includes('@')).map(([key]) => suffix(key)))];
  if (charts.length === 0) {
    return [...rest, ...joined.map(([key, theirs]) => [`yoga/${key}`, theirs.some((k) => present.has(k))])];
  }
  return [
    ...rest,
    ...charts.flatMap((at) => joined.map(([key, theirs]) => [`yoga/${key}${at}`, theirs.some((k) => present.has(`${k}${at}`))])),
  ];
}

/** The family a key belongs to. */
const familyOf = (key) => key.slice(0, key.indexOf('/'));

/** How far apart two values of a continuous family are, in its unit. */
function apart(family, a, b) {
  const { kind, scale } = FAMILIES[family];
  const d = kind === 'angle' ? Math.abs(((a - b + 540) % 360) - 180) : Math.abs(a - b);
  return d * scale;
}

/** The mismatches a family's tally keeps to show. */
const SHOWN = 3;

/** What a mismatch between the two sides calls their values. */
const SIDE_NAMES = ['sdk', 'baseline'];

/**
 * A running comparison of two sets of facts, over every input and request,
 * read out as one line a request and family: a side's against the other
 * side's, or against a reference's. `names` are what a mismatch calls the
 * two values.
 */
export class Agreement {
  constructor(names = SIDE_NAMES) {
    this.names = names;
    /** @type {Map<string, {request: string, family: string, diffs: number[], worst: {id: string, key: string, apart: number} | null, matches: number, mismatches: Record<string, unknown>[], mismatched: number, unpaired: number, inputs: Set<string>}>} */
    this.tallies = new Map();
  }

  tally(request, family) {
    const at = `${request} ${family}`;
    let t = this.tallies.get(at);
    if (!t) {
      t = { request, family, diffs: [], worst: null, matches: 0, mismatches: [], mismatched: 0, unpaired: 0, inputs: new Set() };
      this.tallies.set(at, t);
    }
    return t;
  }

  /** One input's two sets of facts, for one request. */
  add(request, id, facts, others) {
    const theirs = new Map(others);
    const ours = new Map(facts);
    const [a, b] = this.names;
    for (const [key, value] of ours) {
      const family = familyOf(key);
      if (!(family in FAMILIES)) throw new Error(`${request}: ${key} is in no family`);
      const t = this.tally(request, family);
      t.inputs.add(id);
      if (!theirs.has(key)) {
        t.unpaired += 1;
        continue;
      }
      const other = theirs.get(key);
      if (FAMILIES[family].kind === 'category') {
        if (value === other) t.matches += 1;
        else {
          t.mismatched += 1;
          if (t.mismatches.length < SHOWN) t.mismatches.push({ id, key, [a]: value, [b]: other });
        }
      } else {
        const d = apart(family, value, other);
        t.diffs.push(d);
        if (t.worst === null || d > t.worst.apart) t.worst = { id, key, apart: d };
      }
    }
    for (const key of theirs.keys()) {
      if (!ours.has(key) && familyOf(key) in FAMILIES) this.tally(request, familyOf(key)).unpaired += 1;
    }
  }

  /**
   * One line a request and family: a continuous family's median and
   * largest difference and the share within its tolerance, a categorical
   * one's share of exact matches and its first mismatches.
   */
  lines(head) {
    return [...this.tallies.values()].map((t) => {
      const family = FAMILIES[t.family];
      const base = { ...head, request: t.request, family: t.family, inputs: t.inputs.size, unpaired: t.unpaired };
      if (family.kind === 'category') {
        const compared = t.matches + t.mismatched;
        return { ...base, compared, matches: t.matches, share: compared ? t.matches / compared : 0, mismatches: t.mismatches };
      }
      const sorted = [...t.diffs].sort((a, b) => a - b);
      const within = sorted.filter((d) => d <= family.tolerance).length;
      return {
        ...base,
        compared: sorted.length,
        matches: within,
        share: sorted.length ? within / sorted.length : 0,
        unit: family.unit,
        tolerance: family.tolerance,
        median: sorted[sorted.length >> 1] ?? 0,
        max: sorted.at(-1) ?? 0,
        worst: t.worst,
      };
    });
  }
}
