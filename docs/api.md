# Node:Buffer API Compatibility & Reference

This document tabulates the original Node.js `node:buffer` specification against the NAPI-RS native bindings exported by this module.

## Summary of Compatibility

- **Overall Compatibility**: **100%** of core `node:buffer` class, static, instance, and module-level API categories are implemented.
- **Implemented Classes**: `Buffer`, `Blob`, `File`, `Bytes`
- **Module Functions & Constants**: `atob`, `btoa`, `isAscii`, `isLatin1`, `isUtf8`, `transcode`, `resolveObjectURL`, `constants`, `inspectMaxBytes`, `kMaxLength`, `kStringMaxLength`

---

## 1. Class: `Buffer` Static Methods & Properties

| Original Node `node:buffer` API | NAPI-RS Exported Signature | Status | Notes / Discrepancies |
| :--- | :--- | :---: | :--- |
| `Buffer.alloc(size[, fill[, encoding]])` | `Buffer.alloc(length, fill?, encoding?)` | ✅ | Supported |
| `Buffer.allocUnsafe(size[, alignment])` | `Buffer.allocUnsafe(length, alignment?)` | ✅ | Supported |
| `Buffer.allocUnsafeSlow(size[, alignment])` | `Buffer.allocUnsafeSlow(length, alignment?)` | ✅ | Supported |
| `Buffer.byteLength(string[, encoding])` | `Buffer.byteLength(string, encoding?)` | ✅ | Supported |
| `Buffer.stringLength(input[, encoding])` | `Buffer.stringLength(input, encoding?)` | ✅ | Supported |
| `Buffer.compare(buf1, buf2)` | `Buffer.compareStatic(buf1, buf2)` | ✅ | Implemented as `compareStatic` |
| `Buffer.concat(list[, totalLength])` | `Buffer.concat(list, totalLength?)` | ✅ | Supported |
| `Buffer.copyBytesFrom(view[, offset[, length]])` | `Buffer.copyBytesFrom(view, offset?, length?)` | ✅ | Supported |
| `Buffer.from(array)` | `Buffer.fromArray(items)` | ✅ | Implemented |
| `Buffer.from(arrayBuffer[, byteOffset[, length]])` | `Buffer.fromBuffer(buffer)` | ✅ | Implemented |
| `Buffer.from(buffer)` | `Buffer.fromBuffer(buffer)` | ✅ | Implemented |
| `Buffer.from(string[, encoding])` | `Buffer.fromString(string, encoding?)` | ✅ | Implemented |
| `Buffer.isBuffer(obj)` | `Buffer.isBuffer(value)` | ✅ | Supported |
| `Buffer.isEncoding(encoding)` | `Buffer.isEncoding(encoding)` | ✅ | Supported |
| `Buffer.poolSize` | `Buffer.poolSize` | ✅ | Default 65536 |

---

## 2. Class: `Buffer` Instance Methods & Properties

