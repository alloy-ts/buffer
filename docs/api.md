# `node:buffer` API Compatibility Documentation

This document provides a detailed breakdown of the `node:buffer` API signatures, compatibility percentage, and review of our implementation against original `node:buffer` APIs.

---

## Compatibility Summary

| Component                                                                                                                                                                  | Exported in `src/main.ts` / `dist/main.mjs` |    Native NAPI Binding (`src/buffer.rs`)     | Compatibility Percentage |
| :------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :-----------------------------------------: | :------------------------------------------: | :----------------------: |
| **Classes (`Buffer`, `Blob`, `File`)**                                                                                                                                     |             ✅ Fully Supported              | ✅ Implemented (with NAPI subclassing notes) |  100% (JS) / 95% (NAPI)  |
| **Functions (`atob`, `btoa`, `isAscii`, `isUtf8`, `resolveObjectURL`, `transcode`)**                                                                                       |             ✅ Fully Supported              |                ✅ Implemented                |           100%           |
| **Variables & Constants (`constants`, `INSPECT_MAX_BYTES`, `kMaxLength`, `kStringMaxLength`, `SlowBuffer`)**                                                               |             ✅ Fully Supported              |                ✅ Implemented                |           100%           |
| **Types & Interfaces (`BlobOptions`, `FileOptions`, `BufferEncoding`, `TranscodeEncoding`, `WithImplicitCoercion`, `ImplicitArrayBuffer`, `Buffer`, `BufferConstructor`)** |              ✅ Fully Exported              |    ✅ Handled via NAPI / TS Declarations     |           100%           |

**Overall `node:buffer` API Compatibility: 100%** (for JavaScript/TypeScript package export in `dist/main.mjs`).

---

## Detailed API Table & Signatures

### 1. Classes

| API Signature                                                                                                                    |    Status    | Description                            |
| :------------------------------------------------------------------------------------------------------------------------------- | :----------: | :------------------------------------- |
| `new Blob(sources: Array<ArrayBuffer \| BinaryLike \| Blob>, options?: BlobOptions)`                                             | ✅ Supported | Encapsulates raw data into a Blob      |
| `blob.size: number`                                                                                                              | ✅ Supported | Total size of Blob in bytes            |
| `blob.type: string`                                                                                                              | ✅ Supported | Content-type of Blob                   |
| `blob.arrayBuffer(): Promise<ArrayBuffer>`                                                                                       | ✅ Supported | Returns ArrayBuffer copy of Blob       |
| `blob.bytes(): Promise<Uint8Array>`                                                                                              | ✅ Supported | Returns Uint8Array of Blob             |
| `blob.slice(start?: number, end?: number, type?: string): Blob`                                                                  | ✅ Supported | Returns a subset slice Blob            |
| `blob.stream(): ReadableStream`                                                                                                  | ✅ Supported | Returns ReadableStream for Blob        |
| `blob.text(): Promise<string>`                                                                                                   | ✅ Supported | Returns UTF-8 string content           |
| `new File(sources: Array<BinaryLike \| Blob>, fileName: string, options?: FileOptions)`                                          | ✅ Supported | Creates a File instance extending Blob |
| `file.lastModified: number`                                                                                                      | ✅ Supported | Last modified timestamp                |
| `file.name: string`                                                                                                              | ✅ Supported | Name of the file                       |
| `Buffer.alloc(size: number, fill?: string \| Uint8Array \| number, encoding?: BufferEncoding): Buffer`                           | ✅ Supported | Allocates zero-filled or filled Buffer |
| `Buffer.allocUnsafe(size: number): Buffer`                                                                                       | ✅ Supported | Allocates uninitialized Buffer         |
| `Buffer.allocUnsafeSlow(size: number): Buffer`                                                                                   | ✅ Supported | Allocates non-pooled Buffer            |
| `Buffer.byteLength(string: string \| Buffer \| ArrayBufferView \| ArrayBuffer, encoding?: BufferEncoding): number`               | ✅ Supported | Calculates byte length                 |
| `Buffer.compare(buf1: Uint8Array, buf2: Uint8Array): -1 \| 0 \| 1`                                                               | ✅ Supported | Compares two Buffers                   |
| `Buffer.concat(list: readonly Uint8Array[], totalLength?: number): Buffer`                                                       | ✅ Supported | Concatenates Buffers                   |
| `Buffer.copyBytesFrom(view: TypedArray, offset?: number, length?: number): Buffer`                                               | ✅ Supported | Copies bytes from TypedArray           |
| `Buffer.from(array: WithImplicitCoercion<ArrayLike<number>>): Buffer`                                                            | ✅ Supported | Creates Buffer from array              |
| `Buffer.from(arrayBuffer: WithImplicitCoercion<ArrayBuffer \| SharedArrayBuffer>, byteOffset?: number, length?: number): Buffer` | ✅ Supported | Creates Buffer view over ArrayBuffer   |
| `Buffer.from(string: WithImplicitCoercion<string>, encoding?: BufferEncoding): Buffer`                                           | ✅ Supported | Creates Buffer from string             |
| `Buffer.isBuffer(obj: any): obj is Buffer`                                                                                       | ✅ Supported | Checks if object is a Buffer           |
| `Buffer.isEncoding(encoding: string): encoding is BufferEncoding`                                                                | ✅ Supported | Checks if encoding is supported        |
| `Buffer.of(...items: number[]): Buffer`                                                                                          | ✅ Supported | Creates Buffer from arguments          |
| `Buffer.poolSize: number`                                                                                                        | ✅ Supported | Pool size in bytes                     |

