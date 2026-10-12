import {
  atob,
  btoa,
  constants,
  isAscii,
  isUtf8,
  resolveObjectURL,
  transcode,
} from "../src/main.ts";

console.log("=== Utility Functions Examples ===");

const encoded = btoa("Hello World");
console.log("btoa('Hello World'):", encoded);
console.log("atob(encoded):", atob(encoded));

const asciiBuffer = new Uint8Array([0x41, 0x42, 0x43]);
console.log("isAscii:", isAscii(asciiBuffer));

const utf8Buffer = new Uint8Array([0x74, 0xc3, 0xa9, 0x73, 0x74]);
console.log("isUtf8:", isUtf8(utf8Buffer));

const transcoded = transcode(utf8Buffer, "utf8", "ascii");
console.log("transcoded ascii string:", transcoded.toString("ascii"));

console.log("resolveObjectURL:", resolveObjectURL("blob:nodedata:123"));
console.log("constants:", constants);
