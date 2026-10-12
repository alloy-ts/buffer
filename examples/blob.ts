import { Blob, File } from "../src/main.ts";

console.log("=== Alloy Blob & File Demo ===");

// 1. Blob creation
const blob = new Blob(["Hello, ", "world!", new Uint8Array([33])], {
  type: "text/plain",
});

console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);
console.log("Blob text():", blob.text());

// Slicing
const subBlob = blob.slice(0, 5);
console.log("Sliced Blob text():", subBlob.text());

// ArrayBuffer access
const ab = blob.arrayBuffer();
console.log("ArrayBuffer byteLength:", ab.byteLength);

// 2. File creation
const file = new File(["Sample file contents"], "sample.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("\n--- File Object ---");
console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File type:", file.type);
console.log("File lastModified:", file.lastModified);
console.log("File text():", file.text());
