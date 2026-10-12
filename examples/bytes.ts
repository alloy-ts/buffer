import { Buffer, Bytes } from "../src/main.ts";

console.log("--- Bytes (Rust bytes::Bytes) Demo ---");
const buf = Buffer.from([10, 20, 30, 40, 50, 60, 70, 80]);
const bytes = Bytes.fromBuffer(buf);

console.log("Bytes length:", bytes.len());
console.log("Bytes isUnique():", bytes.isUnique());
console.log("Bytes asSlice():", Array.from(bytes.asSlice()));

const sliced = bytes.slice(2, 6);
console.log("Sliced Bytes (2..6):", Array.from(sliced.asSlice()));

const right = bytes.splitOff(4);
console.log("After splitOff(4) - Left:", Array.from(bytes.asSlice()));
console.log("After splitOff(4) - Right:", Array.from(right.asSlice()));
