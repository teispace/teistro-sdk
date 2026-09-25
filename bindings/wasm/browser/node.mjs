/**
 * The probe under Node, through the staged package's own `node` loader,
 * for the gate to hold the browser's answer against.
 *
 * Usage: node node.mjs <staged package>
 */

import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { probe } from './probe.mjs';

const sdk = await import(pathToFileURL(resolve(process.argv[2], 'lib/index.js')).href);
process.stdout.write(`${JSON.stringify({ answer: probe(sdk) })}\n`);
