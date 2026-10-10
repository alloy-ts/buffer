import assert from "node:assert";
import test from "node:test";
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
  assert.strictEqual(buf1.length, 10);

  const buf2 = Buffer.alloc(10, 1);
  assert.strictEqual(buf2.length, 10);
  assert.strictEqual(buf2.readUInt8(0), 1);

  const buf3 = Buffer.allocUnsafe(10);
  assert.strictEqual(buf3.length, 10);
});

test("Buffer.from variations", () => {
  const buf4 = Buffer.from([1, 2, 3]);
  assert.strictEqual(buf4.length, 3);

  const buf5 = Buffer.from([257, 257.5, -255, 1]);
  assert.strictEqual(buf5.length, 4);

  const buf6 = Buffer.from("tést");
  assert.strictEqual(buf6.toString("utf8"), "tést");

  const buf7 = Buffer.from("tést", "latin1");
  assert.strictEqual(buf7.length, 4);
});

test("Buffer methods write, toString, slice, subarray, fill, swap", () => {
  const buf = Buffer.alloc(20);
  const written = buf.write("hello world", 0, "utf8");
  assert.strictEqual(written, 11);
  assert.strictEqual(buf.toString("utf8", 0, 11), "hello world");

  buf.fill(0xaa);
  assert.strictEqual(buf.readUInt8(0), 0xaa);

  const bufSwap = Buffer.from([0x01, 0x02, 0x03, 0x04]);
  bufSwap.swap16();
  assert.strictEqual(bufSwap.readUInt8(0), 0x02);
  assert.strictEqual(bufSwap.readUInt8(1), 0x01);
});

test("Buffer read and write numeric types", () => {
  const buf = Buffer.alloc(16);
  buf.writeUInt16BE(0x1234, 0);
  assert.strictEqual(buf.readUInt16BE(0), 0x1234);

  buf.writeUInt32LE(0x87654321, 2);
  assert.strictEqual(buf.readUInt32LE(2), 0x87654321);

  buf.writeBigUInt64BE(0x1234567890abcdefn, 6);
  assert.strictEqual(buf.readBigUInt64BE(6), 0x1234567890abcdefn);
});

test("Blob and File classes", () => {
  const blob = new Blob(["hello ", "world"], { type: "text/plain" });
  assert.strictEqual(blob.size, 11);
  assert.strictEqual(blob.type, "text/plain");
  assert.strictEqual(blob.text(), "hello world");

  const file = new File(["file content"], "test.txt", { type: "text/plain" });
  assert.strictEqual(file.name, "test.txt");
  assert.strictEqual(file.size, 12);
  assert.strictEqual(file.text(), "file content");
});

test("Utility functions: atob, btoa, isAscii, isUtf8, transcode", () => {
  const encoded = btoa("hello");
  assert.strictEqual(encoded, "aGVsbG8=");
  assert.strictEqual(atob("aGVsbG8="), "hello");

  assert.strictEqual(isAscii(Buffer.from("hello")), true);
  assert.strictEqual(isUtf8(Buffer.from("tést")), true);

  const transcoded = transcode(Buffer.from("€"), "utf8", "ascii");
  assert.ok(transcoded);

  assert.throws(() => resolveObjectURL("blob:nodedata:123"));
});

test("Module constants and variables", () => {
  assert.notStrictEqual(constants.MAX_LENGTH, undefined);
  assert.strictEqual(INSPECT_MAX_BYTES, 50);
  assert.notStrictEqual(kMaxLength, undefined);
  assert.notStrictEqual(kStringMaxLength, undefined);

  const slow = SlowBuffer(10);
  assert.strictEqual(slow.length, 10);
});
