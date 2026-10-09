import { expect, test, describe } from "vite-plus/test";
import {
  Buffer,
  Blob,
  File,
  atob,
  btoa,
  isAscii,
  isLatin1,
  isUtf8,
  transcode,
  constants,
  inspectMaxBytes,
  kMaxLength,
  kStringMaxLength,
  resolveObjectUrl,
} from "./main.ts";

describe("Buffer static methods and properties", () => {
  test("Buffer.alloc and length", () => {
    const buf = Buffer.alloc(10);
    expect(buf.length).toBe(10);
    expect(buf.byteOffset).toBe(0);
    expect(buf.readUInt8(0)).toBe(0);
  });

  test("Buffer.alloc with fill", () => {
    const buf = Buffer.alloc(5, 0x61);
    expect(buf.length).toBe(5);
    expect(buf.readUInt8(0)).toBe(0x61);
    expect(buf.toString("utf8")).toBe("aaaaa");
  });

  test("Buffer.allocUnsafe & allocUnsafeSlow", () => {
    const buf1 = Buffer.allocUnsafe(10);
    expect(buf1.length).toBe(10);
    const buf2 = Buffer.allocUnsafeSlow(10);
    expect(buf2.length).toBe(10);
  });

  test("Buffer.fromString and toString with encodings", () => {
    const bufUtf8 = Buffer.fromString("hello world", "utf8");
    expect(bufUtf8.toString("hex")).toBe("68656c6c6f20776f726c64");
    expect(bufUtf8.toString("base64")).toBe("aGVsbG8gd29ybGQ=");

    const bufHex = Buffer.fromString("1634", "hex");
    expect(bufHex.length).toBe(2);
    expect(bufHex.readUInt8(0)).toBe(0x16);
    expect(bufHex.readUInt8(1)).toBe(0x34);

    const bufBase64 = Buffer.fromString("aGVsbG8=", "base64");
    expect(bufBase64.toString("utf8")).toBe("hello");
  });

  test("Buffer.fromArray", () => {
    const buf = Buffer.fromArray([1, 2, 3, 257]);
    expect(buf.length).toBe(4);
    expect(buf.readUInt8(0)).toBe(1);
    expect(buf.readUInt8(3)).toBe(1);
  });

  test("Buffer.concat", () => {
    const b1 = Buffer.fromString("foo");
    const b2 = Buffer.fromString("bar");
    const concat = Buffer.concat([b1.bytes(), b2.bytes()]);
    expect(concat.toString("utf8")).toBe("foobar");
  });

  test("Buffer.isBuffer & Buffer.isEncoding", () => {
    expect(Buffer.isEncoding("utf8")).toBe(true);
    expect(Buffer.isEncoding("hex")).toBe(true);
    expect(Buffer.isEncoding("invalid")).toBe(false);
  });

  test("Buffer.byteLength and Buffer.stringLength", () => {
    expect(Buffer.byteLength("½ + ¼ = ¾", "utf8")).toBe(12);
    expect(Buffer.stringLength("€ 100", "utf8")).toBe(5);
  });

  test("Buffer.copyBytesFrom and compareStatic", () => {
    const src = Buffer.fromString("hello");
    const copy = Buffer.copyBytesFrom(src.bytes(), 0, 5);
    expect(copy.toString("utf8")).toBe("hello");

    const b1 = Buffer.fromString("ABC");
    const b2 = Buffer.fromString("BCD");
    expect(Buffer.compareStatic(b1.bytes(), b2.bytes())).toBe(-1);
  });
});

