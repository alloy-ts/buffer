import { Buffer } from "../src/main.ts";

console.log("=== Node.js Buffer API Demonstration ===");

// 1. Static Methods & Allocation
console.log("\n--- 1. Static Methods & Creation ---");

// Buffer.alloc & Buffer.allocUnsafe
const bufAlloc = Buffer.alloc(8, 0x41);
console.log("Buffer.alloc(8, 0x41):", bufAlloc, "->", bufAlloc.toString());

const bufUnsafe = Buffer.allocUnsafe(8);
bufUnsafe.fill(0);
console.log("Buffer.allocUnsafe(8).fill(0):", bufUnsafe);

// Buffer.from
const bufStr = Buffer.from("Hello World!", "utf8");
console.log("Buffer.from string:", bufStr.toString());

const bufHex = Buffer.from("48656c6c6f", "hex");
console.log("Buffer.from hex:", bufHex.toString("utf8"));

const bufArr = Buffer.from([10, 20, 30, 40, 50]);
console.log("Buffer.from array:", bufArr);

// Buffer.concat & Buffer.compare
const combined = Buffer.concat([bufHex, Buffer.from(" World!")]);
console.log("Buffer.concat:", combined.toString());

console.log("Buffer.isBuffer(bufStr):", Buffer.isBuffer(bufStr));
console.log("Buffer.isEncoding('utf8'):", Buffer.isEncoding("utf8"));
console.log("Buffer.byteLength('Hello World!'):", Buffer.byteLength("Hello World!"));

// 2. Properties & Subarray
console.log("\n--- 2. Buffer Properties & Array Views ---");
const buf = Buffer.from("abcdefghijklmnopqrstuvwxyz");
console.log("length:", buf.length);
console.log("byteLength:", buf.byteLength);
console.log("byteOffset:", buf.byteOffset);
console.log("BYTES_PER_ELEMENT:", buf.BYTES_PER_ELEMENT);
console.log("Symbol.toStringTag:", buf[Symbol.toStringTag]);

const sub = buf.subarray(0, 5);
console.log("buf.subarray(0, 5):", sub.toString());
console.log("buf.at(0):", buf.at(0), "->", String.fromCharCode(buf.at(0)!));
console.log("buf.at(-1):", buf.at(-1), "->", String.fromCharCode(buf.at(-1)!));

// 3. String Encoding, Decoding & Conversion
console.log("\n--- 3. String & Format Conversion ---");
const writeBuf = Buffer.alloc(16);
const bytesWritten = writeBuf.write("Node.js", 0, "utf8");
console.log("buf.write 'Node.js':", bytesWritten, "bytes written:", writeBuf.toString("utf8", 0, bytesWritten));

console.log("buf.toJSON():", Buffer.from([1, 2, 3, 4]).toJSON());
console.log("buf.valueOf():", Buffer.from([65, 66]).valueOf());
console.log("buf.toLocaleString():", Buffer.from([1, 2, 3]).toLocaleString());

// Modern Uint8Array Base64/Hex helpers (if available) or Buffer equivalents
console.log("buf.toHex():", typeof buf.toHex === "function" ? buf.toHex() : buf.toString("hex"));
console.log("buf.toBase64():", typeof buf.toBase64 === "function" ? buf.toBase64() : buf.toString("base64"));

// 4. Comparison, Search & Copying
console.log("\n--- 4. Comparison, Search & Copy ---");
const buf1 = Buffer.from("ABC");
const buf2 = Buffer.from("ABC");
const buf3 = Buffer.from("ABD");

console.log("buf1.equals(buf2):", buf1.equals(buf2));
console.log("buf1.compare(buf3):", buf1.compare(buf3));

const src = Buffer.from("Search in this buffer");
console.log("src.indexOf('this'):", src.indexOf("this"));
console.log("src.lastIndexOf('in'):", src.lastIndexOf("in"));
console.log("src.includes('buffer'):", src.includes("buffer"));

const target = Buffer.alloc(10);
src.copy(target, 0, 0, 6);
console.log("src.copy(target, 0, 0, 6):", target.toString());

const fillBuf = Buffer.alloc(8);
fillBuf.fill("Z");
console.log("fillBuf.fill('Z'):", fillBuf.toString());

// 5. Byte Swapping
console.log("\n--- 5. Byte Swapping ---");
const swap16Buf = Buffer.from([0x01, 0x02, 0x03, 0x04]);
swap16Buf.swap16();
console.log("swap16:", swap16Buf);

const swap32Buf = Buffer.from([0x01, 0x02, 0x03, 0x04]);
swap32Buf.swap32();
console.log("swap32:", swap32Buf);

const swap64Buf = Buffer.from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
swap64Buf.swap64();
console.log("swap64:", swap64Buf);

// 6. Numeric & BigInt Read / Write Methods
console.log("\n--- 6. Numeric & BigInt I/O ---");
const numBuf = Buffer.alloc(32);

