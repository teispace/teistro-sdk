/**
 * What the browser check computes, written once and run twice: in a
 * headless browser against the wasm package's web loader, and in Node
 * against its Node loader. The gate holds the two answers equal, so the
 * browser path is not only loaded but agrees to the bit.
 *
 * @param {typeof import('../../node/lib/index.js')} sdk the package
 * @returns {object} what the gate compares
 */
export function probe(sdk) {
  const { Body, Context, buildInfo } = sdk;
  const ctx = new Context({ profile: 'parashari-classical', ephemeris: 'BUILTIN' });
  try {
    const sky = ctx.positions({
      instants: [2451545.0, 2460000.5],
      bodies: [Body.Sun, Body.Moon, Body.Mars],
    });
    const longitudes = [];
    for (let instant = 0; instant < 2; instant += 1) {
      for (let body = 0; body < 3; body += 1) {
        // The exact bits, as text: a browser's answer and Node's are
        // compared, not rounded.
        longitudes.push(sky.at(instant, body).longitude.toString());
      }
    }
    let refusal = null;
    try {
      new Context({ ephemeris: { plugin: '/any/adapter.so' } }).dispose();
    } catch (error) {
      refusal = `${error.name}: ${error.message}`;
    }
    return {
      target: buildInfo.target,
      settingsHash: ctx.settingsHash,
      longitudes,
      refusal,
    };
  } finally {
    ctx.dispose();
  }
}

/**
 * The same again through the lazy entry: the calendar module first, which
 * refuses a chart as a capability, then the jyotish module, which draws
 * one, and the calendar answer each gives. Loading twice is one load.
 *
 * @param {typeof import('../lib/lazy.js')} lazy the lazy entry
 * @returns {Promise<object>} what the gate compares
 */
export async function lazyProbe(lazy) {
  const calendar = await lazy.loadCalendar();
  if (calendar !== (await lazy.loadCalendar())) throw new Error('the calendar loaded twice');
  const answers = (sdk) => {
    const ctx = new sdk.Context({ profile: 'nepali-default', ephemeris: 'BUILTIN' });
    try {
      const day = {
        calendar: sdk.Calendar.Gregorian,
        year: 2015,
        eraYear: 0,
        month: 4,
        day: 14,
        resolution: sdk.Resolution.Defined,
        computedMonth: 0,
        computedDay: 0,
      };
      let chart = 'answered';
      try {
        ctx.chart.found({ instant: 2451545, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0 });
      } catch (error) {
        chart = error.status ?? String(error);
      }
      return { bikramSambat: ctx.calendar.convert(day, sdk.Calendar.BikramSambat), chart };
    } finally {
      ctx.dispose();
    }
  };
  return { calendar: answers(calendar), jyotish: answers(await lazy.loadJyotish()) };
}
