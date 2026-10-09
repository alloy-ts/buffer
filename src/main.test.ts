import { describe, expect, test } from "vite-plus/test";
import {
  Blob,
  Buffer,
  File,
  INSPECT_MAX_BYTES,
  SlowBuffer,
  atob,
  btoa,
  constants,
  isAscii,
  isUtf8,
  kMaxLength,
  kStringMaxLength,
  resolveObjectURL,
  transcode,
} from "./main.ts";

describe("Buffer class and static methods", () => {
  test("Buffer.alloc", () => {
    const buf1 = Buffer.alloc(10);
    expect(buf1.length).toBe(10);
    expect([...buf1]).toEqual([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    const buf2 = Buffer.alloc(5, 1);
    expect([...buf2]).toEqual([1, 1, 1, 1, 1]);

    const buf3 = Buffer.alloc(11, "aGVsbG8gd29ybGQ=", "base64");
    expect(buf3.toString()).toBe("hello world");
  });

  test("Buffer.allocUnsafe & allocUnsafeSlow", () => {
    const buf1 = Buffer.allocUnsafe(10);
    expect(buf1.length).toBe(10);

    const buf2 = Buffer.allocUnsafeSlow(10);
    expect(buf2.length).toBe(10);
  });

  test("Buffer.from overloads", () => {
    const buf1 = Buffer.from([1, 2, 3]);
    expect([...buf1]).toEqual([1, 2, 3]);

    const buf2 = Buffer.from([257, 257.5, -255, 1]);
    expect([...buf2]).toEqual([1, 1, 1, 1]);

    const buf3 = Buffer.from("tést");
    expect([...buf3]).toEqual([116, 195, 169, 115, 116]);

    const buf4 = Buffer.from("tést", "latin1");
    expect([...buf4]).toEqual([0x74, 0xe9, 0x73, 0x74]);

    const ab = new ArrayBuffer(8);
    const buf5 = Buffer.from(ab, 2, 4);
    expect(buf5.length).toBe(4);
  });

  test("Buffer.concat, isBuffer, isEncoding, byteLength, compare", () => {
    const b1 = Buffer.from([1, 2]);
    const b2 = Buffer.from([3, 4]);
    const combined = Buffer.concat([b1, b2]);
    expect([...combined]).toEqual([1, 2, 3, 4]);

    expect(Buffer.isBuffer(b1)).toBe(true);
    expect(Buffer.isBuffer([1, 2])).toBe(false);

    expect(Buffer.isEncoding("utf8")).toBe(true);
    expect(Buffer.isEncoding("hex")).toBe(true);
    expect(Buffer.isEncoding("invalid-enc")).toBe(false);

    expect(Buffer.byteLength("tést")).toBe(5);
    expect(Buffer.byteLength("tést", "latin1")).toBe(4);

    expect(Buffer.compare(Buffer.from("A"), Buffer.from("B"))).toBe(-1);
  });

  test("Buffer.copyBytesFrom & Buffer.of", () => {
    const u16 = new Uint16Array([0, 0xffff]);
    const buf = Buffer.copyBytesFrom(u16, 1, 1);
    expect(buf.length).toBe(2);

    const bufOf = Buffer.of(1, 2, 3);
    expect([...bufOf]).toEqual([1, 2, 3]);
  });
});

describe("Buffer instance methods", () => {
  test("read and write integers and floats", () => {
    const buf = Buffer.alloc(16);

    buf.writeUInt8(255, 0);
    expect(buf.readUInt8(0)).toBe(255);
    expect(buf.readUint8(0)).toBe(255);

    buf.writeInt8(-128, 1);
    expect(buf.readInt8(1)).toBe(-128);

    buf.writeUInt16LE(0xabcd, 2);
    expect(buf.readUInt16LE(2)).toBe(0xabcd);
    expect(buf.readUint16LE(2)).toBe(0xabcd);

    buf.writeUInt16BE(0xabcd, 4);
    expect(buf.readUInt16BE(4)).toBe(0xabcd);

    buf.writeUInt32LE(0xdeadbeef, 6);
    expect(buf.readUInt32LE(6)).toBe(0xdeadbeef);

    buf.writeFloatLE(1.5, 10);
    expect(buf.readFloatLE(10)).toBe(1.5);
  });

  test("read and write BigInts", () => {
    const buf = Buffer.alloc(16);

    buf.writeBigUInt64BE(0xdecafafecacefaden, 0);
    expect(buf.readBigUInt64BE(0)).toBe(0xdecafafecacefaden);
    expect(buf.readBigUint64BE(0)).toBe(0xdecafafecacefaden);

    buf.writeBigInt64LE(-123456789012345678n, 8);
    expect(buf.readBigInt64LE(8)).toBe(-123456789012345678n);
  });

  test("read and write LE/BE 48-bit integers", () => {
    const buf = Buffer.alloc(6);
    buf.writeUIntLE(0x1234567890ab, 0, 6);
    expect(buf.readUIntLE(0, 6)).toBe(0x1234567890ab);

    buf.writeUIntBE(0x1234567890ab, 0, 6);
    expect(buf.readUIntBE(0, 6)).toBe(0x1234567890ab);
  });

  test("write, toString, toJSON, fill, equals, compare, copy, indexOf, includes, reverse, swap", () => {
    const buf = Buffer.alloc(10);
    const len = buf.write("hello", 0);
    expect(len).toBe(5);
    expect(buf.toString("utf8", 0, 5)).toBe("hello");

    const json = buf.toJSON();
    expect(json.type).toBe("Buffer");
    expect(json.data.slice(0, 5)).toEqual([104, 101, 108, 108, 111]);

    const buf2 = Buffer.from("hello\0\0\0\0\0");
    expect(buf.equals(buf2)).toBe(true);

    expect(buf.indexOf("e")).toBe(1);
    expect(buf.lastIndexOf("l")).toBe(3);
    expect(buf.includes("ll")).toBe(true);

    const b = Buffer.from([1, 2, 3, 4]);
    b.reverse();
    expect([...b]).toEqual([4, 3, 2, 1]);

    const swapBuf = Buffer.from([0x01, 0x02, 0x03, 0x04]);
    swapBuf.swap16();
    expect([...swapBuf]).toEqual([0x02, 0x01, 0x04, 0x03]);

    const sub = buf.subarray(0, 5);
    expect(sub.toString()).toBe("hello");
  });
});

describe("Blob and File", () => {
  test("Blob operations", async () => {
    const blob = new Blob(["hello ", "world"], { type: "text/plain" });
    expect(blob.size).toBe(11);
    expect(blob.type).toBe("text/plain");

    expect(await blob.text()).toBe("hello world");

    const bytes = await blob.bytes();
    expect(bytes).toBeInstanceOf(Uint8Array);
    expect(bytes.length).toBe(11);

    const ab = await blob.arrayBuffer();
    expect(ab.byteLength).toBe(11);

    const sliced = blob.slice(0, 5);
    expect(await sliced.text()).toBe("hello");
  });

  test("File operations", async () => {
    const file = new File(["file content"], "test.txt", {
      type: "text/plain",
      lastModified: 1000,
    });
    expect(file.name).toBe("test.txt");
    expect(file.lastModified).toBe(1000);
    expect(file.size).toBe(12);
    expect(await file.text()).toBe("file content");
  });
});

describe("Utility functions & Constants", () => {
  test("atob and btoa", () => {
    const encoded = btoa("hello");
    expect(encoded).toBe("aGVsbG8=");
    expect(atob(encoded)).toBe("hello");
  });

  test("isAscii & isUtf8", () => {
    expect(isAscii(Buffer.from("abc"))).toBe(true);
    expect(isUtf8(Buffer.from("tést"))).toBe(true);
  });

  test("transcode", () => {
    const buf = Buffer.from("€", "utf8");
    const transcoded = transcode(buf, "utf8", "ascii");
    expect(transcoded.toString("ascii")).toBe("?");
  });

  test("constants", () => {
    expect(typeof constants.MAX_LENGTH).toBe("number");
    expect(typeof constants.MAX_STRING_LENGTH).toBe("number");
    expect(typeof INSPECT_MAX_BYTES).toBe("number");
    expect(typeof kMaxLength).toBe("number");
    expect(typeof kStringMaxLength).toBe("number");
    expect(SlowBuffer).toBeDefined();
    expect(typeof resolveObjectURL).toBe("function");
  });
});
