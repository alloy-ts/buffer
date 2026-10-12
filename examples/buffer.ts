import { Buffer } from "../src/main.ts";

console.log("--- Buffer Allocation ---");
const bufAlloc = Buffer.alloc(10);
console.log("Alloc 10 zero-filled bytes:", bufAlloc);

const bufFilled = Buffer.alloc(10, 1);
console.log("Alloc 10 bytes filled with 1:", bufFilled);

const bufUnsafe = Buffer.allocUnsafe(10);
console.log("AllocUnsafe 10 bytes:", bufUnsafe);

console.log("\n--- Buffer.from Variants ---");
const bufArray = Buffer.from([1, 2, 3]);
console.log("From array [1, 2, 3]:", bufArray);

const bufTruncated = Buffer.from([257, 257.5, -255, "1"]);
console.log("From truncated array [257, 257.5, -255, '1']:", bufTruncated);

const bufStringUtf8 = Buffer.from("tést");
console.log("From UTF-8 string 'tést':", bufStringUtf8);

const bufStringLatin1 = Buffer.from("tést", "latin1");
console.log("From Latin1 string 'tést':", bufStringLatin1);

const bufHex = Buffer.from("74c3a97374", "hex");
console.log("From hex string '74c3a97374':", bufHex.toString("utf8"));

console.log("\n--- Buffer Operations ---");
const bufWrite = Buffer.alloc(20);
const written = bufWrite.write("hello world", 0);
console.log(`Written ${written} bytes:`, bufWrite.toString("utf8", 0, written));

const bufSearch = Buffer.from("this is a buffer");
console.log("indexOf('buffer'):", bufSearch.indexOf("buffer"));
console.log("includes('a buffer'):", bufSearch.includes("a buffer"));

const numBuf = Buffer.alloc(8);
numBuf.writeInt32BE(0x01020304, 0);
console.log("writeInt32BE / readInt32BE:", numBuf.readInt32BE(0).toString(16));

numBuf.writeDoubleBE(123.456, 0);
console.log("writeDoubleBE / readDoubleBE:", numBuf.readDoubleBE(0));
