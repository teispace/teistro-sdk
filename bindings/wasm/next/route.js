/**
 * The edge check's route: the installed package, imported by its name in
 * an edge route as a Next.js app imports it, and the browser check's
 * probe run once per request. It answers the probe's answer, or the error
 * that stopped it, as JSON, which `run.mjs` reads back.
 */

import * as sdk from '@teistro/sdk-wasm';

import { probe } from './probe.mjs';

export const runtime = 'edge';
export const dynamic = 'force-dynamic';

export function GET() {
  try {
    return Response.json({ answer: probe(sdk) });
  } catch (error) {
    return Response.json({ error: `${error.name}: ${error.message}` });
  }
}
