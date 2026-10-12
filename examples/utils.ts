/**
 * examples/utils.ts
 *
 * Demonstrates module utility functions: atob, btoa, isAscii, isUtf8, isLatin1,
 * transcode, constants, and SlowBuffer.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/utils.ts
 */
import {
  atob,
  btoa,
  isAscii,
  isUtf8,
  isLatin1,
  transcode,
  constants,
  Buffer,
  SlowBuffer,
} from "../src/buffer.js";

console.log("=== Utility Functions Examples ===");

// 1. Base64 atob and btoa
const encoded = btoa("Hello, Node!");
console.log("btoa('Hello, Node!'):", encoded);
console.log("atob(encoded):", atob(encoded));

// 2. Encoding detection helpers
const asciiBuf = Buffer.from("ASCII text");
const utf8Buf = Buffer.from("UTF-8 text: 🚀");

console.log("isAscii(asciiBuf):", isAscii(asciiBuf));
console.log("isUtf8(utf8Buf):", isUtf8(utf8Buf));
console.log("isLatin1('Hello World'):", isLatin1("Hello World"));
console.log("isLatin1('Unicode: 🚀'):", isLatin1("Unicode: 🚀"));

// 3. Character encoding transcode
const euroBuf = Buffer.from("€", "utf8");
const transcoded = transcode(euroBuf, "utf8", "ascii");
console.log("Transcoded '€' to ascii:", transcoded.toString("ascii"));

// 4. Constants
console.log("constants.MAX_LENGTH:", constants.MAX_LENGTH);
console.log("constants.MAX_STRING_LENGTH:", constants.MAX_STRING_LENGTH);

// 5. SlowBuffer
const slow = SlowBuffer(16);
console.log("SlowBuffer size:", slow.length);
