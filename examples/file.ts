import { File } from "../src/main.ts";

console.log("=== File API Usage ===");
const file = new File(["sample file content"], "example.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File type:", file.type);
console.log("File lastModified:", new Date(file.lastModified).toISOString());
console.log("File text():", file.text());

console.log("\n=== File Slice ===");
const slice = file.slice(0, 6);
console.log("Sliced File text():", slice.text());
