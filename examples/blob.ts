/**
 * examples/blob.ts
 *
 * Demonstrates Blob creation, MIME types, size, text(), bytes(), slice(), and arrayBuffer().
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/blob.ts
 */
import { Blob } from "../src/buffer.js";

console.log("=== Blob Examples ===");

// 1. Create a Blob
const parts = ["Hello ", new Uint8Array([119, 111, 114, 108, 100]), "!"];
const blob = new Blob(parts, { type: "text/plain" });

console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);

// 2. Read Blob text
console.log("Blob text():", blob.text());

// 3. Read Blob as Buffer
const bytes = blob.bytes();
console.log("Blob bytes():", bytes);

// 4. Slice Blob
const sliced = blob.slice(0, 5, "text/plain");
console.log("Sliced Blob size:", sliced.size);
console.log("Sliced Blob text():", sliced.text());

// 5. Read Blob arrayBuffer
const ab = blob.arrayBuffer();
console.log("Blob arrayBuffer byteLength:", ab.byteLength);
