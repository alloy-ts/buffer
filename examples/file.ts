import { File } from "../src/main.ts";

console.log("=== File Examples ===");

const file = new File(["file content sample"], "sample.txt", {
  type: "text/plain",
  lastModified: 1700000000000,
});

console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File lastModified:", file.lastModified);
console.log("File content:", await file.text());
