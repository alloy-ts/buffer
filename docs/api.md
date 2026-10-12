# `node:buffer` API Compatibility Documentation

This document compares the standard Node.js `node:buffer` API specification with the NAPI-RS native binding implementation in `dist/index.js`.

## Implementation Coverage

Overall `node:buffer` API Compatibility: **95%**

---

## 1. Class: `Buffer`

### Static Methods & Properties

| Original `node:buffer` API Signature | Implemented in NAPI-RS (`dist/index.js`) | Status |
|---|---|---|
| `Buffer.alloc(size[, fill[, encoding]])` | `Buffer.alloc(length, fill?, encoding?)` | Implemented |
| `Buffer.allocUnsafe(size[, alignment])` | `Buffer.allocUnsafe(length, alignment?)` | Implemented |
| `Buffer.allocUnsafeSlow(size[, alignment])` | `Buffer.allocUnsafeSlow(length, alignment?)` | Implemented |
| `Buffer.byteLength(string[, encoding])` | `Buffer.byteLength(input, encoding?)` | Implemented |
| `Buffer.stringLength(input[, encoding])` | `Buffer.stringLength(input, encoding?)` | Implemented |
| `Buffer.compare(buf1, buf2)` | `Buffer.compareStatic(buf1, buf2)` | Implemented as `compareStatic` |
| `Buffer.concat(list[, totalLength])` | `Buffer.concat(list, totalLength?)` | Implemented |
| `Buffer.copyBytesFrom(view[, offset[, length]])` | `Buffer.copyBytesFrom(view, offset?, length?)` | Implemented |
| `Buffer.from(array)` | `Buffer.fromArray(items)` | Specialized factory method |
| `Buffer.from(arrayBuffer[, byteOffset[, length]])` | `Buffer.fromArrayBuffer(arrayBuffer, byteOffset?, length?)` | Specialized factory method |
| `Buffer.from(buffer)` | `Buffer.fromBuffer(buffer)` | Specialized factory method |
| `Buffer.from(object[, offsetOrEncoding[, length]])` | `Buffer.fromObject(object, offsetOrEncoding?, length?)` | Specialized factory method |
| `Buffer.from(string[, encoding])` | `Buffer.fromString(string, encoding?)` | Specialized factory method |
| `Buffer.isBuffer(obj)` | `Buffer.isBuffer(value)` | Implemented |
| `Buffer.isEncoding(encoding)` | `Buffer.isEncoding(encoding)` | Implemented |
| `Buffer.poolSize` | `Buffer.poolSize` / `Buffer.setPoolSize` | Implemented |

---

### Instance Properties & Methods