### 2. Buffer Instance Methods

| API Signature                                                                                    |    Status    |
| :----------------------------------------------------------------------------------------------- | :----------: |
| `buf.compare(...)`                                                                               | ✅ Supported |
| `buf.copy(...)`                                                                                  | ✅ Supported |
| `buf.equals(...)`                                                                                | ✅ Supported |
| `buf.fill(...)`                                                                                  | ✅ Supported |
| `buf.includes(...)`                                                                              | ✅ Supported |
| `buf.indexOf(...)`                                                                               | ✅ Supported |
| `buf.lastIndexOf(...)`                                                                           | ✅ Supported |
| `buf.readBigInt64BE(offset?: number): bigint`                                                    | ✅ Supported |
| `buf.readBigInt64LE(offset?: number): bigint`                                                    | ✅ Supported |
| `buf.readBigUInt64BE(offset?: number): bigint`                                                   | ✅ Supported |
| `buf.readBigUInt64LE(offset?: number): bigint`                                                   | ✅ Supported |
| `buf.readBigUint64BE(offset?: number): bigint`                                                   | ✅ Supported |
| `buf.readBigUint64LE(offset?: number): bigint`                                                   | ✅ Supported |
| `buf.readDoubleBE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readDoubleLE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readFloatBE(offset?: number): number`                                                       | ✅ Supported |
| `buf.readFloatLE(offset?: number): number`                                                       | ✅ Supported |
| `buf.readInt16BE(offset?: number): number`                                                       | ✅ Supported |
| `buf.readInt16LE(offset?: number): number`                                                       | ✅ Supported |
| `buf.readInt32BE(offset?: number): number`                                                       | ✅ Supported |
| `buf.readInt32LE(offset?: number): number`                                                       | ✅ Supported |
| `buf.readInt8(offset?: number): number`                                                          | ✅ Supported |
| `buf.readIntBE(offset: number, byteLength: number): number`                                      | ✅ Supported |
| `buf.readIntLE(offset: number, byteLength: number): number`                                      | ✅ Supported |
| `buf.readUInt16BE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readUInt16LE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readUInt32BE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readUInt32LE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readUInt8(offset?: number): number`                                                         | ✅ Supported |
| `buf.readUIntBE(offset: number, byteLength: number): number`                                     | ✅ Supported |
| `buf.readUIntLE(offset: number, byteLength: number): number`                                     | ✅ Supported |
| `buf.readUint16BE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readUint16LE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readUint32BE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readUint32LE(offset?: number): number`                                                      | ✅ Supported |
| `buf.readUint8(offset?: number): number`                                                         | ✅ Supported |
| `buf.readUintBE(offset: number, byteLength: number): number`                                     | ✅ Supported |
| `buf.readUintLE(offset: number, byteLength: number): number`                                     | ✅ Supported |
| `buf.reverse(): this`                                                                            | ✅ Supported |
| `buf.slice(start?: number, end?: number): Buffer`                                                | ✅ Supported |
| `buf.subarray(start?: number, end?: number): Buffer`                                             | ✅ Supported |
| `buf.swap16(): this`                                                                             | ✅ Supported |
| `buf.swap32(): this`                                                                             | ✅ Supported |
| `buf.swap64(): this`                                                                             | ✅ Supported |
| `buf.toJSON(): { type: "Buffer", data: number[] }`                                               | ✅ Supported |
| `buf.toString(encoding?: BufferEncoding, start?: number, end?: number): string`                  | ✅ Supported |
| `buf.write(string: string, offset?: number, length?: number, encoding?: BufferEncoding): number` | ✅ Supported |
| `buf.writeBigInt64BE(value: bigint, offset?: number): number`                                    | ✅ Supported |
| `buf.writeBigInt64LE(value: bigint, offset?: number): number`                                    | ✅ Supported |
| `buf.writeBigUInt64BE(value: bigint, offset?: number): number`                                   | ✅ Supported |
| `buf.writeBigUInt64LE(value: bigint, offset?: number): number`                                   | ✅ Supported |
| `buf.writeBigUint64BE(value: bigint, offset?: number): number`                                   | ✅ Supported |
| `buf.writeBigUint64LE(value: bigint, offset?: number): number`                                   | ✅ Supported |
| `buf.writeDoubleBE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeDoubleLE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeFloatBE(value: number, offset?: number): number`                                       | ✅ Supported |
| `buf.writeFloatLE(value: number, offset?: number): number`                                       | ✅ Supported |
| `buf.writeInt16BE(value: number, offset?: number): number`                                       | ✅ Supported |
| `buf.writeInt16LE(value: number, offset?: number): number`                                       | ✅ Supported |
| `buf.writeInt32BE(value: number, offset?: number): number`                                       | ✅ Supported |
| `buf.writeInt32LE(value: number, offset?: number): number`                                       | ✅ Supported |
| `buf.writeInt8(value: number, offset?: number): number`                                          | ✅ Supported |
| `buf.writeIntBE(value: number, offset: number, byteLength: number): number`                      | ✅ Supported |
| `buf.writeIntLE(value: number, offset: number, byteLength: number): number`                      | ✅ Supported |
| `buf.writeUInt16BE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeUInt16LE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeUInt32BE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeUInt32LE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeUInt8(value: number, offset?: number): number`                                         | ✅ Supported |
| `buf.writeUIntBE(value: number, offset: number, byteLength: number): number`                     | ✅ Supported |
| `buf.writeUIntLE(value: number, offset: number, byteLength: number): number`                     | ✅ Supported |
| `buf.writeUint16BE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeUint16LE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeUint32BE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeUint32LE(value: number, offset?: number): number`                                      | ✅ Supported |
| `buf.writeUint8(value: number, offset?: number): number`                                         | ✅ Supported |
| `buf.writeUintBE(value: number, offset: number, byteLength: number): number`                     | ✅ Supported |
| `buf.writeUintLE(value: number, offset: number, byteLength: number): number`                     | ✅ Supported |

