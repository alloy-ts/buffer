import { Bytes, Buffer } from "../src/main.ts";

console.log("=== Rust Bytes API Demo ===");

// 1. Create Bytes from Buffer
const input = Buffer.from("Rust NAPI Bytes handles high-performance zero-copy slice operations");
const bytes = Bytes.fromBuffer(input);

console.log("Bytes len:", bytes.len());
console.log("Is empty:", bytes.isEmpty());
console.log("Is unique:", bytes.isUnique());

// 2. Zero-copy slice operation
const slice1 = bytes.slice(0, 10);
console.log("Slice 0..10 as Buffer:", slice1.buf().toString());

// 3. Mutating operations: splitOff & splitTo
let mutableBytes = Bytes.fromBuffer(Buffer.from("Hello World!"));
const tail = mutableBytes.splitOff(5);
console.log("Head after splitOff(5):", mutableBytes.buf().toString()); // "Hello"
console.log("Tail from splitOff(5):", tail.buf().toString()); // " World!"

// 4. Truncate & Clear
let truncBytes = Bytes.fromBuffer(Buffer.from("1234567890"));
truncBytes.truncate(5);
console.log("Truncated to 5 bytes:", truncBytes.buf().toString());

truncBytes.clear();
console.log("After clear len:", truncBytes.len());
