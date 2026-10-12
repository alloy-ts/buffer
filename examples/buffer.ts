import { Buffer, constants } from "../src/main.ts";

console.log("=== Buffer Allocation & Initialization ===");
const buf1 = Buffer.alloc(10);
console.log("Buffer.alloc(10):", buf1.toString("hex"));

const buf2 = Buffer.alloc(10, 0xab);
console.log("Buffer.alloc(10, 0xab):", buf2.toString("hex"));

const buf3 = Buffer.from([1, 2, 3, 4, 5]);
console.log("Buffer.from([1,2,3,4,5]):", buf3.toString("hex"));

const buf4 = Buffer.from("hello world", "utf8");
console.log("Buffer.from('hello world'):", buf4.toString("utf8"));

console.log("\n=== Buffer Iteration & Array Methods ===");
console.log("at(0):", buf3.at(0));
console.log("at(-1):", buf3.at(-1));

const mapped = buf3.map((v: number) => v * 10);
console.log("map(*10):", Array.from(mapped));

const filtered = buf3.filter((v: number) => v > 2);
console.log("filter(>2):", Array.from(filtered));

console.log("reduce(sum):", buf3.reduce((acc: number, v: number) => acc + v, 0));
console.log("join('-'):", buf3.join("-"));

console.log("\n=== Buffer Reads & Writes ===");
const numBuf = Buffer.alloc(16);
numBuf.writeUInt16BE(0x1234, 0);
console.log("readUInt16BE(0):", "0x" + numBuf.readUInt16BE(0).toString(16));

numBuf.writeUInt32LE(0x87654321, 2);
console.log("readUInt32LE(2):", "0x" + numBuf.readUInt32LE(2).toString(16));

numBuf.writeBigUInt64BE(0x1234567890abcdefn, 6);
console.log("readBigUInt64BE(6):", "0x" + numBuf.readBigUInt64BE(6).toString(16));

console.log("\n=== Encoding Transformations ===");
const hexBuf = Buffer.from("hello", "utf8");
console.log("toHex():", hexBuf.toHex());
console.log("toBase64():", hexBuf.toBase64());

console.log("\n=== Buffer Constants ===");
console.log("MAX_LENGTH:", constants.MAX_LENGTH);
