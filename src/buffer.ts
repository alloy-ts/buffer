import * as nativeModule from "../index.js";
import { Bytes as NativeBytes } from "../index.js";

const native = nativeModule as any;

export const Bytes = NativeBytes;

export class Buffer extends Uint8Array {
  static poolSize: number = native.poolSize ?? 65536;

  static isBuffer(obj: unknown): obj is Buffer {
    return (
      obj instanceof Buffer ||
      (obj != null &&
        typeof obj === "object" &&
        (obj as { _isBuffer?: boolean })._isBuffer === true)
    );
  }

  static isEncoding(encoding: string): boolean {
    if (typeof encoding !== "string" || encoding === "") return false;
    return native.bufIsEncoding(encoding);
  }

  static alloc(size: number, fill?: string | Uint8Array | number, encoding?: string): Buffer {
    if (typeof size !== "number" || Number.isNaN(size)) {
      throw new TypeError('The "size" argument must be of type number');
    }
    if (size < 0 || size > (native.kMaxLength ?? 0x7fffffff)) {
      throw new RangeError('The value of "size" is out of range');
    }
    const buf = Buffer.allocUnsafe(size);
    if (fill !== undefined) {
      buf.fill(fill, 0, size, encoding);
    } else {
      buf.fill(0);
    }
    return buf;
  }

  static allocUnsafe(size: number): Buffer {
    if (typeof size !== "number" || Number.isNaN(size)) {
      throw new TypeError('The "size" argument must be of type number');
    }
    if (size < 0 || size > (native.kMaxLength ?? 0x7fffffff)) {
      throw new RangeError('The value of "size" is out of range');
    }
    return new Buffer(size);
  }

  static allocUnsafeSlow(size: number): Buffer {
    return Buffer.allocUnsafe(size);
  }

  static byteLength(
    string: string | Buffer | ArrayBufferView | ArrayBuffer | SharedArrayBuffer,
    encoding?: string,
  ): number {
    if (typeof string !== "string") {
      if (ArrayBuffer.isView(string)) {
        return string.byteLength;
      }
      if (
        string instanceof ArrayBuffer ||
        (typeof SharedArrayBuffer !== "undefined" && string instanceof SharedArrayBuffer)
      ) {
        return string.byteLength;
      }
      throw new TypeError(
        'The "string" argument must be of type string or an instance of Buffer or ArrayBuffer',
      );
    }
    return native.bufByteLength(string, encoding);
  }

  static compare(buf1: Uint8Array<any>, buf2: Uint8Array<any>): -1 | 0 | 1 {
    if (!(buf1 instanceof Uint8Array) || !(buf2 instanceof Uint8Array)) {
      throw new TypeError('The "buf1" and "buf2" arguments must be an instance of Uint8Array');
    }
    const res = native.bufCompare(buf1, buf2);
    return (res < 0 ? -1 : res > 0 ? 1 : 0) as -1 | 0 | 1;
  }

  static concat(list: readonly Uint8Array<any>[], totalLength?: number): Buffer {
    if (!Array.isArray(list)) {
      throw new TypeError('The "list" argument must be an instance of Array');
    }
    if (list.length === 0 || totalLength === 0) {
      return Buffer.alloc(0);
    }
    if (totalLength === undefined) {
      totalLength = 0;
      for (const b of list) {
        totalLength += b.length;
      }
    }
    const bufs = list.map((item) => (item instanceof Uint8Array ? item : new Uint8Array(item)));
    const res = native.bufConcat(bufs, totalLength);
    return Buffer.from(res.buffer, res.byteOffset, res.byteLength);
  }

  static copyBytesFrom(view: ArrayBufferView, offset?: number, length?: number): Buffer {
    if (!ArrayBuffer.isView(view)) {
      throw new TypeError('The "view" argument must be an instance of ArrayBufferView');
    }
    const res = native.bufCopyBytesFrom(view, offset, length);
    return Buffer.from(res.buffer, res.byteOffset, res.byteLength);
  }

