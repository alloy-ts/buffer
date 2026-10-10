import { expect, test } from "vite-plus/test";
import {
  atob,
  Blob,
  btoa,
  Buffer,
  constants,
  File,
  INSPECT_MAX_BYTES,
  isAscii,
  isUtf8,
  kMaxLength,
  kStringMaxLength,
  MAX_LENGTH,
  MAX_STRING_LENGTH,
  resolveObjectURL,
  SlowBuffer,
  transcode,
} from "./main.ts";

test("Buffer extends Uint8Array", () => {
  const buf = Buffer.alloc(10);
  expect(buf instanceof Uint8Array).toBe(true);
  expect(buf instanceof Buffer).toBe(true);
  expect(Buffer.isBuffer(buf)).toBe(true);
  expect(Buffer.isBuffer(new Uint8Array(10))).toBe(false);
  expect(Buffer.isBuffer("string")).toBe(false);
});

test("Buffer.alloc and allocUnsafe", () => {
  const buf1 = Buffer.alloc(10);
  expect(buf1.length).toBe(10);
  expect(Array.from(buf1)).toEqual([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

  const buf2 = Buffer.alloc(10, 1);
  expect(Array.from(buf2)).toEqual([1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

  const buf3 = Buffer.alloc(5, "a");
  expect(Array.from(buf3)).toEqual([0x61, 0x61, 0x61, 0x61, 0x61]);

  const buf4 = Buffer.alloc(11, "aGVsbG8gd29ybGQ=", "base64");
  expect(buf4.toString()).toBe("hello world");

  const buf5 = Buffer.allocUnsafe(10);
  expect(buf5.length).toBe(10);
});

test("Buffer.from variations", () => {
  const buf1 = Buffer.from([1, 2, 3]);
  expect(Array.from(buf1)).toEqual([1, 2, 3]);

  const buf2 = Buffer.from([257, 257.5, -255, 1]);
  expect(Array.from(buf2)).toEqual([1, 1, 1, 1]);

  const buf3 = Buffer.from("tést");
  expect(Array.from(buf3)).toEqual([116, 195, 169, 115, 116]);
  expect(buf3.toString()).toBe("tést");

  const buf4 = Buffer.from("tést", "latin1");
  expect(Array.from(buf4)).toEqual([0x74, 0xe9, 0x73, 0x74]);

  const buf5 = Buffer.from("7468697320697320612074c3a97374", "hex");
  expect(buf5.toString("utf8")).toBe("this is a tést");

  const ab = new ArrayBuffer(10);
  const buf6 = Buffer.from(ab, 2, 4);
  expect(buf6.length).toBe(4);
  expect(buf6.buffer).toBe(ab);

  const u16 = new Uint16Array([0, 0xffff]);
  const buf7 = Buffer.copyBytesFrom(u16, 1, 1);
  expect(buf7.length).toBe(2);
  expect(buf7[0]).toBe(255);
  expect(buf7[1]).toBe(255);
});

test("Buffer.concat and Buffer.compare", () => {
  const buf1 = Buffer.from("ABC");
  const buf2 = Buffer.from("BCD");
  const buf3 = Buffer.from("ABCD");

  expect(Buffer.compare(buf1, buf1)).toBe(0);
  expect(Buffer.compare(buf1, buf2)).toBe(-1);
  expect(Buffer.compare(buf2, buf1)).toBe(1);

  const concat = Buffer.concat([buf1, buf2, buf3]);
  expect(concat.toString()).toBe("ABCBCDABCD");
});

test("Buffer.prototype instance methods", () => {
  const buf = Buffer.alloc(256);
  const len = buf.write("½ + ¼ = ¾", 0);
  expect(len).toBe(12);
  expect(buf.toString("utf8", 0, len)).toBe("½ + ¼ = ¾");

  const json = buf.subarray(0, 3).toJSON();
  expect(json).toEqual({ type: "Buffer", data: [0xc2, 0xbd, 0x20] });

  const bufA = Buffer.from("ABC");
  const bufB = Buffer.from("414243", "hex");
  expect(bufA.equals(bufB)).toBe(true);

  const bCopy = Buffer.alloc(10);
  bufA.copy(bCopy, 2);
  expect(Array.from(bCopy.subarray(2, 5))).toEqual([65, 66, 67]);

  const sub = bufA.subarray(0, 2);
  expect(sub.toString()).toBe("AB");
  bufA[0] = 33;
  expect(sub.toString()).toBe("!B");

  const searchBuf = Buffer.from("this is a buffer");
  expect(searchBuf.indexOf("this")).toBe(0);
  expect(searchBuf.indexOf("is")).toBe(2);
  expect(searchBuf.includes("buffer")).toBe(true);
  expect(searchBuf.lastIndexOf("is")).toBe(5);

  const swapBuf = Buffer.from([0x1, 0x2, 0x3, 0x4]);
  swapBuf.swap16();
  expect(Array.from(swapBuf)).toEqual([0x2, 0x1, 0x4, 0x3]);

  const swapBuf32 = Buffer.from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
  swapBuf32.swap32();
  expect(Array.from(swapBuf32)).toEqual([0x4, 0x3, 0x2, 0x1, 0x8, 0x7, 0x6, 0x5]);
});

test("Buffer numeric read and write", () => {
  const buf = Buffer.alloc(16);

  buf.writeUInt8(0x42, 0);
  expect(buf.readUInt8(0)).toBe(0x42);

  buf.writeInt8(-2, 1);
  expect(buf.readInt8(1)).toBe(-2);

  buf.writeUInt16LE(0xdead, 2);
  expect(buf.readUInt16LE(2)).toBe(0xdead);

  buf.writeUInt16BE(0xbeef, 4);
  expect(buf.readUInt16BE(4)).toBe(0xbeef);

  buf.writeUInt32LE(0xfeedface, 6);
  expect(buf.readUInt32LE(6)).toBe(0xfeedface);

  buf.writeBigUInt64BE(0xdecafafecacefaden, 0);
  expect(buf.readBigUInt64BE(0)).toBe(0xdecafafecacefaden);

  buf.writeFloatLE(1.5, 0);
  expect(buf.readFloatLE(0)).toBe(1.5);

  buf.writeDoubleBE(123.456, 0);
  expect(buf.readDoubleBE(0)).toBe(123.456);

  buf.writeUIntBE(0x1234567890ab, 0, 6);
  expect(buf.readUIntBE(0, 6)).toBe(0x1234567890ab);
});

test("Blob and File", async () => {
  const blob = new Blob(["hello ", "world"], { type: "text/plain" });
  expect(blob.size).toBe(11);
  expect(blob.type).toBe("text/plain");

  const text = await blob.text();
  expect(text).toBe("hello world");

  const bytes = await blob.bytes();
  expect(bytes.length).toBe(11);

  const sliced = blob.slice(0, 5);
  expect(await sliced.text()).toBe("hello");

  const file = new File(["foo", "bar"], "test.txt", {
    type: "text/plain",
    lastModified: 1000,
  });
  expect(file.name).toBe("test.txt");
  expect(file.lastModified).toBe(1000);
  expect(file.size).toBe(6);
  expect(await file.text()).toBe("foobar");
});

test("Standalone functions and constants", () => {
  expect(atob("aGVsbG8=")).toBe("hello");
  expect(btoa("hello")).toBe("aGVsbG8=");

  expect(isAscii(Buffer.from("hello"))).toBe(true);
  expect(isAscii(Buffer.from([200]))).toBe(false);

  expect(isUtf8(Buffer.from("tést"))).toBe(true);

  const tc = transcode(Buffer.from("€"), "utf8", "ascii");
  expect(tc.toString("ascii")).toBe("?");

  expect(resolveObjectURL("blob:nodedata:123")).toBeUndefined();

  expect(constants.MAX_LENGTH).toBe(MAX_LENGTH);
  expect(constants.MAX_STRING_LENGTH).toBe(MAX_STRING_LENGTH);
  expect(INSPECT_MAX_BYTES).toBe(50);
  expect(kMaxLength).toBe(MAX_LENGTH);
  expect(kStringMaxLength).toBe(MAX_STRING_LENGTH);
  expect(SlowBuffer).toBe(Buffer);
});
