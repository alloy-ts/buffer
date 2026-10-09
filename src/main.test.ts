import { expect, test, describe } from "vite-plus/test";
import {
  Buffer,
  Blob,
  File,
  atob,
  btoa,
  isAscii,
  isUtf8,
  resolveObjectURL,
  transcode,
  constants,
  INSPECT_MAX_BYTES,
  kMaxLength,
  kStringMaxLength,
  SlowBuffer,
} from "./main.ts";

describe("Buffer static methods and allocation", () => {
  test("Buffer.alloc and allocUnsafe", () => {
    const buf1 = Buffer.alloc(10);
    expect(buf1.length).toBe(10);
    expect(buf1[0]).toBe(0);

    const buf2 = Buffer.alloc(10, 1);
    expect(buf2[0]).toBe(1);
    expect(buf2[9]).toBe(1);

    const buf3 = Buffer.allocUnsafe(10);
    expect(buf3.length).toBe(10);

    const buf4 = Buffer.allocUnsafeSlow(10);
    expect(buf4.length).toBe(10);
  });

  test("Buffer.from", () => {
    const buf1 = Buffer.from([1, 2, 3]);
    expect(buf1.length).toBe(3);
    expect(buf1[0]).toBe(1);

    const buf2 = Buffer.from([257, 257.5, -255, "1" as any]);
    expect(buf2[0]).toBe(1);

    const buf3 = Buffer.from("tést");
    expect(buf3.toString()).toBe("tést");

    const buf4 = Buffer.from("tést", "latin1");
    expect(buf4.toString("latin1")).toBe("tést");
  });

  test("Buffer.concat and equals", () => {
    const buf1 = Buffer.from("hello ");
    const buf2 = Buffer.from("world");
    const concatenated = Buffer.concat([buf1, buf2]);
    expect(concatenated.toString()).toBe("hello world");

    const copy = Buffer.from("hello world");
    expect(concatenated.equals(copy)).toBe(true);
  });

  test("Buffer.isBuffer and isEncoding", () => {
    expect(Buffer.isBuffer(Buffer.alloc(5))).toBe(true);
    expect(Buffer.isBuffer(new Uint8Array(5))).toBe(false);
    expect(Buffer.isEncoding("utf8")).toBe(true);
    expect(Buffer.isEncoding("invalid-enc")).toBe(false);
  });

  test("Buffer.copyBytesFrom and of", () => {
    const u16 = new Uint16Array([0, 0xffff]);
    const buf = Buffer.copyBytesFrom(u16, 1, 1);
    expect(buf.length).toBe(2);

    const bufOf = Buffer.of(1, 2, 3);
    expect(bufOf.length).toBe(3);
    expect(bufOf[0]).toBe(1);
  });
});

describe("Buffer read/write methods and transformations", () => {
  test("read/write integers and floats", () => {
    const buf = Buffer.alloc(16);
    buf.writeUInt8(255, 0);
    expect(buf.readUInt8(0)).toBe(255);

    buf.writeInt16BE(0x1234, 1);
    expect(buf.readInt16BE(1)).toBe(0x1234);

    buf.writeFloatLE(1.234, 4);
    expect(Math.fround(buf.readFloatLE(4))).toBe(Math.fround(1.234));

    buf.writeBigInt64BE(1234567890123456789n, 8);
    expect(buf.readBigInt64BE(8)).toBe(1234567890123456789n);
  });

  test("swap16, swap32, swap64", () => {
    const buf1 = Buffer.from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
    buf1.swap16();
    expect(Array.from(buf1)).toEqual([0x2, 0x1, 0x4, 0x3, 0x6, 0x5, 0x8, 0x7]);

    const buf2 = Buffer.from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
    buf2.swap32();
    expect(Array.from(buf2)).toEqual([0x4, 0x3, 0x2, 0x1, 0x8, 0x7, 0x6, 0x5]);

    const buf3 = Buffer.from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
    buf3.swap64();
    expect(Array.from(buf3)).toEqual([0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]);
  });

  test("toJSON, indexOf, includes", () => {
    const buf = Buffer.from("this is a buffer");
    expect(buf.indexOf("is")).toBe(2);
    expect(buf.includes("buffer")).toBe(true);

    const json = buf.toJSON();
    expect(json.type).toBe("Buffer");
    expect(Array.isArray(json.data)).toBe(true);
  });
});

describe("Blob and File classes", () => {
  test("Blob creation and methods", async () => {
    const blob = new Blob(["hello world"], { type: "text/plain" });
    expect(blob.size).toBe(11);
    expect(blob.type).toBe("text/plain");

    const text = await blob.text();
    expect(text).toBe("hello world");

    const bytes = await blob.bytes();
    expect(bytes instanceof Uint8Array).toBe(true);
    expect(bytes.length).toBe(11);

    const ab = await blob.arrayBuffer();
    expect(ab.byteLength).toBe(11);

    const sliced = blob.slice(0, 5);
    expect(await sliced.text()).toBe("hello");
  });

  test("File creation and properties", async () => {
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

describe("Module utility functions and constants", () => {
  test("atob and btoa", () => {
    const encoded = btoa("hello");
    expect(encoded).toBe("aGVsbG8=");
    expect(atob(encoded)).toBe("hello");
  });

  test("isAscii and isUtf8", () => {
    const asciiBuf = Buffer.from("abc");
    expect(isAscii(asciiBuf)).toBe(true);

    const utf8Buf = Buffer.from("tést");
    expect(isUtf8(utf8Buf)).toBe(true);
  });

  test("transcode", () => {
    const transcoded = transcode(Buffer.from("€"), "utf8", "ascii");
    expect(transcoded.toString("ascii")).toBe("?");
  });

  test("constants and variables", () => {
    expect(typeof constants.MAX_LENGTH).toBe("number");
    expect(typeof constants.MAX_STRING_LENGTH).toBe("number");
    expect(typeof INSPECT_MAX_BYTES).toBe("number");
    expect(typeof kMaxLength).toBe("number");
    expect(typeof kStringMaxLength).toBe("number");
    expect(typeof SlowBuffer).toBe("function");
    expect(resolveObjectURL("blob:nodedata:test")).toBeUndefined();
  });
});
