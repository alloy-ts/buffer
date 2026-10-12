import { expect, test } from "vite-plus/test";
import { Buffer, Bytes } from "./main.ts";

test("Bytes constructor, len, isEmpty, and isUnique", () => {
  const bytes = new Bytes();
  expect(bytes.len()).toBe(0);
  expect(bytes.isEmpty()).toBe(true);
  // Empty bytes (static empty buffer in Rust bytes library) is not unique.
  expect(typeof bytes.isUnique()).toBe("boolean");
});

test("Bytes.fromBuffer and asSlice / buf", () => {
  const buf = Buffer.from([1, 2, 3, 4, 5]);
  const bytes = Bytes.fromBuffer(buf as any);

  expect(bytes.len()).toBe(5);
  expect(bytes.isEmpty()).toBe(false);

  const copyBuf = bytes.asSlice();
  expect(Array.from(copyBuf)).toEqual([1, 2, 3, 4, 5]);

  const bufCopy = bytes.buf();
  expect(Array.from(bufCopy)).toEqual([1, 2, 3, 4, 5]);
});

test("Bytes slice", () => {
  const buf = Buffer.from([10, 20, 30, 40, 50]);
  const bytes = Bytes.fromBuffer(buf as any);

  const sub = bytes.slice(1, 4);
  expect(sub.len()).toBe(3);
  expect(Array.from(sub.asSlice())).toEqual([20, 30, 40]);

  const subToEnd = bytes.slice(2);
  expect(subToEnd.len()).toBe(3);
  expect(Array.from(subToEnd.asSlice())).toEqual([30, 40, 50]);

  expect(() => bytes.slice(3, 1)).toThrow();
  expect(() => bytes.slice(0, 10)).toThrow();
});

test("Bytes splitOff and splitTo", () => {
  const buf1 = Buffer.from([1, 2, 3, 4, 5]);
  const bytes1 = Bytes.fromBuffer(buf1 as any);

  const tail = bytes1.splitOff(2);
  expect(Array.from(bytes1.asSlice())).toEqual([1, 2]);
  expect(Array.from(tail.asSlice())).toEqual([3, 4, 5]);

  const buf2 = Buffer.from([10, 20, 30, 40, 50]);
  const bytes2 = Bytes.fromBuffer(buf2 as any);

  const head = bytes2.splitTo(2);
  expect(Array.from(head.asSlice())).toEqual([10, 20]);
  expect(Array.from(bytes2.asSlice())).toEqual([30, 40, 50]);

  expect(() => bytes1.splitOff(10)).toThrow();
  expect(() => bytes2.splitTo(10)).toThrow();
});

test("Bytes truncate and clear", () => {
  const buf = Buffer.from([1, 2, 3, 4, 5]);
  const bytes = Bytes.fromBuffer(buf as any);

  bytes.truncate(3);
  expect(bytes.len()).toBe(3);
  expect(Array.from(bytes.asSlice())).toEqual([1, 2, 3]);

  bytes.clear();
  expect(bytes.len()).toBe(0);
  expect(bytes.isEmpty()).toBe(true);
});

test("Bytes tryIntoMut", () => {
  const buf = Buffer.from([1, 2, 3]);
  const bytes = Bytes.fromBuffer(buf as any);

  const mutBytes = bytes.tryIntoMut();
  expect(mutBytes).not.toBeNull();
  expect(mutBytes?.len()).toBe(3);
  expect(Array.from(mutBytes!.asSlice())).toEqual([1, 2, 3]);
});