// 8-bit integers
numBuf.writeInt8(-42, 0);
numBuf.writeUInt8(200, 1);
numBuf.writeUint8(205, 2);
console.log("readInt8:", numBuf.readInt8(0));
console.log("readUInt8:", numBuf.readUInt8(1));
console.log("readUint8:", numBuf.readUint8(2));

// 16-bit integers
numBuf.writeInt16LE(-1234, 3);
numBuf.writeInt16BE(-1234, 5);
numBuf.writeUInt16LE(40000, 7);
numBuf.writeUint16BE(40000, 9);
console.log("readInt16LE:", numBuf.readInt16LE(3));
console.log("readInt16BE:", numBuf.readInt16BE(5));
console.log("readUInt16LE:", numBuf.readUInt16LE(7));
console.log("readUint16BE:", numBuf.readUint16BE(9));

// 32-bit integers
numBuf.writeInt32LE(-1234567, 11);
numBuf.writeInt32BE(-1234567, 15);
numBuf.writeUInt32LE(3000000000, 19);
numBuf.writeUint32BE(3000000000, 23);
console.log("readInt32LE:", numBuf.readInt32LE(11));
console.log("readInt32BE:", numBuf.readInt32BE(15));
console.log("readUInt32LE:", numBuf.readUInt32LE(19));
console.log("readUint32BE:", numBuf.readUint32BE(23));

// Floats
const floatBuf = Buffer.alloc(8);
floatBuf.writeFloatLE(3.14159, 0);
floatBuf.writeFloatBE(3.14159, 4);
console.log("readFloatLE:", Math.fround(floatBuf.readFloatLE(0)));
console.log("readFloatBE:", Math.fround(floatBuf.readFloatBE(4)));

// Doubles
const doubleBuf = Buffer.alloc(16);
doubleBuf.writeDoubleLE(3.141592653589793, 0);
doubleBuf.writeDoubleBE(2.718281828459045, 8);
console.log("readDoubleLE:", doubleBuf.readDoubleLE(0));
console.log("readDoubleBE:", doubleBuf.readDoubleBE(8));

// BigInt 64-bit integers
const bigBuf = Buffer.alloc(32);
bigBuf.writeBigInt64LE(-9000000000000000000n, 0);
bigBuf.writeBigInt64BE(-9000000000000000000n, 8);
bigBuf.writeBigUInt64LE(18000000000000000000n, 16);
bigBuf.writeBigUint64BE(18000000000000000000n, 24);
console.log("readBigInt64LE:", bigBuf.readBigInt64LE(0));
console.log("readBigInt64BE:", bigBuf.readBigInt64BE(8));
console.log("readBigUInt64LE:", bigBuf.readBigUInt64LE(16));
console.log("readBigUint64BE:", bigBuf.readBigUint64BE(24));

// Variable length integers (1-6 bytes)
const varBuf = Buffer.alloc(12);
varBuf.writeIntLE(0x1234567890ab, 0, 6);
varBuf.writeUIntBE(0x1234567890ab, 6, 6);
console.log("readIntLE (6 bytes):", varBuf.readIntLE(0, 6).toString(16));
console.log("readUIntBE (6 bytes):", varBuf.readUIntBE(6, 6).toString(16));

// 7. TypedArray & Iteration Methods
console.log("\n--- 7. Array & Iteration Methods ---");
const nums = Buffer.from([10, 20, 30, 40, 50]);

nums.forEach((val, idx) => console.log(`forEach [${idx}]: ${val}`));

console.log("map:", nums.map((x) => x * 2));
console.log("filter:", nums.filter((x) => x > 25));
console.log("reduce:", nums.reduce((acc, curr) => acc + curr, 0));
console.log("reduceRight:", nums.reduceRight((acc, curr) => acc + curr, 0));
console.log("find:", nums.find((x) => x > 25));
console.log("findIndex:", nums.findIndex((x) => x > 25));
console.log("findLast:", typeof nums.findLast === "function" ? nums.findLast((x) => x < 45) : undefined);
console.log("findLastIndex:", typeof nums.findLastIndex === "function" ? nums.findLastIndex((x) => x < 45) : undefined);
console.log("every (>5):", nums.every((x) => x > 5));
console.log("some (>45):", nums.some((x) => x > 45));
console.log("join(', '):", nums.join(", "));

console.log("keys():", Array.from(nums.keys()));
console.log("values():", Array.from(nums.values()));
console.log("entries():", Array.from(nums.entries()));

const unsorted = Buffer.from([5, 2, 8, 1, 9]);
console.log("reverse():", Buffer.from(unsorted).reverse());

if (typeof unsorted.toSorted === "function") {
  console.log("toSorted():", unsorted.toSorted());
}
if (typeof unsorted.toReversed === "function") {
  console.log("toReversed():", unsorted.toReversed());
}
if (typeof unsorted.with === "function") {
  console.log("with(2, 99):", unsorted.with(2, 99));
}

// Iterable demonstration
console.log("Symbol.iterator:");
for (const byte of Buffer.from("Hi")) {
  console.log("  byte:", byte);
}

console.log("\n=== Buffer API Demo Completed Successfully ===");
