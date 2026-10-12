import { Blob } from "../src/main.ts";

console.log("--- Blob Usage ---");

// 1. Constructing a Blob
const parts = ["Hello ", "world!", new Uint8Array([32, 102, 114, 111, 109, 32, 66, 108, 111, 98])];
const blob = new Blob(parts, { type: "text/plain" });

console.log("Blob size (bytes):", blob.size);
console.log("Blob MIME type:", blob.type);

// 2. Reading Blob contents
console.log("Blob text():", blob.text());

// 3. Converting Blob to Uint8Array/Buffer or ArrayBuffer
const bytes = blob.bytes();
console.log("Blob bytes():", bytes);

const ab = blob.arrayBuffer();
console.log("Blob arrayBuffer() byteLength:", ab.byteLength);

// 4. Slicing a Blob
const slicedBlob = blob.slice(0, 5, "text/plain");
console.log("Sliced Blob size:", slicedBlob.size);
console.log("Sliced Blob text():", slicedBlob.text());
