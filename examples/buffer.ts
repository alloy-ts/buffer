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

// 3. Buffer manipulation
const writeBuf = Buffer.alloc(20);
const bytesWritten = writeBuf.write("alloy-ts", 0, "utf8");
console.log(`Written ${bytesWritten} bytes:`, writeBuf.toString("utf8", 0, bytesWritten));

// 4. Endian numeric operations
const numBuf = Buffer.alloc(8);
numBuf.writeUInt16BE(0x1234, 0);
numBuf.writeUInt32LE(0x87654321, 2);
console.log("UInt16BE at 0:", numBuf.readUInt16BE(0).toString(16));
console.log("UInt32LE at 2:", numBuf.readUInt32LE(2).toString(16));

// 5. Byte swapping
const swapBuf = Buffer.from([0x01, 0x02, 0x03, 0x04]);
swapBuf.swap16();
console.log("swap16 result:", Array.from(swapBuf));
