export type BufferEncoding =
  | "ascii"
  | "utf8"
  | "utf-8"
  | "utf16le"
  | "utf-16le"
  | "ucs2"
  | "ucs-2"
  | "base64"
  | "base64url"
  | "latin1"
  | "binary"
  | "hex";

export type TranscodeEncoding =
  | "ascii"
  | "utf8"
  | "utf-8"
  | "utf16le"
  | "utf-16le"
  | "ucs2"
  | "ucs-2"
  | "latin1"
  | "binary";

export interface BlobOptions {
  endings?: "transparent" | "native";
  type?: string;
}

export interface FileOptions {
  endings?: "transparent" | "native";
  type?: string;
  lastModified?: number;
}

export type WithImplicitCoercion<T> =
  | T
  | { valueOf(): T }
  | (T extends string ? { [Symbol.toPrimitive](hint: "string"): T } : never);

export type ImplicitArrayBuffer<T extends WithImplicitCoercion<ArrayBufferLike>> = T extends {
  valueOf(): infer V extends ArrayBufferLike;
}
  ? V
  : T;

export const MAX_LENGTH = 0x7fffffff;
export const MAX_STRING_LENGTH = 0x1fffffe8;

export const constants = {
  MAX_LENGTH,
  MAX_STRING_LENGTH,
};

export const INSPECT_MAX_BYTES = 50;
export const kMaxLength = MAX_LENGTH;
export const kStringMaxLength = MAX_STRING_LENGTH;

const B64_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const B64URL_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

function normalizeEncoding(encoding?: string): BufferEncoding {
  if (!encoding) return "utf8";
  const lower = encoding.toLowerCase();
  switch (lower) {
    case "utf8":
    case "utf-8":
      return "utf8";
    case "ascii":
      return "ascii";
    case "latin1":
    case "binary":
      return "latin1";
    case "utf16le":
    case "utf-16le":
    case "ucs2":
    case "ucs-2":
      return "utf16le";
    case "hex":
      return "hex";
    case "base64":
      return "base64";
    case "base64url":
      return "base64url";
    default:
      return "utf8";
  }
}

function stringToBytes(str: string, encoding?: string): Uint8Array {
  const enc = normalizeEncoding(encoding);
  switch (enc) {
    case "utf8":
      return new TextEncoder().encode(str);
    case "ascii": {
      const buf = new Uint8Array(str.length);
      for (let i = 0; i < str.length; i++) {
        const code = str.charCodeAt(i);
        buf[i] = code > 127 ? 0x3f : code & 0x7f;
      }
      return buf;
    }
    case "latin1": {
      const buf = new Uint8Array(str.length);
      for (let i = 0; i < str.length; i++) {
        buf[i] = str.charCodeAt(i) & 0xff;
      }
      return buf;
    }
    case "utf16le": {
      const buf = new Uint8Array(str.length * 2);
      for (let i = 0; i < str.length; i++) {
        const code = str.charCodeAt(i);
        buf[i * 2] = code & 0xff;
        buf[i * 2 + 1] = (code >> 8) & 0xff;
      }
      return buf;
    }
    case "hex": {
      const cleaned = str.replace(/[^0-9a-fA-F]/g, "");
      const len = Math.floor(cleaned.length / 2);
      const buf = new Uint8Array(len);
      for (let i = 0; i < len; i++) {
        buf[i] = Number.parseInt(cleaned.substring(i * 2, i * 2 + 2), 16);
      }
      return buf;
    }
    case "base64":
    case "base64url":
      return decodeBase64(str, enc === "base64url");
    default:
      return new TextEncoder().encode(str);
  }
}

function bytesToString(bytes: Uint8Array, encoding?: string): string {
  const enc = normalizeEncoding(encoding);
  switch (enc) {
    case "utf8":
      return new TextDecoder("utf-8").decode(bytes);
    case "ascii": {
      let res = "";
      for (let i = 0; i < bytes.length; i++) {
        res += String.fromCharCode(bytes[i]! & 0x7f);
      }
      return res;
    }
    case "latin1": {
      let res = "";
      for (let i = 0; i < bytes.length; i++) {
        res += String.fromCharCode(bytes[i]!);
      }
      return res;
    }
    case "utf16le": {
      let res = "";
      for (let i = 0; i < bytes.length - 1; i += 2) {
        res += String.fromCharCode(bytes[i]! | (bytes[i + 1]! << 8));
      }
      return res;
    }
    case "hex": {
      let res = "";
      for (let i = 0; i < bytes.length; i++) {
        res += bytes[i]!.toString(16).padStart(2, "0");
      }
      return res;
    }
    case "base64":
      return encodeBase64(bytes, false);
    case "base64url":
      return encodeBase64(bytes, true);
    default:
      return new TextDecoder("utf-8").decode(bytes);
  }
}

