// The typed accessors: every message of the SDK's own locale as a
// function of its parameters, and every catalogued entity as its forms.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { Context } from '../lib/index.js';
import { entityForms, messages } from '../lib/messages.js';

const context = () =>
  new Context({ profile: 'nepali-default', locale: 'ne-Deva-NP', testProvider: true });

test('a message is spelled by its accessor, never by its key', () => {
  const ctx = context();
  assert.equal(
    ctx.messages.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 }),
    ctx.render('sdk.reason.grahaInBhava', {
      graha: { $entity: 'graha.JUPITER' },
      bhava: 7,
    }).text,
    'the accessor wraps the entity as the engine takes it',
  );
  assert.match(ctx.messages.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 }), /७/u);

  ctx.locale = 'en-Latn';
  assert.equal(
    ctx.messages.sdk.calendar.BIKRAM_SAMBAT.date.long({ day: 1, monthName: 'Baisakh', year: 2072 }),
    '1 Baisakh 2072 BS',
  );
  assert.equal(
    ctx.messages.sdk.calendar.GREGORIAN.date.numeric({ day: 14, month: 4, year: 2015 }),
    '2015-04-14',
  );
});

test("an entity's forms come from the locale, not from the caller", () => {
  const ctx = context();
  const sun = ctx.entity('graha.SUN');
  assert.equal(sun.name, 'सूर्य');
  assert.equal(sun.iast, 'Sūrya');
  assert.equal(sun.glyph, '☉');
  assert.equal(sun.gender, 'm');
  assert.deepEqual(ctx.messages.sdk.entity.graha.SUN(), sun, 'the accessor reads the same forms');
  assert.equal(Object.isFrozen(sun), true);

  ctx.locale = 'en-Latn';
  assert.equal(ctx.entity('graha.SUN').name, 'Sun');
  assert.equal(ctx.entity('rashi.ARIES').name, 'Aries');
});

test('an entity the locale does not carry is refused by name', () => {
  const ctx = context();
  assert.throws(
    () => ctx.entity('graha.PLUTO'),
    (error) => error.status === 'unsupported' && error.field === 'key',
  );
});

test('the accessors are a tree over any renderer', () => {
  const asked = [];
  const tree = messages({
    render: (key, params) => {
      asked.push([key, params]);
      return key;
    },
    entity: (key) => entityForms({ name: key }),
  });
  assert.equal(tree.sdk.reason.welcome(), 'sdk.reason.welcome');
  assert.equal(tree.sdk.entity.graha.KETU().name, 'graha.KETU');
  tree.sdk.calendar.ghati.long({ ghati: 12, pala: 30 });
  assert.deepEqual(asked.at(-1), ['sdk.calendar.ghati.long', { ghati: 12, pala: 30 }]);
  assert.equal(entityForms('{"name":"x"}').iast, '', 'a form the locale lacks is empty');
});
