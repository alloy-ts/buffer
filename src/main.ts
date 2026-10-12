import * as native from "../dist/index.js";

export const Buffer = native.Buffer;
export const Blob = native.Blob;
export const File = native.File;

export const atob = native.atob;
export const btoa = native.btoa;
export const isAscii = native.isAscii;
export const isUtf8 = native.isUtf8;
export const resolveObjectURL = native.resolveObjectURL;
export const transcode = native.transcode;

export const constants = (native as any).constants ?? {
  MAX_LENGTH: 2147483647,
  MAX_STRING_LENGTH: 536870888,
};

export const INSPECT_MAX_BYTES = (native as any).INSPECT_MAX_BYTES ?? 50;
export const kMaxLength = (native as any).kMaxLength ?? 2147483647;
export const kStringMaxLength = (native as any).kStringMaxLength ?? 536870888;
export const poolSize = (native as any).poolSize ?? 65536;

export const SlowBuffer = function (size: number) {
  return Buffer.allocUnsafeSlow(size);
};
SlowBuffer.prototype = Buffer.prototype;

// Attach Uint8Array / Buffer prototype enhancements
const proto = Buffer.prototype as any;

Object.defineProperty(proto, Symbol.toStringTag, {
  value: "Uint8Array",
  configurable: true,
  writable: false,
});

proto.BYTES_PER_ELEMENT = 1;

proto.at = function (index: number) {
  const k = index >= 0 ? index : this.length + index;
  if (k < 0 || k >= this.length) return undefined;
  return this.readUInt8(k);
};

proto[Symbol.iterator] = function* () {
  for (let i = 0; i < this.length; i++) {
    yield this.readUInt8(i);
  }
};

proto.toHex = function () {
  return this.toString("hex");
};

proto.toBase64 = function (options?: { alphabet?: "base64" | "base64url"; omitPadding?: boolean }) {
  const enc = options?.alphabet === "base64url" ? "base64url" : "base64";
  let str = this.toString(enc);
  if (options?.omitPadding) {
    str = str.replace(/=+$/, "");
  }
  return str;
};

proto.forEach = function (callbackfn: (value: number, index: number, array: any) => void, thisArg?: any) {
  for (let i = 0; i < this.length; i++) {
    callbackfn.call(thisArg, this.readUInt8(i), i, this);
  }
};

proto.map = function (callbackfn: (value: number, index: number, array: any) => number, thisArg?: any) {
  const res = new Uint8Array(this.length);
  for (let i = 0; i < this.length; i++) {
    res[i] = callbackfn.call(thisArg, this.readUInt8(i), i, this);
  }
  return res;
};

proto.filter = function (predicate: (value: number, index: number, array: any) => boolean, thisArg?: any) {
  const arr: number[] = [];
  for (let i = 0; i < this.length; i++) {
    const val = this.readUInt8(i);
    if (predicate.call(thisArg, val, i, this)) {
      arr.push(val);
    }
  }
  return new Uint8Array(arr);
};

proto.find = function (predicate: (value: number, index: number, obj: any) => boolean, thisArg?: any) {
  for (let i = 0; i < this.length; i++) {
    const val = this.readUInt8(i);
    if (predicate.call(thisArg, val, i, this)) {
      return val;
    }
  }
  return undefined;
};

proto.findIndex = function (predicate: (value: number, index: number, obj: any) => boolean, thisArg?: any) {
  for (let i = 0; i < this.length; i++) {
    const val = this.readUInt8(i);
    if (predicate.call(thisArg, val, i, this)) {
      return i;
    }
  }
  return -1;
};

proto.findLast = function (predicate: (value: number, index: number, array: any) => boolean, thisArg?: any) {
  for (let i = this.length - 1; i >= 0; i--) {
    const val = this.readUInt8(i);
    if (predicate.call(thisArg, val, i, this)) {
      return val;
    }
  }
  return undefined;
};

proto.findLastIndex = function (predicate: (value: number, index: number, array: any) => boolean, thisArg?: any) {
  for (let i = this.length - 1; i >= 0; i--) {
    const val = this.readUInt8(i);
    if (predicate.call(thisArg, val, i, this)) {
      return i;
    }
  }
  return -1;
};

proto.every = function (predicate: (value: number, index: number, array: any) => boolean, thisArg?: any) {
  for (let i = 0; i < this.length; i++) {
    if (!predicate.call(thisArg, this.readUInt8(i), i, this)) {
      return false;
    }
  }
  return true;
};

proto.some = function (predicate: (value: number, index: number, array: any) => boolean, thisArg?: any) {
  for (let i = 0; i < this.length; i++) {
    if (predicate.call(thisArg, this.readUInt8(i), i, this)) {
      return true;
    }
  }
  return false;
};