  static override from(value: any, encodingOrOffset?: any, length?: number): Buffer {
    if (typeof value === "string") {
      const res = native.bufDecode(value, encodingOrOffset);
      return Buffer.from(res.buffer, res.byteOffset, res.byteLength);
    }
    if (
      value instanceof ArrayBuffer ||
      (typeof SharedArrayBuffer !== "undefined" && value instanceof SharedArrayBuffer)
    ) {
      const byteOffset = encodingOrOffset ?? 0;
      const len = length ?? value.byteLength - byteOffset;
      return new Buffer(value as any, byteOffset, len);
    }
    if (ArrayBuffer.isView(value)) {
      const u8 = new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
      const res = new Buffer(u8.length);
      res.set(u8);
      return res;
    }
    if (
      Array.isArray(value) ||
      (value != null &&
        typeof value === "object" &&
        typeof (value as { length: unknown }).length === "number")
    ) {
      const arr = value as ArrayLike<unknown>;
      const res = new Buffer(arr.length);
      for (let i = 0; i < arr.length; i++) {
        res[i] = Number(arr[i]) & 255 || 0;
      }
      return res;
    }
    if (value != null && typeof value === "object") {
      if (typeof value.valueOf === "function" && value.valueOf() !== value) {
        return Buffer.from(value.valueOf(), encodingOrOffset, length);
      }
      if (typeof value[Symbol.toPrimitive] === "function") {
        return Buffer.from(value[Symbol.toPrimitive]("string"), encodingOrOffset, length);
      }
    }
    throw new TypeError(
      "First argument must be a string, Buffer, ArrayBuffer, Array, or Array-like object.",
    );
  }

  static override of(...items: number[]): Buffer {
    return Buffer.from(items);
  }

  override get [Symbol.toStringTag](): "Uint8Array" {
    return "Uint8Array";
  }

  get _isBuffer(): boolean {
    return true;
  }

  override toBase64(options?: {
    alphabet?: "base64" | "base64url";
    omitPadding?: boolean;
  }): string {
    const proto = Uint8Array.prototype as any;
    if (typeof proto.toBase64 === "function" && options === undefined) {
      return proto.toBase64.call(this, options);
    }
    const alphabet = options?.alphabet ?? "base64";
    const omitPadding = options?.omitPadding ?? false;
    let b64 = native.bufEncode(this, alphabet === "base64url" ? "base64url" : "base64");
    if (omitPadding) {
      b64 = b64.replace(/=+$/, "");
    }
    return b64;
  }

  override toHex(): string {
    const proto = Uint8Array.prototype as any;
    if (typeof proto.toHex === "function") {
      return proto.toHex.call(this);
    }
    return native.bufEncode(this, "hex");
  }

  override setFromBase64(
    base64: string,
    offsetOrOptions: any = 0,
  ): { read: number; written: number } {
    const proto = Uint8Array.prototype as any;
    if (typeof proto.setFromBase64 === "function") {
      return proto.setFromBase64.call(this, base64, offsetOrOptions);
    }
    const offset = typeof offsetOrOptions === "number" ? offsetOrOptions : 0;
    const decoded = native.bufDecode(base64, "base64");
    const targetSlice = this.subarray(offset);
    const written = decoded.copy(targetSlice, 0);
    return { read: base64.length, written };
  }

  override setFromHex(hex: string, offsetOrOptions: any = 0): { read: number; written: number } {
    const proto = Uint8Array.prototype as any;
    if (typeof proto.setFromHex === "function") {
      return proto.setFromHex.call(this, hex, offsetOrOptions);
    }
    const offset = typeof offsetOrOptions === "number" ? offsetOrOptions : 0;
    if (hex.length % 2 !== 0) {
      throw new TypeError("Hex string must have an even number of characters");
    }
    const decoded = native.bufDecode(hex, "hex");
    const targetSlice = this.subarray(offset);
    const written = decoded.copy(targetSlice, 0);
    return { read: hex.length, written };
  }

  private _dv(): DataView {
    return new DataView(this.buffer, this.byteOffset, this.byteLength);
  }

  override toString(encoding?: string, start?: number, end?: number): string {
    return native.bufEncode(this, encoding, start, end);
  }

  write(
    string: string,
    offsetOrEncoding?: number | string,
    lengthOrEncoding?: number | string,
    encoding?: string,
  ): number {
    let offset = 0;
    let length = this.length;
    let enc = encoding ?? "utf8";

    if (typeof offsetOrEncoding === "string") {
      enc = offsetOrEncoding;
    } else if (typeof offsetOrEncoding === "number") {
      offset = offsetOrEncoding;
      if (typeof lengthOrEncoding === "string") {
        enc = lengthOrEncoding;
      } else if (typeof lengthOrEncoding === "number") {
        length = lengthOrEncoding;
      }
    }

    return native.bufWrite(this, string, offset, length, enc);
  }

