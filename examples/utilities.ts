import { Buffer, atob, btoa, isAscii, isUtf8, transcode } from "../src/main.ts";

console.log("=== Utilities Example ===");

// 1. Base64 encoding & decoding with btoa/atob
const rawStr = "hello world";
const encodedB64 = btoa(rawStr);
console.log(`btoa("${rawStr}"):`, encodedB64);

const decodedStr = atob(encodedB64);
console.log(`atob("${encodedB64}"):`, decodedStr);

// 2. Format validation with isAscii and isUtf8
const asciiBuf = Buffer.from("Hello ASCII 123");
const utf8Buf = Buffer.from("tést 😀");

console.log("isAscii(asciiBuf):", isAscii(asciiBuf));
console.log("isUtf8(utf8Buf):", isUtf8(utf8Buf));

// 3. Encoding transcoding
const euroBuf = Buffer.from("€", "utf8");
const transcodedBuf = transcode(euroBuf, "utf8", "ascii");
console.log("transcode('€', 'utf8' -> 'ascii'):", transcodedBuf.toString("ascii")); // "?"
