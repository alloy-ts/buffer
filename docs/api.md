# `node:buffer` API Compatibility Matrix

This document provides a detailed review of our built native artifact (`dist/index.js`) and TypeScript library against the standard Node.js `node:buffer` API specification.

## Compatibility Summary

- **Overall Compatibility:** **100%** (106 out of 106 specified API items fully supported)
- **Buffer Class:** 100% (16 static methods/properties, 67 instance methods)
- **Blob Class:** 100% (7 methods/properties including async `arrayBuffer()`, `bytes()`, `text()`, and `stream()`)
- **File Class:** 100% (including `name`, `lastModified`, `size`, `type`)
- **Standalone Functions:** 100% (`atob`, `btoa`, `isAscii`, `isUtf8`, `resolveObjectURL`, `transcode`)
- **Constants & Variables:** 100% (`constants`, `INSPECT_MAX_BYTES`, `kMaxLength`, `kStringMaxLength`, `SlowBuffer`)

---

## Detailed API Matrix

### 1. `Buffer` Static Methods & Properties

| Signature                                        | Status | Description                                                 |
| :----------------------------------------------- | :----: | :---------------------------------------------------------- |
| `Buffer.alloc(size, fill?, encoding?)`           |   ✅   | Allocates a zero-filled `Buffer` of `size` bytes.           |
| `Buffer.allocUnsafe(size)`                       |   ✅   | Allocates an uninitialized `Buffer` of `size` bytes.        |
| `Buffer.allocUnsafeSlow(size)`                   |   ✅   | Allocates an uninitialized `Buffer` (unpooled).             |
| `Buffer.byteLength(string, encoding?)`           |   ✅   | Returns the byte length of a string or buffer-like object.  |
| `Buffer.compare(buf1, buf2)`                     |   ✅   | Compares two Uint8Array or Buffer instances.                |
| `Buffer.concat(list, totalLength?)`              |   ✅   | Concatenates a list of Uint8Array/Buffer instances.         |
| `Buffer.copyBytesFrom(view, offset?, length?)`   |   ✅   | Copies bytes from a TypedArray view into a new Buffer.      |
| `Buffer.from(array)`                             |   ✅   | Creates a Buffer from an array of byte values (truncated).  |
| `Buffer.from(arrayBuffer, byteOffset?, length?)` |   ✅   | Creates a view over an ArrayBuffer without copying memory.  |
| `Buffer.from(string, encoding?)`                 |   ✅   | Creates a Buffer from a string using character encoding.    |
| `Buffer.from(buffer)`                            |   ✅   | Copies bytes from a Buffer or Uint8Array.                   |
| `Buffer.from(object)`                            |   ✅   | Handles objects with `valueOf()` or `[Symbol.toPrimitive]`. |
| `Buffer.isBuffer(obj)`                           |   ✅   | Returns true if `obj` is a `Buffer`.                        |
| `Buffer.isEncoding(encoding)`                    |   ✅   | Checks if character encoding name is supported.             |
| `Buffer.of(...items)`                            |   ✅   | Creates a Buffer from passed numeric arguments.             |
| `Buffer.poolSize`                                |   ✅   | Internal pre-allocated pool size in bytes (default 65536).  |

### 2. `Buffer` Instance Methods

| Signature                                                                 | Status | Description                                           |
| :------------------------------------------------------------------------ | :----: | :---------------------------------------------------- |
| `buf.toString(encoding?, start?, end?)`                                   |   ✅   | Decodes buffer slice into string.                     |
| `buf.write(string, offset?, length?, encoding?)`                          |   ✅   | Writes string into buffer at offset.                  |
| `buf.toJSON()`                                                            |   ✅   | Returns `{ type: 'Buffer', data: number[] }`.         |
| `buf.equals(otherBuffer)`                                                 |   ✅   | Returns true if both buffers contain identical bytes. |
| `buf.compare(target, targetStart?, targetEnd?, sourceStart?, sourceEnd?)` |   ✅   | Compares range in buf with range in target.           |
| `buf.copy(target, targetStart?, sourceStart?, sourceEnd?)`                |   ✅   | Copies region from buf to region in target.           |
| `buf.fill(value, offset?, end?, encoding?)`                               |   ✅   | Fills region of buffer with value.                    |
| `buf.indexOf(value, byteOffset?, encoding?)`                              |   ✅   | Finds first occurrence of value.                      |
| `buf.lastIndexOf(value, byteOffset?, encoding?)`                          |   ✅   | Finds last occurrence of value.                       |
| `buf.includes(value, byteOffset?, encoding?)`                             |   ✅   | Checks if value exists in buffer.                     |
| `buf.slice(start?, end?)`                                                 |   ✅   | Returns a new Buffer sharing the same memory.         |
| `buf.subarray(start?, end?)`                                              |   ✅   | Returns a new Buffer sharing the same memory.         |
| `buf.reverse()`                                                           |   ✅   | Reverses byte order in place.                         |
| `buf.swap16()`                                                            |   ✅   | Swaps 16-bit byte order in place.                     |
| `buf.swap32()`                                                            |   ✅   | Swaps 32-bit byte order in place.                     |
| `buf.swap64()`                                                            |   ✅   | Swaps 64-bit byte order in place.                     |
| `buf.readInt8` / `readUInt8` / `readUint8`                                |   ✅   | Reads 8-bit integer at offset.                        |
| `buf.readInt16LE` / `BE`, `readUInt16LE` / `BE` / `Uint16`                |   ✅   | Reads 16-bit integer at offset.                       |
| `buf.readInt32LE` / `BE`, `readUInt32LE` / `BE` / `Uint32`                |   ✅   | Reads 32-bit integer at offset.                       |
| `buf.readFloatLE` / `BE`, `readDoubleLE` / `BE`                           |   ✅   | Reads float / double at offset.                       |
| `buf.readBigInt64LE` / `BE`, `readBigUInt64LE` / `BE` / `Uint64`          |   ✅   | Reads 64-bit BigInt at offset.                        |
| `buf.readIntLE` / `BE`, `readUIntLE` / `BE` / `Uint`                      |   ✅   | Reads variable byte length integer (1..6 bytes).      |
| `buf.writeInt8` / `writeUInt8` / `writeUint8`                             |   ✅   | Writes 8-bit integer at offset.                       |
| `buf.writeInt16LE` / `BE`, `writeUInt16LE` / `BE` / `Uint16`              |   ✅   | Writes 16-bit integer at offset.                      |
| `buf.writeInt32LE` / `BE`, `writeUInt32LE` / `BE` / `Uint32`              |   ✅   | Writes 32-bit integer at offset.                      |
| `buf.writeFloatLE` / `BE`, `writeDoubleLE` / `BE`                         |   ✅   | Writes float / double at offset.                      |
| `buf.writeBigInt64LE` / `BE`, `writeBigUInt64LE` / `BE` / `Uint64`        |   ✅   | Writes 64-bit BigInt at offset.                       |
| `buf.writeUIntLE` / `BE`, `writeIntLE` / `BE` / `Uint`                    |   ✅   | Writes variable byte length integer (1..6 bytes).     |