  toJSON(): { type: "Buffer"; data: number[] } {
    return {
      type: "Buffer",
      data: Array.from(this),
    };
  }

  equals(otherBuffer: Uint8Array<any>): boolean {
    if (!(otherBuffer instanceof Uint8Array)) {
      throw new TypeError("Argument must be a Uint8Array or Buffer");
    }
    return Buffer.compare(this, otherBuffer) === 0;
  }

  compare(
    target: Uint8Array<any>,
    targetStart?: number,
    targetEnd?: number,
    sourceStart?: number,
    sourceEnd?: number,
  ): -1 | 0 | 1 {
    if (!(target instanceof Uint8Array)) {
      throw new TypeError("Argument must be a Uint8Array or Buffer");
    }
    const tStart = targetStart ?? 0;
    const tEnd = targetEnd ?? target.byteLength;
    const sStart = sourceStart ?? 0;
    const sEnd = sourceEnd ?? this.byteLength;

    if (tStart < 0 || sStart < 0 || tEnd > target.byteLength || sEnd > this.byteLength) {
      throw new RangeError("ERR_OUT_OF_RANGE: Index out of range");
    }

    const srcSlice = this.subarray(sStart, sEnd);
    const tgtSlice = target.subarray(tStart, tEnd);
    return Buffer.compare(srcSlice, tgtSlice);
  }

  copy(
    target: Uint8Array<any>,
    targetStart?: number,
    sourceStart?: number,
    sourceEnd?: number,
  ): number {
    if (!(target instanceof Uint8Array)) {
      throw new TypeError("Argument must be a Uint8Array or Buffer");
    }
    let tStart = targetStart ?? 0;
    let sStart = sourceStart ?? 0;
    let sEnd = sourceEnd ?? this.length;

    if (tStart < 0) tStart = 0;
    if (sStart < 0) sStart = 0;
    if (sEnd > this.length) sEnd = this.length;
    if (sStart >= sEnd || tStart >= target.length) return 0;

    const toCopy = Math.min(sEnd - sStart, target.length - tStart);
    target.set(this.subarray(sStart, sStart + toCopy), tStart);
    return toCopy;
  }

  override fill(
    value: string | Uint8Array<any> | number,
    offset?: number,
    end?: number,
    encoding?: string,
  ): this {
    native.bufFill(this, value, offset, end, encoding);
    return this;
  }

  override indexOf(
    value: string | number | Uint8Array<any>,
    byteOffset?: number,
    encoding?: string,
  ): number {
    return native.bufIndexOf(this, value, byteOffset, undefined, encoding);
  }

  override lastIndexOf(
    value: string | number | Uint8Array<any>,
    byteOffset?: number,
    encoding?: string,
  ): number {
    return native.bufLastIndexOf(this, value, byteOffset, undefined, encoding);
  }

  override includes(
    value: string | number | Uint8Array<any>,
    byteOffset?: number,
    encoding?: string,
  ): boolean {
    return native.bufIncludes(this, value, byteOffset, undefined, encoding);
  }

  override slice(start?: number, end?: number): Buffer {
    return this.subarray(start, end);
  }

