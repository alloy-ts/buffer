import assert from "node:assert";
import test from "node:test";
import { Bytes } from "../dist/index.js";

test("Bytes constructor, len, is_empty", () => {
  const bytes = new Bytes();
  assert.strictEqual(bytes.len(), 0);
  assert.strictEqual(bytes.isEmpty(), true);
});

test("Bytes fromBuffer, asSlice, buf", () => {
  const nodeBuf = Buffer.from([1, 2, 3, 4, 5]);
  const bytes = Bytes.fromBuffer(nodeBuf);
  assert.strictEqual(bytes.len(), 5);
  assert.strictEqual(bytes.isEmpty(), false);

  const slice = bytes.asSlice();
  assert.strictEqual(slice.length, 5);
  assert.strictEqual(slice[0], 1);

  const customBuf = bytes.buf();
  assert.strictEqual(customBuf.length, 5);
  assert.strictEqual(customBuf.readUInt8(0), 1);
});

test("Bytes slice and split", () => {
  const nodeBuf = Buffer.from("hello world");
  const bytes = Bytes.fromBuffer(nodeBuf);

  const sub = bytes.slice(0, 5);
  assert.strictEqual(sub.len(), 5);
  assert.strictEqual(sub.asSlice().toString(), "hello");

  const splitOff = bytes.splitOff(5);
  assert.strictEqual(bytes.len(), 5);
  assert.strictEqual(splitOff.len(), 6);
  assert.strictEqual(bytes.asSlice().toString(), "hello");
  assert.strictEqual(splitOff.asSlice().toString(), " world");
});

test("Bytes truncate and clear", () => {
  const bytes = Bytes.fromBuffer(Buffer.from("testing"));
  bytes.truncate(4);
  assert.strictEqual(bytes.len(), 4);
  assert.strictEqual(bytes.asSlice().toString(), "test");

  bytes.clear();
  assert.strictEqual(bytes.len(), 0);
  assert.strictEqual(bytes.isEmpty(), true);
});
