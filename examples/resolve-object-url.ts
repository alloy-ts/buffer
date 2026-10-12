/**
 * examples/resolve-object-url.ts
 *
 * Documents the `resolveObjectURL` gap. In the real Node.js `node:buffer`,
 * `resolveObjectURL(id)` resolves a `blob:nodedata:` URL (created via the
 * internal Blob registry) back into a `Blob`. This binding does NOT implement
 * that registry, so the function is a stub that always throws. This example
 * shows the behavior so consumers know not to depend on it yet.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/resolve-object-url.ts
 */
import { resolveObjectURL } from "../index.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== resolveObjectURL (stub — not supported) ===");

let threw = false;
let message = "";
try {
  resolveObjectURL("blob:nodedata:abc123");
} catch (err) {
  threw = true;
  message = err instanceof Error ? err.message : String(err);
}

log("resolveObjectURL threw", threw); // true
log("error message", message); // explains blob: URLs are not supported

console.log("\nTakeaway: blob: URL support is unimplemented in this binding.");
console.log("Use Blob.arrayBuffer() / Blob.bytes() / Blob.text() to read Blob data directly.");
