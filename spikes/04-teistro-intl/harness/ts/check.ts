// Runtime check of the generated accessors: every call reaches the
// renderer with the key and the parameters it was given, and nothing else.
import assert from 'node:assert/strict';
import { messages, type EntityForms, type EntityKey, type MessageKey, type Renderer } from './sdk.ts';

const calls: Array<[string, unknown]> = [];
const renderer: Renderer = {
  render(key: MessageKey, params?: Readonly<Record<string, unknown>>): string {
    calls.push([key, params ?? null]);
    return key;
  },
  entity(key: EntityKey): EntityForms {
    calls.push(['entity', key]);
    return { short: key, name: key, prose: key, iast: key };
  },
};

const t = messages(renderer);
assert.equal(t.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 }), 'sdk.reason.grahaInBhava');
t.sdk.reason.greeting({ gender: 'f', name: 'Sita' });
t.sdk.reason.occupants({ grahas: ['graha.SUN', 'graha.MOON'], rashi: 'rashi.LEO' });
t.sdk.reason.strength.rank({ rank: 3 });
t.sdk.reason.appName();
assert.equal(t.sdk.entity.graha.SUN().name, 'graha.SUN');
assert.deepEqual(calls, [
  ['sdk.reason.grahaInBhava', { graha: 'graha.JUPITER', bhava: 7 }],
  ['sdk.reason.greeting', { gender: 'f', name: 'Sita' }],
  ['sdk.reason.occupants', { grahas: ['graha.SUN', 'graha.MOON'], rashi: 'rashi.LEO' }],
  ['sdk.reason.strength.rank', { rank: 3 }],
  ['sdk.reason.appName', null],
  ['entity', 'graha.SUN'],
]);

export const checked = calls.length;
