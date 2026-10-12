import {
  Buffer,
  atob,
  btoa,
  isAscii,
  isUtf8,
  constants,
  INSPECT_MAX_BYTES,
  SlowBuffer,
} from "../src/main.ts";

console.log("=== Utilities Usage Example ===");

// Base64 helpers
const base64Str = btoa("Hello World!");
console.log("btoa('Hello World!'):", base64Str);
console.log("atob('aGVsbG8='):", atob("aGVsbG8="));

// Format validation
console.log("isAscii('hello'):", isAscii(Buffer.from("hello")));
console.log("isUtf8('tést'):", isUtf8(Buffer.from("tést")));

// Constants
console.log("constants.MAX_LENGTH:", constants.MAX_LENGTH);
console.log("constants.MAX_STRING_LENGTH:", constants.MAX_STRING_LENGTH);
console.log("INSPECT_MAX_BYTES:", INSPECT_MAX_BYTES);

// SlowBuffer
const slowBuf = SlowBuffer(16);
console.log("SlowBuffer(16) length:", slowBuf.length);