proto.reduce = function (callbackfn: any, initialValue?: any) {
  let accumulator = initialValue;
  let startIndex = 0;
  if (accumulator === undefined) {
    if (this.length === 0) throw new TypeError("Reduce of empty buffer with no initial value");
    accumulator = this.readUInt8(0);
    startIndex = 1;
  }
  for (let i = startIndex; i < this.length; i++) {
    accumulator = callbackfn(accumulator, this.readUInt8(i), i, this);
  }
  return accumulator;
};

proto.reduceRight = function (callbackfn: any, initialValue?: any) {
  let accumulator = initialValue;
  let startIndex = this.length - 1;
  if (accumulator === undefined) {
    if (this.length === 0) throw new TypeError("Reduce of empty buffer with no initial value");
    accumulator = this.readUInt8(startIndex);
    startIndex--;
  }
  for (let i = startIndex; i >= 0; i--) {
    accumulator = callbackfn(accumulator, this.readUInt8(i), i, this);
  }
  return accumulator;
};

proto.join = function (separator: string = ",") {
  const arr: number[] = [];
  for (let i = 0; i < this.length; i++) {
    arr.push(this.readUInt8(i));
  }
  return arr.join(separator);
};

proto.set = function (array: ArrayLike<number>, offset: number = 0) {
  for (let i = 0; i < array.length; i++) {
    if (offset + i < this.length) {
      this.writeUInt8(array[i] & 0xff, offset + i);
    }
  }
};

proto.setFromBase64 = function (base64: string, offset: number = 0) {
  const decoded = globalThis.Buffer ? globalThis.Buffer.from(base64, "base64") : native.Buffer.from(base64, "base64");
  let written = 0;
  for (let i = 0; i < decoded.length && offset + i < this.length; i++) {
    const val = typeof (decoded as any).readUInt8 === "function" ? (decoded as any).readUInt8(i) : decoded[i];
    this.writeUInt8(val, offset + i);
    written++;
  }
  return { read: base64.length, written };
};

proto.setFromHex = function (hex: string, offset: number = 0) {
  const decoded = native.Buffer.from(hex, "hex");
  let written = 0;
  for (let i = 0; i < decoded.length && offset + i < this.length; i++) {
    this.writeUInt8(decoded.readUInt8(i), offset + i);
    written++;
  }
  return { read: hex.length, written };
};

proto.copyWithin = function (target: number, start: number, end?: number) {
  const len = this.length;
  const to = target < 0 ? Math.max(len + target, 0) : Math.min(target, len);
  const from = start < 0 ? Math.max(len + start, 0) : Math.min(start, len);
  const final = end === undefined ? len : end < 0 ? Math.max(len + end, 0) : Math.min(end, len);
  const count = Math.min(final - from, len - to);

  if (count > 0) {
    const temp = new Uint8Array(count);
    for (let i = 0; i < count; i++) {
      temp[i] = this.readUInt8(from + i);
    }
    for (let i = 0; i < count; i++) {
      this.writeUInt8(temp[i], to + i);
    }
  }
  return this;
};

proto.sort = function (compareFn?: (a: number, b: number) => number) {
  const arr: number[] = [];
  for (let i = 0; i < this.length; i++) {
    arr.push(this.readUInt8(i));
  }
  arr.sort(compareFn ?? ((a, b) => a - b));
  for (let i = 0; i < this.length; i++) {
    this.writeUInt8(arr[i], i);
  }
  return this;
};

proto.toReversed = function () {
  const res = new Uint8Array(this.length);
  for (let i = 0; i < this.length; i++) {
    res[i] = this.readUInt8(this.length - 1 - i);
  }
  return res;
};

proto.toSorted = function (compareFn?: (a: number, b: number) => number) {
  const arr: number[] = [];
  for (let i = 0; i < this.length; i++) {
    arr.push(this.readUInt8(i));
  }
  arr.sort(compareFn ?? ((a, b) => a - b));
  return new Uint8Array(arr);
};

proto.with = function (index: number, value: number) {
  const len = this.length;
  const relativeIndex = index < 0 ? len + index : index;
  if (relativeIndex < 0 || relativeIndex >= len) {
    throw new RangeError("Invalid index");
  }
  const res = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    res[i] = i === relativeIndex ? value & 0xff : this.readUInt8(i);
  }
  return res;
};

proto.valueOf = function () {
  return this;
};

proto.toLocaleString = function (locales?: any, options?: any) {
  const arr: number[] = [];
  for (let i = 0; i < this.length; i++) {
    arr.push(this.readUInt8(i));
  }
  return arr.toLocaleString(locales, options);
};

export type {
  BlobOptions,
  FileOptions,
  BufferJson,
} from "../dist/index.d.ts";
