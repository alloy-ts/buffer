import { expect, test } from "vite-plus/test";
import {
  Blob,
  Buffer,
  File,
  atob,
  btoa,
  constants,
  isAscii,
  isLatin1,
  isUtf8,
  transcode,
} from "../index.js";

test("Buffer.alloc and Buffer.allocUnsafe", () => {
  const buf1 = Buffer.alloc(10);
  expect(buf1.length).toBe(10);

  const buf2 = Buffer.alloc(5, 0x61);
  expect(buf2.toString()).toBe("aaaaa");

  const buf3 = Buffer.alloc(6, "hi", "utf8");
  expect(buf3.toString()).toBe("hihihi");

  const bufUnsafe = Buffer.allocUnsafe(8);
  expect(bufUnsafe.length).toBe(8);
});

test("Buffer.from overloading with various encodings and types", () => {
  const bufUtf8 = Buffer.from("hello world", "utf8");
  expect(bufUtf8.toString("hex")).toBe("68656c6c6f20776f726c64");
  expect(bufUtf8.toString("base64")).toBe("aGVsbG8gd29ybGQ=");

  const bufHex = Buffer.from("1ag123", "hex");
  expect(bufHex.toString("hex")).toBe("1a");

  const bufBase64 = Buffer.from("aGVsbG8gd29ybGQ=", "base64");
  expect(bufBase64.toString("utf8")).toBe("hello world");

  const bufBase64Url = Buffer.from("aGVsbG8", "base64url");
  expect(bufBase64Url.toString("utf8")).toBe("hello");

  const bufLatin1 = Buffer.from("café", "latin1");
  expect(bufLatin1.length).toBe(4);
  expect(bufLatin1.toString("latin1")).toBe("café");

  const bufUtf16 = Buffer.from("hello", "utf16le");
  expect(bufUtf16.length).toBe(10);
  expect(bufUtf16.toString("utf16le")).toBe("hello");

  const bufArray = Buffer.from([257, 257.5, -255, 49]);
  expect(bufArray.length).toBe(4);
  expect(bufArray.readUint8(0)).toBe(1);
  expect(bufArray.readUint8(1)).toBe(1);
  expect(bufArray.readUint8(2)).toBe(1);
  expect(bufArray.readUint8(3)).toBe(49);

  const bufCopy = Buffer.from(bufUtf8);
  expect(bufCopy.toString()).toBe("hello world");

  const ab = new ArrayBuffer(8);
  const bufAB = Buffer.from(ab, 0, 4);
  expect(bufAB.length).toBe(4);

  const bufObj = Buffer.from(new String("this is a test"));
  expect(bufObj.toString()).toBe("this is a test");
});

test("Buffer static helpers: byteLength, stringLength, compare, concat, isBuffer, isEncoding", () => {
  const str = "½ + ¼ = ¾";
  expect(Buffer.byteLength(str, "utf8")).toBe(12);

  const buf = Buffer.from("€ 100", "utf8");
  expect(Buffer.stringLength(buf.bytes(), "utf8")).toBe(5);

  const b1 = Buffer.from("1234");
  const b2 = Buffer.from("0123");
  expect(Buffer.compare(b1.bytes(), b2.bytes())).toBe(1);
  expect(Buffer.compare(b2.bytes(), b1.bytes())).toBe(-1);
  expect(b1.compare(b1.bytes())).toBe(0);

  const concatBuf = Buffer.concat([b2, b1]);
  expect(concatBuf.toString()).toBe("01231234");

  expect(Buffer.isBuffer(b1)).toBe(true);
  expect(Buffer.isBuffer("string")).toBe(false);

  expect(Buffer.isEncoding("utf8")).toBe(true);
  expect(Buffer.isEncoding("hex")).toBe(true);
  expect(Buffer.isEncoding("invalid-encoding")).toBe(false);
});

test("Buffer integer read/write methods and aliases", () => {
  const buf = Buffer.alloc(16);

  buf.writeUint8(0x12, 0);
  expect(buf.readUint8(0)).toBe(0x12);
  expect(buf.readUInt8(0)).toBe(0x12);

  buf.writeInt8(-5, 1);
  expect(buf.readInt8(1)).toBe(-5);

  buf.writeUint16LE(0x1234, 2);
  expect(buf.readUint16LE(2)).toBe(0x1234);

  buf.writeUint16BE(0x1234, 4);
  expect(buf.readUint16BE(4)).toBe(0x1234);

  buf.writeUint32LE(0x12345678, 6);
  expect(buf.readUint32LE(6)).toBe(0x12345678);

  buf.writeUint32BE(0x12345678, 10);
  expect(buf.readUint32BE(10)).toBe(0x12345678);
});

