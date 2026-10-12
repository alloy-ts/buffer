import { Buffer } from "../src/main.ts";

console.log("=== Buffer API Demo ===");

// 1. Allocation
const buf1 = Buffer.alloc(10);
console.log("Allocated zero-filled buffer:", buf1);

const buf2 = Buffer.alloc(10, "a");
console.log("Allocated filled buffer:", buf2.toString());

const buf3 = Buffer.allocUnsafe(10);
console.log("Allocated unsafe buffer length:", buf3.length);

// 2. Buffer.from overloads
const fromString = Buffer.from("Hello, World!", "utf8");
console.log("From UTF-8 string:", fromString);
console.log("To hex:", fromString.toHex());
console.log("To base64:", fromString.toBase64());
console.log(
  "To base64url (no padding):",
  fromString.toBase64({ alphabet: "base64url", omitPadding: true }),
);

const fromArray = Buffer.from([72, 101, 108, 108, 111]);
console.log("From byte array:", fromArray.toString("utf8"));

// 3. Numeric & BigInt I/O
const numBuf = Buffer.alloc(16);
numBuf.writeInt16BE(0x1234, 0);
numBuf.writeInt32LE(0x56789abc, 2);
numBuf.writeDoubleBE(3.141592653589793, 6);

console.log("readInt16BE(0):", numBuf.readInt16BE(0).toString(16));
console.log("readInt32LE(2):", numBuf.readInt32LE(2).toString(16));
console.log("readDoubleBE(6):", numBuf.readDoubleBE(6));

const bigBuf = Buffer.alloc(8);
bigBuf.writeBigInt64BE(0x0102030405060708n, 0);
console.log("readBigInt64BE(0):", bigBuf.readBigInt64BE(0).toString(16));

// 4. setFromHex and setFromBase64
const hexBuf = Buffer.alloc(5);
hexBuf.setFromHex("68656c6c6f");
console.log("setFromHex result:", hexBuf.toString());

const b64Buf = Buffer.alloc(5);
b64Buf.setFromBase64("aGVsbG8=");
console.log("setFromBase64 result:", b64Buf.toString());

// 5. Compare, Equals, Copy, Swap
const cmp1 = Buffer.from("ABC");
const cmp2 = Buffer.from("ABC");
const cmp3 = Buffer.from("ABD");

console.log("equals:", cmp1.equals(cmp2)); // true
console.log("compare:", cmp1.compare(cmp3)); // -1

const swapBuf = Buffer.from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
swapBuf.swap32();
console.log("swap32 result:", Array.from(swapBuf));

// 6. Subarray memory sharing
const original = Buffer.from("JavaScript");
const sub = original.subarray(0, 4);
console.log("Subarray:", sub.toString()); // Java
original[0] = 89; // 'Y'
console.log("Subarray after mutating original:", sub.toString()); // Yava