function encodeBase64(bytes: Uint8Array, url: boolean): string {
  const chars = url ? B64URL_CHARS : B64_CHARS;
  let res = "";
  let i = 0;
  for (; i + 2 < bytes.length; i += 3) {
    const n = (bytes[i]! << 16) | (bytes[i + 1]! << 8) | bytes[i + 2]!;
    res += chars[(n >> 18) & 63]! + chars[(n >> 12) & 63]! + chars[(n >> 6) & 63]! + chars[n & 63]!;
  }
  const rem = bytes.length - i;
  if (rem === 1) {
    const n = bytes[i]! << 16;
    res += chars[(n >> 18) & 63]! + chars[(n >> 12) & 63]!;
    if (!url) res += "==";
  } else if (rem === 2) {
    const n = (bytes[i]! << 16) | (bytes[i + 1]! << 8);
    res += chars[(n >> 18) & 63]! + chars[(n >> 12) & 63]! + chars[(n >> 6) & 63]!;
    if (!url) res += "=";
  }
  return res;
}

function decodeBase64(str: string, url: boolean): Uint8Array {
  let cleaned = str.replace(/[\s=]/g, "");
  if (url) {
    cleaned = cleaned.replace(/-/g, "+").replace(/_/g, "/");
  } else {
    cleaned = cleaned.replace(/[^A-Za-z0-9+/]/g, "");
  }
  const out: number[] = [];
  let i = 0;
  for (; i + 3 < cleaned.length; i += 4) {
    const b1 = B64_CHARS.indexOf(cleaned[i]!);
    const b2 = B64_CHARS.indexOf(cleaned[i + 1]!);
    const b3 = B64_CHARS.indexOf(cleaned[i + 2]!);
    const b4 = B64_CHARS.indexOf(cleaned[i + 3]!);
    if (b1 === -1 || b2 === -1 || b3 === -1 || b4 === -1) continue;
    const n = (b1 << 18) | (b2 << 12) | (b3 << 6) | b4;
    out.push((n >> 16) & 255, (n >> 8) & 255, n & 255);
  }
  const rem = cleaned.length - i;
  if (rem === 2) {
    const b1 = B64_CHARS.indexOf(cleaned[i]!);
    const b2 = B64_CHARS.indexOf(cleaned[i + 1]!);
    if (b1 !== -1 && b2 !== -1) {
      const n = (b1 << 18) | (b2 << 12);
      out.push((n >> 16) & 255);
    }
  } else if (rem === 3) {
    const b1 = B64_CHARS.indexOf(cleaned[i]!);
    const b2 = B64_CHARS.indexOf(cleaned[i + 1]!);
    const b3 = B64_CHARS.indexOf(cleaned[i + 2]!);
    if (b1 !== -1 && b2 !== -1 && b3 !== -1) {
      const n = (b1 << 18) | (b2 << 12) | (b3 << 6);
      out.push((n >> 16) & 255, (n >> 8) & 255);
    }
  }
  return new Uint8Array(out);
}

function stringToBuffer(str: string, encoding?: string): Buffer {
  const bytes = stringToBytes(str, encoding);
  const buf = new Buffer(bytes.length);
  buf.set(bytes);
  return buf;
}

function searchBuffer(
  buf: Uint8Array,
  value: string | number | Uint8Array,
  byteOffset: number,
  encoding: string,
  reverse: boolean,
): number {
  let needle: Uint8Array;
  if (typeof value === "number") {
    needle = new Uint8Array([value & 255]);
  } else if (typeof value === "string") {
    needle = stringToBytes(value, encoding);
  } else if (value instanceof Uint8Array) {
    needle = value;
  } else {
    throw new TypeError("value must be a string, number, or Uint8Array/Buffer");
  }

  if (needle.length === 0) {
    if (reverse) {
      return Math.min(byteOffset, buf.length);
    }
    return Math.min(byteOffset, buf.length);
  }

  if (buf.length === 0 || needle.length > buf.length) return -1;

  let start = byteOffset;
  if (Number.isNaN(start)) start = reverse ? buf.length - 1 : 0;
  if (start < 0) start = buf.length + start;

  if (reverse) {
    start = Math.min(start, buf.length - needle.length);
    for (let i = start; i >= 0; i--) {
      let match = true;
      for (let j = 0; j < needle.length; j++) {
        if (buf[i + j] !== needle[j]) {
          match = false;
          break;
        }
      }
      if (match) return i;
    }
  } else {
    start = Math.max(0, start);
    const limit = buf.length - needle.length;
    for (let i = start; i <= limit; i++) {
      let match = true;
      for (let j = 0; j < needle.length; j++) {
        if (buf[i + j] !== needle[j]) {
          match = false;
          break;
        }
      }
      if (match) return i;
    }
  }
  return -1;
}

