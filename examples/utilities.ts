import {
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
  Buffer,
} from "../src/main.ts";

console.log("=== Utility Functions and Constants Demo ===");

// 1. atob and btoa
const text = "Hello, Web & Node!";
const encoded = btoa(text);
console.log("btoa():", encoded);
const decoded = atob(encoded);
console.log("atob():", decoded);

// 2. Character Set Verification Functions
const asciiData = Buffer.from("Hello World", "ascii");
const utf8Data = Buffer.from("Café", "utf8");

console.log("isAscii(asciiData):", isAscii(asciiData));
console.log("isUtf8(utf8Data):", isUtf8(utf8Data));
console.log("isLatin1('Café'):", isLatin1("Café"));
console.log("isLatin1('€'):", isLatin1("€"));

// 3. Buffer Transcoding
const euroUtf8 = Buffer.from("€", "utf8");
const euroAscii = transcode(euroUtf8, "utf8", "ascii");
console.log("Transcoded '€' from UTF-8 to ASCII:", euroAscii.toString("ascii"));

// 4. Module Constants
console.log("constants.MAX_LENGTH:", constants.MAX_LENGTH);
console.log("constants.MAX_STRING_LENGTH:", constants.MAX_STRING_LENGTH);
console.log("kMaxLength:", kMaxLength);
console.log("kStringMaxLength:", kStringMaxLength);
console.log("INSPECT_MAX_BYTES:", INSPECT_MAX_BYTES);

console.log("Utilities demo completed successfully.");
