import { Buffer } from "../src/main.ts";

console.log("=== Buffer Examples ===");

// Creating Buffers
const buf1 = Buffer.alloc(10);
console.log("buf1 alloc(10):", buf1);

const buf2 = Buffer.alloc(10, 1);
console.log("buf2 alloc(10, 1):", buf2);

const buf3 = Buffer.from([1, 2, 3, 4]);
console.log("buf3 from array:", buf3);

const buf4 = Buffer.from("hello world", "utf8");
console.log("buf4 from string (hex):", buf4.toString("hex"));
console.log("buf4 from string (base64):", buf4.toString("base64"));

// Integer & Float reads/writes
const numBuf = Buffer.alloc(8);
numBuf.writeInt32BE(0x12345678, 0);
numBuf.writeFloatLE(3.14159, 4);
console.log("numBuf readInt32BE:", numBuf.readInt32BE(0).toString(16));
console.log("numBuf readFloatLE:", numBuf.readFloatLE(4));

// Swaps
const swapBuf = Buffer.from([0x01, 0x02, 0x03, 0x04]);
swapBuf.swap16();
console.log("swap16 result:", swapBuf);