export class Buffer extends Uint8Array {
  static poolSize = 8192;

  constructor(
    arg: number | ArrayLike<number> | ArrayBufferLike,
    byteOffset?: number,
    length?: number,
  ) {
    if (typeof arg === "number") {
      super(arg);
    } else if (
      arg instanceof ArrayBuffer ||
      (typeof SharedArrayBuffer !== "undefined" && arg instanceof SharedArrayBuffer)
    ) {
      super(arg as ArrayBuffer, byteOffset, length);
    } else {
      super(arg as ArrayLike<number>);
    }
  }

  static alloc(
    size: number,
    fill?: string | Uint8Array | number,
    encoding?: BufferEncoding,
  ): Buffer {
    if (typeof size !== "number" || size < 0 || size > MAX_LENGTH) {
      throw new RangeError(`The value of "size" is out of range. Received ${size}`);
    }
    const buf = new Buffer(size);
    if (fill !== undefined) {
      buf.fill(fill, 0, size, encoding);
    }
    return buf;
  }

  static allocUnsafe(size: number): Buffer {
    return Buffer.alloc(size);
  }

  static allocUnsafeSlow(size: number): Buffer {
    return Buffer.alloc(size);
  }

  static override from(value: any, encodingOrOffset?: any, length?: any): Buffer {
    if (typeof value === "string") {
      return stringToBuffer(value, encodingOrOffset);
    }
    if (
      value instanceof ArrayBuffer ||
      (typeof SharedArrayBuffer !== "undefined" && value instanceof SharedArrayBuffer)
    ) {
      return new Buffer(value, encodingOrOffset, length);
    }
    if (ArrayBuffer.isView(value)) {
      return Buffer.copyBytesFrom(value as ArrayBufferView);
    }
    if (Array.isArray(value) || (value && typeof value === "object" && "length" in value)) {
      const array = Array.from(value, (v: any) => Number(v) & 255);
      return new Buffer(array);
    }
    if (value && typeof value === "object") {
      if (Symbol.toPrimitive in value) {
        const prim = value[Symbol.toPrimitive]("string");
        if (typeof prim === "string") {
          return stringToBuffer(prim, encodingOrOffset);
        }
      }
      if ("valueOf" in value) {
        const val = value.valueOf();
        if (val !== value) {
          return Buffer.from(val, encodingOrOffset, length);
        }
      }
    }
    throw new TypeError(
      "The first argument must be of type string or an instance of Buffer, ArrayBuffer, or Array or Array-like Object.",
    );
  }

  static override of(...items: number[]): Buffer {
    return new Buffer(items.map((x) => x & 255));
  }

  static concat(list: readonly Uint8Array[], totalLength?: number): Buffer {
    if (!Array.isArray(list)) {
      throw new TypeError(`The "list" argument must be an instance of Array.`);
    }
    if (list.length === 0) return new Buffer(0);
    let calculatedLength = 0;
    for (const buf of list) {
      calculatedLength += buf.length;
    }
    const actualLength =
      totalLength !== undefined ? Math.max(0, Math.floor(totalLength)) : calculatedLength;
    const result = new Buffer(actualLength);
    let pos = 0;
    for (const buf of list) {
      if (pos >= actualLength) break;
      const copyLen = Math.min(buf.length, actualLength - pos);
      result.set(buf.subarray(0, copyLen), pos);
      pos += copyLen;
    }
    return result;
  }

  static copyBytesFrom(view: ArrayBufferView, offset = 0, length?: number): Buffer {
    if (!ArrayBuffer.isView(view)) {
      throw new TypeError('The "view" argument must be a TypedArray or DataView.');
    }
    const elementSize = (view as any).BYTES_PER_ELEMENT || 1;
    const byteOffset = view.byteOffset + offset * elementSize;
    const byteLength =
      length !== undefined ? length * elementSize : view.byteLength - offset * elementSize;
    const u8 = new Uint8Array(view.buffer, byteOffset, Math.max(0, byteLength));
    const res = new Buffer(u8.length);
    res.set(u8);
    return res;
  }

