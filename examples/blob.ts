import { Blob, File } from "../src/main.ts";

// 1. Creating Blob
const blob = new Blob(["Hello ", "world!"], { type: "text/plain" });
console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);
console.log("Blob text():", blob.text());

// 2. Blob slicing
const slice = blob.slice(0, 5);
console.log("Sliced Blob text():", slice.text());

// 3. Creating File
const file = new File(["file content"], "example.txt", { type: "text/plain" });
console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File lastModified:", file.lastModified);
console.log("File text():", file.text());
