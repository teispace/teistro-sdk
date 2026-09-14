// The layer's declarations against the layer itself, member by member.
//
// `index.d.ts` is hand-written beside `index.js`, and `tsc` proves only that
// what a program USES is declared. A getter the runtime has and the
// declarations lack is invisible to every type-checked file that does not
// happen to use it — which is how a chart's divisional charts, drishti,
// points, bhavas and states crossed the boundary, were compared by the
// parity runner (untyped `.mjs`), and could not be named by a TypeScript
// consumer at all.
//
// So this asks both sides. The runtime answers for itself: every exported
// class, its prototype's members and its static ones. The declarations are
// asked through the pinned compiler, by writing a throwaway file of
// type-level assertions — `Exclude<runtime, keyof Declared>` must be
// `never`, and so must the reverse — and type-checking it at the same
// strictness as the rest. A failure names the members, because the
// compiler prints the type that was not `never`. Both directions fail: a
// member declared and absent is a type that lies, which is worse than one
// that is silent. Symbol-keyed members (`[Symbol.iterator]`) are compared by
// neither side, and `#private` ones exist on neither.
//
// The compiler is TypeScript 7, the native port, which has no JavaScript
// API to parse a declaration with — hence a file for it to check rather than
// a tree to walk. Run by `cargo xtask check-node`, from `bindings/node`.

import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const layerUrl = new URL('../lib/index.js', import.meta.url);
const layer = await import(layerUrl.href);
const tsc = fileURLToPath(new URL('node_modules/typescript/bin/tsc', import.meta.url));

/** Whether an export is a class, as opposed to a function or a value. */
const isClass = (value) =>
  typeof value === 'function' && /^class\b/.test(Function.prototype.toString.call(value));

/**
 * One real instance of every exported class, made the way a consumer makes
 * it — because a member can live on the instance rather than the
 * prototype (a context's areas are frozen fields) and the only way to see
 * those is to hold one. A class exported with no recipe here fails the
 * gate, so a new one cannot slip past it unmeasured.
 */
function instances() {
  const { Calendar, ChartKind, Context, TeistroError, Body, date } = layer;
  const ctx = new Context({ profile: 'parashari-classical', ephemeris: 'builtin' });
  const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
  const charts = ctx.chart.foundMany({ instants: [2460482.5], place, utcOffsetSeconds: 20700, kind: ChartKind.Natal });
  let error;
  const bare = new Context({ profile: 'parashari-classical' });
  try {
    // A refusal the layer raises as its own error: no ephemeris to ask.
    bare.positions({ instants: [2451545], bodies: [Body.Sun] });
  } catch (caught) {
    error = caught;
  }
  bare.dispose();
  const made = {
    Context: ctx,
    Positions: ctx.positions({ instants: [2451545], bodies: [Body.Sun] }),
    Charts: charts,
    Chart: charts.at(0),
    Almanac: ctx.almanac.of({ from: date(Calendar.Gregorian, 2024, 6, 21), to: date(Calendar.Gregorian, 2024, 6, 21), place, utcOffsetSeconds: 20700 }),
    AlmanacDay: ctx.almanac.day({ date: date(Calendar.Gregorian, 2024, 6, 21), place, utcOffsetSeconds: 20700 }),
    Rendered: ctx.intl.render('entity.graha.sun.name'),
    TeistroError: error instanceof TeistroError ? error : undefined,
  };
  return { made, dispose: () => ctx.dispose() };
}

/**
 * The string-keyed members an instance answers to, own and inherited, up
 * to the first built-in ancestor — whose members (`Error`'s `message`,
 * `Object`'s `toString`) belong to the language and are subtracted from the
 * declarations the same way.
 */
function held(instance) {
  const names = new Set(Object.getOwnPropertyNames(instance));
  let proto = Object.getPrototypeOf(instance);
  while (proto && proto !== Object.prototype && proto !== Error.prototype) {
    for (const name of Object.getOwnPropertyNames(proto)) {
      if (name !== 'constructor') names.add(name);
    }
    proto = Object.getPrototypeOf(proto);
  }
  // A member that overrides the language's own — an error's `message`, a
  // `toString` — is the language's member, declared by the language.
  for (const name of language) names.delete(name);
  return [...names];
}

/**
 * The language's own member names — `Object`'s and `Error`'s — which a class
 * may override and a declaration may restate, and which neither side's
 * comparison counts.
 */
const language = new Set([
  ...[Object.prototype, Error.prototype, new Error()].flatMap((one) => Object.getOwnPropertyNames(one)),
  // ES2022's `Error.cause`, which the type declares and an error made
  // without one does not carry as an own property.
  'cause',
]);

/** A union of string literal types, or `never` for none. */
const union = (names) => (names.length === 0 ? 'never' : names.map((n) => JSON.stringify(n)).join(' | '));

const classes = Object.entries(layer).filter(([, value]) => isClass(value));
const { made, dispose } = instances();
const lines = [
  '// Generated by typecheck/surface.mjs for one run and deleted after it.',
  `import type * as Layer from ${JSON.stringify(fileURLToPath(layerUrl))};`,
  '',
  '/** Compiles only when its type argument is `never`. */',
  'declare function none<T extends never>(): void;',
  '/** The string-keyed members a declared type names. */',
  'type Named<T> = Extract<keyof T, string>;',
  '',
];
const unmeasured = [];
for (const [name] of classes) {
  const instance = made[name];
  if (!instance) {
    unmeasured.push(name);
    continue;
  }
  const members = union(held(instance));
  const declared = `Exclude<Named<Layer.${name}>, ${union([...language])}>`;
  lines.push(
    `// ${name}: at run time, and not declared.`,
    `none<Exclude<${members}, ${declared}>>();`,
    `// ${name}: declared, and not at run time.`,
    `none<Exclude<${declared}, ${members}>>();`,
  );
}
dispose();
if (unmeasured.length > 0) {
  console.log(`no instance to measure for ${unmeasured.join(', ')}: add a recipe to typecheck/surface.mjs`);
  process.exit(1);
}

const scratch = mkdtempSync(join(tmpdir(), 'teistro-surface-'));
try {
  writeFileSync(join(scratch, 'surface.ts'), `${lines.join('\n')}\n`);
  writeFileSync(
    join(scratch, 'tsconfig.json'),
    JSON.stringify({
      extends: fileURLToPath(new URL('tsconfig.json', import.meta.url)),
      include: ['surface.ts'],
    }),
  );
  const run = spawnSync(process.execPath, [tsc, '-p', 'tsconfig.json'], {
    cwd: scratch,
    encoding: 'utf8',
  });
  if (run.status !== 0) {
    // The compiler names the members; the comment above the assertion it
    // refused names the class and the direction, and the file is gone once
    // this exits, so both are printed together here.
    const found = [...run.stdout.matchAll(/surface\.ts\((\d+),\d+\): error TS\d+: Type '?(.*?)'? does not satisfy/g)];
    for (const [, line, members] of found) {
      console.log(`  ${lines[Number(line) - 2].replace(/^\/\/ /, '')} ${members}`);
    }
    if (found.length === 0) process.stdout.write(run.stdout + run.stderr);
    console.log('the layer and index.d.ts disagree');
    process.exit(1);
  }
  console.log(
    `${classes.length} exported classes, measured on real instances: every member is declared, and none is declared that is not there`,
  );
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