  static isBuffer(obj: any): obj is Buffer {
    return (
      obj instanceof Buffer ||
      (obj != null && typeof obj === "object" && (obj as any)._isBuffer === true)
    );
  }

  static isEncoding(encoding: string): encoding is BufferEncoding {
    if (typeof encoding !== "string") return false;
    const normalized = encoding.toLowerCase();
    return [
      "utf8",
      "utf-8",
      "ascii",
      "latin1",
      "binary",
      "hex",
      "base64",
      "base64url",
      "utf16le",
      "utf-16le",
      "ucs2",
      "ucs-2",
    ].includes(normalized);
  }

  static byteLength(
    string: string | Buffer | ArrayBufferView | ArrayBuffer | SharedArrayBuffer,
    encoding: BufferEncoding = "utf8",
  ): number {
    if (typeof string !== "string") {
      if (ArrayBuffer.isView(string)) return string.byteLength;
      if (
        string instanceof ArrayBuffer ||
        (typeof SharedArrayBuffer !== "undefined" && string instanceof SharedArrayBuffer)
      ) {
        return string.byteLength;
      }
      string = (string as any).toString();
    }
    return stringToBytes(string as string, encoding).length;
  }

  static compare(buf1: Uint8Array, buf2: Uint8Array): -1 | 0 | 1 {
    if (!(buf1 instanceof Uint8Array) || !(buf2 instanceof Uint8Array)) {
      throw new TypeError("Arguments must be instances of Uint8Array or Buffer");
    }
    const len = Math.min(buf1.length, buf2.length);
    for (let i = 0; i < len; i++) {
      if (buf1[i]! < buf2[i]!) return -1;
      if (buf1[i]! > buf2[i]!) return 1;
    }
    if (buf1.length < buf2.length) return -1;
    if (buf1.length > buf2.length) return 1;
    return 0;
  }

  get _isBuffer() {
    return true;
  }

  override subarray(start?: number, end?: number): Buffer {
    const sub = super.subarray(start, end);
    return new Buffer(sub.buffer, sub.byteOffset, sub.length);
  }

  override slice(start?: number, end?: number): Buffer {
    return this.subarray(start, end);
  }

  write(
    string: string,
    offsetOrEncoding?: number | BufferEncoding,
    lengthOrEncoding?: number | BufferEncoding,
    encoding: BufferEncoding = "utf8",
  ): number {
    let offset = 0;
    let length = this.length;
    let enc = encoding;

    if (typeof offsetOrEncoding === "number") {
      offset = offsetOrEncoding;
      if (typeof lengthOrEncoding === "number") {
        length = lengthOrEncoding;
      } else if (typeof lengthOrEncoding === "string") {
        enc = lengthOrEncoding;
      }
    } else if (typeof offsetOrEncoding === "string") {
      enc = offsetOrEncoding;
    }

    if (offset < 0 || offset > this.length) {
      throw new RangeError("Offset is out of bounds");
    }

    const bytes = stringToBytes(string, enc);
    const maxBytes = Math.min(length, this.length - offset);
    const toWrite = Math.min(bytes.length, maxBytes);
    this.set(bytes.subarray(0, toWrite), offset);
    return toWrite;
  }

  override toString(encoding: BufferEncoding = "utf8", start = 0, end = this.length): string {
    const s = Math.max(0, start);
    const e = Math.min(this.length, Math.max(s, end));
    return bytesToString(this.subarray(s, e), encoding);
  }

  toJSON(): { type: "Buffer"; data: number[] } {
    return {
      type: "Buffer",
      data: Array.from(this),
    };
  }

  equals(otherBuffer: Uint8Array): boolean {
    if (!(otherBuffer instanceof Uint8Array)) {
      throw new TypeError("Argument must be an instance of Uint8Array or Buffer");
    }
    return Buffer.compare(this, otherBuffer) === 0;
  }

  compare(
    target: Uint8Array,
    targetStart = 0,
    targetEnd = target?.length,
    sourceStart = 0,
    sourceEnd = this.length,
  ): -1 | 0 | 1 {
    if (!(target instanceof Uint8Array)) {
      throw new TypeError('The "target" argument must be an instance of Uint8Array or Buffer');
    }
    if (
      targetStart < 0 ||
      sourceStart < 0 ||
      targetEnd > target.length ||
      sourceEnd > this.length
    ) {
      throw new RangeError("ERR_OUT_OF_RANGE: Index out of range");
    }
    const subSource = this.subarray(sourceStart, sourceEnd);
    const subTarget = target.subarray(targetStart, targetEnd);
    return Buffer.compare(subSource, subTarget);
  }

