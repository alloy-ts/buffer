/**
 * examples/file.ts
 *
 * Demonstrates File creation, name, lastModified, text(), bytes(), and slice().
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/file.ts
 */
import { File } from "../src/buffer.js";

console.log("=== File Examples ===");

// 1. Create a File
const file = new File(["sample file content"], "example.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File type:", file.type);
console.log("File lastModified:", new Date(file.lastModified).toISOString());

// 2. Read File text & bytes
console.log("File text():", file.text());
console.log("File bytes():", file.bytes());

// 3. Slice File
const sliced = file.slice(0, 6, "text/plain");
console.log("Sliced file text():", sliced.text());
