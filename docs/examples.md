# Code Examples & Demos

This guide provides practical examples demonstrating how to use `@alloy-ts/buffer` for binary data manipulation, file handling, character encoding conversion, and high-performance Rust buffer handles.

Executable example scripts are available in the [`examples/`](../examples/) directory:

- [`examples/buffer.ts`](../examples/buffer.ts)
- [`examples/blob.ts`](../examples/blob.ts)
- [`examples/file.ts`](../examples/file.ts)
- [`examples/bytes.ts`](../examples/bytes.ts)

Running examples:

```bash
npx tsx examples/buffer.ts
```

---

## 1. `Buffer` Usage

The `Buffer` class is a subclass of JavaScript's `Uint8Array` that extends it with Node.js-compatible binary methods and encodings.

```ts
import { Buffer } from "@alloy-ts/buffer";

// Creates a zero-filled Buffer of length 10
const buf1 = Buffer.alloc(10);

// Creates a Buffer filled with byte value 1
const buf2 = Buffer.alloc(10, 1);

// Creates an uninitialized Buffer of length 10
const buf3 = Buffer.allocUnsafe(10);

// Creates a Buffer from an array of byte values
const buf4 = Buffer.from([1, 2, 3]);

// Values > 255 are truncated using (value & 255)
const buf5 = Buffer.from([257, 257.5, -255, "1"]); // [1, 1, 1, 1]

// Creates a Buffer from string (UTF-8 by default)
const buf6 = Buffer.from("tést");

// Creates a Buffer from string using Latin-1 encoding
const buf7 = Buffer.from("tést", "latin1");

// Decodes Buffer contents into string using various encodings
console.log(buf6.toString("utf8")); // "tést"
console.log(buf7.toString("latin1")); // "tést"

// Decodes hex and base64 strings
const hexBuf = Buffer.from("74c3a97374", "hex");
console.log(hexBuf.toString("utf8")); // "tést"

const b64Buf = Buffer.from("dOlzdA==", "base64");
console.log(b64Buf.toString("latin1")); // "tést"
```

### Subarrays & Memory Sharing

`Buffer.prototype.subarray()` and `Buffer.prototype.slice()` return views over the same underlying `ArrayBuffer` memory:

```ts
const alphabet = Buffer.allocUnsafe(26);
for (let i = 0; i < 26; i++) {
  alphabet[i] = i + 97; // 'a' through 'z'
}

const sub = alphabet.subarray(0, 3);
console.log(sub.toString("ascii")); // "abc"

alphabet[0] = 33; // ASCII '!'
console.log(sub.toString("ascii")); // "!bc"
```

### Numeric Reads & Writes

Read and write multi-byte integers, floats, doubles, and BigInts in big-endian or little-endian byte orders:

```ts
const numBuf = Buffer.alloc(16);
numBuf.writeInt8(-42, 0);
numBuf.writeUInt16LE(0xdead, 2);
numBuf.writeUInt32BE(0xfeedface, 4);
numBuf.writeBigInt64LE(0x0102030405060708n, 8);

console.log(numBuf.readInt8(0)); // -42
console.log(numBuf.readUInt16LE(2).toString(16)); // "dead"
console.log(numBuf.readUInt32BE(4).toString(16)); // "feedface"
console.log(numBuf.readBigInt64LE(8)); // 72623859790382856n
```

---

## 2. `Blob` Usage

`Blob` encapsulates immutable raw data across multiple thread workers and streams:

```ts
import { Blob } from "@alloy-ts/buffer";

// Construct a Blob from string, Uint8Array, Buffer, or ArrayBuffer parts
const parts = [
  "Hello ",
  "world!",
  new Uint8Array([32, 102, 114, 111, 109, 32, 66, 108, 108, 111, 98]),
];
const blob = new Blob(parts, { type: "text/plain" });

console.log(blob.size); // 22 bytes
console.log(blob.type); // "text/plain"
console.log(blob.text()); // "Hello world! from Blob"

// Read byte representations
const bytes = blob.bytes(); // Uint8Array/Buffer
const ab = blob.arrayBuffer(); // Promise/ArrayBuffer

// Slice a Blob
const sliced = blob.slice(0, 5, "text/plain");
console.log(sliced.text()); // "Hello"
```

---

## 3. `File` Usage

`File` extends `Blob` to represent file system metadata including file name and last modified date:

```ts
import { File } from "@alloy-ts/buffer";

const file = new File(["Hello ", "file content"], "example.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});

console.log(file.name); // "example.txt"
console.log(file.size); // 18
console.log(file.type); // "text/plain"
console.log(new Date(file.lastModified).toISOString());

console.log(file.text()); // "Hello file content"
```

---

## 4. `Bytes` (Rust NAPI Handle) Usage

`Bytes` is an efficient Rust memory slice handle exposed via NAPI bindings:

```ts
import { Bytes, Buffer } from "@alloy-ts/buffer";

const buf = Buffer.from("Hello from Bytes!");
const bytes = Bytes.fromBuffer(buf);

console.log(bytes.len()); // 17
console.log(bytes.isEmpty()); // false

// Cheap O(1) slice sharing memory reference
const subBytes = bytes.slice(0, 5);
console.log(subBytes.len()); // 5

// Convert back to writable Buffer
const converted = bytes.asSlice();
console.log(converted.toString()); // "Hello from Bytes!"
```
