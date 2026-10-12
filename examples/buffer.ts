import { Buffer } from "../src/main.ts";

console.log("=== Buffer Usage Example ===");

// Allocation
const bufAlloc = Buffer.alloc(10, 0xff);
console.log("Buffer.alloc(10, 0xff):", bufAlloc);

// Creation from string
const bufString = Buffer.from("Hello, Buffer!", "utf8");
console.log("Buffer.from string (hex):", bufString.toHex());
console.log("Buffer.from string (base64):", bufString.toBase64());

// Integer read and write
const numBuf = Buffer.alloc(8);
numBuf.writeInt32BE(0x12345678, 0);
numBuf.writeFloatLE(123.45, 4);

console.log("readInt32BE(0):", numBuf.readInt32BE(0).toString(16));
console.log("readFloatLE(4):", numBuf.readFloatLE(4));

// Slicing & Memory Sharing
const parent = Buffer.from("abcdef");
const child = parent.subarray(0, 3);
console.log("Child before parent edit:", child.toString());
parent[0] = 33; // '!'
console.log("Child after parent edit:", child.toString());

// Byte Swapping
const swapBuf = Buffer.from([0x01, 0x02, 0x03, 0x04]);
swapBuf.swap16();
console.log("swap16():", swapBuf);
