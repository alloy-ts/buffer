import { Bytes } from "../index.js";
import { Buffer } from "../src/main.ts";

console.log("=== Bytes Example ===");

// 1. Creating a Bytes instance from a Buffer
const initialBuf = Buffer.from("hello world from bytes");
const bytes = Bytes.fromBuffer(initialBuf);

console.log("Bytes length:", bytes.len());
console.log("Bytes isEmpty:", bytes.isEmpty());
console.log("Bytes isUnique:", bytes.isUnique());

// 2. Cheap O(1) slicing
const slicedBytes = bytes.slice(0, 5);
console.log("Sliced Bytes length:", slicedBytes.len());
console.log("Sliced Bytes content:", slicedBytes.asSlice().toString());

// 3. Splitting Bytes
let mutBytes = Bytes.fromBuffer(Buffer.from("part1:part2"));
const rightPart = mutBytes.splitOff(6);
console.log("Left part after splitOff(6):", mutBytes.asSlice().toString());
console.log("Right part from splitOff(6):", rightPart.asSlice().toString());

// 4. Truncating and Clearing
let truncBytes = Bytes.fromBuffer(Buffer.from("truncated text"));
truncBytes.truncate(9);
console.log("Truncated bytes:", truncBytes.asSlice().toString());

truncBytes.clear();
console.log("Cleared bytes length:", truncBytes.len());
