import { File } from "../src/main.ts";

console.log("--- File Demo ---");
const file = new File(["Hello ", "File API!"], "hello.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File type:", file.type);
console.log("File lastModified:", new Date(file.lastModified).toISOString());
console.log("File text():", file.text());
