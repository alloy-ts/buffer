import { Buffer } from "../dist/index.js";

console.log("=== Buffer Hex & Base64 In-Place Mutators ===");

const buf = Buffer.alloc(16);

// 1. setFromHex: Decode hex string directly into Buffer memory
const hexResult = buf.setFromHex("48656c6c6f"); // "Hello" in hex
console.log("setFromHex bytes written:", hexResult.written, "read:", hexResult.read);
console.log("Decoded string from hex:", buf.toString("utf8", 0, hexResult.written)); // "Hello"

// 2. setFromBase64: Decode Base64 string into Buffer memory
const b64Result = buf.setFromBase64("V29ybGQ=", 6); // "World" starting at offset 6
console.log("setFromBase64 bytes written:", b64Result.written, "read:", b64Result.read);
console.log("Full buffer text:", buf.toString("utf8", 0, 11)); // "Hello World"

// 3. toHex and toBase64 instance methods
console.log("toHex():", buf.subarray(0, 5).toHex()); // "48656c6c6f"
console.log("toBase64():", buf.subarray(0, 5).toBase64()); // "SGVsbG8="
console.log(
  "toBase64({ alphabet: 'base64url', omitPadding: true }):",
  buf.subarray(0, 5).toBase64({ alphabet: "base64url", omitPadding: true }),
);

console.log("All hex & base64 mutator examples executed successfully.");
