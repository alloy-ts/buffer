import { Buffer } from "../src/main.ts";

// 1. Creating Buffers
console.log("--- Creating Buffers ---");

// Creates a zero-filled Buffer of length 10
const buf1 = Buffer.alloc(10);
console.log("Buffer.alloc(10):", buf1);

// Creates a Buffer of length 10 filled with value 1
const buf2 = Buffer.alloc(10, 1);
console.log("Buffer.alloc(10, 1):", buf2);

// Creates an uninitialized Buffer of length 10
const buf3 = Buffer.allocUnsafe(10);
console.log("Buffer.allocUnsafe(10) length:", buf3.length);

// Creates a Buffer from an array of numbers
const buf4 = Buffer.from([1, 2, 3]);
console.log("Buffer.from([1, 2, 3]):", buf4);

// Truncated byte values (values & 255)
const buf5 = Buffer.from([257, 257.5, -255, "1"]);
console.log("Buffer.from([257, 257.5, -255, '1']):", buf5);

// Creates a Buffer from string (UTF-8 default)
const buf6 = Buffer.from("tést");
console.log("Buffer.from('tést'):", buf6);

// Creates a Buffer from string with Latin-1 encoding
const buf7 = Buffer.from("tést", "latin1");
console.log("Buffer.from('tést', 'latin1'):", buf7);

// 2. Buffer String Conversions and Encodings
console.log("\n--- Buffer Encodings ---");
console.log("buf6 as UTF-8:", buf6.toString("utf8"));
console.log("buf7 as Latin-1:", buf7.toString("latin1"));

const hexBuf = Buffer.from("74c3a97374", "hex");
console.log("Hex decoded to UTF-8 string:", hexBuf.toString("utf8"));

const b64Buf = Buffer.from("dOlzdA==", "base64");
console.log("Base64 decoded to Latin-1 string:", b64Buf.toString("latin1"));

// 3. Slicing, Subarray, and In-place Modification
console.log("\n--- Memory Sharing & Subarray ---");
const alphabet = Buffer.allocUnsafe(26);
for (let i = 0; i < 26; i++) {
  alphabet[i] = i + 97; // 'a' through 'z'
}

const sub = alphabet.subarray(0, 3);
console.log("Subarray (0..3) as ASCII:", sub.toString("ascii")); // abc

alphabet[0] = 33; // ASCII '!'
console.log("After modifying original buffer, sub as ASCII:", sub.toString("ascii")); // !bc

// 4. Numeric Read & Write
console.log("\n--- Numeric Reads & Writes ---");
const numBuf = Buffer.alloc(16);
numBuf.writeInt8(-42, 0);
numBuf.writeUInt16LE(0xdead, 2);
numBuf.writeUInt32BE(0xfeedface, 4);
numBuf.writeBigInt64LE(0x0102030405060708n, 8);

console.log("readInt8(0):", numBuf.readInt8(0)); // -42
console.log("readUInt16LE(2) hex:", numBuf.readUInt16LE(2).toString(16)); // dead
console.log("readUInt32BE(4) hex:", numBuf.readUInt32BE(4).toString(16)); // feedface
console.log("readBigInt64LE(8) BigInt:", numBuf.readBigInt64LE(8)); // 72623859790382856n

// 5. JSON Serialization
console.log("\n--- JSON Serialization ---");
const jsonBuf = Buffer.from([1, 2, 3, 4, 5]);
const jsonString = JSON.stringify(jsonBuf);
console.log("JSON.stringify(buf):", jsonString);
const parsed = JSON.parse(jsonString, (key, value) => {
  return value && value.type === "Buffer" ? Buffer.from(value.data) : value;
});
console.log("Reconstructed Buffer from JSON:", parsed);
