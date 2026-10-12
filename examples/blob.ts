import { Blob, Buffer } from "../src/main.ts";

console.log("=== Blob API Demo ===");

// 1. Create a Blob from string and buffer parts
const blob = new Blob(["Hello, ", "world!", " ", new Uint8Array([240, 159, 153, 130])], {
  type: "text/plain;charset=utf-8",
});

console.log("Blob size (bytes):", blob.size);
console.log("Blob content type:", blob.type);

// 2. Read Blob contents as text
console.log("Blob text:", blob.text());

// 3. Slice a Blob
const sliced = blob.slice(0, 5, "text/plain");
console.log("Sliced size:", sliced.size);
console.log("Sliced text:", sliced.text());

// 4. Extract as Buffer and ArrayBuffer
const buf = Buffer.from(blob.bytes());
console.log("Extracted Buffer:", buf);
console.log("Buffer to hex:", buf.toHex());

const ab = blob.arrayBuffer();
console.log("ArrayBuffer byteLength:", ab.byteLength);
