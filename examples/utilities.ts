import {
  transcode,
  atob,
  btoa,
  isAscii,
  isUtf8,
  isLatin1,
} from "../dist/index.js";

console.log("=== node:buffer Utility Functions Example ===");

// 1. Base64 encoding and decoding utilities (atob / btoa)
const text = "Hello, World!";
const encoded = btoa(text);
console.log("btoa('Hello, World!'):", encoded); // "SGVsbG8sIFdvcmxkIQ=="
console.log("atob(encoded):", atob(encoded)); // "Hello, World!"

// 2. Character encoding validation helpers
const asciiBytes = new Uint8Array([72, 101, 108, 108, 111]); // "Hello"
const utf8Bytes = new Uint8Array([67, 97, 102, 195, 169]); // "Café"

console.log("isAscii('Hello'):", isAscii(asciiBytes)); // true
console.log("isUtf8('Café'):", isUtf8(utf8Bytes)); // true

console.log("isLatin1('Hello'):", isLatin1("Hello")); // true
console.log("isLatin1('Café'):", isLatin1("Café")); // true
console.log("isLatin1('☕'):", isLatin1("☕")); // false

// 3. Transcoding strings between character encodings
const sourceBytes = new TextEncoder().encode("Euro sign: €");

// Transcode UTF-8 to ASCII (unrepresentable '€' becomes '?')
const transcodedAscii = transcode(sourceBytes, "utf8", "ascii");
console.log("Transcoded to ASCII:", transcodedAscii.toString("ascii")); // "Euro sign: ?"

// Transcode UTF-8 to Latin1
const transcodedLatin1 = transcode(new TextEncoder().encode("Café"), "utf8", "latin1");
console.log("Transcoded to Latin1:", transcodedLatin1.toString("latin1")); // "Café"

console.log("All utility functions executed successfully.");
