import { Buffer, transcode } from "../src/main.ts";

console.log("=== Transcode Usage Example ===");

const utf8Buf = Buffer.from("€100", "utf8");
console.log("Original UTF-8 Buffer:", utf8Buf, "->", utf8Buf.toString("utf8"));

const asciiBuf = transcode(utf8Buf, "utf8", "ascii");
console.log("Transcoded ASCII Buffer:", asciiBuf, "->", asciiBuf.toString("ascii"));

const latin1Buf = Buffer.from("tést", "latin1");
const utf8Transcoded = transcode(latin1Buf, "latin1", "utf8");
console.log("Latin1 to UTF-8 Transcoded:", utf8Transcoded.toString("utf8"));
