import { Blob } from "../src/main.ts";

console.log("=== Blob Usage Example ===");

const blob = new Blob(["Hello ", "World!"], { type: "text/plain" });

console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);
console.log("Blob text:", blob.text());

const slicedBlob = blob.slice(0, 5);
console.log("Sliced Blob size:", slicedBlob.size);
console.log("Sliced Blob text:", slicedBlob.text());

const bytes = blob.bytes();
console.log("Blob bytes:", bytes);
