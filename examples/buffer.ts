import { Buffer, constants } from "../src/main.ts";

console.log("=== Buffer Allocation ===");
const allocBuf = Buffer.alloc(10, "a");
console.log("Allocated Buffer:", allocBuf.toString());

const bufFromString = Buffer.from("Hello, World!", "utf8");
console.log("Buffer from string (utf8):", bufFromString.toString());
console.log("Buffer in hex:", bufFromString.toString("hex"));
console.log("Buffer in base64:", bufFromString.toString("base64"));

console.log("\n=== Subarray and Slice ===");
const subBuf = bufFromString.subarray(0, 5);
console.log("Subarray (0, 5):", subBuf.toString());
// Mutating original reflects in subarray
bufFromString[0] = 104; // 'h'
console.log("Subarray after mutating original:", subBuf.toString());

console.log("\n=== Search Methods ===");
const searchBuf = Buffer.from("this is a test buffer");
console.log("indexOf 'test':", searchBuf.indexOf("test"));
console.log("includes 'buffer':", searchBuf.includes("buffer"));

console.log("\n=== Numeric Reads and Writes ===");
const numBuf = Buffer.alloc(8);
numBuf.writeInt32BE(0x12345678, 0);
console.log("Read Int32BE at 0:", numBuf.readInt32BE(0).toString(16));

numBuf.writeDoubleLE(3.14159, 0);
console.log("Read DoubleLE at 0:", numBuf.readDoubleLE(0));

console.log("\n=== Constants ===");
console.log("MAX_LENGTH:", constants.MAX_LENGTH);
console.log("MAX_STRING_LENGTH:", constants.MAX_STRING_LENGTH);
