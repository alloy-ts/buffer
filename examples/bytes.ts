import { Bytes, Buffer } from "../src/main.ts";

console.log("=== Bytes API Usage ===");
const emptyBytes = new Bytes();
console.log("Empty Bytes len:", emptyBytes.len(), "isEmpty:", emptyBytes.isEmpty());

const buf = Buffer.from("Rust NAPI Bytes performance example");
const bytesFromBuf = Bytes.fromBuffer(buf);
console.log("Bytes len:", bytesFromBuf.len());
console.log("Bytes asSlice Buffer:", bytesFromBuf.asSlice().toString());

console.log("\n=== Bytes Slicing and Splitting ===");
const subBytes = bytesFromBuf.slice(0, 4);
console.log("Slice (0..4):", subBytes.asSlice().toString());

const mutBytes = Bytes.fromBuffer(Buffer.from("0123456789"));
const rightPart = mutBytes.splitOff(5);
console.log("After splitOff(5) -> left:", mutBytes.asSlice().toString(), "right:", rightPart.asSlice().toString());