describe("Buffer instance reads and writes", () => {
  test("integer reads and writes", () => {
    const buf = Buffer.alloc(8);
    buf.writeUInt8(0x12, 0);
    expect(buf.readUInt8(0)).toBe(0x12);
    expect(buf.readUint8(0)).toBe(0x12);

    buf.writeInt8(-5, 1);
    expect(buf.readInt8(1)).toBe(-5);

    buf.writeUInt16LE(0x1234, 2);
    expect(buf.readUInt16LE(2)).toBe(0x1234);

    buf.writeUInt32BE(0x12345678, 4);
    expect(buf.readUInt32BE(4)).toBe(0x12345678);
  });

  test("64-bit BigInt reads and writes", () => {
    const buf = Buffer.alloc(8);
    const val = 0x0102030405060708n;
    buf.writeBigUInt64BE(val, 0);
    expect(buf.readBigUInt64BE(0)).toBe(val);

    buf.writeBigInt64LE(-123456789101112n, 0);
    expect(buf.readBigInt64LE(0)).toBe(-123456789101112n);
  });

  test("variable byte length integer reads and writes", () => {
    const buf = Buffer.alloc(6);
    buf.writeUIntBE(0x1234567890abn, 0, 6);
    expect(buf.readUIntBE(0, 6)).toBe(0x1234567890abn);
  });

  test("floats and doubles", () => {
    const buf = Buffer.alloc(8);
    buf.writeFloatLE(1.25, 0);
    expect(buf.readFloatLE(0)).toBe(1.25);

    buf.writeDoubleBE(123.456, 0);
    expect(buf.readDoubleBE(0)).toBeCloseTo(123.456);
  });

  test("string write & toString", () => {
    const buf = Buffer.alloc(20);
    const written = buf.write("hello", 0, 5, "utf8");
    expect(written).toBe(5);
    expect(buf.toString("utf8", 0, 5)).toBe("hello");
  });

  test("fill, equals, compare, copy", () => {
    const b1 = Buffer.alloc(5);
    b1.fillNumber(0x41);
    expect(b1.toString("utf8")).toBe("AAAAA");

    const b2 = Buffer.alloc(5);
    b2.fillString("A");
    expect(b1.equals(b2.bytes())).toBe(true);

    const target = Buffer.alloc(5);
    b1.copy(target, 0, 0, 5);
    expect(target.toString("utf8")).toBe("AAAAA");
  });

  test("indexOf, lastIndexOf, includes, reverse, swap", () => {
    const buf = Buffer.fromString("this buffer is a buffer");
    expect(buf.indexOfString("buffer")).toBe(5);
    expect(buf.lastIndexOfString("buffer")).toBe(17);
    expect(buf.includesString("is")).toBe(true);

    const swapBuf = Buffer.fromArray([0x1, 0x2, 0x3, 0x4]);
    swapBuf.swap16();
    expect(swapBuf.readUInt8(0)).toBe(0x2);
    expect(swapBuf.readUInt8(1)).toBe(0x1);
  });

  test("keys, values, entries, toJson", () => {
    const buf = Buffer.fromArray([10, 20]);
    expect(buf.keys()).toEqual([0, 1]);
    expect(buf.values()).toEqual([10, 20]);
    expect(buf.entries()).toEqual([[0, 10], [1, 20]]);
    expect(buf.toJson()).toEqual({ type: "Buffer", data: [10, 20] });
  });
});

describe("Blob & File classes", () => {
  test("Blob basics", () => {
    const blob = new Blob(["hello ", "world"], { type: "text/plain" });
    expect(blob.size).toBe(11);
    expect(blob.type).toBe("text/plain");
    expect(blob.text()).toBe("hello world");

    const sliced = blob.slice(0, 5);
    expect(sliced.size).toBe(5);
    expect(sliced.text()).toBe("hello");
  });

  test("Blob with buffer sources", () => {
    const buf1 = Buffer.fromString("hello ");
    const buf2 = Buffer.fromString("world");
    const blob = new Blob([buf1.bytes(), buf2.bytes()], { type: "text/plain" });
    expect(blob.size).toBe(11);
    expect(blob.text()).toBe("hello world");
  });

  test("File basics", () => {
    const file = new File(["file content"], "test.txt", { type: "text/plain" });
    expect(file.name).toBe("test.txt");
    expect(file.size).toBe(12);
    expect(file.type).toBe("text/plain");
    expect(file.text()).toBe("file content");
    expect(typeof file.lastModified).toBe("bigint");
  });
});

describe("node:buffer module utilities", () => {
  test("atob and btoa", () => {
    const encoded = btoa("hello");
    expect(encoded).toBe("aGVsbG8=");
    expect(atob(encoded)).toBe("hello");
  });

  test("isAscii, isLatin1, isUtf8", () => {
    expect(isLatin1("hello")).toBe(true);
    expect(isLatin1("café")).toBe(true);
    expect(isLatin1("€")).toBe(false);

    const asciiBuf = Buffer.fromString("hello");
    expect(isAscii(asciiBuf.bytes())).toBe(true);
    expect(isUtf8(asciiBuf.bytes())).toBe(true);
  });

  test("transcode", () => {
    const utf8Buf = Buffer.fromString("hello");
    const transcoded = transcode(utf8Buf.bytes(), "utf8", "ascii");
    expect(transcoded.length).toBe(5);
  });

  test("constants", () => {
    const consts = constants();
    expect(consts.inspectMaxBytes).toBe(50);
    expect(consts.kStringMaxLength).toBe(536870888);
    expect(inspectMaxBytes()).toBe(50);
    expect(kStringMaxLength()).toBe(536870888);
    expect(typeof kMaxLength()).toBe("bigint");
  });
});
