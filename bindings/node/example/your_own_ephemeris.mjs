// An ephemeris of your own, and what happens when it goes wrong.
//
// The SDK computes no positions itself: it asks a **provider**, and a
// provider written in JavaScript is a first-class one. That is the point
// of the port — an application that already has an ephemeris, a cache, or
// a table of precomputed positions can put it behind the SDK and get the
// whole chart layer for free.
//
// The contract is small and worth reading carefully:
//
// - **One call for the whole grid, never a loop.** The SDK hands over
//   every instant and every body at once and expects every cell back.
// - **Return nothing for "not in that frame".** Answering at all asserts
//   the answer is in the frame that was asked for, so a provider that
//   computes in one frame must compare `request.frameBits` with its own
//   and return nothing instead; the SDK then asks again in the
//   provider's native frame and completes the rest itself, stamping each
//   step. This is why an engine that knows nothing about the ayanamsha
//   can serve a Vedic chart.
// - **Say what you cover.** `bodies`, `jdMin` and `jdMax` are checked
//   *before* the provider is called, so a request it cannot serve is
//   refused by name rather than by a wrong answer.
// - **Throwing is allowed.** What it threw reaches the caller, so the
//   sentence is not lost; only a code crosses the C ABI.

import {
  Ayanamsha,
  Body,
  Context,
  canonicalFrame,
  packFrame,
} from '../lib/index.js';

/**
 * An ephemeris backed by whatever you already have.
 *
 * This one is a two-body toy — a circular Sun and Moon — standing in for
 * the real thing: a `.se1` reader, a JPL kernel, a database of
 * precomputed rows, or a cache in front of any of them. What matters is
 * the shape, not the arithmetic.
 */
function tableEphemeris({ wantedFrame = null, broken = false } = {}) {
  const state = { calls: 0, cells: 0, refusals: 0 };
  return {
    state,
    name: 'table-ephemeris',
    version: '1.0.0',
    // What identifies the data, not the code. A result's provenance
    // carries it, so two runs against different data are distinguishable
    // even when the code is identical.
    dataVersion: 'demo-rows-2025a',
    bodies: [Body.Sun, Body.Moon],
    // Cover only what you have. A request outside this is refused before
    // `positions` is ever called.
    jdMin: 2451545.0,
    jdMax: 2469807.0,
    positions(request) {
      if (broken) {
        throw new Error('ephemeris file de431.eph is not where the index says');
      }
      // **Check the frame first.** Answering at all asserts that the
      // answer is in the frame that was asked for; a provider that
      // computes only its own must say so by returning nothing.
      if (wantedFrame !== null && request.frameBits !== wantedFrame) {
        state.refusals += 1;
        return null;
      }
      state.calls += 1;
      const cells = request.jds.length * request.bodies.length;
      state.cells += cells;

      // Cells run instants outermost: cell `i * bodies.length + j` is
      // instant `i`, body `j`. Building the columns in that order is the
      // whole of the contract.
      const lon = new Float64Array(cells);
      const lonSpeed = new Float64Array(cells);
      let at = 0;
      for (const jd of request.jds) {
        const days = jd - 2451545.0;
        for (const body of request.bodies) {
          const rate = body === Body.Sun ? 0.9856 : 13.1764;
          const start = body === Body.Sun ? 280.46 : 218.32;
          lon[at] = (start + rate * days) % 360;
          lonSpeed[at] = rate;
          at += 1;
        }
      }
      // Speeds are optional; a column left out is zeroes. Saying
      // `speeds: false` would tell the SDK not to expect them at all.
      return { lon, lat: new Float64Array(cells), dist: new Float64Array(cells).fill(1), lonSpeed };
    },
  };
}

