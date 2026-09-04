// Six mistakes a consumer could make; each must be a compile error.
import { messages, type Renderer } from './sdk.ts';

declare const r: Renderer;
const t = messages(r);
t.sdk.reason.grahaInBhava({ graha: 'graha.PLUTO', bhava: 7 });
t.sdk.reason.grahaInBhava({ graha: 'graha.MARS' });
t.sdk.reason.greeting({ gender: 'x', name: 'S' });
t.sdk.reason.strength.rank({ rank: 'third' });
t.sdk.reason.nothing();
r.render('sdk.reason.nowhere');
