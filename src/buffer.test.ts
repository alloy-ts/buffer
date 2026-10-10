import { describe, expect, test } from "vite-plus/test";
import {
  Blob,
  Buffer,
  File,
  INSPECT_MAX_BYTES,
  atob,
  btoa,
  constants,
  isAscii,
  isLatin1,
  isUtf8,
  kMaxLength,
  kStringMaxLength,
  transcode,
} from "../index.js";

describe("Buffer static methods", () => {
  test("Buffer.alloc and allocUnsafe", () => {
    const buf1 = Buffer.alloc(10);
    expect(buf1.length).toBe(10);
    expect(buf1.getByte(0)).toBe(0);

    const buf2 = Buffer.alloc(5, 65);
    expect(buf2.length).toBe(5);
    expect(buf2.getByte(0)).toBe(65);
    expect(buf2.getByte(4)).toBe(65);

    const buf3 = Buffer.allocUnsafe(10);
    expect(buf3.length).toBe(10);
  });

  test("Buffer.from variants", () => {
    const buf1 = Buffer.fromArray([1, 2, 3]);
    expect(buf1.length).toBe(3);
    expect(buf1.getByte(0)).toBe(1);
    expect(buf1.getByte(1)).toBe(2);
    expect(buf1.getByte(2)).toBe(3);

    const buf2 = Buffer.fromString("hello world", "utf8");
    expect(buf2.length).toBe(11);
    expect(buf2.toString()).toBe("hello world");

    const bufHex = Buffer.fromString("68656c6c6f", "hex");
    expect(bufHex.toString()).toBe("hello");

    const bufBase64 = Buffer.fromString("aGVsbG8=", "base64");
    expect(bufBase64.toString()).toBe("hello");

    const buf3 = Buffer.fromBuffer(buf1.bytes());
    expect(buf3.length).toBe(3);
    expect(buf3.equals(buf1.bytes())).toBe(true);

    const bufObj = Buffer.fromObject("test string", "utf8");
    expect(bufObj.toString()).toBe("test string");

    const bufAB = Buffer.fromArrayBuffer(buf1.bytes(), 1, 2);
    expect(bufAB.length).toBe(2);
    expect(bufAB.getByte(0)).toBe(2);
  });

  test("Buffer.concat and compareStatic", () => {
    const b1 = Buffer.fromString("hello ");
    const b2 = Buffer.fromString("world");
    const concatenated = Buffer.concat([b1.bytes(), b2.bytes()]);
    expect(concatenated.toString()).toBe("hello world");

    expect(Buffer.compareStatic(b1.bytes(), b1.bytes())).toBe(0);
  });

  test("Buffer.isBuffer and Buffer.isEncoding", () => {
    expect(Buffer.isEncoding("utf8")).toBe(true);
    expect(Buffer.isEncoding("latin1")).toBe(true);
    expect(Buffer.isEncoding("hex")).toBe(true);
    expect(Buffer.isEncoding("invalid-encoding")).toBe(false);
  });

  test("Buffer.byteLength and stringLength", () => {
    expect(Buffer.byteLength("½ + ¼ = ¾", "utf8")).toBe(12);
    expect(Buffer.stringLength(Buffer.fromString("hello").bytes(), "utf8")).toBe(5);
  });
});

