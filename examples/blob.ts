import { Blob, File } from "../src/main.ts";

console.log("=== Blob and File API Demo ===");

// 1. Blob creation and properties
const blobParts = ["Hello ", "world!", new Uint8Array([32, 70, 114, 111, 109, 32, 66, 108, 111, 98])];
const blob = new Blob(blobParts, { type: "text/plain" });

console.log("Blob size:", blob.size, "bytes");
console.log("Blob type:", blob.type);

// 2. Reading Blob contents
const textContent = blob.text();
console.log("blob.text():", textContent);

const bytes = blob.bytes();
console.log("blob.bytes():", bytes.toString());

const arrayBuf = blob.arrayBuffer();
console.log("blob.arrayBuffer() byteLength:", arrayBuf.byteLength);

// 3. Blob Slicing
const slicedBlob = blob.slice(0, 12);
console.log("slicedBlob.text():", slicedBlob.text());

// 4. File class usage
const file = new File(["Sample file contents"], "example.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("File name:", file.name);
console.log("File size:", file.size, "bytes");
console.log("File type:", file.type);
console.log("File lastModified:", new Date(file.lastModified).toISOString());
console.log("file.text():", file.text());

console.log("Blob & File demo completed successfully.");