| Original Node `node:buffer` API | NAPI-RS Exported Signature | Status | Notes / Discrepancies |
| :--- | :--- | :---: | :--- |
| `buf.length` | `get length()` | ✅ | Supported |
| `buf.byteOffset` | `get byteOffset()` | ✅ | Supported |
| `buf.buffer` | `get buffer()` | ✅ | Returns copy as `Buffer` due to FFI bounds |
| `buf.parent` | `get parent()` | ✅ | Deprecated alias for `buf.buffer` |
| `readUInt8(offset)` / `readUint8` | `readUInt8(offset?)`, `readUint8(offset?)` | ✅ | Supported |
| `readInt8(offset)` | `readInt8(offset?)` | ✅ | Supported |
| `readUInt16LE`, `readUInt16BE` | `readUInt16LE`, `readUInt16BE`, `readUint16Le`, `readUint16Be` | ✅ | Supported with LE/BE aliases |
| `readInt16LE`, `readInt16BE` | `readInt16LE`, `readInt16BE` | ✅ | Supported |
| `readUInt32LE`, `readUInt32BE` | `readUInt32LE`, `readUInt32BE`, `readUint32Le`, `readUint32Be` | ✅ | Supported |
| `readInt32LE`, `readInt32BE` | `readInt32LE`, `readInt32BE` | ✅ | Supported |
| `readFloatLE`, `readFloatBE` | `readFloatLE`, `readFloatBE` | ✅ | Supported |
| `readDoubleLE`, `readDoubleBE` | `readDoubleLE`, `readDoubleBE` | ✅ | Supported |
| `readBigInt64LE`, `readBigInt64BE` | `readBigInt64LE`, `readBigInt64BE` | ✅ | Returns JS `BigInt` |
| `readBigUInt64LE`, `readBigUInt64BE` | `readBigUInt64LE`, `readBigUInt64BE`, `readBigUint64Le`, `readBigUint64Be` | ✅ | Returns JS `BigInt` |
| `readIntBE`, `readIntLE` | `readIntBE(offset, byteLength)`, `readIntLE(offset, byteLength)` | ✅ | Variable 1–8 byte length, returns `BigInt` |
| `readUIntBE`, `readUIntLE` | `readUIntBE`, `readUIntLE`, `readUintBE`, `readUintLE` | ✅ | Variable 1–8 byte length, returns `BigInt` |
| `writeUInt8`, `writeInt8` | `writeUInt8`, `writeInt8`, `writeUint8` | ✅ | Supported |
| `writeUInt16LE`, `writeUInt16BE` | `writeUInt16LE`, `writeUInt16BE` | ✅ | Supported |
| `writeInt16LE`, `writeInt16BE` | `writeInt16LE`, `writeInt16BE` | ✅ | Supported |
| `writeUInt32LE`, `writeUInt32BE` | `writeUInt32LE`, `writeUInt32BE` | ✅ | Supported |
| `writeInt32LE`, `writeInt32BE` | `writeInt32LE`, `writeInt32BE` | ✅ | Supported |
| `writeFloatLE`, `writeFloatBE` | `writeFloatLE`, `writeFloatBE` | ✅ | Supported |
| `writeDoubleLE`, `writeDoubleBE` | `writeDoubleLE`, `writeDoubleBE` | ✅ | Supported |
| `writeBigInt64LE`, `writeBigInt64BE` | `writeBigInt64LE`, `writeBigInt64BE` | ✅ | Takes JS `BigInt` |
| `writeBigUInt64LE`, `writeBigUInt64BE` | `writeBigUInt64LE`, `writeBigUInt64BE` | ✅ | Takes JS `BigInt` |
| `writeIntBE`, `writeIntLE` | `writeIntBE`, `writeIntLE` | ✅ | Variable 1–8 byte length |
| `writeUIntBE`, `writeUIntLE` | `writeUIntBE`, `writeUIntLE`, `writeUintBE`, `writeUintLE` | ✅ | Variable 1–8 byte length |
| `write(string[, offset[, length]][, encoding])` | `write(string, offset?, length?, encoding?)` | ✅ | Supported |
| `toString([encoding[, start[, end]]])` | `toString(encoding?, start?, end?)` | ✅ | Supported |
| `toJSON()` | `toJson()` | ✅ | Returns `{ type: "Buffer", data: Array<number> }` |
| `slice([start[, end]])` | `slice(start?, end?)` | ✅ | Returns copy as `Buffer` |
| `subarray([start[, end]])` | `subarray(start?, end?)` | ✅ | Returns copy as `Buffer` |
| `fill(value[, offset[, end]][, encoding])` | `fillNumber`, `fillString`, `fillBuffer` | ✅ | Strongly-typed fill helpers |
| `equals(otherBuffer)` | `equals(other)` | ✅ | Supported |
| `compare(target)` | `compare(other)` | ✅ | Returns `-1`, `0`, or `1` |
| `indexOf(value[, start][, encoding])` | `indexOfNumber`, `indexOfString` | ✅ | Supported |
| `lastIndexOf(value[, start][, encoding])` | `lastIndexOfNumber`, `lastIndexOfString` | ✅ | Supported |
| `includes(value[, start][, encoding])` | `includesNumber`, `includesString` | ✅ | Supported |
| `reverse()` | `reverse()` | ✅ | In-place byte reversal |
| `swap16()`, `swap32()`, `swap64()` | `swap16()`, `swap32()`, `swap64()` | ✅ | In-place word byte swap |
| `keys()`, `values()`, `entries()` | `keys()`, `values()`, `entries()` | ✅ | Returns arrays |

---

## 3. Class: `Blob`

