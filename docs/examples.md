# Code Examples & Demos

This guide provides practical examples demonstrating how to use `@alloy-ts/buffer` for binary data manipulation, file handling, character encoding conversion, interop with TypedArrays, and high-performance Rust buffer handles.

Executable example scripts are available in the [`examples/`](../examples/) directory:

- [`examples/buffer.ts`](../examples/buffer.ts)
- [`examples/binary-io.ts`](../examples/binary-io.ts)
- [`examples/encodings.ts`](../examples/encodings.ts)
- [`examples/interop.ts`](../examples/interop.ts)
- [`examples/blob.ts`](../examples/blob.ts)
- [`examples/file.ts`](../examples/file.ts)
- [`examples/bytes.ts`](../examples/bytes.ts)

Running examples:

```bash
npx tsx examples/buffer.ts
npx tsx examples/binary-io.ts
npx tsx examples/encodings.ts
```

---

## 1. Basic `Buffer` Usage

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
```

---

## 2. Encodings & String Conversions ([`examples/encodings.ts`](../examples/encodings.ts))

Supported encodings: `utf8`, `ascii`, `latin1`/`binary`, `hex`, `base64`, `base64url`, and `utf16le`/`ucs2`.

```ts
import { Buffer } from "@alloy-ts/buffer";

// Base64 & Base64URL
const b64 = Buffer.from("hello world").toString("base64"); // "aGVsbG8gd29ybGQ="
const b64url = Buffer.from("hello world").toString("base64url"); // "aGVsbG8gd29ybGQ"

// Hex
const hex = Buffer.from("ABC").toString("hex"); // "414243"

// Latin1 / Binary (1:1 byte-to-char)
const latin1 = Buffer.from("café", "latin1");

// UTF-16LE / UCS2
const u16 = Buffer.from("Hi☺", "utf16le");

// Buffer.write with explicit offset, length, and encoding
const buf = Buffer.alloc(12);
buf.write("ABCDEFG", 0, 3, "utf8"); // Writes 3 bytes ("ABC")
```

---

## 3. Numeric & Binary I/O ([`examples/binary-io.ts`](../examples/binary-io.ts))

Read and write multi-byte integers, IEEE-754 floats/doubles, BigInts, and variable byte length (1–6 bytes) integers:

```ts
import { Buffer } from "@alloy-ts/buffer";

const b = Buffer.alloc(8);
b.writeInt8(-1, 0);
b.writeUInt16LE(0x1234, 1);
b.writeInt32BE(-123456, 3);

// 64-bit BigInt
const big = Buffer.alloc(8);
big.writeBigUInt64LE(18446744073709551615n, 0);
big.writeBigInt64BE(-9007199254740991n, 0);

// IEEE-754 floats / doubles
const f = Buffer.alloc(8);
f.writeFloatLE(1.5, 0);
f.writeDoubleBE(Math.PI, 0);

// Variable length integers (1..6 bytes)
const v = Buffer.alloc(6);
v.writeUIntLE(0x123456, 0, 3);
v.writeIntLE(-1000, 0, 2);

// Byte swapping
const sw = Buffer.from([0x01, 0x02, 0x03, 0x04]);
sw.swap32(); // [4, 3, 2, 1]
```

---

## 4. TypedArray & Memory Interop ([`examples/interop.ts`](../examples/interop.ts))

```ts
import { Buffer } from "@alloy-ts/buffer";

// Buffer is a Uint8Array
const buf = Buffer.from([1, 2, 3]);
console.log(buf instanceof Uint8Array); // true

// Share memory with ArrayBuffer
const ab = new ArrayBuffer(4);
const shared = Buffer.from(ab);
shared[0] = 42;

// copyBytesFrom: copy raw backing bytes of wider views
const f64 = new Float64Array([1.0, 2.0]);
const fromF64 = Buffer.copyBytesFrom(f64); // 16 bytes
```

---

## 5. `Blob` & `File` Usage ([`examples/blob.ts`](../examples/blob.ts), [`examples/file.ts`](../examples/file.ts))

```ts
import { Blob, File } from "@alloy-ts/buffer";

// Blob
const blob = new Blob(["Hello ", "world!"], { type: "text/plain" });
console.log(blob.size); // 12
console.log(blob.text()); // "Hello world!"

// File
const file = new File(["file content"], "example.txt", {
  type: "text/plain",
  lastModified: Date.now(),
});
console.log(file.name); // "example.txt"
```

---

## 6. `Bytes` (Rust NAPI Handle) Usage ([`examples/bytes.ts`](../examples/bytes.ts))

```ts
import { Bytes, Buffer } from "@alloy-ts/buffer";

const buf = Buffer.from("Hello from Bytes!");
const bytes = Bytes.fromBuffer(buf);

// Cheap O(1) slice sharing memory reference
const subBytes = bytes.slice(0, 5);

// Convert back to Buffer copy
const converted = bytes.asSlice();
console.log(converted.toString()); // "Hello"
```