  copy(target: Uint8Array, targetStart = 0, sourceStart = 0, sourceEnd = this.length): number {
    if (!(target instanceof Uint8Array)) {
      throw new TypeError('The "target" argument must be an instance of Uint8Array or Buffer');
    }
    if (sourceStart < 0) sourceStart = 0;
    if (sourceEnd > this.length) sourceEnd = this.length;
    if (sourceStart >= sourceEnd) return 0;
    if (targetStart >= target.length) return 0;
    if (targetStart < 0) targetStart = 0;

    const toCopy = Math.min(sourceEnd - sourceStart, target.length - targetStart);
    if (toCopy <= 0) return 0;

    const srcSlice = this.subarray(sourceStart, sourceStart + toCopy);
    target.set(srcSlice, targetStart);
    return toCopy;
  }

  override fill(
    value: string | Uint8Array | number,
    offset = 0,
    end = this.length,
    encoding: BufferEncoding = "utf8",
  ): this {
    const s = Math.max(0, offset);
    const e = Math.min(this.length, Math.max(s, end));
    if (s >= e) return this;

    if (typeof value === "number") {
      const val = value & 255;
      for (let i = s; i < e; i++) {
        this[i] = val;
      }
    } else {
      let bytes: Uint8Array;
      if (typeof value === "string") {
        bytes = stringToBytes(value, encoding);
      } else {
        bytes = value;
      }
      if (bytes.length === 0) {
        for (let i = s; i < e; i++) {
          this[i] = 0;
        }
      } else {
        for (let i = s; i < e; i++) {
          this[i] = bytes[(i - s) % bytes.length]!;
        }
      }
    }
    return this;
  }

  override indexOf(
    value: string | number | Uint8Array,
    byteOffset = 0,
    encoding: BufferEncoding = "utf8",
  ): number {
    return searchBuffer(this, value, byteOffset, encoding, false);
  }

  override lastIndexOf(
    value: string | number | Uint8Array,
    byteOffset = this.length - 1,
    encoding: BufferEncoding = "utf8",
  ): number {
    return searchBuffer(this, value, byteOffset, encoding, true);
  }

  override includes(
    value: string | number | Uint8Array,
    byteOffset = 0,
    encoding: BufferEncoding = "utf8",
  ): boolean {
    return this.indexOf(value, byteOffset, encoding) !== -1;
  }

  override reverse(): this {
    super.reverse();
    return this;
  }

  swap16(): this {
    const len = this.length;
    if (len % 2 !== 0) {
      throw new RangeError("ERR_INVALID_BUFFER_SIZE: Buffer size must be a multiple of 16-bits");
    }
    for (let i = 0; i < len; i += 2) {
      const b = this[i]!;
      this[i] = this[i + 1]!;
      this[i + 1] = b;
    }
    return this;
  }

  swap32(): this {
    const len = this.length;
    if (len % 4 !== 0) {
      throw new RangeError("ERR_INVALID_BUFFER_SIZE: Buffer size must be a multiple of 32-bits");
    }
    for (let i = 0; i < len; i += 4) {
      const b0 = this[i]!,
        b1 = this[i + 1]!,
        b2 = this[i + 2]!,
        b3 = this[i + 3]!;
      this[i] = b3;
      this[i + 1] = b2;
      this[i + 2] = b1;
      this[i + 3] = b0;
    }
    return this;
  }

  swap64(): this {
    const len = this.length;
    if (len % 8 !== 0) {
      throw new RangeError("ERR_INVALID_BUFFER_SIZE: Buffer size must be a multiple of 64-bits");
    }
    for (let i = 0; i < len; i += 8) {
      const b0 = this[i]!,
        b1 = this[i + 1]!,
        b2 = this[i + 2]!,
        b3 = this[i + 3]!;
      const b4 = this[i + 4]!,
        b5 = this[i + 5]!,
        b6 = this[i + 6]!,
        b7 = this[i + 7]!;
      this[i] = b7;
      this[i + 1] = b6;
      this[i + 2] = b5;
      this[i + 3] = b4;
      this[i + 4] = b3;
      this[i + 5] = b2;
      this[i + 6] = b1;
      this[i + 7] = b0;
    }
    return this;
  }

  private _view(): DataView {
    return new DataView(this.buffer, this.byteOffset, this.byteLength);
  }

  readUInt8(offset = 0): number {
    return this._view().getUint8(offset);
  }

  readUint8(offset = 0): number {
    return this.readUInt8(offset);
  }

  readInt8(offset = 0): number {
    return this._view().getInt8(offset);
  }

