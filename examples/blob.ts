/**
 * examples/blob.ts
 *
 * Demonstrates the `Blob` and `File` classes (plus the related `transcode`,
 * `atob`/`btoa`, `isAscii`/`isUtf8`/`isLatin1`, and the native `Bytes`
 * binding) shipped by `@alloy-ts/buffer`.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/blob.ts
 *
 * Note: unlike the Web `Blob`/`File` (whose `text()`/`bytes()`/`arrayBuffer()`
 * return Promises), this native binding implements them synchronously and
 * returns the value directly.
 */
import {
  Blob,
  File,
  Bytes,
  atob,
  btoa,
  isAscii,
  isUtf8,
  isLatin1,
  transcode,
  Buffer,
} from "../index.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== Blob basics ===");

// A Blob can be built from strings, ArrayBuffers, TypedArrays, DataViews, Blobs.
const blob = new Blob(["hello ", "world"], { type: "text/plain" });
log("blob.size", blob.size); // 11
log("blob.type", blob.type); // "text/plain"

// Synchronous accessors (NOT promises in this binding).
log("blob.text()", blob.text()); // "hello world"
// `blob.bytes()` returns the host `Buffer` (a `Uint8Array` view over the bytes).
const bytesBuf = blob.bytes();
log("blob.bytes() is a TypedArray view", ArrayBuffer.isView(bytesBuf)); // true
log("blob.bytes() contents", bytesBuf.toString()); // "hello world"
const ab = blob.arrayBuffer();
log("blob.arrayBuffer() is ArrayBuffer", ab instanceof ArrayBuffer); // true
log("blob.arrayBuffer() contents", Buffer.from(ab).toString()); // "hello world"

console.log("\n=== Blob slicing ===");

const sliced = blob.slice(0, 5, "text/plain");
log("blob.slice(0, 5).text()", sliced.text()); // "hello"
log("blob.slice type", sliced.type); // "text/plain"

// Build a Blob from mixed binary parts
const mixed = new Blob([
  new Uint8Array([1, 2, 3]),
  new Uint8Array([4, 5, 6]).buffer,
  new DataView(new Uint8Array([7, 8, 9]).buffer),
]);
log("mixed Blob size", mixed.size); // 9
log("mixed Blob bytes", Array.from(mixed.bytes())); // [1..9]

console.log("\n=== File ===");

// File extends Blob with `name` and `lastModified`.
const file = new File(["config data"], "config.txt", {
  type: "text/plain",
  lastModified: 1_700_000_000_000,
});
log("file.name", file.name); // "config.txt"
log("file.lastModified", file.lastModified); // 1700000000000
log("file.type", file.type); // "text/plain"
log("file.size", file.size); // 11
log("file.text()", file.text()); // "config data"

// A File sliced still carries its metadata.
const fileSlice = file.slice(0, 6);
log("file.slice name", fileSlice.name); // "config.txt"
log("file.slice text", fileSlice.text()); // "config"

console.log("\n=== Base64 (atob / btoa) ===");

const b64 = btoa("hello"); // Latin-1 -> Base64
log("btoa('hello')", b64); // "aGVsbG8="
log("atob(b64)", atob(b64)); // "hello"

console.log("\n=== Encoding validation ===");

log("isAscii(Buffer.from('abc'))", isAscii(Buffer.from("abc"))); // true
log("isAscii(Buffer.from('é'))", isAscii(Buffer.from("é"))); // false
log("isUtf8(Buffer.from('✓'))", isUtf8(Buffer.from("✓"))); // true
log("isUtf8(Buffer.from([0xff]))", isUtf8(Buffer.from([0xff]))); // false
log("isLatin1('café')", isLatin1("café")); // true
log("isLatin1('🚀')", isLatin1("🚀")); // false (code point > 0xFF)

console.log("\n=== transcode ===");

// Re-encode bytes from one encoding to another. UTF-16LE -> UTF-8 is lossless.
const le = Buffer.from("café", "utf16le"); // 10 bytes
const asUtf8 = transcode(le, "utf16le", "utf8");
log("transcode(utf16le -> utf8)", asUtf8.toString("utf8")); // "café"
log("transcode result length", asUtf8.length); // 5

console.log("\n=== Native Bytes binding (extra) ===");

// `Bytes` is a value-add: O(1) reference-counted slicing of Rust `bytes::Bytes`.
const bytes = Bytes.fromBuffer(Buffer.from([10, 20, 30, 40, 50]));
log("bytes.len()", bytes.len()); // 5
log("bytes.isEmpty()", bytes.isEmpty()); // false

const slice = bytes.slice(1, 4);
log("bytes.slice(1, 4) -> buf()", Array.from(slice.buf())); // [20, 30, 40]

const tail = bytes.splitOff(3);
log("bytes.splitOff(3) tail", Array.from(tail.buf())); // [40, 50]
log("bytes.len() after splitOff", bytes.len()); // 3

const head = bytes.splitTo(2);
log("bytes.splitTo(2) head", Array.from(head.buf())); // [10, 20]

bytes.truncate(1);
log("bytes.truncate(1) len", bytes.len()); // 1

bytes.clear();
log("bytes.clear() isEmpty", bytes.isEmpty()); // true

console.log("\nAll Blob/File examples executed successfully.");
