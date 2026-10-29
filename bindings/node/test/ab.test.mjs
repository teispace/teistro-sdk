// The A/B runner (`bench/ab.mjs`, `bench/ab-load.mjs`,
// `docs/03-design/ab-baseline.md`) without the baseline engine: a stand-in
// adapter answers through the SDK itself, so what is tested is the runner,
// not either engine.

import assert from 'node:assert/strict';
import test from 'node:test';

import { Empty, REQUESTS, SAME_SKY_DEG, agreeing, inputs, interleaved, openSdk, surfaceCount } from '../bench/ab.mjs';
import { counts, drive, measured, threadAgent, trials } from '../bench/ab-load.mjs';
import { Agreement, figures, joinYogas } from '../bench/ab-agree.mjs';
import { Kept, charakRung, kalaprakasikaRung, ramanRung, whoKey } from '../bench/ab-reference.mjs';

/** The SDK's side over the built-in ephemeris, disposed after `body`. */
function withSdk(body) {
  const sdk = openSdk('BUILTIN', 'sdk');
  try {
    return body(sdk.side);
  } finally {
    sdk.dispose();
  }
}

/** The two requests the runner's own tests time: the chart, whose Suns are compared, and a day. */
const FEW = ['chart', 'day'];

test('every golden input is read, with its instant, place, civil date and what its requests need', () => {
  const all = inputs();
  assert.equal(all.length, 55, 'the corpus holds 55 births');
  const first = all[0];
  assert.equal(first.id, 'c001');
  assert.equal(first.civilDate.year, 1990);
  assert.equal(first.utcOffsetSeconds, 345 * 60);
  assert.ok(Math.abs(first.jdUt - 2447995.4895833335) < 1e-9);
  assert.equal(first.utc, '1990-04-13T23:45:00.000Z');
  assert.equal(first.iana, 'Asia/Kathmandu');
  assert.equal(first.partner.id, 'c002');
  assert.equal(all.at(-1).partner.id, 'c001', 'the last birth is matched with the first');
  assert.equal(first.instants.length, 10);
  assert.ok(Math.abs(first.instants[1] - first.jdUt - 1 / 24) < 1e-9, 'an hour apart');
  assert.equal(first.window.from, first.jdUt);
  assert.ok(Math.abs(first.window.to - first.jdUt - 30 * 365.25) < 1e-6);
  assert.equal(first.transitAt, first.window.to);
  // The muhurta's 25 days, thirty years on from the civil date, at Kathmandu.
  assert.deepEqual(first.muhurta, {
    from: '2020-04-14',
    to: '2020-05-08',
    iana: 'Asia/Kathmandu',
    utcOffsetSeconds: 345 * 60,
    place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
  });
});

test('the SDK answers every request the runner times, on one birth', () => {
  withSdk((sdk) => {
    const [first] = inputs();
    for (const request of REQUESTS) assert.ok(sdk[request](first), request);
  });
});

