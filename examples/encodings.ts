/**
 * examples/encodings.ts
 *
 * Demonstrates every character encoding supported by `@alloy-ts/buffer`
 * (`utf8`, `ascii`, `latin1`/`binary`, `utf16le`/`ucs2`, `base64`, `base64url`,
 * `hex`) plus `Buffer.write` with explicit offset/length/encoding and
 * `Buffer.isEncoding`.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/encodings.ts
 */
import { Buffer } from "../src/main.ts";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== Base64 & Base64URL ===");

const b64 = Buffer.from("hello world").toString("base64");
log("toString(base64)", b64); // "aGVsbG8gd29ybGQ="
log("roundtrip", Buffer.from(b64, "base64").toString()); // "hello world"

const b64url = Buffer.from("hello world").toString("base64url");
log("toString(base64url) (padding stripped)", b64url); // "aGVsbG8gd29ybGQ"
log("base64url decode (padded)", Buffer.from(b64url + "==", "base64url").toString()); // "hello world"

console.log("\n=== Hex ===");

const hex = Buffer.from("ABC").toString("hex");
log("toString(hex)", hex); // "414243"
log("hex roundtrip", Buffer.from(hex, "hex").toString()); // "ABC"

console.log("\n=== Latin1 / binary ===");

// latin1 (a.k.a. binary) maps each byte 1:1 to a U+0000..U+00FF code point.
const latin1 = Buffer.from("café", "latin1");
log("from latin1 bytes", Array.from(latin1)); // [ 99, 97, 102, 233 ]
log("latin1 roundtrip", latin1.toString("latin1")); // "café"

console.log("\n=== UTF-16LE / UCS2 ===");

// Each code unit is 2 bytes, so the source string length must produce an
// even number of bytes.
const u16 = Buffer.from("Hi☺", "utf16le");
log("utf16le length (2x units)", u16.length); // 6
log("utf16le roundtrip", u16.toString("utf16le")); // "Hi☺"
// `ucs2` is an alias of `utf16le`.
log("ucs2 alias matches", Buffer.from("Hi☺", "ucs2").equals(u16)); // true

console.log("\n=== ASCII ===");

log("ascii keeps 7-bit chars", Buffer.from("abc", "ascii").toString("ascii")); // "abc"
log("ascii write substitutes high bytes with ?", Array.from(Buffer.from("é", "ascii"))); // [ 63 ]

let asciiReadThrew = false;
try {
  Buffer.from([0xe9]).toString("ascii");
} catch {
  asciiReadThrew = true;
}
log("ascii read of >0x7F byte throws (strict, vs Node lenient)", asciiReadThrew); // true

console.log("\n=== Buffer.write with offset / length / encoding ===");

const buf = Buffer.alloc(12);
// write(string, offset, encoding)
const n1 = buf.write("ABC", 2, "utf8");
log("write('ABC', 2) -> bytes written", n1); // 3
log("content", buf.subarray(0, 5)); // 00 00 41 42 43

// write(string, offset, length, encoding) — length limits how many BYTES are written
const buf2 = Buffer.alloc(12);
const n2 = buf2.write("ABCDEFG", 0, 3, "utf8");
log("write('ABCDEFG', 0, 3) -> bytes written", n2); // 3 (truncated to 3 bytes)
log("content (only 3 bytes written)", buf2.subarray(0, 3).toString()); // "ABC"

// write at an offset with an explicit encoding
const buf3 = Buffer.alloc(8);
buf3.write("xy", 4, "latin1");
log("write at offset 4 (latin1)", Array.from(buf3.subarray(4))); // [ 120, 121 ]

console.log("\n=== Buffer.isEncoding ===");

for (const enc of ["utf8", "utf16le", "ucs2", "base64", "base64url", "hex", "latin1", "binary", "ascii", "utf-8", "UTF-8", "", "rot13"]) {
  log(`isEncoding(${JSON.stringify(enc)})`, Buffer.isEncoding(enc));
}

console.log("\nAll encoding examples executed successfully.");
