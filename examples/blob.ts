import { Blob, File } from "../src/main.ts";

console.log("=== Blob and File Example ===");

// 1. Creating a Blob
const blob = new Blob(["hello ", "world!"], { type: "text/plain" });
console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);
console.log("Blob text:", blob.text());

// 2. Blob slicing
const slicedBlob = blob.slice(0, 5);
console.log("Sliced Blob size:", slicedBlob.size);
console.log("Sliced Blob text:", slicedBlob.text());

// 3. Blob bytes / ArrayBuffer
const bytes = blob.bytes();
console.log("Blob bytes:", Array.from(bytes));

// 4. Creating a File
const file = new File(["file content line 1\n", "line 2"], "example.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File lastModified:", new Date(file.lastModified).toISOString());
console.log("File content:", file.text());
