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
