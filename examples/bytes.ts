import { Bytes, Buffer } from "../src/main.ts";

console.log("--- Bytes (Rust napi-rs handle) Usage ---");

// 1. Creating Bytes from a Buffer
const buf = Buffer.from("Hello from Bytes!");
const bytes = Bytes.fromBuffer(buf);

console.log("Bytes len():", bytes.len());
console.log("Bytes isEmpty():", bytes.isEmpty());

// 2. Cheap O(1) Slicing
const subBytes = bytes.slice(0, 5);
console.log("Sliced Bytes len():", subBytes.len());

// 3. Converting back to a Node Buffer copy
const convertedBuf = bytes.asSlice();
console.log("Converted Buffer:", convertedBuf.toString());
