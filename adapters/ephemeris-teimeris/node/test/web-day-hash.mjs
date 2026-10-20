// **The answer a browser must give**: the content hash of one month's
// almanac at Kathmandu under each setup over Teimeris, through the SDK's
// wasm build and the engine's own wasm module, positions only. It prints
// the lines `teistro-ephemeris-teimeris-day-hash` prints natively, and
// `cargo xtask check-wasm` holds the two equal byte for byte.
//
//   TEIMERIS_WASM_PACKAGE=<teimeris>/bindings/wasm \
//   node test/web-day-hash.mjs <staged @teistro/sdk-wasm>

import { createRequire } from 'node:module';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { teimerisWeb } from '../web.js';

const staged = resolve(process.argv[2]);
const sdk = await import(pathToFileURL(join(staged, 'lib', 'index.js')).href);
const { Calendar, Resolution } = await import(pathToFileURL(join(staged, 'lib', 'catalogue.js')).href);
const engine = createRequire(import.meta.url)(join(process.env.TEIMERIS_WASM_PACKAGE, 'lib', 'index.js'));
await engine.init();

const date = (day) => ({
  calendar: Calendar.Gregorian,
  year: 2025,
  eraYear: 0,
  month: 11,
  day,
  resolution: Resolution.Defined,
  computedMonth: 0,
  computedDay: 0,
});
const month = {
  from: date(1),
  to: date(30),
  place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
  utcOffsetSeconds: 20700,
};
const setups = [
  ['drik-teimeris', async () => ({ profile: 'parashari-classical', provider: await teimerisWeb(engine) })],
  ['committee-teimeris', async () => sdk.nepalOfficial(await teimerisWeb(engine))],
];
for (const [key, options] of setups) {
  const context = new sdk.Context(await options());
  try {
    const { provenance } = context.almanac.of(month);
    process.stdout.write(`${key} settings ${provenance.settingsHash} content ${provenance.contentHash}\n`);
  } finally {
    context.dispose();
  }
}
