import { Blob, File } from "../src/main.ts";

console.log("=== Blob Creation & Usage ===");
const blob = new Blob(["hello ", "world"], { type: "text/plain" });
console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);
console.log("Blob text():", blob.text());

const slicedBlob = blob.slice(0, 5);
console.log("Sliced Blob text():", slicedBlob.text());

console.log("\n=== File Creation & Usage ===");
const file = new File(["file content"], "example.txt", { type: "text/plain", lastModified: Date.now() });
console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File lastModified:", file.lastModified);
console.log("File text():", file.text());
