// The lazy entry as a consumer reads it (`lib/lazy.d.ts`); `check-wasm`
// compiles this from inside the staged package.
import { INTERNAL_CODE, loadCalendar, loadJyotish, onInternalError } from '../lib/lazy.js';
import type { InternalReport, Teistro } from '../lib/lazy.js';

const calendar: Promise<Teistro> = loadCalendar();
const jyotish: Promise<Teistro> = loadJyotish();
const stop: () => void = onInternalError((report: InternalReport) => {
  void [report.kind, report.code === INTERNAL_CODE];
});
stop();
void (await calendar).Context;
void (await jyotish).Context;
// @ts-expect-error a load takes nothing
void loadCalendar('jyotish');
// @ts-expect-error an entry is a module of the layer, and has no such member
void (await calendar).loadJyotish;