test('a muhurta search is presented as the baseline ranks it', () => {
  withSdk((sdk) => {
    const [first] = inputs();
    const facts = new Map(sdk.facts('muhurta', sdk.muhurta(first), first));
    for (const kind of ['general', 'native']) {
      const ranking = facts.get(`muhurta/${kind}/ranking`);
      const rows = ranking === '' ? [] : ranking.split(' ').map((row) => row.split('='));
      assert.ok(rows.length > 0, `${kind}: the window has open days`);
      assert.ok(rows.length <= 30, 'thirty rows at most');
      assert.equal(facts.get(`muhurta/${kind}/count`), rows.length);
      const days = new Map();
      for (const [key, score] of rows) {
        assert.match(key, /^\d{4}-\d{2}-\d{2}#\d{1,2}b?$/);
        assert.equal(facts.get(`muhurta/${kind}/window/${key}`), Number(score));
        const date = key.slice(0, 10);
        days.set(date, (days.get(date) ?? 0) + 1);
        assert.ok(facts.has(`muhurta/${kind}/day/${date}`), date);
      }
      assert.ok([...days.values()].every((n) => n <= 2), 'two windows a day at most');
      const scores = rows.map(([, score]) => Number(score));
      assert.deepEqual(scores, [...scores].sort((a, b) => b - a), 'highest first');
    }
  });
});

test('the baseline\'s entry points are counted against the shim\'s rows, or not counted at all', () => {
  assert.equal(surfaceCount({}), undefined, 'an adapter with no surface is not counted as none');
  const counted = surfaceCount(
    {
      surface: [
        { entry: 'the foundation', row: 'chart' },
        { entry: 'the daily panchanga', row: 'day' },
        { entry: 'something the shim lacks', row: null },
        { entry: 'a row of another name', row: 'horoscope' },
      ],
    },
    ['chart', 'day'],
  );
  assert.deepEqual(counted, { kind: 'surface', entries: 4, unmapped: 2, unknownRows: ['horoscope'] });
  // Against the shim's own surface.json, its rows are all mapped.
  assert.equal(surfaceCount({ surface: [{ entry: 'the foundation', row: 'chart' }] }).unmapped, 0);
  assert.throws(() => surfaceCount({ surface: 'chart' }), TypeError);
});

test('a side that computed another sky is kept out of the timing', () => {
  withSdk((sdk) => {
    // A stand-in an hour late: the Sun moves 2.5′, past the arcminute.
    const late = { ...sdk, chart: (input) => sdk.chart({ ...input, jdUt: input.jdUt + 1 / 24 }) };
    const few = inputs().slice(0, 3);
    assert.deepEqual(agreeing(sdk, { ...sdk }, few, FEW).differ, []);
    const { kept, differ } = agreeing(sdk, late, few, FEW);
    assert.deepEqual([kept.chart.length, kept.day.length], [0, 0]);
    assert.equal(differ.length, 3);
    assert.ok(Math.abs(differ[0].a - differ[0].b) > SAME_SKY_DEG);
  });
});

test('a request a side refuses keeps that input out of that request alone', () => {
  withSdk((sdk) => {
    const few = inputs().slice(0, 3);
    const edge = few[1].id;
    const refusing = {
      ...sdk,
      day: (input) => {
        if (input.id === edge) throw new Error('past the data');
        return sdk.day(input);
      },
    };
    const { kept, differ, refused } = agreeing(sdk, refusing, few, FEW);
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
    const lines = interleaved([sdk, refusing], kept, 1, FEW);
    assert.deepEqual(
      lines.map((l) => `${l.request} ${l.inputs}`),
      ['chart 3', 'chart 3', 'day 2', 'day 2'],
    );
  });
});

test('an empty answer keeps that input out of that request, reported apart from the refusals', () => {
  withSdk((sdk) => {
    const few = inputs().slice(0, 3);
    const edge = few[2].id;
    const emptying = {
      ...sdk,
      day: (input) => (input.id === edge ? new Empty('answered nothing') : sdk.day(input)),
    };
    const { kept, refused, empty } = agreeing(emptying, sdk, few, FEW);
    assert.deepEqual(refused, []);
    assert.deepEqual(empty, [{ id: edge, side: 0, request: 'day', reason: 'answered nothing' }]);
    assert.deepEqual(
      kept.day.map((input) => input.id),
      [few[0].id, few[1].id],
    );
    assert.equal(kept.chart.length, 3);
  });
});

test('both sides are timed per request, alternating, under the results schema', () => {
  withSdk((inner) => {
    const order = [];
    const side = (name) => ({
      ...inner,
      version: name,
      chart: (input) => {
        order.push(name);
        return inner.chart(input);
      },
    });
    const one = inputs().slice(0, 1);
    const lines = interleaved([side('a'), side('b')], one, 3, FEW);
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
  });
});

test('the load counts double up to the machine and take it whole', () => {
  assert.deepEqual(counts(1), [1]);
  assert.deepEqual(counts(4), [1, 2, 4]);
  assert.deepEqual(counts(12), [1, 2, 4, 8, 12]);
});

test('a trial counts what every agent answered inside its span, a second', async () => {
  const agents = [7, 5].map((answered) => ({ trial: async () => answered }));
  assert.deepEqual(await trials(agents, 2, 2), [6, 6]);
});

test('an agent answers inside its span, and refuses a chart whose Sun moved under load', () => {
  withSdk((side) => {
    const all = inputs().slice(0, 2);
    const suns = all.map((input) => side.sun(side.chart(input)));
    const startAt = performance.timeOrigin + performance.now();
    assert.ok(drive({ side, all, suns }, 'chart', startAt, 0.05) > 0);
    const shared = { ...side, sun: (answer) => side.sun(answer) + 1e-9 };
    assert.throws(
      () => drive({ side: shared, all, suns }, 'chart', startAt, 0.05),
      /c001: the Sun under load stands at/,
    );
  });
});

test('worker threads each open their own side and are measured together', async () => {
  const ids = inputs()
    .slice(0, 2)
    .map((input) => input.id);
  const pair = [0, 1].map(() => threadAgent('sdk', 'chart', ids, 'BUILTIN'));
  const { rps, reason } = await measured(pair, 2, 0.2);
  assert.equal(reason, undefined);
  assert.equal(rps.length, 2);
  assert.ok(rps.every((n) => n > 0));
  // A side that cannot be opened in a worker is a reason, not a crash.
  delete process.env.TEISTRO_AB_BASELINE;
  const away = await measured([threadAgent('baseline', 'chart', ids)], 1, 0.1);
  assert.match(away.reason, /set TEISTRO_AB_BASELINE/);
});

test('a side agrees with itself on every fact of every request it answers', () => {
  withSdk((sdk) => {
    const few = inputs().slice(0, 2);
    const agreement = new Agreement();
    for (const request of ['chart', 'day', 'strength', 'milan', 'varsha', 'kp', 'gochar']) {
      for (const input of few) {
        const facts = sdk.facts(request, sdk[request](input), input);
        assert.ok(facts.length > 0, `${request} holds facts`);
        agreement.add(request, input.id, facts, facts);
      }
    }
    const lines = agreement.lines({ kind: 'agree', convention: 'timed' });
    assert.ok(lines.length >= 7);
    for (const line of lines) {
      assert.equal(line.share, 1, `${line.request} ${line.family}`);
      assert.equal(line.unpaired, 0);
      if (line.unit) assert.equal(line.max, 0);
    }
  });
});

test('a fact apart is counted in its unit, and a fact one side lacks is unpaired', () => {
  const agreement = new Agreement();
  const ours = [['longitude/MOON', 10], ['sign/MOON', 0], ['event/sunrise', 2451545], ['koota/TOTAL', 20]];
  const theirs = [['longitude/MOON', 10 + 2 / 3600], ['sign/MOON', 1], ['event/sunrise', 2451545 + 30 / 86400]];
  agreement.add('chart', 'c001', ours, theirs);
  const by = Object.fromEntries(agreement.lines({}).map((l) => [l.family, l]));
  assert.equal(by.longitude.unit, '″');
  assert.ok(Math.abs(by.longitude.max - 2) < 1e-6);
  assert.equal(by.longitude.share, 0, 'two arcseconds is past the tolerance');
  assert.equal(by.event.share, 1, 'thirty seconds is within it');
  assert.deepEqual(by.sign.mismatches, [{ id: 'c001', key: 'sign/MOON', sdk: 0, baseline: 1 }]);
  assert.equal(by.koota.unpaired, 1);
  assert.equal(by.koota.compared, 0);
});

test('a yoga is joined only where an answer holds yogas', () => {
  const joined = figures();
  assert.ok(joined.length > 0);
  const facts = [['sign/SUN', 0]];
  assert.deepEqual(joinYogas(facts, joined), facts);
});

test('the worked examples judge the SDK, and a reading the source names is its own row', () => {
  const opened = [];
  const sdk = openSdk('BUILTIN', 'sdk');
  try {
    const who = {
      side: 'sdk',
      convention: 'timed',
      answers: sdk.side,
      reopened: (patch) => {
        const again = openSdk('BUILTIN', 'sdk', 'timed', patch);
        opened.push(again);
        return again.side;
      },
    };
    const charak = charakRung([who]);
    const offices = charak.find((l) => l.family === 'office');
    assert.equal(offices.compared, 7);
    assert.equal(offices.share, 1, JSON.stringify(offices.mismatches));
    const raman = ramanRung([who]);
    const source = raman.find((l) => l.convention === 'source');
    assert.equal(source.rung, 'raman');
    assert.equal(source.compared, 7);
    assert.ok(source.max < 1, `Sripati's readings reproduce the printed totals, ${source.max}`);
    // Kalaprakasika reads each side's own Moons from its kept charts.
    const few = inputs().slice(0, 3);
    const byId = new Map(inputs().map((input) => [input.id, input]));
    const kept = new Kept();
    for (const input of inputs()) {
      kept.add(whoKey(who), 'chart', input.id, sdk.side.facts('chart', sdk.side.chart(input), input));
    }
    for (const input of few) {
      kept.add(whoKey(who), 'milan', input.id, sdk.side.facts('milan', sdk.side.milan(input), input));
    }
    const [lines] = kalaprakasikaRung([who], kept, byId);
    assert.equal(lines.family, 'porutham');
    assert.equal(lines.compared, 9);
    assert.equal(lines.share, 1, JSON.stringify(lines.mismatches));
  } finally {
    for (const again of opened) again.dispose();
    sdk.dispose();
  }
});
