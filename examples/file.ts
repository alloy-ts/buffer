import { File } from "../src/main.ts";

console.log("--- File Usage ---");

// 1. Constructing a File
const file = new File(["Hello ", "file content"], "example.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("File name:", file.name);
console.log("File size (bytes):", file.size);
console.log("File MIME type:", file.type);
console.log("File lastModified:", new Date(file.lastModified).toISOString());

// 2. Reading File text
console.log("File text():", file.text());

// 3. Slicing a File
const slicedFile = file.slice(0, 5);
console.log("Sliced File size:", slicedFile.size);
console.log("Sliced File text():", slicedFile.text());
