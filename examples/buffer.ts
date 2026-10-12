import { Buffer } from "../src/main.ts";

console.log("=== Buffer Example ===");

// 1. Buffer.alloc and allocUnsafe
const zeroBuf = Buffer.alloc(10);
console.log("Buffer.alloc(10):", zeroBuf);

const filledBuf = Buffer.alloc(10, 0xff);
console.log("Buffer.alloc(10, 0xff):", filledBuf);

const unsafeBuf = Buffer.allocUnsafe(10);
console.log("Buffer.allocUnsafe(10) length:", unsafeBuf.length);

// 2. Buffer.from
const strBuf = Buffer.from("Hello, Node.js Buffers!", "utf8");
console.log("Buffer.from string (hex):", strBuf.toString("hex"));
console.log("Buffer.from string (utf8):", strBuf.toString("utf8"));

const latin1Buf = Buffer.from("tést", "latin1");
console.log("Buffer.from latin1 bytes:", Array.from(latin1Buf));

const arrBuf = Buffer.from([1, 2, 3, 257]); // 257 truncated to 1
console.log("Buffer.from array [1, 2, 3, 257]:", Array.from(arrBuf));

// 3. Buffer searching & comparison
const searchBuf = Buffer.from("this is a buffer instance in alloy-ts");
console.log("indexOf('buffer'):", searchBuf.indexOf("buffer"));
console.log("lastIndexOf('i'):", searchBuf.lastIndexOf("i"));
console.log("includes('alloy-ts'):", searchBuf.includes("alloy-ts"));

const compBuf1 = Buffer.from("ABC");
const compBuf2 = Buffer.from("BCD");
console.log("equals:", compBuf1.equals(Buffer.from("414243", "hex")));
console.log("compare ('ABC' vs 'BCD'):", compBuf1.compare(compBuf2));

// 4. Memory slicing & copying
const srcBuf = Buffer.from("abcdefghijklmnopqrstuvwxyz");
const subBuf = srcBuf.subarray(0, 5);
console.log("subarray(0, 5):", subBuf.toString("utf8"));

const targetBuf = Buffer.alloc(10).fill("!");
srcBuf.copy(targetBuf, 0, 0, 5);
console.log("copy result:", targetBuf.toString("utf8"));

// 5. Numeric & BigInt I/O
const numBuf = Buffer.alloc(16);
numBuf.writeUInt16BE(0x1234, 0);
numBuf.writeUInt32LE(0x87654321, 2);
numBuf.writeBigInt64BE(0x0102030405060708n, 6);

console.log("UInt16BE at 0:", numBuf.readUInt16BE(0).toString(16));
console.log("UInt32LE at 2:", numBuf.readUInt32LE(2).toString(16));
console.log("BigInt64BE at 6:", numBuf.readBigInt64BE(6).toString(16));

// 6. Byte swapping & reversal
const swapBuf = Buffer.from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
swapBuf.swap16();
console.log("swap16 result:", Array.from(swapBuf));

const swapBuf64 = Buffer.from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
swapBuf64.swap64();
console.log("swap64 result:", Array.from(swapBuf64));

const revBuf = Buffer.from([1, 2, 3, 4]);
revBuf.reverse();
console.log("reverse result:", Array.from(revBuf));
