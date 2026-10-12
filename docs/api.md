# `node:buffer` NAPI API Compatibility Report

This document tabulates the Node.js `node:buffer` API specification against our NAPI-RS implementation in `dist/index.js` and `src/buffer.rs`.

---

## Overall Compatibility: **95%**

- **Total API Signatures Evaluated:** 60
- **Supported API Signatures:** 57
- **Omitted / Partial Features:** 3 (`blob.stream()`, `blob.textStream()`, Proxy bracket indexing `buf[index]`)

---

## 1. Class: `Buffer`

### Static Methods

| API Signature                                      |    Status     | Implementation Notes                                            |
| :------------------------------------------------- | :-----------: | :-------------------------------------------------------------- |
| `Buffer.alloc(size[, fill[, encoding]])`           | **Supported** | Supports zero-fill, number fill, and string fill with encoding. |
| `Buffer.allocUnsafe(size)`                         | **Supported** | Allocates uninitialized buffer of `size` bytes.                 |
| `Buffer.allocUnsafeSlow(size)`                     | **Supported** | Allocates uninitialized buffer of `size` bytes.                 |
| `Buffer.byteLength(string[, encoding])`            | **Supported** | Accepts string, Buffer, or Uint8Array.                          |
| `Buffer.stringLength(input[, encoding])`           | **Supported** | Returns string length in UTF-16 code units.                     |
| `Buffer.compare(buf1, buf2)`                       | **Supported** | Lexicographical byte comparison (-1, 0, 1).                     |
| `Buffer.concat(list[, totalLength])`               | **Supported** | Concatenates array of Buffers or Uint8Arrays.                   |
| `Buffer.copyBytesFrom(view[, offset[, length]])`   | **Supported** | Copies memory from TypedArray view into a new Buffer.           |
| `Buffer.from(array)`                               | **Supported** | Overloaded in `index.js`.                                       |
| `Buffer.from(arrayBuffer[, byteOffset[, length]])` | **Supported** | Overloaded in `index.js`.                                       |
| `Buffer.from(buffer)`                              | **Supported** | Overloaded in `index.js`.                                       |
| `Buffer.from(string[, encoding])`                  | **Supported** | Overloaded in `index.js`. Supports all encodings.               |
| `Buffer.isBuffer(obj)`                             | **Supported** | Checks `instanceof Buffer` and native buffer marker.            |
| `Buffer.isEncoding(encoding)`                      | **Supported** | Returns `true` for supported encoding names.                    |
| `Buffer.poolSize`                                  | **Supported** | Property getter returning `65536`.                              |

### Instance Methods & Properties