  readUInt16LE(offset = 0): number {
    return this._view().getUint16(offset, true);
  }

  readUint16LE(offset = 0): number {
    return this.readUInt16LE(offset);
  }

  readUInt16BE(offset = 0): number {
    return this._view().getUint16(offset, false);
  }

  readUint16BE(offset = 0): number {
    return this.readUInt16BE(offset);
  }

  readInt16LE(offset = 0): number {
    return this._view().getInt16(offset, true);
  }

  readInt16BE(offset = 0): number {
    return this._view().getInt16(offset, false);
  }

  readUInt32LE(offset = 0): number {
    return this._view().getUint32(offset, true);
  }

  readUint32LE(offset = 0): number {
    return this.readUInt32LE(offset);
  }

  readUInt32BE(offset = 0): number {
    return this._view().getUint32(offset, false);
  }

  readUint32BE(offset = 0): number {
    return this.readUInt32BE(offset);
  }

  readInt32LE(offset = 0): number {
    return this._view().getInt32(offset, true);
  }

  readInt32BE(offset = 0): number {
    return this._view().getInt32(offset, false);
  }

  readFloatLE(offset = 0): number {
    return this._view().getFloat32(offset, true);
  }

  readFloatBE(offset = 0): number {
    return this._view().getFloat32(offset, false);
  }

  readDoubleLE(offset = 0): number {
    return this._view().getFloat64(offset, true);
  }

  readDoubleBE(offset = 0): number {
    return this._view().getFloat64(offset, false);
  }

  readBigUInt64LE(offset = 0): bigint {
    return this._view().getBigUint64(offset, true);
  }

  readBigUint64LE(offset = 0): bigint {
    return this.readBigUInt64LE(offset);
  }

  readBigUInt64BE(offset = 0): bigint {
    return this._view().getBigUint64(offset, false);
  }

  readBigUint64BE(offset = 0): bigint {
    return this.readBigUInt64BE(offset);
  }

  readBigInt64LE(offset = 0): bigint {
    return this._view().getBigInt64(offset, true);
  }

  readBigInt64BE(offset = 0): bigint {
    return this._view().getBigInt64(offset, false);
  }

  readUIntLE(offset: number, byteLength: number): number {
    offset = offset >>> 0;
    byteLength = byteLength >>> 0;
    let val = this[offset]!;
    let mul = 1;
    let i = 0;
    while (++i < byteLength && (mul *= 0x100)) {
      val += this[offset + i]! * mul;
    }
    return val;
  }

  readUintLE(offset: number, byteLength: number): number {
    return this.readUIntLE(offset, byteLength);
  }

  readUIntBE(offset: number, byteLength: number): number {
    offset = offset >>> 0;
    byteLength = byteLength >>> 0;
    let val = this[offset + --byteLength]!;
    let mul = 1;
    while (byteLength > 0 && (mul *= 0x100)) {
      val += this[offset + --byteLength]! * mul;
    }
    return val;
  }

  readUintBE(offset: number, byteLength: number): number {
    return this.readUIntBE(offset, byteLength);
  }

  readIntLE(offset: number, byteLength: number): number {
    offset = offset >>> 0;
    byteLength = byteLength >>> 0;
    let val = this.readUIntLE(offset, byteLength);
    const limit = 1 << (byteLength * 8 - 1);
    if (val >= limit) {
      val -= Math.pow(2, byteLength * 8);
    }
    return val;
  }

  readIntBE(offset: number, byteLength: number): number {
    offset = offset >>> 0;
    byteLength = byteLength >>> 0;
    let val = this.readUIntBE(offset, byteLength);
    const limit = 1 << (byteLength * 8 - 1);
    if (val >= limit) {
      val -= Math.pow(2, byteLength * 8);
    }
    return val;
  }

  writeUInt8(value: number, offset = 0): number {
    this._view().setUint8(offset, value);
    return offset + 1;
  }

  writeUint8(value: number, offset = 0): number {
    return this.writeUInt8(value, offset);
  }

  writeInt8(value: number, offset = 0): number {
    this._view().setInt8(offset, value);
    return offset + 1;
  }

  writeUInt16LE(value: number, offset = 0): number {
    this._view().setUint16(offset, value, true);
    return offset + 2;
  }

  writeUint16LE(value: number, offset = 0): number {
    return this.writeUInt16LE(value, offset);
  }

  writeUInt16BE(value: number, offset = 0): number {
    this._view().setUint16(offset, value, false);
    return offset + 2;
  }