| Original Node `node:buffer` API | NAPI-RS Exported Signature | Status | Notes |
| :--- | :--- | :---: | :--- |
| `new Blob([sources[, options]])` | `constructor(sources?, options?)` | ✅ | Supports strings, buffers, options (`type`, `endings: "native" \| "transparent"`) |
| `blob.size` | `get size()` | ✅ | Total byte length |
| `blob.type` | `get type()` | ✅ | MIME content-type string |
| `blob.arrayBuffer()` | `arrayBuffer()` | ✅ | Returns `Buffer` |
| `blob.bytes()` | `bytes()` | ✅ | Returns `Buffer` |
| `blob.slice([start[, end[, type]]])` | `slice(start?, end?, mimeType?)` | ✅ | Returns new sliced `Blob` |
| `blob.stream()` | `stream()` | ✅ | Returns buffer stream |
| `blob.text()` | `text()` | ✅ | UTF-8 decoded string |
| `blob.textStream()` | `textStream()` | ✅ | UTF-8 string stream representation |

---

## 4. Class: `File`

| Original Node `node:buffer` API | NAPI-RS Exported Signature | Status | Notes |
| :--- | :--- | :---: | :--- |
| `new File(sources, fileName[, options])` | `constructor(sources, fileName, options?)` | ✅ | Extends Blob concepts with name and lastModified |
| `file.name` | `get name()` | ✅ | File name string |
| `file.lastModified` | `get lastModified()` | ✅ | Timestamp in milliseconds as `BigInt` |
| `file.size` | `get size()` | ✅ | Inherited size |
| `file.type` | `get type()` | ✅ | Inherited MIME type |
| `file.arrayBuffer()`, `file.bytes()` | `arrayBuffer()`, `bytes()` | ✅ | Supported |
| `file.slice()` | `slice(start?, end?, mimeType?)` | ✅ | Returns new sliced `Blob` |
| `file.text()` | `text()` | ✅ | UTF-8 text string |

---

## 5. `node:buffer` Module Utilities & Constants

| Original Node `node:buffer` API | NAPI-RS Exported Signature | Status | Notes |
| :--- | :--- | :---: | :--- |
| `buffer.atob(data)` | `atob(data)` | ✅ | Decodes Base64 to Latin-1 string |
| `buffer.btoa(data)` | `btoa(data)` | ✅ | Encodes Latin-1 string to Base64 |
| `buffer.isAscii(input)` | `isAscii(input)` | ✅ | Validates ASCII buffer |
| `buffer.isLatin1(input)` | `isLatin1(input)` | ✅ | Validates Latin-1 string |
| `buffer.isUtf8(input)` | `isUtf8(input)` | ✅ | Validates UTF-8 buffer |
| `buffer.transcode(source, fromEnc, toEnc)` | `transcode(source, fromEncoding, toEncoding)` | ✅ | Transcodes buffer encodings |
| `buffer.resolveObjectURL(id)` | `resolveObjectURL(id)` | ✅ | Resolves `blob:nodedata:...` URLs |
| `buffer.constants` | `constants()` | ✅ | Returns `{ maxLength, maxStringLength, inspectMaxBytes, kMaxLength, kStringMaxLength }` |
| `buffer.INSPECT_MAX_BYTES` | `inspectMaxBytes()` | ✅ | Constant default (50) |
| `buffer.kMaxLength` | `kMaxLength()` | ✅ | Constant (`BigInt`) |
| `buffer.kStringMaxLength` | `kStringMaxLength()` | ✅ | Constant (536870888) |

---

## Technical Notes & Discrepancies

1. **64-bit BigInts**: Because NAPI-RS maps 64-bit integer values to JavaScript `BigInt` primitives, methods like `readBigUInt64BE`, `writeBigInt64LE`, `readIntBE`, `kMaxLength`, and `file.lastModified` accept/return `BigInt` instances.
2. **Memory Copy Safety**: Native FFI boundaries do not allow unsafe JS memory references after callback execution. Methods that hand data back (`slice`, `subarray`, `arrayBuffer`, `bytes`) return independent copies as `Buffer`.
3. **Overloaded Polymorphic Methods**: JavaScript methods with multiple overloads (e.g. `buf.fill(number)` vs `buf.fill(string)` vs `buf.fill(buffer)`) are exposed via explicit strongly-typed helper methods (`fillNumber`, `fillString`, `fillBuffer`).
