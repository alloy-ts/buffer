import { Buffer, atob, btoa, isAscii, isUtf8, transcode } from "../src/main.ts";

console.log("--- Transcode & Encoding Helpers Demo ---");
const euroUtf8 = Buffer.from("€");
console.log("Original UTF-8 Euro sign buffer:", euroUtf8);

const transcoded = transcode(euroUtf8, "utf8", "ascii");
console.log("Transcoded to ASCII (with substitution):", transcoded.toString("ascii"));

console.log("\n--- Base64 Helpers (atob / btoa) ---");
const encoded = btoa("Hello, world!");
console.log("btoa('Hello, world!'):", encoded);
console.log("atob(encoded):", atob(encoded));

console.log("\n--- Encoding Inspection (isAscii / isUtf8) ---");
console.log("isAscii('hello'):", isAscii(Buffer.from("hello")));
console.log("isAscii('tést'):", isAscii(Buffer.from("tést")));
console.log("isUtf8('tést'):", isUtf8(Buffer.from("tést")));
