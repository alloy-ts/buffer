# Node:Buffer NAPI-RS Bindings

High-performance, strongly-typed NAPI-RS bindings implementing a Node.js `node:buffer` compatible API in Rust for Node.js environments.

## Features

- **`Buffer` Class**: Full set of static allocators (`alloc`, `allocUnsafe`, `fromBuffer`, `fromString`, `fromArray`, `concat`), integer/float/BigInt LE & BE offset-based reads and writes, variable byte-length reads/writes, string encoding/decoding, search (`indexOf`, `lastIndexOf`, `includes`), `copy`, `fill`, `slice`, `subarray`, `swap16/32/64`, `equals`, `compare`, `keys`, `values`, `entries`, and `toJSON`.
- **`Blob` & `File` Classes**: Full W3C / Node `Blob` and `File` support with constructor options (`type`, `endings: "native" | "transparent"`), `slice`, `arrayBuffer`, `bytes`, `text`, `stream`, and `textStream`.
- **`node:buffer` Module Utilities**: `atob`, `btoa`, `isAscii`, `isLatin1`, `isUtf8`, `transcode`, `resolveObjectURL`, and buffer `constants`.
- **`Bytes` Integration**: NAPI bindings for `bytes::Bytes` with zero-cost reference counting and slicing.

## Usage

```ts
import {
  Buffer,
  Blob,
  File,
  atob,
  btoa,
  isAscii,
  isLatin1,
  isUtf8,
  transcode,
  constants,
} from "@lib/module";

// Allocating buffers
const buf1 = Buffer.alloc(10);
const buf2 = Buffer.fromString("hello world", "utf8");

// Integer & Float reads/writes
buf1.writeUInt16LE(0x1234, 0);
console.log(buf1.readUInt16LE(0)); // 0x1234

// 64-bit BigInt reads/writes
buf1.writeBigUInt64BE(0x0102030405060708n, 0);
console.log(buf1.readBigUInt64BE(0)); // 0x0102030405060708n

// Blobs and Files
const blob = new Blob(["hello ", "world"], { type: "text/plain" });
console.log(blob.size); // 11
console.log(blob.text()); // "hello world"

const file = new File(["file data"], "example.txt", { type: "text/plain" });
console.log(file.name); // "example.txt"

// Utility functions
console.log(isLatin1("café")); // true
console.log(btoa("hello")); // "aGVsbG8="
```

For complete API signatures and compatibility details, see [docs/api.md](docs/api.md).

## Development

- Install dependencies:

```bash
npm install
```

- Run tests:

```bash
npm test
```

- Run Rust tests:

```bash
cargo test
```

- Build library:

```bash
npm run build
```

- Code formatting & linting:

```bash
npm run fmt
npm run lint
```
