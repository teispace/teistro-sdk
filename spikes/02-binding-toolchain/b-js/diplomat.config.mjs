/**
 * Where Diplomat's generated loader finds the wasm module: the workspace's
 * release build of the bridge for `wasm32-unknown-unknown`.
 */
import { fileURLToPath } from 'node:url';

export default {
  wasm_path: fileURLToPath(
    new URL('../../../target/wasm32-unknown-unknown/release/teistro_spike_b_bridge.wasm', import.meta.url),
  ),
};
