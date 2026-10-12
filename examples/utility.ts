import {
  Buffer,
  atob,
  btoa,
  isAscii,
  isUtf8,
  isLatin1,
  transcode,
  constants,
  INSPECT_MAX_BYTES,
  kMaxLength,
  kStringMaxLength,
} from "../src/main.ts";

console.log("=== Alloy node:buffer Utilities Demo ===");

// 1. Base64 atob / btoa
const encoded = btoa("Hello World");
console.log("btoa('Hello World'):", encoded);
console.log("atob(encoded):", atob(encoded));

// 2. Encoding validation helpers
const asciiBuf = Buffer.from("Hello");
const utf8Buf = Buffer.from("tést");

console.log("\n--- Validation Helpers ---");
console.log("isAscii('Hello'):", isAscii(asciiBuf));
console.log("isUtf8('tést'):", isUtf8(utf8Buf));
console.log("isLatin1('café'):", isLatin1("café"));
console.log("isLatin1('€'):", isLatin1("€")); // false, euro sign > 0xFF

// 3. Transcoding
console.log("\n--- Transcoding ---");
const euroUtf8 = Buffer.from("€", "utf8");
const transcoded = transcode(euroUtf8, "utf8", "ascii");
console.log("Transcoded '€' (utf8 -> ascii):", transcoded.toString("ascii")); // "?"

// 4. Constants
console.log("\n--- Constants ---");
console.log("constants.MAX_LENGTH:", constants.MAX_LENGTH);
console.log("constants.MAX_STRING_LENGTH:", constants.MAX_STRING_LENGTH);
console.log("kMaxLength:", kMaxLength);
console.log("kStringMaxLength:", kStringMaxLength);
console.log("INSPECT_MAX_BYTES:", INSPECT_MAX_BYTES);
