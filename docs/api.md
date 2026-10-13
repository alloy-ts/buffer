# `node:buffer` API Compatibility & Discrepancy Report

This document reviews the current `@alloy-ts/buffer` implementation (`src/buffer.rs` NAPI class) against the standard Node.js [`node:buffer`](https://nodejs.org/api/buffer.html) API and the requested `buffer.Buffer` specification.

## Architectural Overview & Key Discrepancies

Unlike standard Node.js where `Buffer` inherits directly from `Uint8Array` (`class Buffer extends Uint8Array`), `@alloy-ts/buffer` exports a standalone NAPI class (`AlloyBuffer`) implemented in Rust. This design choice introduces several key structural discrepancies:

1. **Not a `Uint8Array` Subclass:** `buf instanceof Uint8Array` returns `false`.
2. **Indexed Access (`buf[i]`):** Array index access (e.g. `buf[0]`) returns `undefined` on `AlloyBuffer` instances.
3. **Missing TypedArray Methods:** Standard `Uint8Array` prototype methods (`at`, `copyWithin`, `every`, `filter`, `find`, `findIndex`, `findLast`, `findLastIndex`, `forEach`, `join`, `keys`, `map`, `reduce`, `reduceRight`, `set`, `some`, `sort`, `toLocaleString`, `toReversed`, `toSorted`, `valueOf`, `values`, `with`, `[Symbol.iterator]`, `[Symbol.toStringTag]`) are not implemented on the Rust class.
4. **Mutating Copy (`buf.copy`):** NAPI parameter pass-by-ref semantics prevent `buf.copy(target, ...)` from mutating `target` in-place when called on `AlloyBuffer` instances from JavaScript.
5. **Method Chaining (`buf.fill`):** `fill()` returns `void` (`Result<()>`) instead of `this`, breaking chained method calls (e.g., `buf.fill('a').toString()`).
6. **Subarray / Slice Memory Sharing:** `buf.subarray()` returns a cloned `AlloyBuffer` rather than a zero-copy view sharing the underlying `ArrayBuffer`.
7. **Helper Interoperability:** Module helpers like `isAscii`, `isUtf8`, and `transcode` check for native JavaScript `Uint8Array`/`ArrayBuffer` objects and throw when passed an `AlloyBuffer` instance.

---

## Detailed Method & Property Matrix

Below is the complete discrepancy matrix for all `Buffer` methods and properties.

| Method / Property                       | Status | Discrepancy / Notes                                                                                |
| :-------------------------------------- | :----: | :------------------------------------------------------------------------------------------------- |
| `[Symbol.iterator]`                     |   ❌   | Missing. Iteration via `for...of` or `Array.from(buf)` is not supported.                           |
| `[Symbol.toStringTag]`                  |   ❌   | Missing. Does not return `'Uint8Array'`.                                                           |
| `at(index)`                             |   ❌   | Missing. Negative index lookup is not supported.                                                   |
| `buffer`                                |   ❌   | Missing. No `ArrayBuffer` getter reference.                                                        |
| `byteLength`                            |   ✅   | Implemented as getter (`get byteLength`).                                                          |
| `byteOffset`                            |   ❌   | Missing. Always 0 in Node's Uint8Array views.                                                      |
| `BYTES_PER_ELEMENT`                     |   ❌   | Missing. (Value should be `1`).                                                                    |
| `compare`                               |   ✅   | Implemented (`compare`).                                                                           |
| `copy`                                  |   ⚠️   | Discrepancy: Implemented, but does not mutate `target` in-place due to NAPI pass-by-ref semantics. |
| `copyWithin`                            |   ❌   | Missing.                                                                                           |
| `entries`                               |   ❌   | Missing.                                                                                           |
| `equals`                                |   ✅   | Implemented (`equals`).                                                                            |
| `every`                                 |   ❌   | Missing.                                                                                           |
| `fill`                                  |   ⚠️   | Discrepancy: Implemented, but returns `void` instead of `this`, breaking method chaining.          |
| `filter`                                |   ❌   | Missing.                                                                                           |
| `find`                                  |   ❌   | Missing.                                                                                           |
| `findIndex`                             |   ❌   | Missing.                                                                                           |
| `findLast`                              |   ❌   | Missing.                                                                                           |
| `findLastIndex`                         |   ❌   | Missing.                                                                                           |
| `forEach`                               |   ❌   | Missing.                                                                                           |
| `includes`                              |   ✅   | Implemented (`includes`).                                                                          |
| `indexOf`                               |   ✅   | Implemented (`indexOf`).                                                                           |
| `join`                                  |   ❌   | Missing.                                                                                           |
| `keys`                                  |   ❌   | Missing.                                                                                           |
| `lastIndexOf`                           |   ✅   | Implemented (`lastIndexOf`).                                                                       |
| `length`                                |   ✅   | Implemented as getter (`get length`).                                                              |
| `map`                                   |   ❌   | Missing.                                                                                           |
| `readBigInt64BE`                        |   ✅   | Implemented.                                                                                       |
| `readBigInt64LE`                        |   ✅   | Implemented.                                                                                       |
| `readBigUint64BE` / `readBigUInt64BE`   |   ✅   | Implemented.                                                                                       |
| `readBigUint64LE` / `readBigUInt64LE`   |   ✅   | Implemented.                                                                                       |
| `readDoubleBE`                          |   ✅   | Implemented.                                                                                       |
| `readDoubleLE`                          |   ✅   | Implemented.                                                                                       |
| `readFloatBE`                           |   ✅   | Implemented.                                                                                       |
| `readFloatLE`                           |   ✅   | Implemented.                                                                                       |
| `readInt16BE`                           |   ✅   | Implemented.                                                                                       |
| `readInt16LE`                           |   ✅   | Implemented.                                                                                       |
| `readInt32BE`                           |   ✅   | Implemented.                                                                                       |
| `readInt32LE`                           |   ✅   | Implemented.                                                                                       |
| `readInt8`                              |   ✅   | Implemented.                                                                                       |
| `readIntBE`                             |   ✅   | Implemented.                                                                                       |
| `readIntLE`                             |   ✅   | Implemented.                                                                                       |
| `readUint16BE` / `readUInt16BE`         |   ✅   | Implemented.                                                                                       |
| `readUint16LE` / `readUInt16LE`         |   ✅   | Implemented.                                                                                       |
| `readUint32BE` / `readUInt32BE`         |   ✅   | Implemented.                                                                                       |
| `readUint32LE` / `readUInt32LE`         |   ✅   | Implemented.                                                                                       |
| `readUint8` / `readUInt8`               |   ✅   | Implemented.                                                                                       |
| `readUintBE` / `readUIntBE`             |   ✅   | Implemented.                                                                                       |
| `readUintLE` / `readUIntLE`             |   ✅   | Implemented.                                                                                       |
| `reduce`                                |   ❌   | Missing.                                                                                           |
| `reduceRight`                           |   ❌   | Missing.                                                                                           |
| `reverse`                               |   ✅   | Implemented (`reverse`).                                                                           |
| `set`                                   |   ❌   | Missing.                                                                                           |
| `setFromBase64`                         |   ✅   | Implemented (`setFromBase64`).                                                                     |
| `setFromHex`                            |   ✅   | Implemented (`setFromHex`).                                                                        |
| `some`                                  |   ❌   | Missing.                                                                                           |
| `sort`                                  |   ❌   | Missing.                                                                                           |
| `subarray`                              |   ⚠️   | Discrepancy: Returns a copied `AlloyBuffer` rather than a zero-copy memory view.                   |
| `swap16`                                |   ✅   | Implemented.                                                                                       |
| `swap32`                                |   ✅   | Implemented.                                                                                       |
| `swap64`                                |   ✅   | Implemented.                                                                                       |
| `toBase64`                              |   ✅   | Implemented.                                                                                       |
| `toHex`                                 |   ✅   | Implemented.                                                                                       |
| `toJSON`                                |   ✅   | Implemented.                                                                                       |
| `toLocaleString`                        |   ❌   | Missing.                                                                                           |
| `toReversed`                            |   ❌   | Missing.                                                                                           |
| `toSorted`                              |   ❌   | Missing.                                                                                           |
| `toString`                              |   ✅   | Implemented (`toString`).                                                                          |
| `valueOf`                               |   ❌   | Missing.                                                                                           |
| `values`                                |   ❌   | Missing.                                                                                           |
| `with`                                  |   ❌   | Missing.                                                                                           |
| `write`                                 |   ✅   | Implemented (`write`).                                                                             |
| `writeBigInt64BE`                       |   ✅   | Implemented.                                                                                       |
| `writeBigInt64LE`                       |   ✅   | Implemented.                                                                                       |
| `writeBigUint64BE` / `writeBigUInt64BE` |   ✅   | Implemented.                                                                                       |
| `writeBigUint64LE` / `writeBigUInt64LE` |   ✅   | Implemented.                                                                                       |
| `writeDoubleBE`                         |   ✅   | Implemented.                                                                                       |
| `writeDoubleLE`                         |   ✅   | Implemented.                                                                                       |
| `writeFloatBE`                          |   ✅   | Implemented.                                                                                       |
| `writeFloatLE`                          |   ✅   | Implemented.                                                                                       |
| `writeInt16BE`                          |   ✅   | Implemented.                                                                                       |
| `writeInt16LE`                          |   ✅   | Implemented.                                                                                       |
| `writeInt32BE`                          |   ✅   | Implemented.                                                                                       |
| `writeInt32LE`                          |   ✅   | Implemented.                                                                                       |
| `writeInt8`                             |   ✅   | Implemented.                                                                                       |
| `writeIntBE`                            |   ✅   | Implemented.                                                                                       |
| `writeIntLE`                            |   ✅   | Implemented.                                                                                       |
| `writeUint16BE` / `writeUInt16BE`       |   ✅   | Implemented.                                                                                       |
| `writeUint16LE` / `writeUInt16LE`       |   ✅   | Implemented.                                                                                       |
| `writeUint32BE` / `writeUInt32BE`       |   ✅   | Implemented.                                                                                       |
| `writeUint32LE` / `writeUInt32LE`       |   ✅   | Implemented.                                                                                       |
| `writeUint8` / `writeUInt8`             |   ✅   | Implemented.                                                                                       |
| `writeUintBE` / `writeUIntBE`           |   ✅   | Implemented.                                                                                       |
| `writeUintLE` / `writeUIntLE`           |   ✅   | Implemented.                                                                                       |

---

## Other Module Features

| Feature / Class | Status | Notes |
| :--- | :---: | : |
| `Blob` | ⚠️ | Implemented except `blob.stream()`. |
| `File` | ⚠️ | Implemented except `file.stream()`. |
| `atob` / `btoa` | ✅ | Implemented. |
| `isAscii` / `isUtf8` | ⚠️ | Implemented, but throws if passed `@alloy-ts/buffer`'s own `Buffer` instance. |
| `transcode` | ⚠️ | Implemented, but throws if passed `@alloy-ts/buffer`'s own `Buffer` instance. |
| `constants` | ✅ | `MAX_LENGTH`, `MAX_STRING_LENGTH` available. |
| `resolveObjectURL` | ⚠️ | Stub — throws "blob: URLs are not supported". |
