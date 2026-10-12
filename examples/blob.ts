import { Blob } from "../src/main.ts";

console.log("=== Blob Examples ===");

const blob = new Blob(["Hello, ", "world!"], { type: "text/plain" });
console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);

console.log("Blob text:", await blob.text());

const sliced = blob.slice(0, 5);
console.log("Sliced Blob text:", await sliced.text());

const bytes = await blob.bytes();
console.log("Blob bytes:", bytes);
