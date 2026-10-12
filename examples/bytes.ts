import { Bytes, Buffer } from "../src/main.ts";

console.log("=== Native Bytes API Demo ===");

// 1. Creation from Buffer
const inputBuf = Buffer.from("Reference Counted Byte Storage", "utf8");
const bytes = Bytes.fromBuffer(inputBuf);

console.log("Bytes len:", bytes.len());
console.log("Bytes isEmpty:", bytes.isEmpty());
console.log("Bytes isUnique:", bytes.isUnique());

// 2. Cheap O(1) Slicing
const slice1 = bytes.slice(0, 17);
console.log("slice(0, 17) as Buffer:", slice1.asSlice().toString());

// 3. Splitting
let mutBytes = Bytes.fromBuffer(Buffer.from("Part1 - Part2", "utf8"));
const part2 = mutBytes.splitOff(8);
console.log("mutBytes after splitOff(8):", mutBytes.asSlice().toString());
console.log("part2 from splitOff(8):", part2.asSlice().toString());

// 4. Converting back to Buffer
const outputBuf = bytes.buf();
console.log("Converted back to Buffer:", outputBuf.toString());

console.log("Native Bytes demo completed successfully.");
