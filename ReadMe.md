# Node:buffer Compatible Module

A Node.js and Deno compatible implementation of the `node:buffer` API.

## Features

- **`Buffer`**: Full `Uint8Array` subclass implementing `Buffer.alloc`, `Buffer.allocUnsafe`, `Buffer.from`, `Buffer.concat`, `Buffer.copyBytesFrom`, `Buffer.isBuffer`, `Buffer.isEncoding`, `Buffer.byteLength`, `Buffer.compare`, encoding conversions (`utf8`, `ascii`, `latin1`, `hex`, `base64`, `base64url`, `utf16le`), numeric read/write methods (`readUInt8`, `writeUInt32BE`, `readFloatLE`, `readDoubleBE`, `readBigInt64LE`, 48-bit `readUIntBE/LE`, `writeIntBE/LE`, etc.), `slice`, `subarray`, `copy`, `fill`, `indexOf`, `lastIndexOf`, `includes`, `swap16/32/64`, `reverse`, and `toJSON`.
- **`Blob` & `File`**: Full implementation of Web/Node `Blob` and `File` classes (`arrayBuffer()`, `bytes()`, `text()`, `slice()`, `stream()`, `name`, `lastModified`).
- **Utility Functions**: `atob`, `btoa`, `isAscii`, `isUtf8`, `resolveObjectURL`, `transcode`.
- **Constants & Variables**: `constants` (`MAX_LENGTH`, `MAX_STRING_LENGTH`), `INSPECT_MAX_BYTES`, `kMaxLength`, `kStringMaxLength`, `SlowBuffer`.

## Usage

```ts
import { atob, Blob, btoa, Buffer, File, isAscii, isUtf8, transcode } from "@lib/module";

// Create a zero-filled Buffer of length 10
const buf1 = Buffer.alloc(10);

// Create a Buffer from string
const buf2 = Buffer.from("tést", "utf8");

// Slice and convert
console.log(buf2.toString("hex"));

// Blob and File
const blob = new Blob(["hello world"], { type: "text/plain" });
console.log(await blob.text());
```

## Development

- Install dependencies:

```bash
npm install
```

- Run the unit tests:

```bash
npm test
```

- Build the library:

```bash
npm run build
```

- Code formatting:

```bash
npm run fmt
```

- Linting:

```bash
npm run lint
```

- Type check:

```bash
npm run check
```
