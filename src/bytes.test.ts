import { expect, test } from "vite-plus/test";
import { Bytes, Buffer } from "./main.ts";

test("Bytes constructor, len, and isEmpty", () => {
  const bytes = new Bytes();
  expect(bytes.len()).toBe(0);
  expect(bytes.isEmpty()).toBe(true);
  expect(typeof bytes.isUnique()).toBe("boolean");
});

test("Bytes.fromBuffer, asSlice, and buf", () => {
  const buf = Buffer.from([1, 2, 3, 4, 5]);
  const bytes = Bytes.fromBuffer(buf);
  expect(bytes.len()).toBe(5);
  expect(bytes.isEmpty()).toBe(false);

  const sliceBuf = bytes.asSlice();
  expect(sliceBuf instanceof Uint8Array).toBe(true);
  expect(Array.from(sliceBuf)).toEqual([1, 2, 3, 4, 5]);

  const bBuf = bytes.buf();
  expect(bBuf instanceof Uint8Array).toBe(true);
  expect(Array.from(bBuf)).toEqual([1, 2, 3, 4, 5]);
});

test("Bytes.slice, splitOff, and splitTo", () => {
  const buf = Buffer.from([10, 20, 30, 40, 50]);
  const bytes = Bytes.fromBuffer(buf);

  const sliced = bytes.slice(1, 4);
  expect(sliced.len()).toBe(3);
  expect(Array.from(sliced.asSlice())).toEqual([20, 30, 40]);

  // Out of bounds slice
  expect(() => bytes.slice(2, 10)).toThrow();

  const bytesForSplit = Bytes.fromBuffer(buf);
  const off = bytesForSplit.splitOff(2);
  expect(bytesForSplit.len()).toBe(2);
  expect(Array.from(bytesForSplit.asSlice())).toEqual([10, 20]);
  expect(off.len()).toBe(3);
  expect(Array.from(off.asSlice())).toEqual([30, 40, 50]);

  const bytesForSplitTo = Bytes.fromBuffer(buf);
  const to = bytesForSplitTo.splitTo(2);
  expect(to.len()).toBe(2);
  expect(Array.from(to.asSlice())).toEqual([10, 20]);
  expect(bytesForSplitTo.len()).toBe(3);
  expect(Array.from(bytesForSplitTo.asSlice())).toEqual([30, 40, 50]);
});

test("Bytes.truncate and clear", () => {
  const buf = Buffer.from([1, 2, 3, 4, 5]);
  const bytes = Bytes.fromBuffer(buf);

  bytes.truncate(3);
  expect(bytes.len()).toBe(3);
  expect(Array.from(bytes.asSlice())).toEqual([1, 2, 3]);

  bytes.clear();
  expect(bytes.len()).toBe(0);
  expect(bytes.isEmpty()).toBe(true);
});

test("Bytes.tryIntoMut", () => {
  const buf = Buffer.from([1, 2, 3]);
  const bytes = Bytes.fromBuffer(buf);

  const mutBytes = bytes.tryIntoMut();
  if (mutBytes) {
    expect(mutBytes.len()).toBe(3);
    expect(Array.from(mutBytes.asSlice())).toEqual([1, 2, 3]);
  }
});
