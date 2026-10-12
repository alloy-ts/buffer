/**
 * examples/bytes.ts
 *
 * Demonstrates the native `Bytes` binding — a value-add (not part of
 * `node:buffer`) that wraps Rust's `bytes::Bytes`: reference-counted,
 * O(1) slicing/splitting of byte chunks.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/bytes.ts
 */
import { Bytes } from "../src/main.ts";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== Construction ===");

const empty = new Bytes();
log("new Bytes().isEmpty()", empty.isEmpty()); // true
log("new Bytes().len()", empty.len()); // 0

const bytes = Bytes.fromBuffer(globalThis.Buffer.from([10, 20, 30, 40, 50]));
log("fromBuffer len", bytes.len()); // 5
log("fromBuffer contents via buf()", Array.from(bytes.buf())); // [ 10, 20, 30, 40, 50 ]

console.log("\n=== O(1) slicing (shares storage) ===");

const mid = bytes.slice(1, 4);
log("slice(1, 4) buf()", Array.from(mid.buf())); // [ 20, 30, 40 ]
log("slice(2) to end", Array.from(bytes.slice(2).buf())); // [ 30, 40, 50 ]

console.log("\n=== Splitting (O(1)) ===");

const head = bytes.splitTo(2);
const tail = bytes.splitOff(3);
log("after splitTo(2): head", Array.from(head.buf())); // [ 10, 20 ]
log("after splitOff(3): tail", Array.from(tail.buf())); // [ 40, 50 ]
log("original now [2,3)", Array.from(bytes.buf())); // [ 30, 40 ]

console.log("\n=== Truncation & clearing ===");

bytes.truncate(1);
log("truncate(1) len", bytes.len()); // 1
bytes.clear();
log("clear() isEmpty", bytes.isEmpty()); // true

console.log("\n=== Copying out ===");

const fresh = Bytes.fromBuffer(globalThis.Buffer.from([1, 2, 3, 4]));
const copy1 = fresh.asSlice();
const copy2 = fresh.buf();
copy1[0] = 255;
log("mutating asSlice() copy does not affect buf()", Array.from(fresh.buf())); // [ 1, 2, 3, 4 ]
log("buf() returns equivalent copy", Array.from(copy2)); // [ 1, 2, 3, 4 ]

log("tryIntoMut (often null)", fresh.tryIntoMut());

console.log("\nAll Bytes examples executed successfully.");