// ── The happy path ─────────────────────────────────────────────────────
{
  const provider = tableEphemeris();
  const ctx = new Context({ profile: 'parashari-classical', provider });
  const sky = ctx.positions({
    instants: Array.from({ length: 7 }, (_, day) => 2451545.0 + day),
    bodies: [Body.Sun, Body.Moon],
  });
  console.log(`asked    ${provider.state.calls} time(s) for ${provider.state.cells} cells`);
  console.log(`answered ${sky.cells.length} cells over ${sky.jdCount} days`);
  console.log(
    `  sun  ${sky.at(0, 0).longitude.toFixed(4).padStart(8)}°` +
      ` -> ${sky.at(6, 0).longitude.toFixed(4).padStart(8)}° in a week`,
  );
  console.log(
    `  moon ${sky.at(0, 1).longitude.toFixed(4).padStart(8)}°` +
      ` -> ${sky.at(6, 1).longitude.toFixed(4).padStart(8)}° in a week`,
  );
  // The provider's own name and data version are stamped on the answer,
  // which is how a stored chart says what computed it.
  console.log(`  stamped as ${JSON.stringify(sky.provenance.provider)}`);
  ctx.dispose();
}

// ── A body it never declared ───────────────────────────────────────────
{
  const provider = tableEphemeris();
  const ctx = new Context({ profile: 'parashari-classical', provider });
  try {
    ctx.positions({ instants: [2451545.0], bodies: [Body.Saturn] });
  } catch (error) {
    console.log('');
    console.log(`refused  ${error.message}`);
    console.log(`         and the provider was asked ${provider.state.calls} times`);
  }
  ctx.dispose();
}

// ── An instant outside its coverage ────────────────────────────────────
{
  const ctx = new Context({ profile: 'parashari-classical', provider: tableEphemeris() });
  try {
    ctx.positions({ instants: [2200000.0], bodies: [Body.Sun] });
  } catch (error) {
    console.log(`refused  ${error.message}`);
  }
  ctx.dispose();
}

// ── A frame it does not compute ────────────────────────────────────────
// This provider computes tropical positions and declares no native frame,
// so the canonical one is what it answers. Ask for a sidereal zodiac and
// the SDK does the rest, naming every step it applied.
{
  const canonical = canonicalFrame();
  const provider = tableEphemeris({ wantedFrame: packFrame(canonical) });
  const ctx = new Context({ profile: 'parashari-classical', provider });
  const tropical = ctx.positions({ instants: [2451545.0], bodies: [Body.Sun] });
  const sidereal = ctx.positions({
    instants: [2451545.0],
    bodies: [Body.Sun],
    frame: { ...canonical, sidereal: true, ayanamsha: Ayanamsha.Lahiri },
  });
  const steps = sidereal.steps.map((step) => `${step.name}:${step.implementation}`).join(', ');
  console.log('');
  console.log(
    `frames   the provider answered ${tropical.at(0, 0).longitude.toFixed(4)}° tropical;` +
      ` a sidereal request is ${sidereal.at(0, 0).longitude.toFixed(4)}°`,
  );
  console.log(`         it refused the frame ${provider.state.refusals} time(s), and the`);
  console.log(`         SDK completed it: ${steps}`);
  ctx.dispose();
}

// ── When the provider itself fails ─────────────────────────────────────
{
  const ctx = new Context({ profile: 'parashari-classical', provider: tableEphemeris({ broken: true }) });
  try {
    ctx.positions({ instants: [2451545.0], bodies: [Body.Sun] });
  } catch (error) {
    console.log('');
    console.log(`thrown   ${error.constructor.name}: ${error.message}`);
    console.log('         the error itself crossed back, not just a code');
  }
  ctx.dispose();
}

// ── A refusal a user should see ────────────────────────────────────────
// Every refusal from the library carries a status a program can match on
// and, where the boundary knows one, the detail and a hint to act on.
// That is what to put in front of a person.
{
  const ctx = new Context({ profile: 'parashari-classical' });
  try {
    ctx.keys.id('graha.SUNN');
  } catch (error) {
    console.log('');
    console.log(`status   ${error.status}`);
    console.log(`message  ${error.message}`);
    console.log(`detail   ${error.detail ?? ''}`);
    console.log(`hint     ${error.hint ?? ''}`);
    console.log(
      '         a program matches on `status`; a person reads the message and the hint',
    );
  }
  ctx.dispose();
}
