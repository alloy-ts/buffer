/**
 * examples/file.ts
 *
 * Demonstrates the `File` class (a `Blob` superset carrying a name and
 * modification time) shipped by `@alloy-ts/buffer`.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/file.ts
 *
 * Note: unlike the Web `File` (whose `text()`/`bytes()`/`arrayBuffer()` return
 * Promises), this native binding implements them synchronously.
 */
import { File, Blob, Buffer } from "../dist/index.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== File basics ===");

// A File is a Blob plus a `name` and `lastModified` timestamp.
const file = new File(["config data"], "config.txt", {
  type: "text/plain",
  lastModified: 1_700_000_000_000,
});

log("file.name", file.name); // "config.txt"
log("file.lastModified", file.lastModified); // 1700000000000
log("file.type", file.type); // "text/plain"
log("file.size", file.size); // 11

// Synchronous content accessors (NOT promises in this binding).
log("file.text()", file.text()); // "config data"
log("file.bytes() is a TypedArray view", ArrayBuffer.isView(file.bytes())); // true
log("file.arrayBuffer() is ArrayBuffer", file.arrayBuffer() instanceof ArrayBuffer); // true

// A File built without `lastModified` falls back to Date.now().
const now = Date.now();
const autoFile = new File(["x"], "auto.txt");
log("auto lastModified close to now", Math.abs(autoFile.lastModified - now) < 5000); // true

console.log("\n=== File slicing preserves metadata ===");

// `slice` returns a new File that keeps the name / type / lastModified.
const slice = file.slice(0, 6, "text/plain");
log("slice is still a File", slice instanceof File); // true
log("slice.name (preserved)", slice.name); // "config.txt"
log("slice.lastModified (preserved)", slice.lastModified); // 1700000000000
log("slice.type", slice.type); // "text/plain"
log("slice.text()", slice.text()); // "config"

console.log("\n=== File vs Blob relationship ===");

// Both share the same accessor method set in this binding. They are distinct
// native classes, so `instanceof Blob` is false even though File "is a" Blob
// conceptually (the wrapper does not re-declare the subtype relationship).
log("file instanceof File", file instanceof File); // true
log("file instanceof Blob", file instanceof Blob); // false (separate native class)
log("File and Blob are distinct classes", File !== Blob); // true

// A File can be used wherever a Blob is expected (same read API).
const asBlob: Blob = file as unknown as Blob;
log("file used as Blob: size", asBlob.size); // 11

console.log("\nAll File examples executed successfully.");
