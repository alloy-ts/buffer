/**
 * examples/buffer.ts
 *
 * Demonstrates the `node:buffer`-compatible `Buffer` API shipped by
 * `@alloy-ts/buffer`. Run it with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/buffer.ts
 */
import { Buffer, constants, INSPECT_MAX_BYTES, kMaxLength, kStringMaxLength, SlowBuffer } from "../dist/index.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== Buffer construction ===");

// Zero-filled allocation
const zero = Buffer.alloc(5);
log("Buffer.alloc(5)", zero); // <Buffer 00 00 00 00 00>

// Allocation filled with a byte value
log("Buffer.alloc(5, 1)", Buffer.alloc(5, 1)); // <Buffer 01 01 01 01 01>

// Uninitialized allocation (contents are arbitrary)
log("Buffer.allocUnsafe(3).length", Buffer.allocUnsafe(3).length); // 3
log("Buffer.allocUnsafeSlow(3).length", Buffer.allocUnsafeSlow(3).length); // 3

// From a byte array (values truncated to & 0xff)
log("Buffer.from([1, 2, 3, 256])", Buffer.from([1, 2, 3, 256])); // 01 02 03 00

// From a string with an explicit encoding
const utf8 = Buffer.from("tést");
log('Buffer.from("tést") hex', utf8.toString("hex")); // 74c3a97374
log('Buffer.from("tést", "latin1") hex', Buffer.from("tést", "latin1").toString("hex")); // 74e97374

// Sharing memory with an ArrayBuffer
const ab = new ArrayBuffer(4);
const view = new Uint8Array(ab);
view[0] = 0x41;
const fromAb = Buffer.from(ab);
log("Buffer.from(arrayBuffer) shares memory", fromAb); // 41 00 00 00

// From another Buffer (copies the bytes)
const copied = Buffer.from(utf8);
log("Buffer.from(buffer) copy equals", copied.equals(utf8)); // true

// Buffer.of(...) convenience constructor
log("Buffer.of(10, 20, 30)", Buffer.of(10, 20, 30)); // 0a 14 1e

console.log("\n=== Static helpers ===");

log('Buffer.byteLength("tést")', Buffer.byteLength("tést")); // 4 (UTF-8)
log('Buffer.byteLength("tést", "utf16le")', Buffer.byteLength("tést", "utf16le")); // 8
log('Buffer.isBuffer(utf8)', Buffer.isBuffer(utf8)); // true
log('Buffer.isBuffer(new Uint8Array(1))', Buffer.isBuffer(new Uint8Array(1))); // false
log('Buffer.isEncoding("utf8")', Buffer.isEncoding("utf8")); // true
log('Buffer.isEncoding("")', Buffer.isEncoding("")); // false

// Concatenation
const concatenated = Buffer.concat([Buffer.from("foo"), Buffer.from("bar")], 6);
log("Buffer.concat", concatenated.toString()); // "foobar"

// Comparison returns -1 | 0 | 1
log("Buffer.compare order", Buffer.compare(Buffer.from("a"), Buffer.from("b"))); // -1

// Copy bytes out of a typed-array view
const u16 = new Uint16Array([0x1234, 0x5678]);
log("Buffer.copyBytesFrom(u16)", Buffer.copyBytesFrom(u16)); // 34 12 78 56

console.log("\n=== Encoding / decoding ===");

const encoded = Buffer.from("Hello, 世界");
log("toString() (utf8)", encoded.toString()); // Hello, 世界
log("toString(hex)", encoded.toString("hex"));
log("toString(base64)", encoded.toString("base64"));

// UTF-16LE: every UTF-16 code unit is 2 bytes, so the buffer length must be even.
const text = "Hello";
const asUtf16 = Buffer.from(text, "utf16le");
log("utf16le length (2x char count)", asUtf16.length); // 10
log("utf16le roundtrip", asUtf16.toString("utf16le")); // "Hello"

// write() returns the number of bytes written
const w = Buffer.alloc(10);
const written = w.write("abc", 0, "utf8");
log("buf.write returns bytes written", written); // 3
log("buf.write content", w.subarray(0, written).toString()); // abc

// Partial range decoding
log("toString(utf8, 0, 5)", encoded.toString("utf8", 0, 5)); // "Hello"

console.log("\n=== Searching & comparison ===");

const haystack = Buffer.from("the quick brown fox");
log('indexOf("brown")', haystack.indexOf("brown")); // 10
log('lastIndexOf("o")', haystack.lastIndexOf("o")); // 17
log('includes("fox")', haystack.includes("fox")); // true
log('includes(0x71 /* q */)', haystack.includes(0x71)); // true

const a = Buffer.from("abc");
const b = Buffer.from("abc");
log("equals (true)", a.equals(b)); // true
log("compare (0)", a.compare(b)); // 0
// Ranged compare: compare a[1..3] with b[0..2]
log("ranged compare", a.compare(b, 0, 2, 1, 3)); // 0

console.log("\n=== Mutation ===");

const fillBuf = Buffer.alloc(6);
fillBuf.fill(0x2a); // '*'
log("fill(0x2a)", fillBuf.toString()); // ******

const target = Buffer.alloc(4);
const source = Buffer.from("data");
const copiedBytes = source.copy(target, 1, 1, 3);
log("copy(dst, 1, 1, 3)", target.toString()); // \x00 at 0, then "at"
log("copy returned count", copiedBytes); // 2

const rev = Buffer.from("abc");
rev.reverse();
log("reverse", rev.toString()); // cba

// Endianness-aware swaps (length must be a multiple of the width)
const sw = Buffer.from([0x01, 0x02, 0x03, 0x04]);
sw.swap16();
log("swap16", sw); // 02 01 04 03

console.log("\n=== Binary I/O (read / write numbers) ===");

const num = Buffer.alloc(8);

// 16-bit little-endian
const off1 = num.writeInt16LE(-12345, 0);
log("writeInt16LE(-12345) next offset", off1); // 2
log("readInt16LE", num.readInt16LE(0)); // -12345

// 32-bit big-endian
num.writeUInt32BE(0xdeadbeef, 2);
log("readUInt32BE", num.readUInt32BE(2).toString(16)); // "deadbeef"

// 64-bit BigInt
const bigOff = num.writeBigUInt64LE(9007199254740993n, 0);
log("writeBigUInt64LE next offset", bigOff); // 8
log("readBigUInt64LE", num.readBigUInt64LE(0)); // 9007199254740993n

// IEEE-754 float / double
num.writeFloatLE(3.14, 0);
log("readFloatLE", num.readFloatLE(0)); // ~3.14
num.writeDoubleBE(2.718281828, 0);
log("readDoubleBE", num.readDoubleBE(0)); // ~2.718281828

// Variable-length (1..6 byte) integers
const v = Buffer.alloc(6);
v.writeUIntLE(0x123456, 0, 3);
log("writeUIntLE(0x123456, 3 bytes) read", v.readUIntLE(0, 3).toString(16)); // 123456

console.log("\n=== Constants & deprecated APIs ===");

log("constants.MAX_LENGTH", constants.MAX_LENGTH);
log("constants.MAX_STRING_LENGTH", constants.MAX_STRING_LENGTH);
log("INSPECT_MAX_BYTES", INSPECT_MAX_BYTES);
log("kMaxLength", kMaxLength);
log("kStringMaxLength", kStringMaxLength);
log("SlowBuffer(2) is Buffer", Buffer.isBuffer(SlowBuffer(2))); // true

console.log("\nAll Buffer examples executed successfully.");