describe("Buffer instance methods", () => {
  test("read and write integers", () => {
    const buf = Buffer.alloc(16);

    buf.writeUInt8(255, 0);
    expect(buf.readUInt8(0)).toBe(255);
    expect(buf.readUint8(0)).toBe(255);

    buf.writeInt8(-128, 1);
    expect(buf.readInt8(1)).toBe(-128);

    buf.writeUInt16BE(0x1234, 2);
    expect(buf.readUInt16BE(2)).toBe(0x1234);
    expect(buf.readUint16BE(2)).toBe(0x1234);

    buf.writeUInt16LE(0x1234, 4);
    expect(buf.readUInt16LE(4)).toBe(0x1234);
    expect(buf.readUint16LE(4)).toBe(0x1234);

    buf.writeUInt32BE(0x12345678, 6);
    expect(buf.readUInt32BE(6)).toBe(0x12345678);
    expect(buf.readUint32BE(6)).toBe(0x12345678);

    buf.writeUInt32LE(0x12345678, 10);
    expect(buf.readUInt32LE(10)).toBe(0x12345678);
    expect(buf.readUint32LE(10)).toBe(0x12345678);
  });

  test("read and write floats / doubles / BigInts", () => {
    const buf = Buffer.alloc(32);

    buf.writeFloatBE(123.5, 0);
    expect(buf.readFloatBE(0)).toBe(123.5);

    buf.writeDoubleLE(123456.789, 4);
    expect(buf.readDoubleLE(4)).toBeCloseTo(123456.789);

    buf.writeBigUInt64BE(0x1234567890abcdefn, 12);
    expect(buf.readBigUInt64BE(12)).toBe(0x1234567890abcdefn);
    expect(buf.readBigUint64BE(12)).toBe(0x1234567890abcdefn);

    buf.writeBigInt64LE(-1234567890n, 20);
    expect(buf.readBigInt64LE(20)).toBe(-1234567890n);
  });

  test("variable byteLength read/write", () => {
    const buf = Buffer.alloc(8);
    buf.writeUIntBE(0x123456, 0, 3);
    expect(buf.readUIntBE(0, 3)).toBe(0x123456);
    expect(buf.readUintBE(0, 3)).toBe(0x123456);

    buf.writeUIntLE(0x654321, 3, 3);
    expect(buf.readUIntLE(3, 3)).toBe(0x654321);
    expect(buf.readUintLE(3, 3)).toBe(0x654321);
  });

  test("string write, toString, toJSON", () => {
    const buf = Buffer.alloc(32);
    const written = buf.write("hello world", 0, 11, "utf8");
    expect(written).toBe(11);
    expect(buf.toString("utf8", 0, 11)).toBe("hello world");

    const json = buf.toJSON();
    expect(json.type).toBe("Buffer");
    expect(json.data.slice(0, 5)).toEqual([104, 101, 108, 108, 111]);
  });

  test("fill, indexOf, includes, equals, compare, copy", () => {
    const buf = Buffer.alloc(10);
    buf.fillNumber(65, 0, 5);
    expect(buf.toString("utf8", 0, 5)).toBe("AAAAA");

    buf.fillString("b", 5, 10);
    expect(buf.toString("utf8", 5, 10)).toBe("bbbbb");

    expect(buf.indexOfNumber(98, 0)).toBe(5);
    expect(buf.indexOfString("b", 0)).toBe(5);
    expect(buf.includesNumber(65, 0)).toBe(true);
    expect(buf.includesString("a", 0)).toBe(false);

    const b1 = Buffer.fromString("abc");
    const b2 = Buffer.fromString("abc");
    const b3 = Buffer.fromString("def");
    expect(b1.equals(b2.bytes())).toBe(true);
    expect(b1.equals(b3.bytes())).toBe(false);
    expect(b1.compare(b3.bytes())).toBeLessThan(0);

    const target = Buffer.alloc(5);
    b1.copy(target, 0, 0, 3);
    expect(target.toString("utf8", 0, 3)).toBe("abc");
  });

  test("swap16, swap32, swap64, reverse, entries, keys, values", () => {
    const buf = Buffer.fromArray([1, 2, 3, 4]);
    buf.swap16();
    expect(buf.getByte(0)).toBe(2);
    expect(buf.getByte(1)).toBe(1);

    buf.swap16(); // swap back
    buf.swap32();
    expect(buf.getByte(0)).toBe(4);
    expect(buf.getByte(3)).toBe(1);

    buf.reverse();
    expect(buf.getByte(0)).toBe(1);

    expect(buf.keys()).toEqual([0, 1, 2, 3]);
    expect(buf.values()).toEqual([1, 2, 3, 4]);
    expect(buf.entries()).toEqual([
      [0, 1],
      [1, 2],
      [2, 3],
      [3, 4],
    ]);
  });
});

describe("Blob class", () => {
  test("Blob properties and async methods", async () => {
    const blob = new Blob(["hello ", "world"], { type: "text/plain" });
    expect(blob.size).toBe(11);
    expect(blob.type).toBe("text/plain");

    expect(await blob.text()).toBe("hello world");

    const bytes = await blob.bytes();
    expect(bytes.length).toBe(11);

    const sliced = blob.slice(0, 5, "text/plain");
    expect(sliced.size).toBe(5);
    expect(await sliced.text()).toBe("hello");
  });

  test("Blob line endings option", async () => {
    const blobNative = new Blob(["hello\r\nworld"], { endings: "native" });
    const text = await blobNative.text();
    expect(text).toContain("hello");
    expect(text).toContain("world");
  });
});

describe("File class", () => {
  test("File properties and inherited Blob behavior", async () => {
    const file = new File(["test content"], "example.txt", {
      type: "text/plain",
      lastModified: 1000000,
    });
    expect(file.name).toBe("example.txt");
    expect(file.lastModified).toBe(1000000);
    expect(file.size).toBe(12);
    expect(file.type).toBe("text/plain");
    expect(await file.text()).toBe("test content");
  });
});

describe("Standalone node:buffer module functions and constants", () => {
  test("atob and btoa", () => {
    const b64 = btoa("hello world");
    expect(b64).toBe("aGVsbG8gd29ybGQ=");
    expect(atob("aGVsbG8gd29ybGQ=")).toBe("hello world");
  });

  test("isAscii, isLatin1, isUtf8", () => {
    const asciiBuf = Buffer.fromString("hello");
    expect(isAscii(asciiBuf.bytes())).toBe(true);

    expect(isLatin1("café")).toBe(true);
    expect(isLatin1("€")).toBe(false);

    expect(isUtf8(asciiBuf.bytes())).toBe(true);
  });

  test("transcode", () => {
    const utf8Buf = Buffer.fromString("hello");
    const asciiBuf = transcode(utf8Buf.bytes(), "utf8", "ascii");
    expect(asciiBuf.length).toBe(5);
  });

  test("constants", () => {
    const c = constants();
    expect(c.maxLength).toBe(2147483647);
    expect(c.inspectMaxBytes).toBe(50);
    expect(INSPECT_MAX_BYTES()).toBe(50);
    expect(kMaxLength()).toBe(2147483647);
    expect(kStringMaxLength()).toBe(536870888);
  });
});