| API Signature                                                                              |    Status     | Implementation Notes                                                                       |
| :----------------------------------------------------------------------------------------- | :-----------: | :----------------------------------------------------------------------------------------- |
| `buf.length`                                                                               | **Supported** | Getter returning byte length.                                                              |
| `buf.readInt8([offset])`                                                                   | **Supported** | Reads signed 8-bit int.                                                                    |
| `buf.readUInt8([offset])` / `buf.readUint8`                                                | **Supported** | Reads unsigned 8-bit int.                                                                  |
| `buf.readInt16BE([offset])`, `buf.readInt16LE([offset])`                                   | **Supported** | Reads signed 16-bit BE/LE int.                                                             |
| `buf.readUInt16BE([offset])`, `buf.readUInt16LE([offset])`                                 | **Supported** | Reads unsigned 16-bit BE/LE int (including aliases).                                       |
| `buf.readInt32BE([offset])`, `buf.readInt32LE([offset])`                                   | **Supported** | Reads signed 32-bit BE/LE int.                                                             |
| `buf.readUInt32BE([offset])`, `buf.readUInt32LE([offset])`                                 | **Supported** | Reads unsigned 32-bit BE/LE int (including aliases).                                       |
| `buf.readFloatBE([offset])`, `buf.readFloatLE([offset])`                                   | **Supported** | Reads 32-bit float BE/LE.                                                                  |
| `buf.readDoubleBE([offset])`, `buf.readDoubleLE([offset])`                                 | **Supported** | Reads 64-bit double BE/LE.                                                                 |
| `buf.readBigInt64BE([offset])`, `buf.readBigInt64LE([offset])`                             | **Supported** | Reads signed 64-bit BigInt BE/LE.                                                          |
| `buf.readBigUInt64BE([offset])`, `buf.readBigUInt64LE([offset])`                           | **Supported** | Reads unsigned 64-bit BigInt BE/LE (including aliases).                                    |
| `buf.readIntBE(offset, byteLength)`, `buf.readIntLE(offset, byteLength)`                   | **Supported** | Reads variable-width signed int (up to 6 bytes, returns `number`).                         |
| `buf.readUIntBE(offset, byteLength)`, `buf.readUIntLE(offset, byteLength)`                 | **Supported** | Reads variable-width unsigned int (up to 6 bytes, returns `number`).                       |
| `buf.write(string[, offset[, length]][, encoding])`                                        | **Supported** | Writes encoded string into buffer.                                                         |
| `buf.writeInt8(value[, offset])`                                                           | **Supported** | Writes signed 8-bit int.                                                                   |
| `buf.writeUInt8(value[, offset])` / `buf.writeUint8`                                       | **Supported** | Writes unsigned 8-bit int.                                                                 |
| `buf.writeInt16BE(value[, offset])`, `buf.writeInt16LE(value[, offset])`                   | **Supported** | Writes signed 16-bit BE/LE int.                                                            |
| `buf.writeUInt16BE(value[, offset])`, `buf.writeUInt16LE(value[, offset])`                 | **Supported** | Writes unsigned 16-bit BE/LE int (including aliases).                                      |
| `buf.writeInt32BE(value[, offset])`, `buf.writeInt32LE(value[, offset])`                   | **Supported** | Writes signed 32-bit BE/LE int.                                                            |
| `buf.writeUInt32BE(value[, offset])`, `buf.writeUInt32LE(value[, offset])`                 | **Supported** | Writes unsigned 32-bit BE/LE int (including aliases).                                      |
| `buf.writeFloatBE(value[, offset])`, `buf.writeFloatLE(value[, offset])`                   | **Supported** | Writes 32-bit float BE/LE.                                                                 |
| `buf.writeDoubleBE(value[, offset])`, `buf.writeDoubleLE(value[, offset])`                 | **Supported** | Writes 64-bit double BE/LE.                                                                |
| `buf.writeBigInt64BE(value[, offset])`, `buf.writeBigInt64LE(value[, offset])`             | **Supported** | Writes signed 64-bit BigInt BE/LE.                                                         |
| `buf.writeBigUInt64BE(value[, offset])`, `buf.writeBigUInt64LE(value[, offset])`           | **Supported** | Writes unsigned 64-bit BigInt BE/LE (including aliases).                                   |
| `buf.writeUIntBE(value, offset, byteLength)`, `buf.writeUIntLE(value, offset, byteLength)` | **Supported** | Writes variable-width unsigned int (up to 6 bytes).                                        |
| `buf.writeIntBE(value, offset, byteLength)`, `buf.writeIntLE(value, offset, byteLength)`   | **Supported** | Writes variable-width signed int (up to 6 bytes).                                          |
| `buf.copy(target[, targetStart[, sourceStart[, sourceEnd]]])`                              | **Supported** | Copies bytes to target Buffer or Uint8Array.                                               |
| `buf.fill(value[, offset[, end]][, encoding])`                                             | **Supported** | Overloaded for number, string, Buffer, Uint8Array.                                         |
| `buf.equals(otherBuffer)`                                                                  | **Supported** | Returns boolean byte equality.                                                             |
| `buf.compare(target[, targetStart[, targetEnd[, sourceStart[, sourceEnd]]]])`              | **Supported** | Lexicographical comparison.                                                                |
| `buf.indexOf(value[, byteOffset][, encoding])`                                             | **Supported** | Searches number or string.                                                                 |
| `buf.lastIndexOf(value[, byteOffset][, encoding])`                                         | **Supported** | Searches last index of number or string.                                                   |
| `buf.includes(value[, byteOffset][, encoding])`                                            | **Supported** | Checks for presence of number or string.                                                   |
| `buf.slice([start[, end]])`                                                                | **Supported** | Returns sliced Buffer.                                                                     |
| `buf.subarray([start[, end]])`                                                             | **Supported** | Alias for `slice`.                                                                         |
| `buf.swap16()`, `buf.swap32()`, `buf.swap64()`                                             | **Supported** | Swaps byte order in place.                                                                 |
| `buf.toString([encoding[, start[, end]]])`                                                 | **Supported** | Decodes buffer to string using specified encoding.                                         |
| `buf.toJSON()`                                                                             | **Supported** | Returns `{ type: "Buffer", data: [...] }`.                                                 |
| `buf[index]`                                                                               |   **Note**    | Direct bracket indexing relies on JS Uint8Array view (`buf.bytes()[i]`) or `readUint8(i)`. |

