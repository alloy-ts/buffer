import { File } from "../src/main.ts";

console.log("=== File Usage Example ===");

const file = new File(["Sample file contents"], "sample.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File mime type:", file.type);
console.log("File lastModified:", new Date(file.lastModified).toISOString());
console.log("File text():", file.text());

const slicedFile = file.slice(0, 6);
console.log("Sliced File text():", slicedFile.text());
