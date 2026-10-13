import { transcode, atob, btoa, isAscii, isUtf8, isLatin1, Buffer } from "../src/main.ts";

console.log("=== Transcode API ===");
const utf8Buf = Buffer.from("café", "utf8");
const latin1Buf = transcode(utf8Buf, "utf8", "latin1");
console.log("Transcoded utf8 -> latin1 bytes:", Array.from(latin1Buf));
console.log("Decoded latin1:", latin1Buf.toString("latin1"));

console.log("\n=== Base64 Helpers (atob / btoa) ===");
const encoded = btoa("Hello World!");
console.log("btoa('Hello World!'):", encoded);
console.log("atob(encoded):", atob(encoded));

console.log("\n=== Validation Helpers ===");
console.log("isAscii('Hello'):", isAscii(Buffer.from("Hello")));
console.log("isUtf8('café'):", isUtf8(Buffer.from("café")));
console.log("isLatin1('café'):", isLatin1("café"));
console.log("isLatin1('€'):", isLatin1("€"));