test("Buffer float, double, and BigInt methods", () => {
  const buf = Buffer.alloc(32);

  buf.writeFloatLE(123.45, 0);
  expect(Math.fround(buf.readFloatLE(0))).toBe(Math.fround(123.45));

  buf.writeDoubleLE(123.456789, 4);
  expect(buf.readDoubleLE(4)).toBe(123.456789);

  buf.writeBigUint64LE(1234567890123456789n, 12);
  expect(buf.readBigUint64LE(12)).toBe(1234567890123456789n);

  buf.writeBigInt64BE(-1234567890123456789n, 20);
  expect(buf.readBigInt64BE(20)).toBe(-1234567890123456789n);
});

test("Buffer variable width read/write methods including negative ints", () => {
  const buf = Buffer.alloc(8);
  buf.writeUintBE(0x1234567890ab, 0, 6);
  expect(buf.readUintBE(0, 6)).toBe(0x1234567890ab);

  const buf2 = Buffer.alloc(8);
  buf2.writeUintLE(0x1234567890ab, 0, 6);
  expect(buf2.readUintLE(0, 6)).toBe(0x1234567890ab);

  const buf3 = Buffer.alloc(8);
  buf3.writeIntBE(-2, 0, 2);
  expect(buf3.readIntBE(0, 2)).toBe(-2);
});

test("Buffer search and utility methods: indexOf, lastIndexOf, includes, copy, fill, swap, toJSON", () => {
  const buf = Buffer.from("this is a buffer");
  expect(buf.indexOf("is")).toBe(2);
  expect(buf.indexOf(97)).toBe(8); // 'a'
  expect(buf.indexOf("\u03a3", "utf16le")).toBe(-1);
  expect(buf.includes("buffer")).toBe(true);
  expect(buf.includes("missing")).toBe(false);

  const buf2 = Buffer.from("this buffer is a buffer");
  expect(buf2.lastIndexOf("buffer")).toBe(17);
  expect(buf2.lastIndexOf(97)).toBe(15);

  const source = Buffer.from("abcdef");
  const target = Buffer.alloc(6);
  source.copy(target, 0, 2, 5);
  expect(target.toString("utf8", 0, 3)).toBe("cde");

  const fillBuf = Buffer.alloc(5);
  fillBuf.fill("h");
  expect(fillBuf.toString()).toBe("hhhhh");

  const fillHex = Buffer.alloc(5);
  fillHex.fill("aazz", "hex");
  expect(fillHex.toString("hex")).toBe("aaaaaaaaaa");

  const swapBuf = Buffer.from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
  swapBuf.swap16();
  expect(swapBuf.toJson().data).toEqual([0x2, 0x1, 0x4, 0x3, 0x6, 0x5, 0x8, 0x7]);

  swapBuf.swap16(); // swap back
  swapBuf.swap32();
  expect(swapBuf.toJson().data).toEqual([0x4, 0x3, 0x2, 0x1, 0x8, 0x7, 0x6, 0x5]);

  const json = fillBuf.toJson();
  expect(json.type).toBe("Buffer");
  expect(json.data).toEqual([104, 104, 104, 104, 104]);
});