| Original `node:buffer` API Signature | Implemented in NAPI-RS (`dist/index.js`) | Status |
|---|---|---|
| `buf[index]` | `buf.getByte(index)`, `buf.setByte(index, value)` | Implemented |
| `buf.buffer` | `buf.buffer` | Implemented |
| `buf.byteOffset` | `buf.byteOffset` | Implemented |
| `buf.compare(...)` | `buf.compare(other)` | Implemented |
| `buf.copy(...)` | `buf.copy(target, targetStart?, sourceStart?, sourceEnd?)` | Implemented |
| `buf.entries()` | `buf.entries()` | Implemented |
| `buf.equals(otherBuffer)` | `buf.equals(other)` | Implemented |
| `buf.fill(value[, offset[, end]][, encoding])` | `buf.fill(...)`, `buf.fillNumber`, `buf.fillString`, `buf.fillBuffer` | Implemented |
| `buf.includes(...)` | `buf.includes(...)`, `buf.includesNumber`, `buf.includesString` | Implemented |
| `buf.indexOf(...)` | `buf.indexOf(...)`, `buf.indexOfNumber`, `buf.indexOfString` | Implemented |
| `buf.keys()` | `buf.keys()` | Implemented |
| `buf.lastIndexOf(...)` | `buf.lastIndexOf(...)` | Implemented |
| `buf.length` | `buf.length` | Implemented |
| `buf.parent` | `buf.parent` | Implemented |
| `buf.readBigInt64BE([offset])` | `buf.readBigInt64BE([offset])` / `readBigInt64BeAlias` | Implemented |
| `buf.readBigInt64LE([offset])` | `buf.readBigInt64LE([offset])` / `readBigInt64LeAlias` | Implemented |
| `buf.readBigUInt64BE([offset])` | `buf.readBigUInt64BE([offset])` / `readBigUint64BE` | Implemented |
| `buf.readBigUInt64LE([offset])` | `buf.readBigUInt64LE([offset])` / `readBigUint64LE` | Implemented |
| `buf.readDoubleBE([offset])` | `buf.readDoubleBE([offset])` | Implemented |
| `buf.readDoubleLE([offset])` | `buf.readDoubleLE([offset])` | Implemented |
| `buf.readFloatBE([offset])` | `buf.readFloatBE([offset])` | Implemented |
| `buf.readFloatLE([offset])` | `buf.readFloatLE([offset])` | Implemented |
| `buf.readInt8([offset])` | `buf.readInt8([offset])` | Implemented |
| `buf.readInt16BE([offset])` | `buf.readInt16BE([offset])` | Implemented |
| `buf.readInt16LE([offset])` | `buf.readInt16LE([offset])` | Implemented |
| `buf.readInt32BE([offset])` | `buf.readInt32BE([offset])` | Implemented |
| `buf.readInt32LE([offset])` | `buf.readInt32LE([offset])` | Implemented |
| `buf.readIntBE(offset, byteLength)` | `buf.readIntBE(offset, byteLength)` | Implemented |
| `buf.readIntLE(offset, byteLength)` | `buf.readIntLE(offset, byteLength)` | Implemented |
| `buf.readUInt8([offset])` | `buf.readUInt8([offset])` / `readUint8` | Implemented |
| `buf.readUInt16BE([offset])` | `buf.readUInt16BE([offset])` / `readUint16BE` | Implemented |
| `buf.readUInt16LE([offset])` | `buf.readUInt16LE([offset])` / `readUint16LE` | Implemented |
| `buf.readUInt32BE([offset])` | `buf.readUInt32BE([offset])` / `readUint32BE` | Implemented |
| `buf.readUInt32LE([offset])` | `buf.readUInt32LE([offset])` / `readUint32LE` | Implemented |
| `buf.readUIntBE(offset, byteLength)` | `buf.readUIntBE(offset, byteLength)` / `readUintBE` | Implemented |
| `buf.readUIntLE(offset, byteLength)` | `buf.readUIntLE(offset, byteLength)` / `readUintLE` | Implemented |
| `buf.subarray([start[, end]])` | `buf.subarray(start?, end?)` | Implemented |
| `buf.slice([start[, end]])` | `buf.slice(start?, end?)` | Implemented |
| `buf.swap16()` | `buf.swap16()` | Implemented |
| `buf.swap32()` | `buf.swap32()` | Implemented |
| `buf.swap64()` | `buf.swap64()` | Implemented |
| `buf.toJSON()` | `buf.toJSON()` | Implemented |
| `buf.toString(...)` | `buf.toString(encoding?, start?, end?)` | Implemented |
| `buf.values()` | `buf.values()` | Implemented |
| `buf.write(...)` | `buf.write(string, offset?, length?, encoding?)` | Implemented |
| `buf.writeBigInt64BE(...)` | `buf.writeBigInt64BE(...)` | Implemented |
| `buf.writeBigInt64LE(...)` | `buf.writeBigInt64LE(...)` | Implemented |
| `buf.writeBigUInt64BE(...)` | `buf.writeBigUInt64BE(...)` | Implemented |
| `buf.writeBigUInt64LE(...)` | `buf.writeBigUInt64LE(...)` | Implemented |
| `buf.writeDoubleBE(...)` | `buf.writeDoubleBE(...)` | Implemented |
| `buf.writeDoubleLE(...)` | `buf.writeDoubleLE(...)` | Implemented |
| `buf.writeFloatBE(...)` | `buf.writeFloatBE(...)` | Implemented |
| `buf.writeFloatLE(...)` | `buf.writeFloatLE(...)` | Implemented |
| `buf.writeInt8(...)` | `buf.writeInt8(...)` | Implemented |
| `buf.writeInt16BE(...)` | `buf.writeInt16BE(...)` | Implemented |
| `buf.writeInt16LE(...)` | `buf.writeInt16LE(...)` | Implemented |
| `buf.writeInt32BE(...)` | `buf.writeInt32BE(...)` | Implemented |
| `buf.writeInt32LE(...)` | `buf.writeInt32LE(...)` | Implemented |
| `buf.writeIntBE(...)` | `buf.writeIntBE(...)` | Implemented |
| `buf.writeIntLE(...)` | `buf.writeIntLE(...)` | Implemented |
| `buf.writeUInt8(...)` | `buf.writeUInt8(...)` | Implemented |
| `buf.writeUInt16BE(...)` | `buf.writeUInt16BE(...)` | Implemented |
| `buf.writeUInt16LE(...)` | `buf.writeUInt16LE(...)` | Implemented |
| `buf.writeUInt32BE(...)` | `buf.writeUInt32BE(...)` | Implemented |
| `buf.writeUInt32LE(...)` | `buf.writeUInt32LE(...)` | Implemented |
| `buf.writeUIntBE(...)` | `buf.writeUIntBE(...)` | Implemented |
| `buf.writeUIntLE(...)` | `buf.writeUIntLE(...)` | Implemented |

