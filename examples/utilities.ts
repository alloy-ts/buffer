import {
  atob,
  btoa,
  isAscii,
  isUtf8,
  transcode,
  constants,
  INSPECT_MAX_BYTES,
  SlowBuffer,
  Buffer,
} from "../src/main.ts";

console.log("=== Utility Functions & Constants Examples ===");

// 1. Base64 atob and btoa
const encoded = btoa("Hello, world!");
console.log("btoa('Hello, world!'):", encoded);
console.log("atob(encoded):", atob(encoded));

// 2. Content validators isAscii & isUtf8
const asciiBuf = Buffer.from("Hello");
const utf8Buf = Buffer.from("tést 🌍");
console.log("isAscii('Hello'):", isAscii(asciiBuf));
console.log("isUtf8('tést 🌍'):", isUtf8(utf8Buf));

// 3. Transcoding between encodings
const euroBuf = Buffer.from("€100", "utf8");
const transcodedAscii = transcode(euroBuf, "utf8", "ascii");
console.log("Transcoded UTF-8 '€100' to ASCII:", transcodedAscii.toString("ascii"));

// 4. Constants
console.log("MAX_LENGTH:", constants.MAX_LENGTH);
console.log("MAX_STRING_LENGTH:", constants.MAX_STRING_LENGTH);
console.log("INSPECT_MAX_BYTES:", INSPECT_MAX_BYTES);

// 5. SlowBuffer
const slow = SlowBuffer(10);
console.log("SlowBuffer(10) length:", slow.length);
