import { Blob } from "../src/main.ts";

console.log("=== Blob Usage ===");
const blob = new Blob(["Hello, ", "World!", new Uint8Array([33])], {
  type: "text/plain",
});

console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);
console.log("Blob text():", blob.text());

console.log("\n=== Blob Slicing ===");
const slicedBlob = blob.slice(0, 5, "text/plain");
console.log("Sliced Blob size:", slicedBlob.size);
console.log("Sliced Blob text():", slicedBlob.text());

console.log("\n=== Blob Bytes and ArrayBuffer ===");
const bytes = blob.bytes();
console.log("Blob bytes as Buffer:", bytes.toString());
const ab = blob.arrayBuffer();
console.log("ArrayBuffer byteLength:", ab.byteLength);
