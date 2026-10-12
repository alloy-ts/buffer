/**
 * examples/buffer.ts
 *
 * Demonstrates Buffer creation, allocation, string decoding, hex/base64
 * methods, numeric I/O, comparison, and subarray slicing.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/buffer.ts
 */
import { Buffer } from "../src/buffer.js";

console.log("=== Buffer Examples ===");

// 1. Allocation
const bufAlloc = Buffer.alloc(10);
console.log("Buffer.alloc(10):", bufAlloc);

const bufAllocFilled = Buffer.alloc(10, 0xab);
console.log("Buffer.alloc(10, 0xab):", bufAllocFilled);

const bufUnsafe = Buffer.allocUnsafe(10);
console.log("Buffer.allocUnsafe(10) length:", bufUnsafe.length);

// 2. Buffer.from
const bufFromString = Buffer.from("Hello, World!", "utf8");
console.log("Buffer.from string:", bufFromString);
console.log("buf.toString():", bufFromString.toString("utf8"));

const bufFromArray = Buffer.from([72, 101, 108, 108, 111]);
console.log("Buffer.from array:", bufFromArray.toString());

// 3. Modern Base64 and Hex methods (toBase64, toHex, setFromBase64, setFromHex)
const hexStr = bufFromString.toHex();
console.log("toHex():", hexStr);

const b64Str = bufFromString.toBase64({ omitPadding: true });
console.log("toBase64(omitPadding):", b64Str);

const targetBuf = Buffer.alloc(20);
const resHex = targetBuf.setFromHex("68656c6c6f");
console.log(
  "setFromHex read/written:",
  resHex,
  "content:",
  targetBuf.subarray(0, resHex.written).toString(),
);

const resB64 = targetBuf.setFromBase64("d29ybGQ=", resHex.written);
console.log(
  "setFromBase64 read/written:",
  resB64,
  "content:",
  targetBuf.subarray(0, resHex.written + resB64.written).toString(),
);

// 4. Numeric I/O
const numBuf = Buffer.alloc(16);
numBuf.writeInt32BE(0x12345678, 0);
console.log("readInt32BE:", numBuf.readInt32BE(0).toString(16));

numBuf.writeDoubleLE(3.1415926535, 4);
console.log("readDoubleLE:", numBuf.readDoubleLE(4));

numBuf.writeBigInt64BE(0x0102030405060708n, 8);
console.log("readBigInt64BE:", numBuf.readBigInt64BE(8).toString(16));

// 5. Comparison & Subarray
const buf1 = Buffer.from("ABC");
const buf2 = Buffer.from("ABC");
const buf3 = Buffer.from("ABD");

console.log("buf1.equals(buf2):", buf1.equals(buf2));
console.log("buf1.compare(buf3):", buf1.compare(buf3));

const sub = bufFromString.subarray(0, 5);
console.log("subarray(0, 5):", sub.toString());
