import { expect, test } from "vite-plus/test";
import {
  Buffer,
  Blob,
  File,
  atob,
  btoa,
  isAscii,
  isUtf8,
  transcode,
  resolveObjectURL,
  constants,
  INSPECT_MAX_BYTES,
  kMaxLength,
  kStringMaxLength,
  SlowBuffer,
} from "./main.ts";

test("Buffer.alloc and allocUnsafe", () => {
  const buf1 = Buffer.alloc(10);
  expect(buf1.length).toBe(10);
  expect(Array.from(buf1)).toEqual([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

  const buf2 = Buffer.alloc(10, 1);
  expect(buf2.length).toBe(10);
  expect(Array.from(buf2)).toEqual([1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

  const buf3 = Buffer.allocUnsafe(10);
  expect(buf3.length).toBe(10);

  const buf4 = SlowBuffer(10);
  expect(buf4.length).toBe(10);

  expect(() => Buffer.alloc(-1)).toThrow();
  expect(() => Buffer.alloc("10" as any)).toThrow();
});

test("Buffer.from array and truncated values", () => {
  const buf1 = Buffer.from([1, 2, 3]);
  expect(Array.from(buf1)).toEqual([1, 2, 3]);

  const buf2 = Buffer.from([257, 257.5, -255, "1"]);
  expect(Array.from(buf2)).toEqual([1, 1, 1, 1]);

  const buf3 = Buffer.of(1, 2, 3);
  expect(Array.from(buf3)).toEqual([1, 2, 3]);
});

test("Buffer.from string and encodings", () => {
  const buf1 = Buffer.from("tést");
  expect(Array.from(buf1)).toEqual([116, 195, 169, 115, 116]);
  expect(buf1.toString("utf8")).toBe("tést");

  const buf2 = Buffer.from("tést", "latin1");
  expect(Array.from(buf2)).toEqual([0x74, 0xe9, 0x73, 0x74]);
  expect(buf2.toString("latin1")).toBe("tést");

  const hexBuf = Buffer.from("74c3a97374", "hex");
  expect(hexBuf.toString("utf8")).toBe("tést");

  const b64Buf = Buffer.from("dOlzdA==", "base64");
  expect(b64Buf.toString("latin1")).toBe("tést");

  const b64UrlBuf = Buffer.from("dOlzdA==", "base64url");
  expect(b64UrlBuf.toString("latin1")).toBe("tést");

  const utf16Buf = Buffer.from("tést", "utf16le");
  expect(utf16Buf.toString("utf16le")).toBe("tést");
});

test("Buffer.from ArrayBuffer, TypedArray, DataView, Object", () => {
  const ab = new ArrayBuffer(10);
  const buf = Buffer.from(ab, 0, 2);
  expect(buf.length).toBe(2);
  expect(buf.buffer).toBe(ab);

  const u8 = new Uint8Array([10, 20, 30]);
  const bufFromTA = Buffer.from(u8);
  expect(Array.from(bufFromTA)).toEqual([10, 20, 30]);

  const objVal = { valueOf: () => [1, 2, 3] };
  expect(Array.from(Buffer.from(objVal as any))).toEqual([1, 2, 3]);

  const objPrim = { [Symbol.toPrimitive]: (_hint: string) => "hello" };
  expect(Buffer.from(objPrim as any).toString()).toBe("hello");

  expect(() => Buffer.from(123 as any)).toThrow();
});

test("Buffer.isBuffer, Buffer.isEncoding, poolSize", () => {
  expect(Buffer.isBuffer(Buffer.alloc(10))).toBe(true);
  expect(Buffer.isBuffer(Buffer.from("foo"))).toBe(true);
  expect(Buffer.isBuffer("a string")).toBe(false);
  expect(Buffer.isBuffer([])).toBe(false);
  expect(Buffer.isBuffer(new Uint8Array(1024))).toBe(false);

  expect(Buffer.isEncoding("utf8")).toBe(true);
  expect(Buffer.isEncoding("hex")).toBe(true);
  expect(Buffer.isEncoding("utf/8")).toBe(false);
  expect(Buffer.isEncoding("")).toBe(false);

  expect(typeof Buffer.poolSize).toBe("number");
});

test("Buffer.byteLength and copyBytesFrom", () => {
  const str = "\u00bd + \u00bc = \u00be";
  expect(Buffer.byteLength(str, "utf8")).toBe(12);
  expect(Buffer.byteLength(Buffer.from(str))).toBe(12);

  const u16 = new Uint16Array([0x1234, 0x5678]);
  const copied = Buffer.copyBytesFrom(u16);
  expect(copied.length).toBe(4);
});

test("Buffer.concat", () => {
  const buf1 = Buffer.alloc(10);
  const buf2 = Buffer.alloc(14);
  const buf3 = Buffer.alloc(18);
  const totalLength = buf1.length + buf2.length + buf3.length;
  expect(totalLength).toBe(42);

  const bufA = Buffer.concat([buf1, buf2, buf3], totalLength);
  expect(bufA.length).toBe(42);
  expect(Buffer.isBuffer(bufA)).toBe(true);

  expect(Buffer.concat([]).length).toBe(0);
});

test("Buffer.compare and buf.equals", () => {
  const buf1 = Buffer.from("ABC");
  const buf2 = Buffer.from("414243", "hex");
  const buf3 = Buffer.from("ABCD");

  expect(buf1.equals(buf2)).toBe(true);
  expect(buf1.equals(buf3)).toBe(false);

  expect(buf1.compare(buf1)).toBe(0);
  expect(buf1.compare(buf2)).toBe(0);
  expect(buf1.compare(buf3)).toBe(-1);
  expect(buf3.compare(buf1)).toBe(1);

  const arr = [buf3, buf1];
  arr.sort((a, b) => Buffer.compare(a, b));
  expect(arr[0]!.toString()).toBe("ABC");
  expect(arr[1]!.toString()).toBe("ABCD");

  const b1 = Buffer.from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
  const b2 = Buffer.from([5, 6, 7, 8, 9, 1, 2, 3, 4]);
  expect(b1.compare(b2, 5, 9, 0, 4)).toBe(0);
});

test("buf.slice and buf.subarray memory sharing", () => {
  const buf1 = Buffer.allocUnsafe(26);
  for (let i = 0; i < 26; i++) {
    buf1[i] = i + 97;
  }

  const buf2 = buf1.subarray(0, 3);
  expect(buf2.toString("ascii")).toBe("abc");

  buf1[0] = 33;
  expect(buf2.toString("ascii")).toBe("!bc");

  const buf3 = Buffer.from("buffer");
  const sub = buf3.subarray(-5, -2);
  expect(sub.toString()).toBe("uff");

  const sliced = buf3.slice(0, 3);
  expect(sliced.toString()).toBe("buf");
});

test("buf.write", () => {
  const buf = Buffer.alloc(256);
  const len = buf.write("\u00bd + \u00bc = \u00be", 0);
  expect(len).toBe(12);
  expect(buf.toString("utf8", 0, len)).toBe("\u00bd + \u00bc = \u00be");

  const b2 = Buffer.alloc(10);
  const l2 = b2.write("abcd", 8);
  expect(l2).toBe(2);
  expect(b2.toString("utf8", 8, 10)).toBe("ab");
});

test("buf.copy", () => {
  const buf1 = Buffer.allocUnsafe(26);
  const buf2 = Buffer.allocUnsafe(26).fill("!");

  for (let i = 0; i < 26; i++) {
    buf1[i] = i + 97;
  }

  buf1.copy(buf2, 8, 16, 20);
  expect(buf2.toString("ascii", 0, 25)).toBe("!!!!!!!!qrst!!!!!!!!!!!!!");

  const overlapBuf = Buffer.allocUnsafe(26);
  for (let i = 0; i < 26; i++) {
    overlapBuf[i] = i + 97;
  }
  overlapBuf.copy(overlapBuf, 0, 4, 10);
  expect(overlapBuf.toString()).toBe("efghijghijklmnopqrstuvwxyz");
});

test("buf.fill", () => {
  const b = Buffer.allocUnsafe(50).fill("h");
  expect(b.toString()).toBe("hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh");

  const c = Buffer.allocUnsafe(5).fill("");
  expect(Array.from(c)).toEqual([0, 0, 0, 0, 0]);

  const multibyte = Buffer.allocUnsafe(5).fill("\u0222");
  expect(Array.from(multibyte)).toEqual([0xc8, 0xa2, 0xc8, 0xa2, 0xc8]);
});

test("buf.indexOf, lastIndexOf, includes", () => {
  const buf = Buffer.from("this is a buffer");
  expect(buf.indexOf("this")).toBe(0);
  expect(buf.indexOf("is")).toBe(2);
  expect(buf.indexOf(97)).toBe(8);
  expect(buf.indexOf("example")).toBe(-1);

  expect(buf.includes("this")).toBe(true);
  expect(buf.includes("example")).toBe(false);

  const buf2 = Buffer.from("this buffer is a buffer");
  expect(buf2.lastIndexOf("buffer")).toBe(17);
});

test("buf.reverse, swap16, swap32, swap64", () => {
  const rBuf = Buffer.from([1, 2, 3]);
  rBuf.reverse();
  expect(Array.from(rBuf)).toEqual([3, 2, 1]);

  const buf1 = Buffer.from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
  buf1.swap16();
  expect(Array.from(buf1)).toEqual([2, 1, 4, 3, 6, 5, 8, 7]);

  const buf2 = Buffer.from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
  buf2.swap32();
  expect(Array.from(buf2)).toEqual([4, 3, 2, 1, 8, 7, 6, 5]);

  const buf3 = Buffer.from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
  buf3.swap64();
  expect(Array.from(buf3)).toEqual([8, 7, 6, 5, 4, 3, 2, 1]);

  expect(() => Buffer.from([1, 2, 3]).swap16()).toThrow();
  expect(() => Buffer.from([1, 2, 3]).swap32()).toThrow();
  expect(() => Buffer.from([1, 2, 3]).swap64()).toThrow();
});

test("buf.toJSON", () => {
  const buf = Buffer.from([1, 2, 3, 4, 5]);
  const json = JSON.stringify(buf);
  expect(json).toBe('{"type":"Buffer","data":[1,2,3,4,5]}');
  expect(JSON.parse(json)).toEqual({ type: "Buffer", data: [1, 2, 3, 4, 5] });
});

test("numeric read and write methods", () => {
  const buf = Buffer.alloc(16);

  buf.writeInt8(-2, 0);
  expect(buf.readInt8(0)).toBe(-2);

  buf.writeUInt8(254, 1);
  expect(buf.readUInt8(1)).toBe(254);
  expect(buf.readUint8(1)).toBe(254);

  buf.writeInt16LE(0x0304, 2);
  expect(buf.readInt16LE(2)).toBe(0x0304);

  buf.writeInt16BE(0x0102, 4);
  expect(buf.readInt16BE(4)).toBe(0x0102);

  buf.writeUInt16LE(0xdead, 2);
  expect(buf.readUInt16LE(2)).toBe(0xdead);
  expect(buf.readUint16LE(2)).toBe(0xdead);

  buf.writeUInt16BE(0xbeef, 4);
  expect(buf.readUInt16BE(4)).toBe(0xbeef);
  expect(buf.readUint16BE(4)).toBe(0xbeef);

  buf.writeInt32LE(0x05060708, 6);
  expect(buf.readInt32LE(6)).toBe(0x05060708);

  buf.writeInt32BE(0x01020304, 10);
  expect(buf.readInt32BE(10)).toBe(0x01020304);

  buf.writeUInt32LE(0xfeedface, 6);
  expect(buf.readUInt32LE(6)).toBe(0xfeedface);
  expect(buf.readUint32LE(6)).toBe(0xfeedface);

  buf.writeUInt32BE(0xfeedface, 10);
  expect(buf.readUInt32BE(10)).toBe(0xfeedface);
  expect(buf.readUint32BE(10)).toBe(0xfeedface);

  const floatBuf = Buffer.alloc(8);
  floatBuf.writeFloatLE(123.45, 0);
  expect(Math.fround(floatBuf.readFloatLE(0))).toBe(Math.fround(123.45));

  floatBuf.writeFloatBE(123.45, 4);
  expect(Math.fround(floatBuf.readFloatBE(4))).toBe(Math.fround(123.45));

  floatBuf.writeDoubleLE(123.456, 0);
  expect(floatBuf.readDoubleLE(0)).toBe(123.456);

  floatBuf.writeDoubleBE(123.456789, 0);
  expect(floatBuf.readDoubleBE(0)).toBe(123.456789);

  const bigBuf = Buffer.alloc(8);
  bigBuf.writeBigInt64BE(0x0102030405060708n, 0);
  expect(bigBuf.readBigInt64BE(0)).toBe(0x0102030405060708n);

  bigBuf.writeBigInt64LE(0x0102030405060708n, 0);
  expect(bigBuf.readBigInt64LE(0)).toBe(0x0102030405060708n);

  bigBuf.writeBigUInt64LE(0xdecafafecacefaden, 0);
  expect(bigBuf.readBigUInt64LE(0)).toBe(0xdecafafecacefaden);
  expect(bigBuf.readBigUint64LE(0)).toBe(0xdecafafecacefaden);

  bigBuf.writeBigUInt64BE(0xdecafafecacefaden, 0);
  expect(bigBuf.readBigUInt64BE(0)).toBe(0xdecafafecacefaden);
  expect(bigBuf.readBigUint64BE(0)).toBe(0xdecafafecacefaden);

  bigBuf.writeBigUint64BE(0xdecafafecacefaden, 0);
  bigBuf.writeBigUint64LE(0xdecafafecacefaden, 0);

  const varBuf = Buffer.alloc(6);
  varBuf.writeUIntLE(0x1234567890ab, 0, 6);
  expect(varBuf.readUIntLE(0, 6).toString(16)).toBe("1234567890ab");
  expect(varBuf.readUintLE(0, 6).toString(16)).toBe("1234567890ab");

  varBuf.writeUintLE(0x1234567890ab, 0, 6);
  varBuf.writeUIntBE(0x1234567890ab, 0, 6);
  expect(varBuf.readUIntBE(0, 6).toString(16)).toBe("1234567890ab");
  expect(varBuf.readUintBE(0, 6).toString(16)).toBe("1234567890ab");
  varBuf.writeUintBE(0x1234567890ab, 0, 6);

  varBuf.writeIntLE(0x1234567890ab, 0, 6);
  expect(varBuf.readIntLE(0, 6)).toBe(0x1234567890ab);

  varBuf.writeIntBE(0x1234567890ab, 0, 6);
  expect(varBuf.readIntBE(0, 6)).toBe(0x1234567890ab);

  const varBuf4 = Buffer.alloc(4);
  varBuf4.writeIntLE(0, 0, 4);
  expect(varBuf4.readIntLE(0, 4)).toBe(0);

  const varBuf4Neg = Buffer.alloc(4);
  varBuf4Neg.writeIntLE(-123456, 0, 4);
  expect(varBuf4Neg.readIntLE(0, 4)).toBe(-123456);
});

test("atob and btoa", () => {
  const encoded = btoa("hello");
  expect(encoded).toBe("aGVsbG8=");
  expect(atob(encoded)).toBe("hello");
});

test("isAscii, isUtf8, and resolveObjectURL", () => {
  expect(isAscii(Buffer.from("hello"))).toBe(true);
  expect(isUtf8(Buffer.from("tést"))).toBe(true);
  expect(() => resolveObjectURL("blob:nodedata:test")).toThrow();
});

test("Blob class", () => {
  const blob = new Blob(["hello ", "world"], { type: "text/plain" });
  expect(blob.size).toBe(11);
  expect(blob.type).toBe("text/plain");
  expect(blob.text()).toBe("hello world");

  const sliced = blob.slice(0, 5);
  expect(sliced.size).toBe(5);
  expect(sliced.text()).toBe("hello");

  expect(blob.bytes() instanceof Uint8Array).toBe(true);
  expect(blob.arrayBuffer() instanceof ArrayBuffer).toBe(true);
});

test("File class", () => {
  const file = new File(["hello world"], "hello.txt", { type: "text/plain" });
  expect(file.name).toBe("hello.txt");
  expect(file.size).toBe(11);
  expect(file.type).toBe("text/plain");
  expect(typeof file.lastModified).toBe("number");
  expect(file.text()).toBe("hello world");

  const sliced = file.slice(0, 5);
  expect(sliced.size).toBe(5);
  expect(sliced.text()).toBe("hello");

  expect(file.bytes() instanceof Uint8Array).toBe(true);
  expect(file.arrayBuffer() instanceof ArrayBuffer).toBe(true);
});

test("transcode and constants", () => {
  const newBuf = transcode(Buffer.from("€"), "utf8", "ascii");
  expect(newBuf.toString("ascii")).toBe("?");

  expect(constants.MAX_LENGTH).toBe(kMaxLength);
  expect(constants.MAX_STRING_LENGTH).toBe(kStringMaxLength);
  expect(INSPECT_MAX_BYTES).toBe(50);
});
