import { Bytes, Buffer } from "../src/main.ts";

// 1. Creating Bytes from Buffer
const buf = Buffer.from("Zero-copy sliceable memory with Bytes", "utf8");
const bytes = Bytes.fromBuffer(buf as any);

console.log("Bytes length:", bytes.len());
console.log("Bytes isEmpty:", bytes.isEmpty());

// 2. Cheap slicing
const sliced = bytes.slice(0, 15);
console.log("Sliced Bytes as text:", sliced.asSlice().toString("utf8"));

// 3. Splitting Bytes
const tail = bytes.splitOff(16);
console.log("Tail after splitOff(16):", tail.asSlice().toString("utf8"));