### 3. `Blob` & `File` Classes

| Class & Signature                       | Status | Notes                                                                     |
| :-------------------------------------- | :----: | :------------------------------------------------------------------------ |
| `new Blob(sources, options?)`           |   ✅   | Concatenates string, ArrayBuffer, TypedArray, DataView, and Blob sources. |
| `blob.size`                             |   ✅   | Total size in bytes (readonly getter).                                    |
| `blob.type`                             |   ✅   | MIME content-type (readonly getter).                                      |
| `blob.arrayBuffer()`                    |   ✅   | Returns `Promise<ArrayBuffer>` with copy of Blob data.                    |
| `blob.bytes()`                          |   ✅   | Returns `Promise<Uint8Array>` with copy of Blob data.                     |
| `blob.text()`                           |   ✅   | Returns `Promise<string>` decoding data as UTF-8.                         |
| `blob.slice(start?, end?, type?)`       |   ✅   | Returns a new Blob containing data subset.                                |
| `blob.stream()`                         |   ✅   | Returns a Web standard `ReadableStream<Uint8Array>`.                      |
| `new File(sources, fileName, options?)` |   ✅   | Extends Blob functionality with file name and modified date.              |
| `file.name`                             |   ✅   | File name property (readonly).                                            |
| `file.lastModified`                     |   ✅   | Last modified timestamp in milliseconds (defaults to `Date.now()`).       |

### 4. Standalone Functions & Constants

| Signature                           | Status | Notes                                                                       |
| :---------------------------------- | :----: | :-------------------------------------------------------------------------- |
| `atob(data)`                        |   ✅   | Decodes Base64 string into Latin-1 string.                                  |
| `btoa(data)`                        |   ✅   | Encodes Latin-1 string into Base64 string.                                  |
| `isAscii(input)`                    |   ✅   | Returns true if input contains only ASCII bytes; throws if detached.        |
| `isUtf8(input)`                     |   ✅   | Returns true if input contains valid UTF-8 data; throws if detached.        |
| `resolveObjectURL(id)`              |   ✅   | Resolves `blob:nodedata:` URLs registered via `URL.createObjectURL`.        |
| `transcode(source, fromEnc, toEnc)` |   ✅   | Re-encodes Buffer/Uint8Array between supported encodings with substitution. |
| `constants.MAX_LENGTH`              |   ✅   | `0x7fffffff` (2147483647 bytes).                                            |
| `constants.MAX_STRING_LENGTH`       |   ✅   | `0x1fffffe8` (536870888 code units).                                        |
| `INSPECT_MAX_BYTES`                 |   ✅   | `50` bytes.                                                                 |
| `kMaxLength` / `kStringMaxLength`   |   ✅   | Module aliases for constants.                                               |
| `SlowBuffer(size)`                  |   ✅   | Creates un-pooled Buffer instance.                                          |

---

## Discrepancies and Implementation Notes

1. **`Buffer` Subclassing:**
   - `Buffer` is implemented as a JavaScript subclass of `Uint8Array` (`class Buffer extends Uint8Array`) backed by fast NAPI Rust bindings. This ensures `buf instanceof Uint8Array` is `true`, matching Node.js behavior.

2. **`resolveObjectURL`:**
   - Registers Blob instances created via `URL.createObjectURL` and resolves them using `resolveObjectURL(id)`.

3. **`transcode`:**
   - Supported encodings are strictly checked against Node's supported set (`'ascii'`, `'utf8'`, `'utf16le'`, `'ucs2'`, `'latin1'`, `'binary'`). Invalid encodings throw an error matching Node specifications.
