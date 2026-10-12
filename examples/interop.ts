/**
 * examples/interop.ts
 *
 * Demonstrates `Buffer` interoperability with the rest of the JavaScript
 * typed-array ecosystem: sharing memory with an `ArrayBuffer`, copying the
 * raw bytes of wider TypedArrays / DataViews via `copyBytesFrom`, and exposing
 * `.buffer` / `.byteOffset` / `.byteLength`.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/interop.ts
 */
import { Buffer } from "../src/main.ts";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== Buffer & ArrayBuffer ===");

const buf = Buffer.from([1, 2, 3]);
log("buf length", buf.length);

const ab = new ArrayBuffer(4);
const shared = Buffer.from(ab);
log("shared byteOffset", shared.byteOffset); // 0
log("shared.byteLength", shared.length); // 4

const partial = Buffer.from(ab, 1, 2);
log("partial view length", partial.length); // 2

console.log("\n=== copyBytesFrom: raw bytes of wider views ===");

const f64 = new Float64Array([1.0, 2.0]); // 16 bytes
const fromF64 = Buffer.copyBytesFrom(f64);
log("copyBytesFrom(Float64Array[2]) length", fromF64.length); // 16

const dv = new DataView(new ArrayBuffer(4));
dv.setUint8(0, 0xaa);
dv.setUint8(3, 0xbb);
const fromDv = Buffer.copyBytesFrom(dv);
log("copyBytesFrom(DataView) bytes", Array.from(fromDv)); // [ 170, 0, 0, 187 ]

const u32 = new Uint32Array([0x01020304]);
const partialCopy = Buffer.copyBytesFrom(u32, 1, 2);
log("copyBytesFrom(Uint32Array, 1, 2)", Array.from(partialCopy)); // [ 2, 3 ]

console.log("\nAll interop examples executed successfully.");
