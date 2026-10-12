import { Buffer } from "../src/main.ts";

console.log("=== Buffer API Demo ===");

// 1. Allocation & Initialization
const allocBuf = Buffer.alloc(10, 0xab);
console.log("Buffer.alloc(10, 0xab):", allocBuf.toHex());

const allocUnsafeBuf = Buffer.allocUnsafe(8);
allocUnsafeBuf.fill(0);
console.log("Buffer.allocUnsafe(8):", allocUnsafeBuf);

// 2. From String & Encodings
const utf8Buf = Buffer.from("Hello World!", "utf8");
console.log("Buffer.from string (utf8):", utf8Buf.toString("utf8"));
console.log("Buffer.toHex():", utf8Buf.toHex());

const hexBuf = Buffer.from("48656c6c6f", "hex");
console.log("Buffer.from hex -> ascii:", hexBuf.toString("ascii"));

// 3. TC39 / Modern Buffer Methods: toBase64, toHex, setFromBase64, setFromHex
const sampleBuf = Buffer.from([0xfb, 0xff, 0xbf]);
console.log("toBase64():", sampleBuf.toBase64());
console.log("toBase64({ alphabet: 'base64url' }):", sampleBuf.toBase64({ alphabet: "base64url" }));
console.log("toHex():", sampleBuf.toHex());

const destBuf = Buffer.alloc(5);
const b64Result = destBuf.setFromBase64("aGVsbG8=");
console.log("setFromBase64 result:", b64Result, "content:", destBuf.toString());

const hexDest = Buffer.alloc(3);
const hexResult = hexDest.setFromHex("414243");
console.log("setFromHex result:", hexResult, "content:", hexDest.toString());

// 4. Numeric & BigInt I/O
const numBuf = Buffer.alloc(16);
numBuf.writeInt32BE(0x12345678, 0);
console.log("readInt32BE(0):", "0x" + numBuf.readInt32BE(0).toString(16));

numBuf.writeDoubleLE(3.1415926535, 4);
console.log("readDoubleLE(4):", numBuf.readDoubleLE(4));

numBuf.writeBigInt64BE(0x0102030405060708n, 8);
console.log("readBigInt64BE(8):", "0x" + numBuf.readBigInt64BE(8).toString(16) + "n");

// 5. Comparison, Concatenation, and Slicing
const bufA = Buffer.from("ABC");
const bufB = Buffer.from("ABC");
const bufC = Buffer.from("DEF");

console.log("bufA.equals(bufB):", bufA.equals(bufB));
console.log("bufA.compare(bufC):", bufA.compare(bufC));

const combined = Buffer.concat([bufA, Buffer.from(" "), bufC]);
console.log("Buffer.concat:", combined.toString());

const sub = combined.subarray(0, 3);
console.log("subarray(0, 3):", sub.toString());

// 6. Byte Swapping
const swapBuf = Buffer.from([0x01, 0x02, 0x03, 0x04]);
swapBuf.swap16();
console.log("swap16():", swapBuf);
swapBuf.swap32();
console.log("swap32():", swapBuf);

console.log("Buffer demo completed successfully.");
