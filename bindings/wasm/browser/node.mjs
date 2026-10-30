/**
 * The probe under Node, through the staged package's own `node` loader,
 * for the gate to hold the browser's answer against.
 *
 * Usage: node node.mjs <staged package>
 */

import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { lazyProbe, probe } from './probe.mjs';

const entry = (file) => import(pathToFileURL(resolve(process.argv[2], 'lib', file)).href);
const answer = { ...probe(await entry('index.js')), lazy: await lazyProbe(await entry('lazy.js')) };
process.stdout.write(`${JSON.stringify({ answer })}\n`);
