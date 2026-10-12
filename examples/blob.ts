import { Blob } from "../src/main.ts";

console.log("--- Blob Demo ---");
const blob = new Blob(["hello ", "world!"], { type: "text/plain" });

console.log("Blob size:", blob.size);
console.log("Blob type:", blob.type);
console.log("Blob text():", blob.text());

const sliced = blob.slice(0, 5);
console.log("Sliced Blob (0..5) text():", sliced.text());

const bytes = blob.bytes();
console.log("Blob bytes():", bytes);

const ab = blob.arrayBuffer();
console.log("Blob arrayBuffer():", ab.byteLength, "bytes");