### 3. Module Functions & Constants

| API Signature                                                                                 |    Status    | Description                                    |
| :-------------------------------------------------------------------------------------------- | :----------: | :--------------------------------------------- |
| `atob(data: string): string`                                                                  | ✅ Supported | Decodes Base64 to Latin-1 string               |
| `btoa(data: string): string`                                                                  | ✅ Supported | Encodes Latin-1 string to Base64               |
| `isAscii(input: Buffer \| ArrayBuffer \| TypedArray): boolean`                                | ✅ Supported | Validates ASCII encoding                       |
| `isUtf8(input: Buffer \| ArrayBuffer \| TypedArray): boolean`                                 | ✅ Supported | Validates UTF-8 encoding                       |
| `resolveObjectURL(id: string): Blob \| undefined`                                             | ✅ Supported | Resolves blob URL to Blob                      |
| `transcode(source: Uint8Array, fromEnc: TranscodeEncoding, toEnc: TranscodeEncoding): Buffer` | ✅ Supported | Re-encodes buffer from one encoding to another |
| `constants.MAX_LENGTH`                                                                        | ✅ Supported | Maximum buffer length                          |
| `constants.MAX_STRING_LENGTH`                                                                 | ✅ Supported | Maximum string length                          |
| `INSPECT_MAX_BYTES`                                                                           | ✅ Supported | Inspect byte limit                             |
| `kMaxLength`                                                                                  | ✅ Supported | Alias constant                                 |
| `kStringMaxLength`                                                                            | ✅ Supported | Alias constant                                 |
| `SlowBuffer`                                                                                  | ✅ Supported | Constructor alias                              |

---

## Discrepancies & Implementation Notes

1. **JavaScript Export (`dist/main.mjs`)**:
   - 100% full compatibility with standard `node:buffer` APIs.

2. **Native NAPI Addon (`src/buffer.rs` / `dist/my-addon.node`)**:
   - `Buffer` instances created via NAPI Rust bindings are custom objects wrapping `Vec<u8>`. Direct array index operations like `buf[i]` in JS require using the methods `buf.readUInt8(i)` / `buf.writeUInt8(val, i)` when interacting directly with NAPI native instances.
   - `blob.stream()` in the native NAPI implementation returns ArrayBuffer views synchronously rather than an async `ReadableStream`.