  override subarray(start?: number, end?: number): Buffer {
    const sub = super.subarray(start, end);
    return Buffer.from(sub.buffer, sub.byteOffset, sub.byteLength);
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
      const val = this[i]!;
      this[i] = this[i + 1]!;
      this[i + 1] = val;
    }
    return this;
  }

  swap32(): this {
    const len = this.length;
    if (len % 4 !== 0) {
      throw new RangeError("ERR_INVALID_BUFFER_SIZE: Buffer size must be a multiple of 32-bits");
    }
    for (let i = 0; i < len; i += 4) {
      const b0 = this[i]!;
      const b1 = this[i + 1]!;
      const b2 = this[i + 2]!;
      const b3 = this[i + 3]!;
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
      const b0 = this[i]!;
      const b1 = this[i + 1]!;
      const b2 = this[i + 2]!;
      const b3 = this[i + 3]!;
      const b4 = this[i + 4]!;
      const b5 = this[i + 5]!;
      const b6 = this[i + 6]!;
      const b7 = this[i + 7]!;
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

  // --- Read Methods ---

  readInt8(offset = 0): number {
    return this._dv().getInt8(offset);
  }

  readUInt8(offset = 0): number {
    return this._dv().getUint8(offset);
  }

  readUint8(offset = 0): number {
    return this.readUInt8(offset);
  }

  readInt16LE(offset = 0): number {
    return this._dv().getInt16(offset, true);
  }

  readInt16BE(offset = 0): number {
    return this._dv().getInt16(offset, false);
  }

  readUInt16LE(offset = 0): number {
    return this._dv().getUint16(offset, true);
  }

  readUint16LE(offset = 0): number {
    return this.readUInt16LE(offset);
  }

  readUInt16BE(offset = 0): number {
    return this._dv().getUint16(offset, false);
  }

  readUint16BE(offset = 0): number {
    return this.readUInt16BE(offset);
  }

  readInt32LE(offset = 0): number {
    return this._dv().getInt32(offset, true);
  }

  readInt32BE(offset = 0): number {
    return this._dv().getInt32(offset, false);
  }

  readUInt32LE(offset = 0): number {
    return this._dv().getUint32(offset, true);
  }

  readUint32LE(offset = 0): number {
    return this.readUInt32LE(offset);
  }

  readUInt32BE(offset = 0): number {
    return this._dv().getUint32(offset, false);
  }

  readUint32BE(offset = 0): number {
    return this.readUInt32BE(offset);
  }

  readFloatLE(offset = 0): number {
    return this._dv().getFloat32(offset, true);
  }

  readFloatBE(offset = 0): number {
    return this._dv().getFloat32(offset, false);
  }

  readDoubleLE(offset = 0): number {
    return this._dv().getFloat64(offset, true);
  }

  readDoubleBE(offset = 0): number {
    return this._dv().getFloat64(offset, false);
  }

  readBigInt64LE(offset = 0): bigint {
    return this._dv().getBigInt64(offset, true);
  }

  readBigInt64BE(offset = 0): bigint {
    return this._dv().getBigInt64(offset, false);
  }

  readBigUInt64LE(offset = 0): bigint {
    return this._dv().getBigUint64(offset, true);
  }

  readBigUint64LE(offset = 0): bigint {
    return this.readBigUInt64LE(offset);
  }

  readBigUInt64BE(offset = 0): bigint {
    return this._dv().getBigUint64(offset, false);
  }

  readBigUint64BE(offset = 0): bigint {
    return this.readBigUInt64BE(offset);
  }

  readUIntLE(offset: number, byteLength: number): number {
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
    let len = byteLength;
    let val = this[offset + --len]!;
    let mul = 1;
    while (len > 0 && (mul *= 0x100)) {
      val += this[offset + --len]! * mul;
    }
    return val;
  }

  readUintBE(offset: number, byteLength: number): number {
    return this.readUIntBE(offset, byteLength);
  }

  readIntLE(offset: number, byteLength: number): number {
    let val = this.readUIntLE(offset, byteLength);
    const limit = 2 ** (8 * byteLength - 1);
    if (val >= limit) val -= limit * 2;
    return val;
  }

  readIntBE(offset: number, byteLength: number): number {
    let val = this.readUIntBE(offset, byteLength);
    const limit = 2 ** (8 * byteLength - 1);
    if (val >= limit) val -= limit * 2;
    return val;
  }

  // --- Write Methods ---

  writeInt8(value: number, offset = 0): number {
    this._dv().setInt8(offset, value);
    return offset + 1;
  }

  writeUInt8(value: number, offset = 0): number {
    this._dv().setUint8(offset, value);
    return offset + 1;
  }

  writeUint8(value: number, offset = 0): number {
    return this.writeUInt8(value, offset);
  }

  writeInt16LE(value: number, offset = 0): number {
    this._dv().setInt16(offset, value, true);
    return offset + 2;
  }

  writeInt16BE(value: number, offset = 0): number {
    this._dv().setInt16(offset, value, false);
    return offset + 2;
  }

  writeUInt16LE(value: number, offset = 0): number {
    this._dv().setUint16(offset, value, true);
    return offset + 2;
  }

  writeUint16LE(value: number, offset = 0): number {
    return this.writeUInt16LE(value, offset);
  }

  writeUInt16BE(value: number, offset = 0): number {
    this._dv().setUint16(offset, value, false);
    return offset + 2;
  }

  writeUint16BE(value: number, offset = 0): number {
    return this.writeUInt16BE(value, offset);
  }

  writeInt32LE(value: number, offset = 0): number {
    this._dv().setInt32(offset, value, true);
    return offset + 4;
  }

  writeInt32BE(value: number, offset = 0): number {
    this._dv().setInt32(offset, value, false);
    return offset + 4;
  }

  writeUInt32LE(value: number, offset = 0): number {
    this._dv().setUint32(offset, value, true);
    return offset + 4;
  }

  writeUint32LE(value: number, offset = 0): number {
    return this.writeUInt32LE(value, offset);
  }

  writeUInt32BE(value: number, offset = 0): number {
    this._dv().setUint32(offset, value, false);
    return offset + 4;
  }

  writeUint32BE(value: number, offset = 0): number {
    return this.writeUInt32BE(value, offset);
  }

  writeFloatLE(value: number, offset = 0): number {
    this._dv().setFloat32(offset, value, true);
    return offset + 4;
  }

  writeFloatBE(value: number, offset = 0): number {
    this._dv().setFloat32(offset, value, false);
    return offset + 4;
  }

  writeDoubleLE(value: number, offset = 0): number {
    this._dv().setFloat64(offset, value, true);
    return offset + 8;
  }

  writeDoubleBE(value: number, offset = 0): number {
    this._dv().setFloat64(offset, value, false);
    return offset + 8;
  }

  writeBigInt64LE(value: bigint, offset = 0): number {
    this._dv().setBigInt64(offset, value, true);
    return offset + 8;
  }

  writeBigInt64BE(value: bigint, offset = 0): number {
    this._dv().setBigInt64(offset, value, false);
    return offset + 8;
  }

  writeBigUInt64LE(value: bigint, offset = 0): number {
    this._dv().setBigUint64(offset, value, true);
    return offset + 8;
  }

  writeBigUint64LE(value: bigint, offset = 0): number {
    return this.writeBigUInt64LE(value, offset);
  }

  writeBigUInt64BE(value: bigint, offset = 0): number {
    this._dv().setBigUint64(offset, value, false);
    return offset + 8;
  }

  writeBigUint64BE(value: bigint, offset = 0): number {
    return this.writeBigUInt64BE(value, offset);
  }

  writeUIntLE(value: number, offset: number, byteLength: number): number {
    let i = 0;
    let mul = 1;
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
    if (value < 0) value = 2 ** (8 * byteLength) + value;
    return this.writeUIntLE(value, offset, byteLength);
  }

  writeIntBE(value: number, offset: number, byteLength: number): number {
    if (value < 0) value = 2 ** (8 * byteLength) + value;
    return this.writeUIntBE(value, offset, byteLength);
  }
}

export const Blob = native.Blob;
export const File = native.File;
export const atob = native.atob;
export const btoa = native.btoa;
export const isAscii = native.isAscii;
export const isUtf8 = native.isUtf8;
export const isLatin1 = native.isLatin1;
export const resolveObjectURL = native.resolveObjectURL;

export function transcode(source: Uint8Array, fromEncoding: string, toEncoding: string): Buffer {
  const res = native.transcode(source, fromEncoding, toEncoding);
  return Buffer.from(res.buffer, res.byteOffset, res.byteLength);
}

export const constants = {
  MAX_LENGTH: native.kMaxLength ?? 0x7fffffff,
  MAX_STRING_LENGTH: native.kStringMaxLength ?? 0x1fffffe8,
};

export const INSPECT_MAX_BYTES = native.INSPECT_MAX_BYTES ?? 50;
export const kMaxLength = native.kMaxLength ?? 0x7fffffff;
export const kStringMaxLength = native.kStringMaxLength ?? 0x1fffffe8;

export function SlowBuffer(size: number): Buffer {
  return Buffer.allocUnsafe(size);
}
(SlowBuffer as any).prototype = Buffer.prototype;
