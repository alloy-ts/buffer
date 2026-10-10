import { expect, test } from "vite-plus/test";
import * as native from "../index.js";
import {
  Buffer,
  Blob,
  File,
  atob,
  btoa,
  isAscii,
  isUtf8,
  isLatin1,
  transcode,
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
});

test("Buffer.from array and truncated values", () => {
  const buf1 = Buffer.from([1, 2, 3]);
  expect(Array.from(buf1)).toEqual([1, 2, 3]);

  const buf2 = Buffer.from([257, 257.5, -255, "1"]);
  expect(Array.from(buf2)).toEqual([1, 1, 1, 1]);
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
});

test("Buffer.from ArrayBuffer", () => {
  const ab = new ArrayBuffer(10);
  const buf = Buffer.from(ab, 0, 2);
  expect(buf.length).toBe(2);
  expect(buf.buffer).toBe(ab);
});

test("Buffer.isBuffer and Buffer.isEncoding", () => {
  expect(Buffer.isBuffer(Buffer.alloc(10))).toBe(true);
  expect(Buffer.isBuffer(Buffer.from("foo"))).toBe(true);
  expect(Buffer.isBuffer("a string")).toBe(false);
  expect(Buffer.isBuffer([])).toBe(false);
  expect(Buffer.isBuffer(new Uint8Array(1024))).toBe(false);

  expect(Buffer.isEncoding("utf8")).toBe(true);
  expect(Buffer.isEncoding("hex")).toBe(true);
  expect(Buffer.isEncoding("utf/8")).toBe(false);
  expect(Buffer.isEncoding("")).toBe(false);
});

test("Buffer.byteLength", () => {
  const str = "\u00bd + \u00bc = \u00be";
  expect(Buffer.byteLength(str, "utf8")).toBe(12);
  expect(Buffer.byteLength(Buffer.from(str))).toBe(12);
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
});

test("buf.write", () => {
  const buf = Buffer.alloc(256);
  const len = buf.write("\u00bd + \u00bc = \u00be", 0);
  expect(len).toBe(12);
  expect(buf.toString("utf8", 0, len)).toBe("\u00bd + \u00bc = \u00be");
});

test("buf.copy", () => {
  const buf1 = Buffer.allocUnsafe(26);
  const buf2 = Buffer.allocUnsafe(26).fill("!");

  for (let i = 0; i < 26; i++) {
    buf1[i] = i + 97;
  }

  buf1.copy(buf2, 8, 16, 20);
  expect(buf2.toString("ascii", 0, 25)).toBe("!!!!!!!!qrst!!!!!!!!!!!!!");
});