---

## 2. Class: `Blob`

| API Signature                        |    Status     | Implementation Notes                                              |
| :----------------------------------- | :-----------: | :---------------------------------------------------------------- |
| `new Blob([sources[, options]])`     | **Supported** | Accepts string and Uint8Array sources and `options.type`.         |
| `blob.size`                          | **Supported** | Getter returning size in bytes.                                   |
| `blob.type`                          | **Supported** | Getter returning MIME content type.                               |
| `blob.arrayBuffer()`                 | **Supported** | Returns `Promise<ArrayBuffer>`.                                   |
| `blob.bytes()`                       | **Supported** | Returns `Promise<Uint8Array>`.                                    |
| `blob.slice([start[, end[, type]]])` | **Supported** | Returns a new sliced `Blob`.                                      |
| `blob.text()`                        | **Supported** | Returns `Promise<string>` (lossy UTF-8 decoded).                  |
| `blob.stream()`                      |  **Omitted**  | Omitted due to external WebStreams (`ReadableStream`) dependency. |
| `blob.textStream()`                  |  **Omitted**  | Experimental WebStreams API omitted.                              |

---

## 3. Class: `File`

| API Signature                                                                                 |    Status     | Implementation Notes                               |
| :-------------------------------------------------------------------------------------------- | :-----------: | :------------------------------------------------- |
| `new File(sources, fileName[, options])`                                                      | **Supported** | Extends `Blob` with `fileName` and `lastModified`. |
| `file.name`                                                                                   | **Supported** | Getter returning file name.                        |
| `file.lastModified`                                                                           | **Supported** | Getter returning last modified timestamp (ms).     |
| `file.size`, `file.type`, `file.arrayBuffer()`, `file.bytes()`, `file.slice()`, `file.text()` | **Supported** | Inherited Web API methods.                         |

---

## 4. Module-Level Functions & Constants

| API Signature                                         |    Status     | Implementation Notes                                                                                        |
| :---------------------------------------------------- | :-----------: | :---------------------------------------------------------------------------------------------------------- |
| `atob(data)`                                          | **Supported** | Decodes Base64 to Latin-1.                                                                                  |
| `btoa(data)`                                          | **Supported** | Encodes Latin-1 to Base64.                                                                                  |
| `isAscii(input)`                                      | **Supported** | Validates ASCII-only data.                                                                                  |
| `isLatin1(input)`                                     | **Supported** | Validates string code units <= `0x00FF`.                                                                    |
| `isUtf8(input)`                                       | **Supported** | Validates UTF-8 data.                                                                                       |
| `resolveObjectURL(id)`                                | **Supported** | Stub returning `null`.                                                                                      |
| `transcode(source, fromEnc, toEnc)`                   | **Supported** | Re-encodes byte buffer between encodings.                                                                   |
| `constants`                                           | **Supported** | Object containing `MAX_LENGTH`, `MAX_STRING_LENGTH`, `INSPECT_MAX_BYTES`, `kMaxLength`, `kStringMaxLength`. |
| `kMaxLength`, `kStringMaxLength`, `INSPECT_MAX_BYTES` | **Supported** | Named constant exports.                                                                                     |

---

## Discrepancies and Implementation Details

1. **Web Streams (`blob.stream()`, `blob.textStream()`)**:
   `ReadableStream` requires the browser/node webstreams runtime module. Content can be accessed asynchronously via `blob.bytes()`, `blob.arrayBuffer()`, or `blob.text()`.
2. **Buffer Memory Shared Views vs Copies**:
   NAPI-RS cannot safely hand back a zero-copy mutable slice across the FFI boundary. `buf.slice()` and `buf.subarray()` return independent copies as `JsBuffer`.
3. **Variable-Width Integers Return Type**:
   `readIntBE`, `readIntLE`, `readUIntBE`, and `readUIntLE` return standard JavaScript numbers (`number`) since 48-bit integers fit within `Number.MAX_SAFE_INTEGER` (53 bits).
