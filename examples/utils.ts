import { atob, btoa, isAscii, isUtf8, transcode, Buffer } from "../src/main.ts";

console.log("=== Base64 Utilities (atob / btoa) ===");
const encoded = btoa("Hello World");
console.log("btoa('Hello World'):", encoded);
console.log("atob('SGVsbG8gV29ybGQ='):", atob(encoded));

console.log("\n=== Encoding Validation (isAscii / isUtf8) ===");
const asciiBuf = Buffer.from("Hello World");
const utf8Buf = Buffer.from("tést UTF-8 🚀");
console.log("isAscii('Hello World'):", isAscii(asciiBuf));
console.log("isAscii('tést UTF-8 🚀'):", isAscii(utf8Buf));
console.log("isUtf8('tést UTF-8 🚀'):", isUtf8(utf8Buf));

console.log("\n=== Transcoding (transcode) ===");
const utf8Buffer = Buffer.from("Hello World €", "utf8");
const asciiBuffer = transcode(utf8Buffer, "utf8", "ascii");
console.log("Transcoded UTF-8 -> ASCII:", asciiBuffer.toString("ascii"));
