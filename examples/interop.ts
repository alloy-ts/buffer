/**
 * examples/interop.ts
 *
 * Demonstrates `Buffer` as a `Uint8Array` subclass and its interoperability
 * with the rest of the JavaScript typed-array ecosystem: sharing memory with
 * an `ArrayBuffer`, copying the raw bytes of wider TypedArrays / DataViews via
 * `copyBytesFrom`, and exposing `.buffer` / `.byteOffset` / `.byteLength`.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/interop.ts
 */
import { Buffer } from "../index.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== Buffer is a Uint8Array ===");

const buf = Buffer.from([1, 2, 3]);
log("buf instanceof Uint8Array", buf instanceof Uint8Array); // true
log("buf instanceof Buffer", buf instanceof Buffer); // true
// It can be passed wherever a Uint8Array is expected.
const u8view = new Uint8Array(buf);
log("new Uint8Array(buf) shares data", u8view.length); // 3

console.log("\n=== Sharing memory with an ArrayBuffer ===");

const ab = new ArrayBuffer(4);
const u8over = new Uint8Array(ab);
const shared = Buffer.from(ab); // view over the same memory
shared[0] = 42;
log("mutating Buffer reflects in Uint8Array", u8over[0]); // 42
log("shared byteOffset", shared.byteOffset); // 0
log("shared.byteLength", shared.byteLength); // 4
log("shared.buffer === ab", shared.buffer === ab); // true

// With an explicit offset/length the Buffer is a partial view.
const partial = Buffer.from(ab, 1, 2);
partial[0] = 7;
log("partial view mutates underlying ArrayBuffer", u8over[1]); // 7

console.log("\n=== copyBytesFrom: raw bytes of wider views ===");

// copyBytesFrom copies the underlying bytes of a TypedArray/DataView, not just
// the low byte per element (unlike Buffer.from(typedArray)).
const f64 = new Float64Array([1.0, 2.0]); // 16 bytes
const fromF64 = Buffer.copyBytesFrom(f64);
log("copyBytesFrom(Float64Array[2]) length", fromF64.length); // 16

const dv = new DataView(new ArrayBuffer(4));
dv.setUint8(0, 0xaa);
dv.setUint8(3, 0xbb);
const fromDv = Buffer.copyBytesFrom(dv);
log("copyBytesFrom(DataView) bytes", Array.from(fromDv)); // [ 170, 0, 0, 187 ]

// Partial copy via offset/length (in bytes).
const u32 = new Uint32Array([0x01020304]);
const partialCopy = Buffer.copyBytesFrom(u32, 1, 2);
log("copyBytesFrom(Uint32Array, 1, 2)", Array.from(partialCopy)); // [ 2, 3 ]

console.log("\nAll interop examples executed successfully.");