  writeUint16BE(value: number, offset = 0): number {
    return this.writeUInt16BE(value, offset);
  }

  writeInt16LE(value: number, offset = 0): number {
    this._view().setInt16(offset, value, true);
    return offset + 2;
  }

  writeInt16BE(value: number, offset = 0): number {
    this._view().setInt16(offset, value, false);
    return offset + 2;
  }

  writeUInt32LE(value: number, offset = 0): number {
    this._view().setUint32(offset, value, true);
    return offset + 4;
  }

  writeUint32LE(value: number, offset = 0): number {
    return this.writeUInt32LE(value, offset);
  }

  writeUInt32BE(value: number, offset = 0): number {
    this._view().setUint32(offset, value, false);
    return offset + 4;
  }

  writeUint32BE(value: number, offset = 0): number {
    return this.writeUInt32BE(value, offset);
  }

  writeInt32LE(value: number, offset = 0): number {
    this._view().setInt32(offset, value, true);
    return offset + 4;
  }

  writeInt32BE(value: number, offset = 0): number {
    this._view().setInt32(offset, value, false);
    return offset + 4;
  }

  writeFloatLE(value: number, offset = 0): number {
    this._view().setFloat32(offset, value, true);
    return offset + 4;
  }

  writeFloatBE(value: number, offset = 0): number {
    this._view().setFloat32(offset, value, false);
    return offset + 4;
  }

  writeDoubleLE(value: number, offset = 0): number {
    this._view().setFloat64(offset, value, true);
    return offset + 8;
  }

  writeDoubleBE(value: number, offset = 0): number {
    this._view().setFloat64(offset, value, false);
    return offset + 8;
  }

  writeBigUInt64LE(value: bigint, offset = 0): number {
    this._view().setBigUint64(offset, value, true);
    return offset + 8;
  }

  writeBigUint64LE(value: bigint, offset = 0): number {
    return this.writeBigUInt64LE(value, offset);
  }

  writeBigUInt64BE(value: bigint, offset = 0): number {
    this._view().setBigUint64(offset, value, false);
    return offset + 8;
  }

  writeBigUint64BE(value: bigint, offset = 0): number {
    return this.writeBigUInt64BE(value, offset);
  }

  writeBigInt64LE(value: bigint, offset = 0): number {
    this._view().setBigInt64(offset, value, true);
    return offset + 8;
  }

  writeBigInt64BE(value: bigint, offset = 0): number {
    this._view().setBigInt64(offset, value, false);
    return offset + 8;
  }

  writeUIntLE(value: number, offset: number, byteLength: number): number {
    offset = offset >>> 0;
    byteLength = byteLength >>> 0;
    let mul = 1;
    let i = 0;
    this[offset] = value & 0xff;
    while (++i < byteLength && (mul *= 0x100)) {
      this[offset + i] = (value / mul) & 0xff;
    }
    return offset + byteLength;
  }

  writeUintLE(value: number, offset: number, byteLength: number): number {
    return this.writeUIntLE(value, offset, byteLength);
  }

  writeUIntBE(value: number, offset: number, byteLength: number): number {
    offset = offset >>> 0;
    byteLength = byteLength >>> 0;
    let i = byteLength - 1;
    let mul = 1;
    this[offset + i] = value & 0xff;
    while (--i >= 0 && (mul *= 0x100)) {
      this[offset + i] = (value / mul) & 0xff;
    }
    return offset + byteLength;
  }

  writeUintBE(value: number, offset: number, byteLength: number): number {
    return this.writeUIntBE(value, offset, byteLength);
  }

  writeIntLE(value: number, offset: number, byteLength: number): number {
    if (value < 0) {
      value = 0xffffffffffff + value + 1;
    }
    return this.writeUIntLE(value, offset, byteLength);
  }

  writeIntBE(value: number, offset: number, byteLength: number): number {
    if (value < 0) {
      value = 0xffffffffffff + value + 1;
    }
    return this.writeUIntBE(value, offset, byteLength);
  }
}

export class Blob {
  private _bytes: Uint8Array;
  private _type: string;

