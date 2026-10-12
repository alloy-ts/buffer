import { Buffer, transcode } from "../src/main.ts";

console.log("=== Transcode Examples ===");

// 1. Transcode UTF-8 to ASCII (with substitution for non-ASCII characters)
const utf8Buf = Buffer.from("tést: €100", "utf8");
console.log("Original UTF-8 string:", utf8Buf.toString("utf8"));

const asciiBuf = transcode(utf8Buf, "utf8", "ascii");
console.log("Transcoded to ASCII (unsupported chars become '?'):", asciiBuf.toString("ascii"));

// 2. Transcode UTF-8 to UTF-16LE
const utf16Buf = transcode(utf8Buf, "utf8", "utf16le");
console.log("Transcoded to UTF-16LE byte length:", utf16Buf.length);
console.log("Decoded back from UTF-16LE:", utf16Buf.toString("utf16le"));

// 3. Transcode Latin-1 to UTF-8
const latin1Buf = Buffer.from("tést", "latin1");
const transcodedUtf8 = transcode(latin1Buf, "latin1", "utf8");
console.log("Transcoded Latin-1 to UTF-8:", transcodedUtf8.toString("utf8"));
