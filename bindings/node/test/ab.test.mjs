// The A/B runner (`bench/ab.mjs`, `docs/03-design/ab-baseline.md`) without
// the baseline engine: a stand-in adapter answers through the SDK itself,
// so what is tested is the runner, not either engine.

import assert from 'node:assert/strict';
import test from 'node:test';

import { SAME_SKY_DEG, agreeing, inputs, interleaved, sdkSide } from '../bench/ab.mjs';
import { Context } from '../lib/index.js';

test('every golden input is read, with its instant, place and civil date', () => {
  const all = inputs();
  assert.equal(all.length, 55, 'the corpus holds 55 births');
  const first = all[0];
  assert.equal(first.id, 'c001');
  assert.equal(first.civilDate.year, 1990);
  assert.equal(first.utcOffsetSeconds, 345 * 60);
  assert.ok(Math.abs(first.jdUt - 2447995.4895833335) < 1e-9);
  assert.equal(first.utc, '1990-04-13T23:45:00.000Z');
  assert.equal(first.iana, 'Asia/Kathmandu');
});

test('a side that computed another sky is kept out of the timing', () => {
  const ctx = new Context({ profile: 'parashari-classical', ephemeris: 'BUILTIN' });
  try {
    const sdk = sdkSide(ctx, 'sdk');
    const same = { ...sdkSide(ctx, 'stand-in') };
    // A stand-in an hour late: the Sun moves 2.5′, past the arcminute.
    const late = { ...same, chart: (input) => same.chart({ ...input, jdUt: input.jdUt + 1 / 24 }) };
    const few = inputs().slice(0, 3);
    assert.deepEqual(agreeing(sdk, same, few).differ, []);
    const { kept, differ } = agreeing(sdk, late, few);
    assert.deepEqual([kept.chart.length, kept.day.length], [0, 0]);
    assert.equal(differ.length, 3);
    assert.ok(Math.abs(differ[0].a - differ[0].b) > SAME_SKY_DEG);
  } finally {
    ctx.dispose();
  }
});

test('a request a side refuses keeps that input out of that request alone', () => {
  const ctx = new Context({ profile: 'parashari-classical', ephemeris: 'BUILTIN' });
  try {
    const sdk = sdkSide(ctx, 'sdk');
    const inner = sdkSide(ctx, 'stand-in');
    const few = inputs().slice(0, 3);
    const edge = few[1].id;
    const refusing = {
      ...inner,
      day: (input) => {
        if (input.id === edge) throw new Error('past the data');
        return inner.day(input);
      },
    };
    const { kept, differ, refused } = agreeing(sdk, refusing, few);
    assert.deepEqual(differ, []);
    assert.deepEqual(refused, [{ id: edge, side: 1, request: 'day', reason: 'past the data' }]);
    assert.deepEqual(
      kept.chart.map((input) => input.id),
      few.map((input) => input.id),
    );
    assert.deepEqual(
      kept.day.map((input) => input.id),
      [few[0].id, few[2].id],
    );
    const lines = interleaved([sdk, refusing], kept, 1);
    assert.deepEqual(
      lines.map((l) => `${l.request} ${l.inputs}`),
      ['chart 3', 'chart 3', 'day 2', 'day 2'],
    );
  } finally {
    ctx.dispose();
  }
});

test('both sides are timed per request, alternating, under the results schema', () => {
  const ctx = new Context({ profile: 'parashari-classical', ephemeris: 'BUILTIN' });
  try {
    const order = [];
    const side = (name) => {
      const inner = sdkSide(ctx, name);
      return {
        ...inner,
        chart: (input) => {
          order.push(name);
          return inner.chart(input);
        },
      };
    };
    const one = inputs().slice(0, 1);
    const lines = interleaved([side('a'), side('b')], one, 3);
    assert.deepEqual(
      lines.map((l) => `${l.side} ${l.request}`),
      ['sdk chart', 'baseline chart', 'sdk day', 'baseline day'],
    );
    // A warm-up each, then a, b; b, a; a, b.
    assert.deepEqual(order, ['a', 'b', 'a', 'b', 'b', 'a', 'a', 'b']);
    for (const l of lines) {
      assert.equal(l.rounds, 3);
      assert.equal(l.inputs, 1);
      assert.ok(l.min_us <= l.median_us && l.median_us <= l.max_us, JSON.stringify(l));
    }
  } finally {
    ctx.dispose();
  }
});