test("buf.fill", () => {
  const b = Buffer.allocUnsafe(50).fill("h");
  expect(b.toString()).toBe("hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh");

  const c = Buffer.allocUnsafe(5).fill("");
  expect(Array.from(c)).toEqual([0, 0, 0, 0, 0]);
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

test("buf.swap16, swap32, swap64", () => {
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

  buf.writeInt32LE(0x05060708, 6);
  expect(buf.readInt32LE(6)).toBe(0x05060708);

  buf.writeInt32BE(0x01020304, 10);
  expect(buf.readInt32BE(10)).toBe(0x01020304);

  const floatBuf = Buffer.alloc(8);
  floatBuf.writeFloatLE(123.45, 0);
  expect(Math.fround(floatBuf.readFloatLE(0))).toBe(Math.fround(123.45));

  floatBuf.writeDoubleBE(123.456789, 0);
  expect(floatBuf.readDoubleBE(0)).toBe(123.456789);

  const bigBuf = Buffer.alloc(8);
  bigBuf.writeBigInt64BE(0x0102030405060708n, 0);
  expect(bigBuf.readBigInt64BE(0)).toBe(0x0102030405060708n);

  bigBuf.writeBigUInt64LE(0xdecafafecacefaden, 0);
  expect(bigBuf.readBigUInt64LE(0)).toBe(0xdecafafecacefaden);

  const varBuf = Buffer.alloc(6);
  varBuf.writeIntLE(0x1234567890ab, 0, 6);
  expect(varBuf.readIntLE(0, 6)).toBe(0x1234567890ab);

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

test("isAscii and isUtf8", () => {
  expect(isAscii(Buffer.from("hello"))).toBe(true);
  expect(isUtf8(Buffer.from("tést"))).toBe(true);
});

test("Blob class", () => {
  const blob = new Blob(["hello ", "world"], { type: "text/plain" });
  expect(blob.size).toBe(11);
  expect(blob.type).toBe("text/plain");
  expect(blob.text()).toBe("hello world");

  const sliced = blob.slice(0, 5);
  expect(sliced.size).toBe(5);
  expect(sliced.text()).toBe("hello");
});

test("File class", () => {
  const file = new File(["hello world"], "hello.txt", { type: "text/plain" });
  expect(file.name).toBe("hello.txt");
  expect(file.size).toBe(11);
  expect(file.type).toBe("text/plain");
  expect(typeof file.lastModified).toBe("number");
  expect(file.text()).toBe("hello world");
});

test("transcode and constants", () => {
  const newBuf = transcode(Buffer.from("€"), "utf8", "ascii");
  expect(newBuf.toString("ascii")).toBe("?");

  expect(constants.MAX_LENGTH).toBe(kMaxLength);
  expect(constants.MAX_STRING_LENGTH).toBe(kStringMaxLength);
  expect(INSPECT_MAX_BYTES).toBe(50);
});

// ===========================================================================
// NAPI binding-level tests (exercising the native exports directly).
// ===========================================================================

test("native bufIsEncoding / Buffer.isEncoding agree", () => {
  expect(native.bufIsEncoding("utf8")).toBe(true);
  expect(native.bufIsEncoding("hex")).toBe(true);
  expect(native.bufIsEncoding("utf/8")).toBe(false);
  expect(native.bufIsEncoding("")).toBe(false);
  expect(Buffer.isEncoding("latin1")).toBe(true);
});

test("native bufByteLength on non-string inputs", () => {
  const u8 = new Uint8Array([1, 2, 3, 4]);
  expect(native.bufByteLength(u8)).toBe(4);
  const ab = new ArrayBuffer(8);
  expect(native.bufByteLength(ab)).toBe(8);
  // String still measures the encoded length.
  expect(native.bufByteLength("tést", "utf8")).toBe(5);
});

test("native bufConcat and bufCompare", () => {
  const a = native.bufDecode("ab");
  const b = native.bufDecode("cd");
  const c = native.bufConcat([a, b]);
  expect(Array.from(c)).toEqual([97, 98, 99, 100]);
  expect(native.bufCompare(native.bufDecode("abc"), native.bufDecode("abd"))).toBe(-1);
  expect(native.bufCompare(native.bufDecode("abd"), native.bufDecode("abc"))).toBe(1);
  expect(native.bufCompare(native.bufDecode("abc"), native.bufDecode("abc"))).toBe(0);
});

test("native bufDecode / bufEncode round-trip for lossless encodings", () => {
  const lossless = ["utf8", "utf-8", "latin1", "binary", "utf16le", "ucs2"];
  for (const enc of lossless) {
    const raw = "AéZ";
    const buf = native.bufDecode(raw, enc);
    const out = native.bufEncode(buf, enc);
    expect(out).toBe(raw);
  }
  // `ascii` substitutes non-ASCII code points with '?'.
  const asciiBuf = native.bufDecode("AéZ", "ascii");
  expect(native.bufEncode(asciiBuf, "ascii")).toBe("A?Z");
});

test("native bufFill with multi-byte truncation", () => {
  const buf = native.bufDecode("aaaaa");
  native.bufFill(buf, "€", 0, 5, "utf8");
  // A 3-byte char can only partially fit; the rest of the char is dropped.
  expect(Array.from(buf)).toEqual([0xe2, 0x82, 0xac, 0xe2, 0x82]);
});

test("native bufWrite respects the length argument", () => {
  const buf = native.bufDecode("          ");
  const n = native.bufWrite(buf, "abcdefgh", 0, 3, "utf8");
  expect(n).toBe(3);
  expect(native.bufEncode(buf, "utf8", 0, 3)).toBe("abc");
});

test("native bufIndexOf / bufLastIndexOf with explicit end and encoding", () => {
  const buf = native.bufDecode("this is a buffer");
  expect(native.bufIndexOf(buf, "is", 0, undefined, "utf8")).toBe(2);
  expect(native.bufIndexOf(buf, 97, undefined, undefined, undefined)).toBe(8);
  expect(native.bufLastIndexOf(buf, "buffer", undefined, undefined, "utf8")).toBe(10);
  // utf16le search
  const u16 = native.bufDecode("ΚΑΣΣΕ", "utf16le");
  expect(native.bufIndexOf(u16, "Σ", 0, undefined, "utf16le")).toBe(4);
});

test("native isAscii / isUtf8 accept ArrayBuffer and TypedArray", () => {
  expect(isAscii(new Uint8Array([104, 101, 108, 108, 111]))).toBe(true);
  expect(isAscii(new Uint8Array([0xff]))).toBe(false);
  expect(isUtf8(native.bufDecode("tést"))).toBe(true);
  expect(isUtf8(new Uint8Array([0xff, 0xfe]))).toBe(false);
});

test("native isLatin1 validates the WebIDL ByteString range", () => {
  expect(isLatin1("hello")).toBe(true);
  expect(isLatin1("héllo")).toBe(true); // 'é' is U+00E9, within U+0000..U+00FF
  expect(isLatin1("h€llo")).toBe(false); // '€' is U+20AC > 0xFF
});

test("native atob / btoa against the browser semantics", () => {
  expect(btoa("hello")).toBe("aGVsbG8=");
  expect(atob("aGVsbG8=")).toBe("hello");
});

test("native transcode between supported pairs", () => {
  // 'é' (U+00E9) fits in Latin-1, so utf8 -> latin1 is a clean round-trip.
  expect(transcode(native.bufDecode("café"), "utf8", "latin1").toString("latin1")).toBe("café");
  // latin1 -> utf8 is a clean expansion.
  expect(transcode(native.bufDecode("café", "latin1"), "latin1", "utf8").toString("utf8")).toBe(
    "café",
  );
});

test("native resolveObjectURL is unsupported and throws", () => {
  expect(() => native.resolveObjectURL("blob:nodedata:abc")).toThrow();
});

test("Buffer.from honours WithImplicitCoercion (valueOf / Symbol.toPrimitive)", () => {
  const withValueOf = {
    valueOf() {
      return "hello";
    },
  };
  expect(Buffer.from(withValueOf).toString()).toBe("hello");
  const withPrimitive = {
    [Symbol.toPrimitive]() {
      return "world";
    },
  };
  expect(Buffer.from(withPrimitive).toString()).toBe("world");
});

test("Buffer.copyBytesFrom copies the underlying memory of a TypedArray", () => {
  const u16 = new Uint16Array([0, 0xffff]);
  const buf = Buffer.copyBytesFrom(u16 as unknown as ArrayBufferView, 1, 1);
  u16[1] = 0;
  expect(buf.length).toBe(2);
  expect(buf[0]).toBe(255);
  expect(buf[1]).toBe(255);
});