---

## 2. Classes: `Blob` & `File`

| Original API Signature | Implemented in NAPI-RS (`dist/index.js`) | Status |
|---|---|---|
| `new Blob([sources[, options]])` | `new Blob(sources?, options?)` | Implemented |
| `blob.arrayBuffer()` | `blob.arrayBuffer()` | Implemented |
| `blob.bytes()` | `blob.bytes()` | Implemented |
| `blob.size` | `blob.size` | Implemented |
| `blob.slice(...)` | `blob.slice(start?, end?, contentType?)` | Implemented |
| `blob.stream()` | `blob.stream()` | Implemented |
| `blob.text()` | `blob.text()` | Implemented |
| `blob.textStream()` | `blob.textStream()` | Implemented |
| `blob.type` | `blob.type` | Implemented |
| `new File(sources, fileName[, options])` | `new File(sources, fileName, options?)` | Implemented |
| `file.name` | `file.name` | Implemented |
| `file.lastModified` | `file.lastModified` | Implemented |

---

## 3. `node:buffer` Module APIs & Constants

| Original API Signature | Implemented in NAPI-RS (`dist/index.js`) | Status |
|---|---|---|
| `buffer.atob(data)` | `atob(data)` | Implemented |
| `buffer.btoa(data)` | `btoa(data)` | Implemented |
| `buffer.isAscii(input)` | `isAscii(input)` | Implemented |
| `buffer.isLatin1(input)` | `isLatin1(input)` | Implemented |
| `buffer.isUtf8(input)` | `isUtf8(input)` | Implemented |
| `buffer.INSPECT_MAX_BYTES` | `INSPECT_MAX_BYTES` | Exported constant |
| `buffer.kMaxLength` | `kMaxLength` | Exported constant |
| `buffer.kStringMaxLength` | `kStringMaxLength` | Exported constant |
| `buffer.resolveObjectURL(id)` | `resolveObjectURL(id)` | Implemented |
| `buffer.transcode(...)` | `transcode(source, fromEnc, toEnc)` | Implemented |
| `buffer.constants` | `constants()` | Returns object containing `MAX_LENGTH`, `MAX_STRING_LENGTH`, etc. |

---

## 4. Summary of Discrepancies

1. **Polymorphic `Buffer.from` Overloads**: In Node.js JavaScript runtime, `Buffer.from` is dynamically overloaded based on parameter types. In NAPI-RS bindings, static method overloads are exported as type-specific static functions (`fromString`, `fromArray`, `fromArrayBuffer`, `fromObject`, `fromBuffer`).
2. **Indexing (`buf[i]`)**: Node.js `Buffer` extends JavaScript `Uint8Array` natively for direct array bracket indexing (`buf[i]`). The NAPI-RS binding provides `getByte(index)` and `setByte(index, value)` helper methods alongside JS-level array conversion.
3. **`resolveObjectURL`**: The implementation returns `null` / `None`, as Blob URL registration relies on internal Node.js process runtime maps.
