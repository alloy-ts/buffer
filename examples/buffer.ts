import { Buffer } from "../src/main.ts";

console.log("=== Alloy Buffer Demo ===");

// 1. Allocation and Creation
const buf1 = Buffer.alloc(10);
console.log("buf1 (alloc 10):", buf1);

const buf2 = Buffer.alloc(5, 0xa5);
console.log("buf2 (alloc 5 filled 0xa5):", buf2);

const buf3 = Buffer.from("Hello World!", "utf8");
console.log("buf3 from string:", buf3.toString());
console.log("buf3 hex:", buf3.toHex());
console.log("buf3 base64:", buf3.toBase64());

// 2. Numeric & BigInt Read/Write
const numBuf = Buffer.alloc(16);
numBuf.writeInt16BE(0x1234, 0);
numBuf.writeFloatLE(123.456, 2);
numBuf.writeBigInt64BE(0x0102030405060708n, 6);

console.log("\n--- Numeric I/O ---");
console.log("readInt16BE(0):", "0x" + numBuf.readInt16BE(0).toString(16));
console.log("readFloatLE(2):", numBuf.readFloatLE(2));
console.log("readBigInt64BE(6):", "0x" + numBuf.readBigInt64BE(6).toString(16) + "n");

// 3. Search and Manipulation
const textBuf = Buffer.from("this is a buffer demonstration");
console.log("\n--- Search & Slice ---");
console.log("indexOf('buffer'):", textBuf.indexOf("buffer"));
console.log("includes('demo'):", textBuf.includes("demo"));

const sliceBuf = textBuf.subarray(10, 18);
console.log("subarray(10, 18):", sliceBuf.toString());

// 4. Byte Swapping
const swapBuf = Buffer.from([0x01, 0x02, 0x03, 0x04]);
console.log("\n--- Byte Swapping ---");
console.log("original:", swapBuf);
swapBuf.swap16();
console.log("after swap16:", swapBuf);
swapBuf.swap32();
console.log("after swap32:", swapBuf);

// 5. Hex & Base64 decoding directly into buffer
const setBuf = Buffer.alloc(10);
setBuf.setFromHex("4142434445"); // "ABCDE"
console.log("\n--- setFromHex ---");
console.log("decoded hex into setBuf:", setBuf.subarray(0, 5).toString());
