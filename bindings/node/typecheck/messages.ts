// The typed accessors' declarations, type-checked with the rest.
//
// Every `@ts-expect-error` is a proof: a key that is not a key, a
// parameter of the wrong type, a missing parameter and a write to a
// result are all compile errors, so a surface that stops refusing one
// fails this file.

import type {
  EntityForms,
  GrahaKey,
  MessageKey,
  Messages,
  Renderer,
} from '../lib/messages.js';

declare const m: Messages;
declare const r: Renderer;

/** Every accessor, typed. */
function scenario(): string {
  const bhava = m.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 });
  const welcome = m.sdk.reason.welcome();
  const date = m.sdk.calendar.BIKRAM_SAMBAT.date.long({
    day: 1,
    monthName: 'Baisakh',
    year: 2072,
  });
  const sun: EntityForms = m.sdk.entity.graha.SUN();
  const key: MessageKey = 'sdk.reason.grahaInBhava';
  const rendered = r.render(key, { bhava: 7 });
  return `${bhava}${welcome}${date}${sun.name}${sun.short ?? ''}${rendered}`;
}

// @ts-expect-error a graha is named by its catalogue key
const wrongEntity: GrahaKey = 'graha.PLUTO';
// @ts-expect-error a message key is one of the locale's own
const wrongKey: MessageKey = 'sdk.reason.nope';
// @ts-expect-error a bhava is a number
const wrongParam = m.sdk.reason.grahaInBhava({ graha: 'graha.SUN', bhava: '7' });
// @ts-expect-error every parameter of a message is required
const missingParam = m.sdk.reason.grahaInBhava({ graha: 'graha.SUN' });
// @ts-expect-error the accessors are read, never replaced
m.sdk = {} as Messages['sdk'];
// @ts-expect-error a form the locale may not carry is checked before use
const unchecked: string = m.sdk.entity.graha.SUN().short;

export { missingParam, scenario, unchecked, wrongEntity, wrongKey, wrongParam };
