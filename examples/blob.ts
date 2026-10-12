import { Blob, File, Buffer } from "../src/main.ts";

console.log("=== Node.js Blob and File API Demonstration ===");

// 1. Creating a Blob
console.log("\n--- 1. Blob Creation & Properties ---");
const buf = Buffer.from("Buffer data ");
const blob = new Blob(["Hello ", buf, "World!"], { type: "text/plain;charset=utf-8" });

console.log("blob.size:", blob.size);
console.log("blob.type:", blob.type);

// 2. Blob Content Extraction Methods
console.log("\n--- 2. Extracting Blob Contents ---");
console.log("blob.text():", blob.text());

const bufferCopy = blob.bytes();
console.log("blob.bytes():", bufferCopy, "-> string:", bufferCopy.toString());

const arrayBuffer = blob.arrayBuffer();
console.log("blob.arrayBuffer():", arrayBuffer, "byteLength:", arrayBuffer.byteLength);

// 3. Blob Slicing
console.log("\n--- 3. Blob Slicing ---");
const sliced = blob.slice(0, 5, "text/plain");
console.log("sliced.size:", sliced.size);
console.log("sliced.type:", sliced.type);
console.log("sliced.text():", sliced.text());

// 4. File Creation & Properties
console.log("\n--- 4. File Creation & Properties ---");
const file = new File(["Sample file content"], "sample.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("file.name:", file.name);
console.log("file.size:", file.size);
console.log("file.type:", file.type);
console.log("file.lastModified:", file.lastModified, "->", new Date(file.lastModified).toISOString());
console.log("file.text():", file.text());

console.log("\n=== Blob and File API Demo Completed Successfully ===");
