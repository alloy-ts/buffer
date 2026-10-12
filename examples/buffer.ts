import { Buffer } from "../src/main.ts";

console.log("=== Buffer Examples ===");

// 1. Buffer allocations
const bufZero = Buffer.alloc(10);
console.log("Alloc zero-filled (10 bytes):", bufZero);

const bufFilled = Buffer.alloc(10, 0xff);
console.log("Alloc filled with 0xff:", bufFilled);

const bufUnsafe = Buffer.allocUnsafe(10);
console.log("Alloc unsafe (10 bytes):", bufUnsafe);

// 2. Buffer.from overloads
const bufArray = Buffer.from([257, 257.5, -255, "1"]);
console.log("From array with byte truncation:", Array.from(bufArray));

const bufUtf8 = Buffer.from("Hello, World! 🌍", "utf8");
console.log("From UTF-8 string hex:", bufUtf8.toString("hex"));

const bufBase64 = Buffer.from("aGVsbG8gd29ybGQ=", "base64");
console.log("From Base64 string:", bufBase64.toString("utf8"));

// 3. Subarrays and Memory Sharing
const bufSlice = bufUtf8.subarray(0, 5);
console.log("Subarray (0..5):", bufSlice.toString("utf8"));

// 4. Numeric Read and Write
const numBuf = Buffer.alloc(16);
numBuf.writeInt16LE(0x1234, 0);
numBuf.writeUInt32BE(0xdeadbeef, 2);
numBuf.writeBigInt64LE(0x0102030405060708n, 6);

console.log("Read Int16LE:", numBuf.readInt16LE(0).toString(16));
console.log("Read UInt32BE:", numBuf.readUInt32BE(2).toString(16));
console.log("Read BigInt64LE:", numBuf.readBigInt64LE(6).toString(16));

// 5. Comparison & Concatenation
const bufA = Buffer.from("ABC");
const bufB = Buffer.from("BCD");
console.log("bufA.compare(bufB):", bufA.compare(bufB));

const bufConcat = Buffer.concat([bufA, bufB]);
console.log("Buffer.concat([bufA, bufB]):", bufConcat.toString());