  constructor(
    sources: Array<ArrayBuffer | ArrayBufferView | Blob | string> = [],
    options?: BlobOptions,
  ) {
    const parts: Uint8Array[] = [];
    for (const src of sources) {
      if (typeof src === "string") {
        parts.push(new TextEncoder().encode(src));
      } else if (src instanceof Blob) {
        parts.push(src._bytes);
      } else if (ArrayBuffer.isView(src)) {
        parts.push(new Uint8Array(src.buffer, src.byteOffset, src.byteLength));
      } else if (
        src instanceof ArrayBuffer ||
        (typeof SharedArrayBuffer !== "undefined" && (src as any) instanceof SharedArrayBuffer)
      ) {
        parts.push(new Uint8Array(src as ArrayBuffer));
      }
    }
    let totalLen = 0;
    for (const p of parts) totalLen += p.length;
    const joined = new Uint8Array(totalLen);
    let pos = 0;
    for (const p of parts) {
      joined.set(p, pos);
      pos += p.length;
    }
    this._bytes = joined;
    this._type = (options?.type || "").toLowerCase();
  }

  get size(): number {
    return this._bytes.length;
  }

  get type(): string {
    return this._type;
  }

  async arrayBuffer(): Promise<ArrayBuffer> {
    const ab = new ArrayBuffer(this._bytes.byteLength);
    new Uint8Array(ab).set(this._bytes);
    return ab;
  }

  async bytes(): Promise<Uint8Array> {
    return new Uint8Array(this._bytes);
  }

  async text(): Promise<string> {
    return new TextDecoder().decode(this._bytes);
  }

  slice(start?: number, end?: number, type?: string): Blob {
    const s =
      start !== undefined
        ? start < 0
          ? Math.max(0, this.size + start)
          : Math.min(this.size, start)
        : 0;
    const e =
      end !== undefined
        ? end < 0
          ? Math.max(0, this.size + end)
          : Math.min(this.size, end)
        : this.size;
    const sub = this._bytes.subarray(s, Math.max(s, e));
    return new Blob([sub], { type: type !== undefined ? type : this._type });
  }

  stream(): ReadableStream<Uint8Array> {
    const bytes = this._bytes;
    return new ReadableStream({
      start(controller) {
        controller.enqueue(bytes);
        controller.close();
      },
    });
  }
}

export class File extends Blob {
  private _name: string;
  private _lastModified: number;

  constructor(
    sources: Array<ArrayBuffer | ArrayBufferView | Blob | string>,
    fileName: string,
    options?: FileOptions,
  ) {
    super(sources, options);
    this._name = String(fileName);
    this._lastModified =
      options?.lastModified !== undefined ? Number(options.lastModified) : Date.now();
  }

  get name(): string {
    return this._name;
  }

  get lastModified(): number {
    return this._lastModified;
  }
}

export function atob(data: string): string {
  const str = String(data);
  const bytes = decodeBase64(str, false);
  let res = "";
  for (let i = 0; i < bytes.length; i++) {
    res += String.fromCharCode(bytes[i]!);
  }
  return res;
}

export function btoa(data: string): string {
  const str = String(data);
  const bytes = new Uint8Array(str.length);
  for (let i = 0; i < str.length; i++) {
    const code = str.charCodeAt(i);
    if (code > 255) {
      throw new TypeError(
        "The string to be encoded contains characters outside of the Latin1 range.",
      );
    }
    bytes[i] = code;
  }
  return encodeBase64(bytes, false);
}

function toUint8Array(input: Buffer | ArrayBuffer | ArrayBufferView): Uint8Array {
  if (ArrayBuffer.isView(input)) {
    return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  }
  if (
    input instanceof ArrayBuffer ||
    (typeof SharedArrayBuffer !== "undefined" && (input as any) instanceof SharedArrayBuffer)
  ) {
    return new Uint8Array(input as ArrayBuffer);
  }
  throw new TypeError("input must be a Buffer, ArrayBuffer, or TypedArray");
}

export function isAscii(input: Buffer | ArrayBuffer | ArrayBufferView): boolean {
  const u8 = toUint8Array(input);
  for (let i = 0; i < u8.length; i++) {
    if (u8[i]! > 127) return false;
  }
  return true;
}

export function isUtf8(input: Buffer | ArrayBuffer | ArrayBufferView): boolean {
  const u8 = toUint8Array(input);
  try {
    const decoder = new TextDecoder("utf-8", { fatal: true });
    decoder.decode(u8);
    return true;
  } catch {
    return false;
  }
}

export function resolveObjectURL(_id: string): Blob | undefined {
  return undefined;
}

export function transcode(
  source: Uint8Array,
  fromEnc: TranscodeEncoding,
  toEnc: TranscodeEncoding,
): Buffer {
  if (!(source instanceof Uint8Array)) {
    throw new TypeError('The "source" argument must be an instance of Uint8Array or Buffer');
  }
  const str = bytesToString(source, fromEnc as BufferEncoding);
  return Buffer.from(str, toEnc as BufferEncoding);
}

export const SlowBuffer = Buffer;
