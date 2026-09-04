// Type-checks the generated surface, proves the wrong usages are compile
// errors, and runs the runtime check. Prints one JSON line.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { checked } from './check.ts';

function tsc(project: string): { status: number | null; errors: number; out: string } {
  const result = spawnSync(process.execPath, ['node_modules/typescript/bin/tsc', '--noEmit', '-p', project], {
    encoding: 'utf8',
  });
  return { status: result.status, errors: (result.stdout.match(/error TS\d+/g) ?? []).length, out: result.stdout };
}

const good = tsc('tsconfig.json');
assert.equal(good.status, 0, good.out);
const wrong = tsc('tsconfig.wrong.json');
assert.equal(wrong.errors, 6, wrong.out);
console.log(JSON.stringify({ typecheck: 'ok', wrongUsagesRejected: wrong.errors, runtimeCalls: checked }));