test("Buffer TypedArray prototype methods: at, copyWithin, entries, keys, values, iterator, set, setFromBase64, setFromHex, sort, toReversed, with", () => {
  const buf = Buffer.from([10, 20, 30, 40, 50]);

  expect(buf.at(0)).toBe(10);
  expect(buf.at(-1)).toBe(50);
  expect(buf.at(100)).toBeUndefined();

  expect(buf.BYTES_PER_ELEMENT).toBe(1);
  expect(buf.byteOffset).toBe(0);
  expect(buf.byteLength).toBe(5);
  expect(buf.buffer).toBeInstanceOf(ArrayBuffer);

  const arrFromIter = Array.from(buf);
  expect(arrFromIter).toEqual([10, 20, 30, 40, 50]);

  const keys = Array.from(buf.keys());
  expect(keys).toEqual([0, 1, 2, 3, 4]);

  const values = Array.from(buf.values());
  expect(values).toEqual([10, 20, 30, 40, 50]);

  const entries = Array.from(buf.entries());
  expect(entries).toEqual([
    [0, 10],
    [1, 20],
    [2, 30],
    [3, 40],
    [4, 50],
  ]);

  expect(buf.every((x) => x > 0)).toBe(true);
  expect(buf.some((x) => x === 30)).toBe(true);
  expect(buf.find((x) => x > 25)).toBe(30);
  expect(buf.findIndex((x) => x === 30)).toBe(2);

  const filtered = buf.filter((x) => x > 25);
  expect(filtered.toString("hex")).toBe(Buffer.from([30, 40, 50]).toString("hex"));

  const mapped = buf.map((x) => x + 1);
  expect(mapped.toString("hex")).toBe(Buffer.from([11, 21, 31, 41, 51]).toString("hex"));

  const rev = buf.toReversed();
  expect(Array.from(rev)).toEqual([50, 40, 30, 20, 10]);

  const withBuf = buf.with(0, 99);
  expect(withBuf.at(0)).toBe(99);

  const mutBuf = Buffer.from([30, 10, 50, 20, 40]);
  mutBuf.sort();
  expect(Array.from(mutBuf)).toEqual([10, 20, 30, 40, 50]);

  const setBuf = Buffer.alloc(5);
  setBuf.set([1, 2, 3], 1);
  expect(Array.from(setBuf)).toEqual([0, 1, 2, 3, 0]);

  const b64Res = setBuf.setFromBase64("aGVsbG8=");
  expect(b64Res.written).toBe(5);

  const hexRes = setBuf.setFromHex("68656c6c6f");
  expect(hexRes.written).toBe(5);
  expect(setBuf.toString()).toBe("hello");

  expect(setBuf.toHex()).toBe("68656c6c6f");
  expect(setBuf.toBase64()).toBe("aGVsbG8=");
});

test("Blob class functionality with Promises and slice prototype", async () => {
  const blob = new Blob(["hello ", "world"], { type: "text/plain" });
  expect(blob.size).toBe(11);
  expect(blob.type).toBe("text/plain");

  const textPromise = blob.text();
  expect(textPromise).toBeInstanceOf(Promise);
  expect(await textPromise).toBe("hello world");

  const abPromise = blob.arrayBuffer();
  expect(abPromise).toBeInstanceOf(Promise);
  expect((await abPromise).byteLength).toBe(11);

  const bytesPromise = blob.bytes();
  expect(bytesPromise).toBeInstanceOf(Promise);
  expect((await bytesPromise).length).toBe(11);

  const sliced = blob.slice(0, 5, "text/plain");
  expect(sliced.size).toBe(5);
  expect(await sliced.text()).toBe("hello");
});

test("File class functionality with Promises", async () => {
  const file = new File(["file content"], "test.txt", {
    type: "text/plain",
    lastModified: 1600000000000,
  });
  expect(file.name).toBe("test.txt");
  expect(file.lastModified).toBe(1600000000000);
  expect(file.size).toBe(12);
  expect(file.type).toBe("text/plain");
  expect(await file.text()).toBe("file content");

  const slicedFile = file.slice(0, 4, "text/plain");
  expect(slicedFile.size).toBe(4);
  expect(await slicedFile.text()).toBe("file");
});

test("Module level functions and constants: atob, btoa, isAscii, isLatin1, isUtf8, transcode, constants", () => {
  expect(atob("aGVsbG8=")).toBe("hello");
  expect(btoa("hello")).toBe("aGVsbG8=");

  const asciiBuf = Buffer.from("hello");
  expect(isAscii(asciiBuf)).toBe(true);

  expect(isLatin1("hello")).toBe(true);
  expect(isLatin1("café")).toBe(true);
  expect(isLatin1("€")).toBe(false);

  expect(isUtf8(asciiBuf)).toBe(true);

  expect(constants.MAX_LENGTH).toBe(2147483647);
  expect(constants.INSPECT_MAX_BYTES).toBe(50);

  const transcoded = transcode(asciiBuf, "utf8", "ascii");
  expect(Buffer.fromBuffer(transcoded).toString("ascii")).toBe("hello");
});
