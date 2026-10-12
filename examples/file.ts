import { File, Buffer, atob, btoa, isAscii, isUtf8, isLatin1, transcode } from "../src/main.ts";

console.log("=== File & Utility API Demo ===");

// 1. File class demo
const file = new File(["sample file content"], "data.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log("File name:", file.name);
console.log("File size:", file.size);
console.log("File content type:", file.type);
console.log("File lastModified:", new Date(file.lastModified).toISOString());
console.log("File text:", file.text());

// 2. Character encoding utilities
const asciiStr = "Hello World";
const utf8Str = "café ☕";

console.log("isAscii('Hello World'):", isAscii(Buffer.from(asciiStr)));
console.log("isUtf8('café'):", isUtf8(Buffer.from(utf8Str)));
console.log("isLatin1('café'):", isLatin1("café"));

// 3. Base64 atob / btoa utilities
const encoded = btoa("Hello Base64");
console.log("btoa('Hello Base64'):", encoded);
console.log("atob(encoded):", atob(encoded));

// 4. Character encoding transcoding
const euro = Buffer.from("€", "utf8");
const transcoded = transcode(euro, "utf8", "ascii");
console.log("Transcoded € from utf8 to ascii:", transcoded.toString("ascii"));
